#![allow(clippy::match_like_matches_macro)]

use std::collections::VecDeque;
use std::ops::ControlFlow::{Break, Continue};

use ErrorType::NoDocStartAfterTag;
use SpanToken::{DocumentStart, Separator, Space};

use crate::tokenizer::reader::{
    is_flow_indicator, is_indicator, is_white_tab, is_white_tab_or_break, ns_plain_safe, Reader,
};
use crate::tokenizer::spanner::ParserState::{
    BlockSeq, FlowKey, FlowMap, FlowSeq, PreDocStart, RootBlock,
};
use crate::tokenizer::spanner::SpanToken::{
    Directive, ErrorToken, KeyEnd, MappingEnd, MappingStart, MarkEnd, MarkStart, NewLine,
    SequenceEnd, SequenceStart,
};
use crate::tokenizer::ErrorType::ExpectedIndent;
use crate::tokenizer::ErrorType::UnexpectedSymbol;
use crate::tokenizer::{DirectiveType, ErrorType};

use super::reader::is_newline;

pub enum ScalarControl {
    SameIndent,
    Continue,
    GreaterIndent,
}

#[derive(Clone)]
pub struct Spanner {
    pub(crate) curr_state: ParserState,
    pub stream_end: bool,
    tokens: VecDeque<SpanToken>,
    stack: VecDeque<ParserState>,
}

impl Default for Spanner {
    fn default() -> Self {
        Self {
            stream_end: false,
            tokens: VecDeque::new(),
            curr_state: PreDocStart,
            stack: VecDeque::new(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub(crate) enum ParserState {
    PreDocStart,
    RootBlock,
    FlowSeq(usize),
    FlowMap(usize),
    FlowKey(usize, bool),
    BlockSeq(usize),
    AfterDocEnd,
}

impl ParserState {
    pub(crate) fn indent(&self) -> usize {
        match self {
            FlowKey(ind, _) | FlowMap(ind) | FlowSeq(ind) | BlockSeq(ind) => *ind,
            _ => 0,
        }
    }

    #[inline]
    pub fn in_flow_collection(&self) -> bool {
        match &self {
            FlowKey(_, _) | FlowSeq(_) | FlowMap(_) => true,
            _ => false,
        }
    }

    #[inline]
    pub(crate) fn is_implicit(&self) -> bool {
        match &self {
            FlowKey(_, true) => true,
            _ => false,
        }
    }
}

enum ChompIndicator {
    /// `-` final line break and any trailing empty lines are excluded from the scalar’s content
    Strip,
    ///  `` final line break character is preserved in the scalar’s content
    Clip,
    /// `+` final line break and any trailing empty lines are considered to be part of the scalar’s content
    Keep,
}

impl Spanner {
    #[inline]
    pub fn peek_token(&self) -> Option<SpanToken> {
        self.tokens.front().copied()
    }

    #[inline(always)]
    pub fn pop_token(&mut self) -> Option<SpanToken> {
        self.tokens.pop_front()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn fetch_next_token<R: Reader>(&mut self, reader: &mut R) {
        self.skip_separation_spaces(reader, true);
        match self.curr_state {
            PreDocStart => {
                if !reader.peek_byte_is(b'%') {
                    self.curr_state = RootBlock;
                    return;
                }

                if reader.try_read_slice_exact("%YAML") {
                    reader.skip_space_tab(true);
                    if let Some(x) = reader.find_next_whitespace() {
                        self.tokens.push_back(Directive(DirectiveType::Yaml));
                        self.tokens.push_back(MarkStart(reader.pos()));
                        self.tokens.push_back(MarkEnd(reader.pos() + x));

                        reader.consume_bytes(x);
                        reader.read_line();
                    }
                } else {
                    let tag = if reader.try_read_slice_exact("%TAG") {
                        Directive(DirectiveType::Tag)
                    } else {
                        Directive(DirectiveType::Reserved)
                    };
                    reader.skip_space_tab(true);
                    let x = reader.read_non_comment_line();
                    if x.0 != x.1 {
                        self.tokens.push_back(tag);
                        self.tokens.push_back(MarkStart(x.0));
                        self.tokens.push_back(MarkEnd(x.1));
                    }
                }
                if reader.try_read_slice_exact("---") {
                    self.tokens.push_back(DocumentStart)
                } else {
                    self.tokens.push_back(ErrorToken(NoDocStartAfterTag))
                }
            }
            RootBlock => match reader.peek_byte() {
                Some(b'{') => self.fetch_flow_col(reader, 0),
                Some(b'[') => self.fetch_flow_col(reader, 0),
                Some(b'&') => self.fetch_alias(reader),
                Some(b'*') => self.fetch_anchor(reader),
                Some(b':') => self.fetch_block_map(reader),
                Some(b'-') => self.fetch_block_seq(reader, 0),
                Some(b'?') => self.fetch_block_map_key(reader),
                Some(b'!') => self.fetch_tag(reader),
                Some(b'|') => self.fetch_block_scalar(reader, true),
                Some(b'>') => self.fetch_block_scalar(reader, false),
                Some(b'\'') => self.fetch_quoted_scalar(reader, b'\''),
                Some(b'"') => self.fetch_quoted_scalar(reader, b'"'),
                Some(b'#') => {
                    // comment
                    reader.read_line();
                }
                Some(x) => {
                    if x != b']' && x != b'}' && x != b'@' {
                        self.fetch_plain_scalar(reader);
                    } else {
                        reader.consume_bytes(1);
                        self.tokens
                            .push_back(ErrorToken(UnexpectedSymbol(x as char)))
                    }
                }
                None => self.stream_end = true,
            },
            BlockSeq(indent) => match reader.peek_byte() {
                Some(b'-') => self.fetch_block_seq(reader, indent + 1),
                Some(b'|') => self.fetch_block_scalar(reader, true),
                Some(b'>') => self.fetch_block_scalar(reader, false),
                Some(_) => {
                    self.fetch_plain_scalar(reader);
                }
                _ => todo!(),
            },
            FlowSeq(indent) => match reader.peek_byte() {
                Some(b'[') => self.fetch_flow_col(reader, indent + 1),
                Some(b'{') => self.fetch_flow_col(reader, indent + 1),
                Some(b']') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(SequenceEnd);
                    self.pop_state();
                }
                Some(b'}') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(ErrorToken(UnexpectedSymbol('}')));
                }
                Some(b',') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(Separator);
                }
                Some(b'\'') => self.fetch_quoted_scalar(reader, b'\''),
                Some(b'"') => self.fetch_quoted_scalar(reader, b'"'),
                Some(b':') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(MappingStart);
                    self.push_state(FlowKey(indent, true));
                }
                Some(b'?') => self.fetch_explicit_map(reader),
                Some(b'#') => {
                    // comment
                    reader.read_line();
                }
                Some(_) => {
                    self.fetch_plain_scalar(reader);
                }
                None => self.stream_end = true,
            },
            FlowMap(indent) | FlowKey(indent, _) => match reader.peek_byte() {
                Some(b'[') => self.fetch_flow_col(reader, indent + 1),
                Some(b'{') => self.fetch_flow_col(reader, indent + 1),
                Some(b'}') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(MappingEnd);
                    self.pop_state();
                }
                Some(b':') => self.process_map_key(reader, indent),
                Some(b']') => {
                    if self.is_prev_sequence() {
                        self.tokens.push_back(MappingEnd);
                        self.pop_state();
                    } else {
                        reader.consume_bytes(1);
                        self.tokens.push_back(ErrorToken(UnexpectedSymbol(']')));
                    }
                }
                Some(b'?') => self.fetch_explicit_map(reader),
                Some(b',') => {
                    reader.consume_bytes(1);
                }
                Some(b'\'') => self.fetch_quoted_scalar(reader, b'\''),
                Some(b'"') => self.fetch_quoted_scalar(reader, b'"'),
                Some(b'#') => {
                    // comment
                    reader.read_line();
                }
                Some(_) => {
                    self.fetch_plain_scalar(reader);
                }
                None => self.stream_end = true,
            },
            _ => {}
        }

        if reader.eof() {
            self.stream_end = true;
            self.stack.push_back(self.curr_state);
            for state in self.stack.iter().rev() {
                let x = match *state {
                    BlockSeq(_) => SequenceEnd,
                    _ => continue,
                };
                self.tokens.push_back(x);
            }
        }
    }

    fn fetch_flow_col<R: Reader>(&mut self, reader: &mut R, indent: usize) {
        let peek = reader.peek_byte_unwrap(0);
        reader.consume_bytes(1);

        if reader.col() != 0 {
            reader.skip_space_tab(true);
        }

        if peek == b'[' {
            self.tokens.push_back(SequenceStart);
            self.push_state(FlowSeq(indent));
        } else if peek == b'{' {
            if reader.col() != 0 {
                reader.skip_space_tab(true);
            }
            if reader.peek_byte_is(b'?') {
                self.push_state(FlowKey(indent, false));
            } else {
                self.push_state(FlowKey(indent, true));
            }
            self.tokens.push_back(MappingStart);
        }
    }

    #[inline]
    fn push_state(&mut self, state: ParserState) {
        self.stack.push_back(self.curr_state);
        self.curr_state = state;
    }

    #[inline]
    fn pop_state(&mut self) {
        match self.stack.pop_back() {
            Some(x) => self.curr_state = x,
            None => self.curr_state = ParserState::AfterDocEnd,
        }
    }

    fn fetch_alias<R: Reader>(&mut self, _reader: &mut R) {
        todo!()
    }

    fn fetch_anchor<R: Reader>(&mut self, _reader: &mut R) {
        todo!()
    }

    fn fetch_block_map<R: Reader>(&mut self, _reader: &mut R) {
        todo!()
    }

    fn fetch_block_seq<R: Reader>(&mut self, reader: &mut R, indent: usize) {
        if reader.peek_byte_at_check(1, is_white_tab_or_break) {
            let new_indent: usize = reader.col();
            if reader.peek_byte_at_check(1, is_newline) {
                reader.consume_bytes(1);
                reader.read_break();
            } else {
                reader.consume_bytes(2);
            }

            if new_indent >= indent {
                self.tokens.push_back(SequenceStart);
                self.push_state(BlockSeq(new_indent));
            }
        } else {
            self.fetch_plain_scalar(reader);
        }
    }

    fn fetch_block_map_key<R: Reader>(&mut self, _reader: &mut R) {
        todo!()
    }

    fn fetch_tag<R: Reader>(&mut self, _reader: &mut R) {
        todo!()
    }

    fn fetch_block_scalar<R: Reader>(&mut self, reader: &mut R, literal: bool) {
        reader.consume_bytes(1);
        let x0 = reader.peek_byte_unwrap(0);
        let x1 = reader.peek_byte_unwrap(1);
        let mut chomp = ChompIndicator::Clip;
        let mut indentation: usize = 0;

        match (x0, x1) {
            (b'-', len) | (len, b'-') if matches!(len, b'1'..=b'9') => {
                reader.consume_bytes(2);
                chomp = ChompIndicator::Strip;
                indentation = (len - b'0') as usize;
            }
            (b'+', len) | (len, b'+') if matches!(len, b'1'..=b'9') => {
                reader.consume_bytes(2);
                chomp = ChompIndicator::Keep;
                indentation = (len - b'0') as usize;
            }
            (b'-', _) => {
                reader.consume_bytes(1);
                chomp = ChompIndicator::Strip;
            }
            (b'+', _) => {
                reader.consume_bytes(1);
                chomp = ChompIndicator::Keep;
            }
            (len, _) if matches!(len, b'1'..=b'9') => {
                reader.consume_bytes(1);
                indentation = (len - b'0') as usize;
            }
            _ => {}
        }

        // allow comment in first line of block scalar
        reader.skip_space_tab(true);
        if reader.peek_byte_is(b'#') {
            reader.read_line();
        } else if reader.read_break().is_none() {
            self.tokens
                .push_back(ErrorToken(ErrorType::ExpectedNewline));
            return;
        }

        let mut new_line_token = 0;
        let mut is_new_seq_entry = false;
        while !reader.eof() {
            // if we encounter a character on a newline at current indent that isn't a whitespace/newline
            // we bail
            match self.finish_scalar(reader, self.curr_state.indent()) {
                Ok(ScalarControl::SameIndent) => {
                    is_new_seq_entry = true;
                    break;
                }
                _ => {}
            };
            // count indents important for folded scalars
            let newline_indent = reader.count_space_tab(false);
            let newline_is_empty = reader.peek_byte_at_check(newline_indent, is_newline);

            if indentation == 0 && newline_indent > 0 {
                if newline_is_empty {
                    new_line_token += 1;
                    reader.read_line();
                    continue;
                } else {
                    // We don't accept indent until it is followed by a non-space char
                    indentation = newline_indent;
                }
            }

            if let Err(x) = reader.skip_n_spaces(indentation) {
                self.tokens.push_back(ErrorToken(x));
                break;
            }

            let (start, end) = reader.read_line();
            if start != end {
                if new_line_token > 0 {
                    let token = if new_line_token == 1 && !literal {
                        Space
                    } else {
                        NewLine(new_line_token)
                    };
                    self.tokens.push_back(token);
                }
                self.tokens.push_back(MarkStart(start));
                self.tokens.push_back(MarkEnd(end));
                new_line_token = 1;
            }
        }
        match chomp {
            ChompIndicator::Keep => self.tokens.push_back(NewLine(new_line_token)),
            ChompIndicator::Clip => self.tokens.push_back(NewLine(1)),
            ChompIndicator::Strip => {}
        }
        if is_new_seq_entry {
            self.tokens.push_back(Separator);
        }
    }

    // TODO Escaping properly
    fn fetch_quoted_scalar<R: Reader>(&mut self, reader: &mut R, quote: u8) {
        let mut start = reader.pos();
        let mut first = 1;
        reader.consume_bytes(1);
        while let Some(offset) = reader.find_fast3_iter(quote, b'\r', b'\n') {
            match reader.peek_byte_at(offset) {
                Some(b'\r') | Some(b'\n') => {
                    if offset > 0 {
                        self.tokens.push_back(MarkStart(start));
                        self.tokens.push_back(MarkEnd(start + offset + first));
                        self.tokens.push_back(Space);
                    }
                    reader.read_line();
                    reader.skip_space_tab(self.curr_state.is_implicit());
                    start = reader.pos();
                    first = 0;
                }
                Some(_) => {
                    // consume offset and the next quote
                    reader.consume_bytes(offset + 1);
                    self.tokens.push_back(MarkStart(start));
                    self.tokens.push_back(MarkEnd(start + offset + first));
                    break;
                }
                None => {}
            };
        }
    }

    fn fetch_plain_scalar<R: Reader>(&mut self, reader: &mut R) {
        let mut num_newlines = 0;
        let mut allow_minus = false;
        while !reader.eof() {
            let (start, end) = match self.read_plain_one_line(reader, allow_minus) {
                None => return,
                Some(x) => x,
            };

            match num_newlines {
                x if x == 1 => self.tokens.push_back(Space),
                x if x > 1 => self.tokens.push_back(NewLine(num_newlines)),
                _ => {}
            }

            self.tokens.push_back(MarkStart(start));
            self.tokens.push_back(MarkEnd(end));

            let chr = reader.peek_byte_unwrap(0);
            if self.curr_state.in_flow_collection() && is_flow_indicator(chr) {
                break;
            }

            if is_white_tab(chr) {
                reader.skip_space_tab(false);
            } else if is_newline(chr) {
                let folded_newline = self.skip_separation_spaces(reader, false);
                if reader.col() >= self.curr_state.indent() {
                    num_newlines = folded_newline as u32;
                }
            }

            match self.finish_scalar(reader, 0) {
                Ok(ScalarControl::SameIndent) => {
                    self.tokens.push_back(Separator);
                }
                Ok(ScalarControl::GreaterIndent) => {
                    allow_minus = true;
                    continue;
                }
                Err(err) => self.tokens.push_back(ErrorToken(err)),
                Ok(ScalarControl::Continue) => continue,
            }
        }
    }

    #[inline]
    fn finish_scalar<R: Reader>(
        &self,
        reader: &mut R,
        offset: usize,
    ) -> Result<ScalarControl, ErrorType> {
        if reader.peek_byte_unwrap(offset) == b'-' {
            let col_pos = reader.col() + offset;
            match self.curr_state {
                BlockSeq(x) if col_pos == x => {
                    reader.consume_bytes(offset + 1);
                    return Ok(ScalarControl::SameIndent);
                }
                BlockSeq(x) if col_pos > x => {
                    return Ok(ScalarControl::GreaterIndent);
                }
                BlockSeq(x) if col_pos < x => {
                    let token = ExpectedIndent {
                        expected: x,
                        actual: col_pos,
                    };
                    reader.read_line();
                    return Err(token);
                }
                _ => {}
            }
        }
        Ok(ScalarControl::Continue)
    }

    fn skip_separation_spaces<R: Reader>(&mut self, reader: &mut R, allow_comments: bool) -> usize {
        let mut num_breaks = 0;
        let mut found_eol = true;
        while !reader.eof() {
            reader.skip_space_tab(true);

            if allow_comments && reader.peek_byte_is(b'#') {
                reader.read_line();
                found_eol = true;
                num_breaks += 1;
            }

            if reader.read_break().is_some() {
                num_breaks += 1;
                found_eol = true;
            }

            if !found_eol {
                break;
            } else {
                reader.skip_space_tab(false);
                found_eol = false;
            }
        }
        num_breaks
    }

    fn read_plain_one_line<R: Reader>(
        &mut self,
        reader: &mut R,
        allow_minus: bool,
    ) -> Option<(usize, usize)> {
        let start = reader.pos();
        let in_flow_collection = self.curr_state.in_flow_collection();

        if !(allow_minus && reader.peek_byte_is(b'-'))
            && (reader.eof()
                || reader.peek_byte_at_check(0, is_white_tab_or_break)
                || reader.peek_byte_at_check(0, is_indicator)
                || (reader.peek_byte_is(b'-') && !reader.peek_byte_at_check(1, is_white_tab))
                || ((reader.peek_byte_is(b'?') || reader.peek_byte_is(b':'))
                    && !reader.peek_byte_at_check(1, is_white_tab_or_break)))
        {
            return None;
        }

        let mut end = reader.consume_bytes(1);

        while !reader.eof() {
            let spaces = reader.count_space_tab(true);
            let read_iter = reader.position_until(spaces, |pos, x0, x1| {
                // ns-plain-char  prevent ` #`
                if is_white_tab_or_break(x0) && x1 == b'#' {
                    return Break(pos);
                }

                // ns-plain-char prevent `: `
                // or `:{`  in flow collections
                if x0 == b':' && !ns_plain_safe(x1, in_flow_collection) {
                    return Break(pos);
                }

                if !ns_plain_safe(x0, in_flow_collection) {
                    return Break(pos);
                } else if !ns_plain_safe(x1, in_flow_collection) {
                    return Break(pos + 1);
                };

                Continue(pos + 1)
            });
            if read_iter == 0 {
                break;
            } else {
                end = reader.consume_bytes(read_iter + spaces);
            }
        }
        Some((start, end))
    }

    fn fetch_explicit_map<R: Reader>(&mut self, reader: &mut R) {
        if !self.is_map() {
            self.tokens.push_back(MappingStart);
        }

        if !reader.peek_byte_at_check(1, is_white_tab_or_break) {
            self.fetch_plain_scalar(reader);
            return;
        }
        reader.consume_bytes(1);
        reader.skip_space_tab(true);
    }

    fn process_map_key<R: Reader>(&mut self, reader: &mut R, indent: usize) {
        reader.consume_bytes(1);

        if self.is_key() {
            self.curr_state = FlowMap(indent);
            self.tokens.push_back(KeyEnd);
        } else {
            self.fetch_plain_scalar(reader);
        }
    }

    #[inline]
    fn is_prev_sequence(&self) -> bool {
        match self.stack.back() {
            Some(FlowSeq(_)) => true,
            _ => false,
        }
    }

    #[inline]
    fn is_map(&self) -> bool {
        match self.curr_state {
            FlowMap(_) | FlowKey(_, _) => true,
            _ => false,
        }
    }

    #[inline]
    fn is_key(&self) -> bool {
        match self.curr_state {
            FlowKey(_, _) => true,
            _ => false,
        }
    }
}

#[derive(Copy, Clone)]
pub enum SpanToken {
    ErrorToken(ErrorType),
    MarkStart(usize),
    MarkEnd(usize),
    NewLine(u32),
    Space,
    Directive(DirectiveType),
    Alias,
    Separator,
    KeyEnd,
    SequenceStart,
    SequenceEnd,
    MappingStart,
    MappingEnd,
    DocumentStart,
    DocumentEnd,
}
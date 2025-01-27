#![allow(clippy::match_like_matches_macro)]

use std::collections::VecDeque;

use ErrorType::NoDocStartAfterTag;
use SpanToken::{DocumentStart, Separator, Space};

use crate::tokenizer::reader::{
    is_flow_indicator, is_indicator, is_white_tab, is_white_tab_or_break, ns_plain_safe, Reader,
};
use crate::tokenizer::spanner::ParserState::{
    AfterDocEnd, BlockMap, BlockSeq, FlowKey, FlowMap, FlowSeq, PreDocStart, RootBlock,
};
use crate::tokenizer::spanner::SpanToken::{
    Directive, ErrorToken, KeyEnd, MappingEnd, MappingStart, MarkEnd, MarkStart, NewLine,
    SequenceEnd, SequenceStart,
};
use crate::tokenizer::ErrorType::ExpectedIndent;
use crate::tokenizer::ErrorType::UnexpectedSymbol;
use crate::tokenizer::{DirectiveType, ErrorType};

use super::reader::is_newline;

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
    BlockMap(usize),
    AfterDocEnd,
}

impl ParserState {
    #[inline]
    pub(crate) fn indent(&self, default: usize) -> usize {
        match self {
            FlowKey(ind, _) | FlowMap(ind) | FlowSeq(ind) | BlockSeq(ind) | BlockMap(ind) => *ind,
            RootBlock => default,
            PreDocStart | AfterDocEnd => 0,
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

    #[inline]
    fn is_block_col(&self) -> bool {
        matches!(self, BlockMap(_) | BlockSeq(_))
    }

    #[inline]
    fn is_new_block_col(&self, curr_indent: usize) -> bool {
        match &self {
            FlowKey(_, _) | FlowMap(_) | FlowSeq(_) => false,
            BlockMap(x) if *x == curr_indent => false,
            _ => true,
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
            RootBlock | BlockMap(_) | BlockSeq(_) => {
                let indent = self.curr_state.indent(reader.col());
                match reader.peek_byte() {
                    Some(b'{') => self.fetch_flow_col(reader, indent),
                    Some(b'[') => self.fetch_flow_col(reader, indent),
                    Some(b'&') => self.fetch_alias(reader),
                    Some(b'*') => self.fetch_anchor(reader),
                    Some(b':') => {
                        reader.consume_bytes(1);
                        self.tokens.push_back(KeyEnd);
                    }
                    Some(b'-') => self.fetch_block_seq(reader, indent),
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
                            self.fetch_plain_scalar(reader, indent);
                        } else {
                            reader.consume_bytes(1);
                            self.tokens
                                .push_back(ErrorToken(UnexpectedSymbol(x as char)))
                        }
                    }
                    None => self.stream_end = true,
                }
            }
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
                    self.fetch_plain_scalar(reader, indent);
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
                    self.fetch_plain_scalar(reader, indent);
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
                    BlockMap(_) => MappingEnd,
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
            self.fetch_plain_scalar(reader, indent);
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
        let mut chomp = ChompIndicator::Clip;
        let mut indentation: usize = 0;

        match (reader.peek_byte_unwrap(0), reader.peek_byte_unwrap(1)) {
            (b'-', len) | (len, b'-') if matches!(len, b'1'..=b'9') => {
                reader.consume_bytes(2);
                chomp = ChompIndicator::Strip;
                indentation = self.curr_state.indent(0) + (len - b'0') as usize;
            }
            (b'+', len) | (len, b'+') if matches!(len, b'1'..=b'9') => {
                reader.consume_bytes(2);
                chomp = ChompIndicator::Keep;
                indentation = self.curr_state.indent(0) + (len - b'0') as usize;
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
                indentation = self.curr_state.indent(0) + (len - b'0') as usize;
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
        let mut trailing = vec![];
        let mut is_trailing_comment = false;
        let mut previous_indent = 0;
        while !reader.eof() {
            let curr_indent = self.curr_state.indent(0);

            if let (b'-', BlockSeq(ind)) = (reader.peek_byte_unwrap(curr_indent), self.curr_state) {
                if reader.col() + curr_indent == ind {
                    reader.consume_bytes(1 + curr_indent);
                    trailing.push(Separator);
                    break;
                }
            }

            // count indents important for folded scalars
            let newline_indent = reader.count_space_tab(false);

            if !is_trailing_comment
                && newline_indent < indentation
                && reader.peek_byte_unwrap(newline_indent) == b'#'
            {
                trailing.push(NewLine(new_line_token - 1));
                is_trailing_comment = true;
                new_line_token = 1;
            };

            let newline_is_empty = reader.peek_byte_at_check(newline_indent, is_newline)
                || (is_trailing_comment && reader.peek_byte_unwrap(newline_indent) == b'#');

            if indentation == 0 && newline_indent > 0 && !newline_is_empty {
                indentation = newline_indent;
            }

            if newline_is_empty {
                new_line_token += 1;
                reader.read_line();
                continue;
            } else if let Err(x) = reader.skip_n_spaces(indentation) {
                self.tokens.push_back(ErrorToken(x));
                break;
            }

            let (start, end) = reader.read_line();
            if start != end {
                if new_line_token > 0 {
                    let token =
                        if new_line_token == 1 && !literal && previous_indent == newline_indent {
                            Space
                        } else {
                            NewLine(new_line_token)
                        };
                    self.tokens.push_back(token);
                }
                previous_indent = newline_indent;
                self.tokens.push_back(MarkStart(start));
                self.tokens.push_back(MarkEnd(end));
                new_line_token = 1;
            }
        }
        match chomp {
            ChompIndicator::Keep => {
                if is_trailing_comment {
                    new_line_token = 1;
                }
                trailing.insert(0, NewLine(new_line_token));
                self.tokens.extend(trailing);
            }
            ChompIndicator::Clip => {
                trailing.insert(0, NewLine(1));
                self.tokens.extend(trailing);
            }
            ChompIndicator::Strip => {}
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

    fn fetch_plain_scalar<R: Reader>(&mut self, reader: &mut R, start_indent: usize) {
        let mut allow_minus = false;
        let mut first_line_block = !self.curr_state.in_flow_collection();
        let mut num_newlines = 0;
        let mut tokens = vec![];
        let mut curr_indent = reader.col();
        let init_indent = if matches!(self.curr_state, BlockMap(_)) {
            reader.col()
        } else {
            start_indent
        };
        let mut had_comment = false;

        while !reader.eof() {
            // if plain scalar is less indentend than previous
            // It can be
            // a) Part of BlockMap
            // b) An error outside of block map
            if curr_indent < init_indent {
                if matches!(self.curr_state, BlockMap(_)) {
                    tokens.push(Separator);
                } else if !self.curr_state.is_block_col() {
                    reader.read_line();
                    tokens.push(ErrorToken(ErrorType::ExpectedIndent {
                        actual: curr_indent,
                        expected: start_indent,
                    }));
                }
                break;
            }

            let (start, end) = match self.read_plain_one_line(reader, allow_minus, &mut had_comment) {
                Some(x) => x,
                None => break,
            };
            

            reader.skip_space_tab(true);

            let chr = reader.peek_byte_unwrap(0);

            if first_line_block && chr == b':' {
                if self.curr_state.is_new_block_col(curr_indent) {
                    self.push_state(BlockMap(curr_indent));
                    self.tokens.push_back(MappingStart);
                }
                self.tokens.push_back(MarkStart(start));
                self.tokens.push_back(MarkEnd(end));
                return;
            }

            match num_newlines {
                x if x == 1 => tokens.push(Space),
                x if x > 1 => tokens.push(NewLine(num_newlines)),
                _ => {}
            }

            tokens.push(MarkStart(start));
            tokens.push(MarkEnd(end));
            first_line_block = false;

            if is_newline(chr) {
                let folded_newline = self.skip_separation_spaces(reader, false);
                if reader.col() >= self.curr_state.indent(0) {
                    num_newlines = folded_newline as u32;
                }
                curr_indent = reader.col();
            }

            if self.curr_state.in_flow_collection() && is_flow_indicator(chr) {
                break;
            }

            match (reader.peek_byte_unwrap(0), self.curr_state) {
                (b'-', BlockSeq(ind)) if reader.col() == ind => {
                    reader.consume_bytes(1);
                    tokens.push(Separator);
                    break;
                }
                (b'-', BlockSeq(ind)) if reader.col() < ind => {
                    reader.read_line();
                    let err_type = ExpectedIndent {
                        expected: ind,
                        actual: curr_indent,
                    };
                    tokens.push(ErrorToken(err_type));
                    break;
                }
                (b'-', BlockSeq(ind)) if reader.col() > ind => {
                    allow_minus = true;
                }
                _ => {}
            }
        }
        self.tokens.extend(tokens);
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
        had_comment: &mut bool,
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

        let end = reader.consume_bytes(1);
        let (_, line_end, _) = reader.get_line_offset();
        let line_end = reader.eof_or_pos(line_end);
        let mut end_of_str = end;

        for (prev, curr, next, pos) in reader.get_lookahead_iterator(end..=line_end) {
            // ns-plain-char  prevent ` #`
            if curr == b'#' && is_white_tab_or_break(prev) {
                // if we encounter two or more comment print error and try to recover
                if *had_comment {
                    self.tokens.push_back(ErrorToken(ErrorType::UnexpectedComment))
                } else {
                    *had_comment = true;
                    reader.set_pos(line_end);
                    return Some((start, end_of_str));
                }
                break;
            }

            // ns-plain-char prevent `: `
            // or `:{`  in flow collections
            if curr == b':' && !ns_plain_safe(next, in_flow_collection) {
                // commit any uncommitted character, but ignore first character
                if !is_white_tab(prev) && pos != end {
                    end_of_str += 1;
                }
                break;
            }

            // if current character is a flow indicator, break
            if is_flow_indicator(curr) {
                break;
            }

            if is_white_tab_or_break(curr) {
                // commit any uncommitted character, but ignore first character
                if !is_white_tab_or_break(prev) && pos != end {
                    end_of_str += 1;
                }
                continue;
            }
            end_of_str = pos;
        }

        reader.set_pos(end_of_str);
        Some((start, end_of_str))
    }

    fn fetch_explicit_map<R: Reader>(&mut self, reader: &mut R) {
        if !self.is_map() {
            self.tokens.push_back(MappingStart);
        }

        if !reader.peek_byte_at_check(1, is_white_tab_or_break) {
            self.fetch_plain_scalar(reader, reader.col());
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
            self.fetch_plain_scalar(reader, indent);
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

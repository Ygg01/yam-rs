#![allow(clippy::match_like_matches_macro)]

use std::collections::VecDeque;
use std::ops::ControlFlow::{Break, Continue};

use ErrorType::NoDocStartAfterTag;
use SpanToken::{DocumentStart, Separator, Space};

use crate::tokenizer::reader::{
    is_flow_indicator, is_indicator, is_white_tab, is_white_tab_or_break, ns_plain_safe, Reader,
};
use crate::tokenizer::spanner::ParserState::{
    BlockKey, BlockMap, BlockSeq, FlowKey, FlowMap, FlowSeq, PreDocStart, RootBlock,
};
use crate::tokenizer::spanner::SpanToken::{
    Directive, ErrorToken, KeyEnd, MappingEnd, MappingStart, MarkEnd, MarkStart, NewLine,
    SequenceEnd, SequenceStart,
};
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
    BlockKey(usize),
    AfterDocEnd,
}

impl ParserState {
    pub(crate) fn indent(&self) -> usize {
        match self {
            FlowKey(ind, _)
            | FlowMap(ind)
            | FlowSeq(ind)
            | BlockKey(ind)
            | BlockSeq(ind)
            | BlockMap(ind) => *ind,
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
                Some(b'-') => self.switch_to_block_seq(reader, 0),
                Some(b'?') => self.fetch_block_map_key(reader),
                Some(b'!') => self.fetch_tag(reader),
                Some(b'>') => self.fetch_block_scalar(reader, false),
                Some(b'|') => self.fetch_block_scalar(reader, false),
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
                Some(b',') => {reader.consume_bytes(1);},
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
            for state in self.stack.iter().rev()  {
                let x = match *state {
                    BlockMap(_) => MappingEnd,
                    BlockSeq(_) => SequenceEnd,
                    _ => continue,
                };
                self.tokens.push_back(x);
            }
        }
    }

    fn fetch_flow_col<R: Reader>(&mut self, reader: &mut R, indent: usize) {
        let peek = reader.peek_byte().unwrap_or(b'\0');
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

    fn switch_to_block_seq<R: Reader>(&mut self, reader: &mut R, indent: usize) {
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

    fn fetch_block_seq<R: Reader>(&self, _reader: &mut R, _indent: usize) {
        todo!()
    }

    fn fetch_block_map_key<R: Reader>(&mut self, _reader: &mut R) {
        todo!()
    }

    fn fetch_tag<R: Reader>(&mut self, _reader: &mut R) {
        todo!()
    }

    fn fetch_block_scalar<R: Reader>(&mut self, _reader: &mut R, _literal: bool) {
        todo!()
    }

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
        while !reader.eof() {
            if let Some((start, end)) = self.read_plain_one_line(reader) {
                match num_newlines {
                    x if x == 1 => self.tokens.push_back(Space),
                    x if x > 1 => self.tokens.push_back(NewLine(num_newlines)),
                    _ => {}
                }

                self.tokens.push_back(MarkStart(start));
                self.tokens.push_back(MarkEnd(end));
            } else {
                return;
            }

            let chr = reader.peek_byte().unwrap_or(b'\0');

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

            if reader.peek_byte_is(b'-') {
                match self.curr_state {
                    BlockSeq(x) if x == reader.col() => {
                        self.tokens.push_back(Separator);
                        reader.consume_bytes(1);
                        return;
                    },
                    BlockSeq(x) if x > reader.col() => continue,
                    _ => {}
                }
            }
        }
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

    fn read_plain_one_line<R: Reader>(&mut self, reader: &mut R) -> Option<(usize, usize)> {
        let start = reader.pos();
        let in_flow_collection = self.curr_state.in_flow_collection();

        if reader.eof()
            || reader.peek_byte_at_check(0, is_white_tab_or_break)
            || reader.peek_byte_at_check(0, is_indicator)
            || (reader.peek_byte_is(b'-') && !reader.peek_byte_at_check(1, is_white_tab))
            || ((reader.peek_byte_is(b'?') || reader.peek_byte_is(b':'))
                && !reader.peek_byte_at_check(1, is_white_tab_or_break))
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

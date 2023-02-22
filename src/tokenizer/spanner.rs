#![allow(clippy::match_like_matches_macro)]

use std::collections::VecDeque;

use ErrorType::NoDocStartAfterTag;
use SpanToken::{DocumentStart, Separator};

use crate::tokenizer::reader::{is_white_tab_or_break, Reader};
use crate::tokenizer::spanner::ParserState::{
    AfterDocEnd, BlockMap, BlockSeq, FlowKey, FlowMap, FlowSeq, PreDocStart, RootBlock,
};
use crate::tokenizer::spanner::SpanToken::{
    ErrorToken, KeyEnd, MappingEnd, MappingStart, SequenceEnd, SequenceStart,
};
use crate::tokenizer::ErrorType::UnexpectedSymbol;
use crate::tokenizer::{DirectiveType, ErrorType};

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
pub enum ParserState {
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
    pub(crate) fn is_block_col(&self) -> bool {
        matches!(self, BlockMap(_) | BlockSeq(_))
    }

    #[inline]
    pub(crate) fn is_new_block_col(&self, curr_indent: usize) -> bool {
        match &self {
            FlowKey(_, _) | FlowMap(_) | FlowSeq(_) => false,
            BlockMap(x) if *x == curr_indent => false,
            _ => true,
        }
    }
}

impl Spanner {
    #[inline(always)]
    pub fn pop_token(&mut self) -> Option<SpanToken> {
        self.tokens.pop_front()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn fetch_next_token<R: Reader>(&mut self, reader: &mut R) {
        reader.skip_separation_spaces(true);
        match self.curr_state {
            PreDocStart => {
                if !reader.peek_byte_is(b'%') {
                    self.curr_state = RootBlock;
                    return;
                }
                reader.try_read_tag(&mut self.tokens);
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
                    Some(b'|') => {
                        reader.read_block_scalar(true, &self.curr_state, &mut self.tokens)
                    }
                    Some(b'>') => {
                        reader.read_block_scalar(false, &self.curr_state, &mut self.tokens)
                    }
                    Some(b'\'') => {
                        reader.read_single_quote(self.curr_state.is_implicit(), &mut self.tokens)
                    }
                    Some(b'"') => {
                        reader.read_double_quote(self.curr_state.is_implicit(), &mut self.tokens)
                    }
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
                Some(b'\'') => {
                    reader.read_single_quote(self.curr_state.is_implicit(), &mut self.tokens)
                }
                Some(b'"') => {
                    reader.read_double_quote(self.curr_state.is_implicit(), &mut self.tokens)
                }
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
                Some(b'\'') => {
                    reader.read_single_quote(self.curr_state.is_implicit(), &mut self.tokens)
                }
                Some(b'"') => {
                    reader.read_double_quote(self.curr_state.is_implicit(), &mut self.tokens)
                }
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
            None => self.curr_state = AfterDocEnd,
        }
    }

    fn fetch_alias<R: Reader>(&mut self, _reader: &mut R) {
        todo!()
    }

    fn fetch_anchor<R: Reader>(&mut self, _reader: &mut R) {
        todo!()
    }

    fn fetch_block_seq<R: Reader>(&mut self, reader: &mut R, indent: usize) {
        if let Some(new_state) = reader.read_block_seq(indent) {
            self.tokens.push_back(SequenceStart);
            self.push_state(new_state);
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

    fn fetch_plain_scalar<R: Reader>(&mut self, reader: &mut R, start_indent: usize) {
        let (tokens, new_state) = reader.read_plain_scalar(start_indent, &self.curr_state);
        if let Some(new_state) = new_state {
            self.push_state(new_state);
        }
        self.tokens.extend(tokens);
    }

    fn fetch_explicit_map<R: Reader>(&mut self, reader: &mut R) {
        if !self.is_map() {
            self.tokens.push_back(MappingStart);
        }

        if !reader.peek_byte_at_check(1, is_white_tab_or_break) {
            self.fetch_plain_scalar(reader, reader.col());
        } else {
            reader.consume_bytes(1);
            reader.skip_space_tab(true);
        }
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

#![allow(clippy::match_like_matches_macro)]

use std::collections::VecDeque;
use std::hint::unreachable_unchecked;

use ErrorType::ExpectedIndent;
use LexerState::PreDocStart;

use crate::tokenizer::reader::{is_white_tab_or_break, Reader};
use crate::tokenizer::spanner::LexerState::{
    AfterDocEnd, BlockMap, BlockMapExp, BlockSeq, DirectiveSection, FlowKeyExp, FlowMap, FlowSeq,
    RootBlock,
};
use crate::tokenizer::spanner::LexerToken::*;
use crate::tokenizer::spanner::MapState::{AfterColon, BeforeColon, BeforeKey};
use crate::tokenizer::spanner::SeqState::{BeforeSeq, InSeq};
use crate::tokenizer::ErrorType;
use crate::tokenizer::ErrorType::UnexpectedSymbol;

use super::iterator::{DirectiveType, ScalarType};
use super::reader::{is_flow_indicator, is_newline};

#[derive(Clone, Default)]
pub struct Lexer {
    pub stream_end: bool,
    pub directive: bool,
    pub(crate) tokens: VecDeque<usize>,
    pub(crate) errors: Vec<ErrorType>,
    stack: Vec<LexerState>,
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum MapState {
    #[default]
    BeforeKey,
    BeforeColon,
    AfterColon,
}

impl MapState {
    pub fn next_state(&self) -> MapState {
        match self {
            BeforeKey => BeforeColon,
            BeforeColon => AfterColon,
            AfterColon => BeforeKey,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum SeqState {
    #[default]
    BeforeSeq,
    InSeq,
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum LexerState {
    #[default]
    PreDocStart,
    DirectiveSection,
    RootBlock,
    FlowSeq(u32, SeqState),
    FlowMap(u32, MapState),
    FlowKeyExp(u32, MapState),
    BlockSeq(u32),
    BlockMap(u32, MapState),
    BlockMapExp(u32, MapState),
    AfterDocEnd,
}

impl LexerState {
    #[inline]
    pub(crate) fn indent(&self, default: usize) -> u32 {
        match self {
            FlowKeyExp(ind, _)
            | FlowMap(ind, _)
            | FlowSeq(ind, _)
            | BlockSeq(ind)
            | BlockMap(ind, _)
            | BlockMapExp(ind, _) => *ind,
            RootBlock => default as u32,
            PreDocStart | AfterDocEnd | DirectiveSection => 0,
        }
    }

    #[inline]
    pub(crate) fn get_block_indent(&self, default: usize) -> usize {
        match self {
            BlockMapExp(ind, _) => *ind as usize,
            _ => default,
        }
    }

    #[inline]
    pub(crate) fn wrong_exp_indent(&self, curr_indent: usize) -> bool {
        match self {
            BlockMapExp(ind, _) => *ind as usize != curr_indent,
            _ => false,
        }
    }

    #[inline]
    pub fn in_flow_collection(&self) -> bool {
        match &self {
            FlowKeyExp(_, _) | FlowSeq(_, _) | FlowMap(_, _) => true,
            _ => false,
        }
    }

    #[inline]
    pub(crate) fn is_implicit(&self) -> bool {
        match &self {
            FlowKeyExp(_, _) => true,
            _ => false,
        }
    }
}

impl Lexer {
    #[inline(always)]
    pub fn curr_state(&self) -> LexerState {
        *self.stack.last().unwrap_or(&LexerState::default())
    }

    #[inline(always)]
    pub fn set_curr_state(&mut self, state: LexerState) {
        match self.stack.last_mut() {
            Some(x) => *x = state,
            None => self.push_state(state),
        }
    }

    #[inline]
    fn set_map_state(&mut self, map_state: MapState) {
        match self.stack.last_mut() {
            Some(BlockMap(_, state)) | Some(BlockMapExp(_, state)) => *state = map_state,
            _ => {}
        };
    }

    #[inline]
    fn set_next_map_state(&mut self) {
        let new_state = match self.stack.last() {
            Some(FlowMap(ind, state)) => FlowMap(*ind, state.next_state()),
            Some(FlowKeyExp(ind, state)) => FlowKeyExp(*ind, state.next_state()),
            Some(BlockMap(ind, state)) => BlockMap(*ind, state.next_state()),
            Some(BlockMapExp(ind, AfterColon)) => BlockMap(*ind, BeforeKey),
            Some(BlockMapExp(ind, state)) => BlockMapExp(*ind, state.next_state()),
            _ => return,
        };
        if let Some(x) = self.stack.last_mut() {
            *x = new_state
        };
    }

    #[inline(always)]
    pub fn pop_token(&mut self) -> Option<usize> {
        self.tokens.pop_front()
    }

    #[inline(always)]
    pub fn tokens(self) -> VecDeque<usize> {
        self.tokens
    }

    #[inline(always)]
    pub fn peek_token(&mut self) -> Option<usize> {
        self.tokens.front().copied()
    }

    #[inline(always)]
    pub fn peek_token_next(&mut self) -> Option<usize> {
        self.tokens.get(1).copied()
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn fetch_next_token<B, R: Reader<B>>(&mut self, reader: &mut R) {
        reader.skip_separation_spaces(true);
        match self.curr_state() {
            PreDocStart => {
                if reader.peek_byte_is(b'%') {
                    self.push_state(DirectiveSection);
                    return;
                } else if reader.peek_byte_is(b'#') {
                    reader.read_line();
                } else if reader.try_read_slice_exact("---") {
                    self.directive = true;
                    self.tokens.push_back(DocumentStart as usize);
                    self.push_state(RootBlock);
                } else {
                    self.tokens.push_back(DocumentStart as usize);
                    self.push_state(RootBlock);
                }
                return;
            }
            DirectiveSection => {
                if !reader.try_read_yaml_directive(&mut self.tokens) {
                    if reader.try_read_slice_exact("---") {
                        self.tokens.push_back(DocumentStart as usize);
                        self.set_curr_state(RootBlock);
                        self.directive = true;
                        return;
                    } else if reader.peek_byte_is(b'#') {
                        reader.read_line();
                    }
                } else if reader.peek_byte_is(b'#') {
                    reader.read_line();
                }
            }
            RootBlock | BlockMap(_, _) | BlockMapExp(_, _) => {
                let curr_state = self.curr_state();
                let (indent, init_indent) = match curr_state {
                    RootBlock => (0, reader.col() as u32),
                    BlockMapExp(ind, _) => (ind, ind),
                    BlockMap(ind, _) => (ind, reader.col() as u32),
                    _ => unsafe { unreachable_unchecked() },
                };
                match reader.peek_byte() {
                    Some(b'{') => self.fetch_flow_map(reader, indent as usize),
                    Some(b'[') => self.fetch_flow_seq(reader, indent as usize),
                    Some(b'&') => {
                        reader.consume_anchor_alias(&mut self.tokens, AnchorToken);
                    }
                    Some(b'*') => reader.consume_anchor_alias(&mut self.tokens, AliasToken),
                    Some(b':') if reader.peek_byte2().map_or(true, is_white_tab_or_break) => {
                        self.process_colon(reader, curr_state, indent);
                    }
                    Some(b'-') if reader.peek_byte2().map_or(false, is_white_tab_or_break) => {
                        let new_indent = reader.col();
                        reader.consume_bytes(1);

                        self.process_seq(new_indent, indent as usize);
                    }
                    Some(b'?') if reader.peek_byte2().map_or(false, is_white_tab_or_break) => {
                        self.fetch_exp_block_map_key(reader)
                    }
                    Some(b'!') => self.fetch_tag(reader),
                    Some(b'|') => reader.read_block_scalar(
                        true,
                        &self.curr_state(),
                        &mut self.tokens,
                        &mut self.errors,
                    ),
                    Some(b'>') => reader.read_block_scalar(
                        false,
                        &self.curr_state(),
                        &mut self.tokens,
                        &mut self.errors,
                    ),
                    Some(b'\'') => reader.read_single_quote(false, &mut self.tokens),
                    Some(b'"') => reader.read_double_quote(false, &mut self.tokens),
                    Some(b'#') => {
                        // comment
                        reader.read_line();
                    }
                    Some(x) => {
                        if x != b']' && x != b'}' && x != b'@' {
                            self.fetch_plain_scalar_block(
                                reader,
                                indent as usize,
                                init_indent as usize,
                            );
                        } else {
                            reader.consume_bytes(1);
                            self.tokens.push_back(ErrorToken as usize);
                            self.errors.push(UnexpectedSymbol(x as char))
                        }
                    }
                    None => self.stream_end = true,
                }
            }
            BlockSeq(indent) => {
                match reader.peek_byte() {
                    Some(b'{') => self.fetch_flow_map(reader, indent as usize),
                    Some(b'[') => self.fetch_flow_seq(reader, indent as usize),
                    Some(b'&') => reader.consume_anchor_alias(&mut self.tokens, AnchorToken),
                    Some(b'*') => reader.consume_anchor_alias(&mut self.tokens, AliasToken),
                    Some(b'-') if reader.peek_byte2().map_or(false, is_white_tab_or_break) => {
                        let new_indent = reader.col();
                        reader.consume_bytes(1);

                        self.process_seq(new_indent, indent as usize);
                    }
                    Some(b'?') if reader.peek_byte2().map_or(false, is_white_tab_or_break) => {
                        self.fetch_exp_block_map_key(reader)
                    }
                    Some(b'!') => self.fetch_tag(reader),
                    Some(b'|') => reader.read_block_scalar(
                        true,
                        &self.curr_state(),
                        &mut self.tokens,
                        &mut self.errors,
                    ),
                    Some(b'>') => reader.read_block_scalar(
                        false,
                        &self.curr_state(),
                        &mut self.tokens,
                        &mut self.errors,
                    ),
                    Some(b'\'') => reader.read_single_quote(false, &mut self.tokens),
                    Some(b'"') => reader.read_double_quote(false, &mut self.tokens),
                    Some(b'#') => {
                        // comment
                        reader.read_line();
                    }
                    Some(x) => {
                        if x != b']' && x != b'}' && x != b'@' {
                            self.fetch_plain_scalar_block(reader, indent as usize, indent as usize);
                        } else {
                            reader.consume_bytes(1);
                            self.tokens.push_back(ErrorToken as usize);
                            self.errors.push(UnexpectedSymbol(x as char))
                        }
                    }
                    None => self.stream_end = true,
                }
            }
            FlowSeq(indent, seq_state) => match reader.peek_byte() {
                Some(b'&') => reader.consume_anchor_alias(&mut self.tokens, AnchorToken),
                Some(b'*') => reader.consume_anchor_alias(&mut self.tokens, AliasToken),
                Some(b'[') => self.fetch_flow_seq(reader, (indent + 1) as usize),
                Some(b'{') => self.fetch_flow_map(reader, (indent + 1) as usize),
                Some(b']') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(SequenceEnd as usize);
                    self.stack.pop();
                }
                Some(b':') if seq_state == BeforeSeq => {
                    self.tokens.push_back(MappingStart as usize);
                    self.tokens.push_back(ScalarPlain as usize);
                    self.tokens.push_back(ScalarEnd as usize);
                    self.set_curr_state(FlowSeq(indent, InSeq));
                    self.push_state(FlowMap(indent + 1, AfterColon));
                }
                Some(b'}') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(ErrorToken as usize);
                    self.errors.push(UnexpectedSymbol('}'));
                }
                Some(b',') => {
                    reader.consume_bytes(1);
                    self.tokens.push_back(ScalarEnd as usize);
                    self.set_curr_state(FlowSeq(indent, BeforeSeq));
                }
                Some(b'\'') => {
                    reader.read_single_quote(self.curr_state().is_implicit(), &mut self.tokens)
                }
                Some(b'"') => {
                    reader.read_double_quote(self.curr_state().is_implicit(), &mut self.tokens)
                }
                Some(b'?') => self.fetch_explicit_map(reader),
                Some(b'#') => {
                    // comment
                    reader.read_line();
                }
                Some(_) => {
                    self.get_plain_scalar_flow(reader, indent as usize, reader.col());
                }
                None => self.stream_end = true,
            },

            FlowMap(indent, _) | FlowKeyExp(indent, _) => {
                match reader.peek_byte() {
                    Some(b'&') => {
                        reader.consume_anchor_alias(&mut self.tokens, AnchorToken);
                    }
                    Some(b'*') => reader.consume_anchor_alias(&mut self.tokens, AliasToken),
                    Some(b'[') => {
                        self.set_next_map_state();
                        self.fetch_flow_seq(reader, (indent + 1) as usize);
                    }
                    Some(b'{') => {
                        self.set_next_map_state();
                        self.fetch_flow_map(reader, (indent + 1) as usize)
                    }
                    Some(b'}') => {
                        reader.consume_bytes(1);
                        if matches!(self.curr_state(), FlowMap(_, BeforeColon)) {
                            self.tokens.push_back(ScalarPlain as usize);
                            self.tokens.push_back(ScalarEnd as usize);
                        }
                        self.tokens.push_back(MappingEnd as usize);
                        self.stack.pop();
                    }
                    Some(b':') => {
                        reader.consume_bytes(1);
                        let curr_state = self.curr_state();

                        if matches!(curr_state, FlowMap(_, BeforeKey)) {
                            self.tokens.push_back(ScalarPlain as usize);
                            self.tokens.push_back(ScalarEnd as usize);
                            self.set_next_map_state();
                        } else if matches!(curr_state, FlowMap(_, BeforeColon) | FlowKeyExp(_, _)) {
                            self.set_next_map_state();
                            self.tokens.push_back(ScalarEnd as usize);
                        }
                    }
                    Some(b']') => {
                        if self.is_prev_sequence() {
                            self.tokens.push_back(ScalarPlain as usize);
                            self.tokens.push_back(ScalarEnd as usize);
                            self.tokens.push_back(MappingEnd as usize);
                            self.stack.pop();
                        } else {
                            reader.consume_bytes(1);
                            self.tokens.push_back(ErrorToken as usize);
                            self.errors.push(UnexpectedSymbol(']'));
                        }
                    }
                    Some(b'?') => self.fetch_explicit_map(reader),
                    Some(b',') => {
                        reader.consume_bytes(1);
                    }
                    Some(b'\'') => {
                        reader.read_single_quote(self.curr_state().is_implicit(), &mut self.tokens)
                    }
                    Some(b'"') => {
                        reader.read_double_quote(self.curr_state().is_implicit(), &mut self.tokens)
                    }
                    Some(b'#') => {
                        // comment
                        reader.read_line();
                    }
                    Some(_) => {
                        self.get_plain_scalar_flow(reader, indent as usize, reader.col());
                    }
                    None => self.stream_end = true,
                }
            }

            _ => {}
        }

        if reader.eof() {
            self.stream_end = true;
            for state in self.stack.iter().rev() {
                let x = match *state {
                    BlockSeq(_) => SequenceEnd,
                    BlockMapExp(_, AfterColon) | BlockMap(_, AfterColon) => {
                        self.tokens.push_back(ScalarPlain as usize);
                        self.tokens.push_back(ScalarEnd as usize);
                        MappingEnd
                    }
                    BlockMapExp(_, _) | BlockMap(_, _) => MappingEnd,
                    DirectiveSection => {
                        self.errors.push(ErrorType::DirectiveEndMark);
                        ErrorToken
                    }
                    _ => continue,
                };
                self.tokens.push_back(x as usize);
            }
            self.tokens.push_back(DocumentEnd as usize);
        }
    }

    fn process_colon<B, R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState, indent: u32) {
        let col = reader.col();
        reader.consume_bytes(1);

        if col == 0 && curr_state == RootBlock {
            self.push_state(BlockMap(0, AfterColon));
            self.tokens.push_back(MappingStart as usize);
            self.tokens.push_back(ScalarPlain as usize);
            self.tokens.push_back(ScalarEnd as usize);
        } else if col == 0 && matches!(curr_state, BlockMap(0, BeforeKey)) {
            self.tokens.push_back(ScalarPlain as usize);
            self.tokens.push_back(ScalarEnd as usize);
            self.set_map_state(AfterColon);
        } else if matches!(curr_state, BlockMapExp(_, _) if col != indent as usize) {
            self.tokens.push_back(ErrorToken as usize);
            self.errors.push(ErrorType::ExpectedIndent {
                actual: col,
                expected: curr_state.indent(0) as usize,
            });
            self.set_next_map_state();
        } else {
            self.set_next_map_state();
        }
    }

    fn fetch_flow_seq<B, R: Reader<B>>(&mut self, reader: &mut R, indent: usize) {
        reader.consume_bytes(1);
        self.tokens.push_back(SequenceStart as usize);
        self.push_state(FlowSeq(indent as u32, BeforeSeq));
    }

    fn fetch_flow_map<B, R: Reader<B>>(&mut self, reader: &mut R, indent: usize) {
        reader.consume_bytes(1);
        reader.skip_space_tab(true);

        if reader.peek_byte_is(b'?') {
            self.push_state(FlowKeyExp(indent as u32, BeforeKey));
        } else {
            self.push_state(FlowMap(indent as u32, BeforeKey));
        }
        self.tokens.push_back(MappingStart as usize);
    }

    #[inline]
    fn push_state(&mut self, state: LexerState) {
        self.stack.push(state);
    }

    fn fetch_exp_block_map_key<B, R: Reader<B>>(&mut self, reader: &mut R) {
        let indent = reader.col();
        reader.consume_bytes(1);
        reader.skip_space_tab(true);
        self.push_state(BlockMapExp(indent as u32, BeforeKey));
        self.tokens.push_back(MappingStart as usize);
    }

    fn fetch_tag<B, R: Reader<B>>(&mut self, reader: &mut R) {
        pub use LexerToken::*;

        let start = reader.consume_bytes(1);
        if let Some((mid, end)) = reader.read_tag() {
            self.tokens.push_back(TagStart as usize);
            self.tokens.push_back(start);
            self.tokens.push_back(mid);
            self.tokens.push_back(end);
            reader.consume_bytes(end - start);
        }
    }

    fn fetch_plain_scalar_block<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        start_indent: usize,
        init_indent: usize,
    ) {
        let mut is_multiline = false;
        let mut ends_with = b'\x7F';
        let scalar_tokens = self.get_plain_scalar(
            reader,
            start_indent,
            init_indent,
            &mut is_multiline,
            &mut ends_with,
        );
        let chr = reader.peek_byte().unwrap_or(b'\0');
        if chr == b':' || matches!(self.curr_state(), BlockMap(_, _) | BlockMapExp(_, _)) {
            self.process_map(init_indent, scalar_tokens, ends_with);
        } else {
            self.tokens.extend(scalar_tokens);
            self.set_next_map_state();
        }
    }

    fn process_map(&mut self, init_indent: usize, scalar_tokens: Vec<usize>, chr: u8) {
        match self.curr_state() {
            
            BlockMap(indent, BeforeKey) | BlockMapExp(indent, _)
                if init_indent == indent as usize =>
            {
                self.set_next_map_state();
                self.tokens.extend(scalar_tokens);
            }
            BlockMap(indent, BeforeColon) | BlockMapExp(indent, _)
                if init_indent > indent as usize =>
            {
                self.set_next_map_state();
                self.push_state(BlockMap(init_indent as u32, BeforeKey));
                self.tokens.push_back(MappingStart as usize);
                self.tokens.extend(scalar_tokens);
            }
            BlockMap(indent, AfterColon) | BlockMapExp(indent, _)
                if indent as usize == init_indent =>
            {
                self.tokens.push_back(ScalarPlain as usize);
                self.tokens.push_back(ScalarEnd as usize);
                self.tokens.extend(scalar_tokens);
                self.set_map_state(BeforeColon);
            }
            BlockMap(indent, AfterColon) | BlockMapExp(indent, _)
                if init_indent > indent as usize && chr == b':' =>
            {
                self.set_next_map_state();
                self.tokens.push_back(MappingStart as usize);
                self.push_state(BlockMap(init_indent as u32, BeforeColon));
                self.tokens.extend(scalar_tokens);
            }
            BlockMap(indent, AfterColon) | BlockMapExp(indent, _)
                if init_indent > indent as usize =>
            {
                self.set_next_map_state();
                self.tokens.extend(scalar_tokens);
            }
            state if !matches!(state, BlockMap(_, _) | BlockMapExp(_, _)) => {
                self.push_state(BlockMap(init_indent as u32, BeforeColon));
                self.tokens.push_back(MappingStart as usize);
                self.tokens.extend(scalar_tokens);
            }
            _ => {
                self.unwind_nested_blocks(
                    init_indent,
                    init_indent,
                    |state, indent| matches!(state, BlockMap(ind, _) | BlockMapExp(ind, _) if ind as usize == indent),
                );
                self.tokens.extend(scalar_tokens);
                self.set_next_map_state();
            }
        }
    }

    fn process_seq(&mut self, indent: usize, expected_indent: usize) {
        let curr_state = self.curr_state();
        match curr_state {
            RootBlock => {
                self.push_state(BlockSeq(indent as u32));
                self.tokens.push_back(SequenceStart as usize);
            }
            BlockSeq(ind) if indent > ind as usize => {
                self.push_state(BlockSeq(indent as u32));
                self.tokens.push_back(SequenceStart as usize);
            }
            BlockSeq(ind) if indent == ind as usize => {}
            _ => {
                self.unwind_nested_blocks(
                    indent,
                    expected_indent,
                    |state, indent| matches!(state, BlockSeq(ind) if ind as usize == indent),
                );
            }
        }
    }

    fn unwind_nested_blocks(
        &mut self,
        init_indent: usize,
        expected_indent: usize,
        f: fn(LexerState, usize) -> bool,
    ) {
        if let Some(unwind) = self
            .stack
            .iter()
            .rposition(|state| f(*state, init_indent))
            .map(|x| self.stack.len() - x - 1)
        {
            if unwind == 0 {
                return;
            }
            for _ in 0..unwind {
                match self.stack.pop() {
                    Some(BlockSeq(_)) => self.tokens.push_back(SequenceEnd as usize),
                    Some(BlockMap(_, AfterColon) | BlockMapExp(_, AfterColon)) => {
                        self.tokens.push_back(ScalarPlain as usize);
                        self.tokens.push_back(ScalarEnd as usize);
                        self.tokens.push_back(MappingEnd as usize)
                    }
                    Some(BlockMap(_, _) | BlockMapExp(_, _)) => {
                        self.tokens.push_back(MappingEnd as usize)
                    }
                    _ => {}
                }
            }
        } else {
            self.tokens.push_back(ErrorToken as usize);
            self.errors.push(ErrorType::ExpectedIndent {
                actual: init_indent,
                expected: expected_indent,
            });
        }
    }

    fn get_plain_scalar_flow<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        start_indent: usize,
        init_indent: usize,
    ) {
        let mut is_multiline = false;
        let mut ends_with = b'\0';
        let scalar = self.get_plain_scalar(
            reader,
            start_indent,
            init_indent,
            &mut is_multiline,
            &mut ends_with,
        );
        self.tokens.extend(scalar);

        match self.curr_state() {
            FlowMap(indent, BeforeKey) => self.set_curr_state(FlowMap(indent, BeforeColon)),
            FlowMap(indent, BeforeColon) => self.set_curr_state(FlowMap(indent, BeforeKey)),
            _ => {}
        }
    }

    fn get_plain_scalar<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        start_indent: usize,
        init_indent: usize,
        is_multiline: &mut bool,
        ends_with: &mut u8,
    ) -> Vec<usize> {
        let mut curr_indent = self.curr_state().get_block_indent(reader.col());
        let mut tokens = vec![ScalarPlain as usize];
        let mut offset_start = None;
        let in_flow_collection = self.curr_state().in_flow_collection();
        let mut had_comment = false;
        let mut num_newlines = 0;

        while !reader.eof() {
            // In explicit key mapping change in indentation is always an error
            if self.curr_state().wrong_exp_indent(curr_indent) && curr_indent != init_indent {
                tokens.push(ErrorToken as usize);
                self.errors.push(ExpectedIndent {
                    actual: curr_indent,
                    expected: init_indent,
                });
                break;
            } else if curr_indent < init_indent {
                // if plain scalar is less indented than previous
                // It can be
                // a) Part of BlockMap so we must break
                // b) An error outside of block map
                if !matches!(self.curr_state(), BlockMap(_, _) | BlockMapExp(_, _)) {
                    reader.read_line();
                    tokens.push(ErrorToken as usize);
                    self.errors.push(ExpectedIndent {
                        actual: curr_indent,
                        expected: start_indent,
                    });
                }
                break;
            }

            let (start, end, error) =
                reader.read_plain_one_line(offset_start, &mut had_comment, in_flow_collection);

            if let Some(err) = error {
                tokens.push(ErrorToken as usize);
                self.errors.push(err);
            };

            match num_newlines {
                x if x == 1 => {
                    *is_multiline = true;
                    tokens.push(NewLine as usize);
                    tokens.push(0);
                }
                x if x > 1 => {
                    *is_multiline = true;
                    tokens.push(NewLine as usize);
                    tokens.push(x as usize - 1);
                }
                _ => {}
            }

            tokens.push(start);
            tokens.push(end);

            reader.skip_space_tab(true);

            if reader.peek_byte().map_or(false, is_newline) {
                let folded_newline = reader.skip_separation_spaces(false);
                if reader.col() >= self.curr_state().indent(0) as usize {
                    num_newlines = folded_newline as u32;
                }
                curr_indent = reader.col();
                *ends_with = u8::min(*ends_with, b'\n')
            }

            let chr = reader.peek_byte_at(0).unwrap_or(b'\0');

            if chr == b'-'
                && matches!(self.curr_state(), BlockSeq(ind) if reader.col() > ind as usize)
            {
                offset_start = Some(reader.pos());
            } else if (in_flow_collection && is_flow_indicator(chr)) || chr == b':' || chr == b'-' {
                *ends_with = u8::min(*ends_with, chr);
                break;
            } else if matches!(
                self.curr_state(), BlockMap(ind,_) | BlockMapExp(ind, _) if ind as usize == curr_indent
            ) {
                break;
            }
        }
        tokens.push(ScalarEnd as usize);
        tokens
    }

    fn fetch_explicit_map<B, R: Reader<B>>(&mut self, reader: &mut R) {
        if !self.is_map() {
            self.tokens.push_back(MappingStart as usize);
        }

        if !reader.peek_byte2().map_or(false, is_white_tab_or_break) {
            let scalar =
                self.get_plain_scalar(reader, reader.col(), reader.col(), &mut true, &mut b'\0');
            self.tokens.extend(scalar);
        } else {
            reader.consume_bytes(1);
            reader.skip_space_tab(true);
        }
    }

    #[inline]
    fn is_prev_sequence(&self) -> bool {
        match self.stack.iter().nth_back(1) {
            Some(FlowSeq(_, _)) => true,
            _ => false,
        }
    }

    #[inline]
    fn is_map(&self) -> bool {
        match self.curr_state() {
            FlowMap(_, _) | FlowKeyExp(_, _) => true,
            _ => false,
        }
    }
}

const DOC_END: usize = usize::MAX;
const DOC_START: usize = usize::MAX - 1;
const MAP_END: usize = usize::MAX - 2;
const MAP_START: usize = usize::MAX - 3;
const SEQ_END: usize = usize::MAX - 4;
const SEQ_START: usize = usize::MAX - 5;
const SCALAR_PLAIN: usize = usize::MAX - 7;
const SCALAR_FOLD: usize = usize::MAX - 8;
const SCALAR_LIT: usize = usize::MAX - 9;
const SCALAR_QUOTE: usize = usize::MAX - 10;
const SCALAR_DQUOTE: usize = usize::MAX - 11;
const SCALAR_END: usize = usize::MAX - 12;
const TAG_START: usize = usize::MAX - 13;
const ANCHOR: usize = usize::MAX - 14;
const ALIAS: usize = usize::MAX - 15;
const DIR_RES: usize = usize::MAX - 16;
const DIR_TAG: usize = usize::MAX - 17;
const DIR_YAML: usize = usize::MAX - 18;
const ERROR: usize = usize::MAX - 19;
const NEWLINE: usize = usize::MAX - 20;

#[repr(usize)]
#[derive(Copy, Clone, Eq, PartialEq)]
#[allow(clippy::enum_clike_unportable_variant)] //false positive see https://github.com/rust-lang/rust-clippy/issues/8043
///
/// [LexerToken] used to Lex YAML files
pub enum LexerToken {
    /// Denotes that value is a [usize] less than [NewLine] and thus its meaning decided by previous Tokens
    /// usually marks a start/end token.
    Mark,
    /// Denotes a newline and must be followed by a [Mark]. If next Mark is 0, it's space otherwise it's a `n`
    /// number of newlines `\n`
    NewLine = NEWLINE,
    /// Error in stream, check [Lexer.errors] for details
    ErrorToken = ERROR,
    /// Directive Tag denoted by `%TAG` and followed by two [Mark] tokens
    DirectiveTag = DIR_TAG,
    /// Directive Tag denoted by `@value` and followed by two [Mark] tokens
    DirectiveReserved = DIR_RES,
    /// YAML directive showing minor/major version of e.g.
    /// ```yaml
    ///     %YAML 1.2
    /// ```
    DirectiveYaml = DIR_YAML,
    /// Plain Scalar that's neither quoted or literal or folded
    /// ```yaml
    ///     example: plain_scalar
    /// ```
    ScalarPlain = SCALAR_PLAIN,
    /// Helper token to end token
    ScalarEnd = SCALAR_END,
    /// Folded scalar token
    /// ```yaml
    ///     example: >
    ///         folded_scalar
    /// ```
    ScalarFold = SCALAR_FOLD,
    /// Literal scalar token
    /// ```yaml
    ///     example: |
    ///         literal_scalar
    /// ```
    ScalarLit = SCALAR_LIT,
    /// Single quoted scalar
    /// ```yaml
    ///     example: 'single quote scalar'
    /// ```
    ScalarSingleQuote = SCALAR_QUOTE,
    /// Double quoted scalar
    /// ```yaml
    ///     example: "double quote scalar"
    /// ```
    ScalarDoubleQuote = SCALAR_DQUOTE,
    /// Element with alternative name e.g. `&foo [x,y]`
    AliasToken = ALIAS,
    /// Reference to an element with alternative name e.g. `*foo`
    AnchorToken = ANCHOR,
    TagStart = TAG_START,
    /// Start of a sequence token, e.g. `[` in
    /// ```yaml
    ///  [a, b, c]
    /// #^-- start of sequence
    /// ```
    SequenceStart = SEQ_START,
    /// End of a sequence token, e.g. `]` in
    /// ```yaml
    ///  [a, b, c]
    /// #        ^-- end of sequence
    /// ```
    SequenceEnd = SEQ_END,
    /// Start of a map  token, e.g. `{` in
    /// ```yaml
    ///  { a: b,}
    /// #^-- start of mapping
    /// ```
    MappingStart = MAP_START,
    /// End of a map  token, e.g. `}` in
    /// ```yaml
    ///  { a: b}
    /// #      ^-- start of mapping
    /// ```
    MappingEnd = MAP_END,
    /// Start of document implicit or otherwise
    DocumentStart = DOC_START,
    /// End of document implicit or otherwise
    DocumentEnd = DOC_END,
}

impl LexerToken {
    ///
    /// This method transforms a [LexerToken] into a [DirectiveType]
    ///
    /// It's UB to call on any [LexexToken] that isn't [DirectiveTag], [DirectiveYaml], or  [DirectiveReserved].
    #[inline(always)]
    pub(crate) unsafe fn to_yaml_directive(self) -> DirectiveType {
        match self {
            DirectiveTag => DirectiveType::Tag,
            DirectiveYaml => DirectiveType::Yaml,
            DirectiveReserved => DirectiveType::Reserved,
            _ => unreachable_unchecked(),
        }
    }

    ///
    /// This method transforms a [LexerToken] into a [ScalarType]
    ///
    /// It's UB to call on any [LexexToken] that isn't [ScalarPlain], [Mark], [ScalarFold], [ScalarLit],
    /// [ScalarSingleQuote], [ScalarDoubleQuote].
    #[inline(always)]
    pub(crate) unsafe fn to_scalar(self) -> ScalarType {
        match self {
            ScalarPlain | Mark => ScalarType::Plain,
            ScalarFold => ScalarType::Folded,
            ScalarLit => ScalarType::Literal,
            ScalarSingleQuote => ScalarType::SingleQuote,
            ScalarDoubleQuote => ScalarType::DoubleQuote,
            _ => unreachable_unchecked(),
        }
    }
}

impl From<usize> for LexerToken {
    fn from(value: usize) -> Self {
        pub use LexerToken::*;

        match value {
            DOC_END => DocumentEnd,
            DOC_START => DocumentStart,
            MAP_END => MappingEnd,
            MAP_START => MappingStart,
            SEQ_END => SequenceEnd,
            SEQ_START => SequenceStart,
            SCALAR_PLAIN => ScalarPlain,
            SCALAR_END => ScalarEnd,
            SCALAR_FOLD => ScalarFold,
            SCALAR_LIT => ScalarLit,
            SCALAR_QUOTE => ScalarSingleQuote,
            SCALAR_DQUOTE => ScalarDoubleQuote,
            TAG_START => TagStart,
            ANCHOR => AnchorToken,
            ALIAS => AliasToken,
            DIR_RES => DirectiveReserved,
            DIR_TAG => DirectiveTag,
            DIR_YAML => DirectiveYaml,
            NEWLINE => NewLine,
            ERROR => ErrorToken,
            _ => Mark,
        }
    }
}

impl From<&usize> for LexerToken {
    fn from(value: &usize) -> Self {
        LexerToken::from(*value)
    }
}

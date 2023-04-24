#![allow(clippy::match_like_matches_macro)]

use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::fs::read;
use std::hint::unreachable_unchecked;

use ErrorType::{ExpectedIndent, ExpectedMapBlock, ImplicitKeysNeedToBeInline};
use LexerState::PreDocStart;
use SeqState::BeforeFirstElem;

use crate::tokenizer::reader::{is_white_tab_or_break, Reader};
use crate::tokenizer::spanner::LexerState::{
    AfterDocEnd, BlockMap, BlockMapExp, BlockSeq, DirectiveSection, DocBlock, FlowKeyExp, FlowMap,
    FlowSeq,
};
use crate::tokenizer::spanner::LexerToken::*;
use crate::tokenizer::spanner::MapState::{AfterColon, BeforeColon, BeforeKey};
use crate::tokenizer::spanner::SeqState::{BeforeElem, InSeq};
use crate::tokenizer::ErrorType;
use crate::tokenizer::ErrorType::UnexpectedSymbol;

use super::iterator::{DirectiveType, ScalarType};
use super::reader::{is_flow_indicator, is_newline};

#[derive(Clone, Default)]
pub struct Lexer {
    pub stream_end: bool,
    pub(crate) tokens: VecDeque<usize>,
    pub(crate) errors: Vec<ErrorType>,
    pub(crate) tags: HashMap<Vec<u8>, (usize, usize)>,
    stack: Vec<LexerState>,
    col_start: Option<usize>,
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
    BeforeFirstElem,
    #[default]
    BeforeElem,
    InSeq,
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum LexerState {
    #[default]
    PreDocStart,
    DirectiveSection,
    DocBlock,
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
    pub(crate) fn indent(&self) -> u32 {
        match self {
            FlowKeyExp(ind, _)
            | FlowMap(ind, _)
            | FlowSeq(ind, _)
            | BlockSeq(ind)
            | BlockMap(ind, _)
            | BlockMapExp(ind, _) => *ind,
            PreDocStart | AfterDocEnd | DirectiveSection | DocBlock => 0,
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

    fn get_map(&self, start_scalar: usize) -> LexerState {
        match *self {
            FlowSeq(indent, _) | FlowMap(indent, _) | FlowKeyExp(indent, _) => {
                FlowMap(indent + 1, BeforeColon)
            }
            BlockSeq(_) | BlockMap(_, _) | BlockMapExp(_, _) | DocBlock => {
                BlockMap(start_scalar as u32, BeforeColon)
            }
            _ => panic!("Unexpected state {:?}", self),
        }
    }
}

impl Lexer {
    pub const fn get_default_namespace(namespace: &[u8]) -> Option<Cow<'static, [u8]>> {
        match namespace {
            b"!!str" => Some(Cow::Borrowed(b"tag:yaml.org,2002:str")),
            b"!!int" => Some(Cow::Borrowed(b"tag:yaml.org,2002:int")),
            b"!!null" => Some(Cow::Borrowed(b"tag:yaml.org,2002:null")),
            b"!!bool" => Some(Cow::Borrowed(b"tag:yaml.org,2002:bool")),
            b"!!float" => Some(Cow::Borrowed(b"tag:yaml.org,2002:float")),
            b"!!map" => Some(Cow::Borrowed(b"tag:yaml.org,2002:map")),
            b"!!seq" => Some(Cow::Borrowed(b"tag:yaml.org,2002:seq")),
            b"!!set" => Some(Cow::Borrowed(b"tag:yaml.org,2002:set")),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn curr_state(&self) -> LexerState {
        *self.stack.last().unwrap_or(&LexerState::default())
    }

    #[inline(always)]
    pub fn set_curr_state(&mut self, state: LexerState) {
        match self.stack.last_mut() {
            Some(x) => *x = state,
            None => {
                self.stack.push(state);
            }
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
        let curr_state = self.curr_state();
        match curr_state {
            PreDocStart => {
                if reader.peek_byte_is(b'%') {
                    self.stack.push(DirectiveSection);
                    return;
                } else if reader.peek_byte_is(b'#') {
                    reader.read_line();
                } else if reader.try_read_slice_exact("---") {
                    self.tokens.push_back(DOC_START_EXP);
                    self.stack.push(DocBlock);
                } else {
                    self.tokens.push_back(DOC_START);
                    self.stack.push(DocBlock);
                }
                return;
            }
            DirectiveSection => {
                if !reader.try_read_yaml_directive(&mut self.tokens) {
                    if reader.try_read_slice_exact("---") {
                        self.tokens.push_back(DOC_START_EXP);
                        self.set_curr_state(DocBlock);
                        return;
                    } else if reader.peek_byte_is(b'#') {
                        reader.read_line();
                    }
                } else if reader.peek_byte_is(b'#') {
                    reader.read_line();
                }
            }
            DocBlock | BlockMap(_, _) | BlockMapExp(_, _) => {
                match reader.peek_byte() {
                    Some(b'{') => self.read_flow_map(reader, curr_state.indent() as usize),
                    Some(b'[') => self.read_flow_seq(reader, curr_state.indent() as usize),
                    Some(b'&') => {
                        self.parse_anchor(reader);
                    }
                    Some(b'*') => {
                        let tok = reader.consume_anchor_alias(AnchorToken);
                        reader.skip_separation_spaces(true);

                        let next_is_colon = reader.peek_byte_is(b':')
                            || matches!(curr_state, BlockMap(_, _) | BlockMapExp(_, _));
                        let scalar_start = reader.col();
                        if next_is_colon {
                            self.process_map(scalar_start, false, b':');
                        } else {
                            self.set_next_map_state();
                        }
                        self.tokens.extend(tok);
                    }
                    Some(b':') if reader.peek_byte2().map_or(true, is_white_tab_or_break) => {
                        self.process_colon(reader, curr_state);
                    }
                    Some(b'-') if reader.peek_byte2().map_or(false, is_white_tab_or_break) => {
                        self.process_seq(reader, curr_state);
                    }
                    Some(b'?') if reader.peek_byte2().map_or(false, is_white_tab_or_break) => {
                        self.fetch_exp_block_map_key(reader)
                    }
                    Some(b'!') => self.fetch_tag(reader),
                    Some(b'|') => {
                        reader.read_block_scalar(
                            true,
                            &self.curr_state(),
                            &mut self.tokens,
                            &mut self.errors,
                        );
                        self.set_next_map_state();
                    }

                    Some(b'>') => {
                        reader.read_block_scalar(
                            false,
                            &self.curr_state(),
                            &mut self.tokens,
                            &mut self.errors,
                        );
                        self.set_next_map_state();
                    }
                    Some(b'\'') => {
                        self.update_col(reader);
                        self.set_next_map_state();
                        self.process_quote(reader);
                    }
                    Some(b'"') => {
                        self.update_col(reader);
                        self.set_next_map_state();
                        self.process_double_quote(reader);
                    }
                    Some(b'#') => {
                        // comment
                        reader.read_line();
                    }
                    Some(peek) => self.fetch_plain_scalar_block(reader, peek, curr_state),
                    None => self.stream_end = true,
                }
            }
            BlockSeq(indent) => {
                match reader.peek_byte() {
                    Some(b'{') => self.read_flow_map(reader, indent as usize),
                    Some(b'[') => self.read_flow_seq(reader, indent as usize),
                    Some(b'&') => self.tokens.extend(reader.consume_anchor_alias(AnchorToken)),
                    Some(b'*') => self.tokens.extend(reader.consume_anchor_alias(AliasToken)),
                    Some(b'-') if reader.peek_byte2().map_or(false, is_white_tab_or_break) => {
                        self.process_seq(reader, curr_state);
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
                    Some(b'\'') => self.process_quote(reader),
                    Some(b'"') => self.process_double_quote(reader),
                    Some(b'#') => {
                        // comment
                        reader.read_line();
                    }
                    Some(peek_chr) => self.fetch_plain_scalar_block(reader, peek_chr, curr_state),
                    None => self.stream_end = true,
                }
            }
            FlowSeq(indent, seq_state) => self.parse_flow_seq(reader, indent, seq_state),
            FlowMap(indent, _) | FlowKeyExp(indent, _) => self.parse_flow_map(reader, indent),
            AfterDocEnd => {
                if reader.eof() {
                    self.tokens.push_back(DOC_END);
                } else if reader.try_read_slice_exact("...") {
                    self.tokens.push_back(DOC_END_EXP);
                } else {
                    let chr = reader.peek_byte().unwrap_or(b'\0');
                    reader.read_line();
                    self.tokens.push_back(DOC_END);
                    self.push_error(UnexpectedSymbol(chr as char));
                }
                self.set_curr_state(PreDocStart);
            }
        }

        if reader.eof() {
            self.stream_end = true;
            for state in self.stack.iter().rev() {
                let x = match *state {
                    BlockSeq(_) => SequenceEnd,
                    BlockMapExp(_, AfterColon) | BlockMap(_, AfterColon) => {
                        self.tokens.push_back(SCALAR_PLAIN);
                        self.tokens.push_back(SCALAR_END);
                        MappingEnd
                    }
                    BlockMapExp(_, _) | BlockMap(_, _) | FlowMap(_, _) => MappingEnd,
                    DirectiveSection => {
                        self.errors.push(ErrorType::DirectiveEndMark);
                        ErrorToken
                    }
                    DocBlock | AfterDocEnd => DocumentEnd,
                    _ => continue,
                };
                self.tokens.push_back(x as usize);
            }
        }
    }

    #[inline(always)]
    fn push_error(&mut self, error: ErrorType) {
        self.tokens.push_back(ERROR_TOKEN);
        self.errors.push(error);
    }

    fn parse_anchor<B, R: Reader<B>>(&mut self, reader: &mut R) {
        self.tokens.extend(reader.consume_anchor_alias(AnchorToken));
    }

    fn parse_flow_seq<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        indent: u32,
        seq_state: SeqState,
    ) {
        match reader.peek_byte() {
            Some(b'&') => self.parse_anchor(reader),
            Some(b'*') => self.tokens.extend(reader.consume_anchor_alias(AliasToken)),
            Some(b'[') => self.read_flow_seq(reader, (indent + 1) as usize),
            Some(b'{') => self.read_flow_map(reader, (indent + 1) as usize),
            Some(b']') => {}
            Some(b'-') if seq_state == BeforeFirstElem => {
                reader.consume_bytes(1);
                self.push_error(UnexpectedSymbol('-'));
            }
            Some(b':') if seq_state != InSeq => {
                self.tokens.push_back(MAP_START);
                self.push_empty_token();
                self.set_curr_state(FlowSeq(indent, InSeq));
                let state = FlowMap(indent + 1, AfterColon);
                self.stack.push(state);
            }
            Some(b'}') => {
                reader.consume_bytes(1);
                self.push_error(UnexpectedSymbol('}'));
            }
            Some(b',') => {
                reader.consume_bytes(1);
                self.set_curr_state(FlowSeq(indent, BeforeElem));
            }
            Some(b'\'') => self.process_quote(reader),
            Some(b'"') => self.process_double_quote(reader),
            Some(b'?') => self.fetch_explicit_map(reader),
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

    fn parse_flow_map<B, R: Reader<B>>(&mut self, reader: &mut R, indent: u32) {
        match reader.peek_byte() {
            Some(b'&') => self.tokens.extend(reader.consume_anchor_alias(AnchorToken)),
            Some(b'*') => self.tokens.extend(reader.consume_anchor_alias(AliasToken)),
            Some(b'[') => {
                self.set_next_map_state();
                self.read_flow_seq(reader, (indent + 1) as usize);
            }
            Some(b'{') => {
                self.set_next_map_state();
                self.read_flow_map(reader, (indent + 1) as usize)
            }
            Some(b'}') => {
                reader.consume_bytes(1);
                if matches!(self.curr_state(), FlowMap(_, BeforeColon)) {
                    self.push_empty_token();
                }
                self.tokens.push_back(MAP_END);
                self.pop_state();
            }
            Some(b':') => {
                reader.consume_bytes(1);
                let curr_state = self.curr_state();

                if matches!(curr_state, FlowMap(_, BeforeKey)) {
                    self.push_empty_token();
                    self.set_next_map_state();
                } else if matches!(curr_state, FlowMap(_, BeforeColon) | FlowKeyExp(_, _)) {
                    self.set_next_map_state();
                    self.tokens.push_back(SCALAR_END);
                }
            }
            Some(b']') => {
                if self.is_prev_sequence() {
                    if self.is_unfinished() {
                        self.push_empty_token();
                    }
                    self.tokens.push_back(MAP_END);
                    self.pop_state();
                } else {
                    reader.consume_bytes(1);
                    self.push_error(UnexpectedSymbol(']'));
                }
            }
            Some(b'?') => self.fetch_explicit_map(reader),
            Some(b',') => {
                reader.consume_bytes(1);
                if self.is_prev_sequence() {
                    self.tokens.push_back(MAP_END);
                    self.pop_state();
                }
            }
            Some(b'\'') => self.process_quote(reader),
            Some(b'"') => self.process_double_quote(reader),
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

    fn unwind_map(&mut self, curr_state: LexerState, scalar_start: usize) {
        if let Some(unwind) = self.find_matching_state(
            scalar_start,
            |state, indent| matches!(state, BlockMap(ind, _) | BlockMapExp(ind, _) if ind as usize == indent),
        ) {
            self.pop_states(unwind);
        } else {
            self.tokens.push_back(MAP_START_BLOCK);
            self.stack.push(curr_state.get_map(scalar_start));
        }
    }

    fn process_quote<B, R: Reader<B>>(&mut self, reader: &mut R) {
        let curr_state = self.curr_state();
        let tokens = reader.read_single_quote(curr_state.is_implicit());

        reader.skip_separation_spaces(true);
        if reader.peek_byte_is(b':') {
            self.unwind_map(curr_state, self.col_start.unwrap_or(reader.col()));
            self.set_map_state(BeforeColon);
        }

        self.tokens.extend(tokens);
    }

    fn process_double_quote<B, R: Reader<B>>(&mut self, reader: &mut R) {
        let curr_state = self.curr_state();
        let tokens = reader.read_double_quote(curr_state.is_implicit());

        reader.skip_separation_spaces(true);
        if reader.peek_byte_is(b':') {
            self.unwind_map(curr_state, self.col_start.unwrap_or(reader.col()));
            self.set_map_state(BeforeColon);
        }

        self.tokens.extend(tokens);
    }

    fn read_flow_seq<B, R: Reader<B>>(&mut self, reader: &mut R, indent: usize) {
        reader.consume_bytes(1);
        let state = FlowSeq(indent as u32, BeforeFirstElem);
        self.stack.push(state);

        let pos = self.tokens.len();
        self.tokens.push_back(SEQ_START);

        while !reader.eof() {
            reader.skip_separation_spaces(true);
            let curr_state = self.curr_state();

            match curr_state {
                FlowSeq(indent, seq_state) => self.parse_flow_seq(reader, indent, seq_state),
                FlowMap(indent, _) | FlowKeyExp(indent, _) => self.parse_flow_map(reader, indent),
                _ => break,
            }

            if matches!(curr_state, FlowSeq(_, _)) && reader.peek_byte_is(b']') {
                reader.consume_bytes(1);
                self.tokens.push_back(SEQ_END);
                self.pop_state();

                if matches!(curr_state, FlowSeq(_, _))
                    && !matches!(self.curr_state(), FlowKeyExp(_, _) | FlowMap(_, _))
                {
                    reader.skip_separation_spaces(true);

                    if reader.peek_byte_is(b':') {
                        let token = if self.curr_state().in_flow_collection() {
                            MAP_START
                        } else {
                            MAP_START_BLOCK
                        };
                        self.tokens.insert(pos, token);
                        let state = FlowMap(indent as u32, AfterColon);
                        self.stack.push(state);
                    }
                }
                break;
            }
        }
    }

    fn read_flow_map<B, R: Reader<B>>(&mut self, reader: &mut R, indent: usize) {
        reader.consume_bytes(1);
        reader.skip_space_tab(true);

        if reader.peek_byte_is(b'?') {
            let state = FlowKeyExp(indent as u32, BeforeKey);
            self.stack.push(state);
        } else {
            let state = FlowMap(indent as u32, BeforeKey);
            self.stack.push(state);
        }
        self.tokens.push_back(MAP_START);
    }

    #[inline]
    fn push_empty_token(&mut self) {
        self.tokens.push_back(SCALAR_PLAIN);
        self.tokens.push_back(SCALAR_END);
    }

    #[inline]
    fn pop_state(&mut self) -> Option<LexerState> {
        let pop_state = self.stack.pop();
        if let Some(state) = self.stack.last_mut() {
            if state == &DocBlock {
                *state = AfterDocEnd;
            }
        };
        pop_state
    }

    fn fetch_exp_block_map_key<B, R: Reader<B>>(&mut self, reader: &mut R) {
        let indent = reader.col();
        reader.consume_bytes(1);
        reader.skip_space_tab(true);
        let state = BlockMapExp(indent as u32, BeforeKey);
        self.stack.push(state);
        self.tokens.push_back(MAP_START_BLOCK);
    }

    fn fetch_tag<B, R: Reader<B>>(&mut self, reader: &mut R) {
        pub use LexerToken::*;
        let start = reader.pos();
        reader.consume_bytes(1);
        if let Ok((mid, end)) = reader.read_tag() {
            self.tokens.push_back(TAG_START);
            self.tokens.push_back(start);
            self.tokens.push_back(mid);
            self.tokens.push_back(end);
            reader.consume_bytes(end - start);
        }
    }
    fn fetch_plain_scalar_block<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        peek_chr: u8,
        curr_state: LexerState,
    ) {
        if peek_chr == b']' || peek_chr == b'}' && peek_chr == b'@' {
            reader.consume_bytes(1);
            self.push_error(UnexpectedSymbol(peek_chr as char));
            return;
        }
        self.update_col(reader);
        let mut is_multiline = false;
        let mut ends_with = b'\x7F';
        let state_indent = self.curr_state().indent() as usize;
        let scalar_start = match curr_state {
            BlockMapExp(ind, _) => ind as usize,
            _ => reader.col(),
        };
        let init_indent = match curr_state {
            BlockMapExp(ind, _) => ind as usize,
            BlockSeq(ind) => ind as usize,
            _ => reader.col(),
        };
        let scalar_tokens = self.get_plain_scalar(
            reader,
            state_indent,
            init_indent,
            &mut is_multiline,
            &mut ends_with,
        );
        let chr = reader.peek_byte().unwrap_or(b'\0');
        if chr == b':' || matches!(curr_state, BlockMap(_, _) | BlockMapExp(_, _)) {
            self.process_map(scalar_start, is_multiline, ends_with);
        } else {
            self.set_next_map_state();
        }
        self.tokens.extend(scalar_tokens);
    }

    fn process_colon<B, R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        let indent = curr_state.indent() as usize;
        let colon_pos = reader.col();
        let col = self.col_start.unwrap_or(colon_pos);
        reader.consume_bytes(1);

        if colon_pos == 0 && curr_state == DocBlock {
            let state = BlockMap(0, AfterColon);
            self.stack.push(state);
            self.tokens.push_back(MAP_START_BLOCK);
            self.push_empty_token();
        } else if colon_pos == 0 && matches!(curr_state, BlockMap(0, BeforeKey)) {
            self.push_empty_token();
            self.set_map_state(AfterColon);
        } else if matches!(curr_state, BlockMapExp(_, _) if colon_pos != indent ) {
            self.push_error(ExpectedIndent {
                actual: col,
                expected: indent,
            });
            self.set_next_map_state();
        } else if matches!(curr_state, BlockMap(ind, BeforeColon) if col == ind as usize)
            || matches!(curr_state, BlockMapExp(ind, _) if colon_pos == ind as usize)
            || matches!(curr_state, BlockMap(_, _) if col > indent)
        {
            self.set_next_map_state();
        } else {
            if let Some(unwind) = self.find_matching_state(
                reader.col(),
                |state, indent| matches!(state, BlockMap(ind, _) | BlockMapExp(ind, _) if ind as usize == indent),
            ) {
                self.pop_states(unwind);
            } else {
                self.push_error(ExpectedIndent {
                    actual: reader.col(),
                    expected: indent,
                });
            }
            self.set_next_map_state();
        }
        self.reset_col();
    }

    fn process_map(&mut self, scalar_start: usize, is_multiline: bool, ends_with: u8) {
        match self.curr_state() {
            BlockMap(indent, BeforeKey) if scalar_start == indent as usize => {
                self.set_next_map_state();
                if ends_with != b':' {
                    self.push_error(ExpectedMapBlock);
                }
            }
            BlockMapExp(indent, _) if scalar_start == indent as usize => {
                self.set_next_map_state();
            }
            BlockMap(indent, BeforeColon) | BlockMapExp(indent, _)
                if scalar_start > indent as usize =>
            {
                self.set_next_map_state();
                let state = BlockMap(scalar_start as u32, BeforeKey);
                self.stack.push(state);
                self.tokens.push_back(MAP_START_BLOCK);
            }
            BlockMap(indent, AfterColon) | BlockMapExp(indent, _)
                if indent as usize == scalar_start =>
            {
                self.push_empty_token();
                self.set_map_state(BeforeColon);
            }
            BlockMap(indent, AfterColon) | BlockMapExp(indent, _)
                if scalar_start > indent as usize && ends_with == b':' =>
            {
                self.set_next_map_state();
                self.tokens.push_back(MAP_START_BLOCK);
                let state = BlockMap(scalar_start as u32, BeforeColon);
                self.stack.push(state);
            }
            BlockMap(indent, AfterColon) | BlockMapExp(indent, _)
                if scalar_start > indent as usize =>
            {
                self.set_next_map_state();
            }
            state if !matches!(state, BlockMap(_, _) | BlockMapExp(_, _)) => {
                let state1 = BlockMap(scalar_start as u32, BeforeColon);
                self.stack.push(state1);
                if is_multiline {
                    self.push_error(ImplicitKeysNeedToBeInline);
                }
                self.tokens.push_back(MAP_START_BLOCK);
            }
            _ => {
                if let Some(unwind) = self.find_matching_state(
                    scalar_start,
                    |state, indent| matches!(state, BlockMap(ind, _) | BlockMapExp(ind, _) if ind as usize == indent),
                ) {
                    self.pop_states(unwind);
                } else {
                    self.push_error(ExpectedIndent {
                        actual: scalar_start,
                        expected: self.curr_state().indent() as usize,
                    });
                }
                self.set_next_map_state();
            }
        }
    }

    fn process_seq<B, R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        let indent = reader.col();
        let expected_indent = curr_state.indent() as usize;
        reader.consume_bytes(1);
        match curr_state {
            DocBlock => {
                let state = BlockSeq(indent as u32);
                self.stack.push(state);
                self.tokens.push_back(SEQ_START_BLOCK);
            }
            BlockSeq(ind) if indent > ind as usize => {
                let state = BlockSeq(indent as u32);
                self.stack.push(state);
                self.tokens.push_back(SEQ_START_BLOCK);
            }
            BlockSeq(ind) if indent == ind as usize => {}
            _ => {
                if let Some(unwind) = self.find_matching_state(
                    indent,
                    |state, indent| matches!(state, BlockSeq(ind) if ind as usize == indent),
                ) {
                    self.pop_states(unwind);
                } else {
                    self.push_error(ExpectedIndent {
                        actual: indent,
                        expected: expected_indent,
                    });
                }
            }
        }
    }

    fn find_matching_state(
        &self,
        matching_indent: usize,
        f: fn(LexerState, usize) -> bool,
    ) -> Option<usize> {
        self.stack
            .iter()
            .rposition(|state| f(*state, matching_indent))
            .map(|x| self.stack.len() - x - 1)
    }

    fn pop_states(&mut self, unwind: usize) {
        if unwind == 0 {
            return;
        }
        for _ in 0..unwind {
            match self.pop_state() {
                Some(BlockSeq(_)) => self.tokens.push_back(SEQ_END),
                Some(BlockMap(_, AfterColon) | BlockMapExp(_, AfterColon)) => {
                    self.push_empty_token();
                    self.tokens.push_back(MAP_END)
                }
                Some(BlockMap(_, _) | BlockMapExp(_, _)) => self.tokens.push_back(MAP_END),
                _ => {}
            }
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
        self.set_next_map_state();
    }

    fn get_plain_scalar<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        state_indent: usize,
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
                        expected: state_indent,
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
                if reader.col() >= self.curr_state().indent() as usize {
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
            self.tokens.push_back(MAP_START);
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
    fn update_col<B, R: Reader<B>>(&mut self, reader: &R) {
        self.col_start = Some(reader.col());
    }

    #[inline]
    fn reset_col(&mut self) {
        self.col_start = None;
    }

    #[inline]
    fn is_prev_sequence(&self) -> bool {
        match self.stack.iter().nth_back(1) {
            Some(FlowSeq(_, _)) => true,
            _ => false,
        }
    }

    #[inline]
    fn is_unfinished(&self) -> bool {
        match self.curr_state() {
            FlowMap(_, AfterColon) | FlowKeyExp(_, AfterColon) => true,
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
const DOC_END_EXP: usize = usize::MAX - 1;
const DOC_START: usize = usize::MAX - 2;
const DOC_START_EXP: usize = usize::MAX - 3;
const MAP_END: usize = usize::MAX - 4;
const MAP_START: usize = usize::MAX - 5;
const MAP_START_BLOCK: usize = usize::MAX - 6;
const SEQ_END: usize = usize::MAX - 7;
const SEQ_START: usize = usize::MAX - 8;
const SEQ_START_BLOCK: usize = usize::MAX - 9;
const SCALAR_PLAIN: usize = usize::MAX - 10;
const SCALAR_FOLD: usize = usize::MAX - 11;
const SCALAR_LIT: usize = usize::MAX - 12;
const SCALAR_QUOTE: usize = usize::MAX - 13;
const SCALAR_DQUOTE: usize = usize::MAX - 14;
const SCALAR_END: usize = usize::MAX - 15;
const TAG_START: usize = usize::MAX - 16;
const ANCHOR: usize = usize::MAX - 17;
const ALIAS: usize = usize::MAX - 18;
const DIR_RES: usize = usize::MAX - 19;
const DIR_TAG: usize = usize::MAX - 20;
const DIR_YAML: usize = usize::MAX - 21;
const ERROR_TOKEN: usize = usize::MAX - 22;
const NEWLINE: usize = usize::MAX - 32;

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
    ErrorToken = ERROR_TOKEN,
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
    AnchorToken = ANCHOR,
    /// Reference to an element with alternative name e.g. `*foo`
    AliasToken = ALIAS,
    /// Tag
    TagStart = TAG_START,
    /// Start of a sequence token, e.g. `[` in
    /// ```yaml
    ///  [a, b, c]
    /// #^-- start of sequence
    /// ```
    SequenceStart = SEQ_START,
    /// Start of a sequence token, e.g. `[` in
    /// ```yaml
    ///  [a, b, c]
    /// #^-- start of sequence
    /// ```
    SequenceStartImplicit = SEQ_START_BLOCK,
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
    /// Start of a map  token, e.g. `{` in
    /// ```yaml
    ///   [a]: 3
    /// #^-- start of mapping
    /// ```
    MappingStartImplicit = MAP_START_BLOCK,
    /// End of a map  token, e.g. `}` in
    /// ```yaml
    ///  { a: b}
    /// #      ^-- start of mapping
    /// ```
    MappingEnd = MAP_END,
    /// Start of implicit Document
    DocumentStart = DOC_START,
    /// Start of explicit Document
    DocumentStartExplicit = DOC_START_EXP,
    /// End of implicit document.
    DocumentEnd = DOC_END,
    /// End of explicit document.
    DocumentEndExplicit = DOC_END_EXP,
}

impl LexerToken {
    ///
    /// This method transforms a [LexerToken] into a [DirectiveType]
    ///
    /// It's UB to call on any [LexerToken] that isn't [DirectiveTag], [DirectiveYaml], or  [DirectiveReserved].
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
    /// It's UB to call on any [LexerToken] that isn't [ScalarPlain], [Mark], [ScalarFold], [ScalarLit],
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
            DOC_END_EXP => DocumentEndExplicit,
            DOC_START => DocumentStart,
            DOC_START_EXP => DocumentStartExplicit,
            MAP_END => MappingEnd,
            MAP_START => MappingStart,
            MAP_START_BLOCK => MappingStartImplicit,
            SEQ_START_BLOCK => SequenceStartImplicit,
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
            ERROR_TOKEN => ErrorToken,
            _ => Mark,
        }
    }
}

impl From<&usize> for LexerToken {
    fn from(value: &usize) -> Self {
        LexerToken::from(*value)
    }
}

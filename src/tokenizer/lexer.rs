#![allow(clippy::match_like_matches_macro)]

use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::hint::unreachable_unchecked;
use std::mem::take;

use ErrorType::{ExpectedIndent, ExpectedMapBlock, ImplicitKeysNeedToBeInline};
use LexerState::PreDocStart;
use SeqState::BeforeFirstElem;

use crate::tokenizer::lexer::LexerState::{
    AfterDocEnd, BlockMap, BlockMapExp, BlockSeq, DirectiveSection, DocBlock, EndOfDirective,
    FlowKeyExp, FlowMap, FlowSeq,
};
use crate::tokenizer::lexer::LexerToken::*;
use crate::tokenizer::lexer::MapState::{AfterColon, BeforeColon, BeforeKey};
use crate::tokenizer::lexer::SeqState::{BeforeElem, InSeq};
use crate::tokenizer::reader::{is_white_tab_or_break, Reader};
use crate::tokenizer::ErrorType;
use crate::tokenizer::ErrorType::UnexpectedSymbol;

use super::iterator::{DirectiveType, ScalarType};
use super::reader::{is_flow_indicator, is_newline, is_not_whitespace};

#[derive(Clone, Default)]
pub struct Lexer {
    pub stream_end: bool,
    pub(crate) tokens: VecDeque<usize>,
    pub(crate) errors: Vec<ErrorType>,
    pub(crate) tags: HashMap<Vec<u8>, (usize, usize)>,
    stack: Vec<LexerState>,
    last_block_indent: usize,
    has_tab: bool,
    prev_anchor: Option<(usize, usize)>,
    continue_processing: bool,
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
    EndOfDirective,
    DocBlock,
    // u32 is the index of the token insertion point for flow nodes
    FlowSeq(u32, SeqState),
    FlowMap(u32, MapState),
    FlowKeyExp(u32, MapState),
    // u32 is the indent of block node
    BlockSeq(u32),
    BlockMap(u32, MapState),
    BlockMapExp(u32, MapState),
    AfterDocEnd,
}

impl LexerState {
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

#[derive(Clone, Copy, Debug, PartialEq)]
enum DirectiveState {
    NoContent,
    Tag,
    Directive,
    DirectiveAndTag,
}

impl DirectiveState {
    fn add_tag(&mut self) {
        *self = match self {
            Self::NoContent => Self::Tag,
            Self::Directive => Self::DirectiveAndTag,
            _ => *self,
        }
    }

    fn add_directive(&mut self) {
        *self = match self {
            Self::NoContent => Self::Directive,
            Self::Tag => Self::DirectiveAndTag,
            _ => *self,
        }
    }
}

impl Lexer {
    pub fn fetch_next_token<B, R: Reader<B>>(&mut self, reader: &mut R) {
        self.continue_processing = true;
        let mut directive_state = DirectiveState::NoContent;

        while self.continue_processing && !reader.eof() {
            let curr_state = self.curr_state();
            if self.skip_separation_spaces(reader, true).1 && !self.has_tab {
                self.has_tab = true;
            }
            match curr_state {
                PreDocStart => self.fetch_pre_doc(reader),
                DirectiveSection => self.fetch_directive_section(reader, &mut directive_state),
                EndOfDirective => self.fetch_end_of_directive(reader, &mut directive_state),
                DocBlock | BlockMap(_, _) | BlockMapExp(_, _) => {
                    self.fetch_map_like_block(reader, curr_state)
                }
                BlockSeq(_) => self.fetch_block_seq(reader, curr_state),
                FlowSeq(_, seq_state) => self.fetch_flow_seq(reader, seq_state),
                FlowMap(_, _) | FlowKeyExp(_, _) => self.fetch_flow_map(reader, curr_state),
                AfterDocEnd => self.fetch_after_doc(reader),
            }
        }

        if reader.eof() {
            self.stream_end = true;
            self.pop_current_states();
        }
    }

    fn pop_current_states(&mut self) {
        for state in self.stack.iter().rev() {
            let token = match *state {
                BlockSeq(_) => SEQ_END,
                BlockMapExp(_, AfterColon | BeforeColon) | BlockMap(_, AfterColon) => {
                    self.tokens.push_back(SCALAR_PLAIN);
                    self.tokens.push_back(SCALAR_END);
                    MAP_END
                }
                BlockMapExp(_, _) | BlockMap(_, _) | FlowMap(_, _) => MAP_END,
                DirectiveSection => {
                    self.errors.push(ErrorType::DirectiveEndMark);
                    ERROR_TOKEN
                }
                DocBlock | AfterDocEnd => DOC_END,
                EndOfDirective => {
                    self.tokens.push_back(SCALAR_PLAIN);
                    self.tokens.push_back(SCALAR_END);
                    DOC_END
                }
                _ => continue,
            };
            self.tokens.push_back(token);
        }
    }

    fn fetch_after_doc<B, R: Reader<B>>(&mut self, reader: &mut R) {
        if reader.eof() {
            // self.tokens.push_back(DOC_END);
        } else if reader.try_read_slice_exact("...") {
            self.tokens.push_back(DOC_END_EXP);
            self.set_curr_state(PreDocStart);
        } else {
            let chr = reader.peek_byte().unwrap_or(b'\0');
            reader.read_line();
            self.tokens.push_back(DOC_END);
            self.push_error(UnexpectedSymbol(chr as char));
            self.set_curr_state(PreDocStart);
        }
        
    }

    fn fetch_block_seq<B, R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        self.continue_processing = false;
        match reader.peek_chars() {
            [b'{', ..] => self.process_flow_map_start(reader),
            [b'[', ..] => self.process_flow_seq_start(reader),
            [b'&', ..] => self.parse_anchor(reader),
            [b'*', ..] => self.parse_alias(reader),
            [b'-', x, ..] if is_white_tab_or_break(*x) => {
                self.process_block_seq(reader, curr_state);
            }
            b"---" => self.unwind_to_root_start(reader),
            b"..." => self.unwind_to_root_end(reader),
            [b'?', x, ..] if is_white_tab_or_break(*x) => {
                self.fetch_exp_block_map_key(reader, curr_state)
            }
            [b'!', ..] => self.fetch_tag(reader),
            [b'|', ..] => self.process_block_literal(reader),
            [b'>', ..] => self.process_block_folded(reader),
            [b'\'', ..] => self.process_quote(reader, curr_state),
            [b'"', ..] => self.process_double_quote_block(reader, curr_state),
            [b'#', ..] => {
                // comment
                reader.read_line();
            }
            [peek_chr, ..] => self.fetch_plain_scalar_block(reader, curr_state, *peek_chr),
            [] => self.stream_end = true,
        }
    }

    fn fetch_map_like_block<B, R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        self.continue_processing = false;
        match reader.peek_chars() {
            [b'{', ..] => self.process_flow_map_start(reader),
            [b'[', ..] => self.process_flow_seq_start(reader),
            [b'&', ..] => self.parse_anchor(reader),
            [b'*', ..] => self.parse_alias(reader),
            [b':'] => self.process_block_colon(reader, curr_state),
            [b':', peek, ..] if is_white_tab_or_break(*peek) => {
                self.process_block_colon(reader, curr_state)
            }
            [b'-', peek, ..] if is_white_tab_or_break(*peek) => {
                self.process_block_seq(reader, curr_state);
            }
            b"..." => {
                self.unwind_to_root_end(reader);
            }
            b"---" => {
                self.unwind_to_root_start(reader);
            }
            [b'?', peek, ..] if is_white_tab_or_break(*peek) => {
                self.fetch_exp_block_map_key(reader, curr_state)
            }
            [b'!', ..] => self.fetch_tag(reader),
            [b'|', ..] => {
                self.process_block_literal(reader);
                self.set_next_map_state();
            }
            [b'>', ..] => {
                self.process_block_folded(reader);
                self.set_next_map_state();
            }
            [b'\'', ..] => {
                self.set_next_map_state();
                self.process_quote(reader, curr_state);
            }
            [b'"', ..] => {
                self.set_next_map_state();
                self.process_double_quote_block(reader, curr_state);
            }
            [b'#', ..] => {
                // comment
                reader.read_line();
            }
            [peek, ..] => self.fetch_plain_scalar_block(reader, curr_state, *peek),
            _ => self.stream_end = true,
        }
    }

    fn fetch_pre_doc<B, R: Reader<B>>(&mut self, reader: &mut R) {
        if reader.peek_byte_is(b'%') {
            self.set_curr_state(DirectiveSection);
        } else if reader.peek_byte_is(b'#') {
            reader.read_line();
        } else if reader.try_read_slice_exact("---") {
            self.tokens.push_back(DOC_START_EXP);
            self.set_curr_state(DocBlock);
        } else if reader.try_read_slice_exact("...") {
            reader.skip_separation_spaces(true);
        } else if !reader.eof() {
            self.tokens.push_back(DOC_START);
            self.set_curr_state(DocBlock);
        }
    }

    fn fetch_directive_section<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        directive_state: &mut DirectiveState,
    ) {
        match reader.peek_chars() {
            [b'%', b'Y', ..] => {
                if matches!(
                    directive_state,
                    DirectiveState::NoContent | DirectiveState::Tag
                ) {
                    if self.try_read_yaml_directive(reader) {
                        directive_state.add_directive();
                    } else {
                        directive_state.add_tag();
                    }
                } else {
                    self.push_error(ErrorType::TwoDirectivesFound);
                    reader.read_line();
                    self.continue_processing = false;
                }
            }
            [b'#', ..] => {
                reader.read_line();
            }
            [b'%', ..] => self.fetch_read_tag(reader, directive_state),
            b"..." => {
                reader.consume_bytes(3);
                self.tokens.push_back(DOC_START);
                self.tokens.push_back(DOC_END_EXP);
                self.prepend_error(ErrorType::UnexpectedEndOfStream);
                self.set_curr_state(PreDocStart);
                self.continue_processing = false;
            }
            b"---" => {
                reader.consume_bytes(3);
                self.tokens.push_back(DOC_START_EXP);
                self.set_curr_state(EndOfDirective);
                self.continue_processing = true;
            }
            [x, ..] if !is_white_tab_or_break(*x) => {
                self.prepend_error(ErrorType::YamlMustHaveOnePart);
                reader.read_line();
            }
            _ => {
                self.continue_processing = false;
            }
        }
    }

    fn try_read_yaml_directive<B, R: Reader<B>>(&mut self, reader: &mut R) -> bool {
        if reader.try_read_slice_exact("%YAML") {
            reader.skip_space_tab();
            return match reader.peek_chars() {
                b"1.0" | b"1.1" | b"1.2" | b"1.3" => {
                    self.tokens.push_back(DIR_YAML);
                    self.tokens.push_back(reader.pos());
                    self.tokens.push_back(reader.consume_bytes(3));
                    let has_ws_break = reader.peek_byte().map_or(false, is_white_tab_or_break);
                    if !has_ws_break {
                        self.prepend_error(ErrorType::UnsupportedYamlVersion);
                        reader.read_line();
                    }
                    has_ws_break
                }
                b"..." | b"---" => false,
                _ => {
                    reader.read_line();
                    false
                }
            };
        } else {
            reader.read_line();
            false
        }
    }

    fn fetch_read_tag<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        directive_state: &mut DirectiveState,
    ) {
        self.continue_processing = false;
        // TODO actual tag handling
        directive_state.add_tag();
        reader.try_read_slice_exact("%TAG");
        reader.read_line();
    }

    fn fetch_end_of_directive<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        _directive_state: &mut DirectiveState,
    ) {
        let col = reader.col();
        self.continue_processing = false;
        match reader.peek_chars() {
            b"..." => {
                reader.consume_bytes(3);
                if col != 0 {
                    self.push_error(ErrorType::UnxpectedIndentDocEnd {
                        actual: col,
                        expected: 0,
                    });
                }
                self.push_empty_token();
                self.tokens.push_back(DOC_END_EXP);
            }
            [b'%', ..] => {
                self.prepend_error(ErrorType::ExpectedDocumentEndOrContents);
                self.tokens.push_back(DOC_END);
                self.set_curr_state(DirectiveSection);
            }
            [x, ..] if is_not_whitespace(x) => {
                self.set_curr_state(DocBlock);
                self.continue_processing = true;
            }
            [..] => {}
        };
    }

    fn process_block_literal<B, R: Reader<B>>(&mut self, reader: &mut R) {
        reader.read_block_scalar(
            true,
            &self.curr_state(),
            self.last_block_indent,
            &mut self.tokens,
            &mut self.errors,
        )
    }

    fn process_block_folded<B, R: Reader<B>>(&mut self, reader: &mut R) {
        reader.read_block_scalar(
            false,
            &self.curr_state(),
            self.last_block_indent,
            &mut self.tokens,
            &mut self.errors,
        )
    }

    #[inline(always)]
    fn push_error(&mut self, error: ErrorType) {
        self.tokens.push_back(ERROR_TOKEN);
        self.errors.push(error);
    }

    #[inline(always)]
    fn prepend_error(&mut self, error: ErrorType) {
        self.tokens.push_front(ERROR_TOKEN);
        self.errors.push(error);
    }

    fn parse_anchor<B, R: Reader<B>>(&mut self, reader: &mut R) {
        self.update_col(reader);
        let anchor = reader.consume_anchor_alias();

        let line = self.skip_separation_spaces(reader, true);
        match line.0 {
            0 => {
                if reader.peek_byte_is(b'*') {
                    self.push_error(ErrorType::AliasAndAnchor);
                }
                self.prev_anchor = Some(anchor);
            }
            _ => {
                self.tokens.push_back(ANCHOR);
                self.tokens.push_back(anchor.0);
                self.tokens.push_back(anchor.1);
            }
        }
    }

    fn parse_alias<B, R: Reader<B>>(&mut self, reader: &mut R) {
        let scalar_start = reader.col();
        let alias = reader.consume_anchor_alias();
        self.skip_separation_spaces(reader, true);

        let next_is_colon = reader.peek_byte_is(b':');

        if next_is_colon {
            self.process_map(scalar_start, false, b':');
        } else {
            self.set_next_map_state();
        }
        self.tokens.push_back(ALIAS);
        self.tokens.push_back(alias.0);
        self.tokens.push_back(alias.1);
    }

    fn fetch_flow_seq<B, R: Reader<B>>(&mut self, reader: &mut R, seq_state: SeqState) {
        match reader.peek_byte() {
            Some(b'&') => self.parse_anchor(reader),
            Some(b'*') => self.parse_alias(reader),
            Some(b'[') => self.process_flow_seq_start(reader),
            Some(b'{') => self.process_flow_map_start(reader),
            Some(b']') => {
                reader.consume_bytes(1);
                self.tokens.push_back(SEQ_END);
                let index = self.pop_state().map_or(0, |f| match f {
                    FlowSeq(x, _) => x.saturating_sub(1) as usize,
                    _ => 0,
                });
                // could be `[a]: b` map
                self.skip_separation_spaces(reader, false);
                let new_curr = self.curr_state();
                // TODO deal with `: `
                if reader.peek_byte_is(b':')
                    && !matches!(new_curr, FlowKeyExp(_, _) | FlowMap(_, _))
                {
                    let token = if new_curr.in_flow_collection() {
                        MAP_START
                    } else {
                        MAP_START_BLOCK
                    };
                    self.tokens.insert(index, token);
                    let state = FlowMap(self.get_token_pos(), AfterColon);
                    self.push_state(state);
                    self.continue_processing = true;
                }
            }
            Some(b'-') if seq_state == BeforeFirstElem => {
                reader.consume_bytes(1);
                self.push_error(UnexpectedSymbol('-'));
            }
            Some(b':') if seq_state != InSeq => {
                self.tokens.push_back(MAP_START);
                let indent = self.get_token_pos();
                self.push_empty_token();
                self.set_curr_state(FlowSeq(indent, InSeq));
                let indent = self.get_token_pos();
                let state = FlowMap(indent, AfterColon);
                self.push_state(state);
            }
            Some(b'}') => {
                reader.consume_bytes(1);
                self.push_error(UnexpectedSymbol('}'));
            }
            Some(b',') => {
                reader.consume_bytes(1);
                self.set_seq_state(BeforeElem);
            }
            Some(b'\'') => self.process_quote(reader, self.curr_state()),
            Some(b'"') => self.process_double_quote_flow(reader, self.curr_state()),
            Some(b'?') => self.fetch_explicit_map(reader, self.curr_state()),
            Some(b'#') => {
                // comment
                reader.read_line();
            }
            Some(_) => {
                self.get_plain_scalar_flow(reader, self.curr_state(), reader.col());
            }
            None => self.stream_end = true,
        }
    }

    fn fetch_flow_map<B, R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        self.skip_separation_spaces(reader, true);
        match reader.peek_byte() {
            Some(b'&') => self.parse_anchor(reader),
            Some(b'*') => self.parse_alias(reader),
            Some(b'[') => {
                self.set_next_map_state();
                self.process_flow_seq_start(reader);
            }
            Some(b'{') => {
                self.set_next_map_state();
                self.process_flow_map_start(reader);
            }
            Some(b'}') => {
                reader.consume_bytes(1);
                if matches!(self.curr_state(), FlowMap(_, BeforeColon)) {
                    self.push_empty_token();
                }
                self.tokens.push_back(MAP_END);
                self.pop_state();
                self.continue_processing = false;
            }
            Some(b':') => {
                reader.consume_bytes(1);
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
            Some(b'?') => self.fetch_explicit_map(reader, curr_state),
            Some(b',') => {
                reader.consume_bytes(1);
                if self.is_prev_sequence() {
                    self.tokens.push_back(MAP_END);
                    self.pop_state();
                } else if matches!(curr_state, FlowMap(_, AfterColon)) {
                    self.push_empty_token();
                    self.set_next_map_state();
                }
            }
            Some(b'\'') => {
                self.process_quote(reader, curr_state);
                self.set_next_map_state();
            }
            Some(b'"') => {
                self.process_double_quote_flow(reader, curr_state);
                self.set_next_map_state();
            }
            Some(b'#') => {
                // comment
                reader.read_line();
            }
            Some(_) => {
                self.get_plain_scalar_flow(reader, curr_state, reader.col());
            }
            None => self.stream_end = true,
        }
    }

    fn unwind_map(&mut self, curr_state: LexerState, scalar_start: usize) {
        if let Some(unwind) = self.find_matching_state(
            scalar_start,
            |state, indent| matches!(state, BlockMap(ind, _) | BlockMapExp(ind, _) if ind as usize == indent),
        ) {
            self.pop_block_states(unwind);
        } else {
            self.tokens.push_back(MAP_START_BLOCK);
            self.push_state(curr_state.get_map(scalar_start));
        }
    }

    fn process_quote<B, R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        let scalar_start = self.update_col(reader);
        let tokens = reader.read_single_quote(curr_state.is_implicit());

        self.skip_separation_spaces(reader, true);
        if reader.peek_byte_is(b':') {
            self.unwind_map(curr_state, scalar_start);
            self.set_map_state(BeforeColon);
        }
        self.emit_prev_anchor();
        self.tokens.extend(tokens);
    }

    #[inline]
    fn process_double_quote<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        curr_state: LexerState,
    ) -> (usize, Vec<usize>) {
        let scalar_start = self.update_col(reader);
        let mut is_multiline = false;
        let tokens = reader.read_double_quote(
            self.last_block_indent,
            curr_state.is_implicit(),
            &mut is_multiline,
            &mut self.errors,
        );
        self.skip_separation_spaces(reader, true);
        (scalar_start, tokens)
    }

    fn process_double_quote_flow<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        curr_state: LexerState,
    ) {
        let (_, tokens) = self.process_double_quote(reader, curr_state);

        self.emit_prev_anchor();
        self.tokens.extend(tokens);
    }

    fn process_double_quote_block<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        curr_state: LexerState,
    ) {
        let (scalar_start, tokens) = self.process_double_quote(reader, curr_state);

        if reader.peek_byte_is(b':') {
            self.unwind_map(curr_state, scalar_start);
            self.set_map_state(BeforeColon);
        }
        self.emit_prev_anchor();
        self.tokens.extend(tokens);
    }

    #[inline]
    fn emit_prev_anchor(&mut self) {
        if let Some(anchor) = take(&mut self.prev_anchor) {
            self.tokens.push_back(ANCHOR);
            self.tokens.push_back(anchor.0);
            self.tokens.push_back(anchor.1);
        };
    }

    #[inline]
    fn skip_separation_spaces<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        allow_comments: bool,
    ) -> (usize, bool) {
        let (lines, has_tab) = reader.skip_separation_spaces(allow_comments);
        if lines > 0 {
            self.reset_col();
        }
        (lines as usize, has_tab)
    }

    fn process_flow_seq_start<B, R: Reader<B>>(&mut self, reader: &mut R) {
        reader.consume_bytes(1);
        self.tokens.push_back(SEQ_START);

        let state = FlowSeq(self.get_token_pos(), BeforeFirstElem);
        self.push_state(state);

        self.continue_processing = true;
    }

    fn process_flow_map_start<B, R: Reader<B>>(&mut self, reader: &mut R) {
        reader.consume_bytes(1);
        reader.skip_space_tab();
        self.emit_prev_anchor();

        if reader.peek_byte_is(b'?') {
            let state = FlowKeyExp(self.get_token_pos(), BeforeKey);
            self.push_state(state);
        } else {
            let state = FlowMap(self.get_token_pos(), BeforeKey);
            self.push_state(state);
        }
        self.tokens.push_back(MAP_START);
    }

    #[inline]
    fn push_empty_token(&mut self) {
        self.tokens.push_back(SCALAR_PLAIN);
        self.tokens.push_back(SCALAR_END);
    }

    #[inline]
    fn get_token_pos(&self) -> u32 {
        self.tokens.len() as u32
    }

    #[inline]
    fn pop_state(&mut self) -> Option<LexerState> {
        let pop_state = self.stack.pop();
        if let Some(state) = self.stack.last_mut() {
            match state {
                BlockMap(indent, _) | BlockMapExp(indent, _) | BlockSeq(indent) => {
                    self.last_block_indent = *indent as usize;
                }
                DocBlock => {
                    *state = AfterDocEnd;
                }
                _ => {}
            }
        };
        pop_state
    }

    fn push_state(&mut self, state: LexerState) {
        match state {
            BlockMap(indent, _) | BlockMapExp(indent, _) | BlockSeq(indent) => {
                self.last_block_indent = indent as usize;
            }
            _ => {}
        }
        self.stack.push(state);
    }

    fn pop_block_states(&mut self, unwind: usize) {
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

    fn unwind_to_root_start<B, R: Reader<B>>(&mut self, reader: &mut R) {
        let pos = reader.col();
        reader.consume_bytes(3);
        self.pop_block_states(self.stack.len().saturating_sub(1));
        self.tokens.push_back(DOC_END);
        if pos != 0 {
            self.push_error(ErrorType::ExpectedIndentDocStart {
                actual: pos,
                expected: 0,
            });
        }
        self.tokens.push_back(DOC_START_EXP);
        self.set_curr_state(DocBlock);
    }

    fn unwind_to_root_end<B, R: Reader<B>>(&mut self, reader: &mut R) {
        let col = reader.col();
        reader.consume_bytes(3);
        self.pop_block_states(self.stack.len().saturating_sub(1));
        self.tokens.push_back(DOC_END_EXP);
        if col != 0 {
            self.push_error(ErrorType::UnxpectedIndentDocEnd {
                actual: col,
                expected: 0,
            });
        }
        self.set_curr_state(PreDocStart);
    }

    fn fetch_exp_block_map_key<B, R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        let indent = reader.col();
        reader.consume_bytes(1);
        reader.skip_space_tab();
        self.emit_prev_anchor();
        match curr_state {
            DocBlock => {
                let state = BlockMapExp(indent as u32, BeforeKey);
                self.push_state(state);
                self.tokens.push_back(MAP_START_BLOCK);
            }
            BlockMapExp(prev_indent, BeforeColon) if prev_indent as usize == indent => {
                self.push_empty_token();
                self.set_map_state(BeforeKey);
            }
            _ => {}
        }
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
            // Dont consume the last character it could be newline
            reader.consume_bytes(end - start - 1);
        }
    }
    fn fetch_plain_scalar_block<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        curr_state: LexerState,
        peek_chr: u8,
    ) {
        if peek_chr == b']' || peek_chr == b'}' && peek_chr == b'@' {
            reader.consume_bytes(1);
            self.push_error(UnexpectedSymbol(peek_chr as char));
            return;
        }
        let mut is_multiline = false;
        let mut ends_with = b'\x7F';
        let state_indent = self.last_block_indent;
        self.update_col(reader);
        let scalar_start = match curr_state {
            BlockMapExp(ind, _) => ind as usize,
            BlockMap(_, BeforeColon) | DocBlock => self.col_start.unwrap_or(reader.col()),
            _ => reader.col(),
        };
        let init_indent = match curr_state {
            BlockMapExp(ind, _) => ind as usize,
            BlockSeq(ind) => ind as usize,
            _ => self.col_start.unwrap_or(reader.col()),
        };
        let scalar_tokens = self.get_plain_scalar(
            reader,
            curr_state,
            state_indent,
            init_indent,
            &mut is_multiline,
            &mut ends_with,
        );
        let chr = reader.peek_byte().unwrap_or(b'\0');
        if chr == b':' || matches!(curr_state, BlockMap(_, _) | BlockMapExp(_, _)) {
            if self.has_tab {
                self.push_error(ErrorType::TabsNotAllowedAsIndentation);
            }
            self.process_map(scalar_start, is_multiline, ends_with);
        } else {
            self.set_next_map_state();
        }
        self.emit_prev_anchor();
        self.tokens.extend(scalar_tokens);
    }

    fn process_block_colon<B, R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        let indent = self.last_block_indent;
        let colon_pos = reader.col();
        let col = self.col_start.unwrap_or(colon_pos);
        reader.consume_bytes(1);

        if colon_pos == 0 && curr_state == DocBlock {
            let state = BlockMap(0, AfterColon);
            self.push_state(state);
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
                self.pop_block_states(unwind);
            } else {
                self.push_error(ExpectedIndent {
                    actual: reader.col(),
                    expected: indent,
                });
            }
            self.set_next_map_state();
        }
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
                self.push_state(state);
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
                self.push_state(state);
            }
            BlockMap(indent, AfterColon) | BlockMapExp(indent, _)
                if scalar_start > indent as usize =>
            {
                self.set_next_map_state();
            }
            state if !matches!(state, BlockMap(_, _) | BlockMapExp(_, _)) => {
                let state1 = BlockMap(scalar_start as u32, BeforeColon);
                self.push_state(state1);
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
                    self.pop_block_states(unwind);
                } else {
                    self.push_error(ExpectedIndent {
                        actual: scalar_start,
                        expected: self.last_block_indent,
                    });
                }
                self.set_next_map_state();
            }
        }
    }

    fn process_block_seq<B, R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        let indent = reader.col();
        let expected_indent = self.last_block_indent;
        reader.consume_bytes(1);
        match curr_state {
            DocBlock => {
                let state = BlockSeq(indent as u32);
                self.push_state(state);
                self.tokens.push_back(SEQ_START_BLOCK);
            }
            BlockSeq(ind) if indent > ind as usize => {
                let state = BlockSeq(indent as u32);
                self.push_state(state);
                self.tokens.push_back(SEQ_START_BLOCK);
            }
            BlockSeq(ind) if indent == ind as usize => {}
            _ => {
                if let Some(last_seq) = self.stack.iter().rposition(|x| matches!(x, BlockSeq(_))) {
                    if let Some(unwind) = self.find_matching_state(
                        indent,
                        |state, indent| matches!(state, BlockSeq(ind) if ind as usize == indent),
                    ) {
                        self.pop_block_states(unwind);
                    } else {
                        self.pop_block_states(self.stack.len() - last_seq);
                        self.push_error(ExpectedIndent {
                            actual: indent,
                            expected: expected_indent,
                        });
                    }
                } else {
                    self.set_next_map_state();
                    self.stack.push(BlockSeq(indent as u32));
                    self.tokens.push_back(SEQ_START_BLOCK);
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

    fn get_plain_scalar_flow<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        curr_state: LexerState,
        indent: usize,
    ) {
        let mut is_multiline = false;
        let mut ends_with = b'\x7F';

        self.emit_prev_anchor();
        let scalar = self.get_plain_scalar(
            reader,
            curr_state,
            indent,
            indent,
            &mut is_multiline,
            &mut ends_with,
        );
        self.tokens.extend(scalar);
        if ends_with == b':' && matches!(curr_state, FlowMap(_, BeforeKey)) {
            reader.consume_bytes(1);
            self.set_map_state(AfterColon);
        } else if ends_with == b',' && matches!(curr_state, FlowMap(_, BeforeKey)) {
            self.set_map_state(AfterColon);
        } else {
            self.set_next_map_state();
        }
    }

    fn get_plain_scalar<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        curr_state: LexerState,
        state_indent: usize,
        init_indent: usize,
        is_multiline: &mut bool,
        ends_with: &mut u8,
    ) -> Vec<usize> {
        let mut curr_indent = match curr_state {
            BlockMapExp(ind, _) => ind as usize,
            _ => reader.col(),
        };
        let mut tokens = vec![ScalarPlain as usize];
        let mut offset_start = None;
        let in_flow_collection = curr_state.in_flow_collection();
        let mut had_comment = false;
        let mut num_newlines = 0;

        while !reader.eof() {
            if curr_indent < init_indent {
                // if plain scalar is less indented than previous
                // It can be
                // a) Part of BlockMap so we must break
                // b) An error outside of block map
                if !matches!(curr_state, BlockMap(_, _) | BlockMapExp(_, _)) {
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
                    tokens.push(x - 1);
                }
                _ => {}
            }

            tokens.push(start);
            tokens.push(end);

            reader.skip_space_tab();

            if reader.peek_byte().map_or(false, is_newline) {
                let folded_newline = self.skip_separation_spaces(reader, false);
                if reader.col() >= self.last_block_indent {
                    num_newlines = folded_newline.0;
                }
                curr_indent = reader.col();
                *ends_with = u8::min(*ends_with, b'\n')
            }

            let chr = reader.peek_byte_at(0).unwrap_or(b'\0');

            if chr == b'-' && matches!(curr_state, BlockSeq(ind) if reader.col() > ind as usize) {
                offset_start = Some(reader.pos());
            } else if (in_flow_collection && is_flow_indicator(chr)) || chr == b':' || chr == b'-' {
                *ends_with = u8::min(*ends_with, chr);
                break;
            } else if matches!(
                curr_state, BlockMap(ind,_) | BlockMapExp(ind, _) if ind as usize == curr_indent
            ) {
                break;
            }
        }
        tokens.push(ScalarEnd as usize);
        tokens
    }

    fn fetch_explicit_map<B, R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        if !self.is_map() {
            self.tokens.push_back(MAP_START);
        }

        if !reader.peek_byte2().map_or(false, is_white_tab_or_break) {
            let scalar = self.get_plain_scalar(
                reader,
                curr_state,
                reader.col(),
                reader.col(),
                &mut true,
                &mut b'\0',
            );
            self.tokens.extend(scalar);
        } else {
            reader.consume_bytes(1);
            reader.skip_space_tab();
        }
    }

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
            None => self.push_state(state),
        }
    }

    #[inline]
    fn set_map_state(&mut self, map_state: MapState) {
        match self.stack.last_mut() {
            Some(FlowMap(_, state)) | Some(BlockMap(_, state)) | Some(BlockMapExp(_, state)) => {
                *state = map_state
            }
            _ => {}
        };
    }

    #[inline]
    fn set_seq_state(&mut self, seq_state: SeqState) {
        match self.stack.last_mut() {
            Some(FlowSeq(_, state)) => {
                *state = seq_state;
            }
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

    #[inline]
    fn update_col<B, R: Reader<B>>(&mut self, reader: &R) -> usize {
        match self.col_start {
            Some(x) => x,
            None => {
                let col = reader.col();
                self.col_start = Some(col);
                col
            }
        }
    }

    #[inline]
    fn reset_col(&mut self) {
        self.col_start = None;
        self.has_tab = false;
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

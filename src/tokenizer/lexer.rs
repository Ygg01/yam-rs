#![allow(clippy::match_like_matches_macro)]

use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::hint::unreachable_unchecked;
use std::mem::take;
use std::num::NonZeroU32;

use LexerState::PreDocStart;
use SeqState::BeforeFirstElem;

use crate::tokenizer::lexer::LexerState::{
    AfterDocBlock, BlockMap, BlockMapExp, BlockSeq, DirectiveSection, DocBlock, EndOfDirective,
    FlowKeyExp, FlowMap, FlowSeq, InDocEnd,
};
use crate::tokenizer::lexer::LexerToken::*;
use crate::tokenizer::lexer::MapState::{AfterColon, BeforeColon, BeforeKey};
use crate::tokenizer::lexer::SeqState::{BeforeElem, InSeq};
use crate::tokenizer::reader::{is_white_tab_or_break, Reader};
use crate::tokenizer::ErrorType::*;

use super::iterator::{DirectiveType, ScalarType};
use super::reader::{
    is_flow_indicator, is_newline, is_not_whitespace, is_valid_escape, ns_plain_safe,
};
use crate::tokenizer::ErrorType;

#[derive(Clone)]
pub struct Lexer<B = ()> {
    pub stream_end: bool,
    pub(crate) tokens: VecDeque<usize>,
    pub(crate) errors: Vec<ErrorType>,
    pub(crate) tags: HashMap<Vec<u8>, (usize, usize)>,
    buf: B,
    stack: Vec<LexerState>,
    last_block_indent: u32,
    had_anchor: bool,
    has_tab: bool,
    prev_anchor: Option<(usize, usize)>,
    continue_processing: bool,
    prev_scalar: Scalar,
    last_map_line: Option<u32>,
    col_start: Option<u32>,
}

impl<S> Lexer<S> {
    pub fn new_from_buf(src: S) -> Self {
        Lexer {
            stream_end: false,
            tokens: VecDeque::default(),
            errors: Vec::default(),
            tags: HashMap::default(),
            buf: src,
            stack: Vec::default(),
            last_block_indent: 0,
            had_anchor: false,
            has_tab: false,
            prev_anchor: None,
            continue_processing: false,
            prev_scalar: Scalar::default(),
            last_map_line: None,
            col_start: None,
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct Scalar {
    scalar_start: u32,
    is_multiline: bool,
    tokens: Vec<usize>,
}

impl Scalar {
    pub fn is_empty(&self) -> bool {
        self.tokens.len() == 0
    }
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
    AfterDocBlock,
    InDocEnd,
    // Flow nodes
    // u32 is the index of the token insertion point for flow nodes
    FlowSeq(u32, SeqState),
    FlowMap(u32, MapState),
    FlowKeyExp(u32, MapState),
    // Blocks nodes
    // u32 is the indent of block node
    DocBlock,
    BlockSeq(u32),
    BlockMap(u32, MapState),
    BlockMapExp(u32, MapState),
}

pub(crate) enum ChompIndicator {
    /// `-` final line break and any trailing empty lines are excluded from the scalar’s content
    Strip,
    ///  `` final line break character is preserved in the scalar’s content
    Clip,
    /// `+` final line break and any trailing empty lines are considered to be part of the scalar’s content
    Keep,
}

impl LexerState {
    #[inline]
    pub fn in_flow_collection(&self) -> bool {
        match &self {
            FlowKeyExp(_, _) | FlowSeq(_, _) | FlowMap(_, _) => true,
            _ => false,
        }
    }

    fn get_map(&self, scalar_start: u32) -> LexerState {
        match *self {
            BlockSeq(_) | BlockMap(_, _) | BlockMapExp(_, _) | DocBlock => {
                BlockMap(scalar_start, BeforeColon)
            }
            _ => panic!("Unexpected state {:?}", self),
        }
    }

    pub(crate) fn is_map_start(&self, scalar_start: u32) -> bool {
        match self {
            DocBlock => true,
            BlockSeq(ind) | BlockMap(ind, _) if scalar_start > *ind => true,
            _ => false,
        }
    }

    fn is_incorrectly_indented(&self, scalar_start: u32) -> bool {
        match &self {
            BlockMapExp(ind, _) => scalar_start < *ind,
            BlockMap(ind, _) | BlockSeq(ind) => scalar_start <= *ind,
            _ => false,
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

macro_rules! impl_quote {
    ($quote:ident($quote_start:expr), $trim:ident($trim_fn:ident, $lit:literal), $start:ident($quote_fn:ident) => $match_fn:ident) => {
        fn $quote<R: Reader<B>>(&mut self, reader: &mut R) -> Scalar {
            let scalar_start = self.update_col(reader);
            let start_line = reader.line();

            let mut start_str = reader.consume_bytes(1);
            let mut tokens = vec![$quote_start];
            let mut newspaces = None;
            let mut state = QuoteState::Start;

            loop {
                state = match state {
                    QuoteState::Start => {
                        self.$start(reader, &mut start_str, &mut newspaces, &mut tokens)
                    }
                    QuoteState::Trim => {
                        self.$trim(reader, &mut start_str, &mut newspaces, &mut tokens)
                    }
                    QuoteState::End | QuoteState::Error => break,
                };
            }
            tokens.push(ScalarEnd as usize);
            let is_multiline = start_line != reader.line();
            Scalar {
                scalar_start,
                is_multiline,
                tokens,
            }
        }

        fn $start<R: Reader<B>>(
            &mut self,
            reader: &mut R,
            start_str: &mut usize,
            newspaces: &mut Option<usize>,
            tokens: &mut Vec<usize>,
        ) -> QuoteState {
            if let Some(pos) = reader.$quote_fn(&mut self.buf) {
                let match_pos = reader.consume_bytes(pos);
                self.$match_fn(reader, match_pos, start_str, newspaces, tokens)
            } else if reader.eof() {
                self.prepend_error(ErrorType::UnexpectedEndOfFile);
                QuoteState::Error
            } else {
                QuoteState::Trim
            }
        }

        fn $trim<R: Reader<B>>(
            &mut self,
            reader: &mut R,
            start_str: &mut usize,
            newspaces: &mut Option<usize>,
            tokens: &mut Vec<usize>,
        ) -> QuoteState {
            if reader.col() == 0 && (matches!(reader.peek_chars(&mut self.buf), b"..." | b"---")) {
                self.errors.push(ErrorType::UnexpectedEndOfStream);
                tokens.insert(0, ErrorToken as usize);
            };
            if !matches!(self.curr_state(), DocBlock) && reader.col() <= self.last_block_indent {
                self.push_error(ErrorType::InvalidQuoteIndent { actual: reader.col(), expected: self.last_block_indent});
            }

            if let Some((match_pos, len)) = reader.$trim_fn(&mut self.buf, *start_str) {
                emit_token_mut(start_str, match_pos, newspaces, tokens);
                reader.consume_bytes(len);
            } else {
                update_newlines(reader, newspaces, start_str);
            }

            match reader.peek_byte() {
                Some(b'\n') | Some(b'\r')=> {
                    update_newlines(reader, newspaces, start_str);
                    QuoteState::Start
                }
                Some($lit) => {
                    if let Some(x) = newspaces {
                        tokens.push(NewLine as usize);
                        tokens.push(*x as usize);
                    }
                    reader.consume_bytes(1);
                    QuoteState::End
                }
                Some(_) => QuoteState::Start,
                None => {
                    self.prepend_error(ErrorType::UnexpectedEndOfFile);
                    QuoteState::Error
                }
            }
        }
    };
}

impl<B> Lexer<B> {
    pub fn fetch_next_token<R: Reader<B>>(&mut self, reader: &mut R) {
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
                    self.fetch_block_map(reader, curr_state);
                }
                BlockSeq(_) => self.fetch_block_seq(reader, curr_state),
                FlowSeq(_, seq_state) => self.fetch_flow_seq(reader, seq_state),
                FlowMap(_, _) | FlowKeyExp(_, _) => self.fetch_flow_map(reader, curr_state),
                AfterDocBlock => self.fetch_after_doc(reader),
                InDocEnd => self.fetch_end_doc(reader),
            }
        }

        if reader.eof() {
            self.stream_end = true;
            self.finish_eof();
        }
    }

    fn finish_eof(&mut self) {
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
                EndOfDirective => {
                    self.tokens.push_back(SCALAR_PLAIN);
                    self.tokens.push_back(SCALAR_END);
                    DOC_END
                }
                DocBlock | AfterDocBlock => DOC_END,
                _ => continue,
            };
            self.tokens.push_back(token);
        }
    }

    fn fetch_pre_doc<R: Reader<B>>(&mut self, reader: &mut R) {
        match reader.peek_chars(&mut self.buf) {
            [b'%', ..] => {
                self.set_curr_state(DirectiveSection, 0);
            }
            b"---" => {
                reader.consume_bytes(3);
                self.tokens.push_back(DOC_START_EXP);
                self.set_curr_state(EndOfDirective, 0);
            }
            b"..." => {
                reader.consume_bytes(3);
                reader.skip_separation_spaces(true);
                self.set_curr_state(InDocEnd, 0);
            }
            [b'#', ..] => {
                reader.read_line();
            }
            [_, ..] => {
                self.tokens.push_back(DOC_START);
                self.set_curr_state(DocBlock, 0);
            }
            [] => {}
        }
    }

    fn fetch_directive_section<R: Reader<B>>(
        &mut self,
        reader: &mut R,
        directive_state: &mut DirectiveState,
    ) {
        match reader.peek_chars(&mut self.buf) {
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
                self.set_curr_state(PreDocStart, 0);
                self.continue_processing = false;
            }
            b"---" => {
                reader.consume_bytes(3);
                self.tokens.push_back(DOC_START_EXP);
                self.set_curr_state(EndOfDirective, 0);
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

    fn try_read_yaml_directive<R: Reader<B>>(&mut self, reader: &mut R) -> bool {
        if reader.try_read_slice_exact("%YAML") {
            reader.skip_space_tab();
            return match reader.peek_chars(&mut self.buf) {
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

    fn fetch_read_tag<R: Reader<B>>(
        &mut self,
        reader: &mut R,
        directive_state: &mut DirectiveState,
    ) {
        self.continue_processing = false;
        directive_state.add_tag();
        reader.try_read_slice_exact("%TAG");
        reader.skip_space_tab();

        if let Ok(key) = reader.read_tag_handle() {
            reader.skip_space_tab();
            if let Some(val) = reader.read_tag_uri() {
                self.tags.insert(key, val);
            }
        }
    }

    fn fetch_end_of_directive<R: Reader<B>>(
        &mut self,
        reader: &mut R,
        _directive_state: &mut DirectiveState,
    ) {
        let col = reader.col();
        self.continue_processing = false;
        match reader.peek_chars(&mut self.buf) {
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
                self.set_curr_state(InDocEnd, 0);
            }
            [b'%', ..] => {
                self.prepend_error(ErrorType::ExpectedDocumentEndOrContents);
                self.tokens.push_back(DOC_END);
                self.set_curr_state(DirectiveSection, 0);
            }
            [x, ..] if is_not_whitespace(*x) => {
                self.set_curr_state(DocBlock, reader.line());
                self.continue_processing = true;
            }
            [..] => {}
        };
    }

    fn fetch_after_doc<R: Reader<B>>(&mut self, reader: &mut R) {
        let mut consume_line = false;
        match reader.peek_chars(&mut self.buf) {
            b"..." => {
                let col = reader.col();
                reader.consume_bytes(3);
                if col != 0 {
                    self.push_error(ErrorType::UnxpectedIndentDocEnd {
                        actual: col,
                        expected: 0,
                    });
                }
                self.tokens.push_back(DOC_END_EXP);
                self.set_curr_state(InDocEnd, 0);
            }
            [b'#', ..] => {
                consume_line = true;
                self.set_curr_state(PreDocStart, 0);
            }
            [chr, ..] => {
                consume_line = true;
                self.tokens.push_back(DOC_END);
                self.push_error(UnexpectedSymbol(*chr as char));
                self.set_curr_state(PreDocStart, 0);
            }
            [] => {}
        }
        if consume_line {
            reader.read_line();
        }
    }

    fn fetch_end_doc<R: Reader<B>>(&mut self, reader: &mut R) {
        reader.skip_space_tab();
        let read_line = reader.line();
        match reader.peek_byte() {
            Some(b'#') => {
                reader.read_line();
            }
            Some(b'%') => {
                self.set_curr_state(DirectiveSection, read_line);
            }
            Some(b'-') => {
                if reader.peek_chars(&mut self.buf) == b"---" {
                    reader.consume_bytes(3);
                    self.tokens.push_back(DOC_START_EXP);
                }
            }
            Some(b'.') => {
                if reader.peek_chars(&mut self.buf) == b"..." {
                    reader.consume_bytes(3);
                    self.tokens.push_back(DOC_END_EXP);
                }
            }
            Some(chr) if chr == b' ' || chr == b'\t' || chr == b'\r' || chr == b'\n' => {
                self.set_curr_state(PreDocStart, read_line);
            }
            Some(_) => {
                reader.read_line();
                self.push_error(ErrorType::ExpectedDocumentStartOrContents);
            }
            None => {
                self.stream_end = true;
            }
        }
    }

    fn fetch_block_seq<R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        self.continue_processing = false;
        match reader.peek_chars(&mut self.buf) {
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
            [b'|', ..] => self.process_block_literal(reader, curr_state, true),
            [b'>', ..] => self.process_block_literal(reader, curr_state, false),
            [b'\'', ..] => self.process_single_quote_block(reader, curr_state),
            [b'"', ..] => self.process_double_quote_block(reader, curr_state),
            [b'#', ..] => {
                // comment
                reader.read_line();
            }
            [peek_chr, ..] => self.fetch_plain_scalar_block(reader, curr_state, *peek_chr),
            [] => self.stream_end = true,
        }
    }

    fn fetch_block_map<R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        self.continue_processing = false;
        match reader.peek_chars(&mut self.buf) {
            [b'{', ..] => self.process_flow_map_start(reader),
            [b'[', ..] => self.process_flow_seq_start(reader),
            [b'&', ..] => self.parse_anchor(reader),
            [b'*', ..] => self.parse_alias(reader),
            [b':'] => self.process_colon_block(reader, curr_state),
            [b':', peek, ..] if !ns_plain_safe(*peek) => {
                self.process_colon_block(reader, curr_state)
            }
            [b'-', peek, ..] if !ns_plain_safe(*peek) => {
                self.process_block_seq(reader, curr_state);
            }
            b"..." => {
                self.unwind_to_root_end(reader);
            }
            b"---" => {
                self.unwind_to_root_start(reader);
            }
            [b'?', peek, ..] if !ns_plain_safe(*peek) => {
                self.fetch_exp_block_map_key(reader, curr_state)
            }
            [b'!', ..] => self.fetch_tag(reader),
            [b'|', ..] => {
                self.next_map_state();
                self.process_block_literal(reader, curr_state, true);
            }
            [b'>', ..] => {
                self.next_map_state();
                self.process_block_literal(reader, curr_state, false);
            }
            [b'\'', ..] => {
                self.process_single_quote_block(reader, curr_state);
            }
            [b'"', ..] => {
                self.process_double_quote_block(reader, curr_state);
            }
            [b'#', ..] => {
                // comment
                reader.read_line();
            }
            [peek, ..] => {
                self.fetch_plain_scalar_block(reader, curr_state, *peek);
            }
            _ => self.stream_end = true,
        }
    }

    fn process_block_literal<R: Reader<B>>(
        &mut self,
        reader: &mut R,
        curr_state: LexerState,
        literal: bool,
    ) {
        let had_tab = self.has_tab;
        let scalar_line = reader.line();
        let scalar_start = reader.col();

        let tokens =
            self.read_block_scalar(reader, literal, &self.curr_state(), self.last_block_indent);
        let is_multiline = reader.line() != scalar_line;
        reader.skip_space_tab();

        let is_key = reader.peek_byte().map_or(false, |chr| chr == b':');

        self.process_block_scalar(
            curr_state,
            is_key,
            scalar_start,
            had_tab,
            is_multiline,
            tokens,
            scalar_line,
        );
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

    fn parse_anchor<R: Reader<B>>(&mut self, reader: &mut R) {
        self.update_col(reader);
        let anchor = reader.consume_anchor_alias();

        let line = self.skip_separation_spaces(reader, true);
        match line.0 {
            0 => {
                self.prev_anchor = Some(anchor);
            }
            _ => {
                self.tokens.push_back(ANCHOR);
                self.tokens.push_back(anchor.0);
                self.tokens.push_back(anchor.1);
                self.had_anchor = true;
            }
        }
    }

    fn parse_alias<R: Reader<B>>(&mut self, reader: &mut R) {
        let alias_start = reader.col();
        let had_tab = self.has_tab;
        let alias = reader.consume_anchor_alias();
        self.skip_separation_spaces(reader, true);

        let next_is_colon = reader.peek_byte_is(b':');

        self.next_map_state();
        if next_is_colon {
            self.process_block_scalar(
                self.curr_state(),
                true,
                alias_start,
                had_tab,
                false,
                vec![ALIAS, alias.0, alias.1],
                reader.line(),
            );
        } else {
            self.tokens.push_back(ALIAS);
            self.tokens.push_back(alias.0);
            self.tokens.push_back(alias.1);
        }
    }

    fn process_post_seq<R: Reader<B>>(&mut self, reader: &mut R, index: u32, in_flow: bool) {
        // could be `[a]: b` map
        if reader.peek_byte_is(b':') {
            if !self.is_flow_map() {
                let token = if in_flow { MAP_START } else { MAP_START_BLOCK };
                self.tokens.insert(index as usize, token);
                let state = FlowMap(self.get_token_pos(), AfterColon);
                self.push_state(state, reader.line());
                self.continue_processing = true;
            }
            reader.consume_bytes(1);
        }
    }

    fn fetch_flow_seq<R: Reader<B>>(&mut self, reader: &mut R, seq_state: SeqState) {
        match reader.peek_chars(&mut self.buf) {
            [b'&', ..] => self.parse_anchor(reader),
            [b'*', ..] => self.parse_alias(reader),
            [b'[', ..] => self.process_flow_seq_start(reader),
            [b'{', ..] => self.process_flow_map_start(reader),
            [b']', ..] => {
                reader.consume_bytes(1);
                self.tokens.push_back(SEQ_END);
                let index = self.pop_state().map_or(0, |f| match f {
                    FlowSeq(x, _) => x,
                    _ => 0,
                });
                self.process_post_seq(reader, index, self.curr_state().in_flow_collection());
            }
            [b'-', ..] if seq_state == BeforeFirstElem => {
                reader.consume_bytes(1);
                self.push_error(UnexpectedSymbol('-'));
            }
            [b':', chr, ..] if seq_state != InSeq && !ns_plain_safe(*chr) => {
                self.tokens.push_back(MAP_START);
                let indent = self.get_token_pos();
                self.push_empty_token();
                self.set_curr_state(FlowSeq(indent, InSeq), reader.line());
                let indent = self.get_token_pos();
                let state = FlowMap(indent, AfterColon);
                self.push_state(state, reader.line());
            }
            [b'}', ..] => {
                reader.consume_bytes(1);
                self.push_error(UnexpectedSymbol('}'));
            }
            [b',', ..] => {
                reader.consume_bytes(1);
                self.set_seq_state(BeforeElem);
            }
            [b'\'', ..] => self.process_single_quote_flow(reader, self.curr_state()),
            [b'"', ..] => self.process_double_quote_flow(reader),
            [b'?', chr, ..] if !ns_plain_safe(*chr) => {
                self.fetch_explicit_map(reader, self.curr_state())
            }
            [b'#', ..] => {
                // comment
                reader.read_line();
            }
            [_, ..] => {
                self.fetch_plain_scalar_flow(reader, self.curr_state());
            }
            [] => self.stream_end = true,
        }
    }

    fn fetch_flow_map<R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        match reader.peek_chars(&mut self.buf) {
            [b'&', ..] => self.parse_anchor(reader),
            [b'*', ..] => self.parse_alias(reader),
            [b'[', ..] => {
                self.next_map_state();
                self.process_flow_seq_start(reader);
            }
            [b'{', ..] => {
                self.next_map_state();
                self.process_flow_map_start(reader);
            }
            [b'}', ..] => {
                reader.consume_bytes(1);
                if matches!(self.curr_state(), FlowMap(_, BeforeColon)) {
                    self.push_empty_token();
                }
                self.tokens.push_back(MAP_END);
                self.pop_state();
                self.continue_processing = false;
            }
            [b':', peek, ..] if !ns_plain_safe(*peek) => {
                reader.consume_bytes(1);
                self.process_colon_flow(curr_state);
            }
            [b':', peek, ..]
                if ns_plain_safe(*peek) && matches!(curr_state, FlowMap(_, BeforeColon)) =>
            {
                reader.consume_bytes(1);
                self.process_colon_flow(curr_state);
            }
            [b']', ..] => {
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
            [b'?', peek, ..] if !ns_plain_safe(*peek) => {
                self.fetch_explicit_map(reader, curr_state)
            }
            [b',', ..] => {
                reader.consume_bytes(1);
                if !self.prev_scalar.is_empty() {
                    self.emit_prev_anchor();
                    self.tokens.extend(take(&mut self.prev_scalar.tokens));
                    self.set_map_state(BeforeKey);
                } else if self.is_prev_sequence() {
                    self.tokens.push_back(MAP_END);
                    self.pop_state();
                } else if matches!(curr_state, FlowMap(_, AfterColon) | FlowMap(_, BeforeColon)) {
                    self.push_empty_token();
                    self.set_map_state(BeforeKey);
                }
            }
            [b'\'', ..] => {
                self.next_map_state();
                self.process_single_quote_flow(reader, curr_state);
            }
            [b'"', ..] => {
                self.next_map_state();
                self.process_double_quote_flow(reader);
            }
            [b'#', ..] => {
                // comment
                reader.read_line();
            }
            [_, ..] => {
                self.next_map_state();
                self.fetch_plain_scalar_flow(reader, curr_state);
            }
            [] => self.stream_end = true,
        }
    }

    fn process_colon_flow(&mut self, curr_state: LexerState) {
        if !self.prev_scalar.is_empty() {
            self.emit_prev_anchor();
            self.tokens.extend(take(&mut self.prev_scalar.tokens));
            self.set_map_state(AfterColon);
        } else if matches!(curr_state, FlowMap(_, BeforeKey)) {
            self.push_empty_token();
            self.next_map_state();
        } else if matches!(curr_state, FlowKeyExp(_, _)) {
            self.next_map_state();
            self.tokens.push_back(SCALAR_END);
        }
    }

    fn unwind_map(&mut self, curr_state: LexerState, scalar_start: u32, reader_line: u32) {
        if let Some(unwind) = self.find_matching_state(
            scalar_start,
            |state, indent| matches!(state, BlockMap(ind, _) | BlockMapExp(ind, _) if ind == indent),
        ) {
            self.pop_block_states(unwind);
        } else {
            self.tokens.push_back(MAP_START_BLOCK);
            self.push_state(curr_state.get_map(scalar_start), reader_line);
        }
    }

    fn process_single_quote_flow<R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        let scalar_start = self.update_col(reader);
        let Scalar { tokens, .. } = self.process_single_quote(reader);

        self.skip_separation_spaces(reader, true);
        if reader.peek_byte_is(b':') {
            self.unwind_map(curr_state, scalar_start, reader.line());
            self.set_map_state(BeforeColon);
        }
        self.emit_prev_anchor();
        self.tokens.extend(tokens);
    }

    impl_quote!(process_single_quote(SCALAR_QUOTE), single_quote_trim(get_single_quote_trim, b'\''), single_quote_start(get_single_quote) => single_quote_match);

    fn single_quote_match<R: Reader<B>>(
        &mut self,
        reader: &mut R,
        match_pos: usize,
        start_str: &mut usize,
        newspaces: &mut Option<usize>,
        tokens: &mut Vec<usize>,
    ) -> QuoteState {
        match reader.peek_chars(&mut self.buf) {
            [b'\'', b'\'', ..] => {
                emit_token_mut(start_str, match_pos + 1, newspaces, tokens);
                reader.consume_bytes(2);
                *start_str = reader.pos();
            }
            [b'\'', ..] => {
                emit_token_mut(start_str, match_pos, newspaces, tokens);
                reader.consume_bytes(1);
                return QuoteState::End;
            }
            _ => {}
        }
        QuoteState::Start
    }

    fn process_double_quote_flow<R: Reader<B>>(&mut self, reader: &mut R) {
        let scalar = self.process_double_quote(reader);
        reader.skip_space_tab();

        if reader.peek_byte().map_or(false, |c| c == b':') {
            self.prev_scalar = scalar;
            self.continue_processing = true;
        } else {
            self.emit_prev_anchor();
            self.tokens.extend(scalar.tokens);
        }
    }

    fn process_double_quote_block<R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        let had_tab = self.has_tab;
        let scalar_line: u32 = reader.line();
        let Scalar {
            scalar_start,
            is_multiline,
            tokens,
        } = self.process_double_quote(reader);
        reader.skip_space_tab();

        let is_key = reader.peek_byte().map_or(false, |chr| chr == b':');

        self.process_block_scalar(
            curr_state,
            is_key,
            scalar_start,
            had_tab,
            is_multiline,
            tokens,
            scalar_line,
        );
    }

    impl_quote!(process_double_quote(SCALAR_DQUOTE), double_quote_trim(get_double_quote_trim, b'"'), double_quote_start(get_double_quote) => double_quote_match);

    fn double_quote_match<R: Reader<B>>(
        &mut self,
        reader: &mut R,
        match_pos: usize,
        start_str: &mut usize,
        newspaces: &mut Option<usize>,
        tokens: &mut Vec<usize>,
    ) -> QuoteState {
        match reader.peek_chars(&mut self.buf) {
            [b'\\', b'\t', ..] => {
                emit_token_mut(start_str, match_pos, newspaces, tokens);
                emit_token_mut(&mut (match_pos + 1), match_pos + 2, newspaces, tokens);
                reader.consume_bytes(2);
                *start_str = reader.pos();
            }
            [b'\\', b't', ..] => {
                emit_token_mut(start_str, match_pos + 2, newspaces, tokens);
                reader.consume_bytes(2);
            }
            [b'\\', b'\r' | b'\n', ..] => {
                emit_token_mut(start_str, match_pos, newspaces, tokens);
                reader.consume_bytes(1);
                update_newlines(reader, &mut None, start_str);
            }
            [b'\\', b'"', ..] => {
                emit_token_mut(start_str, match_pos, newspaces, tokens);
                *start_str = reader.pos() + 1;
                reader.consume_bytes(2);
            }
            [b'\\', b'/', ..] => {
                emit_token_mut(start_str, match_pos, newspaces, tokens);
                *start_str = reader.consume_bytes(1);
            }
            [b'\\', x, ..] => {
                if is_valid_escape(*x) {
                    emit_token_mut(start_str, match_pos, newspaces, tokens);
                    reader.consume_bytes(2);
                } else {
                    tokens.insert(0, ErrorToken as usize);
                    self.errors.push(ErrorType::InvalidEscapeCharacter);
                    reader.consume_bytes(2);
                }
            }
            [b'"', ..] => {
                emit_token_mut(start_str, match_pos, newspaces, tokens);
                reader.consume_bytes(1);
                return QuoteState::End;
            }
            [b'\\'] => {
                reader.consume_bytes(1);
            }
            _ => {}
        }
        QuoteState::Start
    }

    fn process_block_scalar(
        &mut self,
        curr_state: LexerState,
        is_key: bool,
        scalar_start: u32,
        had_tab: bool,
        is_multiline: bool,
        tokens: Vec<usize>,
        scalar_line: u32,
    ) {
        if is_key {
            let is_map_start = curr_state.is_map_start(scalar_start);
            let scalar_start = scalar_start;
            self.prev_scalar = Scalar {
                scalar_start,
                is_multiline,
                tokens,
            };
            if !matches!(curr_state, BlockMapExp(_, _)) {
                if self.last_map_line == Some(scalar_line) {
                    self.push_error(ErrorType::ImplicitKeysNeedToBeInline);
                }
                if self.prev_scalar.is_multiline {
                    self.push_error(ErrorType::ImplicitKeysNeedToBeInline);
                }
            }

            self.last_map_line = Some(scalar_line);
            if is_map_start {
                self.next_map_state();
                self.continue_processing = true;
                if had_tab {
                    self.push_error(ErrorType::TabsNotAllowedAsIndentation);
                }
                self.tokens.push_back(MAP_START_BLOCK);
                self.push_state(BlockMap(scalar_start, BeforeColon), scalar_line);
            }
        } else {
            if self.last_map_line != Some(scalar_line)
                && curr_state.is_incorrectly_indented(scalar_start)
            {
                self.push_error(ErrorType::ImplicitKeysNeedToBeInline);
            }
            if self.last_map_line == Some(scalar_line) && matches!(curr_state, BlockMap(_, BeforeKey)) {
                self.push_error(ErrorType::UnexpectedScalarAtMapEnd);
            } else {
                self.next_map_state();
            }
            self.emit_prev_anchor();
            self.tokens.extend(tokens);
        }
    }

    #[inline]
    fn emit_prev_anchor(&mut self) {
        if let Some(anchor) = take(&mut self.prev_anchor) {
            if self.had_anchor {
                self.push_error(ErrorType::NodeWithTwoAnchors);
            }
            self.tokens.push_back(ANCHOR);
            self.tokens.push_back(anchor.0);
            self.tokens.push_back(anchor.1);
        };
        self.had_anchor = false;
    }

    #[inline]
    fn skip_separation_spaces<R: Reader<B>>(
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

    fn process_flow_seq_start<R: Reader<B>>(&mut self, reader: &mut R) {
        reader.consume_bytes(1);
        let pos = self.get_token_pos();
        self.had_anchor = false;
        self.emit_prev_anchor();
        self.tokens.push_back(SEQ_START);

        let state = FlowSeq(pos, BeforeFirstElem);

        self.push_state(state, reader.line());

        self.continue_processing = true;
    }

    fn process_flow_map_start<R: Reader<B>>(&mut self, reader: &mut R) {
        reader.consume_bytes(1);
        reader.skip_space_tab();
        self.emit_prev_anchor();

        if reader.peek_byte_is(b'?') {
            let state = FlowKeyExp(self.get_token_pos(), BeforeKey);
            self.push_state(state, reader.line());
        } else {
            let state = FlowMap(self.get_token_pos(), BeforeKey);
            self.push_state(state, reader.line());
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
                    self.last_block_indent = *indent;
                }
                DocBlock => {
                    *state = AfterDocBlock;
                }
                _ => {}
            }
        };
        pop_state
    }

    fn push_state(&mut self, state: LexerState, read_line: u32) {
        match state {
            BlockMap(indent, _) | BlockMapExp(indent, _) => {
                self.last_block_indent = indent;
                self.had_anchor = false;
                self.last_map_line = Some(read_line);
            }
            BlockSeq(indent) => {
                self.last_block_indent = indent;
                self.had_anchor = false;
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

    fn unwind_to_root_start<R: Reader<B>>(&mut self, reader: &mut R) {
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
        self.set_curr_state(DocBlock, reader.line());
    }

    fn unwind_to_root_end<R: Reader<B>>(&mut self, reader: &mut R) {
        let col = reader.col();
        self.pop_block_states(self.stack.len().saturating_sub(1));
        if col != 0 {
            self.push_error(ErrorType::UnxpectedIndentDocEnd {
                actual: col,
                expected: 0,
            });
        }
        self.tags.clear();
        self.set_curr_state(AfterDocBlock, reader.line());
    }

    fn fetch_exp_block_map_key<R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        let indent = reader.col();
        self.last_map_line = Some(reader.line());
        reader.consume_bytes(1);
        reader.skip_space_tab();
        self.emit_prev_anchor();
        match curr_state {
            DocBlock => {
                let state = BlockMapExp(indent, BeforeKey);
                self.push_state(state, reader.line());
                self.tokens.push_back(MAP_START_BLOCK);
            }
            BlockMapExp(prev_indent, BeforeColon) if prev_indent == indent => {
                self.push_empty_token();
                self.set_map_state(BeforeKey);
            }
            _ => {}
        }
    }

    fn fetch_tag<R: Reader<B>>(&mut self, reader: &mut R) {
        pub use LexerToken::*;
        let (err, start, mid, end) = reader.read_tag();
        if let Some(err) = err {
            self.push_error(err);
        } else {
            self.tokens.push_back(TAG_START);
            self.tokens.push_back(start);
            self.tokens.push_back(mid);
            self.tokens.push_back(end);
        }
    }
    fn fetch_plain_scalar_block<R: Reader<B>>(
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
        let mut ends_with = b'\x7F';
        self.update_col(reader);

        let curr_state = curr_state;
        let has_tab = self.has_tab;
        let scalar_line = reader.line();

        let Scalar {
            scalar_start,
            is_multiline,
            tokens,
        } = self.get_plain_scalar(reader, curr_state, &mut ends_with);

        let is_key = ends_with == b':'
            || matches!(reader.peek_chars(&mut self.buf), [b':', x, ..] if is_white_tab_or_break(*x))
                && matches!(curr_state, BlockMap(_, BeforeKey) | BlockSeq(_) | DocBlock);

        self.pop_other_states(is_key, curr_state, scalar_start);

        self.process_block_scalar(
            curr_state,
            is_key,
            scalar_start,
            has_tab,
            is_multiline,
            tokens,
            scalar_line,
        );
    }

    fn pop_other_states(&mut self, is_key: bool, curr_state: LexerState, scalar_start: u32) {
        let find_unwind = if is_key && matches!(curr_state, BlockSeq(_)) {
            self.find_matching_state(scalar_start,
                |curr_state, curr_indent| { matches!(curr_state, BlockMap(ind,_) if ind  == curr_indent)
            })
        } else if !is_key && !matches!(curr_state, BlockMap(_, _)) {
            self.find_matching_state(
                scalar_start,
                |curr_state, curr_indent| matches!(curr_state, BlockSeq(ind) if ind == curr_indent),
            )
        } else {
            None
        };
        if let Some(unwind) = find_unwind {
            self.pop_block_states(unwind);
        }
    }

    fn process_colon_block<R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        let indent = self.last_block_indent;
        let colon_pos = reader.col();
        let col = self.col_start.unwrap_or(colon_pos);
        reader.consume_bytes(1);

        if colon_pos == 0 && curr_state == DocBlock {
            let state = BlockMap(0, AfterColon);
            self.push_state(state, reader.line());
            self.tokens.push_back(MAP_START_BLOCK);
            self.push_empty_token();
        } else if matches!(curr_state, BlockMap(ind, BeforeKey) if colon_pos == ind) {
            self.push_empty_token();
            self.set_map_state(AfterColon);
        } else if matches!(curr_state, BlockMap(ind, AfterColon) if colon_pos == ind )
            && !self.prev_scalar.is_empty()
        {
            if !self.prev_scalar.is_empty() {
                self.emit_prev_anchor();
                self.set_map_state(AfterColon);
                self.tokens.extend(take(&mut self.prev_scalar.tokens));
            }
            self.push_empty_token();
        } else if matches!(curr_state, BlockMapExp(_, _) if colon_pos != indent ) {
            self.push_error(ExpectedIndent {
                actual: col,
                expected: indent,
            });
            self.next_map_state();
        } else if !self.prev_scalar.is_empty()
            && matches!(curr_state, BlockMap(ind, AfterColon) if ind == self.prev_scalar.scalar_start)
        {
            self.push_empty_token();
        } else if matches!(curr_state, BlockMap(ind, BeforeColon) if col == ind)
            || matches!(curr_state, BlockMapExp(ind, _) if colon_pos == ind )
            || matches!(curr_state, BlockMap(_, _) if col > indent)
        {
            self.next_map_state();
        } else if let Some(unwind) = self.find_matching_state(
                col,
                |state, indent| matches!(state, BlockMap(ind, _) | BlockMapExp(ind, _) if ind == indent),
            ) {
                self.pop_block_states(unwind);
            } else {
                self.push_error(ExpectedIndent {
                    actual: reader.col(),
                    expected: indent,
                });
            }

        if !self.prev_scalar.is_empty() {
            self.emit_prev_anchor();
            self.set_map_state(AfterColon);
            self.tokens.extend(take(&mut self.prev_scalar.tokens));
        }
    }

    fn process_block_seq<R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        let indent = reader.col();
        let expected_indent = self.last_block_indent;
        reader.consume_bytes(1);

        if !matches!(curr_state, BlockMapExp(_, _)) && self.last_map_line == Some(reader.line()) {
            self.push_error(ErrorType::SequenceOnSameLineAsKey);
        }

        let new_seq = match curr_state {
            DocBlock => true,
            BlockSeq(ind) if indent > ind => true,
            BlockSeq(ind) if indent == ind => false,
            _ => {
                if let Some(last_seq) = self.stack.iter().rposition(|x| matches!(x, BlockSeq(_))) {
                    if let Some(unwind) = self.find_matching_state(
                        indent,
                        |state, indent| matches!(state, BlockSeq(ind) if ind == indent),
                    ) {
                        self.pop_block_states(unwind);
                    } else {
                        self.pop_block_states(self.stack.len() - last_seq);
                        self.push_error(ExpectedIndent {
                            actual: indent,
                            expected: expected_indent,
                        });
                    }
                    false
                } else {
                    self.next_map_state();
                    true
                }
            }
        };
        if new_seq {
            if self.prev_anchor.is_some() && !self.had_anchor {
                self.push_error(ErrorType::InvalidAnchorDeclaration);
            }
            self.emit_prev_anchor();
            self.push_state(BlockSeq(indent), reader.line());
            self.tokens.push_back(SEQ_START_BLOCK);
        }
    }

    fn find_matching_state(
        &self,
        matching_indent: u32,
        f: fn(LexerState, u32) -> bool,
    ) -> Option<usize> {
        self.stack
            .iter()
            .rposition(|state| f(*state, matching_indent))
            .map(|x| self.stack.len() - x - 1)
    }

    fn fetch_plain_scalar_flow<R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        let mut ends_with = b'\x7F';
        let scalar = self.get_plain_scalar(reader, curr_state, &mut ends_with);

        if ends_with == b':' {
            self.prev_scalar = scalar;
            self.continue_processing = true;
        } else {
            self.emit_prev_anchor();
            self.tokens.extend(scalar.tokens);
        }
    }

    fn get_plain_scalar<R: Reader<B>>(
        &mut self,
        reader: &mut R,
        curr_state: LexerState,
        ends_with: &mut u8,
    ) -> Scalar {
        let mut curr_indent = match curr_state {
            BlockMapExp(ind, _) => ind,
            _ => reader.col(),
        };
        let start_line = reader.line();
        let mut end_line = reader.line();
        let mut tokens = vec![ScalarPlain as usize];
        let mut offset_start = None;
        let in_flow_collection = curr_state.in_flow_collection();
        let mut had_comment = false;
        let mut num_newlines = 0;
        let scalar_start = self.scalar_start(curr_state, reader.col());
        let scalar_limit = match curr_state {
            BlockSeq(x) => x,
            _ => scalar_start,
        };

        while !reader.eof() {
            if curr_indent < scalar_limit && start_line != reader.line() {
                // if plain scalar is less indented than previous
                // It can be
                // a) Part of BlockMap so we must break
                // b) An error outside of block map
                // However not important for first line.
                match curr_state {
                    DocBlock => {
                        reader.read_line();
                        tokens.push(ErrorToken as usize);
                        self.errors.push(ExpectedIndent {
                            actual: curr_indent,
                            expected: scalar_start,
                        });
                    }
                    BlockMap(ind, _) | BlockMapExp(ind, _) => {
                        reader.read_line();
                        tokens.push(ErrorToken as usize);
                        self.errors.push(ExpectedIndent {
                            actual: curr_indent,
                            expected: ind,
                        });
                    }
                    _ => {}
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
                    tokens.push(NewLine as usize);
                    tokens.push(0);
                }
                x if x > 1 => {
                    tokens.push(NewLine as usize);
                    tokens.push(x - 1);
                }
                _ => {}
            }

            tokens.push(start);
            tokens.push(end);
            end_line = reader.line();
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

            if chr == b'-' && matches!(curr_state, BlockSeq(ind) if reader.col() > ind) {
                offset_start = Some(reader.pos());
            } else if (in_flow_collection && is_flow_indicator(chr)) || chr == b':' || chr == b'-' {
                *ends_with = u8::min(*ends_with, chr);
                break;
            } else if reader.peek_chars(&mut self.buf) == b"..." ||  self.find_matching_state(
                curr_indent,
                |state, indent| matches!(state, BlockMap(ind, _)| BlockSeq(ind) | BlockMapExp(ind, _) if ind == indent)
            ).is_some() {
                break;
            }
        }
        let is_multiline = end_line != start_line;
        tokens.push(ScalarEnd as usize);
        Scalar {
            scalar_start,
            is_multiline,
            tokens,
        }
    }

    fn scalar_start(&mut self, curr_state: LexerState, curr_col: u32) -> u32 {
        match curr_state {
            BlockMapExp(ind, _) => ind,
            BlockSeq(_) | BlockMap(_, BeforeColon | AfterColon) | DocBlock => {
                self.col_start.unwrap_or(curr_col)
            }
            _ => curr_col,
        }
    }

    fn fetch_explicit_map<R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        if !self.is_flow_map() {
            self.tokens.push_back(MAP_START);
        }

        if !reader.peek_byte_at(1).map_or(false, is_white_tab_or_break) {
            let scalar = self.get_plain_scalar(reader, curr_state, &mut b'\0');
            self.prev_scalar = scalar;
            self.continue_processing = true;
        } else {
            reader.consume_bytes(1);
            reader.skip_space_tab();
        }
    }

    pub const fn get_default_namespace(namespace: &[u8]) -> Option<Cow<'static, [u8]>> {
        match namespace {
            b"!!" => Some(Cow::Borrowed(b"tag:yaml.org,2002:")),
            b"!" => Some(Cow::Borrowed(b"!")),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn curr_state(&self) -> LexerState {
        *self.stack.last().unwrap_or(&LexerState::default())
    }

    #[inline(always)]
    pub fn set_curr_state(&mut self, state: LexerState, read_line: u32) {
        match self.stack.last_mut() {
            Some(x) => *x = state,
            None => self.push_state(state, read_line),
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
        if let Some(FlowSeq(_, state)) = self.stack.last_mut() {
            *state = seq_state;
        };
    }

    #[inline]
    fn next_map_state(&mut self) {
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
    fn update_col<R: Reader<B>>(&mut self, reader: &R) -> u32 {
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
    fn is_flow_map(&self) -> bool {
        match self.curr_state() {
            FlowMap(_, _) | FlowKeyExp(_, _) => true,
            _ => false,
        }
    }

    fn process_single_quote_block<R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        let has_tab = self.has_tab;
        let scalar_line = reader.line();
        let Scalar {
            scalar_start,
            is_multiline,
            tokens,
        } = self.process_single_quote(reader);
        reader.skip_space_tab();

        let ends_with = reader.peek_byte().map_or(false, |chr| chr == b':');

        self.process_block_scalar(
            curr_state,
            ends_with,
            scalar_start,
            has_tab,
            is_multiline,
            tokens,
            scalar_line,
        );
    }

    fn read_block_scalar<R: Reader<B>>(
        &mut self,
        reader: &mut R,
        literal: bool,
        curr_state: &LexerState,
        block_indent: u32,
    ) -> Vec<usize> {
        reader.consume_bytes(1);
        let mut chomp = ChompIndicator::Clip;
        let mut indentation: Option<NonZeroU32> = None;
        let mut tokens = Vec::with_capacity(8);

        let amount = match reader.peek_chars(&mut self.buf) {
            [_, b'0', ..] | [b'0', _, ..] => {
                reader.consume_bytes(2);
                self.push_error(ErrorType::ExpectedChompBetween1and9);
                return tokens;
            }
            [b'-', len, ..] | [len, b'-', ..] if matches!(len, b'1'..=b'9') => {
                chomp = ChompIndicator::Strip;
                indentation = NonZeroU32::new(block_indent + (len - b'0') as u32);
                2
            }
            [b'+', len, ..] | [len, b'+', ..] if matches!(len, b'1'..=b'9') => {
                chomp = ChompIndicator::Keep;
                indentation = NonZeroU32::new(block_indent + (len - b'0') as u32);
                2
            }
            [b'-', ..] => {
                chomp = ChompIndicator::Strip;
                1
            }
            [b'+', ..] => {
                chomp = ChompIndicator::Keep;
                1
            }
            [len, ..] if matches!(len, b'1'..=b'9') => {
                indentation = NonZeroU32::new(block_indent + (len - b'0') as u32);
                1
            }
            _ => 0,
        };
        reader.consume_bytes(amount);

        let token = if literal {
            ScalarLit as usize
        } else {
            ScalarFold as usize
        };

        // allow comment in first line of block scalar
        reader.skip_space_tab();
        match reader.peek_byte() {
            Some(b'#' | b'\r' | b'\n') => {
                reader.read_line();
            }
            Some(chr) => {
                reader.read_line();
                self.push_error(ErrorType::UnexpectedSymbol(chr as char));
                return tokens;
            }
            _ => {}
        }

        let mut new_line_token = 0;

        tokens.push(token);
        if reader.eof() {
            tokens.push(ScalarEnd as usize);
            return tokens;
        }
        let mut trailing = vec![];
        let mut is_trailing_comment = false;
        let mut previous_indent = 0;
        let mut max_prev_indent = 0;

        loop {
            let map_indent = reader.col() + reader.count_spaces();
            let prefix_indent = reader.col() + block_indent;
            let indent_has_reduced = map_indent <= block_indent && previous_indent != block_indent;
            let check_block_indent = reader.peek_byte_at(block_indent as usize).unwrap_or(b'\0');

            if (check_block_indent == b'-'
                && matches!(curr_state, BlockSeq(ind) if prefix_indent <= *ind ))
                || (check_block_indent == b':'
                    && matches!(curr_state, BlockMapExp(ind, _) if prefix_indent <= *ind))
            {
                reader.consume_bytes(block_indent as usize);
                break;
            } else if indent_has_reduced
                && matches!(curr_state, BlockMap(ind, _) if *ind <= map_indent)
            {
                break;
            }

            // count indents important for folded scalars
            let newline_indent = reader.count_spaces();

            if !is_trailing_comment
                && indentation.map_or(false, |x| newline_indent < x.into())
                && reader
                    .peek_byte_at(newline_indent as usize)
                    .map_or(false, |c| c == b'#')
            {
                trailing.push(NewLine as usize);
                trailing.push(new_line_token - 1);
                is_trailing_comment = true;
                new_line_token = 1;
            };

            let newline_is_empty = reader.is_empty_newline();

            if newline_is_empty && max_prev_indent < newline_indent {
                max_prev_indent = newline_indent;
            }

            if indentation.is_none() && newline_indent > 0 && !newline_is_empty {
                indentation = NonZeroU32::new(newline_indent);
                if max_prev_indent > newline_indent {
                    tokens.insert(0, ErrorToken as usize);
                    self.errors.push(ErrorType::SpacesFoundAfterIndent);
                }
            }

            let num_spaces = match indentation {
                None => newline_indent,
                Some(x) => x.into(),
            };

            if reader.peek_chars(&mut self.buf) == b"..."
                || reader.peek_chars(&mut self.buf) == b"---"
                || reader.eof()
            {
                break;
            }
            match reader.count_spaces_till(num_spaces) {
                count if count == block_indent as usize => {}
                count if count != num_spaces as usize => {
                    if newline_is_empty {
                        reader.consume_bytes(count);
                    } else {
                        self.push_error(ExpectedIndent {
                            actual: count as u32,
                            expected: num_spaces,
                        });

                        break;
                    }
                }
                count => {
                    reader.consume_bytes(count);
                }
            };

            let (start, end) = reader.read_line();
            if start != end {
                if new_line_token > 0 {
                    if !literal && previous_indent == newline_indent {
                        tokens.push(NewLine as usize);
                        tokens.push(new_line_token - 1);
                    } else {
                        tokens.push(NewLine as usize);
                        tokens.push(new_line_token);
                    }
                }
                previous_indent = newline_indent;
                tokens.push(start);
                tokens.push(end);
                new_line_token = 1;
            } else {
                new_line_token += 1;
            }
        }
        match chomp {
            ChompIndicator::Keep => {
                if is_trailing_comment {
                    new_line_token = 1;
                }
                trailing.push(NewLine as usize);
                trailing.push(new_line_token);
                tokens.extend(trailing);
            }
            ChompIndicator::Clip => {
                trailing.push(NewLine as usize);
                trailing.push(1);
                tokens.extend(trailing);
            }
            ChompIndicator::Strip => {}
        }
        tokens.push(ScalarEnd as usize);
        tokens
    }
}

pub(crate) enum QuoteState {
    Start,
    Trim,
    End,
    Error,
}

fn emit_token_mut(
    start: &mut usize,
    end: usize,
    newspaces: &mut Option<usize>,
    tokens: &mut Vec<usize>,
) {
    if end > *start {
        if let Some(newspace) = newspaces.take() {
            tokens.push(NewLine as usize);
            tokens.push(newspace);
        }
        tokens.push(*start);
        tokens.push(end);
        *start = end;
    }
}

#[inline]
fn update_newlines<B, R: Reader<B>>(
    reader: &mut R,
    newspaces: &mut Option<usize>,
    start_str: &mut usize,
) {
    *newspaces = Some(reader.skip_separation_spaces(true).0.saturating_sub(1) as usize);
    *start_str = reader.pos();
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

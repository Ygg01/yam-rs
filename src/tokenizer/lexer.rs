#![allow(clippy::match_like_matches_macro)]

use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::hint::unreachable_unchecked;
use std::mem::take;

use LexerState::PreDocStart;

use crate::tokenizer::lexer::LexerState::{
    AfterDocBlock, BlockMap, BlockMapExp, BlockSeq, DocBlock, FlowMap, FlowSeq, InDocEnd,
};
use crate::tokenizer::lexer::LexerToken::*;
use crate::tokenizer::lexer::MapState::*;
use crate::tokenizer::lexer::SeqState::*;

use crate::tokenizer::reader::{is_white_tab_or_break, Reader};
use crate::tokenizer::ErrorType::*;

use super::iterator::{DirectiveType, ScalarType};
use super::reader::{
    is_flow_indicator, is_plain_unsafe, is_valid_escape, is_valid_skip_char, is_white_tab,
};
use crate::tokenizer::ErrorType;

#[derive(Clone, Default)]
pub struct Lexer {
    pub stream_end: bool,
    pub(crate) tokens: VecDeque<usize>,
    pub(crate) errors: Vec<ErrorType>,
    pub(crate) tags: HashMap<Vec<u8>, (usize, usize)>,
    continue_processing: bool,
    col_start: Option<u32>,
    last_block_indent: Option<u32>,
    last_map_line: Option<u32>,
    had_anchor: bool,
    has_tab: bool,
    prev_anchor: Option<(usize, usize)>,
    prev_scalar: NodeSpans,
    prev_tag: Option<(usize, usize, usize)>,
    stack: Vec<LexerState>,
}

pub(crate) struct SeparationSpaceInfo {
    num_breaks: u32,
    num_indent: u32,
    has_comment: bool,
    has_tab: bool,
}

#[derive(Clone, Default)]
pub(crate) struct NodeSpans {
    col_start: u32,
    is_multiline: bool,
    spans: Vec<usize>,
}

impl NodeSpans {
    pub fn is_empty(&self) -> bool {
        self.spans.len() == 0
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum MapState {
    BeforeComplexKey,
    #[default]
    BeforeFirstKey,
    BeforeKey,
    BeforeColon,
    AfterColon,
}

impl MapState {
    pub fn next_state(self) -> MapState {
        match self {
            BeforeComplexKey | BeforeKey | BeforeFirstKey => BeforeColon,
            BeforeColon => AfterColon,
            AfterColon => BeforeKey,
        }
    }

    pub fn set_next_state(&mut self) {
        *self = self.next_state();
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum SeqState {
    /// State of sequence before the first element
    BeforeFirst,
    /// State of sequence before sequence separator
    #[default]
    BeforeElem,
    /// State of sequenec dealing with sequence node
    InSeqElem,
}
impl SeqState {
    fn set_next_state(&mut self) {
        *self = self.next_state();
    }
    fn next_state(self) -> SeqState {
        match self {
            InSeqElem => BeforeElem,
            BeforeFirst | BeforeElem => InSeqElem,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum LiteralStringState {
    AutoIndentation,
    Indentation(u32),
    End,
    Comment,
    TabError,
}

impl LiteralStringState {
    pub fn from_indentation(indent: u32) -> LiteralStringState {
        match indent {
            0 => Self::AutoIndentation,
            x => Self::Indentation(x),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum LexerState {
    #[default]
    PreDocStart,
    // DirectiveSection,
    // EndOfDirective,
    AfterDocBlock,
    InDocEnd,
    // Flow nodes
    // u32 is the index of the token insertion point for flow nodes
    FlowSeq,
    FlowMap(MapState),
    // Blocks nodes
    // u32 is the indent of block node
    DocBlock,
    BlockSeq(u32, SeqState),
    BlockMap(u32, MapState),
    //TODO Move Explicit key to MapState
    BlockMapExp(u32, MapState),
}

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum KeyType {
    NotKey,
    KeyCandidate,
    ComplexKey,
}

impl LexerState {
    pub(crate) fn get_key_type(self) -> KeyType {
        match self {
            DocBlock
            | FlowMap(BeforeFirstKey | BeforeKey)
            | BlockMap(_, BeforeFirstKey | BeforeKey) => KeyType::KeyCandidate,
            FlowMap(BeforeComplexKey) | BlockMapExp(_, _) | BlockMap(_, BeforeComplexKey) => {
                KeyType::ComplexKey
            }
            _ => KeyType::NotKey,
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum ChompIndicator {
    /// `-` final line break and any trailing empty lines are excluded from the scalar’s content
    Strip,
    ///  `` final line break character is preserved in the scalar’s content
    Clip,
    ///  `` final line break character is preserved in the scalar’s content but only containing whitespaces
    // ClipEmpty,
    /// `+` final line break and any trailing empty lines are considered to be part of the scalar’s content
    Keep,
}

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum ScalarEnd {
    /// Scalar ends with `-`
    Seq,
    ///  `:` terminated scalar
    Map,
    /// Other cases
    Plain,
}

impl ScalarEnd {
    fn set_to(&mut self, chr: u8) {
        match chr {
            b'-' => *self = ScalarEnd::Seq,
            b':' => *self = ScalarEnd::Map,
            _ => {}
        }
    }
}

impl LexerState {
    #[inline]
    pub fn in_flow_collection(self) -> bool {
        match &self {
            FlowSeq | FlowMap(_) => true,
            _ => false,
        }
    }

    fn is_incorrectly_indented(self, scalar_start: u32) -> bool {
        match self {
            BlockMapExp(ind, _) | BlockMap(ind, _) | BlockSeq(ind, _) => scalar_start < ind,
            _ => false,
        }
    }

    fn matches(self, scalar_start: u32, scalar_type: ScalarEnd) -> bool {
        match (self, scalar_type) {
            (BlockMapExp(ind, _) | BlockMap(ind, _), ScalarEnd::Map)
            | (BlockSeq(ind, _) | BlockMapExp(ind, _), ScalarEnd::Seq)
            | (BlockMap(ind, _) | BlockMapExp(ind, _) | BlockSeq(ind, _), ScalarEnd::Plain)
                if ind == scalar_start =>
            {
                true
            }
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum DirectiveState {
    NoDirective,
    OneDirective,
    TwoDirectiveError,
}

#[derive(Clone, Copy, PartialEq)]
enum HeaderState {
    Bare,
    Directive(DirectiveState),
    HeaderStart,
    HeaderEnd,
}

impl DirectiveState {
    fn add_directive(&mut self) {
        *self = match self {
            Self::NoDirective => Self::OneDirective,
            Self::OneDirective | Self::TwoDirectiveError => Self::TwoDirectiveError,
        }
    }
}

macro_rules! impl_quote {
    ($quote:ident($quote_start:expr), $trim:ident($trim_fn:ident, $lit:literal), $start:ident($quote_fn:ident) => $match_fn:ident) => {
        fn $quote<B, R: Reader<B>>(&mut self, reader: &mut R, allow_tab: bool) -> NodeSpans {
            let col_start = self.update_col(reader);
            let start_line = reader.line();

            let mut start_str = reader.consume_bytes(1);
            let mut spans = Vec::with_capacity(10);
            spans.push($quote_start);
            let mut newspaces = None;
            let mut state = QuoteState::Start;

            loop {
                state = match state {
                    QuoteState::Start => {
                        self.$start(reader, &mut start_str, &mut newspaces, &mut spans)
                    }
                    QuoteState::Trim => self.$trim(
                        reader,
                        allow_tab,
                        &mut start_str,
                        &mut newspaces,
                        &mut spans,
                    ),
                    QuoteState::End | QuoteState::Error => break,
                };
            }
            spans.push(ScalarEnd as usize);
            let is_multiline = start_line != reader.line();
            NodeSpans {
                col_start,
                is_multiline,
                spans,
            }
        }

        fn $start<B, R: Reader<B>>(
            &mut self,
            reader: &mut R,
            start_str: &mut usize,
            newspaces: &mut Option<usize>,
            tokens: &mut Vec<usize>,
        ) -> QuoteState {
            if let Some(pos) = reader.$quote_fn() {
                let match_pos = reader.consume_bytes(pos);
                self.$match_fn(reader, match_pos, start_str, newspaces, tokens)
            } else if reader.eof() {
                self.prepend_error(ErrorType::UnexpectedEndOfFile);
                QuoteState::Error
            } else {
                QuoteState::Trim
            }
        }

        fn $trim<B, R: Reader<B>>(
            &mut self,
            reader: &mut R,
            allow_tab: bool,
            start_str: &mut usize,
            newspaces: &mut Option<usize>,
            tokens: &mut Vec<usize>,
        ) -> QuoteState {
            if reader.peek_stream_ending() {
                self.errors.push(ErrorType::UnexpectedEndOfStream);
                tokens.insert(0, ErrorToken as usize);
            };
            let indent = self.indent();
            if !matches!(self.curr_state(), DocBlock) && reader.col() <= indent {
                self.push_error(ErrorType::InvalidQuoteIndent {
                    actual: reader.col(),
                    expected: indent,
                });
            }

            if let Some((match_pos, len)) = reader.$trim_fn(*start_str) {
                emit_token_mut(start_str, match_pos, newspaces, tokens);
                reader.consume_bytes(len);
            } else {
                self.update_newlines(reader, newspaces, start_str);
            }

            match reader.peek_byte() {
                Some(b'\n' | b'\r') => {
                    if self.update_newlines(reader, newspaces, start_str) && !allow_tab {
                        self.prepend_error_token(ErrorType::TabsNotAllowedAsIndentation, tokens);
                    }
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

impl Lexer {
    pub fn fetch_next_token<B, R: Reader<B>>(&mut self, reader: &mut R) {
        self.continue_processing = true;

        while self.continue_processing && !reader.eof() {
            let curr_state = self.curr_state();

            match curr_state {
                PreDocStart => self.fetch_pre_doc(reader),
                DocBlock | BlockMap(_, _) | BlockMapExp(_, _) => {
                    self.fetch_block_map(reader, curr_state);
                }
                BlockSeq(_, _) => self.fetch_block_seq(reader, curr_state),
                FlowSeq | FlowMap(_) => self.fetch_flow_node(reader),
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
                BlockSeq(_, BeforeFirst) => {
                    self.tokens.push_back(SCALAR_PLAIN);
                    self.tokens.push_back(SCALAR_END);
                    SEQ_END
                }
                BlockSeq(_, _) => SEQ_END,
                BlockMapExp(_, AfterColon | BeforeColon) | BlockMap(_, AfterColon) => {
                    self.tokens.push_back(SCALAR_PLAIN);
                    self.tokens.push_back(SCALAR_END);
                    MAP_END
                }
                BlockMapExp(_, _) | BlockMap(_, _) | FlowMap(_) => MAP_END,
                FlowSeq => {
                    self.tokens.push_back(ERROR_TOKEN);
                    self.errors.push(MissingFlowClosingBracket);
                    SEQ_END
                }
                DocBlock | AfterDocBlock => DOC_END,
                _ => continue,
            };
            self.tokens.push_back(token);
        }
    }

    fn fetch_pre_doc<B, R: Reader<B>>(&mut self, reader: &mut R) {
        use DirectiveState::NoDirective;
        use HeaderState::{Bare, Directive, HeaderEnd, HeaderStart};

        let mut header_state = Bare;

        loop {
            let chr = match reader.peek_byte() {
                None => {
                    match header_state {
                        Directive(_) => self.push_error(ExpectedDocumentEndOrContents),
                        HeaderStart => {
                            self.push_empty_token();
                            self.tokens.push_back(DOC_END);
                        }
                        _ => {}
                    }
                    self.stream_end = true;
                    return;
                }
                Some(b'#') => {
                    self.read_line(reader);
                    continue;
                }
                Some(x) if is_white_tab_or_break(x) => {
                    self.skip_separation_spaces(reader);
                    continue;
                }
                Some(x) => x,
            };

            match (header_state, chr) {
                (Bare, b'%') => {
                    let mut directive_state = NoDirective;
                    if !self.try_read_yaml_directive(reader, &mut directive_state)
                        && !self.try_read_tag(reader)
                    {}
                    header_state = Directive(directive_state);
                }
                (Bare, b'.') => {
                    if reader.peek_stream_ending() {
                        reader.consume_bytes(3);
                    }
                }
                (Directive(mut directive_state), b'%') => {
                    if !self.try_read_yaml_directive(reader, &mut directive_state)
                        && !self.try_read_tag(reader)
                    {}
                }
                (HeaderEnd, b'%') => {
                    header_state = Directive(NoDirective);
                }
                (Directive(_) | Bare, b'-') => {
                    if reader.peek_stream_ending() {
                        reader.consume_bytes(3);
                        self.last_map_line = Some(reader.line());
                        self.tokens.push_back(DOC_START_EXP);
                        header_state = HeaderStart;
                    } else {
                        self.tokens.push_back(DOC_START);
                        self.set_state(DocBlock);
                        break;
                    }
                }
                (Directive(_), b'.') => {
                    self.tokens.push_back(DOC_START);
                    if reader.peek_stream_ending() {
                        reader.consume_bytes(3);
                        self.tokens.push_front(ERROR_TOKEN);
                        self.errors.push(UnexpectedEndOfStream);
                        self.tokens.push_back(DOC_END_EXP);
                    } else {
                        self.push_error(UnexpectedSymbol('.'));
                    }
                    break;
                }
                (HeaderEnd | HeaderStart, b'.') => {
                    if reader.peek_stream_ending() {
                        reader.consume_bytes(3);
                        self.push_empty_token();
                        self.tokens.push_back(DOC_END_EXP);
                        header_state = match header_state {
                            HeaderStart => HeaderEnd,
                            _ => Bare,
                        };
                    } else {
                        self.tokens.push_back(DOC_START);
                        self.set_state(DocBlock);
                        break;
                    }
                }
                (HeaderEnd | HeaderStart, b'-') => {
                    if reader.peek_stream_ending() {
                        reader.consume_bytes(3);
                        self.push_empty_token();
                        self.tokens.push_back(DOC_END);
                        self.tokens.push_back(DOC_START_EXP);
                    } else {
                        self.set_state(DocBlock);
                        break;
                    }
                }
                (Bare | Directive(_), _) => {
                    self.tokens.push_back(DOC_START);
                    self.set_state(DocBlock);
                    break;
                }
                (HeaderStart, _) => {
                    self.set_state(DocBlock);
                    break;
                }
                (HeaderEnd, _) => {
                    reader.skip_space_tab();
                    if reader
                        .peek_byte()
                        .map_or(false, |c| c != b'\r' && c != b'\n' && c != b'#')
                    {
                        self.push_error(ExpectedDocumentEnd);
                    }
                    self.set_state(DocBlock);
                    break;
                }
            }
        }
    }

    fn try_read_yaml_directive<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        directive_state: &mut DirectiveState,
    ) -> bool {
        if reader.col() == 0 && reader.try_read_slice_exact("%YAML") {
            reader.skip_space_tab();
            return match reader.peek_chars() {
                b"1.0" | b"1.1" | b"1.2" | b"1.3" => {
                    directive_state.add_directive();
                    if *directive_state == DirectiveState::TwoDirectiveError {
                        self.tokens.push_back(ERROR_TOKEN);
                        self.errors.push(TwoDirectivesFound);
                    }
                    self.tokens.push_back(DIR_YAML);
                    self.tokens.push_back(reader.pos());
                    self.tokens.push_back(reader.consume_bytes(3));
                    reader.skip_space_tab();
                    let invalid_char = reader
                        .peek_byte()
                        .map_or(false, |c| c != b'\r' && c != b'\n' && c != b'#');
                    if invalid_char {
                        self.prepend_error(InvalidAnchorDeclaration);
                        self.read_line(reader);
                    }
                    true
                }
                b"..." | b"---" => false,
                _ => {
                    self.read_line(reader);
                    false
                }
            };
        }
        false
    }

    fn try_read_tag<B, R: Reader<B>>(&mut self, reader: &mut R) -> bool {
        self.continue_processing = false;
        reader.try_read_slice_exact("%TAG");
        reader.skip_space_tab();

        if let Ok(key) = reader.read_tag_handle() {
            reader.skip_space_tab();
            if let Some(val) = reader.read_tag_uri() {
                self.tags.insert(key, val);
            }
            true
        } else {
            false
        }
    }

    fn fetch_after_doc<B, R: Reader<B>>(&mut self, reader: &mut R) {
        let mut consume_line = false;

        let is_stream_ending = reader.peek_stream_ending();
        let chars = reader.peek_chars();
        match chars {
            b"..." if is_stream_ending => {
                let col = reader.col();
                reader.consume_bytes(3);
                if col != 0 {
                    self.push_error(UnexpectedIndentDocEnd {
                        actual: col,
                        expected: 0,
                    });
                }
                self.tokens.push_back(DOC_END_EXP);
                self.set_block_state(InDocEnd, 0);
            }
            [peek, b'#', ..] if is_white_tab(*peek) => {
                // comment
                self.read_line(reader);
            }
            [b'#', ..] if reader.col() > 0 => {
                // comment that doesnt
                self.push_error(MissingWhitespaceBeforeComment);
                self.read_line(reader);
            }
            [chr, ..] if is_white_tab_or_break(*chr) => {
                self.skip_separation_spaces(reader);
            }
            [chr, ..] => {
                consume_line = true;
                self.tokens.push_back(DOC_END);
                self.push_error(UnexpectedSymbol(*chr as char));
                self.set_block_state(PreDocStart, 0);
            }
            [] => {}
        }
        if consume_line {
            self.read_line(reader);
        }
    }

    fn fetch_end_doc<B, R: Reader<B>>(&mut self, reader: &mut R) {
        reader.skip_space_tab();
        match reader.peek_byte() {
            Some(b'#') => {
                self.read_line(reader);
            }
            Some(b'%') => {
                self.set_state(PreDocStart);
            }
            Some(b'-') => {
                if reader.peek_stream_ending() {
                    reader.consume_bytes(3);
                    self.tokens.push_back(DOC_START_EXP);
                }
            }
            Some(b'.') => {
                if reader.peek_stream_ending() {
                    reader.consume_bytes(3);
                    self.tokens.push_back(DOC_END_EXP);
                }
            }
            Some(chr) if chr == b' ' || chr == b'\t' || chr == b'\r' || chr == b'\n' => {
                self.set_state(PreDocStart);
            }
            Some(_) => {
                self.read_line(reader);
                self.push_error(ExpectedDocumentStartOrContents);
            }
            None => {
                self.stream_end = true;
            }
        }
    }

    fn fetch_block_seq<B, R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        self.continue_processing = false;
        let is_stream_ending = reader.peek_stream_ending();
        let chars = reader.peek_chars();

        match chars {
            [b'{' | b'[', ..] => {
                self.next_substate();
                self.fetch_flow_node(reader);
            }
            [b'&', ..] => self.parse_anchor(reader),
            [b'*', ..] => self.parse_alias(reader),
            [b'-'] => {
                self.process_block_seq(reader, curr_state);
            }
            [b'-', x, ..] if is_white_tab_or_break(*x) => {
                self.process_block_seq(reader, curr_state);
            }
            b"---" if is_stream_ending => self.unwind_to_root_start(reader),
            b"..." if is_stream_ending => self.unwind_to_root_end(reader),
            [b'?', x, ..] if is_white_tab_or_break(*x) => {
                self.fetch_exp_block_map_key(reader, curr_state);
            }
            [b':'] => self.process_colon_block(reader, curr_state),
            [b':', peek, ..] if is_plain_unsafe(*peek) => {
                self.process_colon_block(reader, curr_state);
            }
            [b'!', ..] => self.fetch_tag(reader),
            [b'|', ..] => self.process_block_literal(reader, curr_state, true),
            [b'>', ..] => self.process_block_literal(reader, curr_state, false),
            [b'\'', ..] => self.process_single_quote_block(reader, curr_state),
            [b'"', ..] => self.process_double_quote_block(reader, curr_state),
            [peek, b'#', ..] if is_white_tab(*peek) => {
                // comment
                self.read_line(reader);
            }
            [b'#', ..] if reader.col() > 0 => {
                // comment that doesnt
                self.push_error(MissingWhitespaceBeforeComment);
                self.read_line(reader);
            }
            [b'%', ..] => {
                self.push_error(UnexpectedDirective);
                self.read_line(reader);
            }
            [peek, ..] if is_white_tab_or_break(*peek) => {
                self.has_tab = self.skip_separation_spaces(reader).has_tab;
                self.continue_processing = true;
            }
            [peek_chr, ..] => {
                self.fetch_plain_scalar_block(reader, curr_state, *peek_chr);
            }
            [] => self.stream_end = true,
        }
    }

    fn fetch_block_map<B, R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        self.continue_processing = false;
        let is_stream_ending = reader.peek_stream_ending();

        let chars = reader.peek_chars();
        match chars {
            [b'{' | b'[', ..] => {
                self.fetch_flow_node(reader);
                self.next_substate();
            }
            [b'&', ..] => self.parse_anchor(reader),
            [b'*', ..] => self.parse_alias(reader),
            [b':'] => self.process_colon_block(reader, curr_state),
            [b':', peek, ..] if is_plain_unsafe(*peek) => {
                self.process_colon_block(reader, curr_state);
            }
            [b'-', peek, ..] if is_plain_unsafe(*peek) => {
                self.process_block_seq(reader, curr_state);
            }
            [b'-'] => {
                self.process_block_seq(reader, curr_state);
            }
            b"..." if is_stream_ending => {
                self.unwind_to_root_end(reader);
            }
            b"---" if is_stream_ending => {
                self.unwind_to_root_start(reader);
            }
            [b'?', peek, ..] if is_plain_unsafe(*peek) => {
                self.fetch_exp_block_map_key(reader, curr_state);
            }
            [b'!', ..] => self.fetch_tag(reader),
            [b'|', ..] => {
                self.next_substate();
                self.process_block_literal(reader, curr_state, true);
            }
            [b'>', ..] => {
                self.next_substate();
                self.process_block_literal(reader, curr_state, false);
            }
            [b'\'', ..] => {
                self.process_single_quote_block(reader, curr_state);
            }
            [b'"', ..] => {
                self.process_double_quote_block(reader, curr_state);
            }
            [peek, b'#', ..] if is_white_tab(*peek) => {
                // comment
                self.read_line(reader);
            }
            [b'#', ..] if reader.col() > 0 => {
                // comment that doesnt
                self.push_error(MissingWhitespaceBeforeComment);
                self.read_line(reader);
            }
            [b'%', ..] => {
                self.push_error(UnexpectedDirective);
                self.read_line(reader);
            }
            [peek, ..] if is_white_tab_or_break(*peek) => {
                self.has_tab = self.skip_separation_spaces(reader).has_tab;
                self.continue_processing = true;
            }
            [peek, ..] => {
                self.fetch_plain_scalar_block(reader, curr_state, *peek);
            }
            _ => self.stream_end = true,
        }
    }

    fn process_block_literal<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        curr_state: LexerState,
        literal: bool,
    ) {
        let had_tab = self.has_tab;
        let scalar_line = reader.line();
        let scalar_start = reader.col();

        let block_indent = self.indent();
        let tokens = self.read_block_scalar(reader, literal, self.curr_state(), block_indent);
        let is_multiline = reader.line() != scalar_line;
        reader.skip_space_tab();

        let is_key = reader.peek_byte().map_or(false, |chr| chr == b':');

        self.process_block_scalar(
            reader,
            curr_state,
            is_key,
            NodeSpans {
                col_start: scalar_start,
                is_multiline,
                spans: tokens,
            },
            had_tab,
            scalar_line,
        );
    }

    // TODO Uncomment once all test pass
    // #[inline]
    fn push_error(&mut self, error: ErrorType) {
        self.tokens.push_back(ERROR_TOKEN);
        self.errors.push(error);
    }

    // TODO Uncomment once all test pass
    fn push_error_token(&mut self, error: ErrorType, spans: &mut Vec<usize>) {
        spans.push(ERROR_TOKEN);
        self.errors.push(error);
    }

    // TODO Uncomment once all test pass
    fn prepend_error_token(&mut self, error: ErrorType, spans: &mut Vec<usize>) {
        spans.insert(0, ERROR_TOKEN);
        self.errors.push(error);
    }

    // TODO Uncomment once all test pass
    // #[inline]
    fn prepend_error(&mut self, error: ErrorType) {
        self.tokens.push_front(ERROR_TOKEN);
        self.errors.push(error);
    }

    fn parse_anchor<B, R: Reader<B>>(&mut self, reader: &mut R) {
        self.update_col(reader);
        let anchor = reader.consume_anchor_alias();

        let line = self.skip_separation_spaces(reader);
        if line.num_breaks == 0 {
            self.prev_anchor = Some(anchor);
        } else {
            self.tokens.push_back(ANCHOR);
            self.tokens.push_back(anchor.0);
            self.tokens.push_back(anchor.1);
            self.had_anchor = true;
        }
    }

    fn parse_alias<B, R: Reader<B>>(&mut self, reader: &mut R) {
        let alias_start = reader.col();
        let had_tab = self.has_tab;
        let alias = reader.consume_anchor_alias();
        self.skip_separation_spaces(reader);

        let next_is_colon = reader.peek_byte_is(b':');

        self.next_substate();
        if next_is_colon {
            self.process_block_scalar(
                reader,
                self.curr_state(),
                true,
                NodeSpans {
                    col_start: alias_start,
                    is_multiline: false,
                    spans: vec![ALIAS, alias.0, alias.1],
                },
                had_tab,
                reader.line(),
            );
        } else {
            self.tokens.push_back(ALIAS);
            self.tokens.push_back(alias.0);
            self.tokens.push_back(alias.1);
        }
    }

    fn try_parse_anchor<B, R: Reader<B>>(&mut self, reader: &mut R) -> Option<(usize, usize)> {
        self.update_col(reader);
        let anchor = reader.consume_anchor_alias();

        let is_anchor_inline = self.skip_separation_spaces(reader).num_breaks == 0;
        if is_anchor_inline {
            Some(anchor)
        } else {
            self.tokens.push_back(ANCHOR);
            self.tokens.push_back(anchor.0);
            self.tokens.push_back(anchor.1);
            self.had_anchor = true;
            None
        }
    }

    fn try_parse_tag<B, R: Reader<B>>(&mut self, reader: &mut R, spans: &mut Vec<usize>) -> bool {
        match reader.read_tag() {
            (Some(err), ..) => {
                self.push_error(err);
                false
            }
            (None, start, mid, end) => {
                spans.push(TAG_START);
                spans.push(start);
                spans.push(mid);
                spans.push(end);
                true
            }
        }
    }

    fn fetch_flow_node<B, R: Reader<B>>(&mut self, reader: &mut R) {
        let tokens = self.get_flow_node(reader);
        self.tokens.extend(tokens.spans);
        if matches!(self.curr_state(), DocBlock) {
            self.set_state(AfterDocBlock);
        }
    }

    fn get_flow_node<B, R: Reader<B>>(&mut self, reader: &mut R) -> NodeSpans {
        let Some(chr) = reader.peek_byte() else {
                self.stream_end = true;
                return NodeSpans::default();
            };

        if chr == b',' || chr == b']' || chr == b'}' {
            return NodeSpans::default();
        }
        if chr == b'[' {
            self.push_state(FlowSeq);
            self.had_anchor = false;
            self.get_flow_seq(reader)
        } else if chr == b'{' {
            self.had_anchor = false;
            self.get_flow_map(reader, MapState::default())
        } else {
            let start = reader.line();
            let mut scalar = self.get_scalar_node(reader, chr);
            let post_end = reader.line();
            scalar.is_multiline = start != post_end;
            scalar
        }
    }

    fn get_scalar_node<B, R: Reader<B>>(&mut self, reader: &mut R, chr: u8) -> NodeSpans {
        let mut scal_spans: Vec<usize> = Vec::with_capacity(10);
        let col_start = reader.col();
        if is_white_tab_or_break(chr) {
            self.skip_separation_spaces(reader);
            NodeSpans::default()
        } else if chr == b'&' {
            self.prev_anchor = self.try_parse_anchor(reader);
            NodeSpans::default()
        } else if chr == b'!' && self.try_parse_tag(reader, &mut scal_spans) {
            NodeSpans {
                col_start,
                is_multiline: false,
                spans: scal_spans,
            }
        } else if chr == b'*' {
            let alias = reader.consume_anchor_alias();

            scal_spans.push(ALIAS);
            scal_spans.push(alias.0);
            scal_spans.push(alias.1);

            NodeSpans {
                col_start,
                is_multiline: false,
                spans: scal_spans,
            }
        } else if matches!(chr, b'-' | b'?' | b':')
            && reader.peek_byte_at(1).map_or(false, is_plain_unsafe)
        {
            reader.consume_bytes(1);
            self.push_error_token(InvalidScalarStart, &mut scal_spans);
            NodeSpans {
                col_start: 0,
                is_multiline: false,
                spans: scal_spans,
            }
        } else if chr == b'\'' {
            let mut spans = self.prepend_tags_n_anchor();
            let scal = self.process_single_quote(reader, true);
            spans.extend(scal.spans);
            NodeSpans {
                col_start: scal.col_start,
                is_multiline: scal.is_multiline,
                spans,
            }
        } else if chr == b'"' {
            let mut spans = self.prepend_tags_n_anchor();
            let scal = self.process_double_quote(reader, true);
            spans.extend(scal.spans);
            NodeSpans {
                col_start: scal.col_start,
                is_multiline: scal.is_multiline,
                spans,
            }
        } else {
            let mut spans = self.prepend_tags_n_anchor();
            let scal = self.get_plain_scalar(reader, self.curr_state(), &mut ScalarEnd::Plain);
            spans.extend(scal.spans);
            NodeSpans {
                col_start: scal.col_start,
                is_multiline: scal.is_multiline,
                spans,
            }
        }
    }

    fn get_flow_seq<B, R: Reader<B>>(&mut self, reader: &mut R) -> NodeSpans {
        let line_begin = reader.line();
        let col_start = reader.col();
        let mut seq_state = BeforeFirst;
        let mut spans = self.prepend_tags_n_anchor();
        let mut end_found = false;

        spans.push(SEQ_START_EXP);
        reader.consume_bytes(1);

        loop {
            let Some(chr) = reader.peek_byte() else {
                self.stream_end = true;
                break;
            };

            let peek_next = reader.peek_byte_at(1).unwrap_or(b'\0');

            if is_white_tab_or_break(chr) {
                let num_ind = self.skip_separation_spaces(reader).num_indent;

                if num_ind < self.indent() {
                    self.push_error_token(ErrorType::TabsNotAllowedAsIndentation, &mut spans);
                }
            } else if chr == b']' {
                reader.consume_bytes(1);
                end_found = true;
                break;
            } else if chr == b'#' {
                self.push_error_token(ErrorType::InvalidCommentStart, &mut spans);
                self.read_line(reader);
            } else if chr == b',' {
                reader.consume_bytes(1);
                if matches!(seq_state, BeforeElem | BeforeFirst) {
                    self.push_error_token(ExpectedNodeButFound { found: ',' }, &mut spans);
                }
                seq_state = BeforeElem;
            } else if chr == b'?' && is_white_tab_or_break(peek_next) {
                spans.push(MAP_START_EXP);
                spans.extend(self.get_flow_map(reader, MapState::BeforeComplexKey).spans);
            } else if chr == b':'
                && matches!(
                    peek_next,
                    b'[' | b']' | b'{' | b'}' | b' ' | b'\t' | b'\r' | b'\n'
                )
            {
                match peek_next {
                    b'[' | b'{' | b'}' => {
                        self.push_error_token(UnexpectedSymbol(peek_next as char), &mut spans);
                        reader.consume_bytes(2);
                    }
                    _ => {
                        reader.consume_bytes(1);
                    }
                }

                spans.push(MAP_START_EXP);
                spans.push(SCALAR_PLAIN);
                spans.push(SCALAR_END);
                spans.extend(self.get_flow_map(reader, AfterColon).spans);
            } else {
                let node = self.get_flow_node(reader);
                self.check_flow_indent(node.col_start, &mut spans);

                let skip_colon_space = is_skip_colon_space(&node);
                let offset = reader.count_whitespace();
                if reader.peek_byte_at(offset).map_or(false, |c| c == b':') {
                    reader.consume_bytes(offset + 1);
                    if !skip_colon_space && reader.skip_space_tab() == 0 {
                        self.push_error_token(MissingWhitespaceAfterColon, &mut spans);
                    }

                    if node.is_multiline {
                        spans.extend(node.spans);
                        self.push_error_token(ImplicitKeysNeedToBeInline, &mut spans);
                    } else {
                        let map_start = if self.curr_state().in_flow_collection() {
                            MAP_START_EXP
                        } else {
                            MAP_START
                        };
                        spans.push(map_start);
                        spans.extend(node.spans);
                        spans.extend(self.get_flow_map(reader, AfterColon).spans);
                    }
                    seq_state.set_next_state();
                } else if !node.spans.is_empty() {
                    if !Lexer::is_fake_node(&node) {
                        seq_state.set_next_state();
                    }
                    spans.extend(node.spans);
                }
            }
        }

        let offset = reader.count_whitespace();
        let curr_state = self.curr_state();
        if reader.peek_byte_at(offset) == Some(b':')
            && matches!(curr_state, FlowSeq | DocBlock)
            && reader.peek_byte_at(1).map_or(true, is_white_tab_or_break)
        {
            reader.consume_bytes(1 + offset);
            reader.skip_space_tab();
            if line_begin == reader.line() {
                let map_start = if self.prev_state().in_flow_collection() {
                    MAP_START_EXP
                } else {
                    MAP_START
                };
                spans.insert(0, map_start);
                spans.push(SEQ_END);
                spans.extend(self.get_flow_map(reader, AfterColon).spans);
                self.pop_state();
            } else {
                self.push_error_token(ImplicitKeysNeedToBeInline, &mut spans);
            }
        } else if end_found {
            self.pop_state();
            spans.push(SEQ_END);
        }

        NodeSpans {
            col_start,
            is_multiline: line_begin != reader.line(),
            spans,
        }
    }

    #[inline]
    fn check_flow_indent(&mut self, actual: u32, spans: &mut Vec<usize>) {
        let expected = self.indent();
        if actual < expected {
            self.push_error_token(ExpectedIndent { actual, expected }, spans);
        }
    }

    #[inline]
    fn is_fake_node(node: &NodeSpans) -> bool {
        matches!(node.spans.first(), None | Some(&TAG_START))
    }

    fn get_flow_map<B, R: Reader<B>>(&mut self, reader: &mut R, init_state: MapState) -> NodeSpans {
        let mut map_state = init_state;
        let mut spans = self.prepend_tags_n_anchor();
        let mut skip_colon_space = true;

        let start_begin = reader.line();
        let col_start = reader.col();
        let is_nested = init_state != MapState::default();

        self.push_state(FlowMap(map_state));

        if reader.peek_byte_is(b'{') {
            reader.consume_bytes(1);
            spans.push(MAP_START_EXP);
        }

        let mut is_end_emitted = is_nested;

        loop {
            let chr = match reader.peek_byte() {
                None => {
                    self.stream_end = true;
                    break;
                }
                Some(b',' | b']') if is_nested => {
                    break;
                }
                Some(x) => x,
            };

            let peek_next = reader.peek_byte_at(1);

            if is_white_tab_or_break(chr) {
                self.skip_separation_spaces(reader);
            } else if chr == b'}' {
                reader.consume_bytes(1);
                is_end_emitted = true;
                break;
            } else if chr == b':' && matches!(map_state, BeforeColon) {
                reader.consume_bytes(1);
                map_state.set_next_state();
            } else if chr == b':'
                && matches!(map_state, BeforeKey | BeforeFirstKey)
                && (skip_colon_space
                    && peek_next.map_or(false, |c| matches!(c, b',' | b'[' | b'{' | b'}'))
                    || peek_next.map_or(false, is_white_tab_or_break))
            {
                reader.consume_bytes(1);
                reader.skip_space_tab();
                map_state = AfterColon;
                push_empty(&mut spans);
            } else if chr == b':'
                && map_state == AfterColon
                && peek_next.map_or(false, is_plain_unsafe)
            {
                push_empty(&mut spans);
                map_state = BeforeKey;
            } else if chr == b',' {
                reader.consume_bytes(1);
                if matches!(map_state, AfterColon | BeforeColon) {
                    push_empty(&mut spans);
                    map_state = BeforeKey;
                }
            } else if chr == b'?' && peek_next.map_or(false, is_white_tab_or_break) {
                reader.consume_bytes(1);
                reader.skip_space_tab();

                let node_spans = self.get_flow_node(reader);
                self.check_flow_indent(node_spans.col_start, &mut spans);
                if node_spans.is_empty() {
                    push_empty(&mut spans);
                } else {
                    spans.extend(node_spans.spans);
                }
                map_state.set_next_state();
            } else {
                let scalar_spans = self.get_flow_node(reader);
                self.check_flow_indent(scalar_spans.col_start, &mut spans);
                skip_colon_space = is_skip_colon_space(&scalar_spans);
                if !Lexer::is_fake_node(&scalar_spans) {
                    map_state.set_next_state();
                }
                spans.extend(scalar_spans.spans);
            }
        }
        if matches!(map_state, BeforeColon | AfterColon) {
            push_empty(&mut spans);
        }
        if is_end_emitted {
            self.pop_state();
            spans.push(MAP_END);
        }
        NodeSpans {
            col_start,
            is_multiline: start_begin != reader.line(),
            spans,
        }
    }

    impl_quote!(process_single_quote(SCALAR_QUOTE), single_quote_trim(get_single_quote_trim, b'\''), single_quote_start(get_single_quote) => single_quote_match);

    fn single_quote_match<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        match_pos: usize,
        start_str: &mut usize,
        newspaces: &mut Option<usize>,
        tokens: &mut Vec<usize>,
    ) -> QuoteState {
        match reader.peek_chars() {
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

    fn process_double_quote_block<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        curr_state: LexerState,
    ) {
        let had_tab = self.has_tab;
        let scalar_line: u32 = reader.line();
        let scalar = self.process_double_quote(reader, false);
        reader.skip_space_tab();

        let is_key = reader.peek_byte().map_or(false, |chr| chr == b':');

        self.process_block_scalar(reader, curr_state, is_key, scalar, had_tab, scalar_line);
    }

    impl_quote!(process_double_quote(SCALAR_DQUOTE), double_quote_trim(get_double_quote_trim, b'"'), double_quote_start(get_double_quote) => double_quote_match);

    fn double_quote_match<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        match_pos: usize,
        start_str: &mut usize,
        newspaces: &mut Option<usize>,
        tokens: &mut Vec<usize>,
    ) -> QuoteState {
        match reader.peek_chars() {
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
                self.update_newlines(reader, &mut None, start_str);
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
                    self.errors.push(InvalidEscapeCharacter);
                    reader.consume_bytes(2);
                }
            }
            [b'"', ..] => {
                emit_newspace(tokens, newspaces);
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

    #[inline]
    fn update_newlines<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        newspaces: &mut Option<usize>,
        start_str: &mut usize,
    ) -> bool {
        let x = self.skip_separation_spaces(reader);
        *newspaces = Some(x.num_breaks.saturating_sub(1) as usize);
        *start_str = reader.pos();
        self.last_block_indent
            .map_or(false, |indent| indent >= x.num_indent)
    }

    fn process_block_scalar<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        curr_state: LexerState,
        is_key: bool,
        scalar: NodeSpans,
        has_tab: bool,
        scalar_line: u32,
    ) {
        if is_key {
            let scal = self.col_start.unwrap_or(scalar.col_start);
            let is_map_start = match curr_state {
                DocBlock => true,
                BlockSeq(ind, _) if scal == ind => {
                    let is_prev_map =
                        matches!(self.curr_state(), BlockMap(indent, _) if indent == ind);
                    if is_prev_map {
                        false
                    } else {
                        self.push_error(UnexpectedScalarAtNodeEnd);
                        true
                    }
                }
                BlockMap(ind, _) | BlockSeq(ind, _) if scal > ind => true,
                BlockMapExp(ind, _)
                    if scal > ind
                        && matches!(self.last_map_line, Some(x) if x == reader.line()) =>
                {
                    true
                }
                _ => false,
            };
            let scalar_start = scalar;
            self.prev_scalar = scalar_start;
            if !matches!(curr_state, BlockMapExp(_, _)) {
                if self.last_map_line == Some(scalar_line) {
                    self.push_error(ImplicitKeysNeedToBeInline);
                }
                if self.prev_scalar.is_multiline {
                    self.push_error(ImplicitKeysNeedToBeInline);
                }
            }
            self.last_map_line = Some(scalar_line);
            if is_map_start {
                self.had_anchor = false;
                self.next_substate();
                self.continue_processing = true;
                if has_tab {
                    self.push_error(TabsNotAllowedAsIndentation);
                }
                self.tokens.push_back(MAP_START);
                self.emit_meta_nodes();
                self.push_block_state(
                    BlockMap(self.prev_scalar.col_start, BeforeColon),
                    scalar_line,
                );
            } else if matches!(curr_state, BlockMapExp(ind, _) if ind == self.prev_scalar.col_start)
            {
                if has_tab {
                    self.push_error(TabsNotAllowedAsIndentation);
                }

                if let BlockMapExp(indent, BeforeColon) = curr_state {
                    self.push_empty_token();
                    self.set_block_state(BlockMap(indent, BeforeColon), scalar_line);
                }
            }
        } else {
            if self.last_map_line != Some(scalar_line)
                && curr_state.is_incorrectly_indented(scalar.col_start)
            {
                self.push_error(ImplicitKeysNeedToBeInline);
            }
            match curr_state {
                BlockMap(ind, BeforeKey) if ind == scalar.col_start => {
                    self.push_error(UnexpectedScalarAtNodeEnd);
                }
                BlockMap(_, BeforeKey) if self.last_map_line == Some(scalar_line) => {
                    self.push_error(UnexpectedScalarAtNodeEnd);
                }
                BlockMapExp(_, _) | BlockMap(_, _) | BlockSeq(_, BeforeElem | BeforeFirst) => {
                    self.next_substate();
                }
                BlockSeq(_, InSeqElem) => {
                    self.push_error(ErrorType::ExpectedSeqStart);
                }
                _ => {}
            }
            self.emit_meta_nodes();
            self.tokens.extend(scalar.spans);
        }
    }

    #[inline]
    fn emit_meta_nodes(&mut self) {
        if let Some(anchor) = take(&mut self.prev_anchor) {
            if self.had_anchor {
                self.push_error(NodeWithTwoAnchors);
            }
            self.tokens.push_back(ANCHOR);
            self.tokens.push_back(anchor.0);
            self.tokens.push_back(anchor.1);
        };
        if let Some(tag) = take(&mut self.prev_tag) {
            self.tokens.push_back(TAG_START);
            self.tokens.push_back(tag.0);
            self.tokens.push_back(tag.1);
            self.tokens.push_back(tag.2);
        }
        self.had_anchor = false;
    }

    fn prepend_tags_n_anchor(&mut self) -> Vec<usize> {
        let mut tokens: Vec<usize> = Vec::with_capacity(10);
        if let Some(anchor) = take(&mut self.prev_anchor) {
            if self.had_anchor {
                self.push_error_token(NodeWithTwoAnchors, &mut tokens);
            }
            tokens.push(ANCHOR);
            tokens.push(anchor.0);
            tokens.push(anchor.1);
        };
        if let Some(tag) = take(&mut self.prev_tag) {
            tokens.push(TAG_START);
            tokens.push(tag.0);
            tokens.push(tag.1);
            tokens.push(tag.2);
        }
        self.had_anchor = false;
        tokens
    }

    fn skip_separation_spaces<B, R: Reader<B>>(&mut self, reader: &mut R) -> SeparationSpaceInfo {
        let lines = {
            let mut num_breaks = 0u32;
            let mut num_indent = 0u32;
            let mut found_eol = true;
            let mut has_tab = false;
            let mut has_comment = false;

            loop {
                if !reader.peek_byte().map_or(false, is_valid_skip_char) || reader.eof() {
                    break;
                }
                let sep = reader.count_space_then_tab();
                num_indent = sep.0;
                let amount = sep.1;
                has_tab = num_indent as usize != amount;
                let is_comment = reader.peek_byte_at(amount).map_or(false, |c| c == b'#');

                if has_comment && !is_comment {
                    break;
                }
                if is_comment {
                    has_comment = true;
                    if amount > 0
                        && !reader
                            .peek_byte_at(amount.saturating_sub(1))
                            .map_or(false, |c| c == b' ' || c == b'\t' || c == b'\n')
                    {
                        self.push_error(MissingWhitespaceBeforeComment);
                    }
                    self.read_line(reader);
                    found_eol = true;
                    num_breaks += 1;
                    continue;
                }

                if reader.read_break().is_some() {
                    num_breaks += 1;
                    has_tab = false;
                    found_eol = true;
                }

                if found_eol {
                    let (indent, amount) = reader.count_space_then_tab();
                    num_indent = indent;
                    has_tab = indent as usize != amount;
                    reader.consume_bytes(amount);
                    found_eol = false;
                } else {
                    break;
                }
            }
            SeparationSpaceInfo {
                num_breaks,
                num_indent,
                has_comment,
                has_tab,
            }
        };
        if lines.num_breaks > 0 {
            self.col_start = None;
        }
        lines
    }

    // TODO enable after test
    // #[inline]
    fn push_empty_token(&mut self) {
        self.tokens.push_back(SCALAR_PLAIN);
        self.tokens.push_back(SCALAR_END);
    }

    #[inline]
    fn pop_state(&mut self) -> Option<LexerState> {
        let pop_state = self.stack.pop();
        if let Some(state) = self.stack.last_mut() {
            match state {
                BlockMap(indent, _) | BlockMapExp(indent, _) | BlockSeq(indent, _) => {
                    self.last_block_indent = Some(*indent);
                }
                _ => {}
            }
        };
        pop_state
    }

    fn push_state(&mut self, state: LexerState) {
        assert!(!matches!(
            state,
            BlockMap(_, _) | BlockSeq(_, _) | BlockMapExp(_, _)
        ));
        self.stack.push(state);
    }

    fn push_block_state(&mut self, state: LexerState, read_line: u32) {
        match state {
            BlockMap(indent, _) | BlockMapExp(indent, _) => {
                self.last_block_indent = Some(indent);
                self.had_anchor = false;
                self.last_map_line = Some(read_line);
            }
            BlockSeq(indent, _) => {
                self.last_block_indent = Some(indent);
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
                Some(BlockSeq(_, SeqState::BeforeFirst)) => {
                    self.push_empty_token();
                    self.tokens.push_back(SEQ_END);
                }
                Some(BlockSeq(_, _)) => self.tokens.push_back(SEQ_END),
                Some(BlockMap(_, AfterColon) | BlockMapExp(_, AfterColon)) => {
                    self.push_empty_token();
                    self.tokens.push_back(MAP_END);
                }
                Some(BlockMap(_, _) | BlockMapExp(_, _)) => self.tokens.push_back(MAP_END),
                _ => {}
            }
        }
    }

    fn pop_states_in_err(&mut self, unwind: usize, tokens: &mut Vec<usize>) {
        if unwind == 0 || self.curr_state() == DocBlock {
            return;
        }
        for _ in 0..unwind {
            match self.pop_state() {
                Some(BlockSeq(_, _)) => tokens.push(SEQ_END),
                Some(BlockMap(_, _) | BlockMapExp(_, _)) => tokens.push(MAP_END),
                _ => {}
            }
        }
    }

    fn unwind_to_root_start<B, R: Reader<B>>(&mut self, reader: &mut R) {
        let pos = reader.col();
        self.pop_block_states(self.stack.len().saturating_sub(1));
        self.tokens.push_back(DOC_END);
        if pos != 0 {
            self.push_error(ExpectedIndentDocStart {
                actual: pos,
                expected: 0,
            });
        }
        self.tags.clear();
        self.set_block_state(PreDocStart, reader.line());
    }

    fn unwind_to_root_end<B, R: Reader<B>>(&mut self, reader: &mut R) {
        let col = reader.col();
        self.pop_block_states(self.stack.len().saturating_sub(1));
        if col != 0 {
            self.push_error(UnexpectedIndentDocEnd {
                actual: col,
                expected: 0,
            });
        }
        self.tags.clear();
        self.set_block_state(AfterDocBlock, reader.line());
    }

    fn fetch_exp_block_map_key<B, R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        let indent = reader.col();
        self.last_map_line = Some(reader.line());
        reader.consume_bytes(1);
        reader.skip_space_and_tab_detect(&mut self.has_tab);
        self.emit_meta_nodes();
        match curr_state {
            DocBlock => {
                let state = BlockMapExp(indent, BeforeKey);
                self.push_block_state(state, reader.line());
                self.tokens.push_back(MAP_START);
            }
            BlockMapExp(prev_indent, BeforeColon) if prev_indent == indent => {
                self.push_empty_token();
                self.set_map_state(BeforeKey);
            }
            _ => {}
        }
    }

    fn fetch_tag<B, R: Reader<B>>(&mut self, reader: &mut R) {
        pub use LexerToken::*;
        self.update_col(reader);
        let (err, start, mid, end) = reader.read_tag();
        if let Some(err) = err {
            self.push_error(err);
        } else {
            let lines = self.skip_separation_spaces(reader);
            if lines.num_breaks == 0 {
                self.prev_tag = Some((start, mid, end));
            } else {
                self.emit_meta_nodes();
                self.tokens.push_back(TAG_START);
                self.tokens.push_back(start);
                self.tokens.push_back(mid);
                self.tokens.push_back(end);
            }
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
        let mut ends_with = ScalarEnd::Plain;
        self.update_col(reader);

        let curr_state = curr_state;
        let has_tab = self.has_tab;
        let scalar_line = reader.line();

        let scalar = self.get_plain_scalar(reader, curr_state, &mut ends_with);

        let is_key = ends_with == ScalarEnd::Map
            || (ends_with != ScalarEnd::Plain
                && matches!(reader.peek_chars(), [b':', x, ..] if is_white_tab_or_break(*x)))
                && matches!(
                    curr_state,
                    BlockMap(_, BeforeKey) | BlockSeq(_, _) | DocBlock
                );

        let scalar_type = if is_key {
            ScalarEnd::Map
        } else if matches!(curr_state, BlockSeq(_, BeforeElem | BeforeFirst)) {
            ScalarEnd::Seq
        } else {
            ScalarEnd::Plain
        };
        self.pop_other_states(scalar.col_start, scalar_type);

        self.process_block_scalar(reader, curr_state, is_key, scalar, has_tab, scalar_line);
    }

    fn pop_other_states(&mut self, scalar_start: u32, scalar_type: ScalarEnd) {
        let find_unwind = self
            .stack
            .iter()
            .rposition(|state| state.matches(scalar_start, scalar_type))
            .map(|x| self.stack.len() - x - 1);
        if let Some(unwind) = find_unwind {
            self.pop_block_states(unwind);
        }
    }

    fn process_colon_block<B, R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        let indent = self.indent();
        let colon_pos = reader.col();
        let col = self.col_start.unwrap_or(colon_pos);
        reader.consume_bytes(1);

        if colon_pos == 0 && curr_state == DocBlock {
            let state = BlockMap(0, AfterColon);
            self.push_block_state(state, reader.line());
            self.tokens.push_back(MAP_START);
            self.push_empty_token();
        } else if matches!(curr_state, BlockMap(ind, BeforeKey) if colon_pos == ind) {
            self.push_empty_token();
            self.set_map_state(AfterColon);
        } else if matches!(curr_state, BlockMap(ind, AfterColon) if colon_pos == ind )
            && !self.prev_scalar.is_empty()
        {

            self.emit_meta_nodes();
            self.set_map_state(AfterColon);
            self.tokens.extend(take(&mut self.prev_scalar.spans));
            self.push_empty_token();
        } else if matches!(curr_state, BlockMapExp(_, _) if colon_pos != indent ) {
            self.push_error(ExpectedIndent {
                actual: col,
                expected: indent,
            });
            self.next_substate();
        } else if !self.prev_scalar.is_empty()
            && matches!(curr_state, BlockMap(ind, AfterColon) if ind == self.prev_scalar.col_start)
        {
            self.push_empty_token();
        } else if matches!(curr_state, BlockMap(ind, BeforeColon) if col == ind)
            || matches!(curr_state, BlockMapExp(ind, _) if colon_pos == ind )
            || matches!(curr_state, BlockMap(_, _) if col > indent)
        {
            self.next_substate();
        } else if let Some(unwind) = self.find_matching_state(
                col,
                |state, indent| matches!(state, BlockMap(ind, _) | BlockMapExp(ind, _) if ind == indent),
            ) {
                self.pop_block_states(unwind);
                self.next_substate();
            } else {
                self.push_block_state(BlockMap(col, AfterColon), reader.line());
            }

        if !self.prev_scalar.is_empty() {
            self.emit_meta_nodes();
            self.set_map_state(AfterColon);
            self.tokens.extend(take(&mut self.prev_scalar.spans));
        }
    }

    fn process_block_seq<B, R: Reader<B>>(&mut self, reader: &mut R, curr_state: LexerState) {
        let indent = reader.col();
        let expected_indent = self.indent();
        reader.consume_bytes(1);

        if !matches!(curr_state, BlockMapExp(_, _)) && self.last_map_line == Some(reader.line()) {
            self.push_error(SequenceOnSameLineAsKey);
        }

        let new_seq = match curr_state {
            DocBlock => true,
            BlockSeq(ind, _) if indent > ind => true,
            BlockSeq(ind, _) if indent == ind => false,
            _ => {
                if let Some(last_seq) = self.stack.iter().rposition(|x| matches!(x, BlockSeq(_, _)))
                {
                    if let Some(unwind) = self.find_matching_state(
                        indent,
                        |state, indent| matches!(state, BlockSeq(ind, _) if ind == indent),
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
                    true
                }
            }
        };

        if new_seq {
            if self.has_tab {
                self.push_error(ErrorType::TabsNotAllowedAsIndentation);
            }
            if self.prev_anchor.is_some() && !self.had_anchor {
                self.push_error(InvalidAnchorDeclaration);
            }
            self.next_substate();
            self.emit_meta_nodes();
            self.push_block_state(
                BlockSeq(self.col_start.unwrap_or(indent), BeforeFirst),
                reader.line(),
            );
            self.tokens.push_back(SEQ_START);
        } else if matches!(curr_state, BlockSeq(_, BeforeFirst)) {
            self.push_empty_token();
        } else {
            self.next_seq_substate();
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

    fn get_plain_scalar<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        curr_state: LexerState,
        ends_with: &mut ScalarEnd,
    ) -> NodeSpans {
        let mut curr_indent = match curr_state {
            // BlockMapExp(ind, _) => ind,
            _ => reader.col(),
        };
        let start_line = reader.line();
        let mut end_line = reader.line();
        let mut tokens = Vec::with_capacity(10);
        tokens.push(SCALAR_PLAIN);
        let mut offset_start = None;
        let in_flow_collection = curr_state.in_flow_collection();
        let mut had_comment = false;
        let mut num_newlines = 0;
        let scalar_start = self.scalar_start(curr_state, reader.col());
        let scalar_limit = match curr_state {
            BlockMapExp(x, _) | BlockSeq(x, _) => x,
            _ => scalar_start,
        };
        let last_indent = self.indent();
        let key_type = curr_state.get_key_type();
        let mut error = None;

        loop {
            let had_comm = had_comment;

            let (start, end, consume) =
                reader.read_plain_one_line(offset_start, &mut had_comment, in_flow_collection);

            if had_comm {
                if curr_state == DocBlock {
                    tokens.push(DOC_END);
                    tokens.push(ERROR_TOKEN);
                    tokens.push(SCALAR_PLAIN);
                    tokens.push(start);
                    tokens.push(end);
                    self.errors.push(UnexpectedScalarAtNodeEnd);
                    self.set_block_state(AfterDocBlock, reader.line());
                    break;
                }

                self.push_error_token(UnexpectedCommentInScalar, &mut tokens);
                tokens.push(SCALAR_PLAIN);
                num_newlines = 0;
            }

            if key_type == KeyType::NotKey && start_line != reader.line() {
                let offset = reader.count_whitespace_from(consume);
                if reader.peek_byte_at(offset).map_or(false, |c| c == b':') {
                    if !self.has_tab && reader.col() > self.last_block_indent.unwrap_or(0) {
                        self.prepend_error_token(ErrorType::NestedMappingsNotAllowed, &mut tokens);
                    }
                    *ends_with = ScalarEnd::Plain;
                    break;
                }
            }

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
            reader.consume_bytes(consume);

            end_line = reader.line();
            let mut multliline_comment = false;

            if reader.peek_byte().map_or(false, is_white_tab_or_break) {
                let folded_newline = self.skip_separation_spaces(reader);
                multliline_comment = folded_newline.has_comment;

                if reader.col() >= last_indent {
                    num_newlines = folded_newline.num_breaks as usize;
                }
                reader.skip_space_tab();
                if multliline_comment {
                    had_comment = true;
                }
                if folded_newline.has_tab {
                    self.has_tab = true;
                }
                curr_indent = reader.col();
            }

            let chr = reader.peek_byte_at(0).unwrap_or(b'\0');
            let same_line = reader.line() == end_line;

            if chr == b'?'
                && matches!(key_type, KeyType::ComplexKey)
                && matches!(curr_state, BlockMapExp(indent, _) if reader.col() == indent)
            {
                break;
            }
            if chr == b'-' && matches!(curr_state, BlockSeq(indent, _) if reader.col() > indent) {
                offset_start = Some(reader.pos());
            } else if (in_flow_collection && is_flow_indicator(chr)) || chr == b':' || chr == b'-' {
                if chr == b':' && same_line {
                    ends_with.set_to(chr);
                }
                break;
            } else if reader.eof() || reader.peek_chars() == b"..." || !multliline_comment && self.find_matching_state(
                curr_indent,
                |state, indent| matches!(state, BlockMap(ind_col, _)| BlockSeq(ind_col, _) | BlockMapExp(ind_col, _) if ind_col == indent)
            ).is_some() {
                break;
            } else if curr_indent < scalar_limit && start_line != reader.line() {
                // if plain scalar is less indented than previous
                // It can be
                // a) Part of BlockMap so we must break
                // b) An error outside of block map
                // c) Flow state
                // However not important for first line.
                match curr_state {
                    DocBlock => {
                        self.read_line(reader);
                        error = Some(ExpectedIndent {
                            actual: curr_indent,
                            expected: scalar_start,
                        });
                    }
                    BlockMap(indent, _) | BlockMapExp(indent, _) => {
                        self.read_line(reader);
                        error = Some(ExpectedIndent {
                            actual: curr_indent,
                            expected: indent,
                        });
                    }
                    FlowMap(_) | FlowSeq  if last_indent < reader.col() => {
                        continue;
                    }
                    _ => {}
                }
                break;
            }
        }
        let is_multiline = end_line != start_line;
        if let Some(err) = error {
            self.pop_states_in_err(1, &mut tokens);
            self.push_error_token(err, &mut tokens);
        }
        tokens.push(ScalarEnd as usize);
        NodeSpans {
            col_start: scalar_start,
            is_multiline,
            spans: tokens,
        }
    }

    fn scalar_start(&mut self, curr_state: LexerState, curr_col: u32) -> u32 {
        match curr_state {
            BlockSeq(_, _) | BlockMap(_, BeforeColon | AfterColon) | DocBlock => {
                self.col_start.unwrap_or(curr_col)
            }
            _ => curr_col,
        }
    }

    #[inline]
    fn read_line<B, R: Reader<B>>(&mut self, reader: &mut R) -> (usize, usize) {
        let line = reader.read_line();
        self.col_start = None;
        line
    }

    #[must_use]
    pub const fn get_default_namespace(namespace: &[u8]) -> Option<Cow<'static, [u8]>> {
        match namespace {
            b"!!" => Some(Cow::Borrowed(b"tag:yaml.org,2002:")),
            b"!" => Some(Cow::Borrowed(b"!")),
            _ => None,
        }
    }

    #[inline]
    pub fn curr_state(&self) -> LexerState {
        *self.stack.last().unwrap_or(&LexerState::default())
    }

    #[inline]
    pub fn prev_state(&self) -> LexerState {
        *self
            .stack
            .iter()
            .rev()
            .nth(1)
            .unwrap_or(&LexerState::default())
    }

    #[inline]
    pub fn set_block_state(&mut self, state: LexerState, read_line: u32) {
        match self.stack.last_mut() {
            Some(x) => *x = state,
            None => self.push_block_state(state, read_line),
        }
    }

    #[inline]
    pub fn set_state(&mut self, state: LexerState) {
        match self.stack.last_mut() {
            Some(x) => *x = state,
            None => self.stack.push(state),
        }
    }

    #[inline]
    fn set_map_state(&mut self, map_state: MapState) {
        if let Some(BlockMap(_, state) | BlockMapExp(_, state)) = self.stack.last_mut() {
            *state = map_state;
        }
    }

    #[inline]
    fn next_substate(&mut self) {
        let new_state = match self.stack.last() {
            Some(BlockMap(ind, state)) => BlockMap(*ind, state.next_state()),
            Some(BlockMapExp(ind, AfterColon)) => BlockMap(*ind, BeforeKey),
            Some(BlockMapExp(ind, state)) => BlockMapExp(*ind, state.next_state()),
            Some(BlockSeq(ind, state)) => BlockSeq(*ind, state.next_state()),
            _ => return,
        };
        if let Some(x) = self.stack.last_mut() {
            *x = new_state;
        };
    }

    #[inline]
    fn next_seq_substate(&mut self) {
        if let Some(BlockSeq(_, state)) = self.stack.last_mut() {
            *state = state.next_state();
        };
    }

    #[inline]
    pub fn pop_token(&mut self) -> Option<usize> {
        self.tokens.pop_front()
    }

    #[inline]
    pub fn indent(&self) -> u32 {
        match self.last_block_indent {
            None => 0,
            Some(x) if self.curr_state().in_flow_collection() => x + 1,
            Some(x) => x,
        }
    }

    #[inline]
    pub fn tokens(self) -> VecDeque<usize> {
        self.tokens
    }

    #[inline]
    pub fn peek_token(&mut self) -> Option<usize> {
        self.tokens.front().copied()
    }

    #[inline]
    pub fn peek_token_next(&mut self) -> Option<usize> {
        self.tokens.get(1).copied()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    #[inline]
    fn update_col<B, R: Reader<B>>(&mut self, reader: &R) -> u32 {
        if let Some(x) = self.col_start {
            x
        } else {
            let col = reader.col();
            self.col_start = Some(col);
            col
        }
    }

    fn process_single_quote_block<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        curr_state: LexerState,
    ) {
        let has_tab = self.has_tab;
        let scalar_line = reader.line();
        let scalar = self.process_single_quote(reader, false);
        reader.skip_space_tab();

        let ends_with = reader.peek_byte().map_or(false, |chr| chr == b':');

        self.process_block_scalar(reader, curr_state, ends_with, scalar, has_tab, scalar_line);
    }

    fn read_block_scalar<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        literal: bool,
        //TODO remove  _curr_state
        _curr_state: LexerState,
        block_indent: u32,
    ) -> Vec<usize> {
        let mut chomp = ChompIndicator::Clip;
        let mut tokens = Vec::with_capacity(8);
        reader.consume_bytes(1);

        let token = if literal {
            ScalarLit as usize
        } else {
            ScalarFold as usize
        };

        tokens.push(token);

        let mut new_lines = 0;
        let mut prev_indent = 0;

        let mut state = self.get_initial_indent(reader, block_indent, &mut prev_indent, &mut chomp);
        if reader.eof() {
            tokens.push(ScalarEnd as usize);
            return tokens;
        }
        loop {
            if reader.eof() || reader.peek_stream_ending() {
                break;
            }

            state = match state {
                LiteralStringState::AutoIndentation => self.process_autoindentation(
                    reader,
                    &mut prev_indent,
                    &mut new_lines,
                    &mut tokens,
                ),
                LiteralStringState::Indentation(indent) => {
                    if reader.is_empty_newline() {
                        self.process_trim(reader, indent, &mut new_lines, &mut tokens)
                    } else {
                        self.process_indentation(
                            reader,
                            indent,
                            (literal, chomp),
                            &mut prev_indent,
                            &mut new_lines,
                            &mut tokens,
                        )
                    }
                }

                LiteralStringState::Comment => self.process_comment(reader),
                LiteralStringState::TabError => {
                    self.skip_separation_spaces(reader);
                    if !(reader.eof() || reader.peek_stream_ending()) {
                        self.prepend_error_token(ErrorType::InvalidScalarIndent, &mut tokens);
                    }

                    break;
                }
                LiteralStringState::End => break,
            };
        }

        match chomp {
            ChompIndicator::Keep => {
                tokens.push(NEWLINE);
                tokens.push(new_lines as usize);
            }
            ChompIndicator::Clip if new_lines > 0 => {
                tokens.push(NEWLINE);
                tokens.push(1);
            }
            _ => {}
        }
        tokens.push(ScalarEnd as usize);

        tokens
    }

    fn get_initial_indent<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        block_indent: u32,
        prev_indent: &mut u32,
        chomp: &mut ChompIndicator,
    ) -> LiteralStringState {
        let (amount, state) = match reader.peek_chars() {
            [_, b'0', ..] | [b'0', _, ..] => {
                self.push_error(ExpectedChompBetween1and9);
                reader.consume_bytes(2);
                return LiteralStringState::End;
            }
            [b'-', len, ..] | [len, b'-', ..] if matches!(len, b'1'..=b'9') => {
                *chomp = ChompIndicator::Strip;
                (
                    2,
                    LiteralStringState::from_indentation(block_indent + u32::from(len - b'0')),
                )
            }
            [b'+', len, ..] | [len, b'+', ..] if matches!(len, b'1'..=b'9') => {
                *chomp = ChompIndicator::Keep;
                (
                    2,
                    LiteralStringState::from_indentation(block_indent + u32::from(len - b'0')),
                )
            }
            [b'-', ..] => {
                *chomp = ChompIndicator::Strip;
                (1, LiteralStringState::AutoIndentation)
            }
            [b'+', ..] => {
                *chomp = ChompIndicator::Keep;
                (1, LiteralStringState::AutoIndentation)
            }
            [len, ..] if matches!(len, b'1'..=b'9') => (
                1,
                LiteralStringState::from_indentation(block_indent + u32::from(len - b'0')),
            ),
            _ => (0, LiteralStringState::AutoIndentation),
        };
        reader.consume_bytes(amount);
        if let LiteralStringState::Indentation(x) = state {
            *prev_indent = x;
        }

        // allow comment in first line of block scalar
        reader.skip_space_tab();
        match reader.peek_byte() {
            Some(b'#' | b'\r' | b'\n') => {
                self.read_line(reader);
            }
            Some(chr) => {
                self.read_line(reader);
                self.push_error(UnexpectedSymbol(chr as char));
                return LiteralStringState::End;
            }
            _ => {}
        }

        state
    }

    fn process_autoindentation<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        prev_indent: &mut u32,
        new_lines: &mut u32,
        tokens: &mut Vec<usize>,
    ) -> LiteralStringState {
        let mut max_prev_indent = 0;
        loop {
            if reader.eof() {
                return LiteralStringState::End;
            }

            let newline_indent = reader.count_spaces();
            self.has_tab = matches!(reader.peek_byte_at(newline_indent as usize), Some(b'\t'));

            let newline_is_empty = reader.is_empty_newline();
            if newline_is_empty && max_prev_indent < newline_indent {
                max_prev_indent = newline_indent;
            }
            if max_prev_indent > newline_indent {
                self.prepend_error_token(SpacesFoundAfterIndent, tokens);
            }
            if !newline_is_empty {
                *prev_indent = newline_indent;
                if *new_lines > 0 {
                    tokens.push(NEWLINE);
                    tokens.push(*new_lines as usize);
                    *new_lines = 0;
                }
                return LiteralStringState::Indentation(newline_indent);
            }
            *new_lines += 1;
            self.read_line(reader);
        }
    }

    fn process_trim<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        indent: u32,
        new_lines: &mut u32,
        tokens: &mut Vec<usize>,
    ) -> LiteralStringState {
        loop {
            if reader.eof() {
                return LiteralStringState::End;
            }
            let newline_indent: u32 = reader.count_spaces();
            let newline_is_empty = reader.is_empty_newline();
            if !newline_is_empty {
                return LiteralStringState::Indentation(indent);
            }
            if newline_indent > indent {
                reader.consume_bytes(indent as usize);
                if reader.peek_byte_is(b'#') {
                    return LiteralStringState::Comment;
                }
                let (start, end) = self.read_line(reader);
                if start != end {
                    tokens.push(NEWLINE);
                    tokens.push(*new_lines as usize);
                    tokens.push(start);
                    tokens.push(end);
                    *new_lines = 1;
                }
            } else {
                *new_lines += 1;
                self.read_line(reader);
            }
        }
    }

    fn process_comment<B, R: Reader<B>>(&mut self, reader: &mut R) -> LiteralStringState {
        loop {
            if reader.eof() {
                return LiteralStringState::End;
            }
            let space_offset = reader.count_spaces() as usize;
            if reader.peek_byte_at(space_offset) != Some(b'#') {
                return LiteralStringState::End;
            }
            self.read_line(reader);
        }
    }

    fn process_indentation<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        indent: u32,
        lit_chomp: (bool, ChompIndicator),
        prev_indent: &mut u32,
        new_lines: &mut u32,
        tokens: &mut Vec<usize>,
    ) -> LiteralStringState {
        let curr_indent = reader.count_spaces();
        let mut next_state = next_process_indentation(
            curr_indent,
            indent,
            reader,
            lit_chomp,
            new_lines,
            prev_indent,
        );
        match next_state {
            v @ (LiteralStringState::Comment | LiteralStringState::End) => return v,
            x => x,
        };

        reader.consume_bytes(indent as usize);
        let (start, end, _) = reader.get_read_line();
        if start == end {
            *new_lines += 1;
        } else {
            if *new_lines > 0 {
                if !lit_chomp.0 && *prev_indent == curr_indent && curr_indent == indent {
                    tokens.push(NewLine as usize);
                    tokens.push(new_lines.saturating_sub(1) as usize);
                } else {
                    tokens.push(NewLine as usize);
                    tokens.push(*new_lines as usize);
                }
            }
            match self.last_block_indent {
                Some(i) if i >= curr_indent => {
                    *new_lines = 0;
                    if reader.peek_byte_is(b'\t') {
                        self.has_tab = true;
                        next_state = LiteralStringState::TabError;
                    } else {
                        self.prepend_error_token(
                            ErrorType::ExpectedIndent {
                                actual: i,
                                expected: curr_indent,
                            },
                            tokens,
                        );
                        next_state = LiteralStringState::End;
                    }
                }
                _ => {
                    *prev_indent = curr_indent;
                    tokens.push(start);
                    tokens.push(end);
                    self.read_line(reader);
                    *new_lines = 1;
                }
            };
        }

        next_state
    }
}

fn next_process_indentation<B, R: Reader<B>>(
    curr_indent: u32,
    indent: u32,
    reader: &mut R,
    lit_chomp: (bool, ChompIndicator),
    new_lines: &mut u32,
    prev_indent: &mut u32,
) -> LiteralStringState {
    if curr_indent < indent {
        if reader.peek_byte_at(curr_indent as usize) == Some(b'#') {
            return LiteralStringState::Comment;
        }

        match lit_chomp {
            (_, ChompIndicator::Strip) => {
                *new_lines = 0;
            }
            (true, _) => {
                *prev_indent = curr_indent;
            }
            (false, ChompIndicator::Keep) => {
                *new_lines += 1;
            }

            _ => {}
        }

        return LiteralStringState::End;
    }
    LiteralStringState::Indentation(indent)
}

#[inline]
fn is_skip_colon_space(scalar_spans: &NodeSpans) -> bool {
    match scalar_spans.spans.first() {
        Some(&SCALAR_DQUOTE | &SCALAR_QUOTE | &SEQ_START_EXP | &MAP_START | &MAP_START_EXP) => true,
        _ => false,
    }
}

// TODO enable
// #[inline]
fn push_empty(tokens: &mut Vec<usize>) {
    tokens.push(SCALAR_PLAIN);
    tokens.push(SCALAR_END);
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

fn emit_newspace(tokens: &mut Vec<usize>, newspaces: &mut Option<usize>) {
    if let Some(newspace) = newspaces.take() {
        tokens.push(NewLine as usize);
        tokens.push(newspace);
    }
}

const DOC_END: usize = usize::MAX;
const DOC_END_EXP: usize = usize::MAX - 1;
const DOC_START: usize = usize::MAX - 2;
const DOC_START_EXP: usize = usize::MAX - 3;
const MAP_END: usize = usize::MAX - 4;
const MAP_START_EXP: usize = usize::MAX - 5;
const MAP_START: usize = usize::MAX - 6;
const SEQ_END: usize = usize::MAX - 7;
const SEQ_START_EXP: usize = usize::MAX - 8;
const SEQ_START: usize = usize::MAX - 9;
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
/// [`LexerToken`] used to Lex YAML files
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
    SequenceStart = SEQ_START_EXP,
    /// Start of a sequence token, e.g. `[` in
    /// ```yaml
    ///  [a, b, c]
    /// #^-- start of sequence
    /// ```
    SequenceStartImplicit = SEQ_START,
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
    MappingStart = MAP_START_EXP,
    /// Start of a map  token, e.g. `{` in
    /// ```yaml
    ///   [a]: 3
    /// #^-- start of mapping
    /// ```
    MappingStartImplicit = MAP_START,
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
    /// This method transforms a [`LexerToken`] into a [`DirectiveType`]
    ///
    /// It's UB to call on any [`LexerToken`] that isn't [`DirectiveTag`], [`DirectiveYaml`], or  [`DirectiveReserved`].
    #[inline]
    pub(crate) unsafe fn to_yaml_directive(self) -> DirectiveType {
        match self {
            DirectiveTag => DirectiveType::Tag,
            DirectiveYaml => DirectiveType::Yaml,
            DirectiveReserved => DirectiveType::Reserved,
            _ => unreachable_unchecked(),
        }
    }

    ///
    /// This method transforms a [`LexerToken`] into a [`ScalarType`]
    ///
    /// It's UB to call on any [`LexerToken`] that isn't [`ScalarPlain`], [`Mark`], [`ScalarFold`], [`ScalarLit`],
    /// [`ScalarSingleQuote`], [`ScalarDoubleQuote`].
    #[inline]
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
            MAP_START_EXP => MappingStart,
            MAP_START => MappingStartImplicit,
            SEQ_START => SequenceStartImplicit,
            SEQ_END => SequenceEnd,
            SEQ_START_EXP => SequenceStart,
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

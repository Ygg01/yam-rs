#![allow(clippy::match_like_matches_macro)]

use std::borrow::Cow;
use std::collections::{HashMap, VecDeque};
use std::hint::unreachable_unchecked;
use std::mem::take;
use std::vec;

use LexerState::PreDocStart;

use crate::tokenizer::lexer::LexerState::*;
use crate::tokenizer::lexer::LexerToken::*;
use crate::tokenizer::lexer::MapState::*;
use crate::tokenizer::lexer::PropType::*;
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
    space_indent: Option<u32>,
    last_block_indent: Option<u32>,
    last_map_line: Option<u32>,
    prev_prop: PropSpans,
    
    has_tab: bool,
    stack: Vec<LexerState>,
}

#[derive(Clone, Copy)]
pub(crate) struct SeparationSpaceInfo {
    num_breaks: u32,
    space_indent: u32,
    has_comment: bool,
    has_tab: bool,
}

#[derive(Clone, Default)]
pub(crate) struct NodeSpans {
    col_start: u32,
    line_start: u32,
    is_multiline: bool,
    spans: Vec<usize>,
}

impl NodeSpans {
    pub fn from_reader<B, R: Reader<B>>(reader: &R) -> NodeSpans {
        NodeSpans {
            col_start: reader.col(),
            line_start: reader.line(),
            is_multiline: false,
            spans: vec![],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    pub fn merge_spans(&mut self, other: NodeSpans) {
        if other.is_empty() {
            return;
        }
        if self.spans.is_empty() {
            *self = other;
        } else {
            self.spans.extend(other.spans);
        }
    }

    pub fn merge_tokens(&mut self, other: Vec<usize>) {
        if other.is_empty() {
            return;
        }
        if self.spans.is_empty() {
            self.spans = other;
        } else {
            self.spans.extend(other);
        }
    }

    pub fn push(&mut self, token: usize) {
        self.spans.push(token);
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum PropType {
    #[default]
    Unset,
    Tag,
    Anchor,
    TagAndAnchor,
}

impl PropType {
    pub(crate) fn merge_prop_type(&self, other: PropType) -> Result<PropType, PropType> {
        match (&self, &other) {
            (Unset, _) => Ok(other),
            (_, Unset) => Ok(*self),
            (Tag, Anchor) | (Anchor, Tag) => Ok(TagAndAnchor),
            (Tag, Tag | TagAndAnchor) | (TagAndAnchor, Tag) => Err(Tag),
            (Anchor, Anchor | TagAndAnchor) | (TagAndAnchor, Anchor) => Err(Anchor),
            (TagAndAnchor, TagAndAnchor) => Err(TagAndAnchor),
        }
    }
}

#[derive(Clone, Default)]
pub(crate) struct PropSpans {
    col_start: u32,
    line_start: u32,
    prop_type: PropType,
    spans: Vec<usize>,
}

impl PropSpans {
    pub fn from_reader<B, R: Reader<B>>(reader: &R) -> PropSpans {
        PropSpans {
            col_start: reader.col(),
            line_start: reader.line(),
            prop_type: PropType::Unset,
            spans: vec![],
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    fn merge_prop(&mut self, other: &mut PropSpans) -> Result<(), PropType> {
        if other.is_empty() {
            return Ok(());
        }
        match self.prop_type.merge_prop_type(other.prop_type) {
            Ok(new_type) => {
                if self.is_empty() {
                    self.line_start = other.line_start;
                    self.col_start = other.col_start;
                }
                self.spans.extend(take(other).spans);
                self.prop_type = new_type;

                Ok(())
            }
            Err(r) => Err(r),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum MapState {
    BeforeFlowComplexKey,
    BeforeBlockComplexKey,
    ExpectComplexValue,
    #[default]
    BeforeFirstKey,
    ExpectKey,
    ExpectValue,
}

impl MapState {
    #[must_use]
    fn next_state(self) -> MapState {
        match self {
            ExpectKey | BeforeFirstKey => ExpectValue,
            BeforeFlowComplexKey | BeforeBlockComplexKey => ExpectComplexValue,
            ExpectComplexValue | ExpectValue => ExpectKey,
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

trait Pusher {
    fn front_push(&mut self, token: usize);
    fn push(&mut self, token: usize);
    fn push_all<T: IntoIterator<Item = usize>>(&mut self, iter: T);
}

impl Pusher for Vec<usize> {
    #[inline]
    fn front_push(&mut self, token: usize) {
        self.insert(0, token);
    }

    #[inline]
    fn push(&mut self, token: usize) {
        self.push(token);
    }

    fn push_all<T: IntoIterator<Item = usize>>(&mut self, iter: T) {
        self.extend(iter);
    }
}

impl Pusher for VecDeque<usize> {
    #[inline]
    fn front_push(&mut self, token: usize) {
        self.push_front(token);
    }

    #[inline]
    fn push(&mut self, token: usize) {
        self.push_back(token);
    }

    fn push_all<T: IntoIterator<Item = usize>>(&mut self, iter: T) {
        self.extend(iter);
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

impl LexerState {
    #[inline]
    pub fn in_flow_collection(self) -> bool {
        match &self {
            FlowSeq | FlowMap(_) => true,
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
        fn $quote<B, R: Reader<B>>(&mut self, reader: &mut R) -> NodeSpans {
            let col_start = reader.col();
            let line_start = reader.line();

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
                    QuoteState::Trim => {
                        self.$trim(reader, &mut start_str, &mut newspaces, &mut spans)
                    }
                    QuoteState::End | QuoteState::Error => break,
                };
            }
            spans.push(ScalarEnd as usize);
            let is_multiline = line_start != reader.line();
            NodeSpans {
                col_start,
                line_start,
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
                prepend_error(ErrorType::UnexpectedEndOfFile, tokens, &mut self.errors);
                QuoteState::Error
            } else {
                QuoteState::Trim
            }
        }

        #[allow(unused_must_use)]
        fn $trim<B, R: Reader<B>>(
            &mut self,
            reader: &mut R,
            start_str: &mut usize,
            newspaces: &mut Option<usize>,
            tokens: &mut Vec<usize>,
        ) -> QuoteState {
            if reader.peek_stream_ending() {
                prepend_error(ErrorType::UnexpectedEndOfStream, tokens, &mut self.errors);
            };
            let indent = self.indent();
            if !matches!(self.curr_state(), DocBlock) && reader.col() <= indent {
                prepend_error(
                    ErrorType::InvalidQuoteIndent {
                        actual: reader.col(),
                        expected: indent,
                    },
                    tokens,
                    &mut self.errors,
                );
            }

            if let Some((match_pos, len)) = reader.$trim_fn(*start_str) {
                emit_token_mut(start_str, match_pos, newspaces, tokens);
                reader.consume_bytes(len);
            } else {
                self.update_newlines(reader, newspaces, start_str);
            }

            match reader.peek_byte() {
                Some(b'\n' | b'\r') => {
                    if let Err(err) = self.update_newlines(reader, newspaces, start_str) {
                        prepend_error(err, tokens, &mut self.errors);
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
                    prepend_error(
                        ErrorType::UnexpectedEndOfFile,
                        &mut self.tokens,
                        &mut self.errors,
                    );
                    QuoteState::Error
                }
            }
        }
    };
}

impl Lexer {
    pub fn fetch_next_token<B, R: Reader<B>>(&mut self, reader: &mut R) {
        let curr_state = self.curr_state();

        match curr_state {
            DocBlock | BlockMap(_, _) | BlockSeq(_, _) => {
                self.fetch_block_node(reader);
            }
            FlowSeq | FlowMap(_) => self.fetch_flow_node(reader),
            PreDocStart => self.fetch_pre_doc(reader),
            AfterDocBlock => self.fetch_after_doc(reader),
            InDocEnd => self.fetch_end_doc(reader),
        }

        if reader.eof() {
            self.stream_end = true;
            self.finish_eof();
        }
    }

    fn fetch_block_node<B, R: Reader<B>>(&mut self, reader: &mut R) {
        let mut tokens = Vec::new();

        let Some(chr) = reader.peek_byte() else {
            self.stream_end = true;
            return;
        };

        let is_doc_end = reader.peek_stream_ending();

        match chr {
            b'.' if is_doc_end => {
                self.pop_block_states(self.stack.len().saturating_sub(1), &mut tokens);
                tokens.push(DOC_END_EXP);
                self.set_state(PreDocStart);
                reader.consume_bytes(3);
                self.last_map_line = Some(reader.line());
            }
            b'-' if is_doc_end => {
                self.pop_block_states(self.stack.len().saturating_sub(1), &mut tokens);
                tokens.push(DOC_END);
                self.set_state(PreDocStart);
            }
            b'#' if reader.col() > 0 => {
                // comment that doesnt have literal
                push_error(
                    MissingWhitespaceBeforeComment,
                    &mut self.tokens,
                    &mut self.errors,
                );
                self.read_line(reader);
            }
            b'%' => {
                push_error(UnexpectedDirective, &mut self.tokens, &mut self.errors);
            }
            chr if is_white_tab_or_break(chr) => {
                self.skip_sep_spaces(reader);
            }
            _ => {
                self.get_block_collection(reader, &mut tokens);
            }
        }

        self.tokens.extend(tokens);
        // We are in Root node and remnants in properties
        if matches!(self.curr_state(), DocBlock) && self.prev_prop.is_empty() {
            self.set_state(AfterDocBlock);
        }
    }

    fn get_block_collection<B, R: Reader<B>>(&mut self, reader: &mut R, tokens: &mut Vec<usize>) {
        if self.process_line_start(reader, tokens) {
            return;
        }

        let mut prop_node = PropSpans::from_reader(reader);
        let mut curr_node = loop {
            let Some(chr) = reader.peek_byte() else {
                self.stream_end = true;
                tokens.extend(take(&mut prop_node).spans);
                return;
            };

            match chr {
                b'&' | b'!' => {
                    if let Err(err) =
                        prop_node.merge_prop(&mut self.process_inline_properties(reader))
                    {
                        push_error(NodeWithTwoProperties(err), tokens, &mut self.errors);
                    }
                }
                b'-' if reader.peek_byte_at(1).map_or(false, is_plain_unsafe)
                    && !prop_node.is_empty()
                    && prop_node.line_start == reader.line() =>
                {
                    push_error(UnexpectedScalarAtNodeEnd, tokens, &mut self.errors);
                    self.process_block_seq(reader, tokens);
                }
                b'{' | b'[' => break self.get_flow_node(reader, &mut prop_node),
                b'|' => break self.process_block_literal(reader, true),
                b'>' => break self.process_block_literal(reader, false),
                b' ' | b'\t' | b'\n' | b'\r' => {
                    if self
                        .skip_sep_spaces(reader)
                        .map_or(false, |info| info.num_breaks > 0)
                    {
                        match self.curr_state() {
                            BlockMap(ind, ExpectValue) | BlockSeq(ind, _)
                                if prop_node.col_start <= ind =>
                            {
                                push_error(
                                    ExpectedIndent {
                                        actual: prop_node.col_start,
                                        expected: ind,
                                    },
                                    tokens,
                                    &mut self.errors,
                                );
                            }
                            _ => {}
                        }
                        self.merge_prop(&mut prop_node, tokens);
                    }
                    continue;
                }
                _ => break self.get_scalar_node(reader, &mut false),
            };
        };

        let merge = self.merge_prop_with(&mut curr_node, prop_node);

        self.skip_sep_spaces(reader);
        match reader.peek_two_chars() {
            [b':', peek, ..] if is_white_tab_or_break(*peek) => {
                self.process_colon_block(reader, tokens, &mut curr_node);
                if let Err(err) = merge.merge_prop_type(self.prev_prop.prop_type) {
                    push_error(NodeWithTwoProperties(err), tokens, &mut self.errors);
                } else {
                    tokens.extend(take(&mut self.prev_prop).spans);
                }
                tokens.extend(take(&mut curr_node).spans);
            }
            [b':'] => {
                self.process_colon_block(reader, tokens, &mut curr_node);
                if let Err(err) = merge.merge_prop_type(self.prev_prop.prop_type) {
                    push_error(NodeWithTwoProperties(err), tokens, &mut self.errors);
                } else {
                    tokens.extend(take(&mut self.prev_prop).spans);
                }
                tokens.extend(take(&mut curr_node).spans);
            }
            _ if !curr_node.is_empty() => {
                let node_col = curr_node.col_start;
                if let Err(err) = merge.merge_prop_type(self.prev_prop.prop_type) {
                    push_error(NodeWithTwoProperties(err), tokens, &mut self.errors);
                } else {
                    tokens.extend(take(&mut self.prev_prop).spans);
                }

                // scalar found in invalid state
                match self.curr_state() {
                    BlockSeq(_, InSeqElem) => {
                        if let Some(unwind) = self.find_matching_state(
                            |state| matches!(state, BlockSeq(ind, _) | BlockMap(ind, _) if ind <= node_col),
                        ) {
                            self.pop_block_states(unwind, tokens);
                        }
                        push_error(UnexpectedScalarAtNodeEnd, tokens, &mut self.errors);
                        tokens.extend(take(&mut curr_node.spans));
                    }
                    BlockMap(_, ExpectKey) => {
                        if let Some(unwind) = self.find_matching_state(
                            |state| matches!(state, BlockSeq(ind, _) | BlockMap(ind, _) if ind <= node_col),
                        ) {
                            self.pop_block_states(unwind, tokens);
                        }
                        push_error(UnexpectedScalarAtNodeEnd, tokens, &mut self.errors);
                        tokens.extend(take(&mut curr_node.spans));
                    }
                    _ => {
                        tokens.extend(take(&mut curr_node.spans));
                        self.next_substate();
                    }
                }
            }
            _ => {}
        }
    }

    fn merge_prop(&mut self, prop_node: &mut PropSpans, tokens: &mut Vec<usize>) {
        if let Err(err) = self.prev_prop.merge_prop(prop_node) {
            push_error(NodeWithTwoProperties(err), tokens, &mut self.errors);
        }
    }

    fn merge_prop_with(&mut self, curr_node: &mut NodeSpans, prop_node: PropSpans) -> PropType {
        if prop_node.is_empty() {
            return PropType::Unset;
        }
        curr_node.col_start = prop_node.col_start;
        let mut pass = prop_node.spans;
        if matches!(curr_node.spans.first(), Some(&ALIAS)) {
            push_error(
                ErrorType::AliasAndAnchor,
                &mut self.tokens,
                &mut self.errors,
            );
            return PropType::Unset;
        }
        if !curr_node.spans.is_empty() {
            pass.extend(take(&mut curr_node.spans));
        }
        curr_node.spans = pass;
        prop_node.prop_type
    }

    fn process_line_start<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        tokens: &mut Vec<usize>,
    ) -> bool {
        let val = loop {
            let mut node = NodeSpans {
                col_start: reader.col(),
                line_start: reader.line(),
                ..Default::default()
            };

            match reader.peek_two_chars() {
                [b'?', peek, ..] if is_white_tab_or_break(*peek) => {
                    self.fetch_exp_block_map_key(reader, tokens)
                }
                [b'?'] => self.fetch_exp_block_map_key(reader, tokens),
                [b':', peek, ..] if is_white_tab_or_break(*peek) => {
                    self.process_colon_block(reader, tokens, &mut node)
                }
                [b':'] => self.process_colon_block(reader, tokens, &mut node),
                [b'-', peek, ..] if is_white_tab_or_break(*peek) => {
                    self.process_block_seq(reader, tokens)
                }
                [b'-'] => self.process_block_seq(reader, tokens),
                [b' ' | b'\t' | b'\r' | b'\n', ..] => {
                    self.skip_sep_spaces(reader);
                    false
                }
                [] => {
                    self.stream_end = true;
                    break true;
                }
                _ => {
                    break false;
                }
            };
            tokens.extend(node.spans);
        };
        val
    }

    fn fetch_exp_block_map_key<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        tokens: &mut Vec<usize>,
    ) -> bool {
        let indent = reader.col();
        self.last_map_line = Some(reader.line());
        reader.consume_bytes(1);
        self.skip_space_tab(reader);
        match self.curr_state() {
            DocBlock | BlockSeq(_, _) => {
                self.next_substate();
                let state = BlockMap(indent, BeforeBlockComplexKey);
                self.push_block_state(state, reader.line());
                tokens.extend(take(&mut self.prev_prop).spans);
                tokens.push(MAP_START);
                true
            }
            BlockMap(map_indent, _) if indent > map_indent => {
                self.next_substate();
                let state = BlockMap(indent, BeforeBlockComplexKey);
                self.push_block_state(state, reader.line());
                tokens.extend(take(&mut self.prev_prop).spans);
                tokens.push(MAP_START);
                true
            }
            BlockMap(prev_indent, ExpectComplexValue) if prev_indent == indent => {
                push_empty(tokens, &mut self.prev_prop);
                self.set_map_state(BeforeBlockComplexKey);
                false
            }
            BlockMap(prev_indent, _) if prev_indent == indent => {
                self.set_map_state(BeforeBlockComplexKey);
                false
            }
            _ => false,
        }
    }

    fn process_colon_block<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        tokens: &mut Vec<usize>,
        curr_node: &mut NodeSpans,
    ) -> bool {
        let col_pos = reader.col();
        let col_line = reader.line();
        let node_indents = curr_node.col_start;

        let mut is_empty = curr_node.is_empty();
        let is_inline_key = curr_node.line_start == reader.line();
        let matches_exp_map = self.find_matching_state(|x| matches!(x, BlockMap(ind , ExpectComplexValue) if ind == col_pos));
        if is_empty
            && curr_node.col_start == col_pos
            && matches_exp_map.is_some()
        {
            reader.consume_bytes(1);
            if let Some(unwind) = matches_exp_map {
                self.pop_block_states(unwind, &mut curr_node.spans);
            }
            return false;
        } else if curr_node.line_start < col_line && !curr_node.is_multiline {
            self.next_substate();
            return false;
        }
        reader.consume_bytes(1);

        if self.prev_prop.line_start == curr_node.line_start {
            let prop = take(&mut self.prev_prop);
            self.merge_prop_with(curr_node, prop);
            is_empty = curr_node.is_empty();
        }

        if self
            .last_block_indent
            .map_or(false, |indent| node_indents <= indent)
        {
            if let Some(unwind) = self.find_matching_state(
                |state| matches!(state, BlockMap(ind, _) if node_indents >= ind),
            ) {
                self.pop_block_states(unwind, tokens);
                match self.curr_state() {
                    BlockMap(ind, ExpectValue) if ind == node_indents && is_inline_key => {
                        push_empty(tokens, &mut self.prev_prop);
                        self.next_substate();
                    }
                    BlockMap(ind, _) if ind != node_indents => {
                        is_empty = false;
                        push_error(
                            ExpectedIndent {
                                actual: node_indents,
                                expected: ind,
                            },
                            tokens,
                            &mut self.errors,
                        );
                    }
                    _ => {}
                }
            }
        }

        let curr_state = self.curr_state();
        let is_new_map = match curr_state {
            BlockMap(ind, BeforeFirstKey | ExpectKey) if ind == node_indents => false,
            BlockMap(ind, BeforeBlockComplexKey) if ind == col_pos => {
                // push_empty(&mut curr_node.spans, &mut self.prev_prop);
                false
            }
            BlockMap(_, BeforeBlockComplexKey) => is_inline_key,
            BlockMap(ind, ExpectValue) => {
                if is_inline_key {
                    is_empty = curr_node.col_start <= ind || is_empty;
                } else if !is_inline_key
                    && col_line > curr_node.line_start
                    && !curr_node.is_multiline
                {
                    push_empty(&mut curr_node.spans, &mut self.prev_prop);
                }

                curr_node.col_start > ind && is_inline_key
            }
            BlockMap(ind, ExpectComplexValue) => {
                if ind != col_pos {
                    if curr_node.col_start == ind {
                        is_empty = true;
                    } else {
                        push_error(
                            ErrorType::ExpectedIndent {
                                actual: col_pos,
                                expected: ind,
                            },
                            tokens,
                            &mut self.errors,
                        );
                        is_empty = false;
                    }
                } else {
                    is_empty = false;
                }
                false
            }
            BlockSeq(ind, _) if ind == curr_node.col_start => {
                push_error(UnexpectedScalarAtNodeEnd, tokens, &mut self.errors);
                true
            }
            _ => true,
        };
        if is_inline_key {
            if curr_node.is_multiline {
                push_error(ImplicitKeysNeedToBeInline, tokens, &mut self.errors);
            }
            if self.has_tab {
                push_error(
                    ErrorType::TabsNotAllowedAsIndentation,
                    tokens,
                    &mut self.errors,
                );
            }
            if self
                .last_map_line
                .map_or(false, |c| c == curr_node.line_start)
                && !matches!(curr_state, BlockMap(_, BeforeBlockComplexKey))
            {
                push_error(NestedMappingsNotAllowed, tokens, &mut self.errors);
            }
            self.last_map_line = Some(reader.line());
        } else if !is_inline_key
            && !is_new_map
            && !matches!(curr_state, BlockMap(_, BeforeBlockComplexKey))
        {
            push_error(ImplicitKeysNeedToBeInline, tokens, &mut self.errors);
        }

        if is_new_map {
            if curr_node.is_multiline {
                push_error(ImplicitKeysNeedToBeInline, tokens, &mut self.errors);
            }
            if self.prev_prop.line_start != curr_node.line_start {
                tokens.extend(take(&mut self.prev_prop).spans);
            }
            self.next_substate();
            self.push_block_state(BlockMap(curr_node.col_start, ExpectValue), reader.line());
            tokens.push(MAP_START);
        }

        if is_empty {
            push_empty(tokens, &mut self.prev_prop);
        }

        self.set_map_state(ExpectValue);
        is_new_map
    }

    fn process_block_seq<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        tokens: &mut Vec<usize>,
    ) -> bool {
        let curr_state = self.curr_state();
        let indent = reader.col();
        let expected_indent = self.indent();
        reader.consume_bytes(1);

        if !matches!(
            curr_state,
            BlockMap(_, BeforeBlockComplexKey | ExpectComplexValue)
        ) && self.last_map_line == Some(reader.line())
        {
            push_error(SequenceOnSameLineAsKey, tokens, &mut self.errors);
        }

        let new_seq = match curr_state {
            DocBlock => true,
            BlockSeq(ind, _) if indent > ind => true,
            BlockSeq(ind, _) if indent == ind => false,
            _ => {
                if let Some(last_seq) = self.stack.iter().rposition(|x| matches!(x, BlockSeq(_, _)))
                {
                    tokens.extend(take(&mut self.prev_prop).spans);
                    if let Some(unwind) = self.find_matching_state(
                        |state| matches!(state, BlockSeq(ind, _) if ind == indent),
                    ) {
                        self.pop_block_states(unwind, tokens);
                    } else {
                        self.pop_block_states(self.stack.len() - last_seq, tokens);
                        push_error(
                            ExpectedIndent {
                                actual: indent,
                                expected: expected_indent,
                            },
                            tokens,
                            &mut self.errors,
                        );
                    }
                    false
                } else {
                    true
                }
            }
        };

        if new_seq {
            if self.has_tab {
                push_error(
                    ErrorType::TabsNotAllowedAsIndentation,
                    tokens,
                    &mut self.errors,
                );
            }

            self.next_substate();
            self.push_block_state(BlockSeq(indent, BeforeFirst), reader.line());
            if !self.prev_prop.is_empty() && self.prev_prop.line_start != reader.line() {
                tokens.extend(take(&mut self.prev_prop).spans);
            }
            tokens.push(SEQ_START);
        } else if matches!(curr_state, BlockSeq(_, BeforeFirst | BeforeElem)) {
            push_empty(tokens, &mut self.prev_prop);
        } else {
            self.next_seq_substate();
        }
        new_seq
    }

    fn skip_sep_spaces<B, R: Reader<B>>(&mut self, reader: &mut R) -> Option<SeparationSpaceInfo> {
        let sep_opt = self.skip_separation_spaces(reader);
        if let Some(sep_info) = sep_opt {
            self.has_tab = sep_info.has_tab;
            if sep_info.num_breaks > 0 || self.space_indent.is_none() {
                self.space_indent = Some(sep_info.space_indent);
            }
        }
        sep_opt
    }

    fn skip_space_tab<B, R: Reader<B>>(&mut self, reader: &mut R) -> usize {
        let (num_spaces, amount) = reader.count_space_then_tab();
        if amount > 0 {
            if self.space_indent.is_none() {
                self.space_indent = Some(num_spaces);
            }
            self.has_tab = num_spaces != amount;
            reader.consume_bytes(amount as usize);
        }
        amount as usize
    }

    fn consume_spaces<B, R: Reader<B>>(&mut self, reader: &mut R, indent: u32) -> bool {
        let x = reader.count_spaces_till(indent);
        if self.space_indent.is_none() {
            self.space_indent = Some(x as u32);
        }
        reader.consume_bytes(x);
        x == indent as usize
    }

    fn process_block_literal<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        literal: bool,
    ) -> NodeSpans {
        let line_start = reader.line();
        let col_start = reader.col();

        let block_indent = self.indent();
        let tokens = self.read_block_scalar(reader, literal, block_indent);
        let is_multiline = reader.line() != line_start;

        NodeSpans {
            col_start,
            line_start,
            is_multiline,
            spans: tokens,
        }
    }

    fn try_parse_tag<B, R: Reader<B>>(&mut self, reader: &mut R, node: &mut Vec<usize>) -> bool {
        match reader.read_tag() {
            (Some(err), ..) => {
                push_error(err, &mut self.tokens, &mut self.errors);
                false
            }
            (None, start, mid, end) => {
                node.push(TAG_START);
                node.push(start);
                node.push(mid);
                node.push(end);
                true
            }
        }
    }

    fn fetch_flow_node<B, R: Reader<B>>(&mut self, reader: &mut R) {
        let tokens = self.get_flow_node(reader, &mut PropSpans::default());
        self.tokens.extend(tokens.spans);
        if matches!(self.curr_state(), DocBlock) {
            self.set_state(AfterDocBlock);
        }
    }

    fn get_flow_node<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        prop_node: &mut PropSpans,
    ) -> NodeSpans {
        let mut node = NodeSpans::from_reader(reader);
        self.skip_space_tab(reader);
        let Some(chr) = reader.peek_byte() else {
                self.stream_end = true;
                return node;
            };

        if chr == b',' || chr == b']' || chr == b'}' {
            return node;
        }

        let mut is_plain_scalar = false;

        if chr == b'!' || chr == b'&' {
            let prop = self.process_inline_properties(reader);
            self.merge_prop_with(&mut node, prop);
            self.skip_sep_spaces(reader);
        }

        let start_line = reader.line();
        let prev_node = if reader.peek_byte_is(b'[') {
            self.push_state(FlowSeq);
            self.get_flow_seq(reader, prop_node)
        } else if reader.peek_byte_is(b'{') {
            self.get_flow_map(reader, MapState::default(), prop_node)
        } else {
            let mut scal = self.get_scalar_node(reader, &mut is_plain_scalar);
            self.merge_prop_with(&mut scal, take(prop_node));
            scal
        };

        let ws_offset = reader.count_whitespace();
        if reader.peek_byte_at(ws_offset).map_or(false, |c| c == b':')
            && !matches!(self.curr_state(), FlowMap(_))
        {
            reader.consume_bytes(ws_offset);
            if start_line != reader.line() {
                reader.consume_bytes(1);
                node.merge_spans(prev_node);
                push_error(
                    ColonMustBeOnSameLineAsKey,
                    &mut node.spans,
                    &mut self.errors,
                );
                return node;
            }
            let peek_next = reader.peek_byte_at(1).unwrap_or(b'\0');

            if is_plain_scalar && matches!(peek_next, b'[' | b'{' | b'}') {
                push_error(
                    UnexpectedSymbol(peek_next as char),
                    &mut node.spans,
                    &mut self.errors,
                );
                reader.consume_bytes(2);
                node.merge_spans(prev_node);
                return node;
            }

            reader.consume_bytes(1);
            let map_start = if self.curr_state().in_flow_collection() {
                MAP_START_EXP
            } else {
                MAP_START
            };

            node.spans.push(map_start);
            if prev_node.is_empty() {
                node.push(SCALAR_PLAIN);
                node.push(SCALAR_END);
            }
            node.spans.extend(prev_node.spans);
            node.spans
                .extend(self.get_flow_map(reader, ExpectValue, prop_node).spans);
        } else {
            node.merge_spans(prev_node);
        }
        node
    }

    fn get_scalar_node<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        is_plain_scalar: &mut bool,
    ) -> NodeSpans {
        let mut node = NodeSpans::from_reader(reader);

        let Some(chr) = reader.peek_byte() else {
            return node;
        };
        if chr == b'*' {
            let alias = reader.consume_anchor_alias();

            node.spans.push(ALIAS);
            node.spans.push(alias.0);
            node.spans.push(alias.1);
        } else if chr == b':' && self.is_valid_map(reader, &mut node.spans) {
            push_empty(&mut node.spans, &mut PropSpans::default());
            node.line_start = reader.line();
            node.col_start = reader.col();
        } else if matches!(chr, b'-' | b'?')
            && reader.peek_byte_at(1).map_or(false, is_plain_unsafe)
        {
            if self.curr_state().in_flow_collection() {
                reader.consume_bytes(1);
                push_error(InvalidScalarStart, &mut node.spans, &mut self.errors);
            }
        } else if chr == b'\'' {
            node.merge_spans(self.process_single_quote(reader));
        } else if chr == b'"' {
            node.merge_spans(self.process_double_quote(reader));
        } else {
            *is_plain_scalar = true;
            node.merge_spans(self.get_plain_scalar(reader, self.curr_state()));
        }
        node
    }

    fn is_valid_map<B, R: Reader<B>>(&mut self, reader: &mut R, spans: &mut Vec<usize>) -> bool {
        match reader.peek_byte_at(1) {
            Some(b' ' | b'\t' | b',' | b'[' | b']' | b'{' | b'}') => true,
            Some(b'\r' | b'\n') => {
                reader.consume_bytes(1);
                push_error(
                    ErrorType::ColonMustBeOnSameLineAsKey,
                    spans,
                    &mut self.errors,
                );
                false
            }
            _ => false,
        }
    }

    fn process_inline_properties<B, R: Reader<B>>(&mut self, reader: &mut R) -> PropSpans {
        let mut node = PropSpans::from_reader(reader);

        if reader.peek_byte_is(b'&') && try_parse_anchor_alias(reader, ANCHOR, &mut node.spans) {
            node.prop_type = PropType::Anchor;
            let offset = reader.count_space_then_tab().1;
            if reader.peek_byte_is_off(b'!', offset as usize) {
                node.prop_type = PropType::TagAndAnchor;
                self.skip_space_tab(reader);
                self.try_parse_tag(reader, &mut node.spans);
            }
        } else if reader.peek_byte_is(b'!') && self.try_parse_tag(reader, &mut node.spans) {
            node.prop_type = PropType::Tag;
            let offset = reader.count_space_then_tab().1;
            if reader.peek_byte_is_off(b'&', offset as usize) {
                node.prop_type = PropType::TagAndAnchor;
                self.skip_space_tab(reader);
                try_parse_anchor_alias(reader, ANCHOR, &mut node.spans);
            }
        }
        node
    }

    fn get_flow_seq<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        prop_node: &mut PropSpans,
    ) -> NodeSpans {
        let line_begin = reader.line();
        let mut seq_state = BeforeFirst;
        let mut node = NodeSpans::from_reader(reader);
        let mut end_found = false;

        node.col_start = reader.col();

        if !prop_node.is_empty() {
            node.merge_tokens(take(prop_node).spans);
        }
        node.spans.push(SEQ_START_EXP);
        reader.consume_bytes(1);

        let mut prop = PropSpans::default();

        loop {
            let Some(chr) = reader.peek_byte() else {
                self.stream_end = true;
                break;
            };

            let peek_next = reader.peek_byte_at(1).unwrap_or(b'\0');

            if is_white_tab_or_break(chr) {
                let num_ind = self.skip_sep_spaces(reader).map_or(0, |x| x.space_indent);

                if num_ind < self.indent() {
                    push_error(
                        ErrorType::TabsNotAllowedAsIndentation,
                        &mut node.spans,
                        &mut self.errors,
                    );
                }
            } else if chr == b'!' || chr == b'&' {
                prop = self.process_inline_properties(reader);
            } else if chr == b']' {
                reader.consume_bytes(1);
                end_found = true;
                break;
            } else if chr == b'#' {
                push_error(
                    ErrorType::InvalidCommentStart,
                    &mut node.spans,
                    &mut self.errors,
                );
                self.read_line(reader);
            } else if chr == b',' {
                reader.consume_bytes(1);
                if matches!(seq_state, BeforeElem | BeforeFirst) {
                    push_error(
                        ExpectedNodeButFound { found: ',' },
                        &mut node.spans,
                        &mut self.errors,
                    );
                }
                seq_state = BeforeElem;
            } else if chr == b'?' && is_white_tab_or_break(peek_next) {
                node.spans.push(MAP_START_EXP);
                node.merge_spans(self.get_flow_map(
                    reader,
                    MapState::BeforeFlowComplexKey,
                    &mut prop,
                ));
            } else {
                let mut flow_node = self.get_flow_node(reader, &mut prop);
                self.check_flow_indent(flow_node.col_start, &mut flow_node.spans);

                if !flow_node.spans.is_empty() {
                    seq_state.set_next_state();
                    node.spans.extend(flow_node.spans);
                }
            }
        }

        let offset = reader.count_whitespace();
        let prev_state = self.prev_state();
        if reader.peek_byte_at(offset) == Some(b':')
            && matches!(prev_state, FlowSeq | DocBlock)
            && reader.peek_byte_at(1).map_or(true, is_white_tab_or_break)
        {
            reader.consume_bytes(1 + offset);
            self.skip_space_tab(reader);
            if line_begin == reader.line() {
                let map_start = if prev_state.in_flow_collection() {
                    MAP_START_EXP
                } else {
                    MAP_START
                };
                node.spans.insert(0, map_start);
                node.spans.push(SEQ_END);
                node.spans
                    .extend(self.get_flow_map(reader, ExpectValue, prop_node).spans);
                self.pop_state();
            } else {
                push_error(
                    ImplicitKeysNeedToBeInline,
                    &mut node.spans,
                    &mut self.errors,
                );
            }
        } else if end_found {
            self.pop_state();
            node.spans.push(SEQ_END);
        }
        node
    }

    #[inline]
    fn check_flow_indent(&mut self, actual: u32, spans: &mut Vec<usize>) {
        let expected = self.indent();
        if actual < expected {
            push_error(ExpectedIndent { actual, expected }, spans, &mut self.errors);
        }
    }

    fn get_flow_map<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        init_state: MapState,
        prop_node: &mut PropSpans,
    ) -> NodeSpans {
        let mut map_state = init_state;
        let mut node = NodeSpans::from_reader(reader);
        let mut skip_colon_space = false;
        let is_nested = init_state != MapState::default();

        self.push_state(FlowMap(map_state));

        if !prop_node.is_empty() {
            node.merge_tokens(take(prop_node).spans);
        }

        if reader.peek_byte_is(b'{') {
            reader.consume_bytes(1);
            node.push(MAP_START_EXP);
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
                self.skip_sep_spaces(reader);
                continue;
            } else if chr == b'}' {
                reader.consume_bytes(1);
                is_end_emitted = true;
                break;
            } else if chr == b'?' && peek_next.map_or(false, is_white_tab_or_break) {
                reader.consume_bytes(1);
                self.skip_sep_spaces(reader);
                map_state = BeforeFlowComplexKey;
            } else if chr == b',' {
                reader.consume_bytes(1);
                if matches!(map_state, ExpectValue) {
                    push_empty(&mut node.spans, &mut PropSpans::default());
                    map_state = ExpectKey;
                }
                self.skip_sep_spaces(reader);
                continue;
            } else if chr == b':' && (skip_colon_space || peek_next.map_or(true, is_plain_unsafe)) {
                reader.consume_bytes(1);
                if matches!(map_state, ExpectKey) {
                    push_empty(&mut node.spans, &mut PropSpans::default());
                    map_state = ExpectValue;
                    continue;
                }
                self.skip_sep_spaces(reader);
            }

            let scalar_spans = self.get_flow_node(reader, prop_node);
            self.check_flow_indent(scalar_spans.col_start, &mut node.spans);
            skip_colon_space = is_skip_colon_space(&scalar_spans);
            if scalar_spans.is_empty() {
                push_empty(&mut node.spans, &mut PropSpans::default());
            } else {
                node.merge_spans(scalar_spans);
            }
            map_state.set_next_state();
        }
        if matches!(map_state, ExpectValue | ExpectComplexValue) {
            push_empty(&mut node.spans, &mut PropSpans::default());
        }
        if is_end_emitted {
            self.pop_state();
            node.spans.push(MAP_END);
        }
        node
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

    impl_quote!(process_double_quote(SCALAR_DQUOTE), double_quote_trim(get_double_quote_trim, b'"'), double_quote_start(get_double_quote) => double_quote_match);

    #[allow(unused_must_use)]
    fn double_quote_match<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        match_pos: usize,
        start_str: &mut usize,
        newspaces: &mut Option<usize>,
        tokens: &mut Vec<usize>,
    ) -> QuoteState {
        match reader.peek_chars() {
            [b'\\', b' ', ..] => {
                *start_str = reader.consume_bytes(1);
            }
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
            [b'\\', b'u' | b'U' | b'x', ..] => {
                reader.consume_bytes(2);
            }
            [b'\\', x, ..] => {
                if is_valid_escape(*x) {
                    emit_token_mut(start_str, match_pos, newspaces, tokens);
                    reader.consume_bytes(2);
                } else {
                    prepend_error(InvalidEscapeCharacter, tokens, &mut self.errors);
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

    fn update_newlines<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        newspaces: &mut Option<usize>,
        start_str: &mut usize,
    ) -> Result<(), ErrorType> {
        if let Some(x) = self.skip_sep_spaces(reader) {
            *newspaces = Some(x.num_breaks.saturating_sub(1) as usize);
            *start_str = reader.pos();
            if self
                .last_block_indent
                .map_or(false, |indent| indent >= x.space_indent)
            {
                return Err(TabsNotAllowedAsIndentation);
            }
        }
        Ok(())
    }

    fn skip_separation_spaces<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
    ) -> Option<SeparationSpaceInfo> {
        if !reader.peek_byte().map_or(true, is_white_tab_or_break) {
            return None;
        }
        let mut num_breaks = 0u32;
        let mut space_indent = 0u32;
        let mut found_eol = true;
        let mut has_tab = false;
        let mut has_comment = false;

        loop {
            if !reader.peek_byte().map_or(false, is_valid_skip_char) || reader.eof() {
                break;
            }
            let sep = reader.count_space_then_tab();
            space_indent = sep.0;
            let amount = sep.1;
            has_tab = space_indent != amount;
            let is_comment = reader.peek_byte_at(amount as usize).map_or(false, |c| c == b'#');

            if has_comment && !is_comment {
                break;
            }
            if is_comment {
                has_comment = true;
                if amount > 0
                    && !reader
                        .peek_byte_at(amount.saturating_sub(1) as usize)
                        .map_or(false, |c| c == b' ' || c == b'\t' || c == b'\n')
                {
                    push_error(
                        MissingWhitespaceBeforeComment,
                        &mut self.tokens,
                        &mut self.errors,
                    );
                }
                self.read_line(reader);
                found_eol = true;
                num_breaks += 1;
                space_indent = 0;
                continue;
            }

            if reader.read_break().is_some() {
                num_breaks += 1;
                space_indent = 0;
                has_tab = false;
                found_eol = true;
            }

            if found_eol {
                let (indent, amount) = reader.count_space_then_tab();
                space_indent = indent;
                has_tab = indent != amount;
                reader.consume_bytes(amount as usize);
                found_eol = false;
            } else {
                break;
            }
        }
        Some(SeparationSpaceInfo {
            num_breaks,
            space_indent,
            has_comment,
            has_tab,
        })
    }

    #[inline]
    fn pop_state(&mut self) -> Option<LexerState> {
        let pop_state = self.stack.pop();
        if let Some(state) = self.stack.last_mut() {
            match state {
                BlockMap(indent, _) | BlockSeq(indent, _) => {
                    self.last_block_indent = Some(*indent);
                }
                _ => {}
            }
        };
        pop_state
    }

    fn push_state(&mut self, state: LexerState) {
        assert!(!matches!(state, BlockMap(_, _) | BlockSeq(_, _)));
        self.stack.push(state);
    }

    fn push_block_state(&mut self, state: LexerState, read_line: u32) {
        match state {
            BlockMap(indent, _) => {
                self.last_block_indent = Some(indent);
                self.last_map_line = Some(read_line);
            }
            BlockSeq(indent, _) => {
                self.last_block_indent = Some(indent);
            }
            _ => {}
        }
        self.stack.push(state);
    }

    fn pop_block_states<T: Pusher>(&mut self, unwind: usize, spans: &mut T) {
        if unwind == 0 {
            return;
        }
        for _ in 0..unwind {
            if let Some(state @ (BlockMap(_, _) | BlockSeq(_, _))) = self.pop_state() {
                close_block_state(state, &mut self.prev_prop, spans);
            }
        }
    }

    fn find_matching_state<F: Fn(LexerState) -> bool>(&self, f: F) -> Option<usize> {
        self.stack
            .iter()
            .rposition(|state| f(*state))
            .map(|x| self.stack.len() - x - 1)
    }

    fn get_plain_scalar<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        curr_state: LexerState,
    ) -> NodeSpans {
        let col_start = reader.col();
        let mut curr_indent = reader.col();
        let line_start = reader.line();
        let mut end_line = reader.line();
        let mut tokens = Vec::with_capacity(10);
        tokens.push(SCALAR_PLAIN);
        let mut offset_start: Option<usize> = None;
        let in_flow_collection = curr_state.in_flow_collection();
        let mut had_comment = false;
        let mut num_newlines = 0;
        let last_indent = self.indent();

        loop {
            if had_comment {
                if curr_state != DocBlock {
                    push_error(InvalidCommentInScalar, &mut tokens, &mut self.errors);
                }
                break;
            }

            let (start, end, consume) =
                reader.read_plain_one_line(offset_start, &mut had_comment, in_flow_collection);

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

            if reader.peek_byte().map_or(false, is_white_tab_or_break) {
                if let Some(folded_newline) = self.skip_sep_spaces(reader) {
                    if reader.col() >= last_indent {
                        num_newlines = folded_newline.num_breaks as usize;
                    }
                    self.skip_space_tab(reader);
                    if folded_newline.has_comment {
                        had_comment = true;
                    }
                    curr_indent = folded_newline.space_indent;
                }
            }

            let chr = reader.peek_byte_at(0).unwrap_or(b'\0');
            let end_of_stream = reader.eof() || reader.peek_stream_ending();

            if chr == b'-' && matches!(curr_state, BlockSeq(indent, _) if curr_indent > indent)
                || chr == b'?' && matches!(curr_state, BlockMap(indent, BeforeBlockComplexKey) if curr_indent > indent ) {
                offset_start = Some(reader.pos());

            } else if end_of_stream || chr == b'?' || chr == b':' || chr == b'-'
                || (in_flow_collection && is_flow_indicator(chr))
                || self.find_matching_state(|state| matches!(state, BlockMap(ind_col, _)| BlockSeq(ind_col, _) if ind_col >= curr_indent)
                ).is_some()
            {
                break;
            }
        }
        let is_multiline = end_line != line_start;
        tokens.push(ScalarEnd as usize);
        NodeSpans {
            col_start,
            line_start,
            is_multiline,
            spans: tokens,
        }
    }

    #[inline]
    fn read_line<B, R: Reader<B>>(&mut self, reader: &mut R) -> (usize, usize) {
        let line = reader.read_line();
        self.space_indent = None;
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
        if let Some(BlockMap(_, state)) = self.stack.last_mut() {
            *state = map_state;
        }
    }

    #[inline]
    fn next_substate(&mut self) {
        let new_state = match self.stack.last() {
            Some(BlockMap(ind, state)) => BlockMap(*ind, state.next_state()),
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

    fn read_block_scalar<B, R: Reader<B>>(
        &mut self,
        reader: &mut R,
        literal: bool,
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
                    self.skip_sep_spaces(reader);
                    if !(reader.eof() || reader.peek_stream_ending()) {
                        prepend_error(
                            ErrorType::InvalidScalarIndent,
                            &mut tokens,
                            &mut self.errors,
                        );
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
                push_error(
                    ExpectedChompBetween1and9,
                    &mut self.tokens,
                    &mut self.errors,
                );
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
        self.skip_space_tab(reader);
        match reader.peek_byte() {
            Some(b'#' | b'\r' | b'\n') => {
                self.read_line(reader);
            }
            Some(chr) => {
                self.read_line(reader);
                push_error(
                    UnexpectedSymbol(chr as char),
                    &mut self.tokens,
                    &mut self.errors,
                );
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
            self.has_tab = matches!(
                reader.peek_byte_at(newline_indent.saturating_sub(1) as usize),
                Some(b'\t')
            );

            let newline_is_empty = reader.is_empty_newline();
            if newline_is_empty && max_prev_indent < newline_indent {
                max_prev_indent = newline_indent;
            }
            if max_prev_indent > newline_indent {
                prepend_error(SpacesFoundAfterIndent, tokens, &mut self.errors);
            }
            if !newline_is_empty {
                *prev_indent = newline_indent;
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

        self.consume_spaces(reader, indent);
        let (start, end, _) = reader.get_read_line();
        if start == end {
            *new_lines += 1;
        } else {
            match self.last_block_indent {
                Some(i) if i >= curr_indent => {
                    *new_lines = 0;
                    if reader.peek_byte_is(b'\t') {
                        self.has_tab = true;
                        next_state = LiteralStringState::TabError;
                    } else {
                        next_state = LiteralStringState::End;
                    }
                }
                _ => {
                    if *new_lines > 0 {
                        // First empty line after block literal is treated in a special way
                        let is_first_non_empty_line = tokens.len() > 1;

                        // That's on the same identation level as previously detected indentation
                        if is_first_non_empty_line && !lit_chomp.0 && *prev_indent == curr_indent && curr_indent == indent  {
                            tokens.push(NewLine as usize);
                            tokens.push(new_lines.saturating_sub(1) as usize);
                        } else {
                            tokens.push(NewLine as usize);
                            tokens.push(*new_lines as usize);
                        }
                    }
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

    fn fetch_pre_doc<B, R: Reader<B>>(&mut self, reader: &mut R) {
        use DirectiveState::NoDirective;
        use HeaderState::{Bare, Directive, HeaderEnd, HeaderStart};

        let mut header_state = Bare;

        loop {
            let chr = match reader.peek_byte() {
                None => {
                    match header_state {
                        Directive(_) => push_error(
                            ExpectedDocumentEndOrContents,
                            &mut self.tokens,
                            &mut self.errors,
                        ),
                        HeaderStart => {
                            push_empty(&mut self.tokens, &mut PropSpans::default());
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
                    self.skip_sep_spaces(reader);
                    continue;
                }
                Some(x) => x,
            };

            //TODO clear tags when new document
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
                        self.last_map_line = Some(reader.line());
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
                        push_error(UnexpectedSymbol('.'), &mut self.tokens, &mut self.errors);
                    }
                    break;
                }
                (HeaderEnd | HeaderStart, b'.') => {
                    if reader.peek_stream_ending() {
                        reader.consume_bytes(3);
                        push_empty(&mut self.tokens, &mut PropSpans::default());
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
                        push_empty(&mut self.tokens, &mut PropSpans::default());
                        self.tokens.push_back(DOC_END);
                        self.tokens.push_back(DOC_START_EXP);
                    } else {
                        self.set_state(DocBlock);
                        break;
                    }
                }
                (Bare | Directive(_), _) => {
                    if matches!(self.last_map_line, Some(x) if x == reader.line()) {
                        push_error(InvalidScalarAtNodeEnd, &mut self.tokens, &mut self.errors);
                    }
                    self.tokens.push_back(DOC_START);
                    self.set_state(DocBlock);
                    break;
                }
                (HeaderStart, _) => {
                    self.set_state(DocBlock);
                    break;
                }
                (HeaderEnd, _) => {
                    self.skip_space_tab(reader);
                    if reader
                        .peek_byte()
                        .map_or(false, |c| c != b'\r' && c != b'\n' && c != b'#')
                    {
                        push_error(ExpectedDocumentEnd, &mut self.tokens, &mut self.errors);
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
            self.skip_space_tab(reader);
            return match reader.peek_chars() {
                b"1.0" | b"1.1" | b"1.2" | b"1.3" => {
                    directive_state.add_directive();
                    if *directive_state == DirectiveState::TwoDirectiveError {
                        push_error(TwoDirectivesFound, &mut self.tokens, &mut self.errors);
                    }
                    self.tokens.push_back(DIR_YAML);
                    self.tokens.push_back(reader.pos());
                    self.tokens.push_back(reader.consume_bytes(3));
                    self.skip_space_tab(reader);
                    let invalid_char = reader
                        .peek_byte()
                        .map_or(false, |c| c != b'\r' && c != b'\n' && c != b'#');
                    if invalid_char {
                        prepend_error(InvalidAnchorDeclaration, &mut self.tokens, &mut self.errors);
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
        reader.try_read_slice_exact("%TAG");
        self.skip_space_tab(reader);

        if let Ok(key) = reader.read_tag_handle() {
            self.skip_space_tab(reader);
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
                    push_error(
                        UnexpectedIndentDocEnd {
                            actual: col,
                            expected: 0,
                        },
                        &mut self.tokens,
                        &mut self.errors,
                    );
                }
                self.tokens.push_back(DOC_END_EXP);
                self.set_state(InDocEnd);
            }
            b"---" if is_stream_ending => {
                self.tokens.push_back(DOC_END);
                self.set_state(PreDocStart);
            }
            [peek, b'#', ..] if is_white_tab(*peek) => {
                // comment
                self.read_line(reader);
            }
            [b'#', ..] if reader.col() > 0 => {
                // comment that doesnt
                push_error(
                    MissingWhitespaceBeforeComment,
                    &mut self.tokens,
                    &mut self.errors,
                );
                self.read_line(reader);
            }
            [chr, ..] if is_white_tab_or_break(*chr) => {
                self.skip_sep_spaces(reader);
            }
            [b'%', ..] => {
                self.tokens.push_back(DOC_END);
                push_error(UnexpectedEndOfDocument, &mut self.tokens, &mut self.errors);
                self.set_state(PreDocStart);
            }
            [chr, ..] => {
                consume_line = true;
                self.tokens.push_back(DOC_END);
                push_error(
                    UnexpectedSymbol(*chr as char),
                    &mut self.tokens,
                    &mut self.errors,
                );
                self.set_state(PreDocStart);
            }
            [] => {}
        }
        if consume_line {
            self.read_line(reader);
        }
    }

    fn fetch_end_doc<B, R: Reader<B>>(&mut self, reader: &mut R) {
        self.skip_space_tab(reader);
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
                push_error(
                    ExpectedDocumentStartOrContents,
                    &mut self.tokens,
                    &mut self.errors,
                );
            }
            None => {
                self.stream_end = true;
            }
        }
    }

    fn finish_eof(&mut self) {
        for state in self.stack.iter().rev() {
            match *state {
                v @ (BlockSeq(_, _) | BlockMap(_, _)) => {
                    close_block_state(v, &mut self.prev_prop, &mut self.tokens);
                }
                FlowMap(_) => {
                    self.tokens.push(MAP_END);
                }
                FlowSeq => {
                    push_error(
                        MissingFlowClosingBracket,
                        &mut self.tokens,
                        &mut self.errors,
                    );
                    self.tokens.push(SEQ_END);
                }
                DocBlock | AfterDocBlock => {
                    self.tokens.push(DOC_END);
                }
                _ => continue,
            };
        }
    }
}

#[inline]
fn close_block_state<T: Pusher>(state: LexerState, prop: &mut PropSpans, spans: &mut T) {
    match state {
        BlockSeq(_, BeforeFirst | BeforeElem) => {
            push_empty(spans, prop);
            spans.push(SEQ_END);
        }
        BlockSeq(_, _) => {
            spans.push(SEQ_END);
        }
        BlockMap(_, ExpectValue | ExpectComplexValue) => {
            push_empty(spans, prop);
            spans.push(MAP_END);
        }
        BlockMap(_, BeforeBlockComplexKey) => {
            push_empty(spans, prop);
            push_empty(spans, &mut PropSpans::default());
            spans.push(MAP_END);
        }
        BlockMap(_, _) => {
            spans.push(MAP_END);
        }
        _ => {}
    }
}

fn try_parse_anchor_alias<B, R: Reader<B>>(
    reader: &mut R,
    start_token: usize,
    node: &mut Vec<usize>,
) -> bool {
    let anchor = reader.consume_anchor_alias();

    if anchor.0 == anchor.1 {
        false
    } else {
        node.push(start_token);
        node.push(anchor.0);
        node.push(anchor.1);
        true
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

//TODO Enable inlining
// #[inline]
fn push_empty<T: Pusher>(tokens: &mut T, prop: &mut PropSpans) {
    tokens.push_all(take(prop).spans);
    tokens.push(SCALAR_PLAIN);
    tokens.push(SCALAR_END);
}

// #[inline]
fn push_error<T: Pusher>(error: ErrorType, tokens: &mut T, errors: &mut Vec<ErrorType>) {
    tokens.push(ERROR_TOKEN);
    errors.push(error);
}

// #[inline]
fn prepend_error<T: Pusher>(error: ErrorType, tokens: &mut T, errors: &mut Vec<ErrorType>) {
    tokens.front_push(ERROR_TOKEN);
    errors.push(error);
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

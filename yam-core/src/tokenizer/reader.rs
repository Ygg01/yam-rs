#![allow(clippy::match_like_matches_macro)]
#![allow(clippy::wrong_self_convention)]

use alloc::collections::VecDeque;
use alloc::vec::Vec;

use super::lexer::{prepend_error, LiteralStringState, NodeSpans, QuoteState};
use super::LexerToken::*;
use super::{
    lexer::{push_error, LexerState},
    ErrorType,
};
use crate::tokenizer::lexer::MapState::*;
use crate::tokenizer::lexer::{find_matching_state, DirectiveState, SeparationSpaceInfo};

use crate::tokenizer::lexer::LexerState::*;
use crate::tokenizer::ErrorType::*;

use memchr::{memchr, memchr2};


#[doc(hidden)]
pub struct LexMutState<'a> {
    pub(crate) curr_state: LexerState,
    pub(crate) last_block_indent: &'a Option<u32>,
    pub(crate) stack: &'a Vec<LexerState>,
    pub(crate) has_tab: &'a mut bool,
    pub(crate) errors: &'a mut Vec<ErrorType>,
    pub(crate) tokens: &'a mut VecDeque<usize>,
    pub(crate) space_indent: &'a mut Option<u32>,
}

#[doc(hidden)]
#[derive(PartialEq, Clone, Copy)]
pub enum ChompIndicator {
    /// `-` final line break and any trailing empty lines are excluded from the scalar’s content
    Strip,
    ///  `` final line break character is preserved in the scalar’s content
    Clip,
    /// `+` final line break and any trailing empty lines are considered to be part of the scalar’s content
    Keep,
}

pub trait QuoteType {
    fn get_token(&self) -> usize;
    fn get_literal(&self) -> u8;
    fn match_fn<R: Reader + ?Sized>(
        &self,
        reader: &mut R,
        match_pos: usize,
        start_str: &mut usize,
        new_lines: &mut Option<usize>,
        lexer_state: &mut LexMutState,
        tokens: &mut Vec<usize>,
    ) -> QuoteState;
    fn get_quote(&self, input: &[u8]) -> Option<usize>;
    fn get_quote_trim(&self, input: &[u8], start_str: usize) -> Option<(usize, usize)> {
        input
            .iter()
            .rposition(|chr| *chr != b' ' && *chr != b'\t')
            .map(|find| (start_str + find + 1, find + 1))
    }
}

#[derive(Clone, Copy)]
pub struct SingleQuote {}

impl QuoteType for SingleQuote {
    fn get_token(&self) -> usize {
        ScalarSingleQuote as usize
    }
    fn get_literal(&self) -> u8 {
        b'\''
    }

    fn match_fn<R: Reader + ?Sized>(
        &self,
        reader: &mut R,
        match_pos: usize,
        start_str: &mut usize,
        new_lines: &mut Option<usize>,
        _lexer_state: &mut LexMutState,
        tokens: &mut Vec<usize>,
    ) -> QuoteState {
        match reader.peek_chars() {
            [b'\'', b'\'', ..] => {
                emit_token_mut(start_str, match_pos + 1, new_lines, tokens);
                reader.skip_bytes(2);
                *start_str = reader.offset();
            }
            [b'\'', ..] => {
                emit_token_mut(start_str, match_pos, new_lines, tokens);
                reader.skip_bytes(1);
                return QuoteState::End;
            }
            _ => {}
        }
        QuoteState::Start
    }

    fn get_quote(&self, input: &[u8]) -> Option<usize> {
        memchr(b'\'', input)
    }
}

#[derive(Clone, Copy)]
pub struct DoubleQuote {}

impl QuoteType for DoubleQuote {
    fn get_token(&self) -> usize {
        ScalarDoubleQuote as usize
    }

    fn get_literal(&self) -> u8 {
        b'"'
    }

    #[allow(unused_must_use)]
    fn match_fn<R: Reader + ?Sized>(
        &self,
        reader: &mut R,
        match_pos: usize,
        start_str: &mut usize,
        new_lines: &mut Option<usize>,
        lexer_state: &mut LexMutState,
        tokens: &mut Vec<usize>,
    ) -> QuoteState {
        match reader.peek_chars() {
            [b'\\', b' ', ..] => {
                *start_str = reader.skip_bytes(1);
            }
            [b'\\', b'\t', ..] => {
                emit_token_mut(start_str, match_pos, new_lines, tokens);
                emit_token_mut(&mut (match_pos + 1), match_pos + 2, new_lines, tokens);
                reader.skip_bytes(2);
                *start_str = reader.offset();
            }
            [b'\\', b't', ..] => {
                emit_token_mut(start_str, match_pos + 2, new_lines, tokens);
                reader.skip_bytes(2);
            }
            [b'\\', b'\r' | b'\n', ..] => {
                emit_token_mut(start_str, match_pos, new_lines, tokens);
                reader.skip_bytes(1);
                reader.update_newlines(&mut None, start_str, lexer_state);
            }
            [b'\\', b'"', ..] => {
                emit_token_mut(start_str, match_pos, new_lines, tokens);
                *start_str = reader.offset() + 1;
                reader.skip_bytes(2);
            }
            [b'\\', b'/', ..] => {
                emit_token_mut(start_str, match_pos, new_lines, tokens);
                *start_str = reader.skip_bytes(1);
            }
            [b'\\', b'u' | b'U' | b'x', ..] => {
                reader.skip_bytes(2);
            }
            [b'\\', x, ..] => {
                if is_valid_escape(*x) {
                    emit_token_mut(start_str, match_pos, new_lines, tokens);
                    reader.skip_bytes(2);
                } else {
                    prepend_error(InvalidEscapeCharacter, tokens, lexer_state.errors);
                    reader.skip_bytes(2);
                }
            }
            [b'"', ..] => {
                reader.emit_new_space(tokens, new_lines);
                emit_token_mut(start_str, match_pos, new_lines, tokens);
                reader.skip_bytes(1);
                return QuoteState::End;
            }
            [b'\\'] => {
                reader.skip_bytes(1);
            }
            _ => {}
        }
        QuoteState::Start
    }

    fn get_quote(&self, input: &[u8]) -> Option<usize> {
        memchr2(b'\\', b'"', input)
    }
}

pub trait Reader {
    fn eof(&mut self) -> bool;
    fn col(&self) -> u32;
    fn line(&self) -> u32;
    fn offset(&self) -> usize;
    fn peek_chars(&mut self) -> &[u8];
    fn peek_byte_at(&mut self, offset: usize) -> Option<u8>;
    #[inline]
    fn peek_byte(&mut self) -> Option<u8> {
        self.peek_byte_at(0)
    }
    #[inline]
    fn peek_byte_is(&mut self, needle: u8) -> bool {
        match self.peek_byte_at(0) {
            Some(x) if x == needle => true,
            _ => false,
        }
    }
    #[inline]
    fn peek_byte_is_off(&mut self, needle: u8, offset: usize) -> bool {
        match self.peek_byte_at(offset) {
            Some(x) if x == needle => true,
            _ => false,
        }
    }
    fn peek_stream_ending(&mut self) -> bool;
    fn skip_space_tab(&mut self) -> usize;
    fn skip_space_and_tab_detect(&mut self, has_tab: &mut bool) -> usize {
        let (indent, amount) = self.count_space_then_tab();
        *has_tab = indent != amount;
        let amount = amount.try_into().unwrap();
        self.skip_bytes(amount);
        amount
    }
    fn skip_bytes(&mut self, amount: usize) -> usize;
    fn save_bytes(
        &mut self,
        tokens: &mut Vec<usize>,
        start: usize,
        end: usize,
        new_lines: Option<u32>,
    );
    fn save_to_buf(&mut self, start: usize, input: &[u8]) -> (usize, usize);
    fn emit_tokens(&mut self, tokens: &mut Vec<usize>, start: usize, end: usize, new_lines: u32);

    fn try_read_slice_exact(&mut self, needle: &str) -> bool;
    fn get_read_line(&mut self) -> (usize, usize, usize);
    fn get_zero_slice(&mut self) -> Vec<u8>;

    fn read_line(&mut self, space_indent: &mut Option<u32>) -> (usize, usize);
    fn count_spaces(&mut self) -> u32;
    fn count_whitespace(&mut self) -> usize {
        self.count_whitespace_from(0)
    }
    fn count_whitespace_from(&mut self, offset: usize) -> usize;
    fn count_spaces_till(&mut self, indent: u32) -> usize;
    fn is_empty_newline(&mut self) -> bool;
    fn count_space_then_tab(&mut self) -> (u32, u32);
    fn consume_anchor_alias(&mut self) -> (usize, usize);
    fn read_tag(&mut self, lexer_state: &mut LexMutState) -> (usize, usize, usize);
    fn read_tag_handle(&mut self, space_indent: &mut Option<u32>) -> Result<Vec<u8>, ErrorType>;
    fn read_tag_uri(&mut self) -> Option<(usize, usize)>;
    fn read_directive(
        &mut self,
        directive_state: &mut DirectiveState,
        lexer_state: &mut LexMutState,
    ) -> bool;
    fn read_break(&mut self) -> Option<(usize, usize)>;
    fn emit_new_space(&mut self, tokens: &mut Vec<usize>, new_lines: &mut Option<usize>);

    fn read_plain(&mut self, lex_state: &mut LexMutState) -> NodeSpans {
        let mut spans = self.get_curr_node();
        let mut had_comment = false;
        let mut end_line = self.line();
        let mut curr_indent = self.col();
        let mut num_newlines = 0;
        let block_indent = indent(lex_state);

        let in_flow_collection = lex_state.curr_state.in_flow_collection();
        spans.push(ScalarPlain as usize);
        loop {
            if had_comment {
                if lex_state.curr_state != DocBlock {
                    push_error(InvalidCommentInScalar, &mut spans.spans, lex_state.errors);
                }
                break;
            }

            let (start, end) =
                self.read_plain_one_line(in_flow_collection, &mut had_comment, lex_state);
            let new_lines = match num_newlines {
                x if x >= 1 => Some(x - 1),
                _ => None,
            };
            self.save_bytes(&mut spans.spans, start, end, new_lines);

            end_line = self.line();

            if self.peek_byte().map_or(false, is_white_tab_or_break) {
                if let Some(folded_newline) = self.skip_separation_spaces(lex_state) {
                    if self.col() >= block_indent {
                        num_newlines = folded_newline.num_breaks;
                    }
                    self.skip_space_tab();
                    if folded_newline.has_comment {
                        had_comment = true;
                    }
                    curr_indent = folded_newline.space_indent;
                }
            }

            let chr = self.peek_byte_at(0).unwrap_or(b'\0');
            let end_of_stream = self.eof() || self.peek_stream_ending();

            if chr == b'-' && matches!(lex_state.curr_state, BlockSeq(indent, _) if curr_indent > indent)
                || chr == b'?' && matches!(lex_state.curr_state, BlockMap(indent, ExpectComplexKey) if curr_indent > indent ) {
                continue;
            } else if end_of_stream || chr == b'?' || chr == b':' || chr == b'-'
                || (in_flow_collection && is_flow_indicator(chr))
                || find_matching_state(lex_state.stack, |state| matches!(state, BlockMap(ind_col, _)| BlockSeq(ind_col, _) if ind_col >= curr_indent)
                ).is_some()
            {
                break;
            }
        }
        spans.push(ScalarEnd as usize);
        spans.is_multiline = spans.line_start != end_line;
        spans
    }

    #[inline]
    fn read_plain_one_line(
        &mut self,
        in_flow_collection: bool,
        had_comment: &mut bool,
        _lexer_state: &mut LexMutState,
    ) -> (usize, usize) {
        let start = self.offset();
        let slice = self.get_zero_slice();
        let mut end_of_str = 0;

        for (pos,win) in slice.windows(2).enumerate() {
            match win {
                // ns-plain-char  prevent ` #`
                [peek, b'#', ..] if is_white_tab_or_break(*peek) => {
                    if !*had_comment {
                        *had_comment = true;
                    }
                    break;
                }
                [b':', peek, ..] if is_plain_unsafe(*peek, in_flow_collection) => {
                    break;
                }
                [b',' | b'[' | b']' | b'{' | b'}', ..] if in_flow_collection => break,
                [curr, ..] if is_newline(*curr) => break,
                [curr, ..] if is_newline(*curr) => break,
                [curr, ..] if is_white_tab(*curr) => {},
                [_, ..] => end_of_str = pos + 1,
                [] => break,
            };
        }
        self.save_to_buf(start, &slice[0..end_of_str])

    }

    fn skip_separation_spaces(
        &mut self,
        lex_state: &mut LexMutState,
    ) -> Option<SeparationSpaceInfo> {
        if !self.peek_byte().map_or(true, is_white_tab_or_break) {
            return None;
        }

        let mut num_breaks = 0u32;
        let mut space_indent = 0u32;
        let mut found_eol = true;
        let mut has_tab = false;
        let mut has_comment = false;

        loop {
            if !self.peek_byte().map_or(false, is_valid_skip_char) || self.eof() {
                break;
            }
            let sep = self.count_space_then_tab();
            space_indent = sep.0;
            let amount = sep.1;
            has_tab = space_indent != amount;
            let is_comment = self
                .peek_byte_at(amount as usize)
                .map_or(false, |c| c == b'#');

            if has_comment && !is_comment {
                break;
            }
            if is_comment {
                has_comment = true;
                if amount > 0
                    && !self
                        .peek_byte_at(amount.saturating_sub(1) as usize)
                        .map_or(false, |c| c == b' ' || c == b'\t' || c == b'\n')
                {
                    push_error(
                        MissingWhitespaceBeforeComment,
                        lex_state.tokens,
                        lex_state.errors,
                    );
                }
                self.read_line(lex_state.space_indent);
                found_eol = true;
                num_breaks += 1;
                space_indent = 0;
                continue;
            }

            if self.read_break().is_some() {
                num_breaks += 1;
                space_indent = 0;
                has_tab = false;
                found_eol = true;
            }

            if found_eol {
                let (indent, amount) = self.count_space_then_tab();
                space_indent = indent;
                has_tab = indent != amount;
                self.skip_bytes(amount as usize);
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

    fn read_comment(&mut self, lexer_state: &mut LexMutState) {
        self.read_line(lexer_state.space_indent);
    }


    #[doc(hidden)]
    fn read_quote<T: QuoteType + Copy>(
        &mut self,
        quote: T,
        lexer_state: &mut LexMutState,
    ) -> NodeSpans {
        let mut node = self.get_curr_node();
        node.push(quote.get_token());
        let mut start_str = self.skip_bytes(1);
        let mut new_lines = None;
        let mut state = QuoteState::Start;

        loop {
            state = match state {
                QuoteState::Start => self.start_fn(
                    quote,
                    &mut start_str,
                    &mut new_lines,
                    lexer_state,
                    &mut node.spans,
                ),
                QuoteState::Trim => self.trim_fn(
                    quote,
                    &mut start_str,
                    &mut new_lines,
                    lexer_state,
                    &mut node.spans,
                ),
                QuoteState::End | QuoteState::Error => break,
            };
        }
        node.push(ScalarEnd as usize);

        node.is_multiline = node.line_start != self.line();
        node
    }

    fn start_fn<T: QuoteType>(
        &mut self,
        quote: T,
        start_str: &mut usize,
        new_lines: &mut Option<usize>,
        lexer_state: &mut LexMutState,
        tokens: &mut Vec<usize>,
    ) -> QuoteState {
        let input = self.get_quote_line_offset(quote.get_literal());
        if let Some(pos) = quote.get_quote(input) {
            let match_pos = self.skip_bytes(pos);
            quote.match_fn(self, match_pos, start_str, new_lines, lexer_state, tokens)
        } else if self.eof() {
            prepend_error(UnexpectedEndOfFile, tokens, lexer_state.errors);
            QuoteState::Error
        } else {
            QuoteState::Trim
        }
    }

    #[allow(unused_must_use)]
    fn trim_fn<Q: QuoteType>(
        &mut self,
        quote_type: Q,
        start_str: &mut usize,
        new_lines: &mut Option<usize>,
        lexer_state: &mut LexMutState,
        tokens: &mut Vec<usize>,
    ) -> QuoteState {
        if self.peek_stream_ending() {
            prepend_error(UnexpectedEndOfStream, tokens, lexer_state.errors);
        };
        let indent = indent(lexer_state);
        if !matches!(lexer_state.curr_state, DocBlock) && self.col() <= indent {
            prepend_error(
                InvalidQuoteIndent {
                    actual: self.col(),
                    expected: indent,
                },
                tokens,
                lexer_state.errors,
            );
        }
        let input = self.get_quote_line_offset(b'\'');
        if let Some((match_pos, len)) = quote_type.get_quote_trim(input, *start_str) {
            emit_token_mut(start_str, match_pos, new_lines, tokens);
            self.skip_bytes(len);
        } else {
            self.update_newlines(new_lines, start_str, lexer_state);
        }

        match self.peek_byte() {
            Some(b'\n' | b'\r') => {
                if let Err(err) = self.update_newlines(new_lines, start_str, lexer_state) {
                    prepend_error(err, tokens, lexer_state.errors);
                }
                QuoteState::Start
            }
            Some(x) if x == quote_type.get_literal() => {
                if let Some(x) = new_lines {
                    tokens.push(NewLine as usize);
                    tokens.push(*x);
                }
                self.skip_bytes(1);
                QuoteState::End
            }
            Some(_) => QuoteState::Start,
            None => {
                prepend_error(UnexpectedEndOfFile, tokens, lexer_state.errors);
                QuoteState::Error
            }
        }
    }

    fn get_quote_line_offset(&mut self, quote: u8) -> &[u8];

    #[allow(unused_must_use)]
    fn update_newlines(
        &mut self,
        new_lines: &mut Option<usize>,
        start_str: &mut usize,
        lexer_state: &mut LexMutState,
    ) -> Result<(), ErrorType> {
        if let Some(x) = self.skip_separation_spaces(lexer_state) {
            *new_lines = Some(x.num_breaks.saturating_sub(1) as usize);
            *start_str = self.offset();
            if lexer_state
                .last_block_indent
                .map_or(false, |indent| indent >= x.space_indent)
            {
                return Err(TabsNotAllowedAsIndentation);
            }
        }
        Ok(())
    }

    fn get_curr_node(&self) -> NodeSpans {
        NodeSpans {
            col_start: self.col(),
            line_start: self.line(),
            ..Default::default()
        }
    }

    fn read_block_scalar(&mut self, literal: bool, lexer_state: &mut LexMutState) -> NodeSpans {
        let mut chomp = ChompIndicator::Clip;
        let mut node = self.get_curr_node();
        self.skip_bytes(1);

        let token = if literal {
            ScalarLit as usize
        } else {
            ScalarFold as usize
        };

        node.push(token);

        let mut new_lines = 0;
        let mut prev_indent = 0;

        let mut state = self.get_initial_indent(lexer_state, &mut prev_indent, &mut chomp);
        if self.eof() {
            node.push(ScalarEnd as usize);
            return node;
        }
        loop {
            if self.eof() || self.peek_stream_ending() {
                break;
            }

            state = match state {
                LiteralStringState::AutoIndentation => self.process_auto_indentation(
                    &mut prev_indent,
                    &mut new_lines,
                    &mut node.spans,
                    lexer_state,
                ),
                LiteralStringState::Indentation(indent) => {
                    if self.is_empty_newline() {
                        self.process_trim(indent, &mut new_lines, &mut node.spans, lexer_state)
                    } else {
                        self.process_indentation(
                            indent,
                            (literal, chomp),
                            &mut prev_indent,
                            &mut new_lines,
                            &mut node.spans,
                            lexer_state,
                        )
                    }
                }

                LiteralStringState::Comment => self.process_comment(lexer_state),
                LiteralStringState::TabError => {
                    self.skip_separation_spaces(lexer_state);
                    if !(self.eof() || self.peek_stream_ending()) {
                        prepend_error(InvalidScalarIndent, &mut node.spans, lexer_state.errors);
                    }

                    break;
                }
                LiteralStringState::End => break,
            };
        }

        match chomp {
            ChompIndicator::Keep => {
                self.emit_new_space(&mut node.spans, &mut Some(new_lines as usize));
            }
            ChompIndicator::Clip if new_lines > 0 => {
                self.emit_new_space(&mut node.spans, &mut Some(1));
            }
            _ => {}
        }
        node.push(ScalarEnd as usize);
        node.is_multiline = true;

        node
    }

    fn process_auto_indentation(
        &mut self,
        prev_indent: &mut u32,
        new_lines: &mut u32,
        tokens: &mut Vec<usize>,
        lexer_state: &mut LexMutState,
    ) -> LiteralStringState {
        let mut max_prev_indent = 0;
        loop {
            if self.eof() {
                return LiteralStringState::End;
            }

            let newline_indent = self.count_spaces();
            *lexer_state.has_tab = matches!(
                self.peek_byte_at(newline_indent.saturating_sub(1) as usize),
                Some(b'\t')
            );

            let newline_is_empty = self.is_empty_newline();
            if newline_is_empty && max_prev_indent < newline_indent {
                max_prev_indent = newline_indent;
            }
            if max_prev_indent > newline_indent {
                prepend_error(SpacesFoundAfterIndent, tokens, lexer_state.errors);
            }
            if !newline_is_empty {
                *prev_indent = newline_indent;
                return LiteralStringState::Indentation(newline_indent);
            }
            *new_lines += 1;
            self.read_line(lexer_state.space_indent);
        }
    }

    fn process_comment(&mut self, lexer_state: &mut LexMutState) -> LiteralStringState {
        loop {
            if self.eof() {
                return LiteralStringState::End;
            }
            let space_offset = self.count_spaces() as usize;
            if self.peek_byte_at(space_offset) != Some(b'#') {
                return LiteralStringState::End;
            }
            self.read_line(lexer_state.space_indent);
        }
    }

    fn process_trim(
        &mut self,
        indent: u32,
        new_lines: &mut u32,
        tokens: &mut Vec<usize>,
        lexer_state: &mut LexMutState,
    ) -> LiteralStringState {
        loop {
            if self.eof() {
                return LiteralStringState::End;
            }
            let newline_indent: u32 = self.count_spaces();
            let newline_is_empty = self.is_empty_newline();
            if !newline_is_empty {
                return LiteralStringState::Indentation(indent);
            }
            if newline_indent > indent {
                self.skip_bytes(indent as usize);
                if self.peek_byte_is(b'#') {
                    return LiteralStringState::Comment;
                }
                let (start, end) = self.read_line(lexer_state.space_indent);
                if start != end {
                    self.emit_tokens(tokens, start, end, *new_lines);
                    *new_lines = 1;
                }
            } else {
                *new_lines += 1;
                self.read_line(lexer_state.space_indent);
            }
        }
    }

    fn process_indentation(
        &mut self,
        indent: u32,
        lit_chomp: (bool, ChompIndicator),
        prev_indent: &mut u32,
        new_lines: &mut u32,
        tokens: &mut Vec<usize>,
        lexer_state: &mut LexMutState,
    ) -> LiteralStringState {
        let curr_indent = self.count_spaces();
        let mut next_state =
            self.next_process_indentation(curr_indent, indent, lit_chomp, new_lines, prev_indent);
        match next_state {
            v @ (LiteralStringState::Comment | LiteralStringState::End) => return v,
            x => x,
        };

        self.consume_spaces(indent, lexer_state);
        let (start, end, _) = self.get_read_line();
        if start == end {
            *new_lines += 1;
        } else {
            match lexer_state.last_block_indent {
                Some(i) if *i >= curr_indent => {
                    *new_lines = 0;
                    if self.peek_byte_is(b'\t') {
                        *lexer_state.has_tab = true;
                        next_state = LiteralStringState::TabError;
                    } else {
                        next_state = LiteralStringState::End;
                    }
                }
                _ => {
                    let count_tab = self.count_space_then_tab().1;
                    if *new_lines > 0 {
                        // First empty line after block literal is treated in a special way
                        let is_first_non_empty_line = tokens.len() > 1;

                        // That's on the same indentation level as previously detected indentation
                        if is_first_non_empty_line
                            && !lit_chomp.0
                            && *prev_indent == curr_indent + count_tab
                            && curr_indent == indent
                        {
                            tokens.push(NewLine as usize);
                            tokens.push(new_lines.saturating_sub(1) as usize);
                        } else {
                            tokens.push(NewLine as usize);
                            tokens.push(*new_lines as usize);
                        }
                    }
                    *prev_indent = curr_indent + count_tab;
                    tokens.push(start);
                    tokens.push(end);
                    self.read_line(lexer_state.space_indent);
                    *new_lines = 1;
                }
            };
        }

        next_state
    }

    fn consume_spaces(&mut self, indent: u32, lex_state: &mut LexMutState) -> bool {
        let x = self.count_spaces_till(indent);
        if lex_state.space_indent.is_none() {
            *lex_state.space_indent = Some(x as u32);
        }
        self.skip_bytes(x);
        x == indent as usize
    }

    fn next_process_indentation(
        &mut self,
        curr_indent: u32,
        indent: u32,
        lit_chomp: (bool, ChompIndicator),
        new_lines: &mut u32,
        prev_indent: &mut u32,
    ) -> LiteralStringState {
        if curr_indent < indent {
            if self.peek_byte_at(curr_indent as usize) == Some(b'#') {
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

    fn get_initial_indent(
        &mut self,
        lexer_state: &mut LexMutState,
        prev_indent: &mut u32,
        chomp: &mut ChompIndicator,
    ) -> LiteralStringState {
        let block_indent = indent(lexer_state);

        let (amount, state) = match self.peek_chars() {
            [_, b'0', ..] | [b'0', _, ..] => {
                push_error(
                    ExpectedChompBetween1and9,
                    lexer_state.tokens,
                    lexer_state.errors,
                );
                self.skip_bytes(2);
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
            [b'#', ..] => {
                push_error(UnexpectedComment, lexer_state.tokens, lexer_state.errors);
                self.skip_bytes(1);
                return LiteralStringState::End;
            }
            _ => (0, LiteralStringState::AutoIndentation),
        };
        self.skip_bytes(amount);
        if let LiteralStringState::Indentation(x) = state {
            *prev_indent = x;
        }

        // allow comment in first line of block scalar
        self.skip_space_tab();
        match self.peek_byte() {
            Some(b'#' | b'\r' | b'\n') => {
                self.read_line(lexer_state.space_indent);
            }
            Some(chr) => {
                self.read_line(lexer_state.space_indent);
                push_error(
                    UnexpectedSymbol(chr as char),
                    lexer_state.tokens,
                    lexer_state.errors,
                );
                return LiteralStringState::End;
            }
            _ => {}
        }

        state
    }
}

fn indent(lexer_state: &mut LexMutState) -> u32 {
    match lexer_state.last_block_indent {
        None => 0,
        Some(x) if lexer_state.curr_state.in_flow_collection() => x + 1,
        Some(x) => *x,
    }
}

fn emit_token_mut(
    start: &mut usize,
    end: usize,
    new_lines: &mut Option<usize>,
    tokens: &mut Vec<usize>,
) {
    if end > *start {
        if let Some(new_line) = new_lines.take() {
            tokens.push(NewLine as usize);
            tokens.push(new_line);
        }
        tokens.push(*start);
        tokens.push(end);
        *start = end;
    }
}

#[inline]
pub(crate) const fn is_white_tab_or_break(chr: u8) -> bool {
    match chr {
        b' ' | b'\t' | b'\r' | b'\n' => true,
        _ => false,
    }
}

#[inline]
pub(crate) const fn is_valid_skip_char(chr: u8) -> bool {
    match chr {
        b' ' | b'\t' | b'\r' | b'\n' | b'#' => true,
        _ => false,
    }
}

#[inline]
pub(crate) const fn is_white_tab(chr: u8) -> bool {
    match chr {
        b' ' | b'\t' => true,
        _ => false,
    }
}

#[inline]
pub(crate) const fn is_plain_unsafe(chr: u8, in_flow: bool) -> bool {
    match chr {
        b',' | b'[' | b']' | b'{' | b'}' => in_flow,
        b'\0' | b' ' | b'\t' | b'\r' | b'\n' => true,
        _ => false,
    }
}

#[inline]
pub(crate) const fn is_newline(chr: u8) -> bool {
    match chr {
        b'\r' | b'\n' => true,
        _ => false,
    }
}

#[inline]
pub(crate) const fn is_flow_indicator(chr: u8) -> bool {
    match chr {
        b',' | b'[' | b']' | b'{' | b'}' => true,
        _ => false,
    }
}

#[inline]
pub(crate) fn is_uri_char(chr: u8) -> bool {
    chr == b'!'
        || (b'#'..=b',').contains(&chr)
        || (b'-'..=b';').contains(&chr)
        || (b'?'..=b'[').contains(&chr)
        || chr == b'_'
        || chr == b']'
        || chr.is_ascii_lowercase()
}

#[inline]
pub(crate) fn is_tag_char_short(chr: u8) -> bool {
    // can't contain `!`, `,` `[`, `]` , `{` , `}`
    (b'#'..=b'+').contains(&chr)
        || (b'-'..=b';').contains(&chr)
        || (b'?'..=b'Z').contains(&chr)
        || chr == b'_'
        || chr.is_ascii_lowercase()
}

#[inline]
pub(crate) fn is_tag_char(chr: u8) -> bool {
    matches!(chr, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9')
}

#[inline]
pub(crate) fn is_valid_escape(x: u8) -> bool {
    x == b'0'
        || x == b'r'
        || x == b'n'
        || x == b'b'
        || x == b'\\'
        || x == b'/'
        || x == b'"'
        || x == b'N'
        || x == b'_'
        || x == b'L'
        || x == b'P'
}

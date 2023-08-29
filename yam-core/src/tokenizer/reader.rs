#![allow(clippy::match_like_matches_macro)]

use alloc::collections::VecDeque;
use alloc::vec::Vec;


use core::ops::Range;

use super::lexer::{prepend_error, NodeSpans, QuoteState};
use super::LexerToken::*;
use super::{
    lexer::{push_error, LexerState},
    ErrorType,
};
use crate::tokenizer::lexer::MapState::*;
use crate::tokenizer::lexer::{find_matching_state, SeparationSpaceInfo};

use crate::tokenizer::lexer::LexerState::*;
use crate::tokenizer::ErrorType::*;

use memchr::{memchr, memchr2};

pub struct LookAroundBytes<'a> {
    iter: &'a [u8],
    pos: usize,
    end: usize,
}

impl<'a> LookAroundBytes<'a> {
    pub(crate) fn new(iter: &'a [u8], range: Range<usize>) -> LookAroundBytes<'a> {
        let (pos, end) = (range.start, range.end);

        LookAroundBytes { iter, pos, end }
    }
}

impl<'a> Iterator for LookAroundBytes<'a> {
    type Item = (u8, u8, u8, usize);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos <= self.end {
            let prev = if self.pos < 1 {
                b'\0'
            } else {
                unsafe { *self.iter.get_unchecked(self.pos - 1) }
            };
            let curr = unsafe { *self.iter.get_unchecked(self.pos) };
            let next = unsafe {
                if self.pos + 1 < self.iter.len() {
                    *self.iter.get_unchecked(self.pos + 1)
                } else {
                    b'\0'
                }
            };
            let x = Some((prev, curr, next, self.pos));
            self.pos += 1;
            x
        } else {
            None
        }
    }
}

pub struct LexMutState<'a> {
    pub(crate) curr_state: LexerState,
    pub(crate) last_block_indent: &'a Option<u32>,
    pub(crate) tokens: &'a mut VecDeque<usize>,
    pub(crate) errors: &'a mut Vec<ErrorType>,
    pub(crate) stack: &'a Vec<LexerState>,
    pub(crate) space_indent: &'a mut Option<u32>,
    pub(crate) has_tab: &'a mut bool,
}

pub trait QuoteType {
    fn get_token(&self) -> usize;
    fn get_liteal(&self) -> u8;
    fn match_fn<B, R: Reader<B> + ?Sized>(
        &self,
        reader: &mut R,
        match_pos: usize,
        start_str: &mut usize,
        newspaces: &mut Option<usize>,
        lexer_state: &mut LexMutState,
        tokens: &mut Vec<usize>,
    ) -> QuoteState;
    fn get_quote(&self, input: &[u8]) -> Option<usize>;
    fn get_quote_trim(&self, input: &[u8], start_str: usize) -> Option<(usize, usize)>;
}

#[derive(Clone, Copy)]
pub struct SingleQuote {}

impl QuoteType for SingleQuote {
    fn get_token(&self) -> usize {
        ScalarSingleQuote as usize
    }
    fn match_fn<B, R: Reader<B> + ?Sized>(
        &self,
        reader: &mut R,
        match_pos: usize,
        start_str: &mut usize,
        newspaces: &mut Option<usize>,
        _lexer_state: &mut LexMutState,
        tokens: &mut Vec<usize>,
    ) -> QuoteState {
        match reader.peek_chars() {
            [b'\'', b'\'', ..] => {
                emit_token_mut(start_str, match_pos + 1, newspaces, tokens);
                reader.skip_bytes(2);
                *start_str = reader.offset();
            }
            [b'\'', ..] => {
                emit_token_mut(start_str, match_pos, newspaces, tokens);
                reader.skip_bytes(1);
                return QuoteState::End;
            }
            _ => {}
        }
        QuoteState::Start
    }

    fn get_liteal(&self) -> u8 {
        b'\''
    }

    fn get_quote(&self, input: &[u8]) -> Option<usize> {
        memchr(b'\'', input)
    }

    fn get_quote_trim(&self, input: &[u8], start_str: usize) -> Option<(usize, usize)> {
        input
            .iter()
            .rposition(|chr| *chr != b' ' && *chr != b'\t')
            .map(|find| (start_str + find + 1, find + 1))
    }
}

#[derive(Clone, Copy)]
pub struct DoubleQuote {}

impl QuoteType for DoubleQuote {
    fn get_token(&self) -> usize {
        ScalarDoubleQuote as usize
    }

    fn match_fn<B, R: Reader<B> + ?Sized>(
        &self,
        reader: &mut R,
        match_pos: usize,
        start_str: &mut usize,
        newspaces: &mut Option<usize>,
        lexer_state: &mut LexMutState,
        tokens: &mut Vec<usize>,
    ) -> QuoteState {
        match reader.peek_chars() {
            [b'\\', b' ', ..] => {
                *start_str = reader.skip_bytes(1);
            }
            [b'\\', b'\t', ..] => {
                emit_token_mut(start_str, match_pos, newspaces, tokens);
                emit_token_mut(&mut (match_pos + 1), match_pos + 2, newspaces, tokens);
                reader.skip_bytes(2);
                *start_str = reader.offset();
            }
            [b'\\', b't', ..] => {
                emit_token_mut(start_str, match_pos + 2, newspaces, tokens);
                reader.skip_bytes(2);
            }
            [b'\\', b'\r' | b'\n', ..] => {
                emit_token_mut(start_str, match_pos, newspaces, tokens);
                reader.skip_bytes(1);
                reader.update_newlines(&mut None, start_str, lexer_state);
            }
            [b'\\', b'"', ..] => {
                emit_token_mut(start_str, match_pos, newspaces, tokens);
                *start_str = reader.offset() + 1;
                reader.skip_bytes(2);
            }
            [b'\\', b'/', ..] => {
                emit_token_mut(start_str, match_pos, newspaces, tokens);
                *start_str = reader.skip_bytes(1);
            }
            [b'\\', b'u' | b'U' | b'x', ..] => {
                reader.skip_bytes(2);
            }
            [b'\\', x, ..] => {
                if is_valid_escape(*x) {
                    emit_token_mut(start_str, match_pos, newspaces, tokens);
                    reader.skip_bytes(2);
                } else {
                    prepend_error(InvalidEscapeCharacter, tokens, lexer_state.errors);
                    reader.skip_bytes(2);
                }
            }
            [b'"', ..] => {
                reader.emit_newspace(tokens, newspaces);
                emit_token_mut(start_str, match_pos, newspaces, tokens);
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

    fn get_liteal(&self) -> u8 {
        b'"'
    }

    fn get_quote(&self, input: &[u8]) -> Option<usize> {
        memchr2(b'\\', b'"', input)
    }

    fn get_quote_trim(&self, input: &[u8], start_str: usize) -> Option<(usize, usize)> {
        input
            .iter()
            .rposition(|chr| *chr != b' ' && *chr != b'\t')
            .map(|find| (start_str + find + 1, find + 1))
    }


}

pub trait Reader<B> {
    fn eof(&mut self) -> bool;
    fn col(&self) -> u32;
    fn line(&self) -> u32;
    fn offset(&self) -> usize;
    fn peek_chars(&mut self) -> &[u8];
    fn peek_two_chars(&mut self) -> &[u8];
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
    fn peek_stream_ending(&mut self) -> bool {
        let chars = self.peek_chars();
        (chars == b"..." || chars == b"---")
            && self.peek_byte_at(3).map_or(true, |c| {
                c == b'\t' || c == b' ' || c == b'\r' || c == b'\n' || c == b'[' || c == b'{'
            })
            && self.col() == 0
    }
    fn skip_space_tab(&mut self) -> usize;
    fn skip_space_and_tab_detect(&mut self, has_tab: &mut bool) -> usize;
    fn skip_bytes(&mut self, amount: usize) -> usize;
    fn save_bytes(&mut self, tokens: &mut Vec<usize>, start: usize, end: usize, newline: u32);
    fn try_read_slice_exact(&mut self, needle: &str) -> bool;
    fn get_read_line(&mut self) -> (usize, usize, usize);
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
    fn read_tag(&mut self) -> (Option<ErrorType>, usize, usize, usize);
    fn read_tag_handle(&mut self, space_indent: &mut Option<u32>) -> Result<Vec<u8>, ErrorType>;
    fn read_tag_uri(&mut self) -> Option<(usize, usize)>;
    fn read_break(&mut self) -> Option<(usize, usize)>;
    fn emit_newspace(&mut self, tokens: &mut Vec<usize>, newspaces: &mut Option<usize>);

    #[doc(hidden)]
    fn read_plain_one_line(
        &mut self,
        offset_start: Option<usize>,
        had_comment: &mut bool,
        in_flow_collection: bool,
    ) -> (usize, usize, usize);

    fn read_plain(
        &mut self,
        curr_state: LexerState,
        block_indent: u32,
        lex_state: &mut LexMutState,
    ) -> NodeSpans {
        let mut spans = NodeSpans {
            col_start: self.col(),
            line_start: self.line(),
            ..Default::default()
        };
        let mut had_comment = false;
        let mut offset_start: Option<usize> = None;
        let mut end_line = self.line();
        let mut curr_indent = self.col();
        let mut num_newlines = 0;

        let in_flow_collection = curr_state.in_flow_collection();
        spans.push(ScalarPlain as usize);
        loop {
            if had_comment {
                if curr_state != DocBlock {
                    push_error(InvalidCommentInScalar, &mut spans.spans, lex_state.errors);
                }
                break;
            }

            let (start, end, _consume) =
                self.read_plain_one_line(offset_start, &mut had_comment, in_flow_collection);

            self.save_bytes(&mut spans.spans, start, end, num_newlines);

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

            if chr == b'-' && matches!(curr_state, BlockSeq(indent, _) if curr_indent > indent)
                || chr == b'?' && matches!(curr_state, BlockMap(indent, ExpectComplexKey) if curr_indent > indent ) {
                offset_start = Some(self.offset());

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

    #[doc(hidden)]
    fn read_quote<T: QuoteType + Copy>(&mut self, quote: T, lexer_state: &mut LexMutState) -> NodeSpans {
        let mut node = NodeSpans {
            col_start: self.col(),
            line_start: self.line(),
            ..Default::default()
        };

        node.push(quote.get_token());
        let mut start_str = self.skip_bytes(1);
        let mut newspaces = None;
        let mut state = QuoteState::Start;

        loop {
            state = match state {
                QuoteState::Start => {
                    self.start_fn(quote, &mut start_str, &mut newspaces, lexer_state, &mut node.spans)
                }
                QuoteState::Trim => self.trim_fn(
                    quote,
                    &mut start_str,
                    &mut newspaces,
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
        newspaces: &mut Option<usize>,
        lexer_state: &mut LexMutState,
        tokens: &mut Vec<usize>,
    ) -> QuoteState {
        let input = self.get_quoteline_offset(quote.get_liteal());
        if let Some(pos) = quote.get_quote(input) {
            let match_pos = self.skip_bytes(pos);
            quote.match_fn(self, match_pos, start_str, newspaces, lexer_state, tokens)
        } else if self.eof() {
            prepend_error(ErrorType::UnexpectedEndOfFile, tokens, lexer_state.errors);
            QuoteState::Error
        } else {
            QuoteState::Trim
        }
    }

    fn trim_fn<Q: QuoteType>(
        &mut self,
        quote_type: Q,
        start_str: &mut usize,
        newspaces: &mut Option<usize>,
        lexer_state: &mut LexMutState,
        tokens: &mut Vec<usize>,
    ) -> QuoteState {
        if self.peek_stream_ending() {
            prepend_error(ErrorType::UnexpectedEndOfStream, tokens, lexer_state.errors);
        };
        let indent = indent(lexer_state);
        if !matches!(lexer_state.curr_state, DocBlock) && self.col() <= indent {
            prepend_error(
                ErrorType::InvalidQuoteIndent {
                    actual: self.col(),
                    expected: indent,
                },
                tokens,
                lexer_state.errors,
            );
        }
        let input = self.get_quoteline_offset(b'\'');
        if let Some((match_pos, len)) = quote_type.get_quote_trim(input, *start_str) {
            emit_token_mut(start_str, match_pos, newspaces, tokens);
            self.skip_bytes(len);
        } else {
            self.update_newlines(newspaces, start_str, lexer_state);
        }

        match self.peek_byte() {
            Some(b'\n' | b'\r') => {
                if let Err(err) = self.update_newlines(newspaces, start_str, lexer_state) {
                    prepend_error(err, tokens, lexer_state.errors);
                }
                QuoteState::Start
            }
            Some(x) if x == quote_type.get_liteal() => {
                if let Some(x) = newspaces {
                    tokens.push(NewLine as usize);
                    tokens.push(*x);
                }
                self.skip_bytes(1);
                QuoteState::End
            }
            Some(_) => QuoteState::Start,
            None => {
                prepend_error(ErrorType::UnexpectedEndOfFile, tokens, lexer_state.errors);
                QuoteState::Error
            }
        }
    }

    fn get_quoteline_offset(&mut self, quote: u8) -> &[u8];

    fn update_newlines(
        &mut self,
        newspaces: &mut Option<usize>,
        start_str: &mut usize,
        lexer_state: &mut LexMutState,
    ) -> Result<(), ErrorType> {
        if let Some(x) = self.skip_separation_spaces(lexer_state) {
            *newspaces = Some(x.num_breaks.saturating_sub(1) as usize);
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

    fn read_block_scalar(&mut self, _literal: bool, _block_indent: u32) -> NodeSpans {
        
        NodeSpans {
            col_start: self.col(),
            line_start: self.col(),
            ..Default::default()
        }
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
pub(crate) const fn is_plain_unsafe(chr: u8) -> bool {
    match chr {
        b'\0' | b' ' | b'\t' | b'\r' | b'\n' | b',' | b'[' | b']' | b'{' | b'}' => true,
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

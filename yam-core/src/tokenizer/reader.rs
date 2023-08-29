#![allow(clippy::match_like_matches_macro)]

use alloc::collections::VecDeque;
use alloc::vec::Vec;

use core::ops::Range;

use super::lexer::NodeSpans;
use super::LexerToken::*;
use super::{
    lexer::{push_error, LexerState},
    ErrorType,
};
use crate::tokenizer::lexer::MapState::*;
use crate::tokenizer::lexer::{find_matching_state, SeparationSpaceInfo};

use crate::tokenizer::lexer::LexerState::*;
use crate::tokenizer::ErrorType::*;

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
    pub(crate) tokens: &'a mut VecDeque<usize>,
    pub(crate) errors: &'a mut Vec<ErrorType>,
    pub(crate) stack: &'a Vec<LexerState>,
    pub(crate) space_indent: &'a mut Option<u32>,
    pub(crate) has_tab: &'a mut bool,
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
    fn get_double_quote(&mut self) -> Option<usize>;
    fn get_double_quote_trim(&mut self, start_str: usize) -> Option<(usize, usize)>;
    fn get_single_quote(&mut self) -> Option<usize>;
    fn get_single_quote_trim(&mut self, start_str: usize) -> Option<(usize, usize)>;
    fn count_space_then_tab(&mut self) -> (u32, u32);
    fn consume_anchor_alias(&mut self) -> (usize, usize);
    fn read_tag(&mut self) -> (Option<ErrorType>, usize, usize, usize);
    fn read_tag_handle(&mut self, space_indent: &mut Option<u32>) -> Result<Vec<u8>, ErrorType>;
    fn read_tag_uri(&mut self) -> Option<(usize, usize)>;
    fn read_break(&mut self) -> Option<(usize, usize)>;

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
    fn read_quote(&mut self) -> Vec<usize> {
        Vec::new()
    }
    fn read_dquote(&mut self) -> Vec<usize> {
        Vec::new()
    }
    fn read_block_scalar(&mut self, _literal: bool, _block_indent: u32) -> Vec<usize> {
        Vec::new()
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

#![allow(clippy::match_like_matches_macro)]

use std::collections::VecDeque;
use std::ops::ControlFlow;
use std::ops::ControlFlow::{Break, Continue};
use std::ops::RangeInclusive;

use super::spanner::ParserState;
use super::SpanToken;

pub struct LookAroundBytes<'a> {
    iter: &'a [u8],
    pos: usize,
    end: usize,
}

impl<'a> LookAroundBytes<'a> {
    pub(crate) fn new(iter: &'a [u8], range: RangeInclusive<usize>) -> LookAroundBytes<'a> {
        let (&pos, &end) = (range.start(), range.end());

        LookAroundBytes { iter, pos, end }
    }
}

pub(crate) enum ChompIndicator {
    /// `-` final line break and any trailing empty lines are excluded from the scalar’s content
    Strip,
    ///  `` final line break character is preserved in the scalar’s content
    Clip,
    /// `+` final line break and any trailing empty lines are considered to be part of the scalar’s content
    Keep,
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

pub trait Reader {
    fn eof(&self) -> bool;
    fn col(&self) -> usize;
    fn peek_byte_at(&self, offset: usize) -> Option<u8>;
    fn peek_byte(&self) -> Option<u8>;
    fn peek_byte_is(&self, needle: u8) -> bool {
        match self.peek_byte_at(0) {
            Some(x) if x == needle => true,
            _ => false,
        }
    }
    fn peek_byte_at_check(&self, offset: usize, check: fn(u8) -> bool) -> bool {
        match self.peek_byte_at(offset) {
            Some(x) if check(x) => true,
            _ => false,
        }
    }
    #[inline]
    fn skip_space_tab(&mut self, allow_tab: bool) -> usize {
        let x = self.count_space_tab(allow_tab);
        self.consume_bytes(x);
        x
    }
    fn count_space_tab(&self, allow_tab: bool) -> usize;
    fn consume_bytes(&mut self, amount: usize) -> usize;
    fn try_read_slice_exact(&mut self, needle: &str) -> bool;
    fn read_line(&mut self) -> (usize, usize);
    // Refactor
    fn read_block_seq(&mut self, indent: usize) -> Option<ParserState>;
    fn read_single_quote(&mut self, is_implicit: bool, tokens: &mut VecDeque<SpanToken>);
    fn read_plain_scalar(
        &mut self,
        start_indent: usize,
        curr_state: &ParserState,
        offset_indent: &mut Option<usize>,
    ) -> (Vec<SpanToken>, Option<ParserState>);
    fn skip_separation_spaces(&mut self, allow_comments: bool) -> usize;
    fn read_double_quote(&mut self, is_implicit: bool, tokens: &mut VecDeque<SpanToken>);
    fn read_block_scalar(
        &mut self,
        literal: bool,
        curr_state: &ParserState,
        tokens: &mut VecDeque<SpanToken>,
    );
    fn try_read_tag(&mut self, tokens: &mut VecDeque<SpanToken>);
}

#[inline]
pub fn is_tab_space(pos: usize, chr: u8, allow_tab: bool) -> ControlFlow<usize, usize> {
    if chr == b' ' || (allow_tab && chr == b'\t') {
        Continue(pos + 1)
    } else {
        Break(pos)
    }
}

#[inline]
pub(crate) fn is_white_tab_or_break(chr: u8) -> bool {
    match chr {
        b' ' | b'\t' | b'\r' | b'\n' => true,
        _ => false,
    }
}

#[inline]
pub(crate) fn ns_plain_safe(chr: u8, in_flow: bool) -> bool {
    match chr {
        b' ' | b'\t' | b'\r' | b'\n' => false,
        b',' | b'[' | b']' | b'{' | b'}' if in_flow => false,
        _ => true,
    }
}

#[inline]
pub(crate) fn is_white_tab(chr: u8) -> bool {
    match chr {
        b' ' | b'\t' => true,
        _ => false,
    }
}

#[inline]
pub(crate) fn is_newline(chr: u8) -> bool {
    match chr {
        b'\r' | b'\n' => true,
        _ => false,
    }
}

#[inline]
pub(crate) fn is_flow_indicator(chr: u8) -> bool {
    match chr {
        b',' | b'[' | b']' | b'{' | b'}' => true,
        _ => false,
    }
}

#[inline]
pub(crate) fn is_indicator(chr: u8) -> bool {
    match chr {
        b'-' | b'?' | b':' | b',' | b'[' | b']' | b'{' | b'}' | b'#' | b'&' | b'*' | b'!'
        | b'|' | b'>' | b'\'' | b'"' | b'%' | b'@' | b'`' => true,
        _ => false,
    }
}

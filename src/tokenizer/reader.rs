#![allow(clippy::match_like_matches_macro)]

use std::ops::Range;

use super::ErrorType;

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

pub trait Reader<B> {
    fn eof(&self) -> bool;
    fn col(&self) -> u32;
    fn line(&self) -> u32;
    fn pos(&self) -> usize;
    fn peek_chars(&self, buf: &mut B) -> &[u8];
    fn peek_byte(&self) -> Option<u8> {
        self.peek_byte_at(0)
    }
    fn peek_byte_at(&self, offset: usize) -> Option<u8>;
    #[inline]
    fn peek_byte_is(&self, needle: u8) -> bool {
        match self.peek_byte() {
            Some(x) if x == needle => true,
            _ => false,
        }
    }
    fn skip_space_tab(&mut self) -> usize;
    fn consume_bytes(&mut self, amount: usize) -> usize;
    fn try_read_slice_exact(&mut self, needle: &str) -> bool;
    fn read_line(&mut self) -> (usize, usize);
    fn count_spaces(&self) -> u32;
    fn count_spaces_till(&self, indent: u32) -> usize;
    fn is_empty_newline(&self) -> bool;
    // Refactor
    fn read_plain_one_line(
        &mut self,
        offset_start: Option<usize>,
        had_comment: &mut bool,
        in_flow_collection: bool,
    ) -> (usize, usize, Option<ErrorType>);
    fn read_double_quote(&mut self, errors: &mut Vec<ErrorType>) -> Vec<usize>;
    fn read_single_quote(&mut self, is_implicit: bool) -> Vec<usize>;
    fn skip_separation_spaces(&mut self, allow_comments: bool) -> (u32, bool);
    fn consume_anchor_alias(&mut self) -> (usize, usize);
    fn read_tag(&mut self) -> (Option<ErrorType>, usize, usize, usize);
    fn read_tag_handle(&mut self) -> Result<Vec<u8>, ErrorType>;
    fn read_tag_uri(&mut self) -> Option<(usize, usize)>;
    fn read_break(&mut self) -> Option<(usize, usize)>;
}

#[inline]
pub(crate) const fn is_white_tab_or_break(chr: u8) -> bool {
    match chr {
        b' ' | b'\t' | b'\r' | b'\n' => true,
        _ => false,
    }
}

#[inline]
pub(crate) const fn is_not_whitespace(chr: u8) -> bool {
    match chr {
        b' ' | b'\t' | b'\r' | b'\n' => false,
        _ => true,
    }
}

#[inline]
pub(crate) const fn ns_plain_safe(chr: u8) -> bool {
    match chr {
        b'\0' | b' ' | b'\t' | b'\r' | b'\n' | b',' | b'[' | b']' | b'{' | b'}' => false,
        _ => true,
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
pub(crate) fn is_tag_char(chr: u8) -> bool {
    matches!(chr, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9')
}

#[inline]
pub(crate) fn is_valid_escape(x: u8) -> bool {
    x == b'0'
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

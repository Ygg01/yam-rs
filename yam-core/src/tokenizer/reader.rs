#![allow(clippy::match_like_matches_macro)]

use alloc::vec::Vec;
use core::ops::Range;

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

impl Iterator for LookAroundBytes<'_> {
    type Item = (u8, u8, u8, usize);

    #[cfg_attr(not(feature = "no-inline"), inline)]
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
    fn offset(&self) -> usize;
    fn peek_chars(&self) -> &[u8];
    fn peek_two_chars(&self) -> &[u8];
    fn peek_byte_at(&self, offset: usize) -> Option<u8>;
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn peek_byte(&self) -> Option<u8> {
        self.peek_byte_at(0)
    }
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn peek_byte_is(&self, needle: u8) -> bool {
        match self.peek_byte_at(0) {
            Some(x) if x == needle => true,
            _ => false,
        }
    }
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn peek_byte_is_off(&self, needle: u8, offset: usize) -> bool {
        match self.peek_byte_at(offset) {
            Some(x) if x == needle => true,
            _ => false,
        }
    }
    fn peek_stream_ending(&self) -> bool {
        let chars = self.peek_chars();
        (chars == b"..." || chars == b"---")
            && self.peek_byte_at(3).map_or(true, |c| {
                c == b'\t' || c == b' ' || c == b'\r' || c == b'\n' || c == b'[' || c == b'{'
            })
            && self.col() == 0
    }
    fn skip_space_tab(&mut self) -> usize;
    fn skip_space_and_tab_detect(&mut self, has_tab: &mut bool) -> usize;
    fn consume_bytes(&mut self, amount: usize) -> usize;
    fn try_read_slice_exact(&mut self, needle: &str) -> bool;
    fn get_read_line(&self) -> (usize, usize, usize);
    fn read_line(&mut self) -> (usize, usize);
    fn count_spaces(&self) -> u32;
    fn count_whitespace(&self) -> usize {
        self.count_whitespace_from(0)
    }
    fn count_whitespace_from(&self, offset: usize) -> usize;
    fn count_spaces_till(&self, indent: u32) -> usize;
    fn is_empty_newline(&self) -> bool;
    fn get_double_quote(&self) -> Option<usize>;
    fn get_double_quote_trim(&self, start_str: usize) -> Option<(usize, usize)>;
    fn get_single_quote(&self) -> Option<usize>;
    fn get_single_quote_trim(&self, start_str: usize) -> Option<(usize, usize)>;
    fn count_space_then_tab(&mut self) -> (u32, u32);
    fn consume_anchor_alias(&mut self) -> (usize, usize);
    fn read_tag(&mut self) -> (Option<ErrorType>, usize, usize, usize);
    fn read_tag_handle(&mut self) -> Result<Vec<u8>, ErrorType>;
    fn read_tag_uri(&mut self) -> Option<(usize, usize)>;
    fn read_break(&mut self) -> Option<(usize, usize)>;
    fn read_plain_one_line(
        &mut self,
        offset_start: Option<usize>,
        had_comment: &mut bool,
        in_flow_collection: bool,
    ) -> (usize, usize, usize);
}

#[cfg_attr(not(feature = "no-inline"), inline)]
pub(crate) const fn is_white_tab_or_break(chr: u8) -> bool {
    match chr {
        b' ' | b'\t' | b'\r' | b'\n' => true,
        _ => false,
    }
}

#[cfg_attr(not(feature = "no-inline"), inline)]
pub(crate) const fn is_valid_skip_char(chr: u8) -> bool {
    match chr {
        b' ' | b'\t' | b'\r' | b'\n' | b'#' => true,
        _ => false,
    }
}

#[cfg_attr(not(feature = "no-inline"), inline)]
pub(crate) const fn is_white_tab(chr: u8) -> bool {
    match chr {
        b' ' | b'\t' => true,
        _ => false,
    }
}

#[cfg_attr(not(feature = "no-inline"), inline)]
pub(crate) const fn is_plain_unsafe(chr: u8) -> bool {
    match chr {
        b'\0' | b' ' | b'\t' | b'\r' | b'\n' | b',' | b'[' | b']' | b'{' | b'}' => true,
        _ => false,
    }
}

#[cfg_attr(not(feature = "no-inline"), inline)]
pub(crate) const fn is_newline(chr: u8) -> bool {
    match chr {
        b'\r' | b'\n' => true,
        _ => false,
    }
}

#[cfg_attr(not(feature = "no-inline"), inline)]
pub(crate) const fn is_flow_indicator(chr: u8) -> bool {
    match chr {
        b',' | b'[' | b']' | b'{' | b'}' => true,
        _ => false,
    }
}

#[cfg_attr(not(feature = "no-inline"), inline)]
pub(crate) fn is_uri_char(chr: u8) -> bool {
    chr == b'!'
        || (b'#'..=b',').contains(&chr)
        || (b'-'..=b';').contains(&chr)
        || (b'?'..=b'[').contains(&chr)
        || chr == b'_'
        || chr == b']'
        || chr.is_ascii_lowercase()
}

#[cfg_attr(not(feature = "no-inline"), inline)]
pub(crate) fn is_tag_char_short(chr: u8) -> bool {
    // can't contain `!`, `,` `[`, `]` , `{` , `}`
    (b'#'..=b'+').contains(&chr)
        || (b'-'..=b';').contains(&chr)
        || (b'?'..=b'Z').contains(&chr)
        || chr == b'_'
        || chr.is_ascii_lowercase()
}

#[cfg_attr(not(feature = "no-inline"), inline)]
pub(crate) fn is_tag_char(chr: u8) -> bool {
    matches!(chr, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9')
}

#[cfg_attr(not(feature = "no-inline"), inline)]
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

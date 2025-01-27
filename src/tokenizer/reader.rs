#![allow(clippy::match_like_matches_macro)]

use std::ops::ControlFlow;
use std::ops::ControlFlow::{Break, Continue};
use std::ops::{Range, RangeFrom, RangeInclusive};

use memchr::memchr3_iter;

use super::ErrorType;

pub struct StrReader<'a> {
    pub slice: &'a str,
    pub(crate) pos: usize,
    pub(crate) col: usize,
}

impl<'a> StrReader<'a> {
    pub fn new(slice: &'a str) -> StrReader<'a> {
        Self {
            slice,
            pos: 0,
            col: 0,
        }
    }
}

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
    #[inline]
    fn eof(&self) -> bool {
        self.is_eof(0)
    }
    fn eof_or_pos(&self, pos: usize) -> usize;
    fn is_eof(&self, offset: usize) -> bool;
    fn pos(&self) -> usize;
    fn set_pos(&mut self, new_pos: usize);
    fn col(&self) -> usize;
    fn set_col(&mut self, col: usize);
    fn peek_byte_at(&self, offset: usize) -> Option<u8>;
    fn peek_byte(&self) -> Option<u8>;
    fn peek_byte_unwrap(&self, offset: usize) -> u8;
    fn peek_byte_is(&self, needle: u8) -> bool {
        match self.peek_byte() {
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
    fn get_lookahead_iterator(&self, range: RangeInclusive<usize>) -> LookAroundBytes;
    #[inline]
    fn count_space_tab(&self, allow_tab: bool) -> usize {
        self.count_space_tab_range_from(self.pos().., allow_tab)
    }
    fn count_space_tab_range_from(&self, range: RangeFrom<usize>, allow_tab: bool) -> usize;
    fn count_space_tab_range(&self, range: Range<usize>, allow_tab: bool) -> usize;
    fn skip_n_spaces(&mut self, skip: usize) -> Result<(), ErrorType>;
    fn consume_bytes(&mut self, amount: usize) -> usize;
    fn slice_bytes(&self, start: usize, end: usize) -> &[u8];
    fn slice_bytes_from(&self, start: usize) -> &[u8];
    fn try_read_slice_exact(&mut self, needle: &str) -> bool;
    fn find_next_whitespace(&self) -> Option<usize>;
    fn read_break(&mut self) -> Option<(usize, usize)>;
    fn skip_whitespace(&mut self) -> usize;
    #[inline]
    fn read_line(&mut self) -> (usize, usize) {
        let (start, end, consume) = self.get_line_offset();
        self.set_pos(consume);
        self.set_col(0);
        (start, end)
    }
    fn get_line_offset(&self) -> (usize, usize, usize);
    fn read_non_comment_line(&mut self) -> (usize, usize);
}

impl<'r> Reader for StrReader<'r> {
    #[inline]
    fn eof_or_pos(&self, pos: usize) -> usize {
        pos.min(self.slice.as_bytes().len() - 1)
    }

    #[inline]
    fn is_eof(&self, offset: usize) -> bool {
        self.pos + offset >= self.slice.as_bytes().len()
    }

    fn pos(&self) -> usize {
        self.pos
    }

    fn set_pos(&mut self, new_pos: usize) {
        self.pos = new_pos;
    }

    fn col(&self) -> usize {
        self.col
    }

    fn set_col(&mut self, col: usize) {
        self.col = col;
    }

    fn peek_byte_at(&self, offset: usize) -> Option<u8> {
        self.slice.as_bytes().get(self.pos + offset).copied()
    }

    fn peek_byte(&self) -> Option<u8> {
        self.slice.as_bytes().get(self.pos).copied()
    }

    fn peek_byte_unwrap(&self, offset: usize) -> u8 {
        match self.slice.as_bytes().get(self.pos + offset) {
            Some(x) => *x,
            _ => b'\0',
        }
    }

    fn get_lookahead_iterator(&self, range: RangeInclusive<usize>) -> LookAroundBytes {
        LookAroundBytes::new(&self.slice.as_bytes(), range)
    }

    #[inline]
    fn count_space_tab_range_from(&self, range: RangeFrom<usize>, allow_tab: bool) -> usize {
        match self.slice.as_bytes()[range]
            .iter()
            .try_fold(0usize, |acc, x| is_tab_space(acc, *x, allow_tab))
        {
            Continue(x) | Break(x) => x,
        }
    }

    #[inline]
    fn count_space_tab_range(&self, range: Range<usize>, allow_tab: bool) -> usize {
        match self.slice.as_bytes()[range]
            .iter()
            .try_fold(0usize, |acc, x| is_tab_space(acc, *x, allow_tab))
        {
            Continue(x) | Break(x) => x,
        }
    }

    fn skip_n_spaces(&mut self, num_spaces: usize) -> Result<(), ErrorType> {
        let count = self.slice.as_bytes()[self.pos..]
            .iter()
            .enumerate()
            .take_while(|&(count, &x)| x == b' ' && count < num_spaces)
            .count();

        if count != num_spaces {
            return Err(ErrorType::ExpectedIndent {
                actual: count,
                expected: num_spaces,
            });
        }
        self.pos += count;

        Ok(())
    }

    #[inline(always)]
    fn consume_bytes(&mut self, amount: usize) -> usize {
        self.pos += amount;
        self.col += amount;
        self.pos
    }

    #[inline(always)]
    fn slice_bytes(&self, start: usize, end: usize) -> &'r [u8] {
        &self.slice.as_bytes()[start..end]
    }

    #[inline(always)]
    fn slice_bytes_from(&self, start: usize) -> &'r [u8] {
        &self.slice.as_bytes()[start..]
    }

    #[inline(always)]
    fn try_read_slice_exact(&mut self, needle: &str) -> bool {
        if self.slice.len() < self.pos + needle.len() {
            return false;
        }
        if self.slice.as_bytes()[self.pos..self.pos + needle.len()].starts_with(needle.as_bytes()) {
            self.pos += needle.len();
            return true;
        }
        false
    }

    fn find_next_whitespace(&self) -> Option<usize> {
        self.slice.as_bytes()[self.pos..]
            .iter()
            .position(|p| is_white_tab_or_break(*p))
    }

    fn read_break(&mut self) -> Option<(usize, usize)> {
        let start = self.pos;
        if self.peek_byte_is(b'\n') {
            self.pos += 1;
            self.col = 0;
            Some((start, start + 1))
        } else if self.peek_byte_is(b'\r') {
            let amount = match self.slice.as_bytes().get(start + 1) {
                Some(b'\n') => 2,
                _ => 1,
            };
            self.col = 0;
            self.pos += amount;
            Some((start, start + amount))
        } else {
            None
        }
    }

    fn skip_whitespace(&mut self) -> usize {
        let n = self.slice.as_bytes()[self.pos..]
            .iter()
            .position(|b| !is_white_tab_or_break(*b))
            .unwrap_or(0);
        self.consume_bytes(n);
        n
    }

    fn get_line_offset(&self) -> (usize, usize, usize) {
        let slice = self.slice.as_bytes();
        let start = self.pos;
        let remaining = slice.len() - start;
        let content = &slice[start..];
        let (n, newline) = memchr::memchr2_iter(b'\r', b'\n', content).next().map_or(
            (remaining, remaining),
            |p| {
                if content[p] == b'\r' && p < content.len() - 1 && content[p + 1] == b'\n' {
                    (p, 2)
                } else {
                    (p, 1)
                }
            },
        );
        (start, start + n, start + n + newline)
    }

    fn read_non_comment_line(&mut self) -> (usize, usize) {
        let start = self.pos;
        let content = &self.slice.as_bytes()[start..];
        let mut iter = memchr3_iter(b'\r', b'\n', b'#', content);
        let mut end = self.pos;
        let consume: usize;

        if let Some((new_end, c)) = iter.next().map(|p| (p, content[p])) {
            end = new_end;
            consume = end + 1;

            if c == b'\n' {
                self.consume_bytes(consume);
                self.col = 0;
                return (start, end);
            }
        }
        for pos in iter {
            let ascii = content[pos];
            if ascii == b'\r' && pos < content.len() - 1 && content[pos + 1] == b'\n' {
                self.consume_bytes(pos + 2);
                self.col = 0;
                return (start, end);
            } else if ascii == b'\r' || ascii == b'\n' {
                self.consume_bytes(pos + 1);
                self.col = 0;
                return (start, end);
            }
        }

        (start, end)
    }
}

#[inline]
pub fn is_tab_space(pos: usize, chr: u8, allow_tab: bool) -> ControlFlow<usize, usize> {
    if chr == b' ' || (allow_tab && chr == b'\t') {
        Continue(pos + 1)
    } else {
        Break(pos)
    }
}

#[test]
pub fn test_skip_space_tab() {
    let mut ws1 = StrReader::new("    |");
    let mut ws2 = StrReader::new("\t");
    let mut ws3 = StrReader::new("test");

    assert_eq!(4, ws1.skip_space_tab(false));
    assert_eq!(0, ws2.skip_space_tab(false));
    assert_eq!(0, ws3.skip_space_tab(false));

    let mut wst1 = StrReader::new("\t   ");
    let mut wst2 = StrReader::new("\t");
    let mut wst3 = StrReader::new("test");

    assert_eq!(4, wst1.skip_space_tab(true));
    assert_eq!(1, wst2.skip_space_tab(true));
    assert_eq!(0, wst3.skip_space_tab(true));
}

#[test]
pub fn test_readline() {
    let mut win_reader = StrReader::new("#   |\r\n");
    let mut lin_reader = StrReader::new("#   |\n");
    let mut mac_reader = StrReader::new("#   |\r");

    assert_eq!((0, 5), win_reader.read_line());
    assert_eq!(None, win_reader.peek_byte());
    assert_eq!(0, win_reader.col);

    assert_eq!((0, 5), lin_reader.read_line());
    assert_eq!(None, lin_reader.peek_byte());
    assert_eq!(0, lin_reader.col);

    assert_eq!((0, 5), mac_reader.read_line());
    assert_eq!(None, mac_reader.peek_byte());
    assert_eq!(0, mac_reader.col);
}

#[test]
pub fn test_read2lines() {
    let mut win_reader = StrReader::new("#   |\r\n \r\n");
    let mut lin_reader = StrReader::new("#   |\n\n");
    let mut mac_reader = StrReader::new("#   |\r\r");

    assert_eq!((0, 5), win_reader.read_line());
    assert_eq!(Some(b' '), win_reader.peek_byte());
    assert_eq!(0, win_reader.col);
    assert_eq!((7, 8), win_reader.read_line());
    assert_eq!(0, win_reader.col);
    assert_eq!(None, win_reader.peek_byte());

    assert_eq!((0, 5), lin_reader.read_line());
    assert_eq!(Some(b'\n'), lin_reader.peek_byte());
    assert_eq!(0, lin_reader.col);
    assert_eq!((6, 6), lin_reader.read_line());
    assert_eq!(0, lin_reader.col);
    assert_eq!(None, lin_reader.peek_byte());

    assert_eq!((0, 5), mac_reader.read_line());
    assert_eq!(Some(b'\r'), mac_reader.peek_byte());
    assert_eq!(0, mac_reader.col);
    assert_eq!((6, 6), mac_reader.read_line());
    assert_eq!(0, mac_reader.col);
    assert_eq!(None, mac_reader.peek_byte());
}

#[test]
pub fn read_non_comment_line() {
    let mut win_reader = StrReader::new("   # # \r\n");
    let mut mac_reader = StrReader::new("   # # \r");
    let mut lin_reader = StrReader::new("   # # \n");

    assert_eq!((0, 3), win_reader.read_non_comment_line());
    assert_eq!(None, win_reader.peek_byte());
    assert_eq!(9, win_reader.pos);
    assert_eq!(0, win_reader.col);

    assert_eq!((0, 3), mac_reader.read_non_comment_line());
    assert_eq!(None, mac_reader.peek_byte());
    assert_eq!(8, mac_reader.pos);
    assert_eq!(0, mac_reader.col);

    assert_eq!((0, 3), lin_reader.read_non_comment_line());
    assert_eq!(None, lin_reader.peek_byte());
    assert_eq!(8, lin_reader.pos);
    assert_eq!(0, lin_reader.col);
}

#[test]
pub fn skip_whitespace() {
    assert_eq!(0, StrReader::new("null").skip_whitespace());
    assert_eq!(0, StrReader::new("").skip_whitespace());
    assert_eq!(1, StrReader::new(" null").skip_whitespace());
    assert_eq!(2, StrReader::new("\t null").skip_whitespace());
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

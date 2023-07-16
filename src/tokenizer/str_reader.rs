use std::ops::ControlFlow::{Break, Continue};
use std::ops::Range;
use std::usize;

use memchr::{memchr, memchr2};

use reader::{is_flow_indicator, is_plain_unsafe};

use crate::tokenizer::reader::{is_uri_char, is_white_tab_or_break, LookAroundBytes};
use crate::tokenizer::{reader, ErrorType, Reader};

use super::reader::{is_newline, is_tag_char};

pub struct StrReader<'a> {
    pub slice: &'a [u8],
    pub(crate) pos: usize,
    pub(crate) col: u32,
    pub(crate) line: u32,
}

impl<'a> From<&'a str> for StrReader<'a> {
    fn from(value: &'a str) -> Self {
        Self {
            slice: value.as_bytes(),
            pos: 0,
            col: 0,
            line: 0,
        }
    }
}

impl<'a> From<&'a [u8]> for StrReader<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self {
            slice: value,
            pos: 0,
            col: 0,
            line: 0,
        }
    }
}

impl<'a> StrReader<'a> {
    #[inline]
    fn eof_or_pos(&self, pos: usize) -> usize {
        pos.min(self.slice.len() - 1)
    }

    #[inline]
    fn get_lookahead_iterator(&self, range: Range<usize>) -> LookAroundBytes {
        LookAroundBytes::new(self.slice, range)
    }

    #[inline]
    fn count_space_tab_range_from(&self, allow_tab: bool) -> usize {
        if self.pos >= self.slice.len() {
            return 0;
        }
        match self.slice[self.pos..].iter().try_fold(0usize, |pos, chr| {
            if *chr == b' ' || (allow_tab && *chr == b'\t') {
                Continue(pos + 1)
            } else {
                Break(pos)
            }
        }) {
            Continue(x) | Break(x) => x,
        }
    }

    pub(crate) fn get_quoteline_offset(&self, quote: u8) -> (usize, usize) {
        let slice = self.slice;
        let start = self.pos;
        let remaining = slice.len().saturating_sub(start);
        let content = &slice[start..];
        let n = memchr::memchr3_iter(b'\r', b'\n', quote, content)
            .next()
            .map_or(remaining, |p| if content[p] == quote { p + 1 } else { p });
        (start, start + n)
    }
}

impl<'r> Reader<()> for StrReader<'r> {
    #[inline]
    fn eof(&self) -> bool {
        self.pos >= self.slice.len()
    }

    #[inline]
    fn col(&self) -> u32 {
        self.col
    }

    #[inline]
    fn line(&self) -> u32 {
        self.line
    }

    #[inline]
    fn pos(&self) -> usize {
        self.pos
    }

    fn peek_chars(&self) -> &[u8] {
        let max = std::cmp::min(self.slice.len(), self.pos + 3);
        &self.slice[self.pos..max]
    }

    #[inline]
    fn peek_byte_at(&self, offset: usize) -> Option<u8> {
        self.slice.get(self.pos + offset).copied()
    }

    #[inline]
    fn skip_space_tab(&mut self) -> usize {
        let amount = self.count_space_tab_range_from(true);
        self.consume_bytes(amount);
        amount
    }

    #[inline]
    fn consume_bytes(&mut self, amount: usize) -> usize {
        self.pos += amount;
        self.col += TryInto::<u32>::try_into(amount).expect("Amount to not exceed u32");
        self.pos
    }

    #[inline]
    fn try_read_slice_exact(&mut self, needle: &str) -> bool {
        if self.slice.len() < self.pos + needle.len() {
            return false;
        }
        if self.slice[self.pos..self.pos + needle.len()].starts_with(needle.as_bytes()) {
            self.pos += needle.len();
            return true;
        }
        false
    }

    fn get_read_line(&self) -> (usize, usize, usize) {
        let slice = self.slice;
        let start = self.pos;
        let haystack: &[u8] = &slice[start..];
        memchr::memchr2_iter(b'\r', b'\n', haystack).next().map_or(
            (start, self.slice.len(), self.slice.len()),
            |pos| {
                if haystack[pos] == b'\r' && pos < haystack.len() - 1 && haystack[pos + 1] == b'\n'
                {
                    (start, start + pos, start + pos + 2)
                } else {
                    (start, start + pos, start + pos + 1)
                }
            },
        )
    }

    #[inline]
    fn read_line(&mut self) -> (usize, usize) {
        let (start, end, consume) = self.get_read_line();
        self.pos = consume;
        self.line += 1;
        self.col = 0;
        (start, end)
    }

    #[inline]
    fn count_spaces(&self) -> u32 {
        match self.slice[self.pos..].iter().try_fold(0usize, |pos, chr| {
            if *chr == b' ' {
                Continue(pos + 1)
            } else {
                Break(pos)
            }
        }) {
            Continue(x) | Break(x) => x as u32,
        }
    }

    fn count_whitespace_from(&self, offset: usize) -> usize {
        match self.slice[self.pos + offset..]
            .iter()
            .try_fold(offset, |pos, chr| {
                if *chr == b' ' || *chr == b'\t' || *chr == b'\r' || *chr == b'\n' {
                    Continue(pos + 1)
                } else {
                    Break(pos)
                }
            }) {
            Continue(x) | Break(x) => x,
        }
    }

    fn count_spaces_till(&self, num_spaces: u32) -> usize {
        self.slice[self.pos..]
            .iter()
            .enumerate()
            .take_while(|&(count, &x)| x == b' ' && count < num_spaces as usize)
            .count()
    }

    fn is_empty_newline(&self) -> bool {
        self.slice[self.pos..self.get_read_line().1]
            .iter()
            .rev()
            .all(|c| *c == b' ')
    }

    fn get_double_quote(&self) -> Option<usize> {
        let (line_start, line_end) = self.get_quoteline_offset(b'"');
        memchr2(b'\\', b'"', &self.slice[line_start..line_end])
    }

    fn get_double_quote_trim(&self, start_str: usize) -> Option<(usize, usize)> {
        let (_, line_end) = self.get_quoteline_offset(b'"');
        self.slice[start_str..line_end]
            .iter()
            .rposition(|chr| *chr != b' ' && *chr != b'\t')
            .map(|find| (start_str + find + 1, find + 1))
    }
    fn get_single_quote(&self) -> Option<usize> {
        let (line_start, line_end) = self.get_quoteline_offset(b'\'');
        memchr(b'\'', &self.slice[line_start..line_end])
    }
    fn get_single_quote_trim(&self, start_str: usize) -> Option<(usize, usize)> {
        let (_, line_end) = self.get_quoteline_offset(b'\'');
        self.slice[start_str..line_end]
            .iter()
            .rposition(|chr| *chr != b' ' && *chr != b'\t')
            .map(|find| (start_str + find + 1, find + 1))
    }

    fn read_plain_one_line(
        &mut self,
        offset_start: Option<usize>,
        had_comment: &mut bool,
        in_flow_collection: bool,
    ) -> (usize, usize, usize) {
        let start = offset_start.unwrap_or(self.pos);
        let (_, line_end, _) = self.get_read_line();
        let end = self.pos + 1;
        let line_end = StrReader::eof_or_pos(self, line_end);
        let mut end_of_str = end;

        for (prev, curr, next, pos) in self.get_lookahead_iterator(end..line_end) {
            // ns-plain-char  prevent ` #`
            if curr == b'#' && is_white_tab_or_break(prev) {
                // if we encounter two or more comment print error and try to recover
                return if *had_comment {
                    (start, end_of_str, end_of_str - start)
                } else {
                    *had_comment = true;
                    (start, end_of_str, end_of_str - start)
                };
            }

            // ns-plain-char prevent `: `
            // or `:{`  in flow collections
            if curr == b':' && is_plain_unsafe(next) {
                break;
            }

            // // if current character is a flow indicator, break
            if in_flow_collection && is_flow_indicator(curr) {
                break;
            }

            if is_white_tab_or_break(curr) {
                if is_newline(curr) {
                    break;
                }
            } else {
                end_of_str = pos + 1;
            }
        }
        (start, end_of_str, end_of_str - start)
    }

    fn count_space_then_tab(&mut self) -> (u32, usize) {
        let spaces = match self.slice[self.pos..]
            .iter()
            .try_fold(0u32, |ws_cnt, chr| match *chr {
                b' ' => Continue(ws_cnt + 1),
                _ => Break(ws_cnt),
            }) {
            Continue(x) | Break(x) => x,
        };
        let tabs = match self.slice[self.pos..]
            .iter()
            .try_fold(0usize, |ws_cnt, chr| match *chr {
                b' ' | b'\t' => Continue(ws_cnt + 1),
                _ => Break(ws_cnt),
            }) {
            Continue(x) | Break(x) => x,
        };
        (spaces, tabs)
    }

    fn consume_anchor_alias(&mut self) -> (usize, usize) {
        let start = self.consume_bytes(1);

        let amount = self.slice[self.pos..]
            .iter()
            .position(|p| is_white_tab_or_break(*p) || is_flow_indicator(*p))
            .unwrap_or(self.slice.len() - self.pos);
        self.consume_bytes(amount);
        (start, start + amount)
    }

    fn read_tag(&mut self) -> (Option<ErrorType>, usize, usize, usize) {
        match self.peek_chars() {
            [b'!', b'<', ..] => {
                let start = self.consume_bytes(2);
                let (line_start, line_end, _) = self.get_read_line();
                let haystack = &self.slice[line_start..line_end];
                if let Some(end) = memchr(b'>', haystack) {
                    self.consume_bytes(end + 1);
                    (None, start, start + end, 0)
                } else {
                    self.skip_space_tab();
                    (Some(ErrorType::UnfinishedTag), 0, 0, 0)
                }
            }
            [b'!', peek, ..] if is_white_tab_or_break(*peek) => {
                let start = self.pos;
                self.consume_bytes(1);
                (None, start, start + 1, start + 1)
            }
            [b'!', ..] => {
                let start = self.pos;
                self.consume_bytes(1);
                let (_, line_end, _) = self.get_read_line();
                let haystack = &self.slice[self.pos..line_end];
                let find_pos = match memchr(b'!', haystack) {
                    Some(find) => find + 1,
                    None => 0,
                };
                let mid: usize = self.pos + find_pos;
                let amount = self.slice[mid..line_end]
                    .iter()
                    .position(|c| !is_uri_char(*c))
                    .unwrap_or(line_end.saturating_sub(mid));
                let end = self.consume_bytes(amount + find_pos);
                (None, start, mid, end)
            }
            _ => panic!("Tag must start with `!`"),
        }
    }

    fn read_tag_handle(&mut self) -> Result<Vec<u8>, ErrorType> {
        match self.peek_chars() {
            [b'!', x, ..] if *x == b' ' || *x == b'\t' => {
                self.consume_bytes(1);
                self.skip_space_tab();
                Ok(vec![b'!'])
            }
            [b'!', _x, ..] => {
                let start = self.pos;
                self.consume_bytes(1);
                let amount: usize = self.slice[self.pos..]
                    .iter()
                    .position(|c: &u8| !is_tag_char(*c))
                    .unwrap_or(self.slice.len() - self.pos);
                self.consume_bytes(amount);
                if self.peek_byte_is(b'!') {
                    let bac = self.slice[start..start + amount + 2].to_vec();
                    self.consume_bytes(1);
                    Ok(bac)
                } else {
                    self.read_line();
                    Err(ErrorType::TagNotTerminated)
                }
            }
            [x, ..] => {
                let err = Err(ErrorType::InvalidTagHandleCharacter { found: *x as char });
                self.read_line();
                err
            }
            &[] => Err(ErrorType::UnexpectedEndOfFile),
        }
    }

    fn read_tag_uri(&mut self) -> Option<(usize, usize)> {
        if self.peek_byte_at(0).map_or(false, is_uri_char) {
            let start = self.pos;
            let amount = self.slice[start..]
                .iter()
                .position(|c| !is_uri_char(*c))
                .unwrap_or(self.slice.len() - self.pos);
            let end = self.consume_bytes(amount);
            Some((start, end))
        } else {
            None
        }
    }

    fn read_break(&mut self) -> Option<(usize, usize)> {
        let start = self.pos;
        if self.peek_byte_is(b'\n') {
            self.pos += 1;
            self.col = 0;
            self.line += 1;
            Some((start, start + 1))
        } else if self.peek_byte_is(b'\r') {
            let amount = match self.slice.get(start + 1) {
                Some(b'\n') => 2,
                _ => 1,
            };
            self.col = 0;
            self.pos += amount;
            self.line += 1;
            Some((start, start + amount))
        } else {
            None
        }
    }
}

#[test]
pub fn test_offset() {
    use crate::tokenizer::Slicer;

    let input = "\n  rst\n".as_bytes();
    let mut reader = StrReader::from(input);
    let (start, end, consume) = reader.get_read_line();
    assert_eq!(start, 0);
    assert_eq!(end, 0);
    assert_eq!(b"", input.slice(start, end));
    assert_eq!(consume, 1);
    reader.read_line();
    let (start, end, consume) = reader.get_read_line();
    assert_eq!(start, 1);
    assert_eq!(end, 6);
    assert_eq!(b"  rst", input.slice(start, end));
    assert_eq!(consume, 7);
    reader.read_line();
    let (start, end, consume) = reader.get_read_line();
    assert_eq!(start, 7);
    assert_eq!(end, 7);
    assert_eq!(b"", input.slice(start, end));
    assert_eq!(consume, 7);
}

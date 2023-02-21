#![allow(clippy::match_like_matches_macro)]

use std::collections::VecDeque;
use std::ops::ControlFlow;
use std::ops::ControlFlow::{Break, Continue};
use std::ops::{Range, RangeFrom, RangeInclusive};

use crate::tokenizer::SpanToken::Separator;
use memchr::memchr3_iter;
use ErrorType::UnexpectedComment;
use ParserState::{BlockMap, BlockSeq};
use SpanToken::{ErrorToken, MappingStart, MarkEnd, MarkStart, NewLine, Space};

use super::spanner::ParserState;
use super::{ErrorType, SpanToken};

pub struct StrReader<'a> {
    pub slice: &'a [u8],
    pub(crate) pos: usize,
    pub(crate) col: usize,
}

impl<'a> StrReader<'a> {
    pub fn new(slice: &'a str) -> StrReader<'a> {
        Self {
            slice: slice.as_bytes(),
            pos: 0,
            col: 0,
        }
    }

    fn read_plain_one_line(
        &mut self,
        allow_minus: bool,
        had_comment: &mut bool,
        in_flow_collection: bool,
        tokens: &mut Vec<SpanToken>,
    ) -> Option<(usize, usize)> {
        let start = self.pos();

        if !(allow_minus && self.peek_byte_is(b'-'))
            && (self.eof()
                || self.peek_byte_at_check(0, is_white_tab_or_break)
                || self.peek_byte_at_check(0, is_indicator)
                || (self.peek_byte_is(b'-') && !self.peek_byte_at_check(1, is_white_tab))
                || ((self.peek_byte_is(b'?') || self.peek_byte_is(b':'))
                    && !self.peek_byte_at_check(1, is_white_tab_or_break)))
        {
            return None;
        }

        let end = self.consume_bytes(1);
        let (_, line_end, _) = self.get_line_offset();
        let line_end = self.eof_or_pos(line_end);
        let mut end_of_str = end;

        for (prev, curr, next, pos) in self.get_lookahead_iterator(end..=line_end) {
            // ns-plain-char  prevent ` #`
            if curr == b'#' && is_white_tab_or_break(prev) {
                // if we encounter two or more comment print error and try to recover
                if *had_comment {
                    tokens.push(ErrorToken(UnexpectedComment))
                } else {
                    *had_comment = true;
                    self.set_pos(line_end);
                    return Some((start, end_of_str));
                }
                break;
            }

            // ns-plain-char prevent `: `
            // or `:{`  in flow collections
            if curr == b':' && !ns_plain_safe(next, in_flow_collection) {
                // commit any uncommitted character, but ignore first character
                if !is_white_tab(prev) && pos != end {
                    end_of_str += 1;
                }
                break;
            }

            // if current character is a flow indicator, break
            if is_flow_indicator(curr) {
                break;
            }

            if is_white_tab_or_break(curr) {
                // commit any uncommitted character, but ignore first character
                if !is_white_tab_or_break(prev) && pos != end {
                    end_of_str += 1;
                }
                continue;
            }
            end_of_str = pos;
        }

        self.set_pos(end_of_str);
        Some((start, end_of_str))
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

enum ChompIndicator {
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
    // Refactor
    fn read_block_seq(&mut self, indent: usize) -> Option<ParserState>;
    fn read_single_quote(&mut self, is_implicit: bool, tokens: &mut VecDeque<SpanToken>);
    fn read_plain_scalar(
        &mut self,
        start_indent: usize,
        curr_state: &ParserState,
    ) -> (Vec<SpanToken>, Option<ParserState>);
    fn skip_separation_spaces(&mut self, allow_comments: bool) -> usize;
    fn read_double_quote(&mut self, is_implicit: bool, tokens: &mut VecDeque<SpanToken>);
    fn read_block_scalar(
        &mut self,
        literal: bool,
        curr_state: &ParserState,
        tokens: &mut VecDeque<SpanToken>,
    );
}

impl<'r> Reader for StrReader<'r> {
    #[inline]
    fn eof_or_pos(&self, pos: usize) -> usize {
        pos.min(self.slice.len() - 1)
    }

    #[inline]
    fn is_eof(&self, offset: usize) -> bool {
        self.pos + offset >= self.slice.len()
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
        self.slice.get(self.pos + offset).copied()
    }

    fn peek_byte(&self) -> Option<u8> {
        self.slice.get(self.pos).copied()
    }

    fn peek_byte_unwrap(&self, offset: usize) -> u8 {
        match self.slice.get(self.pos + offset) {
            Some(x) => *x,
            _ => b'\0',
        }
    }

    fn get_lookahead_iterator(&self, range: RangeInclusive<usize>) -> LookAroundBytes {
        LookAroundBytes::new(self.slice, range)
    }

    #[inline]
    fn count_space_tab_range_from(&self, range: RangeFrom<usize>, allow_tab: bool) -> usize {
        match self.slice[range]
            .iter()
            .try_fold(0usize, |acc, x| is_tab_space(acc, *x, allow_tab))
        {
            Continue(x) | Break(x) => x,
        }
    }

    #[inline]
    fn count_space_tab_range(&self, range: Range<usize>, allow_tab: bool) -> usize {
        match self.slice[range]
            .iter()
            .try_fold(0usize, |acc, x| is_tab_space(acc, *x, allow_tab))
        {
            Continue(x) | Break(x) => x,
        }
    }

    fn skip_n_spaces(&mut self, num_spaces: usize) -> Result<(), ErrorType> {
        let count = self.slice[self.pos..]
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
        &self.slice[start..end]
    }

    #[inline(always)]
    fn slice_bytes_from(&self, start: usize) -> &'r [u8] {
        &self.slice[start..]
    }

    #[inline(always)]
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

    fn find_next_whitespace(&self) -> Option<usize> {
        self.slice[self.pos..]
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
            let amount = match self.slice.get(start + 1) {
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
        let n = self.slice[self.pos..]
            .iter()
            .position(|b| !is_white_tab_or_break(*b))
            .unwrap_or(0);
        self.consume_bytes(n);
        n
    }

    fn get_line_offset(&self) -> (usize, usize, usize) {
        let slice = self.slice;
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
        let content = &self.slice[start..];
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

    fn read_block_seq(&mut self, indent: usize) -> Option<ParserState> {
        if self.peek_byte_at_check(1, is_white_tab_or_break) {
            let new_indent: usize = self.col();
            if self.peek_byte_at_check(1, is_newline) {
                self.consume_bytes(1);
                self.read_break();
            } else {
                self.consume_bytes(2);
            }

            if new_indent >= indent {
                return Some(BlockSeq(new_indent));
            }
        }
        None
    }

    fn read_single_quote(&mut self, is_implicit: bool, tokens: &mut VecDeque<SpanToken>) {
        self.consume_bytes(1);

        while !self.eof() {
            let (line_start, line_end, _) = self.get_line_offset();
            let pos = memchr::memchr(b'\'', self.slice_bytes(line_start, line_end));
            match pos {
                Some(len) => {
                    // Converts double '' to ' hence why we consume one extra char
                    if self.peek_byte_at(len + 1) == Some(b'\'') {
                        tokens.push_back(MarkStart(line_start));
                        tokens.push_back(MarkEnd(line_start + len + 1));
                        self.consume_bytes(len + 2);
                        continue;
                    } else {
                        tokens.push_back(MarkStart(line_start));
                        tokens.push_back(MarkEnd(line_start + len));
                        self.consume_bytes(len + 1);
                        break;
                    }
                }
                None => {
                    tokens.push_back(MarkStart(line_start));
                    tokens.push_back(MarkEnd(line_end));
                    tokens.push_back(Space);
                    self.read_line();
                    self.skip_space_tab(is_implicit);
                }
            }
        }
    }

    fn read_plain_scalar(
        &mut self,
        start_indent: usize,
        curr_state: &ParserState,
    ) -> (Vec<SpanToken>, Option<ParserState>) {
        let mut allow_minus = false;
        let mut first_line_block = !curr_state.in_flow_collection();

        let mut num_newlines = 0;
        let mut tokens = vec![];
        let mut curr_indent = self.col();
        let init_indent = if matches!(curr_state, ParserState::BlockMap(_)) {
            self.col()
        } else {
            start_indent
        };
        let mut had_comment = false;
        let mut new_state = None;

        while !self.eof() {
            // if plain scalar is less indented than previous
            // It can be
            // a) Part of BlockMap
            // b) An error outside of block map
            if curr_indent < init_indent {
                if matches!(curr_state, ParserState::BlockMap(_)) {
                    tokens.push(Separator);
                } else if !curr_state.is_block_col() {
                    self.read_line();
                    tokens.push(ErrorToken(ErrorType::ExpectedIndent {
                        actual: curr_indent,
                        expected: start_indent,
                    }));
                }
                break;
            }

            let (start, end) = match self.read_plain_one_line(
                allow_minus,
                &mut had_comment,
                curr_state.in_flow_collection(),
                &mut tokens,
            ) {
                Some(x) => x,
                None => break,
            };

            self.skip_space_tab(true);

            let chr = self.peek_byte_unwrap(0);

            if first_line_block && chr == b':' {
                if curr_state.is_new_block_col(curr_indent) {
                    new_state = Some(BlockMap(curr_indent));
                    tokens.push(MappingStart);
                }
                tokens.push(MarkStart(start));
                tokens.push(MarkEnd(end));
                break;
            }

            match num_newlines {
                x if x == 1 => tokens.push(Space),
                x if x > 1 => tokens.push(NewLine(num_newlines)),
                _ => {}
            }

            tokens.push(MarkStart(start));
            tokens.push(MarkEnd(end));
            first_line_block = false;

            if is_newline(chr) {
                let folded_newline = self.skip_separation_spaces(false);
                if self.col() >= curr_state.indent(0) {
                    num_newlines = folded_newline as u32;
                }
                curr_indent = self.col();
            }

            if curr_state.in_flow_collection() && is_flow_indicator(chr) {
                break;
            }

            match (self.peek_byte_unwrap(0), curr_state) {
                (b'-', BlockSeq(ind)) if self.col() == *ind => {
                    self.consume_bytes(1);
                    tokens.push(Separator);
                    break;
                }
                (b'-', BlockSeq(ind)) if self.col() < *ind => {
                    self.read_line();
                    let err_type = ErrorType::ExpectedIndent {
                        expected: *ind,
                        actual: curr_indent,
                    };
                    tokens.push(ErrorToken(err_type));
                    break;
                }
                (b'-', BlockSeq(ind)) if self.col() > *ind => {
                    allow_minus = true;
                }
                _ => {}
            }
        }
        (tokens, new_state)
    }

    fn skip_separation_spaces(&mut self, allow_comments: bool) -> usize {
        let mut num_breaks = 0;
        let mut found_eol = true;
        while !self.eof() {
            self.skip_space_tab(true);

            if allow_comments && self.peek_byte_is(b'#') {
                self.read_line();
                found_eol = true;
                num_breaks += 1;
            }

            if self.read_break().is_some() {
                num_breaks += 1;
                found_eol = true;
            }

            if !found_eol {
                break;
            } else {
                self.skip_space_tab(false);
                found_eol = false;
            }
        }
        num_breaks
    }

    fn read_double_quote(&mut self, is_implicit: bool, tokens: &mut VecDeque<SpanToken>) {
        self.consume_bytes(1);

        while !self.eof() {
            let (line_start, line_end, _) = self.get_line_offset();
            let pos = memchr::memchr(b'"', self.slice_bytes(line_start, line_end));
            match pos {
                Some(len) if len > 1 => {
                    // Check for `\` escape
                    if self.peek_byte_at(len - 1) != Some(b'\\') {
                        tokens.push_back(MarkStart(line_start));
                        tokens.push_back(MarkEnd(line_start + len));
                        self.consume_bytes(len + 1);
                        break;
                    } else {
                        tokens.push_back(MarkStart(line_start));
                        tokens.push_back(MarkEnd(line_start + len - 1));
                        // we add the escaped `"` in `\"`
                        tokens.push_back(MarkStart(line_start + len));
                        tokens.push_back(MarkEnd(line_start + len + 1));
                        self.consume_bytes(len + 1);
                        continue;
                    }
                }
                Some(len) => {
                    tokens.push_back(MarkStart(line_start));
                    tokens.push_back(MarkEnd(line_start + len));
                    self.consume_bytes(len + 1);
                    break;
                }
                None => {
                    tokens.push_back(MarkStart(line_start));
                    tokens.push_back(MarkEnd(line_end));
                    tokens.push_back(Space);
                    self.read_line();
                    self.skip_space_tab(is_implicit);
                }
            }
        }
    }

    fn read_block_scalar(
        &mut self,
        literal: bool,
        curr_state: &ParserState,
        tokens: &mut VecDeque<SpanToken>,
    ) {
        self.consume_bytes(1);
        let mut chomp = ChompIndicator::Clip;
        let mut indentation: usize = 0;

        match (self.peek_byte_unwrap(0), self.peek_byte_unwrap(1)) {
            (b'-', len) | (len, b'-') if matches!(len, b'1'..=b'9') => {
                self.consume_bytes(2);
                chomp = ChompIndicator::Strip;
                indentation = curr_state.indent(0) + (len - b'0') as usize;
            }
            (b'+', len) | (len, b'+') if matches!(len, b'1'..=b'9') => {
                self.consume_bytes(2);
                chomp = ChompIndicator::Keep;
                indentation = curr_state.indent(0) + (len - b'0') as usize;
            }
            (b'-', _) => {
                self.consume_bytes(1);
                chomp = ChompIndicator::Strip;
            }
            (b'+', _) => {
                self.consume_bytes(1);
                chomp = ChompIndicator::Keep;
            }
            (len, _) if matches!(len, b'1'..=b'9') => {
                self.consume_bytes(1);
                indentation = curr_state.indent(0) + (len - b'0') as usize;
            }
            _ => {}
        }

        // allow comment in first line of block scalar
        self.skip_space_tab(true);
        if self.peek_byte_is(b'#') {
            self.read_line();
        } else if self.read_break().is_none() {
            tokens.push_back(ErrorToken(ErrorType::ExpectedNewline));
            return;
        }

        let mut new_line_token = 0;
        let mut trailing = vec![];
        let mut is_trailing_comment = false;
        let mut previous_indent = 0;
        while !self.eof() {
            let curr_indent = curr_state.indent(0);

            if let (b'-', BlockSeq(ind)) = (self.peek_byte_unwrap(curr_indent), curr_state) {
                if self.col() + curr_indent == *ind {
                    self.consume_bytes(1 + curr_indent);
                    trailing.push(Separator);
                    break;
                }
            }

            // count indents important for folded scalars
            let newline_indent = self.count_space_tab(false);

            if !is_trailing_comment
                && newline_indent < indentation
                && self.peek_byte_unwrap(newline_indent) == b'#'
            {
                trailing.push(NewLine(new_line_token - 1));
                is_trailing_comment = true;
                new_line_token = 1;
            };

            let newline_is_empty = self.peek_byte_at_check(newline_indent, is_newline)
                || (is_trailing_comment && self.peek_byte_unwrap(newline_indent) == b'#');

            if indentation == 0 && newline_indent > 0 && !newline_is_empty {
                indentation = newline_indent;
            }

            if newline_is_empty {
                new_line_token += 1;
                self.read_line();
                continue;
            } else if let Err(x) = self.skip_n_spaces(indentation) {
                tokens.push_back(ErrorToken(x));
                break;
            }

            let (start, end) = self.read_line();
            if start != end {
                if new_line_token > 0 {
                    let token =
                        if new_line_token == 1 && !literal && previous_indent == newline_indent {
                            Space
                        } else {
                            NewLine(new_line_token)
                        };
                    tokens.push_back(token);
                }
                previous_indent = newline_indent;
                tokens.push_back(MarkStart(start));
                tokens.push_back(MarkEnd(end));
                new_line_token = 1;
            }
        }
        match chomp {
            ChompIndicator::Keep => {
                if is_trailing_comment {
                    new_line_token = 1;
                }
                trailing.insert(0, NewLine(new_line_token));
                tokens.extend(trailing);
            }
            ChompIndicator::Clip => {
                trailing.insert(0, NewLine(1));
                tokens.extend(trailing);
            }
            ChompIndicator::Strip => {}
        }
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

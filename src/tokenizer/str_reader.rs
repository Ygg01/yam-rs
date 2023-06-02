use std::ops::ControlFlow::{Break, Continue};
use std::ops::Range;
use std::usize;

use memchr::{memchr, memchr2};

use reader::{is_flow_indicator, ns_plain_safe};

use crate::tokenizer::reader::{is_uri_char, is_white_tab_or_break, LookAroundBytes};
use crate::tokenizer::ErrorType::UnexpectedComment;
use crate::tokenizer::LexerToken::*;
use crate::tokenizer::{reader, ErrorType, Reader};

use super::reader::{is_newline, is_tag_char, is_valid_escape};

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

pub(crate) enum QuoteState {
    Start,
    Trim,
    End,
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

    fn skip_detect_space_tab(&mut self, has_tab: &mut bool) {
        let amount = match self.slice[self.pos..].iter().try_fold(0usize, |pos, chr| {
            if !*has_tab && *chr == b'\t' {
                *has_tab = true;
            }
            if *chr == b' ' || *chr == b'\t' {
                Continue(pos + 1)
            } else {
                Break(pos)
            }
        }) {
            Continue(x) | Break(x) => x,
        };
        self.consume_bytes(amount);
    }

    pub(crate) fn get_line_offset(&self) -> (usize, usize, usize) {
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

    #[inline]
    fn update_newlines(&mut self, newspaces: &mut Option<usize>, start_str: &mut usize) {
        *newspaces = Some(self.skip_separation_spaces(true).0.saturating_sub(1) as usize);
        *start_str = self.pos;
    }

    fn quote_start(
        &mut self,
        start_str: &mut usize,
        newspaces: &mut Option<usize>,
        tokens: &mut Vec<usize>,
        errors: &mut Vec<ErrorType>,
    ) -> QuoteState {
        let (_, line_end) = self.get_quoteline_offset(b'"');
        if let Some(pos) = memchr2(b'\\', b'"', &self.slice[self.pos..line_end]) {
            let match_pos = self.consume_bytes(pos);
            match self.peek_chars(&mut ()) {
                [b'\\', b'\t', ..] => {
                    emit_token_mut(start_str, match_pos, newspaces, tokens);
                    emit_token_mut(&mut (match_pos + 1), match_pos + 2, newspaces, tokens);
                    self.consume_bytes(2);
                    *start_str = self.pos;
                }
                [b'\\', b't', ..] => {
                    emit_token_mut(start_str, match_pos + 2, newspaces, tokens);
                    self.consume_bytes(2);
                }
                [b'\\', b'\r' | b'\n', ..] => {
                    emit_token_mut(start_str, match_pos, newspaces, tokens);
                    self.consume_bytes(1);
                    self.update_newlines(&mut None, start_str);
                }
                [b'\\', b'"', ..] => {
                    emit_token_mut(start_str, match_pos, newspaces, tokens);
                    *start_str = self.pos + 1;
                    self.consume_bytes(2);
                }
                [b'\\', b'/', ..] => {
                    emit_token_mut(start_str, match_pos, newspaces, tokens);
                    *start_str = self.consume_bytes(1);
                }
                [b'\\', x, ..] => {
                    if is_valid_escape(*x) {
                        emit_token_mut(start_str, match_pos, newspaces, tokens);
                        self.consume_bytes(2);
                    } else {
                        tokens.insert(0, ErrorToken as usize);
                        errors.push(ErrorType::InvalidEscapeCharacter);
                        self.consume_bytes(2);
                    }
                }
                [b'"', ..] => {
                    emit_token_mut(start_str, match_pos, newspaces, tokens);
                    self.pos += 1;
                    return QuoteState::End;
                }
                [b'\\'] => {
                    self.pos += 1;
                }
                _ => {}
            }
            QuoteState::Start
        } else {
            QuoteState::Trim
        }
    }

    fn quote_trim(
        &mut self,
        start_str: &mut usize,
        newspaces: &mut Option<usize>,
        errors: &mut Vec<ErrorType>,
        tokens: &mut Vec<usize>,
    ) -> QuoteState {
        let (_, line_end) = self.get_quoteline_offset(b'"');

        if self.col == 0 && (matches!(self.peek_chars(&mut ()), b"..." | b"---")) {
            errors.push(ErrorType::UnexpectedEndOfStream);
            tokens.insert(0, ErrorToken as usize);
        };

        if let Some((match_pos, len)) = self.slice[*start_str..line_end]
            .iter()
            .rposition(|chr| *chr != b' ' && *chr != b'\t')
            .map(|find| (*start_str + find + 1, find + 1))
        {
            emit_token_mut(start_str, match_pos, newspaces, tokens);
            self.consume_bytes(len);
        } else {
            self.update_newlines(newspaces, start_str);
        }

        match self.peek_byte() {
            Some(b'\n') => {
                self.update_newlines(newspaces, start_str);
                QuoteState::Start
            }
            Some(b'"') | None => {
                self.consume_bytes(1);
                QuoteState::End
            }
            Some(_) => QuoteState::Start,
        }
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

    fn peek_chars(&self, _buf: &mut ()) -> &[u8] {
        let max = std::cmp::min(self.slice.len(), self.pos + 3);
        &self.slice[self.pos..max]
    }

    #[inline]
    fn peek_byte(&self) -> Option<u8> {
        self.slice.get(self.pos).copied()
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

    #[inline(always)]
    fn consume_bytes(&mut self, amount: usize) -> usize {
        self.pos += amount;
        self.col += TryInto::<u32>::try_into(amount).expect("Amount to not exceed u32");
        self.pos
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

    #[inline]
    fn read_line(&mut self) -> (usize, usize) {
        let (start, end, consume) = self.get_line_offset();
        self.pos = consume;
        self.line += 1;
        self.col = 0;
        (start, end)
    }

    fn read_plain_one_line(
        &mut self,
        offset_start: Option<usize>,
        had_comment: &mut bool,
        in_flow_collection: bool,
    ) -> (usize, usize, Option<ErrorType>) {
        let start = offset_start.unwrap_or(self.pos);
        let (_, line_end, _) = self.get_line_offset();
        let end = self.consume_bytes(1);
        let mut pos_end = end;
        let line_end = StrReader::eof_or_pos(self, line_end);
        let mut end_of_str = end;

        for (prev, curr, next, pos) in self.get_lookahead_iterator(end..line_end) {
            // ns-plain-char  prevent ` #`
            if curr == b'#' && is_white_tab_or_break(prev) {
                // if we encounter two or more comment print error and try to recover
                return if *had_comment {
                    self.pos = line_end;
                    (start, end_of_str, Some(UnexpectedComment))
                } else {
                    *had_comment = true;
                    self.pos = line_end;
                    (start, end_of_str, None)
                };
            }

            // ns-plain-char prevent `: `
            // or `:{`  in flow collections
            if curr == b':' && !ns_plain_safe(next) {
                pos_end = end_of_str;
                break;
            }

            // // if current character is a flow indicator, break
            if in_flow_collection && is_flow_indicator(curr) {
                pos_end = end_of_str;
                break;
            }

            if is_white_tab_or_break(curr) {
                if is_newline(curr) {
                    pos_end = line_end;
                    break;
                }
                pos_end = pos;
            } else {
                end_of_str = pos + 1;
                pos_end = end_of_str;
            }
        }
        self.pos = pos_end;
        (start, end_of_str, None)
    }

    fn read_double_quote(&mut self, errors: &mut Vec<ErrorType>) -> Vec<usize> {
        let mut start_str = self.consume_bytes(1);
        let mut tokens = vec![ScalarDoubleQuote as usize];
        let mut newspaces = None;
        let mut state = QuoteState::Start;

        loop {
            state = match state {
                QuoteState::Start => {
                    self.quote_start(&mut start_str, &mut newspaces, &mut tokens, errors)
                }
                QuoteState::Trim => {
                    self.quote_trim(&mut start_str, &mut newspaces, errors, &mut tokens)
                }
                QuoteState::End => break,
            };
        }
        tokens.push(ScalarEnd as usize);
        tokens
    }

    fn read_single_quote(&mut self, is_implicit: bool) -> Vec<usize> {
        self.consume_bytes(1);
        let mut tokens = Vec::new();
        tokens.push(ScalarSingleQuote as usize);

        while !self.eof() {
            let (line_start, line_end) = self.get_quoteline_offset(b'\'');
            let pos = memchr::memchr(b'\'', &self.slice[line_start..line_end]);
            match pos {
                Some(len) => {
                    // Converts double '' to ' hence why we consume one extra char
                    let offset = len + 1;
                    if self.slice.get(self.pos + offset).copied() == Some(b'\'') {
                        tokens.push(line_start);
                        tokens.push(line_start + len + 1);
                        self.consume_bytes(len + 2);
                        continue;
                    } else {
                        tokens.push(line_start);
                        tokens.push(line_start + len);
                        self.consume_bytes(len + 1);
                        break;
                    }
                }
                None => {
                    tokens.push(line_start);
                    tokens.push(line_end);
                    tokens.push(NewLine as usize);
                    tokens.push(0);
                    self.read_line();
                    let amount = self.count_space_tab_range_from(is_implicit);
                    self.consume_bytes(amount);
                }
            }
        }
        tokens.push(ScalarEnd as usize);
        tokens
    }

    fn skip_separation_spaces(&mut self, allow_comments: bool) -> (u32, bool) {
        let mut num_breaks = 0;
        let mut found_eol = true;
        let mut has_tab = false;
        while !self.eof() && self.peek_byte().map_or(false, is_white_tab_or_break) {
            self.skip_detect_space_tab(&mut has_tab);

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
                self.skip_detect_space_tab(&mut has_tab);
                found_eol = false;
            }
        }
        (num_breaks, has_tab)
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
        match self.peek_chars(&mut ()) {
            [b'!', b'<', ..] => {
                let start = self.consume_bytes(2);
                let (line_start, line_end, _) = self.get_line_offset();
                let haystack = &self.slice[line_start..line_end];
                if let Some(end) = memchr(b'>', haystack) {
                    let err = if self.slice[self.pos + end + 1] != b'!' {
                        Some(ErrorType::UnfinishedTag)
                    } else {
                        None
                    };
                    self.consume_bytes(end + 1);
                    (err, start, end, 0)
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
                let (_, line_end, _) = self.get_line_offset();
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
        match self.peek_chars(&mut ()) {
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
        if self.peek_byte().map_or(false, is_uri_char) {
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

    fn is_empty_newline(&self) -> bool {
        self.slice[self.pos..self.get_line_offset().1]
            .iter()
            .rev()
            .all(|c| *c == b' ')
    }

    fn count_spaces_till(&self, num_spaces: u32) -> usize {
        self.slice[self.pos..]
            .iter()
            .enumerate()
            .take_while(|&(count, &x)| x == b' ' && count < num_spaces as usize)
            .count()
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

#[test]
pub fn test_plain_scalar() {
    let mut reader = StrReader::from("ab  \n xyz ");
    let mut had_comment = true;
    let (start, end, _) = reader.read_plain_one_line(None, &mut had_comment, false);
    assert_eq!("ab".as_bytes(), &reader.slice[start..end]);
    reader.skip_separation_spaces(false);
    let (start, end, _) = reader.read_plain_one_line(None, &mut had_comment, false);
    assert_eq!("xyz".as_bytes(), &reader.slice[start..end]);
}

/*
#[test]
pub fn test_offset() {
    let mut reader = StrReader::from("\n  rst\n");
    let (start, end, consume) = reader.get_line_offset();
    assert_eq!(start, 0);
    assert_eq!(end, 0);
    assert_eq!(b"", reader.slice(start, end));
    assert_eq!(consume, 1);
    reader.read_line();
    let (start, end, consume) = reader.get_line_offset();
    assert_eq!(start, 1);
    assert_eq!(end, 6);
    assert_eq!(b"  rst", reader.slice(start, end));
    assert_eq!(consume, 7);
    reader.read_line();
    let (start, end, consume) = reader.get_line_offset();
    assert_eq!(start, 7);
    assert_eq!(end, 7);
    assert_eq!(b"", reader.slice(start, end));
    assert_eq!(consume, 7);
}
 */
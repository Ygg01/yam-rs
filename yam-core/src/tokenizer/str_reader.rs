use core::ops::ControlFlow::{Break, Continue};
use core::ops::Range;
use core::usize;

use alloc::vec;
use alloc::vec::Vec;

use memchr::memchr;

use reader::{is_flow_indicator, is_plain_unsafe};

use crate::tokenizer::lexer::{push_error, DirectiveState};
use crate::tokenizer::reader::{is_uri_char, is_white_tab_or_break, LexMutState, LookAroundBytes};
use crate::tokenizer::ErrorType::TwoDirectivesFound;
use crate::tokenizer::LexerToken::{DirectiveYaml, NewLine};
use crate::tokenizer::{reader, ErrorType, Reader};

use super::reader::{is_newline, is_tag_char, is_tag_char_short};

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
}

impl<'r> Reader for StrReader<'r> {
    #[inline]
    fn eof(&mut self) -> bool {
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
    fn offset(&self) -> usize {
        self.pos
    }

    fn peek_chars(&mut self) -> &[u8] {
        let max = core::cmp::min(self.slice.len(), self.pos + 2);
        &self.slice[self.pos..max]
    }

    #[inline]
    fn peek_byte_at(&mut self, offset: usize) -> Option<u8> {
        self.slice.get(self.pos + offset).copied()
    }

    fn peek_stream_ending(&mut self) -> bool {
        let max = core::cmp::min(self.slice.len(), self.pos + 3);
        let chars = &self.slice[self.pos..max];
        (chars == b"..." || chars == b"---")
            && self.peek_byte_at(3).map_or(true, |c| {
                c == b'\t' || c == b' ' || c == b'\r' || c == b'\n' || c == b'[' || c == b'{'
            })
            && self.col() == 0
    }

    #[inline]
    fn skip_space_tab(&mut self) -> usize {
        let amount = self.count_space_tab_range_from(true);
        self.skip_bytes(amount);
        amount
    }

    #[inline]
    fn skip_bytes(&mut self, amount: usize) -> usize {
        self.pos += amount;
        self.col += TryInto::<u32>::try_into(amount).expect("Amount of indents can't exceed u32");
        self.pos
    }

    fn save_bytes(
        &mut self,
        tokens: &mut Vec<usize>,
        start: usize,
        end: usize,
        new_lines: Option<u32>,
    ) {
        if let Some(x) = new_lines {
            tokens.push(NewLine as usize);
            tokens.push(x as usize);
        }
        self.skip_bytes(end - start);
        tokens.push(start);
        tokens.push(end);
    }

    fn emit_tokens(&mut self, tokens: &mut Vec<usize>, start: usize, end: usize, new_lines: u32) {
        tokens.push(NewLine as usize);
        tokens.push(new_lines as usize);
        tokens.push(start);
        tokens.push(end);
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

    fn get_read_line(&mut self) -> (usize, usize, usize) {
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
    fn read_line(&mut self, space_indent: &mut Option<u32>) -> (usize, usize) {
        let (start, end, consume) = self.get_read_line();
        *space_indent = None;
        self.pos = consume;
        self.line += 1;
        self.col = 0;
        (start, end)
    }

    #[inline]
    fn count_spaces(&mut self) -> u32 {
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

    fn count_whitespace_from(&mut self, offset: usize) -> usize {
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

    fn count_spaces_till(&mut self, num_spaces: u32) -> usize {
        self.slice[self.pos..]
            .iter()
            .enumerate()
            .take_while(|&(count, &x)| x == b' ' && count < num_spaces as usize)
            .count()
    }

    fn is_empty_newline(&mut self) -> bool {
        self.slice[self.pos..self.get_read_line().1]
            .iter()
            .rev()
            .all(|c| *c == b' ')
    }

    fn count_space_then_tab(&mut self) -> (u32, u32) {
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
            .try_fold(0u32, |ws_cnt, chr| match *chr {
                b' ' | b'\t' => Continue(ws_cnt + 1),
                _ => Break(ws_cnt),
            }) {
            Continue(x) | Break(x) => x,
        };
        (spaces, tabs)
    }

    fn consume_anchor_alias(&mut self) -> (usize, usize) {
        let start = self.skip_bytes(1);

        let amount = self.slice[self.pos..]
            .iter()
            .position(|p| is_white_tab_or_break(*p) || is_flow_indicator(*p))
            .unwrap_or(self.slice.len() - self.pos);
        self.skip_bytes(amount);
        (start, start + amount)
    }

    fn read_tag(&mut self, lexer_state: &mut LexMutState) -> (usize, usize, usize) {
        match self.peek_chars() {
            [b'!', b'<', ..] => {
                let start = self.skip_bytes(2);
                let (line_start, line_end, _) = self.get_read_line();
                let haystack = &self.slice[line_start..line_end];
                if let Some(end) = memchr(b'>', haystack) {
                    self.skip_bytes(end + 1);
                    (start, start + end, 0)
                } else {
                    self.skip_space_tab();
                    lexer_state.errors.push(ErrorType::UnfinishedTag);
                    (0, 0, 0)
                }
            }
            [b'!', peek, ..] if is_white_tab_or_break(*peek) => {
                let start = self.pos;
                self.skip_bytes(1);
                (start, start + 1, start + 1)
            }
            [b'!', ..] => {
                let start = self.pos;
                self.skip_bytes(1);
                let (_, line_end, _) = self.get_read_line();
                let haystack = &self.slice[self.pos..line_end];
                let find_pos = match memchr(b'!', haystack) {
                    Some(find) => find + 1,
                    None => 0,
                };
                let mid: usize = self.pos + find_pos;
                let amount = self.slice[mid..line_end]
                    .iter()
                    .position(|c| !is_tag_char_short(*c))
                    .unwrap_or(line_end.saturating_sub(mid));
                let end = self.skip_bytes(amount + find_pos);
                (start, mid, end)
            }
            _ => {
                lexer_state
                    .errors
                    .push(ErrorType::TagMustStartWithExclamation);
                (0, 0, 0)
            }
        }
    }

    fn read_tag_handle(&mut self, space_indent: &mut Option<u32>) -> Result<Vec<u8>, ErrorType> {
        match self.peek_chars() {
            [b'!', x, ..] if *x == b' ' || *x == b'\t' => {
                self.skip_bytes(1);
                self.skip_space_tab();
                Ok(vec![b'!'])
            }
            [b'!', _x, ..] => {
                let start = self.pos;
                self.skip_bytes(1);
                let amount: usize = self.slice[self.pos..]
                    .iter()
                    .position(|c: &u8| !is_tag_char(*c))
                    .unwrap_or(self.slice.len() - self.pos);
                self.skip_bytes(amount);
                if self.peek_byte_is(b'!') {
                    let bac = self.slice[start..start + amount + 2].to_vec();
                    self.skip_bytes(1);
                    Ok(bac)
                } else {
                    self.read_line(space_indent);
                    Err(ErrorType::TagNotTerminated)
                }
            }
            [x, ..] => {
                let err = Err(ErrorType::InvalidTagHandleCharacter { found: *x as char });
                self.read_line(space_indent);
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
            let end = self.skip_bytes(amount);
            Some((start, end))
        } else {
            None
        }
    }

    fn read_directive(
        &mut self,
        directive_state: &mut DirectiveState,
        lexer_state: &mut LexMutState,
    ) -> bool {
        let max = core::cmp::min(self.slice.len(), self.pos + 3);
        let chars = &self.slice[self.pos..max];
        match chars {
            b"1.0" | b"1.1" | b"1.2" | b"1.3" => {
                directive_state.add_directive();
                if *directive_state == DirectiveState::TwoDirectiveError {
                    push_error(
                        TwoDirectivesFound,
                        &mut lexer_state.tokens,
                        lexer_state.errors,
                    );
                }
                lexer_state.tokens.push_back(DirectiveYaml as usize);
                lexer_state.tokens.push_back(self.pos);
                lexer_state.tokens.push_back(self.skip_bytes(3));
                true
            }
            b"..." | b"---" => false,
            _ => {
                self.read_line(lexer_state.space_indent);
                false
            }
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

    fn emit_new_space(&mut self, tokens: &mut Vec<usize>, new_spaces: &mut Option<usize>) {
        if let Some(new_line) = new_spaces.take() {
            tokens.push(NewLine as usize);
            tokens.push(new_line);
        }
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

    fn get_quote_line_offset(&mut self, quote: u8) -> &[u8] {
        let slice = self.slice;
        let start = self.pos;
        let remaining = slice.len().saturating_sub(start);
        let content = &slice[start..];
        let n = memchr::memchr3_iter(b'\r', b'\n', quote, content)
            .next()
            .map_or(remaining, |p| if content[p] == quote { p + 1 } else { p });
        &slice[start..start + n]
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
    reader.read_line(&mut None);
    let (start, end, consume) = reader.get_read_line();
    assert_eq!(start, 1);
    assert_eq!(end, 6);
    assert_eq!(b"  rst", input.slice(start, end));
    assert_eq!(consume, 7);
    reader.read_line(&mut None);
    let (start, end, consume) = reader.get_read_line();
    assert_eq!(start, 7);
    assert_eq!(end, 7);
    assert_eq!(b"", input.slice(start, end));
    assert_eq!(consume, 7);
}

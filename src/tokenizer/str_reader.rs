use std::collections::VecDeque;
use std::ops::ControlFlow::{Break, Continue};
use std::ops::{RangeFrom, RangeInclusive};

use memchr::memchr3_iter;
use reader::{is_flow_indicator, ns_plain_safe};
use ErrorType::ExpectedIndent;

use crate::tokenizer::reader::{
    is_indicator, is_white_tab, is_white_tab_or_break, ChompIndicator, LookAroundBytes,
};
use crate::tokenizer::spanner::ParserState;
use crate::tokenizer::spanner::ParserState::{BlockKeyExp, BlockMap, BlockSeq, BlockValExp};
use crate::tokenizer::ErrorType::UnexpectedComment;
use crate::tokenizer::SpanToken::{
    Directive, ErrorToken, MappingStart, MarkEnd, MarkStart, NewLine, Separator, Space,
};
use crate::tokenizer::{reader, DirectiveType, ErrorType, Reader, SpanToken};

pub struct StrReader<'a> {
    pub slice: &'a [u8],
    pub(crate) pos: usize,
    pub(crate) col: usize,
}

enum Flow {
    Continue,
    Break,
    Error(usize),
}

impl<'a> StrReader<'a> {
    pub fn new(slice: &'a str) -> StrReader<'a> {
        Self {
            slice: slice.as_bytes(),
            pos: 0,
            col: 0,
        }
    }

    #[inline]
    fn eof_or_pos(&self, pos: usize) -> usize {
        pos.min(self.slice.len() - 1)
    }

    #[inline]
    fn get_lookahead_iterator(&self, range: RangeInclusive<usize>) -> LookAroundBytes {
        LookAroundBytes::new(self.slice, range)
    }

    #[inline]
    fn peek_byte_unwrap(&self, offset: usize) -> u8 {
        match self.slice.get(self.pos + offset) {
            Some(x) => *x,
            _ => b'\0',
        }
    }

    #[inline]
    fn count_space_tab_range_from(&self, range: RangeFrom<usize>, allow_tab: bool) -> usize {
        match self.slice[range]
            .iter()
            .try_fold(0usize, |acc, x| reader::is_tab_space(acc, *x, allow_tab))
        {
            Continue(x) | Break(x) => x,
        }
    }

    fn find_next_whitespace(&self) -> Option<usize> {
        self.slice[self.pos..]
            .iter()
            .position(|p| is_white_tab_or_break(*p))
    }

    fn skip_n_spaces(&mut self, num_spaces: usize, prev_indent: usize) -> Flow {
        let count = self.slice[self.pos..]
            .iter()
            .enumerate()
            .take_while(|&(count, &x)| x == b' ' && count < num_spaces)
            .count();

        if count == prev_indent {
            Flow::Break
        } else if count != num_spaces {
            Flow::Error(count)
        } else {
            self.pos += count;
            Flow::Continue
        }
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

    fn read_plain_one_line(
        &mut self,
        allow_minus: bool,
        had_comment: &mut bool,
        in_flow_collection: bool,
        tokens: &mut Vec<SpanToken>,
    ) -> Option<(usize, usize)> {
        let start = self.pos;

        if !(allow_minus && self.peek_byte_is(b'-')) && (self.eof() || self.not_safe_char()) {
            return None;
        }

        let end = self.consume_bytes(1);
        let (_, line_end, _) = self.get_line_offset();
        let line_end = StrReader::eof_or_pos(self, line_end);
        let mut end_of_str = end;

        for (prev, curr, next, pos) in self.get_lookahead_iterator(end..=line_end) {
            // ns-plain-char  prevent ` #`
            if curr == b'#' && is_white_tab_or_break(prev) {
                // if we encounter two or more comment print error and try to recover
                if *had_comment {
                    tokens.push(ErrorToken(UnexpectedComment))
                } else {
                    *had_comment = true;
                    self.pos = line_end;
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

        self.pos = end_of_str;
        Some((start, end_of_str))
    }

    #[inline]
    fn not_safe_char(&self) -> bool {
        match self.slice[self.pos..] {
            [x, ..] if is_white_tab_or_break(x) || is_indicator(x) => true,
            [b'-', x, ..] if is_white_tab(x) => true,
            [b'?', x, ..] if is_white_tab_or_break(x) => true,
            [b':', x, ..] if is_white_tab_or_break(x) => true,
            _ => false,
        }
    }
}

impl<'r> Reader for StrReader<'r> {
    #[inline]
    fn eof(&self) -> bool {
        self.pos >= self.slice.len()
    }

    #[inline]
    fn col(&self) -> usize {
        self.col
    }

    #[inline]
    fn peek_byte_at(&self, offset: usize) -> Option<u8> {
        self.slice.get(self.pos + offset).copied()
    }

    #[inline]
    fn peek_byte(&self) -> Option<u8> {
        self.slice.get(self.pos).copied()
    }

    #[inline]
    fn count_space_tab(&self, allow_tab: bool) -> usize {
        self.count_space_tab_range_from(self.pos.., allow_tab)
    }

    #[inline(always)]
    fn consume_bytes(&mut self, amount: usize) -> usize {
        self.pos += amount;
        self.col += amount;
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
        self.col = 0;
        (start, end)
    }

    fn read_block_seq(&mut self, indent: usize) -> Option<ParserState> {
        if self.peek_byte_at_check(1, is_white_tab_or_break) {
            let new_indent: usize = self.col;
            if self.peek_byte_at_check(1, reader::is_newline) {
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
            let pos = memchr::memchr(b'\'', &self.slice[line_start..line_end]);
            match pos {
                Some(len) => {
                    // Converts double '' to ' hence why we consume one extra char
                    let offset = len + 1;
                    if self.slice.get(self.pos + offset).copied() == Some(b'\'') {
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
        init_indent: usize,
        curr_state: &ParserState,
    ) -> (Vec<SpanToken>, Option<ParserState>) {
        let mut allow_minus = false;
        let mut first_line_block = !curr_state.in_flow_collection();

        let mut num_newlines = 0;
        let mut tokens = vec![];
        let mut new_state = match curr_state {
            BlockKeyExp(ind) => Some(BlockValExp(*ind)),
            BlockValExp(ind) => Some(BlockMap(*ind)),
            _ => None,
        };
        let mut curr_indent = curr_state.get_block_indent(self.col);
        let mut had_comment = false;

        while !self.eof() {
            // In explicit key mapping change in indentation is always an error
            if curr_state.wrong_exp_indent(curr_indent) && curr_indent != init_indent {
                tokens.push(ErrorToken(ErrorType::MappingExpectedIndent {
                    actual: curr_indent,
                    expected: init_indent,
                }));
                break;
            } else if curr_indent < init_indent {
                // if plain scalar is less indented than previous
                // It can be
                // a) Part of BlockMap
                // b) An error outside of block map
                if matches!(curr_state, BlockMap(_) | BlockKeyExp(_) | BlockValExp(_)) {
                    tokens.push(Separator);
                } else {
                    self.read_line();
                    tokens.push(ErrorToken(ExpectedIndent {
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

            if chr == b':' && first_line_block {
                if curr_state.is_new_block_col(curr_indent) {
                    new_state = Some(BlockMap(curr_indent));
                    tokens.push(MappingStart);
                }
                tokens.push(MarkStart(start));
                tokens.push(MarkEnd(end));
                break;
            } else if chr == b':' && matches!(curr_state, BlockValExp(ind) if *ind == curr_indent) {
                tokens.push(Separator);
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

            if reader::is_newline(chr) {
                let folded_newline = self.skip_separation_spaces(false);
                if self.col >= curr_state.indent(0) {
                    num_newlines = folded_newline as u32;
                }
                curr_indent = self.col;
            }

            if curr_state.in_flow_collection() && is_flow_indicator(chr) {
                break;
            }

            match (self.peek_byte_unwrap(0), curr_state) {
                (b'-', BlockSeq(ind)) if self.col == *ind => {
                    self.consume_bytes(1);
                    tokens.push(Separator);
                    break;
                }
                (b'-', BlockSeq(ind)) if self.col < *ind => {
                    self.read_line();
                    let err_type = ExpectedIndent {
                        expected: *ind,
                        actual: curr_indent,
                    };
                    tokens.push(ErrorToken(err_type));
                    break;
                }
                (b'-', BlockSeq(ind)) if self.col > *ind => {
                    allow_minus = true;
                }
                (b':', BlockValExp(ind)) if self.col == *ind => {
                    break;
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
            let pos = memchr::memchr(b'"', &self.slice[line_start..line_end]);
            match pos {
                Some(len) if len > 1 => {
                    // Check for `\` escape
                    let offset = len - 1;
                    if self.slice.get(self.pos + offset).copied() != Some(b'\\') {
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
                if self.col + curr_indent == *ind {
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

            let newline_is_empty = self.peek_byte_at_check(newline_indent, reader::is_newline)
                || (is_trailing_comment && self.peek_byte_unwrap(newline_indent) == b'#');

            if indentation == 0 && newline_indent > 0 && !newline_is_empty {
                indentation = newline_indent;
            }

            if newline_is_empty {
                new_line_token += 1;
                self.read_line();
                continue;
            } else {
                match self.skip_n_spaces(indentation, curr_state.indent(0)) {
                    Flow::Break => break,
                    Flow::Error(actual) => {
                        tokens.push_back(ErrorToken(ExpectedIndent {
                            actual,
                            expected: indentation,
                        }));
                        break;
                    }
                    _ => {}
                }
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

    fn try_read_tag(&mut self, tokens: &mut VecDeque<SpanToken>) {
        if self.try_read_slice_exact("%YAML") {
            self.skip_space_tab(true);
            if let Some(x) = self.find_next_whitespace() {
                tokens.push_back(Directive(DirectiveType::Yaml));
                tokens.push_back(MarkStart(self.pos));
                tokens.push_back(MarkEnd(self.pos + x));

                self.consume_bytes(x);
                self.read_line();
            }
        } else {
            let tag = if self.try_read_slice_exact("%TAG") {
                Directive(DirectiveType::Tag)
            } else {
                Directive(DirectiveType::Reserved)
            };
            self.skip_space_tab(true);
            let x = self.read_non_comment_line();
            if x.0 != x.1 {
                tokens.push_back(tag);
                tokens.push_back(MarkStart(x.0));
                tokens.push_back(MarkEnd(x.1));
            }
        }
    }
}

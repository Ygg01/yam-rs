use crate::saphyr_tokenizer::char_utils::{
    is_alpha, is_blank, is_blank_or_break, is_blank_or_breakz, is_break, is_breakz, is_flow,
};
use crate::saphyr_tokenizer::scanner::SkipTabs;
use alloc::vec::Vec;

pub trait Source {
    #[must_use]
    fn peek_arbitrary(&self, n: usize) -> u8;

    #[must_use]
    fn peek(&self) -> u8;

    #[must_use]
    fn peek_n1(&self) -> u8 {
        self.peek_arbitrary(1)
    }

    #[must_use]
    fn peek_n2(&self) -> u8 {
        self.peek_arbitrary(2)
    }

    #[must_use]
    fn peek_n3(&self) -> u8 {
        self.peek_arbitrary(3)
    }

    #[must_use]
    fn peek_char(&self) -> char;

    fn skip(&mut self, n: usize);

    #[must_use]
    fn buf_max_len(&self) -> usize {
        128
    }

    fn fetch_while_is_alpha(&mut self, out: &mut Vec<u8>) -> usize {
        let mut n_chars = 0;
        while is_alpha(self.peek()) {
            n_chars += 1;
            out.push(self.peek());
            self.skip(1);
        }
        n_chars
    }

    fn skip_while_blank(&mut self) -> usize {
        let mut n_chars = 0;
        while is_blank(self.peek()) {
            n_chars += 1;
            self.skip(1);
        }
        n_chars
    }

    fn buf_is_empty(&self) -> bool;

    fn skip_ws_to_eol(&mut self, skip_tabs: bool) -> (u32, Result<SkipTabs, &'static str>);
    fn next_byte_is(&self, chr: u8) -> bool {
        chr == self.peek()
    }

    fn next_next_byte_is(&self, chr: u8) -> bool {
        self.peek_n1() == chr
    }

    fn peek_two(&self) -> [u8; 2] {
        [self.peek(), self.peek_n1()]
    }

    fn next_is_three(&self, chr: u8) -> bool {
        self.peek() == chr && self.peek_n1() == chr && self.peek_n2() == chr
    }

    #[must_use]
    fn next_is_flow(&self) -> bool {
        is_flow(self.peek())
    }

    #[must_use]
    fn next_is_break(&self) -> bool {
        is_break(self.peek())
    }

    #[must_use]
    fn next_is_blank(&self) -> bool {
        is_blank(self.peek())
    }

    #[must_use]
    fn next_is_breakz(&self) -> bool {
        is_break(self.peek()) || self.peek() == b'\0'
    }

    fn skip_while_non_breakz(&mut self) -> usize {
        let mut count = 0;
        while !is_break(self.peek()) {
            count += 1;
            self.skip(1);
        }
        count
    }

    fn next_is_blank_or_break(&self) -> bool {
        is_blank_or_break(self.peek())
    }

    fn next_is_blank_or_breakz(&self) -> bool {
        is_blank_or_break(self.peek()) || self.peek() == b'\0'
    }

    fn next_can_be_plain_scalar(&self, in_flow: bool) -> bool {
        let nc = self.peek_n1();
        match self.peek() {
            // indicators can end a plain scalar, see 7.3.3. Plain Style
            b':' if is_blank_or_breakz(nc) || (in_flow && is_flow(nc)) => false,
            c if in_flow && is_flow(c) => false,
            _ => true,
        }
    }

    fn next_is_document_indicator(&self) -> bool {
        (self.next_is_three(b'-') || self.next_is_three(b'.')) && is_blank_or_breakz(self.peek_n3())
    }

    fn next_is_z(&self) -> bool;

    fn next_is_alpha(&self) -> bool {
        is_alpha(self.peek())
    }
    fn push_non_breakz_chr(&mut self, vec: &mut Vec<u8>);
}

#[inline]
pub(crate) fn shared_skip_ws_to_eol<T: Source>(
    x: &mut T,
    skip_tabs: bool,
    mut any_tabs: bool,
    mut has_yaml_ws: bool,
) -> (u32, Result<SkipTabs, &'static str>) {
    let mut chars_consumed = 0;
    loop {
        match x.peek() {
            b' ' => {
                has_yaml_ws = true;
                x.skip(1);
            }
            b'\t' if skip_tabs => {
                any_tabs = true;
                x.skip(1);
            }
            // YAML comments must be preceded by whitespace.
            b'#' if !any_tabs && !has_yaml_ws => {
                return (
                    chars_consumed,
                    Err("comments must be separated from other tokens by whitespace"),
                );
            }
            b'#' => {
                x.skip(1); // Skip over '#'
                while !is_breakz(x.peek()) {
                    x.skip(1);
                    chars_consumed += 1;
                }
            }
            _ => break,
        }
        chars_consumed += 1;
    }

    (
        chars_consumed,
        Ok(SkipTabs::Result {
            any_tabs,
            has_yaml_ws,
        }),
    )
}

pub struct StrSource<'input> {
    input: &'input [u8],
    pos: usize,
}

impl StrSource<'_> {
    pub fn new(input: &str) -> StrSource<'_> {
        StrSource {
            input: input.as_bytes(),
            pos: 0,
        }
    }
}

impl<'input> Source for StrSource<'input> {
    fn peek_arbitrary(&self, n: usize) -> u8 {
        debug_assert!(
            n <= self.buf_max_len(),
            "Can only support limited lookahead"
        );
        match self.input.get(self.pos + n) {
            Some(x) => *x,
            None => b'\0',
        }
    }

    fn peek(&self) -> u8 {
        match self.input.get(self.pos) {
            Some(x) => *x,
            None => b'\0',
        }
    }

    fn peek_char(&self) -> char {
        // TODO make it ACTUALLY safe
        let mut bytes = unsafe {
            str::from_utf8_unchecked(self.input.get_unchecked(self.pos..self.pos + 4)).chars()
        };
        bytes.next().unwrap()
    }

    fn skip(&mut self, n: usize) {
        self.pos += n;
    }

    fn buf_is_empty(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn skip_ws_to_eol(&mut self, skip_tabs: bool) -> (u32, Result<SkipTabs, &'static str>) {
        shared_skip_ws_to_eol(self, skip_tabs, false, false)
    }

    fn next_is_z(&self) -> bool {
        self.buf_is_empty()
    }

    fn push_non_breakz_chr(&mut self, vec: &mut Vec<u8>) {
        let len = self.input[self.pos..]
            .iter()
            .position(|&c| is_break(c))
            .unwrap_or(0);
        let slice = &self.input[self.pos..self.pos + len];
        self.skip(len);
        vec.extend_from_slice(slice);
    }
}

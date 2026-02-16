use crate::saphyr_tokenizer::char_utils::{
    is_alpha, is_blank, is_blank_or_break, is_blank_or_breakz, is_break, is_breakz, is_flow,
};
use crate::saphyr_tokenizer::scanner::SkipTabs;
use alloc::vec::Vec;

///
/// A trait that defines a source of input data, with methods for peeking, skipping,
/// and inspecting bytes and characters.
///
/// # Safety
/// This is an unsafe trait because methods involve pushing raw Vec<u8> bytes before
/// they are converted to UTF8 (possibly violating memory safety), and the methods for unsafely
/// accessing the input source.
///
/// # Associated Methods
/// - The trait provides methods to inspect upcoming bytes in the input source non-destructively.
/// - It supports skipping bytes and extracting content based on specific conditions (e.g., alphanumeric content).
///
/// # Methods
/// ## Peeking
/// - `peekz_arbitrary(n: usize) -> u8`: Returns the byte at an arbitrary position `n` from the current position.
/// - `peek_unsafe(n: usize) -> u8`: Unsafely retrieves the byte at an arbitrary position `n` from the current position. Must be handled carefully to avoid memory safety issues.
/// - `peek_check(n: usize) -> Option<u8>`: Retrieves the byte at position `n` if it exists, otherwise returns `None`.
/// - `peek() -> Option<u8>`: Retrieves the next byte without advancing the position.
/// - `peekz() -> u8`: Returns the next byte, defaulting to `0` if unavailable.
/// - `peekz_n1() -> u8`, `peekz_n2() -> u8`, `peekz_n3() -> u8`: Retrieves the first, second, or third upcoming byte, defaulting to `0` if unavailable.
/// - `peek_char() -> char`: Returns the next character from the source.
///
/// ## Skipping and Buffer Control
/// - `skip(n: usize)`: Skips `n` bytes in the source.
/// - `buf_max_len() -> usize`: Returns the maximum recommended buffer length. Default is `128`.
/// - `buf_is_empty() -> bool`: Checks if the buffer is empty.
///
/// ## Parsing Helpers
/// - `fetch_while_is_alpha(out: &mut Vec<u8>) -> usize`: Fetches and appends all consecutive alphanumeric characters into `out`, returning the count.
/// - `skip_while_blank() -> usize`: Skips all consecutive blank characters.
/// - `skip_ws_to_eol(skip_tabs: bool) -> (u32, Result<SkipTabs, &'static str>)`: Skips whitespace characters until the end of the line, optionally skipping tabs.
///
/// ## Flow and Blank/Binary Checks
/// - `next_is_flow() -> bool`: Checks if the next byte represents a `flow character.
/// - `next_is_break() -> bool`: Checks if the next byte represents a `break` character.
/// - `next_is_blank() -> bool`: Checks if the next byte represents a `blank` character.
/// - `next_is_breakz() -> bool`: Checks if the next byte is a `break` or null-terminator (`'\0'`).
/// - `next_is_blank_or_break() -> bool`: Checks if the next byte is either blank or a break.
/// - `next_is_blank_or_breakz() -> bool`: Checks if the next byte is blank, a break, or a null-terminator (`'\0'`).
pub unsafe trait Source {
    #[must_use]
    fn peekz_arbitrary(&self, n: usize) -> u8 {
        self.peek_check(n).unwrap_or(0)
    }

    #[must_use]
    unsafe fn peek_unsafe(&self, n: usize) -> u8;

    #[must_use]
    fn peek_check(&self, n: usize) -> Option<u8>;

    #[must_use]
    fn peek(&self) -> Option<u8>;

    #[must_use]
    fn peekz(&self) -> u8 {
        self.peekz_arbitrary(0)
    }

    #[must_use]
    fn peekz_n1(&self) -> u8 {
        self.peekz_arbitrary(1)
    }

    #[must_use]
    fn peekz_n2(&self) -> u8 {
        self.peekz_arbitrary(2)
    }

    #[must_use]
    fn peekz_n3(&self) -> u8 {
        self.peekz_arbitrary(3)
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
        while is_alpha(self.peekz()) {
            n_chars += 1;
            out.push(self.peekz());
            self.skip(1);
        }
        n_chars
    }

    fn skip_while_blank(&mut self) -> usize {
        let mut n_chars = 0;
        while is_blank(self.peekz()) {
            n_chars += 1;
            self.skip(1);
        }
        n_chars
    }

    fn buf_is_empty(&self) -> bool;

    fn skip_ws_to_eol(&mut self, skip_tabs: bool) -> (u32, Result<SkipTabs, &'static str>);
    fn next_byte_is(&self, chr: u8) -> bool {
        chr == self.peekz()
    }

    fn next_next_byte_is(&self, chr: u8) -> bool {
        self.peekz_n1() == chr
    }

    fn peek_two(&self) -> [u8; 2] {
        [self.peekz(), self.peekz_n1()]
    }

    fn next_is_three(&self, chr: u8) -> bool {
        self.peekz() == chr && self.peekz_n1() == chr && self.peekz_n2() == chr
    }

    #[must_use]
    fn next_is_flow(&self) -> bool {
        is_flow(self.peekz())
    }

    #[must_use]
    fn next_is_break(&self) -> bool {
        is_break(self.peekz())
    }

    #[must_use]
    fn next_is_blank(&self) -> bool {
        is_blank(self.peekz())
    }

    #[must_use]
    fn next_is_breakz(&self) -> bool {
        is_break(self.peekz()) || self.peekz() == b'\0'
    }

    fn skip_while_non_breakz(&mut self) -> usize {
        let mut count = 0;
        while !is_break(self.peekz()) {
            count += 1;
            self.skip(1);
        }
        count
    }

    fn next_is_blank_or_break(&self) -> bool {
        is_blank_or_break(self.peekz())
    }

    fn next_is_blank_or_breakz(&self) -> bool {
        match self.peek_check(0) {
            None => true,
            Some(x) => is_blank_or_break(x),
        }
    }

    fn next_can_be_plain_scalar(&self, in_flow: bool) -> bool {
        let nc = self.peekz_n1();
        match self.peekz() {
            // indicators can end a plain scalar, see 7.3.3. Plain Style
            b':' if is_blank_or_breakz(nc) || (in_flow && is_flow(nc)) => false,
            c if in_flow && is_flow(c) => false,
            _ => true,
        }
    }

    fn next_is_document_indicator(&self) -> bool {
        (self.next_is_three(b'-') || self.next_is_three(b'.'))
            && is_blank_or_breakz(self.peekz_n3())
    }

    fn next_is_z(&self) -> bool;

    fn next_is_alpha(&self) -> bool {
        is_alpha(self.peekz())
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
        match x.peekz() {
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
                while !is_breakz(x.peekz()) {
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

unsafe impl<'input> Source for StrSource<'input> {
    unsafe fn peek_unsafe(&self, n: usize) -> u8 {
        unsafe { *self.input.get_unchecked(self.pos + n) }
    }

    fn peek_check(&self, n: usize) -> Option<u8> {
        self.input.get(self.pos + n).copied()
    }

    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
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

#[cfg(test)]
mod test {
    use crate::Source;
    use crate::saphyr_tokenizer::buffered_source::BufferedBytesSource;
    use crate::saphyr_tokenizer::scanner::SkipTabs;

    const TEST_STR: &str = "                                      \
                                    \n                     \
                hello ";

    #[test]
    fn test_str_source() {
        // let mut x = StrSource::new(TEST_STR);
        // let (consume , skip) = x.skip_ws_to_eol(true);
        // assert_eq!(consume, 38);
        // assert_eq!(skip, Ok(SkipTabs::Result { has_yaml_ws: true, any_tabs: false}));

        let mut x = BufferedBytesSource::from_str(TEST_STR);
        let (consume, skip) = x.skip_ws_to_eol(true);
        assert_eq!(consume, 38);
        assert_eq!(
            skip,
            Ok(SkipTabs::Result {
                has_yaml_ws: true,
                any_tabs: false
            })
        );
    }
}

use crate::parsing::char_utils::{
    is_alpha, is_blank, is_blank_or_breakz, is_break, is_breakz, is_flow,
};
use crate::parsing::scanner::SkipTabs;
use alloc::vec::Vec;

///
/// A trait that defines a source of input data, with methods for peeking, skipping,
/// and inspecting bytes and characters.
///
/// # Safety
/// This is an unsafe trait because methods involve pushing raw `Vec<u8>` bytes before
/// they are converted to UTF8 (possibly violating memory safety), and the methods for unsafely
/// accessing the input source.
///
/// # Associated Methods
/// - The trait provides methods to inspect upcoming bytes in the input source non-destructively.
/// - It supports skipping bytes and extracting content based on specific conditions (e.g., alphanumeric content).
///
/// # Methods
/// ## Peeking
/// - `peekz(n: usize) -> u8`: Returns the byte at an arbitrary position `n` from the current position.
/// - `peek_unsafe(n: usize) -> u8`: Unsafely retrieves the byte at an arbitrary position `n` from the current position. Must be handled carefully to avoid memory safety issues.
/// - `peek_checked(n: usize) -> Option<u8>`: Retrieves the byte at position `n` if it exists, otherwise returns `None`.
/// - `peek_char() -> char`: Returns the next character from the source.
///
/// ## Buffer control
/// - `buf_max_len() -> usize`: Returns the maximum recommended buffer length. The `Default` is `128`.
/// - `buf_is_empty() -> bool`: Checks if the buffer is empty.
///
/// ## Parsing Helpers
/// - `fetch_while_is_alpha(out: &mut Vec<u8>) -> usize`: Fetches and appends all consecutive alphanumeric characters into `out`, returning the count.
/// - `push_non_breakz_chr(out: &mut Vec<u8>) -> usize`: Fetches and appends all consecutive alphanumeric characters into `out`, returning the count.
///
/// ## Skipping
/// - `skip(n: usize)`: Skips `n` bytes in the source.
/// - `skip_while_blank() -> usize`: Skips all consecutive blank characters.
/// - `skip_ws_to_eol(skip_tabs: bool, prev_ws: bool) -> (u32, Result<SkipTabs, &'static str>)`: Skips whitespace characters until the end of the line, optionally skipping tabs.
/// - `skip_while_non_breakz() -> usize`: skips all non-break characters
/// - `skip_and_accumulate_to_eol(buf: &mut Vec<u8>)`
///
/// ## Flow and Blank/Binary Checks
/// - `next_next_byte_is(chr: u8) -> bool`: Checks if the 2nd byte is exactly the same as chr
/// - `next_three_is(chr: u8) -> bool`: Checks if the next three consecutive characters are chr.
/// - `next_can_be_plain_scalar(in_flow: bool) -> bool`: Checks if the next characters can be plain
///   scalar. `in_flow` it determines if its plain scalar is in flow or block mode.
/// - `next_is_document_indicator() -> bool`: Checks if the next characters form a document indicator.
pub unsafe trait Source {
    #[must_use]
    fn peekz(&self, n: usize) -> u8 {
        self.peek_checked(n).unwrap_or(0)
    }

    ///
    /// Peeks a byte from the specified position `n` within the underlying data structure
    /// without performing bounds checking or other safety checks. This function provides
    /// direct, unsafe access to the data by bypassing normal safeguards.
    ///
    /// # Safety
    ///
    /// This function is `unsafe` because:
    /// - The caller must ensure that the index `n` is within bounds of the underlying data.
    /// - Accessing an invalid index may result in undefined behavior, such as memory corruption
    ///   or a program crash.
    ///
    /// # Parameters
    /// - `n`: The zero-based index of the byte to read from the data structure.
    ///
    /// # Returns
    /// - `u8`: The byte located at the specified index `n`.
    ///
    /// # Attributes
    /// - `#[must_use]`: The return value of this function must not be ignored.
    ///
    #[must_use]
    unsafe fn peek_unsafe(&self, n: usize) -> u8;

    #[must_use]
    fn peek_checked(&self, n: usize) -> Option<u8>;

    #[must_use]
    fn peek_char(&self) -> char;

    #[must_use]
    fn buf_max_len(&self) -> usize {
        128
    }

    fn buf_is_empty(&self) -> bool;

    fn fetch_while_is_alpha(&mut self, out: &mut Vec<u8>) -> usize {
        let mut n_chars = 0;
        while is_alpha(self.peekz(0)) {
            n_chars += 1;
            out.push(self.peekz(0));
            self.skip(1);
        }
        n_chars
    }

    fn push_non_breakz_chr(&mut self, vec: &mut Vec<u8>);

    fn skip(&mut self, n: usize);

    fn skip_while_blank(&mut self) -> usize {
        let mut n_chars = 0;
        while is_blank(self.peekz(0)) {
            n_chars += 1;
            self.skip(1);
        }
        n_chars
    }

    fn skip_ws_to_eol(
        &mut self,
        skip_tabs: bool,
        prev_ws: bool,
    ) -> (u32, Result<SkipTabs, &'static str>);

    fn skip_while_non_breakz(&mut self) -> usize {
        let mut count = 0;
        while !is_breakz(self.peekz(0)) {
            count += 1;
            self.skip(1);
        }
        count
    }

    fn skip_and_accumulate_to_eol(&mut self, accumulator: &mut Vec<u8>) {
        while let Some(x) = self.peek_checked(0)
            && !is_break(x)
        {
            accumulator.push(x);
            self.skip(1);
        }
    }

    fn next_next_byte_is(&self, chr: u8) -> bool {
        self.peekz(1) == chr
    }

    fn next_is_three(&self, chr: u8) -> bool {
        self.peekz(0) == chr && self.peekz(1) == chr && self.peekz(2) == chr
    }

    fn next_can_be_plain_scalar(&self, in_flow: bool) -> bool {
        let nc = self.peekz(1);
        match self.peekz(0) {
            // indicators can end a plain scalar, see 7.3.3. Plain Style
            b':' if is_blank_or_breakz(nc) || (in_flow && is_flow(nc)) => false,
            c if in_flow && is_flow(c) => false,
            _ => true,
        }
    }

    fn next_is_document_indicator(&self) -> bool {
        (self.next_is_three(b'-') || self.next_is_three(b'.')) && is_blank_or_breakz(self.peekz(3))
    }
}

#[inline]
pub(crate) fn shared_skip_ws_to_eol<T: Source>(
    x: &mut T,
    skip_tabs: bool,
    mut bytes_consumed: u32,
    mut any_tabs: bool,
    mut has_yaml_ws: bool,
) -> (u32, Result<SkipTabs, &'static str>) {
    loop {
        match x.peekz(0) {
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
                    bytes_consumed,
                    Err("comments must be separated from other tokens by whitespace"),
                );
            }
            #[cfg(feature = "comment")]
            b'#' => break,
            #[cfg(not(feature = "comment"))]
            b'#' => {
                // Skip `#`
                x.skip(1);
                while !is_breakz(x.peekz(0)) {
                    x.skip(1);
                    bytes_consumed += 1;
                }
            }
            _ => break,
        }
        bytes_consumed += 1;
    }

    (
        bytes_consumed,
        Ok(SkipTabs::Result {
            any_tabs,
            has_yaml_ws,
        }),
    )
}

/// A structure that wraps a slice os strings
///
/// `StrSource` provides a way to work with string-like byte input slices (`&[u8]`)
/// and keep track of the current position within the input.
///
/// # Type Parameters
/// - `'input`: Lifetime of the input data slice.
/// # Example
/// ```
/// use yam_core::StrSource;
/// let source = StrSource::new("Hello, world!");
/// assert_eq!(source.pos, 0);
/// assert_eq!(source.input, b"Hello, world!");
/// ```
///
/// This struct can be used for parsing, tokenizing, or other scenarios
/// where keeping track of a position in a byte slice is required.
/// ```
pub struct StrSource<'input> {
    /// A reference to the byte slice (`&[u8]`) that serves as the source data.
    input: &'input [u8],
    /// A zero based index (`usize`) representing the current position within the input.
    pos: usize,
}

impl StrSource<'_> {
    ///
    /// Creates a new instance of `StrSource` from a given string slice.
    ///
    /// # Parameters
    /// - `input`: A string slice that serves as the source for the `StrSource` instance.
    ///
    /// # Returns
    /// Returns a new `StrSource` instance initialized with the provided string slice.
    /// The string slice is internally converted to bytes and a starting position of 0 is set.
    ///
    /// # Attributes
    /// - `#[must_use]`: Indicates that the returned `StrSource` instance must be used,
    ///   otherwise a warning will be emitted by the compiler.
    ///
    /// # Example
    /// ```rust
    /// use yam_core::parsing::StrSource;
    /// use crate::yam_core::Source;
    /// let source = StrSource::new("example");
    /// assert_eq!(source.peek_char(),'e');
    /// ```
    #[must_use]
    pub fn new(input: &str) -> StrSource<'_> {
        StrSource {
            input: input.as_bytes(),
            pos: 0,
        }
    }
}

unsafe impl Source for StrSource<'_> {
    unsafe fn peek_unsafe(&self, n: usize) -> u8 {
        unsafe { *self.input.get_unchecked(self.pos + n) }
    }

    fn peek_checked(&self, n: usize) -> Option<u8> {
        self.input.get(self.pos + n).copied()
    }

    fn peek_char(&self) -> char {
        // TODO make it ACTUALLY safe
        let mut bytes = unsafe {
            str::from_utf8_unchecked(self.input.get_unchecked(self.pos..self.pos + 4)).chars()
        };
        bytes.next().unwrap_or('\u{FFFD}')
    }

    fn buf_is_empty(&self) -> bool {
        self.pos >= self.input.len()
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

    fn skip(&mut self, n: usize) {
        self.pos += n;
    }

    fn skip_ws_to_eol(
        &mut self,
        skip_tabs: bool,
        has_yaml_ws: bool,
    ) -> (u32, Result<SkipTabs, &'static str>) {
        shared_skip_ws_to_eol(self, skip_tabs, 0, false, has_yaml_ws)
    }
}

#[cfg(test)]
mod test {
    use crate::StrSource;
    use crate::parsing::Source;
    use crate::parsing::buffered_source::BufferedBytesSource;
    use crate::parsing::scanner::SkipTabs;

    const TEST_STR: &str = "                                      \
                                    \n                     \
                hello ";

    #[test]
    fn test_str_source() {
        let mut x = StrSource::new(TEST_STR);
        let (consume, skip) = x.skip_ws_to_eol(true, false);
        assert_eq!(consume, 38);
        assert_eq!(
            skip,
            Ok(SkipTabs::Result {
                has_yaml_ws: true,
                any_tabs: false
            })
        );

        let mut x = BufferedBytesSource::from_str(TEST_STR);
        let (consume, skip) = x.skip_ws_to_eol(true, false);
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

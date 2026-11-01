use crate::parsing::buffered_source::BUF_MAX_LEN;
use crate::parsing::char_utils::{
    LO_BYTES, eq_byte_mask, is_alpha, is_blank, is_blank_or_breakz, is_break, is_breakz, is_flow,
    len_until,
};
use crate::parsing::scanner::SkipTabs;
use alloc::vec::Vec;
use core::cmp::min;

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
    /// Returns the `n`-th byte from the internal buffer without consuming it.
    ///
    /// This function attempts to retrieve the byte at the specified index `n` in the buffer.
    /// If the index `n` is out of bounds or greater than `MAX_BUFFER_SIZE`, it returns `0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use yam_core::parsing::StrSource;
    /// use yam_core::parsing::Source;
    ///
    /// let buffer = StrSource::new("example");
    /// assert_eq!(buffer.peekz(2), b'a');
    /// ```
    ///
    /// # Notes
    ///
    /// This function is marked with `#[must_use]`, meaning its return value should not
    /// be ignored. If unused, the compiler will emit a warning.
    ///
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

    /// Tries to peek a byte at position `n`.
    ///
    /// `n` indicates how far the peeked character is ahead of the current position.
    /// So `n == 0` is the byte at the current position.
    /// For `n > 0` peeks `n` bytes ahead of the current position.
    /// If `n >= self.buf_max_len()` method returns `None`.
    ///
    /// Returns an `Option<u8>`:
    ///  - `Some(u8)` if the byte is located at the specified index `n`
    ///  - `None` if the index is out of bounds or out of buffer.
    ///
    /// # Notes
    ///
    /// - Before invoking this function with `n > 0`, one must ensure
    ///   the buffer if exist is properly initialized via [`ensure_lookahead`].
    /// - Most peeks in [`StrSource`] will return `Some(u8)`, but in [`BufferedSource`]
    ///   they can fail when encountering buffer bounds.
    #[must_use]
    fn peek_checked(&self, n: usize) -> Option<u8>;

    /// Tries to peek a UTF [`char`].
    ///
    /// Next character in the source. If no valid character is found, returns `\u{FFFD}`.
    #[must_use]
    fn peek_char(&self) -> char;

    /// Returns the maximum length of the lookahead buffer.
    #[must_use]
    fn buf_max_len(&self) -> usize {
        BUF_MAX_LEN
    }

    /// Ensures that the lookahead buffer has at least `n` bytes available from the source.
    ///
    /// # Note:
    /// In [`StrSource`], this function is a no-op as the buffer is always pre-filled.
    fn ensure_lookahead(&mut self, _n: usize);

    /// Returns whether the lookahead buffer is empty.
    fn buf_is_empty(&self) -> bool;

    /// Consumes bytes that are alphanumeric or `-` or `_` characters.
    ///
    /// Consumed characters are added to `out` and the number of consumed bytes is returned.
    fn fetch_while_is_alpha(&mut self, out: &mut Vec<u8>) -> usize {
        let mut n_chars = 0;
        while is_alpha(self.peekz(0)) {
            n_chars += 1;
            out.push(self.peekz(0));
            self.skip(1);
        }
        n_chars
    }

    /// Appends a non-breaking character (`\r` and `\n` or EOF) to the given vector of bytes until
    /// a breaking character is reached.
    ///
    /// This function adds a specific byte (representing a non-breaking character)
    /// to the provided mutable vector (`vec`). It ensures that only non-breaking characters
    /// are appended during its execution.
    ///
    /// # Notes
    /// - The implementation assumes that the underlying input is UTF8.
    /// - This function does not modify any existing elements in the vector.
    fn push_non_breakz_chr(&mut self, vec: &mut Vec<u8>);

    /// Appends and skips bytes for as long as they can continue a plain scalar holds.
    ///
    /// It takes into consideration the current context (i.e., whether the context is within a flow style)
    /// and outputs the bytes that match the rules into the provided mutable vector (`out`).
    ///
    /// # Notes
    /// - The implementation assumes that the underlying input is UTF8.
    /// - This function does not modify any existing elements in the vector.
    fn push_plain_run(&mut self, in_flow: bool, out: &mut Vec<u8>) -> usize;

    /// Advances the iterator by skipping over a specified number of bytes.
    ///
    /// # Parameters
    /// - `n`: The number of bytes to skip in the iterator.
    ///
    /// # Panics
    /// This function does not panic under normal circumstances.
    fn skip(&mut self, n: usize);

    /// Skip spaces (` `) up to `max` consecutive bytes and returns how many were skipped.
    ///
    /// # Parameters
    /// - `max`: The maximum number of consecutive spaces to skip.
    ///
    /// Returns the number of bytes skipped this way.
    fn skip_spaces(&mut self, max: usize) -> usize;

    /// Skip spaces all (` `) and (`\t`).
    ///
    /// Returns the number of bytes skipped this way.
    fn skip_while_blank(&mut self) -> usize {
        let mut n_chars = 0;
        while is_blank(self.peekz(0)) {
            n_chars += 1;
            self.skip(1);
        }
        n_chars
    }

    /// Advances the internal pointer of the object to skip over whitespace
    /// until the end of the current line, optionally allowing the skipping
    /// of tabs.
    ///
    /// If `skip_tabs` is `true`, tab characters (`\t`) are also skipped.
    /// While `prev_ws` is a boolean representing whether the function is already called in a context
    /// where previous characters were already whitespace.
    ///
    /// # Returns
    /// A tuple containing:
    /// - A `u32` representing the number of whitespace characters skipped.
    /// - A `Result` where:
    ///   - `Ok(SkipTabs)` is returned when the operation completes successfully.
    ///     `SkipTabs` may encode additional state or information about the operation.
    ///   - `Err(&'static str)` is returned when an error occurs, with
    ///     `&'static str` providing a description of the error.
    ///
    /// # Errors
    /// This function will return an `Err` if an unexpected parsing condition
    /// or invalid state occurs that prevents the function from skipping whitespace.
    ///
    /// # Notes
    /// - This function assumes the caller manages and maintains the internal state
    ///   of the object correctly to avoid misalignment or unexpected results.
    /// - The behavior may vary depending on the implementation of `SkipTabs`.
    fn skip_ws_to_eol(
        &mut self,
        skip_tabs: bool,
        prev_ws: bool,
    ) -> (u32, Result<SkipTabs, &'static str>);

    /// Skips all non-break and non-EOF characters.
    ///
    /// Returns the number of bytes skipped.
    fn skip_while_non_breakz(&mut self) -> usize {
        let mut count = 0;
        while !is_breakz(self.peekz(0)) {
            count += 1;
            self.skip(1);
        }
        count
    }

    /// Skip all non-break characters while accumulating them into the provided accumulator.
    ///
    /// # Side Effect
    /// Pushes bytes into the accumulator and updates the current position using [`skip`](Self::skip).
    fn skip_and_accumulate_to_eol(&mut self, accumulator: &mut Vec<u8>) {
        while let Some(x) = self.peek_checked(0)
            && !is_break(x)
        {
            accumulator.push(x);
            self.skip(1);
        }
    }

    /// Checks if the byte after the current position is `chr`.
    fn next_next_byte_is(&self, chr: u8) -> bool {
        self.peekz(1) == chr
    }

    /// Returns if the next three bytes are all a single `chr`. Useful for checking document indicators.
    fn next_is_three(&self, chr: u8) -> bool {
        self.peekz(0) == chr && self.peekz(1) == chr && self.peekz(2) == chr
    }

    /// Checks if the next two bytes can be inside a plain scalar.
    fn next_can_be_plain_scalar(&self, in_flow: bool) -> bool {
        let nc = self.peekz(1);
        match self.peekz(0) {
            // indicators can end a plain scalar, see 7.3.3. Plain Style
            b':' if is_blank_or_breakz(nc) || (in_flow && is_flow(nc)) => false,
            c if in_flow && is_flow(c) => false,
            _ => true,
        }
    }

    /// Returns if the next three bytes are `---` or `...`
    fn next_is_document_indicator(&self) -> bool {
        (self.next_is_three(b'-') || self.next_is_three(b'.')) && is_blank_or_breakz(self.peekz(3))
    }

    // # SIMD Methods

    /// This method returns a chunk of bytes from the source, divisible by 64 chunked for SIMD usage.
    ///
    /// Note: While in-memory doesn't need any updating, the [`BufferedSource`]
    /// requires updates using [`reset_head_to_start`]
    fn get_chunked(&self) -> Option<&[u8]>;

    /// This method returns the remaining bytes in the source buffer after exhausting all [`get_chunked`] bytes.
    ///
    /// # Returns
    /// A slice of bytes that are not chunked for SIMD usage. In SIMD algorithms this slice should be taken by
    /// the slow path.
    fn get_remaining(&self) -> &[u8];

    /// Resets the head of the source to the start of the buffer.
    ///
    /// # Side effects:
    ///   - In [`BufferedSource`], this method resets the head of the buffer to zero and refills the buffer.
    ///   - In [`StrSource`], this method does not have any side effects.
    ///
    /// # Default implementation
    /// Is a no-op.
    fn reset_head_to_start(&mut self) {}
}

#[inline]
pub(crate) fn shared_skip_ws_to_eol<T: Source>(
    source: &mut T,
    skip_tabs: bool,
    mut bytes_consumed: u32,
    mut any_tabs: bool,
    mut has_yaml_ws: bool,
) -> (u32, Result<SkipTabs, &'static str>) {
    loop {
        match source.peekz(0) {
            b' ' => {
                has_yaml_ws = true;
                source.skip(1);
            }
            b'\t' if skip_tabs => {
                any_tabs = true;
                source.skip(1);
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
                source.skip(1);
                while !is_breakz(source.peekz(0)) {
                    source.skip(1);
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
/// use yam_core::prelude::{StrSource, Source};
/// let source = StrSource::new("Hello, world!");
/// assert_eq!(source.peekz(0), b'H');
/// assert_eq!(source.peekz(1), b'e');
/// ```
///
/// This struct can be used for parsing, tokenizing, or other scenarios
/// where keeping track of a position in a byte slice is required.
pub struct StrSource<'input> {
    /// A reference to the byte slice (`&[u8]`) that serves as the source data.
    input: &'input [u8],
    /// A zero based index (`usize`) representing the current position within the input.
    pos: usize,
}

impl<'input> StrSource<'input> {
    /// Returns the not yet consumed part of the input.
    #[inline]
    fn rest(&self) -> &'input [u8] {
        self.input.get(self.pos..).unwrap_or_default()
    }

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
    /// use yam_core::prelude::{StrSource, Source};
    /// let source = StrSource::new("example");
    /// assert_eq!(source.peek_char(),'e');
    /// ```
    #[must_use]
    pub fn new(input: &'input str) -> StrSource<'input> {
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
        if n >= self.buf_max_len() {
            return None;
        }
        self.input.get(self.pos + n).copied()
    }

    fn peek_char(&self) -> char {
        let end_position = min(self.pos + 4, self.input.len());
        let mut bytes = unsafe {
            str::from_utf8_unchecked(self.input.get_unchecked(self.pos..end_position)).chars()
        };
        bytes.next().unwrap_or('\u{FFFD}')
    }

    fn ensure_lookahead(&mut self, _n: usize) {}

    fn buf_is_empty(&self) -> bool {
        self.pos >= self.input.len()
    }

    fn push_non_breakz_chr(&mut self, vec: &mut Vec<u8>) {
        let rest = self.rest();
        let len = len_until(
            rest,
            |x| eq_byte_mask(x, b'\r') | eq_byte_mask(x, b'\n'),
            is_break,
        );
        vec.extend_from_slice(&rest[..len]);
        self.skip(len);
    }

    fn push_plain_run(&mut self, in_flow: bool, out: &mut Vec<u8>) -> usize {
        let rest = self.rest();
        let mut len = 0;
        while let Some(&c) = rest.get(len) {
            match c {
                b' ' | b'\t' | b'\r' | b'\n' | b'\0' => break,
                // indicators can end a plain scalar, see 7.3.3. Plain Style
                b':' => {
                    let nc = rest.get(len + 1).copied().unwrap_or(0);
                    if is_blank_or_breakz(nc) || (in_flow && is_flow(nc)) {
                        break;
                    }
                }
                b',' | b'[' | b']' | b'{' | b'}' if in_flow => break,
                _ => {}
            }
            len += 1;
        }
        out.extend_from_slice(&rest[..len]);
        self.skip(len);
        len
    }

    fn skip(&mut self, n: usize) {
        self.pos += n;
    }

    fn skip_spaces(&mut self, max: usize) -> usize {
        let rest = self.rest();
        let rest = &rest[..max.min(rest.len())];
        // Every byte that isn't a space keeps some bits after the XOR.
        let len = len_until(rest, |x| x ^ (LO_BYTES * u64::from(b' ')), |c| c != b' ');
        self.skip(len);
        len
    }

    fn skip_ws_to_eol(
        &mut self,
        skip_tabs: bool,
        has_yaml_ws: bool,
    ) -> (u32, Result<SkipTabs, &'static str>) {
        shared_skip_ws_to_eol(self, skip_tabs, 0, false, has_yaml_ws)
    }

    fn get_chunked(&self) -> Option<&[u8]> {
        let remaining_part = self.input.len() % 64;
        let chunk_end = self.input.len() - remaining_part;
        Some(&self.input[..chunk_end])
    }

    fn get_remaining(&self) -> &[u8] {
        &self.input[self.pos..]
    }
}

#[cfg(test)]
mod test {
    use crate::parsing::Source;
    use crate::parsing::buffered_source::BufferedBytesSource;
    use crate::parsing::scanner::SkipTabs;
    use crate::prelude::StrSource;

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

        let mut x = BufferedBytesSource::new(TEST_STR);
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

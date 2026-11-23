pub(crate) mod byte_buf;
pub(crate) mod in_mem;

use crate::parsing::char_utils::{
    is_alpha, is_blank, is_blank_or_breakz, is_break, is_breakz, is_flow,
};
use crate::parsing::scanner::SkipTabs;
use crate::parsing::source::byte_buf::BUF_MAX_LEN;
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
    /// Returns the `n`-th byte from the internal buffer without consuming it.
    ///
    /// This function attempts to retrieve the byte at the specified index `n` in the buffer.
    /// If the index `n` is out of bounds or greater than `MAX_BUFFER_SIZE`, it returns `0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use yam_core::parsing::InMemSource;
    /// use yam_core::parsing::Source;
    ///
    /// let buffer = InMemSource::new("example");
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
    ///   the buffer if exist is properly initialized via [`Source::ensure_lookahead`].
    /// - Most peeks in [`in_mem::InMemSource`] will return `Some(u8)`, but in [`byte_buf::BufBytesSource`]
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
    /// In [`in_mem::InMemSource`], this function is a no-op as the buffer is always pre-filled.
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
    /// Note: While in-memory doesn't need any updating, the [`byte_buf::BufBytesSource`]
    /// requires updates using [`Source::reset_head_to_start`]
    fn get_chunked(&self) -> Option<&[u8]>;

    /// This method returns the remaining bytes in the source buffer after exhausting all [`Source::get_chunked`] bytes.
    ///
    /// # Returns
    /// A slice of bytes that are not chunked for SIMD usage. In SIMD algorithms this slice should be taken by
    /// the slow path.
    fn get_remaining(&self) -> &[u8];

    /// Resets the head of the source to the start of the buffer.
    ///
    /// # Side effects
    ///   - In [`byte_buf::BufBytesSource`], this method resets the head of the buffer to zero and refills the buffer.
    ///   - In [`in_mem::InMemSource`], this method does not have any side effects.
    ///
    /// # Default implementation
    /// Is a no-op.
    fn reset_head_to_start(&mut self) {}

    /// Fills the buffer to its maximum capacity.
    ///
    /// # Side effects
    /// This possibly changes the output of [`Source::get_chunked`], depending on implementation.
    fn fill_buf_to_max(&mut self);

    /// Sets the position of the head to `pos`.
    fn update_pos(&mut self, pos: usize);
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

#[cfg(test)]
mod test {
    use crate::parsing::Source;
    use crate::parsing::scanner::SkipTabs;
    use crate::parsing::source::byte_buf::BufBytesSource;
    use crate::parsing::source::in_mem::InMemSource;

    const TEST_STR: &str = "                                      \
                                    \n                     \
                hello ";

    #[test]
    fn test_str_source() {
        let mut x = InMemSource::new(TEST_STR);
        let (consume, skip) = x.skip_ws_to_eol(true, false);
        assert_eq!(consume, 38);
        assert_eq!(
            skip,
            Ok(SkipTabs::Result {
                has_yaml_ws: true,
                any_tabs: false
            })
        );

        let mut x = BufBytesSource::new(TEST_STR);
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

    const INPUT: &str = "XYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZX\
    XYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZXYZX";

    #[test]
    fn test_overflow() {
        let mut stringed = InMemSource::new(INPUT);
        stringed.skip(512);
        let x = stringed.peek_char();
        assert_eq!(x, '\u{FFFD}');

        let mut stringed = BufBytesSource::new(INPUT);
        stringed.skip(512);
        let x = stringed.peek_char();
        assert_eq!(x, '\u{FFFD}');
    }
}

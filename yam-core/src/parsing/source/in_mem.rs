use crate::parsing::Source;
use crate::parsing::char_utils::{
    LO_BYTES, eq_byte_mask, is_blank_or_breakz, is_break, is_flow, len_until,
};
use crate::parsing::scanner::SkipTabs;
use crate::parsing::source::shared_skip_ws_to_eol;
use alloc::vec::Vec;
use core::cmp::min;

/// A structure that wraps a slice os strings
///
/// `StrSource` provides a way to work with string-like byte input slices (`&[u8]`)
/// and keep track of the current position within the input.
///
/// # Type Parameters
/// - `'input`: Lifetime of the input data slice.
/// # Example
/// ```
/// use yam_core::prelude::{InMemSource, Source};
/// let source = InMemSource::new("Hello, world!");
/// assert_eq!(source.peekz(0), b'H');
/// assert_eq!(source.peekz(1), b'e');
/// ```
///
/// This struct can be used for parsing, tokenizing, or other scenarios
/// where keeping track of a position in a byte slice is required.
pub struct InMemSource<'input> {
    /// A reference to the byte slice (`&[u8]`) that serves as the source data.
    input: &'input [u8],
    /// A zero based index (`usize`) representing the current position within the input.
    pos: usize,
}

impl<'input> InMemSource<'input> {
    /// Returns the not yet consumed part of the input.
    #[inline]
    fn rest(&self) -> &'input [u8] {
        self.input.get(self.pos..).unwrap_or_default()
    }

    ///
    /// Creates a new instance of `StrSource` from a given string slice.
    ///
    /// Returns a new `StrSource` instance initialized with the provided string slice.
    /// The string slice is internally converted to bytes and a starting position of 0 is set.
    ///
    /// # Attributes
    /// - `#[must_use]`: Indicates that the returned `StrSource` instance must be used,
    ///   otherwise a warning will be emitted by the compiler.
    ///
    /// # Example
    /// ```rust
    /// use yam_core::prelude::{InMemSource, Source};
    /// let source = InMemSource::new("example");
    /// assert_eq!(source.peek_char(),'e');
    /// ```
    #[must_use]
    pub fn new(input: &'input str) -> InMemSource<'input> {
        InMemSource {
            input: input.as_bytes(),
            pos: 0,
        }
    }
}

unsafe impl Source for InMemSource<'_> {
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
        self.update_pos(self.pos + n);
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

    fn fill_buf_to_max(&mut self) {}

    #[inline]
    fn update_pos(&mut self, pos: usize) {
        let capped = min(self.input.len(), pos);
        self.pos = capped;
    }
}

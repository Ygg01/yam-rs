// MIT License
//
// Copyright (c) [2024] [simd-json.rs developers]
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#![allow(unused)]

use alloc::vec;
use alloc::vec::Vec;

use simdutf8::basic::imp::ChunkedUtf8Validator;

use crate::tokenizer::stage2::{Buffer, YamlParserState};
use crate::{util, NativeScanner, ParseResult, EVEN_BITS, ODD_BITS};

pub struct YamlChunkState {
    pub double_quote: YamlDoubleQuoteChunk,
    pub single_quote: YamlSingleQuoteChunk,
    pub characters: YamlCharacterChunk,
    pub rows: Vec<u32>,
    pub cols: Vec<u32>,
    pub indents: Vec<u32>,
    follows_non_quote_scalar: u64,
    error_mask: u64,
}

impl Default for YamlChunkState {
    fn default() -> Self {
        // Safety
        // To ensure cols/rows/indents are always 64 elements long.
        YamlChunkState {
            rows: vec![0, 64],
            cols: vec![0, 64],
            indents: vec![0, 64],
            ..Self::default()
        }
    }
}

#[derive(Default)]
pub struct YamlDoubleQuoteChunk {
    /// Escaped characters
    escaped: u64,
    /// Real double quotes
    quote_bits: u64,
    /// Bitmask showing which characters are in string
    in_string: u64,
}

#[derive(Default)]
pub struct YamlSingleQuoteChunk {
    /// Finds group of paired quotes
    pub odd_quotes: u64,

    /// Finds group of paired quotes like `''` or `''''` that are escaped.
    pub escaped_quotes: u64,

    /// Bitmask showing which characters are in string
    pub in_string: u64,
}

#[derive(Default)]
pub struct YamlCharacterChunk {
    /// Whitespace bitmask SPACE  (`0x20`) , TABS (`0x09`), LINE_FEED (`0x0A`) or CARRIAGE_RETURN (`0x0D`)
    pub whitespace: u64,
    /// SPACE (`0x20`) bitmask
    pub spaces: u64,
    /// LINE_FEED (`0x0A`) bitmask
    pub line_feeds: u64,
    /// Operators used in YAML
    pub structurals: u64,
}

pub(crate) type NextFn<B> = for<'buffer, 'input> unsafe fn(
    chunk: &'buffer [u8; 64],
    buffers: &'input mut B,
    state: &'input mut YamlParserState,
) -> ParseResult<YamlChunkState>;

/// A trait representing a stage 1 scanner for parsing YAML input.
///
/// This trait provides methods for validating and scanning chunks of data, and finding important
/// parts like structural starts and so on.
pub unsafe trait Stage1Scanner {
    /// Type [`Stage1Scanner`] uses to perform it's SIMD-ifed actions.
    type SimdType;

    /// [`ChunkedUtf8Validator`] that matches the [`Stage1Scanner`] architecture.
    type Validator: ChunkedUtf8Validator;

    /// Returns the validator for the given type.
    ///
    /// The `validator` function is a generic method that returns the validator for the type it is called on. The `Self` keyword is used to refer to the type of the implementing struct or trait.
    fn validator() -> Self::Validator;

    /// Constructs a new instance of `Self` by converting a slice of 64 `u8` values.
    ///
    /// # Arguments
    ///
    /// * `values` - A slice of 64 `u8` values that represents a chunk of data.
    ///
    /// # Example
    ///
    /// ```
    /// use yam_dark_core::Stage1Scanner;
    /// use yam_dark_core::NativeScanner;
    ///
    /// let values: [u8; 64] = [0; 64];
    /// let result = NativeScanner::from_chunk(&values);
    /// ```
    ///
    /// # Returns
    ///
    /// A new instance of [`Stage1Scanner`] constructed from the given `values`.
    fn from_chunk(values: &[u8; 64]) -> Self;

    /// Compares the ASCII value of the given input with the internal value
    /// of the struct and returns a 64-bit bitmask.
    ///
    /// # Arguments
    ///
    /// * `m` - A u8 value representing the ASCII character to compare with.
    ///
    /// # Returns
    ///
    /// An `u64` value representing the bitmask of the comparison.
    ///
    /// # Example
    ///
    /// ```
    /// use yam_dark_core::Stage1Scanner;
    /// use yam_dark_core::NativeScanner;
    ///
    /// let values: [u8; 64] = [0; 64];
    /// let result = NativeScanner::from_chunk(&values);
    /// let bitmask = result.cmp_ascii_to_input(1);
    /// assert_eq!(bitmask, 0);
    /// ```
    fn cmp_ascii_to_input(&self, m: u8) -> u64;

    /// Calculates the indents of the given chunk and updates the `chunk_state` accordingly.
    ///
    /// For a chunk represented by this scanner, will calculate indents for each 64-character and
    /// will update `chunk_state`, taking into consideration previous indents in `prev_state`
    ///
    /// # Implementation
    ///
    /// It's important for implementation to first check where spaces `0x20` and line feed characters are located
    /// Since newline on Windows is `\r\n` Unicode `0x0A` and `0x0D` respectively we can approximate a newline with `\n`.
    /// Spaces are important because only `0x20` is a valid YAML indentation mechanism.
    ///
    /// # Arguments
    ///
    /// - `chunk_state`: A mutable reference to a [`YamlChunkState`] that represents the YAML
    ///    chunk to calculate the indents for.
    /// - `prev_state`: A mutable reference to a [`YamlParserState`] that represents the previous
    ///    state of the YAML parser.
    ///
    /// # Examples
    /// ```
    /// // TODO
    /// ```
    fn calculate_indents(&self, chunk_state: &mut YamlChunkState, prev_state: &mut YamlParserState);

    /// Computes a quote mask based on the given quote bit mask.
    ///
    /// The `compute_quote_mask` function takes an input `quote_bits` of type `u64` and calculates
    /// a quote mask. The quote mask is a bitmask that has a binary 1 in every position where the
    /// corresponding byte is `"` (keep in mind that binary representation is big endian, while array
    /// representation is little endian).
    ///
    /// # Arguments
    ///
    /// * `quote_bits` - The quote bits of type `u64` that specify the positions to be masked.
    ///
    /// # Returns
    ///
    /// The computed quote mask of type `u64`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use yam_dark_core::{NativeScanner, Stage1Scanner};
    ///
    /// let quote_bits = 0b0000100001;
    /// let quote_mask = NativeScanner::compute_quote_mask(quote_bits);
    /// assert_eq!(quote_mask, 0b11111);
    /// ```
    fn compute_quote_mask(quote_bits: u64) -> u64;

    /// Checks if the value of `cmp` is less than or equal to the value of `self`.
    ///
    /// Returns the result as a `u64` value.
    ///
    /// # Arguments
    ///
    /// * `cmp` - An `i8` value representing the number to be compared against `self`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use yam_dark_core::{NativeScanner, Stage1Scanner, YamlCharacterChunk};
    ///
    /// let bin_str = b"                                                                ";
    /// let mut chunk = YamlCharacterChunk::default();
    /// let scanner = NativeScanner::from_chunk(bin_str);
    /// let result = scanner.unsigned_lteq_against_splat(0x20);
    /// assert_eq!(result, 0b1111111111111111111111111111111111111111111111111111111111111111);
    /// ```
    fn unsigned_lteq_against_splat(&self, cmp: i8) -> u64;

    /// Counts the number of odd bits in a given 64-bit bitmask.
    ///
    /// # Arguments
    ///
    /// * `bitmask` - A 64-bit unsigned integer representing the quote bits.
    ///
    /// # Returns
    ///
    /// Returns `0` if bitmask contains even number of `1` bits, and `1` if it contains odd number
    /// of `1` bits. Zero bits is considered even.
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::yam_dark_core::NativeScanner;
    /// use crate::yam_dark_core::Stage1Scanner;
    ///
    /// let quote_bits = 0b10101010;
    /// let count = NativeScanner::count_odd_bits(quote_bits);
    /// assert_eq!(count, 0);
    ///
    /// let quote_bits = 0b1000101;
    /// let count = NativeScanner::count_odd_bits(quote_bits);
    /// assert_eq!(count, 1);
    /// ```
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn count_odd_bits(bitmask: u64) -> u32 {
        bitmask.count_ones() % 2
    }

    /// This function processes the next chunk of a YAML input.
    ///
    /// It takes a reference to a byte slice `chunk` containing the next 64 bytes of input data,
    /// a mutable reference to a `buffers` object implementing the `Buffer` trait,
    /// and a mutable reference to a `prev_state` object of type `YamlParserState`.
    ///
    /// # Arguments
    ///
    /// * `chunk` - A reference to a byte slice `chunk` containing the next 64 bytes of input data.
    /// * `buffers` - A mutable reference to a `buffers` object implementing the `Buffer` trait.
    /// * `prev_state` - A mutable reference to a [`YamlParserState`] object that stores previous iteration state information.
    ///
    /// # Returns
    ///
    /// Returns the Result that returns an error if it encounters a parse error or [`YamlChunkState`].
    /// [`YamlChunkState`] stores current iteration information and is merged on each [`Stage1Scanner::next`]
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn next<T: Buffer>(
        chunk: &[u8; 64],
        buffers: &mut T,
        prev_state: &mut YamlParserState,
    ) -> ParseResult<YamlChunkState>
    where
        Self: Sized,
    {
        let mut chunk_state = YamlChunkState::default();
        let mut simd = Self::from_chunk(chunk);
        let double_quotes = simd.cmp_ascii_to_input(b'"');

        simd.scan_whitespace_and_structurals(&mut chunk_state);

        // Pre-requisite
        // LINE FEED needs to be gathered before calling `calculate_indents`
        simd.calculate_indents(&mut chunk_state, prev_state);

        simd.scan_double_quote_bitmask(&mut chunk_state, prev_state);
        simd.scan_single_quote_bitmask(&mut chunk_state, prev_state);

        prev_state.merge_state(chunk, buffers, &mut chunk_state)
    }

    /// Returns a bitvector indicating where there are characters that end an odd-length sequence
    /// of backslashes. An odd-length sequence of backslashes changes the behavior of the next
    /// character that follows. An even-length sequence of backslashes, as well as the largest
    /// even-length prefix of an odd-length sequence of backslashes, modify the behavior of the
    /// backslashes themselves.
    ///
    /// The `prev_iteration_odd` reference parameter is also updated to indicate whether the iteration
    /// ends on an odd-length sequence of backslashes. This modification affects the subsequent search
    /// for odd-length sequences of backslashes.
    ///
    /// # Arguments
    ///
    /// * `prev_iteration_odd` - A mutable reference to a `u64` representing the previous iteration's
    ///                          odd-length sequence of backslashes.
    ///
    /// # Returns
    ///
    /// Returns a `u64` as a bitvector indicating the positions where odd-length sequences of
    /// backslashes end.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use crate::yam_dark_core::Stage1Scanner;
    /// use crate::yam_dark_core::NativeScanner;
    /// let mut prev_iteration_odd = 0;
    ///
    /// let chunk = b" \\ \\\\  \\\\\\    \\   \\\\  \\\\    \\   \\\\        \\     \\    \\\\    \\    ";
    /// let scanner = NativeScanner::from_chunk(chunk);
    /// let result = scanner.scan_for_odd_backslashes(&mut prev_iteration_odd);
    /// assert_eq!(result, 0b1000000000010000010000000000000100000000000001000010000000100);
    /// ```
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn scan_for_odd_backslashes(&self, prev_iteration_odd: &mut u32) -> u64 {
        Self::scan_for_mask(self.cmp_ascii_to_input(b'\\'), prev_iteration_odd, ODD_BITS)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn scan_for_mask(bits: u64, prev_iteration_odd: &mut u32, mask: u64) -> u64 {
        let start_edges = bits & !(bits << 1);
        let prev_iter_odd = u64::from(*prev_iteration_odd);

        // flip lowest if we have an odd-length run at the end of the prior iteration
        let even_start_mask = EVEN_BITS ^ prev_iter_odd;
        let even_starts = start_edges & even_start_mask;
        let odd_starts = start_edges & !even_start_mask;
        let even_carries = bits.wrapping_add(even_starts);

        // must record the carry-out of our odd-carries out of bit 63; this
        // indicates whether the sense of any edge going to the next iteration
        // should be flipped
        let (mut odd_carries, iter_ends_odd_backslash) = bits.overflowing_add(odd_starts);

        odd_carries |= prev_iter_odd;
        // push in a bit zero as a potential end
        // if we had an odd-numbered run at the
        // end of the previous iteration
        *prev_iteration_odd = u32::from(iter_ends_odd_backslash);
        let even_carry_ends = even_carries & !bits;
        let odd_carry_ends = odd_carries & !bits;
        let even_start_odd_end = even_carry_ends & mask;
        let odd_start_even_end = odd_carry_ends & !mask;
        even_start_odd_end | odd_start_even_end
    }

    /// Scans for single quote bitmask.
    ///
    /// # Arguments
    ///
    /// - `block_state`: A mutable reference to a current [`YamlChunkState`]. It will  update the
    /// [YamlSingleQuoteChunk] with data for scanned single quotes.
    /// - `prev_iter_state`: A mutable reference to previous iteration [`YamlParserState`].
    ///
    /// # Example
    ///
    /// ```rust
    ///  use yam_dark_core::{NativeScanner, Stage1Scanner, YamlChunkState, YamlParserState};
    ///  let mut block_state = YamlChunkState::default();
    ///  let mut prev_iter_state = YamlParserState::default();
    ///
    ///  let chunk = b" ' ''  '                                                        ";
    ///  let scanner = NativeScanner::from_chunk(chunk);
    ///  scanner.scan_single_quote_bitmask(&mut block_state, &mut prev_iter_state);
    ///  let expected = 0b0000000000000000000000000000000000000000000000000000010000010;
    ///  assert_eq!(block_state.single_quote.odd_quotes, expected, "Expected:    {:#066b} \nGot instead: {:#066b} ", expected, block_state.single_quote.odd_quotes);
    /// ```
    // #[cfg_attr(not(feature = "no-inline"), inline)]
    fn scan_single_quote_bitmask(
        &self,
        chunk_state: &mut YamlChunkState,
        prev_iter_state: &mut YamlParserState,
    ) {
        let quotes = self.cmp_ascii_to_input(b'\'');
        let end_edge = quotes & !(quotes >> 1);
        let start_edge = quotes & !(quotes << 1);

        let even_ends =
            Self::scan_for_mask(quotes, &mut prev_iter_state.prev_iter_odd_quote, EVEN_BITS);

        let even_mask = Self::calculate_mask_from_end(quotes, even_ends >> 1);

        chunk_state.single_quote.odd_quotes = quotes & !even_mask;
        chunk_state.single_quote.escaped_quotes = even_mask;
    }

    /// Calculates a mask from the provided quote bits and an even boundary value.
    /// Given a set of bitmask and highest bits in consecutive group of `1` it will select all neighboring ones to the right (using big endian number notation)
    ///
    /// # Arguments
    ///
    /// * `quote_bits`: A 64-bit unsigned integer representing bitmask
    /// * `even_ends`: Highest bit of a group of `1` in `quote_bits`, used for selecting those bits
    ///
    /// # Returns
    ///
    /// A 64-bit unsigned integer representing the bits that were selected based on the `even_ends`
    ///
    /// # Examples
    /// ```
    ///  use yam_dark_core::{NativeScanner, Stage1Scanner};
    ///
    ///  let actual = NativeScanner::calculate_mask_from_end(
    ///     0b1111_0000_0000_0000_0000_0000_0000_1110_0000_0000_0000_0000_0000_0000_0000_0110,
    ///     0b1000_0010_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0100
    ///  );
    ///  let expected = 0b1111_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0110;
    ///  assert_eq!(
    ///     actual, expected,
    ///     "\nExpected: {:#018b}\n  Actual: {:#018b}",
    ///     expected, actual
    ///  );
    /// ```
    fn calculate_mask_from_end(quote_bits: u64, even_ends: u64) -> u64 {
        util::select_consecutive_bits_branchless(quote_bits, even_ends)
    }

    /// Scans the whitespace and structurals in the given YAML chunk state.
    ///
    /// # Arguments
    ///
    /// - `block_state` - A mutable reference to the [`YamlChunkState`] for scanning.
    ///
    /// # Nibble mask
    ///
    /// Based on structure in structure.md, we compute low and high nibble mask and use them to swizzle
    /// higher and lower component of a byte. E.g. if a byte is `0x23`, we use the `low_nibble[2]` and
    /// `high_nibble[3]` for swizzling.
    ///
    /// # Example
    /// ```rust
    ///  use yam_dark_core::{NativeScanner, Stage1Scanner, YamlChunkState, YamlParserState};
    ///  let mut block_state = YamlChunkState::default();
    ///  let mut prev_iter_state = YamlParserState::default();
    ///  let chunk = b" -                                                              ";
    ///  let scanner = NativeScanner::from_chunk(chunk);
    ///  scanner.scan_whitespace_and_structurals(&mut block_state);
    ///  let expected = 0b000000000000000000000000000000000000000000000000000000000010;
    ///  assert_eq!(
    ///     block_state.characters.structurals,
    ///     expected, "Expected:    {:#066b} \nGot instead: {:#066b} ", expected, block_state.single_quote.odd_quotes
    ///  );
    /// ```
    fn scan_whitespace_and_structurals(&self, chunk_state: &mut YamlChunkState);

    /// Scans the input for double quote bitmask.
    ///
    /// # Arguments
    ///
    /// * `block_state` - A mutable reference to the [`YamlChunkState`] struct.
    /// * `prev_iter_state` - A mutable reference to the [`YamlParserState`] struct.
    ///
    /// # Example
    ///
    /// ```rust
    ///  use yam_dark_core::{NativeScanner, Stage1Scanner, YamlChunkState, YamlParserState};
    ///
    ///  let mut block_state = YamlChunkState::default();
    ///  let mut prev_iter_state = YamlParserState::default();
    ///  let chunk = b" \"  \"                                                           ";
    ///  let scanner = NativeScanner::from_chunk(chunk);
    ///  let result = scanner.scan_double_quote_bitmask(&mut block_state, &mut prev_iter_state);
    ///  let expected = 0b000000000000000000000000000000000000000000000000000000000010;
    /// ```
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn scan_double_quote_bitmask(
        &self,
        chunk_state: &mut YamlChunkState,
        prev_iter_state: &mut YamlParserState,
    ) {
        let odds_ends = self.scan_for_odd_backslashes(&mut prev_iter_state.prev_iter_odd_backslash);

        chunk_state.double_quote.quote_bits = self.cmp_ascii_to_input(b'"');
        chunk_state.double_quote.quote_bits &= !odds_ends;

        // remove from the valid quoted region the unescaped characters.
        let mut quote_mask: u64 = Self::compute_quote_mask(chunk_state.double_quote.quote_bits);
        quote_mask ^= prev_iter_state.prev_iter_inside_quote;

        // All Unicode characters may be placed within the
        // quotation marks, except for the characters that MUST be escaped:
        // quotation mark, reverse solidus, and the control characters (U+0000
        //through U+001F).
        // https://tools.ietf.org/html/rfc8259
        let unescaped: u64 = self.unsigned_lteq_against_splat(0x1F);
        chunk_state.error_mask |= quote_mask & unescaped;
        // right shift of a signed value expected to be well-defined and standard
        // compliant as of C++20,
        // John Regher from Utah U. says this is fine code
        prev_iter_state.prev_iter_inside_quote = unsafe {
            core::mem::transmute::<i64, u64>(core::mem::transmute::<u64, i64>(quote_mask) >> 63)
        };
        chunk_state.double_quote.quote_bits = quote_mask;
    }
}

#[test]
fn test_single_quotes1() {
    let mut block_state = YamlChunkState::default();
    let mut prev_iter_state = YamlParserState::default();

    let chunk = b" ' ''  '''                                                      ";
    let scanner = NativeScanner::from_chunk(chunk);
    scanner.scan_single_quote_bitmask(&mut block_state, &mut prev_iter_state);
    let expected = 0b000000000000000000000000000000000000000000000000001110000010;
    assert_eq!(
        expected, block_state.single_quote.odd_quotes,
        "Expected:    {:#066b} \nGot instead: {:#066b} ",
        expected, block_state.single_quote.odd_quotes
    );
}

#[test]
fn test_single_quotes2() {
    let mut block_state = YamlChunkState::default();
    let mut prev_iter_state = YamlParserState::default();

    let chunk = b" ' ''  '' '                                                     ";
    let scanner = NativeScanner::from_chunk(chunk);
    scanner.scan_single_quote_bitmask(&mut block_state, &mut prev_iter_state);
    let expected = 0b0000000000000000000000000000000000000000000000000010000000010;
    assert_eq!(
        expected, block_state.single_quote.odd_quotes,
        "Expected:    {:#066b} \nGot instead: {:#066b} ",
        expected, block_state.single_quote.odd_quotes
    );
}

#[test]
fn test_structurals() {
    let mut block_state = YamlChunkState::default();
    let mut prev_iter_state = YamlParserState::default();
    let chunk = b" -                                                              ";
    let scanner = NativeScanner::from_chunk(chunk);
    scanner.scan_whitespace_and_structurals(&mut block_state);
    let expected = 0b000000000000000000000000000000000000000000000000000000000010;
    assert_eq!(
        block_state.characters.structurals, expected,
        "Expected:    {:#066b} \nGot instead: {:#066b} ",
        expected, block_state.single_quote.escaped_quotes
    );
}

#[test]
fn test_lteq() {
    let bin_str = b"                                                                ";
    let mut chunk = YamlCharacterChunk::default();
    let scanner = NativeScanner::from_chunk(bin_str);
    let result = scanner.unsigned_lteq_against_splat(0x20);
    assert_eq!(
        result,
        0b1111111111111111111111111111111111111111111111111111111111111111
    );
}

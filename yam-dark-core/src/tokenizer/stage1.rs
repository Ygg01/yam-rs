// MIT License
//
// Copyright (c) [2024] [simd-json.rs developers]
// Copyright (c) [2024] Ygg One
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

use crate::tokenizer::stage2::{Buffer, YamlParserState};
use crate::{util, EvenOrOddBits, NativeScanner, ParseResult, SIMD_CHUNK_LENGTH};
use simdutf8::basic::imp::ChunkedUtf8Validator;
use EvenOrOddBits::OddBits;

/// Represents the state of YAML chunk processing.
///
/// `YamlChunkState` is used to track the state of various chunks of YAML content,
/// including double-quoted strings, single-quoted strings, and character classifications
/// such as whitespace and structural characters.
///
/// This struct also maintains vectors for row and column positions and
/// indent levels, which are updated as the YAML content is processed.
///
/// # Fields
///
/// * `double_quote` - [`YamlDoubleQuoteChunk`] struct containing double-quoted YAML strings information.
/// * `single_quote` - [`YamlSingleQuoteChunk`] struct containing single-quoted YAML strings information.
/// * `characters` - [`YamlCharacterChunk`] struct containing info for characters (e.g., whitespace, operators).
/// * `rows` - [`Vec`] maintaining the row positions in the chunk.
/// * `cols` - [`Vec`] maintaining the column positions in the chunk.
/// * `indents` - [`Vec`] maintaining the indent levels in the chunk.
/// * `follows_non_quote_scalar` - Bitmask indicating positions following non-quote scalar values.
/// * `error_mask` - Bitmask indicating positions with errors.
///
/// # Default Implementation
///
/// The `Default` implementation for `YamlChunkState` initializes
/// the vectors (`rows`, `cols`, `indents`) to be 64 elements long
/// and sets the `double_quote`, `single_quote`, `characters` to its default values.
///
/// ```rust
/// use yam_dark_core::YamlChunkState;
/// let x = YamlChunkState::default();
/// ```
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
            double_quote: YamlDoubleQuoteChunk::default(),
            single_quote: YamlSingleQuoteChunk::default(),
            characters: YamlCharacterChunk::default(),
            rows: vec![0, 64],
            cols: vec![0, 64],
            indents: vec![0, 64],
            follows_non_quote_scalar: 0,
            error_mask: 0,
        }
    }
}

#[derive(Default)]
/// Represents the state of double-quoted YAML string processing.
///
/// `YamlDoubleQuoteChunk` is used to track the state of double-quoted YAML strings,
/// maintaining information about escaped characters, real double quotes, and whether
/// characters are within the string context.
///
/// # Fields
///
/// * `escaped` - A bitmask indicating the positions of escaped characters.
/// * `quote_bits` - A bitmask indicating the positions of real double quotes.
/// * `in_string` - A bitmask showing which characters are currently within a double-quoted string.
///
/// # Default Implementation
///
/// The `Default` implementation for `YamlDoubleQuoteChunk` initializes
/// the fields `escaped`, `quote_bits`, and `in_string` to 0.
///
/// ```rust
/// use yam_dark_core::YamlDoubleQuoteChunk;
/// let y = YamlDoubleQuoteChunk::default();
/// assert_eq!(y.escaped, 0);
/// assert_eq!(y.quote_bits, 0);
/// assert_eq!(y.in_string, 0);
/// ```
pub struct YamlDoubleQuoteChunk {
    /// Escaped characters
    pub escaped: u64,
    /// Real double quotes
    pub quote_bits: u64,
    /// Bitmask showing which characters are in string
    pub in_string: u64,
}

#[derive(Default)]
///
/// Represents the state of single-quoted YAML string processing.
///
/// `YamlSingleQuoteChunk` is used to track the state of single-quoted YAML strings,
/// maintaining information about odd quotes, escaped quotes, and whether
/// characters are within the string context.
///
/// # Fields
///
/// * `odd_quotes` - A bitmask indicating the positions of odd quotes.
/// * `escaped_quotes` - A bitmask indicating the positions of escaped quotes.
/// * `in_string` - A bitmask showing which characters are currently within a single-quoted string.
///
/// # Default Implementation
///
/// The `Default` implementation for `YamlSingleQuoteChunk` initializes
/// the fields `odd_quotes`, `escaped_quotes`, and `in_string` to 0.
///
/// ```rust
/// use yam_dark_core::YamlSingleQuoteChunk;
/// let y = YamlSingleQuoteChunk::default();
/// assert_eq!(y.odd_quotes, 0);
/// assert_eq!(y.escaped_quotes, 0);
/// assert_eq!(y.in_string, 0);
/// ```
pub struct YamlSingleQuoteChunk {
    /// Finds group of paired quotes
    pub odd_quotes: u64,

    /// Finds group of paired quotes like `''` or `''''` that are escaped.
    pub escaped_quotes: u64,

    /// Bitmask showing which characters are in string
    pub in_string: u64,
}

#[derive(Default)]
/// Represents the state of general character processing in YAML parsing.
///
/// `YamlCharacterChunk` is used to track the state of various character types such as whitespace,
/// line feeds, spaces, and structural characters within a YAML document.
///
/// # Fields
///
/// * `whitespace` - A bitmask indicating the positions of whitespace characters (`SPACE`, `TAB`, `LINE_FEED`, or `CARRIAGE_RETURN`).
/// * `spaces` - A bitmask indicating the positions of `SPACE` (`0x20`) characters.
/// * `line_feeds` - A bitmask indicating the positions of `LINE_FEED` (`0x0A`) characters.
/// * `structurals` - A bitmask indicating the positions of characters used as operators in YAML.
///
/// # Default Implementation
///
/// The `Default` implementation for `YamlCharacterChunk` initializes the fields `whitespace`,
/// `spaces`, `line_feeds`, and `structurals` to 0.
///
/// ```rust
/// use yam_dark_core::YamlCharacterChunk;
/// let y = YamlCharacterChunk::default();
/// assert_eq!(y.whitespace, 0);
/// assert_eq!(y.spaces, 0);
/// assert_eq!(y.line_feeds, 0);
/// assert_eq!(y.block_structurals, 0);
/// ```
pub struct YamlCharacterChunk {
    /// Whitespace bitmask SPACE  (`0x20`) , TABS (`0x09`), LINE_FEED (`0x0A`) or CARRIAGE_RETURN (`0x0D`)
    pub whitespace: u64,
    /// SPACE (`0x20`) bitmask
    pub spaces: u64,
    /// LINE_FEED (`0x0A`) bitmask
    pub line_feeds: u64,
    /// Block operators used in YAML
    pub block_structurals: u64,
    /// Flow operators used in YAML
    pub flow_structurals: u64,
    /// Bitmask showing if chunk character is in_comment
    pub in_comment: u64,
}

impl YamlCharacterChunk {
    pub const fn all_structurals(&self) -> u64 {
        self.flow_structurals | self.block_structurals
    }
}

pub(crate) type NextFn<B> = for<'buffer, 'input> unsafe fn(
    chunk: &'buffer [u8; SIMD_CHUNK_LENGTH],
    buffers: &'input mut B,
    state: &'input mut YamlParserState,
) -> ParseResult<YamlChunkState>;

const SIMD_CHUNK_SIZE: usize = 64;

/// A trait representing a stage 1 scanner for parsing `YAML` input.
///
/// This trait provides methods for validating and scanning chunks of data, and finding important
/// parts like structural starts and so on.
///
/// # Safety
///
/// This trait MUST ALWAYS return valid positions in given stream in bytes. They will be used for unchecked
/// access to the underlying bytes.
pub unsafe trait Stage1Scanner {
    /// Type [`Stage1Scanner`] uses to perform SIMD accelerated actions.
    type SimdType;

    /// [`ChunkedUtf8Validator`] that matches the [`Stage1Scanner`] architecture.
    type Validator: ChunkedUtf8Validator;

    /// Returns the  [`Self::Validator`] for the given trait implementor.
    ///
    /// The `validator` function is a generic method that returns the validator for the type it is called on.
    ///
    /// # Safety
    /// Method implementers need to make sure they are calling the right implementation for correct architecture.
    unsafe fn validator() -> Self::Validator;

    /// Constructs a new instance of `Self` by converting a slice of 64 `u8` values.
    ///
    /// # Arguments
    ///
    /// * `data_chunk` - A reference to an array of 64 `u8` values that represents a chunk of data.
    ///
    /// # Example
    ///
    /// ```
    /// use yam_dark_core::{Stage1Scanner, SIMD_CHUNK_LENGTH};
    /// use yam_dark_core::NativeScanner;
    ///
    /// let data_chunk: [u8; SIMD_CHUNK_LENGTH] = [0; SIMD_CHUNK_LENGTH];
    /// let result = NativeScanner::from_chunk(&data_chunk);
    /// ```
    ///
    /// # Returns
    ///
    /// A new instance of [`Stage1Scanner`] constructed from the given `values`.
    fn from_chunk(data_chunk: &[u8; SIMD_CHUNK_LENGTH]) -> Self;

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
    /// use yam_dark_core::{Stage1Scanner, SIMD_CHUNK_LENGTH};
    /// use yam_dark_core::NativeScanner;
    ///
    /// let values: [u8; SIMD_CHUNK_LENGTH] = [0; SIMD_CHUNK_LENGTH];
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

    /// Scans the whitespace and structurals in the given YAML chunk state.
    /// This method sets [`YamlCharacterChunk`] part of [`YamlChunkState`].
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
    ///  scanner.classify_yaml_characters(&mut block_state);
    ///  let expected = 0b000000000000000000000000000000000000000000000000000000000010;
    ///  assert_eq!(
    ///     block_state.characters.block_structurals,
    ///     expected, "Expected:    {:#066b} \nGot instead: {:#066b} ", expected, block_state.single_quote.odd_quotes
    ///  );
    /// ```
    fn classify_yaml_characters(&self, chunk_state: &mut YamlChunkState);

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
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn compute_quote_mask(quote_bits: u64) -> u64 {
        let mut quote_mask: u64 = quote_bits ^ (quote_bits << 1);
        quote_mask = quote_mask ^ (quote_mask << 2);
        quote_mask = quote_mask ^ (quote_mask << 4);
        quote_mask = quote_mask ^ (quote_mask << 8);
        quote_mask = quote_mask ^ (quote_mask << 16);
        quote_mask = quote_mask ^ (quote_mask << 32);
        quote_mask
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
    /// * `buffers` - A mutable reference to a `buffers` object implementing the [`Buffer`] trait.
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

        simd.classify_yaml_characters(&mut chunk_state);

        // Pre-requisite
        // LINE FEED needs to be gathered before calling `calculate_indents`/`scan_for_comments`/
        // `scan_for_double_quote_bitmask`/`scan_single_quote_bitmask`
        simd.scan_for_comments(&mut chunk_state, prev_state);
        simd.calculate_indents(&mut chunk_state, prev_state);

        simd.scan_double_quote_bitmask(&mut chunk_state, prev_state);
        simd.scan_single_quote_bitmask(&mut chunk_state, prev_state);

        prev_state.merge_state(chunk, buffers, &mut chunk_state)
    }

    /// This function processes the comments for current chunk of characters.
    ///
    /// It takes a mutable reference to current [`chunk_state`](YamlChunkState) containing the current chunk data (like spaces and line feeds, etc.)
    /// and a  mutable reference to a [`parser_state`](YamlParserState) which tracks parser state.
    ///
    /// # Arguments
    ///
    /// * `chunk_state` - A mutable reference to a [YamlChunkState] object that contains current chunk data.
    /// * `parser_state` - A mutable reference to a [YamlParserState] object that stores parser's state information.
    ///
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn scan_for_comments(
        &self,
        chunk_state: &mut YamlChunkState,
        parser_state: &mut YamlParserState,
    ) {
        let character = self.cmp_ascii_to_input(b'#');
        let shifted_spaces =
            (chunk_state.characters.spaces << 1) ^ u64::from(parser_state.is_previous_white_space);

        let comment_start = (character & shifted_spaces) | u64::from(parser_state.is_in_comment);
        let not_whitespace = !chunk_state.characters.line_feeds;

        chunk_state.characters.in_comment =
            util::select_left_bits_branch_less(not_whitespace, comment_start);

        // Update values for next iteration.
        parser_state.is_in_comment = chunk_state.characters.in_comment >> 63 == 1;
        parser_state.is_previous_white_space = (chunk_state.characters.spaces >> 63) == 1;
    }

    /// Returns a bitmask indicating where there are characters that end an odd-length sequence
    /// of ones.
    ///
    /// The `prev_iteration_result` reference parameter is also updated to indicate whether the iteration
    /// needs to be taken into account by subsequent search.
    ///
    /// # Arguments
    ///
    /// * `prev_iteration_result` - A mutable reference to a `u64` representing the previous iteration's
    ///                          result of backslashes. It will be updated with post result info.
    /// * `mask` - A bitmask determining ODD or Even Mask to be used.
    ///
    /// # Returns
    ///
    /// Returns a `u64` as a bitvector indicating the positions where odd-length sequences of
    /// backslashes end.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use yam_dark_core::EvenOrOddBits::OddBits;
    /// use yam_dark_core::{EvenOrOddBits};
    /// use crate::yam_dark_core::Stage1Scanner;
    /// use crate::yam_dark_core::NativeScanner;
    /// let mut prev_iteration_odd = false;
    ///
    /// let chunk = b" \\ \\\\  \\\\\\    \\   \\\\  \\\\    \\   \\\\        \\     \\    \\\\    \\    ";
    /// let scanner = NativeScanner::from_chunk(chunk);
    /// let result = NativeScanner::scan_for_mask(scanner.cmp_ascii_to_input(b'\\'), &mut prev_iteration_odd, OddBits);
    /// assert_eq!(result, 0b1000000000010000010000000000000100000000000001000010000000100);
    /// ```
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn scan_for_mask(bits: u64, prev_iteration_result: &mut bool, mask: EvenOrOddBits) -> u64 {
        let start_edges = bits & !(bits << 1);
        let prev_iter_odd = u64::from(*prev_iteration_result);

        // flip lowest if we have an odd-length run at the end of the prior iteration
        let even_start_mask = (EvenOrOddBits::EvenBits as u64) ^ prev_iter_odd;
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
        *prev_iteration_result = iter_ends_odd_backslash;
        let even_carry_ends = even_carries & !bits;
        let odd_carry_ends = odd_carries & !bits;
        let even_start_odd_end = even_carry_ends & mask as u64;
        let odd_start_even_end = odd_carry_ends & !(mask as u64);
        even_start_odd_end | odd_start_even_end
    }

    /// Scans for single quote bitmask.
    ///
    /// # Arguments
    ///
    /// - `block_state`: A mutable reference to a current [`YamlChunkState`]. It will  update the
    ///   [YamlSingleQuoteChunk] with data for scanned single quotes.
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
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn scan_single_quote_bitmask(
        &self,
        chunk_state: &mut YamlChunkState,
        prev_iter_state: &mut YamlParserState,
    ) {
        let quotes = self.cmp_ascii_to_input(b'\'');

        let even_ends = Self::scan_for_mask(
            quotes,
            &mut prev_iter_state.is_prev_iter_odd_single_quote,
            EvenOrOddBits::EvenBits,
        );

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
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn calculate_mask_from_end(quote_bits: u64, even_ends: u64) -> u64 {
        util::select_right_bits_branch_less(quote_bits, even_ends)
    }

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
        let prev_iteration_odd = &mut prev_iter_state.is_prev_double_quotes;
        let odds_ends =
            Self::scan_for_mask(self.cmp_ascii_to_input(b'\\'), prev_iteration_odd, OddBits);

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
    scanner.classify_yaml_characters(&mut block_state);
    let expected = 0b000000000000000000000000000000000000000000000000000000000010;
    assert_eq!(
        block_state.characters.block_structurals, expected,
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

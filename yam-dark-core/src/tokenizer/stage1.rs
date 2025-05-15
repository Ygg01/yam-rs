// MIT License
//
// Copyright (c) [2024] [simd-json.rs developers]
// Copyright (c) [2025] Ygg One
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
#![allow(clippy::module_name_repetitions)]

use crate::tokenizer::chunk::YamlChunkState;
use crate::tokenizer::stage2::{YamlBuffer, YamlIndentInfo, YamlParserState};
use crate::util::{
    add_cols_unchecked, add_rows_unchecked, fast_select_high_bits, fast_select_low_bits,
};
use crate::{EvenOrOddBits, YamlCharacterChunk, YamlDoubleQuoteChunk, YamlSingleQuoteChunk};
use alloc::vec::Vec;
use simdutf8::basic::imp::ChunkedUtf8Validator;
use EvenOrOddBits::OddBits;

pub(crate) type NextFn<B> = for<'buffer, 'input> unsafe fn(
    chunk: &'buffer [u8; 64],
    buffers: &'input mut B,
    state: &'input mut YamlParserState,
) -> YamlChunkState;

/// A trait representing a stage 1 scanner for parsing `YAML` input.
///
/// This trait provides methods for validating and scanning chunks of data, and finding important
/// parts like structural starts and so on.
///
/// # Safety
///
/// This trait MUST ALWAYS return valid positions in given stream in bytes. They will be used for unchecked
/// access to the underlying bytes.
///
/// # Dyn compatibility
///
/// This trait isn't `dyn` compatible.
pub unsafe trait Stage1Scanner {
    /// Type [`Stage1Scanner`] uses to perform SIMD accelerated actions.
    type SimdType;

    /// [`ChunkedUtf8Validator`] that matches the [`Stage1Scanner`] architecture.
    type Validator: ChunkedUtf8Validator;

    /// Returns the [`Self::Validator`] for the given trait implementor.
    ///
    /// The `validator` function is a generic method that returns the validator for the type it is called on.
    ///
    /// # Safety
    /// Method implementers need to make sure they are calling the right implementation for the correct architecture.
    unsafe fn validator() -> impl ChunkedUtf8Validator;

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
    /// let data_chunk: [u8; 64] = [0; 64];
    /// let result = NativeScanner::from_chunk(&data_chunk);
    /// ```
    ///
    /// # Returns
    ///
    /// A new instance of [`Stage1Scanner`] constructed from the given `values`.
    fn from_chunk(data_chunk: &[u8; 64]) -> Self;

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
    /// ```rust
    /// use yam_dark_core::{Stage1Scanner, SIMD_CHUNK_LENGTH};
    /// use yam_dark_core::NativeScanner;
    ///
    /// let values: [u8; 64] = [0; 64];
    /// let result = NativeScanner::from_chunk(&values);
    /// let bitmask = result.cmp_ascii_to_input(1);
    /// assert_eq!(bitmask, 0);
    /// ```
    fn cmp_ascii_to_input(&self, m: u8) -> u64;

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
    fn unsigned_lteq_against_splat(&self, cmp: u8) -> u64;

    /// Scans the whitespace and structurals in the given YAML chunk state.
    /// This method sets [`YamlCharacterChunk`] part of [`YamlChunkState`].
    ///
    /// # Returns: A mutable reference to the [`YamlChunkState`] for scanning.
    ///
    /// # Example
    /// ```rust
    ///  use yam_dark_core::{NativeScanner, Stage1Scanner, YamlChunkState, YamlParserState};
    ///  let mut prev_iter_state = YamlParserState::default();
    ///  let chunk = b" -                                                              ";
    ///  let scanner = NativeScanner::from_chunk(chunk);
    ///  let characters = scanner.classify_yaml_characters();
    ///  let expected = 0b000000000000000000000000000000000000000000000000000000000010;
    ///  assert_eq!(
    ///     characters.block_structurals,
    ///     expected,
    ///     "Expected:    {:#066b} \nGot instead: {:#066b} ",
    ///     expected, characters.block_structurals
    ///  );
    /// ```
    fn classify_yaml_characters(&self) -> YamlCharacterChunk;

    /// Combines all structurals and pseudo structurals into a single flat structure and stores it
    /// in [`YamlParserState::structurals`]. For every entry in `structurals` there will be
    /// corresponding fields in called `cols`, `rows` and `indents`.
    ///
    /// # Arguments:
    ///
    /// * `chunk` - A [`YamlChunkState`] that contains the current chunk info.
    /// * `parser_state` - A mutable [`YamlParserState`] that contains the current parser state and will be
    ///   updated with a new chunk state.
    /// * `indent_info` - A mutable [`YamlIndentInfo`] that will be updated with info.
    ///
    fn flatten_bits_yaml(
        chunk: &YamlChunkState,
        parser_state: &mut YamlParserState,
        indent_info: &mut YamlIndentInfo,
    );

    /// Calculates rows and cols part of the [`YamlIndentInfo`]
    ///
    /// Arguments:
    ///
    /// * `chunk` - A u64 bitmask that contains position of Line feed characters.
    /// * `parser_state` - A [`YamlParserState`] which will update `last_row` and `last_col` with the
    ///   chunk info.
    /// * `info` - A [`YamlIndentInfo`] that will be updated from chunk with `cols`/`rows`/`rows_indent_mask`.
    fn calculate_row_col_info(
        line_feeds: u64,
        parser_state: &mut YamlParserState,
        info: &mut YamlIndentInfo,
    ) {
        // Avoid copy/paste with this inline macro
        macro_rules! add_cols_rows_unchecked {
            ($e:expr) => {
                let nl_ind = ((line_feeds >> $e) & 0xFF) as usize;
                unsafe {
                    add_rows_unchecked(&mut info.rows, nl_ind, &mut parser_state.last_row, parser_state.pos + $e);
                    add_cols_unchecked(&mut info.cols, nl_ind, &mut parser_state.last_col, parser_state.pos + $e);
                };
            };
        }
        info.row_indent_mask = parser_state.last_row & 63;

        add_cols_rows_unchecked!(0);
        add_cols_rows_unchecked!(8);
        add_cols_rows_unchecked!(16);
        add_cols_rows_unchecked!(24);
        add_cols_rows_unchecked!(32);
        add_cols_rows_unchecked!(40);
        add_cols_rows_unchecked!(48);
        add_cols_rows_unchecked!(56);
    }

    /// Calculates `indents` part of [`YamlIndentInfo`] based on previous newlines and spaces position
    ///
    /// # Arguments:
    ///
    /// * `characters` - Current [`YamlChunkState`], from which relative indents are calculated.
    /// * `parser_state` - [`YamlParserState`] being updated with indent, and related data.
    /// * `info` - Current [`YamlIndentInfo`] being updated with indent, and related data.
    fn calculate_relative_indents(
        chunk: &YamlChunkState,
        parser_state: &mut YamlParserState,
        info: &mut YamlIndentInfo,
    ) {
        let mut neg_indents_mask = fast_select_high_bits(
            chunk.characters.spaces,
            (chunk.characters.line_feeds << 1) ^ u64::from(parser_state.is_indent_running),
        );
        neg_indents_mask &= !(neg_indents_mask >> 1);

        if neg_indents_mask == 0 {
            return;
        }

        let count = neg_indents_mask.count_ones();
        let last_bit = (neg_indents_mask | chunk.characters.line_feeds) & (1 << 63) != 0;

        let mut compressed_indents: Vec<u32> = Vec::with_capacity(64);
        let mut i = 0;

        parser_state.is_indent_running = last_bit;

        while neg_indents_mask != 0 {
            let part0 = neg_indents_mask.trailing_zeros();
            neg_indents_mask &= neg_indents_mask.saturating_sub(1);

            let part1 = neg_indents_mask.trailing_zeros();
            neg_indents_mask &= neg_indents_mask.saturating_sub(1);

            let part2 = neg_indents_mask.trailing_zeros();
            neg_indents_mask &= neg_indents_mask.saturating_sub(1);

            let part3 = neg_indents_mask.trailing_zeros();
            neg_indents_mask &= neg_indents_mask.saturating_sub(1);

            let v = [part0, part1, part2, part3];

            // SAFETY:
            // We need to maintain guarantee safety of `ptr::write` (*dst must be valid
            // and properly aligned) and `ptr::add` safety (*dst pointer must be in bounds,
            // count must not overflow `isize` and must not overflow `usize`).
            //
            // Invariants:
            // If compressed_indent is a `Vec<u32>` that's 64 elements wide
            // - `ptr::write` is aligned and ptr is valid
            // - `ptr::add` can't overflow isize or usize because it's only adding 64 elements.
            // - `ptr::add` pointer is valid
            unsafe {
                core::ptr::write(compressed_indents.as_mut_ptr().add(i).cast::<[u32; 4]>(), v);
            };
            i += 4;
        }

        // SAFETY:
        // We need to maintain guarantee safety of `vec::set_len` (`new_len <= capacity`
        // and elements `old_len..new_len` must be initialized) and `slice::get_unchecked_mut`
        // safety (index must not be out of bounds).
        //
        // Invariants:
        // - If `count` < 64 then `new_len <= capacity` must hold
        // - Since loop initializes more chunks than length, there `old_len..new_len` will be true
        // - loop runs once which compressed_indent will have one first element, which is guaranteed
        // by if neg_indents_mask == 0 early return.
        unsafe {
            // Snip the size of compressed only interesting ones
            compressed_indents.set_len(count as usize);
            // First indent of compressed will always take the previous chunk into account
            *compressed_indents.get_unchecked_mut(0) += parser_state.last_indent;
        }

        for index in 0..64 {
            // SAFETY:
            // Block will be safe if `get_unchecked_mut`/`get_unchecked` index is within bounds.
            //
            // Invariants:
            // - `info.rows.get_unchecked` must be safe because index goes from 0..<64
            // - `row_pos` must be less than 63 because a `info.row_indent_mask` is applied.
            // - `info.indents.get_unchecked_mut` access must be safe as long as row_pos is below 0..<64
            // - `compressed_indents.get_unchecked` must be safe because row_pos is below 0..<64
            unsafe {
                let row_pos = *info.rows.get_unchecked(index) & info.row_indent_mask;
                *info.indents.get_unchecked_mut(index) =
                    *compressed_indents.get_unchecked(row_pos as usize);
            }
        }

        // SAFETY:
        // Block is safe if `get_unchecked` is within bounds of `compressed_indents`
        //
        // Invariants:
        // - safe as long as `count.saturating_sub(1)` is 0..<64 (the size of `compressed_indents`,
        // count can be 64 (on 64-bit platforms)
        unsafe {
            parser_state.last_indent = compressed_indents
                .get_unchecked(count.saturating_sub(1) as usize)
                * u32::from(last_bit);
        }
    }

    /// Computes a quote mask based on the given quote bit mask.
    ///
    /// The `compute_quote_mask` function takes an input `quote_bits` of type `u64` and calculates
    /// a quote mask. The quote mask is a bitmask that has a binary 1 in every position where the
    /// corresponding byte is `"` (keep in mind that binary representation is big endian, while array
    /// representation is little endian).
    ///
    /// # Arguments
    ///
    /// * `quote_bits` - The quote bits of a type `u64` that specify the positions to be masked.
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
    #[must_use]
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
    /// * `buffers` - A mutable reference to a `buffers`
    ///   object implementing the [`YamlBuffer`] trait.
    /// * `prev_state` -
    ///   A mutable reference to a [`YamlParserState`] object that stores previous iteration state information.
    ///
    /// # Returns
    ///
    /// Returns the Result that returns an error if it encounters a parse error or [`YamlChunkState`].
    /// [`YamlChunkState`] stores current iteration information and is merged on each [`Stage1Scanner::next`]
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn next<T: YamlBuffer>(
        chunk: &[u8; 64],
        buffers: &mut T,
        prev_iter_state: &mut YamlParserState,
    ) -> YamlChunkState
    where
        Self: Sized,
    {
        let mut simd = Self::from_chunk(chunk);

        let mut characters = simd.classify_yaml_characters();

        // Pre-requisite
        // LINE FEED needs to be gathered before calling `calculate_indents`/`scan_for_comments`/
        // `scan_for_double_quote_bitmask`/`scan_single_quote_bitmask`
        simd.scan_for_comments(&mut characters, prev_iter_state);

        let double_quotes = simd.scan_double_quote_bitmask(prev_iter_state);
        let single_quotes = simd.scan_single_quote_bitmask(prev_iter_state);

        YamlChunkState::new_from_parts(single_quotes, double_quotes, characters)
    }

    /// This function processes the comments for the current chunk of characters.
    ///
    /// It takes a mutable reference to current [`chunk_state`](YamlChunkState) containing the current chunk data (like spaces and line feeds, etc.)
    /// and a mutable reference to a [`parser_state`](YamlParserState) which tracks parser state.
    ///
    /// # Arguments
    ///
    /// * `chunk` - A mutable reference to a [`YamlChunkState`] object that contains current chunk data.
    /// * `parser_state` - A mutable reference to a [`YamlParserState`] object that stores parser's state information.
    ///
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn scan_for_comments(
        &self,
        chunk: &mut YamlCharacterChunk,
        parser_state: &mut YamlParserState,
    ) {
        let character = self.cmp_ascii_to_input(b'#');
        let shifted_spaces = (chunk.spaces << 1) ^ u64::from(parser_state.is_previous_white_space);

        let comment_start = (character & shifted_spaces) | u64::from(parser_state.is_in_comment);
        let not_whitespace = !chunk.line_feeds;

        chunk.in_comment = fast_select_high_bits(not_whitespace, comment_start);

        // Update values for the next iteration.
        parser_state.is_in_comment = chunk.in_comment >> 63 == 1;
        parser_state.is_previous_white_space = (chunk.spaces >> 63) == 1;
    }

    /// Returns a bitmask indicating where there are characters that end an odd-length sequence
    /// of ones.
    ///
    /// The `prev_iteration_result` reference parameter is also updated to indicate whether the iteration
    /// needs to be taken into account by a later search.
    ///
    /// # Arguments
    ///
    /// * `prev_iteration_result` - A mutable reference to a `u64` representing the previous
    ///   iteration's result of backslashes. It will be updated with post-result info.
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
        // push a zero bit as a potential end
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
    /// * `block_state`: A mutable reference to a current [`YamlChunkState`].
    ///   It will update the
    ///   [`YamlSingleQuoteChunk`] with data for scanned single quotes.
    /// * `prev_iter_state`: A mutable reference to previous iteration [`YamlParserState`].
    ///
    /// # Example
    ///
    /// ```rust
    ///  use yam_dark_core::{NativeScanner, Stage1Scanner, YamlChunkState, YamlParserState};
    ///
    ///  let mut prev_iter_state = YamlParserState::default();
    ///
    ///  let chunk = b" ' ''  '                                                        ";
    ///  let scanner = NativeScanner::from_chunk(chunk);
    ///  let single_quote = scanner.scan_single_quote_bitmask(&mut prev_iter_state);
    ///  let expected = 0b0000000000000000000000000000000000000000000000000000010000010;
    ///  assert_eq!(single_quote.quote_bits, expected, "Expected:    {:#066b} \nGot instead: {:#066b} ", expected, single_quote.quote_bits);
    /// ```
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn scan_single_quote_bitmask(
        &self,
        prev_iter_state: &mut YamlParserState,
    ) -> YamlSingleQuoteChunk {
        let mut single_quote = YamlSingleQuoteChunk::default();

        let quotes = self.cmp_ascii_to_input(b'\'');

        let even_ends = Self::scan_for_mask(
            quotes,
            &mut prev_iter_state.is_prev_iter_odd_single_quote,
            EvenOrOddBits::EvenBits,
        );

        let exclude_even = fast_select_low_bits(quotes, even_ends >> 1);

        let odd_quotes = quotes ^ exclude_even;
        let odd_starts = odd_quotes & !(odd_quotes << 1);
        let odd_ends = odd_quotes & !(odd_quotes >> 1);

        let odd_end_mask = Self::compute_quote_mask(odd_ends);
        let odd_start_mask = Self::compute_quote_mask(odd_starts);
        let in_string = odd_end_mask | odd_start_mask;

        let quote_starts = in_string & !(in_string << 1);
        let quote_ends = in_string & !(in_string >> 1);

        single_quote.in_string = in_string;
        single_quote.quote_bits = quote_starts | quote_ends << 1;
        single_quote.quote_starts = quote_starts;
        single_quote.escaped_quotes = quotes ^ single_quote.quote_bits;

        single_quote
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
    #[must_use]
    fn calculate_mask_from_end(quote_bits: u64, even_ends: u64) -> u64 {
        fast_select_low_bits(quote_bits, even_ends)
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
    ///  let mut prev_iter_state = YamlParserState::default();
    ///  let chunk = b" \"  \"                                                           ";
    ///  let scanner = NativeScanner::from_chunk(chunk);
    ///  let result = scanner.scan_double_quote_bitmask(&mut prev_iter_state);
    ///  let expected = 0b000000000000000000000000000000000000000000000000000000010010;
    ///  assert_eq!(result.quote_bits, expected)
    /// ```
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn scan_double_quote_bitmask(
        &self,
        prev_iter_state: &mut YamlParserState,
    ) -> YamlDoubleQuoteChunk {
        let mut double_quote = YamlDoubleQuoteChunk::default();
        let prev_iteration_odd = &mut prev_iter_state.is_prev_double_quotes;
        let odds_ends =
            Self::scan_for_mask(self.cmp_ascii_to_input(b'\\'), prev_iteration_odd, OddBits);

        double_quote.quote_bits = self.cmp_ascii_to_input(b'"');
        double_quote.quote_bits &= !odds_ends;

        // remove from the valid quoted region the unescaped characters.
        let mut quote_mask: u64 = Self::compute_quote_mask(double_quote.quote_bits);
        quote_mask ^= prev_iter_state.prev_iter_inside_quote;

        // All Unicode characters may be placed within the
        // quotation marks, except for the characters that MUST be escaped:
        // quotation mark, reverse solidus, and the control characters (U+0000
        //through U+001F).
        // https://tools.ietf.org/html/rfc8259
        let unescaped: u64 = self.unsigned_lteq_against_splat(0x1F);
        double_quote.error_mask |= quote_mask & unescaped;
        // right shift of a signed value expected to be well-defined and standard
        // compliant as of C++20,
        // John Regher from Utah U. says this is fine code
        prev_iter_state.prev_iter_inside_quote = quote_mask >> 63;

        double_quote.in_string = quote_mask;
        double_quote.quote_starts = quote_mask & !(quote_mask << 1);
        double_quote.escaped = odds_ends;

        double_quote
    }
}

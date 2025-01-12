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
#![allow(clippy::module_name_repetitions)]

use alloc::vec::Vec;
use core::ptr::write;

use crate::tokenizer::chunk::YamlChunkState;
use crate::tokenizer::stage2::{Buffer, YamlIndentInfo, YamlParserState};
use crate::util::{
    add_cols_unchecked, add_rows_unchecked, calculate_byte_rows, calculate_cols,
    select_right_bits_branch_less, U8_BYTE_COL_TABLE, U8_ROW_TABLE,
};
use crate::{util, EvenOrOddBits};
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
    /// ```
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

    fn flatten_bits_yaml(
        base: &mut YamlParserState,
        yaml_chunk_state: &YamlChunkState,
        indent_info: &mut YamlIndentInfo,
    );

    #[deprecated]
    /// Calculates the indents of the given chunk and updates the `chunk_state` accordingly.
    ///
    /// For a chunk represented by this scanner, will calculate indents for each 64-character and
    /// will update `chunk_state`, taking into consideration previous indents in `prev_state`
    ///
    /// # Implementation
    ///
    /// It's important for implementation to first check where spaces `0x20` and line feed characters are located
    /// Since newline on Windows is `\r\n` Unicode `0x0A` and `0x0D` respectively we can approximate a newline with `\n`.
    /// Spaces are important because only ` `(code point `0x20`) is a valid YAML indentation mechanism.
    ///
    /// # Arguments
    ///
    /// - `chunk_state`: A mutable reference to a [`YamlChunkState`] that represents the YAML
    ///    chunk to calculate the indents for.
    /// - `prev_state`: A mutable reference to a [`YamlParserState`] that represents the previous
    ///    state of the YAML parser.
    ///
    /// # Examples
    /// ```rust
    /// use yam_dark_core::{u8x64_eq, NativeScanner, Stage1Scanner, YamlCharacterChunk, YamlChunkState, YamlParserState};
    ///
    /// let bin_str = b"                                                                ";
    /// let range1_to_64 = (0..=63).collect::<Vec<_>>();
    /// let scanner = NativeScanner::from_chunk(bin_str);
    ///
    /// // Needs to be called before calculate indent
    /// let line_feeds = u8x64_eq(bin_str, b'\n');
    /// let mut cols = vec![0; 64];
    /// let mut rows = vec![0; 64];
    /// // Will calculate col/row/indent
    /// NativeScanner::calculate_cols_rows(&mut cols, &mut rows, 0, line_feeds);
    /// assert_eq!(
    ///     cols,
    ///     range1_to_64
    /// );
    /// assert_eq!(
    ///     rows,
    ///     vec![0; 64]
    /// );
    /// ```
    fn calculate_cols_rows(cols: &mut [u32], rows: &mut [u32], idx: usize, line_feeds: u64) {
        let nl_ind = (line_feeds & 0xFF) as usize;

        let mut prev_col = 0;
        let mut prev_row = 0;

        rows[0..8].copy_from_slice(&calculate_byte_rows(nl_ind, &mut prev_row));
        cols[0..8].copy_from_slice(&calculate_cols(
            U8_BYTE_COL_TABLE[nl_ind],
            U8_ROW_TABLE[nl_ind],
            &prev_col,
        ));
        prev_col = cols[7] + 1;

        let nl_ind = ((line_feeds >> 8) & 0xFF) as usize;
        rows[8..16].copy_from_slice(&calculate_byte_rows(nl_ind, &mut prev_row));
        cols[8..16].copy_from_slice(&calculate_cols(
            U8_BYTE_COL_TABLE[nl_ind],
            U8_ROW_TABLE[nl_ind],
            &prev_col,
        ));
        prev_col = cols[15] + 1;

        let nl_ind = ((line_feeds >> 16) & 0xFF) as usize;
        rows[16..24].copy_from_slice(&calculate_byte_rows(nl_ind, &mut prev_row));
        cols[16..24].copy_from_slice(&calculate_cols(
            U8_BYTE_COL_TABLE[nl_ind],
            U8_ROW_TABLE[nl_ind],
            &prev_col,
        ));
        prev_col = cols[23] + 1;

        let nl_ind = ((line_feeds >> 24) & 0xFF) as usize;
        rows[24..32].copy_from_slice(&calculate_byte_rows(nl_ind, &mut prev_row));
        cols[24..32].copy_from_slice(&calculate_cols(
            U8_BYTE_COL_TABLE[nl_ind],
            U8_ROW_TABLE[nl_ind],
            &prev_col,
        ));
        prev_col = cols[31] + 1;

        let nl_ind = ((line_feeds >> 32) & 0xFF) as usize;
        rows[32..40].copy_from_slice(&calculate_byte_rows(nl_ind, &mut prev_row));
        cols[32..40].copy_from_slice(&calculate_cols(
            U8_BYTE_COL_TABLE[nl_ind],
            U8_ROW_TABLE[nl_ind],
            &prev_col,
        ));
        prev_col = cols[39] + 1;

        let nl_ind = ((line_feeds >> 40) & 0xFF) as usize;
        rows[40..48].copy_from_slice(&calculate_byte_rows(nl_ind, &mut prev_row));
        cols[40..48].copy_from_slice(&calculate_cols(
            U8_BYTE_COL_TABLE[nl_ind],
            U8_ROW_TABLE[nl_ind],
            &prev_col,
        ));
        prev_col = cols[47] + 1;

        let nl_ind = ((line_feeds >> 48) & 0xFF) as usize;
        rows[48..56].copy_from_slice(&calculate_byte_rows(nl_ind, &mut prev_row));
        cols[48..56].copy_from_slice(&calculate_cols(
            U8_BYTE_COL_TABLE[nl_ind],
            U8_ROW_TABLE[nl_ind],
            &prev_col,
        ));
        prev_col = cols[55] + 1;

        let nl_ind = ((line_feeds >> 56) & 0xFF) as usize;
        rows[56..64].copy_from_slice(&calculate_byte_rows(nl_ind, &mut prev_row));
        cols[56..64].copy_from_slice(&calculate_cols(
            U8_BYTE_COL_TABLE[nl_ind],
            U8_ROW_TABLE[nl_ind],
            &prev_col,
        ));
    }

    #[deprecated]
    fn calculate_indents(
        indents: &mut Vec<usize>,
        mut newline_mask: u64,
        space_mask: u64,
        is_indent_running: &mut bool,
    ) {
        let mut i = 0;
        let count_cols = (newline_mask.count_ones() + 1);
        let mut neg_indents_mask = select_right_bits_branch_less(
            space_mask,
            (newline_mask << 1) ^ u64::from(*is_indent_running),
        );
        let last_bit = (neg_indents_mask | newline_mask) & (1 << 63) != 0;
        indents.reserve(68);
        // To calculate indent we need to:
        // 1. Count trailing ones in space_mask this is the current indent
        // 2. Count the trailing zeros in newline mask to know how long the line is
        // 3. Check to see if the indent is equal to how much we need to1 shift it, if true we set mask to 1 otherwise to 0.
        while newline_mask != 0 {
            let part0 = neg_indents_mask.trailing_ones() & 127;
            let v0 = newline_mask.trailing_zeros() + 1;
            newline_mask = newline_mask.overflowing_shr(v0).0;
            neg_indents_mask = neg_indents_mask.overflowing_shr(v0).0 | 1 << 63;

            let part1 = neg_indents_mask.trailing_ones() & 127;
            let v1 = newline_mask.trailing_zeros() + 1;
            newline_mask = newline_mask.overflowing_shr(v1).0;
            neg_indents_mask = neg_indents_mask.overflowing_shr(v1).0 | 1 << 63;

            let part2 = neg_indents_mask.trailing_ones() & 127;
            let v2 = newline_mask.trailing_zeros() + 1;
            newline_mask = newline_mask.overflowing_shr(v2).0;
            neg_indents_mask = neg_indents_mask.overflowing_shr(v2).0 | 1 << 63;

            let part3 = neg_indents_mask.trailing_ones() & 127;
            let v3 = newline_mask.trailing_zeros() + 1;
            newline_mask = newline_mask.overflowing_shr(v3).0;
            neg_indents_mask = neg_indents_mask.overflowing_shr(v3).0 | 1 << 63;

            let v = [
                part0 as usize,
                part1 as usize,
                part2 as usize,
                part3 as usize,
            ];
            unsafe {
                write(indents.as_mut_ptr().add(i).cast::<[usize; 4]>(), v);
            }
            i += 4;
        }
        unsafe {
            indents.set_len(count_cols as usize);
        }
        *is_indent_running = last_bit;
    }

    fn calculate_positions_vectorized(
        state: &mut YamlParserState,
        chunk_state: &YamlChunkState,
        info: &mut YamlIndentInfo,
    ) {
        Self::calculate_indent_info_vectorized(state, chunk_state, info);
        Self::calculate_indents_vectorized(state, chunk_state, info);
    }

    fn calculate_indent_info_vectorized(
        state: &mut YamlParserState,
        chunk_state: &YamlChunkState,
        info: &mut YamlIndentInfo,
    ) {
        let nl_ind = (chunk_state.characters.line_feeds & 0xFF) as usize;
        unsafe {
            add_rows_unchecked(&mut state.byte_rows, nl_ind, &mut state.last_row, state.pos);
            add_cols_unchecked(&mut state.byte_cols, nl_ind, &mut state.last_col, state.pos);
        }

        let nl_ind = ((chunk_state.characters.line_feeds >> 8) & 0xFF) as usize;
        unsafe {
            add_rows_unchecked(
                &mut state.byte_rows,
                nl_ind,
                &mut state.last_row,
                state.pos + 8,
            );
            add_cols_unchecked(
                &mut state.byte_cols,
                nl_ind,
                &mut state.last_col,
                state.pos + 8,
            );
        }

        let nl_ind = ((chunk_state.characters.line_feeds >> 16) & 0xFF) as usize;
        unsafe {
            add_rows_unchecked(
                &mut state.byte_rows,
                nl_ind,
                &mut state.last_row,
                state.pos + 16,
            );
            add_cols_unchecked(
                &mut state.byte_cols,
                nl_ind,
                &mut state.last_col,
                state.pos + 16,
            );
        }

        let nl_ind = ((chunk_state.characters.line_feeds >> 24) & 0xFF) as usize;
        unsafe {
            add_rows_unchecked(
                &mut state.byte_rows,
                nl_ind,
                &mut state.last_row,
                state.pos + 24,
            );
            add_cols_unchecked(
                &mut state.byte_cols,
                nl_ind,
                &mut state.last_col,
                state.pos + 24,
            );
        }

        let nl_ind = ((chunk_state.characters.line_feeds >> 32) & 0xFF) as usize;
        unsafe {
            add_rows_unchecked(
                &mut state.byte_rows,
                nl_ind,
                &mut state.last_row,
                state.pos + 32,
            );
            add_cols_unchecked(
                &mut state.byte_cols,
                nl_ind,
                &mut state.last_col,
                state.pos + 32,
            );
        }

        let nl_ind = ((chunk_state.characters.line_feeds >> 40) & 0xFF) as usize;
        unsafe {
            add_rows_unchecked(
                &mut state.byte_rows,
                nl_ind,
                &mut state.last_row,
                state.pos + 40,
            );
            add_cols_unchecked(
                &mut state.byte_cols,
                nl_ind,
                &mut state.last_col,
                state.pos + 40,
            );
        }

        let nl_ind = ((chunk_state.characters.line_feeds >> 48) & 0xFF) as usize;
        unsafe {
            add_rows_unchecked(
                &mut state.byte_rows,
                nl_ind,
                &mut state.last_row,
                state.pos + 48,
            );
            add_cols_unchecked(
                &mut state.byte_cols,
                nl_ind,
                &mut state.last_col,
                state.pos + 48,
            );
        }

        let nl_ind = ((chunk_state.characters.line_feeds >> 56) & 0xFF) as usize;
        unsafe {
            add_rows_unchecked(
                &mut state.byte_rows,
                nl_ind,
                &mut state.last_row,
                state.pos + 56,
            );
            add_cols_unchecked(
                &mut state.byte_cols,
                nl_ind,
                &mut state.last_col,
                state.pos + 56,
            );
        }

        state.pos += 64;
    }

    fn calculate_indents_vectorized(
        state: &mut YamlParserState,
        chunk_state: &YamlChunkState,
        info: &mut YamlIndentInfo,
    ) {
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
        prev_iter_state: &mut YamlParserState,
    ) -> YamlChunkState
    where
        Self: Sized,
    {
        let mut chunk_state = YamlChunkState::default();
        let mut simd = Self::from_chunk(chunk);

        simd.classify_yaml_characters(&mut chunk_state);

        // Pre-requisite
        // LINE FEED needs to be gathered before calling `calculate_indents`/`scan_for_comments`/
        // `scan_for_double_quote_bitmask`/`scan_single_quote_bitmask`
        simd.scan_for_comments(&mut chunk_state, prev_iter_state);

        simd.scan_double_quote_bitmask(&mut chunk_state, prev_iter_state);
        simd.scan_single_quote_bitmask(&mut chunk_state, prev_iter_state);

        chunk_state
    }

    /// This function processes the comments for current chunk of characters.
    ///
    /// It takes a mutable reference to current [`chunk_state`](YamlChunkState) containing the current chunk data (like spaces and line feeds, etc.)
    /// and a  mutable reference to a [`parser_state`](YamlParserState) which tracks parser state.
    ///
    /// # Arguments
    ///
    /// * `chunk_state` - A mutable reference to a [`YamlChunkState`] object that contains current chunk data.
    /// * `parser_state` - A mutable reference to a [`YamlParserState`] object that stores parser's state information.
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
            select_right_bits_branch_less(not_whitespace, comment_start);

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
    ///   [`YamlSingleQuoteChunk`] with data for scanned single quotes.
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
    #[must_use]
    fn calculate_mask_from_end(quote_bits: u64, even_ends: u64) -> u64 {
        util::select_left_bits_branch_less(quote_bits, even_ends)
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
    ///  let expected = 0b000000000000000000000000000000000000000000000000000000010010;
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
        prev_iter_state.prev_iter_inside_quote = quote_mask >> 63;
        chunk_state.double_quote.quote_bits = quote_mask;
    }
}

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

use simdutf8::basic::imp::ChunkedUtf8Validator;

use crate::tokenizer::stage2::{Buffer, YamlParserState};
use crate::ParseResult;

#[derive(Default)]
pub struct YamlChunkState {
    double_quote: YamlDoubleQuoteChunk,
    single_quote: YamlSingleQuoteChunk,
    characters: YamlCharacterChunk,
    follows_non_quote_scalar: u64,
    error_mask: u64,
}

#[derive(Default)]
pub struct YamlDoubleQuoteChunk {
    /// Escaped characters
    escaped: u64,
    /// Real double quotes
    quote_bits: u64,
    /// String characters
    in_string: u64,
}

#[derive(Default)]
pub struct YamlSingleQuoteChunk {
    /// Real single quotes
    quote: u64,
    /// String characters
    in_string: u64,
}

#[derive(Default)]
pub struct YamlCharacterChunk {
    /// Space bitmask
    pub(crate) spaces: u64,
    /// Newline bitmask
    pub(crate) newline: u64,
    /// Operators
    pub(crate) op: u64,
}

pub(crate) type NextFn<B> = for<'buffer, 'input> unsafe fn(
    chunk: &'buffer [u8; 64],
    buffers: &'input mut B,
    state: &'input mut YamlParserState,
) -> ParseResult<YamlChunkState>;

const EVEN_BITS: u64 = 0x5555_5555_5555_5555;
const ODD_BITS: u64 = !EVEN_BITS;

pub trait Stage1Scanner {
    type SimdType;
    type Validator: ChunkedUtf8Validator;

    fn validator() -> Self::Validator;

    fn from_chunk(values: &[u8; 64]) -> Self;

    fn cmp_ascii_to_input(&self, m: u8) -> u64;

    fn leading_spaces(&self, spaces: &YamlCharacterChunk) -> (u32, u32);

    fn compute_quote_mask(quote_bits: u64) -> u64;

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
    /// * `prev_state` - A mutable reference to a [YamlParserState] object that stores previous iteration state information.
    ///
    /// # Returns
    ///
    /// Returns the Result that returns an error if it encounters a parse error or [YamlChunkState].
    /// [YamlChunkState] stores current iteration information and is merged on each [Stage1Scanner::next]
    fn next<T: Buffer>(
        chunk: &[u8; 64],
        buffers: &mut T,
        prev_state: &mut YamlParserState,
    ) -> ParseResult<YamlChunkState>
    where
        Self: Sized,
    {
        let mut block = YamlChunkState::default();
        let mut simd = Self::from_chunk(chunk);
        let double_quotes = simd.cmp_ascii_to_input(b'"');

        simd.scan_whitespace_and_structurals(&mut block);
        simd.scan_double_quote_bitmask(&mut block, prev_state);
        simd.scan_single_quote_bitmask(&mut block, prev_state);

        prev_state.merge_state(chunk, buffers, &mut block)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
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
    fn scan_for_odd_backslashes(&self, prev_iteration_odd: &mut u64) -> u64 {
        let backslash_bits = self.cmp_ascii_to_input(b'\\');
        let start_edges = backslash_bits & !(backslash_bits << 1);

        // flip lowest if we have an odd-length run at the end of the prior iteration
        let even_start_mask = EVEN_BITS ^ *prev_iteration_odd;
        let even_starts = start_edges & even_start_mask;
        let odd_starts = start_edges & !even_start_mask;
        let even_carries = backslash_bits.wrapping_add(even_starts);

        // must record the carry-out of our odd-carries out of bit 63; this
        // indicates whether the sense of any edge going to the next iteration
        // should be flipped
        let (mut odd_carries, iter_ends_odd_backslash) = backslash_bits.overflowing_add(odd_starts);

        odd_carries |= *prev_iteration_odd;
        // push in a bit zero as a potential end
        // if we had an odd-numbered run at the
        // end of the previous iteration
        *prev_iteration_odd = u64::from(iter_ends_odd_backslash);
        let even_carry_ends = even_carries & !backslash_bits;
        let odd_carry_ends = odd_carries & !backslash_bits;
        let even_start_odd_end = even_carry_ends & ODD_BITS;
        let odd_start_even_end = odd_carry_ends & EVEN_BITS;
        even_start_odd_end | odd_start_even_end
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn scan_single_quote_bitmask(
        &self,
        block_state: &mut YamlChunkState,
        prev_iter_state: &mut YamlParserState,
    ) {
        let quote_bits = self.cmp_ascii_to_input(b'\'');

        let even_start_mask = EVEN_BITS ^ (prev_iter_state.prev_iter_odd_quote as u64);
        let odd_bit = quote_bits & !even_start_mask;
        let even_bit = quote_bits & even_start_mask;

        let shift_even = even_bit << 1;
        let odd_starts = shift_even ^ odd_bit;
        block_state.single_quote.quote = odd_starts;
        prev_iter_state.prev_iter_odd_quote = Self::count_odd_bits(odd_starts);
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn scan_whitespace_and_structurals(&self, block_state: &mut YamlChunkState) {
        block_state.characters.spaces = self.cmp_ascii_to_input(b' ');
        block_state.characters.newline = self.cmp_ascii_to_input(b'\n');
        todo!()
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn scan_double_quote_bitmask(
        &self,
        block_state: &mut YamlChunkState,
        prev_iter_state: &mut YamlParserState,
    ) {
        let odds_ends = self.scan_for_odd_backslashes(&mut prev_iter_state.prev_iter_odd_backslash);

        block_state.double_quote.quote_bits = self.cmp_ascii_to_input(b'"');
        block_state.double_quote.quote_bits &= !odds_ends;

        // remove from the valid quoted region the unescaped characters.
        let mut quote_mask: u64 = Self::compute_quote_mask(block_state.double_quote.quote_bits);
        quote_mask ^= prev_iter_state.prev_iter_inside_quote;

        // All Unicode characters may be placed within the
        // quotation marks, except for the characters that MUST be escaped:
        // quotation mark, reverse solidus, and the control characters (U+0000
        //through U+001F).
        // https://tools.ietf.org/html/rfc8259
        let unescaped: u64 = self.unsigned_lteq_against_splat(0x1F);
        block_state.error_mask |= quote_mask & unescaped;
        // right shift of a signed value expected to be well-defined and standard
        // compliant as of C++20,
        // John Regher from Utah U. says this is fine code
        prev_iter_state.prev_iter_inside_quote = unsafe {
            core::mem::transmute::<_, u64>(core::mem::transmute::<_, i64>(quote_mask) >> 63)
        };
        block_state.double_quote.quote_bits = quote_mask;
    }
}

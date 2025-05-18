//! Various utility methods that are straightforward to auto-vectorize.

use alloc::collections::VecDeque;
use alloc::format;
use alloc::string::{String, ToString};
use core::cmp::max;
use core::fmt::Write;
use core::{mem, ptr};
use simdutf8::basic::imp::ChunkedUtf8Validator;

pub(crate) use chunked_iter::ChunkyIterator;
pub use native::{mask_merge, u8x16_swizzle, u8x64_eq, u8x64_lteq, U8X16};
pub use table::{U8_BYTE_COL_TABLE, U8_ROW_TABLE};

mod chunked_iter;
mod native;
mod table;

#[doc(hidden)]
pub struct NoopValidator();

impl ChunkedUtf8Validator for NoopValidator {
    unsafe fn new() -> Self
    where
        Self: Sized,
    {
        NoopValidator()
    }

    unsafe fn update_from_chunks(&mut self, _input: &[u8]) {}

    unsafe fn finalize(
        self,
        _remaining_input: Option<&[u8]>,
    ) -> Result<(), simdutf8::basic::Utf8Error> {
        Ok(())
    }
}

#[cfg(test)]
/// Used for tests
pub(crate) fn str_to_chunk(s: &str) -> [u8; 64] {
    let mut chunk = [b' '; 64];
    chunk[0..s.len()].copy_from_slice(s.as_bytes());
    chunk
}

/// Selects bits from the input according to the specified mask, using a branch-less approach.
///
/// This function takes two `u64` values as input: `input` and `mask`. It selects a sequence of 1-bits from
/// `input` if the leftmost (largest) bit in the mask corresponds to a bit in the mask. It essentially
/// selects all groups bits left of a 1-bit in mask.
///   
///
/// # Parameters
///
/// * `input`: The input `u64` value from which bits will be selected.
/// * `mask`: The mask `u64` value that determines which bits in the `input` will be selected.
///
/// # Returns
///
/// A `u64` value with the selected bits from the input as specified by the mask.
///
/// # Example
///
/// ```rust
/// let input = 0b1100_1100;
/// let mask  = 0b1010_1010;
/// let result = yam_dark_core::util::fast_select_low_bits(input, mask);
/// assert_eq!(result, 0b1100_1100);
/// ```
#[doc(hidden)]
#[cfg_attr(not(feature = "no-inline"), inline)]
#[must_use]
pub fn fast_select_low_bits(input: u64, mask: u64) -> u64 {
    let mut result = 0;

    result |= input & mask;

    let mut a = input & 0x7FFF_FFFF_FFFF_FFFF;
    result |= (result >> 1) & a;

    a &= a << 1;
    result |= ((result >> 1) & a) >> 1;

    a &= a << 2;
    result |= ((result >> 1) & a) >> 3;

    a &= a << 4;
    result |= ((result >> 1) & a) >> 7;

    a &= a << 8;
    result |= ((result >> 1) & a) >> 15;

    a &= a << 16;
    result |= ((result >> 1) & a) >> 31;

    result
}

/// Selects bits from the input according to the specified mask, using a branch-less approach.
///
/// This function takes two `u64` values as input: `input` and `mask`. It selects a sequence of 1-bits from
/// `input` if the rightmost (smallest) bit in the mask corresponds to a bit in the mask. It essentially
/// selects all groups bits right of a 1-bit in mask.
///
/// # Parameters
///
/// * `input`: The input `u64` value from which bits will be selected.
/// *  `mask`: The mask `u64` value that determines which bits in the `input` will be selected.
///
/// # Returns
///
/// A `u64` value with the selected bits from the input as specified by the mask.
///
/// # Example
///
/// ```rust
/// let input = 0b1100_1110;
/// let mask  = 0b0100_0100;
/// let result = yam_dark_core::util::fast_select_high_bits(input, mask);
/// assert_eq!(result, 0b1100_1100);
/// ```
#[doc(hidden)]
#[cfg_attr(not(feature = "no-inline"), inline)]
#[must_use]
pub fn fast_select_high_bits(input: u64, mask: u64) -> u64 {
    input & (mask | !input.wrapping_add(input & mask))
}
#[doc(hidden)]
#[allow(unused)]
#[must_use]
pub fn canonical_select_high_bits(input: u64, mask: u64) -> u64 {
    let mut result = input & mask;

    let mut a = input;
    result |= (result << 1) & a;

    a &= a << 1;
    result |= (result << 2) & a;

    a &= a << 2;
    result |= (result << 4) & a;

    a &= a << 4;
    result |= (result << 8) & a;

    a &= a << 8;
    result |= (result << 16) & a;

    a &= a << 16;
    result |= (result << 32) & a;

    result
}

#[doc(hidden)]
#[inline]
pub fn calculate_byte_rows(index_mask: usize, prev_row: &mut u32) -> [u32; 8] {
    let pre_calc_row = U8_ROW_TABLE[index_mask];
    let rows = [
        *prev_row,
        *prev_row + u32::from(pre_calc_row[0]),
        *prev_row + u32::from(pre_calc_row[1]),
        *prev_row + u32::from(pre_calc_row[2]),
        *prev_row + u32::from(pre_calc_row[3]),
        *prev_row + u32::from(pre_calc_row[4]),
        *prev_row + u32::from(pre_calc_row[5]),
        *prev_row + u32::from(pre_calc_row[6]),
    ];
    *prev_row += u32::from(pre_calc_row[7]);
    rows
}

#[doc(hidden)]
/// Utility function that for given a `newlines` mask will calculate eight rows at once
///
/// The function uses particular access and format to achieve auto-vectorization even without any
/// SIMD enhancements.
///
/// # Arguments:
/// * `dst` - An array or vector to which the row data will be written. It's expected to be at
///   least `idx + 8` long.
/// * `newlines` - Bit mask of an 8-bit chunk that determines which precomputed hash we should use.
/// * `prev_row` - Value of the previous row, which tells us how much to adjust the row value.
///    After running, it will be updated to reflect the newest row.
/// * `idx` - Index offset
///
/// # Safety:
/// * This function is safe assuming that `U8_ROW_TABLE` must be correct (entries less than 8).
/// * That `dst` must be at least `idx + 8` long.
pub unsafe fn add_rows_unchecked(dst: &mut [u32], newlines: usize, prev_row: &mut u32, idx: usize) {
    let src = U8_ROW_TABLE[newlines];
    *dst.get_unchecked_mut(idx) = *prev_row;
    *dst.get_unchecked_mut(idx + 1) = u32::from(*src.get_unchecked(0)) + *prev_row;
    *dst.get_unchecked_mut(idx + 2) = u32::from(*src.get_unchecked(1)) + *prev_row;
    *dst.get_unchecked_mut(idx + 3) = u32::from(*src.get_unchecked(2)) + *prev_row;
    *dst.get_unchecked_mut(idx + 4) = u32::from(*src.get_unchecked(3)) + *prev_row;
    *dst.get_unchecked_mut(idx + 5) = u32::from(*src.get_unchecked(4)) + *prev_row;
    *dst.get_unchecked_mut(idx + 6) = u32::from(*src.get_unchecked(5)) + *prev_row;
    *dst.get_unchecked_mut(idx + 7) = u32::from(*src.get_unchecked(6)) + *prev_row;
    *prev_row += *dst.get_unchecked(idx + 7);
}

#[doc(hidden)]
/// Utility function that for a given ` newlines ` mask will calculate eight cols at once
///
/// The function uses particular access and format to achieve auto-vectorization even without any
/// SIMD enhancements.
///
/// # Arguments:
/// * `dst` - An array or vector to which the row data will be written. It's expected to be at
///   least `idx + 8` long.
/// * `newlines` - Bit mask of an 8-bit chunk that determines which precomputed hash we should use.
/// * `prev_col` - Value of the previous column, which tells us how much to adjust the column value.
///    After running, it will be updated to reflect the newest column.
/// * `idx` - Index offset
///
/// # Safety:
/// * This function is safe assuming that `U8_ROW_TABLE` and `U8_BYTE_COL_TABLE` must be correct (entries less than 8).
/// * That `dst` must be at least `idx + 8` long.
pub unsafe fn add_cols_unchecked(dst: &mut [u32], newlines: usize, prev_col: &mut u32, idx: usize) {
    let cols = U8_BYTE_COL_TABLE[newlines];
    let rows = U8_ROW_TABLE[newlines];
    let cols_calc = calculate_cols(cols, rows, prev_col);

    ptr::copy_nonoverlapping(cols_calc.as_ptr(), dst.as_mut_ptr().add(idx), 8);
    *prev_col = cols_calc[7] + 1;
}

#[doc(hidden)]
#[inline]
#[must_use]
pub fn calculate_cols(cols: [u8; 8], rows: [u8; 8], prev_col: &u32) -> [u32; 8] {
    [
        u32::from(cols[0]) + *prev_col,
        if rows[0] == 0 {
            u32::from(cols[1]) + *prev_col
        } else {
            u32::from(cols[1])
        },
        if rows[1] == 0 {
            u32::from(cols[2]) + *prev_col
        } else {
            u32::from(cols[2])
        },
        if rows[2] == 0 {
            u32::from(cols[3]) + *prev_col
        } else {
            u32::from(cols[3])
        },
        if rows[3] == 0 {
            u32::from(cols[4]) + *prev_col
        } else {
            u32::from(cols[4])
        },
        if rows[4] == 0 {
            u32::from(cols[5]) + *prev_col
        } else {
            u32::from(cols[5])
        },
        if rows[5] == 0 {
            u32::from(cols[6]) + *prev_col
        } else {
            u32::from(cols[6])
        },
        if rows[6] == 0 {
            u32::from(cols[7]) + *prev_col
        } else {
            u32::from(cols[7])
        },
    ]
}

#[must_use]
#[allow(unused)]
#[doc(hidden)]
/// Pretty print diff between two u64
///
/// Prints difference between two `u64`. Separates numbers into four-bit chunks, printed to
/// the highest 1-bit.
///
/// # Arguments
/// * `left` - left number for comparison.
/// * `right` - right number for comparison.
///
/// # Panics
/// - If it fails `from_utf8` conversion.
pub fn print_bin_diff(left: u64, right: u64) -> String {
    let mut buf = String::new();

    let max_len = usize::try_from(max(left, right).ilog2() / 4 + 1)
        .unwrap_or_else(|_| panic!("Expected log2 of {left} or {right} to fit in pointer size"));

    let left_str = print_bin_till(left, max_len);
    let right_str = print_bin_till(right, max_len);

    write!(buf, "Expected:\n{left_str}\nActual:\n{right_str}",).expect("Can't write to buffer");
    buf
}

#[doc(hidden)]
#[must_use]
pub fn print_bin_till(number: u64, max: usize) -> String {
    let number_str = format!("{number:b}");
    let mut double_buf = VecDeque::with_capacity(128);
    let mut reverse_str_chunker = number_str.as_bytes().rchunks(4);
    let mut chunk = [b'0'; 4];

    for i in 0..max {
        if let Some(rev) = reverse_str_chunker.next() {
            let len = rev.len();
            for i in 0..len {
                chunk[i] = rev[len - i - 1];
            }
        }
        let temp = mem::replace(&mut chunk, *b"0000");
        double_buf.push_front(temp[0]);
        double_buf.push_front(temp[1]);
        double_buf.push_front(temp[2]);
        double_buf.push_front(temp[3]);

        if i == max - 1 {
            continue;
        }
        double_buf.push_front(b' ');
    }
    let buf = String::from_utf8(double_buf.make_contiguous().to_vec()).unwrap();
    buf.to_string()
}

/// Asserts that two `u64` are binary equal to each other.
///
/// On panic, this macro will print the quartet of bits, separated by
/// a whitespace character, and printed to the highest common 1-bit.
///
/// # Examples
/// ```
/// use yam_dark_core::assert_bin_eq;
/// let a = 3;
/// let b = 1 + 2;
///
/// assert_bin_eq!(a, b);
/// // If it was assert_bin_eq!(3, 5);
/// // The output would be
/// // Expected:
/// // 0011
/// // Actual:
/// // 0101
/// ```
#[macro_export]
macro_rules! assert_bin_eq {
    ($left:expr, $right:expr) => {
        match (&$left, &$right) {
            (left_val, right_val) => {
                use $crate::util::print_bin_diff;
                if !(*left_val == *right_val) {
                    panic!("{}", print_bin_diff(*left_val, *right_val));
                }
            }
        }
    };
}

#[cfg(test)]
mod test {
    use crate::util::{fast_select_high_bits, fast_select_low_bits};
    use rstest::rstest;

    #[rstest]
    #[case(
        0b1111_0000_1110_0000_0110,
        0b1000_0010_0000_0000_0100,
        0b1111_0000_0000_0000_0110
    )]
    #[case(0b1100_1100, 0b1010_1010, 0b1100_1100)]
    #[case(1434, 272, 0b1_1001_1000)]
    #[case(1434, 0, 0)]
    fn test_select_low(#[case] input: u64, #[case] mask: u64, #[case] expected: u64) {
        let actual = fast_select_low_bits(input, mask);
        assert_bin_eq!(expected, actual);
    }

    #[rstest]
    #[case(
        0b1110_0000_0000_0000_0000_0000_0000_1110_0000_0000_0000_0000_0000_0000_1110_0110,
        0b0110_0010_0000_0000_0000_1100_0000_0000_0000_0000_0000_0000_0000_0000_0100_0010,
        0b1110_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1100_0110
    )]
    #[case(
        0b1110_0000_0000_0000_0000_0000_0000_1110_0000_0000_0000_0000_0000_0000_1111_0110,
        0b1110_0010_0000_0000_0000_1100_0000_0000_0000_0000_0000_0000_0000_0000_0101_0010,
        0b1110_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1111_0110
    )]
    #[case(0b1111_1110, 0b0100_0100, 0b1111_1100)]
    #[case(0b1111, 0b1101, 0b0000_1111)]
    #[case(1434, 0, 0)]
    fn test_select_high(#[case] input: u64, #[case] mask: u64, #[case] expected: u64) {
        let actual = fast_select_high_bits(input, mask);
        assert_bin_eq!(actual, expected);
    }
}

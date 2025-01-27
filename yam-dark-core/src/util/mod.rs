use alloc::vec::Vec;
use core::ptr;
use core::ptr::write;
use simdutf8::basic::imp::ChunkedUtf8Validator;

pub(crate) use chunked_iter::ChunkyIterator;
pub use native::U8X8;
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

/// Selects bits from the input according to the specified mask, using a branch-less approach.
///
/// This function takes two `u64` values as input: `input` and `mask`. It selects sequence of ones from
/// `input` if the leftmost (largest) bit in mask corresponds to a bit in mask. It essentially
/// selects all groups bits left of a 1-bit in mask.
///   
///
/// # Parameters
///
/// - `input`: The input `u64` value from which bits will be selected.
/// - `mask`:  The mask `u64` value that determines which bits in the `input` will be selected.
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
/// let result = yam_dark_core::util::select_left_bits_branch_less(input, mask);
/// assert_eq!(result, 0b1100_1100);
/// ```
#[doc(hidden)]
#[cfg_attr(not(feature = "no-inline"), inline)]
#[must_use]
pub fn select_left_bits_branch_less(input: u64, mask: u64) -> u64 {
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
/// This function takes two `u64` values as input: `input` and `mask`. It selects sequence of ones from
/// `input` if the rightmost (smallest) bit in mask corresponds to a bit in mask. It essentially
/// selects all groups bits right of a 1-bit in mask.
///
/// # Parameters
///
/// - `input`: The input `u64` value from which bits will be selected.
/// - `mask`:  The mask `u64` value that determines which bits in the `input` will be selected.
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
/// let result = yam_dark_core::util::select_right_bits_branch_less(input, mask);
/// assert_eq!(result, 0b1100_1100);
/// ```
#[doc(hidden)]
#[cfg_attr(not(feature = "no-inline"), inline)]
#[must_use]
pub fn select_right_bits_branch_less(input: u64, mask: u64) -> u64 {
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
        *prev_row + pre_calc_row[0] as u32,
        *prev_row + pre_calc_row[1] as u32,
        *prev_row + pre_calc_row[2] as u32,
        *prev_row + pre_calc_row[3] as u32,
        *prev_row + pre_calc_row[4] as u32,
        *prev_row + pre_calc_row[5] as u32,
        *prev_row + pre_calc_row[6] as u32,
    ];
    *prev_row += pre_calc_row[7] as u32;
    rows
}

pub unsafe fn add_rows_unchecked(dst: &mut [u32], newlines: usize, prev_row: &mut u32, idx: usize) {
    let src = U8_ROW_TABLE[newlines];
    *dst.get_unchecked_mut(idx) = *prev_row;
    *dst.get_unchecked_mut(idx + 1) = *src.get_unchecked(0) as u32 + *prev_row;
    *dst.get_unchecked_mut(idx + 2) = *src.get_unchecked(1) as u32 + *prev_row;
    *dst.get_unchecked_mut(idx + 3) = *src.get_unchecked(2) as u32 + *prev_row;
    *dst.get_unchecked_mut(idx + 4) = *src.get_unchecked(3) as u32 + *prev_row;
    *dst.get_unchecked_mut(idx + 5) = *src.get_unchecked(4) as u32 + *prev_row;
    *dst.get_unchecked_mut(idx + 6) = *src.get_unchecked(5) as u32 + *prev_row;
    *dst.get_unchecked_mut(idx + 7) = *src.get_unchecked(6) as u32 + *prev_row;
    *prev_row += *dst.get_unchecked(idx + 7)
}

pub unsafe fn compress(src: &[u32; 8], k1: &[bool; 8], dst: &mut [u32; 8]) {
    let mut k = 0;

    let x0 = k1.get_unchecked(0);
    *dst.get_unchecked_mut(k) = *src.get_unchecked(0) * *x0 as u32;
    k += *x0 as usize;

    let x1 = k1.get_unchecked(1);
    *dst.get_unchecked_mut(k) = *src.get_unchecked(1) * *x1 as u32;
    k += *x1 as usize;

    let x2 = k1.get_unchecked(2);
    *dst.get_unchecked_mut(k) = *src.get_unchecked(2) * *x2 as u32;
    k += *x2 as usize;

    let x3 = k1.get_unchecked(3);
    *dst.get_unchecked_mut(k) = *src.get_unchecked(3) * *x3 as u32;
    k += *x3 as usize;

    let x4 = k1.get_unchecked(4);
    *dst.get_unchecked_mut(k) = *src.get_unchecked(4) * *x4 as u32;
    k += *x4 as usize;

    let x5 = k1.get_unchecked(5);
    *dst.get_unchecked_mut(k) = *src.get_unchecked(5) * *x5 as u32;
    k += *x5 as usize;

    let x6 = k1.get_unchecked(6);
    *dst.get_unchecked_mut(k) = *src.get_unchecked(6) * *x6 as u32;
    k += *x6 as usize;

    let x7 = k1.get_unchecked(7);
    *dst.get_unchecked_mut(k) = *src.get_unchecked(7) * *x7 as u32;
}

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

#[test]
fn test_branch_less_right1() {
    let actual = select_left_bits_branch_less(
        0b1111_0000_0000_0000_0000_0000_0000_1110_0000_0000_0000_0000_0000_0000_0000_0110,
        0b1000_0010_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0100,
    );
    let expected =
        0b1111_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0110;
    assert_eq!(
        actual, expected,
        "\nExpected: {:#018b}\n  Actual: {:#018b}",
        expected, actual
    );
}

#[test]
fn test_branch_less_right2() {
    let actual = select_left_bits_branch_less(0b1100_1100, 0b1010_1010);
    let expected = 0b1100_1100;
    assert_eq!(
        actual, expected,
        "\nExpected: {:#018b}\n  Actual: {:#018b}",
        expected, actual
    );
}

#[test]
fn test_branch_less_left() {
    let actual = select_right_bits_branch_less(
        0b1110_0000_0000_0000_0000_0000_0000_1110_0000_0000_0000_0000_0000_0000_1110_0110,
        0b0010_0010_0000_0000_0000_1100_0000_0000_0000_0000_0000_0000_0000_0000_0100_0010,
    );

    let expected =
        0b1110_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1100_0110;
    assert_eq!(
        actual, expected,
        "\nExpected: {:#066b}\n  Actual: {:#066b}",
        expected, actual
    );
}

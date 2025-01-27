use simdutf8::basic::imp::ChunkedUtf8Validator;

pub(crate) use chunked_iter::ChunkyIterator;
pub use native::U8X8;
pub use native::{mask_merge, u8x16_swizzle, u8x64_eq, u8x64_lteq, U8X16};
pub use table::{INDENT_SWIZZLE_TABLE, U8_BYTE_COL_TABLE, U8_ROW_TABLE};

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
pub fn calculate_byte_rows(index_mask: usize, prev_row: &mut u8) -> [u8; 8] {
    let pre_calc_row = U8_ROW_TABLE[index_mask];
    let rows = [
        *prev_row,
        *prev_row + pre_calc_row[0],
        *prev_row + pre_calc_row[1],
        *prev_row + pre_calc_row[2],
        *prev_row + pre_calc_row[3],
        *prev_row + pre_calc_row[4],
        *prev_row + pre_calc_row[5],
        *prev_row + pre_calc_row[6],
    ];
    *prev_row += pre_calc_row[7];
    rows
}

#[doc(hidden)]
#[inline]
#[must_use]
pub fn calculate_cols(cols: [u8; 8], rows_data: [u8; 8], prev_col: &u8) -> [u8; 8] {
    [
        cols[0] + *prev_col,
        if rows_data[0] == 0 {
            cols[1] + *prev_col
        } else {
            cols[1]
        },
        if rows_data[1] == 0 {
            cols[2] + *prev_col
        } else {
            cols[2]
        },
        if rows_data[2] == 0 {
            cols[3] + *prev_col
        } else {
            cols[3]
        },
        if rows_data[3] == 0 {
            cols[4] + *prev_col
        } else {
            cols[4]
        },
        if rows_data[4] == 0 {
            cols[5] + *prev_col
        } else {
            cols[5]
        },
        if rows_data[5] == 0 {
            cols[6] + *prev_col
        } else {
            cols[6]
        },
        if rows_data[6] == 0 {
            cols[7] + *prev_col
        } else {
            cols[7]
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

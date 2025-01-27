use alloc::vec::Vec;
use core::ptr::write;
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

#[doc(hidden)]
#[cfg_attr(not(feature = "no-inline"), inline)]
pub fn select_right_bits_branch_less(input: u64, mask: u64) -> u64 {
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

#[doc(hidden)]
#[cfg_attr(not(feature = "no-inline"), inline)]
pub fn select_left_bits_branch_less(input: u64, mask: u64) -> u64 {
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

#[doc(hidden)]
pub fn count_indent_native(mut newline_mask: u64, mut space_mask: u64, indents: &mut Vec<u32>) {
    let mut base_len = indents.len();
    indents.reserve(64);

    while newline_mask != 0 {
        let v0 = newline_mask.trailing_zeros() + 1;
        newline_mask &= newline_mask.wrapping_sub(1);
        let part0 = space_mask % (1 << v0);
        space_mask >>= v0;

        let v1 = newline_mask.trailing_zeros() + 1;
        newline_mask &= newline_mask.wrapping_sub(1);
        let part1 = space_mask % (1 << v1);
        space_mask >>= v1;

        let v2 = newline_mask.trailing_zeros() + 1;
        newline_mask &= newline_mask.wrapping_sub(1);
        let part2 = space_mask % (1 << v2);
        space_mask >>= v2;

        let v3 = newline_mask.trailing_zeros() + 1;
        newline_mask &= newline_mask.wrapping_sub(1);
        let part3 = space_mask % (1 << v3);
        space_mask >>= v3;

        let v = [part0 as u32, part1 as u32, part2 as u32, part3 as u32];
        unsafe { write(indents.as_mut_ptr().add(base_len).cast::<[u32; 4]>(), v) }
        base_len += 4;
    }
}

#[test]
fn test_branch_less_right() {
    let actual = select_right_bits_branch_less(
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
fn test_branch_less_left() {
    let actual = select_left_bits_branch_less(
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

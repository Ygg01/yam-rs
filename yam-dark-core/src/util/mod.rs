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

fn from_part_indent(part_indent: u32) -> (u32, bool) {
    let is_frozen = (part_indent & 0x1) != 0;
    let indent = part_indent >> 1;
    (indent, is_frozen)
}

fn into_part_indent(indent: u32, is_running: bool) -> u32 {
    assert!(indent <= (1 << 31));
    let frozen_bit = is_running as u32;
    (indent << 1) | frozen_bit
}

#[doc(hidden)]
pub fn count_indent_native(
    mut newline_mask: u64,
    space_mask: u64,
    indents: &mut Vec<u32>,
    is_running: bool,
    previous_indent: &mut u32,
) {
    let start_len = indents.len();
    let mut i = 0;

    // Reserve enough space for the worst case since it can have
    indents.reserve(68);
    let count_cols = newline_mask.count_ones() + 1;
    let mut runners = Vec::<bool>::with_capacity(count_cols as usize);
    let mut neg_indents_mask = !select_left_bits_branch_less(space_mask, (newline_mask << 1) ^ (is_running as u64));

    // To calculate indent we need to:
    // 1. Count trailing ones in space_mask this is the current indent
    // 2. Count the trailing zeros in newline mask to know how long the line is
    // 3. Check to see if the indent is equal to how much we need to1 shift it, if true we set mask to 1 otherwise to 0.
    // 4. when returning indent, the 32nd bit will represent the if the indent is still running or if it has stopped
    while newline_mask != 0 {
        let part0 = neg_indents_mask.trailing_zeros();
        let v0 = newline_mask.trailing_zeros() + 1;
        newline_mask = newline_mask.overflowing_shr(v0).0;
        neg_indents_mask = neg_indents_mask.overflowing_shr(v0).0;

        let part1 = neg_indents_mask.trailing_zeros();
        let v1 = newline_mask.trailing_zeros() + 1;
        newline_mask = newline_mask.overflowing_shr(v1).0;
        neg_indents_mask = neg_indents_mask.overflowing_shr(v1).0;

        let part2 = neg_indents_mask.trailing_zeros();
        let v2 = newline_mask.trailing_zeros() + 1;
        newline_mask = newline_mask.overflowing_shr(v2).0;
        neg_indents_mask = neg_indents_mask.overflowing_shr(v2).0;

        let part3 = neg_indents_mask.trailing_zeros();
        let v3 = newline_mask.trailing_zeros() + 1;
        newline_mask = newline_mask.overflowing_shr(v3).0;
        neg_indents_mask = neg_indents_mask.overflowing_shr(v3).0;

        let v = [part0, part1, part2, part3];
        let running = [part0 == v0, part1 == v1, part2 == v2, part3 == v3];
        unsafe { 
            write(indents.as_mut_ptr().add(i+ start_len).cast::<[u32; 4]>(), v);
            write(runners.as_mut_ptr().add(i).cast::<[bool; 4]>(), running); 
 
        }
        i += 4;
    }
    // We do some safety vector snipping here, then handle previous indent.

    let last_len = start_len + count_cols as usize;

    // SAFETY: we have reserved enough space, but we will only use start_len + number of newlines + 1
    // or start_len + count_cols
    unsafe {
        indents.set_len(last_len);
    }
    if *previous_indent > 0 {
        // SAFETY: start_len is starting length and since indents are guaranteed to have at least 1
        // element, we can safely access the element at start_len
        unsafe {
            *indents.get_unchecked_mut(start_len) = *previous_indent - 1;
        }
    }
    // SAFETY: last element should be exactly last_len - 1 (because arrays are 0 based)
    *previous_indent = unsafe {
        // TODO do actual indent logic here
        // TODO check what happens for 64 newlines
        *indents.get_unchecked(last_len - 1)
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

use simdutf8::basic::imp::ChunkedUtf8Validator;

pub(crate) use chunked_iter::ChunkyIterator;
pub use native::{mask_merge, u8x16_swizzle, u8x64_eq, u8x64_lteq, U8X16};
pub use native::{mask_merge_u8x8, U8X8};
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

#[inline]
fn calculate_byte_col(index_mask: usize, reset_bool: bool, prev_indent: &mut u32) -> [u32; 8] {
    let byte_col1 = U8_BYTE_COL_TABLE[index_mask];
    let rows1 = U8_ROW_TABLE[index_mask];
    let row_calc = crate::util::calculate_cols(byte_col1, rows1, prev_indent);
    let mask_sec = (-(reset_bool as i32)) as u32;
    *prev_indent = (row_calc[7] + 1) & mask_sec;
    row_calc
}

#[inline]
fn calculate_cols(cols: [u8; 8], rows_data: [u8; 8], prev_col: &mut u32) -> [u32; 8] {
    [
        cols[0] as u32 + *prev_col,
        if rows_data[0] == 0 {
            cols[1] as u32 + *prev_col
        } else {
            cols[1] as u32
        },
        if rows_data[1] == 0 {
            cols[2] as u32 + *prev_col
        } else {
            cols[2] as u32
        },
        if rows_data[2] == 0 {
            cols[3] as u32 + *prev_col
        } else {
            cols[3] as u32
        },
        if rows_data[3] == 0 {
            cols[4] as u32 + *prev_col
        } else {
            cols[4] as u32
        },
        if rows_data[4] == 0 {
            cols[5] as u32 + *prev_col
        } else {
            cols[5] as u32
        },
        if rows_data[5] == 0 {
            cols[6] as u32 + *prev_col
        } else {
            cols[6] as u32
        },
        if rows_data[6] == 0 {
            cols[7] as u32 + *prev_col
        } else {
            cols[7] as u32
        },
    ]
}

#[inline]
fn calculate_byte_rows(index_mask: usize, prev_row: &mut u32) -> [u32; 8] {
    let rows1 = U8_ROW_TABLE[index_mask];
    calculate_rows(rows1, prev_row)
}

#[inline]
fn calculate_rows(rows: [u8; 8], prev_row: &mut u32) -> [u32; 8] {
    let x = [
        *prev_row,
        *prev_row + rows[0] as u32,
        *prev_row + rows[1] as u32,
        *prev_row + rows[2] as u32,
        *prev_row + rows[3] as u32,
        *prev_row + rows[4] as u32,
        *prev_row + rows[5] as u32,
        *prev_row + rows[6] as u32,
    ];
    *prev_row += rows[7] as u32;
    x
}

#[doc(hidden)]
pub fn count_col_rows(
    newline_mask: u64,
    prev_byte_col: &mut u32,
    prev_byte_row: &mut u32,
    byte_cols: &mut [u32; 64],
    byte_rows: &mut [u32; 64],
) {
    // First 8 bits
    let mask = (newline_mask & 0xFF) as usize;
    let col_result = calculate_byte_col(mask, newline_mask & 0x80 == 0, prev_byte_col);
    byte_cols[0..8].copy_from_slice(&col_result);

    let rows_result = calculate_byte_rows(mask, prev_byte_row);
    byte_rows[0..8].copy_from_slice(&rows_result);

    // Second 8 bits
    let mask = ((newline_mask & 0xFF00) >> 8) as usize;
    let col_result = calculate_byte_col(mask, newline_mask & 0x8000 == 0, prev_byte_col);
    byte_cols[8..16].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, prev_byte_row);
    byte_rows[8..16].copy_from_slice(&col_rows);

    // Third 8 bits
    let mask = ((newline_mask & 0xFF_0000) >> 16) as usize;
    let col_result = calculate_byte_col(mask, newline_mask & 0x80_0000 == 0, prev_byte_col);
    byte_cols[16..24].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, prev_byte_row);
    byte_rows[16..24].copy_from_slice(&col_rows);

    // Fourth 8 bits
    let mask = ((newline_mask & 0xFF00_0000) >> 24) as usize;
    let col_result = calculate_byte_col(mask, newline_mask & 0x8000_0000 == 0, prev_byte_col);
    byte_cols[24..32].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, prev_byte_row);
    byte_rows[24..32].copy_from_slice(&col_rows);

    // Fifth 8 bits
    let mask = ((newline_mask & 0xFF_0000_0000) >> 32) as usize;
    let col_result = calculate_byte_col(mask, newline_mask & 0x80_0000_0000 == 0, prev_byte_col);
    byte_cols[32..40].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, prev_byte_row);
    byte_rows[32..40].copy_from_slice(&col_rows);

    // Sixth 8 bits
    let mask = ((newline_mask & 0xFF00_0000_0000) >> 40) as usize;
    let col_result = calculate_byte_col(mask, newline_mask & 0x8000_0000_0000 == 0, prev_byte_col);
    byte_cols[40..48].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, prev_byte_row);
    byte_rows[40..48].copy_from_slice(&col_rows);

    // Seventh 8 bits
    let mask = ((newline_mask & 0xFF_0000_0000_0000) >> 48) as usize;
    let col_result =
        calculate_byte_col(mask, newline_mask & 0x80_0000_0000_0000 == 0, prev_byte_col);
    byte_cols[48..56].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, prev_byte_row);
    byte_rows[48..56].copy_from_slice(&col_rows);

    // Eight 8 bits
    let mask = ((newline_mask & 0xFF00_0000_0000_0000) >> 56) as usize;
    let col_result = calculate_byte_col(
        mask,
        newline_mask & 0x8000_0000_0000_0000 == 0,
        prev_byte_col,
    );
    byte_cols[56..64].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, prev_byte_row);
    byte_rows[56..64].copy_from_slice(&col_rows);
}

#[test]
fn test_quick_count() {
    let mask = 0b10000010_00000000_00000000;
    let expected_value = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 0, 1, 2, 3, 4, 5,
    ];
    let mut prev_value = 0;
    let mut prev_rows = 0;

    let mut actual_cols = [0; 64];
    let mut actual_rows = [0; 64];
    count_col_rows(
        mask,
        &mut prev_value,
        &mut prev_rows,
        &mut actual_cols,
        &mut actual_rows,
    );
    assert_eq!(&actual_cols[0..24], &expected_value[0..24]);
    assert_eq!(prev_value, 40);
}

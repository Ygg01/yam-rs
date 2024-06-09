use simdutf8::basic::imp::ChunkedUtf8Validator;

pub(crate) use chunked_iter::ChunkyIterator;
pub use native::{mask_merge, U8X16, u8x16_swizzle, u8x64_eq, u8x64_lteq};
pub use native::{mask_merge_u8x8, U8X8};
pub use table::{U8_BYTE_COL_TABLE, U8_ROW_TABLE};

use crate::util::table::U8_INDENT_TABLE;

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
pub fn count_table_small(
    newline_mask: u64,
    space_mask: u64,
    prev_byte_col: &mut u32,
    prev_byte_row: &mut u32,
    prev_byte_indent: &mut i32,
    byte_cols: &mut [u32; 64],
    byte_rows: &mut [u32; 64],
    byte_indents: &mut [u32; 64],
) {
    #[inline]
    fn calculate_byte_col(index_mask: usize, reset_bool: bool, prev_indent: &mut u32) -> [u32; 8] {
        let byte_col1 = U8_BYTE_COL_TABLE[index_mask];
        let rows1 = U8_ROW_TABLE[index_mask];
        let row_calc = calculate_cols(byte_col1, rows1, prev_indent);
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

    fn calculate_indent(
        newline_mask: usize,
        white_space_mask: usize,
        last_index: &mut i32,
        col: [u32; 8],
    ) -> [u32; 8] {
        let indent_mask = (newline_mask ^ !white_space_mask) & 0xFF;
        let ind = U8_INDENT_TABLE[indent_mask];
        let col_ind = U8_BYTE_COL_TABLE[newline_mask];
        let indent = [
            col_ind[0] as usize as usize,
            col_ind[1] as usize as usize,
            col_ind[2] as usize as usize,
            col_ind[3] as usize,
            col_ind[4] as usize,
            col_ind[5] as usize,
            col_ind[6] as usize,
            col_ind[7] as usize,
        ];
        // let mut indent = col.clone();
        // for i in 0..8usize {
        //     if white_space_mask & (1 << i) == 0 {
        //         if *last_index == -1 {
        //             *last_index = i as i32;
        //         }
        //         indent[i] = *last_index as u32;
        //     }
        //     if newline_mask & (1 << i) == 0 {
        //         *last_index = -1;
        //     }
        // }

        [
            indent[0] as u32,
            indent[1] as u32,
            indent[2] as u32,
            indent[3] as u32,
            indent[4] as u32,
            indent[4] as u32,
            indent[4] as u32,
            indent[4] as u32,
        ]
    }
    // First 8 bits
    let mask = (newline_mask & 0xFF) as usize;
    let col_result = calculate_byte_col(mask, newline_mask & 0x80 == 0, prev_byte_col);
    byte_cols[0..8].copy_from_slice(&col_result);

    let rows_result = calculate_byte_rows(mask, prev_byte_row);
    byte_rows[0..8].copy_from_slice(&rows_result);

    let indent_mask = (space_mask & 0xFF) as usize;
    let indent = calculate_indent(indent_mask, mask, prev_byte_indent, col_result);
    byte_indents[0..8].copy_from_slice(&indent);

    // Second 8 bits
    let mask = ((newline_mask & 0xFF00) >> 8) as usize;
    let col_result = calculate_byte_col(mask, newline_mask & 0x8000 == 0, prev_byte_col);
    byte_cols[8..16].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, prev_byte_row);
    byte_rows[8..16].copy_from_slice(&col_rows);

    let indent_mask = ((space_mask & 0xFF00) >> 8) as usize;
    let indent = calculate_indent(indent_mask, mask, prev_byte_indent, col_result);
    byte_indents[8..16].copy_from_slice(&indent);

    // Third 8 bits
    let mask = ((newline_mask & 0xFF_0000) >> 16) as usize;
    let col_result = calculate_byte_col(mask, newline_mask & 0x80_0000 == 0, prev_byte_col);
    byte_cols[16..24].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, prev_byte_row);
    byte_rows[16..24].copy_from_slice(&col_rows);

    let indent_mask = ((space_mask & 0xFF_0000) >> 16) as usize;
    let indent = calculate_indent(indent_mask, mask, prev_byte_indent, col_result);
    byte_indents[8..16].copy_from_slice(&indent);

    // Fourth 8 bits
    let mask = ((newline_mask & 0xFF00_0000) >> 24) as usize;
    let col_result = calculate_byte_col(mask, newline_mask & 0x8000_0000 == 0, prev_byte_col);
    byte_cols[24..32].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, prev_byte_row);
    byte_rows[24..32].copy_from_slice(&col_rows);

    let indent_mask = ((space_mask & 0xFF00_0000) >> 24) as usize;
    let indent = calculate_indent(indent_mask, mask, prev_byte_indent, col_result);
    byte_indents[24..32].copy_from_slice(&indent);

    // Fifth 8 bits
    let mask = ((newline_mask & 0xFF_0000_0000) >> 32) as usize;
    let col_result = calculate_byte_col(mask, newline_mask & 0x80_0000_0000 == 0, prev_byte_col);
    byte_cols[32..40].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, prev_byte_row);
    byte_rows[32..40].copy_from_slice(&col_rows);

    let indent_mask = ((space_mask & 0xFF_0000_0000) >> 32) as usize;
    let indent = calculate_indent(indent_mask, mask, prev_byte_indent, col_result);
    byte_indents[32..40].copy_from_slice(&indent);

    // Sixth 8 bits
    let mask = ((newline_mask & 0xFF00_0000_0000) >> 40) as usize;
    let col_result = calculate_byte_col(mask, newline_mask & 0x8000_0000_0000 == 0, prev_byte_col);
    byte_cols[40..48].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, prev_byte_row);
    byte_rows[40..48].copy_from_slice(&col_rows);

    let indent_mask = ((space_mask & 0xFF00_0000_0000) >> 40) as usize;
    let indent = calculate_indent(indent_mask, mask, prev_byte_indent, col_result);
    byte_indents[40..48].copy_from_slice(&indent);

    // Seventh 8 bits
    let mask = ((newline_mask & 0xFF_0000_0000_0000) >> 48) as usize;
    let col_result =
        calculate_byte_col(mask, newline_mask & 0x80_0000_0000_0000 == 0, prev_byte_col);
    byte_cols[48..56].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, prev_byte_row);
    byte_rows[48..56].copy_from_slice(&col_rows);

    let indent_mask = ((space_mask & 0xFF_0000_0000_0000) >> 48) as usize;
    let indent = calculate_indent(indent_mask, mask, prev_byte_indent, col_result);
    byte_indents[48..56].copy_from_slice(&indent);

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

    let indent_mask = ((space_mask & 0xFF00_0000_0000_0000) >> 56) as usize;
    let indent = calculate_indent(indent_mask, mask, prev_byte_indent, col_result);
    byte_indents[56..64].copy_from_slice(&indent);
}

#[test]
fn test_quick_count() {
    let mask = 0b10000010_00000000_00000000;
    let space_mask = 0b0;
    let expected_value = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 0, 1, 2, 3, 4, 5,
    ];
    let mut prev_value = 0;
    let mut prev_indent = -1;
    let mut prev_rows = 0;

    let mut actual_cols = [0; 64];
    let mut actual_rows = [0; 64];
    let mut actual_indent = [0; 64];
    count_table_small(
        mask,
        space_mask,
        &mut prev_value,
        &mut prev_rows,
        &mut prev_indent,
        &mut actual_cols,
        &mut actual_rows,
        &mut actual_indent,
    );
    assert_eq!(&actual_cols[0..24], &expected_value[0..24]);
    assert_eq!(prev_value, 40);
}

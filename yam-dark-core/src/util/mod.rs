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

pub fn count_table_small(newline_mask: u64, prev_indent: &mut u32, byte_cols: &mut [u32; 64]) {
    #[inline]
    fn calculate_byte_col(index_mask: u64, reset_bool: bool, prev_indent: &mut u32) -> [u32; 8] {
        let byte_col1 = U8_BYTE_COL_TABLE[index_mask as usize];
        let rows1 = U8_ROW_TABLE[index_mask as usize];
        let row_calc = add_offset_and_mask(byte_col1, rows1, prev_indent);
        let mask_sec = (-(reset_bool as i32)) as u32;
        *prev_indent = (row_calc[7] + 1) & mask_sec;
        row_calc
    }

    let row_calc = calculate_byte_col(newline_mask & 0xFF, newline_mask & 0x80 == 0, prev_indent);
    byte_cols[0..8].copy_from_slice(&row_calc);

    let row_calc = calculate_byte_col(
        (newline_mask & 0xFF00) >> 8,
        newline_mask & 0x8000 == 0,
        prev_indent,
    );
    byte_cols[8..16].copy_from_slice(&row_calc);

    let row_calc = calculate_byte_col(
        (newline_mask & 0xFF_0000) >> 16,
        newline_mask & 0x80_0000 == 0,
        prev_indent,
    );
    byte_cols[16..24].copy_from_slice(&row_calc);

    let row_calc = calculate_byte_col(
        (newline_mask & 0xFF00_0000) >> 24,
        newline_mask & 0x8000_0000 == 0,
        prev_indent,
    );
    byte_cols[24..32].copy_from_slice(&row_calc);

    let row_calc = calculate_byte_col(
        (newline_mask & 0xFF_0000_0000) >> 32,
        newline_mask & 0x80_0000_0000 == 0,
        prev_indent,
    );
    byte_cols[32..40].copy_from_slice(&row_calc);

    let row_calc = calculate_byte_col(
        (newline_mask & 0xFF00_0000_0000) >> 40,
        newline_mask & 0x8000_0000_0000 == 0,
        prev_indent,
    );
    byte_cols[40..48].copy_from_slice(&row_calc);

    let row_calc = calculate_byte_col(
        (newline_mask & 0xFF_0000_0000_0000) >> 48,
        newline_mask & 0x80_0000_0000_0000 == 0,
        prev_indent,
    );
    byte_cols[48..56].copy_from_slice(&row_calc);

    let row_calc = calculate_byte_col(
        (newline_mask & 0xFF00_0000_0000_0000) >> 56,
        newline_mask & 0x8000_0000_0000_0000 == 0,
        prev_indent,
    );
    byte_cols[56..64].copy_from_slice(&row_calc);
}

pub fn add_offset_and_mask(x: [u8; 8], mask: [u8; 8], offset: &mut u32) -> [u32; 8] {
    [
        x[0] as u32 + *offset,
        if mask[0] == 0 {
            x[1] as u32 + *offset
        } else {
            x[1] as u32
        },
        if mask[1] == 0 {
            x[2] as u32 + *offset
        } else {
            x[2] as u32
        },
        if mask[2] == 0 {
            x[3] as u32 + *offset
        } else {
            x[3] as u32
        },
        if mask[3] == 0 {
            x[4] as u32 + *offset
        } else {
            x[4] as u32
        },
        if mask[4] == 0 {
            x[5] as u32 + *offset
        } else {
            x[5] as u32
        },
        if mask[5] == 0 {
            x[6] as u32 + *offset
        } else {
            x[6] as u32
        },
        if mask[6] == 0 {
            x[7] as u32 + *offset
        } else {
            x[7] as u32
        },
    ]
}

#[test]
fn test_quick_count() {
    let mask = 0b10000010_00000000_00000000;
    let expected_value = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 0, 1, 2, 3, 4, 5,
    ];
    let mut prev_value = 0;
    let mut actual_cols = [0; 64];
    count_table_small(mask, &mut prev_value, &mut actual_cols);
    assert_eq!(&actual_cols[0..24], &expected_value[0..24]);
    assert_eq!(prev_value, 40);
}

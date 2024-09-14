use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use yam_dark_core::util::{
    count_col_rows, count_indent_dependent, count_indent_naive, U8X8,
    U8_BYTE_COL_TABLE, U8_ROW_TABLE,
};
use yam_dark_core::{u8x64_eq, ChunkyIterator};

const YAML: &[u8] = r#"
   a: b                      
   c: b       

a  st
   d: a   as
                  
                  
                  
    a: a"#
    .as_bytes();

#[inline]
pub fn count_table_u8x8(chunk: [u8; 64], prev_col: &mut u32) -> [u32; 64] {
    let mut shift_mask = u8x64_eq(&chunk, b'\n');

    let mut result = [0; 64];
    for i in 0..8 {
        let ind = (shift_mask & 0x0000_0000_0000_00FF) as usize;
        let byte_col = U8X8::from_array(U8_BYTE_COL_TABLE[ind]);
        let rows = U8X8::from_array(U8_ROW_TABLE[ind]);
        let row_calc: [u32; 8] = byte_col.add_offset_and_mask(rows, *prev_col);
        result[i * 8..i * 8 + 8].copy_from_slice(&row_calc[..]);
        *prev_col = row_calc[7];
        shift_mask >>= 8;
    }

    result
}

fn col_count_indent(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-col");
    group.significance_level(0.05).sample_size(100);
    group.throughput(Throughput::Bytes(64 * 2));

    let mut chunk_iter = ChunkyIterator::from_bytes(YAML);
    let chunk = chunk_iter.next().unwrap();
    let newline_mask = u8x64_eq(chunk, b'\n');
    let space_mask = u8x64_eq(chunk, b' ');

    let chunk2 = chunk_iter.next().unwrap();
    let newline_mask2 = u8x64_eq(chunk2, b'\n');
    let space_mask2 = u8x64_eq(chunk2, b' ');
    let mut indents = [0; 64];
    let mut byte_cols = [0; 64];
    let mut byte_rows = [0; 64];

    group.bench_function("col_count_indent_dependent", |b| {
        b.iter(|| {
            let mut prev_indent = 0;
            let mut prev_iter_char = 1;
            let mut prev_row = 0;
            let mut prev_col = 0;

            count_col_rows(
                newline_mask,
                &mut prev_col,
                &mut prev_row,
                &mut byte_cols,
                &mut byte_rows,
            );
            count_indent_dependent(
                newline_mask,
                space_mask,
                &mut prev_iter_char,
                &mut prev_indent,
                &byte_cols,
                &mut indents,
            );
            black_box(indents[3] == 0);

            count_col_rows(
                newline_mask2,
                &mut prev_col,
                &mut prev_row,
                &mut byte_cols,
                &mut byte_rows,
            );
            count_indent_dependent(
                newline_mask2,
                space_mask2,
                &mut prev_iter_char,
                &mut prev_indent,
                &byte_cols,
                &mut indents,
            );
            black_box(indents[30] == 0);
        })
    });
    group.finish();
}

fn col_count_indent_naive(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-col");
    group.significance_level(0.05).sample_size(100);
    group.throughput(Throughput::Bytes(64 * 2));

    let mut chunk_iter = ChunkyIterator::from_bytes(YAML);
    let chunk = chunk_iter.next().unwrap();
    let newline_mask = u8x64_eq(chunk, b'\n');
    let space_mask = u8x64_eq(chunk, b' ');

    let chunk2 = chunk_iter.next().unwrap();
    let newline_mask2 = u8x64_eq(chunk2, b'\n');
    let space_mask2 = u8x64_eq(chunk2, b' ');
    let mut indents = [0; 64];
    let mut prev_col = 0;
    let mut prev_row = 0;
    let mut byte_cols = [0; 64];
    let mut byte_rows = [0; 64];

    group.bench_function("col_count_indent_naive", |b| {
        b.iter(|| {
            let mut prev_indent = 0;
            let mut prev_iter_char = 1;

            count_col_rows(
                newline_mask,
                &mut prev_col,
                &mut prev_row,
                &mut byte_cols,
                &mut byte_rows,
            );
            count_indent_naive(
                newline_mask,
                space_mask,
                &mut prev_iter_char,
                &mut prev_indent,
                &mut indents,
            );
            black_box(indents[56] == 0);

            count_col_rows(
                newline_mask2,
                &mut prev_col,
                &mut prev_row,
                &mut byte_cols,
                &mut byte_rows,
            );
            count_indent_naive(
                newline_mask2,
                space_mask2,
                &mut prev_iter_char,
                &mut prev_indent,
                &mut indents,
            );
            black_box(indents[60] == 0);
        })
    });
    group.finish();
}

fn count_naive(
    newline_bits: u64,
    space_bits: u64,
    prev_col: &mut u32,
    prev_row: &mut u32,
    prev_indent: &mut u32,
    is_indent_frozen: &mut bool,
    byte_cols: &mut [u32; 64],
    byte_rows: &mut [u32; 64],
    byte_indent: &mut [u32; 64],
) {
    let mut curr_row = *prev_row;
    let mut curr_col = *prev_col;
    let mut curr_indent = *prev_indent;
    for pos in 0..64 {
        let is_newline = newline_bits & (1 << pos) != 0;
        let is_space = space_bits & (1 << pos) != 0;

        if is_space && !*is_indent_frozen {
            curr_indent += 1;
        } else if !is_space && *is_indent_frozen {
            *is_indent_frozen = true;
        }

        if is_newline {
            unsafe {
                *byte_cols.get_unchecked_mut(pos) = curr_col + 1;
                *byte_rows.get_unchecked_mut(pos) = curr_row;
            }
            curr_col = 0;
            curr_indent = 0;
            curr_row += 1;
            *is_indent_frozen = false;
            continue;
        }

        curr_col += 1;
        unsafe {
            *byte_cols.get_unchecked_mut(pos) = curr_col;
            *byte_rows.get_unchecked_mut(pos) = curr_row;
            *byte_indent.get_unchecked_mut(pos) = curr_indent;
        }
    }
    *prev_indent = curr_indent;
    *prev_col = curr_col;
    *prev_row = curr_row;
}

fn col_count_all_naive(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-col-naive");
    group.significance_level(0.05).sample_size(100);
    group.throughput(Throughput::Bytes(64 * 2));

    let mut chunk_iter = ChunkyIterator::from_bytes(YAML);
    let chunk = chunk_iter.next().unwrap();
    let newline_mask = u8x64_eq(chunk, b'\n');
    let space_mask = u8x64_eq(chunk, b' ');

    let chunk2 = chunk_iter.next().unwrap();
    let newline_mask2 = u8x64_eq(chunk2, b'\n');
    let space_mask2 = u8x64_eq(chunk2, b' ');

    let mut indents = [0; 64];
    let mut prev_col = 0;
    let mut prev_row = 0;
    let mut is_indent_frozen = false;
    let mut byte_cols = [0; 64];
    let mut byte_rows = [0; 64];

    group.bench_function("col_naive", |b| {
        b.iter(|| {
            let mut prev_indent = 0;

            count_naive(
                newline_mask,
                space_mask,
                &mut prev_col,
                &mut prev_row,
                &mut prev_indent,
                &mut is_indent_frozen,
                &mut byte_cols,
                &mut byte_rows,
                &mut indents,
            );
            black_box(indents[56] == 0);


            count_naive(
                newline_mask2,
                space_mask2,
                &mut prev_col,
                &mut prev_row,
                &mut prev_indent,
                &mut is_indent_frozen,
                &mut byte_cols,
                &mut byte_rows,
                &mut indents,
            );
            black_box(byte_rows[3] == 0);
        })
    });
    group.finish();
}

criterion_group!(benches, col_count_indent, col_count_indent_naive, col_count_all_naive);
criterion_main!(benches);

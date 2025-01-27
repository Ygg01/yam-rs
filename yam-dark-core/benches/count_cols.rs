use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use yam_dark_core::util::{
    count_table_small, mask_merge, U8X16, U8X8, U8_BYTE_COL_TABLE, U8_ROW_TABLE,
};
use yam_dark_core::{u8x64_eq, ChunkyIterator};

const YAML: &[u8] = r#"
   a: b                      
   c: b       

a  st
   d: a   as
                  
                  
                  
  "#
.as_bytes();

pub fn count_cols(newline_mask: u64, prev_indent: &mut u32) -> [u32; 64] {
    let mut res = [0; 64];

    for offset in 0u8..64 {
        res[offset as usize] = *prev_indent;
        *prev_indent = if newline_mask & (1 << offset) == 0 {
            0
        } else {
            *prev_indent + 1
        };
    }

    res
}

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

#[inline]
pub fn count_table_u8x16(chunk: [u8; 64]) -> [u32; 64] {
    let mut prev_col = 0;
    let mut prev_row = 0;
    let v0: U8X16 = unsafe { U8X16::from_slice(&chunk[0..16]) };
    let v1 = unsafe { U8X16::from_slice(&chunk[16..32]) };
    let v2 = unsafe { U8X16::from_slice(&chunk[32..48]) };
    let v3 = unsafe { U8X16::from_slice(&chunk[48..64]) };

    let x0 = count_u8x16(v0, &mut prev_col, &mut prev_row);
    let x1 = count_u8x16(v1, &mut prev_col, &mut prev_row);
    let x2 = count_u8x16(v2, &mut prev_col, &mut prev_row);
    let x3 = count_u8x16(v3, &mut prev_col, &mut prev_row);

    mask_merge(x0, x1, x2, x3)
}

#[inline]
fn count_u8x16(vec: U8X16, _prev_col: &mut u8, prev_row: &mut u8) -> U8X16 {
    let bitmask = vec.comp_all(b'\n').to_bitmask();
    let high_ind = ((bitmask & 0xFF00) >> 8) as usize;
    let low_ind = (bitmask & 0xFF) as usize;

    let high_row = U8_ROW_TABLE[high_ind];
    let high_byte_col = U8_BYTE_COL_TABLE[high_ind];
    let low_row = U8_ROW_TABLE[low_ind];
    let low_byte_col = U8_BYTE_COL_TABLE[low_ind];

    let y = U8X16::from_merge_rows(low_row, high_row, bitmask, *prev_row);
    *prev_row = y.0[15];
    let col_bitmask = y.comp_all(y.0[7]).to_bitmask();
    let x = U8X16::from_merge_cols(low_byte_col, high_byte_col, col_bitmask);

    x
}

fn col_count_naive(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-col");
    group.significance_level(0.05).sample_size(100);
    group.throughput(Throughput::Bytes(64));

    let mut chunk_iter = ChunkyIterator::from_bytes(YAML);
    let chunk = chunk_iter.next().unwrap();
    let mask = u8x64_eq(chunk, b'\n');

    group.bench_function("col_count_naive", |b| {
        b.iter(|| {
            let mut prev_indent = 0;
            let count = count_cols(mask, &mut prev_indent);
            black_box(count[0] == 0);
        })
    });
    group.finish();
}

fn col_count_u8x8(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-col");
    group.significance_level(0.05).sample_size(100);
    group.throughput(Throughput::Bytes(64));

    let mut chunk_iter = ChunkyIterator::from_bytes(YAML);
    let chunk = chunk_iter.next().unwrap();

    group.bench_function("col_count_u8x8", |b| {
        b.iter(|| {
            let mut prev_indent = 0;
            let count = count_table_u8x8(*chunk, &mut prev_indent);
            black_box(count[0] > 0);
        })
    });
    group.finish();
}

fn col_count_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-col");
    group.significance_level(0.05).sample_size(100);
    group.throughput(Throughput::Bytes(64));

    let mut chunk_iter = ChunkyIterator::from_bytes(YAML);
    let chunk = chunk_iter.next().unwrap();
    let mask = u8x64_eq(chunk, b'\n');

    group.bench_function("col_count_small", |b| {
        b.iter(|| {
            let mut prev_indent = 0;
            let mut count = [0; 64];
            count_table_small(mask, &mut prev_indent, &mut count);
            black_box(count[0] > 0);
        })
    });
    group.finish();
}

fn col_count_u8x16(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-col");
    group.significance_level(0.05).sample_size(100);
    group.throughput(Throughput::Bytes(64));

    let mut chunk_iter = ChunkyIterator::from_bytes(YAML);
    let chunk = chunk_iter.next().unwrap();

    group.bench_function("col_count_u8x16", |b| {
        b.iter(|| {
            let count = count_table_u8x16(*chunk);
            black_box(count[0] > 0);
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    col_count_naive,
    col_count_u8x8,
    col_count_u8x16,
    col_count_small
);
criterion_main!(benches);

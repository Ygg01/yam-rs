use criterion::{black_box, Criterion, criterion_group, criterion_main, Throughput};

use yam_dark_core::{ChunkyIterator, u8x64_eq};
use yam_dark_core::util::{
    mask_merge, U8_BYTE_COL_TABLE, U8_ROW_TABLE, U8X16, U8X8,
};

const YAML: &[u8] = r#"
   a: b                      
   c: b       

a  st
   d: a   as
                  
                  
                  
  "#
.as_bytes();

fn count_subslice(sublice: &mut U8X16) {
    let mut shift_mask = sublice.comp_all(b'\n').to_bitmask();
    shift_mask = !(shift_mask << 1);

    let shift = sublice.shift_right(1);
    *sublice = shift.mask_value(shift_mask);

    let shift1 = sublice.shift_right(1).mask_value(shift_mask);
    *sublice += shift1;

    shift_mask &= shift_mask << 1;
    let shift2 = sublice.shift_right(2).mask_value(shift_mask);
    *sublice += shift2;

    shift_mask &= shift_mask << 2;
    let shift4 = sublice.shift_right(4).mask_value(shift_mask);
    *sublice += shift4;

    shift_mask &= shift_mask << 4;
    let shift8 = sublice.shift_right(8).mask_value(shift_mask);
    *sublice += shift8;
}

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

pub fn add_offset_and_mask(x: [u8; 8], mask: [u8; 8], offset: u32) -> [u32; 8] {
    [
        if mask[0] == 0 {
            x[0] as u32 + offset
        } else {
            x[0] as u32
        },
        if mask[1] == 0 {
            x[1] as u32 + offset
        } else {
            x[1] as u32
        },
        if mask[2] == 0 {
            x[2] as u32 + offset
        } else {
            x[2] as u32
        },
        if mask[3] == 0 {
            x[3] as u32 + offset
        } else {
            x[3] as u32
        },
        if mask[4] == 0 {
            x[4] as u32 + offset
        } else {
            x[4] as u32
        },
        if mask[5] == 0 {
            x[5] as u32 + offset
        } else {
            x[5] as u32
        },
        if mask[6] == 0 {
            x[6] as u32 + offset
        } else {
            x[6] as u32
        },
        if mask[7] == 0 {
            x[7] as u32 + offset
        } else {
            x[7] as u32
        },
    ]
}

pub fn count_table_small(newline_mask: u64, prev_indent: &mut u32) -> [u32; 64] {
    let mut res = [0; 64];

    let mask1 = (newline_mask & 0xFF) as usize;
    let byte_col1 = U8_BYTE_COL_TABLE[mask1];
    let rows1 = U8_ROW_TABLE[mask1];
    let row_calc = add_offset_and_mask(byte_col1, rows1, *prev_indent);
    *prev_indent = row_calc[7];
    res[0..8].copy_from_slice(&row_calc);

    let mask2 = ((newline_mask & 0xFF00) >> 8) as usize;
    let byte_col2 = U8_BYTE_COL_TABLE[mask2];
    let rows2 = U8_ROW_TABLE[mask2];
    let row_calc = add_offset_and_mask(byte_col2, rows2, *prev_indent);
    *prev_indent = row_calc[7];
    res[8..16].copy_from_slice(&row_calc);

    let mask3 = ((newline_mask & 0xFF0000) >> 16) as usize;
    let byte_col3 = U8_BYTE_COL_TABLE[mask3];
    let rows3 = U8_ROW_TABLE[mask3];
    let row_calc = add_offset_and_mask(byte_col3, rows3, *prev_indent);
    *prev_indent = row_calc[7];
    res[16..24].copy_from_slice(&row_calc);

    let mask4 = ((newline_mask & 0xFF00_0000) >> 24) as usize;
    let byte_col4 = U8_BYTE_COL_TABLE[mask4];
    let rows4 = U8_ROW_TABLE[mask4];
    let row_calc = add_offset_and_mask(byte_col4, rows4, *prev_indent);
    *prev_indent = row_calc[7];
    res[24..32].copy_from_slice(&row_calc);

    let mask5 = ((newline_mask & 0xFF_0000_0000) >> 32) as usize;
    let byte_col5 = U8_BYTE_COL_TABLE[mask5];
    let rows5 = U8_ROW_TABLE[mask5];
    let row_calc = add_offset_and_mask(byte_col5, rows5, *prev_indent);
    *prev_indent = row_calc[7];
    res[32..40].copy_from_slice(&row_calc);

    let mask6 = ((newline_mask & 0xFF00_0000_0000) >> 40) as usize;
    let byte_col6 = U8_BYTE_COL_TABLE[mask6];
    let rows6 = U8_ROW_TABLE[mask6];
    let row_calc = add_offset_and_mask(byte_col6, rows6, *prev_indent);
    *prev_indent = row_calc[7];
    res[40..48].copy_from_slice(&row_calc);

    let mask7 = ((newline_mask & 0xFF_0000_0000_0000) >> 48) as usize;
    let byte_col7 = U8_BYTE_COL_TABLE[mask7];
    let rows7 = U8_ROW_TABLE[mask7];
    let row_calc = add_offset_and_mask(byte_col7, rows7, *prev_indent);
    *prev_indent = row_calc[7];
    res[48..56].copy_from_slice(&row_calc);

    let mask8 = ((newline_mask & 0xFF00_0000_0000_0000) >> 56) as usize;
    let byte_col8 = U8_BYTE_COL_TABLE[mask8];
    let rows8 = U8_ROW_TABLE[mask8];
    let row_calc = add_offset_and_mask(byte_col8, rows8, *prev_indent);
    *prev_indent = row_calc[7];
    res[56..64].copy_from_slice(&row_calc);

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
fn count_u8x16(vec: U8X16, prev_col: &mut u8, prev_row: &mut u8) -> U8X16 {
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
            let count = count_table_small(mask, &mut prev_indent);
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

criterion_group!(benches, col_count_naive, col_count_u8x8, col_count_u8x16, col_count_small);
criterion_main!(benches);

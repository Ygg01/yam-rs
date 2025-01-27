use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use yam_dark_core::util::{INDENT_SWIZZLE_TABLE, U8X8, U8_BYTE_COL_TABLE, U8_ROW_TABLE};
use yam_dark_core::{u8x64_eq, ChunkyIterator, SIMD_CHUNK_LENGTH};

const YAML: &[u8] = r#"
   a: b                      
   c: b       

a  st
   d: a   as
                  
                  
                  
    a: a"#
    .as_bytes();

#[inline]
fn calculate_byte_col(index_mask: usize, reset_bool: bool, prev_indent: &mut u8) -> [u8; 8] {
    let byte_col1 = U8_BYTE_COL_TABLE[index_mask];
    let rows1 = U8_ROW_TABLE[index_mask];
    let row_calc = calculate_cols(byte_col1, rows1, prev_indent);
    let mask_sec = (-(reset_bool as i8)) as u8;
    *prev_indent = (row_calc[7] + 1) & mask_sec;
    row_calc
}

#[inline]
fn calculate_cols(cols: [u8; 8], rows_data: [u8; 8], prev_col: &mut u8) -> [u8; 8] {
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

#[inline]
fn calculate_byte_rows(index_mask: usize, prev_row: &mut u8) -> [u8; 8] {
    let rows1 = U8_ROW_TABLE[index_mask];
    calculate_rows(rows1, prev_row)
}

#[inline]
fn calculate_rows(rows: [u8; 8], prev_row: &mut u8) -> [u8; 8] {
    let x = [
        *prev_row,
        *prev_row + rows[0],
        *prev_row + rows[1],
        *prev_row + rows[2],
        *prev_row + rows[3],
        *prev_row + rows[4],
        *prev_row + rows[5],
        *prev_row + rows[6],
    ];
    *prev_row += rows[7];
    x
}

#[doc(hidden)]
pub fn count_col_rows(
    newline_mask: u64,

    byte_cols: &mut [u8; SIMD_CHUNK_LENGTH],
    byte_rows: &mut [u8; SIMD_CHUNK_LENGTH],
) {
    let mut prev_byte_col = 0;
    let mut prev_byte_row = 0;
    // First 8 bits
    let mask = (newline_mask & 0xFF) as usize;
    let col_result = calculate_byte_col(mask, newline_mask & 0x80 == 0, &mut prev_byte_col);
    byte_cols[0..8].copy_from_slice(&col_result);

    let rows_result = calculate_byte_rows(mask, &mut prev_byte_row);
    byte_rows[0..8].copy_from_slice(&rows_result);

    // Second 8 bits
    let mask = ((newline_mask & 0xFF00) >> 8) as usize;
    let col_result = calculate_byte_col(mask, newline_mask & 0x8000 == 0, &mut prev_byte_col);
    byte_cols[8..16].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, &mut prev_byte_row);
    byte_rows[8..16].copy_from_slice(&col_rows);

    // Third 8 bits
    let mask = ((newline_mask & 0xFF_0000) >> 16) as usize;
    let col_result = calculate_byte_col(mask, newline_mask & 0x80_0000 == 0, &mut prev_byte_col);
    byte_cols[16..24].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, &mut prev_byte_row);
    byte_rows[16..24].copy_from_slice(&col_rows);

    // Fourth 8 bits
    let mask = ((newline_mask & 0xFF00_0000) >> 24) as usize;
    let col_result = calculate_byte_col(mask, newline_mask & 0x8000_0000 == 0, &mut prev_byte_col);
    byte_cols[24..32].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, &mut prev_byte_row);
    byte_rows[24..32].copy_from_slice(&col_rows);

    // Fifth 8 bits
    let mask = ((newline_mask & 0xFF_0000_0000) >> 32) as usize;
    let col_result =
        calculate_byte_col(mask, newline_mask & 0x80_0000_0000 == 0, &mut prev_byte_col);
    byte_cols[32..40].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, &mut prev_byte_row);
    byte_rows[32..40].copy_from_slice(&col_rows);

    // Sixth 8 bits
    let mask = ((newline_mask & 0xFF00_0000_0000) >> 40) as usize;
    let col_result = calculate_byte_col(
        mask,
        newline_mask & 0x8000_0000_0000 == 0,
        &mut prev_byte_col,
    );
    byte_cols[40..48].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, &mut prev_byte_row);
    byte_rows[40..48].copy_from_slice(&col_rows);

    // Seventh 8 bits
    let mask = ((newline_mask & 0xFF_0000_0000_0000) >> 48) as usize;
    let col_result = calculate_byte_col(
        mask,
        newline_mask & 0x80_0000_0000_0000 == 0,
        &mut prev_byte_col,
    );
    byte_cols[48..56].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, &mut prev_byte_row);
    byte_rows[48..56].copy_from_slice(&col_rows);

    // Eight 8 bits
    let mask = ((newline_mask & 0xFF00_0000_0000_0000) >> 56) as usize;
    let col_result = calculate_byte_col(
        mask,
        newline_mask & 0x8000_0000_0000_0000 == 0,
        &mut prev_byte_col,
    );
    byte_cols[56..64].copy_from_slice(&col_result);

    let col_rows = calculate_byte_rows(mask, &mut prev_byte_row);
    byte_rows[56..64].copy_from_slice(&col_rows);
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

            count_col_rows(newline_mask, &mut byte_cols, &mut byte_rows);
            count_indent_dependent(
                newline_mask,
                space_mask,
                &mut prev_iter_char,
                &mut prev_indent,
                &byte_cols,
                &mut indents,
            );
            black_box(indents[3] == 0);

            count_col_rows(newline_mask2, &mut byte_cols, &mut byte_rows);
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
    let mut byte_cols = [0; 64];
    let mut byte_rows = [0; 64];

    group.bench_function("col_count_indent_naive", |b| {
        b.iter(|| {
            let mut prev_indent = 0;
            let mut prev_iter_char = 1;

            count_col_rows(newline_mask, &mut byte_cols, &mut byte_rows);
            count_indent_naive(
                newline_mask,
                space_mask,
                &mut prev_iter_char,
                &mut prev_indent,
                &mut indents,
            );
            black_box(indents[56] == 0);

            count_col_rows(newline_mask2, &mut byte_cols, &mut byte_rows);
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
    byte_cols: &mut [u8; 64],
    byte_rows: &mut [u8; 64],
    byte_indent: &mut [u8; 64],
) {
    let mut curr_row = 0;
    let mut curr_col = 0;
    let mut curr_indent = 0;
    let mut is_indent_frozen = false;
    for pos in 0..64 {
        let is_newline = newline_bits & (1 << pos) != 0;
        let is_space = space_bits & (1 << pos) != 0;

        if is_space && !is_indent_frozen {
            curr_indent += 1;
        } else if !is_space && is_indent_frozen {
            is_indent_frozen = true;
        }

        if is_newline {
            unsafe {
                *byte_cols.get_unchecked_mut(pos) = curr_col + 1;
                *byte_rows.get_unchecked_mut(pos) = curr_row;
            }
            curr_col = 0;
            curr_indent = 0;
            curr_row += 1;
            is_indent_frozen = false;
            continue;
        }

        curr_col += 1;
        unsafe {
            *byte_cols.get_unchecked_mut(pos) = curr_col;
            *byte_rows.get_unchecked_mut(pos) = curr_row;
            *byte_indent.get_unchecked_mut(pos) = curr_indent;
        }
    }
}

#[doc(hidden)]
pub fn count_indent_naive(
    newline_mask: u64,
    space_mask: u64,
    prev_iter_char: &mut u8,
    prev_indent: &mut u8,
    indents: &mut [u8; SIMD_CHUNK_LENGTH],
) {
    for (pos, item) in indents.iter_mut().enumerate().take(64) {
        let is_space = (space_mask & (1 << pos)) != 0;
        let is_newline = (newline_mask & (1 << pos)) != 0;

        *item = *prev_indent;

        match (is_space, is_newline) {
            (true, true) => unreachable!("Character can't be both space and newline at same time"),
            (true, false) => {
                *prev_indent += *prev_iter_char;
            }
            (false, true) => {
                *prev_iter_char = 1;
                *prev_indent = 0
            }
            (false, false) => {
                *prev_iter_char = 0;
            }
        }
    }
}

///
///
/// # Arguments
///
/// * `input`: mutable vector being swizzled
/// * `swizzle`: the array used to alter the order of input vector
///
///
/// # Safety:
///
/// * `swizzle` array must have values in `0..=7` range.
/// * `input` vector
///
unsafe fn swizzle_u32x8(input: &mut [u8], swizzle: &[u8; 8]) {
    *input.get_unchecked_mut(0) = *input.get_unchecked(*swizzle.get_unchecked(0) as usize);
    *input.get_unchecked_mut(1) = *input.get_unchecked(*swizzle.get_unchecked(1) as usize);
    *input.get_unchecked_mut(2) = *input.get_unchecked(*swizzle.get_unchecked(2) as usize);
    *input.get_unchecked_mut(3) = *input.get_unchecked(*swizzle.get_unchecked(3) as usize);
    *input.get_unchecked_mut(4) = *input.get_unchecked(*swizzle.get_unchecked(4) as usize);
    *input.get_unchecked_mut(5) = *input.get_unchecked(*swizzle.get_unchecked(5) as usize);
    *input.get_unchecked_mut(6) = *input.get_unchecked(*swizzle.get_unchecked(6) as usize);
    *input.get_unchecked_mut(7) = *input.get_unchecked(*swizzle.get_unchecked(7) as usize);
}

///
/// Counts the indentation dependent on previously calculated byte columns.
///
/// # Arguments:
///
/// * `newline_mask` - A 64-bit mask representing the position of newline characters.
/// * `space_mask` - A 64-bit mask representing the position of space characters.
/// * `prev_iter_char` - A mutable reference to a flag indicating if the previous character was iterated (0 or 1).
/// * `prev_indent` - A mutable reference to the previous indentation value.
/// * `byte_cols` - An array representing the column positions of each byte.
/// * `indents` - A mutable array to store the calculated indentation values.
///
#[doc(hidden)]
pub fn count_indent_dependent(
    newline_mask: u64,
    space_mask: u64,
    prev_iter_char: &mut u8,
    prev_indent: &mut u8,
    byte_cols: &[u8; SIMD_CHUNK_LENGTH],
    indents: &mut [u8; SIMD_CHUNK_LENGTH],
) {
    ///
    /// # Arguments:
    ///
    /// * `byte_indents`: mutable vector representing indentation of bytes.
    /// * `starts`: starting index which to swizzle
    /// * `spaces_mask`: The bit mask representing spaces.
    /// * `newline_mask`: The bit mask representing newlines.
    /// * `prev_iter_char`: A mutable reference whether to continue or restart indentation count. `1` means ignore and `0` means continue.
    /// * `prev_indent`: A mutable reference to the previous indent value.
    ///
    #[inline]
    fn swizzle_slice(
        byte_indents: &mut [u8; SIMD_CHUNK_LENGTH],
        starts: usize,
        spaces_mask: usize,
        newline_mask: usize,
        prev_iter_char: &mut u8,
        prev_indent: &mut u8,
    ) {
        if *prev_iter_char == 0 {
            byte_indents[starts] = *prev_indent;
        }
        // This is safe because:
        // - INDENT SWIZZLE TABLE is guaranteed to have all swizzle array values
        // - input is always a 64 long slice with start being 56 (56+8 = 64) at most.
        unsafe {
            let swizzle_vec = INDENT_SWIZZLE_TABLE.get_unchecked(spaces_mask);
            swizzle_u32x8(&mut byte_indents[starts..starts + 8], swizzle_vec);
        }
        *prev_iter_char = ((spaces_mask | newline_mask) & (1 << 7)) as u8;
        *prev_indent = byte_indents[starts + 7];
    }

    let space_start_edge =
        space_mask & !(space_mask << 1) & (newline_mask << 1) | newline_mask << 1;
    let mut space_end_mask = !((space_mask & !(space_mask >> 1)) << 1);

    let mut after_indent_bits = space_start_edge & space_end_mask;
    space_end_mask &= space_end_mask << 1;
    after_indent_bits |= (after_indent_bits << 1) & space_end_mask;

    space_end_mask &= space_end_mask << 2;
    after_indent_bits |= (after_indent_bits << 2) & space_end_mask;

    space_end_mask &= space_end_mask << 4;
    after_indent_bits |= (after_indent_bits << 4) & space_end_mask;

    space_end_mask &= space_end_mask << 8;
    after_indent_bits |= (after_indent_bits << 8) & space_end_mask;

    space_end_mask &= space_end_mask << 16;
    after_indent_bits |= (after_indent_bits << 16) & space_end_mask;

    space_end_mask &= space_end_mask << 32;
    after_indent_bits |= (after_indent_bits << 32) & space_end_mask;

    unsafe {
        // Safety INVARIANT:
        // This is always safe since have same alignment and size assuming that both are &[u32; SIMD_CHUNK_LENGTH]
        //  byte_cols: &[u32; SIMD_CHUNK_LENGTH],
        //  indents: &mut [u32; SIMD_CHUNK_LENGTH],
        core::ptr::copy_nonoverlapping(byte_cols.as_ptr(), indents.as_mut_ptr(), 64);
    }

    swizzle_slice(
        indents,
        0,
        (after_indent_bits & 0xFF) as usize,
        (newline_mask & 0xFF) as usize,
        prev_iter_char,
        prev_indent,
    );

    swizzle_slice(
        indents,
        8,
        ((after_indent_bits >> 8) & 0xFF) as usize,
        ((newline_mask >> 8) & 0xFF) as usize,
        prev_iter_char,
        prev_indent,
    );

    swizzle_slice(
        indents,
        16,
        ((after_indent_bits >> 16) & 0xFF) as usize,
        ((newline_mask >> 16) & 0xFF) as usize,
        prev_iter_char,
        prev_indent,
    );

    swizzle_slice(
        indents,
        24,
        ((after_indent_bits >> 24) & 0xFF) as usize,
        ((newline_mask >> 24) & 0xFF) as usize,
        prev_iter_char,
        prev_indent,
    );

    swizzle_slice(
        indents,
        32,
        ((after_indent_bits >> 32) & 0xFF) as usize,
        ((newline_mask >> 32) & 0xFF) as usize,
        prev_iter_char,
        prev_indent,
    );

    swizzle_slice(
        indents,
        40,
        ((after_indent_bits >> 40) & 0xFF) as usize,
        ((newline_mask >> 40) & 0xFF) as usize,
        prev_iter_char,
        prev_indent,
    );

    swizzle_slice(
        indents,
        48,
        ((after_indent_bits >> 48) & 0xFF) as usize,
        ((newline_mask >> 48) & 0xFF) as usize,
        prev_iter_char,
        prev_indent,
    );

    swizzle_slice(
        indents,
        56,
        ((after_indent_bits >> 56) & 0xFF) as usize,
        ((newline_mask >> 56) & 0xFF) as usize,
        prev_iter_char,
        prev_indent,
    );
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
    let mut byte_cols = [0; 64];
    let mut byte_rows = [0; 64];

    group.bench_function("col_naive", |b| {
        b.iter(|| {
            count_naive(
                newline_mask,
                space_mask,
                &mut byte_cols,
                &mut byte_rows,
                &mut indents,
            );
            black_box(indents[56] == 0);

            count_naive(
                newline_mask2,
                space_mask2,
                &mut byte_cols,
                &mut byte_rows,
                &mut indents,
            );
            black_box(byte_rows[3] == 0);
        })
    });
    group.finish();
}

fn col_count_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench-col-batch");
    group.significance_level(0.05).sample_size(100);
    group.throughput(Throughput::Bytes(64 * 2));

    let mut chunk_iter = ChunkyIterator::from_bytes(YAML);
    let chunk = chunk_iter.next().unwrap();
    let newline_mask = u8x64_eq(chunk, b'\n');
    let space_mask = u8x64_eq(chunk, b' ');

    let chunk2 = chunk_iter.next().unwrap();
    let newline_mask2 = u8x64_eq(chunk2, b'\n');
    let space_mask2 = u8x64_eq(chunk2, b' ');

    let mut indents = Vec::new();
    let mut byte_cols = [0; 64];
    let mut byte_rows = [0; 64];

    group.bench_function("col_batch", |b| {
        b.iter(|| {
            count_batch(
                newline_mask,
                space_mask,
                &mut byte_cols,
                &mut byte_rows,
                &mut indents,
            );
            black_box(indents.get(9) == None);

            count_batch(
                newline_mask2,
                space_mask2,
                &mut byte_cols,
                &mut byte_rows,
                &mut indents,
            );
            black_box(byte_rows[3] == 0);
        })
    });
    group.finish();
}

fn count_batch(
    newline_mask: u64,
    space_mask: u64,
    byte_rows: &mut [u8; 64],
    byte_cols: &mut [u8; 64],
    indents: &mut Vec<u32>,
) {
    fn cnt_naive(mut newline_mask: u64, mut space_mask: u64, indents: &mut Vec<u32>) {
        let mut base_len = indents.len();
        indents.reserve(64);
        // let idx_32_8: [u32; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

        let is_unaligned = base_len % 4 != 0;
        let write_fn = if is_unaligned {
            core::ptr::write_unaligned
        } else {
            core::ptr::write
        };

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
            unsafe { write_fn(indents.as_mut_ptr().add(base_len).cast::<[u32; 4]>(), v) }
            base_len += 4;
        }
    }

    let nl_ind0 = (newline_mask & 0xFF) as usize;
    let row0 = U8_ROW_TABLE[nl_ind0];
    let col0 = U8_BYTE_COL_TABLE[nl_ind0];
    byte_rows[0..8].copy_from_slice(&row0);
    byte_cols[0..8].copy_from_slice(&col0);

    let mut prev_col = col0[7];
    let mut prev_row = row0[7];

    let nl_ind = ((newline_mask >> 8) & 0xFF) as usize;
    byte_rows[8..16].copy_from_slice(&calculate_byte_rows(nl_ind, &mut prev_row));
    byte_cols[8..16].copy_from_slice(&calculate_cols(
        U8_ROW_TABLE[nl_ind],
        U8_BYTE_COL_TABLE[nl_ind],
        &mut prev_col,
    ));

    let nl_ind = ((newline_mask >> 16) & 0xFF) as usize;
    byte_rows[16..24].copy_from_slice(&calculate_byte_rows(nl_ind, &mut prev_row));
    byte_cols[16..24].copy_from_slice(&calculate_cols(
        U8_ROW_TABLE[nl_ind],
        U8_BYTE_COL_TABLE[nl_ind],
        &mut prev_col,
    ));

    let nl_ind = ((newline_mask >> 24) & 0xFF) as usize;
    byte_rows[24..32].copy_from_slice(&calculate_byte_rows(nl_ind, &mut prev_row));
    byte_cols[24..32].copy_from_slice(&calculate_cols(
        U8_ROW_TABLE[nl_ind],
        U8_BYTE_COL_TABLE[nl_ind],
        &mut prev_col,
    ));

    let nl_ind = ((newline_mask >> 32) & 0xFF) as usize;
    byte_rows[32..40].copy_from_slice(&calculate_byte_rows(nl_ind, &mut prev_row));
    byte_cols[32..40].copy_from_slice(&calculate_cols(
        U8_ROW_TABLE[nl_ind],
        U8_BYTE_COL_TABLE[nl_ind],
        &mut prev_col,
    ));

    let nl_ind = ((newline_mask >> 40) & 0xFF) as usize;
    byte_rows[40..48].copy_from_slice(&calculate_byte_rows(nl_ind, &mut prev_row));
    byte_cols[40..48].copy_from_slice(&calculate_cols(
        U8_ROW_TABLE[nl_ind],
        U8_BYTE_COL_TABLE[nl_ind],
        &mut prev_col,
    ));

    let nl_ind = ((newline_mask >> 48) & 0xFF) as usize;
    byte_rows[48..56].copy_from_slice(&calculate_byte_rows(nl_ind, &mut prev_row));
    byte_cols[48..56].copy_from_slice(&calculate_cols(
        U8_ROW_TABLE[nl_ind],
        U8_BYTE_COL_TABLE[nl_ind],
        &mut prev_col,
    ));

    let nl_ind = ((newline_mask >> 56) & 0xFF) as usize;
    byte_rows[56..64].copy_from_slice(&calculate_byte_rows(nl_ind, &mut prev_row));
    byte_cols[56..64].copy_from_slice(&calculate_cols(
        U8_ROW_TABLE[nl_ind],
        U8_BYTE_COL_TABLE[nl_ind],
        &mut prev_col,
    ));

    // indents.copy_from_slice(&byte_cols[0..64]);
    cnt_naive(newline_mask, space_mask, indents);
}

criterion_group!(
    benches,
    // col_count_indent,
    // col_count_indent_naive,
    // col_count_all_naive,
    col_count_batch,
);
criterion_main!(benches);

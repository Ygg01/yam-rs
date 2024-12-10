use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use yam_dark_core::util::{
    calculate_byte_rows, calculate_cols, INDENT_SWIZZLE_TABLE, U8_BYTE_COL_TABLE, U8_ROW_TABLE,
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
fn calculate_byte_col(index_mask: usize, reset_bool: bool, prev_indent: &mut u32) -> [u32; 8] {
    let byte_col1 = U8_BYTE_COL_TABLE[index_mask];
    let rows1 = U8_ROW_TABLE[index_mask];
    let row_calc = calculate_cols(byte_col1, rows1, prev_indent);
    let mask_sec = (-(reset_bool as i32)) as u32;
    *prev_indent = (row_calc[7] + 1) & mask_sec;
    row_calc
}

#[doc(hidden)]
pub fn count_col_rows(newline_mask: u64, byte_cols: &mut [u32; 64], byte_rows: &mut [u32; 64]) {
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

fn count_naive(
    newline_bits: u64,
    space_bits: u64,
    byte_cols: &mut [u8; 64],
    byte_rows: &mut [u8; 64],
    byte_indent: &mut [u8; 64],
    _prev_indent: &mut u8,
    is_indent_frozen: &mut bool,
) {
    let mut curr_row = 0;
    let mut curr_col = 0;
    let mut curr_indent = 0;
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
    byte_cols: &[u8; 64],
    indents: &mut [u8; 64],
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
        byte_indents: &mut [u8; 64],
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
    let mut is_frozen = false;
    let mut prev_indent = 0;

    group.bench_function("col_naive", |b| {
        b.iter(|| {
            count_naive(
                newline_mask,
                space_mask,
                &mut byte_cols,
                &mut byte_rows,
                &mut indents,
                &mut prev_indent,
                &mut is_frozen,
            );
            black_box(indents[56] == 0);

            count_naive(
                newline_mask2,
                space_mask2,
                &mut byte_cols,
                &mut byte_rows,
                &mut indents,
                &mut prev_indent,
                &mut is_frozen,
            );
            black_box(byte_rows[3] == 0);
        })
    });
    group.finish();
}

// #[doc(hidden)]
// pub fn count_indent_native_batch(
//     mut newline_mask: u64,
//     space_mask: u64,
//     indents: &mut Vec<u32>,
//     is_running: &mut bool,
//     previous_indent: &mut u32,
// ) {
//     let start_len = indents.len();
//     let mut i = 0;

//     // Reserve enough space for the worst case since it can have
//     indents.reserve(68);
//     let count_cols = newline_mask.count_ones() + 1;
//     let mut runners = Vec::<bool>::with_capacity(count_cols as usize);
//     let mut neg_indents_mask =
//         !select_left_bits_branch_less(space_mask, (newline_mask << 1) ^ (*is_running as u64));

//     // To calculate indent we need to:
//     // 1. Count trailing ones in space_mask this is the current indent
//     // 2. Count the trailing zeros in newline mask to know how long the line is
//     // 3. Check to see if the indent is equal to how much we need to1 shift it, if true we set mask to 1 otherwise to 0.
//     // 4. when returning indent, the 32nd bit will represent the if the indent is still running or if it has stopped
//     while newline_mask != 0 {
//         let part0 = neg_indents_mask.trailing_zeros();
//         let v0 = newline_mask.trailing_zeros() + 1;
//         newline_mask = newline_mask.overflowing_shr(v0).0;
//         neg_indents_mask = neg_indents_mask.overflowing_shr(v0).0;

//         let part1 = neg_indents_mask.trailing_zeros();
//         let v1 = newline_mask.trailing_zeros() + 1;
//         newline_mask = newline_mask.overflowing_shr(v1).0;
//         neg_indents_mask = neg_indents_mask.overflowing_shr(v1).0;

//         let part2 = neg_indents_mask.trailing_zeros();
//         let v2 = newline_mask.trailing_zeros() + 1;
//         newline_mask = newline_mask.overflowing_shr(v2).0;
//         neg_indents_mask = neg_indents_mask.overflowing_shr(v2).0;

//         let part3 = neg_indents_mask.trailing_zeros();
//         let v3 = newline_mask.trailing_zeros() + 1;
//         newline_mask = newline_mask.overflowing_shr(v3).0;
//         neg_indents_mask = neg_indents_mask.overflowing_shr(v3).0;

//         let v = [part0, part1, part2, part3];
//         let running = [part0 == v0, part1 == v1, part2 == v2, part3 == v3];
//         unsafe {
//             write(
//                 indents.as_mut_ptr().add(i + start_len).cast::<[u32; 4]>(),
//                 v,
//             );
//             write(runners.as_mut_ptr().add(i).cast::<[bool; 4]>(), running);
//         }
//         i += 4;
//     }
//     // We do some safety vector snipping here, then handle previous indent.

//     let last_len = start_len + count_cols as usize;

//     // SAFETY: we have reserved enough space, but we will only use start_len + number of newlines + 1
//     // or start_len + count_cols
//     unsafe {
//         indents.set_len(last_len);
//     }
//     if *previous_indent > 0 {
//         // SAFETY: start_len is starting length and since indents are guaranteed to have at least 1
//         // element, we can safely access the element at start_len
//         unsafe {
//             *indents.get_unchecked_mut(start_len) = *previous_indent - 1;
//         }
//     }
//     // SAFETY: last element should be exactly last_len - 1 (because arrays are 0 based)
//     *previous_indent = unsafe {
//         // TODO do actual indent logic here
//         // TODO check what happens for 64 newlines
//         *indents.get_unchecked(last_len - 1)
//     };
//     *is_running = unsafe {
//         *runners.get_unchecked(count_cols as usize -1)
//     };
// }

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

    // let mut indents = Vec::new();
    let mut byte_cols = [0; 64];
    let mut byte_rows = [0; 64];
    let mut byte_indents = [0; 64];

    group.bench_function("col_batch", |b| {
        b.iter(|| {
            count_batch(
                newline_mask,
                space_mask,
                &mut byte_cols,
                &mut byte_rows,
                &mut byte_indents,
            );
            black_box(byte_indents[9] == 1);

            count_batch(
                newline_mask2,
                space_mask2,
                &mut byte_cols,
                &mut byte_rows,
                &mut byte_indents,
            );
            black_box(byte_rows[3] == 0);
        })
    });
    group.finish();
}

#[doc(hidden)]
pub fn count_indent_naive(
    newline_bits: u64,
    space_bits: u64,
    // byte_cols: &mut [u8; 64],
    // byte_rows: &mut [u8; 64],
    byte_indent: &mut [u32; 64],
    is_indent_frozen: &mut bool,
) {
    // let mut curr_row = 0;
    // let mut curr_col = 0;
    let mut curr_indent = 0;
    let mut is_frozen = false;
    for pos in 0..64 {
        let is_newline = newline_bits & (1 << pos) != 0;
        let is_space = space_bits & (1 << pos) != 0;

        if is_space && !is_frozen {
            curr_indent += 1;
        } else if !is_space && is_frozen {
            is_frozen = true;
        }

        if is_newline {
            // curr_col = 0;
            curr_indent = 0;
            // curr_row += 1;
            is_frozen = false;
            continue;
        }

        // curr_col += 1;
        unsafe {
            // *byte_cols.get_unchecked_mut(pos) = curr_col;
            // *byte_rows.get_unchecked_mut(pos) = curr_row;
            *byte_indent.get_unchecked_mut(pos) = curr_indent;
        }
    }
    *is_indent_frozen = is_frozen;
}

fn count_batch(
    newline_mask: u64,
    space_mask: u64,
    byte_rows: &mut [u32; 64],
    byte_cols: &mut [u32; 64],
    byte_indent: &mut [u32; 64],
) {
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

    count_indent_naive(newline_mask, space_mask, byte_indent, &mut false);
}

criterion_group!(
    benches,
    // col_count_indent,
    // col_count_indent_naive,
    col_count_all_naive,
    col_count_batch,
);
criterion_main!(benches);

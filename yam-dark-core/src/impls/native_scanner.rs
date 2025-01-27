#[allow(unused_imports)]
use alloc::vec;
#[allow(unused_imports)]
use alloc::vec::Vec;
use core::ptr::write;
use util::u8x16_swizzle;

use crate::tokenizer::stage1::{Stage1Scanner, YamlChunkState};
use crate::util::NoopValidator;
use crate::util::{u8x64_eq, u8x64_lteq, U8X16};
use crate::{util, YamlParserState, HIGH_NIBBLE, LOW_NIBBLE};

#[doc(hidden)]
pub struct NativeScanner {
    inner_chunk: [u8; 64],
}

impl NativeScanner {}

unsafe impl Stage1Scanner for NativeScanner {
    type SimdType = [u8; 64];
    type Validator = NoopValidator;

    unsafe fn validator() -> Self::Validator {
        NoopValidator {}
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn from_chunk(values: &[u8; 64]) -> Self {
        NativeScanner {
            inner_chunk: *values,
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn cmp_ascii_to_input(&self, cmp: u8) -> u64 {
        u8x64_eq(&self.inner_chunk, cmp)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn unsigned_lteq_against_splat(&self, cmp: i8) -> u64 {
        u8x64_lteq(self.inner_chunk, cmp as u8)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn classify_yaml_characters(&self, block_state: &mut YamlChunkState) {
        // Setup swizzle table
        //
        // Step 1: Setup swizzle mask
        let low_nib_and_mask = U8X16::splat(0xF);
        let high_nib_and_mask = U8X16::splat(0x7F);

        // Step 2: Fill U8X16 SIMD-like vectors with content from chunk
        // SAFETY: All inner chunk slices are 16 bytes long.
        let v0 = unsafe { U8X16::from_slice(&self.inner_chunk[0..16]) };
        let v1 = unsafe { U8X16::from_slice(&self.inner_chunk[16..32]) };
        let v2 = unsafe { U8X16::from_slice(&self.inner_chunk[32..48]) };
        let v3 = unsafe { U8X16::from_slice(&self.inner_chunk[48..64]) };

        // Step 3: Do the lookup via swizzle
        let v_v0 = u8x16_swizzle(LOW_NIBBLE, v0 & low_nib_and_mask)
            & u8x16_swizzle(HIGH_NIBBLE, (v0 >> 4) & high_nib_and_mask);
        let v_v1 = u8x16_swizzle(LOW_NIBBLE, v1 & low_nib_and_mask)
            & u8x16_swizzle(HIGH_NIBBLE, (v1 >> 4) & high_nib_and_mask);
        let v_v2 = u8x16_swizzle(LOW_NIBBLE, v2 & low_nib_and_mask)
            & u8x16_swizzle(HIGH_NIBBLE, (v2 >> 4) & high_nib_and_mask);
        let v_v3 = u8x16_swizzle(LOW_NIBBLE, v3 & low_nib_and_mask)
            & u8x16_swizzle(HIGH_NIBBLE, (v3 >> 4) & high_nib_and_mask);

        // Extract spaces using simple mask and compare.
        let tmp_sp0 = (v_v0 & 0x40).comp_all(0);
        let tmp_sp1 = (v_v1 & 0x40).comp_all(0);
        let tmp_sp2 = (v_v2 & 0x40).comp_all(0);
        let tmp_sp3 = (v_v3 & 0x40).comp_all(0);

        // Convert the SIMD-like type to bitmask
        let spaces_0 = tmp_sp0.to_bitmask64();
        let spaces_1 = tmp_sp1.to_bitmask64();
        let spaces_2 = tmp_sp2.to_bitmask64();
        let spaces_3 = tmp_sp3.to_bitmask64();

        block_state.characters.spaces =
            !(spaces_0 | (spaces_1 << 16) | (spaces_2 << 32) | (spaces_3 << 48));

        // Extract whitespaces using simple mask and compare.
        let tmp_ws0 = (v_v0 & 0x60).comp_all(0);
        let tmp_ws1 = (v_v1 & 0x60).comp_all(0);
        let tmp_ws2 = (v_v2 & 0x60).comp_all(0);
        let tmp_ws3 = (v_v3 & 0x60).comp_all(0);

        let ws_res_0 = tmp_ws0.to_bitmask64();
        let ws_res_1 = tmp_ws1.to_bitmask64();
        let ws_res_2 = tmp_ws2.to_bitmask64();
        let ws_res_3 = tmp_ws3.to_bitmask64();

        block_state.characters.whitespace =
            !(ws_res_0 | (ws_res_1 << 16) | (ws_res_2 << 32) | (ws_res_3 << 48));

        // Extract block structurals
        let tmp_bl0 = (v_v0 & 0xB).comp_all(0);
        let tmp_bl1 = (v_v1 & 0xB).comp_all(0);
        let tmp_bl2 = (v_v2 & 0xB).comp_all(0);
        let tmp_bl3 = (v_v3 & 0xB).comp_all(0);

        let block_structural_res_0 = tmp_bl0.to_bitmask64();
        let block_structural_res_1 = tmp_bl1.to_bitmask64();
        let block_structural_res_2 = tmp_bl2.to_bitmask64();
        let block_structural_res_3 = tmp_bl3.to_bitmask64();

        let block_structurals_candidates = !(block_structural_res_0
            | (block_structural_res_1 << 16)
            | (block_structural_res_2 << 32)
            | (block_structural_res_3 << 48));

        // YAML block structurals like `? `, `- ` and `:` are only valid if followed by a WHITESPACE
        // character or end of line
        block_state.characters.block_structurals =
            block_structurals_candidates & (block_state.characters.whitespace << 1);

        // Extract block structurals
        let tmp_fl0 = (v_v0 & 0x18).comp_all(0);
        let tmp_fl1 = (v_v1 & 0x18).comp_all(0);
        let tmp_fl2 = (v_v2 & 0x18).comp_all(0);
        let tmp_fl3 = (v_v3 & 0x18).comp_all(0);

        let flow_structural_res_0 = tmp_fl0.to_bitmask64();
        let flow_structural_res_1 = tmp_fl1.to_bitmask64();
        let flow_structural_res_2 = tmp_fl2.to_bitmask64();
        let flow_structural_res_3 = tmp_fl3.to_bitmask64();

        block_state.characters.flow_structurals = !(flow_structural_res_0
            | (flow_structural_res_1 << 16)
            | (flow_structural_res_2 << 32)
            | (flow_structural_res_3 << 48));
    }

    fn flatten_bits_yaml(
        base: &mut YamlParserState,
        _yaml_chunk_state: &YamlChunkState,
        mut bits: u64,
    ) {
        let count_ones: usize = bits.count_ones() as usize;
        let mut base_len = base.structurals.len();

        // We're doing some trickery here.
        // We reserve 64 extra entries, because we've at most 64 bit to set
        // then we truncate the base to the next base (that we calculated above)
        // We later indiscriminately write over the len we set but that's OK
        // since we ensure we reserve the needed space
        base.structurals.reserve(64);
        base.byte_cols.reserve(64);
        base.byte_rows.reserve(64);
        base.indents.reserve(64);
        let final_len = base_len + count_ones;

        let is_unaligned = base_len % 4 != 0;

        while bits != 0 {
            let v0 = bits.trailing_zeros();
            bits &= bits.wrapping_sub(1);
            let v1 = bits.trailing_zeros();
            bits &= bits.wrapping_sub(1);
            let v2 = bits.trailing_zeros();
            bits &= bits.wrapping_sub(1);
            let v3 = bits.trailing_zeros();
            bits &= bits.wrapping_sub(1);

            let v: [usize; 4] = [
                base.idx + v0 as usize,
                base.idx + v1 as usize,
                base.idx + v2 as usize,
                base.idx + v3 as usize,
            ];
            // SAFETY:
            // Get unchecked will be less than 64, because trailing zeros of u64 can't be greater than 64
            // these values will be added to base.last_row. Adding a value to base.last_row might panic but
            // shouldn't be a SAFETY problem.
            let row = unsafe {
                [
                    *_yaml_chunk_state.rows.get_unchecked(v0 as usize) as u32 + base.last_row,
                    *_yaml_chunk_state.rows.get_unchecked(v1 as usize) as u32 + base.last_row,
                    *_yaml_chunk_state.rows.get_unchecked(v2 as usize) as u32 + base.last_row,
                    *_yaml_chunk_state.rows.get_unchecked(v3 as usize) as u32 + base.last_row,
                ]
            };
            // SAFETY:
            //
            unsafe {
                write(base.structurals.as_mut_ptr().add(1).cast::<[usize; 4]>(), v);
                write(base.byte_cols.as_mut_ptr().add(1).cast::<[u32; 4]>(), row);
            }

            base_len += 4;
        }
        // Safety:
        //  1. We have reserved 64 entries
        //  2. We have written only `count_len` entries (maximum number is 64)
        unsafe {
            base.structurals.set_len(final_len);
            base.byte_cols.set_len(final_len);
            base.byte_rows.set_len(final_len);
            base.indents.set_len(final_len);
        }
    }
}

#[test]
fn test_calculate_indents() {
    let bin_str = b"                                                                ";
    let mut chunk = YamlChunkState::default();
    let mut prev_iter_state = YamlParserState::default();
    let range1_to_64 = (1..=64u8).collect::<Vec<_>>();
    let scanner = NativeScanner::from_chunk(bin_str);
    // Needs to be called before indent
    chunk.characters.spaces = u8x64_eq(bin_str, b' ');
    chunk.characters.line_feeds = u8x64_eq(bin_str, b'\n');
    scanner.calculate_row_cols(&mut chunk, &mut prev_iter_state);
    assert_eq!(chunk.cols, range1_to_64);
    assert_eq!(chunk.rows, vec![0; 64]);
}

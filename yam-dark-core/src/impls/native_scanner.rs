#[allow(unused_imports)]
use alloc::vec;
#[allow(unused_imports)]
use alloc::vec::Vec;
use core::ptr::write;
use util::u8x16_swizzle;

use crate::tokenizer::stage1::Stage1Scanner;
use crate::tokenizer::stage2::YamlIndentInfo;
use crate::util::{fast_select_high_bits, fast_select_low_bits, NoopValidator};
use crate::util::{u8x64_eq, u8x64_lteq, U8X16};
use crate::{util, YamlCharacterChunk, YamlChunkState, YamlParserState, HIGH_NIBBLE, LOW_NIBBLE};

#[doc(hidden)]
pub struct NativeScanner {
    inner_chunk: [u8; 64],
}

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
    fn unsigned_lteq_against_splat(&self, cmp: u8) -> u64 {
        u8x64_lteq(self.inner_chunk, cmp)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn classify_yaml_characters(&self) -> YamlCharacterChunk {
        let mut characters = YamlCharacterChunk::default();
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

        characters.spaces = !(spaces_0 | (spaces_1 << 16) | (spaces_2 << 32) | (spaces_3 << 48));

        // Extract whitespaces using simple mask and compare.
        let tmp_ws0 = (v_v0 & 0x60).comp_all(0);
        let tmp_ws1 = (v_v1 & 0x60).comp_all(0);
        let tmp_ws2 = (v_v2 & 0x60).comp_all(0);
        let tmp_ws3 = (v_v3 & 0x60).comp_all(0);

        let ws_res_0 = tmp_ws0.to_bitmask64();
        let ws_res_1 = tmp_ws1.to_bitmask64();
        let ws_res_2 = tmp_ws2.to_bitmask64();
        let ws_res_3 = tmp_ws3.to_bitmask64();

        characters.whitespace =
            !(ws_res_0 | (ws_res_1 << 16) | (ws_res_2 << 32) | (ws_res_3 << 48));

        // Extract block structurals
        let tmp_block0 = (v_v0 & 0xB).comp_all(0);
        let tmp_block1 = (v_v1 & 0xB).comp_all(0);
        let tmp_block2 = (v_v2 & 0xB).comp_all(0);
        let tmp_block3 = (v_v3 & 0xB).comp_all(0);

        let block_structural_res_0 = tmp_block0.to_bitmask64();
        let block_structural_res_1 = tmp_block1.to_bitmask64();
        let block_structural_res_2 = tmp_block2.to_bitmask64();
        let block_structural_res_3 = tmp_block3.to_bitmask64();

        let block_structurals_candidates = !(block_structural_res_0
            | (block_structural_res_1 << 16)
            | (block_structural_res_2 << 32)
            | (block_structural_res_3 << 48));

        // YAML block structurals like `? `, `- ` and `: ` are only valid if followed by a WHITESPACE
        // character or end of line
        characters.block_structurals = block_structurals_candidates & (characters.whitespace << 1);

        // Extract block structurals
        let tmp_flow0 = (v_v0 & 0x18).comp_all(0);
        let tmp_flow1 = (v_v1 & 0x18).comp_all(0);
        let tmp_flow2 = (v_v2 & 0x18).comp_all(0);
        let tmp_flow3 = (v_v3 & 0x18).comp_all(0);

        let flow_structural_res_0 = tmp_flow0.to_bitmask64();
        let flow_structural_res_1 = tmp_flow1.to_bitmask64();
        let flow_structural_res_2 = tmp_flow2.to_bitmask64();
        let flow_structural_res_3 = tmp_flow3.to_bitmask64();

        characters.flow_structurals = !(flow_structural_res_0
            | (flow_structural_res_1 << 16)
            | (flow_structural_res_2 << 32)
            | (flow_structural_res_3 << 48));

        characters.line_feeds = self.cmp_ascii_to_input(b'\n');

        // Unquoted possible start
        let non_white_space_starts = !characters.whitespace & (characters.whitespace << 1);
        let non_structurals = !(characters.flow_structurals | characters.block_structurals);
        let possible_blocks = fast_select_low_bits(non_structurals, non_white_space_starts);
        characters.in_unquoted_scalars =
            fast_select_high_bits(possible_blocks, non_white_space_starts);
        characters.unquoted_scalars_starts =
            characters.in_unquoted_scalars & !(characters.in_unquoted_scalars << 1);

        characters
    }

    fn flatten_bits_yaml(
        chunk_state: &YamlChunkState,
        base: &mut YamlParserState,
        indent_info: &mut YamlIndentInfo,
    ) {
        let mut bits = chunk_state.substructure();
        let count_ones: usize = bits.count_ones() as usize;
        let mut old_len = base.structurals.len();

        // We're doing some trickery here.
        // We reserve 64 extra entries, because we've at most 64 bit to set
        // then we truncate the base to the next base (that we calculated above).
        // We later indiscriminately write over the len we set, but that's OK
        // since we ensure we reserve the necessary space.
        base.structurals.reserve(64);
        base.byte_cols.reserve(64);
        base.byte_rows.reserve(64);
        base.indents.reserve(64);

        let final_len = old_len + count_ones;

        macro_rules! u32x4 {
            ($field:expr, $data:ident, $pos:expr) => {
                core::ptr::write($field.as_mut_ptr().add($pos).cast::<[u32; 4]>(), $data);
            };
        }

        while bits != 0 {
            let v0 = bits.trailing_zeros();
            bits &= bits.saturating_sub(1);
            let v1 = bits.trailing_zeros();
            bits &= bits.saturating_sub(1);
            let v2 = bits.trailing_zeros();
            bits &= bits.saturating_sub(1);
            let v3 = bits.trailing_zeros();
            bits &= bits.saturating_sub(1);

            let v: [usize; 4] = [
                base.pos + v0 as usize,
                base.pos + v1 as usize,
                base.pos + v2 as usize,
                base.pos + v3 as usize,
            ];

            // SAFETY:
            // Get unchecked will be less than 64, because trailing zeros of u64 can't be greater than 64
            // these values will be added to base.last_row. Adding a value to base.last_row might panic but
            // shouldn't be a SAFETY problem.
            let cols: [u32; 4] = unsafe {
                [
                    *indent_info.cols.get_unchecked(v0 as usize),
                    *indent_info.cols.get_unchecked(v1 as usize),
                    *indent_info.cols.get_unchecked(v2 as usize),
                    *indent_info.cols.get_unchecked(v3 as usize),
                ]
            };
            // SAFETY:
            // Get unchecked will be less than 64, because trailing zeros of u64 can't be greater than 64
            // these values will be added to base.last_row. Adding a value to base.last_row might panic but
            // shouldn't be a SAFETY problem.
            let rows = unsafe {
                [
                    *indent_info.rows.get_unchecked(v0 as usize),
                    *indent_info.rows.get_unchecked(v1 as usize),
                    *indent_info.rows.get_unchecked(v2 as usize),
                    *indent_info.rows.get_unchecked(v3 as usize),
                ]
            };

            // SAFETY:
            // Get unchecked will be less than 64, because trailing zeroes of u64 can't be greater than 64
            let indents = unsafe {
                [
                    *indent_info.indents.get_unchecked(v0 as usize),
                    *indent_info.indents.get_unchecked(v1 as usize),
                    *indent_info.indents.get_unchecked(v2 as usize),
                    *indent_info.indents.get_unchecked(v3 as usize),
                ]
            };

            // SAFETY:
            // Writing arrays into vec will always be aligned.
            unsafe {
                write(
                    base.structurals
                        .as_mut_ptr()
                        .add(old_len)
                        .cast::<[usize; 4]>(),
                    v,
                );
                u32x4!(&mut base.byte_cols, cols, old_len);
                u32x4!(&mut base.byte_rows, rows, old_len);
                u32x4!(&mut base.byte_rows, indents, old_len);
            }

            old_len += 4;
        }
        // SAFETY:
        // `set_len` is safe if `new_len` <= `capacity` and `old_len..new_len` is initialized.
        // INVARIANTS:
        // - all four vectors have reserved 64 fields
        // - `final_len` must be less than `old_len + 64`
        // - `old_len..new_len` is initialized by the loop.
        debug_assert!(final_len <= old_len + 64);
        unsafe {
            base.structurals.set_len(final_len);
            base.byte_cols.set_len(final_len);
            base.byte_rows.set_len(final_len);
            base.indents.set_len(final_len);
        }
    }
}

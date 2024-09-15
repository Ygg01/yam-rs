use util::u8x16_swizzle;

use crate::tokenizer::stage1::{Stage1Scanner, YamlChunkState};
use crate::util::NoopValidator;
use crate::util::{u8x64_eq, u8x64_lteq, U8X16};
use crate::{util, YamlParserState, HIGH_NIBBLE_MASK, LOW_NIBBLE_MASK};

#[doc(hidden)]
pub struct NativeScanner {
    v0: [u8; 64],
}

impl NativeScanner {}

unsafe impl Stage1Scanner for NativeScanner {
    type SimdType = [u8; 64];
    type Validator = NoopValidator;

    fn validator() -> Self::Validator {
        NoopValidator {}
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn from_chunk(values: &[u8; 64]) -> Self {
        NativeScanner { v0: *values }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn cmp_ascii_to_input(&self, cmp: u8) -> u64 {
        u8x64_eq(&self.v0, cmp)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn calculate_indents(
        &self,
        chunk_state: &mut YamlChunkState,
        prev_state: &mut YamlParserState,
    ) {
        let mut curr_row = prev_state.last_row;
        let mut curr_col = prev_state.last_col;
        let mut curr_indent = prev_state.last_indent;

        let spaces = u8x64_lteq(self.v0, b' ');
        let line_feeds = u8x64_lteq(self.v0, b'\n');

        for pos in 0..64 {
            let is_newline = line_feeds & (1 << pos) != 0;
            let is_space = spaces & (1 << pos) != 0;

            if is_space && !prev_state.is_indent_frozen {
                curr_indent += 1;
            } else if !is_space && prev_state.is_indent_frozen {
                prev_state.is_indent_frozen = true;
            }

            if is_newline {
                // Safety since pos is guaranteed to be between 0..=63
                // and we initialized cols/rows/indents up to be exactly 64 elements, we can
                // safely access it without bound checks.
                unsafe {
                    *chunk_state.cols.get_unchecked_mut(pos) = curr_col + 1;
                    *chunk_state.rows.get_unchecked_mut(pos) = curr_row;
                }
                curr_col = 0;
                curr_indent = 0;
                curr_row += 1;
                prev_state.is_indent_frozen = false;
                continue;
            }

            curr_col += 1;
            // Safety since pos is guaranteed to be between 0..=63
            // and we initialized cols/rows/indents to be exactly 64 elements, we can
            // safely access it without bound checks.
            unsafe {
                *chunk_state.cols.get_unchecked_mut(pos) = curr_col;
                *chunk_state.rows.get_unchecked_mut(pos) = curr_row;
                *chunk_state.indents.get_unchecked_mut(pos) = curr_indent;
            }
        }
        chunk_state.characters.line_feeds = line_feeds;
        chunk_state.characters.spaces = spaces;
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[allow(clippy::cast_sign_loss)]
    fn compute_quote_mask(quote_bits: u64) -> u64 {
        let mut quote_mask: u64 = quote_bits ^ (quote_bits << 1);
        quote_mask = quote_mask ^ (quote_mask << 2);
        quote_mask = quote_mask ^ (quote_mask << 4);
        quote_mask = quote_mask ^ (quote_mask << 8);
        quote_mask = quote_mask ^ (quote_mask << 16);
        quote_mask = quote_mask ^ (quote_mask << 32);
        quote_mask
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn unsigned_lteq_against_splat(&self, cmp: i8) -> u64 {
        u8x64_lteq(self.v0, cmp as u8)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn scan_whitespace_and_structurals(&self, block_state: &mut YamlChunkState) {
        let low_nib_and_mask = U8X16::splat(0xF);
        let high_nib_and_mask = U8X16::splat(0x7F);

        let v0 = unsafe { U8X16::from_slice(&self.v0[0..16]) };
        let v1 = unsafe { U8X16::from_slice(&self.v0[16..32]) };
        let v2 = unsafe { U8X16::from_slice(&self.v0[32..48]) };
        let v3 = unsafe { U8X16::from_slice(&self.v0[48..64]) };

        let v_v0 = u8x16_swizzle(LOW_NIBBLE_MASK, v0 & low_nib_and_mask)
            & u8x16_swizzle(HIGH_NIBBLE_MASK, (v0 >> 4) & high_nib_and_mask);
        let v_v1 = u8x16_swizzle(LOW_NIBBLE_MASK, v1 & low_nib_and_mask)
            & u8x16_swizzle(HIGH_NIBBLE_MASK, (v1 >> 4) & high_nib_and_mask);
        let v_v2 = u8x16_swizzle(LOW_NIBBLE_MASK, v2 & low_nib_and_mask)
            & u8x16_swizzle(HIGH_NIBBLE_MASK, (v2 >> 4) & high_nib_and_mask);
        let v_v3 = u8x16_swizzle(LOW_NIBBLE_MASK, v3 & low_nib_and_mask)
            & u8x16_swizzle(HIGH_NIBBLE_MASK, (v3 >> 4) & high_nib_and_mask);

        let tmp_v0 = (v_v0 & 0x7).comp_all(0);
        let tmp_v1 = (v_v1 & 0x7).comp_all(0);
        let tmp_v2 = (v_v2 & 0x7).comp_all(0);
        let tmp_v3 = (v_v3 & 0x7).comp_all(0);

        let structural_res_0 = tmp_v0.to_bitmask() as u64;
        let structural_res_1 = tmp_v1.to_bitmask() as u64;
        let structural_res_2 = tmp_v2.to_bitmask() as u64;
        let structural_res_3 = tmp_v3.to_bitmask() as u64;

        block_state.characters.structurals = !(structural_res_0
            | (structural_res_1 << 16)
            | (structural_res_2 << 32)
            | (structural_res_3 << 48));

        let tmp_ws0 = (v_v0 & 0x18).comp_all(0);
        let tmp_ws1 = (v_v1 & 0x18).comp_all(0);
        let tmp_ws2 = (v_v2 & 0x18).comp_all(0);
        let tmp_ws3 = (v_v3 & 0x18).comp_all(0);

        let ws_res_0 = tmp_ws0.to_bitmask() as u64;
        let ws_res_1 = tmp_ws1.to_bitmask() as u64;
        let ws_res_2 = tmp_ws2.to_bitmask() as u64;
        let ws_res_3 = tmp_ws3.to_bitmask() as u64;

        block_state.characters.whitespace =
            !(ws_res_0 | (ws_res_1 << 16) | (ws_res_2 << 32) | (ws_res_3 << 48))
    }
}

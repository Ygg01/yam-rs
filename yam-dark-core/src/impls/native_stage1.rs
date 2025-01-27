use util::u8x16_swizzle;

use crate::tokenizer::stage1::{Stage1Scanner, YamlCharacterChunk, YamlChunkState};
use crate::util::NoopValidator;
use crate::util::{u8x64_eq, u8x64_lteq, U8X16};
use crate::{util, HIGH_NIBBLE_MASK, LOW_NIBBLE_MASK};

#[doc(hidden)]
pub struct NativeScanner {
    v0: [u8; 64],
}

impl Stage1Scanner for NativeScanner {
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
        u8x64_eq(self.v0, cmp)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn leading_spaces(&self, chunks: &YamlCharacterChunk) -> (u32, u32) {
        // TODO actual spaces implementation
        let z = chunks.spaces.leading_zeros();
        (z, z)
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

    fn scan_whitespace_and_structurals(&self, block_state: &mut YamlChunkState) {
        let low_nib_and_mask = U8X16::splat(0xF);
        let high_nib_and_mask = U8X16::splat(0x7F);

        let v0 = unsafe { U8X16::from_slice(&self.v0[0..16]) };
        let v1 = unsafe { U8X16::from_slice(&self.v0[16..32]) };
        let v2 = unsafe { U8X16::from_slice(&self.v0[32..48]) };
        let v3 = unsafe { U8X16::from_slice(&self.v0[48..64]) };

        let v_v0 = u8x16_swizzle(LOW_NIBBLE_MASK, v0 & low_nib_and_mask)
            & u8x16_swizzle(HIGH_NIBBLE_MASK, (v0 >> 4) & high_nib_and_mask);
        let v_v1 = u8x16_swizzle(LOW_NIBBLE_MASK, v0 & low_nib_and_mask)
            & u8x16_swizzle(HIGH_NIBBLE_MASK, (v1 >> 4) & high_nib_and_mask);
        let v_v2 = u8x16_swizzle(LOW_NIBBLE_MASK, v0 & low_nib_and_mask)
            & u8x16_swizzle(HIGH_NIBBLE_MASK, (v2 >> 4) & high_nib_and_mask);
        let v_v3 = u8x16_swizzle(LOW_NIBBLE_MASK, v0 & low_nib_and_mask)
            & u8x16_swizzle(HIGH_NIBBLE_MASK, (v3 >> 4) & high_nib_and_mask);

        let structurals = U8X16::merge(v_v0 & 7, v_v1 & 7, v_v2 & 7, v_v3 & 7);
        block_state.characters.op = !u8x64_eq(structurals, 0);
        let ws = U8X16::merge(v_v0 & 18, v_v1 & 18, v_v2 & 18, v_v3 & 18);
        block_state.characters.spaces = !u8x64_eq(ws, 0);
    }
}

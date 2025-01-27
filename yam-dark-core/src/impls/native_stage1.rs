use core::ops::{Add, BitAnd, Shr};

use crate::{HIGH_NIBBLE, LOW_NIBBLE, u8x64_eq, u8x64_lteq};
use crate::tokenizer::stage1::{Stage1Scanner, YamlCharacterChunk, YamlChunkState};
use crate::util::NoopValidator;

#[doc(hidden)]
pub struct NativeScanner {
    v0: [u8; 64],
}

#[derive(Copy, Clone)]
pub struct U8X16([u8; 16]);

impl U8X16 {
    #[inline]
    pub fn splat(input: u8) -> Self {
        U8X16([input; 16])
    }

    #[inline]
    pub fn from_array(input: [u8; 16]) -> Self {
        U8X16(input)
    }

    /// Creates a new instance of `Self` by converting a slice of `u8` into the desired type.
    #[inline]
    pub unsafe fn from_slice(input: &[u8]) -> Self {
        U8X16(
            [
                *input.get_unchecked(0),
                *input.get_unchecked(1),
                *input.get_unchecked(2),
                *input.get_unchecked(3),
                *input.get_unchecked(4),
                *input.get_unchecked(5),
                *input.get_unchecked(6),
                *input.get_unchecked(7),
                *input.get_unchecked(8),
                *input.get_unchecked(9),
                *input.get_unchecked(10),
                *input.get_unchecked(11),
                *input.get_unchecked(12),
                *input.get_unchecked(13),
                *input.get_unchecked(14),
                *input.get_unchecked(15),
            ]
        )
    }

    pub fn merge(
        input0: U8X16,
        input1: U8X16,
        input2: U8X16,
        input3: U8X16,
    ) -> [u8; 64] {
        [
            input0.0[0],
            input0.0[1],
            input0.0[2],
            input0.0[3],
            input0.0[4],
            input0.0[5],
            input0.0[6],
            input0.0[7],
            input0.0[8],
            input0.0[9],
            input0.0[10],
            input0.0[11],
            input0.0[12],
            input0.0[13],
            input0.0[14],
            input0.0[15],
            input1.0[0],
            input1.0[1],
            input1.0[2],
            input1.0[3],
            input1.0[4],
            input1.0[5],
            input1.0[6],
            input1.0[7],
            input1.0[8],
            input1.0[9],
            input1.0[10],
            input1.0[11],
            input1.0[12],
            input1.0[13],
            input1.0[14],
            input1.0[15],
            input2.0[0],
            input2.0[1],
            input2.0[2],
            input2.0[3],
            input2.0[4],
            input2.0[5],
            input2.0[6],
            input2.0[7],
            input2.0[8],
            input2.0[9],
            input2.0[10],
            input2.0[11],
            input2.0[12],
            input2.0[13],
            input2.0[14],
            input2.0[15],
            input3.0[0],
            input3.0[1],
            input3.0[2],
            input3.0[3],
            input3.0[4],
            input3.0[5],
            input3.0[6],
            input3.0[7],
            input3.0[8],
            input3.0[9],
            input3.0[10],
            input3.0[11],
            input3.0[12],
            input3.0[13],
            input3.0[14],
            input3.0[15],
        ]
    }
}

impl BitAnd<[u8; 16]> for U8X16 {
    type Output = U8X16;

    #[inline]
    fn bitand(self, other: [u8; 16]) -> Self::Output {
        U8X16([
            self.0[0] & other[0],
            self.0[1] & other[1],
            self.0[2] & other[2],
            self.0[3] & other[3],
            self.0[4] & other[4],
            self.0[5] & other[5],
            self.0[6] & other[6],
            self.0[7] & other[7],
            self.0[8] & other[8],
            self.0[9] & other[9],
            self.0[10] & other[10],
            self.0[11] & other[11],
            self.0[12] & other[12],
            self.0[13] & other[13],
            self.0[14] & other[14],
            self.0[15] & other[15],
        ])
    }
}

impl BitAnd<u8> for U8X16 {
    type Output = U8X16;

    #[inline]
    fn bitand(self, other: u8) -> Self::Output {
        U8X16([
            self.0[0] & other,
            self.0[1] & other,
            self.0[2] & other,
            self.0[3] & other,
            self.0[4] & other,
            self.0[5] & other,
            self.0[6] & other,
            self.0[7] & other,
            self.0[8] & other,
            self.0[9] & other,
            self.0[10] & other,
            self.0[11] & other,
            self.0[12] & other,
            self.0[13] & other,
            self.0[14] & other,
            self.0[15] & other,
        ])
    }
}

impl BitAnd for U8X16 {
    type Output = U8X16;

    #[inline]
    fn bitand(self, other: Self) -> Self::Output {
        U8X16([
            self.0[0] & other.0[0],
            self.0[1] & other.0[1],
            self.0[2] & other.0[2],
            self.0[3] & other.0[3],
            self.0[4] & other.0[4],
            self.0[5] & other.0[5],
            self.0[6] & other.0[6],
            self.0[7] & other.0[7],
            self.0[8] & other.0[8],
            self.0[9] & other.0[9],
            self.0[10] & other.0[10],
            self.0[11] & other.0[11],
            self.0[12] & other.0[12],
            self.0[13] & other.0[13],
            self.0[14] & other.0[14],
            self.0[15] & other.0[15],
        ])
    }
}

impl Shr<usize> for U8X16 {
    type Output = U8X16;

    #[inline]
    fn shr(self, rhs: usize) -> Self::Output {
        U8X16([
            self.0[0] >> rhs,
            self.0[1] >> rhs,
            self.0[2] >> rhs,
            self.0[3] >> rhs,
            self.0[4] >> rhs,
            self.0[5] >> rhs,
            self.0[6] >> rhs,
            self.0[7] >> rhs,
            self.0[8] >> rhs,
            self.0[9] >> rhs,
            self.0[10] >> rhs,
            self.0[11] >> rhs,
            self.0[12] >> rhs,
            self.0[13] >> rhs,
            self.0[14] >> rhs,
            self.0[15] >> rhs,
        ])
    }
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


        let v0 = unsafe {
            U8X16::from_slice(&self.v0[0..16])
        };
        let v1 = unsafe {
            U8X16::from_slice(&self.v0[16..32])
        };
        let v2 = unsafe {
            U8X16::from_slice(&self.v0[32..48])
        };
        let v3 = unsafe {
            U8X16::from_slice(&self.v0[48..64])
        };

        let v_v0 = u8x16_swizzle(LOW_NIBBLE, v0 & low_nib_and_mask)
            & u8x16_swizzle(HIGH_NIBBLE, (v0 >> 4) & high_nib_and_mask);
        let v_v1 = u8x16_swizzle(LOW_NIBBLE, v0 & low_nib_and_mask)
            & u8x16_swizzle(HIGH_NIBBLE, (v1 >> 4) & high_nib_and_mask);
        let v_v2 = u8x16_swizzle(LOW_NIBBLE, v0 & low_nib_and_mask)
            & u8x16_swizzle(HIGH_NIBBLE, (v2 >> 4) & high_nib_and_mask);
        let v_v3 = u8x16_swizzle(LOW_NIBBLE, v0 & low_nib_and_mask)
            & u8x16_swizzle(HIGH_NIBBLE, (v3 >> 4) & high_nib_and_mask);

        let structurals = U8X16::merge(
            v_v0 & 7,
            v_v1 & 7,
            v_v2 & 7,
            v_v3 & 7,
        );
        *block_state.characters.op = !u8x64_eq(structurals, 0);
        let ws = U8X16::merge(
            v_v0 & 18,
            v_v1 & 18,
            v_v2 & 18,
            v_v3 & 18,
        );
        *block_state.characters.spaces = !u8x64_eq(ws, 0);

    }
}

#[inline]
fn u8x16_swizzle(mask: [u8; 16], x: U8X16) -> U8X16 {
    U8X16([
        if x.0[0] > 0x0f {
            0
        } else {
            mask[(x.0[0] & 0x0f) as usize]
        },
        if x.0[1] > 0x0f {
            0
        } else {
            mask[(x.0[1] & 0x0f) as usize]
        },
        if x.0[2] > 0x0f {
            0
        } else {
            mask[(x.0[2] & 0x0f) as usize]
        },
        if x.0[3] > 0x0f {
            0
        } else {
            mask[(x.0[3] & 0x0f) as usize]
        },
        if x.0[4] > 0x0f {
            0
        } else {
            mask[(x.0[4] & 0x0f) as usize]
        },
        if x.0[5] > 0x0f {
            0
        } else {
            mask[(x.0[5] & 0x0f) as usize]
        },
        if x.0[6] > 0x0f {
            0
        } else {
            mask[(x.0[6] & 0x0f) as usize]
        },
        if x.0[7] > 0x0f {
            0
        } else {
            mask[(x.0[7] & 0x0f) as usize]
        },
        if x.0[8] > 0x0f {
            0
        } else {
            mask[(x.0[8] & 0x0f) as usize]
        },
        if x.0[9] > 0x0f {
            0
        } else {
            mask[(x.0[9] & 0x0f) as usize]
        },
        if x.0[10] > 0x0f {
            0
        } else {
            mask[(x.0[10] & 0x0f) as usize]
        },
        if x.0[11] > 0x0f {
            0
        } else {
            mask[(x.0[11] & 0x0f) as usize]
        },
        if x.0[12] > 0x0f {
            0
        } else {
            mask[(x.0[12] & 0x0f) as usize]
        },
        if x.0[13] > 0x0f {
            0
        } else {
            mask[(x.0[13] & 0x0f) as usize]
        },
        if x.0[14] > 0x0f {
            0
        } else {
            mask[(x.0[14] & 0x0f) as usize]
        },
        if x.0[15] > 0x0f {
            0
        } else {
            mask[(x.0[15] & 0x0f) as usize]
        },
    ])
}
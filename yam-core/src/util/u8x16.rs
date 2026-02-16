use crate::util::BitOps;
use crate::util::macros::{bitmask, gen_u8_cmp, gen_u8_cmp_all, swizzle};
use core::ops::{Add, BitAnd, Index, Shr};

#[derive(Debug, Copy, Clone)]
pub(crate) struct U8X16([u8; 16]);

impl Index<usize> for U8X16 {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl U8X16 {
    pub(crate) const fn splat(input: u8) -> Self {
        Self([input; 16])
    }

    pub(crate) const fn from_array(input: [u8; 16]) -> Self {
        Self(input)
    }
}

//noinspection ALL
impl BitOps for U8X16 {
    type ByteOut = u32;
    fn comp_to_bitmask(self, cmp: u8) -> Self::ByteOut {
        u32::from(self[0] == cmp)
            | gen_u8_cmp!(self, 1 => cmp)
            | gen_u8_cmp!(self, 2 => cmp)
            | gen_u8_cmp!(self, 3 => cmp)
            | gen_u8_cmp!(self, 4 => cmp)
            | gen_u8_cmp!(self, 5 => cmp)
            | gen_u8_cmp!(self, 6 => cmp)
            | gen_u8_cmp!(self, 7 => cmp)
            | gen_u8_cmp!(self, 8 => cmp)
            | gen_u8_cmp!(self, 9 => cmp)
            | gen_u8_cmp!(self, 10 => cmp)
            | gen_u8_cmp!(self, 11 => cmp)
            | gen_u8_cmp!(self, 12 => cmp)
            | gen_u8_cmp!(self, 13 => cmp)
            | gen_u8_cmp!(self, 14 => cmp)
            | gen_u8_cmp!(self, 15 => cmp)
    }

    #[doc(hidden)]
    /// Compares each element of the `U8X16` with a given `u8` value.
    /// If an element is equal to the given value, the corresponding element
    /// in the resulting `U8X16` is set to `0xFF`; otherwise, it is set to `0x00`.
    ///
    /// # Arguments
    ///
    /// * `cmp` - A `u8` value that each element of the `U8X16` will be compared against.
    ///
    /// # Returns
    ///
    /// A `U8X16` instance where each element is either `0xFF` if the corresponding
    /// element in the original `U8X16` is equal to the `cmp` value, or `0x00` if it is not.
    ///
    /// # Examples
    ///
    fn comp(self, cmp: u8) -> Self {
        Self([
            gen_u8_cmp_all!(self, 0 => cmp),
            gen_u8_cmp_all!(self, 1 => cmp),
            gen_u8_cmp_all!(self, 2 => cmp),
            gen_u8_cmp_all!(self, 3 => cmp),
            gen_u8_cmp_all!(self, 4 => cmp),
            gen_u8_cmp_all!(self, 5 => cmp),
            gen_u8_cmp_all!(self, 6 => cmp),
            gen_u8_cmp_all!(self, 7 => cmp),
            gen_u8_cmp_all!(self, 8 => cmp),
            gen_u8_cmp_all!(self, 9 => cmp),
            gen_u8_cmp_all!(self, 10 => cmp),
            gen_u8_cmp_all!(self, 11 => cmp),
            gen_u8_cmp_all!(self, 12 => cmp),
            gen_u8_cmp_all!(self, 13 => cmp),
            gen_u8_cmp_all!(self, 14 => cmp),
            gen_u8_cmp_all!(self, 15 => cmp),
        ])
    }

    fn to_bitmask(&self) -> Self::ByteOut {
        bitmask!(self, 0 => u32)
            | bitmask!(self, 1 => u32)
            | bitmask!(self, 2 => u32)
            | bitmask!(self, 3 => u32)
            | bitmask!(self, 4 => u32)
            | bitmask!(self, 5 => u32)
            | bitmask!(self, 6 => u32)
            | bitmask!(self, 7 => u32)
            | bitmask!(self, 8 => u32)
            | bitmask!(self, 9 => u32)
            | bitmask!(self, 10 => u32)
            | bitmask!(self, 11 => u32)
            | bitmask!(self, 12 => u32)
            | bitmask!(self, 13 => u32)
            | bitmask!(self, 14 => u32)
            | bitmask!(self, 15 => u32)
            | bitmask!(self, 16 => u32)
            | bitmask!(self, 17 => u32)
            | bitmask!(self, 18 => u32)
            | bitmask!(self, 19 => u32)
            | bitmask!(self, 20 => u32)
            | bitmask!(self, 21 => u32)
            | bitmask!(self, 22 => u32)
            | bitmask!(self, 23 => u32)
            | bitmask!(self, 24 => u32)
            | bitmask!(self, 25 => u32)
            | bitmask!(self, 26 => u32)
            | bitmask!(self, 27 => u32)
            | bitmask!(self, 28 => u32)
            | bitmask!(self, 29 => u32)
            | bitmask!(self, 30 => u32)
            | bitmask!(self, 31 => u32)
    }

    fn swizzle(self, other: Self) -> Self {
        Self([
            swizzle!(self, 0 => other),
            swizzle!(self, 1 => other),
            swizzle!(self, 2 => other),
            swizzle!(self, 3 => other),
            swizzle!(self, 4 => other),
            swizzle!(self, 5 => other),
            swizzle!(self, 6 => other),
            swizzle!(self, 7 => other),
            swizzle!(self, 8 => other),
            swizzle!(self, 9 => other),
            swizzle!(self, 10 => other),
            swizzle!(self, 11 => other),
            swizzle!(self, 12 => other),
            swizzle!(self, 13 => other),
            swizzle!(self, 14 => other),
            swizzle!(self, 15 => other),
        ])
    }

    fn and(self, other: Self) -> Self {
        Self([
            self[0] & other[0],
            self[1] & other[1],
            self[2] & other[2],
            self[3] & other[3],
            self[4] & other[4],
            self[5] & other[5],
            self[6] & other[6],
            self[7] & other[7],
            self[8] & other[8],
            self[9] & other[9],
            self[10] & other[10],
            self[11] & other[11],
            self[12] & other[12],
            self[13] & other[13],
            self[14] & other[14],
            self[15] & other[15],
        ])
    }

    fn shift_right(self, rhs: usize) -> Self {
        U8X16([
            self[0] >> rhs,
            self[1] >> rhs,
            self[2] >> rhs,
            self[3] >> rhs,
            self[4] >> rhs,
            self[5] >> rhs,
            self[6] >> rhs,
            self[7] >> rhs,
            self[8] >> rhs,
            self[9] >> rhs,
            self[10] >> rhs,
            self[11] >> rhs,
            self[12] >> rhs,
            self[13] >> rhs,
            self[14] >> rhs,
            self[15] >> rhs,
        ])
    }

    fn add_other(self, other: Self) -> Self {
        Self([
            self[0] + other[0],
            self[1] + other[1],
            self[2] + other[2],
            self[3] + other[3],
            self[4] + other[4],
            self[5] + other[5],
            self[6] + other[6],
            self[7] + other[7],
            self[8] + other[8],
            self[9] + other[9],
            self[10] + other[10],
            self[11] + other[11],
            self[12] + other[12],
            self[13] + other[13],
            self[14] + other[14],
            self[15] + other[15],
        ])
    }
    fn and_byte(self, rhs: u8) -> Self {
        Self(self.0.map(|v| v & rhs))
    }
}

impl BitAnd<U8X16> for U8X16 {
    type Output = U8X16;

    fn bitand(self, other: U8X16) -> Self::Output {
        self.and(other)
    }
}

impl BitAnd<u8> for U8X16 {
    type Output = U8X16;

    fn bitand(self, other: u8) -> Self::Output {
        self.and_byte(other)
    }
}

impl Add<U8X16> for U8X16 {
    type Output = U8X16;

    fn add(self, other: U8X16) -> Self::Output {
        self.add_other(other)
    }
}

impl Shr<usize> for U8X16 {
    type Output = U8X16;

    fn shr(self, rhs: usize) -> Self::Output {
        self.shift_right(rhs)
    }
}

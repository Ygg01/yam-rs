use crate::util::macros::gen_u8_cmp;
use crate::util::{BitOps, U8X16};
use core::ops::{Add, BitAnd, Index, Shr};

#[derive(Debug, Copy, Clone)]
pub(crate) struct U8X32([u8; 32]);

#[allow(dead_code)]
impl U8X32 {
    fn splat(input: u8) -> Self {
        Self([input; 32])
    }

    pub(crate) fn from_array(input: [u8; 32]) -> Self {
        Self(input)
    }

    pub(crate) fn split(self) -> (U8X16, U8X16) {
        let (low, high) = self.0.split_at(16);
        (unsafe { U8X16::from_array(*low.as_ptr().cast()) }, unsafe {
            U8X16::from_array(*high.as_ptr().cast())
        })
    }

    pub(crate) fn merge(low: U8X16, high: U8X16) -> Self {
        U8X32([
            low[0], low[1], low[2], low[3], low[4], low[5], low[6], low[7], low[8], low[9],
            low[10], low[11], low[12], low[13], low[14], low[15], high[0], high[1], high[2],
            high[3], high[4], high[5], high[6], high[7], high[8], high[9], high[10], high[11],
            high[12], high[13], high[14], high[15],
        ])
    }
}

impl Index<usize> for U8X32 {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

//noinspection ALL
impl BitOps for U8X32 {
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
            | gen_u8_cmp!(self, 16 => cmp)
            | gen_u8_cmp!(self, 17 => cmp)
            | gen_u8_cmp!(self, 18 => cmp)
            | gen_u8_cmp!(self, 19 => cmp)
            | gen_u8_cmp!(self, 20 => cmp)
            | gen_u8_cmp!(self, 21 => cmp)
            | gen_u8_cmp!(self, 22 => cmp)
            | gen_u8_cmp!(self, 23 => cmp)
            | gen_u8_cmp!(self, 24 => cmp)
            | gen_u8_cmp!(self, 25 => cmp)
            | gen_u8_cmp!(self, 26 => cmp)
            | gen_u8_cmp!(self, 27 => cmp)
            | gen_u8_cmp!(self, 28 => cmp)
            | gen_u8_cmp!(self, 29 => cmp)
            | gen_u8_cmp!(self, 30 => cmp)
            | gen_u8_cmp!(self, 31 => cmp)
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
            self[16] & other[16],
            self[17] & other[17],
            self[18] & other[18],
            self[19] & other[19],
            self[20] & other[20],
            self[21] & other[21],
            self[22] & other[22],
            self[23] & other[23],
            self[24] & other[24],
            self[25] & other[25],
            self[26] & other[26],
            self[27] & other[27],
            self[28] & other[28],
            self[29] & other[29],
            self[30] & other[30],
            self[31] & other[31],
        ])
    }

    fn and_byte(self, rhs: u8) -> Self {
        Self(self.0.map(|v| v & rhs))
    }

    fn shift_right(self, rhs: usize) -> Self {
        Self([
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
            self[16] >> rhs,
            self[17] >> rhs,
            self[18] >> rhs,
            self[19] >> rhs,
            self[20] >> rhs,
            self[21] >> rhs,
            self[22] >> rhs,
            self[23] >> rhs,
            self[24] >> rhs,
            self[25] >> rhs,
            self[26] >> rhs,
            self[27] >> rhs,
            self[28] >> rhs,
            self[29] >> rhs,
            self[30] >> rhs,
            self[31] >> rhs,
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
            self[16] + other[16],
            self[17] + other[17],
            self[18] + other[18],
            self[19] + other[19],
            self[20] + other[20],
            self[21] + other[21],
            self[22] + other[22],
            self[23] + other[23],
            self[24] + other[24],
            self[25] + other[25],
            self[26] + other[26],
            self[27] + other[27],
            self[28] + other[28],
            self[29] + other[29],
            self[30] + other[30],
            self[31] + other[31],
        ])
    }
}

impl BitAnd<U8X32> for U8X32 {
    type Output = U8X32;

    fn bitand(self, other: U8X32) -> Self::Output {
        self.and(other)
    }
}

impl Add<U8X32> for U8X32 {
    type Output = U8X32;

    fn add(self, other: U8X32) -> Self::Output {
        self.add_other(other)
    }
}

impl Shr<usize> for U8X32 {
    type Output = U8X32;

    fn shr(self, rhs: usize) -> Self::Output {
        self.shift_right(rhs)
    }
}

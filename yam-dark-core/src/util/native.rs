use core::ops::{BitAnd, Shr};

#[doc(hidden)]
pub fn u8x16_bit(a: [u8; 16]) -> u16 {
    (a[0] & 0b1000_0000 != 0) as u16
        | (((a[1] & 0b1000_0000 != 0) as u16) << 1)
        | (((a[2] & 0b1000_0000 != 0) as u16) << 2)
        | (((a[3] & 0b1000_0000 != 0) as u16) << 3)
        | (((a[4] & 0b1000_0000 != 0) as u16) << 4)
        | (((a[5] & 0b1000_0000 != 0) as u16) << 5)
        | (((a[6] & 0b1000_0000 != 0) as u16) << 6)
        | (((a[7] & 0b1000_0000 != 0) as u16) << 7)
        | (((a[8] & 0b1000_0000 != 0) as u16) << 8)
        | (((a[9] & 0b1000_0000 != 0) as u16) << 9)
        | (((a[10] & 0b1000_0000 != 0) as u16) << 10)
        | (((a[11] & 0b1000_0000 != 0) as u16) << 11)
        | (((a[12] & 0b1000_0000 != 0) as u16) << 12)
        | (((a[13] & 0b1000_0000 != 0) as u16) << 13)
        | (((a[14] & 0b1000_0000 != 0) as u16) << 14)
        | (((a[15] & 0b1000_0000 != 0) as u16) << 15)
}

#[doc(hidden)]
pub fn u8x16_bit_iter(a: [u8; 16], c: u8) -> u16 {
    a.iter().fold(0u16, move |b, x| {
        let m = b << 1;
        let z = if *x == c { 1 } else { 0 };
        m | z
    })
}

/// Returns an `u64` value representing the bitmask of each element in the given u8 that is equal to [cmp]
///
/// # Arguments
///
/// * `a` - The u8 array to compare with the comparison value.
/// * `cmp` - The comparison value to check for equality against each element in the array.
///
/// # Returns
///
/// The u64 value consisting of bits set to 1 where the corresponding element in the array is equal to the comparison value, and 0 otherwise.
///
/// # Examples
///
/// ```
/// use yam_dark_core::util::ux;
///
/// let mut array = [1u8; 64];
/// // Set three values to 2
/// array[2] = 2;
/// array[4] = 2;
/// array[9] = 2;
/// let result = u8x64_eq(array, 2);
/// // Expect to find three instances of number `2`
/// assert_eq!(result, 0b1000010100);
/// ```
#[doc(hidden)]
#[cfg_attr(not(feature = "no-inline"), inline)]
pub fn u8x64_eq(a: [u8; 64], cmp: u8) -> u64 {
    (if a[0] == cmp { 1 } else { 0 })
        | (if a[1] == cmp { 1 << 1 } else { 0 })
        | (if a[2] == cmp { 1 << 2 } else { 0 })
        | (if a[3] == cmp { 1 << 3 } else { 0 })
        | (if a[4] == cmp { 1 << 4 } else { 0 })
        | (if a[5] == cmp { 1 << 5 } else { 0 })
        | (if a[6] == cmp { 1 << 6 } else { 0 })
        | (if a[7] == cmp { 1 << 7 } else { 0 })
        | (if a[8] == cmp { 1 << 8 } else { 0 })
        | (if a[9] == cmp { 1 << 9 } else { 0 })
        | (if a[10] == cmp { 1 << 10 } else { 0 })
        | (if a[11] == cmp { 1 << 11 } else { 0 })
        | (if a[12] == cmp { 1 << 12 } else { 0 })
        | (if a[13] == cmp { 1 << 13 } else { 0 })
        | (if a[14] == cmp { 1 << 14 } else { 0 })
        | (if a[15] == cmp { 1 << 15 } else { 0 })
        | (if a[16] == cmp { 1 << 16 } else { 0 })
        | (if a[17] == cmp { 1 << 17 } else { 0 })
        | (if a[18] == cmp { 1 << 18 } else { 0 })
        | (if a[19] == cmp { 1 << 19 } else { 0 })
        | (if a[20] == cmp { 1 << 20 } else { 0 })
        | (if a[21] == cmp { 1 << 21 } else { 0 })
        | (if a[22] == cmp { 1 << 22 } else { 0 })
        | (if a[23] == cmp { 1 << 23 } else { 0 })
        | (if a[24] == cmp { 1 << 24 } else { 0 })
        | (if a[25] == cmp { 1 << 25 } else { 0 })
        | (if a[26] == cmp { 1 << 26 } else { 0 })
        | (if a[27] == cmp { 1 << 27 } else { 0 })
        | (if a[28] == cmp { 1 << 28 } else { 0 })
        | (if a[29] == cmp { 1 << 29 } else { 0 })
        | (if a[30] == cmp { 1 << 30 } else { 0 })
        | (if a[31] == cmp { 1 << 31 } else { 0 })
        | (if a[32] == cmp { 1 << 32 } else { 0 })
        | (if a[33] == cmp { 1 << 33 } else { 0 })
        | (if a[34] == cmp { 1 << 34 } else { 0 })
        | (if a[35] == cmp { 1 << 35 } else { 0 })
        | (if a[36] == cmp { 1 << 36 } else { 0 })
        | (if a[37] == cmp { 1 << 37 } else { 0 })
        | (if a[38] == cmp { 1 << 38 } else { 0 })
        | (if a[39] == cmp { 1 << 39 } else { 0 })
        | (if a[40] == cmp { 1 << 40 } else { 0 })
        | (if a[41] == cmp { 1 << 41 } else { 0 })
        | (if a[42] == cmp { 1 << 42 } else { 0 })
        | (if a[43] == cmp { 1 << 43 } else { 0 })
        | (if a[44] == cmp { 1 << 44 } else { 0 })
        | (if a[45] == cmp { 1 << 45 } else { 0 })
        | (if a[46] == cmp { 1 << 46 } else { 0 })
        | (if a[47] == cmp { 1 << 47 } else { 0 })
        | (if a[48] == cmp { 1 << 48 } else { 0 })
        | (if a[49] == cmp { 1 << 49 } else { 0 })
        | (if a[50] == cmp { 1 << 50 } else { 0 })
        | (if a[51] == cmp { 1 << 51 } else { 0 })
        | (if a[52] == cmp { 1 << 52 } else { 0 })
        | (if a[53] == cmp { 1 << 53 } else { 0 })
        | (if a[54] == cmp { 1 << 54 } else { 0 })
        | (if a[55] == cmp { 1 << 55 } else { 0 })
        | (if a[56] == cmp { 1 << 56 } else { 0 })
        | (if a[57] == cmp { 1 << 57 } else { 0 })
        | (if a[58] == cmp { 1 << 58 } else { 0 })
        | (if a[59] == cmp { 1 << 59 } else { 0 })
        | (if a[60] == cmp { 1 << 60 } else { 0 })
        | (if a[61] == cmp { 1 << 61 } else { 0 })
        | (if a[62] == cmp { 1 << 62 } else { 0 })
        | (if a[63] == cmp { 1 << 63 } else { 0 })
}

#[doc(hidden)]
#[cfg_attr(not(feature = "no-inline"), inline)]
pub fn u8x64_lteq(a: [u8; 64], cmp: u8) -> u64 {
    (if a[0] <= cmp { 1 } else { 0 })
        | (if a[1] <= cmp { 1 << 1 } else { 0 })
        | (if a[2] <= cmp { 1 << 2 } else { 0 })
        | (if a[3] <= cmp { 1 << 3 } else { 0 })
        | (if a[4] <= cmp { 1 << 4 } else { 0 })
        | (if a[5] <= cmp { 1 << 5 } else { 0 })
        | (if a[6] <= cmp { 1 << 6 } else { 0 })
        | (if a[7] <= cmp { 1 << 7 } else { 0 })
        | (if a[8] <= cmp { 1 << 8 } else { 0 })
        | (if a[9] <= cmp { 1 << 9 } else { 0 })
        | (if a[10] <= cmp { 1 << 10 } else { 0 })
        | (if a[11] <= cmp { 1 << 11 } else { 0 })
        | (if a[12] <= cmp { 1 << 12 } else { 0 })
        | (if a[13] <= cmp { 1 << 13 } else { 0 })
        | (if a[14] <= cmp { 1 << 14 } else { 0 })
        | (if a[15] <= cmp { 1 << 15 } else { 0 })
        | (if a[16] <= cmp { 1 << 16 } else { 0 })
        | (if a[17] <= cmp { 1 << 17 } else { 0 })
        | (if a[18] <= cmp { 1 << 18 } else { 0 })
        | (if a[19] <= cmp { 1 << 19 } else { 0 })
        | (if a[20] <= cmp { 1 << 20 } else { 0 })
        | (if a[21] <= cmp { 1 << 21 } else { 0 })
        | (if a[22] <= cmp { 1 << 22 } else { 0 })
        | (if a[23] <= cmp { 1 << 23 } else { 0 })
        | (if a[24] <= cmp { 1 << 24 } else { 0 })
        | (if a[25] <= cmp { 1 << 25 } else { 0 })
        | (if a[26] <= cmp { 1 << 26 } else { 0 })
        | (if a[27] <= cmp { 1 << 27 } else { 0 })
        | (if a[28] <= cmp { 1 << 28 } else { 0 })
        | (if a[29] <= cmp { 1 << 29 } else { 0 })
        | (if a[30] <= cmp { 1 << 30 } else { 0 })
        | (if a[31] <= cmp { 1 << 31 } else { 0 })
        | (if a[32] <= cmp { 1 << 32 } else { 0 })
        | (if a[33] <= cmp { 1 << 33 } else { 0 })
        | (if a[34] <= cmp { 1 << 34 } else { 0 })
        | (if a[35] <= cmp { 1 << 35 } else { 0 })
        | (if a[36] <= cmp { 1 << 36 } else { 0 })
        | (if a[37] <= cmp { 1 << 37 } else { 0 })
        | (if a[38] <= cmp { 1 << 38 } else { 0 })
        | (if a[39] <= cmp { 1 << 39 } else { 0 })
        | (if a[40] <= cmp { 1 << 40 } else { 0 })
        | (if a[41] <= cmp { 1 << 41 } else { 0 })
        | (if a[42] <= cmp { 1 << 42 } else { 0 })
        | (if a[43] <= cmp { 1 << 43 } else { 0 })
        | (if a[44] <= cmp { 1 << 44 } else { 0 })
        | (if a[45] <= cmp { 1 << 45 } else { 0 })
        | (if a[46] <= cmp { 1 << 46 } else { 0 })
        | (if a[47] <= cmp { 1 << 47 } else { 0 })
        | (if a[48] <= cmp { 1 << 48 } else { 0 })
        | (if a[49] <= cmp { 1 << 49 } else { 0 })
        | (if a[50] <= cmp { 1 << 50 } else { 0 })
        | (if a[51] <= cmp { 1 << 51 } else { 0 })
        | (if a[52] <= cmp { 1 << 52 } else { 0 })
        | (if a[53] <= cmp { 1 << 53 } else { 0 })
        | (if a[54] <= cmp { 1 << 54 } else { 0 })
        | (if a[55] <= cmp { 1 << 55 } else { 0 })
        | (if a[56] <= cmp { 1 << 56 } else { 0 })
        | (if a[57] <= cmp { 1 << 57 } else { 0 })
        | (if a[58] <= cmp { 1 << 58 } else { 0 })
        | (if a[59] <= cmp { 1 << 59 } else { 0 })
        | (if a[60] <= cmp { 1 << 60 } else { 0 })
        | (if a[61] <= cmp { 1 << 61 } else { 0 })
        | (if a[62] <= cmp { 1 << 62 } else { 0 })
        | (if a[63] <= cmp { 1 << 63 } else { 0 })
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

    #[inline]
    pub fn comp_all(&self, cmp: u8) -> U8X16 {
        U8X16::from_array([
            if self.0[0] == cmp { 0xFF } else { 0x00 },
            if self.0[1] == cmp { 0xFF } else { 0x00 },
            if self.0[2] == cmp { 0xFF } else { 0x00 },
            if self.0[3] == cmp { 0xFF } else { 0x00 },
            if self.0[4] == cmp { 0xFF } else { 0x00 },
            if self.0[5] == cmp { 0xFF } else { 0x00 },
            if self.0[6] == cmp { 0xFF } else { 0x00 },
            if self.0[7] == cmp { 0xFF } else { 0x00 },
            if self.0[8] == cmp { 0xFF } else { 0x00 },
            if self.0[9] == cmp { 0xFF } else { 0x00 },
            if self.0[10] == cmp { 0xFF } else { 0x00 },
            if self.0[11] == cmp { 0xFF } else { 0x00 },
            if self.0[12] == cmp { 0xFF } else { 0x00 },
            if self.0[13] == cmp { 0xFF } else { 0x00 },
            if self.0[14] == cmp { 0xFF } else { 0x00 },
            if self.0[15] == cmp { 0xFF } else { 0x00 },
        ])
    }

    #[inline]
    pub fn to_u16(&self) -> u16 {
        (self.0[0] & 0b1000_0000 != 0) as u16
            | (((self.0[1] & 0b1000_0000 != 0) as u16) << 1)
            | (((self.0[2] & 0b1000_0000 != 0) as u16) << 2)
            | (((self.0[3] & 0b1000_0000 != 0) as u16) << 3)
            | (((self.0[4] & 0b1000_0000 != 0) as u16) << 4)
            | (((self.0[5] & 0b1000_0000 != 0) as u16) << 5)
            | (((self.0[6] & 0b1000_0000 != 0) as u16) << 6)
            | (((self.0[7] & 0b1000_0000 != 0) as u16) << 7)
            | (((self.0[8] & 0b1000_0000 != 0) as u16) << 8)
            | (((self.0[9] & 0b1000_0000 != 0) as u16) << 9)
            | (((self.0[10] & 0b1000_0000 != 0) as u16) << 10)
            | (((self.0[11] & 0b1000_0000 != 0) as u16) << 11)
            | (((self.0[12] & 0b1000_0000 != 0) as u16) << 12)
            | (((self.0[13] & 0b1000_0000 != 0) as u16) << 13)
            | (((self.0[14] & 0b1000_0000 != 0) as u16) << 14)
            | (((self.0[15] & 0b1000_0000 != 0) as u16) << 15)
    }

    /// Creates a new instance of `Self` by converting a slice of `u8` into the desired type.
    #[inline]
    pub unsafe fn from_slice(input: &[u8]) -> Self {
        U8X16([
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
        ])
    }

    pub fn merge(input0: U8X16, input1: U8X16, input2: U8X16, input3: U8X16) -> [u8; 64] {
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

#[doc(hidden)]
#[inline]
pub fn u8x16_swizzle(mask: [u8; 16], x: U8X16) -> U8X16 {
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

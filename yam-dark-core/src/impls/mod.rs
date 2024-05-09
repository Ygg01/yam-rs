// pub(crate) use avx_stage1::AvxScanner;
pub use native_stage1::NativeScanner;

// mod avx_stage1;
mod native_stage1;

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
/// use yam_dark_core::u8x64_eq;
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

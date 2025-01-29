use core::ops::{Add, AddAssign, BitAnd, Shr};

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
/// use yam_dark_core::SIMD_CHUNK_LENGTH;
/// use yam_dark_core::util::u8x64_eq;
///
/// let mut array = [1u8; 64];
/// // Set three values to 2
/// array[2] = 2;
/// array[4] = 2;
/// array[9] = 2;
/// let result = u8x64_eq(&array, 2);
/// // Expect to find three instances of number `2`
/// assert_eq!(result, 0b1000010100);
/// ```
#[doc(hidden)]
#[cfg_attr(not(feature = "no-inline"), inline)]
#[must_use]
pub fn u8x64_eq(a: &[u8; 64], cmp: u8) -> u64 {
    u64::from(a[0] == cmp)
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

/// Checks if each element in a [u8; 64] array is less than or equal to the given `cmp` value.
///
/// # Arguments
///
/// * `a` - An array of 64 unsigned 8-bit integers.
/// * `cmp` - The value to compare each element against.
///
/// # Returns
///
/// An unsigned 64-bit integer where each bit represents the comparison result for the corresponding element in the array.
///
/// # Example
///
/// ```
/// use yam_dark_core::util::u8x64_lteq;
///
/// let a = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64];
/// let cmp = 10;
/// let result = u8x64_lteq(a, cmp);
/// assert_eq!(result, 0b0000000000000000000000000000000000000000000000000000001111111111);
/// ```
#[cfg_attr(not(feature = "no-inline"), inline)]
#[must_use]
pub fn u8x64_lteq(a: [u8; 64], cmp: u8) -> u64 {
    u64::from(a[0] <= cmp)
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

/// A struct representing a vector of 16 `u8` values.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct U8X16(pub [u8; 16]);

impl U8X16 {
    #[inline]
    /// Creates a new `U8X16` instance where all elements are set to the specified input value.
    ///
    /// This function is inlined to improve performance.
    ///
    /// # Arguments
    ///
    /// * `input` - A `u8` value that will be used to set all elements of the `U8X16` instance.
    ///
    /// # Returns
    ///
    /// A `U8X16` instance where each element is initialized to the `input` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use yam_dark_core::util::U8X16;
    /// let value = 5;
    /// let vector = U8X16::splat(value);
    /// assert_eq!(vector, U8X16([5; 16]));
    /// ```
    ///
    #[must_use]
    pub fn splat(input: u8) -> Self {
        U8X16([input; 16])
    }

    /// Conversion method that takes an array of sixteen `u8` and returns an [U8X16]
    ///
    /// # Arguments
    ///
    /// * `input`: array of sixteen bytes (`u8`).
    ///
    /// returns: [U8X16]
    #[inline]
    #[must_use]
    pub fn from_array(input: [u8; 16]) -> Self {
        U8X16(input)
    }

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
    /// ```
    /// use yam_dark_core::util::U8X16;
    /// let vector = U8X16::from_array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
    /// let result = vector.comp_all(10);
    /// assert_eq!(result, U8X16::from_array([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]));
    /// ```
    #[inline]
    #[must_use]
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

    /// Converts [`self::U8X16`] into a 16-bit unsigned integer bitmask.
    /// The most significant bit of each byte is used to form the 16-bit bitmask.
    /// The resulting bitmask will have its bits set according to the most significant bit
    /// of each byte in the byte array, starting from the least significant bit.
    ///
    /// # Arguments
    ///
    /// * `self` - The [`self::U8X16`] structure to convert to bitmask.
    ///
    /// # Returns
    ///
    /// The converted 64-bit bitmask integer.
    #[inline]
    #[must_use]
    pub fn to_bitmask64(&self) -> u64 {
        u64::from(self.0[0] & 0b1000_0000 != 0)
            | (u64::from(self.0[1] & 0b1000_0000 != 0) << 1)
            | (u64::from(self.0[2] & 0b1000_0000 != 0) << 2)
            | (u64::from(self.0[3] & 0b1000_0000 != 0) << 3)
            | (u64::from(self.0[4] & 0b1000_0000 != 0) << 4)
            | (u64::from(self.0[5] & 0b1000_0000 != 0) << 5)
            | (u64::from(self.0[6] & 0b1000_0000 != 0) << 6)
            | (u64::from(self.0[7] & 0b1000_0000 != 0) << 7)
            | (u64::from(self.0[8] & 0b1000_0000 != 0) << 8)
            | (u64::from(self.0[9] & 0b1000_0000 != 0) << 9)
            | (u64::from(self.0[10] & 0b1000_0000 != 0) << 10)
            | (u64::from(self.0[11] & 0b1000_0000 != 0) << 11)
            | (u64::from(self.0[12] & 0b1000_0000 != 0) << 12)
            | (u64::from(self.0[13] & 0b1000_0000 != 0) << 13)
            | (u64::from(self.0[14] & 0b1000_0000 != 0) << 14)
            | (u64::from(self.0[15] & 0b1000_0000 != 0) << 15)
    }

    /// Creates a new `U8X16` instance from a slice of `u8` values.
    ///
    /// # Safety
    ///
    /// This function is marked as `unsafe` because it dereferences raw pointers and
    /// may result in undefined behavior if the input slice *MUST BE AT LEAST 16*  bytes long.
    ///
    /// # Arguments
    ///
    /// * `input` - A slice of `u8` values from which to create the `U8X16` instance. Input must be at last 16 bytes long.
    ///
    /// # Returns
    ///
    /// A new `U8X16` instance created from the input slice.
    ///
    /// # Example
    ///
    /// ```
    /// # use yam_dark_core::util::U8X16;
    ///
    /// let input = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    /// let result = unsafe { U8X16::from_slice(input) };
    /// ```
    #[inline]
    #[must_use]
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

impl Add<U8X16> for U8X16 {
    type Output = U8X16;

    fn add(self, rhs: U8X16) -> Self::Output {
        U8X16([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
            self.0[3] + rhs.0[3],
            self.0[4] + rhs.0[4],
            self.0[5] + rhs.0[5],
            self.0[6] + rhs.0[6],
            self.0[7] + rhs.0[7],
            self.0[8] + rhs.0[8],
            self.0[9] + rhs.0[9],
            self.0[10] + rhs.0[10],
            self.0[11] + rhs.0[11],
            self.0[12] + rhs.0[12],
            self.0[13] + rhs.0[13],
            self.0[14] + rhs.0[14],
            self.0[15] + rhs.0[15],
        ])
    }
}

impl AddAssign for U8X16 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
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
#[must_use]
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

#[doc(hidden)]
#[inline]
#[must_use]
pub fn mask_merge(v0: U8X16, v1: U8X16, v2: U8X16, v3: U8X16) -> [u32; 64] {
    [
        // first 16 cols
        u32::from(v0.0[0]),
        u32::from(v0.0[1]),
        u32::from(v0.0[2]),
        u32::from(v0.0[3]),
        u32::from(v0.0[4]),
        u32::from(v0.0[5]),
        u32::from(v0.0[6]),
        u32::from(v0.0[7]),
        u32::from(v0.0[8]),
        u32::from(v0.0[9]),
        u32::from(v0.0[10]),
        u32::from(v0.0[11]),
        u32::from(v0.0[12]),
        u32::from(v0.0[13]),
        u32::from(v0.0[14]),
        u32::from(v0.0[15]),
        // second 16 cols
        u32::from(v1.0[0]),
        u32::from(v1.0[1]),
        u32::from(v1.0[2]),
        u32::from(v1.0[3]),
        u32::from(v1.0[4]),
        u32::from(v1.0[5]),
        u32::from(v1.0[6]),
        u32::from(v1.0[7]),
        u32::from(v1.0[8]),
        u32::from(v1.0[9]),
        u32::from(v1.0[10]),
        u32::from(v1.0[11]),
        u32::from(v1.0[12]),
        u32::from(v1.0[13]),
        u32::from(v1.0[14]),
        u32::from(v1.0[15]),
        // third 16 cols
        u32::from(v2.0[0]),
        u32::from(v2.0[1]),
        u32::from(v2.0[2]),
        u32::from(v2.0[3]),
        u32::from(v2.0[4]),
        u32::from(v2.0[5]),
        u32::from(v2.0[6]),
        u32::from(v2.0[7]),
        u32::from(v2.0[8]),
        u32::from(v2.0[9]),
        u32::from(v2.0[10]),
        u32::from(v2.0[11]),
        u32::from(v2.0[12]),
        u32::from(v2.0[13]),
        u32::from(v2.0[14]),
        u32::from(v2.0[15]),
        // fourth 16 cols
        u32::from(v3.0[0]),
        u32::from(v3.0[1]),
        u32::from(v3.0[2]),
        u32::from(v3.0[3]),
        u32::from(v3.0[4]),
        u32::from(v3.0[5]),
        u32::from(v3.0[6]),
        u32::from(v3.0[7]),
        u32::from(v3.0[8]),
        u32::from(v3.0[9]),
        u32::from(v3.0[10]),
        u32::from(v3.0[11]),
        u32::from(v3.0[12]),
        u32::from(v3.0[13]),
        u32::from(v3.0[14]),
        u32::from(v3.0[15]),
    ]
}

#[derive(Copy, Clone)]
pub struct U8X8(pub [u8; 8]);

impl U8X8 {
    #[inline]
    #[must_use]
    pub fn from_array(input: [u8; 8]) -> Self {
        // Safety:
        // This is perfectly safe since bounds are known at compile time
        // But compiler isn't smart enough to figure it out.
        unsafe {
            U8X8([
                *input.get_unchecked(0),
                *input.get_unchecked(1),
                *input.get_unchecked(2),
                *input.get_unchecked(3),
                *input.get_unchecked(4),
                *input.get_unchecked(5),
                *input.get_unchecked(6),
                *input.get_unchecked(7),
            ])
        }
    }

    #[inline]
    #[must_use]
    pub fn add_offset_and_mask(&self, mask: Self, offset: u32) -> [u32; 8] {
        // SAFETY: This is safe because self.0 is [u8; 8] so this will never cause UB.
        [
            if mask.0[0] == 0 {
                unsafe { u32::from(*self.0.get_unchecked(0)) + offset }
            } else {
                u32::from(self.0[0])
            },
            if mask.0[1] == 0 {
                unsafe { u32::from(*self.0.get_unchecked(1)) + offset }
            } else {
                u32::from(self.0[1])
            },
            if mask.0[2] == 0 {
                unsafe { u32::from(*self.0.get_unchecked(2)) + offset }
            } else {
                u32::from(self.0[2])
            },
            if mask.0[3] == 0 {
                unsafe { u32::from(*self.0.get_unchecked(3)) + offset }
            } else {
                u32::from(self.0[3])
            },
            if mask.0[4] == 0 {
                unsafe { u32::from(*self.0.get_unchecked(4)) + offset }
            } else {
                u32::from(self.0[4])
            },
            if mask.0[5] == 0 {
                unsafe { u32::from(*self.0.get_unchecked(5)) + offset }
            } else {
                u32::from(self.0[5])
            },
            if mask.0[6] == 0 {
                unsafe { u32::from(*self.0.get_unchecked(6)) + offset }
            } else {
                u32::from(self.0[6])
            },
            if mask.0[7] == 0 {
                unsafe { u32::from(*self.0.get_unchecked(7)) + offset }
            } else {
                u32::from(self.0[7])
            },
        ]
    }

    #[inline]
    #[must_use]
    pub fn to_bitmask(&self) -> u8 {
        u8::from(self.0[0] & 0b1000_0000 != 0)
            | u8::from(self.0[1] & 0b1000_0000 != 0)
            | u8::from(self.0[2] & 0b1000_0000 != 0)
            | u8::from(self.0[3] & 0b1000_0000 != 0)
            | u8::from(self.0[4] & 0b1000_0000 != 0)
            | u8::from(self.0[5] & 0b1000_0000 != 0)
            | u8::from(self.0[6] & 0b1000_0000 != 0)
            | u8::from(self.0[7] & 0b1000_0000 != 0)
    }
}

use crate::impls::native::{ChunkedUtf8ValidatorImp, SimdInput, V128};
use crate::stage1::Stage1Parse;
use crate::stage2::YamlIndexes;

/// Convert a [u8] value into a  [V128] value, by repeating it sixteen times.
const fn u8x16_splat(n: u8) -> V128 {
    [n, n, n, n, n, n, n, n, n, n, n, n, n, n, n, n]
}

const fn u8x16_eq(a: V128, b: V128) -> V128 {
    [
        bool_to_u8(a[0] == b[0]),
        bool_to_u8(a[1] == b[1]),
        bool_to_u8(a[2] == b[2]),
        bool_to_u8(a[3] == b[3]),
        bool_to_u8(a[4] == b[4]),
        bool_to_u8(a[5] == b[5]),
        bool_to_u8(a[6] == b[6]),
        bool_to_u8(a[7] == b[7]),
        bool_to_u8(a[8] == b[8]),
        bool_to_u8(a[9] == b[9]),
        bool_to_u8(a[10] == b[10]),
        bool_to_u8(a[11] == b[11]),
        bool_to_u8(a[12] == b[12]),
        bool_to_u8(a[13] == b[13]),
        bool_to_u8(a[14] == b[14]),
        bool_to_u8(a[15] == b[15]),
    ]
}

/// Converts a bool value to a `0b1111_1111` if `true` and
/// `0b0000_0000` if `false`.
const fn bool_to_u8(b: bool) -> u8 {
    if b {
        0xFF
    } else {
        0x00
    }
}

const fn u8x16_shr(a: V128, n: i32) -> V128 {
    [
        a[0] >> n,
        a[1] >> n,
        a[2] >> n,
        a[3] >> n,
        a[4] >> n,
        a[5] >> n,
        a[6] >> n,
        a[7] >> n,
        a[8] >> n,
        a[9] >> n,
        a[10] >> n,
        a[11] >> n,
        a[12] >> n,
        a[13] >> n,
        a[14] >> n,
        a[15] >> n,
    ]
}


const fn u8x16_bitmask(a: V128) -> u16 {
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

const fn u8x16_swizzle(a: V128, s: V128) -> [u8; 16] {
    [
        if s[0] > 0x0f {
            0
        } else {
            a[(s[0] & 0x0f) as usize]
        },
        if s[1] > 0x0f {
            0
        } else {
            a[(s[1] & 0x0f) as usize]
        },
        if s[2] > 0x0f {
            0
        } else {
            a[(s[2] & 0x0f) as usize]
        },
        if s[3] > 0x0f {
            0
        } else {
            a[(s[3] & 0x0f) as usize]
        },
        if s[4] > 0x0f {
            0
        } else {
            a[(s[4] & 0x0f) as usize]
        },
        if s[5] > 0x0f {
            0
        } else {
            a[(s[5] & 0x0f) as usize]
        },
        if s[6] > 0x0f {
            0
        } else {
            a[(s[6] & 0x0f) as usize]
        },
        if s[7] > 0x0f {
            0
        } else {
            a[(s[7] & 0x0f) as usize]
        },
        if s[8] > 0x0f {
            0
        } else {
            a[(s[8] & 0x0f) as usize]
        },
        if s[9] > 0x0f {
            0
        } else {
            a[(s[9] & 0x0f) as usize]
        },
        if s[10] > 0x0f {
            0
        } else {
            a[(s[10] & 0x0f) as usize]
        },
        if s[11] > 0x0f {
            0
        } else {
            a[(s[11] & 0x0f) as usize]
        },
        if s[12] > 0x0f {
            0
        } else {
            a[(s[12] & 0x0f) as usize]
        },
        if s[13] > 0x0f {
            0
        } else {
            a[(s[13] & 0x0f) as usize]
        },
        if s[14] > 0x0f {
            0
        } else {
            a[(s[14] & 0x0f) as usize]
        },
        if s[15] > 0x0f {
            0
        } else {
            a[(s[15] & 0x0f) as usize]
        },
    ]
}

const fn v128_and(a: V128, b: V128) -> V128 {
    [
        a[0] & b[0],
        a[1] & b[1],
        a[2] & b[2],
        a[3] & b[3],
        a[4] & b[4],
        a[5] & b[5],
        a[6] & b[6],
        a[7] & b[7],
        a[8] & b[8],
        a[9] & b[9],
        a[10] & b[10],
        a[11] & b[11],
        a[12] & b[12],
        a[13] & b[13],
        a[14] & b[14],
        a[15] & b[15],
    ]
}

const fn u8x16_le(a: V128, b: V128) -> V128 {
    [
        bool_to_u8(a[0] <= b[0]),
        bool_to_u8(a[1] <= b[1]),
        bool_to_u8(a[2] <= b[2]),
        bool_to_u8(a[3] <= b[3]),
        bool_to_u8(a[4] <= b[4]),
        bool_to_u8(a[5] <= b[5]),
        bool_to_u8(a[6] <= b[6]),
        bool_to_u8(a[7] <= b[7]),
        bool_to_u8(a[8] <= b[8]),
        bool_to_u8(a[9] <= b[9]),
        bool_to_u8(a[10] <= b[10]),
        bool_to_u8(a[11] <= b[11]),
        bool_to_u8(a[12] <= b[12]),
        bool_to_u8(a[13] <= b[13]),
        bool_to_u8(a[14] <= b[14]),
        bool_to_u8(a[15] <= b[15]),
    ]
}


impl Stage1Parse for SimdInput {
    type Utf8Validator = ChunkedUtf8ValidatorImp;
    type SimdRepresentation = V128;

    unsafe fn new(ptr: &[u8]) -> Self {
        SimdInput {
            v0: *(ptr.as_ptr().cast::<V128>()),
            v1: *(ptr.as_ptr().add(16).cast::<V128>()),
            v2: *(ptr.as_ptr().add(32).cast::<V128>()),
            v3: *(ptr.as_ptr().add(48).cast::<V128>()),
        }
    }

    unsafe fn compute_quote_mask(quote_bits: u64) -> u64 {
        todo!()
    }

    unsafe fn cmp_mask_against_input(&self, m: u8) -> u64 {
        todo!()
    }

    unsafe fn flatten_bits(base: &mut YamlIndexes, idx: u32, mut bits: u64) {
        todo!()
    }

    unsafe fn find_whitespace_and_structurals(&self, whitespace: &mut u64, structurals: &mut u64) {
        todo!()
    }

    unsafe fn unsigned_lteq_against_input(&self, max_val: Self::SimdRepresentation) -> u64 {
        let cmp_res_0 = u8x16_le(self.v0, max_val);
        let res_0 = u8x16_bitmask(cmp_res_0) as u64;
        let cmp_res_1 = u8x16_le(self.v1, max_val);
        let res_1 = u8x16_bitmask(cmp_res_1) as u64;
        let cmp_res_2 = u8x16_le(self.v2, max_val);
        let res_2 = u8x16_bitmask(cmp_res_2) as u64;
        let cmp_res_3 = u8x16_le(self.v3, max_val);
        let res_3 = u8x16_bitmask(cmp_res_3) as u64;
        res_0 | (res_1 << 16) | (res_2 << 32) | (res_3 << 48)
    }

    unsafe fn fill_s8(n: i8) -> Self::SimdRepresentation {
        u8x16_splat(n as u8)
    }

    unsafe fn zero() -> Self::SimdRepresentation {
        u8x16_splat(0)
    }
}





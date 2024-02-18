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

    unsafe fn flatten_bits(base: &mut YamlIndexes, idx: u32, bits: u64) {
        todo!()
    }

    unsafe fn find_whitespace_and_structurals(&self, whitespace: &mut u64, structurals: &mut u64) {
        todo!()
    }

    unsafe fn unsigned_lteq_against_input(&self, max_val: Self::SimdRepresentation) -> u64 {
        todo!()
    }

    unsafe fn fill_s8(n: i8) -> Self::SimdRepresentation {
        u8x16_splat(n as u8)
    }

    unsafe fn zero() -> Self::SimdRepresentation {
        u8x16_splat(0)
    }
}

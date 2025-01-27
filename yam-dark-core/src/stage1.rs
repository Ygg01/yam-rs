use std::mem::transmute;
use simdutf8::basic::imp::ChunkedUtf8Validator;

pub(crate) trait Stage1Parse {
    type Utf8Validator: ChunkedUtf8Validator;
    type SimdRepresentation;

    unsafe fn new(ptr: &[u8]) -> Self;

    unsafe fn compute_quote_mask(quote_bits: u64) -> u64;

    unsafe fn cmp_mask_against_input(&self, m: u8) -> u64;

    unsafe fn unsigned_lteq_against_input(&self, maxval: Self::SimdRepresentation) -> u64;

    unsafe fn find_whitespace_and_structurals(&self, whitespace: &mut u64, structurals: &mut u64);

    unsafe fn flatten_bits(base: &mut Vec<u32>, idx: u32, bits: u64);

    unsafe fn fill_s8(n: i8) -> Self::SimdRepresentation;

    unsafe fn zero() -> Self::SimdRepresentation;

}
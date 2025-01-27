use crate::impls::native::{ChunkedUtf8ValidatorImp, SimdInput, V128};
use crate::stage1::Stage1Parse;
use crate::stage2::YamlIndexes;

impl Stage1Parse for SimdInput {
    type Utf8Validator = ChunkedUtf8ValidatorImp;
    type SimdRepresentation = V128;

    unsafe fn new(ptr: &[u8]) -> Self {
        todo!()
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
        todo!()
    }

    unsafe fn zero() -> Self::SimdRepresentation {
        todo!()
    }
}

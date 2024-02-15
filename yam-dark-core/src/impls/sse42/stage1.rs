use std::arch::x86_64::__m128i;
use crate::impls::sse42::SimdInput;
use crate::stage1::Stage1Parse;

impl Stage1Parse for SimdInput {
    type Utf8Validator = simdutf8::basic::imp::x86::avx2::ChunkedUtf8ValidatorImp;
    type SimdRepresentation = __m128i;

    unsafe fn new(ptr: &[u8]) -> Self {
        todo!()
    }
}

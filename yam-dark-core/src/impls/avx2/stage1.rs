#![allow(dead_code)]
use crate::stage1::Stage1Parse;
#[cfg(target_arch = "x86")]
use std::arch::x86::__m256i;
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::__m256i;
use crate::impls::avx2::SimdInput;


impl Stage1Parse for SimdInput {
    type Utf8Validator = simdutf8::basic::imp::x86::avx2::ChunkedUtf8ValidatorImp;
    type SimdRepresentation = __m256i;

    unsafe fn new(ptr: &[u8]) -> Self {
        todo!()
    }
}

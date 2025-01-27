use core::arch::x86_64::__m256i;

use simdutf8::basic::imp::ChunkedUtf8Validator;

use crate::tokenizer::stage1::Stage1Scanner;

pub(crate) struct AvxScanner {}

impl Stage1Scanner for AvxScanner {
    type SimdType = [__m256i; 2];
    type Validator = simdutf8::basic::imp::x86::avx2::ChunkedUtf8ValidatorImp;

    fn validator() -> Self::Validator {
        unsafe { simdutf8::basic::imp::x86::avx2::ChunkedUtf8ValidatorImp::new() }
    }

    fn from_chunk(_values: &[u8; 64]) -> Self {
        todo!()
    }

    fn cmp_ascii_to_input(&self, _cmp: u8) -> u64 {
        todo!()
    }

}

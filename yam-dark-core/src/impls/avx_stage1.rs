use crate::tokenizer::stage1::{Stage1Scanner, YamlBlockState};
use crate::tokenizer::stage2::{Buffer, YamlParserState};
use crate::ParseResult;
use core::arch::x86_64::__m256i;
use simdutf8::basic::imp::ChunkedUtf8Validator;

pub(crate) struct AvxScanner {}

impl Stage1Scanner for AvxScanner {
    type SimdType = [__m256i; 2];
    type Validator = simdutf8::basic::imp::x86::avx2::ChunkedUtf8ValidatorImp;

    fn validator() -> Self::Validator {
        unsafe { simdutf8::basic::imp::x86::avx2::ChunkedUtf8ValidatorImp::new() }
    }

    fn from_chunk(values: &[u8; 64]) -> Self {
        todo!()
    }


    fn next<'i, T: Buffer>(
        _chunk: &[u8; 64],
        _buffers: &'i mut T,
        _state: &'i mut YamlParserState,
    ) -> ParseResult<YamlBlockState> {
        todo!()
    }
}

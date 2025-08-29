use crate::{Stage1Scanner, YamlCharacterChunk, YamlChunkState};

use crate::tokenizer::YamlStructurals;
#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m256i, _mm256_add_epi32, _mm256_and_si256, _mm256_cmpeq_epi8, _mm256_loadu_si256,
    _mm256_max_epu8, _mm256_movemask_epi8, _mm256_set1_epi8, _mm256_set_epi32, _mm256_setr_epi8,
    _mm256_setzero_si256, _mm256_shuffle_epi8, _mm256_srli_epi32, _mm256_storeu_si256,
    _mm_clmulepi64_si128, _mm_cvtsi128_si64, _mm_set1_epi8, _mm_set_epi64x,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::__m256i;
use simdutf8::basic::imp::x86::avx2::ChunkedUtf8ValidatorImp;
use simdutf8::basic::imp::ChunkedUtf8Validator;

#[doc(hidden)]
pub struct AvxScanner {
    _v0: __m256i,
    _v1: __m256i,
}

unsafe impl Stage1Scanner for AvxScanner {
    type SimdType = __m256i;
    type Validator = ChunkedUtf8ValidatorImp;

    fn validator() -> impl ChunkedUtf8Validator {
        unsafe { ChunkedUtf8ValidatorImp::new() }
    }

    fn from_chunk(_values: &[u8; 64]) -> Self {
        todo!()
    }

    fn cmp_ascii_to_input(&self, _m: u8) -> u64 {
        todo!()
    }

    fn unsigned_lteq_against_splat(&self, _cmp: u8) -> u64 {
        todo!()
    }

    fn classify_yaml_characters(&self) -> YamlCharacterChunk {
        todo!()
    }

    fn flatten_bits_yaml(_chunk_state: &YamlChunkState, _base: &mut YamlStructurals) {
        todo!()
    }
}

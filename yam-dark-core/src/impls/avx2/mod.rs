use std::arch::x86_64::__m256i;

mod stage1;

#[derive(Debug)]
pub(crate) struct SimdInput {
    v0: __m256i,
    v1: __m256i,
}

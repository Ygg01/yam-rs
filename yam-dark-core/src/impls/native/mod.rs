use simdutf8::basic::imp::ChunkedUtf8Validator;

mod stage1;

type V128 = [u8; 16];

#[derive(Debug)]
pub(crate) struct SimdInput {
    v0: V128,
    v1: V128,
    v2: V128,
    v3: V128,
}
/// This is a hack, since there is no native implementation of the chunked validator we pre-validate the entire
/// input string in the case of a fallback and then always let the chunked validator return true.
pub(crate) struct ChunkedUtf8ValidatorImp();

impl ChunkedUtf8Validator for ChunkedUtf8ValidatorImp {
    unsafe fn new() -> Self
    where
        Self: Sized,
    {
        ChunkedUtf8ValidatorImp()
    }

    unsafe fn update_from_chunks(&mut self, _input: &[u8]) {}

    unsafe fn finalize(
        self,
        _remaining_input: core::option::Option<&[u8]>,
    ) -> core::result::Result<(), simdutf8::basic::Utf8Error> {
        Ok(())
    }
}

use simdutf8::basic::imp::ChunkedUtf8Validator;

pub(crate) struct NoopValidator();

impl ChunkedUtf8Validator for NoopValidator {
    unsafe fn new() -> Self
    where
        Self: Sized,
    {
        NoopValidator()
    }

    unsafe fn update_from_chunks(&mut self, _input: &[u8]) {}

    unsafe fn finalize(
        self,
        _remaining_input: core::option::Option<&[u8]>,
    ) -> core::result::Result<(), simdutf8::basic::Utf8Error> {
        Ok(())
    }
}

use simdutf8::basic::imp::ChunkedUtf8Validator;

pub(crate) use chunked_iter::ChunkyIterator;
pub use native::u8x16_swizzle;
pub use native::u8x64_eq;
pub use native::u8x64_lteq;
pub use native::U8X16;

mod chunked_iter;
mod native;

#[doc(hidden)]
pub struct NoopValidator();

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
        _remaining_input: Option<&[u8]>,
    ) -> Result<(), simdutf8::basic::Utf8Error> {
        Ok(())
    }
}

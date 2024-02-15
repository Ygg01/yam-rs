use simdutf8::basic::imp::ChunkedUtf8Validator;

pub(crate) trait Stage1Parse {
    type Utf8Validator: ChunkedUtf8Validator;
    type SimdRepresentation;

    unsafe fn new(ptr: &[u8]) -> Self;
}

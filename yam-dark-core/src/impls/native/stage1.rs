use crate::impls::native::{ChunkedUtf8ValidatorImp, SimdInput, V128};
use crate::stage1::Stage1Parse;

impl Stage1Parse for SimdInput {
    type Utf8Validator = ChunkedUtf8ValidatorImp;
    type SimdRepresentation = V128;

    unsafe fn new(ptr: &[u8]) -> Self {
        todo!()
    }
}

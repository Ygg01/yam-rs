use crate::tokenizer::stage1::{Stage1Scanner, YamlBlockState};
use crate::tokenizer::stage2::{Buffer, YamlParserState};
use crate::util::NoopValidator;
use crate::ParseResult;

pub(crate) struct NativeScanner {
    v0: [u8; 16],
    v1: [u8; 16],
    v2: [u8; 16],
    v3: [u8; 16],
}


impl Stage1Scanner for NativeScanner {
    type SimdType = [u8; 16];
    type Validator = NoopValidator;

    fn validator() -> Self::Validator {
        NoopValidator {}
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn from_chunk(values: &[u8; 64]) -> Self {
        unsafe {
            NativeScanner {
                v0: *values[0..16].as_ptr().cast::<[u8; 16]>(),
                v1: *values[16..32].as_ptr().cast::<[u8; 16]>(),
                v2: *values[32..48].as_ptr().cast::<[u8; 16]>(),
                v3: *values[48..64].as_ptr().cast::<[u8; 16]>(),
            }
        }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn next<T: Buffer>(
        chunk: &[u8; 64],
        _buffers: &mut T,
        _state: &mut YamlParserState,
    ) -> ParseResult<YamlBlockState> {
        let block = YamlBlockState::default();
        let simd = NativeScanner::from_chunk(chunk);
        Ok(block)
    }
}

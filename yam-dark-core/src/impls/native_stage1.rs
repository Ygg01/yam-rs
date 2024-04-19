use crate::stage1::{Stage1Scanner, YamlBlockState};
use crate::stage2::{Buffer, YamlParserState};
use crate::util::NoopValidator;
use crate::ParseResult;

pub(crate) struct NativeScanner {
    v0: [u8; 16],
    v1: [u8; 16],
    v2: [u8; 16],
    v3: [u8; 16],
}

impl From<&[u8; 64]> for NativeScanner {
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn from(value: &[u8; 64]) -> Self {
        unsafe {
            NativeScanner {
                v0: *value[0..16].as_ptr().cast::<[u8; 16]>(),
                v1: *value[16..32].as_ptr().cast::<[u8; 16]>(),
                v2: *value[32..48].as_ptr().cast::<[u8; 16]>(),
                v3: *value[48..64].as_ptr().cast::<[u8; 16]>(),
            }
        }
    }
}

impl Stage1Scanner for NativeScanner {
    type SimdType = [u8; 16];
    type Validator = NoopValidator;

    fn validator() -> Self::Validator {
        NoopValidator {}
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn next<T: Buffer>(
        chunk: &[u8; 64],
        _buffers: &mut T,
        _state: &mut YamlParserState,
    ) -> ParseResult<YamlBlockState> {
        let block = YamlBlockState::default();
        let scanner = NativeScanner::from(chunk);
        Ok(block)
    }
}

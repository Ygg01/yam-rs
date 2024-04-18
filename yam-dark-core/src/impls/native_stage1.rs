use crate::stage1::{Stage1Scanner, YamlBlockState};
use crate::stage2::{Buffer, YamlParserState};
use crate::util::NoopValidator;
use crate::ParseResult;

pub(crate) struct NativeScanner {}

impl Stage1Scanner for NativeScanner {
    type SimdType = u128;
    type Validator = NoopValidator;

    fn validator() -> Self::Validator {
        NoopValidator {}
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn next<T: Buffer>(
        _chunk: &[u8; 64],
        _buffers: &mut T,
        _state: &mut YamlParserState,
    ) -> ParseResult<YamlBlockState> {
        todo!()
    }
}

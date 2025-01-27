use crate::tokenizer::stage1::{Stage1Scanner, YamlBlockState};
use crate::tokenizer::stage2::{Buffer, YamlParserState};
use crate::util::NoopValidator;
use crate::ParseResult;

pub(crate) struct NativeScanner {
    v0: [u8; 64],
}

impl Stage1Scanner for NativeScanner {
    type SimdType = [u8; 16];
    type Validator = NoopValidator;

    fn validator() -> Self::Validator {
        NoopValidator {}
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn from_chunk(values: &[u8; 64]) -> Self {
        NativeScanner { v0: *values }
    }

    fn cmp_ascii_to_input(&self, c: u8) -> u64 {
        // let x0 = self.v0.iter()
        //     .fold(0u64, move |b, x| {
        //         let m = b << 1;
        //         let z = if *x == c {
        //             0x0001
        //         } else {
        //             0x0000
        //         };
        //         m | z
        //     });
        // let x1 = self.v1.iter()
        //     .fold(x0, move |b, x| {
        //         let m = b << 1;
        //         let z = if *x == c {
        //             0x0001
        //         } else {
        //             0x0000
        //         };
        //         m | z
        //     });
        // let x2 = self.v2.iter()
        //     .fold(x1, move |b, x| {
        //         let m = b << 1;
        //         let z = if *x == c {
        //             0x0001
        //         } else {
        //             0x0000
        //         };
        //         m | z
        //     });
        // self.v3.iter()
        //     .fold(x2, move |b, x| {
        //         let m = b << 1;
        //         let z = if *x == c {
        //             0x0001
        //         } else {
        //             0x0000
        //         };
        //         m | z
        //     })
        9
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

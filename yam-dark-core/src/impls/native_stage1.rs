use crate::tokenizer::stage1::{Stage1Scanner, YamlCharacterChunk, YamlChunkState};
use crate::util::NoopValidator;
use crate::{u8x64_eq, u8x64_lteq};

#[doc(hidden)]
pub struct NativeScanner {
    v0: [u8; 64],
}

impl Stage1Scanner for NativeScanner {
    type SimdType = [u8; 64];
    type Validator = NoopValidator;

    fn validator() -> Self::Validator {
        NoopValidator {}
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn from_chunk(values: &[u8; 64]) -> Self {
        NativeScanner { v0: *values }
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn cmp_ascii_to_input(&self, cmp: u8) -> u64 {
        u8x64_eq(self.v0, cmp)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn leading_spaces(&self, chunks: &YamlCharacterChunk) -> (u32, u32) {
        // TODO actual spaces implementation
        let z = chunks.spaces.leading_zeros();
        (z, z)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    #[allow(clippy::cast_sign_loss)]
    fn compute_quote_mask(quote_bits: u64) -> u64 {
        let mut quote_mask: u64 = quote_bits ^ (quote_bits << 1);
        quote_mask = quote_mask ^ (quote_mask << 2);
        quote_mask = quote_mask ^ (quote_mask << 4);
        quote_mask = quote_mask ^ (quote_mask << 8);
        quote_mask = quote_mask ^ (quote_mask << 16);
        quote_mask = quote_mask ^ (quote_mask << 32);
        quote_mask
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn unsigned_lteq_against_splat(&self, cmp: i8) -> u64 {
        u8x64_lteq(self.v0, cmp as u8)
    }

    fn scan_whitespace_and_structurals(&self, block_state: &mut YamlChunkState) {
        todo!()
    }
}

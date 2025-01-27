#![allow(unused)]

use simdutf8::basic::imp::ChunkedUtf8Validator;

use crate::ParseResult;
use crate::tokenizer::stage2::{Buffer, YamlParserState};

#[derive(Default)]
pub struct YamlChunkState {
    double_quote: YamlDoubleQuoteChunk,
    single_quote: YamlSingleQuoteChunk,
    characters: YamlCharacterChunk,
    follows_non_quote_scalar: u64,
}

#[derive(Default)]
pub struct YamlDoubleQuoteChunk {
    /// Escaped characters
    escaped: u64,
    /// Real double quotes
    quote: u64,
    /// String characters
    in_string: u64,
    prev_iter_odd_backslash: u64,
}

#[derive(Default)]
pub struct YamlSingleQuoteChunk {
    /// Real single quotes
    quote: u64,
    /// String characters
    in_string: u64,
}

#[derive(Default)]
pub struct YamlCharacterChunk {
    /// Space
    spaces: u64,
    /// Operators
    op: u64,
}

impl YamlChunkState {}

pub(crate) type NextFn<B> = for<'buffer, 'input> unsafe fn(
    chunk: &'buffer [u8; 64],
    buffers: &'input mut B,
    state: &'input mut YamlParserState,
) -> ParseResult<YamlChunkState>;

const EVEN_BITS: u64 = 0x5555_5555_5555_5555;
const ODD_BITS: u64 = !EVEN_BITS;

pub trait Stage1Scanner {
    type SimdType;
    type Validator: ChunkedUtf8Validator;

    fn validator() -> Self::Validator;

    fn from_chunk(values: &[u8; 64]) -> Self;

    fn cmp_ascii_to_input(&self, m: u8) -> u64;

    fn leading_spaces(&self, spaces: u64) -> (u32, u32);

    /// Scans a chunk and returns a YamlBlockState
    fn next<T: Buffer>(
        chunk: &[u8; 64],
        buffers: &mut T,
        prev_state: &mut YamlParserState,
    ) -> ParseResult<YamlChunkState>
    where
        Self: Sized,
    {
        let mut block = YamlChunkState::default();
        let mut simd = Self::from_chunk(chunk);
        let single_quotes = simd.cmp_ascii_to_input(b'\'');
        let double_quotes = simd.cmp_ascii_to_input(b'"');

        simd.find_odd_backslash_sequences(&mut block);
        simd.find_whitespace_and_structurals(&mut block);
        simd.find_single_quote_mask_and_bits(&mut block, single_quotes);
        simd.find_double_quote_mask_and_bits(&mut block, double_quotes);

        prev_state.merge_state(chunk, buffers, &mut block)
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn find_odd_backslash_sequences(&self, block_state: &mut YamlChunkState) {
        let backslash_bits: u64 = self.cmp_ascii_to_input(b'\\');
        let start_edges: u64 = backslash_bits & !(backslash_bits << 1);

        let backslash_bits = self.cmp_ascii_to_input(b'\\');
        let start_edges = backslash_bits & !(backslash_bits << 1);
        // flip lowest if we have an odd-length run at the end of the prior iteration
        let even_start_mask = EVEN_BITS ^ block_state.double_quote.prev_iter_odd_backslash;
        let even_starts = start_edges & even_start_mask;
        let odd_start = start_edges & !even_start_mask;
        let even_carries = backslash_bits.wrapping_add(even_starts);

        //
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn find_whitespace_and_structurals(&self, block_state: &mut YamlChunkState) {
        let spaces = self.cmp_ascii_to_input(b' ');
        let newlines = self.cmp_ascii_to_input(b'\n');
        block_state.characters.spaces = spaces;
        todo!()
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn find_single_quote_mask_and_bits(&self, block_state: &mut YamlChunkState, quote_bits: u64) {
        todo!()
    }

    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn find_double_quote_mask_and_bits(&self, block_state: &mut YamlChunkState, quote_bits: u64) {
        todo!()
    }
}

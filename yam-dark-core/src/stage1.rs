#![allow(unused)]
use simdutf8::basic::imp::ChunkedUtf8Validator;

use crate::stage2::{Buffer, YamlParserState};
use crate::ParseResult;

#[derive(Default)]
pub struct YamlBlockState {
    double_quote: YamlDoubleQuoteBlock,
    single_quote: YamlSingleQuoteBlock,
    characters: YamlCharacterBlock,
    follows_non_quote_scalar: u64,
}

#[derive(Default)]
pub struct YamlDoubleQuoteBlock {
    /// Escaped characters
    escaped: u64,
    /// Real double quotes
    quote: u64,
    /// String characters
    in_string: u64,
}

#[derive(Default)]
pub struct YamlSingleQuoteBlock {
    /// Real single quotes
    quote: u64,
    /// String characters
    in_string: u64,
}

#[derive(Default)]
pub struct YamlCharacterBlock {
    /// Whitespaces
    whitespace: u64,
    /// Operators
    op: u64,
}

impl YamlBlockState {}

pub(crate) type NextFn<B> = for<'buffer, 'input> unsafe fn(
    chunk: &'buffer [u8; 64],
    buffers: &'input mut B,
    state: &'input mut YamlParserState,
) -> ParseResult<YamlBlockState>;

pub trait Stage1Scanner {
    type SimdType;
    type Validator: ChunkedUtf8Validator;

    fn validator() -> Self::Validator;

    /// Scans a chunk and returns a YamlBlockState

    fn next<T: Buffer>(
        chunk: &[u8; 64],
        buffers: &mut T,
        state: &mut YamlParserState,
    ) -> ParseResult<YamlBlockState>;
}

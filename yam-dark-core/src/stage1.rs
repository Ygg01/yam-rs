use core::arch::x86_64::__m256i;

use simdutf8::basic::imp::ChunkedUtf8Validator;

use crate::stage2::{Buffer, YamlParserState};
use crate::util::NoopValidator;
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
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn next<T: Buffer>(
        chunk: &[u8; 64],
        buffers: &mut T,
        state: &mut YamlParserState,
    ) -> ParseResult<YamlBlockState>;
}

pub(crate) struct NativeScanner {}

impl Stage1Scanner for NativeScanner {
    type SimdType = u128;
    type Validator = NoopValidator;

    fn validator() -> Self::Validator {
        NoopValidator {}
    }

    fn next<T: Buffer>(
        _chunk: &[u8; 64],
        _buffers: &mut T,
        _state: &mut YamlParserState,
    ) -> ParseResult<YamlBlockState> {
        todo!()
    }
}

pub(crate) struct AvxScanner {}

impl Stage1Scanner for AvxScanner {
    type SimdType = [__m256i; 2];
    type Validator = simdutf8::basic::imp::x86::avx2::ChunkedUtf8ValidatorImp;

    fn validator() -> Self::Validator {
        unsafe { simdutf8::basic::imp::x86::avx2::ChunkedUtf8ValidatorImp::new() }
    }

    fn next<'b, 'i, T: Buffer>(
        _chunk: &'b [u8; 64],
        _buffers: &'i mut T,
        _state: &'i mut YamlParserState,
    ) -> ParseResult<YamlBlockState> {
        todo!()
    }
}

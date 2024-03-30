use crate::ParseResult;
use crate::stage2::{Buffer, Buffers, YamlParserState};

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

pub trait Utf8Validator {}

pub struct NoopValidator {}

impl Utf8Validator for NoopValidator {}

pub trait Stage1Scanner {
    type SimdType;
    fn with_validator<T: Utf8Validator>(validator: T) -> Self;

    fn get_validator<T: Utf8Validator>(&self) -> T;

    /// Scans a chunk and returns a YamlBlockState
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn next<T: Buffer>(
        chunk: &[u8; 64],
        buffers: &mut T,
        state: &mut YamlParserState,
    ) -> ParseResult<YamlSingleQuoteBlock>;
}

struct NativeScanner {}

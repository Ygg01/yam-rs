use crate::ParseResult;

#[derive(Default)]
pub struct YamlBlockState {
    string: YamlStringBlock,
    characters: YamlCharacterBlock,
    follows_non_quote_scalar: u64,
}

#[derive(Default)]
pub struct YamlStringBlock {
    escaped: u64,
    quote: u64,
    in_string: u64,
}

#[derive(Default)]
pub struct YamlCharacterBlock {
    whitespace: u64,
    op: u64,
}

impl YamlBlockState {}

pub trait Utf8Validator {}

pub struct NoopValidator {}

impl Utf8Validator for NoopValidator {}

pub trait Stage1Scanner {
    type SimdType;

    /// Scans a slice and returns a YamlBlockState
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn next<V: Utf8Validator>(
        &mut self,
        input: Self::SimdType,
        unicode_validator: V,
    ) -> YamlBlockState;

    /// Finishes the scan
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn finish<V: Utf8Validator>(
        &mut self,
    ) -> ParseResult<()>;
}

struct NativeScanner {}

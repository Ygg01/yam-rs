#![warn(clippy::pedantic)]
#![allow(clippy::too_many_arguments, clippy::module_name_repetitions)]
#![no_std]
extern crate alloc;
extern crate core_detect;

pub use crate::tokenizer::chunk::{
    YamlCharacterChunk, YamlChunkState, YamlDoubleQuoteChunk, YamlSingleQuoteChunk,
};
pub use crate::tokenizer::stage2::YamlParserState;
pub use crate::util::u8x64_eq;
use alloc::string::String;
use core::str::Utf8Error;
pub use impls::NativeScanner;
pub use tokenizer::stage1::Stage1Scanner;

pub mod impls;
mod tape;
mod tokenizer;
pub mod util;

pub const SIMD_CHUNK_LENGTH: usize = 64;
pub const SIMD_JSON_PADDING: usize = 32;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum YamlError {
    Utf8(Utf8Error),
    Io(String),
    UnexpectedEof,
    /// Input decoding error. If `encoding` feature is disabled, contains `None`,
    /// otherwise contains the UTF-8 decoding error
    NonDecodable(Option<Utf8Error>),
}

pub type YamlResult<T> = Result<T, YamlError>;

#[repr(u64)]
#[derive(Clone, Copy)]
pub enum EvenOrOddBits {
    EvenBits = 0x5555_5555_5555_5555,
    OddBits = 0xAAAA_AAAA_AAAA_AAAA,
}
pub const LOW_NIBBLE: [u8; 16] = [64, 0, 0, 0, 0, 0, 0, 0, 0, 32, 40, 16, 4, 50, 0, 1];
pub const HIGH_NIBBLE: [u8; 16] = [32, 0, 70, 9, 0, 16, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0];

pub type ParseResult<T> = Result<T, YamlError>;
pub type ChunkyIterator<'a> = util::ChunkyIterator<'a>;

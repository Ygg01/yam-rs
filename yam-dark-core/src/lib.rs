#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::too_many_arguments, clippy::module_name_repetitions)]
#![no_std]
// TEMP allow
#![allow(unused, missing_docs)]

//! SIMD enhanced YAML parser for Rust

extern crate alloc;
extern crate core_detect;
extern crate yam_common;

pub use crate::tokenizer::chunk::{
    YamlCharacterChunk, YamlChunkState, YamlDoubleQuoteChunk, YamlSingleQuoteChunk,
};

pub use crate::util::u8x64_eq;
use alloc::string::String;
use core::str::Utf8Error;
pub use impls::NativeScanner;
pub use tape::EventListener;
pub(crate) use tokenizer::buffers::YamlBuffer;
pub use tokenizer::run_tape_to_end;
pub use tokenizer::stage1::Stage1Scanner;
pub use tokenizer::stage2::YamlIndentInfo;
pub use tokenizer::YamlParserState;

pub mod impls;
mod tape;
mod tokenizer;
pub mod util;

#[doc(hidden)]
pub const SIMD_CHUNK_LENGTH: usize = 64;

#[derive(Debug, Clone, Eq, PartialEq)]
/// Enum representing errors in `yam.rs` during parsing
pub enum YamlError {
    /// UTF8 decoding error
    Utf8(Utf8Error),
    /// Input-output error
    Io(String),
    /// Unexpected End-of-File
    UnexpectedEof,
    /// Input decoding error. If the ` encoding ` feature is disabled, it contains `None`,
    /// otherwise contains the UTF-8 decoding error
    NonDecodable(Option<Utf8Error>),
    /// Generic syntax error
    Syntax,
}

#[repr(u64)]
#[derive(Clone, Copy)]
/// Enum representing odd or even bits
pub enum EvenOrOddBits {
    /// Even bits starting from zero
    EvenBits = 0x5555_5555_5555_5555,
    /// Odd bits starting from one
    OddBits = 0xAAAA_AAAA_AAAA_AAAA,
}
#[doc(hidden)]
pub const LOW_NIBBLE: [u8; 16] = [64, 0, 0, 0, 0, 0, 0, 0, 0, 32, 40, 16, 4, 50, 0, 1];
#[doc(hidden)]
pub const HIGH_NIBBLE: [u8; 16] = [32, 0, 70, 9, 0, 16, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0];

/// Convenience type for the result of parsing YAML.
pub type YamlResult<T> = Result<T, YamlError>;

/// Iterator used to iterate over 64 byte chunks
pub type ChunkyIterWrap<'a> = util::ChunkArrayIter<'a>;

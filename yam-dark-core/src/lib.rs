#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::too_many_arguments, clippy::module_name_repetitions)]
#![no_std]
// TEMP allow
#![allow(unused, missing_docs)]

//! SIMD enhanced YAML parser for Rust

extern crate alloc;
extern crate core_detect;
extern crate yam_common;

pub use crate::util::u8x64_eq;
use alloc::string::String;
use core::str::Utf8Error;

mod events;
mod scanner;
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

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct Marker {
    /// Position of byte. Starts at 0.
    pub pos: usize,
    /// Byte column of the marker. Starts at one.
    pub byte_column: usize,
    /// Line of the marker. Starts at one.
    pub line: usize,
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct Span {
    pub start: Marker,
    pub end: usize,
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub enum TokenType<'input>{

}

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct Token<'input>(pub Span, pub TokenType<'input>);

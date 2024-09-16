#![feature(slice_as_chunks)]
#![no_std]
extern crate alloc;
extern crate core_detect;

pub use impls::NativeScanner;
pub use tokenizer::stage1::Stage1Scanner;

use crate::error::Error;
pub use crate::tokenizer::stage1::{
    YamlCharacterChunk, YamlChunkState, YamlDoubleQuoteChunk, YamlSingleQuoteChunk,
};
pub use crate::tokenizer::stage2::YamlParserState;
pub use crate::util::u8x64_eq;

mod error;
pub mod impls;
mod tokenizer;
pub mod util;

pub const SIMD_CHUNK_LENGTH: usize = 64;
pub const SIMD_JSON_PADDING: usize = 32;

pub const EVEN_BITS: u64 = 0x5555_5555_5555_5555;
pub const ODD_BITS: u64 = !EVEN_BITS;

#[doc(hidden)]
pub const LOW_NIBBLE_MASK: [u8; 16] = [16, 2, 0, 2, 0, 2, 2, 2, 0, 8, 11, 4, 2, 14, 2, 1];
#[doc(hidden)]
pub const HIGH_NIBBLE_MASK: [u8; 16] = [8, 0, 18, 1, 0, 4, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0];

pub type ParseResult<T> = Result<T, Error>;
pub type ChunkyIterator<'a> = util::ChunkyIterator<'a>;

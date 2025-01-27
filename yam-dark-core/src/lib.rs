#![feature(slice_as_chunks)]
#![no_std]
extern crate alloc;
extern crate core_detect;

pub use crate::tokenizer::stage1::{
    YamlCharacterChunk, YamlChunkState, YamlDoubleQuoteChunk, YamlSingleQuoteChunk,
};
pub use crate::tokenizer::stage2::YamlParserState;
pub use crate::util::u8x64_eq;
pub use impls::NativeScanner;
pub use tokenizer::stage1::Stage1Scanner;
use yam_core::error::YamlError;

pub mod impls;
mod tokenizer;
pub mod util;

pub const SIMD_CHUNK_LENGTH: usize = 64;
pub const SIMD_JSON_PADDING: usize = 32;

pub const EVEN_BITS: u64 = 0x5555_5555_5555_5555;
pub const ODD_BITS: u64 = !EVEN_BITS;
const LOW_NIBBLE: [u8; 16] = [64, 0, 0, 0, 0, 0, 0, 0, 0, 32, 40, 16, 4, 50, 0, 1];
const HIGH_NIBBLE: [u8; 16] = [32, 0, 70, 9, 0, 16, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0];

pub type ParseResult<T> = Result<T, YamlError>;
pub type ChunkyIterator<'a> = util::ChunkyIterator<'a>;

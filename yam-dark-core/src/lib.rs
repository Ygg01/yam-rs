#![feature(slice_as_chunks)]
#![no_std]
extern crate alloc;
extern crate core_detect;

pub use impls::{u8x16_bit, u8x16_bit_iter};

use crate::error::Error;

mod error;
mod impls;
mod tokenizer;
mod util;

pub const SIMD_INPUT_LENGTH: usize = 64;
pub const SIMD_JSON_PADDING: usize = 32;

pub const EVEN_BITS: u64 = 0x5555_5555_5555_5555;
pub const ODD_BITS: u64 = !EVEN_BITS;

pub type ParseResult<T> = Result<T, Error>;
pub type ChunkyIterator<'a> = util::ChunkyIterator<'a>;

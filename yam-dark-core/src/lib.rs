#![no_std]
extern crate alloc;
extern crate core_detect;

use crate::error::Error;

mod error;
mod safer_unchecked;
mod stage1;
mod stage2;
mod tokenizer;
mod util;
mod impls;

pub const SIMD_INPUT_LENGTH: usize = 64;
pub const SIMD_JSON_PADDING: usize = 32;

pub const EVEN_BITS: u64 = 0x5555_5555_5555_5555;
pub const ODD_BITS: u64 = !EVEN_BITS;

pub type ParseResult<T> = Result<T, Error>;

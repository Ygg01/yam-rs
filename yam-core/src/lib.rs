#![no_std]
extern crate alloc;
extern crate core;
extern crate yam_common;

pub use saphyr_tokenizer::{Parser, Span};
pub use tokenizer::Lexer;

pub type SaphyrEvent<'input> = saphyr_tokenizer::Event<'input>;

pub mod escaper;
pub mod tokenizer;
pub mod treebuild;

pub mod saphyr_tokenizer;

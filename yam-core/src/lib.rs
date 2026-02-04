#![no_std]
extern crate alloc;
extern crate core;
extern crate yam_common;

pub use saphyr_tokenizer::Parser;
pub use saphyr_tokenizer::Source;
pub use treebuild::YamlLoader;

pub type SaphyrEvent<'input> = saphyr_tokenizer::Event<'input>;

pub mod escaper;
pub mod treebuild;

mod saphyr_emitter;
pub mod saphyr_tokenizer;

//! YAML 1.2 parser based on [Saphyr-rs](https://docs.rs/saphyr/latest/saphyr/)
//!
//! # Usage
//! To add [yam-rs](https://github.com/Ygg01/yam-rs) to your project’s `Cargo.toml`:
//!
//! ```sh
//! cargo add yam-core
//! ```
//!
//! Or if you want the [`std`] compatible crate use:
//! ```sh
//! cargo add yam-std
//! ```
//!
//! # Minimal example
//! ```rust
//! use yam_core::{LoadableYamlNode, YamlDoc, YamlLoader};
//!
//! let docs = YamlDoc::load_from("[1, 2, 3]").expect("Valid YAML document");
//! let doc = &docs[0];
//!
//!
//! ```
#![no_std]
extern crate alloc;
extern crate core;

#[doc(hidden)]
pub mod escaper;

pub mod parsing;
pub mod prelude;
mod saphyr_emitter;
mod util;

pub use crate::parsing::{Event, Source, StrSource};
pub use crate::prelude::{YamlDoc, YamlDocAccess, YamlLoader};

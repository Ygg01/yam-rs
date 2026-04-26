//! YAML 1.2 parser based on [Saphyr-rs](https://docs.rs/saphyr/latest/saphyr/)
//!
//! # Usage
//! To add [yam-rs](https://github.com/Ygg01/yam-rs) to your project’s `Cargo.toml`:
//!
//! ```sh
//! cargo add yam-core
//! ```
//!
//! Or if you want the `std` compatible crate use:
//! ```sh
//! cargo add yam-std
//! ```
//!
//! # Minimal example
//! ```rust
//! use yam_core::prelude::{Yaml, YamlLoader};
//!
//! let docs = Yaml::load_single("[1, 2]").expect("Valid YAML document");
//! let val = &docs[0];
//! assert_eq!(val, &Yaml::from(1));
//! ```
//!
//! # YAML Object types
//!
//! This crate comes with multiple types of YAML representations:
//! - [`Yaml`](prelude::Yaml): The default YAML object which borrows from the input.
//! - [`YamlOwned`](prelude::YamlOwned): The version of [`Yaml`](prelude::Yaml) which owns its data.
//! - [`SpannedYaml`](prelude::SpannedYaml): The version of [`Yaml`](prelude::Yaml) which borrows its data and includes [`Span`](prelude::Span) information.
#![no_std]
extern crate alloc;
extern crate core;

#[doc(hidden)]
pub mod escaper;

pub mod node;
pub mod parsing;
pub mod prelude;
mod saphyr_emitter;
mod util;

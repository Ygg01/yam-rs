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
//! use yam_core::prelude::{Yaml, YamlLoader};
//!
//! let docs = Yaml::load_from("[1, 2]").expect("Valid YAML document");
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
pub use crate::prelude::{YamlDocAccess, YamlLoader};

#[cfg(test)]
mod test {
    use crate::prelude::Yaml;

    #[test]
    fn test_ex() {
        let docs = Yaml::load_from("[1, 2]").expect("Valid YAML document");
        let doc = &docs[0];
        // let val = doc[0];
    }
}

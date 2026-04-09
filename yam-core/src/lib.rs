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
use crate::prelude::{LoadableYamlNode, YamlDocAccess};

pub use crate::prelude::YamlDoc;
pub use crate::prelude::YamlLoader;

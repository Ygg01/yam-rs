//! Module dealing with parsing YAML documents into a series of events.
//!
mod buffered_source;
mod char_utils;
mod parser;
mod scanner;
mod source;

pub use parser::EventReceiver;
pub use parser::SpannedEventReceiver;
pub use parser::{Event, Parser, ScalarValue};
pub use source::Source;
pub use source::StrSource;

use std::borrow::Cow;
use std::fmt::{Debug, Formatter, Write};
use std::str::from_utf8_unchecked;

use crate::tokenizer::event::YamlEvent::{DocEnd,
                                         DocStart,
                                         ScalarValue,
                                         SeqEnd,
                                         SeqStart,
                                         StreamEnd, StreamStart};

pub enum YamlEvent<'a> {
    StreamStart,
    StreamEnd,
    DocStart,
    DocEnd,
    SeqStart,
    SeqEnd,
    ScalarValue(Cow<'a, [u8]>),
}

impl<'a> Debug for YamlEvent<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamStart => f.write_str("+STR"),
            StreamEnd => f.write_str("-STR"),
            DocStart => f.write_str("+DOC"),
            DocEnd => f.write_str("-DOC"),
            SeqStart => f.write_str("+SEQ"),
            SeqEnd => f.write_str("-SEQ"),
            ScalarValue(x) => unsafe {
                // todo encoding
                f.write_str(from_utf8_unchecked(x.as_ref()))
            },
        }
    }
}
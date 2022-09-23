use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::str::from_utf8_unchecked;

use crate::tokenizer::event::YamlEvent::{
    DocEnd, DocStart, ScalarValue, SeqEnd, SeqStart, StreamEnd, StreamStart, YamlTag,
};

pub enum YamlEvent<'a> {
    StreamStart,
    StreamEnd,
    DocStart,
    DocEnd,
    SeqStart,
    SeqEnd,
    YamlTag(Cow<'a, [u8]>),
    ScalarValue(Cow<'a, [u8]>),
}

impl<'a> Debug for YamlEvent<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamStart => write!(f, "+STR"),
            StreamEnd => write!(f, "-STR"),
            DocStart => write!(f, "+DOC"),
            DocEnd => write!(f, "-DOC"),
            SeqStart => write!(f, "+SEQ"),
            SeqEnd => write!(f, "-SEQ"),
            YamlTag(x) => write!(f, "#TAG {}", unsafe { from_utf8_unchecked(x.as_ref()) }),
            ScalarValue(x) => write!(f, "+VAL {}", unsafe { from_utf8_unchecked(x.as_ref()) }),
        }
    }
}

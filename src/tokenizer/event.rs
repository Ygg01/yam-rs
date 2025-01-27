use std::borrow::Cow;

pub enum  YamlEvent<'a> {
    Start,
    DocStart,
    SeqStart,
    SeqEnd,
    ScalarValue(Cow<'a, [u8]>),
    StreamStart
}
use crate::Tag;
use std::borrow::Cow;

/// Ordered sequence of one or more [`YamlDoc`]'s
pub type Sequence<'a> = Vec<YamlDoc<'a>>;

/// Sequence of key-value pairing of two [`YamlDoc`]s
pub type Mapping<'a> = Vec<Entry<'a>>;

#[derive(Debug, Default, Clone, PartialEq)]
pub enum YamlDoc<'input> {
    #[default]
    Null,
    String(Cow<'input, str>),
    Bool(bool),
    FloatingPoint(f64),
    Integer(i64),
    // flow style like `[x, x, x]`
    // or block style like:
    //     - x
    //     - x
    Sequence(Sequence<'input>),

    // flow style like `{x: Y, a: B}`
    // or block style like:
    //     x: Y
    //     a: B
    Mapping(Mapping<'input>),
}

impl<'input> YamlDoc<'input> {
    pub fn from_cow_and_tag(
        _cow: Cow<'input, str>,
        _tag: &Option<Cow<'input, Tag>>,
    ) -> YamlDoc<'input> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry<'input> {
    key: YamlDoc<'input>,
    value: YamlDoc<'input>,
}

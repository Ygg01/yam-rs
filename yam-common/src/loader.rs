use std::borrow::Cow;

/// Ordered sequence of one or more [`YamlDoc`]'s
pub type Sequence<'a> = Vec<YamlDoc<'a>>;

/// Sequence of key-value pairing of two [`YamlDoc`]s
pub type Mapping<'a> = Vec<Entry<'a>>;

pub enum YamlDoc<'input> {
    String(Cow<'input, str>),
    Bool(bool),
    FloatingPoint(f64),
    Integer(i64),
    Null,

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

pub struct Entry<'input> {
    key: YamlDoc<'input>,
    value: YamlDoc<'input>,
}

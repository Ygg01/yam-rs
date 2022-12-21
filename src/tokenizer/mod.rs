use std::borrow::Cow;

pub use scanner::Scanner;

mod event;
mod iter;
mod reader;
mod scanner;

pub enum YamlToken<'a> {
    // strings, booleans, numbers, nulls, all treated the same
    Scalar(Cow<'a, [u8]>),

    // flow style like `[x, x, x]`
    // or block style like:
    //     - x
    //     - x
    Sequence(Vec<YamlToken<'a>>),

    // flow style like `{x: Y, a: B}`
    // or block style like:
    //     x: Y
    //     a: B
    Mapping(Vec<Entry<'a>>),

    // Error during parsing
    Error,
}

pub struct Entry<'a> {
    key: YamlToken<'a>,
    value: YamlToken<'a>,
}

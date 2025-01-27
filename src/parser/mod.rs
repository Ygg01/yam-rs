mod machine;
mod event;

use std::borrow::Cow;

#[derive(Clone)]
pub struct Reader<R> {
    /// Source of data for parse
    reader: R,
    /// Configuration and current parse state
    parser: Parser,
}


pub enum Yaml<'a> {
    // strings, booleans, numbers, nulls, all treated the same
    Scalar(Cow<'a,[u8]>),

    // flow style like `[x, x, x]`
    // or block style like:
    //     - x
    //     - x
    Sequence(Vec<Yaml<'a>>),

    // flow style like `{x: X, x: X}`
    // or block style like:
    //     x: X
    //     x: X
    Mapping(Vec<Entry<'a>>),
}

pub struct Entry<'a> {
    key: Yaml<'a>,
    value: Yaml<'a>,
}

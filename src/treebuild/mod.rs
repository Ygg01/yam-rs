use std::borrow::Cow;

use crate::tokenizer::ErrorType;

mod iter;

pub enum YamlToken<'a, TAG> {
    // strings, booleans, numbers, nulls, all treated the same
    Scalar(Cow<'a, [u8]>, TAG),

    // flow style like `[x, x, x]`
    // or block style like:
    //     - x
    //     - x
    Sequence(Vec<YamlToken<'a, TAG>>, TAG),

    // flow style like `{x: Y, a: B}`
    // or block style like:
    //     x: Y
    //     a: B
    Mapping(Vec<Entry<'a, TAG>>, TAG),
}

impl<'a, TAG> YamlToken<'a, TAG> {
    pub fn empty(tag: TAG) -> YamlToken<'a, TAG> {
        YamlToken::Scalar(Cow::default(), tag)
    }
}

pub struct Entry<'a, TAG> {
    key: YamlToken<'a, TAG>,
    value: YamlToken<'a, TAG>,
}

pub struct YamlTokenError<'a, T> {
    partial: YamlToken<'a, T>,
    error: Vec<ErrorType>,
}

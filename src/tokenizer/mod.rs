mod tokenizer;
mod event;
mod reader;

use std::borrow::Cow;
use crate::tokenizer::event::YamlEvent;
use crate::tokenizer::reader::StrReader;
use crate::tokenizer::tokenizer::{SpanToken, YamlTokenizer};


pub enum YamlToken<'a> {
    // strings, booleans, numbers, nulls, all treated the same
    Scalar(Cow<'a,[u8]>),

    // flow style like `[x, x, x]`
    // or block style like:
    //     - x
    //     - x
    Sequence(Vec<YamlToken<'a>>),

    // flow style like `{x: X, x: X}`
    // or block style like:
    //     x: X
    //     x: X
    Mapping(Vec<Entry<'a>>),

    // Error during parsing
    Error
}

pub struct Entry<'a> {
    key: YamlToken<'a>,
    value: YamlToken<'a>,
}

struct StrIterator<'a> {
    state: YamlTokenizer,
    reader: StrReader<'a>,
}

impl<'a> StrIterator<'a> {
    pub(crate) fn to_token(&self, token: SpanToken) -> YamlEvent<'a> {
        match token {
            SpanToken::Scalar(start, end) => YamlEvent::ScalarValue(self.to_cow(start, end)),
        }
    }

    fn to_cow(&self, start: usize, end: usize) -> Cow<'a, [u8]> {
        Cow::Borrowed(self.reader.slice[start..end].as_bytes())
    }
}


impl<'a> Iterator for StrIterator<'a> {
    type Item = YamlEvent<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let span =  self.state.read_token(&mut self.reader);
        match span {
            Some(x) => Some(self.to_token(x)),
            None => None,
        }
    }
}
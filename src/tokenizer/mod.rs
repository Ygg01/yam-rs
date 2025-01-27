mod event;
mod reader;
mod scanner;

use crate::tokenizer::event::YamlEvent;
use crate::tokenizer::reader::StrReader;
use crate::tokenizer::scanner::{Control, SpanToken};
use std::borrow::Cow;

pub use scanner::Scanner;

pub enum YamlToken<'a> {
    // strings, booleans, numbers, nulls, all treated the same
    Scalar(Cow<'a, [u8]>),

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
    Error,
}

pub struct Entry<'a> {
    key: YamlToken<'a>,
    value: YamlToken<'a>,
}

pub struct StrIterator<'a> {
    state: Scanner,
    reader: StrReader<'a>,
}


impl<'a> StrIterator<'a> {
    pub(crate) fn to_token(&self, token: SpanToken) -> YamlEvent<'a> {
        match token {
            SpanToken::StreamStart => YamlEvent::StreamStart,
            SpanToken::StreamEnd => YamlEvent::StreamEnd,
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
        let span = loop {
            if let Some(token) = self.state.pop_token() {
                break token;
            } else if !self.state.eof {
                match self.state.next_state(&mut self.reader) {
                    Control::Continue => (),
                    Control::Eof => {
                        self.state.eof = true;
                        self.state.emit_end_of_stream();
                    }
                    _ => return None,
                }
            } else {
                return None;
            }
        };
        Some(self.to_token(span))
    }
}

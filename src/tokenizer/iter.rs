use std::borrow::Cow;

use crate::Scanner;
use crate::tokenizer::event::YamlEvent;
use crate::tokenizer::reader::{StrReader};
use crate::tokenizer::scanner::{Control, SpanToken};
use crate::tokenizer::scanner::State::StreamEnd;

pub struct StrIterator<'a> {
    pub(crate) state: Scanner,
    pub(crate) reader: StrReader<'a>,
}

impl<'a> StrIterator<'a> {
    pub(crate) fn to_token(&self, token: SpanToken) -> YamlEvent<'a> {
        match token {
            SpanToken::DocStart => YamlEvent::DocStart,
            SpanToken::DocEnd => YamlEvent::DocEnd,
            SpanToken::StreamStart => YamlEvent::StreamStart,
            SpanToken::StreamEnd => YamlEvent::StreamEnd,
            SpanToken::Scalar(start, end) => YamlEvent::ScalarValue(self.to_cow(start, end)),
            SpanToken::Directive(typ, start, end) => {
                YamlEvent::Directive(typ, self.to_cow(start, end))
            }
            SpanToken::ErrorToken(err) => YamlEvent::Error(err),
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
            } else {
                match self.state.next_state(&mut self.reader) {
                    Control::Continue => continue,
                    _ => return None,
                };
            }
        };
        Some(self.to_token(span))
    }
}

#[derive(Copy, Clone)]
pub enum ErrorType {
    ExpectedDocumentStart,
    StartedBlockInFlow,
}

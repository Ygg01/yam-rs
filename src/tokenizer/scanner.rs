use std::collections::VecDeque;

use crate::error::YamlError;
use crate::tokenizer::event::DirectiveType;
use crate::tokenizer::event::YamlEvent::{Directive, DocEnd};
use crate::tokenizer::iter::{ErrorType, StrIterator};
use crate::tokenizer::reader::{Reader, StrReader};
use crate::tokenizer::scanner::FlowStyle::Block;
use crate::tokenizer::scanner::State::{InDoc, StreamEnd, StreamStart};

#[derive(Clone, Default)]
pub struct Scanner {
    state: State,
    tokens: VecDeque<SpanToken>,
}

#[derive(Copy, Clone, PartialEq)]
pub enum State {
    StreamStart,
    InDoc,
    StreamEnd,
    Sequence(FlowStyle),
}

impl Default for State {
    fn default() -> Self {
        StreamStart
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum FlowStyle {
    Block,
    Flow,
}

impl Default for FlowStyle {
    fn default() -> Self {
        Block
    }
}

pub enum Control {
    Continue,
    Eof,
    Err(YamlError),
}

impl Scanner {
    pub fn from_str_reader(slice: &str) -> StrIterator<'_> {
        StrIterator {
            state: Default::default(),
            reader: StrReader::new(slice),
        }
    }

    pub(crate) fn emit_end_of_stream(&mut self) {
        match self.state {
            InDoc => self.tokens.push_back(SpanToken::DocEnd),
            _ => (),
        }
        self.tokens.push_back(SpanToken::StreamEnd);
        self.state = StreamEnd;
    }

    pub(crate) fn pop_token(&mut self) -> Option<SpanToken> {
        self.tokens.pop_front()
    }

    pub(crate) fn next_state<R: Reader>(&mut self, reader: &mut R) -> Control {
        match self.state {
            StreamStart => self.read_start_stream(reader),
            InDoc => self.read_in_doc(reader),
            StreamEnd => return Control::Eof,
            _ => {},
        };
        if reader.eof() && self.state != StreamEnd {
            self.emit_end_of_stream();
        }
        Control::Continue
    }

    pub(crate) fn read_start_stream<R: Reader>(&mut self, reader: &mut R) {
        self.try_skip_comments(reader);
        self.tokens.push_back(SpanToken::StreamStart);
        if reader.peek_byte_is(b'%') {
            if reader.try_read_slice_exact("%YAML") {
                reader.skip_space_tab();
                if let Some(x) = reader.find_next_whitespace() {
                    self.tokens.push_back(SpanToken::Directive(
                        DirectiveType::Yaml,
                        reader.pos(),
                        reader.pos() + x,
                    ));
                    reader.consume_bytes(x);
                    reader.read_line();
                }
            } else {
                let tag = if reader.try_read_slice_exact("%TAG") {
                    DirectiveType::Tag
                } else {
                    DirectiveType::Reserved
                };
                reader.skip_space_tab();
                let x = reader.read_non_comment_line();
                if x.0 != x.1 {
                    self.tokens.push_back(SpanToken::Directive(tag, x.0, x.1));
                }
            }
            if !reader.try_read_slice_exact("---") {
                self.tokens
                    .push_back(SpanToken::ErrorToken(ErrorType::ExpectedDocumentStart));
            }
        }
        self.state = InDoc;
        self.tokens.push_back(SpanToken::DocStart);
    }

    pub(crate) fn read_in_doc<R: Reader>(&mut self, reader: &mut R) {
        reader.skip_whitespace();
        match reader.peek_byte().unwrap_or(b'\0') {
            // b'[' => self.state = Collection(S)
            _ => {}
        };
    }

    fn try_skip_comments<T: Reader>(&self, reader: &mut T) {
        while {
            // do
            reader.skip_whitespace();
            reader.peek_byte_is(b'#')
        } {
            // while
            reader.read_line();
        }
    }
}

#[derive(Copy, Clone)]
pub enum SpanToken {
    ErrorToken(ErrorType),
    Scalar(usize, usize),
    Directive(DirectiveType, usize, usize),
    DocStart,
    DocEnd,
    StreamStart,
    StreamEnd,
}

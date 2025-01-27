use std::collections::VecDeque;

use crate::error::YamlError;
use crate::tokenizer::event::DirectiveType;
use crate::tokenizer::event::YamlEvent::Directive;
use crate::tokenizer::reader::{Reader, StrReader};
use crate::tokenizer::scanner::State::StreamStart;
use crate::tokenizer::{ErrorType, StrIterator};

#[derive(Clone, Default)]
pub struct Scanner {
    state: State,
    tokens: VecDeque<SpanToken>,
    pub(crate) eof: bool,
}

#[derive(Copy, Clone)]
pub enum State {
    StreamStart,
    DocStart,
}

impl Default for State {
    fn default() -> Self {
        StreamStart
    }
}

pub enum Control {
    Continue,
    Eof,
    Err(YamlError),
}

impl Scanner {
    pub fn from_str_reader(string: &str) -> StrIterator<'_> {
        StrIterator {
            state: Default::default(),
            reader: StrReader::new(string),
        }
    }

    pub(crate) fn emit_end_of_stream(&mut self) {
        self.tokens.push_back(SpanToken::StreamEnd);
    }

    pub(crate) fn pop_token(&mut self) -> Option<SpanToken> {
        self.tokens.pop_front()
    }

    pub(crate) fn next_state<R: Reader>(&mut self, reader: &mut R) -> Control {
        match self.state {
            StreamStart => self.read_start_stream(reader),
            _ => return Control::Eof,
        };
        Control::Continue
    }

    pub(crate) fn read_start_stream<T: Reader>(&mut self, reader: &mut T) {
        self.try_skip_comments(reader);
        if reader.peek_byte_is(b'%') {
            if reader.try_read_slice_exact("%YAML") {
                reader.skip_space_tab();
                if let Some(x) = reader.find_next_non_whitespace() {
                    self.tokens.push_back(SpanToken::Directive(
                        DirectiveType::Yaml,
                        reader.pos(),
                        x,
                    ));
                    reader.consume_bytes(x - reader.pos());
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
        self.state = State::DocStart;
        self.tokens.push_back(SpanToken::StreamStart);
    }

    fn try_skip_comments<T: Reader>(&self, reader: &mut T) {
        while {
            // do
            reader.skip_space_tab();
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
    StreamStart,
    StreamEnd,
}

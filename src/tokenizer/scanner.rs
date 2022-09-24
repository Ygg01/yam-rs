use std::collections::VecDeque;

use crate::error::YamlError;
use crate::tokenizer::event::DirectiveType;
use crate::tokenizer::reader::{Reader, StrReader};
use crate::tokenizer::scanner::State::StreamStart;
use crate::tokenizer::StrIterator;

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
    Post,
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
                if let Some(x) = reader.find_fast2_offset(b'\t', b' ') {
                    self.tokens
                        .push_back(SpanToken::Directive(DirectiveType::Yaml, x.0, x.1));
                    reader.consume_bytes(x.1 - x.0);
                    reader.read_line();
                }
            } else if reader.try_read_slice_exact("%TAG") {
                reader.skip_space_tab();
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

    fn read_tag<T: Reader>(&mut self, reader: &mut T) {}
}

#[derive(Copy, Clone)]
pub enum SpanToken {
    Scalar(usize, usize),
    Directive(DirectiveType, usize, usize),
    StreamStart,
    StreamEnd,
}

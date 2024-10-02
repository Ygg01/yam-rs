use crate::error::Error;
use crate::tokenizer::stage2::ParseResult;
use crate::YamlChunkState;
use alloc::string::{String, ToString};

pub trait YamlVisitor<'de> {
    fn visit_error(&mut self, error: Error) -> ParseResult<YamlChunkState>;
}

pub struct EventStringVisitor {
    pub(crate) buffer: String,
}

impl<'vis> YamlVisitor<'vis> for EventStringVisitor {
    fn visit_error(&mut self, error: Error) -> Result<YamlChunkState, Error> {
        self.buffer.push_str("\nERR ");
        self.buffer.push('(');
        self.buffer.push_str(&error.to_string());
        self.buffer.push(')');
        Err(error)
    }
}

impl EventStringVisitor {
    pub fn new_with_hint(hint: Option<usize>) -> Self {
        EventStringVisitor {
            buffer: match hint {
                Some(cap) => String::with_capacity(cap),
                None => String::new(),
            },
        }
    }

    pub fn buffer(self) -> String {
        self.buffer
    }
}

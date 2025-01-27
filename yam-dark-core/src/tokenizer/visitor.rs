use crate::error::Error;
use alloc::string::{String, ToString};

pub trait YamlVisitor<'de> {
    fn visit_error(&mut self, error: Error);
}

pub struct EventStringVisitor {
    pub(crate) buffer: String,
}

impl<'vis> YamlVisitor<'vis> for EventStringVisitor {
    fn visit_error(&mut self, error: Error) {
        self.buffer.push_str("\nERR ");
        self.buffer.push('(');
        self.buffer.push_str(&error.to_string());
        self.buffer.push(')');
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
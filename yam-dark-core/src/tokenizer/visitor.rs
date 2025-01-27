use alloc::format;
use alloc::string::String;
use yam_core::error::{YamlError, YamlResult};

pub trait YamlVisitor<'de> {
    fn visit_error(&mut self, error: YamlError) -> YamlResult<()>;
}

pub struct EventStringVisitor {
    pub(crate) buffer: String,
}

impl<'vis> YamlVisitor<'vis> for EventStringVisitor {
    fn visit_error(&mut self, error: YamlError) -> YamlResult<()> {
        self.buffer.push_str("\nERR ");
        self.buffer.push('(');
        self.buffer.push_str(format!("{:?}", error).as_str());
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

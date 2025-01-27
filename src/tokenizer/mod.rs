use std::fmt::{Display, Formatter};

pub use iterator::EventIterator;
pub use reader::Reader;
pub use reader::StrReader;
pub use spanner::SpanToken;
pub use spanner::Spanner;

mod iterator;
mod reader;
mod spanner;

#[derive(Copy, Clone, Debug)]
pub enum ErrorType {
    NoDocStartAfterTag,
    UnexpectedEndOfFile,
    UnexpectedComment,
    UnexpectedSymbol(char),
    ExpectedDocumentStart,
    ExpectedNewline,
    ExpectedNewlineInFolded,
    ExpectedIndent { actual: usize, expected: usize },
    StartedBlockInFlow,
}

#[derive(Copy, Clone)]
pub enum DirectiveType {
    Yaml,
    Tag,
    Reserved,
}

impl Display for DirectiveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DirectiveType::Yaml => write!(f, "YAML"),
            DirectiveType::Tag => write!(f, "TAG"),
            DirectiveType::Reserved => write!(f, "RESERVED"),
        }
    }
}

use std::fmt::{Display, Formatter};

pub use reader::Reader;
pub use reader::StrReader;
pub use scanner::Scanner;
pub use scanner::SpanToken;

mod event;
mod reader;
mod scanner;

#[derive(Copy, Clone, Debug)]
pub enum ErrorType {
    NoDocStartAfterTag,
    UnexpectedEndOfFile,
    UnexpectedSymbol(char),
    ExpectedDocumentStart,
    ExpectedNewline,
    ExpectedNewlineInFolded,
    ExpectedIndent(u32),
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

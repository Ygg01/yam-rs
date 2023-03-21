pub use iterator::EventIterator;
pub use reader::Reader;
pub use spanner::Lexer;
pub use spanner::LexerToken;
use std::str::from_utf8;
pub use str_reader::StrReader;
pub use iterator::Event;

mod iterator;
mod reader;
mod spanner;
mod str_reader;

pub use iterator::assert_eq_event;

#[derive(Copy, Clone, Debug)]
pub enum ErrorType {
    NoDocStartAfterTag,
    UnexpectedEndOfFile,
    UnexpectedComment,
    UnexpectedSymbol(char),
    ExpectedDocumentStart,
    ExpectedNewline,
    ExpectedNewlineInFolded,
    DirectiveEndMark,
    ExpectedIndent { actual: usize, expected: usize },
    MappingExpectedIndent { actual: usize, expected: usize },
    StartedBlockInFlow,
    UnexpectedEndOfScalar,
    DuplicateKey,
}

pub trait Slicer<'a> {
    fn slice(&self, start: usize, end: usize) -> &'a [u8];
    fn slice_str(&self, start: usize, end: usize) -> &'a str {
        from_utf8(self.slice(start, end)).unwrap_or("")
    }
}

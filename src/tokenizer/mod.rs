pub use iterator::Event;
pub use iterator::EventIterator;
pub use reader::Reader;
pub use lexer::Lexer;
pub use lexer::LexerToken;
use std::str::from_utf8;
pub use str_reader::StrReader;

mod iterator;
mod reader;
mod lexer;
mod str_reader;

pub use iterator::assert_eq_event;

#[derive(Copy, Clone, Debug)]
pub enum ErrorType {
    NoDocStartAfterTag,
    UnexpectedEndOfFile,
    UnexpectedComment,
    ExpectedMapBlock,
    UnexpectedSymbol(char),
    ExpectedDocumentStart,
    ExpectedNewline,
    ExpectedNewlineInFolded,
    DirectiveEndMark,
    ImplicitKeysNeedToBeInline,
    AliasAndAnchor,
    ExpectedIndent { actual: usize, expected: usize },
    StartedBlockInFlow,
    TagNotTerminated,
    UnexpectedEndOfScalar,
    DuplicateKey,
}

pub trait Slicer<'a> {
    fn slice(&self, start: usize, end: usize) -> &'a [u8];
    fn slice_str(&self, start: usize, end: usize) -> &'a str {
        from_utf8(self.slice(start, end)).unwrap_or("")
    }
}

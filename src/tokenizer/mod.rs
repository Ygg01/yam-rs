pub use iterator::EventIterator;
pub use reader::Reader;
pub use spanner::SpanToken;
pub use spanner::Spanner;
pub use str_reader::StrReader;

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
}


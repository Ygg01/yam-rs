pub use iterator::Event;
pub use iterator::EventIterator;
pub use lexer::Lexer;
pub use lexer::LexerToken;
pub use reader::Reader;
use std::str::from_utf8;
pub use str_reader::StrReader;

mod buf_reader;
mod iterator;
mod lexer;
mod reader;
mod str_reader;

pub use iterator::assert_eq_event;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ErrorType {
    NoDocStartAfterTag,
    UnexpectedEndOfFile,
    UnexpectedComment,
    ExpectedMapBlock,
    UnexpectedSymbol(char),
    ExpectedDocumentStart,
    ExpectedDocumentEndOrContents,
    ExpectedDocumentStartOrContents,
    ExpectedNewline,
    ExpectedNewlineInFolded,
    DirectiveEndMark,
    ImplicitKeysNeedToBeInline,
    AliasAndAnchor,
    NodeWithTwoAnchors,
    ExpectedIndentDocStart { actual: u32, expected: u32 },
    UnxpectedIndentDocEnd { actual: u32, expected: u32 },
    ExpectedIndent { actual: u32, expected: u32 },
    StartedBlockInFlow,
    TagNotTerminated,
    UnexpectedEndOfScalar,
    DuplicateKey,
    ExpectedChompBetween1and9,
    TabsNotAllowedAsIndentation,
    TwoDirectivesFound,
    UnexpectedEndOfStream,
    UnsupportedYamlVersion,
    YamlMustHaveOnePart,
    InvalidEscapeCharacter,
    InvalidAnchorDeclaration,
    SpacesFoundAfterIndent,
    UnfinishedTag,
    InvalidTagHandleCharacter { found: char },
    SequenceOnSameLineAsKey,
    UnexpectedScalarAtMapEnd,
    InvalidQuoteIndent { actual: u32, expected: u32 },
}

pub trait Slicer<'a> {
    fn slice(&self, start: usize, end: usize) -> &'a [u8];
    fn slice_str(&self, start: usize, end: usize) -> &'a str {
        from_utf8(self.slice(start, end)).unwrap_or("")
    }
}

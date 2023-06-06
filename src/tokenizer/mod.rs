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
    AliasAndAnchor,
    DirectiveEndMark,
    DuplicateKey,
    ExpectedMapBlock,
    ExpectedDocumentStart,
    ExpectedDocumentEndOrContents,
    ExpectedDocumentStartOrContents,
    ExpectedNewline,
    ExpectedIndentDocStart { actual: u32, expected: u32 },
    ExpectedNewlineInFolded,
    ExpectedChompBetween1and9,
    ExpectedIndent { actual: u32, expected: u32 },
    ImplicitKeysNeedToBeInline,
    InvalidEscapeCharacter,
    InvalidAnchorDeclaration,
    InvalidQuoteIndent { actual: u32, expected: u32 },
    InvalidTagHandleCharacter { found: char },
    MissingWhitespaceBeforeComment,
    MissingFlowClosingBracket,
    NoDocStartAfterTag,
    NodeWithTwoAnchors,
    StartedBlockInFlow,
    SequenceOnSameLineAsKey,
    SpacesFoundAfterIndent,
    TagNotTerminated,
    TabsNotAllowedAsIndentation,
    TwoDirectivesFound,
    UnexpectedEndOfScalar,
    UnxpectedIndentDocEnd { actual: u32, expected: u32 },
    UnexpectedEndOfStream,
    UnsupportedYamlVersion,
    UnexpectedEndOfFile,
    UnexpectedComment,
    UnfinishedTag,
    UnexpectedSymbol(char),
    UnexpectedScalarAtMapEnd,
    YamlMustHaveOnePart,
}

pub trait Slicer<'a> {
    fn slice(&self, start: usize, end: usize) -> &'a [u8];
    fn slice_str(&self, start: usize, end: usize) -> &'a str {
        from_utf8(self.slice(start, end)).unwrap_or("")
    }
}

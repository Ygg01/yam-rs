pub use iterator::Event;
pub use iterator::EventIterator;
pub use lexer::Lexer;
pub use lexer::LexerToken;
pub use reader::Reader;
use std::str::from_utf8;
pub use str_reader::StrReader;

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
    ExpectedChompBetween1and9,
    ExpectedDocumentStart,
    ExpectedDocumentStartOrContents,
    ExpectedDocumentEnd,
    ExpectedDocumentEndOrContents,
    ExpectedNodeButFound { found: char },
    ExpectedNewline,
    ExpectedIndent { actual: u32, expected: u32 },
    ExpectedIndentDocStart { actual: u32, expected: u32 },
    ExpectedNewlineInFolded,
    ImplicitKeysNeedToBeInline,
    InvalidAnchorDeclaration,
    InvalidCommentStart,
    InvalidCommentInScalar,
    InvalidEscapeCharacter,
    InvalidQuoteIndent { actual: u32, expected: u32 },
    InvalidTagHandleCharacter { found: char },
    InvalidScalarStart,
    MissingWhitespaceAfterColon,
    MissingWhitespaceBeforeComment,
    MissingFlowClosingBracket,
    NestedMappingsNotAllowed,
    NoDocStartAfterTag,
    NodeWithTwoAnchors,
    StartedBlockInFlow,
    SequenceOnSameLineAsKey,
    SpacesFoundAfterIndent,
    TagNotTerminated,
    TabsNotAllowedAsIndentation,
    TwoDirectivesFound,
    UnexpectedEndOfScalar,
    UnexpectedIndentDocEnd { actual: u32, expected: u32 },
    UnexpectedComment,
    UnexpectedCommentInScalar,
    UnexpectedDirective,
    UnexpectedEndOfStream,
    UnexpectedEndOfFile,
    UnexpectedSymbol(char),
    UnsupportedYamlVersion,
    UnfinishedTag,
    UnexpectedScalarAtNodeEnd,
    YamlMustHaveOnePart,
}

pub trait Slicer<'a> {
    fn slice(&self, start: usize, end: usize) -> &'a [u8];
    fn slice_str(&self, start: usize, end: usize) -> &'a str {
        from_utf8(self.slice(start, end)).unwrap_or("")
    }
}

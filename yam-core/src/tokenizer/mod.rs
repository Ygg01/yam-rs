use core::str::from_utf8;
pub use iterator::Event;
pub use iterator::EventIterator;
pub use lexer::Lexer;
pub use lexer::{LexerToken, DirectiveState};
pub use reader::{LexMutState, Reader};
pub use str_reader::StrReader;

mod iterator;
mod lexer;
mod reader;
mod str_reader;

pub use iterator::assert_eq_event;

use self::lexer::PropType;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ErrorType {
    AliasAndAnchor,
    DirectiveEndMark,
    DuplicateKey,
    ColonMustBeOnSameLineAsKey,
    ExpectedChompBetween1and9,
    ExpectedDocumentStart,
    ExpectedDocumentStartOrContents,
    ExpectedDocumentEnd,
    ExpectedDocumentEndOrContents,
    ExpectedNodeButFound { found: char },
    ExpectedMapBlock,
    ExpectedSeqStart,
    ExpectedNewline,
    ExpectedNewlineInFolded,
    ExpectedIndent { actual: u32, expected: u32 },
    ExpectedIndentDocStart { actual: u32, expected: u32 },
    ExpectedWhiteSpaceAfterProperty,
    ImplicitKeysNeedToBeInline,
    InvalidAnchorIndent { actual: u32, expected: u32 },
    InvalidAnchorDeclaration,
    InvalidCommentStart,
    InvalidCommentInScalar,
    InvalidEscapeCharacter,
    InvalidMappingValue,
    InvalidMapEnd,
    InvalidMapItemIndent,
    InvalidQuoteIndent { actual: u32, expected: u32 },
    InvalidTagHandleCharacter { found: char },
    InvalidScalarStart,
    InvalidScalarAtNodeEnd,
    InvalidScalarIndent,
    MissingWhitespaceAfterColon,
    MissingWhitespaceBeforeComment,
    MissingFlowClosingBracket,
    NestedMappingsNotAllowed,
    NoDocStartAfterTag,
    NodeWithTwoProperties(PropType),
    NodeWithTwoTags,
    PropertyAtStartOfSequence,
    StartedBlockInFlow,
    SequenceOnSameLineAsKey,
    SpacesFoundAfterIndent,
    TagNotTerminated,
    TagMustStartWithExclamation,
    TabsNotAllowedAsIndentation,
    TwoDirectivesFound,
    UnexpectedEndOfScalar,
    UnexpectedIndentDocEnd { actual: u32, expected: u32 },
    UnexpectedComment,
    UnexpectedCommentInScalar,
    UnexpectedDirective,
    UnexpectedEndOfStream,
    UnexpectedEndOfDocument,
    UnexpectedEndOfFile,
    UnexpectedSeqAtNodeEnd,
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

extern crate core;

pub mod loader;
pub mod node;

pub use loader::{Mapping, Sequence, YamlDoc, YamlEntry};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::ops::Range;
use std::str::Utf8Error;

pub type Mark = Range<usize>;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ScalarType {
    /// Unquoted string type like:
    /// ```yaml
    ///   multiline
    ///   string
    /// ```
    Plain,
    /// Folded string type like:
    /// ```yaml
    ///   >
    ///     folded
    ///     string
    /// ```
    Folded,
    /// Folded string type like:
    /// ```yaml
    ///   |
    ///     folded
    ///     string
    /// ```
    Literal,
    /// Single quote string which permits any symbol inside
    /// E.g. :
    /// ```yaml
    /// ' This is a quoted string
    ///    with ''quoted'' string within.'
    /// ```
    SingleQuote,
    /// Single quote string which permits any symbol inside
    /// E.g. :
    /// ```yaml
    /// "This is a quoted string
    ///    with \"double quoted\" string within."
    /// ```
    DoubleQuote,
}

impl Display for ScalarType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ScalarType::Plain => write!(f, ":"),
            ScalarType::Folded => write!(f, ">"),
            ScalarType::Literal => write!(f, "|"),
            ScalarType::SingleQuote => write!(f, "'"),
            ScalarType::DoubleQuote => write!(f, "\""),
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum TokenType<'input> {
    StreamStart,
    StreamEnd,
    DocumentStart,
    DocumentEnd,
    BlockSequenceStart,
    BlockMappingStart,
    BlockEnd,
    BlockEntry,
    FlowEntry,
    Key,
    Value,
    FlowSequenceStart,
    FlowSequenceEnd,
    FlowMappingStart,
    FlowMappingEnd,
    Alias(Cow<'input, str>),
    Anchor(Cow<'input, str>),
    VersionDirective {
        major: u8,
        minor: u8,
    },
    TagDirective {
        handle: Cow<'input, str>,
        prefix: Cow<'input, str>,
    },
    Tag {
        handle: Cow<'input, str>,
        suffix: Cow<'input, str>,
    },
    Scalar {
        scalar_type: ScalarType,
        value: Cow<'input, str>,
    },
}

impl<'input> TokenType<'input> {
    ///
    /// # Safety
    ///
    /// The passed Vec<u8> must contain only valid UTF-8.
    pub unsafe fn new_tag_unchecked(handle_raw: Vec<u8>, suffix_raw: Vec<u8>) -> TokenType<'input> {
        unsafe {
            TokenType::Tag {
                handle: Cow::Owned(String::from_utf8_unchecked(handle_raw)),
                suffix: Cow::Owned(String::from_utf8_unchecked(suffix_raw)),
            }
        }
    }

    ///
    /// # Safety
    ///
    /// The passed Vec<u8> must contain only valid UTF-8.
    pub unsafe fn new_tag_directive_unchecked(
        handle_raw: Vec<u8>,
        prefix_raw: Vec<u8>,
    ) -> TokenType<'input> {
        unsafe {
            TokenType::TagDirective {
                handle: Cow::Owned(String::from_utf8_unchecked(handle_raw)),
                prefix: Cow::Owned(String::from_utf8_unchecked(prefix_raw)),
            }
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum ChompIndicator {
    /// `-` final line break and any trailing empty lines are excluded from the scalar’s content
    Strip,
    ///  ` ` final line break character is preserved in the scalar’s content
    Clip,
    /// `+` final line break and any trailing empty lines are considered to be part of the scalar’s content
    Keep,
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Marker {
    /// index in bytes of the input string.
    pub pos: usize,
    /// Column of mark. One indexed.
    pub col: u32,
    /// Column of mark. One indexed.
    pub line: u32,
}

#[derive(Clone, Copy, PartialEq, Debug, Eq, Default)]
pub struct Span {
    pub start: Marker,
    pub end: Marker,
}

impl Span {
    pub fn new(start: Marker, end: Marker) -> Self {
        Span { start, end }
    }

    pub fn empty(mark: Marker) -> Self {
        Span {
            start: mark,
            end: mark,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum DirectiveType {
    Yaml,
    Tag,
    Reserved,
}

/// A specialized `Result` type where the error is hard-wired to [`Error`].
///
/// [`Error`]: enum.Error.html
pub type YamlResult<T> = Result<T, YamlError>;
pub type ScanResult = Result<(), YamlError>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum YamlError {
    Utf8(Utf8Error),
    Io(String),
    UnexpectedEof,
    /// Input decoding error. If `encoding` feature is disabled, contains `None`,
    /// otherwise contains the UTF-8 decoding error
    NonDecodable(Option<Utf8Error>),
    ScannerErr {
        mark: Marker,
        info: String,
    },
    NoDocument,
}

impl YamlError {
    pub fn new_str(marker: Marker, info: &str) -> Self {
        YamlError::ScannerErr {
            mark: marker,
            info: info.to_string(),
        }
    }
}

impl From<Utf8Error> for YamlError {
    /// Creates a new `Error::NonDecodable` from the given error
    #[inline]
    fn from(error: Utf8Error) -> YamlError {
        YamlError::NonDecodable(Some(error))
    }
}

/// A YAML tag.
#[derive(Clone, PartialEq, Debug, Eq, Ord, PartialOrd, Hash)]
pub struct Tag {
    /// Handle of the tag (`!` included).
    pub handle: String,
    /// The suffix of the tag.
    pub suffix: String,
}

impl Tag {
    /// Returns whether the tag is a YAML tag from the core schema (`!!str`, `!!int`, ...).
    ///
    /// The YAML specification specifies [a list of
    /// tags](https://yaml.org/spec/1.2.2/#103-core-schema) for the Core Schema. This function
    /// checks whether _the handle_ (but not the suffix) is the handle for the YAML Core Schema.
    ///
    /// # Return
    /// Returns `true` if the handle is `tag:yaml.org,2002`, `false` otherwise.
    #[must_use]
    pub fn is_yaml_core_schema(&self) -> bool {
        self.handle == "tag:yaml.org,2002:"
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        if self.handle == "!" {
            write!(f, "!{}", self.suffix)
        } else {
            write!(f, "{}!{}", self.handle, self.suffix)
        }
    }
}

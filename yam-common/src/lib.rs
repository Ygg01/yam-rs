extern crate core;
#[deny(missing_docs)]
pub(crate) mod loader;
pub(crate) mod node;

pub use crate::loader::LoadableYamlNode;
pub use crate::loader::NodeMapping;
pub use crate::loader::NodeSequence;
pub use crate::loader::YamlDocAccess;
pub use crate::node::YamlCloneNode;
pub use loader::{Mapping, Sequence, YamlDoc, YamlEntry};
use std::borrow::Cow;
use std::fmt::{Display, Formatter};
use std::str::Utf8Error;

/// Represents the different types of scalar values in YAML with distinct formatting styles.
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

///
/// Represents the different types of tokens that can be encountered in the input stream.
///
/// This enum is designed for use in tokenizing structured data formats, particularly YAML.
/// Variants describe the specific types of tokens, including structural markers (`StreamStart`, `DocumentStart`),
/// compound structures (`BlockSequenceStart`, `FlowMappingStart`), and specific data values (`Scalar`, `Alias`).
///
/// Supports cloning, equality comparison, and debug printing.
///
/// # Type Parameters
/// - `'input`: Lifetime of the input data, used for borrowed data within certain token variants.
///
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
    Comment(Cow<'input, str>),
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
    /// The passed `Vec<u8>` must contain only valid UTF-8.
    #[must_use]
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
    /// The passed `Vec<u8>` must contain only valid UTF-8.
    #[must_use]
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
/// Chomp indicator of target block scalar
#[derive(PartialEq, Clone, Copy)]
pub enum ChompIndicator {
    /// `-` final line break and any trailing empty lines are excluded from the scalar’s content
    Strip,
    ///  ` ` final line break character is preserved in the scalar’s content
    Clip,
    /// `+` final line break and any trailing empty lines are considered to be part of the scalar’s content
    Keep,
}

/// Represents a marker within an input string for tracking position.
///
/// The `Marker` struct is often used to store information about a
/// specific location in text-based data. It keeps track of the byte
/// index, as well as the one-indexed column and line numbers.
///
#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Marker {
    /// The zero-based index in bytes of the input string.
    pub pos: usize,
    ///  The one-indexed column number corresponding to the marker's position.
    ///  This is useful for reporting the horizontal location of the marker.
    pub col: u32,
    /// The one-indexed line number corresponding to the marker's position.
    /// This is often used when tracking the vertical location of the marker.
    pub line: u32,
}

/// Span that denotes a start and end of a token
#[derive(Clone, Copy, PartialEq, Debug, Eq, Default)]
pub struct Span {
    /// Start of the `Span`.
    pub start: Marker,
    /// End of the `Span`.
    pub end: Marker,
}

impl Span {
    #[must_use]
    pub fn new(start: Marker, end: Marker) -> Self {
        Span { start, end }
    }

    #[must_use]
    pub fn empty(mark: Marker) -> Self {
        Span {
            start: mark,
            end: mark,
        }
    }
}

///
/// Represents the type of directives that can be encountered.
///
/// * `Tag` - Represents a Tag directive, which is used to associate a handle with a URI prefix for shorthand node tags in YAML.
/// * `Reserved` - Represents a reserved directive type, which is not defined by the YAML 1.2 specification but is reserved for future use or custom extensions.
///
///
#[derive(Copy, Clone, PartialEq)]
pub enum DirectiveType {
    /// Represents a YAML directive, typically used to define version or encoding information in a YAML document. For example:
    /// ```yaml
    /// %YAML 1.1
    /// #^------^
    /// ```
    Yaml,
    /// Represents a Tag directive, which is used to associate a handle with a URI prefix for shorthand node tags in YAML. For example:
    /// ```yaml
    /// %TAG ! !foo
    /// #^--------^
    /// ```
    Tag,
    /// Anything eles that might appear in directive.
    Reserved,
}

/// A specialized `Result` type where the error is hard-wired to [`YamlError`].
pub type YamlResult<T> = Result<T, YamlError>;
/// A result often returned by the `YamlScanner`. It's hard-wired to [`YamlError`].
pub type ScanResult = Result<(), YamlError>;

/// Enumeration representing all YAML errors
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum YamlError {
    /// Error when decoding to UTF8
    Utf8(Utf8Error),
    /// Io error when accessing the input.
    Io(String),
    /// Didn't expect and end of file at that position.
    UnexpectedEof,
    /// Input decoding error. If `encoding` feature is disabled, contains `None`,
    /// otherwise contains the UTF-8 decoding error
    NonDecodable(Option<Utf8Error>),
    ///
    /// Represents an error encountered during scanning or parsing operations.
    ///
    /// `ScannerErr` includes information about the location of the error and a
    /// description of what went wrong.
    ///
    /// # Fields
    /// - `mark: Marker`
    ///   Indicates the location or position in the scanned input where the error occurred.
    ///   This provides context for debugging or fixing the issue by pointing out where
    ///   the problem lies.
    ///
    /// - `info: String`
    ///   A description or message detailing the nature of the error. This provides a human-readable
    ///   explanation of what caused the error, aiding in understanding and resolving the issue.
    ScannerErr { mark: Marker, info: String },
    /// Expected a document but found none.
    NoDocument,
}

impl Display for YamlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            YamlError::Utf8(utf8_error) => write!(f, "UTF-8 decoding error: {utf8_error}"),
            YamlError::Io(io_error) => write!(f, "IO error: {io_error}"),
            YamlError::UnexpectedEof => write!(f, "Unexpected end of file"),
            YamlError::NonDecodable(utf8_error) => {
                write!(f, "Non-decodable input: {utf8_error:?}")
            }
            YamlError::ScannerErr { mark, info } => {
                write!(f, "Scanner error at marker {mark:?}: {info}")
            }
            YamlError::NoDocument => write!(f, "No document found"),
        }
    }
}

impl YamlError {
    /// Creates a new `YamlError::ScannerErr` instance with the provided marker and informational string.
    ///
    /// # Parameters
    /// - `marker`: A `Marker` value that indicates the position or context of the error.
    /// - `info`: A string slice containing descriptive information about the error.
    ///
    /// # Returns
    /// A new instance of `YamlError` with the variant `ScannerErr`.
    ///
    /// # Attributes
    /// - `#[must_use]`: Indicates that the return value of this function must be used by the caller.
    ///
    /// # Example
    /// ```
    /// use yam_common::{Marker, YamlError};
    /// let marker = Marker { pos: 0, col: 1, line: 1}; // Example Marker initialization
    /// let error = YamlError::new_str(marker, "Unexpected token in YAML.");
    /// ```
    #[must_use]
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
    ///
    /// Creates a new instance with the specified handle and suffix.
    ///
    /// # Parameters
    ///
    /// * `handle` - A `String` representing the main identifier or name to initialize.
    /// * `suffix` - A `String` value appended or associated with the `handle`.
    ///
    /// # Returns
    /// Will create a new tag instance
    ///
    /// # Example
    ///
    /// ```rust
    /// use yam_common::Tag;
    /// let instance = Tag::new("example_handle", "example_suffix");
    /// ```
    pub fn new<S: Into<String>>(handle: S, suffix: S) -> Self {
        let handle: String = handle.into();
        let suffix = suffix.into();
        Tag { handle, suffix }
    }

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

/// Check if the string can be expressed a valid literal block scalar.
/// The YAML spec supports all of the following in block literals except `#xFEFF`:
/// ```no_compile
///     #x9 | #xA | [#x20-#x7E]                /* 8 bit */
///   | #x85 | [#xA0-#xD7FF] | [#xE000-#xFFFD] /* 16 bit */
///   | [#x10000-#x10FFFF]                     /* 32 bit */
/// ```
#[inline]
#[doc(hidden)]
#[must_use]
pub fn is_valid_literal_block_scalar(string: &str) -> bool {
    string.chars().all(|character: char|
        matches!(character, '\t' | '\n' | '\x20'..='\x7e' | '\u{0085}' | '\u{00a0}'..='\u{d7fff}'))
}

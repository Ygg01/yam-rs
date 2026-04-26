//! Module dealing with parsing YAML documents into a series of events.
//!
mod buffered_source;
mod char_utils;
mod parser;
mod scanner;
mod source;

use crate::prelude::{ScalarType, YamlError};
use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::fmt::{Display, Formatter};
pub use parser::EventReceiver;
pub use parser::SpannedEventReceiver;
pub use parser::{Event, Parser, ScalarValue};
pub use source::Source;
pub use source::StrSource;

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

/// Check if the string can be expressed a valid literal block scalar.
/// The YAML spec supports all literals except `#xFEFF`:
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
    /// use yam_core::prelude::Tag;
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.handle == "!" {
            write!(f, "!{}", self.suffix)
        } else {
            write!(f, "{}!{}", self.handle, self.suffix)
        }
    }
}

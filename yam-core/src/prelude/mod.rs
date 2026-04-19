//! Use `yam_core::prelude::*` to import common components and traits.
use alloc::borrow::Cow;
use alloc::collections::{BTreeMap, LinkedList};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt;
use core::fmt::{Display, Formatter};
use core::marker::PhantomData;
use core::str::Utf8Error;
pub use loader::MappingLike;
pub use loader::SequenceLike;
pub use loader::YamlLoader;
pub use scalar::YamlScalar;
pub use yaml::Yaml;
pub use yaml_data::YamlData;

mod loader;
mod scalar;
mod spanned_yaml;
mod yaml;
mod yaml_data;

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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
    /// - `#[must_use]`: Caller must use the return value.
    ///
    /// # Example
    /// ```
    /// use yam_core::prelude::{Marker, YamlError};
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

///
/// Represents the different types of nodes that can exist in a data structure or a parsing scenario.
///
/// Each variant of the `NodeType` enum corresponds to a specific type of node. See variants for details.
///
/// This enum is marked with the following traits:
/// - `Copy`: Allows the enum to be copied, rather than moved, when assigned or passed to a function.
/// - `Clone`: Allows for explicitly creating a copy of the enum instance.
/// - `Debug`: Enables formatting the enum for debugging purposes.
///
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeType {
    /// Represents an invalid or malformed node.
    Bad,
    /// Represents a node with a `null` value.
    Null,
    /// Represents a node that contains a string value.
    String,
    /// Represents a node that contains a boolean value (`true` or `false`).
    Bool,
    /// Represents a node that contains a floating-point number.
    Floating,
    /// Represents a node that contains an integer value.
    Integer,
    /// Represents a node that acts as an alias or reference to another node.
    Alias,
    /// Represents a node that contains a mapping (key-value pairs), similar to a dictionary or map.
    Mapping,
    /// Represents a node that contains a sequence (ordered list of elements), similar to an array or list.
    Sequence,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum YamlAccessError {
    ExpectedMapping,
    ExpectedSequence,
}

///   Trait that provides access and utility functions for interacting with a YAML document's structure and nodes.
///
///  This trait establishes a unified interface for working with YAML nodes, allowing you to inspect,
///  access, and convert between different representations of the YAML components such as scalars,
///  collections, mappings, or sequences.
///
///  # Associated Types
///  - `Node`: A cloneable type that represents an individual node within the YAML document.
///
///  # Methods
///  ## Variant Checks
///  - `is_bad_value(&self) -> bool`: Checks if the node represents an invalid value.
///  - `is_null(&self) -> bool`: Checks if the node is a null value.
///  - `is_string(&self) -> bool`: Checks if the node is a string.
///  - `is_bool(&self) -> bool`: Checks if the node is a boolean.
///  - `is_floating_point(&self) -> bool`: Checks if the node is a floating-point number.
///  - `is_integer(&self) -> bool`: Checks if the node is an integer.
///  - `is_alias(&self) -> bool`: Checks if the node is an alias.
///  - `is_non_empty_collection(&self) -> bool`: Determines whether the node represents a non-empty collection.
///  - `is_collection(&self) -> bool`: Determines whether the node represents a collection of either
///    a mapping or a sequence. The default implementation checks `is_mapping()` or `is_sequence()`.
///  - `is_mapping(&self) -> bool`: Determines whether the node represents a mapping.
///  - `is_sequence(&self) -> bool`: Determines whether the node represents a sequence.
///
///  ## Accessor Methods
///  These methods include both immutable and mutable access patterns to the underlying data types:
///  - `as_bool(&self) -> Option<bool>`: Retrieves the value as a boolean if applicable.
///  - `as_bool_mut(&mut self) -> Option<&mut bool>`: Mutable access to the boolean value.
///  - `as_i64(&self) -> Option<i64>`: Retrieves the value as a 64-bit integer if applicable.
///  - `as_i64_mut(&mut self) -> Option<&mut i64>`: Mutable access to the integer value.
///  - `as_f64(&self) -> Option<f64>`: Retrieves the value as a 64-bit floating-point number if applicable.
///  - `as_f64_mut(&mut self) -> Option<&mut f64>`: Mutable access to the floating-point value.
///  - `as_sequence(&self) -> Option<&NodeSequence<Self::Node>>`: Retrieves a reference to the value as a sequence if applicable.
///  - `as_sequence_mut(&mut self) -> Option<&mut NodeSequence<Self::Node>>`: Mutable access to the sequence value.
///  - `as_mapping(&self) -> Option<&NodeMapping<'input, Self::Node>>`: Retrieves a reference to the value as a mapping if applicable.
///  - `as_mapping_mut(&mut self) -> Option<&NodeMapping<'input, Self::Node>>`: Mutable access to the mapping value.
///  - `as_str(&self) -> Option<&str>`: Retrieves the value as a string slice if applicable.
///  - `as_str_mut(&mut self) -> Option<&mut str>`: Mutable access to the string value.
///  - `get_tag(&self) -> Option<Tag>`: Retrieves the YAML tag associated with the node if applicable.
///  - `get_type(&self) -> NodeType`: Returns a simplified [`NodeType`] of the given node.
///
///  ## Conversion Methods
///  These methods consume the node and attempt to convert it into specific types:
///  - `into_bool(self) -> Option<bool>`: Converts the node into a boolean if possible.
///  - `into_string(self) -> Option<String>`: Converts the node into a `String` if possible.
///  - `into_cow(self) -> Option<Cow<'input, str>>`: Converts the node into a `Cow` string if possible.
///  - `into_f64(self) -> Option<f64>`: Converts the node into a floating-point value if possible.
///  - `into_i64(self) -> Option<i64>`: Converts the node into an integer value if possible.
///  - `into_mapping(self) -> Option<NodeMapping<'input, Self::Node>>`: Converts the node into a mapping if possible.
///  - `into_sequence(self) -> Option<NodeSequence<Self::Node>>`: Converts the node into a sequence if possible.
pub trait YamlDocAccess<'input>: Clone {
    /// Type of node used in Sequence or Mapping
    type OutNode: Clone;
    /// Type of sequence node being used.
    type SequenceNode;
    /// Type of mapping node being used.
    type MappingNode;

    /// Determines whether the current node is a bad value.
    ///
    /// # Returns
    /// * `true` - if the value meets the criteria for being "bad".
    /// * `false` - if the value does not meet the criteria for being "bad".
    ///
    /// # Example
    /// ```
    /// use yam_core::prelude::{YamlDoc, YamlDocAccess};
    /// let bad_value = YamlDoc::BadValue;
    ///
    /// assert!(bad_value.is_bad_value());
    ///```
    fn is_bad_value(&self) -> bool {
        matches!(self.get_type(), NodeType::Bad)
    }

    fn key_from_usize(index: usize) -> Self;

    fn key_from_str(index: &str) -> Self;

    /// Determines whether the current node is a null value.
    ///
    /// # Returns
    /// * `true` - if the value current node is null.
    /// * `false` - otherwise.
    ///
    /// # Example
    /// ```
    /// use yam_core::prelude::{YamlDoc, YamlDocAccess};
    ///
    /// let bad_value = YamlDoc::Null;
    /// assert!(bad_value.is_null());
    ///```
    fn is_null(&self) -> bool {
        matches!(self.get_type(), NodeType::Null)
    }

    /// Determines whether the current node is a string.
    ///
    /// # Returns
    /// * `true` - if the value is a string.
    /// * `false` - otherwise.
    ///
    /// # Note
    /// The specific definition of a "bad" value should be implemented
    /// in the context of the struct or enum that provides this method.
    ///
    /// # Example
    /// ```
    /// use std::borrow::Cow;
    /// use yam_core::prelude::{YamlDoc, YamlDocAccess};
    /// let bad_value = YamlDoc::String(Cow::Owned("yes.".into()));
    ///
    /// assert!(bad_value.is_string());
    ///```
    fn is_string(&self) -> bool {
        matches!(self.get_type(), NodeType::String)
    }

    /// Determines whether the current node is a boolean value.
    ///
    /// # Returns
    /// * `true` - if the node contains a boolean value.
    /// * `false` - otherwise.
    ///
    /// # Example
    /// ```
    /// use yam_core::prelude::{YamlDoc, YamlDocAccess};
    /// let bad_value = YamlDoc::Bool(false);
    ///
    /// assert!(bad_value.is_bool());
    ///```
    fn is_bool(&self) -> bool {
        matches!(self.get_type(), NodeType::Bool)
    }
    /// Determines whether the current node is a floating point value.
    ///
    /// # Returns
    /// * `true` - if the node contains a floating point value.
    /// * `false` - otherwise.
    ///
    /// # Example
    /// ```
    /// use yam_core::prelude::{YamlDoc, YamlDocAccess};
    /// let bad_value = YamlDoc::FloatingPoint(3.14);
    ///
    /// assert!(bad_value.is_floating_point());
    ///```
    fn is_floating_point(&self) -> bool {
        matches!(self.get_type(), NodeType::Floating)
    }
    /// Determines whether the current node is an integer point value.
    ///
    /// # Returns
    /// * `true` - if the node contains an integer point value.
    /// * `false` - otherwise.
    ///
    /// # Example
    /// ```
    /// use yam_core::prelude::{YamlDoc, YamlDocAccess};
    /// let bad_value = YamlDoc::Integer(12);
    ///
    /// assert!(bad_value.is_integer());
    ///```
    fn is_integer(&self) -> bool {
        matches!(self.get_type(), NodeType::Integer)
    }

    /// Determines whether the current node is an alias
    ///
    /// # Returns
    /// * `true` - if the node is an alias.
    /// * `false` - otherwise.
    ///
    /// # Example
    /// ```
    /// use yam_core::prelude::{YamlDoc, YamlDocAccess};
    /// let alias = YamlDoc::Alias(12);
    ///
    /// assert!(alias.is_alias());
    ///```
    fn is_alias(&self) -> bool {
        matches!(self.get_type(), NodeType::Alias)
    }
    ///
    /// Checks if the collection is non-empty.
    ///
    /// This method determines whether the collection contains
    /// at least one element.
    ///
    /// # Returns
    /// * `true` if the collection has one or more elements.
    /// * `false` otherwise
    ///
    fn is_non_empty_collection(&self) -> bool;

    ///
    /// Checks if the collection is a mapping or a sequence.
    ///
    /// This method determines whether the value is a collection
    ///
    /// # Returns
    /// * `true` if the collection is a mapping or a sequence.
    /// * `false` otherwise
    ///
    fn is_collection(&self) -> bool {
        matches!(self.get_type(), NodeType::Sequence | NodeType::Mapping)
    }

    ///
    /// Determines if the implementing object represents a mapping.
    ///
    /// # Returns
    ///
    /// * `true` if the object is considered a mapping.
    /// * `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use yam_core::prelude::{YamlDoc, YamlDocAccess};
    ///
    /// let example = YamlDoc::Bool(true);
    /// assert!(!example.is_mapping());
    /// ```
    ///
    /// This method can be used to verify whether an object follows a mapping
    /// structure or behavior based on its implementation.
    ///
    fn is_mapping(&self) -> bool {
        matches!(self.get_type(), NodeType::Mapping)
    }

    ///
    /// Determines if the implementing object represents a sequence.
    ///
    /// # Returns
    ///
    /// * `true` if the object is considered a sequence.
    /// * `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use yam_core::prelude::{YamlDoc, YamlDocAccess};
    ///
    /// let example = YamlDoc::Bool(true);
    /// assert!(!example.is_sequence());
    /// ```
    ///
    /// This method can be used to verify whether an object follows a sequential
    /// structure or behavior based on its implementation.
    ///
    fn is_sequence(&self) -> bool {
        matches!(self.get_type(), NodeType::Sequence)
    }

    /// Attempts to interpret the current instance as a boolean value.
    ///
    /// # Returns
    ///
    /// - `Some(true)` if the instance can be interpreted as a `true` value.
    /// - `Some(false)` if the instance can be interpreted as a `false` value.
    /// - `None` if the instance cannot be reasonably interpreted as a boolean.
    ///
    fn as_bool(&self) -> Option<bool>;

    /// Provides a mutable reference to the inner boolean value if the type supports it.
    ///
    /// This method attempts to convert the current instance into a mutable reference
    /// to a boolean (`bool`) if the type allows such a conversion. If the conversion
    /// is not possible, it returns `None`.
    ///
    /// # Returns
    /// - `Some(&mut bool)` if the type contains a mutable boolean value.
    /// - `None` if the conversion to a mutable boolean reference is not applicable.
    ///
    fn as_bool_mut(&mut self) -> Option<&mut bool>;

    /// Converts the value of the implementing type to an `i64`, if possible.
    ///
    /// # Returns
    /// - `Some(i64)` if the conversion is successful.
    /// - `None` if the conversion cannot be performed or if the value
    ///   cannot be represented as an `i64`.
    fn as_i64(&self) -> Option<i64>;

    /// Provides a mutable reference to the inner `i64` value if the type supports it.
    ///
    /// This method attempts to convert the current instance into a mutable reference
    /// to an i64 if the type allows such a conversion. If the conversion
    /// is not possible, it returns `None`.
    ///
    /// # Returns
    /// - `Some(&mut i64)` if the type contains a mutable boolean value.
    /// - `None` if the conversion to a mutable boolean reference is not applicable.
    ///
    fn as_i64_mut(&mut self) -> Option<&mut i64>;

    /// Converts the value of the implementing type to a `f64`, if possible.
    ///
    /// # Returns
    /// - `Some(f64)` if the conversion is successful.
    /// - `None` if the conversion cannot be performed or if the value
    ///   cannot be represented as an `f64`.
    fn as_f64(&self) -> Option<f64>;

    /// Provides a mutable reference to the inner `f64` value if the type supports it.
    ///
    /// This method attempts to convert the current instance into a mutable reference
    /// to an `f64 ` if the type allows such a conversion. If the conversion
    /// is not possible, it returns `None`.
    ///
    /// # Returns
    /// - `Some(&mut f64)` if the type contains a mutable boolean value.
    /// - `None` if the conversion to a mutable boolean reference is not applicable.
    ///
    fn as_f64_mut(&mut self) -> Option<&mut f64>;

    /// Returns an optional reference to the sequence of nodes (`NodeSequence`).
    ///
    /// This method provides a way to access the underlying sequence of nodes if it exists,
    /// for the current instance. If the instance does not contain a sequence of nodes,
    /// `None` is returned.
    ///
    /// # Returns
    /// * `Result<&Self::SequenceNode, YamlAccessError>` -
    ///   A reference to the node sequence wrapped in `Ok` if the underlying node is `Sequence`.
    ///
    /// # Errors
    /// If calling `as_sequence` on a non-mapping node.
    fn as_sequence(&self) -> Result<&Self::SequenceNode, YamlAccessError>;

    /// Returns a mutable reference to the sequence of nodes (`NodeSequence`).
    ///
    /// This method provides a way to access the underlying sequence of nodes if it exists,
    /// for the current instance. If the instance does not contain a sequence of nodes,
    /// `None` is returned.
    ///
    /// # Returns
    /// * `Result<&mut Self::SequenceNode, YamlAccessError>` -
    ///   A reference to the node sequence wrapped in `Ok` if the underlying node is `Sequence`.
    /// # Errors
    /// If calling `as_sequence_mut` on a non-sequence node.
    fn as_sequence_mut(&mut self) -> Result<&mut Self::SequenceNode, YamlAccessError>;

    /// Returns an optional reference to the mapping of nodes (`NodeMapping`).
    ///
    /// This method provides a way to access the underlying mapping of nodes if it exists,
    /// for the current instance. If the instance does not contain a sequence of nodes,
    /// `None` is returned.
    ///
    /// # Returns
    /// * `Result<&Self::MappingNode, YamlAccessError>` -
    ///   A reference to the node sequence wrapped in `Ok` if the underlying node is `Mapping`.
    ///
    /// # Errors
    /// If calling `as_mapping` on a non-mapping node.
    fn as_mapping(&self) -> Result<&Self::MappingNode, YamlAccessError>;

    /// Returns a mutable reference to the mapping of nodes (`NodeMapping`).
    ///
    /// This method provides a way to access the underlying mapping of nodes if it exists,
    /// for the current instance. If the instance does not contain a sequence of nodes,
    /// `None` is returned.
    ///
    /// # Returns
    /// * `Result<&mut Self::MappingNode, YamlAccessError>` -
    ///   A reference to the node sequence wrapped in `Ok` if the underlying node is `Mapping`.
    ///
    /// # Errors
    /// If calling `as_mapping_mut` on a non-mapping node.
    fn as_mapping_mut(&mut self) -> Result<&mut Self::MappingNode, YamlAccessError>;

    /// Converts the current instance into an `Option` containing a string slice (`&str`).
    ///
    /// # Returns
    ///
    /// - `Some(&str)` if the underlying node is string
    /// - `None` otherwise.
    fn as_str(&self) -> Option<&str>;

    /// Returns a mutable reference `Option` containing an underlying string slice (`&str`).
    ///
    /// # Returns
    ///
    /// - `Some(&mut str)` if the underlying node is string
    /// - `None` otherwise.
    ///
    fn as_str_mut(&mut self) -> Option<&mut str>;

    /// Provides mutable access to the sequence within the implementing type.
    ///
    /// This method allows for getting a mutable reference to a `Vec` associated with
    /// the implementing type. This enables modification of the underlying vector, such
    /// as adding, removing, or altering elements.
    ///
    /// # Panics
    /// If called on a node that isn't a mapping.
    ///
    /// # Returns
    /// A mutable reference to a `Vec` of the type implementing this method.
    ///
    /// # Examples
    /// ```rust
    /// use yam_core::prelude::{YamlDoc, YamlDocAccess};
    ///
    /// let mut instance = YamlDoc::Sequence(vec![YamlDoc::Bool(true)]);
    /// let sequence = instance.sequence_mut();
    /// sequence.push(YamlDoc::Bool(false));
    /// ```
    fn sequence_mut(&mut self) -> &mut Self::SequenceNode;

    /// Provides access to the sequence within the implementing type.
    ///
    /// This method allows for getting a mutable reference to a `Vec` associated with
    /// the implementing type. This enables access of the underlying vector, but not modification.
    ///
    /// # Panics
    /// If called on a node that isn't a mapping.
    ///
    /// # Returns
    /// A reference to a `Vec` of the type implementing this method.
    ///
    /// # Examples
    /// ```rust
    /// use yam_core::prelude::{YamlDoc, YamlDocAccess};
    ///
    /// let mut instance = YamlDoc::Sequence(vec![YamlDoc::Bool(true)]);
    /// let sequence = instance.sequence_mut();
    /// sequence.push(YamlDoc::Bool(false));
    /// ```
    fn sequence(&self) -> &Self::SequenceNode;

    /// Provides mutable access to the mapping within the implementing type.
    ///
    /// This method allows for getting a mutable reference to a `Vec` of `YamlEntry` associated with
    /// the implementing type. This enables modification of the underlying vector, such
    /// as adding, removing, or altering elements.
    ///
    /// # Returns
    /// A mutable reference to a `Vec` of the type implementing this method.
    ///
    /// # Panics
    /// If called on a node that isn't a mapping.
    ///
    /// # Examples
    /// ```rust
    ///
    /// use std::borrow::Cow;
    /// use yam_core::prelude::{YamlDoc, YamlEntry, YamlDocAccess};
    ///
    /// let entry1 = YamlEntry::new("key".into(), "value".into());
    /// let entry2 = YamlEntry::new("another_key".into(), "value2".into());
    /// let mut instance = YamlDoc::Mapping(vec![entry1]);
    /// let sequence = instance.mapping_mut();
    /// sequence.push(entry2);
    /// ```
    fn mapping_mut(&mut self) -> &mut Self::MappingNode;

    /// Provides access to the mapping within the implementing type.
    ///
    /// This method allows for getting a reference to a `Vec` of `YamlEntry` associated with
    /// the implementing type. This enables modification of the underlying vector, such
    /// as adding, removing, or altering elements.
    ///
    /// # Returns
    /// A reference to a `Vec` of the type implementing this method.
    ///
    /// # Panics
    /// If called on a node that isn't a mapping.
    ///
    /// # Examples
    /// ```rust
    ///
    /// use std::borrow::Cow;
    /// use yam_core::prelude::{YamlDoc, YamlEntry, YamlDocAccess};
    ///
    /// let entry1 = YamlEntry::new("key".into(), "value".into());
    /// let entry2 = YamlEntry::new("another_key".into(), "value2".into());
    /// let mut instance = YamlDoc::Mapping(vec![entry1]);
    /// let sequence = instance.mapping();
    /// sequence.push(entry2);
    /// ```
    fn mapping(&self) -> &Self::MappingNode;

    /// Retrieves the `Tag` associated with the current instance, if it exists.
    ///
    /// # Returns
    ///
    /// * `Option<Tag>` - Returns `Some(Tag)` if a tag is present, otherwise `None`.
    ///
    /// This method is useful for checking or retrieving metadata or identifiers
    /// tied to the instance.
    ///
    fn get_tag(&self) -> Option<Tag>;

    /// Retrieves the type of the current node.
    ///
    /// This method returns the `NodeType` of the node on which it is called.
    /// The `NodeType` can represent different kinds of nodes, allowing
    /// consumers of this method to determine the specific type or purpose
    /// of the node.
    ///
    /// # Returns
    /// * `NodeType` - An enum value representing the type of the node.
    ///
    /// # Example
    /// ```rust
    /// use yam_core::prelude::{NodeType, Yaml, YamlDocAccess};
    ///
    /// let node = YamlDoc::from(32i64);
    /// let node_type = node.get_type();
    /// assert_eq!(node_type, NodeType::Integer);
    /// ```
    fn get_type(&self) -> NodeType;

    /// Converts the value of the type implementing this method into an `Option<bool>`.
    ///
    /// # Returns
    /// - `Some(true)` or `Some(false)` if the conversion is successful,
    ///   depending on the implementation.
    /// - `None` if the conversion is not possible or represents an invalid state.
    ///
    fn into_bool(self) -> Option<bool> {
        self.as_bool()
    }

    /// Converts the value of the type implementing this method into an `Option<String>`.
    ///
    /// # Returns
    /// - `Some(String)` if the conversion is successful,
    ///   depending on the implementation.
    /// - `None` if the conversion is not possible or represents an invalid state.
    ///
    fn into_string(self) -> Option<String>;

    /// Converts the value of the implementing type into an `Option<f64>`.
    ///
    /// # Returns
    /// - `Some(f64)` if the conversion is successful.
    /// - `None` if the conversion cannot be performed.
    ///
    ///
    /// This function is particularly useful when working with types that may need
    /// to be represented as `f64` for numerical computations or interoperability.
    ///
    fn into_f64(self) -> Option<f64> {
        self.as_f64()
    }

    /// Converts the value of the implementing type into an `Option<i64>`.
    ///
    /// # Returns
    /// - `Some(i64)` if the conversion is successful.
    /// - `None` if the conversion cannot be performed.
    ///
    ///
    /// This function is particularly useful when working with types that may need
    /// to be represented as `f64` for numerical computations or interoperability.
    ///
    fn into_i64(self) -> Option<i64> {
        self.as_i64()
    }

    ///  Converts the current structure into a `NodeMapping`, if possible.
    ///
    ///  This function attempts to transform the current object into a `NodeMapping` type,
    ///  which is a specific representation of node data used within the system. If the
    ///  conversion is not possible, the function will return `None`.
    ///
    ///  # Returns
    ///  - `Some(NodeMapping<'input, Self::Node>)` if the conversion was successful.
    ///  - `None` if the conversion could not be performed.
    ///
    ///  # Usage
    ///   This function is typically invoked on types that implement the necessary
    ///   conversion logic to map their internal representation into a `NodeMapping`.
    ///   Ensure that the type supports the conversion before calling this method to
    ///   avoid receiving `None`.
    ///  # See also
    ///   [`YamlDocAccess::is_mapping`]
    fn into_mapping(self) -> Option<Self::MappingNode>;

    ///  Converts the current structure into a `NodeSequence`, if possible.
    ///
    ///  This function attempts to transform the current object into a `NodeSequence` type,
    ///  which is a specific representation of node data used within the system. If the
    ///  conversion is not possible, the function will return `None`.
    ///
    ///  # Returns
    ///  - `Some(NodeSequence<Self::Node>)` if the conversion was successful.
    ///  - `None` if the conversion could not be performed.
    ///
    ///  # Usage
    ///   This function is typically invoked on types that implement the necessary
    ///   conversion logic to map their internal representation into a `NodeMapping`.
    ///   Ensure that the type supports the conversion before calling this method to
    ///   avoid receiving `None`.
    ///
    /// # See also
    ///   [`YamlDocAccess::is_sequence`]
    ///
    fn into_sequence(self) -> Option<Self::SequenceNode>;

    ///
    /// Converts the current instance into a tagged version of itself.
    ///
    /// This function associates the given `tag` with the instance and returns
    /// the new tagged instance. The tagged instance retains all the original
    /// properties along with the added tag.
    ///
    /// # Parameters
    /// - `tag`: A `Cow<'input, Tag>` (copy-on-write smart pointer) representing
    ///   the tag that will be associated with the instance.
    ///
    /// # Returns
    /// The new instance with the provided `tag` attached.
    ///
    /// # Attributes
    /// - `#[must_use]`: This function is annotated with `#[must_use]`, meaning the
    ///   returned tagged instance must be used. Ignoring the returned value
    ///   will result in a compiler warning.
    #[must_use]
    fn into_tagged(self, tag: Cow<'input, Tag>) -> Self;

    ///
    /// Constructs an instance of `Self` using a bad or default value.
    ///
    /// # Attributes
    /// - `#[must_use]`: Caller must use this return value or the warning will be emitted.
    ///
    /// # Parameters
    /// - `_span: Span`: A `Span` parameter that gives the bad element _span.
    ///
    /// # Returns
    /// An instance of `Self` created using the `Self::bad_value()` method,
    /// which represents a bad or default value.
    ///
    /// # Note
    /// Since the provided parameter `_: Span` is unused, this function might
    /// not utilize it for any meaningful computation.
    #[must_use]
    fn bad_span_value(_span: Span) -> Self;

    ///
    /// This method represents a constructor or initializer for creating an instance of `Self`
    /// that represents a "bad" or invalid value.
    ///
    /// # Returns
    /// An instance of `Self` that is considered to have a problematic, invalid, or undesirable state.
    /// This method could be used as a placeholder, for testing, or to handle specific error conditions.
    ///
    /// # Note
    /// The specific meaning of "bad" or "invalid" depends on the implementation
    /// within the type that provides this method.
    ///
    #[must_use]
    #[inline]
    fn bad_value() -> Self {
        Self::bad_span_value(Span::default())
    }

    ///
    /// Consumes the current value, leaving the object in an uninitialized or default state,
    /// and returns an instance of `Self` that represents the previous state of the object.
    ///
    /// # Returns
    /// A new instance of `Self` containing the previous state of the object.
    ///
    /// # Notes
    /// - This method has the `#[must_use]` attribute, meaning the return value must be used;
    ///   otherwise, a compiler warning will be issued.
    /// - After calling this method, the current instance may no longer hold meaningful data,
    ///   depending on the implementation.
    ///
    /// # Example
    /// ```rust
    /// use yam_core::prelude::*;
    ///
    /// let mut value = Yaml::from(true);
    /// let previous_value = value.take();
    ///
    /// assert_eq!(previous_value, Yaml(YamlData::Scalar(YamlScalar::Bool(true))));
    /// assert_eq!(value, Yaml::bad_value());
    /// ```
    #[must_use]
    fn take(&mut self) -> Self {
        core::mem::replace(self, Self::bad_value())
    }

    ///
    /// Sets the starting marker for the current instance.
    ///
    /// # Parameters
    /// - `_marker: Marker`: A placeholder for a marker that signifies the starting point.
    ///   This parameter is currently unused in the method's implementation.
    ///
    /// # Returns
    /// - `Self`: Returns the instance of the current type unchanged.
    ///
    /// # Attributes
    /// - `#[must_use]`: Indicates that the return value of this method must be used,
    ///   as it likely holds significance in the context it is called.
    ///
    /// Note: While the `Marker` parameter is unused within the method, it might
    /// be included for future implementation or API design purposes.
    ///
    #[must_use]
    fn with_start(self, _marker: Marker) -> Self {
        self
    }

    ///
    /// Sets the ending marker for the current instance.
    ///
    /// # Parameters
    /// - `_marker: Marker`: A placeholder for a marker that signifies the ending point.
    ///   This parameter is currently unused in the method's implementation.
    ///
    /// # Returns
    /// - `Self`: Returns the instance of the current type unchanged.
    ///
    /// # Attributes
    /// - `#[must_use]`: Indicates that the return value of this method must be used,
    ///   as it likely holds significance in the context it is called.
    ///
    /// Note: While the `Marker` parameter is unused within the method, it might
    /// be included for future implementation or API design purposes.
    ///
    #[must_use]
    fn with_end(self, _marker: Marker) -> Self {
        self
    }

    ///
    /// Sets the ending marker for the current instance.
    ///
    /// # Parameters
    /// - `_marker: Marker`: A placeholder for a marker that signifies the ending point.
    ///   This parameter is currently unused in the method's implementation.
    ///
    /// # Returns
    /// - `Self`: Returns the instance of the current type unchanged.
    ///
    /// # Attributes
    /// - `#[must_use]`: Indicates that the return value of this method must be used,
    ///   as it likely holds significance in the context it is called.
    ///
    /// Note: While the `Marker` parameter is unused within the method, it might
    /// be included for future implementation or API design purposes.
    ///
    #[must_use]
    fn with_span(self, _span: Span) -> Self {
        self
    }
}

///
///
///  A data structure representing an entry in a YAML file, consisting of a key-value pair.
///
///  The `YamlEntry` struct is generic over the type `T`, which represents the type of the key and
///  value. The generic type `T` must implement the `Clone` trait to ensure the key and value
///  can be duplicated as needed.
///
///  The struct also includes a marker field, `_marker`, utilizing `PhantomData` to associate
///  a specific lifetime `'input` with the `YamlEntry`. This is useful for ensuring that any
///  references within the key or value maintain proper lifetimes.
///
///  # Type Parameters
///  - `'input`: Lifetime parameter used by the `_marker` field to link the `YamlEntry` instance
///    with a specific lifetime context.
///  - `T`: Generic type representing the key and value in the YAML entry. It must implement `Clone`.
#[derive(Debug, Clone, PartialEq)]
pub struct YamlEntry<'input, T> {
    /// Represents the key of the YAML entry. It is of type `T`.
    pub key: T,
    /// Represents the value of the YAML entry. It is of type `T`.
    pub value: T,
    pub _marker: PhantomData<&'input ()>,
}

impl<T: Clone> YamlEntry<'_, T> {
    /// Creates a new `YamlEntry` with the given key and value.
    ///
    /// # Parameters
    ///
    /// - `key`: The key for the YAML entry.
    /// - `value`: The value associated with the key in the YAML entry.
    ///
    /// # Returns
    ///
    /// A new instance of `YamlEntry` containing the specified key and value.
    pub fn new(key: T, value: T) -> Self {
        YamlEntry {
            key,
            value,
            _marker: PhantomData,
        }
    }
}

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
    /// Single quote string that permits any symbol inside
    /// E.g. :
    /// ```yaml
    /// ' This is a quoted string
    ///    with ''quoted'' string within.'
    /// ```
    SingleQuote,
    /// Single quote string that permits any symbol inside
    /// E.g. :
    /// ```yaml
    /// "This is a quoted string
    ///    with \"double quoted\" string within."
    /// ```
    DoubleQuote,
}

impl Display for ScalarType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ScalarType::Plain => write!(f, ":"),
            ScalarType::Folded => write!(f, ">"),
            ScalarType::Literal => write!(f, "|"),
            ScalarType::SingleQuote => write!(f, "'"),
            ScalarType::DoubleQuote => write!(f, "\""),
        }
    }
}

pub trait IsEmpty: Clone {
    fn is_collection_empty(&self) -> bool;
}

pub trait ToMutStr {
    fn mut_str(&mut self) -> &mut str;
}

impl<'a> ToMutStr for Cow<'a, str> {
    fn mut_str(&mut self) -> &mut str {
        self.to_mut()
    }
}

impl ToMutStr for str {
    fn mut_str(&mut self) -> &mut str {
        self
    }
}

impl<T> IsEmpty for Vec<T>
where
    T: Clone,
{
    fn is_collection_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<T> IsEmpty for LinkedList<T>
where
    T: Clone,
{
    fn is_collection_empty(&self) -> bool {
        self.is_empty()
    }
}

impl<K, V> IsEmpty for BTreeMap<K, V>
where
    K: Clone,
    V: Clone,
{
    fn is_collection_empty(&self) -> bool {
        self.is_empty()
    }
}

//! Use `yam_core::prelude::*` to import common components and traits.
pub use crate::emitter::YamlEmitter;
pub use crate::node::YamlScalar;
pub use crate::node::span_yam::SpanYam;
pub use crate::node::yaml::Yaml;
pub use crate::node::yaml_data::YamlData;
pub use crate::parsing::{InMemSource, Source, Tag};

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
pub use loader::YamLoader;
#[cfg(feature = "serde")]
pub use serde::YamlFormatter;

mod loader;
#[cfg(feature = "serde")]
mod serde;

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

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Span [{}..{}] From (row: {}, col {}) To (row: {}, col {})",
            self.start.pos,
            self.end.pos,
            self.start.line,
            self.start.col,
            self.end.line,
            self.end.col,
        )
    }
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

impl Marker {
    /// Creates a new instance of the struct with the specified position, column, and line.
    ///
    /// # Parameters
    /// - `pos`: The position of the item, typically representing a character index in the source.
    /// - `col`: The column number associated with the position, starting from 1.
    /// - `line`: The line number associated with the position, starting from 1.
    ///
    /// # Returns
    /// A new instance of the struct initialized with the provided `pos`, `col`, and `line` values.
    ///
    /// # Example
    /// ```
    /// use yam_core::prelude::Marker;
    /// let instance = Marker::new(42, 5, 3);
    /// assert_eq!(instance.pos, 42);
    /// assert_eq!(instance.col, 5);
    /// assert_eq!(instance.line, 3);
    /// ```
    #[must_use]
    pub fn new(pos: usize, col: u32, line: u32) -> Self {
        Self { pos, col, line }
    }
}

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
    UnExpectedEvent {
        expected: &'static str,
        found: &'static str,
    },
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
            YamlError::UnExpectedEvent { expected, found } => {
                write!(f, "Expected event '{expected}' but found '{found}' instead")
            }
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
    Text,
    /// Represents a node that contains a boolean value (`true` or `false`).
    Bool,
    /// Represents a node that contains a floating-point number.
    FloatingPoint,
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
/// # Methods
/// ## Construction Methods
/// These methods construct the Node for given input.
/// - [`bad_value() -> Self`](YamlDocAccess::bad_value): Creates a node with bad value. If encountered it signals an error.
///   It serializes into an empty value much like [`null()`]
/// - [`null() -> Self`](YamlDocAccess::null): Creates a node with a missing value (i.e., null).
/// - [`from_bool(bool) -> Self`](YamlDocAccess::from_bool): Creates a node based on a [`bool`] value.
/// - [`from_char(bool) -> Self`](YamlDocAccess::from_char): Creates a node based on a [`char`] value. It serializes into a string.
/// - [`from_string(String) -> Self`](YamlDocAccess::from_string): Creates a node based on a [`String`]-like value. Value will be copied into the Node.
/// - [`from_str(&str) -> Self`](YamlDocAccess::from_str): Creates a node based on a [`str`] value. Value will be borrowed if possible. Useful for zero-copy parsing.
/// - [`from_integer(impl Into<i64>) -> Self`](YamlDocAccess::from_integer): Creates a node based on a [`i64`]-like value.
/// - [`from_float(impl Into<f64>) -> Self`](YamlDocAccess::from_float): Creates a node based on a [`f64`]-like value.
/// - [`with_start(Marker) -> Self`](YamlDocAccess::with_start): Creates a node new node from [`Marker`] representing the start of a Node's span.
/// - [`with_end(Marker) -> Self`](YamlDocAccess::with_start): Creates a node new node from [`Marker`] representing the end of a Node's  span.
/// - [`with_span(Span) -> Self`](YamlDocAccess::with_start): Creates a node new node from [`Span`] representing the Node's span.
///
///  ## Variant Checks
///  - [`is_bad_value(&self) -> bool`](YamlDocAccess::is_bad_value): Checks if the node represents an invalid value.
///  - [`is_null(&self) -> bool`](YamlDocAccess::is_null): Checks if the node is a null value.
///  - [`is_string(&self) -> bool`](YamlDocAccess::is_string): Checks if the node is a string.
///  - [`is_bool(&self) -> bool`](YamlDocAccess::is_bool): Checks if the node is a boolean.
///  - [`is_floating_point(&self) -> bool`](YamlDocAccess::is_floating_point): Checks if the node is a floating-point number.
///  - [`is_integer(&self) -> bool`](YamlDocAccess::is_integer): Checks if the node is an integer.
///  - [`is_non_empty_collection(&self) -> bool`](YamlDocAccess::is_non_empty_collection): Determines whether the node represents a non-empty collection.
///  - [`is_collection(&self) -> bool`](YamlDocAccess::is_collection): Determines whether the node represents a collection of either
///    a mapping or a sequence. The default implementation checks `is_mapping()` or `is_sequence()`.
///  - [`is_mapping(&self) -> bool`](YamlDocAccess::is_mapping): Determines whether the node represents a mapping.
///  - [`is_sequence(&self) -> bool`](YamlDocAccess::is_sequence): Determines whether the node represents a sequence.
///  - [`is_tagged(&self) -> bool`](YamlDocAccess::is_tagged): Determines whether the node represents has a tag.
///  - [`is_binary_schema(&self) -> bool`](YamlDocAccess::is_binary_schema): Determines whether the node represents has a `!!binary` tag.
///  ## Accessor Methods
///  These methods include both immutable and mutable access patterns to the underlying data types:
///  - [`as_bool(&self) -> Option<bool>`](YamlDocAccess::as_bool): Retrieves the value as a boolean if applicable.
///  - [`as_bool_mut(&mut self) -> Option<&mut bool>`](YamlDocAccess::as_bool_mut): Mutable access to the boolean value.
///  - [`as_i64(&self) -> Option<i64>`](YamlDocAccess::as_i64): Retrieves the value as a 64-bit integer if applicable.
///  - [`as_i64_mut(&mut self) -> Option<&mut i64>`](YamlDocAccess::as_i64_mut): Mutable access to the integer value.
///  - [`as_f64(&self) -> Option<f64>`](YamlDocAccess::as_f64): Retrieves the value as a 64-bit floating-point number if applicable.
///  - [`as_f64_mut(&mut self) -> Option<&mut f64>`](YamlDocAccess::as_f64_mut): Mutable access to the floating-point value.
///  - [`as_sequence(&self) -> Option<&NodeSequence<Self::Node>>`](YamlDocAccess::as_sequence): Retrieves a reference to the value as a sequence if applicable.
///  - [`as_sequence_mut(&mut self) -> Option<&mut NodeSequence<Self::Node>>`](YamlDocAccess::as_sequence_mut): Mutable access to the sequence value.
///  - [`as_mapping(&self) -> Option<&NodeMapping<'input, Self::Node>>`](YamlDocAccess::as_mapping): Retrieves a reference to the value as a mapping if applicable.
///  - [`as_mapping_mut(&mut self) -> Option<&NodeMapping<'input, Self::Node>>`](YamlDocAccess::as_mapping_mut): Mutable access to the mapping value.
///  - [`as_str(&self) -> Option<&str>`](YamlDocAccess::as_str): Retrieves the value as a string slice if applicable.
///  - [`as_str_mut(&mut self) -> Option<&mut str>`](YamlDocAccess::as_str_mut): Mutable access to the string value.
///  - [`get_tag(&self) -> Option<Tag>`](YamlDocAccess::get_tag): Retrieves the YAML tag associated with the node if applicable.
///  - [`get_type(&self) -> NodeType`](YamlDocAccess::get_type): Returns a simplified [`NodeType`] of the given node.
///
///  ## Conversion Methods
///  These methods consume the node and attempt to convert it into specific types:
///  - [`into_bool(self) -> Option<bool>`](YamlDocAccess::into_bool): Converts the node into a boolean if possible.
///  - [`into_string(self) -> Option<String>`](YamlDocAccess::into_string): Converts the node into a `String` if possible.
///  - [`into_cow(self) -> Option<Cow<'input, str>>`](YamlDocAccess::into_cow): Converts the node into a `Cow` string if possible.
///  - [`into_f64(self) -> Option<f64>`](YamlDocAccess::into_f64): Converts the node into a floating-point value if possible.
///  - [`into_i64(self) -> Option<i64>`](YamlDocAccess::into_f64): Converts the node into an integer value if possible.
///  - [`into_mapping(self) -> Option<NodeMapping<'input, Self::Node>>`](YamlDocAccess::into_mapping): Converts the node into a mapping if possible.
///  - [`into_sequence(self) -> Option<NodeSequence<Self::Node>>`](YamlDocAccess::into_sequence): Converts the node into a sequence if possible.
///  - [`take(&mut self) -> Self`](YamlDocAccess::take): Gets the node out and replaces it with a default value.
pub trait YamlDocAccess<'input>: Clone {
    /// Type of node used in Sequence or Mapping
    type OutNode: Clone;
    /// Type of sequence node being used.
    type SequenceNode;
    /// Type of mapping node being used.
    type MappingNode;

    /// Determines whether the current node is a bad value.
    ///
    /// # Example
    /// ```
    /// use yam_core::prelude::{Yaml, YamlDocAccess};
    /// let bad_value = Yaml::<f64>::bad_value();
    ///
    /// assert!(bad_value.is_bad_value());
    ///```
    fn is_bad_value(&self) -> bool {
        matches!(self.get_type(), NodeType::Bad)
    }

    /// Determines whether the current node is a null value.
    ///
    /// # Example
    /// ```
    /// use yam_core::prelude::{Yaml, YamlDocAccess};
    ///
    /// let nul = Yaml::<f64>::null();
    /// assert!(nul.is_null());
    ///```
    fn is_null(&self) -> bool {
        matches!(self.get_type(), NodeType::Null)
    }

    /// Determines whether the current node is a string.
    ///
    /// # Example
    /// ```
    /// use std::borrow::Cow;
    /// use yam_core::prelude::{Yaml, YamlDocAccess};
    /// let str = Yaml::from("yes.");
    ///
    /// assert!(str.is_string());
    ///```
    fn is_string(&self) -> bool {
        matches!(self.get_type(), NodeType::Text)
    }

    /// Determines whether the current node is a boolean value.
    ///
    /// # Example
    /// ```
    /// use yam_core::prelude::{Yaml, YamlDocAccess};
    /// let boolean = Yaml::from(false);
    ///
    /// assert!(boolean.is_bool());
    ///```
    fn is_bool(&self) -> bool {
        matches!(self.get_type(), NodeType::Bool)
    }
    /// Determines whether the current node is a floating point value.
    ///
    /// # Example
    /// ```
    /// use yam_core::prelude::{Yaml, YamlDocAccess};
    /// let fp = Yaml::from(3.14);
    ///
    /// assert!(fp.is_floating_point());
    ///```
    fn is_floating_point(&self) -> bool {
        matches!(self.get_type(), NodeType::FloatingPoint)
    }
    /// Determines whether the current node is an integer point value.
    ///
    /// # Example
    /// ```
    /// use yam_core::prelude::{Yaml, YamlDocAccess};
    /// let bad_value = Yaml::from(12);
    ///
    /// assert!(bad_value.is_integer());
    ///```
    fn is_integer(&self) -> bool {
        matches!(self.get_type(), NodeType::Integer)
    }

    #[doc(hidden)]
    /// Determines whether the current node is an alias
    ///
    /// # Example
    /// ```
    /// use yam_core::prelude::{Yaml, YamlData, YamlDocAccess};
    /// let alias : Yaml = Yaml(YamlData::Alias(3));
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
    /// at least one element. It will return false if the node is not a collection or an empty
    /// collection.
    fn is_non_empty_collection(&self) -> bool;

    /// Checks if the node is a collection (i.e. a mapping or a sequence).
    fn is_collection(&self) -> bool {
        matches!(self.get_type(), NodeType::Sequence | NodeType::Mapping)
    }

    ///
    /// Determines if the implementing object represents a mapping.
    ///
    /// # Example
    ///
    /// ```rust
    /// use yam_core::prelude::{Yaml, YamlDocAccess};
    ///
    /// let example = Yaml::from(true);
    /// assert!(!example.is_mapping());
    /// ```
    fn is_mapping(&self) -> bool {
        matches!(self.get_type(), NodeType::Mapping)
    }

    ///
    /// Determines if the implementing object represents a sequence.
    /// Returning `true` if it's a sequence, and `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use yam_core::prelude::{Yaml, YamlData, YamlDocAccess, YamlScalar};
    ///
    /// let example = Yaml(YamlData::Sequence(vec![
    ///     Yaml::from(1), Yaml::from(2)
    /// ]));
    /// assert!(example.is_sequence());
    /// ```
    fn is_sequence(&self) -> bool {
        matches!(self.get_type(), NodeType::Sequence)
    }

    /// Determines if the implementing object is a tagged node.
    /// Returning `true` if node has a tag, and `false` otherwise.
    ///
    /// # Example
    /// ```rust
    /// use std::borrow::Cow;
    /// use yam_core::prelude::{Yaml, YamlDocAccess ,Tag, YamlData};
    ///
    /// let yaml : Yaml = Yaml(YamlData::Tagged(Cow::Owned(Tag::new("test", "???")), Box::new(Yaml::from_bool(true))));
    /// assert!(yaml.is_tagged());
    /// ```
    fn is_tagged(&self) -> bool;

    /// Determines if the implementing object is a `binary schema` node.
    /// Returning `true` if node has a binary schema tag, and `false` otherwise.
    /// # Example
    /// ```rust
    /// use std::borrow::Cow;
    /// use yam_core::prelude::{Yaml, YamlDocAccess ,Tag, YamlData};
    ///
    /// let yaml : Yaml = Yaml(
    ///     YamlData::Tagged(
    ///         Cow::Owned(Tag::binary_schema()),
    ///         Box::new(Yaml::from_str("atarst")),
    ///     )
    /// );
    /// assert!(yaml.is_binary_schema());
    /// ```
    fn is_binary_schema(&self) -> bool {
        matches!(self.get_tag(), Some(x) if x.is_binary_schema())
    }

    /// Attempts to interpret the current instance as a boolean value.
    fn as_bool(&self) -> Option<bool>;

    /// Provides a mutable reference to the inner boolean value if the type supports it.
    ///
    /// This method attempts to convert the current instance into a mutable reference
    /// to a boolean (`bool`) if the type allows such a conversion. If the conversion
    /// is not possible, it returns `None`.
    fn as_bool_mut(&mut self) -> Option<&mut bool>;

    /// Converts the value of the implementing type to an `i64`, if possible.
    fn as_i64(&self) -> Option<i64>;

    /// Provides a mutable reference to the inner `i64` value if the type supports it.
    ///
    /// This method attempts to convert the current instance into a mutable reference
    /// to an i64 if the type allows such a conversion. If the conversion
    /// is not possible, it returns `None`.
    fn as_i64_mut(&mut self) -> Option<&mut i64>;

    /// Converts the value of the implementing type to a `f64`, if possible. Otherwise, returns
    /// `None`.
    fn as_f64(&self) -> Option<f64>;

    /// Provides a mutable reference to the inner `f64` value if the type supports it.
    ///
    /// This method attempts to convert the current instance into a mutable reference
    /// to an `f64 ` if the type allows such a conversion. If the conversion
    /// is not possible, it returns `None`.
    fn as_f64_mut(&mut self) -> Option<&mut f64>;

    /// Returns an optional reference to the sequence of nodes (`NodeSequence`).
    ///
    /// This method provides a way to access the underlying sequence of nodes if it exists,
    /// for the current instance. If the instance does not contain a sequence of nodes,
    /// `None` is returned.
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
    /// # Errors
    /// If calling `as_sequence_mut` on a non-sequence node.
    fn as_sequence_mut(&mut self) -> Result<&mut Self::SequenceNode, YamlAccessError>;

    /// Returns an optional reference to the mapping of nodes (`NodeMapping`).
    ///
    /// This method provides a way to access the underlying mapping of nodes if it exists,
    /// for the current instance. If the instance does not contain a sequence of nodes,
    /// `None` is returned.
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
    /// # Errors
    /// If calling `as_mapping_mut` on a non-mapping node.
    fn as_mapping_mut(&mut self) -> Result<&mut Self::MappingNode, YamlAccessError>;

    /// Converts the current instance into an `Option` containing a string slice (`&str`).
    fn as_str(&self) -> Option<&str>;

    /// Returns a mutable reference `Option` containing an underlying string slice (`&str`).
    fn as_str_mut(&mut self) -> Option<&mut str>;

    /// Provides mutable access to the sequence within the implementing type.
    ///
    /// # Safety
    /// Calling this on a non-sequence node is undefined-behavior.
    unsafe fn sequence_mut(&mut self) -> &mut Self::SequenceNode;

    #[doc(hidden)]
    /// Provides unsafe access to the sequence without checking the node.
    ///
    /// # Safety
    /// Calling this on a non-sequence node is undefined-behavior.
    unsafe fn sequence(&self) -> &Self::SequenceNode;

    #[doc(hidden)]
    /// Provides unsafe mutable access to the mapping without checking the node.
    ///
    /// # Safety
    /// Calling this on a non-mapping node is undefined-behavior.
    unsafe fn mapping_mut(&mut self) -> &mut Self::MappingNode;

    #[doc(hidden)]
    /// Provides unsafe access to the mapping without checking the node.
    ///
    /// # Safety
    /// Calling this on a non-mapping node is undefined-behavior.
    unsafe fn mapping(&self) -> &Self::MappingNode;

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
    /// # Example
    /// ```rust
    /// use yam_core::prelude::{NodeType, Yaml, YamlDocAccess};
    ///
    /// let node = Yaml::from(32i64);
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

    /// Converts the value of the type implementing this method into an `Option<String>`.
    ///
    /// # Returns
    /// - `Some(String)` if the conversion is successful,
    ///   depending on the implementation.
    /// - `None` if the conversion is not possible or represents an invalid state.
    ///
    fn into_cow(self) -> Option<Cow<'input, str>>;

    /// Converts the value of the implementing type into an `Option<f64>`.
    ///
    /// This function is particularly useful when working with types that may need
    /// to be represented as `f64` for numerical computations or interoperability.
    ///
    fn into_f64(self) -> Option<f64> {
        self.as_f64()
    }

    /// Converts the value of the implementing type into an `Option<i64>`.
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
    ///
    /// # Attributes
    /// - `#[must_use]`: This function is annotated with `#[must_use]`, meaning the
    ///   returned tagged instance must be used. Ignoring the returned value
    ///   will result in a compiler warning.
    #[must_use]
    fn into_tagged(self, tag: Cow<'input, Tag>) -> Self;

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
    /// # Attributes
    /// - `#[must_use]`: Caller must use this return value or the warning will be emitted.
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

    /// Creates a new instance of the type with a "null" value.
    ///
    /// This function is typically used to indicate an empty value.
    ///
    /// # Example
    /// ```rust
    /// use yam_core::prelude::{Yaml, YamlDocAccess};
    ///
    /// let value : Yaml = Yaml::null();
    /// assert!(value.is_null())
    /// ```
    fn null() -> Self;

    #[must_use]
    /// Creates a new instance of the type from a `bool` value.
    ///
    /// This function is typically used to initialize the type to bool scalar.
    ///
    /// # Example
    /// ```rust
    /// use yam_core::prelude::{Yaml, YamlDocAccess};
    ///
    /// let value : Yaml = Yaml::from_bool(true);
    /// assert!(value.is_bool())
    /// ```
    fn from_bool(b: bool) -> Self;

    #[must_use]
    /// Creates a new string scalar from a char value.
    ///
    /// # Example
    /// ```rust
    /// use yam_core::prelude::{Yaml, YamlDocAccess};
    ///
    /// let value : Yaml = Yaml::from_char('b');
    /// assert!(value.is_string())
    /// ```
    fn from_char(chr: char) -> Self;

    #[must_use]
    /// Creates a new instance of the type from a char value.
    ///
    /// # Example
    /// ```rust
    /// use yam_core::prelude::{Yaml, YamlDocAccess};
    ///
    /// let value : Yaml = Yaml::from_string("32".to_string());
    /// assert!(value.is_string())
    /// ```
    fn from_string(input: impl Into<String>) -> Self;

    #[must_use]
    /// Creates a new string scalar from a slice.
    ///
    /// # Example
    /// ```rust
    /// use yam_core::prelude::{Yaml, YamlDocAccess};
    ///
    /// let value : Yaml = Yaml::from_str("32");
    /// assert!(value.is_string())
    /// ```
    fn from_str(input: &'input str) -> Self;

    #[must_use]
    /// Creates a new instance of the type from a float value.
    ///
    /// # Example
    /// ```rust
    /// use yam_core::prelude::{Yaml, YamlDocAccess};
    ///
    /// let value : Yaml = Yaml::from_str("32");
    /// assert!(value.is_string())
    /// ```
    fn from_float(input: impl Into<f64>) -> Self;

    #[must_use]
    /// Creates a new integer scalar from a value convertible into `i64`.
    ///
    /// # Example
    /// ```rust
    /// use yam_core::prelude::{Yaml, YamlDocAccess};
    ///
    /// let value : Yaml = Yaml::from_integer(32);
    /// assert!(value.is_integer())
    /// ```
    fn from_integer(input: impl Into<i64>) -> Self;

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
    _marker: PhantomData<&'input ()>,
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
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum ScalarType {
    #[default]
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

pub trait ToMutStr<'a> {
    fn as_mut_str(&mut self) -> &mut str;
    fn as_str(&self) -> &str;
    fn from_string(input: impl Into<String>) -> Self;
    fn from_str(input: &'a str) -> Self;
    fn into_string(self) -> String;

    fn into_cow(self) -> Cow<'a, str>;
}

impl<'a> ToMutStr<'a> for Cow<'a, str> {
    fn as_mut_str(&mut self) -> &mut str {
        self.to_mut()
    }

    fn as_str(&self) -> &str {
        self.as_ref()
    }

    fn from_string(input: impl Into<String>) -> Self {
        Cow::Owned(input.into())
    }

    fn from_str(input: &'a str) -> Self {
        Cow::Borrowed(input)
    }

    fn into_string(self) -> String {
        Cow::into_owned(self)
    }

    fn into_cow(self) -> Cow<'a, str> {
        self
    }
}

impl ToMutStr<'static> for String {
    fn as_mut_str(&mut self) -> &mut str {
        self
    }

    fn as_str(&self) -> &str {
        self
    }

    fn from_string(input: impl Into<String>) -> Self {
        input.into()
    }

    fn from_str(input: &'static str) -> Self {
        String::from(input)
    }

    fn into_string(self) -> String {
        self
    }

    fn into_cow(self) -> Cow<'static, str> {
        Cow::Owned(self)
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

pub trait ToMut<T>
where
    T: Copy,
{
    fn as_mut_val(&mut self) -> &mut T;
    fn as_owned(&self) -> T;

    fn into_owned(input: T) -> Self;
}

impl ToMut<i64> for i64 {
    #[inline]
    fn as_mut_val(&mut self) -> &mut i64 {
        self
    }

    #[inline]
    fn as_owned(&self) -> i64 {
        *self
    }

    #[inline]
    fn into_owned(input: i64) -> Self {
        input
    }
}

impl ToMut<f64> for f64 {
    #[inline]
    fn as_mut_val(&mut self) -> &mut f64 {
        self
    }

    #[inline]
    fn as_owned(&self) -> f64 {
        *self
    }

    #[inline]
    fn into_owned(input: f64) -> Self {
        input
    }
}

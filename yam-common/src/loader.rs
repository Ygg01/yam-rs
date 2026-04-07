//! Loader
//!
use crate::{Marker, ScalarType, Span, Tag};
use std::borrow::Cow;
use std::marker::PhantomData;
use std::mem;

///
/// Trait representing a loadable YAML node with various utility methods for manipulation
/// and inspection of YAML data structures. Each implementation can handle tagged YAML nodes,
/// sequences (arrays), mappings (objects), and invalid (bad) values.
///
/// # Type Parameters
/// - `'input`: Lifetime of the input YAML data being processed.
///
/// # Required Implementations
/// All implementations of this trait must define the behavior for converting a YAML node into
/// a tagged node, creating nodes from bare YAML data, accessing/modifying sequences and mappings,
/// handling invalid values, and checking node types (sequence, mapping, or bad value).
///
/// # Associated Types
/// - This trait requires the associated `Cow<'input, Tag>` type for handling YAML tags.
/// - Input YAML must be represented as a `YamlDoc<'input>` type.
/// - YAML mappings are represented with `YamlEntry<'input, Self>` entries.
///
/// # Methods
///
/// ## Conversion
///
/// - `into_tagged(self, tag: Cow<'input, Tag>) -> Self`
///   Converts the current YAML node into a tagged node with the provided tag.
///
/// - `from_bare_yaml(yaml: YamlDoc<'input>) -> Self`
///   Creates a loadable YAML node from a bare YAML document.
///
/// ## Access and Mutation
///
/// - `sequence_mut(&mut self) -> &mut Vec<Self>`
///   Provides mutable access to the underlying sequence of nodes if the current node is a sequence.
///
/// - `mapping_mut(&mut self) -> &mut Vec<YamlEntry<'input, Self>>`
///   Provides mutable access to the underlying mapping entries if the current node is a mapping.
///
/// ## Special Values
///
/// - `bad(span: Span) -> Self`
///   Creates a node representing an invalid (bad) value. A default implementation is provided that
///   delegates to the required
pub trait LoadableYamlNode<'input>: Clone + PartialEq + YamlDocAccess<'input> {
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
    /// Constructs an instance of the implementing type from a bare YAML document.
    ///
    /// # Parameters
    /// - `yaml`: A `YamlDoc` containing the parsed YAML content from which the instance will be initialized.
    ///   This parameter is expected to hold the raw YAML representation that adheres to the structure
    ///   required by the implementing type.
    ///
    /// # Returns
    /// Method returns an instance of `Self` initialized with the data from the provided YAML document.
    ///
    /// # Errors
    /// This function may panic or return an error if the structure of the YAML document does not match
    /// the requirements of the implementing type or if there are any other parsing issues.
    ///
    /// # Example
    /// ```rust
    /// use yam_common::YamlDoc;
    /// use yam_common::LoadableYamlNode;
    /// let yaml_doc: YamlDoc = YamlDoc::from_bare_yaml(YamlDoc::Null);
    /// ```
    ///
    /// # Note
    /// Make sure the YAML document being passed conforms to the expected structure to avoid runtime errors.
    fn from_bare_yaml(yaml: YamlDoc<'input>) -> Self;

    ///
    /// Constructs an instance of `Self` using a bad or default value.
    ///
    /// # Attributes
    /// - `#[must_use]`: This attribute indicates that the return value of the
    ///   function must be used by the caller. Ignoring the return value may result
    ///   in a warning from the compiler.
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
    fn bad(_span: Span) -> Self {
        Self::bad_value()
    }

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
    fn bad_value() -> Self;

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
    /// use yam_common::YamlDoc;
    /// use yam_common::LoadableYamlNode;
    /// let mut value = YamlDoc::Bool(true);
    /// let previous_value = value.take();
    ///
    /// assert_eq!(previous_value, YamlDoc::Bool(true));
    /// assert_eq!(value, YamlDoc::BadValue);
    /// ```
    #[must_use]
    fn take(&mut self) -> Self;

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
}

/// Return type of the `YamlDocAccess` sequence methods.
pub type NodeSequence<Node> = Vec<Node>;

/// Return type fo the `YamlDocAccess` mapping methods.
pub type NodeMapping<'input, Node> = Vec<YamlEntry<'input, Node>>;

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
///   - `as_sequence(&self) -> Option<&NodeSequence<Self::Node>>`: Retrieves a reference to the value as a sequence if applicable.
///   - `as_sequence_mut(&mut self) -> Option<&mut NodeSequence<Self::Node>>`: Mutable access to the sequence value.
///   - `as_mapping(&self) -> Option<&NodeMapping<'input, Self::Node>>`: Retrieves a reference to the value as a mapping if applicable.
///   - `as_mapping_mut(&mut self) -> Option<&NodeMapping<'input, Self::Node>>`: Mutable access to the mapping value.
///   - `as_str(&self) -> Option<&str>`: Retrieves the value as a string slice if applicable.
///   - `as_str_mut(&mut self) -> Option<&mut str>`: Mutable access to the string value.
///   - `get_tag(&self) -> Option<Tag>`: Retrieves the YAML tag associated with the node if applicable.
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
pub trait YamlDocAccess<'input> {
    /// Type of node used in Sequence or Mapping
    type Node: Clone;

    /// Determines whether the current node is a bad value.
    ///
    /// # Returns
    /// * `true` - if the value meets the criteria for being "bad".
    /// * `false` - if the value does not meet the criteria for being "bad".
    ///
    /// # Example
    /// ```
    /// use yam_common::YamlDoc;
    /// let bad_value = YamlDoc::BadValue;
    ///
    /// assert!(bad_value.is_bad_value());
    ///```
    fn is_bad_value(&self) -> bool;

    /// Determines whether the current node is a null value.
    ///
    /// # Returns
    /// * `true` - if the value current node is null.
    /// * `false` - otherwise.
    ///
    /// # Example
    /// ```
    /// use yam_common::YamlDoc;
    /// let bad_value = YamlDoc::Null;
    ///
    /// assert!(bad_value.is_null());
    ///```
    fn is_null(&self) -> bool;

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
    /// use yam_common::YamlDoc;
    /// let bad_value = YamlDoc::String(Cow::Owned("yes.".into()));
    ///
    /// assert!(bad_value.is_string());
    ///```
    fn is_string(&self) -> bool;

    /// Determines whether the current node is a boolean value.
    ///
    /// # Returns
    /// * `true` - if the node contains a boolean value.
    /// * `false` - otherwise.
    ///
    /// # Example
    /// ```
    /// use std::borrow::Cow;
    /// use yam_common::YamlDoc;
    /// let bad_value = YamlDoc::Bool(false);
    ///
    /// assert!(bad_value.is_bool());
    ///```
    fn is_bool(&self) -> bool;
    /// Determines whether the current node is a floating point value.
    ///
    /// # Returns
    /// * `true` - if the node contains a floating point value.
    /// * `false` - otherwise.
    ///
    /// # Example
    /// ```
    /// use std::borrow::Cow;
    /// use yam_common::YamlDoc;
    /// let bad_value = YamlDoc::FloatingPoint(3.14);
    ///
    /// assert!(bad_value.is_floating_point());
    ///```
    fn is_floating_point(&self) -> bool;
    /// Determines whether the current node is an integer point value.
    ///
    /// # Returns
    /// * `true` - if the node contains an integer point value.
    /// * `false` - otherwise.
    ///
    /// # Example
    /// ```
    /// use std::borrow::Cow;
    /// use yam_common::YamlDoc;
    /// let bad_value = YamlDoc::Integer(12);
    ///
    /// assert!(bad_value.is_integer());
    ///```
    fn is_integer(&self) -> bool;

    /// Determines whether the current node is an alias
    ///
    /// # Returns
    /// * `true` - if the node is an alias.
    /// * `false` - otherwise.
    ///
    /// # Example
    /// ```
    /// use std::borrow::Cow;
    /// use yam_common::YamlDoc;
    /// let bad_value = YamlDoc::Alias(12);
    ///
    /// assert!(bad_value.is_alias());
    ///```
    fn is_alias(&self) -> bool;
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
        self.is_mapping() || self.is_sequence()
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
    /// use yam_common::YamlDoc;
    /// use yam_common::LoadableYamlNode;
    ///
    /// let example = YamlDoc::Bool(true);
    /// assert!(!example.is_mapping());
    /// ```
    ///
    /// This method can be used to verify whether an object follows a mapping
    /// structure or behavior based on its implementation.
    ///
    fn is_mapping(&self) -> bool;

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
    /// use yam_common::YamlDoc;
    /// use yam_common::LoadableYamlNode;
    ///
    /// let example = YamlDoc::Bool(true);
    /// assert!(!example.is_sequence());
    /// ```
    ///
    /// This method can be used to verify whether an object follows a sequential
    /// structure or behavior based on its implementation.
    ///
    fn is_sequence(&self) -> bool;

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
    /// * `Option<&NodeSequence<Self::Node>>` -
    ///   A reference to the node sequence wrapped in `Some` if it exists, or `None` otherwise.
    ///
    fn as_sequence(&self) -> Option<&NodeSequence<Self::Node>>;

    /// Returns a mutable reference to the sequence of nodes (`NodeSequence`).
    ///
    /// This method provides a way to access the underlying sequence of nodes if it exists,
    /// for the current instance. If the instance does not contain a sequence of nodes,
    /// `None` is returned.
    ///
    /// # Returns
    /// * `Option<&mut NodeSequence<Self::Node>>` -
    ///   A reference to the node sequence wrapped in `Some` if it exists, or `None` otherwise.
    ///
    fn as_sequence_mut(&mut self) -> Option<&mut NodeSequence<Self::Node>>;

    /// Returns an optional reference to the mapping of nodes (`NodeMapping`).
    ///
    /// This method provides a way to access the underlying mapping of nodes if it exists,
    /// for the current instance. If the instance does not contain a sequence of nodes,
    /// `None` is returned.
    ///
    /// # Returns
    /// * `Option<&mut NodeMapping<Self::Node>>` -
    ///   A reference to the node sequence wrapped in `Some` if it exists, or `None` otherwise.
    ///
    fn as_mapping(&self) -> Option<&NodeMapping<'input, Self::Node>>;

    /// Returns a mutable reference to the mapping of nodes (`NodeMapping`).
    ///
    /// This method provides a way to access the underlying mapping of nodes if it exists,
    /// for the current instance. If the instance does not contain a sequence of nodes,
    /// `None` is returned.
    ///
    /// # Returns
    /// * `Option<&mut NodeMapping<Self::Node>>` -
    ///   A reference to the node sequence wrapped in `Some` if it exists, or `None` otherwise.
    ///
    fn as_mapping_mut(&mut self) -> Option<&NodeMapping<'input, Self::Node>>;

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
    /// use yam_common::YamlDoc;
    /// use yam_common::LoadableYamlNode;
    ///
    /// let mut instance = YamlDoc::Sequence(vec![YamlDoc::Bool(true)]);
    /// let sequence = instance.sequence_mut();
    /// sequence.push(YamlDoc::Bool(false));
    /// ```
    fn sequence_mut(&mut self) -> &mut Vec<Self::Node>;

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
    /// use yam_common::YamlDoc;
    /// use yam_common::YamlEntry;
    /// use yam_common::LoadableYamlNode;
    ///
    /// let entry1 = YamlEntry::new("key".into(), "value".into());
    /// let entry2 = YamlEntry::new("another_key".into(), "value2".into());
    /// let mut instance = YamlDoc::Mapping(vec![entry1]);
    /// let sequence = instance.mapping_mut();
    /// sequence.push(entry2);
    /// ```
    fn mapping_mut(&mut self) -> &mut Vec<YamlEntry<'input, Self::Node>>;

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

    /// Converts the value of the type implementing this method into an `Option<bool>`.
    ///
    /// # Returns
    /// - `Some(true)` or `Some(false)` if the conversion is successful,
    ///   depending on the implementation.
    /// - `None` if the conversion is not possible or represents an invalid state.
    ///
    fn into_bool(self) -> Option<bool>;

    /// Converts the value of the type implementing this method into an `Option<String>`.
    ///
    /// # Returns
    /// - `Some(Strehg)` if the conversion is successful,
    ///   depending on the implementation.
    /// - `None` if the conversion is not possible or represents an invalid state.
    ///
    fn into_string(self) -> Option<String>;

    /// Converts the value of the type implementing this method into an `Option<Cow<'input, str>>`.
    ///
    /// # Returns
    /// - `Some(true)` or `Some(false)` if the node is a string
    /// - `None` if the conversion is not possible or represents an invalid state.
    ///
    fn into_cow(self) -> Option<Cow<'input, str>>;

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
    fn into_f64(self) -> Option<f64>;

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
    fn into_i64(self) -> Option<i64>;

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
    fn into_mapping(self) -> Option<NodeMapping<'input, Self::Node>>;

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
    fn into_sequence(self) -> Option<NodeSequence<Self::Node>>;
}

impl<'input> YamlDocAccess<'input> for YamlDoc<'input> {
    type Node = YamlDoc<'input>;

    fn is_bad_value(&self) -> bool {
        matches!(self, YamlDoc::BadValue)
    }

    fn is_null(&self) -> bool {
        matches!(self, YamlDoc::Null)
    }

    fn is_string(&self) -> bool {
        matches!(self, YamlDoc::String(_))
    }

    fn is_bool(&self) -> bool {
        matches!(self, YamlDoc::Bool(_))
    }

    fn is_floating_point(&self) -> bool {
        matches!(self, YamlDoc::FloatingPoint(_))
    }

    fn is_integer(&self) -> bool {
        matches!(self, YamlDoc::Integer(_))
    }

    fn is_alias(&self) -> bool {
        matches!(self, YamlDoc::Alias(_))
    }

    fn is_non_empty_collection(&self) -> bool {
        match self {
            YamlDoc::Sequence(s) => !s.is_empty(),
            YamlDoc::Mapping(m) => !m.is_empty(),
            _ => false,
        }
    }

    fn is_mapping(&self) -> bool {
        matches!(self, YamlDoc::Mapping(_))
    }

    fn is_sequence(&self) -> bool {
        matches!(self, YamlDoc::Sequence(_))
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            YamlDoc::Bool(x) => Some(*x),
            _ => None,
        }
    }

    fn as_bool_mut(&mut self) -> Option<&mut bool> {
        match self {
            YamlDoc::Bool(x) => Some(x),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            YamlDoc::Integer(x) => Some(*x),
            _ => None,
        }
    }

    fn as_i64_mut(&mut self) -> Option<&mut i64> {
        match self {
            YamlDoc::Integer(x) => Some(x),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            YamlDoc::FloatingPoint(x) => Some(*x),
            _ => None,
        }
    }

    fn as_f64_mut(&mut self) -> Option<&mut f64> {
        match self {
            YamlDoc::FloatingPoint(x) => Some(x),
            _ => None,
        }
    }

    fn as_sequence(&self) -> Option<&NodeSequence<Self::Node>> {
        match self {
            YamlDoc::Sequence(x) => Some(x),
            _ => None,
        }
    }

    fn as_sequence_mut(&mut self) -> Option<&mut NodeSequence<Self::Node>> {
        match self {
            YamlDoc::Sequence(x) => Some(x),
            _ => None,
        }
    }

    fn as_mapping(&self) -> Option<&NodeMapping<'input, Self::Node>> {
        match self {
            YamlDoc::Mapping(x) => Some(x),
            _ => None,
        }
    }

    fn as_mapping_mut(&mut self) -> Option<&NodeMapping<'input, Self::Node>> {
        match self {
            YamlDoc::Mapping(x) => Some(x),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            YamlDoc::String(x) => Some(x.as_ref()),
            _ => None,
        }
    }

    fn as_str_mut(&mut self) -> Option<&mut str> {
        match self {
            &mut YamlDoc::String(ref mut v) => Some(v.to_mut()),
            _ => None,
        }
    }

    fn sequence_mut(&mut self) -> &mut Vec<Self> {
        match self {
            YamlDoc::Sequence(seq) => seq,
            _ => core::panic!("Expected sequence got {:?}", self),
        }
    }

    fn mapping_mut(&mut self) -> &mut Vec<YamlEntry<'input, Self>> {
        match self {
            YamlDoc::Mapping(map) => map,
            _ => core::panic!("Expected mapping got {:?}", self),
        }
    }

    fn get_tag(&self) -> Option<Tag> {
        match self {
            YamlDoc::Tagged(tag, ..) => Some(Tag::new(&tag.handle, &tag.suffix)),
            _ => None,
        }
    }

    fn into_bool(self) -> Option<bool> {
        match self {
            YamlDoc::Bool(b) => Some(b),
            _ => None,
        }
    }

    fn into_string(self) -> Option<String> {
        match self {
            YamlDoc::String(s) => Some(s.to_string()),
            _ => None,
        }
    }

    fn into_cow(self) -> Option<Cow<'input, str>> {
        match self {
            YamlDoc::String(s) => Some(s),
            _ => None,
        }
    }

    fn into_f64(self) -> Option<f64> {
        match self {
            YamlDoc::FloatingPoint(f) => Some(f),
            _ => None,
        }
    }

    fn into_i64(self) -> Option<i64> {
        match self {
            YamlDoc::Integer(i) => Some(i),
            _ => None,
        }
    }

    fn into_mapping(self) -> Option<NodeMapping<'input, Self::Node>> {
        match self {
            YamlDoc::Mapping(mapping) => Some(mapping),
            _ => None,
        }
    }

    fn into_sequence(self) -> Option<NodeSequence<Self::Node>> {
        match self {
            YamlDoc::Sequence(seq) => Some(seq),
            _ => None,
        }
    }
}

impl<'input> LoadableYamlNode<'input> for YamlDoc<'input> {
    fn into_tagged(self, tag: Cow<'input, Tag>) -> Self {
        Self::Tagged(tag, Box::new(self))
    }

    fn from_bare_yaml(yaml: YamlDoc<'input>) -> Self {
        yaml
    }

    fn bad_value() -> Self {
        YamlDoc::BadValue
    }

    fn take(&mut self) -> Self {
        mem::take(self)
    }
}

/// Ordered sequence of one or more [`YamlDoc`]'s
pub type Sequence<'a> = Vec<YamlDoc<'a>>;

/// Sequence of key-value pairing of two [`YamlDoc`]s
pub type Mapping<'a> = Vec<YamlEntry<'a, YamlDoc<'a>>>;

/// Represents a YAML document structure in Rust, capturing various types of YAML values.
///
///
/// # Notes
///
/// * The `'input` lifetime parameter allows borrowed data to remain valid for the lifetime of the `YamlDoc` instance.
/// * The type derives commonly used traits such as `Debug`, `Default`, `Clone`, and `PartialEq` to facilitate
///   debugging, default value initialization, cloning, and equality comparisons.
///
/// # Example
///
/// ```rust
/// use std::borrow::Cow;
/// use yam_common::LoadableYamlNode;
/// use yam_common::YamlDoc;
///
/// let yaml_string = YamlDoc::String(Cow::Borrowed("example"));
/// let yaml_bool = YamlDoc::Bool(true);
/// let yaml_null = YamlDoc::Null;
///
/// println!("{:?}", yaml_string); // Outputs: String("example")
/// println!("{:?}", yaml_bool);   // Outputs: Bool(true)
/// println!("{:?}", yaml_null);   // Outputs: Null
/// ```
///
#[derive(Debug, Default, Clone, PartialEq)]
pub enum YamlDoc<'input> {
    #[default]
    /// Invalid value for `YamlDoc`
    BadValue,
    /// Represents a `null` value for `YamlDoc`
    Null,
    /// Represents a YAML string value.
    String(Cow<'input, str>),
    /// Represents a value that's either `true` or `false`
    Bool(bool),
    /// Floating point representation.
    FloatingPoint(f64),
    /// Integer number representation.
    Integer(i64),
    /// Represents a series of values either as flow style like:
    /// ```yaml
    /// [x, x, x]
    /// ```
    /// or block style like
    /// ```yaml
    /// - x
    /// - x
    /// - x
    /// ```
    Sequence(Sequence<'input>),

    /// Represents a series of key to map values either in flow style like:
    /// ```yaml
    /// {x: Y, a: B}
    /// ```
    /// or block style like
    /// ```yaml
    /// x: Y
    /// a: B
    /// ```
    Mapping(Mapping<'input>),
    /// Represents a pointer to another node like `[*lol, *lol]`
    Alias(usize),
    /// Tagged `YamlDoc` value, contains a [`Tag`] and a node that's a [`Box<YamlDoc<'input>>`]
    Tagged(Cow<'input, Tag>, Box<YamlDoc<'input>>),
}

impl<'input> YamlDoc<'input> {
    ///
    /// Constructs a `YamlDoc` instance from a `Cow<str>` value, a `ScalarType`, and an optional `Tag`.
    ///
    /// # Parameters
    ///
    /// - `value`: The `Cow<str>` containing the value to be parsed into a [`YamlDoc`].
    /// - `scalar_type`: A `ScalarType` specifying the type of scalar (e.g., `Plain`, `Quoted`).
    /// - `tag`: An optional `Tag`, wrapped in a `Cow<str>`, that provides additional context
    ///   about the scalar value, such as its type in YAML core schema.
    ///
    /// # Returns
    ///
    /// - If the `scalar_type` is not [`ScalarType::Plain`], this function directly returns a
    ///  [`YamlDoc::String`] with the provided `value`.
    ///
    /// - If a `tag` is provided and it is valid according to the YAML core schema, the method
    ///   attempts to interpret the value based on the `tag.suffix`:
    ///   - `"bool"`: Parses the value as a boolean using `parse_bool`.
    ///   - `"int"`: Parses the value as an integer. If successful, returns `YamlDoc::Integer`.
    ///     Otherwise, returns `YamlDoc::BadValue`.
    ///   - `"null"`: Parses the value as a null using `parse_null`.
    ///   - `"float"`: Parses the value as a floating-point number using `parse_float`.
    ///     If successful, returns `YamlDoc::FloatingPoint`. Otherwise, returns `YamlDoc::BadValue`.
    ///   - Any other tag suffix results in `YamlDoc::BadValue`.
    ///
    /// - If no valid `tag` is provided, the method invokes `Self::parse_from_cow` to parse
    ///   the value.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::borrow::Cow;
    /// use yam_common::{LoadableYamlNode, YamlDoc, ScalarType, Tag};
    ///
    /// let value = Cow::Borrowed("true");
    /// let scalar_type = ScalarType::Plain;
    /// let tag = Some(Cow::Owned(Tag::new("tag:yaml.org,2002:", "bool")));
    ///
    /// let doc = YamlDoc::from_cow_and_tag(value, scalar_type, &tag);
    ///
    /// ```
    ///
    /// # Notes
    ///
    /// - This method distinguishes between valid YAML core schema tags and invalid ones.
    /// - If parsing fails for any of the known tag types (`bool`, `int`, `null`, `float`),
    ///   the method returns `YamlDoc::BadValue`.
    ///
    /// # See Also
    ///
    /// - `Self::String`
    /// - `Self::parse_from_cow`
    /// - `parse_bool`
    /// - `parse_null`
    /// - `parse_float`
    ///
    pub fn from_cow_and_tag(
        value: Cow<'input, str>,
        scalar_type: ScalarType,
        tag: &Option<Cow<'input, Tag>>,
    ) -> YamlDoc<'input> {
        if scalar_type != ScalarType::Plain {
            return Self::String(value);
        }
        if let Some(tag) = tag
            && tag.is_yaml_core_schema()
        {
            return match &*tag.suffix {
                "bool" => parse_bool(value),
                "int" => value
                    .parse()
                    .ok()
                    .map_or(YamlDoc::BadValue, YamlDoc::Integer),
                "null" => parse_null(value),
                "float" => parse_float(&value).map_or(YamlDoc::BadValue, YamlDoc::FloatingPoint),
                _ => YamlDoc::BadValue,
            };
        }
        Self::parse_from_cow(value)
    }

    #[must_use]
    fn parse_from_cow(value: Cow<str>) -> YamlDoc {
        let bytes = value.as_bytes();
        let str_v = &*value;
        let early_check = match bytes {
            b"null" | b"~" => Some(YamlDoc::Null),
            b"true" | b"True" | b"TRUE" => Some(YamlDoc::Bool(true)),
            b"false" | b"False" | b"FALSE" => Some(YamlDoc::Bool(false)),
            _ => None,
        };
        if let Some(x) = early_check {
            return x;
        }

        match bytes {
            [b'0', b'x', ..] => {
                if let Ok(x) = i64::from_str_radix(&str_v[2..], 16) {
                    return YamlDoc::Integer(x);
                }
            }
            [b'0', b'o', ..] => {
                if let Ok(x) = i64::from_str_radix(&str_v[2..], 8) {
                    return YamlDoc::Integer(x);
                }
            }
            _ => {}
        }

        if let Ok(integer) = value.parse::<i64>() {
            return YamlDoc::Integer(integer);
        }

        if let Some(float) = parse_float(&value) {
            return YamlDoc::FloatingPoint(float);
        }

        YamlDoc::String(value)
    }
}

impl From<&str> for YamlDoc<'_> {
    fn from(value: &str) -> Self {
        YamlDoc::String(Cow::Owned(value.into()))
    }
}

impl From<i64> for YamlDoc<'_> {
    fn from(value: i64) -> Self {
        YamlDoc::Integer(value)
    }
}

impl From<i32> for YamlDoc<'_> {
    fn from(value: i32) -> Self {
        YamlDoc::Integer(value.into())
    }
}

impl From<i16> for YamlDoc<'_> {
    fn from(value: i16) -> Self {
        YamlDoc::Integer(value.into())
    }
}

impl From<i8> for YamlDoc<'_> {
    fn from(value: i8) -> Self {
        YamlDoc::Integer(value.into())
    }
}

impl From<f64> for YamlDoc<'_> {
    fn from(value: f64) -> Self {
        YamlDoc::FloatingPoint(value)
    }
}

impl From<f32> for YamlDoc<'_> {
    fn from(value: f32) -> Self {
        YamlDoc::FloatingPoint(value.into())
    }
}

impl From<bool> for YamlDoc<'_> {
    fn from(value: bool) -> Self {
        YamlDoc::Bool(value)
    }
}

#[allow(clippy::needless_pass_by_value)]
fn parse_bool(v: Cow<str>) -> YamlDoc<'static> {
    match v.as_bytes() {
        b"true" | b"True" | b"TRUE" => YamlDoc::Bool(true),
        b"false" | b"False" | b"FALSE" => YamlDoc::Bool(false),
        _ => YamlDoc::BadValue,
    }
}

#[allow(clippy::needless_pass_by_value)]
fn parse_null(v: Cow<str>) -> YamlDoc<'static> {
    match v.as_bytes() {
        b"~" | b"null" => YamlDoc::Null,
        _ => YamlDoc::BadValue,
    }
}

fn parse_float(v: &str) -> Option<f64> {
    match v.as_bytes() {
        b".inf" | b".Inf" | b".INF" | b"+.inf" | b"+.Inf" | b"+.INF" => Some(f64::INFINITY),
        b"-.inf" | b"-.Inf" | b"-.INF" => Some(f64::NEG_INFINITY),
        b".nan" | b".NaN" | b".NAN" => Some(f64::NAN),
        // Test that `v` contains a digit so as not to pass in strings like `inf`,
        // which rust will parse as a float.
        _ => v.parse::<f64>().ok(),
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
pub struct YamlEntry<'input, T>
where
    T: Clone,
{
    /// Represents the key of the YAML entry. It is of type `T`.
    pub key: T,
    /// Represents the value of the YAML entry. It is of type `T`.
    pub value: T,
    pub(crate) _marker: PhantomData<&'input ()>,
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

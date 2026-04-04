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
pub trait LoadableYamlNode<'input>: Clone + PartialEq {
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

    /// Provides mutable access to the sequence within the implementing type.
    ///
    /// This method allows for getting a mutable reference to a `Vec` associated with
    /// the implementing type. This enables modification of the underlying vector, such
    /// as adding, removing, or altering elements.
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
    fn sequence_mut(&mut self) -> &mut Vec<Self>;

    /// Provides mutable access to the mapping within the implementing type.
    ///
    /// This method allows for getting a mutable reference to a `Vec` of `YamlEntry` associated with
    /// the implementing type. This enables modification of the underlying vector, such
    /// as adding, removing, or altering elements.
    ///
    /// # Returns
    /// A mutable reference to a `Vec` of the type implementing this method.
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
    fn mapping_mut(&mut self) -> &mut Vec<YamlEntry<'input, Self>>;

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
    /// Determines if the implementing object represents a bad value, which often signifies an
    /// error during parsing.
    ///
    /// # Returns
    ///
    /// * `true` if the object is considered a bad value.
    /// * `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use yam_common::YamlDoc;
    /// use yam_common::LoadableYamlNode;
    ///
    /// let example = YamlDoc::BadValue;
    /// assert!(example.is_bad_value());
    /// ```
    ///
    fn is_bad_value(&self) -> bool;

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

impl<'input> LoadableYamlNode<'input> for YamlDoc<'input> {
    fn into_tagged(self, tag: Cow<'input, Tag>) -> Self {
        Self::Tagged(tag, Box::new(self))
    }

    fn from_bare_yaml(yaml: YamlDoc<'input>) -> Self {
        yaml
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
            _ => core::panic!("Expected sequence got {:?}", self),
        }
    }

    fn bad_value() -> Self {
        YamlDoc::BadValue
    }

    fn is_sequence(&self) -> bool {
        matches!(self, YamlDoc::Sequence(_))
    }

    fn is_mapping(&self) -> bool {
        matches!(self, YamlDoc::Mapping(_))
    }

    fn is_bad_value(&self) -> bool {
        matches!(self, YamlDoc::BadValue)
    }

    fn take(&mut self) -> Self {
        mem::take(self)
    }

    fn is_non_empty_collection(&self) -> bool {
        match self {
            YamlDoc::Sequence(x) => !x.is_empty(),
            YamlDoc::Mapping(x) => !x.is_empty(),
            _ => false,
        }
    }
}

/// Ordered sequence of one or more [`YamlDoc`]'s
pub type Sequence<'a> = Vec<YamlDoc<'a>>;

/// Sequence of key-value pairing of two [`YamlDoc`]s
pub type Mapping<'a> = Vec<YamlEntry<'a, YamlDoc<'a>>>;

/// Represents a YAML document structure in Rust, capturing various types of YAML values.
///
/// # Enum Variants
///
/// - `BadValue`: A default variant representing an invalid or uninitialized YAML value.
/// - `Null`: Represents a `null` value in YAML.
/// - `String(Cow<'input, str>)`: Represents a YAML string value. Uses `Cow` to support both borrowed
///   and owned string data for efficient memory usage.
/// - `Bool(bool)`: Represents a YAML boolean value (`true` or `false`).
/// - `FloatingPoint(f64)`: Represents a YAML floating-point number.
/// - `Integer(i64)`: Represents a YAML integer.
/// - `Sequence(Sequence<'input>)`: Represents a YAML sequence (array-like structure), which can
///   use either flow style (e.g., `[x, y, z]`) or block style:
/// ```yaml
/// - x
/// - y
/// - z
/// ```
/// - `Mapping(Mapping<'input>)`: Represents a YAML mapping (key-value pairs), which can use either flow style
///   (e.g., `{x: Y, a: B}`) or block style:
/// ```yaml
/// x: Y
/// a: B
/// ```
/// - `Alias(usize)`: Represents an alias (reference) to another YAML value.
/// - `Tagged(Cow<'input, Tag>, Box<YamlDoc<'input>>)`: Represents a tagged YAML value, which includes a custom
///   tag (`Tag`) and an associated value wrapped as a `YamlDoc`.
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
    /// Tagged YamlDoc value, contains a [`Tag`] and a node that's a [`Box<YamlDoc<'input>>`]
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
    ///   [`YamlDoc::String`] with the provided `value`.
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

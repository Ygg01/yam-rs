use crate::prelude::{
    LoadableYamlNode, NodeType, ScalarType, Tag, YamlAccessError, YamlDocAccess, YamlEntry,
};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::mem;
use core::ops::{Index, IndexMut};

impl<'input> YamlDocAccess<'input> for YamlDoc<'input> {
    type Node = YamlDoc<'input>;
    type SequenceNode = Vec<Self::Node>;
    type MappingNode = Vec<YamlEntry<'input, YamlDoc<'input>>>;

    fn key_from_usize(index: usize) -> Self {
        YamlDoc::Integer(index as i64)
    }

    fn key_from_str(index: &str) -> Self {
        YamlDoc::String(Cow::Owned(index.to_string()))
    }

    fn is_non_empty_collection(&self) -> bool {
        match self {
            YamlDoc::Sequence(s) => !s.is_empty(),
            YamlDoc::Mapping(m) => !m.is_empty(),
            _ => false,
        }
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

    fn as_sequence(&self) -> Result<&Self::SequenceNode, YamlAccessError> {
        match self {
            YamlDoc::Sequence(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedSequence),
        }
    }

    fn as_sequence_mut(&mut self) -> Result<&mut Self::SequenceNode, YamlAccessError> {
        match self {
            YamlDoc::Sequence(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedSequence),
        }
    }

    fn as_mapping(&self) -> Result<&Self::MappingNode, YamlAccessError> {
        match self {
            YamlDoc::Mapping(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedMapping),
        }
    }

    fn as_mapping_mut(&mut self) -> Result<&mut Self::MappingNode, YamlAccessError> {
        match self {
            YamlDoc::Mapping(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedMapping),
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

    fn get_type(&self) -> NodeType {
        match self {
            YamlDoc::BadValue => NodeType::Bad,
            YamlDoc::Null => NodeType::Null,
            YamlDoc::String(_) => NodeType::String,
            YamlDoc::Bool(_) => NodeType::Bool,
            YamlDoc::FloatingPoint(_) => NodeType::Floating,
            YamlDoc::Integer(_) => NodeType::Integer,
            YamlDoc::Sequence(_) => NodeType::Sequence,
            YamlDoc::Mapping(_) => NodeType::Mapping,
            YamlDoc::Alias(_) => NodeType::Alias,
            YamlDoc::Tagged(_, a) => a.get_type(),
        }
    }

    fn into_string(self) -> Option<String> {
        match self {
            YamlDoc::String(s) => Some(s.to_string()),
            _ => None,
        }
    }

    fn into_mapping(self) -> Option<Self::MappingNode> {
        match self {
            YamlDoc::Mapping(mapping) => Some(mapping),
            _ => None,
        }
    }

    fn into_sequence(self) -> Option<Self::SequenceNode> {
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
/// use yam_core::{YamlLoader, YamlDoc};
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
    /// use yam_core::prelude::{ScalarType, Tag, LoadableYamlNode};
    /// use yam_core::{YamlDoc};
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

#[allow(clippy::cast_possible_wrap)]
impl<'input> Index<usize> for YamlDoc<'input> {
    type Output = YamlDoc<'input>;

    /// Perform index by integer.
    ///
    /// When `self` is a sequence, the method will attempt to access the underlying vector at a given position.
    /// When `self` is a mapping, the method will attempt to access the underlying map assuming `index` is a key
    /// to its value. For example, YAML `{ 0: "test" }` can be accessed using `0`.
    ///
    /// # Panics
    /// This function panics if the index doesn't exist in sequence or if the mapping doesn't contain
    /// an index key with the same value.
    fn index(&self, index: usize) -> &YamlDoc<'input> {
        let get_type = self.get_type();
        match self {
            YamlDoc::Sequence(sequence) => sequence.index(index),
            YamlDoc::Mapping(mapping) => {
                let find_key = mapping
                    .iter()
                    .find(|x| x.key.as_i64() == Some(index as i64));
                &find_key
                    .unwrap_or_else(|| panic!("Key {index} not found in `YamlCloneNode` mapping"))
                    .value
            }
            _ => panic!("Attempt to index {get_type:?} in `YamlCloneNode`"),
        }
    }
}
#[allow(clippy::cast_possible_wrap)]
impl<'input> IndexMut<usize> for YamlDoc<'input> {
    /// Perform index by integer.
    ///
    /// When `self` is a sequence, the method will attempt to access underlying vector at given position.
    /// When `self` is a mapping, the method will attempt to access underlying map assuming `index` is a key
    /// to its value. For example, YAML `{ 0: "test" }` can be accessed using `0`.
    ///
    /// # Panics
    /// This function panics if the index doesn't exist in sequence or if the mapping doesn't contain
    /// an index key with the same value.
    fn index_mut(&mut self, index: usize) -> &mut YamlDoc<'input> {
        let get_type = self.get_type();
        match self {
            YamlDoc::Sequence(sequence) => sequence.index_mut(index),
            YamlDoc::Mapping(mapping) => {
                let find_key = mapping
                    .iter_mut()
                    .find(|x| x.key.as_i64() == Some(index as i64));
                &mut find_key
                    .unwrap_or_else(|| panic!("Key {index} not found in `YamlCloneNode` mapping"))
                    .value
            }
            _ => panic!("Attempt to index {get_type:?} with {index} in YamlCloneNode"),
        }
    }
}

impl<'input, 'key> Index<&'key str> for YamlDoc<'input> {
    type Output = YamlDoc<'input>;

    /// Perform index by string.
    ///
    /// When `self` is a mapping, the method will attempt to access the underlying map assuming `index` is a key
    /// to its value. For example, YAML `{ key: "test" }` can be accessed using the `key` string.
    ///
    /// # Panics
    /// This function panics if the index doesn't exist in the map.
    fn index(&self, index: &'key str) -> &YamlDoc<'input> {
        let get_type = self.get_type();
        match self {
            YamlDoc::Mapping(mapping) => {
                let find_key = mapping.iter().find(|x| x.key.as_str() == Some(index));
                &find_key
                    .unwrap_or_else(|| panic!("Key {index} not found in `YamlCloneNode` mapping"))
                    .value
            }
            _ => panic!("Attempt to index {get_type:?} with {index} in `YamlCloneNode`"),
        }
    }
}

impl<'input, 'key> IndexMut<&'key str> for YamlDoc<'input> {
    /// Perform a mutable index by string.
    ///
    /// When `self` is a mapping, the method will attempt to access the underlying map assuming `index` is a key
    /// to its value. For example, YAML `{ key: "test" }` can be accessed using the `key` string.
    ///
    /// # Panics
    /// This function panics if the index doesn't exist in the map.
    fn index_mut(&mut self, index: &'key str) -> &mut YamlDoc<'input> {
        let get_type = self.get_type();
        match self {
            YamlDoc::Mapping(mapping) => {
                let find_key = mapping.iter_mut().find(|x| x.key.as_str() == Some(index));
                &mut find_key
                    .unwrap_or_else(|| panic!("Key {index} not found in `YamlCloneNode` mapping"))
                    .value
            }
            _ => panic!("Attempt to index {get_type:?} in `YamlCloneNode`"),
        }
    }
}

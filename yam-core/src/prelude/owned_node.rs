use crate::prelude::yaml_doc::{Mapping, Sequence};
use crate::prelude::{NodeType, Span, Tag, YamlAccessError, YamlDoc, YamlDocAccess, YamlEntry};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::ops::{Index, IndexMut};

///
/// Represents a YAML node that always owns its data.
///
/// The `YamlOwnedNode` enum provides a structured way to represent YAML values
/// in Rust. Each variant corresponds to a possible type of value that a YAML
/// node can hold, such as `null`, `string`, `integer`, `sequence`, or `mapping`.
///
/// # Type Parameters
///
/// * `Node`: Generic parameter for representing nested or child nodes. It must
///   implement the `Clone` trait.
///
/// # Traits Implementations
///
/// * `Default`: The default value for `YamlOwnedNode` is the `Null` variant.
/// * `PartialEq`: Supports equality comparisons between two `YamlOwnedNode`
///   instances.
/// * `Clone`: Allows cloning of `YamlOwnedNode` instances.
/// * `Debug`: Enables formatted output for debugging purposes.
///
#[derive(Debug, Default, PartialEq, Clone)]
pub enum YamlOwnedNode<Node: Clone> {
    #[default]
    /// Invalid value for `YamlDoc`
    BadValue,
    /// Represents a `null` value for `YamlDoc`
    Null,
    /// Represents a YAML string value.
    String(String),
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
    Sequence(Vec<Node>),

    /// Represents a series of key to map values either in flow style like:
    /// ```yaml
    /// {x: Y, a: B}
    /// ```
    /// or block style like
    /// ```yaml
    /// x: Y
    /// a: B
    /// ```
    Mapping(Vec<YamlEntry<'static, Node>>),
    /// Represents a pointer to another node like `[*lol, *lol]`
    Alias(usize),
    /// Tagged `YamlDoc` value, contains a [`Tag`] and a node that's a [`Box<Node>`]
    Tagged(Tag, Box<Node>),
}

impl<Node> YamlDocAccess<'static> for YamlOwnedNode<Node>
where
    Node: Clone + YamlDocAccess<'static>,
{
    type Node = Node;
    type SequenceNode = Vec<Node>;
    type MappingNode = Vec<YamlEntry<'static, Node>>;

    fn key_from_usize(index: usize) -> Self {
        YamlOwnedNode::Integer(index as i64)
    }

    fn key_from_str(index: &str) -> Self {
        YamlOwnedNode::String(index.to_string())
    }

    fn is_non_empty_collection(&self) -> bool {
        match self {
            YamlOwnedNode::Sequence(s) => !s.is_empty(),
            YamlOwnedNode::Mapping(m) => !m.is_empty(),
            _ => false,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            YamlOwnedNode::Bool(x) => Some(*x),
            _ => None,
        }
    }

    fn as_bool_mut(&mut self) -> Option<&mut bool> {
        match self {
            YamlOwnedNode::Bool(x) => Some(x),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            YamlOwnedNode::Integer(x) => Some(*x),
            _ => None,
        }
    }

    fn as_i64_mut(&mut self) -> Option<&mut i64> {
        match self {
            YamlOwnedNode::Integer(x) => Some(x),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            YamlOwnedNode::FloatingPoint(x) => Some(*x),
            _ => None,
        }
    }

    fn as_f64_mut(&mut self) -> Option<&mut f64> {
        match self {
            YamlOwnedNode::FloatingPoint(x) => Some(x),
            _ => None,
        }
    }

    fn as_sequence(&self) -> Result<&Self::SequenceNode, YamlAccessError> {
        match self {
            YamlOwnedNode::Sequence(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedSequence),
        }
    }

    fn as_sequence_mut(&mut self) -> Result<&mut Self::SequenceNode, YamlAccessError> {
        match self {
            YamlOwnedNode::Sequence(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedSequence),
        }
    }

    fn as_mapping(&self) -> Result<&Self::MappingNode, YamlAccessError> {
        match self {
            YamlOwnedNode::Mapping(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedMapping),
        }
    }

    fn as_mapping_mut(&mut self) -> Result<&mut Self::MappingNode, YamlAccessError> {
        match self {
            YamlOwnedNode::Mapping(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedMapping),
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            YamlOwnedNode::String(x) => Some(x.as_ref()),
            _ => None,
        }
    }

    fn as_str_mut(&mut self) -> Option<&mut str> {
        match self {
            &mut YamlOwnedNode::String(ref mut v) => Some(v),
            _ => None,
        }
    }

    fn sequence_mut(&mut self) -> &mut Self::SequenceNode {
        match self {
            YamlOwnedNode::Sequence(seq) => seq,
            _ => core::panic!("Expected sequence got {:?}", self.get_type()),
        }
    }

    fn mapping_mut(&mut self) -> &mut Vec<YamlEntry<'static, Node>> {
        match self {
            YamlOwnedNode::Mapping(map) => map,
            _ => core::panic!("Expected mapping got {:?}", self.get_type()),
        }
    }

    fn get_tag(&self) -> Option<Tag> {
        match self {
            YamlOwnedNode::Tagged(tag, ..) => Some(Tag::new(&tag.handle, &tag.suffix)),
            _ => None,
        }
    }

    fn get_type(&self) -> NodeType {
        match self {
            YamlOwnedNode::BadValue => NodeType::Bad,
            YamlOwnedNode::Null => NodeType::Null,
            YamlOwnedNode::String(_) => NodeType::String,
            YamlOwnedNode::Bool(_) => NodeType::Bool,
            YamlOwnedNode::FloatingPoint(_) => NodeType::Floating,
            YamlOwnedNode::Integer(_) => NodeType::Integer,
            YamlOwnedNode::Sequence(_) => NodeType::Sequence,
            YamlOwnedNode::Mapping(_) => NodeType::Mapping,
            YamlOwnedNode::Alias(_) => NodeType::Alias,
            YamlOwnedNode::Tagged(_, a) => a.get_type(),
        }
    }

    fn into_string(self) -> Option<String> {
        match self {
            YamlOwnedNode::String(s) => Some(s.to_string()),
            _ => None,
        }
    }

    fn into_mapping(self) -> Option<Self::MappingNode> {
        match self {
            YamlOwnedNode::Mapping(mapping) => Some(mapping),
            _ => None,
        }
    }

    fn into_sequence(self) -> Option<Self::SequenceNode> {
        match self {
            YamlOwnedNode::Sequence(seq) => Some(seq),
            _ => None,
        }
    }

    fn into_tagged(self, tag: Cow<'static, Tag>) -> Self {
        todo!()
    }

    fn from_bare_yaml(yaml: YamlDoc<'static>) -> Self {
        todo!()
    }

    fn bad_span_value(_span: Span) -> Self {
        todo!()
    }
}

impl<'input, T: Clone> From<YamlDoc<'input>> for YamlOwnedNode<T>
where
    T: From<YamlDoc<'input>>,
{
    fn from(value: YamlDoc<'input>) -> Self {
        match value {
            YamlDoc::BadValue => YamlOwnedNode::BadValue,
            YamlDoc::Null => YamlOwnedNode::Null,
            YamlDoc::String(x) => YamlOwnedNode::String(x.to_string()),
            YamlDoc::Bool(x) => YamlOwnedNode::Bool(x),
            YamlDoc::Alias(x) => YamlOwnedNode::Alias(x),
            YamlDoc::FloatingPoint(x) => YamlOwnedNode::FloatingPoint(x),
            YamlDoc::Integer(x) => YamlOwnedNode::Integer(x),
            YamlDoc::Sequence(s) => YamlOwnedNode::from_sequence(s),
            YamlDoc::Mapping(m) => YamlOwnedNode::from_mapping(m),
            YamlDoc::Tagged(tag, data) => {
                YamlOwnedNode::Tagged(tag.into_owned(), Box::new((*data).into()))
            }
        }
    }
}

impl<'input, T> YamlOwnedNode<T>
where
    T: From<YamlDoc<'input>> + Clone,
{
    fn from_sequence(sequence: Sequence<'input>) -> YamlOwnedNode<T> {
        YamlOwnedNode::Sequence(sequence.into_iter().map(Into::into).collect())
    }

    fn from_mapping(mapping: Mapping<'input>) -> YamlOwnedNode<T> {
        YamlOwnedNode::Mapping(
            mapping
                .into_iter()
                .map(|x| YamlEntry::new(x.key.into(), x.value.into()))
                .collect(),
        )
    }
}

#[allow(clippy::cast_possible_wrap)]
impl<Node> Index<usize> for YamlOwnedNode<Node>
where
    Node: YamlDocAccess<'static> + PartialEq,
{
    type Output = Node;

    /// Perform index by integer.
    ///
    /// When `self` is a sequence, the method will attempt to access the underlying vector at a given position.
    /// When `self` is a mapping, the method will attempt to access the underlying map assuming `index` is a key
    /// to its value. For example, YAML `{ 0: "test" }` can be accessed using `0`.
    ///
    /// # Panics
    /// This function panics if the index doesn't exist in sequence or if the mapping doesn't contain
    /// an index key with the same value.
    fn index(&self, index: usize) -> &Node {
        let get_type = self.get_type();
        match self {
            YamlOwnedNode::Sequence(sequence) => sequence.index(index),
            YamlOwnedNode::Mapping(mapping) => {
                let find_key = mapping
                    .iter()
                    .find(|entry| entry.key.as_i64() == Some(index as i64));
                &find_key
                    .unwrap_or_else(|| panic!("Key {index} not found in `YamlOwnedNode` mapping"))
                    .value
            }
            _ => panic!("Attempt to index {get_type:?} in `YamlOwnedNode`"),
        }
    }
}
#[allow(clippy::cast_possible_wrap)]
impl<Node> IndexMut<usize> for YamlOwnedNode<Node>
where
    Node: Clone + YamlDocAccess<'static> + PartialEq,
{
    /// Perform index by integer.
    ///
    /// When `self` is a sequence, the method will attempt to access the underlying vector at a given position.
    /// When `self` is a mapping, the method will attempt to access the underlying map assuming `index` is a key
    /// to its value. For example, YAML `{ 0: "test" }` can be accessed using `0`.
    ///
    /// # Panics
    /// This function panics if the index doesn't exist in sequence or if the mapping doesn't contain
    /// an index key with the same value.
    ///
    fn index_mut(&mut self, index: usize) -> &mut Node {
        let get_type = self.get_type();
        match self {
            YamlOwnedNode::Sequence(sequence) => sequence.index_mut(index),
            YamlOwnedNode::Mapping(mapping) => {
                let find_key = mapping
                    .iter_mut()
                    .find(|x| x.key.as_i64() == Some(index as i64));
                &mut find_key
                    .unwrap_or_else(|| panic!("Key {index} not found in `YamlOwnedNode` mapping"))
                    .value
            }
            _ => panic!("Attempt to index {get_type:?} with {index} in `YamlOwnedNode`"),
        }
    }
}

impl<'key, Node> Index<&'key str> for YamlOwnedNode<Node>
where
    Node: Clone + YamlDocAccess<'static> + PartialEq,
{
    type Output = Node;

    /// Perform index by string.
    ///
    /// When `self` is a mapping, the method will attempt to access the underlying map assuming `index` is a key
    /// to its value. For example, YAML `{ key: "test" }` can be accessed using the `key` string.
    ///
    /// # Panics
    /// This function panics if the index doesn't exist in the map.
    fn index(&self, index: &'key str) -> &Node {
        let get_type = self.get_type();
        match self {
            YamlOwnedNode::Mapping(mapping) => {
                let find_key = mapping.iter().find(|x| x.key.as_str() == Some(index));
                &find_key
                    .unwrap_or_else(|| panic!("Key {index} not found in `YamlOwnedNode` mapping"))
                    .value
            }
            _ => panic!("Attempt to index {get_type:?} with {index} in `YamlOwnedNode`"),
        }
    }
}

impl<'key, Node> IndexMut<&'key str> for YamlOwnedNode<Node>
where
    Node: Clone + YamlDocAccess<'static> + PartialEq,
{
    /// Perform a mutable index by string.
    ///
    /// When `self` is a mapping, the method will attempt to access the underlying map assuming `index` is a key
    /// to its value. For example, YAML `{ key: "test" }` can be accessed using the `key` string.
    ///
    /// # Panics
    /// This function panics if the index doesn't exist in the map.
    fn index_mut(&mut self, index: &'key str) -> &mut Node {
        let get_type = self.get_type();
        match self {
            YamlOwnedNode::Mapping(mapping) => {
                let find_key = mapping.iter_mut().find(|x| x.key.as_str() == Some(index));
                &mut find_key
                    .unwrap_or_else(|| panic!("Key {index} not found in `YamlOwnedNode` mapping"))
                    .value
            }
            _ => panic!("Attempt to index {get_type:?} in `YamlOwnedNode`"),
        }
    }
}

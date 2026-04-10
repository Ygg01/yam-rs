use alloc::borrow::Cow;
use core::borrow::Borrow;
use ordered_float::OrderedFloat;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hash, Hasher};
use std::ops::{Index, IndexMut};
use yam_core::prelude::{NodeType, Span, Tag, YamlAccessError, YamlDoc, YamlDocAccess, YamlEntry};

/// Wrapper for hashmap that allows for hashing of hashmap.
#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct HashedMapWrap<Node: PartialEq + Eq + Hash>(HashMap<Node, Node>);

impl<Node> Hash for HashedMapWrap<Node>
where
    Node: Hash + Eq,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut h = 0u64;

        for elt in &self.0 {
            let hasher = self.0.hasher();
            h ^= hasher.hash_one(elt);
        }

        state.write_u64(h);
    }
}

///
/// Represents a YAML node that owns its data, but
///
/// The `YamlHashNode` enum provides a structured way to represent YAML values
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
/// * `Default`: The default value for `YamlHashNode` is the `Null` variant.
/// * `PartialEq`: Supports equality comparisons between two `YamlHashNode`
///   instances.
/// * `Eq`: Supports full equality comparisons between two `YamlHashNode`
///   instances.
/// * `Hash`: Supports hashing of nodes. This is possibly a problem for `Map`, and `FloatingPoint`
/// * `Clone`: Allows cloning of `YamlHashNode` instances.
/// * `Debug`: Enables formatted output for debugging purposes.
///
#[derive(Debug, Default, PartialEq, Eq, Clone, Hash)]
pub enum YamlHashNode<'input, Node>
where
    Node: Clone + PartialEq + Eq + Hash + Eq,
{
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
    FloatingPoint(OrderedFloat<f64>),
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
    Mapping(HashedMapWrap<Node>),
    /// Represents a pointer to another node like `[*lol, *lol]`
    Alias(usize),
    /// Tagged `YamlDoc` value, contains a [`Tag`] and a node that's a [`Box<Node>`]
    Tagged(Cow<'input, Tag>, Box<YamlHashNode<'input, Node>>),
}

impl<'input, T> YamlHashNode<'input, T>
where
    T: Clone + PartialEq + Eq + Hash + Eq + From<YamlDoc<'input>>,
{
    fn from_sequence(sequence: Vec<YamlDoc<'input>>) -> YamlHashNode<'input, T> {
        YamlHashNode::Sequence(sequence.into_iter().map(Into::into).collect())
    }
}

impl<'input, T> YamlHashNode<'input, T>
where
    T: Clone + PartialEq + Eq + Hash + Eq + From<YamlDoc<'input>>,
{
    fn from_mapping(mapping: Vec<YamlEntry<'input, YamlDoc<'input>>>) -> YamlHashNode<'input, T>
    where
        T: Clone + Eq + From<YamlDoc<'input>> + Hash + PartialEq,
    {
        YamlHashNode::Mapping(HashedMapWrap(
            mapping
                .into_iter()
                .map(|x| (x.key.into(), x.value.into()))
                .collect(),
        ))
    }
}

impl<'input, T> From<YamlDoc<'input>> for YamlHashNode<'input, T>
where
    T: Clone + PartialEq + Eq + Hash + Eq + From<YamlDoc<'input>> + Borrow<YamlDoc<'input>>,
{
    fn from(value: YamlDoc<'input>) -> Self {
        match value {
            YamlDoc::BadValue => YamlHashNode::BadValue,
            YamlDoc::Null => YamlHashNode::Null,
            YamlDoc::String(x) => YamlHashNode::String(x),
            YamlDoc::Bool(x) => YamlHashNode::Bool(x),
            YamlDoc::Alias(x) => YamlHashNode::Alias(x),
            YamlDoc::FloatingPoint(x) => YamlHashNode::FloatingPoint(OrderedFloat(x)),
            YamlDoc::Integer(x) => YamlHashNode::Integer(x),
            YamlDoc::Sequence(s) => YamlHashNode::from_sequence(s),
            YamlDoc::Mapping(m) => YamlHashNode::from_mapping(m),
            YamlDoc::Tagged(tag, data) => YamlHashNode::Tagged(tag, Box::new((*data).into())),
        }
    }
}

impl<'input, Node> YamlDocAccess<'input> for YamlHashNode<'input, Node>
where
    Node: Clone + PartialEq + Eq + Hash + Eq + YamlDocAccess<'input> + From<YamlDoc<'input>>,
{
    type Node = Node;
    type SequenceNode = Vec<Node>;
    type MappingNode = HashMap<Node, Node>;

    fn key_from_usize(index: usize) -> Self {
        YamlHashNode::Integer(index as i64)
    }

    fn key_from_str(index: &str) -> Self {
        YamlHashNode::String(Cow::Owned(index.to_string()))
    }

    fn is_non_empty_collection(&self) -> bool {
        match self {
            YamlHashNode::Sequence(v) => !v.is_empty(),
            YamlHashNode::Mapping(v) => !v.0.is_empty(),
            _ => false,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            YamlHashNode::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn as_bool_mut(&mut self) -> Option<&mut bool> {
        match self {
            YamlHashNode::Bool(b) => Some(b),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            YamlHashNode::Integer(i) => Some(*i),
            _ => None,
        }
    }

    fn as_i64_mut(&mut self) -> Option<&mut i64> {
        match self {
            YamlHashNode::Integer(i) => Some(i),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            YamlHashNode::FloatingPoint(i) => Some(i.0),
            _ => None,
        }
    }

    fn as_f64_mut(&mut self) -> Option<&mut f64> {
        match self {
            YamlHashNode::FloatingPoint(i) => Some(i.as_mut()),
            _ => None,
        }
    }

    fn as_sequence(&self) -> Result<&Self::SequenceNode, YamlAccessError> {
        match self {
            YamlHashNode::Sequence(i) => Ok(i),
            _ => Err(YamlAccessError::ExpectedSequence),
        }
    }

    fn as_sequence_mut(&mut self) -> Result<&mut Self::SequenceNode, YamlAccessError> {
        match self {
            YamlHashNode::Sequence(i) => Ok(i),
            _ => Err(YamlAccessError::ExpectedSequence),
        }
    }

    fn as_mapping(&self) -> Result<&Self::MappingNode, YamlAccessError> {
        match self {
            YamlHashNode::Mapping(i) => Ok(&i.0),
            _ => Err(YamlAccessError::ExpectedMapping),
        }
    }

    fn as_mapping_mut(&mut self) -> Result<&mut Self::MappingNode, YamlAccessError> {
        match self {
            YamlHashNode::Mapping(i) => Ok(&mut i.0),
            _ => Err(YamlAccessError::ExpectedMapping),
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            YamlHashNode::String(s) => Some(s.as_ref()),
            _ => None,
        }
    }

    fn as_str_mut(&mut self) -> Option<&mut str> {
        match self {
            YamlHashNode::String(s) => Some(s.to_mut()),
            _ => None,
        }
    }

    fn sequence_mut(&mut self) -> &mut Self::SequenceNode {
        match self {
            YamlHashNode::Sequence(seq) => seq,
            _ => core::panic!("Expected sequence got {:?}", self.get_type()),
        }
    }

    fn mapping_mut(&mut self) -> &mut Self::MappingNode {
        match self {
            YamlHashNode::Mapping(seq) => &mut seq.0,
            _ => core::panic!("Expected sequence got {:?}", self.get_type()),
        }
    }

    fn get_tag(&self) -> Option<Tag> {
        match self {
            YamlHashNode::Tagged(tag, ..) => Some(Tag::new(&tag.handle, &tag.suffix)),
            _ => None,
        }
    }

    fn get_type(&self) -> NodeType {
        match self {
            YamlHashNode::BadValue => NodeType::Bad,
            YamlHashNode::Null => NodeType::Null,
            YamlHashNode::String(_) => NodeType::String,
            YamlHashNode::Bool(_) => NodeType::Bool,
            YamlHashNode::FloatingPoint(_) => NodeType::Floating,
            YamlHashNode::Integer(_) => NodeType::Integer,
            YamlHashNode::Sequence(_) => NodeType::Sequence,
            YamlHashNode::Mapping(_) => NodeType::Mapping,
            YamlHashNode::Alias(_) => NodeType::Alias,
            YamlHashNode::Tagged(_, a) => a.get_type(),
        }
    }

    fn into_string(self) -> Option<String> {
        match self {
            YamlHashNode::String(string) => Some(string.to_string()),
            _ => None,
        }
    }

    fn into_mapping(self) -> Option<Self::MappingNode> {
        match self {
            YamlHashNode::Mapping(mapping) => Some(mapping.0),
            _ => None,
        }
    }

    fn into_sequence(self) -> Option<Self::SequenceNode> {
        match self {
            YamlHashNode::Sequence(seq) => Some(seq),
            _ => None,
        }
    }

    fn into_tagged(self, tag: Cow<'input, Tag>) -> Self {
        YamlHashNode::Tagged(tag, Box::new(self))
    }

    fn from_bare_yaml(yaml: YamlDoc<'input>) -> Self {
        match yaml {
            YamlDoc::BadValue => YamlHashNode::BadValue,
            YamlDoc::Null => YamlHashNode::Null,
            YamlDoc::String(s) => YamlHashNode::String(s),
            YamlDoc::Bool(b) => YamlHashNode::Bool(b),
            YamlDoc::FloatingPoint(fp) => YamlHashNode::FloatingPoint(OrderedFloat::from(fp)),
            YamlDoc::Integer(i) => YamlHashNode::Integer(i),
            YamlDoc::Sequence(seq) => YamlHashNode::from_sequence(seq),
            YamlDoc::Mapping(map) => YamlHashNode::from_mapping(map),
            YamlDoc::Alias(alias) => YamlHashNode::Alias(alias),
            YamlDoc::Tagged(tag, yaml) => {
                YamlHashNode::Tagged(tag, Box::new(YamlHashNode::from_bare_yaml(*yaml)))
            }
        }
    }

    fn bad_span_value(_span: Span) -> Self {
        YamlHashNode::BadValue
    }
}

#[allow(clippy::cast_possible_wrap)]
impl<'input, Node> Index<usize> for YamlHashNode<'input, Node>
where
    Node: Clone + YamlDocAccess<'input> + PartialEq + Hash + Eq + From<YamlDoc<'input>>,
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
            YamlHashNode::Sequence(sequence) => sequence.index(index),
            YamlHashNode::Mapping(mapping) => mapping
                .0
                .get(&Node::key_from_usize(index))
                .unwrap_or_else(|| panic!("Key {index} not found in `YamlHashNode` mapping")),
            _ => panic!("Attempt to index {get_type:?} in `YamlHashNode`"),
        }
    }
}
#[allow(clippy::cast_possible_wrap)]
impl<'input, Node> IndexMut<usize> for YamlHashNode<'input, Node>
where
    Node: Clone + YamlDocAccess<'input> + PartialEq + Hash + Eq + From<YamlDoc<'input>>,
{
    /// Perform index by integer.
    ///
    /// When `self` is a sequence, the method will attempt to access underlying vector at given position.
    /// When `self` is a mapping, the method will attempt the underlying vector at amap assuming `index` is a key
    /// to its value. For example, YAML `{ 0: "test" }` can be accessed using `0`.
    ///
    /// # Panics
    /// This function panics if the index doesn't exist in sequence or if the mapping doesn't contain
    /// an index key with the same value.
    fn index_mut(&mut self, index: usize) -> &mut Node {
        let get_type = self.get_type();
        match self {
            YamlHashNode::Sequence(sequence) => sequence.index_mut(index),
            YamlHashNode::Mapping(mapping) => mapping
                .0
                .get_mut(&Node::key_from_usize(index))
                .unwrap_or_else(|| panic!("Key {index} not found in `YamlHashNode` mapping")),
            _ => panic!("Attempt to index {get_type:?} with {index} in `YamlHashNode`"),
        }
    }
}

impl<'input, 'key, Node> Index<&'key str> for YamlHashNode<'input, Node>
where
    Node: Clone + YamlDocAccess<'input> + PartialEq + Hash + Eq + From<YamlDoc<'input>>,
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
            YamlHashNode::Mapping(mapping) => mapping
                .0
                .get(&Node::key_from_str(index))
                .unwrap_or_else(|| panic!("Key {index} not found in `YamlHashNode` mapping")),
            _ => panic!("Attempt to index {get_type:?} with {index} in `YamlHashNode`"),
        }
    }
}

impl<'input, 'key, Node> IndexMut<&'key str> for YamlHashNode<'input, Node>
where
    Node: Clone + YamlDocAccess<'input> + PartialEq + Hash + Eq + From<YamlDoc<'input>>,
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
            YamlHashNode::Mapping(mapping) => mapping
                .0
                .get_mut(&Node::key_from_str(index))
                .unwrap_or_else(|| panic!("Key {index} not found in `YamlHashNode` mapping")),
            _ => panic!("Attempt to index {get_type:?} in `YamlHashNode`"),
        }
    }
}

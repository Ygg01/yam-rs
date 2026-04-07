use crate::yaml_doc::NodeType;
use crate::{Mapping, Sequence, Tag, YamlDoc, YamlDocAccess, YamlEntry};
use std::borrow::Cow;
use std::ops::{Index, IndexMut};

///
/// Represents a cloned YAML node with various possible types of values.
///
/// The `YamlCloneNode` enum provides a structured way to represent YAML values
/// in Rust. Each variant corresponds to a possible type of value that a YAML
/// node can hold, such as `null`, `string`, `integer`, `sequence`, or `mapping`.
///
/// # Type Parameters
///
/// * `'input`: Lifetime parameter for borrowed string slices (`&str`) used in
///   the `String` and `Tagged` variants.
/// * `Node`: Generic parameter for representing nested or child nodes. It must
///   implement the `Clone` trait.
///
/// # Traits Implementations
///
/// * `Default`: The default value for `YamlCloneNode` is the `Null` variant.
/// * `PartialEq`: Supports equality comparisons between two `YamlCloneNode`
///   instances.
/// * `Clone`: Allows cloning of `YamlCloneNode` instances.
/// * `Debug`: Enables formatted output for debugging purposes.
///
#[derive(Debug, Default, PartialEq, Clone)]
pub enum YamlCloneNode<'input, Node: Clone> {
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
    Mapping(Vec<YamlEntry<'input, Node>>),
    /// Represents a pointer to another node like `[*lol, *lol]`
    Alias(usize),
    /// Tagged `YamlDoc` value, contains a [`Tag`] and a node that's a [`Box<Node>`]
    Tagged(Cow<'input, Tag>, Box<Node>),
}

impl<'input, Node> YamlDocAccess<'input> for YamlCloneNode<'input, Node>
where
    Node: Clone + YamlDocAccess<'input>,
{
    type Node = Node;
    type SequenceNode = Vec<Node>;
    type MappingNode = Vec<YamlEntry<'input, Node>>;

    fn is_non_empty_collection(&self) -> bool {
        match self {
            YamlCloneNode::Sequence(s) => !s.is_empty(),
            YamlCloneNode::Mapping(m) => !m.is_empty(),
            _ => false,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            YamlCloneNode::Bool(x) => Some(*x),
            _ => None,
        }
    }

    fn as_bool_mut(&mut self) -> Option<&mut bool> {
        match self {
            YamlCloneNode::Bool(x) => Some(x),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            YamlCloneNode::Integer(x) => Some(*x),
            _ => None,
        }
    }

    fn as_i64_mut(&mut self) -> Option<&mut i64> {
        match self {
            YamlCloneNode::Integer(x) => Some(x),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            YamlCloneNode::FloatingPoint(x) => Some(*x),
            _ => None,
        }
    }

    fn as_f64_mut(&mut self) -> Option<&mut f64> {
        match self {
            YamlCloneNode::FloatingPoint(x) => Some(x),
            _ => None,
        }
    }

    fn as_sequence(&self) -> Option<&Self::SequenceNode> {
        match self {
            YamlCloneNode::Sequence(x) => Some(x),
            _ => None,
        }
    }

    fn as_sequence_mut(&mut self) -> Option<&mut Self::SequenceNode> {
        match self {
            YamlCloneNode::Sequence(x) => Some(x),
            _ => None,
        }
    }

    fn as_mapping(&self) -> Option<&Self::MappingNode> {
        match self {
            YamlCloneNode::Mapping(x) => Some(x),
            _ => None,
        }
    }

    fn as_mapping_mut(&mut self) -> Option<&mut Self::MappingNode> {
        match self {
            YamlCloneNode::Mapping(x) => Some(x),
            _ => None,
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            YamlCloneNode::String(x) => Some(x.as_ref()),
            _ => None,
        }
    }

    fn as_str_mut(&mut self) -> Option<&mut str> {
        match self {
            &mut YamlCloneNode::String(ref mut v) => Some(v.to_mut()),
            _ => None,
        }
    }

    fn sequence_mut(&mut self) -> &mut Self::SequenceNode {
        match self {
            YamlCloneNode::Sequence(seq) => seq,
            _ => core::panic!("Expected sequence got {:?}", self.get_type()),
        }
    }

    fn mapping_mut(&mut self) -> &mut Vec<YamlEntry<'input, Node>> {
        match self {
            YamlCloneNode::Mapping(map) => map,
            _ => core::panic!("Expected mapping got {:?}", self.get_type()),
        }
    }

    fn get_tag(&self) -> Option<Tag> {
        match self {
            YamlCloneNode::Tagged(tag, ..) => Some(Tag::new(&tag.handle, &tag.suffix)),
            _ => None,
        }
    }

    fn get_type(&self) -> NodeType {
        match self {
            YamlCloneNode::BadValue => NodeType::Bad,
            YamlCloneNode::Null => NodeType::Null,
            YamlCloneNode::String(_) => NodeType::String,
            YamlCloneNode::Bool(_) => NodeType::Bool,
            YamlCloneNode::FloatingPoint(_) => NodeType::Floating,
            YamlCloneNode::Integer(_) => NodeType::Integer,
            YamlCloneNode::Sequence(_) => NodeType::Sequence,
            YamlCloneNode::Mapping(_) => NodeType::Mapping,
            YamlCloneNode::Alias(_) => NodeType::Alias,
            YamlCloneNode::Tagged(_, a) => a.get_type(),
        }
    }

    fn into_bool(self) -> Option<bool> {
        match self {
            YamlCloneNode::Bool(b) => Some(b),
            _ => None,
        }
    }

    fn into_string(self) -> Option<String> {
        match self {
            YamlCloneNode::String(s) => Some(s.to_string()),
            _ => None,
        }
    }

    fn into_cow(self) -> Option<Cow<'input, str>> {
        match self {
            YamlCloneNode::String(s) => Some(s),
            _ => None,
        }
    }

    fn into_f64(self) -> Option<f64> {
        match self {
            YamlCloneNode::FloatingPoint(f) => Some(f),
            _ => None,
        }
    }

    fn into_i64(self) -> Option<i64> {
        match self {
            YamlCloneNode::Integer(i) => Some(i),
            _ => None,
        }
    }

    fn into_mapping(self) -> Option<Self::MappingNode> {
        match self {
            YamlCloneNode::Mapping(mapping) => Some(mapping),
            _ => None,
        }
    }

    fn into_sequence(self) -> Option<Self::SequenceNode> {
        match self {
            YamlCloneNode::Sequence(seq) => Some(seq),
            _ => None,
        }
    }
}

impl<'input, T: Clone> From<YamlDoc<'input>> for YamlCloneNode<'input, T>
where
    T: From<YamlDoc<'input>>,
{
    fn from(value: YamlDoc<'input>) -> Self {
        match value {
            YamlDoc::BadValue => YamlCloneNode::BadValue,
            YamlDoc::Null => YamlCloneNode::Null,
            YamlDoc::String(x) => YamlCloneNode::String(x),
            YamlDoc::Bool(x) => YamlCloneNode::Bool(x),
            YamlDoc::Alias(x) => YamlCloneNode::Alias(x),
            YamlDoc::FloatingPoint(x) => YamlCloneNode::FloatingPoint(x),
            YamlDoc::Integer(x) => YamlCloneNode::Integer(x),
            YamlDoc::Sequence(s) => YamlCloneNode::from_sequence(s),
            YamlDoc::Mapping(m) => YamlCloneNode::from_mapping(m),
            YamlDoc::Tagged(tag, data) => YamlCloneNode::Tagged(tag, Box::new((*data).into())),
        }
    }
}

impl<'input, T> YamlCloneNode<'input, T>
where
    T: From<YamlDoc<'input>> + Clone,
{
    fn from_sequence(sequence: Sequence<'input>) -> YamlCloneNode<'input, T> {
        YamlCloneNode::Sequence(sequence.into_iter().map(Into::into).collect())
    }

    fn from_mapping(mapping: Mapping<'input>) -> YamlCloneNode<'input, T> {
        YamlCloneNode::Mapping(
            mapping
                .into_iter()
                .map(|x| YamlEntry::new(x.key.into(), x.value.into()))
                .collect(),
        )
    }
}

impl<'input, Node> Index<usize> for YamlCloneNode<'input, Node>
where
    Node: Clone + YamlDocAccess<'input> + PartialEq,
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
            YamlCloneNode::Sequence(sequence) => sequence.index(index),
            YamlCloneNode::Mapping(mapping) => {
                let key = i64::try_from(index).expect("Expected key to be lesser than `i64::max`");
                let find_key = mapping.iter().find(|entry| entry.key.as_i64() == Some(key));
                &find_key
                    .unwrap_or_else(|| panic!("Key {index} not found in `YamlCloneNode` mapping"))
                    .value
            }
            _ => panic!("Attempt to index {get_type:?} in `YamlCloneNode`"),
        }
    }
}

impl<'input, Node> IndexMut<usize> for YamlCloneNode<'input, Node>
where
    Node: Clone + YamlDocAccess<'input> + PartialEq,
{
    /// Perform index by integer.
    ///
    /// When `self` is a sequence, the method will attempt to access underlying vector at given position.
    /// When `self` is a mapping, the method will attempt to access underlying map assuming `index` is a key
    /// to its value. For example, YAML `{ 0: "test" }` can be accessed using `0`.
    ///
    /// # Panics
    /// This function panics if the index doesn't exist in sequence or if the mapping doesn't contain
    /// an index key with the same value.
    fn index_mut(&mut self, index: usize) -> &mut Node {
        let get_type = self.get_type();
        match self {
            YamlCloneNode::Sequence(sequence) => sequence.index_mut(index),
            YamlCloneNode::Mapping(mapping) => {
                let key_int =
                    i64::try_from(index).expect("Expected key to be lesser than `i64::max`");
                let find_key = mapping.iter_mut().find(|x| x.key.as_i64() == Some(key_int));
                &mut find_key
                    .unwrap_or_else(|| panic!("Key {index} not found in `YamlCloneNode` mapping"))
                    .value
            }
            _ => panic!("Attempt to index {get_type:?} with {index} in YamlCloneNode"),
        }
    }
}

impl<'input, 'key, Node> Index<&'key str> for YamlCloneNode<'input, Node>
where
    Node: Clone + YamlDocAccess<'input> + PartialEq,
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
            YamlCloneNode::Mapping(mapping) => {
                let find_key = mapping.iter().find(|x| x.key.as_str() == Some(index));
                &find_key
                    .unwrap_or_else(|| panic!("Key {index} not found in `YamlCloneNode` mapping"))
                    .value
            }
            _ => panic!("Attempt to index {get_type:?} with {index} in `YamlCloneNode`"),
        }
    }
}

impl<'input, 'key, Node> IndexMut<&'key str> for YamlCloneNode<'input, Node>
where
    Node: Clone + YamlDocAccess<'input> + PartialEq,
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
            YamlCloneNode::Mapping(mapping) => {
                let find_key = mapping.iter_mut().find(|x| x.key.as_str() == Some(index));
                &mut find_key
                    .unwrap_or_else(|| panic!("Key {index} not found in `YamlCloneNode` mapping"))
                    .value
            }
            _ => panic!("Attempt to index {get_type:?} in `YamlCloneNode`"),
        }
    }
}

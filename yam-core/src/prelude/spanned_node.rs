//! Basic node

use crate::prelude::{NodeType, Span, Tag, YamlAccessError, YamlDoc, YamlDocAccess, YamlEntry};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Struct representing a [`YamlOwnedNode`] node and a [`Span`]. Useful when we need Span information
/// about each node.
#[derive(PartialEq, Clone, Default)]
pub struct OwnedSpannedYaml<'a> {
    /// [Clonable](Clone) Yaml Node
    pub data: YamlData<'a, OwnedSpannedYaml<'a>>,
    /// [`Span`] information about the data field node.
    pub span: Span,
}

#[derive(Debug, Default, PartialEq, Clone)]
pub enum YamlData<'a, Node>
where
    Node: Clone,
{
    #[default]
    /// Invalid value for `YamlDoc`
    BadValue,
    /// Represents a `null` value for `YamlDoc`
    Null,
    /// Represents a YAML string value.
    String(Cow<'a, str>),
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
    Mapping(Vec<YamlEntry<'a, Node>>),
    /// Represents a pointer to another node like `[*lol, *lol]`
    Alias(usize),
    /// Tagged `YamlDoc` value, contains a [`Tag`] and a node that's a [`Box<Node>`]
    Tagged(Tag, Box<Node>),
}

impl<'input, Node> From<YamlDoc<'input>> for YamlData<'input, Node>
where
    Node: Clone + YamlDocAccess<'input> + From<YamlDoc<'input>> + Into<YamlDoc<'input>>,
{
    fn from(yaml_data: YamlDoc<'input>) -> Self {
        match yaml_data {
            YamlDoc::BadValue => YamlData::BadValue,
            YamlDoc::Null => YamlData::Null,
            YamlDoc::String(s) => YamlData::String(s),
            YamlDoc::Bool(b) => YamlData::Bool(b),
            YamlDoc::FloatingPoint(fp) => YamlData::FloatingPoint(fp),
            YamlDoc::Integer(i) => YamlData::Integer(i),
            YamlDoc::Sequence(seq) => YamlData::Sequence(seq.into_iter().map(Into::into).collect()),
            YamlDoc::Mapping(map) => YamlData::Mapping(
                map.into_iter()
                    .map(|x| YamlEntry::new(x.key.into(), x.value.into()))
                    .collect(),
            ),
            YamlDoc::Alias(a) => YamlData::Alias(a),
            YamlDoc::Tagged(tag, data) => {
                YamlData::Tagged(tag.into_owned(), Box::new(Node::from(*data)))
            }
        }
    }
}

impl<'input, Node> From<YamlData<'input, Node>> for YamlDoc<'input>
where
    Node: Clone + YamlDocAccess<'input> + From<YamlDoc<'input>> + Into<YamlDoc<'input>>,
{
    fn from(value: YamlData<'input, Node>) -> Self {
        match value {
            YamlData::BadValue => YamlDoc::BadValue,
            YamlData::Null => YamlDoc::Null,
            YamlData::String(s) => YamlDoc::String(s),
            YamlData::Bool(b) => YamlDoc::Bool(b),
            YamlData::FloatingPoint(fp) => YamlDoc::FloatingPoint(fp),
            YamlData::Integer(i) => YamlDoc::Integer(i),
            YamlData::Sequence(seq) => YamlDoc::Sequence(seq.into_iter().map(Into::into).collect()),
            YamlData::Mapping(map) => YamlDoc::Mapping(
                map.into_iter()
                    .map(|x| YamlEntry::new(x.key.into(), x.value.into()))
                    .collect(),
            ),
            YamlData::Alias(a) => YamlDoc::Alias(a),
            YamlData::Tagged(tag, data) => {
                let tag = Cow::Owned(tag);
                let data: Node = *data;
                YamlDoc::Tagged(tag, Box::new(data.into()))
            }
        }
    }
}

impl<'a, Node> YamlData<'a, Node>
where
    Node: Clone + Into<YamlData<'a, Node>>,
{
    fn from_node(node: Node) -> Self {
        node.into()
    }
}

impl<'a, Node> YamlDocAccess<'a> for YamlData<'a, Node>
where
    Node: Clone + YamlDocAccess<'a> + From<YamlDoc<'a>> + Into<YamlDoc<'a>>,
{
    type OutNode = YamlData<'a, Node>;
    type SequenceNode = Vec<Node>;
    type MappingNode = Vec<YamlEntry<'a, Node>>;

    fn key_from_usize(index: usize) -> Self {
        YamlData::Integer(index as i64)
    }

    fn key_from_str(index: &str) -> Self {
        YamlData::String(Cow::Owned(index.to_string()))
    }

    fn is_non_empty_collection(&self) -> bool {
        match self {
            YamlData::Sequence(s) => !s.is_empty(),
            YamlData::Mapping(m) => !m.is_empty(),
            _ => false,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            YamlData::Bool(x) => Some(*x),
            _ => None,
        }
    }

    fn as_bool_mut(&mut self) -> Option<&mut bool> {
        match self {
            YamlData::Bool(x) => Some(x),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            YamlData::Integer(i) => Some(*i),
            _ => None,
        }
    }

    fn as_i64_mut(&mut self) -> Option<&mut i64> {
        match self {
            YamlData::Integer(x) => Some(x),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            YamlData::FloatingPoint(x) => Some(*x),
            _ => None,
        }
    }

    fn as_f64_mut(&mut self) -> Option<&mut f64> {
        match self {
            YamlData::FloatingPoint(x) => Some(x),
            _ => None,
        }
    }

    fn as_sequence(&self) -> Result<&Self::SequenceNode, YamlAccessError> {
        match self {
            YamlData::Sequence(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedSequence),
        }
    }

    fn as_sequence_mut(&mut self) -> Result<&mut Self::SequenceNode, YamlAccessError> {
        match self {
            YamlData::Sequence(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedSequence),
        }
    }

    fn as_mapping(&self) -> Result<&Self::MappingNode, YamlAccessError> {
        match self {
            YamlData::Mapping(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedMapping),
        }
    }

    fn as_mapping_mut(&mut self) -> Result<&mut Self::MappingNode, YamlAccessError> {
        match self {
            YamlData::Mapping(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedMapping),
        }
    }

    fn as_str(&self) -> Option<&str> {
        match self {
            YamlData::String(x) => Some(x.as_ref()),
            _ => None,
        }
    }

    fn as_str_mut(&mut self) -> Option<&mut str> {
        match self {
            YamlData::String(x) => Some(x.to_mut()),
            _ => None,
        }
    }

    fn sequence_mut(&mut self) -> &mut Self::SequenceNode {
        match self {
            YamlData::Sequence(seq) => seq,
            _ => core::panic!("Expected sequence got {:?}", self.get_type()),
        }
    }

    fn mapping_mut(&mut self) -> &mut Self::MappingNode {
        match self {
            YamlData::Mapping(map) => map,
            _ => core::panic!("Expected sequence got {:?}", self.get_type()),
        }
    }

    fn get_tag(&self) -> Option<Tag> {
        match self {
            YamlData::Tagged(tag, ..) => Some(Tag::new(&tag.handle, &tag.suffix)),
            _ => None,
        }
    }

    fn get_type(&self) -> NodeType {
        match self {
            YamlData::BadValue => NodeType::Bad,
            YamlData::Null => NodeType::Null,
            YamlData::String(_) => NodeType::String,
            YamlData::Bool(_) => NodeType::Bool,
            YamlData::FloatingPoint(_) => NodeType::Floating,
            YamlData::Integer(_) => NodeType::Integer,
            YamlData::Sequence(_) => NodeType::Sequence,
            YamlData::Mapping(_) => NodeType::Mapping,
            YamlData::Alias(_) => NodeType::Alias,
            YamlData::Tagged(_, a) => a.get_type(),
        }
    }

    fn into_string(self) -> Option<String> {
        match self {
            YamlData::String(s) => Some(s.to_string()),
            _ => None,
        }
    }

    fn into_mapping(self) -> Option<Self::MappingNode> {
        match self {
            YamlData::Mapping(m) => Some(m),
            _ => None,
        }
    }

    fn into_sequence(self) -> Option<Self::SequenceNode> {
        match self {
            YamlData::Sequence(m) => Some(m),
            _ => None,
        }
    }

    fn into_tagged(self, tag: Cow<'a, Tag>) -> Self {
        YamlData::Tagged(tag.into_owned(), Box::new(Node::from(self.into())))
    }

    fn bad_span_value(_span: Span) -> Self {
        YamlData::BadValue
    }
}

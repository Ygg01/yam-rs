//! Basic node

use crate::prelude::YamlOwnedNode::BadValue;
use crate::prelude::{
    NodeType, Span, Tag, YamlAccessError, YamlDoc, YamlDocAccess, YamlEntry, YamlOwnedNode,
};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Struct representing a [`YamlOwnedNode`] node and a [`Span`]. Useful when we need Span information
/// about each node.
#[derive(PartialEq, Clone, Default)]
pub struct SpannedYaml {
    /// [Clonable](Clone) Yaml Node
    pub data: YamlOwnedNode<SpannedYaml>,
    /// [`Span`] information about the data field node.
    pub span: Span,
}

impl<'input> From<YamlDoc<'input>> for SpannedYaml {
    fn from(yaml_data: YamlDoc<'input>) -> Self {
        SpannedYaml {
            data: yaml_data.into(),
            span: Span::default(),
        }
    }
}

impl YamlDocAccess<'static> for SpannedYaml {
    type Node = SpannedYaml;
    type SequenceNode = Vec<SpannedYaml>;
    type MappingNode = Vec<YamlEntry<'static, SpannedYaml>>;

    fn key_from_usize(index: usize) -> Self {
        SpannedYaml {
            span: Span::default(),
            data: YamlOwnedNode::Integer(index as i64),
        }
    }

    fn key_from_str(index: &str) -> Self {
        SpannedYaml {
            span: Span::default(),
            data: YamlOwnedNode::String(index.to_string()),
        }
    }

    fn is_non_empty_collection(&self) -> bool {
        match &self.data {
            YamlOwnedNode::Sequence(s) => !s.is_empty(),
            YamlOwnedNode::Mapping(m) => !m.is_empty(),
            _ => false,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match &self.data {
            YamlOwnedNode::Bool(x) => Some(*x),
            _ => None,
        }
    }

    fn as_bool_mut(&mut self) -> Option<&mut bool> {
        match &mut self.data {
            YamlOwnedNode::Bool(x) => Some(x),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match &self.data {
            YamlOwnedNode::Integer(x) => Some(*x),
            _ => None,
        }
    }

    fn as_i64_mut(&mut self) -> Option<&mut i64> {
        match &mut self.data {
            YamlOwnedNode::Integer(x) => Some(x),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match &self.data {
            YamlOwnedNode::FloatingPoint(x) => Some(*x),
            _ => None,
        }
    }

    fn as_f64_mut(&mut self) -> Option<&mut f64> {
        match &mut self.data {
            YamlOwnedNode::FloatingPoint(x) => Some(x),
            _ => None,
        }
    }

    fn as_sequence(&self) -> Result<&Vec<SpannedYaml>, YamlAccessError> {
        match &self.data {
            YamlOwnedNode::Sequence(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedSequence),
        }
    }

    fn as_sequence_mut(&mut self) -> Result<&mut Self::SequenceNode, YamlAccessError> {
        match &mut self.data {
            YamlOwnedNode::Sequence(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedSequence),
        }
    }

    fn as_mapping(&self) -> Result<&Self::MappingNode, YamlAccessError> {
        match &self.data {
            YamlOwnedNode::Mapping(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedMapping),
        }
    }

    fn as_mapping_mut(&mut self) -> Result<&mut Self::MappingNode, YamlAccessError> {
        match &mut self.data {
            YamlOwnedNode::Mapping(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedMapping),
        }
    }

    fn as_str(&self) -> Option<&str> {
        match &self.data {
            YamlOwnedNode::String(x) => Some(x.as_ref()),
            _ => None,
        }
    }

    fn as_str_mut(&mut self) -> Option<&mut str> {
        match &mut self.data {
            &mut YamlOwnedNode::String(ref mut v) => Some(v.as_mut_str()),
            _ => None,
        }
    }

    fn sequence_mut(&mut self) -> &mut Vec<Self> {
        match self.data {
            YamlOwnedNode::Sequence(ref mut s) => s,
            _ => panic!("Cannot get sequence_mut for non-sequence data"),
        }
    }

    fn mapping_mut(&mut self) -> &mut Self::MappingNode {
        match self.data {
            YamlOwnedNode::Mapping(ref mut s) => s,
            _ => panic!("Cannot get mapping_mut for non-mappingdata data"),
        }
    }

    fn get_tag(&self) -> Option<Tag> {
        match &self.data {
            YamlOwnedNode::Tagged(tag, ..) => Some(Tag::new(&tag.handle, &tag.suffix)),
            _ => None,
        }
    }

    fn get_type(&self) -> NodeType {
        self.data.get_type()
    }

    fn into_bool(self) -> Option<bool> {
        match self.data {
            YamlOwnedNode::Bool(b) => Some(b),
            _ => None,
        }
    }

    fn into_string(self) -> Option<String> {
        match self.data {
            YamlOwnedNode::String(s) => Some(s.to_string()),
            _ => None,
        }
    }

    fn into_mapping(self) -> Option<Self::MappingNode> {
        match self.data {
            YamlOwnedNode::Mapping(mapping) => Some(mapping),
            _ => None,
        }
    }

    fn into_sequence(self) -> Option<Self::SequenceNode> {
        match self.data {
            YamlOwnedNode::Sequence(seq) => Some(seq),
            _ => None,
        }
    }

    fn into_tagged(self, tag: Cow<'static, Tag>) -> Self {
        SpannedYaml {
            data: YamlOwnedNode::Tagged(tag.into_owned(), Box::new(self.data)),
            span: self.span,
        }
    }

    fn from_bare_yaml(yaml: YamlDoc<'static>) -> Self {
        todo!()
    }

    fn bad_span_value(span: Span) -> Self {
        SpannedYaml {
            span,
            data: BadValue,
        }
    }
}

use crate::{LoadableYamlNode, Span, Tag, YamlAccessError, YamlDoc, YamlDocAccess, YamlEntry};
use crate::{NodeType, YamlCloneNode};
use std::borrow::Cow;

#[derive(PartialEq, Clone, Default)]
pub struct SpannedYaml<'input> {
    pub data: YamlCloneNode<'input, SpannedYaml<'input>>,
    pub span: Span,
}

impl<'input> From<YamlDoc<'input>> for SpannedYaml<'input> {
    fn from(yaml_data: YamlDoc<'input>) -> Self {
        SpannedYaml {
            data: yaml_data.into(),
            span: Span::default(),
        }
    }
}

impl<'input> YamlDocAccess<'input> for SpannedYaml<'input> {
    type Node = SpannedYaml<'input>;
    type SequenceNode = Vec<SpannedYaml<'input>>;
    type MappingNode = Vec<YamlEntry<'input, SpannedYaml<'input>>>;

    fn is_non_empty_collection(&self) -> bool {
        match &self.data {
            YamlCloneNode::Sequence(s) => !s.is_empty(),
            YamlCloneNode::Mapping(m) => !m.is_empty(),
            _ => false,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match &self.data {
            YamlCloneNode::Bool(x) => Some(*x),
            _ => None,
        }
    }

    fn as_bool_mut(&mut self) -> Option<&mut bool> {
        match &mut self.data {
            YamlCloneNode::Bool(x) => Some(x),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match &self.data {
            YamlCloneNode::Integer(x) => Some(*x),
            _ => None,
        }
    }

    fn as_i64_mut(&mut self) -> Option<&mut i64> {
        match &mut self.data {
            YamlCloneNode::Integer(x) => Some(x),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match &self.data {
            YamlCloneNode::FloatingPoint(x) => Some(*x),
            _ => None,
        }
    }

    fn as_f64_mut(&mut self) -> Option<&mut f64> {
        match &mut self.data {
            YamlCloneNode::FloatingPoint(x) => Some(x),
            _ => None,
        }
    }

    fn as_sequence(&self) -> Result<&Vec<SpannedYaml<'input>>, YamlAccessError> {
        match &self.data {
            YamlCloneNode::Sequence(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedSequence),
        }
    }

    fn as_sequence_mut(&mut self) -> Result<&mut Self::SequenceNode, YamlAccessError> {
        match &mut self.data {
            YamlCloneNode::Sequence(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedSequence),
        }
    }

    fn as_mapping(&self) -> Result<&Self::MappingNode, YamlAccessError> {
        match &self.data {
            YamlCloneNode::Mapping(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedMapping),
        }
    }

    fn as_mapping_mut(&mut self) -> Result<&mut Self::MappingNode, YamlAccessError> {
        match &mut self.data {
            YamlCloneNode::Mapping(x) => Ok(x),
            _ => Err(YamlAccessError::ExpectedMapping),
        }
    }

    fn as_str(&self) -> Option<&str> {
        match &self.data {
            YamlCloneNode::String(x) => Some(x.as_ref()),
            _ => None,
        }
    }

    fn as_str_mut(&mut self) -> Option<&mut str> {
        match &mut self.data {
            &mut YamlCloneNode::String(ref mut v) => Some(v.to_mut()),
            _ => None,
        }
    }

    fn sequence_mut(&mut self) -> &mut Vec<Self> {
        match self.data {
            YamlCloneNode::Sequence(ref mut s) => s,
            _ => panic!("Cannot get sequence_mut for non-sequence data"),
        }
    }

    fn mapping_mut(&mut self) -> &mut Self::MappingNode {
        match self.data {
            YamlCloneNode::Mapping(ref mut s) => s,
            _ => panic!("Cannot get mapping_mut for non-mappingdata data"),
        }
    }

    fn get_tag(&self) -> Option<Tag> {
        match &self.data {
            YamlCloneNode::Tagged(tag, ..) => Some(Tag::new(&tag.handle, &tag.suffix)),
            _ => None,
        }
    }

    fn get_type(&self) -> NodeType {
        self.data.get_type()
    }

    fn into_bool(self) -> Option<bool> {
        match self.data {
            YamlCloneNode::Bool(b) => Some(b),
            _ => None,
        }
    }

    fn into_string(self) -> Option<String> {
        match self.data {
            YamlCloneNode::String(s) => Some(s.to_string()),
            _ => None,
        }
    }

    fn into_cow(self) -> Option<Cow<'input, str>> {
        match self.data {
            YamlCloneNode::String(s) => Some(s),
            _ => None,
        }
    }

    fn into_f64(self) -> Option<f64> {
        match self.data {
            YamlCloneNode::FloatingPoint(f) => Some(f),
            _ => None,
        }
    }

    fn into_i64(self) -> Option<i64> {
        match self.data {
            YamlCloneNode::Integer(i) => Some(i),
            _ => None,
        }
    }

    fn into_mapping(self) -> Option<Self::MappingNode> {
        match self.data {
            YamlCloneNode::Mapping(mapping) => Some(mapping),
            _ => None,
        }
    }

    fn into_sequence(self) -> Option<Self::SequenceNode> {
        match self.data {
            YamlCloneNode::Sequence(seq) => Some(seq),
            _ => None,
        }
    }
}

impl<'input> LoadableYamlNode<'input> for SpannedYaml<'input> {
    fn into_tagged(self, tag: Cow<'input, Tag>) -> Self {
        SpannedYaml {
            data: YamlCloneNode::Tagged(tag, Box::new(self)),
            span: Span::default(),
        }
    }

    fn from_bare_yaml(yaml: YamlDoc<'input>) -> Self {
        SpannedYaml {
            data: yaml.into(),
            span: Span::default(),
        }
    }

    fn bad_value() -> Self {
        SpannedYaml {
            data: YamlCloneNode::BadValue,
            span: Span::default(),
        }
    }

    fn take(&mut self) -> Self {
        core::mem::take(self)
    }
}

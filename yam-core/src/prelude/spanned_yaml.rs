use crate::YamlDocAccess;
use crate::prelude::{
    IsEmpty, NodeType, Span, Tag, ToMutStr, YamlAccessError, YamlData, YamlEntry, YamlScalar,
};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub struct SpannedYaml<'a, FP = f64> {
    span: Span,
    yaml: YamlData<'a, SpannedYaml<'a, FP>, FP>,
}

impl<FP> Clone for SpannedYaml<'_, FP>
where
    FP: Copy,
{
    fn clone(&self) -> Self {
        SpannedYaml {
            span: self.span,
            yaml: self.yaml.clone(),
        }
    }
}

impl<'a, FP> YamlDocAccess<'a> for SpannedYaml<'a, FP>
where
    FP: Copy + AsMut<f64> + Into<f64>,
{
    type OutNode = Self;
    type SequenceNode = Vec<Self>;
    type MappingNode = Vec<YamlEntry<'a, Self>>;

    fn key_from_usize(index: usize) -> Self {
        SpannedYaml {
            span: Span::default(),
            yaml: YamlData::Scalar(YamlScalar::Integer(index as i64)),
        }
    }

    fn key_from_str(index: &str) -> Self {
        SpannedYaml {
            span: Span::default(),
            yaml: YamlData::Scalar(YamlScalar::String(Cow::Owned(index.to_string()))),
        }
    }

    fn is_non_empty_collection(&self) -> bool {
        match &self.yaml {
            YamlData::Sequence(s) => !s.is_collection_empty(),
            YamlData::Mapping(m) => !m.is_collection_empty(),
            _ => false,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match &self.yaml {
            YamlData::Scalar(YamlScalar::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    fn as_bool_mut(&mut self) -> Option<&mut bool> {
        match &mut self.yaml {
            YamlData::Scalar(YamlScalar::Bool(b)) => Some(b),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match &self.yaml {
            YamlData::Scalar(YamlScalar::Integer(b)) => Some(*b),
            _ => None,
        }
    }

    fn as_i64_mut(&mut self) -> Option<&mut i64> {
        match &mut self.yaml {
            YamlData::Scalar(YamlScalar::Integer(b)) => Some(b),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match &self.yaml {
            YamlData::Scalar(YamlScalar::FloatingPoint(b)) => Some((*b).into()),
            _ => None,
        }
    }

    fn as_f64_mut(&mut self) -> Option<&mut f64> {
        match &mut self.yaml {
            YamlData::Scalar(YamlScalar::FloatingPoint(b)) => Some(b.as_mut()),
            _ => None,
        }
    }

    fn as_sequence(&self) -> Result<&Self::SequenceNode, YamlAccessError> {
        match &self.yaml {
            YamlData::Sequence(s) => Ok(s),
            _ => Err(YamlAccessError::ExpectedSequence),
        }
    }

    fn as_sequence_mut(&mut self) -> Result<&mut Self::SequenceNode, YamlAccessError> {
        match &mut self.yaml {
            YamlData::Sequence(s) => Ok(s),
            _ => Err(YamlAccessError::ExpectedSequence),
        }
    }

    fn as_mapping(&self) -> Result<&Self::MappingNode, YamlAccessError> {
        match &self.yaml {
            YamlData::Mapping(s) => Ok(s),
            _ => Err(YamlAccessError::ExpectedMapping),
        }
    }

    fn as_mapping_mut(&mut self) -> Result<&mut Self::MappingNode, YamlAccessError> {
        match &mut self.yaml {
            YamlData::Mapping(s) => Ok(s),
            _ => Err(YamlAccessError::ExpectedMapping),
        }
    }

    fn as_str(&self) -> Option<&str> {
        match &self.yaml {
            YamlData::Scalar(YamlScalar::String(s)) => Some(s.as_ref()),
            _ => None,
        }
    }

    fn as_str_mut(&mut self) -> Option<&mut str> {
        match &mut self.yaml {
            YamlData::Scalar(YamlScalar::String(s)) => Some(s.mut_str()),
            _ => None,
        }
    }

    fn sequence_mut(&mut self) -> &mut Self::SequenceNode {
        match &mut self.yaml {
            YamlData::Sequence(s) => s,
            _ => core::panic!("YamlData::sequence_mut() called with non-sequence"),
        }
    }

    fn sequence(&self) -> &Self::SequenceNode {
        match &self.yaml {
            YamlData::Sequence(s) => s,
            _ => core::panic!("YamlData::sequence() called with non-sequence"),
        }
    }

    fn mapping_mut(&mut self) -> &mut Self::MappingNode {
        match &mut self.yaml {
            YamlData::Mapping(m) => m,
            _ => core::panic!("YamlData::mapping_mut() called with non-mapping"),
        }
    }

    fn mapping(&self) -> &Self::MappingNode {
        match &self.yaml {
            YamlData::Mapping(m) => m,
            _ => core::panic!("YamlData::mapping() called with non-mapping"),
        }
    }

    fn get_tag(&self) -> Option<Tag> {
        match &self.yaml {
            YamlData::Tagged(tag, ..) => Some(tag.clone().into_owned()),
            _ => None,
        }
    }

    fn get_type(&self) -> NodeType {
        match &self.yaml {
            YamlData::Mapping(_) => NodeType::Mapping,
            YamlData::Sequence(_) => NodeType::Sequence,
            YamlData::Scalar(YamlScalar::Bool(_)) => NodeType::Bool,
            YamlData::Scalar(YamlScalar::Integer(_)) => NodeType::Integer,
            YamlData::Scalar(YamlScalar::FloatingPoint(_)) => NodeType::Floating,
            YamlData::Scalar(YamlScalar::String(_)) => NodeType::String,
            YamlData::Alias(_) => NodeType::Alias,
            YamlData::Scalar(YamlScalar::Null(_)) => NodeType::Null,
            _ => NodeType::Bad,
        }
    }

    fn into_string(self) -> Option<String> {
        match self.yaml {
            YamlData::Scalar(YamlScalar::String(s)) => Some(s.into()),
            _ => None,
        }
    }

    fn into_mapping(self) -> Option<Self::MappingNode> {
        match self.yaml {
            YamlData::Mapping(s) => Some(s),
            _ => None,
        }
    }

    fn into_sequence(self) -> Option<Self::SequenceNode> {
        match self.yaml {
            YamlData::Sequence(s) => Some(s),
            _ => None,
        }
    }

    fn into_tagged(self, tag: Cow<'a, Tag>) -> Self {
        SpannedYaml {
            span: self.span,
            yaml: YamlData::Tagged(tag, Box::new(self)),
        }
    }

    fn bad_span_value(span: Span) -> Self {
        SpannedYaml {
            span,
            yaml: YamlData::BadValue,
        }
    }
}

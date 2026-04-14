use crate::YamlDocAccess;
use crate::prelude::{IsEmpty, NodeType, Span, Tag, YamlAccessError, YamlData, YamlScalar};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

pub struct Yaml<'a, MAP, STR = Cow<'a, str>, FP = f64>(
    pub YamlData<'a, Self, Vec<Self>, MAP, STR, FP>,
);

impl<MAP, STR, FP> Clone for Yaml<'_, MAP, STR, FP>
where
    MAP: Clone,
    STR: Clone,
    FP: Copy,
{
    fn clone(&self) -> Self {
        Yaml(self.0.clone())
    }
}

impl<'a, MAP, STR, FP> YamlDocAccess<'a> for Yaml<'a, MAP, STR, FP>
where
    MAP: Clone + IsEmpty,
    STR: Clone + for<'x> From<&'x str> + AsRef<str> + AsMut<str> + Into<String>,
    FP: Copy + AsRef<f64> + AsMut<f64>,
{
    type OutNode = Self;
    type SequenceNode = Vec<Self>;
    type MappingNode = MAP;

    fn key_from_usize(index: usize) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::Integer(index as i64)))
    }

    fn key_from_str(index: &str) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::String(index.into())))
    }

    fn is_non_empty_collection(&self) -> bool {
        match &self.0 {
            YamlData::Sequence(s) => !s.is_collection_empty(),
            YamlData::Mapping(m) => !m.is_collection_empty(),
            _ => false,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match &self.0 {
            YamlData::Scalar(YamlScalar::Bool(b)) => Some(*b),
            _ => None,
        }
    }

    fn as_bool_mut(&mut self) -> Option<&mut bool> {
        match &mut self.0 {
            YamlData::Scalar(YamlScalar::Bool(b)) => Some(b),
            _ => None,
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match &self.0 {
            YamlData::Scalar(YamlScalar::Integer(b)) => Some(*b),
            _ => None,
        }
    }

    fn as_i64_mut(&mut self) -> Option<&mut i64> {
        match &mut self.0 {
            YamlData::Scalar(YamlScalar::Integer(b)) => Some(b),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match &self.0 {
            YamlData::Scalar(YamlScalar::FloatingPoint(b)) => Some(*b.as_ref()),
            _ => None,
        }
    }

    fn as_f64_mut(&mut self) -> Option<&mut f64> {
        match &mut self.0 {
            YamlData::Scalar(YamlScalar::FloatingPoint(b)) => Some(b.as_mut()),
            _ => None,
        }
    }

    fn as_sequence(&self) -> Result<&Self::SequenceNode, YamlAccessError> {
        match &self.0 {
            YamlData::Sequence(s) => Ok(s),
            _ => Err(YamlAccessError::ExpectedSequence),
        }
    }

    fn as_sequence_mut(&mut self) -> Result<&mut Self::SequenceNode, YamlAccessError> {
        match &mut self.0 {
            YamlData::Sequence(s) => Ok(s),
            _ => Err(YamlAccessError::ExpectedSequence),
        }
    }

    fn as_mapping(&self) -> Result<&Self::MappingNode, YamlAccessError> {
        match &self.0 {
            YamlData::Mapping(s) => Ok(s),
            _ => Err(YamlAccessError::ExpectedMapping),
        }
    }

    fn as_mapping_mut(&mut self) -> Result<&mut Self::MappingNode, YamlAccessError> {
        match &mut self.0 {
            YamlData::Mapping(s) => Ok(s),
            _ => Err(YamlAccessError::ExpectedMapping),
        }
    }

    fn as_str(&self) -> Option<&str> {
        match &self.0 {
            YamlData::Scalar(YamlScalar::String(s)) => Some(s.as_ref()),
            _ => None,
        }
    }

    fn as_str_mut(&mut self) -> Option<&mut str> {
        match &mut self.0 {
            YamlData::Scalar(YamlScalar::String(s)) => Some(s.as_mut()),
            _ => None,
        }
    }

    fn sequence_mut(&mut self) -> &mut Self::SequenceNode {
        match &mut self.0 {
            YamlData::Sequence(s) => s,
            _ => core::panic!("YamlData::sequence_mut() called with non-sequence"),
        }
    }

    fn mapping_mut(&mut self) -> &mut Self::MappingNode {
        match &mut self.0 {
            YamlData::Mapping(m) => m,
            _ => core::panic!("YamlData::sequence_mut() called with non-mapping"),
        }
    }

    fn get_tag(&self) -> Option<Tag> {
        match &self.0 {
            YamlData::Tagged(tag, ..) => Some(tag.clone().into_owned()),
            _ => None,
        }
    }

    fn get_type(&self) -> NodeType {
        match &self.0 {
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
        match self.0 {
            YamlData::Scalar(YamlScalar::String(s)) => Some(s.into()),
            _ => None,
        }
    }

    fn into_mapping(self) -> Option<Self::MappingNode> {
        match self.0 {
            YamlData::Mapping(s) => Some(s),
            _ => None,
        }
    }

    fn into_sequence(self) -> Option<Self::SequenceNode> {
        match self.0 {
            YamlData::Sequence(s) => Some(s),
            _ => None,
        }
    }

    fn into_tagged(self, tag: Cow<'a, Tag>) -> Self {
        Yaml(YamlData::Tagged(tag, Box::new(self)))
    }

    fn bad_span_value(_span: Span) -> Self {
        Yaml(YamlData::BadValue)
    }
}

use crate::YamlDocAccess;
use crate::prelude::{NodeType, Span, Tag, YamlAccessError, YamlEntry};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;

pub enum YamlScalar<'a, S = String> {
    Null(&'a PhantomData<()>),
    String(S),
    Bool(bool),
    FloatingPoint(f64),
    Integer(i64),
    Alias(usize),
}

impl<S> Clone for YamlScalar<'_, S>
where
    S: Clone,
{
    fn clone(&self) -> Self {
        match self {
            YamlScalar::Null(a) => YamlScalar::Null(a),
            YamlScalar::String(s) => YamlScalar::String(s.clone()),
            YamlScalar::FloatingPoint(f) => YamlScalar::FloatingPoint(*f),
            YamlScalar::Bool(b) => YamlScalar::Bool(*b),
            YamlScalar::Integer(i) => YamlScalar::Integer(*i),
            YamlScalar::Alias(a) => YamlScalar::Alias(*a),
        }
    }
}

pub type OwnedScalar = YamlScalar<'static>;
pub type BorrowedScalar<'a> = YamlScalar<'a, Cow<'a, str>>;

pub enum YamlData<
    'input,
    Node,
    SEQ = Vec<Node>,
    MAP = Vec<YamlEntry<'input, Node>>,
    STR = Cow<'input, str>,
> {
    BadValue,
    Scalar(YamlScalar<'input, STR>),
    Sequence(SEQ),
    Mapping(MAP),
    Tagged(Cow<'input, Tag>, Box<Node>),
}

impl<Node, SEQ, MAP, STR> Clone for YamlData<'_, Node, SEQ, MAP, STR>
where
    Node: Clone,
    SEQ: Clone,
    MAP: Clone,
    STR: Clone,
{
    fn clone(&self) -> Self {
        match self {
            YamlData::BadValue => YamlData::BadValue,
            YamlData::Scalar(s) => YamlData::Scalar(s.clone()),
            YamlData::Sequence(s) => YamlData::Sequence(s.clone()),
            YamlData::Mapping(m) => YamlData::Mapping(m.clone()),
            YamlData::Tagged(tag, node) => YamlData::Tagged(tag.clone(), node.clone()),
        }
    }
}

impl<'a, SEQ, MAP> From<BorrowedScalar<'a>> for YamlData<'a, BorrowedScalar<'a>, SEQ, MAP> {
    fn from(value: BorrowedScalar<'a>) -> Self {
        YamlData::Scalar(value)
    }
}

pub struct SpannedYaml<'a, SEQ, MAP, STR = Cow<'a, str>> {
    span: Span,
    yaml: YamlData<'a, SpannedYaml<'a, SEQ, MAP, STR>, SEQ, MAP, STR>,
}

impl<'a, SEQ, MAP, STR> Clone for SpannedYaml<'a, SEQ, MAP, STR>
where
    SEQ: Clone,
    MAP: Clone,
    STR: Clone,
{
    fn clone(&self) -> Self {
        SpannedYaml {
            span: self.span,
            yaml: self.yaml.clone(),
        }
    }
}

trait IsEmpty {
    fn is_collection_empty(&self) -> bool;
}

impl<'a, SEQ, MAP, STR> YamlDocAccess<'a> for SpannedYaml<'a, SEQ, MAP, STR>
where
    SEQ: Clone + IsEmpty,
    MAP: Clone + IsEmpty,
    STR: Clone + for<'x> From<&'x str> + AsRef<str> + AsMut<str> + Into<String>,
{
    type OutNode = Self;
    type SequenceNode = SEQ;
    type MappingNode = MAP;

    fn key_from_usize(index: usize) -> Self {
        SpannedYaml {
            span: Span::default(),
            yaml: YamlData::Scalar(YamlScalar::Integer(index as i64)),
        }
    }

    fn key_from_str(index: &str) -> Self {
        SpannedYaml {
            span: Span::default(),
            yaml: YamlData::Scalar(YamlScalar::String(index.into())),
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
            YamlData::Scalar(YamlScalar::FloatingPoint(b)) => Some(*b),
            _ => None,
        }
    }

    fn as_f64_mut(&mut self) -> Option<&mut f64> {
        match &mut self.yaml {
            YamlData::Scalar(YamlScalar::FloatingPoint(b)) => Some(b),
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
            YamlData::Scalar(YamlScalar::String(s)) => Some(s.as_mut()),
            _ => None,
        }
    }

    fn sequence_mut(&mut self) -> &mut Self::SequenceNode {
        match &mut self.yaml {
            YamlData::Sequence(s) => s,
            _ => core::panic!("YamlData::sequence_mut() called with non-sequence"),
        }
    }

    fn mapping_mut(&mut self) -> &mut Self::MappingNode {
        match &mut self.yaml {
            YamlData::Mapping(m) => m,
            _ => core::panic!("YamlData::sequence_mut() called with non-mapping"),
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
            YamlData::Scalar(YamlScalar::Alias(_)) => NodeType::Alias,
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

use crate::node::yaml::Yaml;
use crate::prelude::{
    IsEmpty, NodeType, Span, Tag, ToMut, ToMutStr, YamlAccessError, YamlData, YamlDocAccess,
    YamlEntry, YamlScalar,
};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

impl<'a, FP, INT, STR> YamlDocAccess<'a> for Yaml<'a, FP, INT, STR>
where
    FP: Copy + ToMut<f64> + PartialEq,
    INT: Copy + ToMut<i64> + PartialEq,
    STR: Clone + ToMutStr<'a> + PartialEq,
{
    type OutNode = Self;
    type SequenceNode = Vec<Self>;
    type MappingNode = Vec<YamlEntry<'a, Self>>;

    #[inline]
    fn is_non_empty_collection(&self) -> bool {
        match &self.0 {
            YamlData::Sequence(s) => !s.is_collection_empty(),
            YamlData::Mapping(m) => !m.is_collection_empty(),
            _ => false,
        }
    }

    #[inline]
    fn is_tagged(&self) -> bool {
        matches!(self.0, YamlData::Tagged(..))
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
            YamlData::Scalar(YamlScalar::Integer(b)) => Some(b.as_owned()),
            _ => None,
        }
    }

    fn as_i64_mut(&mut self) -> Option<&mut i64> {
        match &mut self.0 {
            YamlData::Scalar(YamlScalar::Integer(b)) => Some(b.as_mut_val()),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match &self.0 {
            YamlData::Scalar(YamlScalar::FloatingPoint(b)) => Some(b.as_owned()),
            _ => None,
        }
    }

    fn as_f64_mut(&mut self) -> Option<&mut f64> {
        match &mut self.0 {
            YamlData::Scalar(YamlScalar::FloatingPoint(b)) => Some(b.as_mut_val()),
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
            YamlData::Scalar(YamlScalar::String(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    fn as_str_mut(&mut self) -> Option<&mut str> {
        match &mut self.0 {
            YamlData::Scalar(YamlScalar::String(s)) => Some(s.as_mut_str()),
            _ => None,
        }
    }

    unsafe fn sequence_mut(&mut self) -> &mut Self::SequenceNode {
        match &mut self.0 {
            YamlData::Sequence(s) => s,
            _ => unreachable!("YamlData::mapping_mut() on a non-sequence node"),
        }
    }

    unsafe fn sequence(&self) -> &Self::SequenceNode {
        match &self.0 {
            YamlData::Sequence(s) => s,
            _ => unreachable!("YamlData::sequence on a non-sequence node"),
        }
    }

    unsafe fn mapping_mut(&mut self) -> &mut Self::MappingNode {
        match &mut self.0 {
            YamlData::Mapping(m) => m,
            _ => unreachable!("YamlData::mapping_mut() called with non-mapping"),
        }
    }

    unsafe fn mapping(&self) -> &Self::MappingNode {
        match &self.0 {
            YamlData::Mapping(m) => m,
            _ => unreachable!("YamlData::mapping() called with non-mapping"),
        }
    }

    fn get_tag(&self) -> Option<Tag> {
        match &self.0 {
            YamlData::Tagged(tag, ..) => Some(tag.clone().into_owned()),
            _ => None,
        }
    }

    fn get_type(&self) -> NodeType {
        self.0.get_type()
    }

    fn into_string(self) -> Option<String> {
        match self.0 {
            YamlData::Scalar(YamlScalar::String(mut s)) => Some(s.as_mut_str().to_string()),
            _ => None,
        }
    }

    fn into_cow(self) -> Option<Cow<'a, str>> {
        match self.0 {
            YamlData::Scalar(YamlScalar::String(s)) => Some(s.into_cow()),
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

    fn bad_value() -> Self {
        Yaml(YamlData::BadValue)
    }

    #[inline]
    fn null() -> Self {
        Yaml(YamlData::Scalar(YamlScalar::Null(PhantomData)))
    }

    fn from_bool(b: bool) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::Bool(b)))
    }

    fn from_char(chr: char) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::String(STR::from_string(chr))))
    }

    fn from_string(input: impl Into<String>) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::String(STR::from_string(
            input,
        ))))
    }

    fn from_str(input: &'a str) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::String(STR::from_str(input))))
    }

    fn from_float(input: impl Into<f64>) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::FloatingPoint(FP::into_owned(
            input.into(),
        ))))
    }

    fn from_integer(int: impl Into<i64>) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::Integer(INT::into_owned(
            int.into(),
        ))))
    }
}

impl<'a, FP> From<YamlData<'a, Self, FP>> for Yaml<'a, FP> {
    fn from(value: YamlData<'a, Self, FP>) -> Self {
        Yaml(value)
    }
}

impl<'a, FP> From<YamlScalar<'a, FP>> for Yaml<'a, FP> {
    fn from(value: YamlScalar<'a, FP>) -> Self {
        Yaml(YamlData::Scalar(value))
    }
}

impl<'a, FP, INT, STR> Index<usize> for Yaml<'a, FP, INT, STR>
where
    FP: Copy + ToMut<f64> + PartialEq,
    INT: Copy + From<i64> + ToMut<i64> + PartialEq,
    STR: Clone + ToMutStr<'a> + PartialEq,
{
    type Output = Self;

    fn index(&self, index: usize) -> &Self::Output {
        let typ = self.0.get_type();
        let ind = Yaml::from_integer(i64::try_from(index).expect("Index less than i64::MAX"));
        match typ {
            // SAFETY: can call mapping on a Mapping node.
            NodeType::Mapping => unsafe {
                &self.mapping().iter().find(|x| x.key == ind).unwrap().value
            },
            // SAFETY: can call sequence on a Sequence node.
            NodeType::Sequence => unsafe { self.sequence().index(index) },
            _ => panic!("Expected Mapping and Sequence got {typ:?} instead"),
        }
    }
}

impl<'a, FP, INT, STR> IndexMut<usize> for Yaml<'a, FP, INT, STR>
where
    FP: Copy + ToMut<f64> + PartialEq,
    INT: Copy + From<i64> + ToMut<i64> + PartialEq,
    STR: Clone + ToMutStr<'a> + PartialEq,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let typ = self.get_type();
        let ind = Yaml::from_integer(i64::try_from(index).expect("Index less than i64::MAX"));
        match typ {
            // SAFETY: can call mapping_mut on a Mapping node.
            NodeType::Mapping => unsafe {
                &mut self
                    .mapping_mut()
                    .iter_mut()
                    .find(|x| x.key == ind)
                    .unwrap()
                    .key
            },
            // SAFETY: Can call sequence_mut on a Sequence node.
            NodeType::Sequence => unsafe { self.sequence_mut().index_mut(index) },
            _ => panic!("Expected Mapping and Sequence got {typ:?} instead"),
        }
    }
}

impl<'a, 'k, FP, INT, STR> Index<&'k str> for Yaml<'a, FP, INT, STR>
where
    FP: Copy + ToMut<f64> + PartialEq,
    INT: Copy + From<i64> + ToMut<i64> + PartialEq,
    STR: Clone + ToMutStr<'a> + PartialEq,
{
    type Output = Self;

    fn index(&self, index: &'k str) -> &Self::Output {
        let typ = self.get_type();
        let ind = Yaml::from_string(index);
        match typ {
            // SAFETY: Can call mapping on a Mapping node.
            NodeType::Mapping => unsafe {
                &self.mapping().iter().find(|x| x.key == ind).unwrap().value
            },
            _ => panic!("Expected Mapping and Sequence got {typ:?} instead"),
        }
    }
}

impl<'a, 'k, FP, INT, STR> IndexMut<&'k str> for Yaml<'a, FP, INT, STR>
where
    FP: Copy + ToMut<f64> + PartialEq,
    INT: Copy + From<i64> + ToMut<i64> + PartialEq,
    STR: Clone + ToMutStr<'a> + PartialEq,
{
    fn index_mut(&mut self, index: &'k str) -> &mut Self::Output {
        let typ = self.get_type();
        let ind = Yaml::from_string(index);
        match typ {
            // SAFETY: Can call mapping_mut on a Mapping node.
            NodeType::Mapping => unsafe {
                &mut self
                    .mapping_mut()
                    .iter_mut()
                    .find(|x| x.key == ind)
                    .unwrap()
                    .value
            },
            _ => panic!("Expected Mapping and Sequence got {typ:?} instead"),
        }
    }
}

// Equality comparators

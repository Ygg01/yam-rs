use crate::prelude::YamlScalar::Null;
use crate::prelude::{
    IsEmpty, NodeType, Span, Tag, ToMutStr, YamlAccessError, YamlData, YamlDocAccess, YamlEntry,
    YamlError, YamlLoader, YamlScalar,
};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

///
/// Basic borrowed YAML data structure.
///
/// The `Yaml` struct is a generic wrapper around `YamlData` with support for specifying
/// a custom floating-point type. By default, the floating-point type is set to `f64`.
///
/// # Type Parameters
/// - `'a`: The lifetime of the YAML data.
/// - `FP`: The floating-point type to be used within the YAML structure. Defaults to `f64`.
///
/// # Fields
/// - `0`: A public field containing the `YamlData` associated with this `Yaml` instance.
///
/// # Derives
/// - `PartialEq`: Enables equality and inequality comparisons for `Yaml`.
/// - `Debug`: Enables formatting the `Yaml` struct for debugging purposes.
///
/// # Examples
/// ```rust
/// use yam_core::node::YamlScalar;
/// use yam_core::prelude::{Yaml, YamlData};
///
/// // Example usage with default floating-point type (f64)
/// let yaml: Yaml = Yaml::from(3.1);
///
/// // Example usage with custom floating-point type
/// let yaml_custom: Yaml<f32> = Yaml(YamlData::Scalar(YamlScalar::FloatingPoint(2.3f32)));
/// ```
#[derive(PartialEq, Debug)]
pub struct Yaml<'a, FP = f64>(pub YamlData<'a, Self, FP>);

impl<'a, FP> Clone for Yaml<'a, FP>
where
    FP: Copy,
{
    fn clone(&self) -> Self {
        Yaml(self.0.clone())
    }
}

impl<'a, FP> YamlDocAccess<'a> for Yaml<'a, FP>
where
    FP: Copy + Borrow<f64> + BorrowMut<f64>,
{
    type OutNode = Self;
    type SequenceNode = Vec<Self>;
    type MappingNode = Vec<YamlEntry<'a, Self>>;

    fn key_from_usize(index: usize) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::Integer(index as i64)))
    }

    fn key_from_str(index: &str) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::String(Cow::Owned(
            index.to_string(),
        ))))
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
            YamlData::Scalar(YamlScalar::FloatingPoint(b)) => Some(*b.borrow()),
            _ => None,
        }
    }

    fn as_f64_mut(&mut self) -> Option<&mut f64> {
        match &mut self.0 {
            YamlData::Scalar(YamlScalar::FloatingPoint(b)) => Some((*b).borrow_mut()),
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
            YamlData::Scalar(YamlScalar::String(s)) => Some(s.mut_str()),
            _ => None,
        }
    }

    fn sequence_mut(&mut self) -> &mut Self::SequenceNode {
        match &mut self.0 {
            YamlData::Sequence(s) => s,
            _ => core::panic!("YamlData::sequence_mut() called with non-sequence"),
        }
    }

    fn sequence(&self) -> &Self::SequenceNode {
        match &self.0 {
            YamlData::Sequence(s) => s,
            _ => core::panic!("YamlData::sequence() called with non-sequence"),
        }
    }

    fn mapping_mut(&mut self) -> &mut Self::MappingNode {
        match &mut self.0 {
            YamlData::Mapping(m) => m,
            _ => core::panic!("YamlData::mapping_mut() called with non-mapping"),
        }
    }

    fn mapping(&self) -> &Self::MappingNode {
        match &self.0 {
            YamlData::Mapping(m) => m,
            _ => core::panic!("YamlData::mapping() called with non-mapping"),
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

    fn bad_value() -> Self {
        Yaml(YamlData::BadValue)
    }

    #[inline]
    fn null() -> Self {
        Yaml(YamlData::Scalar(Null(PhantomData)))
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

impl<'a> Yaml<'a> {
    pub fn load_from<S: AsRef<str>>(input: S) -> Result<Vec<Yaml<'a>>, YamlError> {
        YamlLoader::<Yaml<'a>>::load_from(input)
    }

    pub fn load_single<S: AsRef<str>>(input: S) -> Result<Yaml<'a>, YamlError> {
        YamlLoader::<Yaml<'a>>::load_single(input)
    }
}

impl<'a, FP> Index<usize> for Yaml<'a, FP>
where
    FP: Copy + Borrow<f64> + BorrowMut<f64> + PartialEq,
{
    type Output = Self;

    fn index(&self, index: usize) -> &Self::Output {
        let typ = self.0.get_type();
        let ind = Yaml::key_from_usize(index);
        match typ {
            NodeType::Mapping => &self.mapping().iter().find(|x| x.key == ind).unwrap().value,
            NodeType::Sequence => self.sequence().index(index),
            _ => panic!("Expected Mapping and Sequence got {0:?} instead", typ),
        }
    }
}

impl<'a, FP> IndexMut<usize> for Yaml<'a, FP>
where
    FP: Copy + Borrow<f64> + BorrowMut<f64> + PartialEq,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let typ = self.get_type();
        let ind = Yaml::key_from_usize(index);
        match typ {
            NodeType::Mapping => {
                &mut self
                    .mapping_mut()
                    .iter_mut()
                    .find(|x| x.key == ind)
                    .unwrap()
                    .key
            }
            NodeType::Sequence => self.sequence_mut().index_mut(index),
            _ => panic!("Expected Mapping and Sequence got {0:?} instead", typ),
        }
    }
}

impl<'a, 'k, FP> Index<&'k str> for Yaml<'a, FP>
where
    FP: Copy + Borrow<f64> + BorrowMut<f64> + PartialEq,
{
    type Output = Self;

    fn index(&self, index: &'k str) -> &Self::Output {
        let typ = self.get_type();
        let ind = Yaml::key_from_str(index);
        match typ {
            NodeType::Mapping => &self.mapping().iter().find(|x| x.key == ind).unwrap().value,
            _ => panic!("Expected Mapping and Sequence got {0:?} instead", typ),
        }
    }
}

impl<'a, 'k, FP> IndexMut<&'k str> for Yaml<'a, FP>
where
    FP: Copy + Borrow<f64> + BorrowMut<f64> + PartialEq,
{
    fn index_mut(&mut self, index: &'k str) -> &mut Self::Output {
        let typ = self.get_type();
        let ind = Yaml::key_from_str(index);
        match typ {
            NodeType::Mapping => {
                &mut self
                    .mapping_mut()
                    .iter_mut()
                    .find(|x| x.key == ind)
                    .unwrap()
                    .value
            }
            _ => panic!("Expected Mapping and Sequence got {0:?} instead", typ),
        }
    }
}

impl<'a> From<&'a str> for Yaml<'a> {
    fn from(value: &'a str) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::String(Cow::Borrowed(value))))
    }
}

impl<'a> From<bool> for Yaml<'a> {
    fn from(value: bool) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::Bool(value)))
    }
}

impl<'a> From<f64> for Yaml<'a> {
    fn from(value: f64) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::FloatingPoint(value)))
    }
}

impl<'a> From<f32> for Yaml<'a> {
    fn from(value: f32) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::FloatingPoint(value as f64)))
    }
}

impl<'a> From<i8> for Yaml<'a> {
    fn from(value: i8) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::Integer(value as i64)))
    }
}

impl<'a> From<i16> for Yaml<'a> {
    fn from(value: i16) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::Integer(value as i64)))
    }
}

impl<'a> From<i32> for Yaml<'a> {
    fn from(value: i32) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::Integer(value as i64)))
    }
}

impl<'a> From<i64> for Yaml<'a> {
    fn from(value: i64) -> Self {
        Yaml(YamlData::Scalar(YamlScalar::Integer(value)))
    }
}

impl<'a> From<Vec<Yaml<'a>>> for Yaml<'a> {
    fn from(value: Vec<Yaml<'a>>) -> Self {
        Yaml(YamlData::Sequence(value))
    }
}

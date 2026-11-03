use crate::prelude::{IsEmpty, Marker, NodeType, Span, Tag, ToMut, ToMutStr, YamlAccessError, YamlData, YamlDocAccess, YamlEntry, YamlScalar};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

///
/// A structure representing a YAML node with an associated source code span.
///
/// The `SpannedYaml` structure encapsulates a YAML node along with its
/// corresponding span information from the source. This allows for accurate
/// location tracing in the original source file, which can be useful for
/// error reporting or debugging purposes.
///
/// # Type Parameters
/// - `'a`: The lifetime associated with the YAML data. This ensures that
///   the `SpannedYaml` does not outlive the underlying YAML structure it references.
/// - `FP` (default: `f64`): The floating-point type used for representing
///   numerical data within the YAML. By default, this is `f64`, but it can
///   be customized for specialized use cases.
///
/// # Fields
/// - `span`: A `Span` representing the location of this YAML node
///   in the source code. The `Span` typically includes start and end
///   positions that enable precise error messages or parsing diagnostics.
/// - `yaml`: A `YamlData` instance that represents the structured data
///   of this YAML node. It is capable of holding information such as
///   mappings, sequences, scalars, and more. The `YamlData` is parameterized
///   to support recursive structures and type customization.
///
/// # Example Usage
/// ```rust
/// use yam_core::node::{SpannedYaml, YamlScalar};
/// use yam_core::prelude::{YamlData, Span, Marker};
///
/// let span = Span::new(Marker::new(0, 1, 1), Marker::new(5, 6, 1)); // Represents a span from 0 to 10 in the source.
/// let yaml_data = YamlData::Scalar(YamlScalar::Bool(false));
///
/// let spanned_yaml : SpannedYaml<'_> = SpannedYaml {
///     span,
///     yaml: yaml_data,
/// };
///
/// println!("Span: {:?}", spanned_yaml.span);
/// ```
///
/// This type is particularly useful when working with parsers or tools
/// that need to process YAML documents while keeping track of their
/// original source locations.
#[derive(PartialEq, Debug)]
pub struct SpannedYaml<'a, FP = f64, INT = i64, STR = Cow<'a, str>> {
    pub span: Span,
    pub yaml: YamlData<'a, SpannedYaml<'a, FP, INT, STR>, FP, INT, STR>,
}

impl<FP, INT, STR> Clone for SpannedYaml<'_, FP, INT, STR>
where
    FP: Copy,
    STR: Clone,
    INT: Copy,
{
    fn clone(&self) -> Self {
        SpannedYaml {
            span: self.span,
            yaml: self.yaml.clone(),
        }
    }
}

impl<'a, FP, INT> YamlDocAccess<'a> for SpannedYaml<'a, FP, INT>
where
    FP: Copy + ToMut<f64>,
    INT: Copy + From<i64> + ToMut<i64>,
{
    type OutNode = Self;
    type SequenceNode = Vec<Self>;
    type MappingNode = Vec<YamlEntry<'a, Self>>;

    #[allow(clippy::cast_lossless)]
    fn key_from_usize(index: usize) -> Self {
        SpannedYaml {
            span: Span::default(),
            yaml: YamlData::Scalar(YamlScalar::Integer(INT::from(index.cast_signed() as i64))),
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
            YamlData::Scalar(YamlScalar::Integer(b)) => Some(b.as_owned()),
            _ => None,
        }
    }

    fn as_i64_mut(&mut self) -> Option<&mut i64> {
        match &mut self.yaml {
            YamlData::Scalar(YamlScalar::Integer(b)) => Some(b.as_mut_val()),
            _ => None,
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match &self.yaml {
            YamlData::Scalar(YamlScalar::FloatingPoint(b)) => Some(b.as_owned()),
            _ => None,
        }
    }

    fn as_f64_mut(&mut self) -> Option<&mut f64> {
        match &mut self.yaml {
            YamlData::Scalar(YamlScalar::FloatingPoint(b)) => Some(b.as_mut_val()),
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
            _ => panic!("YamlData::sequence_mut() called with non-sequence"),
        }
    }

    fn sequence(&self) -> &Self::SequenceNode {
        match &self.yaml {
            YamlData::Sequence(s) => s,
            _ => panic!("YamlData::sequence() called with non-sequence"),
        }
    }

    fn mapping_mut(&mut self) -> &mut Self::MappingNode {
        match &mut self.yaml {
            YamlData::Mapping(m) => m,
            _ => panic!("YamlData::mapping_mut() called with non-mapping"),
        }
    }

    fn mapping(&self) -> &Self::MappingNode {
        match &self.yaml {
            YamlData::Mapping(m) => m,
            _ => panic!("YamlData::mapping() called with non-mapping"),
        }
    }

    fn get_tag(&self) -> Option<Tag> {
        match &self.yaml {
            YamlData::Tagged(tag, ..) => Some(tag.clone().into_owned()),
            _ => None,
        }
    }

    fn get_type(&self) -> NodeType {
        self.yaml.get_type()
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

    fn null() -> Self {
        SpannedYaml {
            yaml: YamlData::Scalar(YamlScalar::Null(PhantomData)),
            span: Span::default(),
        }
    }

    fn with_start(self, marker: Marker) -> Self {
        let mut span = self.span;
        span.start = marker;
        SpannedYaml {
            yaml: self.yaml,
            span,
        }
    }

    fn with_end(self, marker: Marker) -> Self {
        let mut span = self.span;
        span.end = marker;
        SpannedYaml {
            yaml: self.yaml,
            span,
        }
    }

    fn with_span(self, span: Span) -> Self {
        SpannedYaml {
            yaml: self.yaml,
            span,
        }
    }
}

impl<'a, FP, INT, STR> From<YamlData<'a, Self, FP, INT, STR>> for SpannedYaml<'a, FP, INT, STR> {
    fn from(value: YamlData<'a, Self, FP, INT, STR>) -> Self {
        SpannedYaml {
            yaml: value,
            span: Span::default(),
        }
    }
}

impl<'a, FP, INT, STR> From<YamlScalar<'a, FP, INT, STR>> for SpannedYaml<'a, FP, INT, STR> {
    fn from(value: YamlScalar<'a, FP, INT, STR>) -> Self {
        SpannedYaml {
            yaml: YamlData::Scalar(value),
            span: Span::default(),
        }
    }
}

impl<FP, INT> Index<usize> for SpannedYaml<'_, FP, INT>
where
    FP: Copy + ToMut<f64> + PartialEq,
    INT: Copy + From<i64> + ToMut<i64> + PartialEq,
{
    type Output = Self;

    fn index(&self, index: usize) -> &Self::Output {
        let typ = self.yaml.get_type();
        let ind = SpannedYaml::key_from_usize(index);
        match typ {
            NodeType::Mapping => &self
                .mapping()
                .iter()
                .find(|x| x.key.yaml == ind.yaml)
                .unwrap().value,
            NodeType::Sequence => self.sequence().index(index),
            _ => panic!("Expected Mapping and Sequence got {typ:?} instead"),
        }
    }
}

impl<FP, INT> IndexMut<usize> for SpannedYaml<'_, FP, INT>
where
    FP: Copy + ToMut<f64> + PartialEq,
    INT: Copy + From<i64> + ToMut<i64> + PartialEq,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let typ = self.get_type();
        let ind = SpannedYaml::key_from_usize(index);
        match typ {
            NodeType::Mapping => {
                &mut self
                    .mapping_mut()
                    .iter_mut()
                    .find(|x| x.key.yaml == ind.yaml)
                    .unwrap()
                    .key
            }
            NodeType::Sequence => self.sequence_mut().index_mut(index),
            _ => panic!("Expected Mapping and Sequence got {typ:?} instead"),
        }
    }
}

impl<'k, FP, INT> Index<&'k str> for SpannedYaml<'_, FP, INT>
where
    FP: Copy + ToMut<f64> + PartialEq,
    INT: Copy + From<i64> + ToMut<i64> + PartialEq,
{
    type Output = Self;

    fn index(&self, index: &'k str) -> &Self::Output {
        let typ = self.get_type();
        let ind = SpannedYaml::key_from_str(index);
        match typ {
            NodeType::Mapping => &self
                .mapping()
                .iter()
                .find(|x| x.key.yaml == ind.yaml)
                .unwrap().value,
            _ => panic!("Expected Mapping and Sequence got {typ:?} instead"),
        }
    }
}

impl<'a> From<&'a str> for SpannedYaml<'a> {
    fn from(value: &'a str) -> Self {
        SpannedYaml {
            yaml: YamlData::Scalar(YamlScalar::String(Cow::Borrowed(value))),
            span: Span::default(),
        }
    }
}

impl<FP, INT, STR> From<bool> for SpannedYaml<'_, FP, INT, STR> {
    fn from(value: bool) -> Self {
        SpannedYaml {
            yaml: YamlData::Scalar(YamlScalar::Bool(value)),
            span: Span::default(),
        }
    }
}

impl From<f64> for SpannedYaml<'_> {
    fn from(value: f64) -> Self {
        SpannedYaml {
            yaml: YamlData::Scalar(YamlScalar::FloatingPoint(value)),
            span: Span::default(),
        }
    }
}

#[allow(clippy::cast_lossless)]
impl From<f32> for SpannedYaml<'_> {
    fn from(value: f32) -> Self {
        SpannedYaml {
            yaml: YamlData::Scalar(YamlScalar::FloatingPoint(value as f64)),
            span: Span::default(),
        }
    }
}

#[allow(clippy::cast_lossless)]
impl From<i8> for SpannedYaml<'_> {
    fn from(value: i8) -> Self {
        SpannedYaml {
            yaml: YamlData::Scalar(YamlScalar::Integer(value as i64)),
            span: Span::default(),
        }
    }
}

#[allow(clippy::cast_lossless)]
impl From<i16> for SpannedYaml<'_> {
    fn from(value: i16) -> Self {
        SpannedYaml {
            yaml: YamlData::Scalar(YamlScalar::Integer(value as i64)),
            span: Span::default(),
        }
    }
}

#[allow(clippy::cast_lossless)]
impl From<i32> for SpannedYaml<'_> {
    fn from(value: i32) -> Self {
        SpannedYaml {
            yaml: YamlData::Scalar(YamlScalar::Integer(value as i64)),
            span: Span::default(),
        }
    }
}

impl From<i64> for SpannedYaml<'_> {
    fn from(value: i64) -> Self {
        SpannedYaml {
            yaml: YamlData::Scalar(YamlScalar::Integer(value)),
            span: Span::default(),
        }
    }
}

impl<'a> From<Vec<SpannedYaml<'a>>> for SpannedYaml<'a> {
    fn from(value: Vec<SpannedYaml<'a>>) -> Self {
        SpannedYaml {
            yaml: YamlData::Sequence(value),
            span: Span::default(),
        }
    }
}

impl From<Vec<f64>> for SpannedYaml<'_> {
    fn from(value: Vec<f64>) -> Self {
        let sequence = value.into_iter().map(SpannedYaml::from).collect();
        SpannedYaml {
            yaml: YamlData::Sequence(sequence),
            span: Span::default(),
        }
    }
}

impl From<Vec<f32>> for SpannedYaml<'_> {
    fn from(value: Vec<f32>) -> Self {
        let sequence = value.into_iter().map(SpannedYaml::from).collect();
        SpannedYaml {
            yaml: YamlData::Sequence(sequence),
            span: Span::default(),
        }
    }
}

impl From<Vec<i64>> for SpannedYaml<'_> {
    fn from(value: Vec<i64>) -> Self {
        let sequence = value.into_iter().map(SpannedYaml::from).collect();
        SpannedYaml {
            yaml: YamlData::Sequence(sequence),
            span: Span::default(),
        }
    }
}

impl From<Vec<i32>> for SpannedYaml<'_> {
    fn from(value: Vec<i32>) -> Self {
        let sequence = value.into_iter().map(SpannedYaml::from).collect();
        SpannedYaml {
            yaml: YamlData::Sequence(sequence),
            span: Span::default(),
        }
    }
}

impl From<Vec<i16>> for SpannedYaml<'_> {
    fn from(value: Vec<i16>) -> Self {
        let sequence = value.into_iter().map(SpannedYaml::from).collect();
        SpannedYaml {
            yaml: YamlData::Sequence(sequence),
            span: Span::default(),
        }
    }
}
impl From<Vec<i8>> for SpannedYaml<'_> {
    fn from(value: Vec<i8>) -> Self {
        let sequence = value.into_iter().map(SpannedYaml::from).collect();
        SpannedYaml {
            yaml: YamlData::Sequence(sequence),
            span: Span::default(),
        }
    }
}

impl<'a> From<Vec<&'a str>> for SpannedYaml<'a> {
    fn from(value: Vec<&'a str>) -> Self {
        let sequence = value.into_iter().map(SpannedYaml::from).collect();
        SpannedYaml {
            yaml: YamlData::Sequence(sequence),
            span: Span::default(),
        }
    }
}

impl<'a, K, V> From<Vec<(K, V)>> for SpannedYaml<'a>
where
    K: Into<SpannedYaml<'a>>,
    V: Into<SpannedYaml<'a>>,
{
    fn from(value: Vec<(K, V)>) -> Self {
        let sequence = value
            .into_iter()
            .map(|(key, value)| YamlEntry::new(key.into(), value.into()))
            .collect();
        SpannedYaml {
            yaml: YamlData::Mapping(sequence),
            span: Span::default(),
        }
    }
}

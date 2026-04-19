use crate::prelude::{
    IsEmpty, NodeType, Span, Tag, ToMutStr, YamlAccessError, YamlData, YamlDocAccess, YamlEntry,
    YamlScalar,
};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::marker::PhantomData;

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
pub struct SpannedYaml<'a, FP = f64> {
    pub span: Span,
    pub yaml: YamlData<'a, SpannedYaml<'a, FP>, FP>,
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
            span: Default::default(),
        }
    }
}

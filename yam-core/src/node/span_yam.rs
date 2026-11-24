use crate::prelude::{
    IsEmpty, Marker, NodeType, Span, Tag, ToMut, ToMutStr, YamLoader, YamlAccessError, YamlData,
    YamlDocAccess, YamlEntry, YamlError, YamlScalar,
};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;
use core::ops::{Index, IndexMut};

///
/// A structure representing a YAML node with an associated source code span.
///
/// The [`SpanYam`] structure encapsulates a YAML node along with its
/// corresponding span information from the source. This allows for accurate
/// location tracing in the original source file, which can be useful for
/// error reporting or debugging purposes.
///
/// # Type Parameters
/// - `'a`: The lifetime associated with the YAML data. This ensures that
///   the [`SpanYam`] does not outlive the underlying YAML structure it references.
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
/// use yam_core::node::{SpanYam, YamlScalar};
/// use yam_core::prelude::{YamlData, Span, Marker};
///
/// let span = Span::new(Marker::new(0, 1, 1), Marker::new(5, 6, 1)); // Represents a span from 0 to 10 in the source.
/// let yaml_data = YamlData::Scalar(YamlScalar::Bool(false));
///
/// let span_yaml : SpanYam<'_> = SpanYam {
///     span,
///     yaml: yaml_data,
/// };
///
/// println!("Span: {:?}", span_yaml.span);
/// ```
///
/// This type is particularly useful when working with parsers or tools
/// that need to process YAML documents while keeping track of their
/// original source locations.
#[derive(PartialEq, Debug)]
pub struct SpanYam<'a, FP = f64, INT = i64, STR = Cow<'a, str>> {
    pub span: Span,
    pub yaml: YamlData<'a, SpanYam<'a, FP, INT, STR>, FP, INT, STR>,
}

impl<'a> SpanYam<'a> {
    ///
    /// Loads a vector of YAML document from the given input string.
    ///
    /// # Type Parameters
    /// - `S`: A type that can be converted into a string slice (`&str`)
    ///   using the `AsRef<str>` trait, typically `String` or `&str`.
    ///
    /// # Parameters
    /// - `input`: The YAML input as a string-like object. This is the source from
    ///   which the function attempts to parse a single YAML document.
    ///
    /// # Returns
    /// - `Ok(Node)`: On success, returns the first YAML document (`Node`) parsed from the input string.
    /// - `Err(YamlError)`: Returns an error if:
    ///   - The input is invalid or cannot be parsed due to syntax or structural issues.
    ///   - No YAML document is found within the input (`YamlError::NoDocument`).
    ///
    /// # Errors
    /// - `YamlError::NoDocument`: Returned if no valid YAML documents are found in the input.
    /// - Any other error encountered during parsing, as propagated from the parser.
    ///
    /// # Example
    /// ```
    /// use yam_core::prelude::{SpanYam, YamLoader};
    ///
    /// let yaml_input = r#"
    /// key: value
    /// array:
    ///   - item1
    ///   - item2
    /// "#;
    ///
    /// match SpanYam::load_from(&yaml_input) {
    ///     Ok(node) => println!("Parsed Node: {node:?}"),
    ///     Err(e) => eprintln!("Error loading YAML: {e}"),
    /// }
    /// ```
    ///
    ///
    pub fn load_from<S: AsRef<str>>(input: S) -> Result<Vec<SpanYam<'a>>, YamlError> {
        YamLoader::<SpanYam<'a>>::load_from(input)
    }

    ///
    /// Loads a single YAML document from the given input string.
    ///
    /// # Type Parameters
    /// - `S`: A type that can be converted into a string slice (`&str`)
    ///   using the `AsRef<str>` trait, typically `String` or `&str`.
    ///
    /// # Parameters
    /// - `input`: The YAML input as a string-like object. This is the source from
    ///   which the function attempts to parse a single YAML document.
    ///
    /// # Returns
    /// - `Ok(Node)`: On success, returns the first YAML document (`Node`) parsed from the input string.
    /// - `Err(YamlError)`: Returns an error if:
    ///   - The input is invalid or cannot be parsed due to syntax or structural issues.
    ///   - No YAML document is found within the input (`YamlError::NoDocument`).
    ///
    /// # Errors
    /// - `YamlError::NoDocument`: Returned if no valid YAML documents are found in the input.
    /// - Any other error encountered during parsing, as propagated from the parser.
    ///
    /// # Example
    /// ```
    /// use yam_core::prelude::{SpanYam, YamLoader};
    ///
    /// let yaml_input = r#"
    /// key: value
    /// array:
    ///   - item1
    ///   - item2
    /// "#;
    ///
    /// match SpanYam::load_single(&yaml_input) {
    ///     Ok(node) => println!("Parsed Node: {:?}", node),
    ///     Err(e) => eprintln!("Error loading YAML: {}", e),
    /// }
    /// ```
    ///
    /// # Notes
    /// This function expects only a single YAML document in the input. Any additional
    /// documents beyond the first will be ignored.
    ///
    pub fn load_single<S: AsRef<str>>(input: S) -> Result<SpanYam<'a>, YamlError> {
        YamLoader::<SpanYam<'a>>::load_single(input)
    }
}

impl<FP, INT, STR> Clone for SpanYam<'_, FP, INT, STR>
where
    FP: Copy,
    STR: Clone,
    INT: Copy,
{
    fn clone(&self) -> Self {
        SpanYam {
            span: self.span,
            yaml: self.yaml.clone(),
        }
    }
}

impl<'a, FP, INT, STR> YamlDocAccess<'a> for SpanYam<'a, FP, INT, STR>
where
    FP: Copy + ToMut<f64>,
    INT: Copy + ToMut<i64>,
    STR: Clone + ToMutStr<'a>,
{
    type OutNode = Self;
    type SequenceNode = Vec<Self>;
    type MappingNode = Vec<YamlEntry<'a, Self>>;

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
            YamlData::Scalar(YamlScalar::String(s)) => Some(s.as_str()),
            _ => None,
        }
    }

    fn as_str_mut(&mut self) -> Option<&mut str> {
        match &mut self.yaml {
            YamlData::Scalar(YamlScalar::String(s)) => Some(s.as_mut_str()),
            _ => None,
        }
    }

    unsafe fn sequence_mut(&mut self) -> &mut Self::SequenceNode {
        match &mut self.yaml {
            YamlData::Sequence(m) => m,
            _ => unreachable!("YamlData::sequence_mut() called with non-mapping"),
        }
    }

    unsafe fn sequence(&self) -> &Self::SequenceNode {
        match &self.yaml {
            YamlData::Sequence(m) => m,
            _ => unreachable!("YamlData::sequence_mut() called with non-mapping"),
        }
    }

    unsafe fn mapping_mut(&mut self) -> &mut Self::MappingNode {
        match &mut self.yaml {
            YamlData::Mapping(m) => m,
            _ => unreachable!("YamlData::mapping_mut() called with non-mapping"),
        }
    }

    unsafe fn mapping(&self) -> &Self::MappingNode {
        match &self.yaml {
            YamlData::Mapping(m) => m,
            _ => unreachable!("YamlData::mapping() called with non-mapping"),
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
            YamlData::Scalar(YamlScalar::String(s)) => Some(s.into_string()),
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
        SpanYam {
            span: self.span,
            yaml: YamlData::Tagged(tag, Box::new(self)),
        }
    }

    fn bad_span_value(span: Span) -> Self {
        SpanYam {
            span,
            yaml: YamlData::BadValue,
        }
    }

    fn null() -> Self {
        SpanYam {
            yaml: YamlData::Scalar(YamlScalar::Null(PhantomData)),
            span: Span::default(),
        }
    }

    fn from_bool(b: bool) -> Self {
        SpanYam {
            yaml: YamlData::Scalar(YamlScalar::Bool(b)),
            span: Span::default(),
        }
    }

    fn from_char(chr: char) -> Self {
        SpanYam {
            yaml: YamlData::Scalar(YamlScalar::String(STR::from_string(chr))),
            span: Span::default(),
        }
    }

    fn from_integer(int: impl Into<i64>) -> Self {
        SpanYam {
            yaml: YamlData::Scalar(YamlScalar::Integer(INT::into_owned(int.into()))),
            span: Span::default(),
        }
    }

    fn from_string(input: impl Into<String>) -> Self {
        SpanYam {
            yaml: YamlData::Scalar(YamlScalar::String(STR::from_string(input))),
            span: Span::default(),
        }
    }

    fn from_str(input: &'a str) -> Self {
        SpanYam {
            yaml: YamlData::Scalar(YamlScalar::String(STR::from_str(input))),
            span: Span::default(),
        }
    }

    fn from_float(input: impl Into<f64>) -> Self {
        SpanYam {
            yaml: YamlData::Scalar(YamlScalar::FloatingPoint(FP::into_owned(input.into()))),
            span: Span::default(),
        }
    }

    fn with_start(self, marker: Marker) -> Self {
        let mut span = self.span;
        span.start = marker;
        SpanYam {
            yaml: self.yaml,
            span,
        }
    }

    fn with_end(self, marker: Marker) -> Self {
        let mut span = self.span;
        span.end = marker;
        SpanYam {
            yaml: self.yaml,
            span,
        }
    }

    fn with_span(self, span: Span) -> Self {
        SpanYam {
            yaml: self.yaml,
            span,
        }
    }
}

impl<'a, FP, INT, STR> From<YamlData<'a, Self, FP, INT, STR>> for SpanYam<'a, FP, INT, STR> {
    fn from(value: YamlData<'a, Self, FP, INT, STR>) -> Self {
        SpanYam {
            yaml: value,
            span: Span::default(),
        }
    }
}

impl<'a, FP, INT, STR> From<YamlScalar<'a, FP, INT, STR>> for SpanYam<'a, FP, INT, STR> {
    fn from(value: YamlScalar<'a, FP, INT, STR>) -> Self {
        SpanYam {
            yaml: YamlData::Scalar(value),
            span: Span::default(),
        }
    }
}

impl<FP, INT> Index<usize> for SpanYam<'_, FP, INT>
where
    FP: Copy + ToMut<f64> + PartialEq,
    INT: Copy + From<i64> + ToMut<i64> + PartialEq,
{
    type Output = Self;

    fn index(&self, index: usize) -> &Self::Output {
        let typ = self.yaml.get_type();
        let ind = SpanYam::from_integer(i64::try_from(index).expect("Index lesser than i64::MAX"));
        match typ {
            NodeType::Mapping => unsafe {
                &self
                    .mapping()
                    .iter()
                    .find(|x| x.key.yaml == ind.yaml)
                    .unwrap()
                    .value
            },
            NodeType::Sequence => unsafe { self.sequence().index(index) },
            _ => panic!("Expected Mapping and Sequence got {typ:?} instead"),
        }
    }
}

impl<FP, INT> IndexMut<usize> for SpanYam<'_, FP, INT>
where
    FP: Copy + ToMut<f64> + PartialEq,
    INT: Copy + From<i64> + ToMut<i64> + PartialEq,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let typ = self.get_type();
        let ind = SpanYam::from_integer(i64::try_from(index).expect("Index lesser than i64::MAX"));
        match typ {
            NodeType::Mapping => unsafe {
                &mut self
                    .mapping_mut()
                    .iter_mut()
                    .find(|x| x.key.yaml == ind.yaml)
                    .unwrap()
                    .key
            },
            NodeType::Sequence => unsafe { self.sequence_mut().index_mut(index) },
            _ => panic!("Expected Mapping and Sequence got {typ:?} instead"),
        }
    }
}

impl<'k, FP, INT> Index<&'k str> for SpanYam<'_, FP, INT>
where
    FP: Copy + ToMut<f64> + PartialEq,
    INT: Copy + From<i64> + ToMut<i64> + PartialEq,
{
    type Output = Self;

    fn index(&self, index: &'k str) -> &Self::Output {
        let typ = self.get_type();
        let ind = SpanYam::from_str(index);
        match typ {
            NodeType::Mapping => unsafe {
                &self
                    .mapping()
                    .iter()
                    .find(|x| x.key.yaml == ind.yaml)
                    .unwrap()
                    .value
            },
            _ => panic!("Expected Mapping and Sequence got {typ:?} instead"),
        }
    }
}

impl<'k, FP, INT> IndexMut<&'k str> for SpanYam<'_, FP, INT>
where
    FP: Copy + ToMut<f64> + PartialEq,
    INT: Copy + From<i64> + ToMut<i64> + PartialEq,
{
    fn index_mut(&mut self, index: &'k str) -> &mut Self::Output {
        let typ = self.get_type();
        let ind = SpanYam::from_str(index);
        match typ {
            NodeType::Mapping => unsafe {
                &mut self
                    .mapping_mut()
                    .iter_mut()
                    .find(|x| x.key.yaml == ind.yaml)
                    .unwrap()
                    .value
            },
            _ => panic!("Expected Mapping and Sequence got {typ:?} instead"),
        }
    }
}

impl<'a> From<&'a str> for SpanYam<'a> {
    fn from(value: &'a str) -> Self {
        SpanYam {
            yaml: YamlData::Scalar(YamlScalar::String(Cow::Borrowed(value))),
            span: Span::default(),
        }
    }
}

impl From<bool> for SpanYam<'_> {
    fn from(value: bool) -> Self {
        SpanYam {
            yaml: YamlData::Scalar(YamlScalar::Bool(value)),
            span: Span::default(),
        }
    }
}

impl From<f64> for SpanYam<'_> {
    fn from(value: f64) -> Self {
        SpanYam {
            yaml: YamlData::Scalar(YamlScalar::FloatingPoint(value)),
            span: Span::default(),
        }
    }
}

#[allow(clippy::cast_lossless)]
impl From<f32> for SpanYam<'_> {
    fn from(value: f32) -> Self {
        SpanYam {
            yaml: YamlData::Scalar(YamlScalar::FloatingPoint(value as f64)),
            span: Span::default(),
        }
    }
}

#[allow(clippy::cast_lossless)]
impl From<i8> for SpanYam<'_> {
    fn from(value: i8) -> Self {
        SpanYam {
            yaml: YamlData::Scalar(YamlScalar::Integer(value as i64)),
            span: Span::default(),
        }
    }
}

#[allow(clippy::cast_lossless)]
impl From<i16> for SpanYam<'_> {
    fn from(value: i16) -> Self {
        SpanYam {
            yaml: YamlData::Scalar(YamlScalar::Integer(value as i64)),
            span: Span::default(),
        }
    }
}

#[allow(clippy::cast_lossless)]
impl From<i32> for SpanYam<'_> {
    fn from(value: i32) -> Self {
        SpanYam {
            yaml: YamlData::Scalar(YamlScalar::Integer(value as i64)),
            span: Span::default(),
        }
    }
}

impl From<i64> for SpanYam<'_> {
    fn from(value: i64) -> Self {
        SpanYam {
            yaml: YamlData::Scalar(YamlScalar::Integer(value)),
            span: Span::default(),
        }
    }
}

impl From<Vec<bool>> for SpanYam<'_> {
    fn from(value: Vec<bool>) -> Self {
        let sequence = value.into_iter().map(SpanYam::from).collect();
        SpanYam {
            yaml: YamlData::Sequence(sequence),
            span: Span::default(),
        }
    }
}

impl<'a> From<Vec<SpanYam<'a>>> for SpanYam<'a> {
    fn from(value: Vec<SpanYam<'a>>) -> Self {
        SpanYam {
            yaml: YamlData::Sequence(value),
            span: Span::default(),
        }
    }
}

impl From<Vec<f64>> for SpanYam<'_> {
    fn from(value: Vec<f64>) -> Self {
        let sequence = value.into_iter().map(SpanYam::from).collect();
        SpanYam {
            yaml: YamlData::Sequence(sequence),
            span: Span::default(),
        }
    }
}

impl From<Vec<f32>> for SpanYam<'_> {
    fn from(value: Vec<f32>) -> Self {
        let sequence = value.into_iter().map(SpanYam::from).collect();
        SpanYam {
            yaml: YamlData::Sequence(sequence),
            span: Span::default(),
        }
    }
}

impl From<Vec<i64>> for SpanYam<'_> {
    fn from(value: Vec<i64>) -> Self {
        let sequence = value.into_iter().map(SpanYam::from).collect();
        SpanYam {
            yaml: YamlData::Sequence(sequence),
            span: Span::default(),
        }
    }
}

impl From<Vec<i32>> for SpanYam<'_> {
    fn from(value: Vec<i32>) -> Self {
        let sequence = value.into_iter().map(SpanYam::from).collect();
        SpanYam {
            yaml: YamlData::Sequence(sequence),
            span: Span::default(),
        }
    }
}

impl From<Vec<i16>> for SpanYam<'_> {
    fn from(value: Vec<i16>) -> Self {
        let sequence = value.into_iter().map(SpanYam::from).collect();
        SpanYam {
            yaml: YamlData::Sequence(sequence),
            span: Span::default(),
        }
    }
}
impl From<Vec<i8>> for SpanYam<'_> {
    fn from(value: Vec<i8>) -> Self {
        let sequence = value.into_iter().map(SpanYam::from).collect();
        SpanYam {
            yaml: YamlData::Sequence(sequence),
            span: Span::default(),
        }
    }
}

impl<'a> From<Vec<&'a str>> for SpanYam<'a> {
    fn from(value: Vec<&'a str>) -> Self {
        let sequence = value.into_iter().map(SpanYam::from).collect();
        SpanYam {
            yaml: YamlData::Sequence(sequence),
            span: Span::default(),
        }
    }
}

impl<'a, K, V> From<Vec<(K, V)>> for SpanYam<'a>
where
    K: Into<SpanYam<'a>>,
    V: Into<SpanYam<'a>>,
{
    fn from(value: Vec<(K, V)>) -> Self {
        let sequence = value
            .into_iter()
            .map(|(key, value)| YamlEntry::new(key.into(), value.into()))
            .collect();
        SpanYam {
            yaml: YamlData::Mapping(sequence),
            span: Span::default(),
        }
    }
}

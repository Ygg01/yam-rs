use crate::prelude::{NodeType, ScalarType, Tag, YamlEntry, YamlScalar};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::vec::Vec;

/// Basic data structure used as backbone for all YAML nodes
///
/// # Type Parameters
/// `'input`: Lifetime of the underlying string data
/// `NODE`: Node being nested inside Maps or Sequences.
/// `FP` (default: f64): Floating point type used for representing numerical data, within YAML.
///  by default, this is `f64` but it can be customized for special cases like if you want `f32` or an
///  `OrderedFloat`.
/// `STR` (default `Cow<'input, str>`): Type of string scalar used.
#[derive(Debug)]
pub enum YamlData<'input, NODE, FP = f64, STR = Cow<'input, str>> {
    /// Bad value encountered during parsing or construction
    BadValue,
    /// Scalar value found during parsing. See [`YamlScalar`].
    Scalar(YamlScalar<'input, FP, STR>),
    /// Sequence of nodes.
    Sequence(Vec<NODE>),
    /// Set of key-value pairs.
    Mapping(Vec<YamlEntry<'input, NODE>>),
    /// Node tagged with a [`Tag`] value.
    Tagged(Cow<'input, Tag>, Box<NODE>),
    /// Alias to another node in the document.
    Alias(usize),
}

impl<'a, NODE, FP, STR> PartialEq for YamlData<'a, NODE, FP, STR>
where
    NODE: PartialEq,
    FP: PartialEq,
    STR: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (YamlData::BadValue, YamlData::BadValue) => true,
            (YamlData::Scalar(s1), YamlData::Scalar(s2)) => s1 == s2,
            (YamlData::Sequence(s1), YamlData::Sequence(s2)) => s1 == s2,
            (YamlData::Mapping(s1), YamlData::Mapping(s2)) => s1 == s2,
            (YamlData::Tagged(t1, b1), YamlData::Tagged(t2, b2)) => t1 == t2 && b1 == b2,
            (YamlData::Alias(a1), YamlData::Alias(a2)) => a1 == a2,
            (_, _) => false,
        }
    }
}

impl<'input, Node, FP> From<YamlScalar<'input, FP>> for YamlData<'input, Node, FP> {
    fn from(value: YamlScalar<'input, FP>) -> Self {
        YamlData::Scalar(value)
    }
}

impl<'a, Node, FP, STR> YamlData<'a, Node, FP, STR> {
    #[inline]
    pub(crate) fn get_type(&self) -> NodeType {
        match &self {
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
}

impl<'input, Node, FP> YamlData<'input, Node, FP>
where
    Node: From<YamlData<'input, Node, FP>>,
    FP: From<f64>,
{
    pub(crate) fn value_from_cow_and_metadata(
        v: Cow<'input, str>,
        style: ScalarType,
        tag: Option<&Cow<'input, Tag>>,
    ) -> Self {
        match tag {
            Some(tag) if !tag.is_yaml_core_schema() => Self::Tagged(
                tag.clone(),
                Box::new(Self::value_from_cow_and_metadata(v, style, None).into()),
            ),
            _ => YamlScalar::parse_from_cow_and_metadata(v, style, tag)
                .map_or(Self::BadValue, |x| YamlData::Scalar(x)),
        }
    }
}

impl<'a, Node, FP, STR> Clone for YamlData<'a, Node, FP, STR>
where
    Node: Clone,
    FP: Copy,
    STR: Clone,
{
    fn clone(&self) -> Self {
        match self {
            YamlData::Alias(a) => YamlData::Alias(*a),
            YamlData::BadValue => YamlData::BadValue,
            YamlData::Scalar(s) => YamlData::Scalar(s.clone()),
            YamlData::Sequence(s) => YamlData::Sequence(s.clone()),
            YamlData::Mapping(m) => YamlData::Mapping(m.clone()),
            YamlData::Tagged(tag, node) => YamlData::Tagged(tag.clone(), node.clone()),
        }
    }
}

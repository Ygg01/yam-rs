use crate::YamlDocAccess;
use crate::prelude::{IsEmpty, NodeType, ScalarType, Span, Tag, YamlAccessError, YamlEntry};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;

pub enum YamlScalar<'a, S = String, F = f64> {
    Null(PhantomData<&'a ()>),
    String(S),
    Bool(bool),
    FloatingPoint(F),
    Integer(i64),
}

impl<'a, S, F> YamlScalar<'a, S, F>
where
    S: From<Cow<'a, str>>,
    F: From<f64>,
{
    /// Parse a scalar node representation into a [`Scalar`].
    ///
    /// If `tag` is not [`None`]:
    ///   - If the handle is `tag:yaml.org,2022:`, attempt to parse as the given suffix. If parsing
    ///     fails or the suffix is unknown, return [`None`].
    ///   - If the handle is unknown, use the fallback parsing schema.
    ///
    /// # Return
    /// Returns the parsed [`Scalar`].
    ///
    pub fn parse_from_cow_and_metadata(
        v: Cow<'a, str>,
        style: ScalarType,
        tag: Option<&Cow<'a, Tag>>,
    ) -> Option<Self> {
        if style != ScalarType::Plain {
            // Any quoted scalar is a string.
            Some(Self::String(v.into()))
        } else if let Some(tag) = tag.map(Cow::as_ref) {
            if tag.is_yaml_core_schema() {
                match tag.suffix.as_ref() {
                    "bool" => v.parse::<bool>().ok().map(|x| Self::Bool(x)),
                    "int" => v.parse::<i64>().ok().map(|x| Self::Integer(x)),
                    "float" => parse_core_schema_fp(&v).map(|x| Self::FloatingPoint(x.into())),
                    "null" => match v.as_ref() {
                        "~" | "null" => Some(Self::Null(PhantomData::default())),
                        _ => None,
                    },
                    "str" => Some(Self::String(v.into())),
                    // If we have a tag we do not recognize, return `None`.
                    _ => None,
                }
            } else {
                // If we have a tag we do not recognize, parse it regularly.
                // This will sound more intuitive when instance reading tagged scalars like
                // `!degree 50`.
                Some(Self::parse_from_cow(v))
            }
        } else {
            // No tag means we have to guess.
            Some(Self::parse_from_cow(v))
        }
    }

    /// Parse a scalar node representation into a [`Scalar`].
    ///
    /// This function cannot fail. It will fallback to [`Scalar::String`] if everything else fails.
    ///
    /// # Return
    /// Returns the parsed [`Scalar`].
    #[must_use]
    pub fn parse_from_cow(v: Cow<'a, str>) -> Self {
        let s = &*v;
        let bytes = s.as_bytes();

        if bytes.len() >= 2 {
            match (bytes[0], bytes[1]) {
                (b'0', b'x') => {
                    if let Ok(i) = i64::from_str_radix(&s[2..], 16) {
                        return Self::Integer(i);
                    }
                }
                (b'0', b'o') => {
                    if let Ok(i) = i64::from_str_radix(&s[2..], 8) {
                        return Self::Integer(i);
                    }
                }
                (b'+', _) => {
                    if let Ok(i) = s[1..].parse::<i64>() {
                        return Self::Integer(i);
                    }
                }
                _ => {}
            }
        }

        match bytes.len() {
            1 if bytes[0] == b'~' => return Self::Null(PhantomData::default()),
            4 => {
                let f = bytes[0] & 0xDF;
                if f == b'N' && matches!(s, "null" | "Null" | "NULL") {
                    return Self::Null(PhantomData::default());
                } else if f == b'T' && matches!(s, "true" | "True" | "TRUE") {
                    return Self::Bool(true);
                }
            }
            5 if matches!(s, "false" | "False" | "FALSE") => {
                return Self::Bool(false);
            }
            _ => {}
        }

        if let Ok(integer) = s.parse::<i64>() {
            return Self::Integer(integer);
        }

        if let Some(float) = parse_core_schema_fp(s) {
            return Self::FloatingPoint(float.into());
        }

        Self::String(v.into())
    }
}

/// Parse the given string as a floating point according to the core schema.
///
/// See [10.2.1.4](https://yaml.org/spec/1.2.2/#10214-floating-point) for the floating point
/// definition.
///
/// # Return
/// Returns `Some` if parsing succeeding, `None` otherwise. This function is used in the process of
/// parsing scalars, where failing to parse a scalar as a floating point is not an error. As such,
/// this function purposefully does not return a `Result`.
pub fn parse_core_schema_fp(v: &str) -> Option<f64> {
    match v {
        ".inf" | ".Inf" | ".INF" | "+.inf" | "+.Inf" | "+.INF" => Some(f64::INFINITY),
        "-.inf" | "-.Inf" | "-.INF" => Some(f64::NEG_INFINITY),
        ".nan" | ".NaN" | ".NAN" => Some(f64::NAN),
        // Test that `v` contains a digit so as not to pass in strings like `inf`,
        // which rust will parse as a float.
        _ if v.as_bytes().iter().any(u8::is_ascii_digit) => v.parse::<f64>().ok(),
        _ => None,
    }
}

impl<S, F> Clone for YamlScalar<'_, S, F>
where
    S: Clone,
    F: Copy,
{
    fn clone(&self) -> Self {
        match self {
            YamlScalar::Null(a) => YamlScalar::Null(*a),
            YamlScalar::String(s) => YamlScalar::String(s.clone()),
            YamlScalar::FloatingPoint(f) => YamlScalar::FloatingPoint(*f),
            YamlScalar::Bool(b) => YamlScalar::Bool(*b),
            YamlScalar::Integer(i) => YamlScalar::Integer(*i),
        }
    }
}
pub type BorrowedScalar<'a> = YamlScalar<'a, Cow<'a, str>, f64>;

pub enum YamlData<
    'input,
    Node,
    SEQ = Vec<Node>,
    MAP = Vec<YamlEntry<'input, Node>>,
    STR = Cow<'input, str>,
    FP = f64,
> {
    BadValue,
    Scalar(YamlScalar<'input, STR, FP>),
    Sequence(SEQ),
    Mapping(MAP),
    Tagged(Cow<'input, Tag>, Box<Node>),
    Alias(usize),
}

impl<'input, Node, SEQ, MAP, STR, FP> From<YamlScalar<'input, STR, FP>>
    for YamlData<'input, Node, SEQ, MAP, STR, FP>
{
    fn from(value: YamlScalar<'input, STR, FP>) -> Self {
        YamlData::Scalar(value)
    }
}

impl<'input, Node, SEQ, MAP, STR, FP> YamlData<'input, Node, SEQ, MAP, STR, FP>
where
    Node: From<YamlData<'input, Node, SEQ, MAP, STR, FP>> + From<YamlScalar<'input, STR, FP>>,
    STR: From<Cow<'input, str>>,
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

impl<Node, SEQ, MAP, STR, FP> Clone for YamlData<'_, Node, SEQ, MAP, STR, FP>
where
    Node: Clone,
    SEQ: Clone,
    MAP: Clone,
    STR: Clone,
    FP: Copy,
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

pub struct SpannedYaml<'a, SEQ, MAP, STR = Cow<'a, str>, FP = f64> {
    span: Span,
    yaml: YamlData<'a, SpannedYaml<'a, SEQ, MAP, STR, FP>, SEQ, MAP, STR, FP>,
}

impl<'a, SEQ, MAP, STR, FP> Clone for SpannedYaml<'a, SEQ, MAP, STR, FP>
where
    SEQ: Clone,
    MAP: Clone,
    STR: Clone,
    FP: Copy,
{
    fn clone(&self) -> Self {
        SpannedYaml {
            span: self.span,
            yaml: self.yaml.clone(),
        }
    }
}

impl<'a, SEQ, MAP, STR, FP> YamlDocAccess<'a> for SpannedYaml<'a, SEQ, MAP, STR, FP>
where
    SEQ: Clone + IsEmpty,
    MAP: Clone + IsEmpty,
    STR: Clone + for<'x> From<&'x str> + AsRef<str> + AsMut<str> + Into<String>,
    FP: Copy + AsRef<f64> + AsMut<f64>,
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
            YamlData::Scalar(YamlScalar::FloatingPoint(b)) => Some(*b.as_ref()),
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

pub struct Yaml<'a, SEQ, MAP, STR = Cow<'a, str>, FP = f64>(
    pub YamlData<'a, Self, SEQ, MAP, STR, FP>,
);

impl<'a, SEQ, MAP, STR, FP> Clone for Yaml<'a, SEQ, MAP, STR, FP>
where
    SEQ: Clone,
    MAP: Clone,
    STR: Clone,
    FP: Copy,
{
    fn clone(&self) -> Self {
        Yaml(self.0.clone())
    }
}

impl<'a, SEQ, MAP, STR, FP> YamlDocAccess<'a> for Yaml<'a, SEQ, MAP, STR, FP>
where
    SEQ: Clone + IsEmpty,
    MAP: Clone + IsEmpty,
    STR: Clone + for<'x> From<&'x str> + AsRef<str> + AsMut<str> + Into<String>,
    FP: Copy + AsRef<f64> + AsMut<f64>,
{
    type OutNode = Self;
    type SequenceNode = SEQ;
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

use crate::{Marker, ScalarType, Span, Tag};
use std::borrow::Cow;
use std::marker::PhantomData;
use std::mem;

pub trait LoadableYamlNode<'input>: Clone + PartialEq {
    #[must_use]
    fn into_tagged(self, tag: Cow<'input, Tag>) -> Self;

    fn from_bare_yaml(yaml: YamlDoc<'input>) -> Self;

    fn sequence_mut(&mut self) -> &mut Vec<Self>;
    fn mapping_mut(&mut self) -> &mut Vec<YamlEntry<'input, Self>>;

    fn bad(_: Span) -> Self {
        Self::bad_value()
    }

    fn bad_value() -> Self;

    fn is_sequence(&self) -> bool;

    fn is_mapping(&self) -> bool;

    fn is_bad_value(&self) -> bool;

    fn take(&mut self) -> Self;

    fn is_non_empty_collection(&self) -> bool;

    fn is_collection(&self) -> bool {
        self.is_mapping() || self.is_sequence()
    }
    fn with_start(self, _: Marker) -> Self {
        self
    }

    fn with_end(self, _: Marker) -> Self {
        self
    }
}

impl<'input> LoadableYamlNode<'input> for YamlDoc<'input> {
    fn into_tagged(self, tag: Cow<'input, Tag>) -> Self {
        Self::Tagged(tag, Box::new(self))
    }

    fn from_bare_yaml(yaml: YamlDoc<'input>) -> Self {
        yaml
    }

    fn sequence_mut(&mut self) -> &mut Vec<Self> {
        match self {
            YamlDoc::Sequence(seq) => seq,
            _ => core::panic!("Expected sequence got {:?}", self),
        }
    }

    fn mapping_mut(&mut self) -> &mut Vec<YamlEntry<'input, Self>> {
        match self {
            YamlDoc::Mapping(map) => map,
            _ => core::panic!("Expected sequence got {:?}", self),
        }
    }

    fn bad_value() -> Self {
        YamlDoc::BadValue
    }

    fn is_sequence(&self) -> bool {
        matches!(self, YamlDoc::Sequence(_))
    }

    fn is_mapping(&self) -> bool {
        matches!(self, YamlDoc::Mapping(_))
    }

    fn is_bad_value(&self) -> bool {
        matches!(self, YamlDoc::BadValue)
    }

    fn take(&mut self) -> Self {
        mem::replace(self, YamlDoc::BadValue)
    }

    fn is_non_empty_collection(&self) -> bool {
        match self {
            YamlDoc::Sequence(x) => !x.is_empty(),
            YamlDoc::Mapping(x) => !x.is_empty(),
            _ => false,
        }
    }
}

/// Ordered sequence of one or more [`YamlDoc`]'s
pub type Sequence<'a> = Vec<YamlDoc<'a>>;

/// Sequence of key-value pairing of two [`YamlDoc`]s
pub type Mapping<'a> = Vec<YamlEntry<'a, YamlDoc<'a>>>;

#[derive(Debug, Default, Clone, PartialEq)]
pub enum YamlDoc<'input> {
    #[default]
    BadValue,
    Null,
    String(Cow<'input, str>),
    Bool(bool),
    FloatingPoint(f64),
    Integer(i64),
    // flow style like `[x, x, x]`
    // or block style like:
    //     - x
    //     - x
    Sequence(Sequence<'input>),

    // flow style like `{x: Y, a: B}`
    // or block style like:
    //     x: Y
    //     a: B
    Mapping(Mapping<'input>),
    Alias(usize),
    Tagged(Cow<'input, Tag>, Box<YamlDoc<'input>>),
}

impl<'input> YamlDoc<'input> {
    pub fn from_cow_and_tag(
        value: Cow<'input, str>,
        scalar_type: ScalarType,
        tag: &Option<Cow<'input, Tag>>,
    ) -> YamlDoc<'input> {
        if scalar_type != ScalarType::Plain {
            return Self::String(value);
        }
        if let Some(tag) = tag
            && tag.is_yaml_core_schema()
        {
            return match &*tag.suffix {
                "bool" => parse_bool(value),
                "int" => value
                    .parse()
                    .ok()
                    .map(YamlDoc::Integer)
                    .unwrap_or(YamlDoc::BadValue),
                "null" => parse_null(value),
                "float" => parse_float(&value)
                    .map(YamlDoc::FloatingPoint)
                    .unwrap_or(YamlDoc::BadValue),
                _ => YamlDoc::BadValue,
            };
        }
        Self::parse_from_cow(value)
    }

    #[must_use]
    fn parse_from_cow(value: Cow<str>) -> YamlDoc {
        let bytes = value.as_bytes();
        let str_v = &*value;
        let early_check = match bytes {
            b"null" | b"~" => Some(YamlDoc::Null),
            b"true" | b"True" | b"TRUE" => Some(YamlDoc::Bool(true)),
            b"false" | b"False" | b"FALSE" => Some(YamlDoc::Bool(false)),
            _ => None,
        };
        if let Some(x) = early_check {
            return x;
        };

        match bytes {
            [b'0', b'x', ..] => {
                if let Ok(x) = i64::from_str_radix(&str_v[2..], 16) {
                    return YamlDoc::Integer(x);
                }
            }
            [b'0', b'o', ..] => {
                if let Ok(x) = i64::from_str_radix(&str_v[2..], 8) {
                    return YamlDoc::Integer(x);
                }
            }
            _ => {}
        }

        if let Ok(integer) = value.parse::<i64>() {
            return YamlDoc::Integer(integer);
        }

        if let Some(float) = parse_float(&value) {
            return YamlDoc::FloatingPoint(float);
        }

        YamlDoc::String(value)
    }
}

fn parse_bool(v: Cow<str>) -> YamlDoc {
    match v.as_bytes() {
        b"true" | b"True" | b"TRUE" => YamlDoc::Bool(true),
        b"false" | b"False" | b"FALSE" => YamlDoc::Bool(false),
        _ => YamlDoc::BadValue,
    }
}

fn parse_null(v: Cow<str>) -> YamlDoc {
    match v.as_bytes() {
        b"~" | b"null" => YamlDoc::Null,
        _ => YamlDoc::BadValue,
    }
}

fn parse_float(v: &str) -> Option<f64> {
    match v.as_bytes() {
        b".inf" | b".Inf" | b".INF" | b"+.inf" | b"+.Inf" | b"+.INF" => Some(f64::INFINITY),
        b"-.inf" | b"-.Inf" | b"-.INF" => Some(f64::NEG_INFINITY),
        b".nan" | b".NaN" | b".NAN" => Some(f64::NAN),
        // Test that `v` contains a digit so as not to pass in strings like `inf`,
        // which rust will parse as a float.
        _ => v.parse::<f64>().ok(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct YamlEntry<'input, T>
where
    T: Clone,
{
    pub key: T,
    pub value: T,
    pub(crate) _marker: PhantomData<&'input ()>,
}

impl<'input, T: Clone> YamlEntry<'input, T> {
    pub fn new(a: T, b: T) -> Self {
        YamlEntry {
            key: a,
            value: b,
            _marker: PhantomData,
        }
    }
}

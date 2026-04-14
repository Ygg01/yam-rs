use crate::prelude::{ScalarType, Tag, YamlEntry, YamlScalar};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::vec::Vec;

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

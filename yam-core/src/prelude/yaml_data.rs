use crate::prelude::{ScalarType, Tag, YamlEntry, YamlScalar};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::vec::Vec;

pub enum YamlData<'input, NODE, FP = f64, STR = Cow<'input, str>> {
    BadValue,
    Scalar(YamlScalar<'input, FP, STR>),
    Sequence(Vec<NODE>),
    Mapping(Vec<YamlEntry<'input, NODE>>),
    Tagged(Cow<'input, Tag>, Box<NODE>),
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

impl<Node, FP> Clone for YamlData<'_, Node, FP>
where
    Node: Clone,
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

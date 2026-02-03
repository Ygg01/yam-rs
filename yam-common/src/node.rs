use crate::{Mapping, Sequence, Tag, YamlDoc, YamlEntry};
use std::borrow::Cow;

#[derive(Debug, Default, PartialEq, Clone)]
pub enum YamlCloneNode<'input, Node: Clone> {
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
    Sequence(Vec<Node>),

    // flow style like `{x: Y, a: B}`
    // or block style like:
    //     x: Y
    //     a: B
    Mapping(Vec<YamlEntry<'input, Node>>),
    Alias(usize),
    Tagged(Cow<'input, Tag>, Box<Node>),
}

impl<'input, T: Clone> From<YamlDoc<'input>> for YamlCloneNode<'input, T>
where
    T: From<YamlDoc<'input>>,
{
    fn from(value: YamlDoc<'input>) -> Self {
        match value {
            YamlDoc::BadValue => YamlCloneNode::BadValue,
            YamlDoc::Null => YamlCloneNode::Null,
            YamlDoc::String(x) => YamlCloneNode::String(x),
            YamlDoc::Bool(x) => YamlCloneNode::Bool(x),
            YamlDoc::Alias(x) => YamlCloneNode::Alias(x),
            YamlDoc::FloatingPoint(x) => YamlCloneNode::FloatingPoint(x),
            YamlDoc::Integer(x) => YamlCloneNode::Integer(x),
            YamlDoc::Sequence(s) => YamlCloneNode::from_sequence(s),
            YamlDoc::Mapping(m) => YamlCloneNode::from_mapping(m),
            YamlDoc::Tagged(tag, data) => YamlCloneNode::Tagged(tag, Box::new((*data).into())),
        }
    }
}

impl<'input, T> YamlCloneNode<'input, T>
where
    T: From<YamlDoc<'input>> + Clone,
{
    fn from_sequence(sequence: Sequence<'input>) -> YamlCloneNode<'input, T> {
        YamlCloneNode::Sequence(sequence.into_iter().map(|x| x.into()).collect())
    }

    fn from_mapping(mapping: Mapping<'input>) -> YamlCloneNode<'input, T> {
        YamlCloneNode::Mapping(
            mapping
                .into_iter()
                .map(|x| YamlEntry::new(x.key.into(), x.value.into()))
                .collect(),
        )
    }
}

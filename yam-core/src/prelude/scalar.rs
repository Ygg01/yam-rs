use crate::prelude::{Span, Tag, YamlEntry};
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;

pub enum YamlScalar<'a, S = String, F = f64, I = i64> {
    Null(&'a PhantomData<()>),
    String(S),
    Bool(bool),
    FloatingPoint(F),
    Integer(I),
    Alias(usize),
}

impl<'a, S, F, I> Clone for YamlScalar<'a, S, F, I>
where
    S: Clone,
    F: Clone,
    I: Clone,
{
    fn clone(&self) -> Self {
        match self {
            YamlScalar::Null(a) => YamlScalar::Null(a.clone()),
            YamlScalar::String(s) => YamlScalar::String(s.clone()),
            YamlScalar::FloatingPoint(f) => YamlScalar::FloatingPoint(f.clone()),
            YamlScalar::Bool(f) => YamlScalar::Bool(*f),
            YamlScalar::Integer(f) => YamlScalar::Integer(f.clone()),
            YamlScalar::Alias(a) => YamlScalar::Alias(*a),
        }
    }
}

pub type OwnedScalar = YamlScalar<'static>;
pub type BorrowedScalar<'a> = YamlScalar<'a, Cow<'a, str>>;

pub enum YamlData<'input, Node, SEQ = Vec<Node>, MAP = Vec<YamlEntry<'input, Node>>> {
    BadValue,
    Scalar(YamlScalar<'input>),
    Sequence(SEQ),
    Mapping(MAP),
    Tagged(Cow<'input, Tag>, Box<Node>),
}

impl<'input, Node> Clone for YamlData<'input, Node>
where
    Node: Clone,
{
    fn clone(&self) -> Self {
        match self {
            YamlData::BadValue => YamlData::BadValue,
            YamlData::Scalar(s) => YamlData::Scalar(s.clone()),
            YamlData::Sequence(s) => YamlData::Sequence(s.to_vec()),
            YamlData::Mapping(m) => YamlData::Mapping(m.to_vec()),
            YamlData::Tagged(tag, node) => YamlData::Tagged(tag.clone(), node.clone()),
        }
    }
}

pub struct SpannedYaml<'a> {
    span: Span,
    yaml: YamlData<'a, SpannedYaml<'a>>,
}

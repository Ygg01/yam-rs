use crate::parsing::ScalarValue;
use alloc::boxed::Box;
use alloc::vec::Vec;

pub enum LazyExpander<'input> {
    Scalar(ScalarValue<'input>),
    Vector(Box<LazyExpanderVec<'input>>),
    Map(Box<LazyExpanderMap<'input>>),
}

pub trait Expanable {
    fn calculate_limit(&self) -> usize;
    fn can_expand(&self, limit: usize) -> bool {
        limit > self.calculate_limit()
    }
}

impl Expanable for LazyExpander<'_> {
    fn calculate_limit(&self) -> usize {
        match self {
            LazyExpander::Scalar(_) => 1,
            LazyExpander::Vector(vec) => vec.calculate_limit(),
            LazyExpander::Map(map) => map.calculate_limit(),
        }
    }
}

pub struct LazyExpanderVec<'input> {
    pub inner: Vec<LazyExpander<'input>>,
}

impl Expanable for LazyExpanderVec<'_> {
    fn calculate_limit(&self) -> usize {
        self.inner
            .iter()
            .fold(0, |acc, expansion| acc + expansion.calculate_limit())
    }
}

pub struct LazyExpanderMap<'input> {
    inner: Vec<(LazyExpander<'input>, LazyExpander<'input>)>,
}

impl Expanable for LazyExpanderMap<'_> {
    fn calculate_limit(&self) -> usize {
        self.inner.iter().fold(0, |acc, expansion| {
            acc + expansion.0.calculate_limit() + expansion.1.calculate_limit()
        })
    }
}

use crate::saphyr_tokenizer::Source;
use crate::treebuild::YamlLoader;
use crate::{Parser, Span};
use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::hash::Hash;
use yam_common::{Marker, Tag, YamlDoc, YamlError};

pub trait LoadableYamlNode<'input>: Clone + Hash + Eq {
    fn load_from_parser<I: Source>(parser: &mut Parser<'input, I>) -> Result<Vec<Self>, YamlError> {
        let mut loader = YamlLoader::default();
        parser.load(&mut loader, true)?;
        Ok(loader.into_documents())
    }

    fn into_tagged(self, tag: Cow<Tag>) -> Self;

    fn from_bare_yaml(yaml: YamlDoc) -> Self;

    fn with_start(self, _: Marker) -> Self {
        self
    }

    fn with_end(self, _: Marker) -> Self {
        self
    }

    fn bad(span: Span) -> Self;
}

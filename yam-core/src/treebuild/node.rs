use crate::Span;
use alloc::borrow::Cow;
use alloc::vec::Vec;
use core::hash::Hash;
use yam_common::{Marker, Tag, YamlDoc, YamlEntry};

pub trait LoadableYamlNode<'input>: Clone + Hash + Eq {
    #[must_use]
    fn into_tagged(self, tag: Cow<Tag>) -> Self;

    fn from_bare_yaml(yaml: YamlDoc) -> Self;

    fn sequence_mut(&mut self) -> &mut Vec<Self>;
    fn mapping_mut(&mut self) -> &mut Vec<YamlEntry<'_, Self>>;

    fn bad(span: Span) -> Self;

    fn bad_value() -> Self;

    fn is_sequence(&self) -> bool;

    fn is_mapping(&self) -> bool;

    fn is_bad_value(&self) -> bool;

    fn take(&mut self) -> Self;

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

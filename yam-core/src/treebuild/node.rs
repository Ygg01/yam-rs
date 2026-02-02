use crate::Span;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem;
use yam_common::{Marker, Tag, YamlDoc, YamlEntry};

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
            _ => panic!("Expected sequence got {:?}", self),
        }
    }

    fn mapping_mut(&mut self) -> &mut Vec<YamlEntry<'input, Self>> {
        match self {
            YamlDoc::Mapping(map) => map,
            _ => panic!("Expected sequence got {:?}", self),
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
}

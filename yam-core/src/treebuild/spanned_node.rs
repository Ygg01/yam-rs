use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::vec::Vec;
use yam_common::{Span, Tag, YamlDoc, YamlEntry};

use yam_common::loader::LoadableYamlNode;
use yam_common::node::YamlCloneNode;

#[derive(PartialEq, Clone, Default)]
pub struct SpannedYaml<'input> {
    pub data: YamlCloneNode<'input, SpannedYaml<'input>>,
    pub span: Span,
}

impl<'input> From<YamlDoc<'input>> for SpannedYaml<'input> {
    fn from(yaml_data: YamlDoc<'input>) -> Self {
        SpannedYaml {
            data: yaml_data.into(),
            span: Span::default(),
        }
    }
}

impl<'input> LoadableYamlNode<'input> for SpannedYaml<'input> {
    fn into_tagged(self, tag: Cow<'input, Tag>) -> Self {
        SpannedYaml {
            data: YamlCloneNode::Tagged(tag, Box::new(self)),
            span: Span::default(),
        }
    }

    fn from_bare_yaml(yaml: YamlDoc<'input>) -> Self {
        SpannedYaml {
            data: yaml.into(),
            span: Span::default(),
        }
    }

    fn sequence_mut(&mut self) -> &mut Vec<Self> {
        match self.data {
            YamlCloneNode::Sequence(ref mut s) => s,
            _ => panic!("Cannot get sequence_mut for non-sequence data"),
        }
    }

    fn mapping_mut(&mut self) -> &mut Vec<YamlEntry<'input, Self>> {
        match self.data {
            YamlCloneNode::Mapping(ref mut s) => s,
            _ => panic!("Cannot get mapping_mut for non-mappingdata data"),
        }
    }

    fn bad_value() -> Self {
        SpannedYaml {
            data: YamlCloneNode::BadValue,
            span: Span::default(),
        }
    }

    fn is_sequence(&self) -> bool {
        matches!(self.data, YamlCloneNode::Sequence(_))
    }

    fn is_mapping(&self) -> bool {
        matches!(self.data, YamlCloneNode::Mapping(_))
    }

    fn is_bad_value(&self) -> bool {
        matches!(self.data, YamlCloneNode::BadValue)
    }

    fn take(&mut self) -> Self {
        core::mem::take(self)
    }

    fn is_non_empty_collection(&self) -> bool {
        match self.data {
            YamlCloneNode::Sequence(ref s) => !s.is_empty(),
            YamlCloneNode::Mapping(ref s) => !s.is_empty(),
            _ => false,
        }
    }
}

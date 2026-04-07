use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use yam_common::{Span, Tag, YamlDoc, YamlDocAccess, YamlEntry};

use yam_common::LoadableYamlNode;
use yam_common::YamlCloneNode;

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

impl<'input> YamlDocAccess<'input> for SpannedYaml<'input> {
    type Node = SpannedYaml<'input>;

    fn is_bad_value(&self) -> bool {
        todo!()
    }

    fn is_null(&self) -> bool {
        todo!()
    }

    fn is_string(&self) -> bool {
        todo!()
    }

    fn is_bool(&self) -> bool {
        todo!()
    }

    fn is_floating_point(&self) -> bool {
        todo!()
    }

    fn is_integer(&self) -> bool {
        todo!()
    }

    fn is_alias(&self) -> bool {
        todo!()
    }

    fn is_non_empty_collection(&self) -> bool {
        todo!()
    }

    fn is_mapping(&self) -> bool {
        todo!()
    }

    fn is_sequence(&self) -> bool {
        todo!()
    }

    fn as_bool(&self) -> Option<bool> {
        todo!()
    }

    fn as_bool_mut(&mut self) -> Option<&mut bool> {
        todo!()
    }

    fn as_i64(&self) -> Option<i64> {
        todo!()
    }

    fn as_i64_mut(&mut self) -> Option<&mut i64> {
        todo!()
    }

    fn as_f64(&self) -> Option<f64> {
        todo!()
    }

    fn as_f64_mut(&mut self) -> Option<&mut f64> {
        todo!()
    }

    fn as_sequence(&self) -> Option<&yam_common::NodeSequence<Self::Node>> {
        todo!()
    }

    fn as_sequence_mut(&mut self) -> Option<&mut yam_common::NodeSequence<Self::Node>> {
        todo!()
    }

    fn as_mapping(&self) -> Option<&yam_common::NodeMapping<'input, Self::Node>> {
        todo!()
    }

    fn as_mapping_mut(&mut self) -> Option<&yam_common::NodeMapping<'input, Self::Node>> {
        todo!()
    }

    fn as_str(&self) -> Option<&str> {
        todo!()
    }

    fn as_str_mut(&mut self) -> Option<&mut str> {
        todo!()
    }

    fn get_tag(&self) -> Option<Tag> {
        todo!()
    }

    fn into_bool(self) -> Option<bool> {
        todo!()
    }

    fn into_string(self) -> Option<String> {
        todo!()
    }

    fn into_cow(self) -> Option<Cow<'input, str>> {
        todo!()
    }

    fn into_f64(self) -> Option<f64> {
        todo!()
    }

    fn into_i64(self) -> Option<i64> {
        todo!()
    }

    fn into_mapping(self) -> Option<yam_common::NodeMapping<'input, Self::Node>> {
        todo!()
    }

    fn into_sequence(self) -> Option<yam_common::NodeSequence<Self::Node>> {
        todo!()
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

    fn take(&mut self) -> Self {
        core::mem::take(self)
    }
}

use crate::node::YamlData;
use alloc::string::String;

#[derive(PartialEq, Debug)]
pub struct YamlOwned(pub YamlData<'static, Self, f64, i64, String>);

impl Clone for YamlOwned {
    fn clone(&self) -> Self {
        match &self.0 {
            YamlData::BadValue => YamlOwned(YamlData::BadValue),
            YamlData::Scalar(s) => YamlOwned(YamlData::Scalar(s.clone())),
            YamlData::Sequence(s) => YamlOwned(YamlData::Sequence(s.clone())),
            YamlData::Mapping(m) => YamlOwned(YamlData::Mapping(m.clone())),
            YamlData::Tagged(a, b) => YamlOwned(YamlData::Tagged(a.clone(), b.clone())),
            YamlData::Alias(a) => YamlOwned(YamlData::Alias(*a)),
        }
    }
}

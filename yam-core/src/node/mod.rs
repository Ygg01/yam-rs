//! Import this module to use various `yam_core` nodes.
pub use scalar::YamlScalar;
pub use spanned_yaml::SpannedYaml;
pub use yaml::Yaml;
pub use yaml_data::YamlData;

pub(crate) mod scalar;
pub(crate) mod spanned_yaml;
pub(crate) mod yaml;
pub(crate) mod yaml_data;
pub mod yaml_owned;

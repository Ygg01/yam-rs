//! Import this module to use various `yam_core` nodes.
pub use scalar::YamlScalar;
pub use scalar::parse_i64_from_cow;
pub use span_yam::SpanYam;
pub use yaml::Yaml;
pub use yaml_data::YamlData;

pub(crate) mod scalar;
pub(crate) mod span_yam;
pub(crate) mod yaml;
pub(crate) mod yaml_data;

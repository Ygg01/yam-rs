use crate::prelude::ScalarType;
use crate::serde::NullFormat;
use crate::serde::ser::{FlowStyle, YamlStyle};
use alloc::borrow::Cow;

#[cfg(feature = "serde")]
#[derive(Debug, PartialEq)]
pub struct YamlFormatter {
    /// Limit depth
    pub block_depth_limit: u32,

    /// Preferred string length
    pub pref_string_length: u32,

    /// Preferred indent length
    indent_len: u32,

    /// How to format null
    pub(crate) null_format: Cow<'static, str>,

    /// How to format a string in block style
    pub block_preferred_style: ScalarType,

    /// How to format a string in a root
    pub root_preferred_style: ScalarType,

    /// How to format a string in block style
    pub flow_string_style: FlowStyle,

    pub key_preferred_style: ScalarType,

    /// Whether to prefer string to fit in a single line
    pub compact_strings: bool,

    /// What is the default style for the Root element
    pub root_style: YamlStyle,
}

impl Default for YamlFormatter {
    fn default() -> Self {
        Self {
            block_depth_limit: 0,
            pref_string_length: 80,
            indent_len: 2,
            null_format: Cow::Borrowed(""),
            block_preferred_style: ScalarType::Plain,
            root_preferred_style: ScalarType::DoubleQuote,
            flow_string_style: FlowStyle::DoubleQuote,
            key_preferred_style: ScalarType::Plain,
            compact_strings: false,
            root_style: YamlStyle::default(),
        }
    }
}
#[cfg(feature = "serde")]
impl YamlFormatter {
    #[must_use]
    pub fn pretty() -> Self {
        Self {
            block_depth_limit: 10,
            pref_string_length: 80,
            indent_len: 2,
            null_format: Cow::Borrowed("null"),
            block_preferred_style: ScalarType::Plain,
            root_preferred_style: ScalarType::DoubleQuote,
            flow_string_style: FlowStyle::DoubleQuote,
            key_preferred_style: ScalarType::Plain,
            compact_strings: false,
            root_style: YamlStyle::default(),
        }
    }

    #[must_use]
    pub fn json() -> Self {
        Self {
            block_depth_limit: 0,
            pref_string_length: 80,
            indent_len: 2,
            null_format: Cow::Borrowed("null"),
            block_preferred_style: ScalarType::DoubleQuote,
            root_preferred_style: ScalarType::DoubleQuote,
            flow_string_style: FlowStyle::DoubleQuote,
            key_preferred_style: ScalarType::DoubleQuote,
            compact_strings: true,
            root_style: YamlStyle::Flow,
        }
    }

    #[inline]
    pub fn set_null_format(&mut self, fmt: NullFormat) {
        self.null_format = fmt.to_null_string();
    }
}

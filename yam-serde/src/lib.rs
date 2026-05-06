#![no_std]
extern crate alloc;

use alloc::string::{String, ToString};
use core::fmt::{Display, Formatter};
use serde_core::de;
use serde_core::de::{MapAccess, SeqAccess};
use yam_core::prelude::YamlError;

#[derive(Debug)]
pub enum YamSerdeError {
    ParsingError(YamlError),
    Custom(String),
}

impl YamSerdeError {
    pub fn new_from_str(msg: &str) -> Self {
        YamSerdeError::Custom(msg.to_string())
    }
}

impl Display for YamSerdeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            YamSerdeError::Custom(x) => write!(f, "Custom error: {x}")?,
            YamSerdeError::ParsingError(yaml_error) => write!(f, "Parsing error: {yaml_error}")?,
        }
        Ok(())
    }
}

impl de::StdError for YamSerdeError {}

impl de::Error for YamSerdeError {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        YamSerdeError::Custom(msg.to_string())
    }
}

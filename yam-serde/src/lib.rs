#![no_std]

extern crate alloc;
pub mod de;
pub mod ser;

use crate::de::DeYamlError;

pub fn from_str<'a, T>(input: &'a str) -> Result<T, DeYamlError>
where
    T: serde_core::de::Deserialize<'a>,
{
    let mut de = crate::de::YamlIterDeserializer::new(input);
    let value = T::deserialize(&mut de)?;

    Ok(value)
}

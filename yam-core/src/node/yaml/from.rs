// From conversions

use crate::node::{Yaml, YamlData};
use crate::prelude::{YamlDocAccess, YamlEntry};
use alloc::string::String;
use alloc::vec::Vec;

macro_rules! simple_from_conv {
    ($from: ty, $method: ident) => {
        impl<'a> From<$from> for Yaml<'a> {
            #[inline]
            fn from(value: $from) -> Self {
                Yaml::$method(value)
            }
        }
    };
}

simple_from_conv!(char, from_char);
simple_from_conv!(String, from_string);
simple_from_conv!(&str, from_string);
simple_from_conv!(bool, from_bool);
simple_from_conv!(f64, from_float);
simple_from_conv!(f32, from_float);
simple_from_conv!(i8, from_integer);
simple_from_conv!(i16, from_integer);
simple_from_conv!(i32, from_integer);
simple_from_conv!(i64, from_integer);
simple_from_conv!(u32, from_integer);
simple_from_conv!(u16, from_integer);
simple_from_conv!(u8, from_integer);

impl<'a, T> From<Vec<T>> for Yaml<'a>
where
    T: Into<Yaml<'a>>,
{
    fn from(value: Vec<T>) -> Self {
        let sequence = value.into_iter().map(T::into).collect::<Vec<Yaml<'a>>>();
        Yaml(YamlData::Sequence(sequence))
    }
}

impl<'a, K, V> From<Vec<(K, V)>> for Yaml<'a>
where
    K: Into<Yaml<'a>>,
    V: Into<Yaml<'a>>,
{
    fn from(value: Vec<(K, V)>) -> Self {
        let sequence = value
            .into_iter()
            .map(|(key, value)| YamlEntry::new(key.into(), value.into()))
            .collect();
        Yaml(YamlData::Mapping(sequence))
    }
}

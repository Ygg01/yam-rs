use crate::node::{Yaml, YamlData};
use crate::prelude::{ToMut, ToMutStr, YamlDocAccess, YamlEntry};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Formatter;
use core::marker::PhantomData;
use serde_core::de::{Error, MapAccess, SeqAccess, Visitor};
use serde_core::{Deserialize, Deserializer};

impl<'de, INT, FLOAT, STR> Deserialize<'de> for Yaml<'de, FLOAT, INT, STR>
where
    FLOAT: Copy + ToMut<f64>,
    INT: Copy + From<i64> + ToMut<i64>,
    STR: Clone + ToMutStr<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct YamlVisitor<F, I, S>(PhantomData<(F, I, S)>);
        impl<F, I, S> YamlVisitor<F, I, S> {
            fn new() -> Self {
                YamlVisitor(PhantomData)
            }
        }

        impl<'de, F, I, S> Visitor<'de> for YamlVisitor<F, I, S>
        where
            F: Copy + ToMut<f64>,
            I: Copy + From<i64> + ToMut<i64>,
            S: Clone + ToMutStr<'de>,
        {
            type Value = Yaml<'de, F, I, S>;

            #[inline]
            fn expecting(&self, formatter: &mut Formatter) -> core::fmt::Result {
                formatter.write_str("any valid YAML value")
            }

            #[inline]
            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_bool(v))
            }

            #[inline]
            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_integer(v))
            }

            #[inline]
            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_integer(v))
            }

            #[inline]
            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_integer(v))
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_integer(v))
            }

            #[inline]
            fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_integer(v))
            }

            #[inline]
            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_integer(v))
            }

            #[inline]
            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_integer(v))
            }

            #[inline]
            fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_char(v))
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_string(v.to_string()))
            }

            #[inline]
            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_str(v))
            }

            #[inline]
            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::from_string(v))
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::null())
            }

            fn visit_seq<VS>(self, mut seq_visitor: VS) -> Result<Self::Value, VS::Error>
            where
                VS: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = seq_visitor.next_element()? {
                    vec.push(elem);
                }

                Ok(Yaml(YamlData::Sequence(vec)))
            }

            fn visit_map<A>(self, mut map_visitor: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(key) = map_visitor.next_key()? {
                    let val = map_visitor.next_value()?;
                    let entry = YamlEntry::new(key, val);
                    vec.push(entry);
                }

                Ok(Yaml(YamlData::Mapping(vec)))
            }
        }

        deserializer.deserialize_any(YamlVisitor::new())
    }
}

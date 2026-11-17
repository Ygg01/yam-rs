use crate::node::Yaml;
use crate::prelude::{ToMut, ToMutStr, YamlDocAccess};
use core::fmt::Formatter;
use core::marker::PhantomData;
use serde_core::de::{Error, Visitor};
use serde_core::{Deserialize, Deserializer};

impl<'de, INT, FLOAT, STR> Deserialize<'de> for Yaml<'de, FLOAT, INT, STR>
where
    FLOAT: Copy + ToMut<f64>,
    INT: Copy + From<i64> + ToMut<i64>,
    STR: Clone + ToMutStr,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct YamlVisitor<I, F, S>(PhantomData<(I, F, S)>);

        impl<'de, F, I, S> Visitor<'de> for YamlVisitor<F, I, S>
        where
            F: Copy + ToMut<f64>,
            I: Copy + From<i64> + ToMut<i64>,
            S: Clone + ToMutStr,
        {
            type Value = Yaml<'de, F, I, S>;

            fn expecting(&self, formatter: &mut Formatter) -> core::fmt::Result {
                formatter.write_str("any valid YAML value")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::<'_, F, I, S>::from_bool(v))
            }

            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::<'_, F, I, S>::from_integer(v))
            }

            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::<'_, F, I, S>::from_integer(v))
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::<'_, F, I, S>::from_integer(v))
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::<'_, F, I, S>::from_integer(v))
            }
            fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::<'_, F, I, S>::from_integer(v))
            }
            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::<'_, F, I, S>::from_integer(v))
            }

            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::<'_, F, I, S>::from_integer(v))
            }

            fn visit_char<E>(self, v: char) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Yaml::<'_, F, I, S>::from_char(v))
            }
        }

        deserializer.deserialize_any(YamlVisitor(PhantomData))
    }
}

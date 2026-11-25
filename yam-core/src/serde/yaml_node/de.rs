use crate::node::{Yaml, YamlData};
use crate::prelude::{ToMut, ToMutStr, YamlDocAccess, YamlEntry};
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt::Formatter;
use core::marker::PhantomData;
use serde_core::de::{Error, MapAccess, SeqAccess, Visitor};
use serde_core::{Deserialize, Deserializer};

impl<'de, FLOAT, INT, STR> Deserialize<'de> for Yaml<'de, FLOAT, INT, STR>
where
    FLOAT: Copy + ToMut<f64>,
    INT: Copy + From<i64> + ToMut<i64>,
    STR: Clone + ToMutStr<'de>,
{
    #[allow(clippy::too_many_lines)]
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

#[cfg(test)]
mod test {
    use crate::node::Yaml;
    use crate::serde::to_yaml;
    use alloc::string::ToString;
    use core::fmt::Error;

    #[test]
    fn test_yaml_eq() {
        let int = 3i8;
        let yaml: Yaml = to_yaml(int).unwrap();
        assert_eq!(yaml, int);

        let int = 4i16;
        let yaml: Yaml = to_yaml(int).unwrap();
        assert_eq!(yaml, int);

        let int = 5i32;
        let yaml: Yaml = to_yaml(int).unwrap();
        assert_eq!(yaml, int);

        let int = 6i64;
        let yaml: Yaml = to_yaml(int).unwrap();
        assert_eq!(yaml, int);

        let int = 13u8;
        let yaml: Yaml = to_yaml(int).unwrap();
        assert_eq!(yaml, int);

        let int = 4u16;
        let yaml: Yaml = to_yaml(int).unwrap();
        assert_eq!(yaml, int);

        let int = 5u32;
        let yaml: Yaml = to_yaml(int).unwrap();
        assert_eq!(yaml, int);

        let int = 6u64;
        let mut yaml: Yaml = to_yaml(int).unwrap();
        assert_eq!(&yaml, 6u64);
        assert_eq!(int, yaml);
        assert_eq!(&int, &mut yaml);
        assert_eq!(&mut yaml, 6u64);
        assert_eq!(yaml, int);

        let int = 343usize;
        let mut yaml: Yaml = to_yaml(int).unwrap();
        assert_eq!(&yaml, 343usize);
        assert_eq!(&int, &yaml);
        assert_eq!(&yaml, &int);
        assert_eq!(&int, &mut yaml);
        assert_eq!(&mut yaml, 343usize);

        let int = u64::MAX - 3;
        let yaml: Result<Yaml, Error> = to_yaml(int);
        assert!(yaml.is_err());

        let float = 3.0f32;
        let yaml: Yaml = to_yaml(float).unwrap();
        assert_eq!(yaml, float);

        let float = 1.0;
        let mut yaml: Yaml = to_yaml(float).unwrap();
        assert_eq!(yaml, float);
        assert_eq!(float, yaml);
        assert_eq!(&yaml, 1.0f64);
        assert_eq!(&mut yaml, 1.0f64);

        let chr = '#';
        let yaml: Yaml = to_yaml(chr).unwrap();
        assert_eq!(&yaml, &chr);
        assert_eq!(&chr, &yaml);

        let str = "A big fat string";
        let mut yaml: Yaml = to_yaml(str).unwrap();
        assert_eq!(&yaml, &str);
        assert_eq!(&str, &yaml);
        assert_eq!(&str, &mut yaml);
        assert_eq!(&mut yaml, "A big fat string");
        assert_eq!(&yaml, "A big fat string");

        let str = "A big fat stringBuf".to_string();
        let mut yaml: Yaml = to_yaml(str.clone()).unwrap();
        assert_eq!(&yaml, &str);
        assert_eq!(&str, &yaml);
        assert_eq!(&yaml, "A big fat stringBuf".to_string());
        assert_eq!(&mut yaml, "A big fat stringBuf".to_string());

        let bool = true;
        let mut yaml: Yaml = to_yaml(bool).unwrap();

        assert_eq!(&yaml, &bool);
        assert_eq!(&bool, &mut yaml);

        let chr = '%';
        let mut yaml: Yaml = to_yaml(chr).unwrap();
        assert_eq!(&yaml, chr);
        assert_eq!(&chr, &yaml);
        assert_eq!(&mut yaml, chr);
    }
}

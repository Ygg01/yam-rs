use crate::node::{Yaml, YamlData};
use crate::prelude::{ToMut, ToMutStr, YamlDocAccess, YamlEntry};
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt::Formatter;
use core::marker::PhantomData;
use serde_core::de::{EnumAccess, Error, MapAccess, SeqAccess, Visitor};
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
    use crate::prelude::YamlDocAccess;
    use alloc::string::String;
    use alloc::vec;
    use alloc::vec::Vec;
    use serde::Deserialize;

    #[test]
    fn test_nested_collections() {
        #[derive(Deserialize, Debug)]
        pub struct Ex {
            a: String,
            b: Vec<f32>,
        }

        let ex = r#"{ a: "x",  b: [2.0, 3.1, -1.2] }"#;
        let deserialized: Ex = crate::from_str(ex).unwrap();
        assert_eq!(deserialized.a, "x");
        assert_eq!(deserialized.b, vec![2.0, 3.1, -1.2]);
    }

    #[test]
    fn test_schemas_null() {
        #[derive(Deserialize, Debug)]
        pub struct Ex {
            a: Option<String>,
        }
        let input = r"a: !!null";
        let deserialized: Ex = crate::from_str(input).unwrap();
        assert_eq!(deserialized.a, None);

        let input = r"a: !!null null";
        let deserialized2: Ex = crate::from_str(input).unwrap();
        assert_eq!(deserialized2.a, None);
    }

    #[test]
    fn test_schemas_float() {
        #[derive(Deserialize, Debug)]
        pub struct Ex {
            a: Option<f32>,
        }
        let input = r"a: 3";
        let deserialized: Ex = crate::from_str(input).unwrap();
        assert_eq!(deserialized.a, Some(3.0));
    }

    #[test]
    fn test_enum() {
        #[derive(Deserialize, PartialEq, Debug)]
        enum E {
            Unit,
            Newtype(f32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        let j = r#"{ Unit }"#;
        let expected = E::Unit;
        assert_eq!(expected, crate::from_str(j).unwrap());

        let j = r#"{ Newtype: 1 }"#;
        let expected = E::Newtype(1.0);
        assert_eq!(expected, crate::from_str(j).unwrap());

        let j = r#"{"Tuple":[1,2]}"#;
        let expected = E::Tuple(1, 2);
        assert_eq!(expected, crate::from_str(j).unwrap());

        let j = r#"{"Struct":{"a":1}}"#;
        let expected = E::Struct { a: 1 };
        assert_eq!(expected, crate::from_str(j).unwrap());
    }

    #[test]
    fn test_doc_example1() {
        #[derive(Deserialize, PartialEq, Debug)]
        pub struct Person {
            name: String,
            address: Option<String>,
            phones: Vec<String>,
        }

        let data = r#"
    name: John Smith
    phones:
        - "+44 1234567"
        - "+32 2345678"
    address:
    "#;

        let person: Person = crate::from_str(data).expect("Yaml to be parsed!");
        assert_eq!(person.name, "John Smith");
        assert_eq!(person.address, None);
        assert_eq!(person.phones, vec!["+44 1234567", "+32 2345678"]);
    }

    #[test]
    fn test_doc_example2() {
        let data = r#"
    name: John Smith
    phones:
        - "+44 1234567"
        - "+32 2345678"
    address:
    "#;

        let person = Yaml::load_single(data).expect("Yaml to be parsed!");
        assert_eq!(person["name"].as_str(), Some("John Smith"));
        assert_eq!(person["address"].as_str(), None);
        assert_eq!(person["phones"][1].as_str(), Some("+32 2345678"));
    }
}

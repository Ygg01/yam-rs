use crate::binary::encode_as_base64;
use crate::node::{Yaml, YamlData, YamlScalar};
use crate::parsing::Tag;
use crate::prelude::{ToMut, ToMutStr, YamlDocAccess, YamlEntry};
use crate::serde::to_yaml;
use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt::{Display, Error};
use core::marker::PhantomData;
use serde_core::ser::{
    SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
    SerializeTupleStruct, SerializeTupleVariant,
};
use serde_core::{Serialize, Serializer};

impl<'de, FLOAT, INT, STR> Serialize for Yaml<'de, FLOAT, INT, STR>
where
    FLOAT: Copy + ToMut<f64>,
    INT: Copy + From<i64> + ToMut<i64>,
    STR: Clone + ToMutStr<'de>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self.0 {
            YamlData::BadValue | YamlData::Scalar(YamlScalar::Null(_)) => {
                serializer.serialize_unit()
            }
            YamlData::Scalar(YamlScalar::String(s)) => serializer.serialize_str(s.as_str()),
            YamlData::Scalar(YamlScalar::FloatingPoint(f)) => {
                serializer.serialize_f64(f.as_owned())
            }
            YamlData::Scalar(YamlScalar::Integer(i)) => serializer.serialize_i64(i.as_owned()),
            YamlData::Scalar(YamlScalar::Bool(n)) => serializer.serialize_bool(*n),
            YamlData::Sequence(seq) => seq.serialize(serializer),
            YamlData::Mapping(map) => {
                use serde_core::ser::SerializeMap;
                let mut map_serializer = serializer.serialize_map(Some(map.len()))?;
                for entry in map {
                    map_serializer.serialize_entry(&entry.key, &entry.value)?;
                }
                map_serializer.end()
            }
            YamlData::Tagged(_, node) => node.serialize(serializer),
            YamlData::Alias(_) => todo!("Implement alias"),
        }
    }
}

#[doc(hidden)]
#[allow(dead_code)]
pub struct YamPhantomSerializer<'de, F, I, S>(&'de PhantomData<()>, PhantomData<(F, I, S)>);

impl<'de, F, I, S> Default for YamPhantomSerializer<'de, F, I, S> {
    fn default() -> Self {
        YamPhantomSerializer(&PhantomData, PhantomData)
    }
}

#[doc(hidden)]
pub struct YamSeqSerializer<'de, F, I, S> {
    seq: Vec<Yaml<'de, F, I, S>>,
}

#[doc(hidden)]
pub struct YamMapSerializer<'de, F, I, S> {
    entries: Vec<YamlEntry<'de, Yaml<'de, F, I, S>>>,
}

#[doc(hidden)]
pub struct YamTupleVariantSerializer<'de, F, I, S> {
    variant: String,
    seq: Vec<Yaml<'de, F, I, S>>,
}

#[doc(hidden)]
pub struct YamStructVariantSerializer<'de, F, I, S> {
    variant: String,
    entries: Vec<YamlEntry<'de, Yaml<'de, F, I, S>>>,
}

impl<'de, F, I, S> Serializer for YamPhantomSerializer<'de, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'de>,
{
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;
    type SerializeSeq = YamSeqSerializer<'de, F, I, S>;
    type SerializeTuple = YamSeqSerializer<'de, F, I, S>;
    type SerializeTupleStruct = YamSeqSerializer<'de, F, I, S>;
    type SerializeTupleVariant = YamTupleVariantSerializer<'de, F, I, S>;
    type SerializeMap = YamMapSerializer<'de, F, I, S>;
    type SerializeStruct = YamMapSerializer<'de, F, I, S>;
    type SerializeStructVariant = YamStructVariantSerializer<'de, F, I, S>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_bool(v))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_integer(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_integer(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_integer(v))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_integer(v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_integer(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_integer(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_integer(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        if let Ok(conv) = i64::try_from(v) {
            Ok(Yaml::from_integer(conv))
        } else {
            Err(Error)
        }
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_float(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_float(v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        let mut str = String::with_capacity(4);
        str.push(v);
        Ok(Yaml::from_string(str))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::from_string(v.to_string()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let encode_bytes = encode_as_base64(v);
        Ok(Yaml(YamlData::Tagged(
            Cow::Owned(Tag::binary_schema()),
            Box::new(Yaml::from_string(encode_bytes)),
        )))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml::null())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut mapping = Vec::new();
        let key = Yaml::from_str(variant);
        let value = to_yaml(&value)?;
        mapping.push(YamlEntry::new(key, value));

        Ok(Yaml(YamlData::Mapping(mapping)))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(YamSeqSerializer {
            seq: Vec::with_capacity(len.unwrap_or_default()),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(YamTupleVariantSerializer {
            variant: variant.to_string(),
            seq: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(YamMapSerializer {
            entries: Vec::with_capacity(len.unwrap_or_default()),
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(YamMapSerializer {
            entries: Vec::with_capacity(len),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(YamStructVariantSerializer {
            variant: variant.to_string(),
            entries: Vec::with_capacity(len),
        })
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Display,
    {
        self.serialize_str(&value.to_string())
    }
}

impl<'de, F, I, S> SerializeSeq for YamSeqSerializer<'de, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'de>,
{
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let v = match to_yaml(value) {
            Ok(x) => x,
            Err(_) => return Err(core::fmt::Error),
        };
        self.seq.push(v);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml(YamlData::Sequence(self.seq)))
    }
}

impl<'de, F, I, S> SerializeTuple for YamSeqSerializer<'de, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'de>,
{
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml(YamlData::Sequence(self.seq)))
    }
}

impl<'de, F, I, S> SerializeTupleStruct for YamSeqSerializer<'de, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'de>,
{
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml(YamlData::Sequence(self.seq)))
    }
}

impl<'de, F, I, S> SerializeMap for YamMapSerializer<'de, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'de>,
{
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        let key = to_yaml(key)?;
        let val = to_yaml(value)?;
        self.entries.push(YamlEntry::new(key, val));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml(YamlData::Mapping(self.entries)))
    }
}

impl<'de, F, I, S> SerializeStruct for YamMapSerializer<'de, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'de>,
{
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        SerializeMap::serialize_entry(self, key, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Yaml(YamlData::Mapping(self.entries)))
    }
}

impl<'de, F, I, S> SerializeTupleVariant for YamTupleVariantSerializer<'de, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'de>,
{
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let val = to_yaml(value)?;
        self.seq.push(val);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let entry = YamlEntry::new(
            Yaml::from_string(self.variant),
            Yaml(YamlData::Sequence(self.seq)),
        );

        Ok(Yaml(YamlData::Mapping(vec![entry])))
    }
}

impl<'de, F, I, S> SerializeStructVariant for YamStructVariantSerializer<'de, F, I, S>
where
    F: Copy + ToMut<f64>,
    I: Copy + ToMut<i64>,
    S: Clone + ToMutStr<'de>,
{
    type Ok = Yaml<'de, F, I, S>;
    type Error = core::fmt::Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let key = Yaml::from_string(key);
        let value = to_yaml(value)?;
        self.entries.push(YamlEntry::new(key, value));
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let entry = YamlEntry::new(
            Yaml::from_string(self.variant),
            Yaml(YamlData::Mapping(self.entries)),
        );
        Ok(Yaml(YamlData::Mapping(vec![entry])))
    }
}

#[cfg(test)]
mod test {
    use crate::node::SpanYam;
    use crate::prelude::{Marker, ScalarType, Span, YamLoader, YamlDocAccess};
    use crate::serde::ser::FlowStyle;
    use crate::serde::{NullFormat, YamlFormatter};
    use crate::to_formatted_string;
    use alloc::collections::BTreeMap;
    use alloc::string::{String, ToString};
    use alloc::vec;
    use core::fmt::Error;
    use serde::Serialize;

    fn assert_eq_strings(result: Result<String, Error>, correct: &str) {
        assert_eq!(result, Ok(correct.to_string()));
    }
    const VEC_EXPECTED: &str = "- xyz\n- abc\n";

    #[test]
    fn test_vec_str() {
        let x = vec!["xyz".to_string(), "abc".to_string()];
        let fmt = YamlFormatter::default();
        let result = to_formatted_string(&x, fmt);

        assert_eq_strings(result, VEC_EXPECTED);
    }

    #[test]
    fn test_null_fmt() {
        let x: Option<i32> = None;
        let fmt = YamlFormatter::default();
        let result = to_formatted_string(&x, fmt);

        assert_eq_strings(result, NullFormat::Plain.to_null_string().as_ref());

        let fmt = YamlFormatter::pretty();
        let result = to_formatted_string(&x, fmt);

        assert_eq_strings(result, NullFormat::JsonNull.to_null_string().as_ref());

        let mut fmt = YamlFormatter::default();
        fmt.set_null_format(NullFormat::OldYaml);
        let result = to_formatted_string(&x, fmt);

        assert_eq_strings(result, NullFormat::OldYaml.to_null_string().as_ref());

        let mut fmt = YamlFormatter::default();
        fmt.set_null_format(NullFormat::TaggedYaml);
        let result = to_formatted_string(&x, fmt);

        assert_eq_strings(result, NullFormat::TaggedYaml.to_null_string().as_ref());
    }

    const MULTI_LINE_STRING1_ACTUAL: &str = "One quick brown fox jumps over the lazy dog";
    const MULTI_LINE_STRING1_EXPECTED: &str = r#""One quick
brown fox
jumps over
the lazy
dog""#;
    #[test]
    fn test_multiline_string() {
        let formatter = {
            let mut x = YamlFormatter::pretty();
            x.pref_string_length = 10;
            x
        };
        let result = to_formatted_string(&MULTI_LINE_STRING1_ACTUAL, formatter);
        assert_eq!(result, Ok(MULTI_LINE_STRING1_EXPECTED.to_string()));
    }

    #[test]
    fn test_unit_struct() {
        #[derive(Serialize)]
        struct Example;

        let formatter = YamlFormatter::pretty();
        let result = to_formatted_string(&Example, formatter);
        assert_eq!(result, Ok("{}".to_string()));
    }

    #[test]
    fn test_serialize_newtype_struct() {
        #[derive(Serialize)]
        struct Measurement(u8);

        let formatter = YamlFormatter::pretty();
        let result = to_formatted_string(&Measurement(0), formatter);
        assert_eq!(result, Ok("0".to_string()));
    }

    const SIMPLE_MAP_EXPECTED: &str = r#"a: 34
b: 1"#;

    #[test]
    fn test_serialize_simple_key() {
        let key = "a";
        let value = 34;
        let mut map = BTreeMap::new();
        map.insert(key, value);
        map.insert("b", 1);

        let result = to_formatted_string(&map, YamlFormatter::pretty());
        assert_eq!(result, Ok(SIMPLE_MAP_EXPECTED.to_string()));
    }

    const SIMPLE_STRUCT_EXPECTED: &str = r#"num: 3
string: test"#;

    #[test]
    fn test_serialize_struct() {
        #[derive(Serialize, Eq, Hash, PartialEq)]
        struct Example {
            num: i16,
            string: String,
        }

        let strct = Example {
            num: 3,
            string: "test".to_string(),
        };
        let result = to_formatted_string(&strct, YamlFormatter::pretty());
        assert_eq!(result, Ok(SIMPLE_STRUCT_EXPECTED.to_string()));
    }

    const LIST_EXPECTED: &str = r#"- 2
- 3
- 4
"#;

    #[test]
    fn test_list() {
        let example = vec![2, 3, 4];
        let result = to_formatted_string(&example, YamlFormatter::pretty());
        assert_eq!(result, Ok(LIST_EXPECTED.to_string()));
    }

    const NESTED_LIST_EXPECTED: &str = r#"- - 1
  - 2
- - 3
  - 4
"#;

    #[test]
    fn test_nested_list() {
        let example = vec![vec![1u64, 2], vec![3, 4]];

        let result = to_formatted_string(&example, YamlFormatter::pretty());
        assert_eq!(result, Ok(NESTED_LIST_EXPECTED.to_string()));
    }

    const COMPLEX_KEY_EXPECTED: &str = r#"? - 1
  - 2
: 34"#;

    #[test]
    fn test_serialize_complex_key() {
        let key = vec![1i64, 2];
        let value = 34u32;
        let map = BTreeMap::from([(key, value)]);

        let result = to_formatted_string(&map, YamlFormatter::pretty());
        assert_eq!(result, Ok(COMPLEX_KEY_EXPECTED.to_string()));
    }

    #[test]
    fn test_cant_be_plain() {
        let str = "@cant_be_plain";
        let mut style = YamlFormatter::pretty();
        style.root_preferred_style = ScalarType::Plain;
        style.flow_string_style = FlowStyle::DoubleQuote;

        let result = to_formatted_string(&str, style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "\"@cant_be_plain\"");

        let str = "@cant_be_plain";
        let mut style = YamlFormatter::pretty();
        style.root_preferred_style = ScalarType::Plain;
        style.flow_string_style = FlowStyle::SingleQuote;

        let result = crate::to_formatted_string(&str, style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "'@cant_be_plain'");
    }

    const FOLD_EXPECTED_1: &str = r#">-
block of text !!!"#;

    const FOLD_EXPECTED_2: &str = r#">
block of text !!!"#;

    const LIT_EXPECTED_1: &str = r#"|-
block of text !!!"#;

    const LIT_EXPECTED_2: &str = r#"|
block of text !!!"#;

    #[test]
    fn test_blocks_text() {
        let str = "block of text !!!";
        let mut style = YamlFormatter::pretty();
        style.root_preferred_style = ScalarType::Folded;

        let result = to_formatted_string(&str, style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FOLD_EXPECTED_1);

        let str = "block of text !!!\n";
        let mut style = YamlFormatter::pretty();
        style.root_preferred_style = ScalarType::Folded;

        let result = to_formatted_string(&str, style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), FOLD_EXPECTED_2);

        let str = "block of text !!!";
        let mut style = YamlFormatter::pretty();
        style.root_preferred_style = ScalarType::Literal;

        let result = to_formatted_string(&str, style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LIT_EXPECTED_1);

        let str = "block of text !!!\n";
        let mut style = YamlFormatter::pretty();
        style.root_preferred_style = ScalarType::Literal;

        let result = to_formatted_string(&str, style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LIT_EXPECTED_2);
    }

    #[test]
    fn test_bool_char() {
        #[derive(Serialize)]
        enum Example {
            Bool((bool, bool)),
            Char(char),
        }

        let style = YamlFormatter::default();
        let result = to_formatted_string(&Example::Bool((true, false)), style);
        assert_eq!(result.unwrap(), "{ Bool: [ true, false ] }");

        let style = YamlFormatter::default();
        let result = to_formatted_string(&Example::Char('b'), style);
        assert_eq!(result.unwrap(), "{ Char: 'b' }");
    }

    #[derive(Serialize)]
    struct Rgb(u8, u8, u8);

    #[test]
    fn test_tuple_struct() {
        let style = YamlFormatter::pretty();
        let result = to_formatted_string(&Rgb(0, 2, 4), style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "[ 0, 2, 4 ]");
    }

    #[test]
    fn test_single_line_str() {
        let string = "AAA";
        let mut style = YamlFormatter::pretty();
        style.root_preferred_style = ScalarType::Plain;
        style.compact_strings = true;
        let result = to_formatted_string(&string, style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "AAA");

        let mut style = YamlFormatter::pretty();
        style.root_preferred_style = ScalarType::DoubleQuote;
        style.compact_strings = true;
        let result = to_formatted_string(&string, style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "\"AAA\"");

        let mut style = YamlFormatter::pretty();
        style.root_preferred_style = ScalarType::SingleQuote;
        style.compact_strings = true;
        let result = to_formatted_string(&string, style);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "'AAA'");
    }

    #[test]
    fn test_span_seq() {
        let input = r#"[
    a,
    b
]"#;

        if let Ok(yaml) = YamLoader::<SpanYam>::load_single(input) {
            let first = &yaml[0];
            let second = &yaml[1];
            assert_eq!(first.as_str().unwrap_or(""), "a");
            assert_eq!(second.as_str().unwrap_or(""), "b");
            let span_1 = Span::new(Marker::new(6, 5, 2), Marker::new(7, 6, 2));
            let span_2 = Span::new(Marker::new(13, 5, 3), Marker::new(14, 6, 3));
            assert_eq!(first.span, span_1);
            assert_eq!(second.span, span_2);
        } else {
            panic!("Error parsing Yaml");
        }
    }

    #[test]
    fn test_span_map() {
        let input = r#"
a: 'aaa'
b: !!null
"#;

        if let Ok(yaml) = YamLoader::<SpanYam>::load_single(input) {
            let first = &yaml["a"];
            let second = &yaml["b"];
            assert!(first.is_string(), "First value should be a string");
            assert_eq!(first.as_str().unwrap_or(""), "aaa");
            assert!(second.is_null(), "second element is null");
            let span_1 = Span::new(Marker::new(4, 4, 2), Marker::new(9, 9, 2));
            let span_2 = Span::new(Marker::new(20, 1, 4), Marker::new(20, 1, 4));
            assert_eq!(first.span, span_1);
            assert_eq!(second.span, span_2);
        } else {
            panic!("Error parsing Yaml");
        }
    }

    #[test]
    fn test_nested_map() {
        let input = r#"
a: !!vec [2, 3]
"#;

        if let Ok(yaml) = YamLoader::<SpanYam>::load_single(input) {
            let vec = &yaml["a"];
            assert!(vec.is_non_empty_collection(), "Vec should be populated");
            let first = &vec[0];
            assert!(first.is_integer(), "First should be an integer");
            let second = &vec[1];
            assert!(second.is_integer(), "First should be an integer");
            assert_eq!(first.as_i64().unwrap_or(-1), 2);
            assert_eq!(second.as_i64().unwrap_or(-1), 3);
            let span_vec = Span::new(Marker::new(10, 10, 2), Marker::new(15, 15, 2));
            let span_first = Span::new(Marker::new(11, 11, 2), Marker::new(12, 12, 2));
            let span_second = Span::new(Marker::new(14, 14, 2), Marker::new(15, 15, 2));
            assert_eq!(vec.span, span_vec);
            assert_eq!(first.span, span_first);
            assert_eq!(second.span, span_second);
        } else {
            panic!("Error parsing Yaml");
        }
    }
}

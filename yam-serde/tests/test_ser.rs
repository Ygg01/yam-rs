use core::fmt::Error;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use yam_core::prelude::ScalarType;
use yam_serde::ser::FlowStyle;
use yam_serde::to_pretty_string;
use yam_serde::{NullFormat, PrettyFormatterConfig};

fn assert_eq_strings(result: Result<String, Error>, correct: &str) {
    assert_eq!(result, Ok(correct.to_string()));
}
const VEC_EXPECTED: &str = "- xyz\n- abc\n";

#[test]
fn test_vec_str() {
    let x = vec!["xyz".to_string(), "abc".to_string()];
    let fmt = PrettyFormatterConfig::default();
    let result = to_pretty_string(&x, fmt);

    assert_eq_strings(result, VEC_EXPECTED);
}

#[test]
fn test_null_fmt() {
    let x: Option<i32> = None;
    let fmt = PrettyFormatterConfig::default();
    let result = to_pretty_string(&x, fmt);

    assert_eq_strings(result, NullFormat::Plain.to_null_string().as_ref());

    let fmt = PrettyFormatterConfig::pretty();
    let result = to_pretty_string(&x, fmt);

    assert_eq_strings(result, NullFormat::JsonNull.to_null_string().as_ref());

    let mut fmt = PrettyFormatterConfig::default();
    fmt.set_null_format(NullFormat::OldYaml);
    let result = to_pretty_string(&x, fmt);

    assert_eq_strings(result, NullFormat::OldYaml.to_null_string().as_ref());

    let mut fmt = PrettyFormatterConfig::default();
    fmt.set_null_format(NullFormat::TaggedYaml);
    let result = to_pretty_string(&x, fmt);

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
        let mut x = PrettyFormatterConfig::pretty();
        x.pref_string_length = 10;
        x
    };
    let result = to_pretty_string(&MULTI_LINE_STRING1_ACTUAL, formatter);
    assert_eq!(result, Ok(MULTI_LINE_STRING1_EXPECTED.to_string()));
}

#[test]
fn test_unit_struct() {
    #[derive(Serialize)]
    struct Example;

    let formatter = PrettyFormatterConfig::pretty();
    let result = to_pretty_string(&Example, formatter);
    assert_eq!(result, Ok("{}".to_string()));
}

#[test]
fn test_serialize_newtype_struct() {
    #[derive(Serialize)]
    struct Measurement(u8);

    let formatter = PrettyFormatterConfig::pretty();
    let result = to_pretty_string(&Measurement(0), formatter);
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

    let result = to_pretty_string(&map, PrettyFormatterConfig::pretty());
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
    let result = to_pretty_string(&strct, PrettyFormatterConfig::pretty());
    assert_eq!(result, Ok(SIMPLE_STRUCT_EXPECTED.to_string()));
}

const LIST_EXPECTED: &str = r#"- 2
- 3
- 4
"#;

#[test]
fn test_list() {
    let example = vec![2, 3, 4];
    let result = to_pretty_string(&example, PrettyFormatterConfig::pretty());
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

    let result = to_pretty_string(&example, PrettyFormatterConfig::pretty());
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

    let result = to_pretty_string(&map, PrettyFormatterConfig::pretty());
    assert_eq!(result, Ok(COMPLEX_KEY_EXPECTED.to_string()));
}

const COMPLEX_STRUCT_EXPECTED: &str = r#"m:
  ? val: 1
  : 3"#;

#[test]
fn test_serialize_complex_value() {
    #[derive(Serialize)]
    struct Measurement {
        m: HashMap<Inner, f64>,
    }

    #[derive(Serialize, Eq, Hash, PartialEq)]
    struct Inner {
        val: i64,
    }

    let cmplx = Measurement {
        m: HashMap::from([(Inner { val: 1 }, 3.0)]),
    };
    let result = to_pretty_string(&cmplx, PrettyFormatterConfig::pretty());
    assert_eq!(result, Ok(COMPLEX_STRUCT_EXPECTED.to_string()));
}

const NESTED_STRUCT_EXPECTED: &str = r#"v:
  - - 0
m:
  ? val: 1
  : 5"#;

#[test]
fn test_serialize_nested_value() {
    #[derive(Serialize)]
    struct Measurement {
        v: Vec<Vec<i8>>,
        m: HashMap<Inner, f32>,
    }

    #[derive(Serialize, Eq, Hash, PartialEq)]
    struct Inner {
        val: i16,
    }

    let cmplx = Measurement {
        v: vec![vec![0]],
        m: HashMap::from([(Inner { val: 1 }, 5.0)]),
    };
    let result = to_pretty_string(&cmplx, PrettyFormatterConfig::pretty());
    assert_eq!(result, Ok(NESTED_STRUCT_EXPECTED.to_string()));
}

#[test]
fn test_cant_be_plain() {
    let str = "@cant_be_plain";
    let mut style = PrettyFormatterConfig::pretty();
    style.root_preferred_style = ScalarType::Plain;
    style.flow_string_style = FlowStyle::DoubleQuote;

    let result = to_pretty_string(&str, style);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "\"@cant_be_plain\"");

    let str = "@cant_be_plain";
    let mut style = PrettyFormatterConfig::pretty();
    style.root_preferred_style = ScalarType::Plain;
    style.flow_string_style = FlowStyle::SingleQuote;

    let result = to_pretty_string(&str, style);
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
    let mut style = PrettyFormatterConfig::pretty();
    style.root_preferred_style = ScalarType::Folded;

    let result = to_pretty_string(&str, style);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), FOLD_EXPECTED_1);

    let str = "block of text !!!\n";
    let mut style = PrettyFormatterConfig::pretty();
    style.root_preferred_style = ScalarType::Folded;

    let result = to_pretty_string(&str, style);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), FOLD_EXPECTED_2);

    let str = "block of text !!!";
    let mut style = PrettyFormatterConfig::pretty();
    style.root_preferred_style = ScalarType::Literal;

    let result = to_pretty_string(&str, style);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), LIT_EXPECTED_1);

    let str = "block of text !!!\n";
    let mut style = PrettyFormatterConfig::pretty();
    style.root_preferred_style = ScalarType::Literal;

    let result = to_pretty_string(&str, style);
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

    let style = PrettyFormatterConfig::default();
    let result = to_pretty_string(&Example::Bool((true, false)), style);
    assert_eq!(result.unwrap(), "{ Bool: [ true, false ] }");

    let style = PrettyFormatterConfig::default();
    let result = to_pretty_string(&Example::Char('b'), style);
    assert_eq!(result.unwrap(), "{ Char: 'b' }");
}

#[derive(Serialize)]
struct Rgb(u8, u8, u8);

#[test]
fn test_tuple_struct() {
    let style = PrettyFormatterConfig::pretty();
    let result = to_pretty_string(&Rgb(0, 2, 4), style);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "[ 0, 2, 4 ]");
}

#[test]
fn test_single_line_str() {
    let string = "AAA";
    let mut style = PrettyFormatterConfig::pretty();
    style.root_preferred_style = ScalarType::Plain;
    style.compact_strings = true;
    let result = to_pretty_string(&string, style);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "AAA");

    let mut style = PrettyFormatterConfig::pretty();
    style.root_preferred_style = ScalarType::DoubleQuote;
    style.compact_strings = true;
    let result = to_pretty_string(&string, style);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "\"AAA\"");

    let mut style = PrettyFormatterConfig::pretty();
    style.root_preferred_style = ScalarType::SingleQuote;
    style.compact_strings = true;
    let result = to_pretty_string(&string, style);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "'AAA'");
}

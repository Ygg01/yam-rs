use core::fmt::Error;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use yam_serde::PrettyFormatterConfig;
use yam_serde::to_pretty_string;

fn assert_eq_strings(result: Result<String, Error>, correct: &str) {
    assert_eq!(result, Ok(correct.to_string()));
}

#[test]
fn test_null_fmt() {
    let x: Option<i32> = None;
    let fmt = PrettyFormatterConfig::default();
    let result = to_pretty_string(&x, fmt);

    assert_eq_strings(result, "");

    let x: Option<i32> = None;
    let fmt = PrettyFormatterConfig::pretty();
    let result = to_pretty_string(&x, fmt);

    assert_eq_strings(result, "null");
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
    let example = vec![vec![1, 2], vec![3, 4]];

    let result = to_pretty_string(&example, PrettyFormatterConfig::pretty());
    assert_eq!(result, Ok(NESTED_LIST_EXPECTED.to_string()));
}

const COMPLEX_KEY_EXPECTED: &str = r#"? - 1
  - 2
: 34"#;

#[test]
fn test_serialize_complex_key() {
    let key = vec![1, 2];
    let value = 34;
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
        m: HashMap<Inner, i64>,
    }

    #[derive(Serialize, Eq, Hash, PartialEq)]
    struct Inner {
        val: i16,
    }

    let cmplx = Measurement {
        m: HashMap::from([(Inner { val: 1 }, 3)]),
    };
    let result = to_pretty_string(&cmplx, PrettyFormatterConfig::pretty());
    assert_eq!(result, Ok(COMPLEX_STRUCT_EXPECTED.to_string()));
}

const NESTED_STRUCT_EXPECTED: &str = r#"v:
  - - 0
m:
  ? val: 1
  : 3"#;

#[test]
fn test_serialize_nested_value() {
    #[derive(Serialize)]
    struct Measurement {
        v: Vec<Vec<u8>>,
        m: HashMap<Inner, i64>,
    }

    #[derive(Serialize, Eq, Hash, PartialEq)]
    struct Inner {
        val: i16,
    }

    let cmplx = Measurement {
        v: vec![vec![0]],
        m: HashMap::from([(Inner { val: 1 }, 3)]),
    };
    let result = to_pretty_string(&cmplx, PrettyFormatterConfig::pretty());
    assert_eq!(result, Ok(NESTED_STRUCT_EXPECTED.to_string()));
}

const ENUM_UNIT_EXPECTED: &str = r#"{ Unit }"#;
const ENUM_TUPLE_EXPECTED: &str = r#"{ Tuple: [ 0, 15 ] }"#;
const ENUM_STRUCT_EXPECTED: &str = r#"{ Struct: { name: "XYZ" } }"#;
const ENUM_OUTER_EXPECTED: &str = r#"{ Outer: [ 2, 4, 16 ] }"#;

#[test]
fn test_various_enum() {
    #[derive(Serialize)]
    enum Example {
        Unit,
        Tuple(u8, u16),
        Struct { name: String },
        Outer(Vec<u8>),
    }

    let enum_unit = Example::Unit;
    let result = to_pretty_string(&enum_unit, PrettyFormatterConfig::pretty());
    assert_eq!(result, Ok(ENUM_UNIT_EXPECTED.to_string()));

    let enum_tuple = Example::Tuple(0, 15);
    let result = to_pretty_string(&enum_tuple, PrettyFormatterConfig::pretty());
    assert_eq!(result, Ok(ENUM_TUPLE_EXPECTED.to_string()));

    let enum_struct = Example::Struct {
        name: "XYZ".to_string(),
    };
    let result = to_pretty_string(&enum_struct, PrettyFormatterConfig::pretty());
    assert_eq!(result, Ok(ENUM_STRUCT_EXPECTED.to_string()));

    let enum_outer = Example::Outer(vec![2, 4, 16]);
    let result = to_pretty_string(&enum_outer, PrettyFormatterConfig::pretty());
    assert_eq!(result, Ok(ENUM_OUTER_EXPECTED.to_string()));
}

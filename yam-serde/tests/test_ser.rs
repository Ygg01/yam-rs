use core::fmt::Error;
use serde::Serialize;
use std::collections::BTreeMap;
use yam_serde::ser::PrettyFormatterConfig;
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

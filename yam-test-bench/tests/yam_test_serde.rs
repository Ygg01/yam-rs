use rstest::rstest;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use yam_core::prelude::ScalarType;
use yam_core::serde::ser::YamlStyle;
use yam_core::serde::YamlFormatter;
use yam_core::{from_str, to_pretty_string};
use yam_test_bench::{OneEnumToHoldThemAll, Person};

#[rstest]
#[case::f32(OneEnumToHoldThemAll::F32(3.0))]
#[case::f64(OneEnumToHoldThemAll::F64(6.0))]
#[case::u8(OneEnumToHoldThemAll::U8(8))]
#[case::u16(OneEnumToHoldThemAll::U16(16))]
#[case::u32(OneEnumToHoldThemAll::U32(32))]
#[case::u64(OneEnumToHoldThemAll::U64(64))]
#[case::i8(OneEnumToHoldThemAll::I8(-8))]
#[case::i16(OneEnumToHoldThemAll::I16(-16))]
#[case::i32(OneEnumToHoldThemAll::I32(-32))]
#[case::i64(OneEnumToHoldThemAll::I64(-64))]
#[case::char(OneEnumToHoldThemAll::Char('#'))]
#[case::char(OneEnumToHoldThemAll::Char('\''))]
#[case::opt_none(OneEnumToHoldThemAll::Option(None))]
#[case::opt_some(OneEnumToHoldThemAll::Option(Some(true)))]
#[case::unit_variant(OneEnumToHoldThemAll::UnitVariant)]
#[case::tupl_variant(OneEnumToHoldThemAll::TupleVariant(-32, false))]
#[case::stct_variant(OneEnumToHoldThemAll::StructVariant{ uid: 2 })]
#[case::nwty_variant(
    OneEnumToHoldThemAll::NewTypeVariant(Person{ name: "Mick".to_string(), uid: 3})
)]
#[case::var_tuple(OneEnumToHoldThemAll::VariantTuple((-3, -4)))]
#[case::var_seq(OneEnumToHoldThemAll::VariantSeq(vec!["xyz".to_string(), "abc".to_string()]))]
#[case::var_seq_empty(OneEnumToHoldThemAll::VariantSeq(vec![]))]
fn test_round_trip_opt(#[case] input: OneEnumToHoldThemAll) {
    let fmt = YamlFormatter::default();
    let result = to_pretty_string(&input, fmt);

    assert!(result.is_ok(), "Expected Ok result for serialization");

    let actual = from_str::<OneEnumToHoldThemAll>(&result.unwrap());

    assert!(actual.is_ok(), "Expected Ok result for deserialization");
    assert_eq!(actual.unwrap(), input);
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct BinaryConvert<'a> {
    #[serde(with = "serde_bytes")]
    byte_buf: Vec<u8>,

    // `Cow` rather than `&[u8]`: `!!binary` is base64-decoded into fresh bytes,
    // which can't be borrowed from the input, so a borrowed slice would always fail.
    #[serde(borrow, with = "serde_bytes")]
    bytes: Cow<'a, [u8]>,

    #[serde(with = "serde_bytes")]
    byte_array: [u8; 4],
}

#[test]
fn test_round_trip_bytes_block() {
    let initial = BinaryConvert {
        byte_buf: br"Test this binary stuff".to_vec(),
        bytes: Cow::Borrowed(b"Test this binary"),
        byte_array: *b"abcd",
    };
    let fmt_config = YamlFormatter::default();
    let result = to_pretty_string(&initial, fmt_config);

    assert!(result.is_ok(), "Expected Ok result for serialization");
    let binding = result.unwrap();

    let actual = from_str::<BinaryConvert>(&binding);

    if let Err(x) = &actual {
        panic!("{x}")
    }
    assert_eq!(actual.unwrap(), initial);
}

#[test]
fn test_round_trip_bytes_flow() {
    let initial = BinaryConvert {
        byte_buf: br"Test this binary stuff".to_vec(),
        bytes: Cow::Borrowed(b"Test this binary"),
        byte_array: *b"abcd",
    };
    let mut fmt_config = YamlFormatter::default();
    fmt_config.root_style = YamlStyle::Flow;

    let result = to_pretty_string(&initial, fmt_config);

    assert!(result.is_ok(), "Expected Ok result for serialization");
    let binding = result.unwrap();
    let actual = from_str::<BinaryConvert>(&binding);

    if let Err(x) = &actual {
        panic!("{x}")
    }
    assert_eq!(actual.unwrap(), initial);
}

const RAW_TAB: &[u8] = b"\t";
const RAW_CRLF: &[u8] = b"\r\n";

#[rstest]
#[case::decode_tab(br"\\t", r#""\\t""#)]
#[case::decode_cr(br"\\r", r#""\\r""#)]
#[case::decode_lf(br"\\n", r#""\\n""#)]
#[case::decode_raw_tab(RAW_TAB, r#""\t""#)]
#[case::decode_raw_crlf(RAW_CRLF, r#""\n""#)]
#[case::decode_raw_lf(b"\n", r#""\n""#)]
#[case::decode_invalid_hex(br"\x0?", r#""�""#)]
#[case::decode_zero(br"\x00", r#""�""#)]
#[case::decode_hex_alert(br"\x07", r#""\a""#)]
#[case::decode_hex_bell(br"\x08", r#""\b""#)]
#[case::decode_hex_tab(br"\x09", r#""\t""#)]
#[case::decode_hex_lf(br"\x0A", r#""\n""#)]
#[case::decode_hex_vertical_tab(br"\x0B", r#""\v""#)]
#[case::decode_hex_form_feed(br"\x0C", r#""\f""#)]
#[case::decode_hex_form_carriage(br"\x0D", r#""\r""#)]
#[case::decode_hex_escape(br"\x1B", r#""\e""#)]
#[case::decode_hex_dquote(br"\x22", r#""\"""#)]
#[case::decode_hex_solidus(br"\x2F", r#""\/""#)]
#[case::decode_hex_rev_solidus(br"\x5c", r#""\\""#)]
#[case::decode_hex_rev_solidus(br"\x85", r#""\N""#)]
#[case::decode_hex_nbsp(br"\xA0", r#""\_""#)]
#[case::decode_hex_line_separator(br"\u2028", r#""\L""#)]
#[case::decode_hex_paragraph_separator(br"\u2029", r#""\P""#)]
#[case::decode_emojis(br"\U0001F600", r#""😀""#)]
fn test_dquote_string_escape(#[case] input: &[u8], #[case] decoded: &str) {
    let fmt_config = YamlFormatter::default();
    let input_str = String::from_utf8_lossy(input);
    let result = to_pretty_string(&input_str, fmt_config);
    let actual = result.expect("Expected Serialization successful");

    assert_eq!(actual, decoded);
}

#[rstest]
#[case::decode_tab(br"test", r#"'test'"#)]
#[case::decode_lf(b"\n", r#"'\n'"#)]
#[case::decode_raw_tab(RAW_TAB, r#"'\t'"#)]
#[case::decode_raw_crlf(RAW_CRLF, r#"'\n'"#)]
#[case::decode_raw_lf(b"Dark Soul's", r#"'Dark Soul''s'"#)]
fn test_squote_string_escape(#[case] input: &[u8], #[case] decoded: &str) {
    let mut fmt_config = YamlFormatter::default();
    fmt_config.root_preferred_style = ScalarType::SingleQuote;
    let input_str = String::from_utf8_lossy(input);
    let result = to_pretty_string(&input_str, fmt_config);
    let actual = result.expect("Expected Serialization successful");

    assert_eq!(actual, decoded);
}

#[test]
fn test_serialize_complex_value() {
    const COMPLEX_STRUCT_EXPECTED: &str = r#"m:
  ? val: 1
  : 3"#;

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
    let result = to_pretty_string(&cmplx, YamlFormatter::pretty());
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
    let result = to_pretty_string(&cmplx, YamlFormatter::pretty());
    assert_eq!(result, Ok(NESTED_STRUCT_EXPECTED.to_string()));
}

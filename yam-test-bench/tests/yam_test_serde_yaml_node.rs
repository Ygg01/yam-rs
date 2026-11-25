use rstest::rstest;
use yam_core::prelude::Yaml;
use yam_core::serde::YamlFormatter;
use yam_core::{from_str, to_formatted_string};
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
#[case::tupl_variant(OneEnumToHoldThemAll::TupleVariant(-32, false))]
#[case::stct_variant(OneEnumToHoldThemAll::StructVariant{ uid: 2 })]
#[case::nwty_variant(
    OneEnumToHoldThemAll::NewTypeVariant(Person{ name: "Mick".to_string(), uid: 3})
)]
#[case::var_tuple(OneEnumToHoldThemAll::VariantTuple((-3, -4)))]
#[case::var_seq(OneEnumToHoldThemAll::VariantSeq(vec!["xyz".to_string(), "abc".to_string()]))]
#[case::var_seq_empty(OneEnumToHoldThemAll::VariantSeq(vec![]))]
fn test_round_trip_opt(#[case] input: OneEnumToHoldThemAll) {
    let formatted_str = to_formatted_string(&input, YamlFormatter::json())
        .expect("Enum to string conversion succeeded");

    let yaml: Yaml = from_str(&formatted_str).expect("Deserialization successful");

    let actual_str =
        to_formatted_string(&yaml, YamlFormatter::json()).expect("Reserialization successful");

    assert_eq!(actual_str, formatted_str);
}

#[test]
fn test_round_trip_cas() {
    // Due to specifics of enumeration stuff, `UnitVariant`s get emitted as
    // `{ UnitVariant }` with diverges from JSON-like `{ "UnitVariant": null }`
    let input = OneEnumToHoldThemAll::UnitVariant;
    let expected =
        to_formatted_string(&input, YamlFormatter::json()).expect("Enum to string conversion");

    let yaml: Yaml = from_str(&expected).expect("Deserialization successful");
    let actual_str =
        to_formatted_string(&yaml, YamlFormatter::json()).expect("Reserialization successful");

    let actual_enum =
        from_str::<OneEnumToHoldThemAll>(&actual_str).expect("Able to parse string to enum");

    assert_eq!(&actual_enum, &input);
}

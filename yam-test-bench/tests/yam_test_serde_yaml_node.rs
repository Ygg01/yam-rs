use yam_core::from_str;
use yam_core::prelude::Yaml;
use yam_core::serde::to_string;
use yam_test_bench::OneEnumToHoldThemAll;

// #[rstest]
// #[case::f32(OneEnumToHoldThemAll::F32(3.0))]
// #[case::f64(OneEnumToHoldThemAll::F64(6.0))]
// #[case::u8(OneEnumToHoldThemAll::U8(8))]
// #[case::u16(OneEnumToHoldThemAll::U16(16))]
// #[case::u32(OneEnumToHoldThemAll::U32(32))]
// #[case::u64(OneEnumToHoldThemAll::U64(64))]
// #[case::i8(OneEnumToHoldThemAll::I8(-8))]
// #[case::i16(OneEnumToHoldThemAll::I16(-16))]
// #[case::i32(OneEnumToHoldThemAll::I32(-32))]
// #[case::i64(OneEnumToHoldThemAll::I64(-64))]
// #[case::char(OneEnumToHoldThemAll::Char('#'))]
// #[case::char(OneEnumToHoldThemAll::Char('\''))]
// #[case::opt_none(OneEnumToHoldThemAll::Option(None))]
// #[case::opt_some(OneEnumToHoldThemAll::Option(Some(true)))]
// #[case::unit_variant(OneEnumToHoldThemAll::UnitVariant)]
// #[case::tupl_variant(OneEnumToHoldThemAll::TupleVariant(-32, false))]
// #[case::stct_variant(OneEnumToHoldThemAll::StructVariant{ uid: 2 })]
// #[case::nwty_variant(
//     OneEnumToHoldThemAll::NewTypeVariant(Person{ name: "Mick".to_string(), uid: 3})
// )]
// #[case::var_tuple(OneEnumToHoldThemAll::VariantTuple((-3, -4)))]
// #[case::var_seq(OneEnumToHoldThemAll::VariantSeq(vec!["xyz".to_string(), "abc".to_string()]))]
// #[case::var_seq_empty(OneEnumToHoldThemAll::VariantSeq(vec![]))]
// fn test_round_trip_opt(#[case] input: OneEnumToHoldThemAll) {
//     let fmt = YamlFormatter::default();
//     let result = to_pretty_string(&input, fmt);
//
//     assert!(result.is_ok(), "Expected Ok result for serialization");
//
//     let actual = from_str::<OneEnumToHoldThemAll>(&result.unwrap());
//
//     assert!(actual.is_ok(), "Expected Ok result for deserialization");
//     assert_eq!(actual.unwrap(), input);
// }

#[test]
fn test_round_trip_opt() {
    let input = OneEnumToHoldThemAll::Option(Some(true));
    let expected = to_string(&input).expect("Enum to string conversion");

    let yaml: Yaml = from_str(&expected).expect("Deserialization successful");
    let actual_str = to_string(&yaml).expect("Reserialization successful");

    assert_eq!(&actual_str, &expected);
}

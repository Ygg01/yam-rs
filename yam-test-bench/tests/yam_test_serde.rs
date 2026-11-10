use rstest::rstest;
use serde::{Deserialize, Serialize};
use yam_serde::{from_str, to_pretty_string, PrettyFormatterConfig};

#[derive(PartialEq, Debug, Serialize, Deserialize)]
struct Person {
    name: String,
    uid: u64,
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
enum OneEnumToHoldThemAll {
    F32(f32),
    F64(f64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Option(Option<bool>),
    UnitVariant,
    TupleVariant(i32, bool),
    StructVariant { uid: u64 },
    NewTypeVariant(Person),
    VariantTuple((i8, i8)),
}

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
#[case::opt_none(OneEnumToHoldThemAll::Option(None))]
#[case::opt_some(OneEnumToHoldThemAll::Option(Some(true)))]
#[case::unit_variant(OneEnumToHoldThemAll::UnitVariant)]
#[case::tupl_variant(OneEnumToHoldThemAll::TupleVariant(-32, false))]
#[case::stct_variant(OneEnumToHoldThemAll::StructVariant{ uid: 2 })]
// #[case::nwty_variant(OneEnumToHoldThemAll::NewTypeVariant(Person { name: "Mick".to_string(), uid: 3}))]
#[case::stct_variant(OneEnumToHoldThemAll::VariantTuple((-3, -4)))]
fn test_round_trip_opt(#[case] input: OneEnumToHoldThemAll) {
    let fmt = PrettyFormatterConfig::default();
    let result = to_pretty_string(&input, fmt);

    assert!(result.is_ok(), "Expected Ok result for serialization");

    let actual = from_str::<OneEnumToHoldThemAll>(&result.unwrap());

    assert!(actual.is_ok(), "Expected Ok result for deserialization");
    assert_eq!(actual.unwrap(), input);
}

#[test]
fn test_round_trip_i16() {
    let initial = OneEnumToHoldThemAll::NewTypeVariant(Person {
        name: "Mick".to_string(),
        uid: 3,
    });
    let fmt = PrettyFormatterConfig::default();
    let result = to_pretty_string(&initial, fmt);

    assert!(result.is_ok(), "Expected Ok result for serialization");

    let actual = from_str::<OneEnumToHoldThemAll>(&result.unwrap());

    assert!(actual.is_ok(), "Expected Ok result for deserialization");
    assert_eq!(actual.unwrap(), initial);
}

#[test]
fn test_round_trip_vec_string() {
    let initial = vec!["xyz".to_string(), "abc".to_string()];
    let fmt = PrettyFormatterConfig::default();
    let result = to_pretty_string(&initial, fmt);

    assert!(result.is_ok(), "Expected Ok result for serialization");

    let actual = from_str::<Vec<String>>(&result.unwrap());

    assert!(actual.is_ok(), "Expected Ok result for deserialization");
    assert_eq!(actual.unwrap(), initial);
}

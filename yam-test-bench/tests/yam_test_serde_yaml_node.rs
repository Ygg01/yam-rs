use rstest::rstest;
use serde::{Deserialize, Serialize};
use std::fmt::Error;
use yam_core::prelude::{Yaml, YamlDocAccess, YamlFormatter};
use yam_core::serde::to_yaml;
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

#[derive(Serialize, Deserialize)]
pub struct Unit;

#[derive(Serialize, Deserialize)]
pub struct TupleStruct(u8, i16);

#[derive(Serialize, Deserialize)]
pub struct Address {
    uid: u32,
    name: String,
}

#[derive(Serialize, Deserialize)]
pub enum TestExample {
    Unit,
    Newtype(TupleStruct),
    Tuple(i16, u32),
    Struct { uid: u64 },
}

#[test]
fn test_yaml_ser_eq() {
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

    let none: Option<f64> = None;
    let yaml: Yaml = to_yaml(none).unwrap();
    assert!(yaml.is_null());

    let none: Option<f64> = Some(3.0f64);
    let yaml: Yaml = to_yaml(none).unwrap();
    assert_eq!(yaml, 3.0f64);

    let vec = vec![2u8, 3u8, 4];
    let yaml: Yaml = to_yaml(&vec).unwrap();
    assert!(yaml.is_sequence());
    assert_eq!(&yaml[0], &vec[0]);
    assert_eq!(&yaml[1], &vec[1]);
    assert_eq!(&yaml[2], &vec[2]);

    let tuple = (3, 5, 7);
    let yaml: Yaml = to_yaml(tuple).unwrap();
    assert!(yaml.is_sequence());
    assert_eq!(yaml[0], tuple.0);
    assert_eq!(yaml[1], tuple.1);
    assert_eq!(yaml[2], tuple.2);

    let unit_struct = Unit;
    let yaml: Yaml = to_yaml(unit_struct).unwrap();
    assert!(yaml.is_mapping());
    assert!(yaml["Unit"].is_null());

    let unit_var = TestExample::Unit;
    let yaml: Yaml = to_yaml(unit_var).unwrap();
    assert!(yaml.is_mapping());
    assert!(yaml["Unit"].is_null());

    let newtype_var = TestExample::Newtype(TupleStruct(2, -3));
    let yaml: Yaml = to_yaml(newtype_var).unwrap();
    assert!(yaml.is_mapping());
    assert_eq!(yaml["Newtype"][0], 2);
    assert_eq!(yaml["Newtype"][1], -3);

    let tuple_struct = TupleStruct(6, -32);
    let yaml: Yaml = to_yaml(tuple_struct).unwrap();
    assert!(yaml.is_sequence());
    assert_eq!(yaml[0], 6);
    assert_eq!(yaml[1], -32);

    let tuple_var = TestExample::Tuple(0, 3);
    let yaml: Yaml = to_yaml(tuple_var).unwrap();
    assert!(yaml.is_mapping());
    assert_eq!(yaml["Tuple"][0], 0);
    assert_eq!(yaml["Tuple"][1], 3);

    let struct_var = TestExample::Struct { uid: 33 };
    let yaml: Yaml = to_yaml(struct_var).unwrap();
    assert!(yaml.is_mapping());
    assert_eq!(yaml["Struct"]["uid"], 33);

    let strct = Address {
        uid: 4343,
        name: "Compton road 3233".to_string(),
    };
    let yaml: Yaml = to_yaml(&strct).unwrap();
    assert!(yaml.is_mapping());
    assert_eq!(yaml["uid"], &strct.uid);
    assert_eq!(yaml["name"], &strct.name);

    let chr = '#';
    let yaml: Yaml = to_yaml(chr).unwrap();
    assert_eq!(yaml, chr);
}

#[test]
fn test_borrow() {
    #[derive(Deserialize)]
    struct Ex {
        optional: Option<char>,
    }

    let ex = Ex {
        optional: Some('z'),
    };
    // let yaml: Yaml = to_yaml(ex).unwrap();
    // assert_eq!(yaml["optional"], );
}

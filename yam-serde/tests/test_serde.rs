pub use serde;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Ex {
    a: String,
    b: Vec<f32>,
}

#[test]
fn test_example() {
    let ex = r#"{ a: "x",  b: [2.0, 3.1, -1.2] }"#;
    let deserialized: Ex = yam_serde::from_str(ex).unwrap();
    assert_eq!(deserialized.a, "x");
    assert_eq!(deserialized.b, vec![2.0, 3.1, -1.2]);
}

#[test]
fn test_deserialize_i8() {
    let input = r#"3"#;
    let deserialized: i8 = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized, 3i8);
}

#[test]
fn test_deserialize_u8() {
    let input = r#"3"#;
    let deserialized: u8 = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized, 3u8);
}

#[test]
fn test_deserialize_i16() {
    let input = r#"3"#;
    let deserialized: i16 = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized, 3i16);
}

#[test]
fn test_deserialize_u16() {
    let input = r#"3"#;
    let deserialized: u16 = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized, 3u16);
}

#[test]
fn test_deserialize_i32() {
    let input = r#"3"#;
    let deserialized: i32 = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized, 3i32);
}

#[test]
fn test_deserialize_u32() {
    let input = r#"3"#;
    let deserialized: u32 = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized, 3u32);
}

#[test]
fn test_deserialize_i64() {
    let input = r#"3"#;
    let deserialized: i64 = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized, 3i64);
}

#[test]
fn test_deserialize_u64() {
    let input = r#"3"#;
    let deserialized: u64 = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized, 3u64);
}

#[test]
fn test_deserialize_list_i64() {
    let input = r#"[3]"#;
    let deserialized: Vec<i64> = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized, vec![3]);
}

#[test]
fn test_deserialize_list_i32() {
    let input = r#"[3, 4]"#;
    let deserialized: Vec<i32> = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized, vec![3, 4]);
}

#[test]
fn test_deserialize_list_u16() {
    let input = r#"[3, 4, 4]"#;
    let deserialized: Vec<u16> = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized, vec![3, 4, 4]);
}

#[test]
fn test_deserialize_list_i16() {
    let input = r#"[3, 4, 4]"#;
    let deserialized: Vec<i16> = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized, vec![3, 4, 4]);
}

#[test]
fn test_deserialize_list_u8() {
    let input = r#"[3, 4, 4]"#;
    let deserialized: Vec<u8> = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized, vec![3, 4, 4]);
}

#[test]
fn test_deserialize_list_i8() {
    let input = r#"[3, 4, 4]"#;
    let deserialized: Vec<i8> = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized, vec![3, 4, 4]);
}

#[test]
fn test_enum() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum E {
        Unit,
        Newtype(u32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    let j = r#""Unit""#;
    let expected = E::Unit;
    assert_eq!(expected, yam_serde::from_str(j).unwrap());
    //
    // let j = r#"{"Newtype":1}"#;
    // let expected = E::Newtype(1);
    // assert_eq!(expected, yam_serde::from_str(j).unwrap());
    //
    // let j = r#"{"Tuple":[1,2]}"#;
    // let expected = E::Tuple(1, 2);
    // assert_eq!(expected, yam_serde::from_str(j).unwrap());
    //
    // let j = r#"{"Struct":{"a":1}}"#;
    // let expected = E::Struct { a: 1 };
    // assert_eq!(expected, yam_serde::from_str(j).unwrap());
}

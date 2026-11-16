pub use serde;
use serde::Deserialize;

#[test]
fn test_nested_collections() {
    #[derive(Deserialize, Debug)]
    pub struct Ex {
        a: String,
        b: Vec<f32>,
    }

    let ex = r#"{ a: "x",  b: [2.0, 3.1, -1.2] }"#;
    let deserialized: Ex = yam_serde::from_str(ex).unwrap();
    assert_eq!(deserialized.a, "x");
    assert_eq!(deserialized.b, vec![2.0, 3.1, -1.2]);
}

#[test]
fn test_schemas_null() {
    #[derive(Deserialize, Debug)]
    pub struct Ex {
        a: Option<String>,
    }
    let input = r"a: !!null";
    let deserialized: Ex = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized.a, None);

    let input = r"a: !!null null";
    let deserialized2: Ex = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized2.a, None);
}

#[test]
fn test_schemas_float() {
    #[derive(Deserialize, Debug)]
    pub struct Ex {
        a: Option<f32>,
    }
    let input = r"a: 3";
    let deserialized: Ex = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized.a, Some(3.0));
}

#[test]
fn test_enum() {
    #[derive(Deserialize, PartialEq, Debug)]
    enum E {
        Unit,
        Newtype(f32),
        Tuple(u32, u32),
        Struct { a: u32 },
    }

    let j = r#"{ Unit }"#;
    let expected = E::Unit;
    assert_eq!(expected, yam_serde::from_str(j).unwrap());

    let j = r#"{ Newtype: 1 }"#;
    let expected = E::Newtype(1.0);
    assert_eq!(expected, yam_serde::from_str(j).unwrap());

    let j = r#"{"Tuple":[1,2]}"#;
    let expected = E::Tuple(1, 2);
    assert_eq!(expected, yam_serde::from_str(j).unwrap());

    let j = r#"{"Struct":{"a":1}}"#;
    let expected = E::Struct { a: 1 };
    assert_eq!(expected, yam_serde::from_str(j).unwrap());
}

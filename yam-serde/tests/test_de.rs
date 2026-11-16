pub use serde;
use serde::Deserialize;
use yam_serde::de::DeYamlError;

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

#[test]
fn test_wrong_de_errors() {
    let evil_input = "true";
    assert_eq!(
        yam_serde::from_str::<i8>(evil_input).unwrap_err(),
        DeYamlError::Custom(r#"invalid type: boolean `true`, expected i8"#.to_string()),
    );
    assert_eq!(
        yam_serde::from_str::<i16>(evil_input).unwrap_err(),
        DeYamlError::Custom(r#"invalid type: boolean `true`, expected i16"#.to_string()),
    );
    assert_eq!(
        yam_serde::from_str::<i32>(evil_input).unwrap_err(),
        DeYamlError::Custom(r#"invalid type: boolean `true`, expected i32"#.to_string()),
    );
    assert_eq!(
        yam_serde::from_str::<i64>(evil_input).unwrap_err(),
        DeYamlError::Custom(r#"invalid type: boolean `true`, expected i64"#.to_string()),
    );
    assert_eq!(
        yam_serde::from_str::<u8>(evil_input).unwrap_err(),
        DeYamlError::Custom(r#"invalid type: boolean `true`, expected u8"#.to_string()),
    );
    assert_eq!(
        yam_serde::from_str::<u16>(evil_input).unwrap_err(),
        DeYamlError::Custom(r#"invalid type: boolean `true`, expected u16"#.to_string()),
    );
    assert_eq!(
        yam_serde::from_str::<u32>(evil_input).unwrap_err(),
        DeYamlError::Custom(r#"invalid type: boolean `true`, expected u32"#.to_string()),
    );
    assert_eq!(
        yam_serde::from_str::<u64>(evil_input).unwrap_err(),
        DeYamlError::Custom(r#"invalid type: boolean `true`, expected u64"#.to_string()),
    );
    assert_eq!(
        yam_serde::from_str::<f32>(evil_input).unwrap_err(),
        DeYamlError::Custom(r#"invalid type: boolean `true`, expected f32"#.to_string()),
    );
    assert_eq!(
        yam_serde::from_str::<f64>(evil_input).unwrap_err(),
        DeYamlError::Custom(r#"invalid type: boolean `true`, expected f64"#.to_string()),
    );
}

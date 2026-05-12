pub use serde;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Ex {
    a: String,
    b: String,
}

#[test]
fn test_deserialize_scalar() {
    let input = r#"3"#;
    let deserialized: i32 = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized, 3);
}

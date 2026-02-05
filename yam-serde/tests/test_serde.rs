pub use serde;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Ex {
    a: String,
    b: String,
}

#[test]
fn test_deserialize() {
    let input = r#"
    a: "hello"
    b: "world"
    "#;
    let deserialized: Ex = yam_serde::from_str(input).unwrap();
    assert_eq!(deserialized.a, "hello");
    assert_eq!(deserialized.b, "world");
}

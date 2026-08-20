use yam_serde::{PrettyFormatterConfig, from_str, to_pretty_string};

#[test]
fn test_round_trip_opt() {
    let initial: Option<i32> = None;
    let fmt = PrettyFormatterConfig::default();
    let result = to_pretty_string(&initial, fmt);

    assert!(result.is_ok(), "Expected Ok result for serialization");

    let actual = from_str::<Option<i32>>(&result.unwrap());

    assert!(actual.is_ok(), "Expected Ok result for deserialization");
    assert_eq!(actual.unwrap(), initial);
}

#[test]
fn test_round_trip_i16() {
    let initial = -231i16;
    let fmt = PrettyFormatterConfig::default();
    let result = to_pretty_string(&initial, fmt);

    assert!(result.is_ok(), "Expected Ok result for serialization");

    let actual = from_str::<i16>(&result.unwrap());

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

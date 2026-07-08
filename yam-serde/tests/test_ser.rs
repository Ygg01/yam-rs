use core::fmt::Error;
use yam_serde::ser::PrettyFormatterConfig;
use yam_serde::to_pretty_string;

fn assert_eq_strings(result: Result<String, Error>, correct: &str) {
    assert_eq!(result, Ok(correct.to_string()));
}

#[test]
fn test_null_fmt() {
    let x: Option<i32> = None;
    let fmt = PrettyFormatterConfig::default();
    let result = to_pretty_string(&x, fmt);

    assert_eq_strings(result, "");

    let x: Option<i32> = None;
    let fmt = PrettyFormatterConfig::pretty();
    let result = to_pretty_string(&x, fmt);

    assert_eq_strings(result, "null");
}

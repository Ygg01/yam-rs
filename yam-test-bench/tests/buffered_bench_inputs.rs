use rstest::rstest;
use yam_core::parsing::{BufferedBytesSource, Parser};

const NESTED_YAML: &str = include_str!("../benches/nested.yaml");
const BIG_TEXT: &str = include_str!("../benches/big_text.yaml");
const BIG_QUOTE_TEXT_YAML: &str = include_str!("../benches/big_quote_text.yaml");
const SMALL_OBJECTS_YAML: &str = include_str!("../benches/small_objects.yaml");

fn assert_same_events(input: &str) {
    let expected: Vec<_> = Parser::new_from_str(input).collect();
    let actual: Vec<_> = Parser::new(BufferedBytesSource::new(input)).collect();
    assert!(
        expected.iter().all(Result::is_ok),
        "bench input must parse with StrSource"
    );
    assert_eq!(expected.len(), actual.len());
    for (i, (exp, act)) in expected.iter().zip(actual.iter()).enumerate() {
        assert_eq!(exp, act, "event #{i} differs");
    }
}

// Bench inputs are checked out with either LF or CRLF depending on the platform, so test both.
#[rstest]
#[case::nested(NESTED_YAML)]
#[case::big_text(BIG_TEXT)]
#[case::big_quote_text(BIG_QUOTE_TEXT_YAML)]
#[case::small_objects(SMALL_OBJECTS_YAML)]
fn buffered_matches_str_source(#[case] input: &str) {
    let lf = input.replace("\r\n", "\n");
    let crlf = lf.replace('\n', "\r\n");
    assert_same_events(&lf);
    assert_same_events(&crlf);
}

#[test]
fn lone_cr_is_line_break() {
    let input = "a: b\rc: d\r";
    assert_same_events(input);
    assert_eq!(
        Parser::new_from_str(input).collect::<Vec<_>>(),
        Parser::new_from_str("a: b\nc: d\n").collect::<Vec<_>>()
    );
}

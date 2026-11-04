use rstest::rstest;
use yam_core::node::{SpannedYaml, Yaml};
use yam_core::prelude::{YamlDocAccess, YamlLoader};

const SIMPLE_SEQ: &str = "[a, b]";
const SIMPLE_MAP: &str = "{0: a, 1: b}";
const SIMPLE_SEQ_BOOL: &str = "[true, false]";
const SIMPLE_SEQ_TAG: &str = "[!!null null, !!int 3]";
const NESTED_SEQ: &str = "[[], {}]";
const ALIAS_SEQ: &str = "[&a 3.0, *a]";

#[rstest]
#[case::span_flow_map(SIMPLE_SEQ)]
#[case::span_flow_map(SIMPLE_MAP)]
#[case::span_flow_map(SIMPLE_SEQ_BOOL)]
#[case::span_flow_map(SIMPLE_SEQ_TAG)]
#[case::span_flow_map(NESTED_SEQ)]
#[case::span_flow_map(ALIAS_SEQ)]
fn test_span(#[case] input: &str) {
    let mut spanned_yaml = YamlLoader::<SpannedYaml>::load_single(input).unwrap();
    let mut borrowed_yaml = YamlLoader::<Yaml>::load_single(input).unwrap();

    assert_eq!(spanned_yaml[0].get_type(), borrowed_yaml[0].get_type());
    assert_eq!(spanned_yaml[0].get_tag(), borrowed_yaml[0].get_tag());
    assert_eq!(spanned_yaml[0].is_alias(), borrowed_yaml[0].is_alias());
    assert_eq!(
        spanned_yaml[0].is_bad_value(),
        borrowed_yaml[0].is_bad_value()
    );
    assert_eq!(spanned_yaml[0].is_bool(), borrowed_yaml[0].is_bool());
    assert_eq!(
        spanned_yaml[0].is_collection(),
        borrowed_yaml[0].is_collection()
    );
    assert_eq!(
        spanned_yaml[0].is_floating_point(),
        borrowed_yaml[0].is_floating_point()
    );
    assert_eq!(spanned_yaml[0].is_integer(), borrowed_yaml[0].is_integer());
    assert_eq!(spanned_yaml[0].is_mapping(), borrowed_yaml[0].is_mapping());
    assert_eq!(
        spanned_yaml[0].is_non_empty_collection(),
        borrowed_yaml[0].is_non_empty_collection()
    );
    assert_eq!(spanned_yaml[0].is_null(), borrowed_yaml[0].is_null());
    assert_eq!(
        spanned_yaml[0].is_sequence(),
        borrowed_yaml[0].is_sequence()
    );
    assert_eq!(spanned_yaml[0].is_string(), borrowed_yaml[0].is_string());

    assert_eq!(spanned_yaml[1].get_tag(), borrowed_yaml[1].get_tag());
    assert_eq!(spanned_yaml[1].get_type(), borrowed_yaml[1].get_type());
    assert_eq!(spanned_yaml[1].as_bool(), borrowed_yaml[1].as_bool());
    assert_eq!(
        spanned_yaml[1].as_bool_mut(),
        borrowed_yaml[1].as_bool_mut()
    );
    assert_eq!(spanned_yaml[1].as_i64(), borrowed_yaml[1].as_i64());
    assert_eq!(spanned_yaml[1].as_i64_mut(), borrowed_yaml[1].as_i64_mut());
    assert_eq!(spanned_yaml[1].as_f64(), borrowed_yaml[1].as_f64());
    assert_eq!(spanned_yaml[1].as_f64_mut(), borrowed_yaml[1].as_f64_mut());
    assert_eq!(spanned_yaml[1].as_str(), borrowed_yaml[1].as_str());
    assert_eq!(spanned_yaml[1].as_str_mut(), borrowed_yaml[1].as_str_mut());
}

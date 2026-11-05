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
        spanned_yaml[1].as_sequence().is_ok(),
        borrowed_yaml[1].as_sequence().is_ok()
    );
    assert_eq!(
        spanned_yaml[1].as_sequence_mut().is_ok(),
        borrowed_yaml[1].as_sequence_mut().is_ok()
    );
    assert_eq!(
        spanned_yaml[1].as_mapping().is_ok(),
        borrowed_yaml[1].as_mapping().is_ok()
    );
    assert_eq!(
        spanned_yaml[1].as_mapping_mut().is_ok(),
        borrowed_yaml[1].as_mapping_mut().is_ok()
    );
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

#[test]
fn test_from_methods_scalar() {
    let input = 3.0;
    let yaml = Yaml::from(input);
    let spanned_yaml = SpannedYaml::from(input);
    assert_eq!(yaml.as_f64(), spanned_yaml.as_f64());

    let input = 3.0f32;
    let yaml = Yaml::from(input);
    let spanned_yaml = SpannedYaml::from(input);
    assert_eq!(yaml.as_f64(), spanned_yaml.as_f64());

    let input = 1i8;
    let yaml = Yaml::from(input);
    let spanned_yaml = SpannedYaml::from(input);
    assert_eq!(yaml.as_i64(), spanned_yaml.as_i64());

    let input = 2i16;
    let yaml = Yaml::from(input);
    let spanned_yaml = SpannedYaml::from(input);
    assert_eq!(yaml.as_i64(), spanned_yaml.as_i64());

    let input = 3i32;
    let yaml = Yaml::from(input);
    let spanned_yaml = SpannedYaml::from(input);
    assert_eq!(yaml.as_i64(), spanned_yaml.as_i64());

    let input = 4i64;
    let yaml = Yaml::from(input);
    let spanned_yaml = SpannedYaml::from(input);
    assert_eq!(yaml.as_i64(), spanned_yaml.as_i64());

    let input = "1";
    let yaml = Yaml::from(input);
    let spanned_yaml = SpannedYaml::from(input);
    assert_eq!(yaml.as_str(), spanned_yaml.as_str());

    let input = true;
    let mut yaml = Yaml::from(input);
    let mut spanned_yaml = SpannedYaml::from(input);
    assert_eq!(
        yaml.as_bool_mut().unwrap_or(&mut false),
        spanned_yaml.as_bool_mut().unwrap_or(&mut false)
    );

    let a: Yaml = Yaml::null();
    let b: SpannedYaml = SpannedYaml::null();
    assert_eq!(a.is_null(), b.is_null())
}

#[test]
fn test_from_sequence() {
    let input = vec![true, false];
    let yaml = Yaml::from(input.clone());
    let spanned_yaml = SpannedYaml::from(input.clone());
    for (yaml_el, span_el) in yaml.sequence().iter().zip(spanned_yaml.sequence()) {
        assert_eq!(yaml_el.as_bool(), span_el.as_bool());
    }

    let input = vec!["zip", "zoom"];
    let yaml = Yaml::from(input.clone());
    let spanned_yaml = SpannedYaml::from(input.clone());
    if let (Ok(x), Ok(y)) = (yaml.as_sequence(), spanned_yaml.as_sequence()) {
        for (y_el, s_el) in x.iter().zip(y) {
            assert_eq!(y_el.as_str(), s_el.as_str());
        }
    }

    let input = vec![2i8, 1i8];
    let yaml = Yaml::from(input.clone());
    let spanned_yaml = SpannedYaml::from(input.clone());
    if let (Ok(x), Ok(y)) = (yaml.as_sequence(), spanned_yaml.as_sequence()) {
        for (y_el, s_el) in x.iter().zip(y) {
            assert_eq!(y_el.as_i64(), s_el.as_i64());
        }
    }

    let input = vec![2i16, 1i16];
    let yaml = Yaml::from(input.clone());
    let spanned_yaml = SpannedYaml::from(input.clone());
    if let (Ok(x), Ok(y)) = (yaml.as_sequence(), spanned_yaml.as_sequence()) {
        for (y_el, s_el) in x.iter().zip(y) {
            assert_eq!(y_el.as_i64(), s_el.as_i64());
        }
    }

    let input = vec![2i32, 1i32];
    let yaml = Yaml::from(input.clone());
    let spanned_yaml = SpannedYaml::from(input.clone());
    if let (Ok(x), Ok(y)) = (yaml.as_sequence(), spanned_yaml.as_sequence()) {
        for (y_el, s_el) in x.iter().zip(y) {
            assert_eq!(y_el.as_i64(), s_el.as_i64());
        }
    }

    let input = vec![2i64, 1i64];
    let yaml = Yaml::from(input.clone());
    let spanned_yaml = SpannedYaml::from(input.clone());
    if let (Ok(x), Ok(y)) = (yaml.as_sequence(), spanned_yaml.as_sequence()) {
        for (y_el, s_el) in x.iter().zip(y) {
            assert_eq!(y_el.as_i64(), s_el.as_i64());
        }
    }

    let input = vec![2f32, 1f32];
    let yaml = Yaml::from(input.clone());
    let spanned_yaml = SpannedYaml::from(input.clone());
    if let (Ok(x), Ok(y)) = (yaml.as_sequence(), spanned_yaml.as_sequence()) {
        for (y_el, s_el) in x.iter().zip(y) {
            assert_eq!(y_el.as_f64(), s_el.as_f64());
        }
    }

    let input = vec![2f64, 1f64];
    let yaml = Yaml::from(input.clone());
    let spanned_yaml = SpannedYaml::from(input.clone());
    if let (Ok(x), Ok(y)) = (yaml.as_sequence(), spanned_yaml.as_sequence()) {
        for (y_el, s_el) in x.iter().zip(y) {
            assert_eq!(y_el.as_f64(), s_el.as_f64());
        }
    }

    let yaml_map = Yaml::from(vec![(Yaml::from("3"), Yaml::null())]);
    let span_map = SpannedYaml::from(vec![(SpannedYaml::from("3"), SpannedYaml::null())]);
    for (yam_entry, span_entry) in yaml_map.mapping().iter().zip(span_map.mapping()) {
        assert_eq!(yam_entry.key.as_i64(), span_entry.key.as_i64());
        assert_eq!(yam_entry.value.is_null(), span_entry.value.is_null());
    }
    assert_eq!(&yaml_map["3"].is_null(), &span_map["3"].is_null());
}

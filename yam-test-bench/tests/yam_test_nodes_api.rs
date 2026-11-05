use rstest::rstest;
use std::ops::{Index, IndexMut};
use yam_core::node::{SpannedYaml, Yaml};
use yam_core::prelude::{YamlDocAccess, YamlLoader};

const SIMPLE_SEQ: &str = "[a, b]";
const SIMPLE_MAP: &str = "{0: a, 1: b}";
const SIMPLE_SEQ_BOOL: &str = "[true, false]";
const SIMPLE_SEQ_TAG: &str = r#"
%TAG ! !foo
---
[! 323, ! 44]"#;

const NESTED_SEQ: &str = "[[], {}]";
const ALIAS_SEQ: &str = "[&a 3.0, *a]";

#[rstest]
#[case::span_seq(SIMPLE_SEQ)]
#[case::span_map(SIMPLE_MAP)]
#[case::span_seq_bool(SIMPLE_SEQ_BOOL)]
#[case::span_seq_tag(SIMPLE_SEQ_TAG)]
#[case::span_nested_seq(NESTED_SEQ)]
#[case::span_alias_seq(ALIAS_SEQ)]
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
    assert_eq!(a.is_null(), b.is_null());

    let a: Yaml = Yaml::from("aa");
    let b: SpannedYaml = SpannedYaml::from("aa");
    assert_eq!(a.into_string(), b.into_string())
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
    assert_eq!(
        yaml_map.is_non_empty_collection(),
        span_map.is_non_empty_collection()
    );
    for (yam_entry, span_entry) in yaml_map.mapping().iter().zip(span_map.mapping()) {
        assert_eq!(yam_entry.key.as_i64(), span_entry.key.as_i64());
        assert_eq!(yam_entry.value.is_null(), span_entry.value.is_null());
    }
    assert_eq!(&yaml_map["3"].is_null(), &span_map["3"].is_null());

    let mut yaml_seq = Yaml::from(vec![Yaml::from(3), Yaml::from("aaa")]);
    let mut span_seq = SpannedYaml::from(vec![SpannedYaml::from(3), SpannedYaml::from("aaa")]);

    if let Ok(x) = yaml_seq.as_sequence_mut() {
        x.clear();
    }
    if let Ok(x) = span_seq.as_sequence_mut() {
        x.clear();
    }

    assert!(!yaml_seq.is_non_empty_collection());
    assert!(!span_seq.is_non_empty_collection());

    let mut yaml_map = Yaml::from(vec![(Yaml::from("3"), Yaml::from(true))]);
    let mut span_map = SpannedYaml::from(vec![(SpannedYaml::from("3"), SpannedYaml::from(true))]);
    yaml_map["3"] = Yaml::from(false);
    span_map["3"] = SpannedYaml::from(false);

    assert_eq!(yaml_map["3"].as_bool(), Some(false));
    assert_eq!(span_map["3"].as_bool(), Some(false));

    assert_eq!(yaml_map.into_string(), None);
    assert_eq!(span_map.into_string(), None);
}

enum TestType {
    Seq,
    SeqMut,
    Map,
    MapMut,
    Ind,
    IndMut,
    IndStr,
    IndStrMut,
}

enum YamlType {
    Yaml,
    Span,
}

#[rstest]
#[case(YamlType::Yaml, TestType::Seq)]
#[case(YamlType::Yaml, TestType::SeqMut)]
#[case(YamlType::Yaml, TestType::Map)]
#[case(YamlType::Yaml, TestType::MapMut)]
#[case(YamlType::Yaml, TestType::Ind)]
#[case(YamlType::Yaml, TestType::IndMut)]
#[case(YamlType::Yaml, TestType::IndStr)]
#[case(YamlType::Yaml, TestType::IndStrMut)]
#[case(YamlType::Span, TestType::Seq)]
#[case(YamlType::Span, TestType::SeqMut)]
#[case(YamlType::Span, TestType::Map)]
#[case(YamlType::Span, TestType::MapMut)]
#[case(YamlType::Span, TestType::Ind)]
#[case(YamlType::Span, TestType::IndMut)]
#[case(YamlType::Span, TestType::IndStr)]
#[case(YamlType::Span, TestType::IndStrMut)]
#[should_panic]
fn test_should_panic(#[case] yaml: YamlType, #[case] test_type: TestType) {
    match (yaml, test_type) {
        (YamlType::Yaml, TestType::Seq) => {
            Yaml::from(3).sequence();
        }
        (YamlType::Yaml, TestType::SeqMut) => {
            Yaml::from(3).sequence_mut();
        }
        (YamlType::Yaml, TestType::Map) => {
            Yaml::from(3).mapping();
        }
        (YamlType::Yaml, TestType::MapMut) => {
            Yaml::from(3).mapping_mut();
        }
        (YamlType::Yaml, TestType::Ind) => {
            Yaml::from(3).index(0);
        }
        (YamlType::Yaml, TestType::IndMut) => {
            Yaml::from(3).index_mut(0);
        }
        (YamlType::Yaml, TestType::IndStr) => {
            Yaml::from(3).index("0");
        }
        (YamlType::Yaml, TestType::IndStrMut) => {
            Yaml::from(3).index_mut("0");
        }
        (YamlType::Span, TestType::Seq) => {
            SpannedYaml::from(3).sequence();
        }
        (YamlType::Span, TestType::SeqMut) => {
            SpannedYaml::from(3).sequence_mut();
        }
        (YamlType::Span, TestType::Map) => {
            SpannedYaml::from(3).mapping();
        }
        (YamlType::Span, TestType::MapMut) => {
            SpannedYaml::from(3).mapping_mut();
        }
        (YamlType::Span, TestType::Ind) => {
            SpannedYaml::from(3).index(0);
        }
        (YamlType::Span, TestType::IndMut) => {
            SpannedYaml::from(3).index_mut(0);
        }
        (YamlType::Span, TestType::IndStr) => {
            SpannedYaml::from(3).index("0");
        }
        (YamlType::Span, TestType::IndStrMut) => {
            SpannedYaml::from(3).index_mut("0");
        }
    };
}

#[rstest]
#[case::seq_empt(true, true)]
#[case::seq_full(true, false)]
#[case::map_empt(false, true)]
#[case::map_full(false, false)]
fn test_into(#[case] is_seq: bool, #[case] is_empty: bool) {
    let (yaml, span) = match (is_seq, is_empty) {
        (_, true) => (Yaml::from(3), SpannedYaml::from(3)),
        (true, false) => (Yaml::from(vec![2]), SpannedYaml::from(vec![2])),
        (false, false) => (Yaml::from(vec![(2, 3)]), SpannedYaml::from(vec![(2, 3)])),
    };
    if is_seq {
        let x = yaml
            .into_sequence()
            .map(|x| x[0].as_i64().unwrap_or(0))
            .unwrap_or(0i64);
        let y = span
            .into_sequence()
            .map(|x| x[0].as_i64().unwrap_or(0))
            .unwrap_or(0i64);
        assert_eq!(x, y);
    } else {
        let x = yaml
            .into_mapping()
            .map(|x| x[0].value.as_i64().unwrap_or(0))
            .unwrap_or(0i64);
        let y = span
            .into_mapping()
            .map(|x| x[0].value.as_i64().unwrap_or(0))
            .unwrap_or(0i64);
        assert_eq!(x, y);
    }
}

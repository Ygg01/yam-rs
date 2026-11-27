use rstest::rstest;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use yam_core::node::{SpanYam, Yaml, YamlScalar};
use yam_core::parsing::Tag;
use yam_core::prelude::{YamLoader, YamlDocAccess};
use yam_core::serde::to_yaml;

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
    let mut span_yaml = YamLoader::<SpanYam>::load_single(input).unwrap();
    let mut borrowed_yaml = YamLoader::<Yaml>::load_single(input).unwrap();

    assert_eq!(span_yaml[0].get_type(), borrowed_yaml[0].get_type());
    assert_eq!(span_yaml[0].get_tag(), borrowed_yaml[0].get_tag());
    assert_eq!(span_yaml[0].is_alias(), borrowed_yaml[0].is_alias());
    assert_eq!(span_yaml[0].is_bad_value(), borrowed_yaml[0].is_bad_value());
    assert_eq!(span_yaml[0].is_bool(), borrowed_yaml[0].is_bool());
    assert_eq!(
        span_yaml[0].is_collection(),
        borrowed_yaml[0].is_collection()
    );
    assert_eq!(
        span_yaml[0].is_floating_point(),
        borrowed_yaml[0].is_floating_point()
    );
    assert_eq!(span_yaml[0].is_integer(), borrowed_yaml[0].is_integer());
    assert_eq!(span_yaml[0].is_mapping(), borrowed_yaml[0].is_mapping());
    assert_eq!(
        span_yaml[0].is_non_empty_collection(),
        borrowed_yaml[0].is_non_empty_collection()
    );
    assert_eq!(span_yaml[0].is_null(), borrowed_yaml[0].is_null());
    assert_eq!(span_yaml[0].is_sequence(), borrowed_yaml[0].is_sequence());
    assert_eq!(span_yaml[0].is_string(), borrowed_yaml[0].is_string());

    assert_eq!(span_yaml[1].get_tag(), borrowed_yaml[1].get_tag());
    assert_eq!(span_yaml[1].get_type(), borrowed_yaml[1].get_type());
    assert_eq!(span_yaml[1].as_bool(), borrowed_yaml[1].as_bool());
    assert_eq!(
        span_yaml[1].as_sequence().is_ok(),
        borrowed_yaml[1].as_sequence().is_ok()
    );
    assert_eq!(
        span_yaml[1].as_sequence_mut().is_ok(),
        borrowed_yaml[1].as_sequence_mut().is_ok()
    );
    assert_eq!(
        span_yaml[1].as_mapping().is_ok(),
        borrowed_yaml[1].as_mapping().is_ok()
    );
    assert_eq!(
        span_yaml[1].as_mapping_mut().is_ok(),
        borrowed_yaml[1].as_mapping_mut().is_ok()
    );
    assert_eq!(span_yaml[1].as_bool_mut(), borrowed_yaml[1].as_bool_mut());
    assert_eq!(span_yaml[1].as_i64(), borrowed_yaml[1].as_i64());
    assert_eq!(span_yaml[1].as_i64_mut(), borrowed_yaml[1].as_i64_mut());
    assert_eq!(span_yaml[1].as_f64(), borrowed_yaml[1].as_f64());
    assert_eq!(span_yaml[1].as_f64_mut(), borrowed_yaml[1].as_f64_mut());
    assert_eq!(span_yaml[1].as_str(), borrowed_yaml[1].as_str());
    assert_eq!(span_yaml[1].as_str_mut(), borrowed_yaml[1].as_str_mut());
}

#[test]
fn test_from_methods_scalar() {
    let input = 3.0;
    let yaml = Yaml::from(input);
    let span_yaml = SpanYam::from(input);
    assert_eq!(yaml.as_f64(), span_yaml.as_f64());
    assert_eq!(yaml, input);

    let input = 3.0f32;
    let yaml = Yaml::from(input);
    let span_yaml = SpanYam::from(input);
    assert_eq!(yaml.as_f64(), span_yaml.as_f64());
    assert_eq!(yaml, input);

    let input = 1i8;
    let yaml = Yaml::from(input);
    let span_yaml = SpanYam::from(input);
    assert_eq!(yaml.as_i64(), span_yaml.as_i64());
    assert_eq!(yaml, input);

    let input = 2i16;
    let yaml = Yaml::from(input);
    let span_yaml = SpanYam::from(input);
    assert_eq!(yaml.as_i64(), span_yaml.as_i64());
    assert_eq!(yaml, input);

    let input = 3i32;
    let yaml = Yaml::from(input);
    let span_yaml = SpanYam::from(input);
    assert_eq!(yaml.as_i64(), span_yaml.as_i64());
    assert_eq!(yaml, input);

    let input = 4i64;
    let yaml = Yaml::from(input);
    let span_yaml = SpanYam::from(input);
    assert_eq!(yaml.as_i64(), span_yaml.as_i64());
    assert_eq!(yaml, input);

    let input = "1";
    let yaml = Yaml::from(input);
    let span_yaml = SpanYam::from(input);
    assert_eq!(yaml.as_str(), span_yaml.as_str());
    assert_eq!(yaml, input);

    let input = true;
    let mut yaml = Yaml::from(input);
    let mut span_yaml = SpanYam::from(input);
    assert_eq!(
        yaml.as_bool_mut().unwrap_or(&mut false),
        span_yaml.as_bool_mut().unwrap_or(&mut false)
    );
    assert_eq!(yaml, input);

    let a: Yaml = Yaml::null();
    let b: SpanYam = SpanYam::null();
    assert_eq!(a.is_null(), b.is_null());

    let a: Yaml = Yaml::from("aa");
    let b: SpanYam = SpanYam::from("aa");
    assert_eq!(a, "aa");
    assert_eq!(a.into_string(), b.into_string());
}

#[test]
fn test_from_sequence() {
    let input = vec![true, false];
    let yaml = Yaml::from(input.clone());
    let span_yaml = SpanYam::from(input.clone());
    unsafe {
        for (yaml_el, span_el) in yaml.sequence().iter().zip(span_yaml.sequence()) {
            assert_eq!(yaml_el.as_bool(), span_el.as_bool());
        }
    }

    let input = vec!["zip", "zoom"];
    let yaml = Yaml::from(input.clone());
    let span_yaml = SpanYam::from(input.clone());
    if let (Ok(x), Ok(y)) = (yaml.as_sequence(), span_yaml.as_sequence()) {
        for (y_el, s_el) in x.iter().zip(y) {
            assert_eq!(y_el.as_str(), s_el.as_str());
        }
    }

    let input = vec![2i8, 1i8];
    let yaml = Yaml::from(input.clone());
    let span_yaml = SpanYam::from(input.clone());
    if let (Ok(x), Ok(y)) = (yaml.as_sequence(), span_yaml.as_sequence()) {
        for (y_el, s_el) in x.iter().zip(y) {
            assert_eq!(y_el.as_i64(), s_el.as_i64());
        }
    }

    let input = vec![2i16, 1i16];
    let yaml = Yaml::from(input.clone());
    let span_yaml = SpanYam::from(input.clone());
    if let (Ok(x), Ok(y)) = (yaml.as_sequence(), span_yaml.as_sequence()) {
        for (y_el, s_el) in x.iter().zip(y) {
            assert_eq!(y_el.as_i64(), s_el.as_i64());
        }
    }

    let input = vec![2i32, 1i32];
    let yaml = Yaml::from(input.clone());
    let span_yaml = SpanYam::from(input.clone());
    if let (Ok(x), Ok(y)) = (yaml.as_sequence(), span_yaml.as_sequence()) {
        for (y_el, s_el) in x.iter().zip(y) {
            assert_eq!(y_el.as_i64(), s_el.as_i64());
        }
    }

    let input = vec![2i64, 1i64];
    let yaml = Yaml::from(input.clone());
    let span_yaml = SpanYam::from(input.clone());
    if let (Ok(x), Ok(y)) = (yaml.as_sequence(), span_yaml.as_sequence()) {
        for (y_el, s_el) in x.iter().zip(y) {
            assert_eq!(y_el.as_i64(), s_el.as_i64());
        }
    }

    let input = vec![2f32, 1f32];
    let yaml = Yaml::from(input.clone());
    let span_yaml = SpanYam::from(input.clone());
    if let (Ok(x), Ok(y)) = (yaml.as_sequence(), span_yaml.as_sequence()) {
        for (y_el, s_el) in x.iter().zip(y) {
            assert_eq!(y_el.as_f64(), s_el.as_f64());
        }
    }

    let input = vec![2f64, 1f64];
    let yaml = Yaml::from(input.clone());
    let span_yaml = SpanYam::from(input.clone());
    if let (Ok(x), Ok(y)) = (yaml.as_sequence(), span_yaml.as_sequence()) {
        for (y_el, s_el) in x.iter().zip(y) {
            assert_eq!(y_el.as_f64(), s_el.as_f64());
        }
    }

    let yaml_map = Yaml::from(vec![(Yaml::from("3"), Yaml::null())]);
    let span_map = SpanYam::from(vec![(SpanYam::from("3"), SpanYam::null())]);
    assert_eq!(
        yaml_map.is_non_empty_collection(),
        span_map.is_non_empty_collection()
    );
    unsafe {
        for (yam_entry, span_entry) in yaml_map.mapping().iter().zip(span_map.mapping()) {
            assert_eq!(yam_entry.key.as_i64(), span_entry.key.as_i64());
            assert_eq!(yam_entry.value.is_null(), span_entry.value.is_null());
        }
    }

    assert_eq!(&yaml_map["3"].is_null(), &span_map["3"].is_null());

    let mut yaml_seq = Yaml::from(vec![Yaml::from(3), Yaml::from("aaa")]);
    let mut span_seq = SpanYam::from(vec![SpanYam::from(3), SpanYam::from("aaa")]);

    if let Ok(x) = yaml_seq.as_sequence_mut() {
        x.clear();
    }
    if let Ok(x) = span_seq.as_sequence_mut() {
        x.clear();
    }

    assert!(!yaml_seq.is_non_empty_collection());
    assert!(!span_seq.is_non_empty_collection());

    let mut yaml_map = Yaml::from(vec![(Yaml::from("3"), Yaml::from(true))]);
    let mut span_map = SpanYam::from(vec![(SpanYam::from("3"), SpanYam::from(true))]);
    yaml_map["3"] = Yaml::from(false);
    span_map["3"] = SpanYam::from(false);

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
            Yaml::from(3).as_sequence().unwrap();
        }
        (YamlType::Yaml, TestType::SeqMut) => {
            Yaml::from(3).as_sequence_mut().unwrap();
        }
        (YamlType::Yaml, TestType::Map) => {
            Yaml::from(3).as_mapping().unwrap();
        }
        (YamlType::Yaml, TestType::MapMut) => {
            Yaml::from(3).as_mapping_mut().unwrap();
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
            SpanYam::from(3).as_sequence().unwrap();
        }
        (YamlType::Span, TestType::SeqMut) => {
            SpanYam::from(3).as_sequence_mut().unwrap();
        }
        (YamlType::Span, TestType::Map) => {
            SpanYam::from(3).as_mapping().unwrap();
        }
        (YamlType::Span, TestType::MapMut) => {
            SpanYam::from(3).as_mapping_mut().unwrap();
        }
        (YamlType::Span, TestType::Ind) => {
            SpanYam::from(3).index(0);
        }
        (YamlType::Span, TestType::IndMut) => {
            SpanYam::from(3).index_mut(0);
        }
        (YamlType::Span, TestType::IndStr) => {
            SpanYam::from(3).index("0");
        }
        (YamlType::Span, TestType::IndStrMut) => {
            SpanYam::from(3).index_mut("0");
        }
    };
}

#[rstest]
#[case::seq_empt(true, true)]
#[case::seq_full(true, false)]
#[case::map_empt(false, true)]
#[case::map_full(false, false)]
fn test_into_collection(#[case] is_seq: bool, #[case] is_empty: bool) {
    let (yaml, span) = match (is_seq, is_empty) {
        (_, true) => (Yaml::from(3), SpanYam::from(3)),
        (true, false) => (Yaml::from(vec![2]), SpanYam::from(vec![2])),
        (false, false) => (Yaml::from(vec![(2, 3)]), SpanYam::from(vec![(2, 3)])),
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

#[test]
fn test_into_tagged() {
    let scalar: YamlScalar<'_> = YamlScalar::null();
    let tag = Tag::new("tag:yaml.org,2002:", "null");

    let mut yaml_span = SpanYam::<'_>::from(scalar.clone());
    yaml_span = yaml_span.into_tagged(Cow::Owned(tag.clone()));

    let mut yaml = Yaml::<'_>::from(scalar);
    yaml = yaml.into_tagged(Cow::Owned(tag.clone()));

    assert_eq!(yaml.get_tag(), Some(tag.clone()));
    assert_eq!(yaml_span.get_tag(), Some(tag));
}

#[test]
fn test_loader() {
    let input = "a: b";
    let yaml = Yaml::load_from(input).expect("Parsing success");
    let yaml_span = SpanYam::load_from(input).expect("Parsing success");

    assert_eq!(&yaml[0]["a"], &"b");
    assert_eq!(yaml_span[0]["a"].as_str().unwrap_or(""), "b");

    let yaml = Yaml::load_single(input).expect("Parsing success");
    let yaml_span = SpanYam::load_single(input).expect("Parsing success");

    assert_eq!(yaml["a"], "b");
    assert_eq!(yaml_span["a"].as_str().unwrap_or(""), "b");
}

#[test]
fn test_mapping() {
    let mut map = HashMap::new();
    map.insert("answer", 42);

    let yaml: Yaml = to_yaml(map).unwrap();
    assert!(yaml.is_mapping());
    assert_eq!(yaml["answer"], 42);
}

#[test]
fn test_binary_mapping() {
    #[derive(PartialEq, Debug, Serialize, Deserialize)]
    struct BinaryConvert {
        #[serde(with = "serde_bytes")]
        byte_buf: Vec<u8>,
    }

    let bin_convert = BinaryConvert {
        byte_buf: b"test".to_vec(),
    };
    let yaml: Yaml = to_yaml(bin_convert).unwrap();
    assert!(yaml.is_mapping());
    assert!(yaml["byte_buf"].is_tagged());
}

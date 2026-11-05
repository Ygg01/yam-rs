use rstest::rstest;
use yam_core::emitter::YamlEmitter;
use yam_core::node::Yaml;

const NESTED_FLOW_IN: &str = "[[], {}]";
const NESTED_FLOW_EXP: &str = r#"---
- []
- {}"#;

const SIMPLE_MAP_IN: &str = "{a: true, b: false}";
const SIMPLE_MAP_EXP: &str = r#"---
a: true
b: false"#;
const COMPLEX_KEY_IN: &str = r#"
? [1, test]
: 'xyz'
? [false, true]
: 3.2"#;
const COMPLEX_KEY_EXP: &str = r#"---
? - 1
  - test
: xyz
? - false
  - true
: 3.2"#;

const TAGGED_MAP_IN: &str = r#"
? >
    test this opossum of a key
    this iq
: !!null null"#;
const TAGGED_MAP_EXP: &str = r#"---
? |
  test this opossum of a key this iq
: ~"#;

const TAGGED_MAP2_IN: &str = r#"
? >-
    test this opossum of a key
    this iq
      
     
    a
: !tag [b]"#;
const TAGGED_MAP2_EXP: &str = r#"---
? |-
  test this opossum of a key this iq
    
   
  a
: !tag  - b"#;

const ESC_STR_IN: &str = "\" \\\\\\\"\x01\x02\x03\x04\x05\x06\x07\x08\\t\\n\x0B\x0C\\r\x0e\x0F\
 \x10\x11\x12\x13\x14\x15\x16\x17\x18\x19\x1a\x1b\x1c\x1d\x1e\x1f\x7f\"";
const ESC_STR_EXP: &str = r#"---
" \\\"\u0001\u0002\u0003\u0004\u0005\u0006\u0007\b\t\n\u000b\f\r\u000e\u000f\u0010\u0011\u0012\u0013\u0014\u0015\u0016\u0017\u0018\u0019\u001a\u001b\u001c\u001d\u001e\u001f\u007f""#;

const SIMPLE_STR_IN: &str = "\"just a string \"";
const SIMPLE_STR_EXP: &str = r#"---
"just a string ""#;

#[must_use]
fn event_to_emitter(input: &str, is_multiline: bool, is_compact: bool) -> String {
    let yaml_node = Yaml::load_single(input).unwrap();
    let mut yaml_emit = YamlEmitter::from(String::with_capacity(100));
    yaml_emit.compact(is_compact);
    yaml_emit.multiline_strings(is_multiline);
    let _ = yaml_emit.dump(&yaml_node);
    yaml_emit.writer()
}

#[rstest]
#[case::span_flow_map(NESTED_FLOW_IN, NESTED_FLOW_EXP, false, true)]
#[case::span_complex_key(COMPLEX_KEY_IN, COMPLEX_KEY_EXP, false, true)]
#[case::span_tagged_map(TAGGED_MAP_IN, TAGGED_MAP_EXP, true, false)]
#[case::span_tagged_map(TAGGED_MAP2_IN, TAGGED_MAP2_EXP, true, false)]
#[case::span_esc_map(ESC_STR_IN, ESC_STR_EXP, true, false)]
#[case::span_simple_str(SIMPLE_STR_IN, SIMPLE_STR_EXP, false, false)]
#[case::span_simple_map(SIMPLE_MAP_IN, SIMPLE_MAP_EXP, false, false)]
fn test_emitter(
    #[case] input: &str,
    #[case] expected: &str,
    #[case] is_multiline: bool,
    #[case] is_compact: bool,
) {
    let emitted = event_to_emitter(input, is_multiline, is_compact);
    assert_eq!(&emitted, expected);
}

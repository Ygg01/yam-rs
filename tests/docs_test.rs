use steel_yaml::tokenizer::assert_eq_event;

const EMPTY_DOC_ERR_INPUT: &str = r#"
# test"
  # test
%YAML 1.3 #arst
"#;
const EMPTY_DOC_ERR_EVENTS: &str = r#"
 %YAML 1.3
 ERR"#;

const EMPTY_DOC_INPUT: &str = r#"
%YAML 1.2
---
"#;
const EMPTY_DOC_EVENTS: &str = r#"
 %YAML 1.2
 +DOC ---
 -DOC"#;

#[test]
fn doc_empty() {
    assert_eq_event(EMPTY_DOC_ERR_INPUT, EMPTY_DOC_ERR_EVENTS);
    assert_eq_event(EMPTY_DOC_INPUT, EMPTY_DOC_EVENTS);
}

const ERR_DIRECTIVE_INPUT: &str = r#"
%YAML 1.2
...
"#;

const ERR_DIRECTIVE_EVENTS: &str = r#"
 ERR
 %YAML 1.2"#;

#[test]
fn flow_err_directive() {
    assert_eq_event(ERR_DIRECTIVE_INPUT, ERR_DIRECTIVE_EVENTS);
    assert_eq_event(EMPTY_DOC_INPUT, EMPTY_DOC_EVENTS);
}

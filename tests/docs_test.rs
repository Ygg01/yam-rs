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
  =VAL :
 -DOC"#;

const DOC_EMPTY_TAG_INPUT: &str = r#"
%YAM 1.2
---
"#;

const DOC_EMPTY_TAG_EVENTS: &str = r#"
 +DOC ---
  =VAL :
 -DOC"#;


#[test]
fn doc_empty() {
    assert_eq_event(DOC_EMPTY_TAG_INPUT, DOC_EMPTY_TAG_EVENTS);
    assert_eq_event(EMPTY_DOC_ERR_INPUT, EMPTY_DOC_ERR_EVENTS);
    assert_eq_event(EMPTY_DOC_INPUT, EMPTY_DOC_EVENTS);
}

const ERR_DIRECTIVE_INPUT: &str = r#"
%YAML 1.2
...
"#;

const ERR_DIRECTIVE_EVENTS: &str = r#"
 ERR
 %YAML 1.2
 +DOC
 -DOC ..."#;

 const ERR_DIRECTIVE2_INPUT: &str = r#"
 %YAML 1.2#err
 ...
 "#;
 
 const ERR_DIRECTIVE2_EVENTS: &str = r#"
 ERR
 ERR
 %YAML 1.2
 +DOC
 -DOC ..."#;

 const ERR_DIRECTIVE3_INPUT: &str = r#"
 %YAML 1.2 err
 ---
 "#;
 
 const ERR_DIRECTIVE3_EVENTS: &str = r#"
 ERR
 %YAML 1.2
 +DOC ---
  =VAL :
 -DOC"#;

 const ERR_MULTIDOC_INPUT: &str = r#"
 %YAML 1.2
 ---
 %YAML 1.2
 ---
 "#;
 
 const ERR_MULTIDOC_EVENTS: &str = r#"
 ERR
 %YAML 1.2
 +DOC ---
 -DOC
 %YAML 1.2
 +DOC ---
  =VAL :
 -DOC"#;


#[test]
fn doc_err_directive() {
    assert_eq_event(ERR_DIRECTIVE_INPUT, ERR_DIRECTIVE_EVENTS);
    assert_eq_event(ERR_DIRECTIVE2_INPUT, ERR_DIRECTIVE2_EVENTS);
    assert_eq_event(ERR_DIRECTIVE3_INPUT, ERR_DIRECTIVE3_EVENTS);
    assert_eq_event(ERR_MULTIDOC_INPUT, ERR_MULTIDOC_EVENTS);
}

const SIMPLE_DOC_INPUT: &str = r#"
---[]"#;

const SIMPLE_DOC_EVENTS: &str = r#"
 +DOC ---
  +SEQ []
  -SEQ
 -DOC"#;


const SIMPLE_DOC2_INPUT: &str = r##"
%YAML 1.3 #comment
          #comment
---
"test"
"##;

const SIMPLE_DOC2_EVENTS: &str = r#"
 %YAML 1.3
 +DOC ---
  =VAL "test
 -DOC"#;

#[test]
fn simple_doc() {
    assert_eq_event(SIMPLE_DOC_INPUT, SIMPLE_DOC_EVENTS);
    assert_eq_event(SIMPLE_DOC2_INPUT, SIMPLE_DOC2_EVENTS);
}


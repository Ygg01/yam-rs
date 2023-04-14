mod common;

use crate::common::assert_eq_event;

const EMPTY_DOC_ERR_INPUT: &str = r#"
# test"
  # test
%YAML 1.3 #arst
"#;
const EMPTY_DOC_ERR_EXPECTED: &str = r#"
 %YAML 1.3
 ERR"#;

const EMPTY_DOC_INPUT: &str = r#"
%YAML 1.2
---
"#;
const EMPTY_DOC_EXPECTED: &str = r#"
 %YAML 1.2
 +DOC ---
 -DOC"#;

#[test]
fn parse_empty_document() {
    assert_eq_event(EMPTY_DOC_ERR_INPUT, EMPTY_DOC_ERR_EXPECTED);
    assert_eq_event(EMPTY_DOC_INPUT, EMPTY_DOC_EXPECTED);
}

const NULL_YAML_INPUT: &str = r#"
null
"#;

const NULL_YAML_INPUT2: &str = "\r\nnull\r\n";
const NULL_YAML_EXPECTED: &str = r#"
 +DOC
  =VAL :null
 -DOC"#;

const MULTI_WORD_INPUT: &str = r#"
  null test xy"#;
const MULTI_WORD_EXPECTED: &str = r#"
 +DOC
  =VAL :null test xy
 -DOC"#;

const MULTILINE_INPUT: &str = r#"
test
xt
"#;
const MULTILINE_EXPECTED: &str = r#"
 +DOC
  =VAL :test xt
 -DOC"#;

#[test]
fn parse_flow_scalars() {
    assert_eq_event(NULL_YAML_INPUT, NULL_YAML_EXPECTED);
    assert_eq_event(NULL_YAML_INPUT2, NULL_YAML_EXPECTED);
    assert_eq_event(MULTI_WORD_INPUT, MULTI_WORD_EXPECTED);
    assert_eq_event(MULTILINE_INPUT, MULTILINE_EXPECTED);
}

const SEQ_FLOW_INPUT: &str = r#"
[x, y]
"#;
const SEQ_FLOW_INPUT2: &str = r#"
[x ,y]
"#;
const SEQ_FLOW_EXPECTED: &str = r#"
 +DOC
  +SEQ []
   =VAL :x
   =VAL :y
  -SEQ
 -DOC"#;

#[test]
fn parse_flow_seq() {
    assert_eq_event(SEQ_FLOW_INPUT, SEQ_FLOW_EXPECTED);
    assert_eq_event(SEQ_FLOW_INPUT2, SEQ_FLOW_EXPECTED);
}

const SEQ_NESTED_COL1: &str = r#"
[:]
"#;
const SEQ_NESTED_COL2: &str = r#"
[{:}]
"#;

const SEQ_NESTED_COL1_EXPECTED: &str = r#"
 +DOC
  +SEQ []
   +MAP {}
    =VAL :
    =VAL :
   -MAP
  -SEQ
 -DOC"#;

#[test]
fn parse_nested_col() {
    assert_eq_event(SEQ_NESTED_COL1, SEQ_NESTED_COL1_EXPECTED);
    assert_eq_event(SEQ_NESTED_COL2, SEQ_NESTED_COL1_EXPECTED);
}

const SEQ_EMPTY_MAP: &str = r#"
{:}
"#;
const SEQ_EMPTY_MAP_EXPECTED: &str = r#"
 +DOC
  +MAP {}
   =VAL :
   =VAL :
  -MAP
 -DOC"#;

const SEQ_XY_MAP1: &str = r#"
{x:y}
"#;
const SEQ_XY_MAP1_EXPECTED: &str = r#"
 +DOC
  +MAP {}
   =VAL :x:y
   =VAL :
  -MAP
 -DOC"#;

const SEQ_X_Y_MAP1: &str = r#"
{x: y}
"#;
const SEQ_X_Y_MAP2: &str = r#"
{? x: y}
"#;
const SEQ_X_Y_MAP3: &str = r#"
{x: #comment
 y}
"#;
const SEQ_X_Y_MAP_EXPECTED: &str = r#"
 +DOC
  +MAP {}
   =VAL :x
   =VAL :y
  -MAP
 -DOC"#;

const SEQ_COMPLEX_MAP: &str = r#"
{[x,y]:a}
"#;

const SEQ_COMPLEX_MAP_EXPECTED: &str = r#"
 +DOC
  +MAP {}
   +SEQ []
    =VAL :x
    =VAL :y
   -SEQ
   =VAL :a
  -MAP
 -DOC"#;

#[test]
fn parse_flow_map() {
    assert_eq_event(SEQ_EMPTY_MAP, SEQ_EMPTY_MAP_EXPECTED);
    assert_eq_event(SEQ_XY_MAP1, SEQ_XY_MAP1_EXPECTED);
    assert_eq_event(SEQ_X_Y_MAP1, SEQ_X_Y_MAP_EXPECTED);
    assert_eq_event(SEQ_X_Y_MAP2, SEQ_X_Y_MAP_EXPECTED);
    assert_eq_event(SEQ_X_Y_MAP3, SEQ_X_Y_MAP_EXPECTED);
}

#[test]
fn parse_complex_map() {
    assert_eq_event(SEQ_COMPLEX_MAP, SEQ_COMPLEX_MAP_EXPECTED);
}

const SQUOTE_STR1: &str = r#"
  'single quote'
    "#;

const SQUOTE_STR2: &str = r#"
  'single
  quote'"#;

const SQUOTE_STR_EXPECTED: &str = r#"
 +DOC
  =VAL 'single quote
 -DOC"#;

const SQUOTE_ESCAPE: &str = r#"'for single quote, use '' two of them'"#;
const SQUOTE_ESCAPE2: &str = r#"'for single quote, use
'' two of them'"#;
const SQUOTE_ESCAPE_EXPECTED: &str = r#"
 +DOC
  =VAL 'for single quote, use ' two of them
 -DOC"#;

#[test]
fn flow_single_quote() {
    assert_eq_event(SQUOTE_STR1, SQUOTE_STR_EXPECTED);
    assert_eq_event(SQUOTE_STR2, SQUOTE_STR_EXPECTED);
    assert_eq_event(SQUOTE_ESCAPE, SQUOTE_ESCAPE_EXPECTED);
    assert_eq_event(SQUOTE_ESCAPE2, SQUOTE_ESCAPE_EXPECTED);
}

const DQUOTE_STR1: &str = r#"
  "double quote"
    "#;

const DQUOTE_STR2: &str = r#"
  "double
  quote"
"#;

const DQUOTE_STR_EXPECTED: &str = r#"
 +DOC
  =VAL "double quote
 -DOC"#;

const DQUOTE_STR_ESCAPE1: &str = r#"
 "double quote (\")""#;

const DQUOTE_STR_ESCAPE_EXPECTED: &str = r#"
 +DOC
  =VAL "double quote (")
 -DOC"#;

#[test]
fn flow_double_quote() {
    assert_eq_event(DQUOTE_STR1, DQUOTE_STR_EXPECTED);
    assert_eq_event(DQUOTE_STR2, DQUOTE_STR_EXPECTED);
    assert_eq_event(DQUOTE_STR_ESCAPE1, DQUOTE_STR_ESCAPE_EXPECTED);
}

const ERR_PLAIN_SCALAR: &str = r#"
  a
  b
 c"#;

const ERR_PLAIN_SCALAR_EXPECTED: &str = r#"
 +DOC
  =VAL :a b
  ERR
 -DOC"#;

#[test]
fn err_plain_scalar() {
    assert_eq_event(ERR_PLAIN_SCALAR, ERR_PLAIN_SCALAR_EXPECTED);
}

const SIMPLE_DOC: &str = r#"
---[]"#;

const SIMPLE_DOC_EXPECTED: &str = r#"
 +DOC ---
  +SEQ []
  -SEQ
 -DOC"#;

#[test]
fn simple_doc() {
    assert_eq_event(SIMPLE_DOC, SIMPLE_DOC_EXPECTED);
}

const SEQ_ERR: &str = r#"
---
[a, b] ]"#;

const SEQ_ERR_EXPECTED: &str = r#"
 +DOC ---
  +SEQ []
   =VAL :a
   =VAL :b
  -SEQ
 -DOC
 ERR"#;

#[test]
fn doc_end_err() {
    assert_eq_event(SEQ_ERR, SEQ_ERR_EXPECTED);
}

const SEQ_KEY: &str = r#"
[a, b]: 3"#;


const SEQ_KEY_EXPECTED: &str = r#"
 +DOC 
  +MAP
   +SEQ []
    =VAL :a
    =VAL :b
   -SEQ
   =VAL :3
  -MAP
 +DOC"#;

 #[test]
fn seq_as_key() {
  assert_eq_event(SEQ_KEY, SEQ_KEY_EXPECTED);
}


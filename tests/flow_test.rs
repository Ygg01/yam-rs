mod common;

use crate::common::assert_eq_event;

const EMPTY_DOC_INPUT: &'static str = r#"
# test"
  # test
%YAML 1.3 #arst
"#;
const EMPTY_DOC_EXPECTED: &'static str = r#"
 %YAML 1.3
 ERR"#;

#[test]
fn parse_empty_document() {
    assert_eq_event(EMPTY_DOC_INPUT, EMPTY_DOC_EXPECTED);
}

const NULL_YAML_INPUT: &'static str = r#"
null
"#;

const NULL_YAML_INPUT2: &'static str = "\r\nnull\r\n";
const NULL_YAML_EXPECTED: &'static str = r#"
 =VAL null"#;

const MULTI_WORD_INPUT: &'static str = r#"
  null test xy"#;
const MULTI_WORD_EXPECTED: &'static str = r#"
 =VAL null test xy"#;

const MULTILINE_INPUT: &'static str = r#"
test
xt
"#;
const MULTILINE_EXPECTED: &'static str = r#"
 =VAL test xt"#;

#[test]
fn parse_flow_scalars() {
    assert_eq_event(NULL_YAML_INPUT, NULL_YAML_EXPECTED);
    assert_eq_event(NULL_YAML_INPUT2, NULL_YAML_EXPECTED);
    assert_eq_event(MULTI_WORD_INPUT, MULTI_WORD_EXPECTED);
    assert_eq_event(MULTILINE_INPUT, MULTILINE_EXPECTED);
}

const SEQ_FLOW_INPUT: &'static str = r#"
[x, y]
"#;
const SEQ_FLOW_INPUT2: &'static str = r#"
[x ,y]
"#;
const SEQ_FLOW_EXPECTED: &'static str = r#"
 +SEQ
  =VAL x
  =VAL y
 -SEQ"#;

#[test]
fn parse_flow_seq() {
    assert_eq_event(SEQ_FLOW_INPUT, SEQ_FLOW_EXPECTED);
    assert_eq_event(SEQ_FLOW_INPUT2, SEQ_FLOW_EXPECTED);
}

const SEQ_NESTED_COL1: &'static str = r#"
[:]
"#;
const SEQ_NESTED_COL2: &'static str = r#"
[{:}]
"#;

const SEQ_NESTED_COL1_EXPECTED: &'static str = r#"
 +SEQ
  +MAP
  -MAP
 -SEQ"#;

const SEQ_NESTED_COL2_EXPECTED: &'static str = r#"
 +SEQ
  +MAP
  -MAP
 -SEQ"#;

#[test]
fn parse_nested_col() {
    assert_eq_event(SEQ_NESTED_COL1, SEQ_NESTED_COL1_EXPECTED);
    assert_eq_event(SEQ_NESTED_COL2, SEQ_NESTED_COL2_EXPECTED);
}

const SEQ_EMPTY_MAP: &'static str = r#"
{:}
"#;
const SEQ_EMPTY_MAP_EXPECTED: &'static str = r#"
 +MAP
 -MAP"#;

const SEQ_XY_MAP1: &'static str = r#"
{x:y}
"#;
const SEQ_XY_MAP1_EXPECTED: &'static str = r#"
 +MAP
  =VAL x:y
 -MAP"#;

const SEQ_X_Y_MAP1: &'static str = r#"
{x: y}
"#;
const SEQ_X_Y_MAP2: &'static str = r#"
{? x: y}
"#;
const SEQ_X_Y_MAP3: &'static str = r#"
{x: #comment
 y}
"#;
const SEQ_X_Y_MAP_EXPECTED: &'static str = r#"
 +MAP
  =VAL x
  =VAL y
 -MAP"#;

const SEQ_COMPLEX_MAP: &'static str = r#"
{[x,y]:a}
"#;

const SEQ_COMPLEX_MAP_EXPECTED: &'static str = r#"
 +MAP
  +SEQ
   =VAL x
   =VAL y
  -SEQ
  =VAL a
 -MAP"#;

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

const SQUOTE_STR1: &'static str = r#"
  'single quote'
    "#;

const SQUOTE_STR2: &'static str = r#"
  'single
  quote'"#;

const SQUOTE_STR_EXPECTED: &'static str = r#"
 =VAL single quote"#;

const SQUOTE_ESCAPE: &'static str = r#"'for single quote, use '' two of them'"#;
const SQUOTE_ESCAPE2: &'static str = r#"'for single quote, use
'' two of them'"#;
const SQUOTE_ESCAPE_EXPECTED: &'static str = r#"
 =VAL for single quote, use ' two of them"#;

#[test]
fn flow_single_quote() {
    assert_eq_event(SQUOTE_STR1, SQUOTE_STR_EXPECTED);
    assert_eq_event(SQUOTE_STR2, SQUOTE_STR_EXPECTED);
    assert_eq_event(SQUOTE_ESCAPE, SQUOTE_ESCAPE_EXPECTED);
    assert_eq_event(SQUOTE_ESCAPE2, SQUOTE_ESCAPE_EXPECTED);
}

const DQUOTE_STR1: &'static str = r#"
  "double quote"
    "#;

const DQUOTE_STR2: &'static str = r#"
  "double
  quote"
"#;

const DQUOTE_STR_EXPECTED: &'static str = r#"
 =VAL double quote"#;

const DQUOTE_STR_ESCAPE1: &'static str = r#"
 "double quote (\")""#;

const DQUOTE_STR_ESCAPE_EXPECTED: &'static str = r#"
 =VAL double quote (")"#;

#[test]
fn flow_double_quote() {
    assert_eq_event(DQUOTE_STR1, DQUOTE_STR_EXPECTED);
    assert_eq_event(DQUOTE_STR2, DQUOTE_STR_EXPECTED);
    assert_eq_event(DQUOTE_STR_ESCAPE1, DQUOTE_STR_ESCAPE_EXPECTED);
}

const ERR_PLAIN_SCALAR: &'static str = r#"
  a
  b
 c"#;

const ERR_PLAIN_SCALAR_EXPECTED: &'static str = r#"
 =VAL a b
 ERR"#;

#[test]
fn err_plain_scalar() {
    assert_eq_event(ERR_PLAIN_SCALAR, ERR_PLAIN_SCALAR_EXPECTED);
}

const SIMPLE_DOC: &'static str = r#"
---[]"#;

const SIMPLE_DOC_EXPECTED: &'static str = r#"
 +SEQ
 -SEQ"#;

#[test]
fn simple_doc() {
    assert_eq_event(SIMPLE_DOC, SIMPLE_DOC_EXPECTED);
}

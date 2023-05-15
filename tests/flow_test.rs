mod common;

use crate::common::assert_eq_event;

const NULL_YAML_INPUT: &str = r#"
null
"#;

const NULL_YAML2_INPUT: &str = "\r\nnull\r\n";
const NULL_YAML_EVENTS: &str = r#"
 +DOC
  =VAL :null
 -DOC"#;

const MULTI_WORD_INPUT: &str = r#"
  null test xy"#;
const MULTI_WORD_EVENTS: &str = r#"
 +DOC
  =VAL :null test xy
 -DOC"#;

const MULTILINE_INPUT: &str = r#"
test
xt
"#;
const MULTILINE_EVENTS: &str = r#"
 +DOC
  =VAL :test xt
 -DOC"#;

#[test]
fn flow_scalars() {
    assert_eq_event(NULL_YAML_INPUT, NULL_YAML_EVENTS);
    assert_eq_event(NULL_YAML2_INPUT, NULL_YAML_EVENTS);
    assert_eq_event(MULTI_WORD_INPUT, MULTI_WORD_EVENTS);
    assert_eq_event(MULTILINE_INPUT, MULTILINE_EVENTS);
}

const SEQ_FLOW_INPUT: &str = r#"
[x, y]
"#;
const SEQ_FLOW2_INPUT: &str = r#"
[x ,y]
"#;
const SEQ_FLOW_EVENTS: &str = r#"
 +DOC
  +SEQ []
   =VAL :x
   =VAL :y
  -SEQ
 -DOC"#;

#[test]
fn flow_flow_seq() {
    assert_eq_event(SEQ_FLOW_INPUT, SEQ_FLOW_EVENTS);
    assert_eq_event(SEQ_FLOW2_INPUT, SEQ_FLOW_EVENTS);
}

const NEST_COL1_INPUT: &str = r#"
[:]
"#;
const NEST_COL2_INPUT: &str = r#"
[{:}]
"#;

const NESTED_COL_EVENTS: &str = r#"
 +DOC
  +SEQ []
   +MAP {}
    =VAL :
    =VAL :
   -MAP
  -SEQ
 -DOC"#;

#[test]
fn flow_nested_col() {
    assert_eq_event(NEST_COL1_INPUT, NESTED_COL_EVENTS);
    assert_eq_event(NEST_COL2_INPUT, NESTED_COL_EVENTS);
}

const MAP_XY_INPUT: &str = r#"
{x:y}
"#;
const MAP_XY_EVENTS: &str = r#"
 +DOC
  +MAP {}
   =VAL :x:y
   =VAL :
  -MAP
 -DOC"#;

const MAP_X_Y_INPUT: &str = r#"
{x: y}
"#;
const MAP_X_Y2_INPUT: &str = r#"
{? x: y}
"#;
const MAP_X_Y3_INPUT: &str = r#"
{x: #comment
 y}
"#;
const MAP_X_Y_EVENTS: &str = r#"
 +DOC
  +MAP {}
   =VAL :x
   =VAL :y
  -MAP
 -DOC"#;

const COMPLEX_MAP_INPUT: &str = r#"
{[x,y]:a}
"#;

const COMPLEX_MAP_EVENTS: &str = r#"
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
fn flow_map() {
    assert_eq_event(MAP_XY_INPUT, MAP_XY_EVENTS);
    assert_eq_event(MAP_X_Y_INPUT, MAP_X_Y_EVENTS);
    assert_eq_event(MAP_X_Y2_INPUT, MAP_X_Y_EVENTS);
    assert_eq_event(MAP_X_Y3_INPUT, MAP_X_Y_EVENTS);
}

const FLOW_QUOTED_INPUT: &str = r#"
{"ab"
: "xy"}
"#;

const FLOW_QUOTED_EVENTS: &str = r#"
 +DOC
  +MAP {}
   =VAL "ab
   =VAL "xy
  -MAP
 -DOC"#;

#[test]
fn flow_map_quoted() {
    assert_eq_event(FLOW_QUOTED_INPUT, FLOW_QUOTED_EVENTS);
}

const EMPTY_MAP_INPUT: &str = r#"
{:}
"#;
const EMPTY_MAP_EVENTS: &str = r#"
 +DOC
  +MAP {}
   =VAL :
   =VAL :
  -MAP
 -DOC"#;

const EMPTY_NODES_INPUT: &str = r#"
{
    a: "b",
    x,
    y:,
}
"#;

const EMPTY_NODES_EVENTS: &str = r#"
 +DOC
  +MAP {}
   =VAL :a
   =VAL "b
   =VAL :x
   =VAL :
   =VAL :y
   =VAL :
  -MAP
 -DOC"#;

#[test]
fn flow_empty_nodes() {
    assert_eq_event(EMPTY_MAP_INPUT, EMPTY_MAP_EVENTS);
    assert_eq_event(EMPTY_NODES_INPUT, EMPTY_NODES_EVENTS);
}

#[test]
fn flow_complex_map() {
    assert_eq_event(COMPLEX_MAP_INPUT, COMPLEX_MAP_EVENTS);
}

const SQUOTE_STR1_INPUT: &str = r#"
  'single quote'
    "#;

const SQUOTE_STR2_INPUT: &str = r#"
  'single
  quote'"#;

const SQUOTE_STR_EVENTS: &str = r#"
 +DOC
  =VAL 'single quote
 -DOC"#;

const SQUOTE_ESCAPE_INPUT: &str = r#"'for single quote, use '' two of them'"#;
const SQUOTE_ESCAPE2_INPUT: &str = r#"'for single quote, use
'' two of them'"#;
const SQUOTE_ESCAPE_EVENTS: &str = r#"
 +DOC
  =VAL 'for single quote, use ' two of them
 -DOC"#;

#[test]
fn flow_single_quote() {
    assert_eq_event(SQUOTE_STR1_INPUT, SQUOTE_STR_EVENTS);
    assert_eq_event(SQUOTE_STR2_INPUT, SQUOTE_STR_EVENTS);
    assert_eq_event(SQUOTE_ESCAPE_INPUT, SQUOTE_ESCAPE_EVENTS);
    assert_eq_event(SQUOTE_ESCAPE2_INPUT, SQUOTE_ESCAPE_EVENTS);
}

const DQUOTE_STR1_INPUT: &str = r#"
  "double quote"
    "#;

const DQUOTE_STR2_INPUT: &str = r#"
  "double
  quote"
"#;

const DQUOTE_STR_EVENTS: &str = r#"
 +DOC
  =VAL "double quote
 -DOC"#;

const DQUOTE_MULTI_INPUT: &str = r##"
 "test  
 
   tab" "##;

const DQUOTE_MULTI_EVENTS: &str = r#"
 +DOC
  =VAL "test\ntab
 -DOC"#;

#[test]
fn flow_double_quote() {
    assert_eq_event(DQUOTE_STR1_INPUT, DQUOTE_STR_EVENTS);
    assert_eq_event(DQUOTE_STR2_INPUT, DQUOTE_STR_EVENTS);
    assert_eq_event(DQUOTE_MULTI_INPUT, DQUOTE_MULTI_EVENTS);
}

const DQUOTE_LEADING_TAB_INPUT: &str = r##""1 test
    \	tab""##;

const DQUOTE_LEADING_TAB2_INPUT: &str = r##"
    "1 test
      \ttab" "##;

const DQUOTE_LEADING_TAB_EVENTS: &str = r#"
 +DOC
  =VAL "1 test \ttab
 -DOC"#;

const DQUOTE_STR_ESC1_INPUT: &str = r#"
 "double quote (\")""#;

const DQUOTE_STR_ESC_EVENTS: &str = r#"
 +DOC
  =VAL "double quote (")
 -DOC"#;

const DQUOTE_ESC_INPUT: &str = r##"
 "a\/b"
"##;

const DQUOTE_ESC_EVENTS: &str = r#"
 +DOC
  =VAL "a/b
 -DOC"#;

#[test]
fn flow_double_quote_escape() {
    assert_eq_event(DQUOTE_ESC_INPUT, DQUOTE_ESC_EVENTS);
    assert_eq_event(DQUOTE_LEADING_TAB_INPUT, DQUOTE_LEADING_TAB_EVENTS);
    assert_eq_event(DQUOTE_LEADING_TAB2_INPUT, DQUOTE_LEADING_TAB_EVENTS);
    assert_eq_event(DQUOTE_STR_ESC1_INPUT, DQUOTE_STR_ESC_EVENTS);
}

const DQUOTE_ERR_INPUT: &str = r##"
- "double   
            
quote" "##;

const DQUOTE_ERR_EVENTS: &str = r#"
 +DOC
  +SEQ
   =VAL "double\nquote
  -SEQ
 -DOC"#;

#[test]
fn flow_double_quote_err() {
    assert_eq_event(DQUOTE_ERR_INPUT, DQUOTE_ERR_EVENTS);
}

const ERR_PLAIN_SCALAR_INPUT: &str = r#"
  a
  b
 c"#;

const ERR_PLAIN_SCALAR_EVENTS: &str = r#"
 +DOC
  =VAL :a b
  ERR
 -DOC"#;

#[test]
fn flow_err_plain_scalar() {
    assert_eq_event(ERR_PLAIN_SCALAR_INPUT, ERR_PLAIN_SCALAR_EVENTS);
}

const DOC_END_ERR_INPUT: &str = r#"
---
[a, b] ]"#;

const DOC_END_ERR_EVENTS: &str = r#"
 +DOC ---
  +SEQ []
   =VAL :a
   =VAL :b
  -SEQ
 -DOC
 ERR"#;

#[test]
fn doc_end_err() {
    assert_eq_event(DOC_END_ERR_INPUT, DOC_END_ERR_EVENTS);
}

const SEQ_KEY_INPUT: &str = r#"
[a, b]: 3 "#;

const SEQ_KEY_EVENTS: &str = r#"
 +DOC
  +MAP
   +SEQ []
    =VAL :a
    =VAL :b
   -SEQ
   =VAL :3
  -MAP
 -DOC"#;

const SEQ_KEY2_INPUT: &str = r#"
[a, [b,c]]: 3 "#;

const SEQ_KEY2_EVENTS: &str = r#"
 +DOC
  +MAP
   +SEQ []
    =VAL :a
    +SEQ []
     =VAL :b
     =VAL :c
    -SEQ
   -SEQ
   =VAL :3
  -MAP
 -DOC"#;

const SEQ_KEY3_INPUT: &str = r#"
 [[a]: 3]"#;

const SEQ_KEY3_EVENTS: &str = r#"
 +DOC
  +SEQ []
   +MAP {}
    +SEQ []
     =VAL :a
    -SEQ
    =VAL :3
   -MAP
  -SEQ
 -DOC"#;

const SEQ_KEY4_INPUT: &str = r#"
 [ [a]: d, e]: 3"#;

const SEQ_KEY4_EVENTS: &str = r#"
 +DOC
  +MAP
   +SEQ []
    +MAP {}
     +SEQ []
      =VAL :a
     -SEQ
     =VAL :d
    -MAP
    =VAL :e
   -SEQ
   =VAL :3
  -MAP
 -DOC"#;

#[test]
fn flow_seq_as_key() {
    assert_eq_event(SEQ_KEY_INPUT, SEQ_KEY_EVENTS);
    assert_eq_event(SEQ_KEY2_INPUT, SEQ_KEY2_EVENTS);
    assert_eq_event(SEQ_KEY3_INPUT, SEQ_KEY3_EVENTS);
    assert_eq_event(SEQ_KEY4_INPUT, SEQ_KEY4_EVENTS);
}

const SEQ_ERR_INPUT: &str = r#"
 [-]"#;

const SEQ_ERR_EVENTS: &str = r#"
 +DOC
  +SEQ []
   ERR
  -SEQ
 -DOC"#;

#[test]
fn flow_seq_err() {
    assert_eq_event(SEQ_ERR_INPUT, SEQ_ERR_EVENTS);
}

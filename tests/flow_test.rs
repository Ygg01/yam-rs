mod common;

use crate::common::assert_eq_event;

const NULL_YAML_INPUT: &str = r"
null
";

const NULL_YAML2_INPUT: &str = "\r\nnull\r\n";
const NULL_YAML_EVENTS: &str = r"
+DOC
=VAL :null
-DOC";

const MULTI_WORD_INPUT: &str = r"
  null test xy";
const MULTI_WORD_EVENTS: &str = r"
+DOC
=VAL :null test xy
-DOC";

const MULTILINE_INPUT: &str = r"
test
xt
";
const MULTILINE_EVENTS: &str = r"
+DOC
=VAL :test xt
-DOC";

#[test]
fn flow_scalars() {
    assert_eq_event(NULL_YAML_INPUT, NULL_YAML_EVENTS);
    assert_eq_event(NULL_YAML2_INPUT, NULL_YAML_EVENTS);
    assert_eq_event(MULTI_WORD_INPUT, MULTI_WORD_EVENTS);
    assert_eq_event(MULTILINE_INPUT, MULTILINE_EVENTS);
}

const SEQ_FLOW_INPUT: &str = r"
[x, y]
";
const SEQ_FLOW2_INPUT: &str = r"
[x ,y]
";
const SEQ_FLOW_EVENTS: &str = r"
+DOC
+SEQ []
=VAL :x
=VAL :y
-SEQ
-DOC";

#[test]
fn flow_seq() {
    assert_eq_event(SEQ_FLOW_INPUT, SEQ_FLOW_EVENTS);
    assert_eq_event(SEQ_FLOW2_INPUT, SEQ_FLOW_EVENTS);
}

const NEST_COL1_INPUT: &str = r"
[:]
";
const NEST_COL2_INPUT: &str = r"
[{:}]
";

const NESTED_COL_EVENTS: &str = r"
+DOC
+SEQ []
+MAP {}
=VAL :
=VAL :
-MAP
-SEQ
-DOC";

#[test]
fn flow_implicit_map_in_seq() {
    assert_eq_event(NEST_COL1_INPUT, NESTED_COL_EVENTS);
    assert_eq_event(NEST_COL2_INPUT, NESTED_COL_EVENTS);
}

const MAP_XY_INPUT: &str = r"
{x:y}
";
const MAP_XY_EVENTS: &str = r"
+DOC
+MAP {}
=VAL :x:y
=VAL :
-MAP
-DOC";

const MAP_X_Y_INPUT: &str = r"
{x: y}
";
const MAP_X_Y2_INPUT: &str = r"
{? x: y}
";
const MAP_X_Y3_INPUT: &str = r"
{x: #comment
 y}
";
const MAP_X_Y_EVENTS: &str = r"
+DOC
+MAP {}
=VAL :x
=VAL :y
-MAP
-DOC";

const COMPLEX_MAP_INPUT: &str = r"
{[x,y]:a}
";

const COMPLEX_MAP_EVENTS: &str = r"
+DOC
+MAP {}
+SEQ []
=VAL :x
=VAL :y
-SEQ
=VAL :a
-MAP
-DOC";

const X1_9MMW_INPUT: &str = r#"
[ "JSON like":adjacent ]"#;

const X1_9MMW_EVENTS: &str = r#"
+DOC
+SEQ []
+MAP {}
=VAL "JSON like
=VAL :adjacent
-MAP
-SEQ
-DOC"#;

const X2_9MMW_INPUT: &str = r#"
[ {JSON: like}:adjacent ]"#;

const X2_9MMW_EVENTS: &str = r"
+DOC
+SEQ []
+MAP {}
+MAP {}
=VAL :JSON
=VAL :like
-MAP
=VAL :adjacent
-MAP
-SEQ
-DOC";

#[test]
fn flow_complex_map() {
    assert_eq_event(COMPLEX_MAP_INPUT, COMPLEX_MAP_EVENTS);
    assert_eq_event(X1_9MMW_INPUT, X1_9MMW_EVENTS);
    assert_eq_event(X2_9MMW_INPUT, X2_9MMW_EVENTS);
}

#[test]
fn flow_map() {
    assert_eq_event(MAP_XY_INPUT, MAP_XY_EVENTS);
    assert_eq_event(MAP_X_Y2_INPUT, MAP_X_Y_EVENTS);
    assert_eq_event(MAP_X_Y_INPUT, MAP_X_Y_EVENTS);
    assert_eq_event(MAP_X_Y3_INPUT, MAP_X_Y_EVENTS);
}

const FLOW_QUOTED1_INPUT: &str = r#"
{"ab"
: "xy"}
"#;

const FLOW_QUOTED2_INPUT: &str = r#"
{"ab"
:xy}
"#;

const FLOW_QUOTED1_EVENTS: &str = r#"
+DOC
+MAP {}
=VAL "ab
=VAL "xy
-MAP
-DOC"#;

const FLOW_QUOTED2_EVENTS: &str = r#"
+DOC
+MAP {}
=VAL "ab
=VAL :xy
-MAP
-DOC"#;

const X_C2DT_INPUT: &str = r#"
{
"empty":
} "#;
const X_C2DT_EVENTS: &str = r#"
+DOC
+MAP {}
=VAL "empty
=VAL :
-MAP
-DOC"#;

#[test]
fn flow_map_quoted() {
    assert_eq_event(FLOW_QUOTED2_INPUT, FLOW_QUOTED2_EVENTS);
    assert_eq_event(FLOW_QUOTED1_INPUT, FLOW_QUOTED1_EVENTS);
    assert_eq_event(X_C2DT_INPUT, X_C2DT_EVENTS);
}

const EMPTY_MAP1_INPUT: &str = r"
{:}
";

const EMPTY_MAP2_INPUT: &str = r"
{ : }
";

const EMPTY_MAP_EVENTS: &str = r"
+DOC
+MAP {}
=VAL :
=VAL :
-MAP
-DOC";

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

const TWO_EMPTY_INPUT: &str = r"
{:, :}
";

const TWO_EMPTY_EVENTS: &str = r"
+DOC
+MAP {}
=VAL :
=VAL :
=VAL :
=VAL :
-MAP
-DOC";

#[test]
fn flow_empty_nodes() {
    assert_eq_event(EMPTY_MAP1_INPUT, EMPTY_MAP_EVENTS);
    assert_eq_event(EMPTY_MAP2_INPUT, EMPTY_MAP_EVENTS);
    assert_eq_event(TWO_EMPTY_INPUT, TWO_EMPTY_EVENTS);
    assert_eq_event(EMPTY_NODES_INPUT, EMPTY_NODES_EVENTS);
}

const ERR_PLAIN_SCALAR_INPUT: &str = r"
  a
  b
 c";

const ERR_PLAIN_SCALAR_EVENTS: &str = r"
+DOC
=VAL :a b c
-DOC";

#[test]
fn flow_err_plain_scalar() {
    assert_eq_event(ERR_PLAIN_SCALAR_INPUT, ERR_PLAIN_SCALAR_EVENTS);
}

const FLOW_ERR1_INPUT: &str = r"
---
[a, b] ]";

const FLOW_ERR1_EVENTS: &str = r"
+DOC ---
+SEQ []
=VAL :a
=VAL :b
-SEQ
-DOC
ERR";

const FLOW_ERR2_INPUT: &str = r"
---
[ [a, b] ";

const FLOW_ERR2_EVENTS: &str = r"
+DOC ---
+SEQ []
+SEQ []
=VAL :a
=VAL :b
-SEQ
ERR";

const SEQ_ERR_INPUT: &str = r"
 [-]";

const SEQ_ERR_EVENTS: &str = r"
+DOC
+SEQ []
ERR";

const X_9JBA_INPUT: &str = r"
 [a, b]#invalid";

const X_9JBA_EVENTS: &str = r"
+DOC
+SEQ []
=VAL :a
=VAL :b
-SEQ
ERR";

const X_9MAG_INPUT: &str = r"
[ , a , b, c] ";

const X_9MAG_EVENTS: &str = r"
+DOC
+SEQ []
ERR";

const X_CML9_INPUT: &str = r"
key: [ word1
  #  xxx
  word2 ]";

const X_CML9_EVENTS: &str = r"
+DOC
+MAP
=VAL :key
+SEQ []
=VAL :word1
ERR";

const X1_CVW2_INPUT: &str = r"
[a,#comment
]";

const X1_CVW2_EVENTS: &str = r"
+DOC
+SEQ []
=VAL :a
ERR";

const X2_CVW2_INPUT: &str = r"
[a, #comment
]";

const X2_CVW2_EVENTS: &str = r"
+DOC
+SEQ []
=VAL :a
-SEQ
-DOC";

#[test]
fn flow_seq_err() {
    assert_eq_event(X2_CVW2_INPUT, X2_CVW2_EVENTS);
    assert_eq_event(X1_CVW2_INPUT, X1_CVW2_EVENTS);
    assert_eq_event(X_CML9_INPUT, X_CML9_EVENTS);
    assert_eq_event(FLOW_ERR2_INPUT, FLOW_ERR2_EVENTS);
    assert_eq_event(FLOW_ERR1_INPUT, FLOW_ERR1_EVENTS);
    assert_eq_event(SEQ_ERR_INPUT, SEQ_ERR_EVENTS);
    assert_eq_event(X_9JBA_INPUT, X_9JBA_EVENTS);
    assert_eq_event(X_9MAG_INPUT, X_9MAG_EVENTS);
}

const SEQ_KEY1_INPUT: &str = r"
[a, b]: 3 ";

const SEQ_KEY1_EVENTS: &str = r"
+DOC
+MAP
+SEQ []
=VAL :a
=VAL :b
-SEQ
=VAL :3
-MAP
-DOC";

const SEQ_KEY2_INPUT: &str = r"
[a, [b,c]]: 3 ";

const SEQ_KEY2_EVENTS: &str = r"
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
-DOC";

const SEQ_KEY3_INPUT: &str = r"
 [[a]: 3]";

const SEQ_KEY3_EVENTS: &str = r"
+DOC
+SEQ []
+MAP {}
+SEQ []
=VAL :a
-SEQ
=VAL :3
-MAP
-SEQ
-DOC";

const SEQ_KEY4_INPUT: &str = r"
 [ [a]: d, e]: 3";

const SEQ_KEY4_EVENTS: &str = r"
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
-DOC";

#[test]
fn flow_seq_as_key() {
    assert_eq_event(SEQ_KEY1_INPUT, SEQ_KEY1_EVENTS);
    assert_eq_event(SEQ_KEY2_INPUT, SEQ_KEY2_EVENTS);
    assert_eq_event(SEQ_KEY3_INPUT, SEQ_KEY3_EVENTS);
    assert_eq_event(SEQ_KEY4_INPUT, SEQ_KEY4_EVENTS);
}

const SEQ_EDGE_INPUT: &str = r"
 [:x]";

const SEQ_EDGE_EVENTS: &str = r"
+DOC
+SEQ []
=VAL ::x
-SEQ
-DOC";

const X1_8UDB_INPUT: &str = r"
[
single: pair,
]";

const X1_8UDB_EVENTS: &str = r#"
+DOC
+SEQ []
+MAP {}
=VAL :single
=VAL :pair
-MAP
-SEQ
-DOC"#;

const X2_8UDB_INPUT: &str = r"
[[ ],
single: pair,]";

const X2_8UDB_EVENTS: &str = r"
+DOC
+SEQ []
+SEQ []
-SEQ
+MAP {}
=VAL :single
=VAL :pair
-MAP
-SEQ
-DOC";

#[test]
fn flow_seq_edge() {
    assert_eq_event(X1_8UDB_INPUT, X1_8UDB_EVENTS);
    assert_eq_event(X2_8UDB_INPUT, X2_8UDB_EVENTS);
    assert_eq_event(SEQ_EDGE_INPUT, SEQ_EDGE_EVENTS);
}

const MAP_EDGE1_INPUT: &str = r"
 {x: :x}";

const MAP_EDGE1_EVENTS: &str = r"
+DOC
+MAP {}
=VAL :x
=VAL ::x
-MAP
-DOC";

const MAP_EDGE2_INPUT: &str = r"
 {:x}";

const MAP_EDGE2_EVENTS: &str = r"
+DOC
+MAP {}
=VAL ::x
=VAL :
-MAP
-DOC";

const MAP_ERR_INPUT: &str = r"
[23
]: 42";

const MAP_ERR_EVENTS: &str = r"
+DOC
+SEQ []
=VAL :23
ERR";

const X_CT4Q_INPUT: &str = r"
[? foo 
    bar: baz ]";

const X_CT4Q_EVENTS: &str = r"
+DOC
+SEQ []
+MAP {}
=VAL :foo bar
=VAL :baz
-MAP
-SEQ
-DOC";

const X_DFF7_INPUT: &str = r"
{
?
}";
const X_DFF7_EVENTS: &str = r"
+DOC
+MAP {}
=VAL :
=VAL :
-MAP
-DOC";

const X1_DK4H_INPUT: &str = r"
[ key
  : value]";
const X1_DK4H_EVENTS: &str = r"
+DOC
+SEQ []
=VAL :key
ERR";

const X2_DK4H_INPUT: &str = r"
[ ? key
  : value]";
const X2_DK4H_EVENTS: &str = r"
+DOC
+SEQ []
+MAP {}
=VAL :key
=VAL :value
-MAP
-SEQ
-DOC";

#[test]
fn flow_map_edge() {
    assert_eq_event(X_CT4Q_INPUT, X_CT4Q_EVENTS);
    assert_eq_event(X1_DK4H_INPUT, X1_DK4H_EVENTS);
    assert_eq_event(X2_DK4H_INPUT, X2_DK4H_EVENTS);
    assert_eq_event(X_DFF7_INPUT, X_DFF7_EVENTS);
    assert_eq_event(MAP_EDGE1_INPUT, MAP_EDGE1_EVENTS);
    assert_eq_event(MAP_EDGE2_INPUT, MAP_EDGE2_EVENTS);
    assert_eq_event(MAP_ERR_INPUT, MAP_ERR_EVENTS);
}

const FLOW_TAG_INPUT: &str = r"
%TAG !m! !my-
--- # Bulb here
!m!light fluorescent
...";

const FLOW_TAG_EVENTS: &str = r"
+DOC ---
=VAL <!my-light> :fluorescent
-DOC ...";

const X1_EHF6_INPUT: &str = r"
!!map {
    k: !!seq [a, !!str b]
}";

const X1_EHF6_EVENTS: &str = r"
+DOC
+MAP {} <tag:yaml.org,2002:map>
=VAL :k
+SEQ [] <tag:yaml.org,2002:seq>
=VAL :a
=VAL <tag:yaml.org,2002:str> :b
-SEQ
-MAP
-DOC";

#[test]
fn flow_custom_tag() {
    assert_eq_event(X1_EHF6_INPUT, X1_EHF6_EVENTS);
    assert_eq_event(FLOW_TAG_INPUT, FLOW_TAG_EVENTS);
}

const X1_CN3R_INPUT: &str = r"
[
 { &e e: f },
 &g { g: h }
]";

const X1_CN3R_EVENTS: &str = r"
+DOC
+SEQ []
+MAP {}
=VAL &e :e
=VAL :f
-MAP
+MAP {} &g
=VAL :g
=VAL :h
-MAP
-SEQ
-DOC";

const X2_CN3R_INPUT: &str = r"
  { &e e: f }
";

const X2_CN3R_EVENTS: &str = r"
+DOC
+MAP {}
=VAL &e :e
=VAL :f
-MAP
-DOC";

const X3_CN3R_INPUT: &str = r"
[&c c: d]
";

const X3_CN3R_EVENTS: &str = r"
+DOC
+SEQ []
+MAP {}
=VAL &c :c
=VAL :d
-MAP
-SEQ
-DOC";

const X4_CN3R_INPUT: &str = r"
[&g {g: h}]";

const X4_CN3R_EVENTS: &str = r"
+DOC
+SEQ []
+MAP {} &g
=VAL :g
=VAL :h
-MAP
-SEQ
-DOC";

const FLOW_ALIAS_INPUT: &str = r"
&seq [ &item 'a']
";

const FLOW_ALIAS_EVENTS: &str = r"
+DOC
+SEQ [] &seq
=VAL &item 'a
-SEQ
-DOC";

const X1_X38W_INPUT: &str = r"
{&a []: *b}
";

const X1_X38W_EVENTS: &str = r"
+DOC
+MAP {}
+SEQ [] &a
-SEQ
=ALI *b
-MAP
-DOC";

#[test]
fn flow_anchor() {
    assert_eq_event(X1_X38W_INPUT, X1_X38W_EVENTS);

    assert_eq_event(X4_CN3R_INPUT, X4_CN3R_EVENTS);
    assert_eq_event(X3_CN3R_INPUT, X3_CN3R_EVENTS);
    assert_eq_event(X2_CN3R_INPUT, X2_CN3R_EVENTS);
    assert_eq_event(X1_CN3R_INPUT, X1_CN3R_EVENTS);
    assert_eq_event(FLOW_ALIAS_INPUT, FLOW_ALIAS_EVENTS);
}

const X1_Y79Y_003_INPUT: &str = r"
- [
 	foo,
 foo,
	 foo,
 ]";

const X1_Y79Y_003_EVENTS: &str = r"
+DOC
+SEQ
+SEQ []
=VAL :foo
=VAL :foo
ERR";

#[test]
fn flow_in_seq_indents() {
    assert_eq_event(X1_Y79Y_003_INPUT, X1_Y79Y_003_EVENTS);
}

const X1_5T43_INPUT: &str = r#"
- { "key":value }
- { "key"::value }"#;

const X1_5T43_EVENTS: &str = r#"
+DOC
+SEQ
+MAP {}
=VAL "key
=VAL :value
-MAP
+MAP {}
=VAL "key
=VAL ::value
-MAP
-SEQ
-DOC"#;

#[test]
fn flow_mix() {
    assert_eq_event(X1_5T43_INPUT, X1_5T43_EVENTS);
}

const X1_FRK4_INPUT: &str = r"
{
    ? foo :,
    : bar,
}";

const X1_FRK4_EVENTS: &str = r"
+DOC
+MAP {}
=VAL :foo
=VAL :
=VAL :
=VAL :bar
-MAP
-DOC";

#[test]
fn flow_exp_map() {
    assert_eq_event(X1_FRK4_INPUT, X1_FRK4_EVENTS);
}

use crate::common::assert_eq_event;

mod common;

const BLOCK1_INPUT: &str = r#"
 - x
 - y
"#;

const BLOCK2_INPUT: &str = r#"
- x
- y
"#;

const BLOCK_EVENTS: &str = r#"
 +DOC
  +SEQ
   =VAL :x
   =VAL :y
  -SEQ
 -DOC"#;

const SEQ_PLAIN_INPUT: &str = r#"
  - x
   - y
"#;

const SEQ_PLAIN2_INPUT: &str = r#"
- x - y
"#;

const SEQ_PLAIN_EVENTS: &str = r#"
 +DOC
  +SEQ
   =VAL :x - y
  -SEQ
 -DOC"#;

#[test]
pub fn block_seq() {
    assert_eq_event(BLOCK1_INPUT, BLOCK_EVENTS);
    assert_eq_event(BLOCK2_INPUT, BLOCK_EVENTS);
    assert_eq_event(SEQ_PLAIN_INPUT, SEQ_PLAIN_EVENTS);
    assert_eq_event(SEQ_PLAIN2_INPUT, SEQ_PLAIN_EVENTS);
}

const BLOCK_ERR_INPUT: &str = r#"
  - x
 - y
"#;

const BLOCK_ERR_EVENTS: &str = r#"
 +DOC
  +SEQ
   =VAL :x
  -SEQ
  ERR
 -DOC
 ERR"#;

const WRONG_SEQ_INDENT_INPUT: &str = r#"
a: 
  - b
 - c
"#;

const WRONG_SEQ_INDENT_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :a
   +SEQ
    =VAL :b
   -SEQ
   ERR
   =VAL :c
  -MAP
 -DOC"#;

#[test]
pub fn block_seq_err() {
    assert_eq_event(BLOCK_ERR_INPUT, BLOCK_ERR_EVENTS);
    assert_eq_event(WRONG_SEQ_INDENT_INPUT, WRONG_SEQ_INDENT_EVENTS);
}

const BLOCK_NESTED_SEQ_INPUT: &str = r#"
  - - a
    - b
  - c
"#;

const BLOCK_NESTED_SEQ_EVENTS: &str = r#"
 +DOC
  +SEQ
   +SEQ
    =VAL :a
    =VAL :b
   -SEQ
   =VAL :c
  -SEQ
 -DOC"#;

const BLOCK_NESTED_SEQ2_INPUT: &str = r#"
  - - a
    - b
    - - c
  - d
"#;

const BLOCK_NESTED_SEQ2_EVENTS: &str = r#"
 +DOC
  +SEQ
   +SEQ
    =VAL :a
    =VAL :b
    +SEQ
     =VAL :c
    -SEQ
   -SEQ
   =VAL :d
  -SEQ
 -DOC"#;

#[test]
pub fn seq_block_nested() {
    assert_eq_event(BLOCK_NESTED_SEQ_INPUT, BLOCK_NESTED_SEQ_EVENTS);
    assert_eq_event(BLOCK_NESTED_SEQ2_INPUT, BLOCK_NESTED_SEQ2_EVENTS);
}

const FOLD_STR1_INPUT: &str = r#"
  - >1-
   1
    2
   3
   4
   
"#;

const FOLD_STR1_EVENTS: &str = r#"
 +DOC
  +SEQ
   =VAL >1\n 2\n3 4
  -SEQ
 -DOC"#;

const FOLD_ERR_INPUT: &str = r#"
 >
    
 invalid
"#;

const FOLD_ERR_EVENTS: &str = r#"
 +DOC
  ERR
  =VAL >\ninvalid\n
 -DOC"#;

const FOLD_STR2_INPUT: &str = r#"
 >
 
  
  valid
"#;

const FOLD_STR2_EVENTS: &str = r#"
 +DOC
  =VAL >\n\nvalid\n
 -DOC"#;

#[test]
pub fn block_fold() {
    assert_eq_event(FOLD_STR1_INPUT, FOLD_STR1_EVENTS);
    assert_eq_event(FOLD_STR2_INPUT, FOLD_STR2_EVENTS);
    assert_eq_event(FOLD_ERR_INPUT, FOLD_ERR_EVENTS);
}

const BLOCK_PLAIN_INPUT: &str = r#"
  a
  b
  c
    d
  e
"#;

const BLOCK_PLAIN_EVENTS: &str = r#"
 +DOC
  =VAL :a b c d e
 -DOC"#;

const BLOCK_PLAIN2_INPUT: &str = r#"
a
b  
  c
d

e

"#;

const BLOCK_PLAIN2_EVENTS: &str = r#"
 +DOC
  =VAL :a b c d\ne
 -DOC"#;

#[test]
pub fn block_plain_scalar() {
    assert_eq_event(BLOCK_PLAIN_INPUT, BLOCK_PLAIN_EVENTS);
    assert_eq_event(BLOCK_PLAIN2_INPUT, BLOCK_PLAIN2_EVENTS);
}

const BLOCK_FOLD_INPUT: &str = r#"
>
 a
 b
 
 c
 
 
 d"#;

const BLOCK_FOLD_EVENTS: &str = r#"
 +DOC
  =VAL >a b\nc\n\nd\n
 -DOC"#;

const SIMPLE_LITERAL1_INPUT: &str = r#"
 --- >1+"#;

const SIMPLE_LITERAL2_INPUT: &str = r#"
 --- >1-"#;

const SIMPLE_LITERAL_EVENTS: &str = r#"
 +DOC ---
  =VAL >
 -DOC"#;

#[test]
pub fn block_fold_literal() {
    assert_eq_event(BLOCK_FOLD_INPUT, BLOCK_FOLD_EVENTS);
    assert_eq_event(SIMPLE_LITERAL1_INPUT, SIMPLE_LITERAL_EVENTS);
    assert_eq_event(SIMPLE_LITERAL2_INPUT, SIMPLE_LITERAL_EVENTS);
}

const LITERAL1_INPUT: &str = r#"
--- |1+ #tsts"#;

const LITERAL2_INPUT: &str = r#"
--- |1-"#;

const SIMPLE_FOLDED_EVENTS: &str = r#"
 +DOC ---
  =VAL |
 -DOC"#;

const LIT_STR2_INPUT: &str = r#"
strip: |-
  text
clip: |
  text
keep: |
  text"#;

const LIT_STR2_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :strip
   =VAL |text
   =VAL :clip
   =VAL |text\n
   =VAL :keep
   =VAL |text\n
  -MAP
 -DOC"#;

const MULTILINE_PLAIN_INPUT: &str = r##"
generic: !!str |
 test
 test
"##;

const MULTILINE_PLAIN_EVENTS: &str = r##"
 +DOC
  +MAP
   =VAL :generic
   =VAL <tag:yaml.org,2002:str> |test\ntest\n
  -MAP
 -DOC"##;

const BLOCK_QUOTE_INPUT: &str = r#"
 plain: 
   spans
   lines
 
 quoted: 
   "text"
"#;

const BLOCK_QUOTE_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :plain
   =VAL :spans lines
   =VAL :quoted
   =VAL "text
  -MAP
 -DOC"#;

#[test]
pub fn block_literal() {
    assert_eq_event(LITERAL1_INPUT, SIMPLE_FOLDED_EVENTS);
    assert_eq_event(LITERAL2_INPUT, SIMPLE_FOLDED_EVENTS);
    assert_eq_event(LIT_STR2_INPUT, LIT_STR2_EVENTS);
    assert_eq_event(MULTILINE_PLAIN_INPUT, MULTILINE_PLAIN_EVENTS);
    assert_eq_event(BLOCK_QUOTE_INPUT, BLOCK_QUOTE_EVENTS);
}
const LITERAL_ERR_INPUT: &str = r#"
--- |0"#;

const LITERAL_ERR2_INPUT: &str = r#"
--- |+10"#;

const SIMPLE_FOLDED_ERR_EVENTS: &str = r#"
 +DOC ---
  ERR
 -DOC"#;

#[test]
pub fn block_literal_err() {
    assert_eq_event(LITERAL_ERR_INPUT, SIMPLE_FOLDED_ERR_EVENTS);
    assert_eq_event(LITERAL_ERR2_INPUT, SIMPLE_FOLDED_ERR_EVENTS);
}

const PLAIN_MULTI_INPUT: &str = r#"
1st line

 2nd non
	3rd non
"#;

const PLAIN_MULTI_EVENTS: &str = r#"
 +DOC
  =VAL :1st line\n2nd non 3rd non
 -DOC"#;

#[test]
pub fn block_plain_multiline() {
    assert_eq_event(PLAIN_MULTI_INPUT, PLAIN_MULTI_EVENTS)
}

const MAP2_INPUT: &str = r#"
:
a: b
: c
d:
"#;

const MAP2_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :
   =VAL :
   =VAL :a
   =VAL :b
   =VAL :
   =VAL :c
   =VAL :d
   =VAL :
  -MAP
 -DOC"#;

const MAP_NESTED_INPUT: &str = r#"
a :
 b:
  c:
d:"#;

const MAP_NESTED_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :a
   +MAP
    =VAL :b
    +MAP
     =VAL :c
     =VAL :
    -MAP
   -MAP
   =VAL :d
   =VAL :
  -MAP
 -DOC"#;

const MAP_SIMPLE_INPUT: &str = r#"
a: b
"#;

const MAP_SIMPLE2_INPUT: &str = r#"
a: 
  b
"#;

const MAP_SIMPLE_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :a
   =VAL :b
  -MAP
 -DOC"#;

#[test]
pub fn block_map() {
    assert_eq_event(MAP_SIMPLE_INPUT, MAP_SIMPLE_EVENTS);
    assert_eq_event(MAP_SIMPLE2_INPUT, MAP_SIMPLE_EVENTS);
    assert_eq_event(MAP2_INPUT, MAP2_EVENTS);
    assert_eq_event(MAP_NESTED_INPUT, MAP_NESTED_EVENTS);
}

const DQUOTE_MAP_INPUT: &str = r##"
quote: "a\/b"
"##;

const DQUOTE_MAP_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :quote
   =VAL "a/b
  -MAP
 -DOC"#;

const DQUOTE_MUL_INPUT: &str = r##"
quoted: "multi
  line"
 "##;

const DQUOTE_MUL_EVENTS: &str = r##"
 +DOC
  +MAP
   =VAL :quoted
   =VAL "multi line
  -MAP
 -DOC"##;

#[test]
pub fn block_quote_map() {
    assert_eq_event(DQUOTE_MAP_INPUT, DQUOTE_MAP_EVENTS);
    assert_eq_event(DQUOTE_MUL_INPUT, DQUOTE_MUL_EVENTS);
}

const EMPTY_MAP_INPUT: &str = r#"
:"#;

const EMPTY_MAP_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :
   =VAL :
  -MAP
 -DOC"#;

const EMPTY_MAP2_INPUT: &str = r#"
:
 a"#;

const EMPTY_MAP2_1_INPUT: &str = r#"
: a"#;

const EMPTY_MAP2_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :
   =VAL :a
  -MAP
 -DOC"#;

const MIX_EMPTY_MAP_INPUT: &str = r#"
 a:
   x
   u
 c :
"#;

const MIX_EMPTY_MAP_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :a
   =VAL :x u
   =VAL :c
   =VAL :
  -MAP
 -DOC"#;

#[test]
pub fn block_empty_map() {
    assert_eq_event(EMPTY_MAP_INPUT, EMPTY_MAP_EVENTS);
    assert_eq_event(EMPTY_MAP2_INPUT, EMPTY_MAP2_EVENTS);
    assert_eq_event(EMPTY_MAP2_1_INPUT, EMPTY_MAP2_EVENTS);
    assert_eq_event(MIX_EMPTY_MAP_INPUT, MIX_EMPTY_MAP_EVENTS);
}

const MULTILINE_COMMENT1_INPUT: &str = r#"
  mul: 
    abc  # a comment
"#;

const MULTILINE_COMMENT1_2_INPUT: &str = r#"
  mul  : 
    abc  # a comment
"#;

const MULTILINE_COMMENT1_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :mul
   =VAL :abc
  -MAP
 -DOC"#;

const MULTILINE_COMMENT2_INPUT: &str = r#"
  multi:
    ab  # a comment
    xyz  # a commeent
"#;

const MULTILINE_COMMENT2_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :multi
   =VAL :ab
   ERR
   =VAL :xyz
  -MAP
 -DOC"#;

const MULTILINE_COMMENT3_INPUT: &str = r#"
  multi:
    ab  
    xyz  # a commeent
"#;

const MULTILINE_COMMENT3_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :multi
   =VAL :ab xyz
  -MAP
 -DOC"#;

#[test]
pub fn block_multiline_comment() {
    assert_eq_event(MULTILINE_COMMENT1_INPUT, MULTILINE_COMMENT1_EVENTS);
    assert_eq_event(MULTILINE_COMMENT1_2_INPUT, MULTILINE_COMMENT1_EVENTS);
    assert_eq_event(MULTILINE_COMMENT2_INPUT, MULTILINE_COMMENT2_EVENTS);
    assert_eq_event(MULTILINE_COMMENT3_INPUT, MULTILINE_COMMENT3_EVENTS);
}

const EXP_MAP_INPUT: &str = r#"
  ? test
  : value
"#;

const EXP_BLOCK_MAP_MIX_INPUT: &str = r#"
  ? test
  : value
  tx: x
"#;

const EXP_MAP_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :test
   =VAL :value
  -MAP
 -DOC"#;

const EXP_BLOCK_MAP_MIX_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :test
   =VAL :value
   =VAL :tx
   =VAL :x
  -MAP
 -DOC"#;

const EXP_MAP_FOLD_INPUT: &str = r#"
 ? >
   test
 : x
"#;

const EXP_MAP_FOLD_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL >test\n
   =VAL :x
  -MAP
 -DOC"#;

#[test]
pub fn block_exp_map() {
    assert_eq_event(EXP_MAP_INPUT, EXP_MAP_EVENTS);
    assert_eq_event(EXP_MAP_FOLD_INPUT, EXP_MAP_FOLD_EVENTS);
    assert_eq_event(EXP_BLOCK_MAP_MIX_INPUT, EXP_BLOCK_MAP_MIX_EVENTS);
}

const EXP_MAP_EMPTY_INPUT: &str = r#"
? a
? b 
? c
"#;

const EXP_MAP_EMPTY_INPUT_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :a
   =VAL :
   =VAL :b
   =VAL :
   =VAL :c
   =VAL :
  -MAP
 -DOC"#;

const EXP_MAP_FAKE_EMPTY_INPUT: &str = r#"
  ? x
   ? x
"#;

const EXP_MAP_FAKE_EMPTY_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :x ? x
   =VAL :
  -MAP
 -DOC"#;

#[test]
pub fn block_empty_node_exp_map() {
    assert_eq_event(EXP_MAP_EMPTY_INPUT, EXP_MAP_EMPTY_INPUT_EVENTS);
    assert_eq_event(EXP_MAP_FAKE_EMPTY_INPUT, EXP_MAP_FAKE_EMPTY_EVENTS);
}
const EMPTY_KEY_MAP_INPUT: &str = r#"
: a
: b
"#;

const EMPTY_KEY_MAP_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :
   =VAL :a
   =VAL :
   =VAL :b
  -MAP
 -DOC"#;
#[test]
pub fn block_empty_node_map() {
    assert_eq_event(EMPTY_KEY_MAP_INPUT, EMPTY_KEY_MAP_EVENTS);
}

const EXP_BLOCK_MAP_ERR1: &str = r#"
   ? test
  : value
"#;

const EXP_BLOCK_MAP_ERR1_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :test
   ERR
   =VAL :value
  -MAP
 -DOC"#;

const EXP_BLOCK_MAP_ERR2: &str = r#"
 ? test
  : value
"#;

const EXP_BLOCK_MAP_ERR2_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :test
   ERR
   =VAL :value
  -MAP
 -DOC"#;

#[test]
pub fn block_exp_map_err() {
    assert_eq_event(EXP_BLOCK_MAP_ERR1, EXP_BLOCK_MAP_ERR1_EVENTS);
    assert_eq_event(EXP_BLOCK_MAP_ERR2, EXP_BLOCK_MAP_ERR2_EVENTS);
}

const ERR_MULTILINE_KEY_INPUT: &str = "
invalid
 key :  x";

const ERR_MULTILINE_KEY_EVENTS: &str = "
 +DOC
  ERR
  +MAP
   =VAL :invalid key
   =VAL :x
  -MAP
 -DOC";

const ERR_INVALID_KEY1_INPUT: &str = "
a:
  b
c";

const ERR_INVALID_KEY1_EVENTS: &str = "
 +DOC
  +MAP
   =VAL :a
   =VAL :b
   ERR
   =VAL :c
  -MAP
 -DOC";

const ERR_INVALID_KEY2_INPUT: &str = r#"
 a:
   b
 "c
  x""#;

const ERR_INVALID_KEY2_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :a
   =VAL :b
   ERR
   =VAL "c x
  -MAP
 -DOC"#;

#[test]
pub fn block_map_err() {
    assert_eq_event(ERR_MULTILINE_KEY_INPUT, ERR_MULTILINE_KEY_EVENTS);
    assert_eq_event(ERR_INVALID_KEY1_INPUT, ERR_INVALID_KEY1_EVENTS);
    assert_eq_event(ERR_INVALID_KEY2_INPUT, ERR_INVALID_KEY2_EVENTS);
}

const COMPLEX_KEYS_INPUT: &str = r##"
a!"#$%&'()*+,-./09:;<=>?@AZ[\]^_`az{|}~: safe
:foo: baz
-foo: boo
"##;

const COMPLEX_KEYS_EVENTS: &str = r##"
 +DOC
  +MAP
   =VAL :a!"#$%&'()*+,-./09:;<=>?@AZ[\\]^_`az{|}~
   =VAL :safe
   =VAL ::foo
   =VAL :baz
   =VAL :-foo
   =VAL :boo
  -MAP
 -DOC"##;

#[test]
pub fn test_complex_block() {
    assert_eq_event(COMPLEX_KEYS_INPUT, COMPLEX_KEYS_EVENTS);
}

const MAPS_WITH_QUOTES_INPUT: &str = r#"
"double" : 
  'single'  :   &alias plain
"#;

const MAPS_WITH_QUOTES_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL "double
   +MAP
    =VAL 'single
    =VAL &alias :plain
   -MAP
  -MAP
 -DOC"#;

#[test]
pub fn block_map_scalar_and_ws() {
    assert_eq_event(MAPS_WITH_QUOTES_INPUT, MAPS_WITH_QUOTES_EVENTS);
}

const NESTED_MAPS_INPUT: &str = r#"
"top1" : 
  'key1' : 
    down : test
'top2' :  
  *x1 :  scalar2
"#;

const NESTED_MAPS_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL "top1
   +MAP
    =VAL 'key1
    +MAP
     =VAL :down
     =VAL :test
    -MAP
   -MAP
   =VAL 'top2
   +MAP
    =ALI *x1
    =VAL :scalar2
   -MAP
  -MAP
 -DOC"#;

#[test]
pub fn block_nested_maps() {
    assert_eq_event(NESTED_MAPS_INPUT, NESTED_MAPS_EVENTS);
}

const ALIAS_N_MAPS_INPUT: &str = r#"
"top1" : &node
  &node2 'key1' : 'val'

'top2' :  
  *x1 :  scalar2
"#;

const ALIAS_N_MAPS_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL "top1
   +MAP &node
    =VAL &node2 'key1
    =VAL 'val
   -MAP
   =VAL 'top2
   +MAP
    =ALI *x1
    =VAL :scalar2
   -MAP
  -MAP
 -DOC"#;

const ALIAS_N_MAPS2_INPUT: &str = r#"
top3: &node3 
  *alias1 : scalar3
 "#;

const ALIAS_N_MAPS2_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :top3
   +MAP &node3
    =ALI *alias1
    =VAL :scalar3
   -MAP
  -MAP
 -DOC"#;

#[test]
pub fn block_map_anchor_alias() {
    assert_eq_event(ALIAS_N_MAPS_INPUT, ALIAS_N_MAPS_EVENTS);
    assert_eq_event(ALIAS_N_MAPS2_INPUT, ALIAS_N_MAPS2_EVENTS);
}

const ALIAS_N_SEQ1_INPUT: &str = r#"
&seq
 - a
 "#;

const ALIAS_N_SEQ1_EVENTS: &str = r#"
 +DOC
  +SEQ &seq
   =VAL :a
  -SEQ
 -DOC"#;

const ALIAS_N_SEQ2_INPUT: &str = r#"
 &seq  - a
  "#;

const ALIAS_N_SEQ2_EVENTS: &str = r#"
 +DOC
  ERR
  +SEQ &seq
   ERR
   =VAL :a
  -SEQ
 -DOC"#;

const ALIAS_N_SEQ3_INPUT: &str = r#"
  - &node a
  "#;

const ALIAS_N_SEQ3_EVENTS: &str = r#"
 +DOC
  +SEQ
   =VAL &node :a
  -SEQ
 -DOC"#;

#[test]
pub fn block_seq_anchor_alias() {
    assert_eq_event(ALIAS_N_SEQ1_INPUT, ALIAS_N_SEQ1_EVENTS);
    assert_eq_event(ALIAS_N_SEQ2_INPUT, ALIAS_N_SEQ2_EVENTS);
    assert_eq_event(ALIAS_N_SEQ3_INPUT, ALIAS_N_SEQ3_EVENTS);
}

const SEQ_AND_TAG_INPUT: &str = r#"
  sequence: !!seq
  - a
  - !!str
    - b
  mapping: !!map
    foo: bar
"#;

const SEQ_AND_TAG_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :sequence
   +SEQ <tag:yaml.org,2002:seq>
    =VAL :a
    +SEQ <tag:yaml.org,2002:str>
     =VAL :b
    -SEQ
   -SEQ
   =VAL :mapping
   +MAP <tag:yaml.org,2002:map>
    =VAL :foo
    =VAL :bar
   -MAP
  -MAP
 -DOC"#;

#[test]
pub fn block_col_tags() {
    assert_eq_event(SEQ_AND_TAG_INPUT, SEQ_AND_TAG_EVENTS);
}

const ANCHOR_COLON_INPUT: &str = r#"
&node3:  key : scalar3
*node3: : x"#;

const ANCHOR_COLON_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL &node3: :key
   =VAL :scalar3
   =ALI *node3:
   =VAL :x
  -MAP
 -DOC"#;

const ANCHOR_MULTI_INPUT: &str = r#"
top2: &node2
  &v2 val: x"#;

const ANCHOR_MULTI_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :top2
   +MAP &node2
    =VAL &v2 :val
    =VAL :x
   -MAP
  -MAP
 -DOC"#;

const ANCHOR_ERR_INPUT: &str = r#"
top2: &node2
  &v2 val"#;

const ANCHOR_ERR_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :top2
   ERR
   =VAL &v2 :val
  -MAP
 -DOC"#;

#[test]
pub fn block_anchor() {
    assert_eq_event(ANCHOR_COLON_INPUT, ANCHOR_COLON_EVENTS);
    assert_eq_event(ANCHOR_MULTI_INPUT, ANCHOR_MULTI_EVENTS);
    assert_eq_event(ANCHOR_ERR_INPUT, ANCHOR_ERR_EVENTS);
}

const MIX_BLOCK_INPUT: &str = r##"
-
  key: x
  val: 8
- 
  val: y
"##;

const MIX_BLOCK_EVENTS: &str = r##"
 +DOC
  +SEQ
   +MAP
    =VAL :key
    =VAL :x
    =VAL :val
    =VAL :8
   -MAP
   +MAP
    =VAL :val
    =VAL :y
   -MAP
  -SEQ
 -DOC"##;

const MIX2_BLOCK_INPUT: &str = r##"
  sequence:
  - a
  mapping:
   foo: bar
 "##;

const MIX2_BLOCK_EVENTS: &str = r##"
 +DOC
  +MAP
   =VAL :sequence
   +SEQ
    =VAL :a
   -SEQ
   =VAL :mapping
   +MAP
    =VAL :foo
    =VAL :bar
   -MAP
  -MAP
 -DOC"##;

#[test]
pub fn block_mix_seq() {
    assert_eq_event(MIX_BLOCK_INPUT, MIX_BLOCK_EVENTS);
    assert_eq_event(MIX2_BLOCK_INPUT, MIX2_BLOCK_EVENTS);
}

const TAG1_INPUT: &str = r#"
 !!str a"#;

const TAG1_EVENTS: &str = r#"
 +DOC
  =VAL <tag:yaml.org,2002:str> :a
 -DOC"#;

const COMPLEX_TAG2_INPUT: &str = r#"
- !!str c
--- !!str
d
e"#;

const COMPLEX_TAG2_EVENTS: &str = r#"
 +DOC
  +SEQ
   =VAL <tag:yaml.org,2002:str> :c
  -SEQ
 -DOC
 +DOC ---
  =VAL <tag:yaml.org,2002:str> :d e
 -DOC"#;

#[test]
fn parse_tag() {
    assert_eq_event(TAG1_INPUT, TAG1_EVENTS);
    assert_eq_event(COMPLEX_TAG2_INPUT, COMPLEX_TAG2_EVENTS);
}

const MULTI_LINE_INPUT: &str = r#"
x: a
 b

 c"#;

const MULTI_LINE_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :x
   =VAL :a b\nc
  -MAP
 -DOC"#;

const MULTI_LINE_SEQ_INPUT: &str = r#"
- a 
 b

 c"#;

const MULTI_LINE_SEQ_EVENTS: &str = r#"
 +DOC
  +SEQ
   =VAL :a b\nc
  -SEQ
 -DOC"#;

#[test]
fn multi_line_value() {
    assert_eq_event(MULTI_LINE_INPUT, MULTI_LINE_EVENTS);
    assert_eq_event(MULTI_LINE_SEQ_INPUT, MULTI_LINE_SEQ_EVENTS);
}

const INDENT_TAB_INPUT: &str = r#"
a: 
	b: c
"#;

const INDENT_TAB_EVENTS: &str = r#"
 +DOC
  +MAP
   =VAL :a
   ERR
   +MAP
    =VAL :b
    =VAL :c
   -MAP
  -MAP
 -DOC"#;

#[test]
fn block_invalid_map_tabs() {
    assert_eq_event(INDENT_TAB_INPUT, INDENT_TAB_EVENTS);
}

use crate::common::assert_eq_event;

mod common;

const BLOCK1_INPUT: &str = r"
    - x
    - y
";

const BLOCK2_INPUT: &str = r"
- x
- y
";

const BLOCK_EVENTS: &str = r"
+DOC
+SEQ
=VAL :x
=VAL :y
-SEQ
-DOC";

const SEQ_PLAIN_INPUT: &str = r"
  - x
   - y
";

const SEQ_PLAIN2_INPUT: &str = r"
- x - y
";

const SEQ_PLAIN_EVENTS: &str = r"
+DOC
+SEQ
=VAL :x - y
-SEQ
-DOC";

#[test]
fn block_seq() {
    assert_eq_event(BLOCK1_INPUT, BLOCK_EVENTS);
    assert_eq_event(BLOCK2_INPUT, BLOCK_EVENTS);
    assert_eq_event(SEQ_PLAIN_INPUT, SEQ_PLAIN_EVENTS);
    assert_eq_event(SEQ_PLAIN2_INPUT, SEQ_PLAIN_EVENTS);
}

const BLOCK_ERR_INPUT: &str = r"
  - x
 - y
";

const BLOCK_ERR_EVENTS: &str = r"
+DOC
+SEQ
=VAL :x
-SEQ
ERR
=VAL :y
-DOC";

const WRONG_SEQ_INDENT_INPUT: &str = r"
a: 
  - b
 - c
";

const WRONG_SEQ_INDENT_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
+SEQ
=VAL :b
-SEQ
ERR
ERR
=VAL :c
-MAP
-DOC";

const SEQ_NO_MINUS_INPUT: &str = r"
map:
 - a
 c
";

const SEQ_NO_MINUS_EVENTS: &str = r"
+DOC
+MAP
=VAL :map
+SEQ
=VAL :a
ERR
-SEQ
=VAL :c
-MAP
-DOC";

const X_9CWY_INPUT: &str = r"
key:
 - item1
 - item2
invalid
";
const X_9CWY_EVENTS: &str = r"
+DOC
+MAP
=VAL :key
+SEQ
=VAL :item1
=VAL :item2
-SEQ
ERR
=VAL :invalid
-MAP
-DOC";

const X_BD7L_INPUT: &str = r"
- item
invalid: x
";

const X_BD7L_EVENTS: &str = r"
+DOC
+SEQ
=VAL :item
ERR
+MAP
=VAL :invalid
=VAL :x
-MAP
-SEQ
-DOC";

#[test]
fn block_seq_err() {
    assert_eq_event(SEQ_NO_MINUS_INPUT, SEQ_NO_MINUS_EVENTS);
    assert_eq_event(X_BD7L_INPUT, X_BD7L_EVENTS);
    assert_eq_event(X_9CWY_INPUT, X_9CWY_EVENTS);
    assert_eq_event(BLOCK_ERR_INPUT, BLOCK_ERR_EVENTS);
    assert_eq_event(WRONG_SEQ_INDENT_INPUT, WRONG_SEQ_INDENT_EVENTS);
}

const X1_3ALJ_INPUT: &str = r"
 - - s1_i1
   - s1_i2
 - s2
";

const X2_3ALJ_INPUT: &str = r"
- - s1_i1
  - s1_i2
- s2
";

const X_3ALJ_EVENTS: &str = r"
+DOC
+SEQ
+SEQ
=VAL :s1_i1
=VAL :s1_i2
-SEQ
=VAL :s2
-SEQ
-DOC";

const BLOCK_NESTED_SEQ2_INPUT: &str = r"
  - - a
    - b
    - - c
  - d
";

const BLOCK_NESTED_SEQ2_EVENTS: &str = r"
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
-DOC";

#[test]
fn seq_block_nested() {
    assert_eq_event(X1_3ALJ_INPUT, X_3ALJ_EVENTS);
    assert_eq_event(X2_3ALJ_INPUT, X_3ALJ_EVENTS);
    assert_eq_event(BLOCK_NESTED_SEQ2_INPUT, BLOCK_NESTED_SEQ2_EVENTS);
}

const FOLD_STR1_INPUT: &str = r"
  - >1-
   1
    2
   3
   4
   
";

const FOLD_STR1_EVENTS: &str = r"
+DOC
+SEQ
=VAL >1\n 2\n3 4
-SEQ
-DOC";

const FOLD_ERR_INPUT: &str = r"
 >
    
 invalid
";

const FOLD_ERR_EVENTS: &str = r"
+DOC
ERR
=VAL >\ninvalid\n
-DOC";

const FOLD_STR2_INPUT: &str = r"
 >
 
  
  valid
";

const FOLD_STR2_EVENTS: &str = r"
+DOC
=VAL >\n\nvalid\n
-DOC";

#[test]
fn block_fold() {
    assert_eq_event(FOLD_STR1_INPUT, FOLD_STR1_EVENTS);
    assert_eq_event(FOLD_STR2_INPUT, FOLD_STR2_EVENTS);
    assert_eq_event(FOLD_ERR_INPUT, FOLD_ERR_EVENTS);
}

const BLOCK_PLAIN_INPUT: &str = r"
  a
  b
  c
    d
  e
";

const BLOCK_PLAIN_EVENTS: &str = r"
+DOC
=VAL :a b c d e
-DOC";

const BLOCK_PLAIN2_INPUT: &str = r"
a
b  
  c
d

e

";

const BLOCK_PLAIN2_EVENTS: &str = r"
+DOC
=VAL :a b c d\ne
-DOC";

const BLOCK_MULTI_INPUT: &str = r"
    word1
    # comment
    word2
";

const BLOCK_MULTI_EVENTS: &str = r"
+DOC
=VAL :word1
-DOC
ERR";

#[test]
fn block_plain_scalar() {
    assert_eq_event(BLOCK_MULTI_INPUT, BLOCK_MULTI_EVENTS);
    assert_eq_event(BLOCK_PLAIN_INPUT, BLOCK_PLAIN_EVENTS);
    assert_eq_event(BLOCK_PLAIN2_INPUT, BLOCK_PLAIN2_EVENTS);
}

const BLOCK_FOLD_INPUT: &str = r"
>
 a
 b
 
 c
 
 
 d";

const BLOCK_FOLD_EVENTS: &str = r"
+DOC
=VAL >a b\nc\n\nd\n
-DOC";

const SIMPLE_FOLD1_INPUT: &str = r"
--- >1+";

const SIMPLE_FOLD2_INPUT: &str = r"
--- >1-";

const SIMPLE_FOLD_EVENTS: &str = r"
+DOC ---
=VAL >
-DOC";

#[test]
fn block_fold_literal() {
    assert_eq_event(BLOCK_FOLD_INPUT, BLOCK_FOLD_EVENTS);
    assert_eq_event(SIMPLE_FOLD1_INPUT, SIMPLE_FOLD_EVENTS);
    assert_eq_event(SIMPLE_FOLD2_INPUT, SIMPLE_FOLD_EVENTS);
}

const LITERAL1_INPUT: &str = r"
--- |1+ #tsts";

const LITERAL2_INPUT: &str = r"
--- |1-";

const SIMPLE_FOLDED_EVENTS: &str = r"
+DOC ---
=VAL |
-DOC";

const LIT_STR2_INPUT: &str = r"
strip: |-
  text
clip: |
  text
keep: |+
  text";

const LIT_STR2_EVENTS: &str = r"
+DOC
+MAP
=VAL :strip
=VAL |text
=VAL :clip
=VAL |text\n
=VAL :keep
=VAL |text\n
-MAP
-DOC";

const MULTILINE_PLAIN_INPUT: &str = r"
generic: !!str |
 test
 test
";

const MULTILINE_PLAIN_EVENTS: &str = r"
+DOC
+MAP
=VAL :generic
=VAL <tag:yaml.org,2002:str> |test\ntest\n
-MAP
-DOC";

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

const LITERAL3_INPUT: &str = r"
--- |+
 ab
 
  
...";

const LITERAL3_EVENTS: &str = r"
+DOC ---
=VAL |ab\n\n \n
-DOC ...";

const LITERAL_CHOMP_INPUT: &str = r"
Chomping: |
  Clipped
 
";

const LITERAL_CHOMP_EVENTS: &str = r"
+DOC
+MAP
=VAL :Chomping
=VAL |Clipped\n
-MAP
-DOC";

const LITERAL_ESCAPE_INPUT: &str = r"
block: |
  Hello\n";

const LITERAL_ESCAPE_EVENTS: &str = r"
+DOC
+MAP
=VAL :block
=VAL |Hello\\n\n
-MAP
-DOC";

#[test]
fn block_literal() {
    assert_eq_event(LITERAL_ESCAPE_INPUT, LITERAL_ESCAPE_EVENTS);
    assert_eq_event(LITERAL1_INPUT, SIMPLE_FOLDED_EVENTS);
    assert_eq_event(LITERAL2_INPUT, SIMPLE_FOLDED_EVENTS);
    assert_eq_event(BLOCK_QUOTE_INPUT, BLOCK_QUOTE_EVENTS);
    assert_eq_event(LITERAL_CHOMP_INPUT, LITERAL_CHOMP_EVENTS);
    assert_eq_event(LITERAL3_INPUT, LITERAL3_EVENTS);
    assert_eq_event(LIT_STR2_INPUT, LIT_STR2_EVENTS);
    assert_eq_event(MULTILINE_PLAIN_INPUT, MULTILINE_PLAIN_EVENTS);
}

const X1_Y79Y_000_INPUT: &str = r"
  foo: |
  			
			
";

const X1_Y79Y_000_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
=VAL |
-MAP
-DOC";

const X2_Y79Y_000_INPUT: &str = r"
foo: 
	
bar: 1";

const X2_Y79Y_000_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
=VAL :
=VAL :bar
=VAL :1
-MAP
-DOC";

const X3_Y79Y_000_INPUT: &str = r"
foo: |
	
bar: 1";

const X3_Y79Y_000_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
ERR
=VAL |
=VAL :bar
=VAL :1
-MAP
-DOC";

const X4_Y79Y_000_INPUT: &str = r"
  foo: |
  x";

const X4_Y79Y_000_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
ERR
=VAL |
ERR
=VAL :x
-MAP
-DOC";

#[test]
fn block_literal_indents() {
    assert_eq_event(X1_Y79Y_000_INPUT, X1_Y79Y_000_EVENTS);
    assert_eq_event(X2_Y79Y_000_INPUT, X2_Y79Y_000_EVENTS);
    assert_eq_event(X3_Y79Y_000_INPUT, X3_Y79Y_000_EVENTS);
    assert_eq_event(X4_Y79Y_000_INPUT, X4_Y79Y_000_EVENTS);
}

const LITERAL_ERR_INPUT: &str = r"
--- |0";

const LITERAL_ERR2_INPUT: &str = r"
--- |+10";

const SIMPLE_FOLDED_ERR_EVENTS: &str = r"
+DOC ---
ERR
=VAL |
-DOC";

#[test]
fn block_literal_err() {
    assert_eq_event(LITERAL_ERR_INPUT, SIMPLE_FOLDED_ERR_EVENTS);
    assert_eq_event(LITERAL_ERR2_INPUT, SIMPLE_FOLDED_ERR_EVENTS);
}

const X1_6VJK_INPUT: &str = r"
|
 XX
 X1

   Y1
   Y2

 Z3
";

const X1_6VJK_EVENTS: &str = r"
+DOC
=VAL |XX\nX1\n\n  Y1\n  Y2\n\nZ3\n
-DOC";

const X2_6VJK_INPUT: &str = r"
>
 X1

   Y1
   Y2

 Z3
";

const X2_6VJK_EVENTS: &str = r"
+DOC
=VAL >X1\n\n  Y1\n  Y2\n\nZ3\n
-DOC";

const X1_7T8X_INPUT: &str = r"
>
 line

 # Comment
";

const X1_7T8X_EVENTS: &str = r"
+DOC
=VAL >line\n# Comment\n
-DOC";

const X2_7T8X_INPUT: &str = r"
>
 line

# Comment
";

const X2_7T8X_EVENTS: &str = r"
+DOC
=VAL >line\n
-DOC";

const X1_JEF9_INPUT: &str = r"
- |+
   ";

const X1_JEF9_EVENTS: &str = r"
+DOC
+SEQ
=VAL |\n
-SEQ
-DOC";

const X1_F6MC_INPUT: &str = r"
a: >2
   more
  normal
";

const X2_F6MC_INPUT: &str = r"
b: >2


   more
  normal
";

const X1_F6MC_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
=VAL > more\nnormal\n
-MAP
-DOC";

const X2_F6MC_EVENTS: &str = r"
+DOC
+MAP
=VAL :b
=VAL >\n\n more\nnormal\n
-MAP
-DOC";

#[test]
fn block_indent_lit_fold() {
    assert_eq_event(X2_7T8X_INPUT, X2_7T8X_EVENTS);
    assert_eq_event(X1_7T8X_INPUT, X1_7T8X_EVENTS);
    assert_eq_event(X1_6VJK_INPUT, X1_6VJK_EVENTS);
    assert_eq_event(X2_6VJK_INPUT, X2_6VJK_EVENTS);
    assert_eq_event(X1_JEF9_INPUT, X1_JEF9_EVENTS);
    assert_eq_event(X1_F6MC_INPUT, X1_F6MC_EVENTS);
    assert_eq_event(X2_F6MC_INPUT, X2_F6MC_EVENTS);
}

const PLAIN_MULTI_INPUT: &str = r"
1st line

 2nd non
    3rd non
";

const PLAIN_MULTI_EVENTS: &str = r"
+DOC
=VAL :1st line\n2nd non 3rd non
-DOC";

const X_8XDJ_INPUT: &str = r"
key: word1
#  xxx
  word2
";

const X_8XDJ_EVENTS: &str = r"
+DOC
+MAP
=VAL :key
=VAL :word1
ERR
ERR
=VAL :word2
-MAP
-DOC";

#[test]
fn block_plain_multiline() {
    assert_eq_event(PLAIN_MULTI_INPUT, PLAIN_MULTI_EVENTS);
    assert_eq_event(X_8XDJ_INPUT, X_8XDJ_EVENTS);
}

const MAP_SIMPLE_INPUT: &str = r"
a: b
";

const MAP_SIMPLE2_INPUT: &str = r"
a: 
  b
";

const MAP_SIMPLE_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
=VAL :b
-MAP
-DOC";

const X1_1_SYW4_INPUT: &str = r"
hr:  65    # Home runs
avg: 0.278 # Batting average
";

const X1_2_SYW4_INPUT: &str = r"
hr:  65    
avg: 0.278 
";

const X1_SYW4_EVENTS: &str = r"
+DOC
+MAP
=VAL :hr
=VAL :65
=VAL :avg
=VAL :0.278
-MAP
-DOC";

#[test]
fn block_map() {
    assert_eq_event(X1_1_SYW4_INPUT, X1_SYW4_EVENTS);
    assert_eq_event(X1_2_SYW4_INPUT, X1_SYW4_EVENTS);

    assert_eq_event(MAP_SIMPLE_INPUT, MAP_SIMPLE_EVENTS);
    assert_eq_event(MAP_SIMPLE2_INPUT, MAP_SIMPLE_EVENTS);
}

const DQUOTE_MAP_INPUT: &str = r#"
quote: "a\/b"
"#;

const DQUOTE_MAP_EVENTS: &str = r#"
+DOC
+MAP
=VAL :quote
=VAL "a/b
-MAP
-DOC"#;

const DQUOTE_MUL_INPUT: &str = r#"
quoted: "multi
  line"
 "#;

const DQUOTE_MUL_EVENTS: &str = r#"
+DOC
+MAP
=VAL :quoted
=VAL "multi line
-MAP
-DOC"#;

#[test]
fn block_quote_map() {
    assert_eq_event(DQUOTE_MAP_INPUT, DQUOTE_MAP_EVENTS);
    assert_eq_event(DQUOTE_MUL_INPUT, DQUOTE_MUL_EVENTS);
}

const EMPTY_MAP_INPUT: &str = r"
:";

const EMPTY_MAP_EVENTS: &str = r"
+DOC
+MAP
=VAL :
=VAL :
-MAP
-DOC";

const EMPTY_MAP2_INPUT: &str = r"
:
 a";

const EMPTY_MAP2_1_INPUT: &str = r"
: a";

const EMPTY_MAP2_EVENTS: &str = r#"
+DOC
+MAP
=VAL :
=VAL :a
-MAP
-DOC"#;

const MIX_EMPTY_MAP_INPUT: &str = r"
 a:
   x
   u
 c :
";

const MIX_EMPTY_MAP_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
=VAL :x u
=VAL :c
=VAL :
-MAP
-DOC";

const MAP2_INPUT: &str = r"
:
a: b
: c
d:
";

const MAP2_EVENTS: &str = r"
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
-DOC";

const NESTED_EMPTY_INPUT: &str = r"
a :
 b: 
  c:
d:";

const NESTED_EMPTY_EVENTS: &str = r"
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
-DOC";

const MULTI_EMPTY_INPUT: &str = r"
: 
  :";

const MULTI_EMPTY_EVENTS: &str = r"
+DOC
+MAP
=VAL :
+MAP
=VAL :
=VAL :
-MAP
-MAP
-DOC";

#[test]
fn block_empty_map() {
    assert_eq_event(NESTED_EMPTY_INPUT, NESTED_EMPTY_EVENTS);

    assert_eq_event(EMPTY_MAP_INPUT, EMPTY_MAP_EVENTS);
    assert_eq_event(MULTI_EMPTY_INPUT, MULTI_EMPTY_EVENTS);

    assert_eq_event(EMPTY_MAP2_INPUT, EMPTY_MAP2_EVENTS);
    assert_eq_event(EMPTY_MAP2_1_INPUT, EMPTY_MAP2_EVENTS);
    assert_eq_event(MIX_EMPTY_MAP_INPUT, MIX_EMPTY_MAP_EVENTS);
    assert_eq_event(MAP2_INPUT, MAP2_EVENTS);
}

const MULTILINE_COMMENT1_INPUT: &str = r"
  mul: 
    abc  # a comment
";

const MULTILINE_COMMENT1_2_INPUT: &str = r"
  mul  : 
    abc  # a comment
";

const MULTILINE_COMMENT1_EVENTS: &str = r"
+DOC
+MAP
=VAL :mul
=VAL :abc
-MAP
-DOC";

const MULTILINE_COMMENT2_INPUT: &str = r"
  multi:
    ab  # a comment
    xyz  # a commeent
";

const MULTILINE_COMMENT2_EVENTS: &str = r"
+DOC
+MAP
=VAL :multi
=VAL :ab
ERR
ERR
=VAL :xyz
-MAP
-DOC";

const MULTILINE_COMMENT3_INPUT: &str = r"
  multi:
    ab  
    xyz  # a commeent
";

const MULTILINE_COMMENT3_EVENTS: &str = r"
+DOC
+MAP
=VAL :multi
=VAL :ab xyz
-MAP
-DOC";

#[test]
fn block_multiline_comment() {
    assert_eq_event(MULTILINE_COMMENT1_INPUT, MULTILINE_COMMENT1_EVENTS);
    assert_eq_event(MULTILINE_COMMENT1_2_INPUT, MULTILINE_COMMENT1_EVENTS);
    assert_eq_event(MULTILINE_COMMENT2_INPUT, MULTILINE_COMMENT2_EVENTS);
    assert_eq_event(MULTILINE_COMMENT3_INPUT, MULTILINE_COMMENT3_EVENTS);
}

const EXP_MAP_INPUT: &str = r"
  ? test
  : value
";

const EXP_BLOCK_MAP_MIX_INPUT: &str = r"
  ? test
  : value
  tx: x
";

const EXP_MAP_EVENTS: &str = r"
+DOC
+MAP
=VAL :test
=VAL :value
-MAP
-DOC";

const EXP_BLOCK_MAP_MIX_EVENTS: &str = r"
+DOC
+MAP
=VAL :test
=VAL :value
=VAL :tx
=VAL :x
-MAP
-DOC";

const EXP_MAP_FOLD_INPUT: &str = r"
 ? >
   test
 : x
";

const EXP_MAP_FOLD_EVENTS: &str = r"
+DOC
+MAP
=VAL >test\n
=VAL :x
-MAP
-DOC";

const EXP_MAP_COMP_INPUT: &str = r"
---
?
- a
- b
:
- c";

const EXP_MAP_COMP_EVENTS: &str = r"
+DOC ---
+MAP
+SEQ
=VAL :a
=VAL :b
-SEQ
+SEQ
=VAL :c
-SEQ
-MAP
-DOC";

const X_7W2P_INPUT: &str = r"
? a
? b
c:
";

const X_7W2P_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
=VAL :
=VAL :b
=VAL :
=VAL :c
=VAL :
-MAP
-DOC";

const X_5WE3_INPUT: &str = r"

? explicit key # Empty value
? |
  block key";

const X_5WE3_EVENTS: &str = r"
+DOC
+MAP
=VAL :explicit key
=VAL :
=VAL |block key\n
=VAL :
-MAP
-DOC";

const X1_A2M4_INPUT: &str = r"
? a
: -	b";

const X1_A2M4_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
+SEQ
=VAL :b
-SEQ
-MAP
-DOC";

#[test]
fn block_exp_map() {
    assert_eq_event(X1_A2M4_INPUT, X1_A2M4_EVENTS);
    assert_eq_event(X_7W2P_INPUT, X_7W2P_EVENTS);
    assert_eq_event(EXP_MAP_FOLD_INPUT, EXP_MAP_FOLD_EVENTS);
    assert_eq_event(X_5WE3_INPUT, X_5WE3_EVENTS);
    assert_eq_event(EXP_MAP_INPUT, EXP_MAP_EVENTS);
    assert_eq_event(EXP_BLOCK_MAP_MIX_INPUT, EXP_BLOCK_MAP_MIX_EVENTS);
    assert_eq_event(EXP_MAP_COMP_INPUT, EXP_MAP_COMP_EVENTS);
}

const EXP_MAP_EMPTY_INPUT: &str = r"
  # Sets are represented as a
# Mapping where each key is
  # associated with a null value
---
? a
? b 
? c
";

const EXP_MAP_EMPTY_INPUT_EVENTS: &str = r"
+DOC ---
+MAP
=VAL :a
=VAL :
=VAL :b
=VAL :
=VAL :c
=VAL :
-MAP
-DOC";

const EXP_MAP_FAKE_EMPTY_INPUT: &str = r"
  ? x
   ? x
";

const EXP_MAP_FAKE_EMPTY_EVENTS: &str = r"
+DOC
+MAP
=VAL :x ? x
=VAL :
-MAP
-DOC";

#[test]
fn block_empty_node_exp_map() {
    assert_eq_event(EXP_MAP_EMPTY_INPUT, EXP_MAP_EMPTY_INPUT_EVENTS);
    assert_eq_event(EXP_MAP_FAKE_EMPTY_INPUT, EXP_MAP_FAKE_EMPTY_EVENTS);
}
const EMPTY_KEY_MAP_INPUT: &str = r"
: a
: b
";

const EMPTY_KEY_MAP_EVENTS: &str = r"
+DOC
+MAP
=VAL :
=VAL :a
=VAL :
=VAL :b
-MAP
-DOC";

#[test]
fn block_empty_node_map() {
    assert_eq_event(EMPTY_KEY_MAP_INPUT, EMPTY_KEY_MAP_EVENTS);
}

const EXP_BLOCK_MAP_ERR1: &str = r"
   ? test
  : value
";

const EXP_BLOCK_MAP_ERR1_EVENTS: &str = r"
+DOC
+MAP
=VAL :test
ERR
=VAL :value
-MAP
-DOC";

const EXP_BLOCK_MAP_ERR2: &str = r"
 ? test
  : value
";

const EXP_BLOCK_MAP_ERR2_EVENTS: &str = r"
+DOC
+MAP
=VAL :test
ERR
=VAL :value
-MAP
-DOC";

#[test]
fn block_exp_map_err() {
    assert_eq_event(EXP_BLOCK_MAP_ERR1, EXP_BLOCK_MAP_ERR1_EVENTS);
    assert_eq_event(EXP_BLOCK_MAP_ERR2, EXP_BLOCK_MAP_ERR2_EVENTS);
}

const INLINE_ERR_INPUT: &str = r"
 a: b:
";
const INLINE_ERR_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
ERR
+MAP
=VAL :b
=VAL :
-MAP
-MAP
-DOC";

#[test]
fn block_map_inline_err() {
    assert_eq_event(INLINE_ERR_INPUT, INLINE_ERR_EVENTS);
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

const ERR_INVALID_KEY3_INPUT: &str = r"
top1:
  key1: val1
top2
";

const ERR_INVALID_KEY3_EVENTS: &str = r"
+DOC
+MAP
=VAL :top1
+MAP
=VAL :key1
=VAL :val1
-MAP
ERR
=VAL :top2
-MAP
-DOC";

const ERR_TRAIL_INPUT: &str = r#"
'key': "quote" trail
"#;

const ERR_TRAIL_EVENTS: &str = r#"
+DOC
+MAP
=VAL 'key
=VAL "quote
ERR
=VAL :trail
-MAP
-DOC"#;

#[test]
fn block_map_err() {
    assert_eq_event(ERR_MULTILINE_KEY_INPUT, ERR_MULTILINE_KEY_EVENTS);
    assert_eq_event(ERR_TRAIL_INPUT, ERR_TRAIL_EVENTS);
    assert_eq_event(ERR_INVALID_KEY1_INPUT, ERR_INVALID_KEY1_EVENTS);
    assert_eq_event(ERR_INVALID_KEY2_INPUT, ERR_INVALID_KEY2_EVENTS);
    assert_eq_event(ERR_INVALID_KEY3_INPUT, ERR_INVALID_KEY3_EVENTS);
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

const COMPLEX_NESTED_INPUT: &str = r"
not:
  two: [
    nest
   ]
  ";

const COMPLEX_NESTED_EVENTS: &str = r"
+DOC
+MAP
=VAL :not
+MAP
=VAL :two
+SEQ []
=VAL :nest
-SEQ
-MAP
-MAP
-DOC";

const NESTED_INPUT: &str = r"
---
hr: # 1998 hr ranking
  - Mark McGwire
  - Sammy Sosa
";

const NESTED_EVENTS: &str = r"
+DOC ---
+MAP
=VAL :hr
+SEQ
=VAL :Mark McGwire
=VAL :Sammy Sosa
-SEQ
-MAP
-DOC";

const X1_9C9N_INPUT: &str = r"
flow: [a,
b,
 c]";

const X1_9C9N_EVENTS: &str = r"
+DOC
+MAP
=VAL :flow
+SEQ []
=VAL :a
ERR
=VAL :b
ERR
=VAL :c
-SEQ
-MAP
-DOC";

const MAP_AND_COMMENT_INPUT: &str = r"
hr:
  - aaa
  # comment
  - &xx bbb
";

const MAP_AND_COMMENT_EVENTS: &str = r"
+DOC
+MAP
=VAL :hr
+SEQ
=VAL :aaa
=VAL &xx :bbb
-SEQ
-MAP
-DOC";

#[test]
fn block_map_complex() {
    assert_eq_event(COMPLEX_NESTED_INPUT, COMPLEX_NESTED_EVENTS);
    assert_eq_event(NESTED_INPUT, NESTED_EVENTS);
    assert_eq_event(COMPLEX_KEYS_INPUT, COMPLEX_KEYS_EVENTS);
    assert_eq_event(X1_9C9N_INPUT, X1_9C9N_EVENTS);
    assert_eq_event(MAP_AND_COMMENT_INPUT, MAP_AND_COMMENT_EVENTS);
}

const X_7ZZ5_INPUT: &str = r"
key2: {}
";

const X_7ZZ5_EVENTS: &str = r"
+DOC
+MAP
=VAL :key2
+MAP {}
-MAP
-MAP
-DOC";

const X1_87E4_INPUT: &str = r"
'implicit block key' : [
  'implicit flow key' : value,
 ]
";

const X2_87E4_INPUT: &str = r"
'implicit block key' : [
  'implicit flow key' : value
 ]
";

const X_87E4_EVENTS: &str = r"
+DOC
+MAP
=VAL 'implicit block key
+SEQ []
+MAP {}
=VAL 'implicit flow key
=VAL :value
-MAP
-SEQ
-MAP
-DOC";

const X_8KB6_INPUT: &str = r"
- { multi
  line, a: b}
";

const X_8KB6_EVENTS: &str = r"
+DOC
+SEQ
+MAP {}
=VAL :multi line
=VAL :
=VAL :a
=VAL :b
-MAP
-SEQ
-DOC";

const X1_6HB6_INPUT: &str = r"
Not indented:
 Flow style: [       # comment
   By two,           # comment
  Also by two,       # comment
  	Still by two     # comment
    ]      # comment
";

const X1_6HB6_EVENTS: &str = r"
+DOC
+MAP
=VAL :Not indented
+MAP
=VAL :Flow style
+SEQ []
=VAL :By two
=VAL :Also by two
=VAL :Still by two
-SEQ
-MAP
-MAP
-DOC";

const X1_4AW9_INPUT: &str = r"
- aaa: |2
    xxx";

const X1_4AW9_EVENTS: &str = r"
+DOC
+SEQ
+MAP
=VAL :aaa
=VAL |xxx\n
-MAP
-SEQ
-DOC";

#[test]
fn block_flow_mix() {
    assert_eq_event(X1_4AW9_INPUT, X1_4AW9_EVENTS);
    assert_eq_event(X1_87E4_INPUT, X_87E4_EVENTS);
    assert_eq_event(X1_6HB6_INPUT, X1_6HB6_EVENTS);
    assert_eq_event(X_7ZZ5_INPUT, X_7ZZ5_EVENTS);
    assert_eq_event(X2_87E4_INPUT, X_87E4_EVENTS);
    assert_eq_event(X_8KB6_INPUT, X_8KB6_EVENTS);
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
fn block_map_scalar_and_ws() {
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
fn block_nested_maps() {
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

const ALIAS_N_MAPS2_INPUT: &str = r"
top3: &node3 
  *alias1 : scalar3
 ";

const ALIAS_N_MAPS2_EVENTS: &str = r"
+DOC
+MAP
=VAL :top3
+MAP &node3
=ALI *alias1
=VAL :scalar3
-MAP
-MAP
-DOC";

const ALIAS_N_COMP_MAP_INPUT: &str = r"
&map
&key [ &item a, b]: value
";

const ALIAS_N_COMP_MAP_EVENTS: &str = r"
+DOC
+MAP &map
+SEQ [] &key
=VAL &item :a
=VAL :b
-SEQ
=VAL :value
-MAP
-DOC";

#[test]
fn block_map_anchor_alias() {
    assert_eq_event(ALIAS_N_MAPS_INPUT, ALIAS_N_MAPS_EVENTS);
    assert_eq_event(ALIAS_N_MAPS2_INPUT, ALIAS_N_MAPS2_EVENTS);
    assert_eq_event(ALIAS_N_COMP_MAP_INPUT, ALIAS_N_COMP_MAP_EVENTS);
}

const ALIAS_N_SEQ1_INPUT: &str = r"
&seq
 - a
 ";

const ALIAS_N_SEQ1_EVENTS: &str = r"
+DOC
+SEQ &seq
=VAL :a
-SEQ
-DOC";

const ALIAS_N_SEQ2_INPUT: &str = r"
 &seq  - a
  ";

const ALIAS_N_SEQ2_EVENTS: &str = r"
+DOC
ERR
+SEQ &seq
=VAL :a
-SEQ
-DOC";

const ALIAS_N_SEQ3_INPUT: &str = r"
  - &node a
  ";

const ALIAS_N_SEQ3_EVENTS: &str = r"
+DOC
+SEQ
=VAL &node :a
-SEQ
-DOC";

const X1_G9HC_INPUT: &str = r"
 seq:
 &anchor
 - a";

const X1_G9HC_EVENTS: &str = r"
+DOC
+MAP
=VAL :seq
ERR
+SEQ &anchor
=VAL :a
-SEQ
-MAP
-DOC";

const X2_1_G9HC_INPUT: &str = r"
seq:
 &anchor
 - a";

const X2_2_G9HC_INPUT: &str = r"
 seq:
  	&anchor
  - a";

const X2_G9HC_EVENTS: &str = r"
+DOC
+MAP
=VAL :seq
+SEQ &anchor
=VAL :a
-SEQ
-MAP
-DOC";

#[test]
fn block_seq_anchor_alias() {
    assert_eq_event(X1_G9HC_INPUT, X1_G9HC_EVENTS);
    assert_eq_event(X2_1_G9HC_INPUT, X2_G9HC_EVENTS);
    assert_eq_event(X2_2_G9HC_INPUT, X2_G9HC_EVENTS);

    assert_eq_event(ALIAS_N_SEQ1_INPUT, ALIAS_N_SEQ1_EVENTS);
    assert_eq_event(ALIAS_N_SEQ2_INPUT, ALIAS_N_SEQ2_EVENTS);
    assert_eq_event(ALIAS_N_SEQ3_INPUT, ALIAS_N_SEQ3_EVENTS);
}

const X1_57H4_INPUT: &str = r"
  sequence: !!seq
  - a
  - !!str
    - b
  mapping: !!map
    foo: bar
";

const X1_57H4_EVENTS: &str = r"
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
-DOC";

const X2_57H4_INPUT: &str = r"
  - !!str
    - b";

const X2_57H4_EVENTS: &str = r"
+DOC
+SEQ
+SEQ <tag:yaml.org,2002:str>
=VAL :b
-SEQ
-SEQ
-DOC";

const TAG_DEF_INPUT: &str = r"
 ! test
";

const TAG_DEF_EVENTS: &str = r"
+DOC
=VAL <!> :test
-DOC";

const EXP_TAG_INPUT: &str = r"
!<tag:yaml.org,2002:str> foo :
  !<!bar> baz";

const EXP_TAG_EVENTS: &str = r"
+DOC
+MAP
=VAL <tag:yaml.org,2002:str> :foo
=VAL <!bar> :baz
-MAP
-DOC";

#[test]
fn block_col_tags() {
    assert_eq_event(X1_57H4_INPUT, X1_57H4_EVENTS);
    assert_eq_event(X2_57H4_INPUT, X2_57H4_EVENTS);
    assert_eq_event(TAG_DEF_INPUT, TAG_DEF_EVENTS);
    assert_eq_event(EXP_TAG_INPUT, EXP_TAG_EVENTS);
}

const ANCHOR_COLON_INPUT: &str = r"
&node3:  key : scalar3
*node3: : x";

const ANCHOR_COLON_EVENTS: &str = r"
+DOC
+MAP
=VAL &node3: :key
=VAL :scalar3
=ALI *node3:
=VAL :x
-MAP
-DOC";

const ANCHOR_MULTI_INPUT: &str = r"
top2: &node2
  &v2 val: x";

const ANCHOR_MULTI_EVENTS: &str = r"
+DOC
+MAP
=VAL :top2
+MAP &node2
=VAL &v2 :val
=VAL :x
-MAP
-MAP
-DOC";

const ANCHOR_ERR_INPUT: &str = r"
top2: &node2
  &v2 val";

const ANCHOR_ERR_EVENTS: &str = r"
+DOC
+MAP
=VAL :top2
ERR
=VAL &v2 :val
-MAP
-DOC";

const X1_735Y_INPUT: &str = r"
- >
 Block scalar
- !!map # Block collection
  foo : bar
";

const X1_735Y_EVENTS: &str = r"
+DOC
+SEQ
=VAL >Block scalar\n
+MAP <tag:yaml.org,2002:map>
=VAL :foo
=VAL :bar
-MAP
-SEQ
-DOC";

#[test]
fn block_anchor() {
    assert_eq_event(X1_735Y_INPUT, X1_735Y_EVENTS);
    assert_eq_event(ANCHOR_COLON_INPUT, ANCHOR_COLON_EVENTS);
    assert_eq_event(ANCHOR_MULTI_INPUT, ANCHOR_MULTI_EVENTS);
    assert_eq_event(ANCHOR_ERR_INPUT, ANCHOR_ERR_EVENTS);
}

const MIX_BLOCK_INPUT: &str = r"
-
  key: x
  val: 8
- 
  val: y
";

const MIX_BLOCK_EVENTS: &str = r"
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
-DOC";

const MIX2_BLOCK_INPUT: &str = r"
  sequence:
  - a
  mapping:
   foo: bar
 ";

const MIX2_BLOCK_EVENTS: &str = r"
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
-DOC";

#[test]
fn block_mix_seq() {
    assert_eq_event(MIX_BLOCK_INPUT, MIX_BLOCK_EVENTS);
    assert_eq_event(MIX2_BLOCK_INPUT, MIX2_BLOCK_EVENTS);
}

const TAG1_INPUT: &str = r"
  !!str a";

const TAG1_EVENTS: &str = r"
+DOC
=VAL <tag:yaml.org,2002:str> :a
-DOC";

const COMPLEX_TAG2_INPUT: &str = r"
- !!str c
--- !!str
d
e";

const COMPLEX_TAG2_EVENTS: &str = r"
+DOC
+SEQ
=VAL <tag:yaml.org,2002:str> :c
-SEQ
-DOC
+DOC ---
=VAL <tag:yaml.org,2002:str> :d e
-DOC";

const X_74H7_INPUT: &str = r"
!!str a: b
c: !!int 42
!!str 23: !!bool false";

const X_74H7_EVENTS: &str = r"
+DOC
+MAP
=VAL <tag:yaml.org,2002:str> :a
=VAL :b
=VAL :c
=VAL <tag:yaml.org,2002:int> :42
=VAL <tag:yaml.org,2002:str> :23
=VAL <tag:yaml.org,2002:bool> :false
-MAP
-DOC";

#[test]
fn block_tag() {
    assert_eq_event(TAG1_INPUT, TAG1_EVENTS);
    assert_eq_event(COMPLEX_TAG2_INPUT, COMPLEX_TAG2_EVENTS);
    assert_eq_event(X_74H7_INPUT, X_74H7_EVENTS);
}

const MULTI_LINE_INPUT: &str = r"
x: a
 b

 c";

const MULTI_LINE_EVENTS: &str = r"
+DOC
+MAP
=VAL :x
=VAL :a b\nc
-MAP
-DOC";

const MULTI_LINE_SEQ_INPUT: &str = r"
- a 
 b

 c";

const MULTI_LINE_SEQ_EVENTS: &str = r"
+DOC
+SEQ
=VAL :a b\nc
-SEQ
-DOC";

const X_BF9H_INPUT: &str = r"
plain:  a
        b # comment
        c
";

const X_BF9H_EVENTS: &str = r"
+DOC
+MAP
=VAL :plain
=VAL :a b
ERR
ERR
=VAL :c
-MAP
-DOC";

const X_BS4K_INPUT: &str = r"
line1 # comment
line2";

const X_BS4K_EVENTS: &str = r"
+DOC
=VAL :line1
-DOC
ERR";

#[test]
fn block_multi_line() {
    assert_eq_event(MULTI_LINE_INPUT, MULTI_LINE_EVENTS);
    assert_eq_event(MULTI_LINE_SEQ_INPUT, MULTI_LINE_SEQ_EVENTS);
    assert_eq_event(X_BF9H_INPUT, X_BF9H_EVENTS);
    assert_eq_event(X_BS4K_INPUT, X_BS4K_EVENTS);
}

const SEQ_SAME_LINE_INPUT: &str = r"
  key: - a
";

const SEQ_SAME_LINE_EVENTS: &str = r"
+DOC
+MAP
=VAL :key
ERR
+SEQ
=VAL :a
-SEQ
-MAP
-DOC";

#[test]
fn block_seq_and_map() {
    assert_eq_event(SEQ_SAME_LINE_INPUT, SEQ_SAME_LINE_EVENTS);
}

const TAG_SHORT_INPUT: &str = "
%TAG !e! tag:example.com,2000:app/
---
- !local foo
- !!str bar
- !e!tag%21 baz";

const TAG_SHORT_EVENTS: &str = "
+DOC ---
+SEQ
=VAL <!local> :foo
=VAL <tag:yaml.org,2002:str> :bar
=VAL <tag:example.com,2000:app/tag!> :baz
-SEQ
-DOC";

#[test]
fn block_tag_short() {
    assert_eq_event(TAG_SHORT_INPUT, TAG_SHORT_EVENTS);
}

const X1_9KAX_INPUT: &str = r"
&a1
!!str
scalar1";

const X2_9KAX_INPUT: &str = r"
!!str
&a1
scalar1";

const X1_9KAX_EVENTS: &str = r"
+DOC
=VAL &a1 <tag:yaml.org,2002:str> :scalar1
-DOC";

const X3_9KAX_INPUT: &str = r"
&a4 !!map
&a5 !!str key5: value4
";

const X3_9KAX_EVENTS: &str = r"
+DOC
+MAP &a4 <tag:yaml.org,2002:map>
=VAL &a5 <tag:yaml.org,2002:str> :key5
=VAL :value4
-MAP
-DOC";

const X4_9KAX_INPUT: &str = r"
a6: 1
&anchor6 b6: 2";

const X4_9KAX_EVENTS: &str = r"
+DOC
+MAP
=VAL :a6
=VAL :1
=VAL &anchor6 :b6
=VAL :2
-MAP
-DOC";

const X5_9KAX_INPUT: &str = r"
!!map
&a8 !!str key8: value7";

const X5_9KAX_EVENTS: &str = r"
+DOC
+MAP <tag:yaml.org,2002:map>
=VAL &a8 <tag:yaml.org,2002:str> :key8
=VAL :value7
-MAP
-DOC";

const X1_6JWB_INPUT: &str = r"
foo:
  - !!map
    k: v
";

const X1_6JWB_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
+SEQ
+MAP <tag:yaml.org,2002:map>
=VAL :k
=VAL :v
-MAP
-SEQ
-MAP
-DOC";

#[test]
fn block_tag_anchor() {
    assert_eq_event(X5_9KAX_INPUT, X5_9KAX_EVENTS);
    assert_eq_event(X4_9KAX_INPUT, X4_9KAX_EVENTS);
    assert_eq_event(X1_9KAX_INPUT, X1_9KAX_EVENTS);
    assert_eq_event(X2_9KAX_INPUT, X1_9KAX_EVENTS);
    assert_eq_event(X3_9KAX_INPUT, X3_9KAX_EVENTS);

    assert_eq_event(X1_6JWB_INPUT, X1_6JWB_EVENTS);
}

const X1_DK95_INPUT: &str = r"
foo: 'bar
	baz'";

const X1_DK95_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
ERR
=VAL 'bar baz
-MAP
-DOC";

const X2_DK95_INPUT: &str = r"
foo: 
	bar: baz";

const X2_DK95_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
ERR
+MAP
=VAL :bar
=VAL :baz
-MAP
-MAP
-DOC";

const X3_DK95_INPUT: &str = r"
foo:
  a: 1
  	b: 2";
const X3_DK95_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
+MAP
=VAL :a
=VAL :1
ERR
+MAP
=VAL :b
=VAL :2
-MAP
=VAL :
-MAP
-MAP
-DOC";

#[test]
fn block_map_tab() {
    assert_eq_event(X1_DK95_INPUT, X1_DK95_EVENTS);
    assert_eq_event(X2_DK95_INPUT, X2_DK95_EVENTS);
    assert_eq_event(X3_DK95_INPUT, X3_DK95_EVENTS);
}

const X1_DMG6_INPUT: &str = r"
key:
  ok: 1
 wrong: 2";

const X1_DMG6_EVENTS: &str = r"
+DOC
+MAP
=VAL :key
+MAP
=VAL :ok
=VAL :1
-MAP
ERR
+MAP
=VAL :wrong
=VAL :2
-MAP
=VAL :
-MAP
-DOC";

const X1_EW3V_INPUT: &str = r"
k1: v1
  key2: v2
";

const X1_EW3V_EVENTS: &str = r"
+DOC
+MAP
=VAL :k1
ERR
=VAL :v1 key2
=VAL :v2
-MAP
-DOC";

const X1_7LBH_INPUT: &str = r#"
a: b
"c
 d": 1"#;

const X1_7LBH_EVENTS: &str = r#"
+DOC
+MAP
=VAL :a
=VAL :b
ERR
=VAL "c d
=VAL :1
-MAP
-DOC"#;

#[test]
fn block_map_err_indent() {
    assert_eq_event(X1_EW3V_INPUT, X1_EW3V_EVENTS);
    assert_eq_event(X1_DMG6_INPUT, X1_DMG6_EVENTS);
    assert_eq_event(X1_7LBH_INPUT, X1_7LBH_EVENTS);
}

const SEQ_EMPTY1_INPUT: &str = r"
- 
";

const SEQ_EMPTY1_EVENTS: &str = r"
+DOC
+SEQ
=VAL :
-SEQ
-DOC";

const SEQ_EMPTY2_INPUT: &str = r"
- 
- ";

const SEQ_EMPTY2_EVENTS: &str = r"
+DOC
+SEQ
=VAL :
=VAL :
-SEQ
-DOC";

#[test]
fn block_seq_empty() {
    assert_eq_event(SEQ_EMPTY1_INPUT, SEQ_EMPTY1_EVENTS);
    assert_eq_event(SEQ_EMPTY2_INPUT, SEQ_EMPTY2_EVENTS);
}

const X1_DC7X_INPUT: &str = r"
seq:	
  - a	";

const X1_DC7X_EVENTS: &str = r"
+DOC
+MAP
=VAL :seq
+SEQ
=VAL :a
-SEQ
-MAP
-DOC";

const X1_Y79Y_001_INPUT: &str = r"
foo: |
 	
bar: 1";

const X1_Y79Y_001_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
=VAL |\t\n
=VAL :bar
=VAL :1
-MAP
-DOC";

const X1_Y79Y_004_INPUT: &str = r"
- -";

const X1_Y79Y_004_EVENTS: &str = r"
+DOC
+SEQ
+SEQ
=VAL :
-SEQ
-SEQ
-DOC";

const X2_Y79Y_004_INPUT: &str = r"
-	-";

const X2_Y79Y_004_EVENTS: &str = r"
+DOC
+SEQ
ERR
+SEQ
=VAL :
-SEQ
-SEQ
-DOC";

const X3_Y79Y_004_INPUT: &str = r"
-	-
 ";

const X1_Y79Y_006_INPUT: &str = r"
?	-";

const X1_Y79Y_006_EVENTS: &str = r"
+DOC
+MAP
ERR
+SEQ
=VAL :
-SEQ
=VAL :
-MAP
-DOC";

const X2_Y79Y_006_INPUT: &str = r"
? -";

const X2_Y79Y_006_EVENTS: &str = r"
+DOC
+MAP
+SEQ
=VAL :
-SEQ
=VAL :
-MAP
-DOC";

const X3_Y79Y_006_INPUT: &str = r"
? -
";

const X3_Y79Y_006_EVENTS: &str = r"
+DOC
+MAP
+SEQ
=VAL :
-SEQ
=VAL :
-MAP
-DOC";

const X1_Y79Y_007_INPUT: &str = r"
? -
:	-
";

const X1_Y79Y_007_EVENTS: &str = r"
+DOC
+MAP
+SEQ
=VAL :
-SEQ
ERR
ERR
+SEQ
=VAL :
-SEQ
-MAP
-DOC";

const X1_Y79Y_009_INPUT: &str = r"
? key:
:	foo:
";

const X1_Y79Y_009_EVENTS: &str = r"
+DOC
+MAP
+MAP
=VAL :key
=VAL :
-MAP
ERR
ERR
+MAP
=VAL :foo
=VAL :
-MAP
-MAP
-DOC";

const X2_Y79Y_009_INPUT: &str = r"
? key:
";

const X2_Y79Y_009_EVENTS: &str = r"
+DOC
+MAP
+MAP
=VAL :key
=VAL :
-MAP
=VAL :
-MAP
-DOC";

const X3_Y79Y_009_INPUT: &str = r"
? key:
: ";

const X3_Y79Y_009_EVENTS: &str = r"
+DOC
+MAP
+MAP
=VAL :key
=VAL :
-MAP
=VAL :
-MAP
-DOC";

#[test]
fn block_tab_indents() {
    assert_eq_event(X1_DC7X_INPUT, X1_DC7X_EVENTS);

    assert_eq_event(X1_Y79Y_001_INPUT, X1_Y79Y_001_EVENTS);

    assert_eq_event(X1_Y79Y_004_INPUT, X1_Y79Y_004_EVENTS);
    assert_eq_event(X2_Y79Y_004_INPUT, X2_Y79Y_004_EVENTS);
    assert_eq_event(X3_Y79Y_004_INPUT, X2_Y79Y_004_EVENTS);

    assert_eq_event(X1_Y79Y_006_INPUT, X1_Y79Y_006_EVENTS);
    assert_eq_event(X2_Y79Y_006_INPUT, X2_Y79Y_006_EVENTS);
    assert_eq_event(X3_Y79Y_006_INPUT, X3_Y79Y_006_EVENTS);

    assert_eq_event(X1_Y79Y_007_INPUT, X1_Y79Y_007_EVENTS);

    assert_eq_event(X1_Y79Y_009_INPUT, X1_Y79Y_009_EVENTS);
    assert_eq_event(X2_Y79Y_009_INPUT, X2_Y79Y_009_EVENTS);
    assert_eq_event(X3_Y79Y_009_INPUT, X3_Y79Y_009_EVENTS);
}

const X1_FH7J_INPUT: &str = r"
- !!str
- !!null : a";

const X1_FH7J_EVENTS: &str = r"
+DOC
+SEQ
=VAL <tag:yaml.org,2002:str> :
+MAP
=VAL <tag:yaml.org,2002:null> :
=VAL :a
-MAP
-SEQ
-DOC";

const X2_FH7J_INPUT: &str = r"
- !!str : !!null";

const X2_FH7J_EVENTS: &str = r"
+DOC
+SEQ
+MAP
=VAL <tag:yaml.org,2002:str> :
=VAL <tag:yaml.org,2002:null> :
-MAP
-SEQ
-DOC";

#[test]
fn block_tags_empty() {
    assert_eq_event(X1_FH7J_INPUT, X1_FH7J_EVENTS);
    assert_eq_event(X2_FH7J_INPUT, X2_FH7J_EVENTS);
}

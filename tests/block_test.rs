use crate::common::assert_eq_event;

const BLOCK1_INPUT: &str = r#"
 - x
 - y
"#;

const BLOCK2_INPUT: &str = r#"
- x
- y
"#;

const BLOCK_EXPECTED: &str = r#"
 +DOC
  +SEQ
   =VAL :x
   =VAL :y
  -SEQ
 -DOC"#;

mod common;

#[test]
pub fn block_seq() {
    assert_eq_event(BLOCK1_INPUT, BLOCK_EXPECTED);
    assert_eq_event(BLOCK2_INPUT, BLOCK_EXPECTED);
}

const BLOCK_ERR_INPUT: &str = r#"
  - x
 - y
"#;

const BLOCK_ERR_EXPECTED: &str = r#"
 +DOC
  +SEQ
   =VAL :x
   ERR
   =VAL :y
  -SEQ
 -DOC"#;

#[test]
pub fn block_seq_err() {
    assert_eq_event(BLOCK_ERR_INPUT, BLOCK_ERR_EXPECTED);
}

const BLOCK_NESTED_SEQ_INPUT: &str = r#"
  - - a
    - b
  - c
"#;

const BLOCK_NESTED_SEQ_EXPECTED: &str = r#"
 +DOC
  +SEQ
   +SEQ
    =VAL :a
    =VAL :b
   -SEQ
   =VAL :c
  -SEQ
 -DOC"#;

const BLOCK_NESTED_SEQ_INPUT2: &str = r#"
  - - a
    - b
    - - c
  - d
"#;

const BLOCK_NESTED_SEQ_EXPECTED2: &str = r#"
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
    assert_eq_event(BLOCK_NESTED_SEQ_INPUT, BLOCK_NESTED_SEQ_EXPECTED);
    assert_eq_event(BLOCK_NESTED_SEQ_INPUT2, BLOCK_NESTED_SEQ_EXPECTED2);
}

const BLOCK_STRINGS_INPUT: &str = r#"
  - |+ # Keep indicator↓
    # keep

  # Trail 
   # comment
  - | # Empty header↓
   literal
   next line
  - > # Indentation indicator↓
    folded
    are continued

  - >1- # Both indicators↓
   strip
    newline
   
"#;

const BLOCK_STRINGS_INPUT2: &str = r#"
  - >1-
   1
    2
   3
   4
   
"#;

const BLOCK_STRINGS_EXPECTED: &str = r#"
 +DOC
  +SEQ
   =VAL |# keep\n\n
   =VAL |literal\nnext line\n
   =VAL >folded are continued\n
   =VAL >strip\n newline
  -SEQ
 -DOC"#;

const BLOCK_STRINGS_EXPECTED2: &str = r#"
 +DOC
  +SEQ
   =VAL >1\n 2\n3 4
  -SEQ
 -DOC"#;

const BLOCK_STRINGS_INPUT3: &str = r#"
strip: |-
  text
clip: |
  text
keep: |
  text"#;

const BLOCK_STRINGS_EXPECTED3: &str = r#"
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

const BLOCK_STRINGS_INPUT4: &str = r#"
plain: 
  spans
  lines

quoted: 
  "text"
"#;

const BLOCK_STRINGS_EXPECTED4: &str = r#"
 +DOC
  +MAP
   =VAL :plain
   =VAL :spans lines
   =VAL :quoted
   =VAL "text
  -MAP
 -DOC"#;

#[test]
pub fn literal_block() {
    assert_eq_event(BLOCK_STRINGS_INPUT, BLOCK_STRINGS_EXPECTED);
    assert_eq_event(BLOCK_STRINGS_INPUT2, BLOCK_STRINGS_EXPECTED2);
    assert_eq_event(BLOCK_STRINGS_INPUT3, BLOCK_STRINGS_EXPECTED3);
    assert_eq_event(BLOCK_STRINGS_INPUT4, BLOCK_STRINGS_EXPECTED4);
}

const BLOCK_PLAIN: &str = r#"
  a
  b
  c
    d
  e
"#;

const BLOCK_PLAIN_EXPECTED: &str = r#"
 +DOC
  =VAL :a b c d e
 -DOC"#;

const BLOCK_PLAIN2: &str = r#"
a
b  
  c
d

e

"#;

const BLOCK_PLAIN2_EXPECTED: &str = r#"
 +DOC
  =VAL :a b c d\ne
 -DOC"#;


#[test]
pub fn plain_block() {
    assert_eq_event(BLOCK_PLAIN, BLOCK_PLAIN_EXPECTED);
    assert_eq_event(BLOCK_PLAIN2, BLOCK_PLAIN2_EXPECTED);
}

const BLOCK_FOLDED: &str = r#"
>
 a
 b
 
 c
 
 
 d
"#;

const BLOCK_FOLDED_EVENTS: &str = r#"
+DOC
 =VAL >a b\nc\n\nd
-DOC"#;


#[test]
pub fn plain_fold() {
  assert_eq_event(BLOCK_FOLDED, BLOCK_FOLDED_EVENTS);

}

const BLOCK_PLAIN_MULTI: &str = r#"
1st line

 2nd non
	3rd non
"#;

const BLOCK_PLAIN_MULTI_EXPECTED: &str = r#"
 +DOC
  =VAL :1st line\n2nd non 3rd non
 -DOC"#;

#[test]
pub fn block_plain_multiline() {
    assert_eq_event(BLOCK_PLAIN_MULTI, BLOCK_PLAIN_MULTI_EXPECTED)
}

const SEQ_PLAIN: &str = r#"
  - x
   - y
"#;

const SEQ_PLAIN2: &str = r#"
- x - y
"#;

const SEQ_PLAIN_EXPECTED: &str = r#"
 +DOC
  +SEQ
   =VAL :x - y
  -SEQ
 -DOC"#;

#[test]
pub fn seq_plain() {
    assert_eq_event(SEQ_PLAIN, SEQ_PLAIN_EXPECTED);
    assert_eq_event(SEQ_PLAIN2, SEQ_PLAIN_EXPECTED);
}

const BLOCK_MAP_INPUT: &str = r#"
  a:
    x
    u
  c :
"#;

const BLOCK_MAP_EXPECTED: &str = r#"
 +DOC
  +MAP
   =VAL :a
   =VAL :x u
   =VAL :c
   =VAL :
  -MAP
 -DOC"#;

const BLOCK_MAP_INPUT2: &str = r#"
:
a: b
: c
d:
"#;

const BLOCK_MAP_EXPECTED2: &str = r#"
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

const BLOCK_MAP_NESTED: &str = r#"
a:
 b:
  c:
d:"#;

const BLOCK_MAP_NESTED_EXPECTED: &str = r#"
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

const BLOCK_MAP_SIMPLE: &str = r#"
a: b
:"#;

const BLOCK_MAP_SIMPLE_EXPECTED: &str = r#"
 +DOC
  +MAP
   =VAL :a
   =VAL :b
   =VAL :
   =VAL :
  -MAP
 -DOC"#;

#[test]
pub fn block_map() {
    assert_eq_event(BLOCK_MAP_SIMPLE, BLOCK_MAP_SIMPLE_EXPECTED);
    assert_eq_event(BLOCK_MAP_INPUT2, BLOCK_MAP_EXPECTED2);
    assert_eq_event(BLOCK_MAP_INPUT, BLOCK_MAP_EXPECTED);
    assert_eq_event(BLOCK_MAP_NESTED, BLOCK_MAP_NESTED_EXPECTED);
}

const EMPTY_MAP_INPUT1: &str = r#"
:"#;

const EMPTY_MAP_EXPECTED1: &str = r#"
 +DOC
  +MAP
   =VAL :
   =VAL :
  -MAP
 -DOC"#;

const EMPTY_MAP_INPUT2: &str = r#"
:
 a"#;

const EMPTY_MAP_INPUT2_1: &str = r#"
: a"#;

const EMPTY_MAP_EXPECTED2: &str = r#"
 +DOC
  +MAP
   =VAL :
   =VAL :a
  -MAP
 -DOC"#;

#[test]
pub fn empty_map() {
    assert_eq_event(EMPTY_MAP_INPUT1, EMPTY_MAP_EXPECTED1);
    assert_eq_event(EMPTY_MAP_INPUT2, EMPTY_MAP_EXPECTED2);
    assert_eq_event(EMPTY_MAP_INPUT2_1, EMPTY_MAP_EXPECTED2);
}

const MULTILINE_COMMENT_BLOCK1: &str = r#"
  mul: 
    abc  # a comment
"#;

const MULTILINE_COMMENT_BLOCK2: &str = r#"
  mul  : 
    abc  # a comment
"#;

const MULTILINE_COMMENT_BLOCK1_EXPECTED: &str = r#"
 +DOC
  +MAP
   =VAL :mul
   =VAL :abc
  -MAP
 -DOC"#;

const MULTILINE_COMMENT_BLOCK3: &str = r#"
  multi:
    ab  # a comment
    xyz  # a commeent
"#;

const MULTILINE_COMMENT_BLOCK3_EXPECTED: &str = r#"
 +DOC
  +MAP
   =VAL :multi
   =VAL :ab
   ERR
   =VAL :xyz
  -MAP
 -DOC"#;

const MULTILINE_COMMENT_BLOCK4: &str = r#"
  multi:
    ab  
    xyz  # a commeent
"#;

const MULTILINE_COMMENT_BLOCK4_EXPECTED: &str = r#"
 +DOC
  +MAP
   =VAL :multi
   =VAL :ab xyz
  -MAP
 -DOC"#;

#[test]
pub fn multiline_block_comment() {
    assert_eq_event(MULTILINE_COMMENT_BLOCK1, MULTILINE_COMMENT_BLOCK1_EXPECTED);
    assert_eq_event(MULTILINE_COMMENT_BLOCK2, MULTILINE_COMMENT_BLOCK1_EXPECTED);
    assert_eq_event(MULTILINE_COMMENT_BLOCK3, MULTILINE_COMMENT_BLOCK3_EXPECTED);
    assert_eq_event(MULTILINE_COMMENT_BLOCK4, MULTILINE_COMMENT_BLOCK4_EXPECTED);
}

const EXPLICIT_BLOCK_MAP1: &str = r#"
  ? test
  : value
"#;

const EXPLICIT_BLOCK_MAP_MIX: &str = r#"
  ? test
  : value
  tx: x
"#;

const EXPLICIT_BLOCK_MAP1_EXPECTED: &str = r#"
 +DOC
  +MAP
   =VAL :test
   =VAL :value
  -MAP
 -DOC"#;

const EXPLICIT_BLOCK_MAP_MIX_EXPECTED: &str = r#"
 +DOC
  +MAP
   =VAL :test
   =VAL :value
   =VAL :tx
   =VAL :x
  -MAP
 -DOC"#;

const EXP_MAP_COMBINATION: &str = r#"
 ? >
   test
 : x
"#;

const EXP_MAP_COMBINATION_EXPECTED: &str = r#"
 +DOC
  +MAP
   =VAL >test\n
   =VAL :x
  -MAP
 -DOC"#;

#[test]
pub fn explicit_block_map() {
    assert_eq_event(EXPLICIT_BLOCK_MAP1, EXPLICIT_BLOCK_MAP1_EXPECTED);
    assert_eq_event(EXP_MAP_COMBINATION, EXP_MAP_COMBINATION_EXPECTED);
    assert_eq_event(EXPLICIT_BLOCK_MAP_MIX, EXPLICIT_BLOCK_MAP_MIX_EXPECTED);
}

const EXPLICIT_BLOCK_MAP_ERR1: &str = r#"
   ? test
  : value
"#;

const EXPLICIT_BLOCK_MAP_ERR1_EXPECTED: &str = r#"
 +DOC
  +MAP
   =VAL :test
   ERR
   =VAL :value
  -MAP
 -DOC"#;

const EXPLICIT_BLOCK_MAP_ERR2: &str = r#"
 ? test
  : value
"#;

const EXPLICIT_BLOCK_MAP_ERR2_EXPECTED: &str = r#"
 +DOC
  +MAP
   =VAL :test
   ERR
   =VAL :value
  -MAP
 -DOC"#;

#[test]
pub fn explicit_block_map_err() {
    assert_eq_event(EXPLICIT_BLOCK_MAP_ERR1, EXPLICIT_BLOCK_MAP_ERR1_EXPECTED);
    assert_eq_event(EXPLICIT_BLOCK_MAP_ERR2, EXPLICIT_BLOCK_MAP_ERR2_EXPECTED);
}

const ERR_MULTILINE_KEY: &str = "
invalid
 key :  x";

const ERR_MULTILINE_KEY_EVENT: &str = "
 +DOC
  ERR
  +MAP
   =VAL :invalid key
   =VAL :x
  -MAP
 -DOC";

const ERR_INVALID_KEY: &str = "
a:
  b
c";

const ERR_INVALID_EVENT: &str = "
 +DOC
  +MAP
   =VAL :a
   =VAL :b
   ERR
   =VAL :c
  -MAP
 -DOC";

#[test]
pub fn block_map_err() {
    assert_eq_event(ERR_MULTILINE_KEY, ERR_MULTILINE_KEY_EVENT);
    assert_eq_event(ERR_INVALID_KEY, ERR_INVALID_EVENT);
}

const COMPLEX_BLOCK_KEY: &str = r##"
a!"#$%&'()*+,-./09:;<=>?@AZ[\]^_`az{|}~: safe
:foo: baz
-foo: boo
"##;

const COMPLEX_BLOCK_EXPECTED: &str = r##"
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
    assert_eq_event(COMPLEX_BLOCK_KEY, COMPLEX_BLOCK_EXPECTED);
}

const MIX_BLOCK: &str = r##"
-
  key: x
  val: 8
- 
  val: y
"##;

const MIX_BLOCK_EXPECTED: &str = r##"
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

#[test]
pub fn test_mix_blocks() {
    assert_eq_event(MIX_BLOCK, MIX_BLOCK_EXPECTED);
}

const TAG1: &str = r#"
 !!str a"#;

const TAG1_EXPECTED: &str = r#"
 +DOC
  =VAL <tag:yaml.org,2002:str> :a
 -DOC"#;

#[test]
fn parse_tag() {
    assert_eq_event(TAG1, TAG1_EXPECTED);
}

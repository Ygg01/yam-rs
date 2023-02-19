use crate::common::assert_eq_event;

const BLOCK1_INPUT: &'static str = r#"
 - x
 - y
"#;

const BLOCK2_INPUT: &'static str = r#"
- x
- y
"#;

const BLOCK_EXPECTED: &'static str = r#"
  +SEQ
    =VAL x
    -SEP-
    =VAL y
  -SEQ"#;

mod common;

#[test]
pub fn block_seq() {
    assert_eq_event(BLOCK1_INPUT, BLOCK_EXPECTED);
    assert_eq_event(BLOCK2_INPUT, BLOCK_EXPECTED);
}

const BLOCK_ERR_INPUT: &'static str = r#"
  - x
 - y
"#;

const BLOCK_ERR_EXPECTED: &'static str = r#"
  +SEQ
    =VAL x
    ERR(ExpectedIndent { actual: 1, expected: 2 })
  -SEQ"#;

#[test]
pub fn block_plain_err() {
    assert_eq_event(BLOCK_ERR_INPUT, BLOCK_ERR_EXPECTED);
}

const BLOCK_NESTED_SEQ_INPUT: &'static str = r#"
  - - a
    - b
"#;

const BLOCK_NESTED_SEQ_EXPECTED: &'static str = r#"
  +SEQ
    +SEQ
      =VAL a
      -SEP-
      =VAL b
    -SEQ
  -SEQ"#;

#[test]
pub fn block_nested() {
    assert_eq_event(BLOCK_NESTED_SEQ_INPUT, BLOCK_NESTED_SEQ_EXPECTED);
}

const BLOCK_STRINGS_INPUT: &'static str = r#"
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

const BLOCK_STRINGS_INPUT2: &'static str = r#"
  - >1-
   1
    2
   3
   4
   
"#;

const BLOCK_STRINGS_EXPECTED: &'static str = r#"
  +SEQ
    =VAL # keep\n\n
    -SEP-
    =VAL literal\nnext line\n
    -SEP-
    =VAL folded are continued\n
    -SEP-
    =VAL strip\n newline
  -SEQ"#;

const BLOCK_STRINGS_EXPECTED2: &'static str = r#"
  +SEQ
    =VAL 1\n 2\n3 4
  -SEQ"#;

#[test]
pub fn literal_block() {
    assert_eq_event(BLOCK_STRINGS_INPUT, BLOCK_STRINGS_EXPECTED);
    assert_eq_event(BLOCK_STRINGS_INPUT2, BLOCK_STRINGS_EXPECTED2);
}
const BLOCK_PLAIN: &'static str = r#"
  a
  b
  c
    d
  e
"#;

const BLOCK_PLAIN_EXPECTED: &'static str = r#"
  =VAL a b c d e"#;

#[test]
pub fn plain_block() {
    assert_eq_event(BLOCK_PLAIN, BLOCK_PLAIN_EXPECTED)
}

const SEQ_PLAIN: &'static str = r#"
  - x
   - y
"#;

const SEQ_PLAIN2: &'static str = r#"
- x - y
"#;

const SEQ_PLAIN_EXPECTED: &'static str = r#"
  +SEQ
    =VAL x - y
  -SEQ"#;

#[test]
pub fn seq_plain() {
    assert_eq_event(SEQ_PLAIN, SEQ_PLAIN_EXPECTED);
    assert_eq_event(SEQ_PLAIN2, SEQ_PLAIN_EXPECTED);
}

const BLOCK_MAP_INPUT: &'static str = r#"
  a:
    x
    u
  c :
"#;

const BLOCK_MAP_EXPECTED: &'static str = r#"
  +MAP
    =VAL a
    -KEY-
    =VAL x u
    -SEP-
    =VAL c
    -KEY-
  -MAP"#;

#[test]
pub fn block_map() {
    assert_eq_event(BLOCK_MAP_INPUT, BLOCK_MAP_EXPECTED);
}

const MULTILINE_COMMENT_BLOCK1: &'static str = r#"
  mul: 
    abc  # a comment
"#;

const MULTILINE_COMMENT_BLOCK2: &'static str = r#"
  mul  : 
    abc  # a comment
"#;

const MULTILINE_COMMENT_BLOCK1_EXPECTED: &'static str = r#"
  +MAP
    =VAL mul
    -KEY-
    =VAL abc
  -MAP"#;

const MULTILINE_COMMENT_BLOCK3: &'static str = r#"
  multi:
    ab  # a comment
    xyz  # a commeent
"#;

const MULTILINE_COMMENT_BLOCK3_EXPECTED: &'static str = r#"
  +MAP
    =VAL multi
    -KEY-
    ERR(UnexpectedComment)
    =VAL ab xyz
  -MAP"#;

const MULTILINE_COMMENT_BLOCK4: &'static str = r#"
  multi:
    ab  
    xyz  # a commeent
"#;

const MULTILINE_COMMENT_BLOCK4_EXPECTED: &'static str = r#"
  +MAP
    =VAL multi
    -KEY-
    =VAL ab xyz
  -MAP"#;

#[test]
pub fn multiline_block_comment() {
    assert_eq_event(MULTILINE_COMMENT_BLOCK1, MULTILINE_COMMENT_BLOCK1_EXPECTED);
    assert_eq_event(MULTILINE_COMMENT_BLOCK2, MULTILINE_COMMENT_BLOCK1_EXPECTED);
    assert_eq_event(MULTILINE_COMMENT_BLOCK3, MULTILINE_COMMENT_BLOCK3_EXPECTED);
    assert_eq_event(MULTILINE_COMMENT_BLOCK4, MULTILINE_COMMENT_BLOCK4_EXPECTED);
}

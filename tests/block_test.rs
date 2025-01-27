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

const BLOCK_ERR_INPUT: &'static str = r#"
  - x
 - y
"#;

const BLOCK_ERR_EXPECTED: &'static str = r#"
  +SEQ
    =VAL x
    ERR(ExpectedIndent { actual: 1, expected: 2 })
  -SEQ"#;

const BLOCK_AS_PLAIN: &'static str = r#"
  - x
   - y
"#;

const BLOCK_AS_PLAIN2: &'static str = r#"
- x - y
"#;

const BLOCK_AS_PLAIN_EXPECTED: &'static str = r#"
  +SEQ
    =VAL x - y
  -SEQ"#;

const BLOCK_NESTED_INPUT: &'static str = r#"
  - - a
    - b
"#;

const BLOCK_NESTED_EXPECTED: &'static str = r#"
  +SEQ
    +SEQ
      =VAL a
      -SEP-
      =VAL b
    -SEQ
  -SEQ"#;

const BLOCK_STRINGS_INPUT: &'static str = r#"
  - | # Empty header↓
   literal
   next line
  - > # Indentation indicator↓
    folded
    are continued
  - |+ # Keep indicator↓
    # keep
   
   # Trail 
    # comment
"#;
// - >1- # Both indicators↓
//  ·strip
// "#;

const BLOCK_STRINGS_EXPECTED: &'static str = r#"
  +SEQ
    =VAL literal\nnext line\n
    -SEP-
    =VAL folded are continued\n
    -SEP-
    =VAL # keep\n\n
  -SEQ"#;

mod common;

#[test]
pub fn block_seq() {
    assert_eq_event(BLOCK1_INPUT, BLOCK_EXPECTED);
    assert_eq_event(BLOCK2_INPUT, BLOCK_EXPECTED);
}

#[test]
pub fn block_plain() {
    assert_eq_event(BLOCK_AS_PLAIN, BLOCK_AS_PLAIN_EXPECTED);
    assert_eq_event(BLOCK_AS_PLAIN2, BLOCK_AS_PLAIN_EXPECTED);
}

#[test]
pub fn block_plain_err() {
    assert_eq_event(BLOCK_ERR_INPUT, BLOCK_ERR_EXPECTED);
}

#[test]
pub fn block_nested() {
    assert_eq_event(BLOCK_NESTED_INPUT, BLOCK_NESTED_EXPECTED);
}

#[test]
pub fn literal_block() {
    assert_eq_event(BLOCK_STRINGS_INPUT, BLOCK_STRINGS_EXPECTED);
}

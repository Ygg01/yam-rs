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

const BLOCK3_INPUT_ERR: &'static str = r#"
  - x
 - y
"#;

const BLOCK_AS_PLAIN: &'static str = r#"
  - x
   - y
"#;



const BLOCK_AS_PLAIN_EXPECTED: &'static str = r#"
  +SEQ
    =VAL x - y
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
}


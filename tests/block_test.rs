const BLOCK1_INPUT: &'static str = r#"
 - x
 - y
"#;

const BLOCK2_INPUT: &'static str = r#"
- x
- y
"#;

const BLOCK_EXPECTED: &'static str = r#"
  +MAP
    =VAL x
    -SEP-
    =VAL y
  -MAP"#;

mod common;

use crate::common::assert_eq_event;

#[test]
pub fn block_seq() {
    assert_eq_event(BLOCK1_INPUT, BLOCK_EXPECTED);
    // assert_eq_event(BLOCK2_INPUT, BLOCK_EXPECTED);
}

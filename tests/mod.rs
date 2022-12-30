extern crate steel_yaml;

#[cfg(test)]
mod tests {
    use std::fmt::{Debug, format, Write};

    use steel_yaml::Scanner;
    use steel_yaml::tokenizer::EventIterator;

    const EMPTY_DOC_INPUT: &'static str = r#"
# test"
  # test
%YAML 1.3 #arst
"#;
    const EMPTY_DOC_EXPECTED: &'static str = r#"
  #YAML 1.3
  ERR(NoDocStartAfterTag)"#;

    const NULL_YAML_INPUT: &'static str = r#"
null
"#;
    const NULL_YAML_EXPECTED: &'static str = r#"
  =VAL null"#;

    const MULTILINE_INPUT: &'static str = r#"
test
xt
"#;
    const MULTILINE_EXPECTED: &'static str = r#"
  =VAL test
  =VAL xt"#;

    const SEQ_FLOW_INPUT: &'static str = r#"
[x, y]
"#;
    const SEQ_FLOW_INPUT2: &'static str = r#"
[x ,y]
"#;
    const SEQ_FLOW_EXPECTED: &'static str = r#"
  +SEQ
    =VAL x
    -SEP-
    =VAL y
  -SEQ"#;

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
      -KEY-
    -MAP
  -SEQ"#;

    const SEQ_EMPTY_MAP: &'static str = r#"
{:}
"#;
    const SEQ_EMPTY_MAP_EXPECTED: &'static str = r#"
  +MAP
    -KEY-
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
    -KEY-
    =VAL y
  -MAP"#;

    const SEQ_COMPLEX_MAP: &'static str = r#"
{[x,y]:a}
"#;

    const SEQ_COMPLEX_MAP_EXPECTED: &'static str = r#"
  +MAP
    +SEQ
      =VAL x
      -SEP-
      =VAL y
    -SEQ
    -KEY-
    =VAL a
  -MAP"#;

    const DQUOTE_STR1: &'static str = r#"
  "double quote"
    "#;

    const DQUOTE_STR2: &'static str = r#"
  "double
  quote"
"#;

    const DQUOTE_STR_EXPECTED: &'static str = r#"
  =VAL "double quote"#;

    const SQUOTE_STR1: &'static str = r#"
  'single quote'
    "#;

    const SQUOTE_STR2: &'static str = r#"
  'single
  quote'"#;

    const SQUOTE_STR_EXPECTED: &'static str = r#"
  =VAL 'single quote"#;

    fn assert_eq_event(input_yaml: &str, expect: &str) {
        let mut event = String::new();
        let scan = EventIterator::new_from_string(input_yaml);
        scan.for_each(|x| event.push_str(x.as_ref()));
        assert_eq!(expect, event);
    }

    #[test]
    fn parse_empty_document() {
        assert_eq_event(EMPTY_DOC_INPUT, EMPTY_DOC_EXPECTED);
    }

    #[test]
    fn parse_flow_scalars() {
        assert_eq_event(NULL_YAML_INPUT, NULL_YAML_EXPECTED);
        assert_eq_event(MULTILINE_INPUT, MULTILINE_EXPECTED);
    }

    #[test]
    fn parse_flow_seq() {
        assert_eq_event(SEQ_FLOW_INPUT, SEQ_FLOW_EXPECTED);
        assert_eq_event(SEQ_FLOW_INPUT2, SEQ_FLOW_EXPECTED);
    }

    #[test]
    fn parse_nested_col() {
        assert_eq_event(SEQ_NESTED_COL1, SEQ_NESTED_COL1_EXPECTED);
        assert_eq_event(SEQ_NESTED_COL2, SEQ_NESTED_COL2_EXPECTED);
    }

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

    #[test]
    fn flow_quote() {
        assert_eq_event(SQUOTE_STR1, SQUOTE_STR_EXPECTED);
        assert_eq_event(SQUOTE_STR2, SQUOTE_STR_EXPECTED);
        assert_eq_event(DQUOTE_STR1, DQUOTE_STR_EXPECTED);
        assert_eq_event(DQUOTE_STR2, DQUOTE_STR_EXPECTED);
    }
}

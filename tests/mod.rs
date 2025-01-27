extern crate steel_yaml;

#[cfg(test)]
mod tests {
    use std::fmt::{Debug, format, Write};

    use steel_yaml::Scanner;

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

    const SEQ_EMPTY_MAP: &'static str = r#"
{:}
"#;
    const SEQ_EMPTY_MAP_EXPECTED: &'static str = r#"
+MAP
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
    const SEQ_X_Y_MAP_EXPECTED: &'static str = r#"
+MAP
=VAL x
=VAL y
-MAP"#;


    fn assert_eq_event(input_yaml: &str, expect: &str) {
        let mut event = String::new();
        let scan = Scanner::from_str_reader(input_yaml);
        scan.for_each(|x| event.push_str(&*format!("\n{:?}", x)));
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
        assert_eq_event(SEQ_NESTED_COL2, SEQ_NESTED_COL1_EXPECTED);
    }

    #[test]
    fn parse_map() {
        assert_eq_event(SEQ_EMPTY_MAP, SEQ_EMPTY_MAP_EXPECTED);
        assert_eq_event(SEQ_XY_MAP1, SEQ_XY_MAP1_EXPECTED);
        assert_eq_event(SEQ_X_Y_MAP1, SEQ_X_Y_MAP_EXPECTED);
        assert_eq_event(SEQ_X_Y_MAP2, SEQ_X_Y_MAP_EXPECTED);
    }
}

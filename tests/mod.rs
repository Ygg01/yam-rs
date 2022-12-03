extern crate steel_yaml;

#[cfg(test)]
mod tests {
    use std::fmt::{format, Debug, Write};

    use steel_yaml::Scanner;

    fn assert_eq_event(input_yaml: &str, expect: &str) {
        let mut event = String::new();
        let scan = Scanner::from_str_reader(input_yaml);
        scan.for_each(|x| event.push_str(&*format!("\n{:?}", x)));
        assert_eq!(expect, event);
    }

    #[test]
    fn parse_empty_document() {
        let input_yaml = r#"
# test"
  # test
%YAML 1.3 #arst
"#;
        let expect = r#"
+STR
#YAML 1.3
ERR
-STR"#;
        assert_eq_event(input_yaml, expect);
    }

    #[test]
    fn parse_flow_scalars() {
        let null_yaml = r#"
null
"#;
        let expected = r#"
+STR
+VAL null
-STR"#;
        assert_eq_event(null_yaml, expected)
    }

    #[test]
    fn parse_flow_scalars_multiline() {
        let multiline = r#"
test
xt
"#;
        let expected = r#"
+STR
+VAL test
+VAL xt
-STR"#;
        assert_eq_event(multiline, expected);

        let multi_newline = r#"
test

xt
"#;
        let expected_multi = r#"
+STR
+VAL test
+VAL
+VAL xt
-STR"#;
    }
}

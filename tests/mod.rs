extern crate steel_yaml;

#[cfg(test)]
mod tests {
    use std::fmt::{format, Debug, Write};

    use steel_yaml::Scanner;

    fn assert_eq_event(input_yaml: &str, expect: &str) {
        let mut event = String::new();
        Scanner::from_str_reader(input_yaml).for_each(|x| event.push_str(&*format!("\n{:?}", x)));
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
+DOC
-DOC
-STR"#;
        assert_eq_event(input_yaml, expect);
    }
}

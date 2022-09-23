extern crate steel_yaml;

#[cfg(test)]
mod tests {
    use std::fmt::{format, Debug, Write};

    use steel_yaml::Scanner;

    #[test]
    fn parse_empty_document() {
        let yaml = r#"
# test"
  # test
"#;
        let expect = r#"
+STR
-STR"#;
        let mut event = String::new();
        Scanner::from_str_reader(yaml).for_each(|x| event.push_str(&*format!("\n{:?}", x)));
        assert_eq!(expect, event);
    }
}

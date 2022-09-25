extern crate steel_yaml;

#[cfg(test)]
mod tests {
    use std::fmt::{Debug, format, Write};

    use steel_yaml::Scanner;

    #[test]
    fn parse_empty_document() {
        let yaml = r#"
# test"
  # test
%YAML 1.3 #arst
"#;
        let expect = r#"
+STR
#TAG 1.3
ERR
+DOC
-DOC
-STR"#;
        let mut event = String::new();
        Scanner::from_str_reader(yaml)
            .for_each(|x| event.push_str(&*format!("\n{:?}", x)));
        assert_eq!(expect, event);
    }
}

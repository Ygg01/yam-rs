extern crate steel_yaml;

#[cfg(test)]
mod tests {
    use std::fmt::{Debug, format, Write};

    use steel_yaml::YamlTokenizer;

    #[test]
    fn parse_document() {
        let yaml = r#"
        "#;
        let expect = r#"
+STR
-STR
"#;
        let tokenizer = YamlTokenizer::default();
        let mut event = String::new();
        tokenizer.from_string(yaml).for_each(
            |x| event.push_str(&*format!("{:?}\n", x))
        );
        assert_eq!(expect, event);
    }
}

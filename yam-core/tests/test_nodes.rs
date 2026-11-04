use yam_core::node::SpannedYaml;
use yam_core::prelude::{Marker, Span, YamlDocAccess, YamlLoader};

#[test]
fn test_spanned_seq() {
    let input = r#"[
    a,
    b
]"#;

    if let Ok(yaml) = YamlLoader::<SpannedYaml>::load_single(input) {
        let first = &yaml[0];
        let second = &yaml[1];
        assert_eq!(first.as_str().unwrap_or(""), "a");
        assert_eq!(second.as_str().unwrap_or(""), "b");
        let span_1 = Span::new(Marker::new(6, 5, 2), Marker::new(7, 6, 2));
        let span_2 = Span::new(Marker::new(13, 5, 3), Marker::new(14, 6, 3));
        assert_eq!(first.span, span_1);
        assert_eq!(second.span, span_2);
    } else {
        panic!("Error parsing Yaml");
    }
}

#[test]
fn test_spanned_map() {
    let input = r#"
a: 'aaa'
b: !!null
"#;

    if let Ok(yaml) = YamlLoader::<SpannedYaml>::load_single(input) {
        let first = &yaml["a"];
        let second = &yaml["b"];
        assert!(first.is_string(), "First value should be a string");
        assert_eq!(first.as_str().unwrap_or(""), "aaa");
        assert!(second.is_null(), "second element is null");
        let span_1 = Span::new(Marker::new(4, 4, 2), Marker::new(9, 9, 2));
        let span_2 = Span::new(Marker::new(20, 1, 4), Marker::new(20, 1, 4));
        assert_eq!(first.span, span_1);
        assert_eq!(second.span, span_2);
    } else {
        panic!("Error parsing Yaml");
    }
}

#[test]
fn test_nested_map() {
    let input = r#"
a: !!vec [2, 3]
"#;

    if let Ok(yaml) = YamlLoader::<SpannedYaml>::load_single(input) {
        let vec = &yaml["a"];
        assert!(vec.is_non_empty_collection(), "Vec should be populated");
        let first = &vec[0];
        assert!(first.is_integer(), "First should be an integer");
        let second = &vec[1];
        assert!(second.is_integer(), "First should be an integer");
        assert_eq!(first.as_i64().unwrap_or(-1), 2);
        assert_eq!(second.as_i64().unwrap_or(-1), 3);
        let span_vec = Span::new(Marker::new(10, 10, 2), Marker::new(15, 15, 2));
        let span_first = Span::new(Marker::new(11, 11, 2), Marker::new(12, 12, 2));
        let span_second = Span::new(Marker::new(14, 14, 2), Marker::new(15, 15, 2));
        assert_eq!(vec.span, span_vec);
        assert_eq!(first.span, span_first);
        assert_eq!(second.span, span_second);
    } else {
        panic!("Error parsing Yaml");
    }
}

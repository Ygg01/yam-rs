use yam_core::node::SpannedYaml;
use yam_core::prelude::{Marker, Span, YamlDocAccess, YamlLoader};

#[test]
fn test_spanned_seq() {
    let input = r#"[
    a,
    b
]"#;

    if let Ok(x) = YamlLoader::<SpannedYaml>::load_single(input) {
        let first = &x[0];
        let second = &x[1];
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

    if let Ok(x) = YamlLoader::<SpannedYaml>::load_single(input) {
        let first = &x["a"];
        let second = &x["b"];
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

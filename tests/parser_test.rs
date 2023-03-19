use std::str::from_utf8;
use steel_yaml::tokenizer::StrReader;
use steel_yaml::treebuild::YamlToken;
use steel_yaml::YamlParser;

#[test]
fn parse_doc() {
    let mut binding: YamlParser<StrReader> = YamlParser::from("null");
    let x: YamlToken<'_, ()> = binding.parse_doc();
    match x {
        YamlToken::Scalar(val, _) => assert_eq!("null", from_utf8(val.as_ref()).unwrap()),
        _ => panic!("unexpected"),
    };
}

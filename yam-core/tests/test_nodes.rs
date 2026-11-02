use yam_core::node::SpannedYaml;
use yam_core::prelude::YamlLoader;

#[test]
fn test_spanned_emitter() {
    let input = "{a, b}";
    let parser = YamlLoader::<SpannedYaml>::load_single(input);
}

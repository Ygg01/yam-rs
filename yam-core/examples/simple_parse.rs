extern crate yam_core;

use yam_core::prelude::*;

fn main() {
    let yaml_str = "{a: b, c: d}";

    if let Ok(yaml) = Yaml::load_single(yaml_str) {
        let b = yaml["a"].as_str().unwrap_or_default();
        assert_eq!(b, "b");
        let d = yaml["c"].as_str().unwrap_or_default();
        assert_eq!(d, "d");

        println!("Parsed YAML: {:?}", yaml);
    }
}

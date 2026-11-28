extern crate yam_core;

use yam_core::prelude::*;

fn main() {
    let yaml_str = "{a: b, c: d}";

    if let Ok(yaml) = Yaml::load_single(yaml_str) {
        let b = &yaml["a"];
        let d = &yaml["c"];
        assert_eq!(b, &"b");
        assert_eq!(d, &"d");
    }
}

extern crate yam_core;

use yam_core::prelude::{Yaml, YamlEmitter};

fn main() {
    let mut string = String::new();
    let mut emitter = YamlEmitter::new(&mut string);

    let yaml = Yaml::from(vec![(vec![1, 2], 1), (vec![1, 3], 2)]);
    if emitter.dump(&yaml).is_ok() {
        println!("{}", string);
    }
}

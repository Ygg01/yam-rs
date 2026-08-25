Yam
-----

This project provides a [saphyr-rs](https://github.com/saphyr-rs/saphyr) derived/forked YAML parser and serde
integration. It implements the [YAML 1.2.2](https://yaml.org/spec/1.2.2/) compliant parser. It passes
the [YAML 1.2.2](https://yaml.org/spec/1.2.2/) test suite.

## Why use it?

If you need a `no_std` zero-copy YAML parser, with few dependencies, spans, and comment support (use `yam-core`). If you
need to serialize structs to and from YAML (use `yam-serde`). If you need a buffered YAML parser (use `yam-std`).

## Why not use it?

- You need to parse YAML in 1.1 mode.
- If you need to serialize complex graphs, aliases and circular references.
- You really need a YAML emitter.
- You need GB/s YAML parsing.
- You need precise whitespace AST handling, for formatting.

# Instalation & Usage

To add yam to your project

```sh 
cargo add yam-core #or `yam-serde`/`yam-std`
```

## Simple parsing

This example creates a simple Yaml string. Parses it and pulls out elements using indexes.

```rust
extern crate yam_core;

use yam_core::prelude::*;

fn main() {
    let yaml_str = "{a: b, c: d}";

    if let Ok(yaml) = Yaml::load_single(yaml_str) {
        let b = yaml["a"].as_str().unwrap_or_default();
        let d = yaml["c"].as_str().unwrap_or_default();
        assert_eq!(d, "d");
        assert_eq!(b, "b");
    }
}
```

Line `let yaml_str = "{a: b, c: d}";` creates a Yaml flow map with two entries.

It's loaded using `Yaml::load_single` and parsed into a `Yaml` struct. `load_single` returns a `Result<Yaml, Error>` while a `load_from`
returns a `Result<Vec<Yaml>, Error>` which is unnecessary for this example.

To access the individual fields, `yaml["a"]` and `yaml["c"]` are used to get the `YamlData`. Since YamlData is an `enum`, the `as_str` part
will return an `Option<&str>`, that needs to be dealt with (via `unwrap_or_default` because we don't care about the error case).

## Simple emitter

Creates simple Yaml model and dump it to a string.

```rust
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
```

In line `let mut emitter = YamlEmitter::new(&mut string);` a string is used as the output for the emitter.

In line `let yaml = Yaml::from(vec![(vec![1,2], 1), (vec![1,3], 2)]);` a vector of tuples is used to create a `Yaml` mapping.

And lines ensure we only print on success.

```
if emitter.dump(&yaml).is_ok() {
    println!("{}", string);
}
```

# Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md)

# Credits

This crate takes a lot of code from saphyr-rs. So many thanks to Ethiarc and ChenYuheng for their work on saphyr-rs.
They are mentioned under [License folder](./.license).

# License

Licensed under either of

- GNU Lesser General Public License, Version 3.0 (LICENSE-LGPL or https://opensource.org/license/lgpl-3-0)
- MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.
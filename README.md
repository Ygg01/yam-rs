Yam
-----
[![codecov](https://codecov.io/gh/Ygg01/yam-rs/graph/badge.svg?token=DL7THX98KW)](https://codecov.io/gh/Ygg01/yam-rs)

This project provides a [saphyr-rs](https://github.com/saphyr-rs/saphyr) derived/forked YAML parser and serde
integration. It implements the [YAML 1.2.2](https://yaml.org/spec/1.2.2/) compliant parser. It passes
the [YAML 1.2.2](https://yaml.org/spec/1.2.2/) test suite.

## Why use it?

- You need a `no_std` zero-copy YAML parser
- With few dependencies (currently `yam-core` only needs `fearless_simd` and its `libm` dependency)
- With spans and comment support (see `comments` feature in `yam-core`)
- If you need to serialize structs to and from YAML (use `yam-serde`) with minimal dependencies (only uses `serde_core` and
  `unicode-segmentation`)

## Why not use it?

- You need to parse YAML in 1.1 mode.
- If you need to serialize complex graphs, aliases and circular references.
- You need precise whitespace AST handling, for formatting.

# Installation & Usage

To add yam to your project

```sh 
cargo add yam-core #or `yam-serde`
```

## Parsing example

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

### Line by line play

Line `let yaml_str = "{a: b, c: d}";` creates a Yaml flow map with two entries as a string.

It's loaded using `Yaml::load_single` and parsed into a `Yaml` struct. `load_single` returns a `Result<Yaml, Error>` while a `load_from`
returns a `Result<Vec<Yaml>, Error>` which is unnecessary for this example.

To access the individual fields, `yaml["a"]` and `yaml["c"]` are used to get the `YamlData`. Since YamlData is an `enum`, the `as_str` part
will return an `Option<&str>`, that needs to be dealt with (via `unwrap_or_default` because we don't care about the error case).

## Emitter example

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

### Line by line play

In line `let mut emitter = YamlEmitter::new(&mut string);` a string is used as the output for the emitter.

In line `let yaml = Yaml::from(vec![(vec![1,2], 1), (vec![1,3], 2)]);` a vector of tuples is used to create a `Yaml` mapping.

And lines ensure we only print on success.

```rust
if emitter.dump(&yaml).is_ok() {
    println!("{}", string);
}
```

## `EventReceiver` example

Simple struct that receives `EventReceiver` events and prints them out.

```rust
extern crate yam_core;

use yam_core::parsing::{Event, EventReceiver, Parser};

struct EventSink<'input> {
    events: Vec<Event<'input>>
}

impl<'input> EventReceiver<'input> for EventSink<'input> {
    fn on_event(&mut self, ev: Event<'input>) {
        self.events.push(ev);
    }
}

fn main() {
    let yaml_str = "{a: b, c: d}";
    let mut sink = EventSink { events: Vec::with_capacity(10) };
    let mut parser = Parser::new_from_str(yaml_str);

    if parser.load(&mut sink, true).is_err() { return; };

    for x in sink.events {
        println!("{:?}", x);
    }
}
```

### Line by line play

Line `struct EventSink<'input>` is a struct that stores events it encountered. We went with borrowed events, although we could have
extracted the events.

Once we have an events sink we need to impleement an `EventReceiver` trait. We could also have used `SpannedEventReceiver` but we didn't
need span information.

Much like in previous examples we take a Yaml string `{a: b, c: d}` and process it in `Parser::new_from_str(yaml_str)`.

To get our events into the `EventSink` we have to call it via `parser.load(&mut sink, true)`. It returns a `Result` so for brevity sake, if
it's `Err` we just return. In a more robust application we could try to recover or do something else.

Last but not least, in

```rust
for x in sink.events {
println!("{:?}", x);
}
```

we print all `Event`s as they come.

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
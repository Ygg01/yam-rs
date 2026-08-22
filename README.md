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
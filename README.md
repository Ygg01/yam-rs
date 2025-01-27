Yam-rs
------
Yam-rs is set of tools for working with YAML files. 

Building from sources
---
1. `git clone https://github.com/Ygg01/yam-rs`
2. `cd yam-rs`
3. `git submodule update --init`
4. `cargo install cargo-nextest`
5. `cargo install cargo-criterion`

Plans
---
It's in development yet, but plans include:
- Emitter
- serde integration
- SIMD?

It contains few crates:
- yam-core - `no_std` + `alloc` lib that contains the core processing logic
- yam - library that relies on `yam-core` to work. It provides IO integration.
- yam-dark-core - experimental `no_std` + `alloc` lib with SIMD acceleration
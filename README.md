Yam-rs
------
Yam-rs is set of tools for working with YAML files. 

It's in development yet, but plans include:
- Emitter
- serde integration
- SIMD?

It contains few crates:
- yam-core - `no_std` + `alloc` lib that contains the core processing logic
- yam - library that relies on `yam-core` to work. It provides IO integration.
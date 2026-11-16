# Serde Yam

Serde is a framework for *ser*ializing and *de*serializing Rust data structures.

## How to use

```shell
cargo add yam-serde
```

Or add

```toml
[dependencies]
yam-serde = "0.1.1"
```

## Why is it not 1.0?

Some design decisions have left in the air on (e.g. do we support `u128` and `i128`?, only one canonical
form or is `SpannedYaml` useful? What data structures to support for (de)serialization?), and I'd love to get feedback
to know how to best inform my future changes.

## What is YAML

YAML stands for YAML Ain't Markup Language™ a recursive abbreviation.

It's a human-friendly data serialization language. It aims to be easy
to read and edit, and it looks like this:

```yaml
%YAML 1.2
---
YAML: YAML Ain't Markup Language™

What It Is:
  YAML is a human-friendly data serialization
  language for all programming languages.

YAML Specifications:
  - YAML 1.2:
      - Revision 1.2.2  # Oct  1, 2021  (About this version)
      - Revision 1.2.1  # Oct  1, 2009
      - Revision 1.2.0  # Jul 21, 2009
  - YAML 1.1          # Jan 18, 2005
  - YAML 1.0          # Jan 29, 2004
```

There are few ways to interact with this library:

- Text data. Pass a string of YAML data that you receive.
- Untyped or loosely typed representation
- Strongly typed Rust data structure

## Working with untyped data

Any valid YAML text can be parsed using `yam-serde::YamLoader`. Representations of
the data vary but can roughly be described as:

```rust
enum YamlData {
    BadValue,
    Scalar(YamlScalar),
    Sequence(Vec<NODE>),
    Mapping(Map<NODE, NODE>),
    Tagged(Cow<Tag>, Box<NODE>),
    Alias,
}

// with Yaml Scalar being one of
enum YamlScalar {
    Null,
    String(String),
    Bool(bool),
    FloatingPoint(f64),
    Integer(i64),
}
```

If you're familiar with JSON, you'll notice it's a much more complicated setup than there
and that's because YAML is a more complicated format (and superset of JSON).

To convert a string to this format we either use `YamlLoader::<Yaml>::load_single`
(there is a convenience method `Yaml::load_single`).

```rust
// Untyped example
fn main() {
    let data = r#"
    name: John Smith
    phones:
        - "+44 1234567"
        - "+32 2345678"
    address:
    "#;

    let person: Person = yam_serde::Yaml::load_single(data).expect("Yaml to be parsed!");

    println!("Please call {} at the number {}", p.name, p.phones[0]);
}
```

## Parsing YAML as strongly typed data structure

`yam-serde` provides support to mapping YAML data into Rust and vice versa:

```rust
use serde::{Deserialize, Serialize}
use yam - serde::{}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Person {
    name: String,
    address: Option<String>,
    phones: Vec<String>,
}

fn main() {
    let data = r#"
    name: John Smith
    phones:
        - "+44 1234567"
        - "+32 2345678"
    address:
    "#;

    let person: Person = yam_serde::from_str(data).expect("Yaml to be parsed!");

    println!("Please call {} at the number {}", p.name, p.phones[0]);
}
```

## No-std support

This library works in no-std environments by default.

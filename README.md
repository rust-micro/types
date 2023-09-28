# micro_types

[![Rust](https://github.com/rust-micro/types/actions/workflows/rust.yml/badge.svg)](https://github.com/rust-micro/types/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/micro_types.svg)](https://crates.io/crates/micro_types)

---

[![Repobeats analytics image](https://repobeats.axiom.co/api/embed/71482b23be17c3b52d28b472b11fdff312bfccb4.svg "Repobeats analytics image")](https://github.com/rust-micro/types/pulse)

---

This crate provides a set of types, which are backed by a server, and can be used to create a distributed system.

## Example

```rust
use dtypes::redis::DString as String;

let client = redis::Client::open("redis://localhost/").unwrap();
let mut s1 = String::with_value("Hello".to_string(), "s1", client.clone());
assert_eq!(s1, "Hello");
```

## Contributing

### Setup

Install `cargo` (recommended through `rustup`), `docker` with `compose`.

```bash
cargo install cargo-make
makers install
makers test
```

Add your change to the `CHANGELOG.md` file.

## License

This project is licensed under the [MIT license](LICENSE.md).

# micro_types

[![Rust](https://github.com/rust-micro/types/actions/workflows/rust.yml/badge.svg)](https://github.com/rust-micro/types/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/micro_types.svg)](https://crates.io/crates/micro_types)

This crate provides a set of types, which are backed by a server, and can be used to create a distributed system.

## Example

```rust=
use types::String;

let client = redis::Client::open("redis://localhost/").unwrap();
let mut s1 = TString::with_value("Hello".to_string(), "s1", client.clone());
assert_eq!(s1, "Hello");
```

## Contributing

### Setup

Install `docker` and `docker-compose`.

```bash
cargo install cargo-make
makers install
makers test
```

Add your change to the `CHANGELOG.md` file.

## License

This project is licensed under the [MIT license](LICENSE.md).

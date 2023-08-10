# paperdoll-tar

[![Latest version](https://img.shields.io/crates/v/paperdoll.svg)](https://crates.io/crates/paperdoll-tar)
[![Documentation](https://docs.rs/paperdoll/badge.svg)](https://docs.rs/paperdoll-tar)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)

[Tar](https://en.wikipedia.org/wiki/Tar_%28computing%29) archive container format for [paperdoll](https://github.com/fralonra/paperdoll).

File extension: `ppd`.

## Usage

```rust
use paperdoll_tar::paperdoll;

let factory = paperdoll::PaperdollFactory::default();

paperdoll_tar::save(&mut factory.to_manifest(), "/path/to/save/your.ppd");

let factory = paperdoll_tar::load("/path/to/save/your.ppd").unwrap();
```

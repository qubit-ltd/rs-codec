# Qubit Codec

[![Rust CI](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-codec/coverage-badge.json)](https://qubit-ltd.github.io/rs-codec/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-codec.svg?color=blue)](https://crates.io/crates/qubit-codec)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Chinese Document](https://img.shields.io/badge/Document-Chinese-blue.svg)](README.zh_CN.md)

Core codec traits and buffer conversion primitives for Rust.

## Overview

Qubit Codec is the domain-neutral foundation for Qubit codec crates. It contains
small traits and value types that are shared by binary, text, misc, and I/O
adapter crates without pulling in `std::io` stream helpers or concrete format
implementations.

This crate provides:

- `Encoder`, `Decoder`, and `Codec` traits for whole-value conversions.
- `Coder`, `CoderProgress`, and `CoderStatus` for caller-managed buffer
  conversion.
- `ByteOrder`, `ByteOrderSpec`, `BigEndian`, and `LittleEndian` for byte-order
  metadata shared by binary and text codecs.

Concrete codecs live in sibling crates such as `qubit-codec-binary`,
`qubit-codec-text`, and `qubit-codec-misc`.

## Installation

```toml
[dependencies]
qubit-codec = "0.1"
```

## Quick Example

```rust
use qubit_codec::{
    CoderProgress,
    CoderStatus,
    Encoder,
};

struct StringEncoder;

impl Encoder<str> for StringEncoder {
    type Output = String;
    type Error = core::convert::Infallible;

    fn encode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

let encoded = Encoder::<str>::encode(&StringEncoder, "codec")?;
assert_eq!("codec", encoded);

let progress = CoderProgress::complete(3, 4);
assert_eq!(CoderStatus::Complete, progress.status());

# Ok::<(), core::convert::Infallible>(())
```

## Crate Boundary

`qubit-codec` does not contain concrete binary formats, character sets,
percent/Base64/hex codecs, or `std::io` reader/writer adapters. Keep those in
domain crates so downstream users can depend on only the layers they need.

## Development

```bash
./align-ci.sh
RS_CI_SKIP_TOOLCHAIN_UPDATE=1 ./ci-check.sh
```

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full license text.

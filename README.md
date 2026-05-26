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

## Design Goals

- **Layered Boundaries**: keep domain-neutral traits separate from binary, text,
  misc, and stream-specific implementations.
- **Small Public Surface**: expose only the primitives that multiple codec
  crates need to share.
- **No I/O Coupling**: avoid `std::io` dependencies so buffer codecs can remain
  usable in non-stream contexts.
- **Policy Neutrality**: leave charset, malformed-input, and wire-format rules to
  domain crates.
- **Zero-Cost Markers**: represent byte order as copyable type/value markers
  without runtime allocation.
- **Stable Progress Reporting**: use `CoderProgress` and `CoderStatus` to make
  caller-managed buffer conversion explicit.

## Features

### Core Conversion Traits

- **`Encoder<Input>`**: converts a borrowed value into an owned output type.
- **`Decoder<Input>`**: converts a borrowed encoded value into an owned decoded
  output type.
- **`Codec<EncodeInput, DecodeInput>`**: marker trait for bidirectional codecs.

### Buffer Coder Primitives

- **`Coder<Input, Output>`**: converts input units into output units inside
  caller-provided buffers.
- **`CoderProgress`**: reports relative input units read and output units
  written.
- **`CoderStatus`**: distinguishes complete conversion from `NeedInput` and
  `NeedOutput` stops.

### Byte Order Markers

- **`ByteOrder`**: runtime byte-order enum for public APIs.
- **`ByteOrderSpec`**: type-level byte-order trait used by hot codecs.
- **`BigEndian` / `LittleEndian`**: zero-sized marker types.

### Focused Public API

- **`prelude` module**: imports the commonly used core traits and markers.
- **No concrete formats**: binary, text, and miscellaneous codecs are published
  in sibling crates.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
qubit-codec = "0.1"
```

## Quick Start

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

## API Reference

### Whole-Value Traits

| Trait | Purpose | Typical Implementor |
|-------|---------|---------------------|
| `Encoder<Input>` | Encode a borrowed input into an owned output | Text, binary, or misc encoder |
| `Decoder<Input>` | Decode a borrowed input into an owned output | Text, binary, or misc decoder |
| `Codec<EncodeInput, DecodeInput>` | Mark one type as both encoder and decoder | Bidirectional codecs |

### `Coder` Operations

| Method | Description |
|--------|-------------|
| `max_output_len(input_len)` | Return a finite output upper bound when known |
| `reset()` | Reset retained conversion state |
| `convert(input, input_index, output, output_index)` | Convert input units into output units |
| `finish(output, output_index)` | Flush buffered output after all input has been consumed |

### `CoderStatus` Values

| Status | Meaning |
|--------|---------|
| `Complete` | The current conversion step completed |
| `NeedInput` | More input units are required |
| `NeedOutput` | More output capacity is required |

### Byte Order Types

| Type | Use Case |
|------|----------|
| `ByteOrder` | Runtime byte-order selection in public APIs |
| `ByteOrderSpec` | Type-level byte-order abstraction |
| `BigEndian` | Big-endian type marker |
| `LittleEndian` | Little-endian type marker |

## Crate Boundary

`qubit-codec` does not contain concrete binary formats, character sets,
percent/Base64/hex codecs, or `std::io` reader/writer adapters. Keep those in
domain crates so downstream users can depend on only the layers they need.

## Performance Considerations

All core abstractions are trait or marker types. `BigEndian` and `LittleEndian`
are zero-sized, and `ByteOrder` is a small copyable enum. The crate performs no
heap allocation by itself; allocation behavior is controlled by concrete codec
implementations in downstream crates.

## Testing & Code Coverage

This project keeps the core trait contracts covered by integration tests under
`tests/`.

### Running Tests

```bash
# Run all tests
cargo test

# Run with coverage report
./coverage.sh

# Generate text format report
./coverage.sh text

# Align code with CI requirements
./align-ci.sh

# Run CI checks (format, clippy, test, coverage, audit)
RS_CI_SKIP_TOOLCHAIN_UPDATE=1 ./ci-check.sh
```

## Dependencies

`qubit-codec` has no runtime dependencies.

## License

Copyright (c) 2026. Haixing Hu.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

See [LICENSE](LICENSE) for the full license text.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Guidelines

- Keep this crate free of concrete format implementations.
- Document public traits and marker types with examples.
- Keep tests comprehensive and deterministic.
- Ensure all checks pass before submitting a PR.

## Author

**Haixing Hu**

## Related Projects

- [qubit-codec-binary](https://github.com/qubit-ltd/rs-codec-binary): binary
  buffer codecs.
- [qubit-codec-text](https://github.com/qubit-ltd/rs-codec-text): charset and
  Unicode buffer codecs.
- [qubit-codec-misc](https://github.com/qubit-ltd/rs-codec-misc): reusable
  miscellaneous byte and text codecs.
- [qubit-io](https://github.com/qubit-ltd/rs-io): generic `std::io` helpers.
- More Rust libraries from Qubit are available under the
  [qubit-ltd](https://github.com/qubit-ltd) GitHub organization.

---

Repository: [https://github.com/qubit-ltd/rs-codec](https://github.com/qubit-ltd/rs-codec)

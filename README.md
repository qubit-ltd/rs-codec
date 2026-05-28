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

- `Codec<Value, Unit>` plus `DecodeFailure` / `DecodeErrorInfo` for low-level
  single-value buffer codecs and buffered-error control flow.
- `CodecValueEncoder` and `CodecBufferedEncoder` adapters for encoding through
  a supplied `Codec`.
- `ValueEncoder` and `ValueDecoder` traits for owned whole-value convenience APIs.
- `Transcoder`, `TranscodeProgress`, and `TranscodeStatus` for caller-managed logical-stream
  conversion.
- `BufferedEncoder`, `BufferedDecoder`, and `BufferedConverter` marker traits
  for semantic transcoder direction.
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
- **Stable Progress Reporting**: use `TranscodeProgress` and `TranscodeStatus` to make
  caller-managed buffer conversion explicit.

## Features

### Core Conversion Traits

- **`Codec<Value, Unit>`**: encodes and decodes one value or codec quantum
  against a caller-managed unit buffer.
- **`DecodeFailure` / `DecodeErrorInfo`**: expose the minimal incomplete-vs-invalid
  view of codec-specific decode errors for buffered adapters.
- **`ValueEncoder<Input>`**: converts a borrowed value into an owned output type.
- **`ValueDecoder<Input>`**: converts a borrowed encoded value into an owned decoded
  output type.
- **`CodecValueEncoder<C, Value, Unit>`**: wraps a `Codec<Value, Unit>` as a
  `ValueEncoder<Value>` that returns owned `Vec<Unit>` output.

### Buffer Transcoder Primitives

- **`Transcoder<Input, Output>`**: converts input units into output units inside
  caller-provided buffers, then finalizes pending stream state at EOF.
- **`BufferedEncoder<Value, Unit>`**: semantic `Transcoder` bound for value-to-unit
  buffered encoding.
- **`BufferedDecoder<Unit, Value>`**: semantic `Transcoder` bound for unit-to-value
  buffered decoding.
- **`BufferedConverter<InputUnit, OutputUnit>`**: semantic `Transcoder` bound for
  unit-to-unit buffered conversion.
- **`CodecBufferedEncoder<C>`**: wraps a `Codec<Value, Unit>` as a
  `BufferedEncoder<Value, Unit>` over caller-provided output buffers.
- **`TranscodeProgress`**: reports relative input units read and output units
  written.
- **`TranscodeStatus`**: distinguishes complete conversion from `NeedInput` and
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
qubit-codec = "0.4"
```

## Quick Start

```rust
use qubit_codec::{
    TranscodeProgress,
    TranscodeStatus,
    ValueEncoder,
};

struct StringEncoder;

impl ValueEncoder<str> for StringEncoder {
    type Output = String;
    type Error = core::convert::Infallible;

    fn encode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

let encoded = ValueEncoder::<str>::encode(&StringEncoder, "codec")?;
assert_eq!("codec", encoded);

let progress = TranscodeProgress::complete(3, 4);
assert_eq!(TranscodeStatus::Complete, progress.status());

# Ok::<(), core::convert::Infallible>(())
```

## API Reference

### Core Codec Traits

| Trait | Purpose | Typical Implementor |
|-------|---------|---------------------|
| `Codec<Value, Unit>` | Encode/decode one value or quantum against caller buffers | Binary scalar, charset char, escaped byte, Base64 quantum |
| `ValueEncoder<Input>` | Encode a borrowed input into an owned output | Convenience text, binary, or misc helper |
| `ValueDecoder<Input>` | Decode a borrowed input into an owned output | Convenience text, binary, or misc helper |
| `BufferedEncoder<Value, Unit>` | Encode logical values into caller-provided unit buffers | Charset or binary buffered encoder |
| `BufferedDecoder<Unit, Value>` | Decode encoded units into caller-provided value buffers | Charset or binary buffered decoder |
| `BufferedConverter<InputUnit, OutputUnit>` | Convert encoded units between representations | Charset or binary buffered converter |

| Type | Purpose |
|------|---------|
| `DecodeFailure` | Generic incomplete-or-invalid view of a codec-specific decode error |
| `DecodeErrorInfo` | Trait implemented by decode errors that expose `DecodeFailure` metadata |

### Codec Adapters

| Type | Purpose |
|------|---------|
| `CodecValueEncoder<C, Value, Unit>` | Allocate owned `Vec<Unit>` output for one borrowed `Value` by using `C: Codec<Value, Unit>` without requiring `Value: Clone` |
| `CodecBufferedEncoder<C>` | Encode `Value` slices into caller-provided `Unit` buffers by using `C: Codec<Value, Unit>` |

### `Transcoder` Operations

| Method | Description |
|--------|-------------|
| `max_output_len(input_len)` | Return a finite output upper bound when known |
| `max_finish_output_len()` | Return a finite finalization output upper bound when known |
| `reset()` | Reset retained stream state while keeping configuration |
| `transcode(input, input_index, output, output_index)` | Convert input units into output units |
| `finish(output, output_index)` | Finalize EOF state, flush trailers, or reject incomplete input |

### `TranscodeStatus` Values

| Status | Meaning |
|--------|---------|
| `Complete` | The current conversion step completed |
| `NeedInput` | More input units are required unless the caller is ready to call `finish()` at EOF |
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

Core traits and buffered adapters do not require heap allocation. `BigEndian`
and `LittleEndian` are zero-sized, and `ByteOrder` is a small copyable enum.
`CodecValueEncoder` allocates owned `Vec<Unit>` output because that is the
`ValueEncoder` contract; concrete downstream codecs may have their own
allocation behavior.

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

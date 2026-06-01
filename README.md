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

- `Codec<Value, Unit>` for low-level single-value buffer codecs.
- `CodecValueEncoder`, `CodecValueDecoder`, `CodecBufferedEncoder`,
  `CodecBufferedDecoder`, and `CodecBufferedConverter` adapters for explicit
  codec-backed value and buffered conversion.
- `BufferedEncodeEngine`, `BufferedEncodeHooks`, and `EncodePlan` for reusing
  the common buffered encoding loop in policy-aware downstream encoders.
- `BufferedDecodeEngine`, `BufferedDecodeHooks`, `DecodeAction`, and
  `DecodeContext` for reusing the common buffered decoding loop in policy-aware
  downstream decoders.
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
- **`CodecEncodeError` / `CodecDecodeError` / `CodecConvertError`**: add
  adapter-level encode, decode, and conversion errors without hiding
  codec-specific failures.
- **`ValueEncoder<Input>`**: converts a borrowed value into an owned output type.
- **`ValueDecoder<Input>`**: converts a borrowed encoded value into an owned decoded
  output type.
- **`CodecValueEncoder<C, Value, Unit>`**: wraps a `Codec<Value, Unit>` as a
  `ValueEncoder<Value>` that returns owned `Vec<Unit>` output.
- **`CodecValueDecoder<C, Value, Unit>`**: wraps a `Codec<Value, Unit>` as a
  `ValueDecoder<[Unit]>` that accepts exactly one encoded value.

### Buffer Transcoder Primitives

- **`Transcoder<Input, Output>`**: converts input units into output units inside
  caller-provided buffers, then finishes internally retained output after the
  caller has handled any incomplete input tail.
- **`BufferedEncoder<Value, Unit>`**: semantic `Transcoder` bound for value-to-unit
  buffered encoding.
- **`BufferedDecoder<Unit, Value>`**: semantic `Transcoder` bound for unit-to-value
  buffered decoding.
- **`BufferedConverter<InputUnit, OutputUnit>`**: semantic `Transcoder` bound for
  unit-to-unit buffered conversion.
- **`CodecBufferedEncoder<C>`**: wraps a `Codec<Value, Unit>` as a
  `BufferedEncoder<Value, Unit>` over caller-provided output buffers.
- **`BufferedEncodeEngine<C, H>`**: reusable engine that owns a codec plus
  policy hooks and runs the common buffered encoding loop.
- **`BufferedEncodeHooks<C, Value, Unit>`**: policy hook trait used by
  codec-backed encoders that need custom transcode/finalization behavior while
  sharing the common loop.
- **`EncodePlan<P>`**: per-value write plan carrying the output capacity bound
  required before a hook writes one value.
- **`CodecBufferedDecoder<C, Unit>`**: wraps a `Codec<Value, Unit>` as a
  strict `BufferedDecoder<Unit, Value>` that leaves engine-detected incomplete
  tails in the caller's input buffer and wraps codec-reported decode errors.
- **`BufferedDecodeEngine<C, H, Unit>`**: reusable engine that owns a codec,
  policy hooks, and the common decode loop.
- **`BufferedDecodeHooks<C, Unit, Value>`**: policy hook trait used by
  codec-backed decoders that need custom malformed/incomplete behavior while
  sharing the common decode loop.
- **`DecodeAction<Value>`**: hook return value used by decoder engines for
  transcode-stage policy decisions.
- **`CodecBufferedConverter<D, E, Value, InputUnit>`**: composes a decoding codec
  and an encoding codec as a policy-free `BufferedConverter`.
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
qubit-codec = "0.5"
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
| `CodecEncodeError<E>` | Adapter-level encode error that wraps codec errors or invalid input indices |
| `CodecDecodeError<E>` | Adapter-level decode error that wraps codec errors, incomplete input, invalid indices, or trailing input |
| `CodecConvertError<D, E>` | Adapter-level converter error that separates decode and encode failures |

### Codec Adapters

| Type | Purpose |
|------|---------|
| `CodecValueEncoder<C, Value, Unit>` | Allocate owned `Vec<Unit>` output for one borrowed `Value` by using `C: Codec<Value, Unit>` without requiring `Value: Clone` |
| `CodecValueDecoder<C, Value, Unit>` | Decode exactly one borrowed `[Unit]` slice into `Value` by using `C: Codec<Value, Unit>` |
| `CodecBufferedEncoder<C>` | Encode `Value` slices into caller-provided `Unit` buffers by using `C: Codec<Value, Unit>` |
| `CodecBufferedDecoder<C, Unit>` | Strictly decode `Unit` slices into caller-provided `Value` buffers by using `C: Codec<Value, Unit>` |
| `CodecBufferedConverter<D, E, Value, InputUnit>` | Decode source units with `D: Codec<Value, InputUnit>` and encode target units with `E: Codec<Value, OutputUnit>` |

### Encoder Hooks And Engines

| Type | Purpose |
|------|---------|
| `BufferedEncodeEngine<C, H>` | Reusable buffered encoder engine backed by a low-level `Codec` and policy hooks |
| `BufferedEncodeHooks<C, Value, Unit>` | Hook contract for planning, writing, resetting, and finalizing encoded output |
| `EncodePlan<P>` | Prepared per-value capacity bound plus implementation-specific write payload |

### Decoder Hooks And Engines

| Type | Purpose |
|------|---------|
| `BufferedDecodeEngine<C, H, Unit>` | Reusable buffered decoder engine backed by a low-level `Codec` and policy hooks |
| `BufferedDecodeHooks<C, Unit, Value>` | Hook contract for malformed/incomplete decode policy |
| `DecodeContext` | Context passed to decode policy hooks |
| `DecodeAction<Value>` | Transcode-stage policy action: need input, skip input, or emit a value |

### `Transcoder` Operations

| Method | Description |
|--------|-------------|
| `max_output_len(input_len)` | Return a finite output upper bound when known |
| `max_finish_output_len()` | Return a finite final-output upper bound when known |
| `reset()` | Reset retained stream state while keeping configuration |
| `transcode(input, input_index, output, output_index)` | Convert input units into output units |
| `finish(output, output_index)` | Finish internally retained output such as reset bytes, digests, or trailers |

### `TranscodeStatus` Values

| Status | Meaning |
|--------|---------|
| `Complete` | The current conversion step completed |
| `NeedInput` | More input units are required; the incomplete tail remains in the caller's input buffer |
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

Runtime dependencies are intentionally small:

- `thiserror` provides public error type implementations.

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

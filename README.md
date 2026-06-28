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
adapter crates, without concrete format implementations.

This crate provides:

- `Codec` for low-level single-value buffer codecs.
- `CodecValueExt`, `CodecValueEncoder`, `CodecValueDecoder`,
  `CodecTranscodeEncoder`,
  `CodecTranscodeDecoder`, and `CodecTranscodeConverter` adapters for explicit
  codec-backed value and buffered conversion.
- `TranscodeEncodeEngine`, `TranscodeEncodeHooks`, and
  `EncodeUnencodableAction` for reusing the common buffered encoding loop in
  policy-aware downstream encoders.
- `TranscodeDecodeEngine`, `TranscodeDecodeHooks`, `DecodeInvalidAction`, and
  `DecodeContext` for reusing the common buffered decoding loop in policy-aware
  downstream decoders.
- `TranscodeConvertEngine` and `TranscodeConvertEngineError` for policy-aware
  unit-to-unit conversion pipelines built from a decode side and an encode side.
- `ValueEncoder` and `ValueDecoder` traits for owned whole-value convenience APIs.
- `Transcoder`, `TranscodeProgress`, and `TranscodeStatus` for
  caller-managed logical-stream conversion.
- `TranscodeEncoder`, `TranscodeDecoder`, and `TranscodeConverter` marker traits
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
- **Policy Neutrality**: leave charset, malformed-input, and wire-format rules to
  domain crates.
- **Zero-Cost Markers**: represent byte order as copyable type/value markers
  without runtime allocation.
- **Stable Progress Reporting**: use `TranscodeProgress` and `TranscodeStatus` to make
  caller-managed buffer conversion explicit.

## Features

### Core Conversion Traits

- **`Codec`**: encodes and decodes one value or codec quantum
  against a caller-managed unit buffer.
- **`DecodeFailure`**: separates incomplete-prefix flow control from
  codec-domain invalid input returned by `Codec::decode`.
- **`CodecEncodeError` / `CodecDecodeError` / `CodecConvertError`**: add
  adapter-level encode, decode, and conversion errors without hiding
  codec-specific failures. Buffer index and capacity failures are represented by
  `TranscodeError`.
- **`ValueEncoder<Input>`**: converts a borrowed value into an owned output type.
- **`ValueDecoder<Input>`**: converts a borrowed encoded value into an owned decoded
  output type.
- **`CodecValueEncoder<C>`**: wraps a `Codec` as a
  `ValueEncoder<C::Value>` that returns owned `Vec<C::Unit>` output.
- **`CodecValueDecoder<C>`**: wraps a `Codec` as a
  `ValueDecoder<[C::Unit]>` that accepts exactly one encoded value.
- **`CodecValueExt`**: extension trait for checked one-value codec helpers such
  as reset-prefixed encode and exact decode with flush handling.

### Buffered Transcoder Primitives

- **`Transcoder<Input, Output>`**: converts input units into output units inside
  caller-provided buffers, then finishes internally retained output after the
  caller has handled any incomplete input tail.
- **`TranscodeEncoder<Value, Unit>`**: semantic `Transcoder` bound for value-to-unit
  buffered encoding.
- **`TranscodeDecoder<Unit, Value>`**: semantic `Transcoder` bound for unit-to-value
  buffered decoding.
- **`TranscodeConverter<InputUnit, OutputUnit>`**: semantic `Transcoder` bound for
  unit-to-unit buffered conversion.
- **`CodecTranscodeEncoder<C>`**: wraps a `Codec` as a
  `TranscodeEncoder<C::Value, C::Unit>` over caller-provided output buffers.
- **`TranscodeEncodeEngine<C, H>`**: reusable engine that owns a
  codec plus policy hooks and runs the common buffered encoding loop.
- **`TranscodeEncodeHooks<C>`**: policy hook trait used by
  codec-backed encoders that need unencodable-value, reset, or finalization
  policy while sharing the common loop.
- **`EncodeUnencodableAction<Value>`**: action returned by encode hooks for
  unencodable values: skip the value or encode a replacement.
- **`EncodeOutcome` / `EncodeContext<'a, Value, Unit>`**: low-level engine
  plumbing for one buffered encode attempt.
- **`CodecTranscodeDecoder<C>`**: wraps a `Codec` as a
  strict `TranscodeDecoder<C::Unit, C::Value>` that leaves engine-detected incomplete
  tails in the caller's input buffer and wraps codec-reported decode errors.
- **`TranscodeDecodeEngine<C, H>`**: reusable engine that owns a
  codec, policy hooks, and the common decode loop.
- **`TranscodeDecodeHooks<C>`**: policy hook trait used by
  codec-backed decoders that need custom invalid-input behavior while
  sharing the common decode loop.
- **`DecodeInvalidAction<Value>`**: hook return value used by decoder engines
  for invalid-input policy decisions.
- **`CodecTranscodeConverter<D, E>`**: composes a
  decoding codec and an encoding codec as a policy-free `TranscodeConverter`.
- **`TranscodeConvertEngine<D, E, DH, EH>`**: reusable unit-to-unit converter
  engine that composes decode hooks, encode hooks, and the common buffered
  conversion loop.
- **`TranscodeDecodeInput<I>`**: owns a unit-level `BufferedInput` and drives
  caller-provided streaming decoders through `transcode_into` /
  `finish_transcode_into`.
- **`TranscodeEncodeOutput<O>`**: owns a unit-level `BufferedOutput`; ordinary
  `flush` drains buffered units. Stateful streaming encoders use `transcode_from`
  and `finish`.
- **`TranscodeProgress`**: reports relative input units read and output units
  written.
- **`TranscodeStatus`**: distinguishes complete conversion from `NeedInput` and
  `NeedOutput` stops.
- **`TranscodeError` / `CapacityError` / `TranscodeContractError`**: report
  framework-level buffer, capacity-planning, and broken-progress contract
  failures separately from codec or policy domain errors.

### Byte Order Markers

- **`ByteOrder`**: runtime byte-order enum for public APIs.
- **`ByteOrderSpec`**: type-level byte-order trait used by hot codecs.
- **`BigEndian` / `LittleEndian`**: zero-sized marker types.

### Focused Public API

- **No concrete formats**: binary, text, and miscellaneous codecs are published
  in sibling crates.

## Choosing the Right Abstraction

`qubit-codec` ships several layers because real codec stacks have different
needs. Use this decision tree to pick the smallest piece that fits your case.

```text
What are you writing?

├── A new codec for one logical value (a UTF-8 char, a LEB128 integer,
│   a Base64 quantum, a fixed-width scalar, …)
│       → implement Codec
│         (unchecked single-value contract; the foundation everything else builds on)
│
├── A whole-string codec where "one logical value" has no useful meaning
│   (Base64 padding, hex with separators, percent encoding, C string literal, …)
│       → implement ValueEncoder<Input> / ValueDecoder<Input> directly
│         (skip Codec; these two traits also serve as the convenience layer)
│
├── A streaming wrapper around an existing Codec, with no error policy:
│   strict pass-through that surfaces every codec error as-is
│       → use CodecTranscodeDecoder<C> / CodecTranscodeEncoder<C>
│         / CodecTranscodeConverter<D, E>
│         (no custom code; you get a fully wired Transcoder)
│
├── An owned-output wrapper around a Codec (one call → one Vec<Unit>
│   or one Value)
│       → use CodecValueEncoder<C> / CodecValueDecoder<C>
│         (allocates per call; convenience-layer ValueEncoder/Decoder)
│
└── A streaming codec that needs to make decisions on malformed input:
    skip, replace, count, or report — not just propagate
        → implement TranscodeDecodeHooks<C> / TranscodeEncodeHooks<C>
          and wrap them in TranscodeDecodeEngine<C, H> / TranscodeEncodeEngine<C, H>
          (you only write the policy; the engine owns the buffered loop,
           cursor bookkeeping, NeedInput/NeedOutput reporting, and capacity checks)

For unit-to-unit conversion (e.g. UTF-8 bytes → UTF-16 bytes), compose a
decode codec + an encode codec:
- strict pipeline    → CodecTranscodeConverter<D, E>
- with policy hooks  → TranscodeConvertEngine<D, E, DH, EH>
```

### Layer overview

```text
┌────────────────────────────────────────────────────────────────┐
│  qubit-io-binary / qubit-io-text             (concrete I/O)    │
├────────────────────────────────────────────────────────────────┤
│  TranscodeDecodeInput / TranscodeEncodeOutput  (I/O bridges)   │
├────────────────────────────────────────────────────────────────┤
│  TranscodeXxxEngine + TranscodeXxxHooks       (policy + loop)  │
│  CodecTranscodeDecoder / Encoder / Converter  (strict bridges) │
├────────────────────────────────────────────────────────────────┤
│  Transcoder<Input, Output> + TranscodeProgress + TranscodeStatus│
│  ValueEncoder<Input> / ValueDecoder<Input>      (convenience)  │
├────────────────────────────────────────────────────────────────┤
│  Codec                                  (single-value, unchecked) │
└────────────────────────────────────────────────────────────────┘
```

Implementing further up the stack does *not* mean rewriting the lower layers:
`CodecValueEncoder<C>` and `CodecTranscodeDecoder<C>` are concrete adapters
that turn any `Codec` into the higher-layer trait for free. Only drop down to
the engine + hooks layer when you actually need policy decisions on invalid
input, replacement output, or stateful finish output.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
qubit-codec = "0.10"
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

    fn encode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

let mut encoder = StringEncoder;
let encoded = ValueEncoder::<str>::encode(&mut encoder, "codec")?;
assert_eq!("codec", encoded);

let progress = TranscodeProgress::complete(3, 4);
assert_eq!(TranscodeStatus::Complete, progress.status());

# Ok::<(), core::convert::Infallible>(())
```

## API Reference

### Core Codec Traits

| Trait | Purpose | Typical Implementor |
|-------|---------|---------------------|
| `Codec` | Encode/decode one value or quantum against caller buffers | Binary scalar, charset char, escaped byte, Base64 quantum |
| `ValueEncoder<Input>` | Encode a borrowed input into an owned output | Convenience text, binary, or misc helper |
| `ValueDecoder<Input>` | Decode a borrowed input into an owned output | Convenience text, binary, or misc helper |
| `TranscodeEncoder<Value, Unit>` | Encode logical values into caller-provided unit buffers | Charset or binary buffered encoder |
| `TranscodeDecoder<Unit, Value>` | Decode encoded units into caller-provided value buffers | Charset or binary buffered decoder |
| `TranscodeConverter<InputUnit, OutputUnit>` | Convert encoded units between representations | Charset or binary buffered converter |

| Type | Purpose |
|------|---------|
| `DecodeFailure<E>` | Low-level decode result for incomplete visible prefixes or invalid codec-domain input |
| `CodecEncodeError<E>` | Adapter-level encode error that wraps codec reset/encode/flush errors or unencodable values |
| `CodecDecodeError<E>` | Adapter-level decode error that wraps codec reset/decode/flush errors, incomplete input, or trailing input |
| `CodecConvertError<D, E>` | Adapter-level converter error that separates decode failures from full encode-side `CodecEncodeError<E>` failures |
| `TranscodeError<E>` | Streaming framework error for invalid indices, insufficient output, output-length overflow, or a domain error |
| `CapacityError` | Capacity-planning error returned before allocating or writing output |
| `TranscodeContractError` | Error reported when a custom `Transcoder` returns inconsistent progress |

### Codec Adapters

| Type | Purpose |
|------|---------|
| `CodecValueExt` | Provide checked one-value helper methods for all `C: Codec` without expanding the low-level `Codec` contract |
| `CodecEncodeValueResult<E>` | Result alias returned by reset-prefixed one-value encode helpers |
| `CodecDecodeValueWithFlushResult<V, E>` | Result alias returned by decode-and-flush one-value helpers with consumed and flushed counts |
| `CodecDecodeExactValueWithFlushResult<V, E>` | Result alias returned by exact decode-and-flush one-value helpers |
| `CodecValueEncoder<C>` | Allocate owned `Vec<C::Unit>` output for one borrowed `C::Value` by using `C: Codec` without requiring `C::Value: Clone` |
| `CodecValueDecoder<C>` | Decode exactly one borrowed `[C::Unit]` slice into `C::Value` by using `C: Codec` |
| `CodecTranscodeEncoder<C>` | Encode `C::Value` slices into caller-provided `C::Unit` buffers by using `C: Codec` |
| `CodecTranscodeDecoder<C>` | Strictly decode `C::Unit` slices into caller-provided `C::Value` buffers by using `C: Codec` |
| `CodecTranscodeConverter<D, E>` | Decode `D::Unit` source units and encode `E::Unit` target units with `E::Value = D::Value` |

### I/O Adapters

| Type | Purpose |
|------|---------|
| `TranscodeDecodeInput<I>` | Decode units from a `qubit_io::Input` by passing a caller-owned streaming decoder to `transcode_into` and `finish_transcode_into` |
| `TranscodeEncodeOutput<O>` | Own a `qubit_io::Output`; ordinary `flush` drains buffered units. Stateful streaming encoders use `transcode_from` and `finish` |

### Encoder Hooks And Engines

| Type | Purpose |
|------|---------|
| `TranscodeEncodeEngine<C, H>` | Reusable buffered encoder engine backed by a low-level `Codec` and policy hooks |
| `TranscodeEncodeHooks<C>` | Hook contract for unencodable-value policy, preparing for reset, and finalizing encoded output |
| `TranscodeEncodeEngineError<C, H>` | Separates codec lifecycle failures from encode-hook policy failures |
| `EncodeUnencodableAction<Value>` | Policy action returned for values outside the codec's encodable domain |
| `EncodeOutcome` | Per-value engine outcome: consumed with written output, or needs more output without consuming |
| `EncodeContext<'a, Value, Unit>` | Input value, input index, output slice, and cursor used by encode engine helpers |

### Decoder Hooks And Engines

| Type | Purpose |
|------|---------|
| `TranscodeDecodeEngine<C, H>` | Reusable buffered decoder engine backed by a low-level `Codec` and policy hooks |
| `TranscodeDecodeHooks<C>` | Hook contract for invalid-input decode policy |
| `TranscodeDecodeEngineError<C, H>` | Separates codec lifecycle failures from decode-hook policy failures |
| `DecodeContext` | Context passed to decode policy hooks |
| `DecodeInvalidAction<Value>` | Invalid-input policy action: skip input or emit a replacement value |

### Converter Engines

| Type | Purpose |
|------|---------|
| `TranscodeConvertEngine<D, E, DH, EH>` | Reusable unit-to-unit converter that decodes with `D`, encodes with `E`, and applies decode/encode hooks |
| `TranscodeConvertEngineError<D, E>` | Separates decode-side and encode-side converter failures |

### `Transcoder` Operations

| Method | Description |
|--------|-------------|
| `max_transcode_output_len(input_len)` | Return a finite streaming-phase output upper bound when known |
| `max_total_output_len(input_len)` | Return the full `reset -> transcode -> finish` output upper bound |
| `max_reset_output_len()` | Return a finite reset-output upper bound when known |
| `max_finish_output_len()` | Return a finite final-output upper bound when known |
| `reset()` | Reset retained stream state while keeping configuration |
| `transcode(input, input_index, output, output_index)` | Convert input units into output units |
| `transcode_complete_into(input, output)` | Run one complete `reset -> transcode -> finish` stream from the start of the supplied slices |
| `finish(output, output_index)` | Finish internally retained output such as reset bytes, digests, or trailers |

### `TranscodeStatus` Values

| Status | Meaning |
|--------|---------|
| `Complete` | The current conversion step completed |
| `NeedInput` | More input units are required; the incomplete tail remains in the caller's input buffer |
| `NeedOutput` | More output capacity is required |

### Contract Notes

- `Codec::MIN_UNITS_PER_VALUE` is the safety lower bound for calling `Codec::decode`;
  `Codec::MAX_UNITS_PER_VALUE` is the per-value output/read upper bound. Checked
  adapters assert `min <= max` before using these values.
- `Codec::decode` returns `DecodeFailure::Incomplete` when the visible input is a
  valid prefix that needs more units, and `DecodeFailure::Invalid` for
  codec-domain malformed, non-canonical, or otherwise invalid input.
- `encode_len(value)` must equal the number of units `Codec::encode` writes for
  the same value and codec state, and it must not exceed
  `Codec::MAX_UNITS_PER_VALUE`.
- Stateful one-value callers should use `CodecValueExt::max_encode_value_units()`
  with `CodecValueExt::encode_value_with_reset()`, or
  `CodecValueExt::decode_exact_value_with_flush()` when the input must contain
  exactly one encoded value. These helpers keep reset/flush capacity checks and
  overflow handling in the value adapter layer.
- `CodecDecodeError` / `CodecEncodeError` are adapter-level wrappers.
  `TranscodeError` is the streaming framework wrapper. Concrete codec,
  charset, or policy failures remain the associated domain error.
- `NeedInput` means the reported tail was not consumed and must remain available
  when the caller retries with more input. It is a streaming boundary signal,
  not an EOF error; `finish` does not receive that source tail. Callers must
  apply their own EOF policy before finalization.
- Default codec-backed decoders and converters are intended for formats whose
  value boundary is locally decidable from the visible prefix plus codec state.
  Formats that require EOF-aware maximal-munch parsing, delayed boundary
  decisions, or reinterpretation of a pending prefix at EOF should use a custom
  `Transcoder` or value-level facade for that policy.
- `NeedOutput` means the reported input was not fully consumed because the
  output slice reached its bound.

### Byte Order Types

| Type | Use Case |
|------|----------|
| `ByteOrder` | Runtime byte-order selection in public APIs |
| `ByteOrderSpec` | Type-level byte-order abstraction |
| `BigEndian` | Big-endian type marker |
| `LittleEndian` | Little-endian type marker |

## Crate Boundary

`qubit-codec` does not contain concrete binary formats, character sets, or
percent/Base64/hex codecs. Its I/O-facing surface is limited to low-level
`qubit_io::Input` / `qubit_io::Output` bridge types used by downstream stream
crates. Keep `std::io::Read` / `std::io::Write` extension traits and concrete
reader/writer adapters in domain crates so downstream users can depend on only
the layers they need.

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
- `qubit-io` provides `BufferedInput` and `BufferedOutput` used by `TranscodeDecodeInput` and `TranscodeEncodeOutput`.

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

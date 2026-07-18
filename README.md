# Qubit Codec

[![Rust CI](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-codec/coverage-badge.json)](https://qubit-ltd.github.io/rs-codec/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-codec.svg?color=blue)](https://crates.io/crates/qubit-codec)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

Core codec traits and buffer conversion primitives for Rust.

## Overview

Qubit Codec is the domain-neutral foundation for Qubit codec crates. It contains
small traits and value types that are shared by binary, text, misc, and I/O
adapter crates, without concrete format implementations.

This crate provides:

- `Codec` for low-level single-value buffer codecs.
- `CodecValueEncoder`, `CodecValueDecoder`, `CodecTranscodeEncoder`,
  `CodecTranscodeDecoder`, and `CodecTranscodeConverter` adapters for explicit
  codec-backed value and buffered conversion.
- `TranscodeEncodeEngine`, `TranscodeEncodeHooks`, and
  `EncodeUnencodableAction` for reusing the common buffered encoding loop in
  policy-aware downstream encoders.
- `TranscodeDecodeEngine`, `TranscodeDecodeHooks`, `DecodeInvalidAction`, and
  `DecodeContext` for reusing the common buffered decoding loop in policy-aware
  downstream decoders.
- `TranscodeConvertEngine` and `TranscodeConvertError` for policy-aware
  unit-to-unit conversion pipelines built from a decode side and an encode side.
- `ValueEncoder` and `ValueDecoder` traits for owned whole-value convenience APIs.
- `Transcoder`, `TranscodeProgress`, and `TranscodeStatus` for
  caller-managed logical-stream conversion.
- `TranscodeEncoder`, `TranscodeDecoder`, and `TranscodeConverter` marker traits
  for semantic transcoder direction.
- `ByteOrder`, `ByteOrderSpec`, `BigEndian`, `LittleEndian`, and
  `NativeEndian` for byte-order metadata shared by binary and text codecs.

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

### Cargo Features

The default feature set is empty. Enable the `io` feature to use the
`qubit-io` bridge, including `TranscodeDecodeInput` and
`TranscodeEncodeOutput`.

### Core Conversion Traits

- **`Codec`**: encodes and decodes one value or codec quantum
  against a caller-managed unit buffer.
- **`DecodeFailure`**: separates incomplete-prefix flow control from
  codec-domain invalid input returned by `Codec::decode`.
- **`TranscodeFailure`**: carries framework failures such as invalid indices,
  insufficient output, overflow, incomplete input, and trailing input.
- **`TranscodeDomainError<E>`**: attaches reset, main, or finish phase context
  to codec and policy errors.
- **`TranscodeEncodeError<E, V>` / `TranscodeDecodeError<E>` /
  `TranscodeConvertError<DE, EE, V>`**: directional public errors that combine
  framework and domain failures.
- **`ValueEncoder<Input>`**: converts a borrowed value into an owned output type.
- **`ValueDecoder<Input>`**: converts a borrowed encoded value into an owned decoded
  output type.
- **`CodecValueEncoder<C>`**: wraps a `Codec` as a
  `ValueEncoder<C::Value>` that returns owned `Vec<C::Unit>` output.
- **`CodecValueDecoder<C>`**: wraps a `Codec` as a
  `ValueDecoder<[C::Unit]>` that accepts exactly one encoded value.

### Buffered Transcoder Primitives

- **`Transcoder`**: converts `Input` associated items into `Output` associated
  items inside caller-provided buffers, then finishes internally retained output
  after the caller has handled any incomplete input tail.
- **`TranscodeEncoder`**: semantic `Transcoder` bound for value-to-unit buffered
  encoding.
- **`TranscodeDecoder`**: semantic `Transcoder` bound for unit-to-value buffered
  decoding.
- **`TranscodeConverter`**: semantic `Transcoder` bound for unit-to-unit buffered
  conversion.
- **`CodecTranscodeEncoder<C>`**: wraps a `Codec` as a `TranscodeEncoder` over
  caller-provided output buffers.
- **`TranscodeEncodeEngine<C, H>`**: reusable engine that owns a
  codec plus policy hooks and runs the common buffered encoding loop.
- **`TranscodeEncodeHooks<C>`**: policy hook trait used by
  codec-backed encoders that need unencodable-value, reset, or finalization
  policy while sharing the common loop.
- **`EncodeUnencodableAction<Value>`**: action returned by encode hooks for
  unencodable values: skip the value or encode a replacement.
- **`EncodeOutcome` / `EncodeContext<'a, Value, Unit>`**: low-level engine
  plumbing for one buffered encode attempt.
- **`CodecTranscodeDecoder<C>`**: wraps a `Codec` as a strict
  `TranscodeDecoder` that leaves engine-detected incomplete tails in the
  caller's input buffer and wraps codec-reported decode errors.
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
- **`TranscodeDecodeInput<I>`** *(requires the `io` feature)*: owns a
  unit-level `BufferedInput` and drives caller-provided streaming decoders
  through `transcode_into` / `finish_transcode_into`.
- **`TranscodeEncodeOutput<O>`** *(requires the `io` feature)*: owns a
  unit-level `BufferedOutput`; ordinary `flush` drains buffered units. Stateful
  streaming encoders use `transcode_from` and `finish`.
- **`TranscodeProgress`**: reports relative input units read and output units
  written.
- **`TranscodeStatus`**: distinguishes complete conversion from `NeedInput` and
  `NeedOutput` stops.
- **`TranscodeFailure` / `CapacityError` / `TranscodeContractError`**: report
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
│  TranscodeDecodeInput / TranscodeEncodeOutput  (I/O bridges; requires io) │
├────────────────────────────────────────────────────────────────┤
│  TranscodeXxxEngine + TranscodeXxxHooks       (policy + loop)  │
│  CodecTranscodeDecoder / Encoder / Converter  (strict bridges) │
├────────────────────────────────────────────────────────────────┤
│  Transcoder + TranscodeProgress + TranscodeStatus               │
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
| `TranscodeEncoder` | Encode logical values into caller-provided unit buffers | Charset or binary buffered encoder |
| `TranscodeDecoder` | Decode encoded units into caller-provided value buffers | Charset or binary buffered decoder |
| `TranscodeConverter` | Convert encoded units between representations | Charset or binary buffered converter |

| Type | Purpose |
|------|---------|
| `DecodeFailure<E>` | Low-level decode result for incomplete visible prefixes or invalid codec-domain input |
| `TranscodeFailure` | Framework failure for invalid indices, insufficient output, output-length overflow, incomplete input, or trailing input |
| `TranscodeDomainError<E>` | Codec or policy error tagged with reset, main, or finish phase context |
| `TranscodeEncodeError<E, V>` | Encode-side error combining framework, unencodable-value, and domain failures |
| `TranscodeDecodeError<E>` | Decode-side error combining framework and domain failures |
| `TranscodeConvertError<DE, EE, V>` | Converter error preserving decode-side, encode-side, unencodable-value, and framework failures |
| `CapacityError` | Capacity-planning error returned before allocating or writing output |
| `TranscodeContractError` | Error reported when a custom `Transcoder` returns inconsistent progress |

### Codec Adapters

| Type | Purpose |
|------|---------|
| `CodecValueEncoder<C>` | Allocate owned `Vec<C::Unit>` output for one borrowed `C::Value` by using `C: Codec` without requiring `C::Value: Clone` |
| `CodecValueDecoder<C>` | Decode exactly one borrowed `[C::Unit]` slice into `C::Value` by using `C: Codec` |
| `CodecTranscodeEncoder<C>` | Encode `C::Value` slices into caller-provided `C::Unit` buffers by using `C: Codec` |
| `CodecTranscodeDecoder<C>` | Strictly decode `C::Unit` slices into caller-provided `C::Value` buffers by using `C: Codec` |
| `CodecTranscodeConverter<D, E>` | Decode `D::Unit` source units and encode `E::Unit` target units with `E::Value = D::Value` |

### I/O Adapters

| Type | Purpose |
|------|---------|
| `TranscodeDecodeInput<I>` *(requires `io`)* | Decode units from a `qubit_io::Input` by passing a caller-owned streaming decoder to `transcode_into` and `finish_transcode_into` |
| `TranscodeEncodeOutput<O>` *(requires `io`)* | Own a `qubit_io::Output`; ordinary `flush` drains buffered units. Stateful streaming encoders use `transcode_from` and `finish` |

### Encoder Hooks And Engines

| Type | Purpose |
|------|---------|
| `TranscodeEncodeEngine<C, H>` | Reusable buffered encoder engine backed by a low-level `Codec` and policy hooks |
| `TranscodeEncodeHooks<C>` | Hook contract for unencodable-value policy, preparing for reset, and finalizing encoded output |
| `TranscodeEncodeError<E, V>` / `TranscodeEncodeErrorOf<C>` | Encode-side error and its codec-derived alias |
| `EncodeUnencodableAction<Value>` | Policy action returned for values outside the codec's encodable domain |
| `EncodeOutcome` | Per-value engine outcome: consumed with written output, or needs more output without consuming |
| `EncodeContext<'a, Value, Unit>` | Input value, input index, output slice, and cursor used by encode engine helpers |

### Decoder Hooks And Engines

| Type | Purpose |
|------|---------|
| `TranscodeDecodeEngine<C, H>` | Reusable buffered decoder engine backed by a low-level `Codec` and policy hooks |
| `TranscodeDecodeHooks<C>` | Hook contract for invalid-input decode policy |
| `TranscodeDecodeError<E>` / `TranscodeDecodeErrorOf<C>` | Decode-side error and its codec-derived alias |
| `DecodeContext` | Context passed to decode policy hooks |
| `DecodeInvalidAction<Value>` | Invalid-input policy action: skip input or emit a replacement value |

### Converter Engines

| Type | Purpose |
|------|---------|
| `TranscodeConvertEngine<D, E, DH, EH>` | Reusable unit-to-unit converter that decodes with `D`, encodes with `E`, and applies decode/encode hooks |
| `TranscodeConvertError<DE, EE, V>` / `TranscodeConvertErrorOf<D, E>` | Converter error and its codec-derived alias |

### `Transcoder` Operations

| Method | Description |
|--------|-------------|
| `max_transcode_output_len(input_len)` | Return a streaming-phase upper bound valid for every reachable transient state |
| `max_total_output_len(input_len)` | Return the full `reset -> transcode -> finish` output upper bound |
| `max_reset_output_len()` | Return a reset-output upper bound valid for every reachable transient state |
| `max_finish_output_len()` | Return a finish-output upper bound valid for every reachable transient state |
| `reset()` | Reset retained stream state while keeping configuration |
| `transcode(input, input_index, output, output_index)` | Convert input units into output units |
| `transcode_complete_into(input, output)` | Run one complete `reset -> transcode -> finish` stream from the start of the supplied slices |
| `finish(output, output_index)` | Finish internally retained output such as digests or trailers |

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
- Stateful owned one-value callers should use `CodecValueEncoder<C>` and
  `CodecValueDecoder<C>`. Callers that need caller-provided buffers should use
  the streaming transcoder adapters so reset, main value, and finish lifecycle
  handling stays centralized.
- Directional adapters expose `TranscodeEncodeError`, `TranscodeDecodeError`,
  or `TranscodeConvertError`. Each keeps framework failures separate from
  phase-tagged codec and policy domain errors.
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
| `NativeEndian` | Native-endian type marker |

## Crate Boundary

`qubit-codec` does not contain concrete binary formats, character sets, or
percent/Base64/hex codecs. When the `io` feature is enabled, its I/O-facing
surface is limited to low-level `qubit_io::Input` / `qubit_io::Output` bridge
types used by downstream stream crates. Keep `std::io::Read` /
`std::io::Write` extension traits and concrete reader/writer adapters in domain
crates so downstream users can depend on only the layers they need.

## Performance Considerations

The streaming traits and engine main loops operate on caller-provided buffers.
`BigEndian` and `LittleEndian` are zero-sized, and `ByteOrder` is a small
copyable enum. Owned value adapters such as `CodecValueEncoder` allocate their
`Vec<Unit>` result, and codec lifecycle output or an I/O decode window that
outgrows the base buffer may use temporary scratch storage. Concrete downstream
codecs may have additional allocation behavior.

## Dependencies

Runtime dependencies are intentionally small:

- `thiserror` provides public error type implementations.
- With the `io` feature, `qubit-io` provides `BufferedInput` and
  `BufferedOutput` used by `TranscodeDecodeInput` and `TranscodeEncodeOutput`.

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

## Testing

```bash
# Run tests with the default empty feature set
cargo test

# Run tests with the optional I/O bridge enabled
cargo test --all-features

# Project CI checks
./ci-check.sh

# Check code coverage
./coverage.sh
```

## License

Copyright (c) 2025 - 2026. Haixing Hu. All rights reserved.

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for the
full license text.

## Contributing

Contributions are welcome. Please follow the Rust API guidelines, keep public
API documentation and tests current, and run `./align-ci.sh` to format code and
`./ci-check.sh` to satisfy CI requirements before submitting a pull request.

## Author

**Haixing Hu** - *Qubit Co. Ltd.*

Repository: [https://github.com/qubit-ltd/rs-codec](https://github.com/qubit-ltd/rs-codec)

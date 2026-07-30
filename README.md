# Qubit Codec

[![Rust CI](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-codec/coverage-badge.json)](https://qubit-ltd.github.io/rs-codec/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-codec.svg?color=blue)](https://crates.io/crates/qubit-codec)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-codec` is the domain-neutral foundation for Rust codecs that need a
clear boundary between one-value encoding, owned-value convenience APIs, and
caller-buffered streaming conversion. It is for codec authors and adapter
authors; concrete binary formats, character sets, and text formats live in
sibling crates.

## Why This Crate Exists

A format crate often starts with a small `Codec` for one value—for example, a
fixed-width scalar or one character—but later needs an owned convenience API,
a buffered stream adapter, or malformed-input policy. Reimplementing cursor
handling and EOF behavior at each layer makes the contracts diverge.

This crate supplies the shared contracts and adapters so a format crate can
keep its format rules local while reusing the checked conversion machinery.

## Installation

```toml
[dependencies]
qubit-codec = "0.11"
```

The default feature set is empty. Enable `io` only when using the
`qubit-io` bridge types:

```toml
[dependencies]
qubit-codec = { version = "0.11", features = ["io"] }
```

## Quick Start

For a whole-value operation whose useful unit is the complete input, implement
`ValueEncoder` directly. This is preferable to forcing a `Codec` abstraction
onto a format with no meaningful single-value quantum.

```rust
use qubit_codec::ValueEncoder;

struct PrefixEncoder;

impl ValueEncoder<str> for PrefixEncoder {
    type Output = String;
    type Error = core::convert::Infallible;

    fn encode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(format!("encoded:{input}"))
    }
}

let mut encoder = PrefixEncoder;
let output = encoder.encode("codec")?;
assert_eq!("encoded:codec", output);

# Ok::<(), core::convert::Infallible>(())
```

When the format does have a local value boundary, implement `Codec` and then
select a supplied adapter: `CodecValueEncoder` or `CodecValueDecoder` for
owned single-value output; `CodecTranscodeEncoder`, `CodecTranscodeDecoder`,
or `CodecTranscodeConverter` for strict caller-buffered conversion.

If a codec emits values from `decode_reset` or `decode_finish`, strict
single-value decode intentionally rejects it. Use `CodecValueDecoder`'s
`decode_lifecycle` or `decode_lifecycle_with_scratch` instead; see the
[lifecycle-aware decode example](examples/decode_lifecycle.rs).

## What It Provides

| Need | Public API |
| --- | --- |
| One value or codec quantum over caller buffers | `Codec` and `DecodeFailure` |
| Owned whole-value conversion | `ValueEncoder`, `ValueDecoder`, `CodecValueEncoder`, `CodecValueDecoder` |
| Strict buffered encode, decode, or unit conversion | `TranscodeEncoder`, `TranscodeDecoder`, `TranscodeConverter`, and the `CodecTranscode*` adapters |
| Policy-aware buffered conversion | `engine::TranscodeEncodeEngine`, `engine::TranscodeDecodeEngine`, `engine::TranscodeConvertEngine`, and their hooks |
| Caller-managed streaming lifecycle | `Transcoder`, `TranscodeProgress`, and `TranscodeStatus` |
| Shared byte-order metadata | `ByteOrder`, `ByteOrderSpec`, `BigEndian`, `LittleEndian`, and `NativeEndian` |
| `qubit-io` buffered bridges | `TranscodeDecodeInput`, `TranscodeEncodeOutput`, and the partial-I/O `AsyncTranscodeDecodeInput` / `AsyncTranscodeEncodeOutput` with feature `io` |

For a streaming converter, the lifecycle is explicit:

```text
reset(output) -> transcode(...) repeatedly -> handle any EOF tail -> finish(output)
```

`Complete` means that a `transcode` call consumed all visible input from its
requested index. `NeedInput` leaves the incomplete tail with the caller for a
retry or an explicit EOF decision; `NeedOutput` means the output buffer must be
extended or drained before conversion continues.

The async bridges expose this same progress directly: every `poll_transcode`
or `transcode_async` call performs at most one transcoder invocation after
any required I/O preparation. A returned `TranscodeProgress` (or
`AsyncTranscodeDecodeStep::Progress`) is already committed and is never
followed by another await in that call. Advance the caller cursor by its
reported count; EOF is the explicit `AsyncTranscodeDecodeStep::EndOfInput`.

## Boundaries and Guarantees

- This crate does not implement concrete binary formats, character sets,
  Base64, hex, percent encoding, or `std::io` reader/writer extensions.
- `Codec::decode` distinguishes an incomplete visible prefix from invalid
  codec-domain input through `DecodeFailure`; an incomplete prefix is not an
  EOF decision.
- `Codec::encode_len` must exactly match a successful `Codec::encode` call for
  the same value and codec state, including intentional zero-output buffering.
- `Transcoder` capacity bounds must cover every reachable transient state, not
  only the current one. `finish` does not receive a caller-owned incomplete
  input tail.
- Owned adapters may allocate their returned `Vec` values. Streaming APIs use
  caller-provided buffers; feature `io` adds `qubit-io` bridge types.

## Learn More

- [User guide](doc/user_guide.md): abstraction selection, lifecycle rules,
  errors, and implementation checklist.
- [中文用户手册](doc/user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-codec)
- [中文 README](README.zh_CN.md)

## Testing

```bash
# Run tests with the default feature set
cargo test

# Run tests with all declared features
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

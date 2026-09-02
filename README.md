# Qubit Codec

[![Rust CI](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-codec/coverage-badge.json)](https://qubit-ltd.github.io/rs-codec/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-codec.svg?color=blue)](https://crates.io/crates/qubit-codec)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-codec` gives Rust codec and adapter authors one set of contracts for
single-value codecs, owned convenience APIs, and caller-buffered streaming.
Format crates keep their binary, text, or character-set rules local while this
crate supplies checked capacity handling, explicit lifecycle semantics, and
policy-ready conversion loops.

## Installation

```toml
[dependencies]
qubit-codec = "0.14"
```

The default feature set is empty. Enable `io` only for the `qubit-io` buffered
bridges, and enable `registry` for global value codec registration and the
`register_value_codec!` macro. The features are independent:

```toml
[dependencies]
qubit-codec = { version = "0.14", features = ["io"] }
```

```toml
[dependencies]
qubit-codec = { version = "0.14", features = ["registry"] }
```

The minimum supported Rust version is 1.94.

## Quick Start

Start by choosing the smallest public contract that matches the format:

| Format requirement | Start with |
| --- | --- |
| One meaningful logical value maps to encoded units | Implement `Codec` |
| Only the complete input is a meaningful operation | Implement `ValueEncoder` / `ValueDecoder` directly |
| An existing `Codec` needs owned output | Use `CodecValueEncoder` / `CodecValueDecoder` |
| An existing `Codec` needs a strict buffered stream | Use a `CodecTranscode*` adapter |
| Invalid or unencodable values need policy | Use a transcode engine with hooks |
| EOF or framing behavior does not fit the supplied engines | Implement `Transcoder` |

For example, a crate that owns a fixed-width big-endian `u16` codec implements
`Codec` once. It can then expose an owned operation without duplicating bounds
or lifecycle handling:

```rust
use core::{convert::Infallible, num::NonZeroUsize};
use qubit_codec::{Codec, CodecValueEncoder, DecodeFailure, ValueEncoder};

#[derive(Default)]
struct U16BeCodec;

impl Codec for U16BeCodec {
    type Value = u16;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u16, NonZeroUsize), DecodeFailure<Infallible>> {
        debug_assert!(input_index + 2 <= input.len());
        let value = u16::from_be_bytes([
            input[input_index],
            input[input_index + 1],
        ]);
        Ok((value, NonZeroUsize::new(2).expect("two is non-zero")))
    }

    unsafe fn encode(
        &mut self,
        value: &u16,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Infallible> {
        debug_assert!(output_index + 2 <= output.len());
        output[output_index..output_index + 2]
            .copy_from_slice(&value.to_be_bytes());
        Ok(2)
    }
}

let mut encoder = CodecValueEncoder::new(U16BeCodec);
let bytes = encoder.encode(&0x1234).expect("encoding is infallible");
assert_eq!(vec![0x12, 0x34], bytes);
```

The [user guide](doc/user_guide.md) continues this exact scenario with owned
decode, caller-buffered streaming, incomplete input, lifecycle output, and
policy hooks.

## Why This Project Exists

A format crate often begins with one value-to-unit operation and later grows
owned helpers, streaming adapters, malformed-input policy, and I/O integration.
If every layer reimplements indices, capacity, EOF, and reset/finish behavior,
the same format acquires incompatible contracts. `qubit-codec` keeps those
mechanics shared while leaving domain rules in the format crate that owns them.

## What It Provides

| Need | Public API |
| --- | --- |
| Low-level value/unit contract | `Codec`, `DecodeFailure` |
| Owned whole-value conversion | `ValueEncoder`, `ValueDecoder`, `CodecValueEncoder`, `CodecValueDecoder` |
| Safely erased whole-value lookup with `registry` | `ValueCodecDescriptor`, `ValueCodecRegistry`, `register_value_codec!` |
| Strict caller-buffered conversion | `Transcoder`, `CodecTranscodeEncoder`, `CodecTranscodeDecoder`, `CodecTranscodeConverter` |
| Policy-aware conversion | `engine::TranscodeEncodeEngine`, `engine::TranscodeDecodeEngine`, `engine::TranscodeConvertEngine`, and hooks including `DecodeIncompleteAction` |
| Progress and backpressure | `TranscodeProgress`, `TranscodeStatus` |
| Runtime or static byte order | `ByteOrder`, `ByteOrderSpec`, `BigEndian`, `LittleEndian`, `NativeEndian` |
| `qubit-io` bridges with `io` | Sync and partial-I/O async transcode input/output adapters |

## Boundaries and Guarantees

- The crate does not implement concrete binary formats, character sets,
  Base64, hex, percent encoding, or `std::io` extensions.
- `Codec::decode` separates an incomplete visible prefix from invalid domain
  input through `DecodeFailure`; an incomplete prefix is not an EOF decision.
- `Codec::encode_len` must equal the units written by the following successful
  `Codec::encode` in the same state, including intentional zero-output
  buffering.
- A `Transcoder` follows `reset -> transcode/transcode_eof -> finish`.
  `NeedInput` leaves its tail with the caller; `NeedOutput` requires more
  destination capacity; `Complete` means all visible input was consumed.
- `TranscodeDecodeHooks::handle_incomplete_decode` lets decode engines choose
  `Reject`, `Skip`, or `Emit` for an incomplete tail only after EOF is known.
- Capacity bounds must cover every reachable transient state. Owned adapters
  may allocate; streaming APIs use caller-provided buffers.
- With `registry`, `ValueCodecDescriptor` erases only a bidirectional string
  codec whose value type is fixed at registration. Registry lookup is by
  validated stable ID; execution checks the erased input type before invoking
  user code.

## Learn More

- [User guide](doc/user_guide.md): implement a codec, expose adapters, drive a
  stream, and diagnose failures.
- [中文用户手册](doc/user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-codec)
- [Lifecycle-aware decode example](examples/decode_lifecycle.rs)
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

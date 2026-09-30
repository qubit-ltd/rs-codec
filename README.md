# Qubit Codec (`rs-codec`)

[![Rust CI](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml/badge.svg)](https://github.com/qubit-ltd/rs-codec/actions/workflows/ci.yml)
[![Coverage](https://img.shields.io/endpoint?url=https://qubit-ltd.github.io/rs-codec/coverage-badge.json)](https://qubit-ltd.github.io/rs-codec/coverage/)
[![Crates.io](https://img.shields.io/crates/v/qubit-codec.svg?color=blue)](https://crates.io/crates/qubit-codec)
[![Rust](https://img.shields.io/badge/rust-1.94+-blue.svg?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![中文文档](https://img.shields.io/badge/文档-中文版-blue.svg)](README.zh_CN.md)

`qubit-codec` addresses a recurring problem in format and adapter crates: once a
team defines how one logical value maps to encoded units, it still needs owned
one-shot APIs, caller-buffered streaming, and optional policy hooks. Rebuilding
indices, capacity bounds, progress, and reset/finish lifecycle in each layer
produces incompatible public surfaces for the same representation rules. This
crate centralizes those mechanics while the format crate keeps domain encoding
and errors. It does not ship concrete binary formats, character sets, Base64,
hex, percent encoding, or general-purpose `std::io` extensions.

## A binary format crate example

A protocol crate must encode `0x1234` as `[0x12, 0x34]`, decode the reverse,
treat a lone input byte as incomplete instead of entering the unsafe decode
entry, and encode a batch of values into a caller-owned buffer—all from one
`Codec` implementation. Callers that only need a single owned conversion use
`CodecValueEncoder` / `CodecValueDecoder`; callers that own the destination
buffer use `CodecTranscodeEncoder` without duplicating bound arithmetic.

## Installation

```toml
[dependencies]
qubit-codec = "0.16"
```

The default feature set is empty. Enable `io` only for the `qubit-io` buffered
bridges, and enable `registry` for global value codec registration and the
`register_value_string_codec!` / `register_value_bytes_codec!` macros. The features are independent:

```toml
[dependencies]
qubit-io = "0.17"
qubit-codec = { version = "0.16", features = ["io"] }
```

```toml
[dependencies]
qubit-codec = { version = "0.16", features = ["registry"] }
```

## Quick start

Implement the representation rule once, then expose owned encoding:

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

The [lifecycle-aware decode example](examples/decode_lifecycle.rs) shows when
`CodecValueDecoder::decode_lifecycle` is required instead of strict one-value
decode. The [user guide](doc/user_guide.md) continues this fixed-width scenario
with owned decode, incomplete input, caller-buffered streaming, policy hooks,
and optional `qubit-io` integration.

The same `U16BeCodec` can back both owned helpers and batch encoding into a
caller buffer:

```rust
use qubit_codec::{
    CodecValueDecoder, CodecValueEncoder, CodecTranscodeEncoder, Transcoder, ValueDecoder,
    ValueEncoder,
};

fn encode_u16(value: u16) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut encoder = CodecValueEncoder::new(U16BeCodec);
    Ok(encoder.encode(&value)?)
}

fn decode_u16(bytes: &[u8]) -> Result<u16, Box<dyn std::error::Error>> {
    let mut decoder = CodecValueDecoder::new(U16BeCodec);
    Ok(decoder.decode(bytes)?)
}

fn encode_batch(values: &[u16]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut encoder = CodecTranscodeEncoder::new(U16BeCodec);
    let capacity = encoder
        .max_total_output_len(values.len())
        .expect("capacity arithmetic should not overflow");
    let mut output = vec![0_u8; capacity];
    let written = encoder.transcode_complete_into(values, &mut output)?;
    output.truncate(written);
    Ok(output)
}

assert_eq!(encode_u16(0x1234)?, vec![0x12, 0x34]);
assert_eq!(decode_u16(&[0x12, 0x34])?, 0x1234);
assert_eq!(
    encode_batch(&[0x1234, 0xabcd])?,
    vec![0x12, 0x34, 0xab, 0xcd],
);
```

Checked adapters establish index and capacity preconditions before the unsafe
`Codec` entry points; a one-byte input therefore surfaces as incomplete or
transcode failure rather than undefined behavior. See the [user guide](doc/user_guide.md)
for incremental `transcode`, EOF handling, and engines with hooks.

## What it provides

- Core value/unit contract: `Codec`, `DecodeFailure`, and explicit encode/decode
  bounds for checked adapters.
- Owned whole-value facades: `ValueEncoder`, `ValueDecoder`,
  `CodecValueEncoder`, `CodecValueDecoder`, including lifecycle-aware decode when
  reset or finish values are declared.
- Strict caller-buffered conversion: `Transcoder`, `CodecTranscodeEncoder`,
  `CodecTranscodeDecoder`, `CodecTranscodeConverter`, with `TranscodeProgress`
  and `TranscodeStatus` for backpressure.
- Policy-aware engines: `engine::TranscodeEncodeEngine`,
  `engine::TranscodeDecodeEngine`, `engine::TranscodeConvertEngine`, and hooks
  such as `DecodeIncompleteAction` for incomplete tails after EOF is known.
- Byte order helpers: `ByteOrder`, `ByteOrderSpec`, `BigEndian`, `LittleEndian`,
  `NativeEndian`.
- With `registry`: validated stable IDs, `ValueStringCodecDescriptor`,
  `ValueBytesCodecDescriptor`, `ValueStringCodecRegistry`, `ValueBytesCodecRegistry`,
  and `register_value_string_codec!` / `register_value_bytes_codec!` for type-checked
  erased whole-value codecs (string or bytes wire) fixed at registration time.
- With `io`: sync and partial-I/O async transcode input/output bridges over
  `qubit-io`.

The crate does not implement domain formats or character-set tables on its own.
`Codec::decode` separates incomplete visible prefixes from invalid domain input
through `DecodeFailure`; an incomplete prefix is not an EOF decision by itself.
In the same codec state, `Codec::encode_len` must match a successful
`Codec::encode`, including intentional zero-output buffering. A `Transcoder`
follows `reset -> transcode/transcode_eof -> finish`; `NeedInput` leaves its tail
with the caller, `NeedOutput` requires more destination capacity, and `Complete`
means all visible input from the call's start index was consumed. Capacity bounds
must cover every reachable transient state; owned adapters may allocate while
streaming APIs use caller-provided buffers. Registry lookup is by validated ID
and execution checks the erased input type before invoking user code.

## Learn more

- [User guide](doc/user_guide.md)
- [API reference](https://docs.rs/qubit-codec)
- [Lifecycle-aware decode example](examples/decode_lifecycle.rs)
- [中文 README](README.zh_CN.md) · [中文用户手册](doc/user_guide.zh_CN.md)

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

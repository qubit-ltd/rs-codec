# Qubit Codec User Guide

[中文](user_guide.zh_CN.md) · [README](../README.md) · [API documentation](https://docs.rs/qubit-codec)

This guide covers `qubit-codec` 0.11 and Rust 1.94 or later. It is written for
authors of codec and adapter crates—not for application developers looking for
a concrete file format or character set implementation.

## Purpose and Audience

Use `qubit-codec` when a format crate needs one or more of these reusable
boundaries:

- a low-level contract for one logical value or codec quantum;
- an owned-output facade over that contract;
- a caller-buffered streaming adapter with explicit progress;
- policy hooks for invalid or unencodable input;
- buffered `qubit-io` integration.

The format crate continues to own representation rules and domain errors.
`qubit-codec` owns the shared mechanics: indices, capacity planning, progress,
reset/finish lifecycle, and the separation between incomplete and invalid
input.

## Conceptual Model

```text
format-owned rules
      |
    Codec --------------> CodecValueEncoder / CodecValueDecoder
      |                              owned output
      |
      +-----------------> CodecTranscodeEncoder / Decoder / Converter
      |                              strict buffered conversion
      |
      +-----------------> Transcode*Engine + hooks
                                     policy-aware conversion

ValueEncoder / ValueDecoder          whole-value formats without a useful
                                     single-value codec quantum

Transcoder                           custom streaming or EOF/framing behavior
```

Choose the smallest layer that preserves the format's real boundary:

| Requirement | Recommended API |
| --- | --- |
| One logical value maps to encoded units | Implement `Codec` |
| Only complete input has useful meaning | Implement `ValueEncoder` / `ValueDecoder` directly |
| Existing `Codec`, owned result | `CodecValueEncoder` / `CodecValueDecoder` |
| Existing `Codec`, strict stream | A `CodecTranscode*` adapter |
| Invalid or unencodable input needs replacement, skipping, or reporting | A transcode engine plus hooks |
| Streaming rules require custom EOF or framing decisions | Implement `Transcoder` |

A fixed-width integer or character usually has a useful `Codec` value
boundary. A formatted hex string, percent-encoded string, or C string literal
usually does not; a value-level implementation is clearer for those formats.

## Scenario: Publish One Fixed-Width Codec at Two Levels

Suppose a binary-format crate must encode a `u16` as two big-endian bytes. Its
success criteria are concrete:

1. `0x1234` encodes to `[0x12, 0x34]`.
2. `[0x12, 0x34]` decodes to `0x1234`.
3. One input byte is reported as incomplete rather than passed to the unsafe
   codec entry point.
4. The same codec can encode many values into a caller-owned buffer.

The crate implements the representation rule once:

```rust
use core::{convert::Infallible, num::NonZeroUsize};
use qubit_codec::{Codec, DecodeFailure};

#[derive(Clone, Copy, Debug, Default)]
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
```

The checked adapters now supply both publishing levels; the format crate does
not reimplement capacity or lifecycle code.

## Installation and Minimal Configuration

```toml
[dependencies]
qubit-codec = "0.11"
```

The default feature set is empty. The scenario above needs no feature. Enable
`io` only when using the `qubit-io` bridges:

```toml
[dependencies]
qubit-codec = { version = "0.11", features = ["io"] }
```

## Core Workflow

### 1. Declare exact `Codec` bounds

The three required constants are public safety and capacity contracts, not
performance hints:

| Contract | Meaning |
| --- | --- |
| `MIN_UNITS_PER_VALUE` | Smallest readable input that could hold one decoded value; it must be non-zero. |
| `MAX_DECODE_UNITS_PER_VALUE` | Largest successful decode consumption or incomplete retry requirement; it must be non-zero and at least the minimum. |
| `MAX_ENCODE_UNITS_PER_VALUE` | Value-independent upper bound for main-phase encode output; zero is allowed only for deliberate buffering. |

The default `encode_len` returns `MAX_ENCODE_UNITS_PER_VALUE`, which is exact
for the fixed-width scenario. A variable-width or stateful codec must override
it. For the same value and codec state, a successful `encode` must write
exactly the reported length.

Override `can_encode_value` when `Value` includes values outside the encoded
domain. Checked encoders call it before `encode_len` and the unsafe `encode`.

### 2. Keep unsafe entry points narrow

Checked adapters establish the documented index and capacity preconditions
before entering `Codec::encode` or `Codec::decode`. The implementation must
still:

- read and write only within those preconditions;
- return a non-zero successful decode count no larger than the decode bound;
- leave state consistent on errors;
- return `DecodeFailure::Incomplete` for a valid open-stream prefix that needs
  more units, and `DecodeFailure::Invalid` for malformed domain input;
- keep reset, main, and finish output within their declared bounds.

Use `debug_assert!` at the entry point to make the assumed ranges visible, as
the scenario does.

### 3. Expose owned one-value operations

`CodecValueEncoder` and `CodecValueDecoder` run a complete codec lifecycle and
return owned output:

```rust
use qubit_codec::{CodecValueDecoder, CodecValueEncoder, ValueEncoder};

let mut encoder = CodecValueEncoder::new(U16BeCodec);
let encoded = encoder.encode(&0x1234).expect("encoding is infallible");
assert_eq!(vec![0x12, 0x34], encoded);

let mut decoder = CodecValueDecoder::new(U16BeCodec);
let decoded = decoder.decode(&encoded).expect("input contains one u16");
assert_eq!(0x1234, decoded);
```

Strict one-value decode requires exactly one main value. Extra units produce
`TranscodeFailure::TrailingInput`. A codec that declares values from
`decode_reset` or `decode_finish` must instead use `decode_lifecycle` or
`decode_lifecycle_with_scratch`; the runnable
[lifecycle example](../examples/decode_lifecycle.rs) preserves reset, main, and
finish output separately.

### 4. Expose caller-buffered conversion

A `CodecTranscodeEncoder` applies the same codec to a sequence of values. The
one-shot helper sizes and executes the full lifecycle while the caller owns the
buffer:

```rust
use qubit_codec::{CodecTranscodeEncoder, Transcoder};

let values = [0x1234, 0xabcd];
let mut encoder = CodecTranscodeEncoder::new(U16BeCodec);
let capacity = encoder
    .max_total_output_len(values.len())
    .expect("capacity arithmetic should not overflow");
let mut output = vec![0_u8; capacity];
let written = encoder
    .transcode_complete_into(&values, &mut output)
    .expect("encoding is infallible");
output.truncate(written);

assert_eq!(vec![0x12, 0x34, 0xab, 0xcd], output);
```

Use `CodecTranscodeDecoder` for units-to-values and
`CodecTranscodeConverter` for a strict decode-plus-encode pipeline. Choose an
engine only when a side needs a real policy decision.

### 5. Drive an incremental stream correctly

The explicit lifecycle is:

```text
size reset output -> reset
                       |
                       v
preserve input tail <- transcode/transcode_eof -> drain or extend output
                       |
                       v
                 size finish output -> finish
```

`TranscodeProgress::read()` and `written()` are relative to the indices passed
to that call. Advance both cursors before retrying.

| Status | Meaning | Caller action |
| --- | --- | --- |
| `Complete` | All visible input from `input_index` was consumed. | Supply another segment, or finish at EOF. |
| `NeedInput` | The incomplete tail was not consumed. | Preserve the tail and refill; at EOF use the format's explicit EOF policy. |
| `NeedOutput` | Conversion stopped before exceeding output capacity. | Drain or extend output and continue from the reported progress. |

Call `transcode_eof` only after the caller knows no more source units will
arrive. The default converts a remaining `NeedInput` into
`TranscodeFailure::IncompleteInput`. `finish` receives no source tail and
cannot reinterpret it.

Built-in transcode engines start uninitialized. The first successful operation
must be `reset`; after a successful `finish`, call `reset` before reuse. A
failed lifecycle operation may poison the instance until a successful reset.

## Advanced Usage

### Lifecycle output

Stateless codecs use zero reset and finish bounds. A stateful encoder that
writes a header or trailer declares `MAX_ENCODE_RESET_UNITS` or
`MAX_ENCODE_FINISH_UNITS`; a decoder that emits lifecycle values declares
`MAX_DECODE_RESET_VALUES` or `MAX_DECODE_FINISH_VALUES`. Bounds must cover
every reachable transient state, not only the current one.

### Policy hooks

Strict `CodecTranscode*` adapters return domain failures. When the format needs
replacement, skipping, counting, or phase-specific reporting, keep the shared
loop and implement hooks:

- `TranscodeEncodeEngine` with `TranscodeEncodeHooks` handles unencodable
  values and encode reset/finish policy.
- `TranscodeDecodeEngine` with `TranscodeDecodeHooks` handles invalid input.
- `TranscodeConvertEngine` composes decode and encode engines with both hook
  sets.

Use a custom `Transcoder` for delayed framing, bespoke stream state, or EOF
behavior that cannot be expressed by a codec plus hooks.

### Byte order and I/O

Use `ByteOrder` in runtime configuration. Use `ByteOrderSpec` with
`BigEndian`, `LittleEndian`, or `NativeEndian` when static selection is useful.
These types describe byte-order policy; they do not implement a concrete
integer codec.

With feature `io`, `TranscodeDecodeInput` and `TranscodeEncodeOutput` bridge
buffered `qubit-io` traits. `AsyncTranscodeDecodeInput` and
`AsyncTranscodeEncodeOutput` support partial I/O. Each async call performs at
most one transcoder invocation after required I/O preparation; returned
progress is already committed. Decoder EOF is the explicit
`AsyncTranscodeDecodeStep::EndOfInput`.

## Errors and Diagnostics

| Error | Boundary | Recovery |
| --- | --- | --- |
| `DecodeFailure::Incomplete` | Open-stream codec input is a valid prefix but too short. | Preserve the tail and retry, or make an explicit EOF decision. |
| `DecodeFailure::Invalid` | Units are malformed, non-canonical, or unmappable. | Apply domain policy or return the codec error. |
| `TranscodeFailure` | Indices, capacity, complete-input shape, allocation, or lifecycle usage is invalid. | Correct caller state; inspect the structured variant. |
| `CapacityError` | Capacity arithmetic cannot produce a valid `usize` bound. | Reject the planned operation before allocating or writing. |
| `TranscodeDomainError<E>` | A codec or hook failed during reset, main, or finish. | Preserve its phase and domain source when reporting. |
| Directional transcode error | Encode, decode, or conversion failed. | Keep the direction; encode/conversion errors may retain an unencodable value. |
| `TranscodeContractError` | A custom transcoder returned inconsistent progress. | Fix the transcoder implementation; this is not recoverable input. |

For the scenario, the checked decoder rejects a one-byte complete input before
calling unsafe `decode`:

```rust
use qubit_codec::{CodecValueDecoder, TranscodeFailure};

let mut decoder = CodecValueDecoder::new(U16BeCodec);
let error = decoder.decode(&[0x12]).expect_err("one byte is incomplete");
assert!(matches!(
    error.failure_ref(),
    Some(TranscodeFailure::IncompleteInput {
        input_index: 0,
        required: 2,
        available: 1,
    })
));
```

Do not flatten incomplete and invalid input unless every downstream caller
genuinely takes the same action for both.

## Troubleshooting

| Symptom | Check in this order |
| --- | --- |
| `NeedInput` at file end | Confirm the tail was preserved; call `transcode_eof`; apply format-specific EOF rules before `finish`. |
| Repeated `NeedOutput` | Advance both progress counters; provide the reported capacity; verify custom bounds and counters. |
| Owned decode rejects otherwise valid input | Check for trailing units or declared decode reset/finish output; use lifecycle-aware decode when needed. |
| Capacity is rejected before conversion | Include reset, main, and finish bounds and check for arithmetic overflow. |
| `TranscodeBeforeReset` or `TranscodeAfterFinish` | Call `reset` before the first stream and before reuse. |
| A malformed-input replacement path is becoming a second loop | Move the decision into the appropriate engine hooks. |

## Limitations and Best Practices

- Keep concrete formats, character sets, and high-level reader/writer adapters
  in their domain crates.
- Keep unsafe codec methods small; test success, incomplete, invalid, boundary,
  and stateful lifecycle behavior through checked public adapters.
- Treat capacity methods as state-independent safety bounds over every
  reachable state, not estimates of typical output.
- Owned adapters allocate `Vec` output. Prefer caller-buffered APIs when
  allocation ownership matters.
- `NeedInput` is a streaming boundary signal, not final EOF.
- Enable `io` only for crates that use the `qubit-io` bridge types.

## Further Reading

- [README](../README.md)
- [中文用户手册](user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-codec)
- [Lifecycle-aware decode example](../examples/decode_lifecycle.rs)

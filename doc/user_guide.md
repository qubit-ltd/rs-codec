# Qubit Codec user guide

[Chinese user guide](user_guide.zh_CN.md) · [README](../README.md) · [API reference](https://docs.rs/qubit-codec)

This guide covers `qubit-codec` 0.16.0 on Rust 1.94 or later. It is for authors of codec and adapter crates who need shared capacity, lifecycle, and streaming mechanics while keeping format rules local. Application developers looking for a ready-made hex, Base64, or character-set implementation should use a domain crate instead. Reading through [Verify conversion results](#verify-conversion-results) is enough to ship a fixed-width `Codec` with owned and caller-buffered adapters. Later sections cover incremental streams, lifecycle decode, policy hooks, optional I/O bridges, and the `registry` feature.

## Contents

- [The problem it solves](#the-problem-it-solves)
- [Where to start](#where-to-start)
- [Publish a fixed-width codec](#publish-a-fixed-width-codec)
  - [Implement the representation once](#implement-the-representation-once)
  - [Declare the Codec bounds](#declare-the-codec-bounds)
  - [Expose owned one-value operations](#expose-owned-one-value-operations)
  - [Encode many values into a caller-owned buffer](#encode-many-values-into-a-caller-owned-buffer)
  - [Types used on this path](#types-used-on-this-path)
- [Verify conversion results](#verify-conversion-results)
  - [What success looks like](#what-success-looks-like)
  - [Incomplete input versus invalid input](#incomplete-input-versus-invalid-input)
- [Drive an incremental stream](#drive-an-incremental-stream)
- [Handle lifecycle decode output](#handle-lifecycle-decode-output)
- [Apply policy for malformed input or EOF](#apply-policy-for-malformed-input-or-eof)
- [Choose byte-order helpers](#choose-byte-order-helpers)
- [Connect buffered I/O](#connect-buffered-io)
- [Register codecs for runtime lookup](#register-codecs-for-runtime-lookup)
- [Errors, diagnostics, and troubleshooting](#errors-diagnostics-and-troubleshooting)
- [Boundaries and a practice checklist](#boundaries-and-a-practice-checklist)
- [Further reading](#further-reading)

## The problem it solves

Take a binary protocol crate. It must map a `u16` field to two big-endian bytes for on-the-wire messages, expose a simple `encode(&u16) -> Vec<u8>` for tests and one-shot callers, and later accept byte chunks from a socket without treating a one-byte prefix as corrupt data. If each layer reimplements index arithmetic, capacity planning, reset/finish lifecycle, and the distinction between “need more bytes” and “bytes are wrong,” the same format accumulates incompatible contracts.

With `qubit-codec`, the format crate implements the representation rule once on `Codec`. Checked adapters supply owned output (`CodecValueEncoder` / `CodecValueDecoder`), strict caller-buffered conversion (`CodecTranscode*`), and optional policy engines with hooks. The crate does **not** ship concrete formats such as hex, percent encoding, or UTF-8 handling; those stay in domain crates. It also does **not** decide EOF for you: an incomplete prefix is reported as incomplete until the caller confirms end-of-input or applies a documented EOF policy.

## Where to start

1. [Publish a fixed-width codec](#publish-a-fixed-width-codec) walks through `U16BeCodec`, owned encode/decode, and batch encoding into a caller buffer.
2. [Verify conversion results](#verify-conversion-results) shows what success looks like and how incomplete input differs from invalid input. The basic integration path ends there.
3. Read on as needed: [Drive an incremental stream](#drive-an-incremental-stream), [Handle lifecycle decode output](#handle-lifecycle-decode-output), [Apply policy for malformed input or EOF](#apply-policy-for-malformed-input-or-eof), [Connect buffered I/O](#connect-buffered-io), or [Register codecs for runtime lookup](#register-codecs-for-runtime-lookup).

Optional features `io` and `registry` are independent; enable only what the crate uses.

## Publish a fixed-width codec

The success criteria for this scenario are concrete:

1. `0x1234` encodes to `[0x12, 0x34]`.
2. `[0x12, 0x34]` decodes to `0x1234`.
3. A one-byte input is rejected as incomplete before the unsafe `Codec::decode` entry runs.
4. The same codec encodes many values into one caller-owned buffer without reimplementing capacity math.

### Implement the representation once

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

Fixed-width integers and code units usually have a useful single-value `Codec` boundary. Formatted hex strings, percent-encoded text, or C string literals often do not; for those, implement `ValueEncoder` / `ValueDecoder` directly instead of forcing a `Codec` quantum.

### Declare the Codec bounds

The three required constants are public safety and capacity contracts, not performance hints:

| Contract | Meaning |
| --- | --- |
| `MIN_UNITS_PER_VALUE` | Smallest readable input that could hold one decoded value; must be non-zero. |
| `MAX_DECODE_UNITS_PER_VALUE` | Largest successful decode consumption or incomplete retry requirement; must be non-zero and at least the minimum. |
| `MAX_ENCODE_UNITS_PER_VALUE` | Value-independent upper bound for main-phase encode output; zero is allowed only for deliberate buffering. |

Default `encode_len` returns `MAX_ENCODE_UNITS_PER_VALUE`, which is exact for this scenario. Variable-width or stateful codecs must override it. For the same value and codec state, a successful `encode` must write exactly the length reported by `encode_len`. Override `can_encode_value` when `Value` includes values outside the encoded domain; checked encoders call it before `encode_len` and the unsafe `encode`.

Checked adapters establish index and capacity preconditions before calling unsafe `Codec` methods. Implementations must read and write only within those ranges, return a non-zero decode count no larger than the decode bound, keep state consistent on errors, and use `DecodeFailure::Incomplete` versus `DecodeFailure::Invalid` consistently. Use `debug_assert!` at the entry point to document the assumed ranges, as above.

### Expose owned one-value operations

`CodecValueEncoder` and `CodecValueDecoder` run a full codec lifecycle and return owned output:

```rust
use qubit_codec::{CodecValueDecoder, CodecValueEncoder, ValueEncoder};

let mut encoder = CodecValueEncoder::new(U16BeCodec);
let encoded = encoder.encode(&0x1234).expect("encoding is infallible");
assert_eq!(vec![0x12, 0x34], encoded);

let mut decoder = CodecValueDecoder::new(U16BeCodec);
let decoded = decoder.decode(&encoded).expect("input contains one u16");
assert_eq!(0x1234, decoded);
```

Strict one-value decode requires exactly one main value. Extra units after that value produce `TranscodeFailure::TrailingInput`.

### Encode many values into a caller-owned buffer

`CodecTranscodeEncoder` applies the same codec to a sequence. The one-shot helper sizes and executes the full lifecycle while the caller owns the buffer:

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

Use `CodecTranscodeDecoder` for units-to-values and `CodecTranscodeConverter` for a strict decode-then-encode pipeline. Choose a transcode engine with hooks only when a side needs a real policy decision rather than a plain domain error.

### Types used on this path

| Type | Role |
| --- | --- |
| `Codec` | Low-level value/unit contract; unsafe `encode` / `decode` with declared bounds. |
| `DecodeFailure` | Distinguishes incomplete visible prefix from invalid domain input inside `Codec::decode`. |
| `CodecValueEncoder` / `CodecValueDecoder` | Owned whole-value facade over one `Codec`. |
| `CodecTranscodeEncoder` | Caller-buffered encode path with reset/finish lifecycle and progress reporting. |
| `TranscodeFailure` | Caller misuse, shape errors, allocation limits, or lifecycle ordering problems at the adapter layer. |
| `Transcoder` | Trait implemented by `CodecTranscode*` adapters and custom streaming engines. |

Conceptually, format-owned rules sit on `Codec`; adapters add checked capacity, lifecycle, and progress without duplicating the representation rule:

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

## Verify conversion results

Owned encode/decode succeeding with the expected bytes and integers means the fixed-width scenario is integrated at the whole-value layer. Streaming and batch paths must also be checked separately when you expose them.

### What success looks like

After the steps above, these checks pass in tests or REPL:

```rust
use qubit_codec::{CodecValueDecoder, CodecValueEncoder, ValueEncoder};

let mut encoder = CodecValueEncoder::new(U16BeCodec);
assert_eq!(encoder.encode(&0x1234).unwrap(), vec![0x12, 0x34]);

let mut decoder = CodecValueDecoder::new(U16BeCodec);
assert_eq!(decoder.decode(&[0x12, 0x34]).unwrap(), 0x1234);
```

That confirms representation, owned adapters, and strict single-value shape. It does **not** prove incremental streaming, EOF policy, or lifecycle decode behavior; add tests when those APIs are public.

### Incomplete input versus invalid input

The checked decoder rejects a one-byte buffer before calling unsafe `decode`:

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

`DecodeFailure::Incomplete` inside `Codec::decode` means the visible prefix could still become valid with more units. `DecodeFailure::Invalid` means the domain rejects the input. Adapters map those distinctions into `TranscodeFailure` or domain errors; do not flatten incomplete and invalid unless every downstream caller truly handles both the same way.

The basic fixed-width path ends here. The following sections are optional: streaming lifecycle first, then lifecycle decode output, policy hooks, byte-order helpers, I/O bridges, and registry lookup.

## Drive an incremental stream

Explicit transcoder lifecycle:

```text
size reset output -> reset
                       |
                       v
preserve input tail <- transcode/transcode_eof -> drain or extend output
                       |
                       v
                 size finish output -> finish
```

`TranscodeProgress::read()` and `written()` are relative to the indices passed to that call. Advance both cursors before retrying.

| Status | Meaning | Caller action |
| --- | --- | --- |
| `Complete` | All visible input from `input_index` was consumed. | Supply another segment, or move to finish at EOF. |
| `NeedInput` | The incomplete tail was not consumed. | Preserve the tail and refill; at EOF use the format's explicit EOF policy (`transcode_eof` or hooks). |
| `NeedOutput` | Conversion stopped before exceeding output capacity. | Drain or extend output and continue from the reported progress. |

Call `transcode_eof` only after the caller knows no more source units will arrive. The default converts a remaining `NeedInput` into `TranscodeFailure::IncompleteInput`. `finish` receives no source tail and cannot reinterpret it.

Built-in transcode engines start uninitialized. The first successful operation must be `reset`; after a successful `finish`, call `reset` before reuse. A failed lifecycle operation may poison the instance until a successful `reset`.

## Handle lifecycle decode output

Stateless codecs use zero reset and finish bounds. A decoder that emits values from `decode_reset` or `decode_finish` must declare `MAX_DECODE_RESET_VALUES` or `MAX_DECODE_FINISH_VALUES`. Strict `CodecValueDecoder::decode` rejects inputs that emit lifecycle values; use `decode_lifecycle` or `decode_lifecycle_with_scratch` instead.

The runnable [lifecycle example](../examples/decode_lifecycle.rs) shows a one-byte codec that emits marker values when decode state opens and closes. After `decode_lifecycle`, inspect reset, main, and finish parts separately rather than collapsing them into one main value.

## Apply policy for malformed input or EOF

Strict `CodecTranscode*` adapters surface domain failures directly. When the format needs replacement, skipping, counting, or phase-specific reporting, keep the shared loop and implement hooks:

- `TranscodeEncodeEngine` with `TranscodeEncodeHooks` for unencodable values and encode reset/finish policy.
- `TranscodeDecodeEngine` with `TranscodeDecodeHooks` for invalid input and incomplete input after EOF.
- `TranscodeConvertEngine` composing both hook sets.

Call `transcode_eof` only after the input source confirms EOF. For `TranscodeDecodeEngine`, the hook's `handle_incomplete_decode` runs then. The default action is `Reject`, preserving a codec-domain incomplete error when available. Intentional recovery can use:

```rust
use qubit_codec::engine::DecodeIncompleteAction;

let skip = DecodeIncompleteAction::<char>::Skip;
let replacement = DecodeIncompleteAction::Emit { value: '\u{fffd}' };
```

`Skip` consumes the whole remaining tail without producing a value. `Emit` also consumes the tail and writes one replacement value, so the caller must provide an output slot. The hook receives `Some(source)` when the codec was invoked and reported incomplete input, and `None` when the tail is shorter than `Codec::MIN_UNITS_PER_VALUE`.

Use a custom `Transcoder` when framing, stream state, or EOF behavior cannot be expressed as a `Codec` plus hooks.

## Choose byte-order helpers

`ByteOrder` supports runtime configuration. `ByteOrderSpec` with `BigEndian`, `LittleEndian`, or `NativeEndian` supports static selection. These types describe byte-order policy; they do not implement a concrete integer codec. The `U16BeCodec` scenario encodes endianness in the representation rule itself.

## Connect buffered I/O

Enable feature `io` only when using the `qubit-io` bridges:

```toml
[dependencies]
qubit-io = "0.17"
qubit-codec = { version = "0.16", features = ["io"] }
```

`TranscodeDecodeInput` and `TranscodeEncodeOutput` integrate buffered `qubit-io` traits. `TranscodeDecodeInput::transcode` applies `transcode_eof` to a retained tail after a refill confirms EOF. For explicit stepwise control, use `TranscodeDecodeInput::transcode_eof_step`. The async counterpart reports `AsyncTranscodeDecodeStep::EndOfInput`; then call `AsyncTranscodeDecodeInput::transcode_eof_step` before `finish`.

## Register codecs for runtime lookup

Enable feature `registry` when several crates must discover bidirectional whole-value codecs (string or bytes wire) by stable ID at link time:

```toml
[dependencies]
qubit-codec = { version = "0.16", features = ["registry"] }
```

Implement paired `ValueEncoder` and `ValueDecoder` for a fixed value type, then register. String and bytes wires use different codec types, macros, and process-wide registries; the same `ValueCodecId` may appear in both.

String wire example:

```rust
use qubit_codec::{ValueDecoder, ValueEncoder, register_value_string_codec};

#[derive(Default)]
struct U32StringCodec;

impl ValueEncoder<u32> for U32StringCodec {
    type Output = String;
    type Error = std::io::Error;

    fn encode(&mut self, input: &u32) -> Result<Self::Output, Self::Error> {
        Ok(input.to_string())
    }
}

impl ValueDecoder<str> for U32StringCodec {
    type Output = u32;
    type Error = std::io::Error;

    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        input.parse().map_err(std::io::Error::other)
    }
}

register_value_string_codec!(id = "example.u32", codec = U32StringCodec, value = u32,);
```

Bytes whole-value wire example (`Vec<u8>` encode output, `&[u8]` decode input):

```rust
use qubit_codec::{ValueDecoder, ValueEncoder, register_value_bytes_codec};

#[derive(Default)]
struct U32BeBytesCodec;

impl ValueEncoder<u32> for U32BeBytesCodec {
    type Output = Vec<u8>;
    type Error = std::io::Error;

    fn encode(&mut self, input: &u32) -> Result<Self::Output, Self::Error> {
        Ok(input.to_be_bytes().to_vec())
    }
}

impl ValueDecoder<[u8]> for U32BeBytesCodec {
    type Output = u32;
    type Error = std::io::Error;

    fn decode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
        let bytes: [u8; 4] = input.try_into().map_err(|_| std::io::Error::other("expected four bytes"))?;
        Ok(u32::from_be_bytes(bytes))
    }
}

register_value_bytes_codec!(id = "example.u32", codec = U32BeBytesCodec, value = u32,);
```

`ValueStringCodecRegistry::global()` and `ValueBytesCodecRegistry::global()` (or each `try_global()` when you need the error) collect linked registrations once per process. Lookup is by validated `ValueCodecId`; execution checks the erased input type before invoking user code. Duplicate IDs within one registry are a registration error, not a silent override. This path erases only bidirectional whole-value codecs whose value type was fixed at registration; it does not replace `Codec` streaming adapters.

## Errors, diagnostics, and troubleshooting

| Error | Boundary | Recovery |
| --- | --- | --- |
| `DecodeFailure::Incomplete` | Open-stream codec input is a valid prefix but too short. | Preserve the tail and retry, or make an explicit EOF decision. |
| `DecodeFailure::Invalid` | Units are malformed, non-canonical, or unmappable. | Apply domain policy or return the codec error. |
| `TranscodeFailure` | Indices, capacity, complete-input shape, allocation, or lifecycle usage is invalid. | Correct caller state; inspect the structured variant. |
| `CapacityError` | Capacity arithmetic cannot produce a valid `usize` bound. | Reject the planned operation before allocating or writing. |
| `TranscodeDomainError<E>` | A codec or hook failed during reset, main, or finish. | Preserve phase and domain source when reporting. |
| `TranscodeContractError` | A custom transcoder returned inconsistent progress. | Fix the transcoder; not recoverable input. |
| `ValueCodecExecutionError` | Registry dispatch: type mismatch or encode/decode failure. | Match value type and ID; inspect domain error source. |

| Symptom | What to check |
| --- | --- |
| `NeedInput` at end of file | Preserve the tail; call `transcode_eof`; apply format EOF rules before `finish`. |
| Repeated `NeedOutput` | Advance both progress counters; provide reported capacity; verify custom bounds. |
| Owned decode rejects valid-looking input | Trailing units, or decode reset/finish output; use lifecycle-aware decode. |
| Capacity rejected before conversion | Include reset, main, and finish bounds; check for arithmetic overflow. |
| `TranscodeBeforeReset` or `TranscodeAfterFinish` | Call `reset` before the first stream and before reuse. |
| Replacement logic duplicates the transcode loop | Move decisions into engine hooks. |
| Registry lookup fails or executes wrong type | Confirm the matching `register_value_string_codec!` / `register_value_bytes_codec!` is linked, the ID is unique in that registry, and input type matches registration. |

## Boundaries and a practice checklist

- Concrete formats, character sets, and high-level reader/writer adapters belong in domain crates, not in `qubit-codec`.
- `NeedInput` is a streaming signal, not final EOF; only the caller or an explicit EOF hook decides what incomplete tail means at end of input.
- Capacity methods must cover every reachable transient state, not typical output size alone.
- Owned adapters may allocate `Vec` output; prefer caller-buffered APIs when allocation ownership matters.
- Test success, incomplete, invalid, boundary, and stateful lifecycle behavior through checked public adapters, not only unsafe `Codec` methods.
- Enable `io` and `registry` only in crates that use those integration paths.

## Further reading

- [README](../README.md) · [中文 README](../README.zh_CN.md) · [API reference](https://docs.rs/qubit-codec)
- [Lifecycle-aware decode example](../examples/decode_lifecycle.rs)

# Qubit Codec user guide

[Chinese user guide](user_guide.zh_CN.md) · [README](../README.md) · [API reference](https://docs.rs/qubit-codec)

This guide covers `qubit-codec` 0.17 on Rust 1.94 or later. It is for authors of codec and adapter crates who need shared capacity, lifecycle, and streaming mechanics while keeping format rules local. Application developers looking for a ready-made hex, Base64, or character-set implementation should use a domain crate instead. Reading through [Verify conversion results](#verify-conversion-results) is enough to ship a fixed-width `Codec` with owned and caller-buffered adapters. Later sections cover incremental streams, lifecycle decode, policy hooks, optional I/O bridges, and the `registry` feature.

## Contents

- [The problem it solves](#the-problem-it-solves)
- [Where to start](#where-to-start)
- [Publish a fixed-width codec](#publish-a-fixed-width-codec)
  - [Implement the representation once](#implement-the-representation-once)
  - [Declare the Codec bounds](#declare-the-codec-bounds)
  - [Expose owned one-value operations](#expose-owned-one-value-operations)
  - [Encode many values into a caller-owned buffer](#encode-many-values-into-a-caller-owned-buffer)
  - [Types used on this path](#types-used-on-this-path)
- [Use generic JSON value codecs](#use-generic-json-value-codecs)
- [Verify conversion results](#verify-conversion-results)
  - [What success looks like](#what-success-looks-like)
  - [Incomplete input versus invalid input](#incomplete-input-versus-invalid-input)
- [Drive an incremental stream](#drive-an-incremental-stream)
- [Handle lifecycle decode output](#handle-lifecycle-decode-output)
- [Apply policy for malformed input or EOF](#apply-policy-for-malformed-input-or-eof)
- [Choose byte-order helpers](#choose-byte-order-helpers)
- [Connect buffered I/O](#connect-buffered-io)
- [Register codecs for runtime lookup](#register-codecs-for-runtime-lookup)
  - [Build a local registry without global registration](#build-a-local-registry-without-global-registration)
  - [Migrate registry users from 0.16 to 0.17](#migrate-registry-users-from-016-to-017)
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

Optional features `json`, `io`, and `registry` are independent; enable only what the crate uses. `json` provides generic Serde JSON codecs for complete JSON string or byte inputs.

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

    unsafe fn encode(&mut self, value: &u16, output: &mut [u8], output_index: usize) -> Result<usize, Infallible> {
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

## Use generic JSON value codecs

Enable `json` for the built-in Serde-backed codecs. Encoding requires `T: serde::Serialize`; decoding requires `T: serde::de::DeserializeOwned` so the result owns its data. Both directions return `serde_json::Error`. The codecs encode a value to an owned JSON `String` or `Vec<u8>` and decode a complete JSON string or byte slice:

```rust
use qubit_codec::{JsonBytesValueCodec, JsonStringValueCodec, ValueDecoder, ValueEncoder};

let mut text = JsonStringValueCodec::<String>::new();
let encoded = text.encode(&String::from("hello")).unwrap();
assert_eq!(text.decode(&encoded).unwrap(), "hello");
let mut bytes = JsonBytesValueCodec::<String>::new();
let encoded = bytes.encode(&String::from("hello")).unwrap();
assert_eq!(bytes.decode(&encoded).unwrap(), "hello");
```

With both `json` and `registry`, the built-in registry descriptors cover `serde_json::Value`, under IDs `qubit.codec.json.string` and `qubit.codec.json.bytes`. A custom DTO can use the generic codecs directly, but registration requires its own unique ID and descriptor; generic DTO types are not automatically registered. See [registry lookup](#register-codecs-for-runtime-lookup).

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
| `Complete` | All visible input from `input_index` was consumed. | Supply another segment; after the source confirms EOF, complete `transcode_eof` (with an empty slice if needed) before `finish`. |
| `NeedInput` | The incomplete tail was not consumed. | Preserve the tail and refill; at EOF use the format's explicit EOF policy (`transcode_eof` or hooks). |
| `NeedOutput` | Conversion stopped before exceeding output capacity. | Drain or extend output and continue from the reported progress. |

Call `transcode_eof` only after the caller knows no more source units will arrive. The default converts a remaining `NeedInput` into `TranscodeFailure::IncompleteInput`. `finish` receives no source tail and cannot reinterpret it.

When the source confirms EOF, preserve any unread tail and keep advancing the `read()` and `written()` cursors from each `transcode_eof` result. If it returns `NeedOutput`, drain or extend the output and retry the EOF operation; do not return to ordinary `transcode`. Once the EOF operation returns `Complete`, allocate up to `max_finish_output_len` and call `finish`. If a call already completed the EOF operation, do not dispatch that same EOF event again. These operations do not provide general rollback: after an EOF error, do not replay the original input unless the transcoder defines a recovery protocol.

An I/O bridge may dispatch this EOF call with an empty input slice after all visible units have been consumed. The default implementation still reports `IncompleteInput` if the transcoder returns `NeedInput`, including when no unread units remain. A custom transcoder that uses EOF to drain buffered state or to finish an empty stream must override `transcode_eof` and return the appropriate progress for that case.

Built-in transcode engines start uninitialized. The first successful operation must be `reset`; after a successful `finish`, call `reset` before reuse. A failed lifecycle operation may poison the instance until a successful `reset`.

## Handle lifecycle decode output

Stateless codecs use zero reset and finish bounds. A decoder that emits values from `decode_reset` or `decode_finish` must declare `MAX_DECODE_RESET_VALUES` or `MAX_DECODE_FINISH_VALUES`. Strict `CodecValueDecoder::decode` rejects inputs that emit lifecycle values; use `decode_lifecycle` or `decode_lifecycle_with_scratch` instead.

The runnable [lifecycle example](../examples/decode_lifecycle.rs) shows a one-byte codec that emits marker values when decode state opens and closes. After `decode_lifecycle`, inspect reset, main, and finish parts separately rather than collapsing them into one main value.

## Apply policy for malformed input or EOF

Strict `CodecTranscode*` adapters surface domain failures directly. When the format needs replacement, skipping, counting, or phase-specific reporting, keep the shared loop and implement hooks:

- `TranscodeEncodeEngine` with `TranscodeEncodeHooks` for unencodable values and encode reset/finish policy.
- `TranscodeDecodeEngine` with `TranscodeDecodeHooks` for invalid input and incomplete input after EOF.
- `TranscodeConvertEngine` composing both hook sets.

Call `transcode_eof` only after the input source confirms EOF. Once at least `Codec::MIN_UNITS_PER_VALUE` units are available, the engine calls `Codec::decode_eof` first; the incomplete hook runs only if that call still reports incomplete input. The hook's `source` is `Some` only when the codec returned an incomplete failure carrying a source through `DecodeFailure::incomplete_with_source`. A codec call alone does not guarantee a source: `DecodeFailure::incomplete` carries none, and a tail shorter than `Codec::MIN_UNITS_PER_VALUE` reaches the hook without calling the codec. The default action is `Reject`, preserving a codec-domain incomplete error when available. Intentional recovery can use:

```rust
use qubit_codec::engine::DecodeIncompleteAction;

let skip = DecodeIncompleteAction::<char>::Skip;
let replacement = DecodeIncompleteAction::Emit { value: '\u{fffd}' };
```

`Skip` consumes the whole remaining tail without producing a value. `Emit` also consumes the tail and writes one replacement value, so the caller must provide an output slot. The complete example below uses `DecodeFailure::incomplete`, so the codec is called for the `~` prefix but the hook receives `None`. A codec using `DecodeFailure::incomplete_with_source` would provide `Some(source)` instead.

```rust
use core::convert::Infallible;
use core::num::NonZeroUsize;

use qubit_codec::{Codec, DecodeFailure, TranscodeDecodeErrorOf};
use qubit_codec::engine::{
    DecodeContext, DecodeIncompleteAction, TranscodeDecodeEngine, TranscodeDecodeHooks,
};

struct PrefixCodec;

impl Codec for PrefixCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
        if input[index] == b'~' && input.len() - index < 2 {
            return Err(DecodeFailure::incomplete(NonZeroUsize::new(2).unwrap()));
        }
        Ok((input[index], NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<usize, Infallible> {
        output[index] = *value;
        Ok(1)
    }
}

struct ReplaceIncomplete;

impl TranscodeDecodeHooks<PrefixCodec> for ReplaceIncomplete {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut PrefixCodec,
        error: &Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<qubit_codec::engine::DecodeInvalidAction<u8>, TranscodeDecodeErrorOf<PrefixCodec>> {
        match *error {}
    }

    fn handle_incomplete_decode(
        &mut self,
        _codec: &mut PrefixCodec,
        source: Option<&Infallible>,
        _required_total: NonZeroUsize,
        _context: DecodeContext,
    ) -> Result<DecodeIncompleteAction<u8>, TranscodeDecodeErrorOf<PrefixCodec>> {
        assert!(source.is_none());
        Ok(DecodeIncompleteAction::Emit { value: b'?' })
    }
}

let mut decoder = TranscodeDecodeEngine::new(PrefixCodec, ReplaceIncomplete);
let mut output = [0_u8; 4];
assert_eq!(0, decoder.reset(&mut output, 0).unwrap());

// Open input emits `A` and retains `~` while waiting for another unit.
let progress = decoder.transcode(b"A~", 0, &mut output, 0).unwrap();
assert!(progress.is_need_input());
assert_eq!((1, 1), (progress.read(), progress.written()));

// At EOF, the policy consumes the incomplete `~` and emits `?`.
let eof = decoder
    .transcode_eof(b"A~", progress.read(), &mut output, progress.written())
    .unwrap();
assert!(eof.is_complete());
assert_eq!((1, 1), (eof.read(), eof.written()));
let written = progress.written() + eof.written();
let finished = decoder.finish(&mut output, written).unwrap();
assert_eq!(0, finished);
assert_eq!(b"A?", &output[..written + finished]);
```

Use a custom `Transcoder` when framing, stream state, or EOF behavior cannot be expressed as a `Codec` plus hooks.

## Choose byte-order helpers

`ByteOrder` supports runtime configuration. `ByteOrderSpec` with `BigEndian`, `LittleEndian`, or `NativeEndian` supports static selection. These types describe byte-order policy; they do not implement a concrete integer codec. The `U16BeCodec` scenario encodes endianness in the representation rule itself.

## Connect buffered I/O

Enable feature `io` only when using the `qubit-io` bridges:

```toml
[dependencies]
qubit-io = "0.19"
qubit-codec = { version = "0.17", features = ["io"] }
```

`TranscodeDecodeInput` and `TranscodeEncodeOutput` integrate buffered `qubit-io` traits. At confirmed EOF, `decode_eof` is tried when the retained tail reaches the codec minimum; the incomplete hook runs only if it still reports incomplete. A shorter tail may reach the hook with no codec source. The synchronous `transcode` method dispatches `transcode_eof` when driven through EOF with output capacity remaining; if no unread tail remains, it passes empty input so a decoder can act on retained state. `transcode_eof_step` also dispatches an empty input when called explicitly with output capacity. `rs-codec-misc`'s percent codec uses this boundary to interpret a trailing `%` or `%A` as a malformed escape; the adapter decides when input is final, while the format codec owns that interpretation.

An EOF step accepts `Complete` or `NeedOutput`. If a custom decoder successfully returns `NeedInput` after EOF, the bridge returns `InvalidData` without committing that progress's source consumption. This validation does not roll back decoder state or output already written by the decoder.

The batch decode `transcode` return count is not a completion signal: a short return, including zero, can mean the decoder needs a larger contiguous output range. Add the reported output to the destination and retry with enough capacity, or use `transcode_step` and `transcode_eof_step` to observe the stop reason and required capacity. See [`TranscodeDecodeInput` Rustdoc](../src/transcode/io/transcode_decode_input.rs) for the step methods.

For async input, reset the decoder and collect its actual output first. Each `AsyncTranscodeDecodeInput::poll_transcode` progress result commits `read()` inside the bridge, so the caller advances its output cursor by `written()` and must not consume the source again. On `NeedInput { required }`, await `fill_until_async(required.get())`: `true` resumes ordinary polling; `false` confirms EOF while preserving any short unread tail. Pass that tail to `transcode_eof_step`, collect each step's `written()` values, and retry `NeedOutput` with enough output capacity. When `poll_transcode` returns `AsyncTranscodeDecodeStep::EndOfInput`, call `transcode_eof_step` even though the unread buffer is empty; it dispatches the EOF event and can emit output from retained decoder state. When the EOF step is `Complete`, call `finish` and collect its output. The bridge may conservatively request output capacity for a whole policy action, so even `Skip` or `Reject` can first report `NeedOutput`. See [`AsyncTranscodeDecodeInput` Rustdoc](../src/transcode/io/async_transcode_decode_input.rs).

Async EOF steps follow the same progress rule: if a custom decoder returns `NeedInput` successfully after EOF, the bridge returns `InvalidData` and does not commit that step's source consumption. Decoder state and output already written are not rolled back.

Before a synchronous `transcode` batch, the bridge reserves the encoder's full output upper bound. This preserves whole-batch consumption and can require substantial memory for a large batch. Keep this API when whole-range consumption is useful; use `transcode_step` when the caller needs to drive a large input incrementally:

The following standalone program can be copied into a file and run with the `io` feature. It uses a two-byte value encoder and checks the bytes delivered by the buffered bridge. The same source is maintained in [`examples/io_transcode_step.rs`](../examples/io_transcode_step.rs).

<!-- io-guide-start -->
```rust
use core::convert::Infallible;
use core::num::NonZeroUsize;
use std::io::Cursor;
use std::io::Error;

use qubit_codec::Codec;
use qubit_codec::CodecTranscodeEncoder;
use qubit_codec::DecodeFailure;
use qubit_codec::TranscodeEncodeOutput;
use qubit_codec::TranscodeStatus;

/// Encodes one fixed-width big-endian value.
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
        Ok((
            u16::from_be_bytes([input[input_index], input[input_index + 1]]),
            NonZeroUsize::new(2).unwrap(),
        ))
    }

    unsafe fn encode(&mut self, value: &u16, out: &mut [u8], index: usize) -> Result<usize, Infallible> {
        debug_assert!(index + 2 <= out.len());
        out[index..index + 2].copy_from_slice(&value.to_be_bytes());
        Ok(2)
    }
}

/// Runs a complete stepwise encode lifecycle and checks the delivered bytes.
fn main() -> Result<(), Error> {
    let values = [0x1234_u16, 0xabcd];
    let mut encoder = CodecTranscodeEncoder::new(U16BeCodec);
    let mut output = TranscodeEncodeOutput::with_capacity(Cursor::new(Vec::<u8>::new()), 2);
    let mut map_error = |_| Error::other("encode error");
    output.reset(&mut encoder, &mut map_error)?;
    let mut offset = 0;
    while offset < values.len() {
        let progress = output.transcode_step(&mut encoder, &mut map_error, &values, offset, values.len() - offset)?;
        offset += progress.read();
        if let TranscodeStatus::NeedOutput { required, .. } = progress.status() {
            output.ensure_spare_capacity(required.get())?;
        }
    }
    output.finish(&mut encoder, &mut map_error)?;
    assert_eq!(output.inner().get_ref(), &[0x12, 0x34, 0xab, 0xcd]);
    Ok(())
}
```
<!-- io-guide-end -->

Run this program with `cargo run --example io_transcode_step --features io`.

The returned `written()` units are already committed to the adapter; do not advance them again. Advance the source by `read()` before requesting more space. A zero `count` validates the range but does not call the encoder or deliver pending output. The single-value bound is only a preflight estimate: `NeedOutput` may request more, and buffer capacity retained from earlier calls does not shrink. This path does not automatically change downstream buffered writers. The operation may deliver previously buffered output before accepting the next step; async polling establishes backpressure first and can return `Pending` before calling the encoder. Explicit `flush` remains the sink flush boundary. `finish_to_buffer` only appends finalization output to memory; `finish` delivers pending units and flushes the sink.

Errors do not provide general transactional rollback or complete progress. A range or capacity error occurs before the encoder runs, although preparing capacity may deliver older pending units. A sink delivery error follows the wrapped output's pending/recovery contract. If the encoder or progress validation fails, the encoder may already have advanced, while units written by that failed call are not committed to this adapter's pending output. Do not replay the original input range unless that encoder defines a recovery protocol. A successful reset may start a new logical stream; it does not undo output already delivered from the old one.

## Register codecs for runtime lookup

Enable feature `registry` when several crates must discover bidirectional whole-value codecs (string or bytes wire) by stable ID at link time:

```toml
[dependencies]
qubit-codec = { version = "0.17", features = ["registry"] }
```

Implement paired `ValueEncoder` and `ValueDecoder` for a fixed value type, then register. String and bytes wires use different codec types, macros, and process-wide registries; the same `ValueCodecId` may appear in both.

Registration IDs and registry contents are static: macro IDs must be compile-time constant expressions, and `ValueCodecId::try_new` accepts an `&'static str`. `ValueCodecRegistry::from_registrations` also takes static registration entries. Each process-wide `global()` registry is built once from linked registrations and is immutable afterward; there is no API for adding registrations at runtime. Runtime selection among already registered codecs is supported: keep the selected ID as a string and look it up with `registry.get(configured_id.as_str())`, which accepts `&str`. If one ID has more than two sources, diagnostics list every source in stable order.

### Build a local registry without global registration

`ValueCodecRegistry::from_registrations` builds an independent registry from static entries. `ValueCodecId::new` creates a validated ID; each registration and its descriptor must have static storage because the constructor takes `&'static ValueCodecRegistration`. This call does not invoke the registration macros, consult the inventory, or change either process-wide `global()` registry. Duplicate IDs return `ValueCodecRegistryError`.

Descriptor construction has the same error bounds as the codec descriptor API: encoder and decoder error types must implement `Error + Send + Sync + 'static`. The descriptor erases the concrete error types, and execution errors expose them through the error source. Codec and value types do not need those bounds. Decode results cross the erased API as `Box<dyn Any>` and can be recovered with `downcast` when the concrete output type is known.

```rust
use std::convert::Infallible;
use qubit_codec::ValueCodecId;
use qubit_codec::ValueCodecRegistration;
use qubit_codec::ValueCodecRegistrationSource;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_codec::ValueStringCodecDescriptor;
use qubit_codec::ValueStringCodecRegistration;
use qubit_codec::ValueStringCodecRegistry;

#[derive(Default)]
struct LocalText;

impl ValueEncoder<String> for LocalText {
    type Output = String;
    type Error = Infallible;

    fn encode(&mut self, input: &String) -> Result<String, Infallible> {
        Ok(input.clone())
    }
}

impl ValueDecoder<str> for LocalText {
    type Output = String;
    type Error = Infallible;

    fn decode(&mut self, input: &str) -> Result<String, Infallible> {
        Ok(input.to_owned())
    }
}

static DESCRIPTOR: ValueStringCodecDescriptor =
    ValueStringCodecDescriptor::of::<LocalText, String>();
static REGISTRATION: ValueStringCodecRegistration = ValueCodecRegistration::new(
    ValueCodecId::new("example.local.text"),
    &DESCRIPTOR,
    ValueCodecRegistrationSource::new("local-example", "example", file!(), line!()),
);

let registry = ValueStringCodecRegistry::from_registrations([&REGISTRATION])
    .expect("unique local registration");
let descriptor = registry.get("example.local.text")
    .expect("registered locally").descriptor();
let encoded = descriptor.encode(&String::from("hello")).expect("encode");
assert_eq!("hello", encoded);
let decoded = descriptor.decode(&encoded).expect("decode");
assert_eq!("hello", *decoded.downcast::<String>().expect("String output"));
assert!(registry.get("example.missing").is_none());
```

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

An owned `String` can also be encoded through its borrowed `str` view. `value` remains the erased and decoded type; `input` selects only the encoder's borrowed input:

```rust
use std::io::Error;
use qubit_codec::{ValueDecoder, ValueEncoder, ValueStringCodecRegistry, register_value_string_codec};

#[derive(Default)]
struct BorrowedTextCodec;

impl ValueEncoder<str> for BorrowedTextCodec {
    type Output = String;
    type Error = Error;

    fn encode(&mut self, input: &str) -> Result<String, Error> {
        Ok(input.to_owned())
    }
}

impl ValueDecoder<str> for BorrowedTextCodec {
    type Output = String;
    type Error = Error;

    fn decode(&mut self, input: &str) -> Result<String, Error> {
        Ok(input.to_owned())
    }
}

register_value_string_codec!(
    id = "example.borrowed.text",
    codec = BorrowedTextCodec,
    value = String,
    input = str,
);

fn main() {
    let configured_id = String::from("example.borrowed.text");
    let registry = ValueStringCodecRegistry::try_global().expect("valid registrations");
    let descriptor = registry
        .get(configured_id.as_str())
        .expect("known codec ID")
        .descriptor();
    let encoded = descriptor.encode(&String::from("hello")).expect("encoding succeeds");
    assert_eq!(encoded, "hello");
    let decoded = descriptor.decode(&encoded).expect("decoding succeeds");
    assert_eq!(*decoded.downcast::<String>().expect("value type is String"), "hello");
}
```

For a byte view, use `value = Vec<u8>, input = [u8]`; the descriptor continues to validate and return `Vec<u8>`.

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

With the `U32BeBytesCodec` registration above linked into the program, this complete lookup and round trip uses a runtime-owned string ID:

```rust
use qubit_codec::ValueBytesCodecRegistry;

let configured_id = String::from("example.u32");
let registry = ValueBytesCodecRegistry::try_global().expect("valid registrations");
let registration = registry.get(configured_id.as_str()).expect("known codec ID");
let descriptor = registration.descriptor();
let bytes = descriptor.encode(&42_u32).expect("u32 encoding succeeds");
let decoded = descriptor.decode(&bytes).expect("u32 decoding succeeds");
let value = decoded.downcast::<u32>().expect("registered value type is u32");
assert_eq!(42, *value);
```

`ValueStringCodecRegistry::try_global()` and `ValueBytesCodecRegistry::try_global()` collect linked registrations once per process and cache initialization failures, including duplicate-ID conflicts. Every later call returns the cached conflict error. The corresponding `global()` methods panic when registrations conflict, so applications that need to handle registration conflicts should use `try_global()`. Successfully built registries remain immutable. Macro and registration IDs are validated from static IDs when declarations are compiled; runtime lookup uses `get(&str)` with the exact ID string, without normalization. Execution checks the erased input type before invoking user code. Duplicate IDs within one registry are a registration error, not a silent override. This path erases only bidirectional whole-value codecs whose value type was fixed at registration; it does not replace `Codec` streaming adapters.

Each registered codec's associated errors must implement `Error + Send + Sync + 'static`, so an execution error can retain its source across threads. A successful decode still returns `Box<dyn Any>`; this constraint does not make the complete decode result transferable across threads. Direct calls to `ValueEncoder` and `ValueDecoder` are unchanged; the public registry descriptor constructors require thread-safe errors.

### Migrate registry users from 0.16 to 0.17

Update every crate that exchanges `ValueCodecId`, registration, descriptor, or registry types to `qubit-codec = "0.17"`. Cargo can build 0.16 and 0.17 side by side, but their Rust types and process-wide inventories are separate; a codec registered through 0.17 is invisible to a 0.16 registry. If a generic integration exposes registered codec errors, update its bounds to `Error + Send + Sync + 'static` or adapt the error at that boundary. Crates that use the buffered I/O adapters should also enable the `io` feature on the same 0.17 dependency.

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
| `NeedInput` at end of file, including with no unread units | Preserve any tail and call `transcode_eof`; if empty input is a valid EOF for the format, override the default EOF behavior to complete or drain retained state before `finish`. |
| Repeated `NeedOutput` | Advance both progress counters; provide reported capacity; verify custom bounds. |
| Streaming encode returns an error | Distinguish preflight, sink delivery, and encoder/progress failures; an encoder failure may advance state without returning full progress. |
| Owned decode rejects valid-looking input | Trailing units, or decode reset/finish output; use lifecycle-aware decode. |
| Capacity rejected before conversion | Include reset, main, and finish bounds; check for arithmetic overflow. |
| `TranscodeBeforeReset` or `TranscodeAfterFinish` | Call `reset` before the first stream and before reuse. |
| Replacement logic duplicates the transcode loop | Move decisions into engine hooks. |
| Registry lookup fails or executes wrong type | Confirm the matching `register_value_string_codec!` / `register_value_bytes_codec!` is linked, the ID is unique in that registry, and input type matches registration. |

## Boundaries and a practice checklist

- Concrete formats, character sets, and high-level reader/writer adapters belong in domain crates, not in `qubit-codec`.
- `NeedInput` is a streaming signal, not final EOF. At confirmed EOF, bridges may call `transcode_eof` with an empty slice; the default still treats `NeedInput` as incomplete, so formats that accept empty EOF or drain internal state must implement that behavior explicitly.
- Capacity methods must cover every reachable transient state, not typical output size alone.
- Owned adapters may allocate `Vec` output; prefer caller-buffered APIs when allocation ownership matters.
- Test success, incomplete, invalid, boundary, and stateful lifecycle behavior through checked public adapters, not only unsafe `Codec` methods.
- Enable `io` and `registry` only in crates that use those integration paths.

## Further reading

- [README](../README.md) · [中文 README](../README.zh_CN.md) · [API reference](https://docs.rs/qubit-codec)
- [Lifecycle-aware decode example](../examples/decode_lifecycle.rs)

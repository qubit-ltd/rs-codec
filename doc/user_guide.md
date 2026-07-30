# Qubit Codec User Guide

[中文](user_guide.zh_CN.md) · [README](../README.md) · [API documentation](https://docs.rs/qubit-codec)

This guide covers `qubit-codec` 0.11 for Rust 1.94 and later. It is for codec
and adapter authors who need to convert logical values into encoded units, or
convert encoded units through caller-managed buffers. It does not define a wire
format or character set.

## Purpose and Audience

Use this crate when a format crate needs a reusable contract for one codec
quantum, an owned whole-value facade, or a streaming conversion loop. Concrete
binary layouts, character sets, and format-specific `std::io` integrations
belong in sibling domain crates.

The guide follows a format crate that starts with a `Codec` for one local value
and then needs both a convenient owned operation and a buffered stream API,
without duplicating cursor, capacity, and EOF rules.

## Conceptual Model

```text
logical value <-> Codec <-> encoded units
                     |
        +------------+-------------+
        |                          |
owned one-value adapters     caller-buffered transcoders
        |                          |
ValueEncoder / ValueDecoder  reset -> transcode* -> finish
```

Choose the smallest layer that fits:

| Requirement | Recommended API |
| --- | --- |
| One local value boundary exists | Implement `Codec` |
| The complete input is the useful unit | Implement `ValueEncoder` / `ValueDecoder` directly |
| Existing `Codec`, no custom policy | `CodecValue*` or `CodecTranscode*` adapters |
| Invalid or unencodable values need policy | `engine::TranscodeXxxEngine` plus hooks |
| EOF-aware or otherwise bespoke stream behavior | Implement `Transcoder` |

A formatted hex string or C string literal often has no meaningful one-value
quantum, so a direct value-level implementation is clearer. A fixed-width
number or character codec normally starts with `Codec`.

## Scenario: Publish One Codec at Two Levels

Suppose your crate has a local-value codec and two consumers: one owns a
complete input and wants owned output; the other supplies reusable buffers.
Implement `Codec` first, then expose `CodecValueEncoder` /
`CodecValueDecoder` for the first consumer and `CodecTranscodeEncoder` /
`CodecTranscodeDecoder` for the strict buffered path.

For unit-to-unit conversion, use `CodecTranscodeConverter` for a strict
decode-plus-encode pipeline. Use `engine::TranscodeConvertEngine` only when
the decode or encode side needs a real policy decision.

## Installation and Minimal Configuration

```toml
[dependencies]
qubit-codec = "0.11"
```

The base crate has no default features. Enable `io` only for
`TranscodeDecodeInput` or `TranscodeEncodeOutput`, which bridge
`qubit-io` buffered input/output traits:

```toml
[dependencies]
qubit-codec = { version = "0.11", features = ["io"] }
```

## Core Workflow

### 1. Implement the `Codec` contract

A `Codec` declares `Value`, `Unit`, error types, and minimum/maximum units
per value. Its unsafe `encode` and `decode` entry points are used by checked
adapters after bounds checks; implementations must still honour their documented
preconditions.

| Rule | Why it matters |
| --- | --- |
| `MIN_UNITS_PER_VALUE` and `MAX_UNITS_PER_VALUE` are non-zero and ordered | Callers use them for safe decode entry and capacity planning. |
| `decode` reads only visible input | A valid but short prefix returns `DecodeFailure::Incomplete` without consuming its tail. |
| Successful `decode` consumes a non-zero visible count | Progress remains sound and retryable. |
| `can_encode_value` rejects out-of-domain values | Checked encoders call it before `encode_len` and `encode`. |
| `encode_len` exactly matches successful `encode` output in the same state | Capacity probing stays correct; deliberate zero-output buffering is allowed. |
| Lifecycle bounds cover every reachable stream state | Callers can allocate safely before reset or finish. |

Stateless codecs use the default zero lifecycle output. A stateful encoder that
writes an EOF trailer must declare `MAX_ENCODE_FINISH_UNITS`; a decoder that
emits retained final values must declare `MAX_DECODE_FINISH_VALUES`.

### 2. Use an adapter or drive a stream

For an owned whole-value operation, the value traits have a minimal shape:

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
assert_eq!("encoded:codec", encoder.encode("codec")?);

# Ok::<(), core::convert::Infallible>(())
```

For caller-buffered conversion, use the explicit lifecycle:

```text
allocate output from declared bounds
        |
reset(output)
        |
transcode(input, input_index, output, output_index) repeatedly
        |
preserve/refill a NeedInput tail, or apply the caller's EOF policy
        |
finish(output)
```

`TranscodeProgress` contains relative `read` and `written` counts from the
provided indices. `TranscodeStatus` explains why conversion stopped:

| Status | Caller action |
| --- | --- |
| `Complete` | All visible input from `input_index` was consumed. Supply another segment or finish at EOF. |
| `NeedInput` | Preserve the unconsumed tail, refill input, and retry; at EOF, apply the format's explicit tail policy. |
| `NeedOutput` | Drain or replace the output buffer and continue from the reported progress. |

Never return `Complete` after consuming only a prefix. `finish` receives no
source input and cannot silently reinterpret a caller-owned incomplete tail.

### 3. Finish and reuse

`finish` emits retained final output such as padding, a checksum, or a stream
trailer after the caller has supplied all input and handled any incomplete tail.
Its success closes the logical stream; call `reset` before another stream.

`max_reset_output_len`, `max_transcode_output_len`, and
`max_finish_output_len` must be conservative for every reachable state.
`max_total_output_len` combines them for one complete
`reset -> transcode -> finish` operation, and `transcode_complete_into`
performs that operation with a caller-provided complete output buffer.

### Decode lifecycle output

`CodecValueDecoder::decode` is intentionally strict: it rejects a codec that
declares output from `decode_reset` or `decode_finish`, because one returned
value cannot preserve all three phases. Use `decode_lifecycle` when owned
`Vec` output is appropriate, or `decode_lifecycle_with_scratch` when the
caller owns reusable reset and finish buffers. The runnable
[lifecycle-aware decode example](../examples/decode_lifecycle.rs) shows the
strict rejection and the preserved reset, main, and finish values.

## Advanced Usage

Strict `CodecTranscode*` adapters surface codec-domain errors directly. When a
format needs replacement, skipping, counting, or phase-specific reporting, keep
the common loop in an engine and implement hooks instead:

- `engine::TranscodeEncodeEngine` with `TranscodeEncodeHooks` handles
  unencodable-value, reset, and finish policy.
- `engine::TranscodeDecodeEngine` with `TranscodeDecodeHooks` handles
  invalid-input policy.
- `engine::TranscodeConvertEngine` composes both sides with their hooks.

Use a custom `Transcoder` or value-level facade for EOF-aware maximal-munch
parsing, delayed boundary decisions, or pending-prefix reinterpretation at EOF.
Those policies cannot be put into `finish`.

Use runtime `ByteOrder` in public configuration. Use `ByteOrderSpec` with
`BigEndian`, `LittleEndian`, or `NativeEndian` where static selection is
valuable.

## Errors and Diagnostics

| Error type | Meaning and usual recovery |
| --- | --- |
| `DecodeFailure` | Low-level incomplete visible prefix or invalid codec-domain input. Only the incomplete case is retryable. |
| `TranscodeFailure` | Framework failure involving indices, capacity, incomplete whole input, trailing input, or related stream conditions. |
| `CapacityError` | Capacity-planning arithmetic failed before output is written. |
| `TranscodeDomainError<E>` | Domain/policy error tagged with reset, main, or finish phase. |
| Directional transcode errors | Preserve whether failure came from the encode, decode, or conversion side; encode/conversion errors may retain an unencodable value. |
| `TranscodeContractError` | A custom transcoder reported inconsistent progress; treat it as an implementation defect. |

Do not flatten an incomplete prefix and malformed input unless downstream callers
genuinely take the same recovery action for both.

## Troubleshooting

| Symptom | Check |
| --- | --- |
| `NeedInput` at a file end | Preserve the tail, apply the format's EOF policy, then call `finish`. |
| Repeated `NeedOutput` | Advance by `TranscodeProgress`, provide the relevant capacity, and verify custom counters are truthful. |
| Checked adapter rejects capacity | Ensure bounds include reset, main, and finish output from every reachable state. |
| One-value decode rejects input | `CodecValueDecoder` is strict; use lifecycle-aware or streaming APIs when the format requires them. |
| A format needs replacement | Use the appropriate engine hooks rather than duplicate the buffered loop. |

## Limitations and Best Practices

- Keep concrete formats, character sets, and high-level reader/writer adapters
  in domain crates.
- Treat `NeedInput` as a streaming boundary signal, not a final EOF error.
- Keep unsafe `Codec` implementations small and make length/consumption
  invariants exact.
- Owned-output helpers can allocate their returned values; choose
  caller-buffered APIs when allocation control matters.
- Enable `io` only for the `qubit-io` bridges you use.

## Further Reading

- [README](../README.md)
- [中文用户手册](user_guide.zh_CN.md)
- [API documentation](https://docs.rs/qubit-codec)

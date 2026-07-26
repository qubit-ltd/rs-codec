# Qubit Codec User Guide

`qubit-codec` provides the domain-neutral contracts shared by Qubit binary,
text, miscellaneous, and I/O codec crates. It intentionally does not provide
concrete wire formats or character sets. This guide explains how to select an
abstraction and, when needed, implement its contracts correctly.

For item-by-item signatures, bounds, and error variants, use the generated API
documentation. This guide concentrates on the relationships between methods
that an implementation must preserve.

## Choose the Smallest Abstraction

| Requirement | Use |
| --- | --- |
| Encode or decode one value/quantum against a caller buffer | `Codec` |
| Convert an entire borrowed value into owned output | `ValueEncoder` / `ValueDecoder` |
| Wrap an existing `Codec` in a strict streaming bridge | `CodecTranscodeEncoder`, `CodecTranscodeDecoder`, or `CodecTranscodeConverter` |
| Apply policy to malformed or unencodable values | `TranscodeXxxEngine` with its hooks |
| Define a bespoke caller-buffered stream transform | `Transcoder` |

Do not implement `Codec` merely because data is encoded. Whole-input formats
whose useful unit is the complete input, such as a formatted hex string or a C
literal, are usually clearer as `ValueEncoder` / `ValueDecoder` implementations.
Conversely, do not duplicate a buffering loop when an existing `Codec` can be
wrapped by one of the provided adapters.

## `Codec` Implementer Guide

`Codec` owns one logical value or quantum and operates on caller-provided unit
buffers. Its `encode` and `decode` entry points are `unsafe` because checked
adapters perform the bounds checks once and keep hot paths free of repeated
slice construction. An implementation must never read or write outside the
documented caller-provided range.

### Required invariants

- `MIN_UNITS_PER_VALUE` and `MAX_UNITS_PER_VALUE` are non-zero, and the former
  is no greater than the latter.
- `decode` only reads visible input. If the visible prefix is valid but cannot
  form a value, return `DecodeFailure::Incomplete`; do not consume it.
- Successful `decode` returns a non-zero consumed count no greater than the
  currently available input.
- For the same value and codec state, `encode_len` exactly equals the count
  returned by successful `encode`, including a deliberate zero when output is
  retained internally.
- `encode_len` never exceeds `MAX_UNITS_PER_VALUE`.
- `can_encode_value(value)` is queried before `encode_len` and `encode`; reject
  a value there instead of treating an unsupported value as an arbitrary
  `EncodeError`.
- Reset and finish bounds cover every reachable stream state, not merely the
  current state. Return zero for stateless lifecycle phases.
- After any error, internal state remains self-consistent and can follow the
  documented retry/reset policy.

### Minimal fixed-width codec

This template is suitable for a stateless one-byte value codec. It uses
`debug_assert!` to document the unchecked preconditions; checked adapters
enforce them before calling these methods.

```rust
use core::{convert::Infallible, num::NonZeroUsize};
use qubit_codec::{Codec, DecodeFailure, nz};

struct ByteCodec;

impl Codec for ByteCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_UNITS_PER_VALUE: usize = 1;

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Infallible> {
        debug_assert!(output_index < output.len());
        output[output_index] = *value;
        Ok(1)
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
        debug_assert!(input_index < input.len());
        Ok((input[input_index], nz!(1)))
    }
}
```

### Variable-width and stateful codecs

Override `encode_len` when output width depends on the value. A buffered
encoder may validly return zero from both `encode_len` and `encode` for a value
that it retains until later input or `encode_finish`; it must then declare a
finish bound large enough for the retained output.

Decode-side state does not receive an EOF input slice. If an input tail must be
reinterpreted at EOF, the default `Codec` bridge is the wrong boundary: put
that policy in a custom `Transcoder` or a value-level facade. `decode_finish`
may emit retained values or validate decode state, but cannot re-read a tail
previously reported as incomplete.

## `Transcoder` Implementer Guide

`Transcoder` converts a logical input stream into an output stream. Its indices
are absolute positions in the supplied slices; `TranscodeProgress::read()` and
`written()` are relative counts from those positions. The caller owns input
refill and decides what an incomplete tail means at EOF.

### Lifecycle

```text
reset(output) -> transcode(...) repeatedly -> caller handles EOF tail -> finish(output)
```

`reset` starts a new logical stream while retaining immutable configuration.
After successful `finish`, portable callers reset before reusing the instance.
Implementations must discard stream-local pending state on reset.

### Progress rules

| Result | Required meaning |
| --- | --- |
| `Complete` | All input visible from `input_index` was consumed. |
| `NeedInput` | The incomplete tail was not consumed and must remain available for retry. |
| `NeedOutput` | Output capacity stopped conversion; report the absolute output boundary and unsatisfied requirement. |

Never return `Complete` after consuming only a prefix. Do not use `finish` to
silently reinterpret an incomplete tail: callers must choose and apply the EOF
policy before finalization.

### Minimal byte-copy transcoder

```rust
use core::{convert::Infallible, num::NonZeroUsize};
use qubit_codec::{
    CapacityError,
    TranscodeDecodeError,
    TranscodeProgress,
    TranscodeStatus,
    Transcoder,
};

struct ByteCopy;

impl Transcoder for ByteCopy {
    type Input = u8;
    type Output = u8;
    type Error = TranscodeDecodeError<Infallible>;

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        Self::Error::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        Self::Error::ensure_transcode_indices(
            input.len(), input_index, output.len(), output_index,
        )?;
        let count = (input.len() - input_index).min(output.len() - output_index);
        output[output_index..output_index + count]
            .copy_from_slice(&input[input_index..input_index + count]);
        if input_index + count == input.len() {
            Ok(TranscodeProgress::complete(count, count))
        } else {
            Ok(TranscodeProgress::new(
                TranscodeStatus::NeedOutput {
                    output_index: output_index + count,
                    required: NonZeroUsize::MIN,
                    available: 0,
                },
                count,
                count,
            ))
        }
    }

    fn finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        Self::Error::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }
}
```

For a stateful transcoder, `max_reset_output_len`,
`max_transcode_output_len`, and `max_finish_output_len` must be conservative
upper bounds from every reachable state. `max_total_output_len` combines them
for a complete `reset -> transcode -> finish` operation.

## Errors and Recovery

Errors deliberately preserve different recovery information:

- `DecodeFailure` is used only at the low-level `Codec::decode` boundary and
  separates incomplete prefixes from invalid units.
- `TranscodeFailure` reports framework problems: invalid indices, insufficient
  output, overflow, incomplete whole input, and trailing input.
- `CapacityError` reports capacity-planning arithmetic before output is written.
- `TranscodeDomainError` adds reset/main/finish phase context to domain errors.
- Directional errors preserve whether a failure came from decode, encode, or a
  converter, and `TranscodeEncodeError` / `TranscodeConvertError` can preserve
  an unencodable value.

Do not flatten these categories in downstream APIs unless the caller truly has
the same recovery action for every category. In particular, an incomplete
prefix is commonly retryable, while malformed input and broken progress
contracts generally are not.

## Frequent Implementation Errors

| Error | Correct rule |
| --- | --- |
| Returning `Complete` after a partial read | Return `NeedInput` or `NeedOutput`; `Complete` consumes every visible input unit. |
| Consuming an incomplete tail | Leave it caller-owned and report `NeedInput` or `DecodeFailure::Incomplete`. |
| Making `encode_len` only an upper bound | It must equal the actual successful `encode` count in the same state. |
| Sizing reset/finish from current state only | Bounds must cover every reachable state. |
| Using `finish` to inspect caller input | `finish` receives no source input; make EOF policy explicit outside the default bridge. |
| Reimplementing engine cursor logic in a domain crate | Use strict adapters or `TranscodeXxxEngine` plus hooks when their policy model fits. |

## Related Documentation

- [README](../README.md): crate overview, feature list, and API map.
- [Chinese user guide](user_guide.zh_CN.md): Chinese version of this guide.
- Rust API documentation: detailed method contracts and error variants.

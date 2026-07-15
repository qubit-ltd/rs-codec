# Codec Lifecycle and Capacity Contract Design

## Goal

Make complete value encoding correct for codecs whose state changes during
reset, make every transcoder capacity bound independent of transient stream
state, order converter reset output as a valid target stream, restore the
configured per-source coverage thresholds, and remove stale documentation and
no-op tests.

## Current consumers

`Transcoder::max_total_output_len` has two production call sites outside
`rs-codec`: `CharsetStringDecoder` and `CharsetStringEncoder` in `rs-io-text`.
They use it to allocate the output slice passed to a complete one-shot
transcode. `rs-codec-text` also uses it in property-test helpers, while the
remaining call sites are engine and trait tests inside `rs-codec`.

## Complete value encode lifecycle

Complete value adapters must not call `can_encode_value` or `encode_len` before
`Codec::encode_reset`, because reset may change both the encodable domain and
the exact encoded width.

The adapters will reserve the checked conservative bound

```text
MAX_ENCODE_RESET_UNITS + MAX_UNITS_PER_VALUE + MAX_ENCODE_FINISH_UNITS
```

before entering the unsafe lifecycle. The lifecycle helper will then:

1. run `encode_reset`;
2. validate `can_encode_value` in the reset state;
3. query `encode_len` in that same state;
4. encode the value and verify the reported width;
5. run `encode_finish`; and
6. return the actual number of units written so owned and buffered adapters can
   truncate or advance by the real length.

This removes the pre-reset exact-size helper. It deliberately prefers a safe,
state-correct upper-bound reservation over speculative cloning or a second
public sizing API.

## Capacity contract

All `Transcoder` capacity methods are configuration-dependent but transient-
state-independent upper bounds:

- `max_reset_output_len` bounds reset output from every reachable stream state;
- `max_transcode_output_len(input_len)` bounds output for `input_len` units from
  every reachable stream state, including retained output that may be drained;
- `max_finish_output_len` bounds finish output from every reachable stream
  state; and
- `max_total_output_len` remains their checked sum.

For example, an encoder that may emit one checksum byte at finish must always
return `1` from `max_finish_output_len`, including before any input is supplied
and after the previous stream has finished. Implementations may return tighter
bounds based on immutable configuration, but not based on whether transient
pending state currently exists.

The engine hook capacity contracts follow the same rule. Codec reset and finish
constants already have this meaning. Decode and encode engine bounds remain
checked sums of codec and hook global bounds.

The converter needs special handling because it can retain one decoded value.
Its global bounds will conservatively include:

- one possible pending value during streaming and finishing;
- all decode-reset or decode-finish values converted at
  `E::MAX_UNITS_PER_VALUE` each; and
- the target encoder's global reset or finish bound.

This avoids consulting whether a pending value happens to exist when the bound
is queried.

## Converter reset order

`TranscodeConvertEngine::reset` will clear converter pending state, reset the
target encode engine first, and only then reset the source decode engine and
encode source reset values. Target-owned stream-start output such as a BOM or
framing header therefore precedes every target-encoded source value, and source
reset values are never encoded with stale target state.

The combined reset bound is unchanged in structure, but it uses global bounds
and the target reset output occupies the first portion of the caller's buffer.

## Coverage and cleanup

The updated `.rs-ci` correctly enforces per-source functions, lines, and
regions. Tests will be added for actual lifecycle, capacity, I/O fallback,
progress, and error behavior until every source file satisfies functions
`>= 100%`, lines `> 95%`, and regions `> 95%`. Thresholds and source inclusion
will not be weakened.

README error names and examples will be updated to the currently exported
`TranscodeFailure`, `TranscodeDomainError`, `TranscodeEncodeError`,
`TranscodeDecodeError`, and `TranscodeConvertError` families. Empty
`test_module_compiles` tests and test modules with no behavior will be removed;
the existing `encode_context` behavioral test remains.

## Compatibility and verification

No public method signatures change. The capacity methods become more
conservative for stateful implementations, which is the intended contract
correction. Existing stateless binary and charset codecs keep the same bounds.

Verification consists of targeted red-green regression cycles, the complete
`rs-codec/ci-check.sh`, and all-feature tests for the five direct downstream
crates: `rs-codec-binary`, `rs-codec-text`, `rs-codec-misc`, `rs-io-binary`, and
`rs-io-text`.

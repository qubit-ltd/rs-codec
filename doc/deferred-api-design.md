# Deferred rs-codec API Design Items

This document records rs-codec design items intentionally deferred while the
current breaking refactor proceeds.

## Deferred Items

### Stateless codec helper or split

Item 11 is deferred. The current decision is to keep `Codec` unified and not
split it into stateful/stateless traits during this refactor.

Open questions for later review:

- Whether a `StatelessCodec` extension trait would materially improve direct
  static helper use without complicating the core `Codec` contract.
- Whether any downstream code is blocked by `Codec::decode` and
  `Codec::encode` taking `&mut self`.
- Whether such a helper should provide only ergonomics or a separate optimized
  adapter path.

### Default-bound removal for decoded values

Item 15 is deferred. `CodecValueDecoder` and `TranscodeConvertEngine` currently
require `Default` for some value-buffer and scratch paths.

Open questions for later review:

- Whether internal scratch storage should move from initialized `Vec<T>` to
  `MaybeUninit<T>` for decoded values.
- Whether streaming `finish` and reset output should expose initialized slices
  only, or introduce a separate uninitialized-output API.
- How to preserve drop safety when partially initialized buffers contain owned
  decoded values.
- Which public adapters should keep requiring `Default` for simpler caller
  ergonomics even if lower layers become more general.

## Resolved Items

### Single-value decode lifecycle output

The silent discard of values emitted by `decode_reset` and `decode_finish` has
a concrete follow-up design. Strict single-value APIs will reject codecs that
declare lifecycle output, while lifecycle-aware owned and scratch APIs will
expose reset, main, and finish results separately. See
[`decode_lifecycle_output_design.zh_CN.md`](decode_lifecycle_output_design.zh_CN.md).

### ValueEncoder and ValueDecoder error mapping

Item 17 has been resolved. `ValueEncoder::map_error`,
`ValueDecoder::map_error`, and their `DomainError` associated types were
removed from the value-level traits. Error mapping now belongs at explicit
facade or I/O adapter boundaries instead of being part of the one-shot value
conversion contract.

## Review Trigger

After the current accepted refactor is implemented, formatted, checked, and
committed, reread this document and propose concrete designs for the deferred
items before making further code changes.

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

### ValueEncoder and ValueDecoder error mapping

Item 17 is deferred. `ValueEncoder::map_error` and `ValueDecoder::map_error`
currently create repeated identity-mapping boilerplate in downstream codecs.

Open questions for later review:

- Whether to remove `DomainError` and `map_error` entirely from the value-level
  traits.
- Whether mapping belongs in I/O adapters instead of value facades.
- Whether a separate extension trait should provide mapped-error helpers for
  callers that still need them.
- How a breaking change would affect `rs-codec-misc` implementations such as
  Base64, hex, percent, C string literal, and form URL encoding.

## Review Trigger

After the current accepted refactor is implemented, formatted, checked, and
committed, reread this document and propose concrete designs for the deferred
items before making further code changes.

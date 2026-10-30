## Generation record

- Generated locally on 2026-10-30 23:59 Asia/Shanghai (2026-10-30 15:59 UTC) from baseline commit `31a5dc944a3858656593f13ac4498a2e9a5df127`; the worktree was dirty with this task's Rustdoc and benchmark documentation changes. No production behavior changed.
- Coverage was generated in worktree `/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17` for the `rs-codec` project at `/data/working/qubit/rust-common/rs-codec`. Embedded text and HTML details retain the worktree's build paths; the accompanying 211-entry manifest uses repository-relative paths and hashes so the source snapshot remains identifiable after worktree cleanup.
- Source snapshot: SHA-256 `5092915dba43a0a91ffc1520cbebdabda2f95ffaf9297d593e31b87dad731a5b`, over 211 sorted repository-relative file paths and per-file SHA-256 values. The full manifest is `target/review-7a98bc17/coverage-source-manifest.json`; each entry hashes `src/**/*.rs`, `tests/**/*.rs`, `examples/**/*.rs`, `benches/**/*.rs`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.infra/coverage/coverage.json`, and `.infra/ci/cargo-matrix.json`. The report itself is excluded.
- Environment: `rustc 1.94.0 (4a4ef493e 2026-03-02)`, host `x86_64-unknown-linux-gnu`, LLVM 21.1.8; `cargo-llvm-cov 0.8.6`; `rs-infra-coverage@407d3f58fddbe55ad7eb62f7c1ce7acfc1c7c9b1`.
- Entry command: `./.infra/bin/coverage.sh` (exit 0). The wrapper selected package `qubit-codec` and ran `cargo llvm-cov --package qubit-codec --all-features --json --output-path target/infra/coverage/raw.json`; coverage collection also enabled `cfg(coverage)`. The report generator completed JSON, LCOV, Cobertura, text, HTML, and badge outputs successfully. Configured thresholds and exemptions were unchanged.
- Local outputs: HTML `target/review-7a98bc17/coverage/html/index.html`, LCOV `target/review-7a98bc17/coverage/lcov.info`, JSON `target/review-7a98bc17/coverage/coverage.json`, text `target/review-7a98bc17/coverage/coverage.txt`, badge `target/review-7a98bc17/coverage/coverage-badge.json`, and Cobertura `target/review-7a98bc17/coverage/cobertura.xml`. These are local files, not uploaded CI artifacts.
- Relevant paths were hit in the all-features report: async EOF dispatch at `async_transcode_decode_input.rs:654` (count 9); EOF `NeedInput` validation branch at `transcode_progress_validation.rs:77-80` (branch count 24, error construction count 5); fallible allocations at `transcode_convert_engine.rs:917` (count 7) and `:1042` (count 9). The report summarizes the source files, not standalone tests; relevant cases are present in `tests/transcode/io/async_transcode_decode_input_tests.rs`, `tests/transcode/io/transcode_decode_input_tests.rs`, and `tests/transcode/engine/transcode_convert_engine_allocation_tests.rs`.

## Code coverage

| Metric | Covered | Total | Coverage |
| --- | ---: | ---: | ---: |
| functions | 606 | 615 | 98.54% |
| lines | 3894 | 4015 | 96.99% |
| regions | 5020 | 5285 | 94.99% |

- HTML report: local path `target/review-7a98bc17/coverage/html/index.html` (not uploaded as a CI artifact).
- LCOV report: local path `target/review-7a98bc17/coverage/lcov.info` (not uploaded as a CI artifact).
- Shields endpoint: local path `target/review-7a98bc17/coverage/coverage-badge.json` (not uploaded as a CI artifact).

- Full text report: local path `target/review-7a98bc17/coverage/coverage.txt` (not uploaded as a CI artifact).

<details><summary>Full text coverage report</summary>

```text
/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/codec/codec.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Low-level value codec trait.
    9|       |
   10|       |mod codec;
   11|       |
   12|       |pub use codec::Codec;
   13|       |
   14|       |/// Compile-time asserts the public unit-bound invariants required by [`Codec`].
   15|       |///
   16|       |/// # Type Parameters
   17|       |///
   18|       |/// - `C`: Codec implementation to validate.
   19|       |///
   20|       |/// # Returns
   21|       |///
   22|       |/// Returns unit `()`.
   23|       |///
   24|       |/// # Panics
   25|       |///
   26|       |/// Panics at compile time when either decode width bound is zero or when
   27|       |/// [`Codec::MIN_UNITS_PER_VALUE`] is greater than
   28|       |/// [`Codec::MAX_DECODE_UNITS_PER_VALUE`], because these invariants must hold
   29|       |/// for any well-formed [`Codec`] implementation and violating them is always a
   30|       |/// bug.
   31|       |#[inline]
   32|  5.38k|pub(crate) fn assert_unit_bounds<C>()
   33|  5.38k|where
   34|  5.38k|    C: codec::Codec,
   35|       |{
   36|       |    const {
   37|       |        assert!(
   38|       |            C::MIN_UNITS_PER_VALUE > 0,
   39|       |            "Codec::MIN_UNITS_PER_VALUE must be non-zero",
   40|       |        );
   41|       |        assert!(
   42|       |            C::MAX_DECODE_UNITS_PER_VALUE > 0,
   43|       |            "Codec::MAX_DECODE_UNITS_PER_VALUE must be non-zero",
   44|       |        );
   45|       |        assert!(
   46|       |            C::MIN_UNITS_PER_VALUE <= C::MAX_DECODE_UNITS_PER_VALUE,
   47|       |            "Codec::MIN_UNITS_PER_VALUE must not exceed Codec::MAX_DECODE_UNITS_PER_VALUE",
   48|       |        );
   49|       |    }
   50|  5.38k|}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/codec/codec/codec.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |
    9|       |use core::num::NonZeroUsize;
   10|       |
   11|       |use super::super::decode_failure::DecodeFailure;
   12|       |
   13|       |/// Encodes and decodes one value or codec quantum against a unit buffer.
   14|       |///
   15|       |/// `Codec` is the lowest-level abstraction in the codec stack. It is intended
   16|       |/// for hot paths that have already validated buffer capacity and want to avoid
   17|       |/// constructing subslices for every value. Higher-level transcoders and
   18|       |/// convenience APIs are responsible for checked buffer management and owned
   19|       |/// output allocation.
   20|       |///
   21|       |/// `MIN_UNITS_PER_VALUE` and `MAX_DECODE_UNITS_PER_VALUE` describe the readable
   22|       |/// representation-width bounds for one decoded value. If fewer than the
   23|       |/// minimum units are available, no complete value can exist, so a streaming
   24|       |/// caller can request more input or report an incomplete EOF tail. The minimum
   25|       |/// is also the smallest safety precondition checked callers must satisfy before
   26|       |/// entering [`decode`](Self::decode). The decode maximum bounds successful
   27|       |/// consumption and incomplete-input requirements for one value.
   28|       |///
   29|       |/// `MAX_ENCODE_UNITS_PER_VALUE` is the independent value-agnostic upper bound
   30|       |/// for main-phase encode output. Reserving that many units makes encoding safe
   31|       |/// for every encodable value. Once a value and encode state are known, callers
   32|       |/// may instead reserve the exact [`encode_len`](Self::encode_len), which must
   33|       |/// not exceed the encode maximum. Complete-lifecycle adapters that still need
   34|       |/// to run [`encode_reset`](Self::encode_reset) must reserve the maximum first
   35|       |/// because reset may change the exact width.
   36|       |///
   37|       |/// A codec may keep decode-side and encode-side stream state. That state is an
   38|       |/// implementation detail owned by the codec. Callers do not snapshot or restore
   39|       |/// it; implementations must keep their own state internally consistent across
   40|       |/// every public operation. A failed decode attempt must not advance decode
   41|       |/// state, so callers can retry the same input after refilling their buffer or
   42|       |/// applying an invalid-input policy.
   43|       |///
   44|       |/// Decode operations see only the currently supplied input slice and codec
   45|       |/// state. [`decode`](Self::decode) treats that slice as an open stream segment;
   46|       |/// [`decode_eof`](Self::decode_eof) receives the same preconditions after the
   47|       |/// caller has established that no more units will arrive. Returning
   48|       |/// [`DecodeFailure::Incomplete`] from `decode` requests more input for the
   49|       |/// current value; it is not itself an EOF error. Codecs that need EOF-aware
   50|       |/// maximal-munch parsing or reinterpretation of a trailing prefix can override
   51|       |/// `decode_eof` without changing their streaming behavior.
   52|       |/// An incomplete failure may also retain codec-specific context through
   53|       |/// [`DecodeFailure::incomplete_with_source`]; stream drivers may ignore that
   54|       |/// context while retrying and use it for detailed EOF diagnostics.
   55|       |///
   56|       |/// # Associated Types
   57|       |///
   58|       |/// - `Value`: Logical value decoded from or encoded into the buffer. This may
   59|       |///   be a scalar such as `u8`, `u16`, `u64`, a `char`, a fixed quantum such as
   60|       |///   `[u8; 3]`, or an owned value such as `String`/`Vec<u8>`. Adapters that
   61|       |///   need scratch initialization add their own bounds at the use site.
   62|       |/// - `Unit`: Buffer unit used by the encoded representation. Implementations
   63|       |///   are typically scalar storage units such as `u8`, `u16`, or `char`.
   64|       |///   Adapters that allocate owned output add their own initialization bounds at
   65|       |///   the use site.
   66|       |///
   67|       |/// Implementors must uphold the safety contract documented by
   68|       |/// [`decode`](Self::decode), [`encode`](Self::encode),
   69|       |/// [`encode_reset`](Self::encode_reset), and
   70|       |/// [`decode_finish`](Self::decode_finish). Unchecked implementations must not
   71|       |/// read or write outside the caller-provided ranges. Implementations should use
   72|       |/// `debug_assert!` to state the expected buffer bounds at the unchecked entry
   73|       |/// point.
   74|       |///
   75|       |/// Implementations must also guarantee that
   76|       |/// [`MIN_UNITS_PER_VALUE`](Self::MIN_UNITS_PER_VALUE) is non-zero and no larger
   77|       |/// than [`MAX_DECODE_UNITS_PER_VALUE`](Self::MAX_DECODE_UNITS_PER_VALUE), which
   78|       |/// must also be non-zero. `MAX_ENCODE_UNITS_PER_VALUE` may be zero for a codec
   79|       |/// that always buffers each accepted value instead of immediately emitting
   80|       |/// units. Checked adapters enforce the decode-bound invariants with
   81|       |/// compile-time assertions before using codec-provided bounds.
   82|       |///
   83|       |/// # Why the low-level methods are unchecked
   84|       |///
   85|       |/// The `unsafe` entry points are intentional: checked adapters validate the
   86|       |/// index and capacity preconditions once, then call [`encode`](Self::encode)
   87|       |/// and [`decode`](Self::decode) without rebuilding subslices or repeating
   88|       |/// bounds checks for every value. The `safe_vs_unchecked` benchmark in
   89|       |/// `benches/transcode.rs` compares the same 64 KiB copy loop in release mode.
   90|       |/// Its result is workload- and toolchain-specific, so callers should use
   91|       |/// checked adapters unless they can establish the documented preconditions at
   92|       |/// a trusted boundary. Measure representative codec workloads before choosing
   93|       |/// an unchecked entry point. Codec implementations should keep the unchecked
   94|       |/// methods small and use `debug_assert!` for those assumptions.
   95|       |///
   96|       |/// # Examples
   97|       |///
   98|       |/// A minimal identity codec exposes its bounds and decodes one byte:
   99|       |///
  100|       |/// ```
  101|       |/// use core::convert::Infallible;
  102|       |/// use core::num::NonZeroUsize;
  103|       |///
  104|       |/// use qubit_codec::Codec;
  105|       |/// use qubit_codec::DecodeFailure;
  106|       |///
  107|       |/// struct Identity;
  108|       |/// impl Identity {
  109|       |///     fn new() -> Self { Self }
  110|       |/// }
  111|       |/// impl Codec for Identity {
  112|       |///     type Value = u8;
  113|       |///     type Unit = u8;
  114|       |///     type DecodeError = Infallible;
  115|       |///     type EncodeError = Infallible;
  116|       |///     const MIN_UNITS_PER_VALUE: usize = 1;
  117|       |///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
  118|       |///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
  119|       |///
  120|       |///     unsafe fn decode(&mut self, input: &[u8], index: usize)
  121|       |///         -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
  122|       |///         Ok((input[index], NonZeroUsize::MIN))
  123|       |///     }
  124|       |///
  125|       |///     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize)
  126|       |///         -> Result<usize, Infallible> {
  127|       |///         output[index] = *value;
  128|       |///         Ok(1)
  129|       |///     }
  130|       |/// }
  131|       |///
  132|       |/// let mut codec = Identity::new();
  133|       |/// assert_eq!(codec.encode_len(&42), 1);
  134|       |/// // SAFETY: the input contains the one readable unit required by Identity.
  135|       |/// let (value, consumed) = unsafe { codec.decode(&[42], 0) }.unwrap();
  136|       |/// assert_eq!((value, consumed.get()), (42, 1));
  137|       |/// ```
  138|       |pub trait Codec {
  139|       |    /// The type of logical values decoded from or encoded into the buffer.
  140|       |    type Value;
  141|       |
  142|       |    /// The type of buffer units used by the encoded representation.
  143|       |    type Unit;
  144|       |
  145|       |    /// The type of errors reported when decoding malformed units.
  146|       |    type DecodeError;
  147|       |
  148|       |    /// The type of errors reported when encoding an unsupported value.
  149|       |    type EncodeError;
  150|       |
  151|       |    /// The minimum possible non-zero readable unit count for one decoded value.
  152|       |    ///
  153|       |    /// This is the minimum readable width of any valid encoded value. Checked
  154|       |    /// callers use it for planning and fast impossibility checks. If a
  155|       |    /// streaming decoder has fewer than this many readable units, no complete
  156|       |    /// value can be present at the current position. Unsafe
  157|       |    /// [`decode`](Self::decode) callers must provide at least this many
  158|       |    /// readable units at `index`.
  159|       |    ///
  160|       |    /// Default decode hooks use the input unit count as their output bound:
  161|       |    /// an invalid-input policy may consume fewer units than a valid value and
  162|       |    /// still emit one replacement value.
  163|       |    const MIN_UNITS_PER_VALUE: usize;
  164|       |
  165|       |    /// The maximum unit count emitted when encoding one value.
  166|       |    ///
  167|       |    /// This is a value-independent upper bound for the main encode phase. When
  168|       |    /// no concrete value is available, reserving this many writable units is
  169|       |    /// sufficient for any encodable value, without first calling
  170|       |    /// [`encode_len`](Self::encode_len). For a known value, callers may reserve
  171|       |    /// only the exact length returned by `encode_len` instead. This bound does
  172|       |    /// not include output produced by encode reset or finish phases.
  173|       |    ///
  174|       |    /// This is also the multiplier used by default encode hooks when estimating
  175|       |    /// the maximum number of output units needed for an input value count.
  176|       |    ///
  177|       |    /// The bound may be zero for a codec that buffers every accepted value and
  178|       |    /// emits retained output only during a later lifecycle phase.
  179|       |    const MAX_ENCODE_UNITS_PER_VALUE: usize;
  180|       |
  181|       |    /// The maximum non-zero unit count consumed to decode one value.
  182|       |    ///
  183|       |    /// This is a value-independent upper bound for one complete encoded value
  184|       |    /// or codec quantum on the decode side. Successful decode consumption and
  185|       |    /// the `required_total` carried by [`DecodeFailure::Incomplete`] must not
  186|       |    /// exceed this bound.
  187|       |    const MAX_DECODE_UNITS_PER_VALUE: usize;
  188|       |
  189|       |    /// The maximum unit count emitted when resetting encode state.
  190|       |    ///
  191|       |    /// This bound covers only output written by
  192|       |    /// [`encode_reset`](Self::encode_reset). It does not include hook-owned
  193|       |    /// reset output or any ordinary encoded input values. It must cover every
  194|       |    /// reachable pre-reset encode state. Stateless codecs should use the
  195|       |    /// default `0`.
  196|       |    const MAX_ENCODE_RESET_UNITS: usize = 0;
  197|       |
  198|       |    /// The maximum unit count emitted when finishing encode state at EOF.
  199|       |    ///
  200|       |    /// This bound covers only output written by
  201|       |    /// [`encode_finish`](Self::encode_finish). It does not include hook-owned
  202|       |    /// finish output and must cover every reachable encode state, even when the
  203|       |    /// current stream has no pending finish output. Stateless codecs should use
  204|       |    /// the default `0`. Codecs that emit a stream trailer (padding, checksum,
  205|       |    /// or end-of-stream marker) should override this with the exact maximum
  206|       |    /// unit count.
  207|       |    const MAX_ENCODE_FINISH_UNITS: usize = 0;
  208|       |
  209|       |    /// The maximum value count emitted when resetting decode state.
  210|       |    ///
  211|       |    /// This bound covers only values written by
  212|       |    /// [`decode_reset`](Self::decode_reset). It does not include hook-owned
  213|       |    /// reset output or values decoded from source units. It must cover every
  214|       |    /// reachable pre-reset decode state. Stateless codecs should use the
  215|       |    /// default `0`. Codecs that emit a stream-start sentinel or BOM on reset
  216|       |    /// should override this.
  217|       |    const MAX_DECODE_RESET_VALUES: usize = 0;
  218|       |
  219|       |    /// The maximum value count emitted when finishing decode state.
  220|       |    ///
  221|       |    /// This bound covers only values written by
  222|       |    /// [`decode_finish`](Self::decode_finish). It does not include hook-owned
  223|       |    /// finish output and must cover every reachable decode state, even when the
  224|       |    /// current stream has no pending finish output. Stateless codecs should use
  225|       |    /// the default `0`.
  226|       |    const MAX_DECODE_FINISH_VALUES: usize = 0;
  227|       |
  228|       |    /// Returns whether `value` is in this codec's encodable value domain.
  229|       |    ///
  230|       |    /// The default implementation returns `true`, which is correct for codecs
  231|       |    /// whose [`Value`](Self::Value) type contains only values they can encode.
  232|       |    /// Codecs whose logical value type is broader than their representation
  233|       |    /// domain, such as an ASCII codec with `Value = char`, must override this
  234|       |    /// method.
  235|       |    ///
  236|       |    /// Checked encoder adapters call this method in the same codec state used
  237|       |    /// to query [`encode_len`](Self::encode_len) and enter the unsafe
  238|       |    /// [`encode`](Self::encode) method. Complete-lifecycle adapters do so after
  239|       |    /// [`encode_reset`](Self::encode_reset). Direct unsafe callers must uphold
  240|       |    /// the same ordering.
  241|       |    ///
  242|       |    /// # Parameters
  243|       |    ///
  244|       |    /// - `value`: Value whose encodability is queried.
  245|       |    ///
  246|       |    /// # Returns
  247|       |    ///
  248|       |    /// Returns `true` when `value` may be passed to
  249|       |    /// [`encode_len`](Self::encode_len) and [`encode`](Self::encode).
  250|       |    #[inline]
  251|       |    #[must_use]
  252|     62|    fn can_encode_value(&self, _value: &Self::Value) -> bool {
  253|     62|        true
  254|     62|    }
  255|       |
  256|       |    /// Returns the exact unit count this codec will write when encoding
  257|       |    /// `value`.
  258|       |    ///
  259|       |    /// The default implementation returns
  260|       |    /// [`MAX_ENCODE_UNITS_PER_VALUE`](Self::MAX_ENCODE_UNITS_PER_VALUE). This
  261|       |    /// default is correct only when every successful main encode writes exactly
  262|       |    /// that many units in the relevant codec state, as fixed-width codecs
  263|       |    /// normally do.
  264|       |    ///
  265|       |    /// Variable-width, buffering, or otherwise state-dependent codecs must
  266|       |    /// override this method to report the true encoded length for encodable
  267|       |    /// `value`s. The return value is never an estimate or an "unknown" marker:
  268|       |    /// it must equal the unit count [`encode`](Self::encode) subsequently
  269|       |    /// writes for the same `value` under the same codec state. It must also
  270|       |    /// never exceed
  271|       |    /// [`MAX_ENCODE_UNITS_PER_VALUE`](Self::MAX_ENCODE_UNITS_PER_VALUE).
  272|       |    /// Callers that do not yet have a concrete value should use
  273|       |    /// `MAX_ENCODE_UNITS_PER_VALUE` directly instead of calling this method.
  274|       |    ///
  275|       |    /// Default codec-backed encoders use this exact value for per-value output
  276|       |    /// capacity. The contract requires callers to use this method only when
  277|       |    /// [`can_encode_value`](Self::can_encode_value) returned `true` for the
  278|       |    /// same `value`.
  279|       |    ///
  280|       |    /// # Parameters
  281|       |    ///
  282|       |    /// - `value`: Value whose encoded length is queried.
  283|       |    ///
  284|       |    /// # Returns
  285|       |    ///
  286|       |    /// Returns the unit count [`encode`](Self::encode) will write for an
  287|       |    /// encodable `value`.
  288|       |    ///
  289|       |    /// A return value of `0` is valid for stateful encoders that accept a
  290|       |    /// value into internal state without immediately emitting units. Examples
  291|       |    /// include codecs that aggregate values into fixed-size quanta, shift-state
  292|       |    /// encodings, or framing layers that defer output until enough values have
  293|       |    /// been seen or EOF is finalized. Stateless and directly value-to-unit
  294|       |    /// codecs should continue returning a positive length.
  295|       |    #[inline]
  296|       |    #[must_use]
  297|    112|    fn encode_len(&self, _value: &Self::Value) -> usize {
  298|    112|        Self::MAX_ENCODE_UNITS_PER_VALUE
  299|    112|    }
  300|       |
  301|       |    /// Emits stream-start output and resets encode-side state.
  302|       |    ///
  303|       |    /// # Parameters
  304|       |    ///
  305|       |    /// - `output`: Destination unit buffer.
  306|       |    /// - `output_index`: Start index in `output`.
  307|       |    ///
  308|       |    /// # Returns
  309|       |    ///
  310|       |    /// Returns the number of reset units written.
  311|       |    ///
  312|       |    /// # Errors
  313|       |    ///
  314|       |    /// Returns `Self::EncodeError` when reset output cannot be emitted.
  315|       |    /// Implementations must leave encode state unchanged when returning an
  316|       |    /// error.
  317|       |    ///
  318|       |    /// # Safety
  319|       |    ///
  320|       |    /// The caller must guarantee that the implementation can write up to
  321|       |    /// [`MAX_ENCODE_RESET_UNITS`](Self::MAX_ENCODE_RESET_UNITS) units starting
  322|       |    /// at `output_index`.
  323|       |    #[inline]
  324|    153|    unsafe fn encode_reset(
  325|    153|        &mut self,
  326|    153|        _output: &mut [Self::Unit],
  327|    153|        _output_index: usize,
  328|    153|    ) -> Result<usize, Self::EncodeError> {
  329|    153|        Ok(0)
  330|    153|    }
  331|       |
  332|       |    /// Encodes one borrowed value into `output` starting at `output_index`.
  333|       |    ///
  334|       |    /// # Parameters
  335|       |    ///
  336|       |    /// - `value`: Value to encode.
  337|       |    /// - `output`: Destination unit buffer.
  338|       |    /// - `output_index`: Start index in `output`.
  339|       |    ///
  340|       |    /// # Returns
  341|       |    ///
  342|       |    /// Returns the number of written units.
  343|       |    ///
  344|       |    /// A successful encode may return `0` when the codec accepted `value` into
  345|       |    /// internal encode-side state but intentionally deferred output. This is
  346|       |    /// the normal shape for accumulative encoders such as base64-byte
  347|       |    /// aggregators, shift-state encodings, or frame builders. In that case
  348|       |    /// [`encode_len`](Self::encode_len) for the same `value` and state must
  349|       |    /// also return `0`, and the retained output must be emitted by a later
  350|       |    /// successful [`encode`](Self::encode),
  351|       |    /// [`encode_finish`](Self::encode_finish), or codec-specific facade.
  352|       |    ///
  353|       |    /// # Errors
  354|       |    ///
  355|       |    /// Returns `Self::EncodeError` for encode-side state or representation
  356|       |    /// failures other than a value being outside the codec's encodable domain.
  357|       |    /// Checked callers reject values for which
  358|       |    /// [`can_encode_value`](Self::can_encode_value) returns `false` before
  359|       |    /// entering this unsafe method. Implementations must leave their internal
  360|       |    /// state consistent when returning an error.
  361|       |    ///
  362|       |    /// # Safety
  363|       |    ///
  364|       |    /// The caller must guarantee that
  365|       |    /// [`can_encode_value`](Self::can_encode_value) returned `true` for
  366|       |    /// `value`. Starting at `output_index`, the caller must also provide either
  367|       |    /// at least
  368|       |    /// [`MAX_ENCODE_UNITS_PER_VALUE`](Self::MAX_ENCODE_UNITS_PER_VALUE)
  369|       |    /// writable
  370|       |    /// units, which is sufficient for every encodable value, or at least the
  371|       |    /// exact [`encode_len`](Self::encode_len) for the same `value` and codec
  372|       |    /// state. On success, implementations must return that exact written unit
  373|       |    /// count, including `0` for deliberate buffering, and the count must be no
  374|       |    /// larger than `MAX_ENCODE_UNITS_PER_VALUE`. Implementations must not write
  375|       |    /// beyond the exact `encode_len` capacity on either the success or error
  376|       |    /// path.
  377|       |    unsafe fn encode(
  378|       |        &mut self,
  379|       |        value: &Self::Value,
  380|       |        output: &mut [Self::Unit],
  381|       |        output_index: usize,
  382|       |    ) -> Result<usize, Self::EncodeError>;
  383|       |
  384|       |    /// Emits EOF trailer output and finishes encode-side state.
  385|       |    ///
  386|       |    /// This is the encode-side counterpart of
  387|       |    /// [`decode_finish`](Self::decode_finish). Codecs that append stream
  388|       |    /// trailers (padding, checksums, end-of-stream markers) emit them here.
  389|       |    /// Stateless codecs use the default no-op.
  390|       |    ///
  391|       |    /// # Parameters
  392|       |    ///
  393|       |    /// - `output`: Destination unit buffer.
  394|       |    /// - `output_index`: Start index in `output`.
  395|       |    ///
  396|       |    /// # Returns
  397|       |    ///
  398|       |    /// Returns the number of finish units written.
  399|       |    ///
  400|       |    /// # Errors
  401|       |    ///
  402|       |    /// Returns `Self::EncodeError` when finish output cannot be emitted.
  403|       |    /// Implementations must leave their internal state consistent when
  404|       |    /// returning an error.
  405|       |    ///
  406|       |    /// # Safety
  407|       |    ///
  408|       |    /// The caller must guarantee that the implementation can write up to
  409|       |    /// [`MAX_ENCODE_FINISH_UNITS`](Self::MAX_ENCODE_FINISH_UNITS) units
  410|       |    /// starting at `output_index`.
  411|       |    #[inline]
  412|     47|    unsafe fn encode_finish(
  413|     47|        &mut self,
  414|     47|        _output: &mut [Self::Unit],
  415|     47|        _output_index: usize,
  416|     47|    ) -> Result<usize, Self::EncodeError> {
  417|     47|        Ok(0)
  418|     47|    }
  419|       |
  420|       |    /// Emits stream-start values and resets decode-side state.
  421|       |    ///
  422|       |    /// This is the decode-side counterpart of
  423|       |    /// [`encode_reset`](Self::encode_reset). Codecs that emit a
  424|       |    /// stream-start marker or BOM before decoding a new stream emit them
  425|       |    /// here. Stateless codecs use the default no-op.
  426|       |    ///
  427|       |    /// # Parameters
  428|       |    ///
  429|       |    /// - `output`: Destination value buffer.
  430|       |    /// - `output_index`: Start index in `output`.
  431|       |    ///
  432|       |    /// # Returns
  433|       |    ///
  434|       |    /// Returns the number of reset values written.
  435|       |    ///
  436|       |    /// # Errors
  437|       |    ///
  438|       |    /// Returns `Self::DecodeError` when reset output cannot be emitted.
  439|       |    /// Implementations must leave their internal state consistent when
  440|       |    /// returning an error.
  441|       |    ///
  442|       |    /// # Safety
  443|       |    ///
  444|       |    /// The caller must guarantee that the implementation can write up to
  445|       |    /// [`MAX_DECODE_RESET_VALUES`](Self::MAX_DECODE_RESET_VALUES) values
  446|       |    /// starting at `output_index`.
  447|       |    #[inline]
  448|  1.39k|    unsafe fn decode_reset(
  449|  1.39k|        &mut self,
  450|  1.39k|        _output: &mut [Self::Value],
  451|  1.39k|        _output_index: usize,
  452|  1.39k|    ) -> Result<usize, Self::DecodeError> {
  453|  1.39k|        Ok(0)
  454|  1.39k|    }
  455|       |
  456|       |    /// Decodes one value from `input` starting at `input_index`.
  457|       |    ///
  458|       |    /// # Parameters
  459|       |    ///
  460|       |    /// - `input`: Source unit buffer.
  461|       |    /// - `input_index`: Start index in `input`.
  462|       |    ///
  463|       |    /// # Returns
  464|       |    ///
  465|       |    /// Returns the decoded value and the non-zero number of consumed units.
  466|       |    ///
  467|       |    /// # Errors
  468|       |    ///
  469|       |    /// Returns [`DecodeFailure::Incomplete`] when the visible input is a
  470|       |    /// valid prefix but more units are needed to decide or complete a value.
  471|       |    /// This reports a streaming boundary, not a final EOF condition; the
  472|       |    /// caller or higher-level adapter decides what an incomplete tail means
  473|       |    /// when the upstream source is closed. Codec-specific context may be
  474|       |    /// retained in the failure's optional `source` field.
  475|       |    /// Returns [`DecodeFailure::Invalid`] when the units are malformed,
  476|       |    /// non-canonical, unmappable, or otherwise invalid for this codec and the
  477|       |    /// span is known or unknown. Unknown span is represented as
  478|       |    /// [`DecodeFailure::Invalid`] with an unknown consumed-unit hint.
  479|       |    /// Implementations must leave their internal state consistent when
  480|       |    /// returning an error.
  481|       |    ///
  482|       |    /// # Safety
  483|       |    ///
  484|       |    /// The caller must guarantee that `input_index` is a valid boundary in
  485|       |    /// `input` and that at least
  486|       |    /// [`MIN_UNITS_PER_VALUE`](Self::MIN_UNITS_PER_VALUE)
  487|       |    /// units are readable from `input_index`. Implementations must not read
  488|       |    /// beyond the currently available units under that precondition. They
  489|       |    /// may return [`DecodeFailure::Incomplete`] when those units are a
  490|       |    /// valid but incomplete prefix.
  491|       |    ///
  492|       |    /// On success, implementations must return a consumed unit count no larger
  493|       |    /// than either the available input or
  494|       |    /// [`MAX_DECODE_UNITS_PER_VALUE`](Self::MAX_DECODE_UNITS_PER_VALUE). An
  495|       |    /// incomplete result's `required_total` must also not exceed that maximum.
  496|       |    /// The return type guarantees that successful decoding always consumes at
  497|       |    /// least one unit. Implementations should use `debug_assert!` to state
  498|       |    /// these unchecked entry-point assumptions.
  499|       |    unsafe fn decode(
  500|       |        &mut self,
  501|       |        input: &[Self::Unit],
  502|       |        input_index: usize,
  503|       |    ) -> Result<(Self::Value, NonZeroUsize), DecodeFailure<Self::DecodeError>>;
  504|       |
  505|       |    /// Decodes one value after the caller has established end of input.
  506|       |    ///
  507|       |    /// This method has the same safety and state guarantees as
  508|       |    /// [`decode`](Self::decode), but may resolve an otherwise incomplete
  509|       |    /// trailing prefix using EOF-aware format rules. The default preserves the
  510|       |    /// open-stream behavior by delegating to [`decode`](Self::decode).
  511|       |    ///
  512|       |    /// # Parameters
  513|       |    ///
  514|       |    /// - `input`: Final source unit buffer; no more units will arrive.
  515|       |    /// - `input_index`: Start index of the value in `input`.
  516|       |    ///
  517|       |    /// # Returns
  518|       |    ///
  519|       |    /// Returns the decoded value and the non-zero number of consumed units.
  520|       |    ///
  521|       |    /// # Errors
  522|       |    ///
  523|       |    /// Returns [`DecodeFailure::Incomplete`] for an unresolved final prefix,
  524|       |    /// or [`DecodeFailure::Invalid`] for malformed input, as in
  525|       |    /// [`decode`](Self::decode). An override may resolve a prefix at EOF.
  526|       |    ///
  527|       |    /// # Safety
  528|       |    ///
  529|       |    /// The caller must satisfy the same input-index and minimum-readable-unit
  530|       |    /// preconditions as [`decode`](Self::decode). Implementations must not
  531|       |    /// read beyond the currently available units.
  532|       |    #[inline]
  533|  2.11k|    unsafe fn decode_eof(
  534|  2.11k|        &mut self,
  535|  2.11k|        input: &[Self::Unit],
  536|  2.11k|        input_index: usize,
  537|  2.11k|    ) -> Result<(Self::Value, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
  538|       |        // SAFETY: `decode_eof` has the same preconditions as `decode`.
  539|  2.11k|        unsafe { self.decode(input, input_index) }
  540|  2.11k|    }
  541|       |
  542|       |    /// Finishes decode-side EOF state into `output`.
  543|       |    ///
  544|       |    /// `decode_finish` receives no source input. Callers must have already
  545|       |    /// handled any tail reported by [`DecodeFailure::Incomplete`] before
  546|       |    /// finishing decode state. Implementations may emit retained values or
  547|       |    /// validate internal EOF state, but they must not depend on re-reading the
  548|       |    /// incomplete source tail.
  549|       |    ///
  550|       |    /// # Parameters
  551|       |    ///
  552|       |    /// - `output`: Destination value buffer.
  553|       |    /// - `output_index`: Start index in `output`.
  554|       |    ///
  555|       |    /// # Returns
  556|       |    ///
  557|       |    /// Returns the number of finished values written.
  558|       |    ///
  559|       |    /// # Errors
  560|       |    ///
  561|       |    /// Returns `Self::DecodeError` when retained decode state is invalid at
  562|       |    /// EOF. Implementations must leave their internal state consistent when
  563|       |    /// returning an error.
  564|       |    ///
  565|       |    /// # Safety
  566|       |    ///
  567|       |    /// The caller must guarantee that the implementation can write up to
  568|       |    /// [`MAX_DECODE_FINISH_VALUES`](Self::MAX_DECODE_FINISH_VALUES) values
  569|       |    /// starting at `output_index`.
  570|       |    #[inline]
  571|  1.21k|    unsafe fn decode_finish(
  572|  1.21k|        &mut self,
  573|  1.21k|        _output: &mut [Self::Value],
  574|  1.21k|        _output_index: usize,
  575|  1.21k|    ) -> Result<usize, Self::DecodeError> {
  576|  1.21k|        Ok(0)
  577|  1.21k|    }
  578|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/codec/decode_failure.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Failures returned by low-level codec decode operations.
    9|       |
   10|       |use core::num::NonZeroUsize;
   11|       |
   12|       |/// Failure reported by [`crate::Codec::decode`].
   13|       |///
   14|       |/// This type separates stream-control failures from codec-domain failures.
   15|       |/// [`Incomplete`](Self::Incomplete) tells buffered adapters to preserve the
   16|       |/// current input tail and request more units. [`Invalid`](Self::Invalid)
   17|       |/// carries the codec-specific malformed, non-canonical, or otherwise invalid
   18|       |/// input error with a known non-zero consumption length, or with unknown
   19|       |/// consumption when set to `None`.
   20|       |///
   21|       |/// # Type Parameters
   22|       |///
   23|       |/// - `E`: Codec-specific invalid-input error type.
   24|       |///
   25|       |/// # Examples
   26|       |///
   27|       |/// ```
   28|       |/// use core::num::NonZeroUsize;
   29|       |///
   30|       |/// use qubit_codec::DecodeFailure;
   31|       |///
   32|       |/// let failure = DecodeFailure::<()>::incomplete(NonZeroUsize::MIN);
   33|       |/// assert_eq!(failure.required_total(), Some(NonZeroUsize::MIN));
   34|       |/// assert_eq!(failure.invalid_source(), None);
   35|       |/// ```
   36|       |#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
   37|       |#[must_use]
   38|       |pub enum DecodeFailure<E> {
   39|       |    /// The visible input is a valid prefix but not enough to decode a value.
   40|       |    #[non_exhaustive]
   41|       |    Incomplete {
   42|       |        /// Codec-specific context for the incomplete prefix, when available.
   43|       |        ///
   44|       |        /// Stream drivers may ignore this field while retrying input. Once the
   45|       |        /// stream reaches EOF, adapters can use it to preserve detailed
   46|       |        /// diagnostics instead of reconstructing an error from the length
   47|       |        /// bound alone.
   48|       |        source: Option<E>,
   49|       |        /// Current non-zero minimum total units required before retrying from
   50|       |        /// the value start.
   51|       |        ///
   52|       |        /// A later retry may raise this lower bound after the codec sees a
   53|       |        /// longer prefix.
   54|       |        required_total: NonZeroUsize,
   55|       |    },
   56|       |
   57|       |    /// The input is invalid for the codec.
   58|       |    #[non_exhaustive]
   59|       |    Invalid {
   60|       |        /// Codec-specific invalid-input error.
   61|       |        source: E,
   62|       |        /// Invalid units that may be consumed by a non-strict policy, when
   63|       |        /// known.
   64|       |        consumed: Option<NonZeroUsize>,
   65|       |    },
   66|       |}
   67|       |
   68|       |impl<E> DecodeFailure<E> {
   69|       |    /// Creates an incomplete-input decode failure.
   70|       |    ///
   71|       |    /// # Parameters
   72|       |    ///
   73|       |    /// - `required_total`: Current non-zero minimum total units required before
   74|       |    ///   retrying from the value start. A later retry may raise this lower
   75|       |    ///   bound.
   76|       |    ///
   77|       |    /// # Returns
   78|       |    ///
   79|       |    /// Returns an incomplete decode failure.
   80|       |    #[inline]
   81|     38|    pub const fn incomplete(required_total: NonZeroUsize) -> Self {
   82|     38|        Self::Incomplete {
   83|     38|            source: None,
   84|     38|            required_total,
   85|     38|        }
   86|     38|    }
   87|       |
   88|       |    /// Creates an incomplete-input decode failure with codec-specific context.
   89|       |    ///
   90|       |    /// # Parameters
   91|       |    ///
   92|       |    /// - `source`: Codec-specific error describing the visible incomplete
   93|       |    ///   prefix.
   94|       |    /// - `required_total`: Current non-zero minimum units required before
   95|       |    ///   retrying from the value start.
   96|       |    ///
   97|       |    /// # Returns
   98|       |    ///
   99|       |    /// Returns an incomplete decode failure retaining `source` for adapters
  100|       |    /// that need detailed EOF diagnostics.
  101|       |    #[inline]
  102|      9|    pub const fn incomplete_with_source(source: E, required_total: NonZeroUsize) -> Self {
  103|      9|        Self::Incomplete {
  104|      9|            source: Some(source),
  105|      9|            required_total,
  106|      9|        }
  107|      9|    }
  108|       |
  109|       |    /// Creates an invalid-input decode failure with a consumption hint.
  110|       |    ///
  111|       |    /// # Parameters
  112|       |    ///
  113|       |    /// - `source`: Codec-specific invalid-input error.
  114|       |    /// - `consumed`: Invalid input units that may be consumed.
  115|       |    ///
  116|       |    /// # Returns
  117|       |    ///
  118|       |    /// Returns an invalid decode failure.
  119|       |    #[inline]
  120|  1.05k|    pub const fn invalid(source: E, consumed: NonZeroUsize) -> Self {
  121|  1.05k|        Self::Invalid {
  122|  1.05k|            source,
  123|  1.05k|            consumed: Some(consumed),
  124|  1.05k|        }
  125|  1.05k|    }
  126|       |
  127|       |    /// Creates an invalid-input decode failure with unknown consumption.
  128|       |    ///
  129|       |    /// # Parameters
  130|       |    ///
  131|       |    /// - `source`: Codec-specific invalid-input error.
  132|       |    ///
  133|       |    /// # Returns
  134|       |    ///
  135|       |    /// Returns an invalid decode failure.
  136|       |    #[inline]
  137|     12|    pub const fn invalid_unknown(source: E) -> Self {
  138|     12|        Self::Invalid { source, consumed: None }
  139|     12|    }
  140|       |
  141|       |    /// Returns the current minimum input length required before retrying an
  142|       |    /// incomplete prefix.
  143|       |    ///
  144|       |    /// # Returns
  145|       |    ///
  146|       |    /// Returns `Some(required_total)` for incomplete failures, where the value
  147|       |    /// is a lower bound that a later retry may raise, or `None` for
  148|       |    /// invalid-input failures.
  149|       |    #[inline]
  150|      5|    pub const fn required_total(&self) -> Option<NonZeroUsize> {
  151|      5|        match self {
  152|      3|            Self::Incomplete { required_total, .. } => Some(*required_total),
  153|      2|            Self::Invalid { .. } => None,
  154|       |        }
  155|      5|    }
  156|       |
  157|       |    /// Borrows the codec-specific error retained for an incomplete prefix.
  158|       |    ///
  159|       |    /// # Returns
  160|       |    ///
  161|       |    /// Returns `Some(source)` when the codec supplied incomplete-input context,
  162|       |    /// or `None` when the failure only carries the retry bound or is invalid.
  163|       |    #[inline]
  164|      1|    pub const fn incomplete_source(&self) -> Option<&E> {
  165|      1|        match self {
  166|      1|            Self::Incomplete { source, .. } => source.as_ref(),
  167|      0|            Self::Invalid { .. } => None,
  168|       |        }
  169|      1|    }
  170|       |
  171|       |    /// Borrows the codec-specific invalid-input error.
  172|       |    ///
  173|       |    /// # Returns
  174|       |    ///
  175|       |    /// Returns `Some(source)` for invalid-input failures, or `None` for
  176|       |    /// incomplete failures.
  177|       |    #[inline]
  178|      3|    pub const fn invalid_source(&self) -> Option<&E> {
  179|      3|        match self {
  180|      2|            Self::Invalid { source, .. } => Some(source),
  181|      1|            Self::Incomplete { .. } => None,
  182|       |        }
  183|      3|    }
  184|       |
  185|       |    /// Returns invalid units that may be consumed by a non-strict policy.
  186|       |    ///
  187|       |    /// # Returns
  188|       |    ///
  189|       |    /// Returns `Some(consumed)` for [`Invalid`](Self::Invalid), or `None` when
  190|       |    /// the failure is incomplete or invalid with unknown consumption.
  191|       |    #[inline]
  192|      3|    pub const fn consumed_units(&self) -> Option<NonZeroUsize> {
  193|      3|        match self {
  194|      2|            Self::Invalid { consumed, .. } => *consumed,
  195|      1|            Self::Incomplete { .. } => None,
  196|       |        }
  197|      3|    }
  198|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/adapter/codec_transcode_converter.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Buffered converter adapter backed by two low-level codecs.
    9|       |
   10|       |use core::fmt;
   11|       |
   12|       |use super::super::engine::TranscodeConvertEngine;
   13|       |use super::CodecTranscodeDecodeHooks;
   14|       |use super::CodecTranscodeEncodeHooks;
   15|       |use crate::CapacityError;
   16|       |use crate::Codec;
   17|       |use crate::TranscodeConvertErrorOf;
   18|       |use crate::TranscodeConverter;
   19|       |use crate::TranscodeProgress;
   20|       |use crate::Transcoder;
   21|       |
   22|       |/// Converts source units to target units through a decoded value by using
   23|       |/// codecs.
   24|       |///
   25|       |/// The converter decodes one source value with the decoder codec, then encodes
   26|       |/// that value with the encoder codec. If the current output buffer cannot hold
   27|       |/// the encoded value, the already decoded value is retained by the common
   28|       |/// converter engine and must be drained before more source input is consumed.
   29|       |/// Incomplete source tails are left in the caller-provided input slice while
   30|       |/// the stream is open; callers own input-buffer refill. After upstream EOF,
   31|       |/// call [`Transcoder::transcode_eof`] to let the source codec apply
   32|       |/// [`Codec::decode_eof`] and the configured incomplete-input policy to the
   33|       |/// remaining tail, then call [`Transcoder::finish`] to finalize the stream.
   34|       |/// This supports EOF-aware maximal-munch decoding when the codec expresses it
   35|       |/// through `decode_eof`. A custom transcoder or facade is needed only when the
   36|       |/// source framing requires behavior outside that codec EOF contract.
   37|       |///
   38|       |/// # Type Parameters
   39|       |///
   40|       |/// - `D`: Low-level codec used to decode source units.
   41|       |/// - `E`: Low-level codec used to encode target units.
   42|       |///
   43|       |/// # Examples
   44|       |///
   45|       |/// A byte-preserving codec can be adapted to the buffered streaming interface.
   46|       |///
   47|       |/// ```
   48|       |/// use core::convert::Infallible;
   49|       |/// use core::num::NonZeroUsize;
   50|       |///
   51|       |/// use qubit_codec::Codec;
   52|       |/// use qubit_codec::CodecTranscodeConverter;
   53|       |/// use qubit_codec::DecodeFailure;
   54|       |///
   55|       |/// # struct ByteCodec;
   56|       |/// # impl Codec for ByteCodec {
   57|       |/// #     type Value = u8;
   58|       |/// #     type Unit = u8;
   59|       |/// #     type DecodeError = Infallible;
   60|       |/// #     type EncodeError = Infallible;
   61|       |/// #     const MIN_UNITS_PER_VALUE: usize = 1;
   62|       |/// #     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
   63|       |/// #     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
   64|       |/// #     unsafe fn decode(&mut self, input: &[u8], index: usize)
   65|       |/// #         -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
   66|       |/// #         Ok((input[index], NonZeroUsize::MIN))
   67|       |/// #     }
   68|       |/// #     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize)
   69|       |/// #         -> Result<usize, Infallible> {
   70|       |/// #         output[index] = *value;
   71|       |/// #         Ok(1)
   72|       |/// #     }
   73|       |/// # }
   74|       |/// let mut converter = CodecTranscodeConverter::new(ByteCodec, ByteCodec);
   75|       |/// let mut output = [0_u8; 2];
   76|       |/// converter.reset(&mut output, 0).unwrap();
   77|       |/// converter.transcode(&[7, 9], 0, &mut output, 0).unwrap();
   78|       |/// assert_eq!(output, [7, 9]);
   79|       |/// ```
   80|       |#[must_use]
   81|       |pub struct CodecTranscodeConverter<D, E>
   82|       |where
   83|       |    D: Codec,
   84|       |    E: Codec<Value = D::Value>,
   85|       |{
   86|       |    /// Common buffered converter engine.
   87|       |    engine: TranscodeConvertEngine<D, E, CodecTranscodeDecodeHooks, CodecTranscodeEncodeHooks>,
   88|       |}
   89|       |
   90|       |impl<D, E> CodecTranscodeConverter<D, E>
   91|       |where
   92|       |    D: Codec,
   93|       |    E: Codec<Value = D::Value>,
   94|       |{
   95|       |    /// Creates a buffered converter backed by decoder and encoder codecs.
   96|       |    ///
   97|       |    /// # Parameters
   98|       |    ///
   99|       |    /// - `decoder`: Low-level codec used to decode source units.
  100|       |    /// - `encoder`: Low-level codec used to encode target units.
  101|       |    ///
  102|       |    /// # Returns
  103|       |    ///
  104|       |    /// Returns a buffered converter adapter for the supplied codecs.
  105|       |    #[inline]
  106|     25|    pub fn new(decoder: D, encoder: E) -> Self {
  107|     25|        Self {
  108|     25|            engine: TranscodeConvertEngine::new(decoder, encoder, CodecTranscodeDecodeHooks, CodecTranscodeEncodeHooks),
  109|     25|        }
  110|     25|    }
  111|       |
  112|       |    /// Returns a shared reference to the source codec.
  113|       |    ///
  114|       |    /// # Returns
  115|       |    ///
  116|       |    /// Borrows the source codec without allocating or changing stream state.
  117|       |    #[inline]
  118|       |    #[must_use]
  119|      1|    pub fn source_codec(&self) -> &D {
  120|      1|        self.engine.source_codec()
  121|      1|    }
  122|       |
  123|       |    /// Returns a mutable reference to the source codec.
  124|       |    ///
  125|       |    /// Mutating a codec during an active stream can invalidate that stream's
  126|       |    /// assumptions; reset the adapter before continuing with the new codec
  127|       |    /// configuration.
  128|       |    ///
  129|       |    /// # Returns
  130|       |    ///
  131|       |    /// Exclusively borrows the codec while the adapter remains borrowed.
  132|       |    #[inline]
  133|       |    #[must_use]
  134|      1|    pub fn source_codec_mut(&mut self) -> &mut D {
  135|      1|        self.engine.source_codec_mut()
  136|      1|    }
  137|       |
  138|       |    /// Returns a shared reference to the target codec.
  139|       |    ///
  140|       |    /// # Returns
  141|       |    ///
  142|       |    /// Borrows the target codec without allocating or changing stream state.
  143|       |    #[inline]
  144|       |    #[must_use]
  145|      1|    pub fn target_codec(&self) -> &E {
  146|      1|        self.engine.target_codec()
  147|      1|    }
  148|       |
  149|       |    /// Returns a mutable reference to the target codec.
  150|       |    ///
  151|       |    /// Mutating a codec during an active stream can invalidate that stream's
  152|       |    /// assumptions; reset the adapter before continuing with the new codec
  153|       |    /// configuration.
  154|       |    ///
  155|       |    /// # Returns
  156|       |    ///
  157|       |    /// Exclusively borrows the codec while the adapter remains borrowed.
  158|       |    #[inline]
  159|       |    #[must_use]
  160|      1|    pub fn target_codec_mut(&mut self) -> &mut E {
  161|      1|        self.engine.target_codec_mut()
  162|      1|    }
  163|       |
  164|       |    /// Consumes the adapter and returns its source and target codecs.
  165|       |    ///
  166|       |    /// Any buffered lifecycle state, pending value, and internal hooks are
  167|       |    /// discarded.
  168|       |    ///
  169|       |    /// # Returns
  170|       |    ///
  171|       |    /// Returns the owned source codec followed by the owned target codec.
  172|       |    #[inline]
  173|       |    #[must_use]
  174|      1|    pub fn into_codecs(self) -> (D, E) {
  175|      1|        let (source, target, _, _) = self.engine.into_parts();
  176|      1|        (source, target)
  177|      1|    }
  178|       |
  179|       |    /// Returns an upper bound for target units produced from `input_len` units.
  180|       |    ///
  181|       |    /// This concrete adapter method is available even when `D::Value` does not
  182|       |    /// implement [`Default`].
  183|       |    ///
  184|       |    /// # Parameters
  185|       |    ///
  186|       |    /// - `input_len`: Source units the caller plans to convert.
  187|       |    ///
  188|       |    /// # Returns
  189|       |    ///
  190|       |    /// Returns a conservative upper bound for produced target units.
  191|       |    ///
  192|       |    /// # Errors
  193|       |    ///
  194|       |    /// Returns [`CapacityError::OutputLengthOverflow`] when component-bound
  195|       |    /// arithmetic overflows.
  196|       |    #[inline]
  197|     10|    pub fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
  198|     10|        self.engine.max_transcode_output_len(input_len)
  199|     10|    }
  200|       |
  201|       |    /// Returns the global maximum target units emitted by finishing internal
  202|       |    /// state.
  203|       |    ///
  204|       |    /// # Returns
  205|       |    ///
  206|       |    /// Returns a conservative upper bound valid for every reachable converter
  207|       |    /// state.
  208|       |    ///
  209|       |    /// # Errors
  210|       |    ///
  211|       |    /// Returns [`CapacityError::OutputLengthOverflow`] when component-bound
  212|       |    /// arithmetic overflows.
  213|       |    #[inline]
  214|      9|    pub fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
  215|      9|        self.engine.max_finish_output_len()
  216|      9|    }
  217|       |
  218|       |    /// Returns the maximum target units emitted when resetting stream state.
  219|       |    ///
  220|       |    /// # Returns
  221|       |    ///
  222|       |    /// Returns a conservative upper bound valid for every reachable converter
  223|       |    /// state.
  224|       |    ///
  225|       |    /// # Errors
  226|       |    ///
  227|       |    /// Returns [`CapacityError::OutputLengthOverflow`] when component-bound
  228|       |    /// arithmetic overflows.
  229|       |    #[inline]
  230|      6|    pub fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
  231|      6|        self.engine.max_reset_output_len()
  232|      6|    }
  233|       |
  234|       |    /// Clears retained pending output, resets target state, and then encodes
  235|       |    /// source-side reset values.
  236|       |    ///
  237|       |    /// `D::Value: Default` is required so the engine can allocate scratch
  238|       |    /// storage for any stream-start values the source decoder emits through
  239|       |    /// [`Codec::decode_reset`] before they are
  240|       |    /// piped through the target encoder. Stateless decoders never reach the
  241|       |    /// allocating path; the bound is consulted only when
  242|       |    /// [`Codec::MAX_DECODE_RESET_VALUES`]
  243|       |    /// is non-zero.
  244|       |    ///
  245|       |    /// # Parameters
  246|       |    ///
  247|       |    /// - `output`: Target unit slice for stream-start output.
  248|       |    /// - `output_index`: Absolute target output index where writing starts.
  249|       |    ///
  250|       |    /// # Returns
  251|       |    ///
  252|       |    /// Returns the number of target units written while resetting component
  253|       |    /// state.
  254|       |    ///
  255|       |    /// # Errors
  256|       |    ///
  257|       |    /// Returns a converter error when the output range is invalid or too
  258|       |    /// small, or when decoder or encoder reset processing fails.
  259|     29|    pub fn reset(&mut self, output: &mut [E::Unit], output_index: usize) -> Result<usize, TranscodeConvertErrorOf<D, E>>
  260|     29|    where
  261|     29|        D::Value: Default,
  262|       |    {
  263|     29|        self.engine.reset(output, output_index)
  264|     29|    }
  265|       |
  266|       |    /// Converts source units into target units.
  267|       |    ///
  268|       |    /// This method's signature does not require `D::Value: Default`. Starting
  269|       |    /// and finishing a usable stream through the current reset/finish APIs
  270|       |    /// still requires that bound. Removing the lifecycle scratch bound is
  271|       |    /// deferred.
  272|       |    ///
  273|       |    /// # Parameters
  274|       |    ///
  275|       |    /// - `input`: Source unit slice.
  276|       |    /// - `input_index`: Absolute source index where conversion starts.
  277|       |    /// - `output`: Target unit slice.
  278|       |    /// - `output_index`: Absolute target index where writing starts.
  279|       |    ///
  280|       |    /// # Returns
  281|       |    ///
  282|       |    /// Returns conversion progress for consumed/produced counters and stop
  283|       |    /// reason.
  284|       |    ///
  285|       |    /// # Errors
  286|       |    ///
  287|       |    /// Returns converter error when source or target indices are invalid, or
  288|       |    /// when decoding/encoding fails under current policy.
  289|     23|    pub fn transcode(
  290|     23|        &mut self,
  291|     23|        input: &[D::Unit],
  292|     23|        input_index: usize,
  293|     23|        output: &mut [E::Unit],
  294|     23|        output_index: usize,
  295|     23|    ) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>> {
  296|     23|        self.engine.transcode(input, input_index, output, output_index)
  297|     23|    }
  298|       |
  299|       |    /// Converts source units after the caller has established end of input.
  300|       |    ///
  301|       |    /// # Parameters
  302|       |    ///
  303|       |    /// - `input`: Final source unit slice; the caller must not supply more
  304|       |    ///   input later.
  305|       |    /// - `input_index`: Absolute index at which unread source units begin.
  306|       |    /// - `output`: Target unit slice receiving converted values.
  307|       |    /// - `output_index`: Absolute index at which target output begins.
  308|       |    ///
  309|       |    /// # Returns
  310|       |    ///
  311|       |    /// Returns consumed and produced counts with the conversion stop reason.
  312|       |    ///
  313|       |    /// # Errors
  314|       |    ///
  315|       |    /// Returns a converter error for invalid indices, invalid or incomplete
  316|       |    /// final source input, or a decoder/encoder failure under the current
  317|       |    /// policy.
  318|      6|    pub fn transcode_eof(
  319|      6|        &mut self,
  320|      6|        input: &[D::Unit],
  321|      6|        input_index: usize,
  322|      6|        output: &mut [E::Unit],
  323|      6|        output_index: usize,
  324|      6|    ) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>> {
  325|      6|        self.engine.transcode_eof(input, input_index, output, output_index)
  326|      6|    }
  327|       |
  328|       |    /// Finishes internally retained output after EOF.
  329|       |    ///
  330|       |    /// Finalization delegates to the reusable converter engine. It drains
  331|       |    /// retained pending output, encodes source-side decode flush values, and
  332|       |    /// then finishes target-side encode hook state.
  333|       |    ///
  334|       |    /// # Parameters
  335|       |    ///
  336|       |    /// - `output`: Target unit slice for finalization output.
  337|       |    /// - `output_index`: Absolute target output index where writing starts.
  338|       |    ///
  339|       |    /// # Returns
  340|       |    ///
  341|       |    /// Returns the number of target units written by finalization.
  342|       |    ///
  343|       |    /// # Errors
  344|       |    ///
  345|       |    /// Returns a converter error for invalid output indices, insufficient
  346|       |    /// output capacity, or failures while draining pending values or
  347|       |    /// finishing either codec.
  348|     12|    pub fn finish(
  349|     12|        &mut self,
  350|     12|        output: &mut [E::Unit],
  351|     12|        output_index: usize,
  352|     12|    ) -> Result<usize, TranscodeConvertErrorOf<D, E>>
  353|     12|    where
  354|     12|        D::Value: Default,
  355|       |    {
  356|     12|        self.engine.finish(output, output_index)
  357|     12|    }
  358|       |}
  359|       |
  360|       |impl<D, E> Transcoder for CodecTranscodeConverter<D, E>
  361|       |where
  362|       |    D: Codec,
  363|       |    E: Codec<Value = D::Value>,
  364|       |    D::Value: Default,
  365|       |{
  366|       |    /// Source units accepted by the decoder codec.
  367|       |    type Input = D::Unit;
  368|       |    /// Target units produced by the encoder codec.
  369|       |    type Output = E::Unit;
  370|       |    /// Conversion failures retaining decoder, encoder, and pending-value
  371|       |    /// context.
  372|       |    type Error = TranscodeConvertErrorOf<D, E>;
  373|       |
  374|       |    /// Returns an upper bound for target units produced from `input_len` units.
  375|       |    ///
  376|       |    /// # Parameters
  377|       |    ///
  378|       |    /// - `input_len`: Source units the caller plans to convert.
  379|       |    ///
  380|       |    /// # Returns
  381|       |    ///
  382|       |    /// Returns a conservative upper bound for produced target units.
  383|       |    ///
  384|       |    /// # Errors
  385|       |    ///
  386|       |    /// Returns [`CapacityError::OutputLengthOverflow`] when component-bound
  387|       |    /// arithmetic overflows.
  388|       |    #[inline]
  389|      4|    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
  390|      4|        CodecTranscodeConverter::max_transcode_output_len(self, input_len)
  391|      4|    }
  392|       |
  393|       |    /// Returns the maximum target units emitted by finishing internal state.
  394|       |    ///
  395|       |    /// # Returns
  396|       |    ///
  397|       |    /// Returns a conservative upper bound valid for every reachable converter
  398|       |    /// state.
  399|       |    ///
  400|       |    /// # Errors
  401|       |    ///
  402|       |    /// Returns [`CapacityError::OutputLengthOverflow`] when component-bound
  403|       |    /// arithmetic overflows.
  404|       |    #[inline]
  405|      4|    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
  406|      4|        CodecTranscodeConverter::max_finish_output_len(self)
  407|      4|    }
  408|       |
  409|       |    /// Returns the maximum target units emitted when resetting stream state.
  410|       |    ///
  411|       |    /// # Returns
  412|       |    ///
  413|       |    /// Returns a conservative upper bound valid for every reachable converter
  414|       |    /// state.
  415|       |    ///
  416|       |    /// # Errors
  417|       |    ///
  418|       |    /// Returns [`CapacityError::OutputLengthOverflow`] when component-bound
  419|       |    /// arithmetic overflows.
  420|       |    #[inline]
  421|      4|    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
  422|      4|        CodecTranscodeConverter::max_reset_output_len(self)
  423|      4|    }
  424|       |
  425|       |    /// Clears retained pending output, resets component state, and emits
  426|       |    /// stream-start encode output.
  427|       |    ///
  428|       |    /// # Parameters
  429|       |    ///
  430|       |    /// - `output`: Target unit slice for stream-start output.
  431|       |    /// - `output_index`: Absolute target output index where writing starts.
  432|       |    ///
  433|       |    /// # Returns
  434|       |    ///
  435|       |    /// Returns the number of target units written while resetting component
  436|       |    /// state.
  437|       |    ///
  438|       |    /// # Errors
  439|       |    ///
  440|       |    /// Returns a converter error when the output range is invalid or too
  441|       |    /// small, or when decoder or encoder reset processing fails.
  442|      5|    fn reset(&mut self, output: &mut [E::Unit], output_index: usize) -> Result<usize, TranscodeConvertErrorOf<D, E>> {
  443|      5|        CodecTranscodeConverter::reset(self, output, output_index)
  444|      5|    }
  445|       |
  446|       |    /// Converts source units into target units.
  447|       |    ///
  448|       |    /// # Parameters
  449|       |    ///
  450|       |    /// - `input`: Source unit slice.
  451|       |    /// - `input_index`: Absolute source index where conversion starts.
  452|       |    /// - `output`: Target unit slice.
  453|       |    /// - `output_index`: Absolute target index where writing starts.
  454|       |    ///
  455|       |    /// # Returns
  456|       |    ///
  457|       |    /// Returns conversion progress for consumed/produced counters and stop
  458|       |    /// reason.
  459|       |    ///
  460|       |    /// # Errors
  461|       |    ///
  462|       |    /// Returns converter error when source or target indices are invalid, or
  463|       |    /// when decoding/encoding fails under current policy.
  464|      2|    fn transcode(
  465|      2|        &mut self,
  466|      2|        input: &[D::Unit],
  467|      2|        input_index: usize,
  468|      2|        output: &mut [E::Unit],
  469|      2|        output_index: usize,
  470|      2|    ) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>> {
  471|      2|        CodecTranscodeConverter::transcode(self, input, input_index, output, output_index)
  472|      2|    }
  473|       |
  474|       |    /// Transcodes an input segment while marking it as the end of the stream.
  475|       |    ///
  476|       |    /// This forwarding implementation delegates to the converter's EOF-aware
  477|       |    /// operation so buffered decoder state can be finalized before encoding.
  478|       |    ///
  479|       |    /// # Parameters
  480|       |    ///
  481|       |    /// - `input`: Final source unit slice; the caller must not supply more
  482|       |    ///   input later.
  483|       |    /// - `input_index`: Absolute index at which unread source units begin.
  484|       |    /// - `output`: Target unit slice receiving converted values.
  485|       |    /// - `output_index`: Absolute index at which target output begins.
  486|       |    ///
  487|       |    /// # Returns
  488|       |    ///
  489|       |    /// Returns consumed and produced counts with the conversion stop reason.
  490|       |    ///
  491|       |    /// # Errors
  492|       |    ///
  493|       |    /// Returns a converter error for invalid indices, invalid or incomplete
  494|       |    /// final source input, or a decoder/encoder failure under the current
  495|       |    /// policy.
  496|      4|    fn transcode_eof(
  497|      4|        &mut self,
  498|      4|        input: &[D::Unit],
  499|      4|        input_index: usize,
  500|      4|        output: &mut [E::Unit],
  501|      4|        output_index: usize,
  502|      4|    ) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>> {
  503|      4|        CodecTranscodeConverter::transcode_eof(self, input, input_index, output, output_index)
  504|      4|    }
  505|       |
  506|       |    /// Finishes internally retained output after EOF.
  507|       |    ///
  508|       |    /// # Parameters
  509|       |    ///
  510|       |    /// - `output`: Target unit slice for finalization output.
  511|       |    /// - `output_index`: Absolute target output index where writing starts.
  512|       |    ///
  513|       |    /// # Returns
  514|       |    ///
  515|       |    /// Returns the number of target units written by finalization.
  516|       |    ///
  517|       |    /// # Errors
  518|       |    ///
  519|       |    /// Returns a converter error for invalid output indices, insufficient
  520|       |    /// output capacity, or failures while draining pending values or
  521|       |    /// finishing either codec.
  522|      3|    fn finish(&mut self, output: &mut [E::Unit], output_index: usize) -> Result<usize, TranscodeConvertErrorOf<D, E>> {
  523|      3|        CodecTranscodeConverter::finish(self, output, output_index)
  524|      3|    }
  525|       |}
  526|       |
  527|       |impl<D, E> TranscodeConverter for CodecTranscodeConverter<D, E>
  528|       |where
  529|       |    D: Codec,
  530|       |    E: Codec<Value = D::Value>,
  531|       |    D::Value: Default,
  532|       |{
  533|       |    /// Domain error reported by the source codec.
  534|       |    type DecodeError = D::DecodeError;
  535|       |    /// Domain error reported by the target codec.
  536|       |    type EncodeError = E::EncodeError;
  537|       |    /// Decoded value transferred from source codec to target codec.
  538|       |    type Value = D::Value;
  539|       |}
  540|       |
  541|       |impl<D, E> Default for CodecTranscodeConverter<D, E>
  542|       |where
  543|       |    D: Codec,
  544|       |    E: Codec<Value = D::Value>,
  545|       |    TranscodeConvertEngine<D, E, CodecTranscodeDecodeHooks, CodecTranscodeEncodeHooks>: Default,
  546|       |{
  547|       |    /// Creates a default codec-backed buffered converter.
  548|       |    ///
  549|       |    /// # Returns
  550|       |    ///
  551|       |    /// Returns a converter with default codecs and hooks.
  552|       |    #[inline]
  553|      1|    fn default() -> Self {
  554|      1|        Self {
  555|      1|            engine: TranscodeConvertEngine::default(),
  556|      1|        }
  557|      1|    }
  558|       |}
  559|       |
  560|       |impl<D, E> fmt::Debug for CodecTranscodeConverter<D, E>
  561|       |where
  562|       |    D: Codec,
  563|       |    E: Codec<Value = D::Value>,
  564|       |    TranscodeConvertEngine<D, E, CodecTranscodeDecodeHooks, CodecTranscodeEncodeHooks>: fmt::Debug,
  565|       |{
  566|       |    /// Formats the wrapped converter engine for debugging.
  567|       |    ///
  568|       |    /// # Parameters
  569|       |    ///
  570|       |    /// - `f`: Destination formatter.
  571|       |    ///
  572|       |    /// # Returns
  573|       |    ///
  574|       |    /// Returns `fmt::Result` from the formatter.
  575|       |    ///
  576|       |    /// # Errors
  577|       |    ///
  578|       |    /// Returns the formatter error if writing the debug representation fails.
  579|      1|    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
  580|      1|        f.debug_struct("CodecTranscodeConverter")
  581|      1|            .field("engine", &self.engine)
  582|      1|            .finish()
  583|      1|    }
  584|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/adapter/codec_transcode_decoder.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Buffered decoder adapter backed by a low-level codec.
    9|       |
   10|       |use super::super::engine::TranscodeDecodeEngine;
   11|       |use super::CodecTranscodeDecodeHooks;
   12|       |use crate::CapacityError;
   13|       |use crate::Codec;
   14|       |use crate::TranscodeDecodeErrorOf;
   15|       |use crate::TranscodeDecoder;
   16|       |use crate::TranscodeProgress;
   17|       |use crate::Transcoder;
   18|       |
   19|       |/// Decodes encoded units into caller-provided value buffers by using a
   20|       |/// [`Codec`].
   21|       |///
   22|       |/// `CodecTranscodeDecoder` is a policy-free bridge from the low-level unchecked
   23|       |/// [`Codec`] contract to [`Transcoder`] and [`TranscodeDecoder`]. It
   24|       |/// leaves incomplete input tails in the caller-provided input slice while the
   25|       |/// stream remains open. Call [`Transcoder::transcode_eof`] after upstream EOF
   26|       |/// so codecs can apply their [`Codec::decode_eof`] policy to a trailing prefix.
   27|       |///
   28|       |/// # Type Parameters
   29|       |///
   30|       |/// - `C`: Low-level codec used to decode values.
   31|       |///
   32|       |/// # Examples
   33|       |///
   34|       |/// ```
   35|       |/// use std::convert::Infallible;
   36|       |/// use std::num::NonZeroUsize;
   37|       |///
   38|       |/// use qubit_codec::Codec;
   39|       |/// use qubit_codec::CodecTranscodeDecoder;
   40|       |/// use qubit_codec::DecodeFailure;
   41|       |/// use qubit_codec::Transcoder;
   42|       |///
   43|       |/// struct Identity;
   44|       |/// impl Codec for Identity {
   45|       |///     type Value = u8;
   46|       |///     type Unit = u8;
   47|       |///     type DecodeError = Infallible;
   48|       |///     type EncodeError = Infallible;
   49|       |///     const MIN_UNITS_PER_VALUE: usize = 1;
   50|       |///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
   51|       |///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
   52|       |///     unsafe fn decode(&mut self, input: &[u8], index: usize)
   53|       |///         -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
   54|       |///         Ok((input[index], NonZeroUsize::new(1).unwrap()))
   55|       |///     }
   56|       |///     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize)
   57|       |///         -> Result<usize, Infallible> {
   58|       |///         output[index] = *value;
   59|       |///         Ok(1)
   60|       |///     }
   61|       |/// }
   62|       |/// let mut decoder = CodecTranscodeDecoder::new(Identity);
   63|       |/// let mut output = [0; 2];
   64|       |/// decoder.reset(&mut output, 0).unwrap();
   65|       |/// decoder.transcode(&[4, 7], 0, &mut output, 0).unwrap();
   66|       |/// assert_eq!(output, [4, 7]);
   67|       |/// ```
   68|       |#[derive(Debug)]
   69|       |pub struct CodecTranscodeDecoder<C> {
   70|       |    /// Common buffered decoding engine.
   71|       |    engine: TranscodeDecodeEngine<C, CodecTranscodeDecodeHooks>,
   72|       |}
   73|       |
   74|       |impl<C> Default for CodecTranscodeDecoder<C>
   75|       |where
   76|       |    C: Codec + Default,
   77|       |{
   78|       |    /// Creates an adapter with the same codec-bound checks as [`Self::new`].
   79|       |    ///
   80|       |    /// # Returns
   81|       |    ///
   82|       |    /// Returns a fresh adapter containing a default codec.
   83|       |    ///
   84|       |    /// # Compile-Time Checks
   85|       |    ///
   86|       |    /// Fails to compile when the codec declares a zero decode unit bound or
   87|       |    /// when [`Codec::MIN_UNITS_PER_VALUE`] exceeds
   88|       |    /// [`Codec::MAX_DECODE_UNITS_PER_VALUE`], matching [`Self::new`].
   89|       |    ///
   90|       |    /// A legal codec can use the default constructor:
   91|       |    ///
   92|       |    /// ```rust
   93|       |    /// use core::convert::Infallible;
   94|       |    /// use core::num::NonZeroUsize;
   95|       |    /// use qubit_codec::{Codec, CodecTranscodeDecoder, DecodeFailure};
   96|       |    ///
   97|       |    /// #[derive(Default)]
   98|       |    /// struct ByteCodec;
   99|       |    /// impl Codec for ByteCodec {
  100|       |    ///     type Value = u8;
  101|       |    ///     type Unit = u8;
  102|       |    ///     type DecodeError = Infallible;
  103|       |    ///     type EncodeError = Infallible;
  104|       |    ///     const MIN_UNITS_PER_VALUE: usize = 1;
  105|       |    ///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
  106|       |    ///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
  107|       |    ///     unsafe fn decode(&mut self, input: &[u8], index: usize) -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
  108|       |    ///         Ok((input[index], NonZeroUsize::MIN))
  109|       |    ///     }
  110|       |    ///     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Infallible> {
  111|       |    ///         output[index] = *value;
  112|       |    ///         Ok(1)
  113|       |    ///     }
  114|       |    /// }
  115|       |    /// let _decoder = CodecTranscodeDecoder::<ByteCodec>::default();
  116|       |    /// ```
  117|       |    ///
  118|       |    /// A zero maximum decode width is rejected even when the minimum is valid:
  119|       |    ///
  120|       |    /// ```compile_fail
  121|       |    /// use core::convert::Infallible;
  122|       |    /// use core::num::NonZeroUsize;
  123|       |    /// use qubit_codec::{Codec, CodecTranscodeDecoder, DecodeFailure};
  124|       |    /// #[derive(Default)] struct BadCodec;
  125|       |    /// impl Codec for BadCodec {
  126|       |    ///     type Value = u8; type Unit = u8; type DecodeError = Infallible; type EncodeError = Infallible;
  127|       |    ///     const MIN_UNITS_PER_VALUE: usize = 1; const MAX_ENCODE_UNITS_PER_VALUE: usize = 1; const MAX_DECODE_UNITS_PER_VALUE: usize = 0;
  128|       |    ///     unsafe fn decode(&mut self, input: &[u8], index: usize) -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> { Ok((input[index], NonZeroUsize::MIN)) }
  129|       |    ///     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Infallible> { output[index] = *value; Ok(1) }
  130|       |    /// }
  131|       |    /// let _ = CodecTranscodeDecoder::<BadCodec>::default();
  132|       |    /// ```
  133|       |    #[inline]
  134|      1|    fn default() -> Self {
  135|      1|        Self::new(C::default())
  136|      1|    }
  137|       |}
  138|       |
  139|       |impl<C> CodecTranscodeDecoder<C>
  140|       |where
  141|       |    C: Codec,
  142|       |{
  143|       |    /// Creates a buffered decoder backed by `codec`.
  144|       |    ///
  145|       |    /// # Parameters
  146|       |    ///
  147|       |    /// - `codec`: Low-level codec used to decode values.
  148|       |    ///
  149|       |    /// # Returns
  150|       |    ///
  151|       |    /// Returns a buffered decoder adapter for the supplied codec.
  152|       |    #[inline]
  153|       |    #[must_use]
  154|     31|    pub fn new(codec: C) -> Self {
  155|     31|        Self {
  156|     31|            engine: TranscodeDecodeEngine::new(codec, CodecTranscodeDecodeHooks),
  157|     31|        }
  158|     31|    }
  159|       |
  160|       |    /// Returns a shared reference to the wrapped codec.
  161|       |    ///
  162|       |    /// # Returns
  163|       |    ///
  164|       |    /// A borrow of the codec that remains valid while this adapter is borrowed.
  165|       |    #[inline]
  166|       |    #[must_use]
  167|      1|    pub fn codec(&self) -> &C {
  168|      1|        self.engine.codec()
  169|      1|    }
  170|       |
  171|       |    /// Returns a mutable reference to the wrapped codec.
  172|       |    ///
  173|       |    /// Mutating a codec during an active stream can invalidate that stream's
  174|       |    /// assumptions; reset the adapter before continuing with the new codec
  175|       |    /// configuration.
  176|       |    ///
  177|       |    /// # Returns
  178|       |    ///
  179|       |    /// An exclusive borrow of the wrapped codec; no allocation is performed.
  180|       |    #[inline]
  181|       |    #[must_use]
  182|      1|    pub fn codec_mut(&mut self) -> &mut C {
  183|      1|        self.engine.codec_mut()
  184|      1|    }
  185|       |
  186|       |    /// Consumes the adapter and returns its wrapped codec.
  187|       |    ///
  188|       |    /// Any buffered lifecycle state and internal hooks are discarded.
  189|       |    ///
  190|       |    /// # Returns
  191|       |    ///
  192|       |    /// The owned codec, with its codec-specific state preserved.
  193|       |    #[inline]
  194|       |    #[must_use]
  195|      1|    pub fn into_codec(self) -> C {
  196|      1|        let (codec, _) = self.engine.into_parts();
  197|      1|        codec
  198|      1|    }
  199|       |}
  200|       |
  201|       |impl<C> Transcoder for CodecTranscodeDecoder<C>
  202|       |where
  203|       |    C: Codec,
  204|       |{
  205|       |    /// Encoded units borrowed from the caller input.
  206|       |    type Input = C::Unit;
  207|       |    /// Decoded values written into caller-provided storage.
  208|       |    type Output = C::Value;
  209|       |    /// Contract or codec failures reported by the decoding engine.
  210|       |    type Error = TranscodeDecodeErrorOf<C>;
  211|       |
  212|       |    /// Returns an upper bound for decoded values produced from `input_len`
  213|       |    /// units.
  214|       |    ///
  215|       |    /// # Parameters
  216|       |    ///
  217|       |    /// - `input_len`: Source units the caller plans to decode.
  218|       |    ///
  219|       |    /// # Returns
  220|       |    ///
  221|       |    /// Returns a conservative upper bound for decoded values.
  222|       |    ///
  223|       |    /// # Errors
  224|       |    ///
  225|       |    /// Returns a capacity error if the requested bound cannot fit in `usize`.
  226|       |    #[inline]
  227|     18|    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
  228|     18|        self.engine.max_transcode_output_len(input_len)
  229|     18|    }
  230|       |
  231|       |    /// Returns the global maximum values emitted by finishing internal state.
  232|       |    ///
  233|       |    /// # Returns
  234|       |    ///
  235|       |    /// Returns the number of values that may still be emitted by finishing
  236|       |    /// state.
  237|       |    ///
  238|       |    /// # Errors
  239|       |    ///
  240|       |    /// Returns a capacity error if the combined finish bound overflows.
  241|       |    #[inline]
  242|     11|    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
  243|     11|        self.engine.max_finish_output_len()
  244|     11|    }
  245|       |
  246|       |    /// Returns the global maximum values emitted when resetting internal state.
  247|       |    ///
  248|       |    /// # Returns
  249|       |    ///
  250|       |    /// The maximum reset output capacity required by the engine.
  251|       |    ///
  252|       |    /// # Errors
  253|       |    ///
  254|       |    /// Returns a capacity error if the combined reset bound overflows.
  255|       |    #[inline]
  256|     11|    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
  257|     11|        self.engine.max_reset_output_len()
  258|     11|    }
  259|       |
  260|       |    /// Runs before-reset cleanup for decoder state.
  261|       |    ///
  262|       |    /// # Parameters
  263|       |    ///
  264|       |    /// - `output`: Destination storage for reset values.
  265|       |    /// - `output_index`: First destination position to write.
  266|       |    ///
  267|       |    /// # Returns
  268|       |    ///
  269|       |    /// The number of reset values written.
  270|       |    ///
  271|       |    /// # Errors
  272|       |    ///
  273|       |    /// Returns a contract error for invalid indices or insufficient capacity,
  274|       |    /// or a codec error when resetting codec state fails.
  275|       |    #[inline]
  276|     29|    fn reset(&mut self, output: &mut [C::Value], output_index: usize) -> Result<usize, TranscodeDecodeErrorOf<C>> {
  277|     29|        self.engine.reset(output, output_index)
  278|     29|    }
  279|       |
  280|       |    /// Decodes source units into logical values.
  281|       |    ///
  282|       |    /// # Parameters
  283|       |    ///
  284|       |    /// - `input`: Source unit slice.
  285|       |    /// - `input_index`: Absolute source index where decoding starts.
  286|       |    /// - `output`: Destination value slice.
  287|       |    /// - `output_index`: Absolute output value index where writing starts.
  288|       |    ///
  289|       |    /// # Returns
  290|       |    ///
  291|       |    /// Returns conversion progress for consumed and written counters.
  292|       |    ///
  293|       |    /// # Errors
  294|       |    ///
  295|       |    /// Returns a decode error when indices are invalid or when conversion fails
  296|       |    /// under hook policy.
  297|       |    #[inline]
  298|     19|    fn transcode(
  299|     19|        &mut self,
  300|     19|        input: &[C::Unit],
  301|     19|        input_index: usize,
  302|     19|        output: &mut [C::Value],
  303|     19|        output_index: usize,
  304|     19|    ) -> Result<TranscodeProgress, TranscodeDecodeErrorOf<C>> {
  305|     19|        self.engine.transcode(input, input_index, output, output_index)
  306|     19|    }
  307|       |
  308|       |    /// Transcodes an input segment while marking it as the end of the stream.
  309|       |    ///
  310|       |    /// This forwarding implementation lets the engine apply EOF-specific
  311|       |    /// decoder behavior before returning progress to the caller.
  312|       |    ///
  313|       |    /// # Parameters
  314|       |    ///
  315|       |    /// - `input`: Final source unit slice.
  316|       |    /// - `input_index`: First source position to consume.
  317|       |    /// - `output`: Destination value storage.
  318|       |    /// - `output_index`: First destination position to write.
  319|       |    ///
  320|       |    /// # Returns
  321|       |    ///
  322|       |    /// Consumed and written counts and the engine progress status.
  323|       |    ///
  324|       |    /// # Errors
  325|       |    ///
  326|       |    /// Returns contract errors for invalid indices or lifecycle transitions,
  327|       |    /// or decode errors for malformed or incomplete final input.
  328|       |    #[inline]
  329|      8|    fn transcode_eof(
  330|      8|        &mut self,
  331|      8|        input: &[C::Unit],
  332|      8|        input_index: usize,
  333|      8|        output: &mut [C::Value],
  334|      8|        output_index: usize,
  335|      8|    ) -> Result<TranscodeProgress, TranscodeDecodeErrorOf<C>> {
  336|      8|        self.engine.transcode_eof(input, input_index, output, output_index)
  337|      8|    }
  338|       |
  339|       |    /// Finishes internally retained output after EOF.
  340|       |    ///
  341|       |    /// # Parameters
  342|       |    ///
  343|       |    /// - `output`: Destination value slice for final retained values.
  344|       |    /// - `output_index`: Absolute output value index where writing starts.
  345|       |    ///
  346|       |    /// # Returns
  347|       |    ///
  348|       |    /// Returns the number of values written by finalization.
  349|       |    ///
  350|       |    /// # Errors
  351|       |    ///
  352|       |    /// Returns contract errors for invalid indices, insufficient capacity, or
  353|       |    /// invalid lifecycle transitions, or a codec error for invalid EOF state.
  354|       |    #[inline]
  355|     14|    fn finish(&mut self, output: &mut [C::Value], output_index: usize) -> Result<usize, TranscodeDecodeErrorOf<C>> {
  356|     14|        self.engine.finish(output, output_index)
  357|     14|    }
  358|       |}
  359|       |
  360|       |impl<C> TranscodeDecoder for CodecTranscodeDecoder<C>
  361|       |where
  362|       |    C: Codec,
  363|       |{
  364|       |    /// Codec-specific failure retained by the decoder error wrapper.
  365|       |    type DecodeError = C::DecodeError;
  366|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/adapter/codec_transcode_encoder.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Buffered encoder adapter backed by a low-level codec.
    9|       |
   10|       |use super::super::engine::TranscodeEncodeEngine;
   11|       |use super::CodecTranscodeEncodeHooks;
   12|       |use crate::CapacityError;
   13|       |use crate::Codec;
   14|       |use crate::TranscodeEncodeErrorOf;
   15|       |use crate::TranscodeEncoder;
   16|       |use crate::TranscodeProgress;
   17|       |use crate::Transcoder;
   18|       |
   19|       |/// Encodes values into caller-provided output units by using a [`Codec`].
   20|       |///
   21|       |/// `CodecTranscodeEncoder` is the default bridge from the low-level unchecked
   22|       |/// [`Codec`] contract to the buffered [`Transcoder`] and
   23|       |/// [`TranscodeEncoder`] contracts. It encodes complete values only; when the
   24|       |/// remaining output capacity is smaller than `codec.encode_len(value)`, it
   25|       |/// stops before consuming that input value and reports
   26|       |/// [`crate::TranscodeStatus::NeedOutput`].
   27|       |///
   28|       |/// # Type Parameters
   29|       |///
   30|       |/// - `C`: Low-level codec used to encode values.
   31|       |///
   32|       |/// # Examples
   33|       |///
   34|       |/// ```
   35|       |/// use core::convert::Infallible;
   36|       |/// use core::num::NonZeroUsize;
   37|       |///
   38|       |/// use qubit_codec::Codec;
   39|       |/// use qubit_codec::CodecTranscodeEncoder;
   40|       |/// use qubit_codec::DecodeFailure;
   41|       |/// use qubit_codec::Transcoder;
   42|       |///
   43|       |/// # struct ByteCodec;
   44|       |/// # impl Codec for ByteCodec {
   45|       |/// #     type Value = u8;
   46|       |/// #     type Unit = u8;
   47|       |/// #     type DecodeError = Infallible;
   48|       |/// #     type EncodeError = Infallible;
   49|       |/// #     const MIN_UNITS_PER_VALUE: usize = 1;
   50|       |/// #     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
   51|       |/// #     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
   52|       |/// #     unsafe fn decode(&mut self, input: &[u8], index: usize)
   53|       |/// #         -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
   54|       |/// #         Ok((input[index], NonZeroUsize::new(1).unwrap()))
   55|       |/// #     }
   56|       |/// #     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize)
   57|       |/// #         -> Result<usize, Infallible> {
   58|       |/// #         output[index] = *value;
   59|       |/// #         Ok(1)
   60|       |/// #     }
   61|       |/// # }
   62|       |/// let mut encoder = CodecTranscodeEncoder::new(ByteCodec);
   63|       |/// let mut output = [0; 2];
   64|       |/// assert_eq!(encoder.reset(&mut output, 0).unwrap(), 0);
   65|       |/// let progress = encoder.transcode(&[7, 9], 0, &mut output, 0).unwrap();
   66|       |/// assert_eq!(progress.written(), 2);
   67|       |/// assert_eq!(output, [7, 9]);
   68|       |/// ```
   69|       |#[derive(Debug)]
   70|       |pub struct CodecTranscodeEncoder<C> {
   71|       |    /// Common buffered encoding engine.
   72|       |    engine: TranscodeEncodeEngine<C, CodecTranscodeEncodeHooks>,
   73|       |}
   74|       |
   75|       |impl<C> CodecTranscodeEncoder<C>
   76|       |where
   77|       |    C: Codec,
   78|       |{
   79|       |    /// Creates a buffered encoder backed by `codec`.
   80|       |    ///
   81|       |    /// # Parameters
   82|       |    ///
   83|       |    /// - `codec`: Low-level codec used to encode values.
   84|       |    ///
   85|       |    /// # Returns
   86|       |    ///
   87|       |    /// Returns a buffered encoder adapter for the supplied codec.
   88|       |    #[inline]
   89|       |    #[must_use]
   90|     16|    pub fn new(codec: C) -> Self {
   91|     16|        Self {
   92|     16|            engine: TranscodeEncodeEngine::new(codec, CodecTranscodeEncodeHooks),
   93|     16|        }
   94|     16|    }
   95|       |
   96|       |    /// Borrows the codec without allocating or changing stream state.
   97|       |    ///
   98|       |    /// # Returns
   99|       |    ///
  100|       |    /// The wrapped codec, borrowed for the lifetime of `self`.
  101|       |    #[inline]
  102|       |    #[must_use]
  103|      1|    pub fn codec(&self) -> &C {
  104|      1|        self.engine.codec()
  105|      1|    }
  106|       |
  107|       |    /// Returns a mutable reference to the wrapped codec.
  108|       |    ///
  109|       |    /// Mutating a codec during an active stream can invalidate that stream's
  110|       |    /// assumptions; reset the adapter before continuing with the new codec
  111|       |    /// configuration.
  112|       |    ///
  113|       |    /// # Returns
  114|       |    ///
  115|       |    /// An exclusive borrow of the wrapped codec tied to `self`.
  116|       |    #[inline]
  117|       |    #[must_use]
  118|      1|    pub fn codec_mut(&mut self) -> &mut C {
  119|      1|        self.engine.codec_mut()
  120|      1|    }
  121|       |
  122|       |    /// Consumes the adapter and returns its wrapped codec.
  123|       |    ///
  124|       |    /// Any buffered lifecycle state and internal hooks are discarded.
  125|       |    ///
  126|       |    /// # Returns
  127|       |    ///
  128|       |    /// The owned codec, without resetting or finishing it.
  129|       |    #[inline]
  130|       |    #[must_use]
  131|      1|    pub fn into_codec(self) -> C {
  132|      1|        let (codec, _) = self.engine.into_parts();
  133|      1|        codec
  134|      1|    }
  135|       |}
  136|       |
  137|       |impl<C> Transcoder for CodecTranscodeEncoder<C>
  138|       |where
  139|       |    C: Codec,
  140|       |{
  141|       |    /// Values accepted by the wrapped codec.
  142|       |    type Input = C::Value;
  143|       |    /// Representation units emitted by the wrapped codec.
  144|       |    type Output = C::Unit;
  145|       |    /// Domain, contract, and codec failures during buffered encoding.
  146|       |    type Error = TranscodeEncodeErrorOf<C>;
  147|       |
  148|       |    /// Gets the maximum number of output units needed for `input_len`
  149|       |    /// values.
  150|       |    ///
  151|       |    /// # Parameters
  152|       |    ///
  153|       |    /// - `input_len`: Logical input values the caller plans to encode.
  154|       |    ///
  155|       |    /// # Returns
  156|       |    ///
  157|       |    /// A conservative upper bound for output units.
  158|       |    ///
  159|       |    /// # Errors
  160|       |    ///
  161|       |    /// Returns [`CapacityError`] if the engine cannot represent the bound in
  162|       |    /// `usize`.
  163|       |    #[inline]
  164|      4|    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
  165|      4|        self.engine.max_transcode_output_len(input_len)
  166|      4|    }
  167|       |
  168|       |    /// Gets the global maximum units emitted when resetting internal state.
  169|       |    ///
  170|       |    /// # Returns
  171|       |    ///
  172|       |    /// The maximum units emitted when resetting internal state.
  173|       |    ///
  174|       |    /// # Errors
  175|       |    ///
  176|       |    /// Returns [`CapacityError`] if the engine cannot represent the bound in
  177|       |    /// `usize`.
  178|       |    #[inline]
  179|      2|    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
  180|      2|        self.engine.max_reset_output_len()
  181|      2|    }
  182|       |
  183|       |    /// Gets the global maximum units emitted by finishing internal state.
  184|       |    ///
  185|       |    /// # Returns
  186|       |    ///
  187|       |    /// The number of units that may be emitted by finishing state.
  188|       |    ///
  189|       |    /// # Errors
  190|       |    ///
  191|       |    /// Returns [`CapacityError`] if the engine cannot represent the bound in
  192|       |    /// `usize`.
  193|       |    #[inline]
  194|      2|    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
  195|      2|        self.engine.max_finish_output_len()
  196|      2|    }
  197|       |
  198|       |    /// Runs before-reset cleanup and emits stream-start output.
  199|       |    ///
  200|       |    /// # Parameters
  201|       |    ///
  202|       |    /// - `output`: Destination for codec reset units.
  203|       |    /// - `output_index`: Start position in `output`.
  204|       |    ///
  205|       |    /// # Returns
  206|       |    ///
  207|       |    /// The number of reset units written; previous lifecycle state is
  208|       |    /// discarded.
  209|       |    ///
  210|       |    /// # Errors
  211|       |    ///
  212|       |    /// Returns an error for an invalid index, insufficient reset capacity,
  213|       |    /// a violated codec contract, or a codec reset failure.
  214|       |    #[inline]
  215|     15|    fn reset(&mut self, output: &mut [C::Unit], output_index: usize) -> Result<usize, TranscodeEncodeErrorOf<C>> {
  216|     15|        self.engine.reset(output, output_index)
  217|     15|    }
  218|       |
  219|       |    /// Encodes values into the supplied output buffer.
  220|       |    ///
  221|       |    /// # Parameters
  222|       |    ///
  223|       |    /// - `input`: Input value slice.
  224|       |    /// - `input_index`: Absolute input index where encoding starts.
  225|       |    /// - `output`: Destination unit slice.
  226|       |    /// - `output_index`: Absolute output index where writing starts.
  227|       |    ///
  228|       |    /// # Returns
  229|       |    ///
  230|       |    /// Returns conversion progress for consumed input and produced output
  231|       |    /// units.
  232|       |    ///
  233|       |    /// # Errors
  234|       |    ///
  235|       |    /// Returns an encode error when indices are invalid or when encoding cannot
  236|       |    /// continue under current policy.
  237|       |    #[inline]
  238|     13|    fn transcode(
  239|     13|        &mut self,
  240|     13|        input: &[C::Value],
  241|     13|        input_index: usize,
  242|     13|        output: &mut [C::Unit],
  243|     13|        output_index: usize,
  244|     13|    ) -> Result<TranscodeProgress, TranscodeEncodeErrorOf<C>> {
  245|     13|        self.engine.transcode(input, input_index, output, output_index)
  246|     13|    }
  247|       |
  248|       |    /// Finishes internally retained output after EOF.
  249|       |    ///
  250|       |    /// # Parameters
  251|       |    ///
  252|       |    /// - `output`: Destination unit slice for finalization output.
  253|       |    /// - `output_index`: Absolute output index where writing starts.
  254|       |    ///
  255|       |    /// # Returns
  256|       |    ///
  257|       |    /// Returns the number of units written by finalization.
  258|       |    ///
  259|       |    /// # Errors
  260|       |    ///
  261|       |    /// Returns a finish error if retained output cannot be fully emitted.
  262|       |    #[inline]
  263|      2|    fn finish(&mut self, output: &mut [C::Unit], output_index: usize) -> Result<usize, TranscodeEncodeErrorOf<C>> {
  264|      2|        self.engine.finish(output, output_index)
  265|      2|    }
  266|       |}
  267|       |
  268|       |impl<C> TranscodeEncoder for CodecTranscodeEncoder<C>
  269|       |where
  270|       |    C: Codec,
  271|       |{
  272|       |    /// Codec-specific encoding failure carried by the adapter.
  273|       |    type EncodeError = C::EncodeError;
  274|       |}
  275|       |
  276|       |impl<C> Default for CodecTranscodeEncoder<C>
  277|       |where
  278|       |    C: Codec + Default,
  279|       |{
  280|       |    /// Creates a default codec-backed buffered encoder.
  281|       |    ///
  282|       |    /// # Returns
  283|       |    ///
  284|       |    /// Returns an encoder backed by `C::default()`.
  285|       |    #[inline]
  286|      1|    fn default() -> Self {
  287|      1|        Self::new(C::default())
  288|      1|    }
  289|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/adapter/internal/codec_transcode_decode_hooks.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Policy hooks used by the default codec-backed buffered decoder.
    9|       |
   10|       |use core::num::NonZeroUsize;
   11|       |
   12|       |use super::super::super::engine::DecodeContext;
   13|       |use super::super::super::engine::DecodeInvalidAction;
   14|       |use super::super::super::engine::TranscodeDecodeHooks;
   15|       |use crate::CapacityError;
   16|       |use crate::Codec;
   17|       |use crate::TranscodeDecodeErrorOf;
   18|       |
   19|       |/// Policy hooks for [`crate::CodecTranscodeDecoder`].
   20|       |#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
   21|       |pub(in crate::transcode) struct CodecTranscodeDecodeHooks;
   22|       |
   23|       |impl<C> TranscodeDecodeHooks<C> for CodecTranscodeDecodeHooks
   24|       |where
   25|       |    C: Codec,
   26|       |{
   27|       |    /// Returns the strict decode capacity, including one possible EOF-tail
   28|       |    /// slot.
   29|       |    ///
   30|       |    /// Rejected tails may still require an output slot before policy dispatch.
   31|       |    /// Reset and finish output are counted independently by the engine.
   32|       |    ///
   33|       |    /// # Parameters
   34|       |    ///
   35|       |    /// - `_codec`: Codec whose non-zero minimum width is validated by the
   36|       |    ///   engine.
   37|       |    /// - `input_len`: Source units planned for the streaming phase.
   38|       |    ///
   39|       |    /// # Returns
   40|       |    ///
   41|       |    /// Returns the source length divided by the minimum width, rounded up.
   42|       |    #[inline]
   43|     28|    fn max_transcode_output_len(&self, _codec: &C, input_len: usize) -> Result<usize, CapacityError> {
   44|     28|        Ok(input_len.div_ceil(C::MIN_UNITS_PER_VALUE))
   45|     28|    }
   46|       |
   47|       |    /// Converts codec decode failures into strict buffered decode errors.
   48|       |    ///
   49|       |    /// # Parameters
   50|       |    ///
   51|       |    /// - `_codec`: Low-level codec instance.
   52|       |    /// - `_error`: Invalid domain error produced by the low-level codec.
   53|       |    /// - `_consumed`: Invalid units that a non-strict policy may consume.
   54|       |    /// - `_context`: Decoding context carrying input position.
   55|       |    ///
   56|       |    /// # Returns
   57|       |    ///
   58|       |    /// Returns the strict invalid-decode policy action.
   59|       |    ///
   60|       |    /// # Errors
   61|       |    ///
   62|       |    /// This strict hook never returns an error; it always rejects invalid
   63|       |    /// input.
   64|       |    #[inline]
   65|      3|    fn handle_invalid_decode(
   66|      3|        &mut self,
   67|      3|        _codec: &mut C,
   68|      3|        _error: &C::DecodeError,
   69|      3|        _consumed: Option<NonZeroUsize>,
   70|      3|        _context: DecodeContext,
   71|      3|    ) -> Result<DecodeInvalidAction<C::Value>, TranscodeDecodeErrorOf<C>> {
   72|      3|        Ok(DecodeInvalidAction::Reject)
   73|      3|    }
   74|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/adapter/internal/codec_transcode_encode_hooks.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Policy hooks used by the default codec-backed buffered encoder.
    9|       |
   10|       |use super::super::super::engine::EncodeContext;
   11|       |use super::super::super::engine::EncodeUnencodableAction;
   12|       |use super::super::super::engine::TranscodeEncodeHooks;
   13|       |use crate::Codec;
   14|       |use crate::TranscodeEncodeErrorOf;
   15|       |
   16|       |/// Policy hooks for [`crate::CodecTranscodeEncoder`].
   17|       |#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
   18|       |pub(in crate::transcode) struct CodecTranscodeEncodeHooks;
   19|       |
   20|       |impl<C> TranscodeEncodeHooks<C> for CodecTranscodeEncodeHooks
   21|       |where
   22|       |    C: Codec,
   23|       |{
   24|       |    /// Rejects values outside the wrapped codec's encodable domain.
   25|       |    ///
   26|       |    /// # Parameters
   27|       |    ///
   28|       |    /// * `_codec` - The codec reporting the value; it is not inspected or
   29|       |    ///   mutated.
   30|       |    /// * `_context` - The rejected value and progress, unused by this fixed
   31|       |    ///   policy.
   32|       |    ///
   33|       |    /// # Returns
   34|       |    ///
   35|       |    /// Always returns `Ok(EncodeUnencodableAction::Reject)` without side
   36|       |    /// effects.
   37|       |    ///
   38|       |    /// # Errors
   39|       |    ///
   40|       |    /// This policy never returns an error directly; the engine applies
   41|       |    /// rejection.
   42|       |    #[inline]
   43|      3|    fn handle_unencodable_encode(
   44|      3|        &mut self,
   45|      3|        _codec: &mut C,
   46|      3|        _context: &EncodeContext<'_, C::Value>,
   47|      3|    ) -> Result<EncodeUnencodableAction<C::Value>, TranscodeEncodeErrorOf<C>> {
   48|      3|        Ok(EncodeUnencodableAction::Reject)
   49|      3|    }
   50|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/engine/decode_context.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Decode context passed to buffered decoder policy hooks.
    9|       |
   10|       |/// Context for one codec decode attempt inside a buffered decoder engine.
   11|       |///
   12|       |/// Input positions are source-unit indexes. Output positions use the driving
   13|       |/// engine's output domain: decoded values for [`TranscodeDecodeEngine`], and
   14|       |/// target encoding units for [`TranscodeConvertEngine`]. `output_written` is
   15|       |/// relative to the current call and does not count intermediate decoded
   16|       |/// values in a converter.
   17|       |///
   18|       |/// [`TranscodeDecodeEngine`]: crate::engine::TranscodeDecodeEngine
   19|       |/// [`TranscodeConvertEngine`]: crate::engine::TranscodeConvertEngine
   20|       |///
   21|       |/// # Examples
   22|       |///
   23|       |/// ```
   24|       |/// use qubit_codec::engine::DecodeContext;
   25|       |///
   26|       |/// let context = DecodeContext::new(0, 0, 0, 0, 2);
   27|       |/// assert_eq!(context.available(), 2);
   28|       |/// ```
   29|       |#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
   30|       |pub struct DecodeContext {
   31|       |    /// Absolute source index where this `transcode` call starts.
   32|       |    input_start: usize,
   33|       |    /// Absolute source index where the attempted value starts.
   34|       |    input_index: usize,
   35|       |    /// Absolute output index where this call starts, in the driving engine's
   36|       |    /// output domain.
   37|       |    output_start: usize,
   38|       |    /// Absolute output index where the next output would be written, in the
   39|       |    /// driving engine's output domain.
   40|       |    output_index: usize,
   41|       |    /// Units visible to the codec from `input_index`.
   42|       |    available: usize,
   43|       |}
   44|       |
   45|       |impl DecodeContext {
   46|       |    /// Creates a decode context.
   47|       |    ///
   48|       |    /// # Parameters
   49|       |    ///
   50|       |    /// - `input_start`: Absolute source index where this `transcode` call
   51|       |    ///   starts.
   52|       |    /// - `input_index`: Absolute source index where the attempted value starts.
   53|       |    /// - `output_start`: Absolute output index where this call starts, in the
   54|       |    ///   driving engine's output domain.
   55|       |    /// - `output_index`: Absolute output index where the next output would be
   56|       |    ///   written, in the driving engine's output domain.
   57|       |    /// - `available`: Units visible to the codec from `input_index`.
   58|       |    ///
   59|       |    /// # Returns
   60|       |    ///
   61|       |    /// Returns a decode context.
   62|       |    ///
   63|       |    /// # Panics
   64|       |    ///
   65|       |    /// Panics when `input_index < input_start` or
   66|       |    /// `output_index < output_start`.
   67|       |    #[inline]
   68|       |    #[must_use]
   69|  4.51k|    pub const fn new(
   70|  4.51k|        input_start: usize,
   71|  4.51k|        input_index: usize,
   72|  4.51k|        output_start: usize,
   73|  4.51k|        output_index: usize,
   74|  4.51k|        available: usize,
   75|  4.51k|    ) -> Self {
   76|  4.51k|        assert!(
   77|  4.51k|            input_start <= input_index,
   78|       |            "decode context input index must not precede input start",
   79|       |        );
   80|  4.51k|        assert!(
   81|  4.51k|            output_start <= output_index,
   82|       |            "decode context output index must not precede output start",
   83|       |        );
   84|  4.51k|        Self {
   85|  4.51k|            input_start,
   86|  4.51k|            input_index,
   87|  4.51k|            output_start,
   88|  4.51k|            output_index,
   89|  4.51k|            available,
   90|  4.51k|        }
   91|  4.51k|    }
   92|       |
   93|       |    /// Returns the absolute source index where this `transcode` call starts.
   94|       |    ///
   95|       |    /// # Returns
   96|       |    ///
   97|       |    /// Returns the input start index.
   98|       |    #[inline]
   99|       |    #[must_use]
  100|      2|    pub const fn input_start(self) -> usize {
  101|      2|        self.input_start
  102|      2|    }
  103|       |
  104|       |    /// Returns the absolute source index where the attempted value starts.
  105|       |    ///
  106|       |    /// # Returns
  107|       |    ///
  108|       |    /// Returns the current input index.
  109|       |    #[inline]
  110|       |    #[must_use]
  111|  8.40k|    pub const fn input_index(self) -> usize {
  112|  8.40k|        self.input_index
  113|  8.40k|    }
  114|       |
  115|       |    /// Returns the absolute output index where this call starts, in the
  116|       |    /// driving engine's output domain.
  117|       |    ///
  118|       |    /// # Returns
  119|       |    ///
  120|       |    /// Returns the output start index.
  121|       |    #[inline]
  122|       |    #[must_use]
  123|      2|    pub const fn output_start(self) -> usize {
  124|      2|        self.output_start
  125|      2|    }
  126|       |
  127|       |    /// Returns the absolute output index where the next output would be
  128|       |    /// written, in the driving engine's output domain.
  129|       |    ///
  130|       |    /// # Returns
  131|       |    ///
  132|       |    /// Returns the current output index.
  133|       |    #[inline]
  134|       |    #[must_use]
  135|      2|    pub const fn output_index(self) -> usize {
  136|      2|        self.output_index
  137|      2|    }
  138|       |
  139|       |    /// Returns units visible to the codec from [`Self::input_index`].
  140|       |    ///
  141|       |    /// # Returns
  142|       |    ///
  143|       |    /// Returns the available input-unit count.
  144|       |    #[inline]
  145|       |    #[must_use]
  146|  13.5k|    pub const fn available(self) -> usize {
  147|  13.5k|        self.available
  148|  13.5k|    }
  149|       |
  150|       |    /// Returns input units consumed since this `transcode` call started.
  151|       |    ///
  152|       |    /// # Returns
  153|       |    ///
  154|       |    /// Returns `input_index - input_start`.
  155|       |    #[inline]
  156|       |    #[must_use]
  157|      1|    pub const fn input_used(self) -> usize {
  158|      1|        self.input_index - self.input_start
  159|      1|    }
  160|       |
  161|       |    /// Returns output written since this call started, in the driving
  162|       |    /// engine's output domain. For a converter, this counts target encoding
  163|       |    /// units rather than intermediate decoded values.
  164|       |    ///
  165|       |    /// # Returns
  166|       |    ///
  167|       |    /// Returns `output_index - output_start`.
  168|       |    #[inline]
  169|       |    #[must_use]
  170|      2|    pub const fn output_written(self) -> usize {
  171|      2|        self.output_index - self.output_start
  172|      2|    }
  173|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/engine/decode_invalid_action.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Invalid-decode actions returned by buffered decoder policy hooks.
    9|       |
   10|       |use core::num::NonZeroUsize;
   11|       |
   12|       |use crate::Codec;
   13|       |
   14|       |/// Invalid-decode action type used by a codec-backed decode hook.
   15|       |///
   16|       |/// This alias binds the action's replacement value to the codec's decoded
   17|       |/// value type, so hook implementations can name their result in terms of the
   18|       |/// codec they serve.
   19|       |///
   20|       |/// # Type Parameters
   21|       |///
   22|       |/// - `C`: Codec whose decoded value is carried by replacement actions.
   23|       |pub type DecodeInvalidActionOf<C> = DecodeInvalidAction<<C as Codec>::Value>;
   24|       |
   25|       |/// Action selected after a codec reports invalid encoded input.
   26|       |///
   27|       |/// While the stream is open, incomplete input is not an invalid-input policy
   28|       |/// action. Codecs report it with [`crate::DecodeFailure::Incomplete`], and the
   29|       |/// decode engine converts it into [`crate::TranscodeStatus::NeedInput`]. After
   30|       |/// upstream EOF, [`crate::Transcoder::transcode_eof`] asks the codec to resolve
   31|       |/// a trailing incomplete sequence through [`crate::Codec::decode_eof`]; if it
   32|       |/// remains incomplete, the engine calls
   33|       |/// [`crate::engine::DecodeIncompleteAction`] through
   34|       |/// [`crate::engine::TranscodeDecodeHooks::handle_incomplete_decode`].
   35|       |///
   36|       |/// # Type Parameters
   37|       |///
   38|       |/// - `Value`: Decoded output value type.
   39|       |///
   40|       |/// # Examples
   41|       |///
   42|       |/// ```
   43|       |/// use core::num::NonZeroUsize;
   44|       |///
   45|       |/// use qubit_codec::engine::DecodeInvalidAction;
   46|       |///
   47|       |/// let action = DecodeInvalidAction::<u8>::Skip {
   48|       |///     consumed: NonZeroUsize::new(1).unwrap(),
   49|       |/// };
   50|       |/// assert!(matches!(action, DecodeInvalidAction::Skip { .. }));
   51|       |/// ```
   52|       |#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
   53|       |#[non_exhaustive]
   54|       |#[must_use]
   55|       |pub enum DecodeInvalidAction<Value> {
   56|       |    /// Reject the current invalid input.
   57|       |    ///
   58|       |    /// The decode engine reports the original codec decode error as
   59|       |    /// [`crate::TranscodeDecodeError::Domain`] at the current input index.
   60|       |    Reject,
   61|       |
   62|       |    /// Consume invalid input without producing output.
   63|       |    ///
   64|       |    /// `consumed` must not exceed the hook context's available input count.
   65|       |    /// Over-consuming is a hook contract violation and panics in the engine.
   66|       |    Skip {
   67|       |        /// Source units to consume.
   68|       |        consumed: NonZeroUsize,
   69|       |    },
   70|       |
   71|       |    /// Produce one replacement value and consume source units.
   72|       |    ///
   73|       |    /// `consumed` must not exceed the hook context's available input count.
   74|       |    /// Over-consuming is a hook contract violation and panics in the engine.
   75|       |    Emit {
   76|       |        /// Value to write to the output buffer.
   77|       |        value: Value,
   78|       |        /// Source units to consume.
   79|       |        consumed: NonZeroUsize,
   80|       |    },
   81|       |}
   82|       |
   83|       |impl<Value> DecodeInvalidAction<Value> {
   84|       |    /// Validates a policy-reported consumed source-unit count against available
   85|       |    /// input.
   86|       |    ///
   87|       |    /// # Parameters
   88|       |    ///
   89|       |    /// - `consumed`: Source units requested by the concrete policy.
   90|       |    /// - `available`: Source units visible from the current decode cursor.
   91|       |    ///
   92|       |    /// # Returns
   93|       |    ///
   94|       |    /// Returns the validated non-zero consumed count.
   95|       |    ///
   96|       |    /// # Panics
   97|       |    ///
   98|       |    /// Panics when `available == 0` or when `consumed > available`.
   99|       |    #[inline]
  100|       |    #[must_use]
  101|  1.03k|    pub(super) fn bound_consumed(consumed: NonZeroUsize, available: usize) -> NonZeroUsize {
  102|  1.03k|        assert!(available > 0, "DecodeInvalidAction cannot consume empty input",);
  103|  1.03k|        assert!(
  104|  1.03k|            consumed.get() <= available,
  105|       |            "DecodeInvalidAction consumed units must not exceed available input",
  106|       |        );
  107|  1.03k|        consumed
  108|  1.03k|    }
  109|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/engine/encode_context.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Read-only policy context for one buffered encode attempt.
    9|       |
   10|       |/// Read-only context supplied to an encode policy hook.
   11|       |///
   12|       |/// The engine retains ownership of the mutable output slice. Hooks can inspect
   13|       |/// the current value and cursor/capacity metadata, then return an
   14|       |/// [`crate::engine::EncodeUnencodableAction`] without being able to mutate
   15|       |/// output behind the engine's progress accounting.
   16|       |///
   17|       |/// # Type Parameters
   18|       |///
   19|       |/// - `Value`: Logical input value type.
   20|       |///
   21|       |/// # Examples
   22|       |///
   23|       |/// ```
   24|       |/// use qubit_codec::engine::EncodeContext;
   25|       |///
   26|       |/// let value = 7_u8;
   27|       |/// let context = EncodeContext::new(&value, 0, 0, 1);
   28|       |/// assert_eq!(context.input_value(), &7);
   29|       |/// ```
   30|       |#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
   31|       |pub struct EncodeContext<'a, Value> {
   32|       |    /// Value borrowed for the lifetime of this policy context.
   33|       |    input_value: &'a Value,
   34|       |    /// Absolute input position of the borrowed value.
   35|       |    input_index: usize,
   36|       |    /// Absolute output cursor at the start of the encode attempt.
   37|       |    output_index: usize,
   38|       |    /// Number of writable output units remaining at the cursor.
   39|       |    available_output: usize,
   40|       |}
   41|       |
   42|       |impl<'a, Value> EncodeContext<'a, Value> {
   43|       |    /// Creates an encode policy context.
   44|       |    ///
   45|       |    /// # Parameters
   46|       |    ///
   47|       |    /// - `input_value`: Borrowed input value being encoded.
   48|       |    /// - `input_index`: Absolute input index of `input_value`.
   49|       |    /// - `output_index`: Absolute output index where writing begins.
   50|       |    /// - `available_output`: Output units writable from `output_index`.
   51|       |    ///
   52|       |    /// # Returns
   53|       |    ///
   54|       |    /// Returns a read-only encode context.
   55|       |    #[must_use]
   56|       |    #[inline]
   57|  5.95k|    pub const fn new(input_value: &'a Value, input_index: usize, output_index: usize, available_output: usize) -> Self {
   58|  5.95k|        Self {
   59|  5.95k|            input_value,
   60|  5.95k|            input_index,
   61|  5.95k|            output_index,
   62|  5.95k|            available_output,
   63|  5.95k|        }
   64|  5.95k|    }
   65|       |
   66|       |    /// Returns the input value being encoded.
   67|       |    ///
   68|       |    /// # Returns
   69|       |    ///
   70|       |    /// The input value borrowed through this context without allocation.
   71|       |    #[must_use = "the borrowed input value should be inspected"]
   72|       |    #[inline]
   73|  4.91k|    pub const fn input_value(&self) -> &Value {
   74|  4.91k|        self.input_value
   75|  4.91k|    }
   76|       |
   77|       |    /// Returns the absolute input index of the current value.
   78|       |    ///
   79|       |    /// # Returns
   80|       |    ///
   81|       |    /// The absolute position of the value in the input sequence.
   82|       |    #[must_use]
   83|       |    #[inline]
   84|      3|    pub const fn input_index(&self) -> usize {
   85|      3|        self.input_index
   86|      3|    }
   87|       |
   88|       |    /// Returns the absolute output index where writing begins.
   89|       |    ///
   90|       |    /// # Returns
   91|       |    ///
   92|       |    /// The starting output cursor for this encode attempt.
   93|       |    #[must_use]
   94|       |    #[inline]
   95|      1|    pub const fn output_index(&self) -> usize {
   96|      1|        self.output_index
   97|      1|    }
   98|       |
   99|       |    /// Returns output capacity visible to this encode attempt.
  100|       |    ///
  101|       |    /// # Returns
  102|       |    ///
  103|       |    /// The number of output units available from the starting cursor.
  104|       |    #[must_use]
  105|       |    #[inline]
  106|      2|    pub const fn available_output(&self) -> usize {
  107|      2|        self.available_output
  108|      2|    }
  109|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/engine/encode_unencodable_action.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Unencodable-value actions returned by buffered encoder policy hooks.
    9|       |
   10|       |use crate::Codec;
   11|       |
   12|       |/// Unencodable-value action for a codec-backed encode hook.
   13|       |///
   14|       |/// This alias uses the codec's [`Codec::Value`] as the replacement payload
   15|       |/// type, keeping hook signatures tied to the codec they serve.
   16|       |///
   17|       |/// # Type Parameters
   18|       |///
   19|       |/// - `C`: Codec whose associated `Value` is carried by replacement actions.
   20|       |pub type EncodeUnencodableActionOf<C> = EncodeUnencodableAction<<C as Codec>::Value>;
   21|       |
   22|       |/// Action selected after a codec reports an unencodable input value.
   23|       |///
   24|       |/// Normal encodable values are handled by the encode engine itself. Hook
   25|       |/// implementations return this action only for values outside the codec's
   26|       |/// encodable domain.
   27|       |///
   28|       |/// # Type Parameters
   29|       |///
   30|       |/// - `Value`: Logical input value type accepted by the codec.
   31|       |///
   32|       |/// # Examples
   33|       |///
   34|       |/// ```
   35|       |/// use qubit_codec::engine::EncodeUnencodableAction;
   36|       |///
   37|       |/// let action = EncodeUnencodableAction::replace(0_u8);
   38|       |/// assert!(matches!(action, EncodeUnencodableAction::Replace { value: 0 }));
   39|       |/// ```
   40|       |#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
   41|       |#[non_exhaustive]
   42|       |#[must_use]
   43|       |pub enum EncodeUnencodableAction<Value> {
   44|       |    /// Reject the current input value.
   45|       |    ///
   46|       |    /// The encode engine reports [`crate::TranscodeEncodeError::Unencodable`]
   47|       |    /// at the current input index.
   48|       |    Reject,
   49|       |
   50|       |    /// Consume the current input value without producing output.
   51|       |    Skip,
   52|       |
   53|       |    /// Encode a replacement value and consume the current input value.
   54|       |    ///
   55|       |    /// The replacement must be encodable by the same codec. Returning an
   56|       |    /// unencodable replacement is a hook contract violation and panics in the
   57|       |    /// encode engine.
   58|       |    Replace {
   59|       |        /// Replacement value to encode.
   60|       |        value: Value,
   61|       |    },
   62|       |}
   63|       |
   64|       |impl<Value> EncodeUnencodableAction<Value> {
   65|       |    /// Creates a reject action.
   66|       |    ///
   67|       |    /// # Returns
   68|       |    ///
   69|       |    /// Returns [`Self::Reject`].
   70|       |    #[inline]
   71|      1|    pub const fn reject() -> Self {
   72|      1|        Self::Reject
   73|      1|    }
   74|       |
   75|       |    /// Creates a replacement action.
   76|       |    ///
   77|       |    /// # Parameters
   78|       |    ///
   79|       |    /// - `value`: Replacement value to encode.
   80|       |    ///
   81|       |    /// # Returns
   82|       |    ///
   83|       |    /// Returns [`Self::Replace`] carrying `value`.
   84|       |    #[inline]
   85|      5|    pub const fn replace(value: Value) -> Self {
   86|      5|        Self::Replace { value }
   87|      5|    }
   88|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/engine/internal/decode_outcome.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Outcome of one buffered decode attempt.
    9|       |
   10|       |use core::num::NonZeroUsize;
   11|       |
   12|       |/// Outcome produced by a decode-side engine for one source attempt.
   13|       |///
   14|       |/// This type describes what happened to source input after the engine has
   15|       |/// decoded, applied invalid-input hooks, and optionally delivered a logical
   16|       |/// value to its caller-provided consumer. It does not describe target output
   17|       |/// capacity for conversion pipelines; downstream encode backpressure is
   18|       |/// reported separately by the encode side.
   19|       |#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
   20|       |pub(crate) enum DecodeOutcome {
   21|       |    /// A logical value was emitted to the decode consumer.
   22|       |    Emitted {
   23|       |        /// Source units consumed for the emitted value.
   24|       |        read: NonZeroUsize,
   25|       |        /// Logical values emitted by this decode attempt.
   26|       |        emitted: NonZeroUsize,
   27|       |    },
   28|       |
   29|       |    /// Source input was consumed without emitting a value.
   30|       |    Skipped {
   31|       |        /// Source units consumed by the skip.
   32|       |        read: NonZeroUsize,
   33|       |    },
   34|       |
   35|       |    /// More source input is required before decoding can continue.
   36|       |    NeedInput {
   37|       |        /// Total source units required from the current input position.
   38|       |        required: NonZeroUsize,
   39|       |    },
   40|       |}
   41|       |
   42|       |impl DecodeOutcome {
   43|       |    /// Creates an emitted-value outcome.
   44|       |    ///
   45|       |    /// # Parameters
   46|       |    ///
   47|       |    /// - `read`: Nonzero source units consumed.
   48|       |    /// - `emitted`: Nonzero logical values delivered to the consumer.
   49|       |    ///
   50|       |    /// # Returns
   51|       |    ///
   52|       |    /// Returns `Emitted` with both counters preserved.
   53|       |    #[inline]
   54|       |    #[must_use]
   55|  3.93k|    pub(crate) const fn emitted(read: NonZeroUsize, emitted: NonZeroUsize) -> Self {
   56|  3.93k|        Self::Emitted { read, emitted }
   57|  3.93k|    }
   58|       |
   59|       |    /// Creates a skipped-input outcome.
   60|       |    ///
   61|       |    /// # Parameters
   62|       |    ///
   63|       |    /// - `read`: Nonzero source units discarded without emitting values.
   64|       |    ///
   65|       |    /// # Returns
   66|       |    ///
   67|       |    /// Returns `Skipped` with the consumed-unit count.
   68|       |    #[inline]
   69|       |    #[must_use]
   70|    538|    pub(crate) const fn skipped(read: NonZeroUsize) -> Self {
   71|    538|        Self::Skipped { read }
   72|    538|    }
   73|       |
   74|       |    /// Creates a missing-input outcome.
   75|       |    ///
   76|       |    /// # Parameters
   77|       |    ///
   78|       |    /// - `required`: Total source units needed at the current input position.
   79|       |    ///
   80|       |    /// # Returns
   81|       |    ///
   82|       |    /// Returns `NeedInput` with the required total, not an additional-unit
   83|       |    /// count.
   84|       |    #[inline]
   85|       |    #[must_use]
   86|      6|    pub(crate) const fn need_input(required: NonZeroUsize) -> Self {
   87|      6|        Self::NeedInput { required }
   88|      6|    }
   89|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/engine/internal/encode_outcome.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Outcome of one buffered encode value attempt.
    9|       |
   10|       |use core::num::NonZeroUsize;
   11|       |
   12|       |/// Outcome produced by an encode-side engine for one input value.
   13|       |///
   14|       |/// This is deliberately smaller than [`crate::TranscodeProgress`]. It only
   15|       |/// describes what happened to the current input value; the encode engine owns
   16|       |/// input/output cursor updates and progress construction.
   17|       |#[must_use]
   18|       |#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
   19|       |pub(crate) enum EncodeOutcome {
   20|       |    /// The current input value was consumed.
   21|       |    Consumed {
   22|       |        /// Output units written for this value.
   23|       |        written: usize,
   24|       |    },
   25|       |
   26|       |    /// The current input value was not consumed because output is too small.
   27|       |    NeedOutput {
   28|       |        /// Total output units required from the current output cursor.
   29|       |        required: NonZeroUsize,
   30|       |    },
   31|       |}
   32|       |
   33|       |impl EncodeOutcome {
   34|       |    /// Creates an outcome for a consumed input value.
   35|       |    ///
   36|       |    /// # Parameters
   37|       |    ///
   38|       |    /// - `written`: Number of output units emitted; zero is valid for
   39|       |    ///   buffering.
   40|       |    ///
   41|       |    /// # Returns
   42|       |    ///
   43|       |    /// A consumed outcome retaining the emitted unit count.
   44|       |    #[inline]
   45|  3.91k|    pub(crate) const fn consumed(written: usize) -> Self {
   46|  3.91k|        Self::Consumed { written }
   47|  3.91k|    }
   48|       |
   49|       |    /// Creates an outcome for insufficient output capacity.
   50|       |    ///
   51|       |    /// # Parameters
   52|       |    ///
   53|       |    /// - `required`: Total capacity needed from the current output cursor.
   54|       |    ///
   55|       |    /// # Returns
   56|       |    ///
   57|       |    /// A retryable outcome that leaves the current input value unconsumed.
   58|       |    #[inline]
   59|    978|    pub(crate) const fn need_output(required: NonZeroUsize) -> Self {
   60|    978|        Self::NeedOutput { required }
   61|    978|    }
   62|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/engine/transcode_convert_engine.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Reusable buffered converter engine for codec-backed transcoding.
    9|       |//!
   10|       |//! Bridges a source [`crate::engine::TranscodeDecodeEngine`] and a target
   11|       |//! [`crate::engine::TranscodeEncodeEngine`] into one unit-to-unit conversion
   12|       |//! pipeline.
   13|       |
   14|       |use core::num::NonZeroUsize;
   15|       |
   16|       |use qubit_utils::try_reserve_vec;
   17|       |
   18|       |use super::super::internal::convert_state::ConvertState;
   19|       |use super::super::internal::encode_attempt::EncodeAttempt;
   20|       |use super::super::internal::lifecycle_guard::LifecycleGuard;
   21|       |use super::super::internal::pending_value::PendingValue;
   22|       |use super::super::internal::pending_value_slot::PendingValueSlot;
   23|       |use super::TranscodeDecodeHooks;
   24|       |use super::TranscodeEncodeHooks;
   25|       |use super::transcode_decode_engine::TranscodeDecodeEngine;
   26|       |use super::transcode_encode_engine::TranscodeEncodeEngine;
   27|       |use crate::CapacityError;
   28|       |use crate::Codec;
   29|       |use crate::TranscodeConvertError;
   30|       |use crate::TranscodeConvertErrorOf;
   31|       |use crate::TranscodeConverter;
   32|       |use crate::TranscodeFailure;
   33|       |use crate::TranscodeProgress;
   34|       |use crate::Transcoder;
   35|       |use crate::codec::assert_unit_bounds;
   36|       |
   37|       |/// Adds two independent target-output capacity bounds.
   38|       |///
   39|       |/// # Parameters
   40|       |///
   41|       |/// - `first`: First independent output bound.
   42|       |/// - `second`: Second independent output bound.
   43|       |///
   44|       |/// # Returns
   45|       |///
   46|       |/// The checked sum of the supplied target-unit bounds.
   47|       |///
   48|       |/// # Errors
   49|       |///
   50|       |/// Returns [`CapacityError::OutputLengthOverflow`] when the sum exceeds
   51|       |/// `usize`.
   52|       |#[inline]
   53|  10.7k|fn add_convert_output_bounds(first: usize, second: usize) -> Result<usize, CapacityError> {
   54|  10.7k|    first.checked_add(second).ok_or(CapacityError::OutputLengthOverflow)
   55|  10.7k|}
   56|       |
   57|       |/// Adds three independent target-output capacity bounds.
   58|       |///
   59|       |/// # Parameters
   60|       |///
   61|       |/// - `first`: First independent target-output bound.
   62|       |/// - `second`: Second independent target-output bound.
   63|       |/// - `third`: Third independent target-output bound.
   64|       |///
   65|       |/// # Returns
   66|       |///
   67|       |/// The checked sum of the supplied target-unit bounds.
   68|       |///
   69|       |/// # Errors
   70|       |///
   71|       |/// Returns [`CapacityError::OutputLengthOverflow`] when the sum exceeds
   72|       |/// `usize`.
   73|       |#[inline]
   74|  3.39k|fn sum_convert_output_bounds(first: usize, second: usize, third: usize) -> Result<usize, CapacityError> {
   75|  3.39k|    let partial = add_convert_output_bounds(first, second)?;
                      ^3.39k                                            ^2
   76|  3.39k|    add_convert_output_bounds(partial, third)
   77|  3.39k|}
   78|       |
   79|       |/// Asserts that a pre-reserved conversion phase did not need more output.
   80|       |///
   81|       |/// # Parameters
   82|       |///
   83|       |/// - `progress`: `None` after draining, or `Some` when output is still needed.
   84|       |/// - `message`: Invariant explanation included in a panic.
   85|       |///
   86|       |/// # Panics
   87|       |///
   88|       |/// Panics with `message` if `progress` is `Some`.
   89|       |#[inline]
   90|  1.17k|fn assert_reserved_output_drained(progress: Option<TranscodeProgress>, message: &'static str) {
   91|  1.17k|    assert!(progress.is_none(), "{message}");
   92|  1.16k|}
   93|       |
   94|       |/// Reusable buffered conversion engine for codec-backed converters.
   95|       |///
   96|       |/// The engine owns reusable buffered decode and encode engines. It keeps
   97|       |/// common converter control flow private: index validation, pending-value
   98|       |/// retention, finish draining, decode-error policy dispatch, encode attempts,
   99|       |/// output-capacity checks, and [`crate::TranscodeStatus`] reporting.
  100|       |///
  101|       |/// Use this type to build a streaming converter over two one-value [`Codec`]
  102|       |/// implementations that share the same logical value type. Each hot-path step
  103|       |/// decodes one source unit sequence into a value, then immediately tries to
  104|       |/// encode that value into the target output buffer. If the target buffer lacks
  105|       |/// capacity, the decoded value is retained in an internal pending slot and
  106|       |/// must be drained before more source input is consumed, preserving output
  107|       |/// order across buffer turns.
  108|       |///
  109|       |/// `TranscodeConvertEngine` is intentionally batch-oriented. Its public
  110|       |/// [`Self::transcode`] method drives a source/output buffer loop and reuses the
  111|       |/// same unchecked codec and hook primitives as
  112|       |/// [`crate::engine::TranscodeDecodeEngine`] and
  113|       |/// [`crate::engine::TranscodeEncodeEngine`]. It does not call one-value public
  114|       |/// transcoders in the hot path.
  115|       |///
  116|       |/// For strict codec-backed conversion with default decode and encode policies,
  117|       |/// use [`crate::CodecTranscodeConverter`]. Use `TranscodeConvertEngine`
  118|       |/// directly when either side needs custom malformed-input repair, encode
  119|       |/// planning, skipped values, or finish-time output.
  120|       |///
  121|       |/// The engine follows the same lifecycle as [`crate::Transcoder`]:
  122|       |/// `reset → transcode* → finish → reset`. Call [`Self::reset`] before starting
  123|       |/// a new logical stream and [`Self::finish`] after EOF once any incomplete
  124|       |/// source tail has been handled.
  125|       |///
  126|       |/// # Examples
  127|       |///
  128|       |/// ```rust
  129|       |/// use core::convert::Infallible;
  130|       |/// use core::num::NonZeroUsize;
  131|       |///
  132|       |/// use qubit_codec::Codec;
  133|       |/// use qubit_codec::DecodeFailure;
  134|       |/// use qubit_codec::TranscodeDecodeErrorOf;
  135|       |/// use qubit_codec::TranscodeEncodeErrorOf;
  136|       |/// use qubit_codec::TranscodeStatus;
  137|       |/// use qubit_codec::engine::DecodeContext;
  138|       |/// use qubit_codec::engine::EncodeContext;
  139|       |/// use qubit_codec::engine::EncodeUnencodableAction;
  140|       |/// use qubit_codec::engine::TranscodeConvertEngine;
  141|       |/// use qubit_codec::engine::TranscodeDecodeHooks;
  142|       |/// use qubit_codec::engine::TranscodeEncodeHooks;
  143|       |///
  144|       |/// #[derive(Clone, Copy)]
  145|       |/// struct SourceCodec;
  146|       |///
  147|       |/// #[derive(Clone, Copy)]
  148|       |/// struct TargetCodec;
  149|       |///
  150|       |/// impl Codec for SourceCodec {
  151|       |///     type Value = u8;
  152|       |///     type Unit = u8;
  153|       |///     type DecodeError = Infallible;
  154|       |///     type EncodeError = Infallible;
  155|       |///
  156|       |///     const MIN_UNITS_PER_VALUE: usize = 1;
  157|       |///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
  158|       |///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
  159|       |///
  160|       |///     unsafe fn decode(
  161|       |///         &mut self,
  162|       |///         input: &[u8],
  163|       |///         index: usize,
  164|       |///     ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
  165|       |///         Ok((input[index].wrapping_add(1), NonZeroUsize::MIN))
  166|       |///     }
  167|       |///
  168|       |///     unsafe fn encode(
  169|       |///         &mut self,
  170|       |///         value: &u8,
  171|       |///         output: &mut [u8],
  172|       |///         index: usize,
  173|       |///     ) -> Result<usize, Self::EncodeError> {
  174|       |///         output[index] = *value;
  175|       |///         Ok(1)
  176|       |///     }
  177|       |/// }
  178|       |///
  179|       |/// impl Codec for TargetCodec {
  180|       |///     type Value = u8;
  181|       |///     type Unit = u8;
  182|       |///     type DecodeError = Infallible;
  183|       |///     type EncodeError = Infallible;
  184|       |///
  185|       |///     const MIN_UNITS_PER_VALUE: usize = 1;
  186|       |///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
  187|       |///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
  188|       |///
  189|       |///     unsafe fn decode(
  190|       |///         &mut self,
  191|       |///         input: &[u8],
  192|       |///         index: usize,
  193|       |///     ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
  194|       |///         Ok((input[index], NonZeroUsize::MIN))
  195|       |///     }
  196|       |///
  197|       |///     unsafe fn encode(
  198|       |///         &mut self,
  199|       |///         value: &u8,
  200|       |///         output: &mut [u8],
  201|       |///         index: usize,
  202|       |///     ) -> Result<usize, Self::EncodeError> {
  203|       |///         output[index] = *value;
  204|       |///         Ok(1)
  205|       |///     }
  206|       |/// }
  207|       |///
  208|       |/// struct StrictDecodeHooks;
  209|       |///
  210|       |/// impl TranscodeDecodeHooks<SourceCodec> for StrictDecodeHooks {
  211|       |///     fn handle_invalid_decode(
  212|       |///         &mut self,
  213|       |///         _codec: &mut SourceCodec,
  214|       |///         error: &Infallible,
  215|       |///         _consumed: Option<NonZeroUsize>,
  216|       |///         _context: DecodeContext,
  217|       |///     ) -> Result<qubit_codec::engine::DecodeInvalidAction<u8>, TranscodeDecodeErrorOf<SourceCodec>> {
  218|       |///         match *error {}
  219|       |///     }
  220|       |/// }
  221|       |///
  222|       |/// struct StrictEncodeHooks;
  223|       |///
  224|       |/// impl TranscodeEncodeHooks<TargetCodec> for StrictEncodeHooks {
  225|       |///     fn handle_unencodable_encode(
  226|       |///         &mut self,
  227|       |///         _codec: &mut TargetCodec,
  228|       |///         _context: &EncodeContext<'_, u8>,
  229|       |///     ) -> Result<EncodeUnencodableAction<u8>, TranscodeEncodeErrorOf<TargetCodec>> {
  230|       |///         unreachable!("TargetCodec accepts every u8")
  231|       |///     }
  232|       |/// }
  233|       |///
  234|       |/// let mut engine = TranscodeConvertEngine::new(
  235|       |///     SourceCodec,
  236|       |///     TargetCodec,
  237|       |///     StrictDecodeHooks,
  238|       |///     StrictEncodeHooks,
  239|       |/// );
  240|       |/// let input = [1_u8, 2, 3];
  241|       |/// let mut output = [0_u8; 2];
  242|       |/// let mut reset_output = [];
  243|       |/// engine.reset(&mut reset_output, 0)?;
  244|       |///
  245|       |/// let progress = engine.transcode(&input, 0, &mut output, 0)?;
  246|       |/// match progress.status() {
  247|       |///     TranscodeStatus::NeedOutput { .. } => {
  248|       |///         assert_eq!(2, progress.written());
  249|       |///         assert_eq!([2, 3], output);
  250|       |///         // Drain `output[..progress.written()]`, then resume at
  251|       |///         // `progress.read()` with fresh output capacity.
  252|       |///     }
  253|       |///     TranscodeStatus::Complete => unreachable!("output is intentionally short"),
  254|       |///     TranscodeStatus::NeedInput { .. } => unreachable!("input is complete"),
  255|       |/// }
  256|       |/// # Ok::<(), qubit_codec::TranscodeConvertErrorOf<SourceCodec, TargetCodec>>(())
  257|       |/// ```
  258|       |///
  259|       |/// # Type Parameters
  260|       |///
  261|       |/// - `D`: Source-side decoder codec.
  262|       |/// - `E`: Target-side encoder codec.
  263|       |/// - `DH`: Source-side decode hooks.
  264|       |/// - `EH`: Target-side encode hooks.
  265|       |#[derive(Debug)]
  266|       |pub struct TranscodeConvertEngine<D, E, DH, EH>
  267|       |where
  268|       |    D: Codec,
  269|       |    E: Codec<Value = D::Value>,
  270|       |    DH: TranscodeDecodeHooks<D>,
  271|       |    EH: TranscodeEncodeHooks<E>,
  272|       |{
  273|       |    /// Source-side buffered decoder engine.
  274|       |    decode_engine: TranscodeDecodeEngine<D, DH>,
  275|       |    /// Target-side buffered encoder engine.
  276|       |    encode_engine: TranscodeEncodeEngine<E, EH>,
  277|       |    /// Decoded value waiting for target output capacity.
  278|       |    pending: PendingValueSlot<D::Value>,
  279|       |    /// Guard for the `reset → transcode* → finish` lifecycle in every build
  280|       |    /// profile. The converter owns its own guard rather than delegating to the
  281|       |    /// inner decode/encode engines, because lifecycle events here describe the
  282|       |    /// converter as a whole.
  283|       |    lifecycle: LifecycleGuard,
  284|       |}
  285|       |
  286|       |impl<D, E, DH, EH> TranscodeConvertEngine<D, E, DH, EH>
  287|       |where
  288|       |    D: Codec,
  289|       |    E: Codec<Value = D::Value>,
  290|       |    DH: TranscodeDecodeHooks<D>,
  291|       |    EH: TranscodeEncodeHooks<E>,
  292|       |{
  293|       |    /// Creates a buffered converter engine.
  294|       |    ///
  295|       |    /// The caller supplies decode hooks and encode hooks directly.
  296|       |    ///
  297|       |    /// # Parameters
  298|       |    ///
  299|       |    /// - `decoder`: Low-level codec used for source decoding.
  300|       |    /// - `encoder`: Low-level codec used for target encoding.
  301|       |    /// - `decode_hooks`: Decode-side policy hooks.
  302|       |    /// - `encode_hooks`: Encode-side policy hooks.
  303|       |    ///
  304|       |    /// # Returns
  305|       |    ///
  306|       |    /// Returns a buffered converter engine.
  307|       |    ///
  308|       |    /// # Compile-Time Checks
  309|       |    ///
  310|       |    /// Fails to compile when either codec declares a zero decode unit bound or
  311|       |    /// when [`Codec::MIN_UNITS_PER_VALUE`] exceeds
  312|       |    /// [`Codec::MAX_DECODE_UNITS_PER_VALUE`].
  313|       |    #[inline]
  314|       |    #[must_use]
  315|  1.22k|    pub fn new(decoder: D, encoder: E, decode_hooks: DH, encode_hooks: EH) -> Self {
  316|  1.22k|        assert_unit_bounds::<D>();
  317|  1.22k|        assert_unit_bounds::<E>();
  318|  1.22k|        Self {
  319|  1.22k|            decode_engine: TranscodeDecodeEngine::new(decoder, decode_hooks),
  320|  1.22k|            encode_engine: TranscodeEncodeEngine::new(encoder, encode_hooks),
  321|  1.22k|            pending: PendingValueSlot::empty(),
  322|  1.22k|            lifecycle: LifecycleGuard::new(),
  323|  1.22k|        }
  324|  1.22k|    }
  325|       |
  326|       |    /// Returns the source-side codec.
  327|       |    ///
  328|       |    /// # Returns
  329|       |    ///
  330|       |    /// Returns a shared reference to the decoder codec owned by this engine.
  331|       |    #[inline]
  332|       |    #[must_use]
  333|      2|    pub fn source_codec(&self) -> &D {
  334|      2|        self.decode_engine.codec()
  335|      2|    }
  336|       |
  337|       |    /// Returns the source-side codec mutably.
  338|       |    ///
  339|       |    /// # Returns
  340|       |    ///
  341|       |    /// Returns a mutable reference to the decoder codec owned by this engine.
  342|       |    #[inline]
  343|       |    #[must_use]
  344|      2|    pub fn source_codec_mut(&mut self) -> &mut D {
  345|      2|        self.decode_engine.codec_mut()
  346|      2|    }
  347|       |
  348|       |    /// Returns the target-side codec.
  349|       |    ///
  350|       |    /// # Returns
  351|       |    ///
  352|       |    /// Returns a shared reference to the encoder codec owned by this engine.
  353|       |    #[inline]
  354|       |    #[must_use]
  355|      2|    pub fn target_codec(&self) -> &E {
  356|      2|        self.encode_engine.codec()
  357|      2|    }
  358|       |
  359|       |    /// Returns the target-side codec mutably.
  360|       |    ///
  361|       |    /// # Returns
  362|       |    ///
  363|       |    /// Returns a mutable reference to the encoder codec owned by this engine.
  364|       |    #[inline]
  365|       |    #[must_use]
  366|      2|    pub fn target_codec_mut(&mut self) -> &mut E {
  367|      2|        self.encode_engine.codec_mut()
  368|      2|    }
  369|       |
  370|       |    /// Returns the decode hooks used by this engine.
  371|       |    ///
  372|       |    /// # Returns
  373|       |    ///
  374|       |    /// Returns a shared reference to the decode hooks owned by this engine.
  375|       |    #[inline]
  376|       |    #[must_use]
  377|      1|    pub const fn decode_hooks(&self) -> &DH {
  378|      1|        self.decode_engine.hooks()
  379|      1|    }
  380|       |
  381|       |    /// Returns the decode hooks mutably.
  382|       |    ///
  383|       |    /// Mutating the returned hooks does not reset the converter or clear a
  384|       |    /// pending value. The replacement hooks must continue to satisfy their
  385|       |    /// global output-capacity contract for every reachable converter state.
  386|       |    ///
  387|       |    /// # Returns
  388|       |    ///
  389|       |    /// Returns a mutable reference to the decode hooks owned by this engine.
  390|       |    #[inline]
  391|       |    #[must_use]
  392|      1|    pub fn decode_hooks_mut(&mut self) -> &mut DH {
  393|      1|        self.decode_engine.hooks_mut()
  394|      1|    }
  395|       |
  396|       |    /// Returns the encode hooks used by this engine.
  397|       |    ///
  398|       |    /// # Returns
  399|       |    ///
  400|       |    /// Returns a shared reference to the encode hooks owned by this engine.
  401|       |    #[inline]
  402|       |    #[must_use]
  403|      3|    pub const fn encode_hooks(&self) -> &EH {
  404|      3|        self.encode_engine.hooks()
  405|      3|    }
  406|       |
  407|       |    /// Returns the encode hooks mutably.
  408|       |    ///
  409|       |    /// Mutating the returned hooks does not reset the converter or clear a
  410|       |    /// pending value. The replacement hooks must continue to satisfy their
  411|       |    /// global output-capacity contract for every reachable converter state.
  412|       |    ///
  413|       |    /// # Returns
  414|       |    ///
  415|       |    /// Returns a mutable reference to the encode hooks owned by this engine.
  416|       |    #[inline]
  417|       |    #[must_use]
  418|      1|    pub fn encode_hooks_mut(&mut self) -> &mut EH {
  419|      1|        self.encode_engine.hooks_mut()
  420|      1|    }
  421|       |
  422|       |    /// Consumes the engine and returns its codecs and hooks.
  423|       |    ///
  424|       |    /// Any pending value and lifecycle state owned by the converter are
  425|       |    /// discarded. Callers should use this only when no further conversion state
  426|       |    /// needs to be preserved.
  427|       |    ///
  428|       |    /// # Returns
  429|       |    ///
  430|       |    /// Returns the source codec, target codec, decode hooks, and encode hooks.
  431|       |    #[inline]
  432|       |    #[must_use]
  433|      2|    pub fn into_parts(self) -> (D, E, DH, EH) {
  434|       |        let Self {
  435|      2|            decode_engine,
  436|      2|            encode_engine,
  437|       |            ..
  438|      2|        } = self;
  439|      2|        let (source, decode_hooks) = decode_engine.into_parts();
  440|      2|        let (target, encode_hooks) = encode_engine.into_parts();
  441|      2|        (source, target, decode_hooks, encode_hooks)
  442|      2|    }
  443|       |
  444|       |    /// Returns an upper bound for target units produced from `input_len` units.
  445|       |    ///
  446|       |    /// The bound sums three parts: one possible retained pending value, the
  447|       |    /// maximum decoded values from the source side, and the maximum target
  448|       |    /// units for those values on the encode side. It covers only the streaming
  449|       |    /// convert phase and remains valid even when no value is currently pending.
  450|       |    /// Downstream converters must use this engine-level API for capacity
  451|       |    /// planning instead of recomputing the bound from the source or target
  452|       |    /// [`Codec`] constants.
  453|       |    ///
  454|       |    /// # Parameters
  455|       |    ///
  456|       |    /// - `input_len`: Number of source units the caller plans to convert.
  457|       |    ///
  458|       |    /// # Returns
  459|       |    ///
  460|       |    /// Returns a conservative upper bound, or a capacity error on arithmetic
  461|       |    /// overflow.
  462|       |    ///
  463|       |    /// # Errors
  464|       |    ///
  465|       |    /// Returns [`CapacityError`] when a component bound or their sum overflows
  466|       |    /// `usize`.
  467|  1.08k|    pub fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
  468|  1.08k|        let pending_units = self.encode_engine.max_transcode_output_len(1)?;
                          ^1.08k                                                        ^1
  469|  1.08k|        let decoded_values = self.decode_engine.max_transcode_output_len(input_len)?;
                          ^1.08k                                                                 ^2
  470|  1.08k|        let converted_units = self.encode_engine.max_transcode_output_len(decoded_values)?;
                          ^1.08k                                                                       ^2
  471|  1.08k|        add_convert_output_bounds(converted_units, pending_units)
  472|  1.08k|    }
  473|       |
  474|       |    /// Returns the global maximum target units emitted when resetting stream
  475|       |    /// state.
  476|       |    ///
  477|       |    /// Covers decode-side reset values (encoded to target units) plus
  478|       |    /// encode-side reset units. Most codecs are stateless and return `0`
  479|       |    /// for [`Codec::MAX_DECODE_RESET_VALUES`]; in that case this equals the
  480|       |    /// encode reset bound only.
  481|       |    ///
  482|       |    /// # Returns
  483|       |    ///
  484|       |    /// Returns the combined decode-reset and encode-reset output bound, or a
  485|       |    /// capacity error on arithmetic overflow.
  486|       |    ///
  487|       |    /// # Errors
  488|       |    ///
  489|       |    /// Returns [`CapacityError`] when a component bound or their sum overflows
  490|       |    /// `usize`.
  491|  2.91k|    pub fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
  492|  2.91k|        let decode_reset_units = self
                          ^2.90k
  493|  2.91k|            .encode_engine
  494|  2.91k|            .max_transcode_output_len(D::MAX_DECODE_RESET_VALUES)?;
                                                                               ^3
  495|  2.90k|        let encode_reset_units = E::MAX_ENCODE_RESET_UNITS;
  496|  2.90k|        add_convert_output_bounds(decode_reset_units, encode_reset_units)
  497|  2.91k|    }
  498|       |
  499|       |    /// Returns the maximum target units emitted by finishing retained state.
  500|       |    ///
  501|       |    /// The bound covers one possible retained pending value, the global
  502|       |    /// decode-side finish-value bound (encoded to target units), and the
  503|       |    /// global encode-side finish-unit bound. It is independent of whether a
  504|       |    /// pending value currently exists.
  505|       |    ///
  506|       |    /// # Returns
  507|       |    ///
  508|       |    /// Returns the combined pending, decode-finish, and encode-finish output
  509|       |    /// bound, or a capacity error on arithmetic overflow.
  510|       |    ///
  511|       |    /// # Errors
  512|       |    ///
  513|       |    /// Returns [`CapacityError`] when a component bound or their sum overflows
  514|       |    /// `usize`.
  515|  1.69k|    pub fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
  516|  1.69k|        let pending_units = self.encode_engine.max_transcode_output_len(1)?;
                          ^1.69k                                                        ^1
  517|  1.69k|        let decoder_finish_values = self.decode_engine.max_finish_output_len()?;
                          ^1.69k                                                            ^1
  518|  1.69k|        let decoder_finish_units = self.encode_engine.max_transcode_output_len(decoder_finish_values)?;
                          ^1.69k                                                                                   ^1
  519|  1.69k|        let encoder_finish_units = self.encode_engine.max_finish_output_len()?;
                          ^1.69k                                                           ^1
  520|  1.69k|        sum_convert_output_bounds(pending_units, decoder_finish_units, encoder_finish_units)
  521|  1.69k|    }
  522|       |
  523|       |    /// Returns the maximum target units needed by a complete one-shot
  524|       |    /// conversion.
  525|       |    ///
  526|       |    /// The returned bound covers conversion reset output, the streaming convert
  527|       |    /// phase for `input_len` source units, and finish output. Its components
  528|       |    /// are global across transient state. Higher-level complete conversion
  529|       |    /// helpers should use this engine-level bound instead of recomputing
  530|       |    /// capacity from the source or target codec constants, because decode
  531|       |    /// and encode hooks may change streaming or finish output.
  532|       |    ///
  533|       |    /// # Parameters
  534|       |    ///
  535|       |    /// - `input_len`: Number of source units in the complete stream.
  536|       |    ///
  537|       |    /// # Returns
  538|       |    ///
  539|       |    /// Returns the complete-stream target-output bound, or a capacity error on
  540|       |    /// arithmetic overflow.
  541|       |    ///
  542|       |    /// # Errors
  543|       |    ///
  544|       |    /// Returns [`CapacityError`] when a component bound or their sum overflows
  545|       |    /// `usize`.
  546|    539|    pub fn max_total_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
  547|    539|        let reset = self.max_reset_output_len()?;
                          ^537                               ^2
  548|    537|        let transcode = self.max_transcode_output_len(input_len)?;
                          ^536                                                ^1
  549|    536|        let finish = self.max_finish_output_len()?;
                          ^534                                 ^2
  550|    534|        sum_convert_output_bounds(reset, transcode, finish)
  551|    539|    }
  552|       |
  553|       |    /// Clears retained conversion state, runs before-reset hooks, and emits
  554|       |    /// stream-start encode output.
  555|       |    ///
  556|       |    /// Reset clears any retained pending value, resets the target encoder, then
  557|       |    /// drains decode-side reset values through that reset encoder state. The
  558|       |    /// caller must provide enough output capacity for
  559|       |    /// [`Self::max_reset_output_len`].
  560|       |    ///
  561|       |    /// # Parameters
  562|       |    ///
  563|       |    /// - `output`: Complete output unit slice visible to the converter.
  564|       |    /// - `output_index`: Absolute output index where writing starts.
  565|       |    ///
  566|       |    /// # Returns
  567|       |    ///
  568|       |    /// Returns the number of target units written while resetting stream state.
  569|       |    ///
  570|       |    /// # Errors
  571|       |    ///
  572|       |    /// Returns a converter error if reset validation, target reset output
  573|       |    /// emission, or reservation of the internal lifecycle value buffer fails.
  574|       |    /// Capacity and index failures occur before converter state is changed.
  575|       |    /// Once lifecycle execution starts, failure leaves the converter poisoned
  576|       |    /// until a later reset succeeds.
  577|       |    ///
  578|       |    /// # Panics
  579|       |    ///
  580|       |    /// Panics when decode-reset values cannot be encoded within the capacity
  581|       |    /// reserved by [`Self::max_reset_output_len`].
  582|  1.23k|    pub fn reset(&mut self, output: &mut [E::Unit], output_index: usize) -> Result<usize, TranscodeConvertErrorOf<D, E>>
  583|  1.23k|    where
  584|  1.23k|        D::Value: Default,
  585|       |    {
  586|  1.23k|        let required = self.max_reset_output_len()?;
                          ^1.23k                                ^1
  587|  1.23k|        TranscodeFailure::ensure_output_capacity(output.len(), output_index, required)?;
                                                                                                    ^3
  588|  1.22k|        self.lifecycle.on_reset_start();
  589|       |
  590|  1.22k|        self.pending.clear();
  591|       |
  592|       |        // Reset the target first because source-side reset values must be
  593|       |        // encoded under the target's new-stream state. The reset bound reserves
  594|       |        // space for both target-owned output and the encoded source-reset
  595|       |        // values, so encode_pending should never report `NeedOutput` here.
  596|  1.22k|        let empty_input: &[D::Unit] = &[];
  597|  1.22k|        let mut state = ConvertState::new(empty_input, 0, output, output_index);
  598|  1.22k|        let output_cursor = state.output_cursor();
  599|  1.22k|        let encoder_written = self.encode_engine.reset(state.output_mut(), output_cursor)?;
                          ^1.22k                                                                       ^2
  600|  1.22k|        state.advance_output(encoder_written);
  601|  1.22k|        self.drain_decoder_reset(&mut state)?;
                                                          ^5
  602|  1.22k|        self.lifecycle.on_reset_success();
  603|  1.22k|        Ok(state.written())
  604|  1.23k|    }
  605|       |
  606|       |    /// Converts source units into target units.
  607|       |    ///
  608|       |    /// The engine drains any retained pending value before consuming new input.
  609|       |    /// Each loop iteration decodes one source value and immediately attempts to
  610|       |    /// encode it. Conversion stops when the input tail is incomplete, when the
  611|       |    /// output buffer cannot hold the next encoded value, or when the visible
  612|       |    /// input is exhausted.
  613|       |    ///
  614|       |    /// # Parameters
  615|       |    ///
  616|       |    /// - `input`: Complete input unit slice visible to the converter.
  617|       |    /// - `input_index`: Absolute input index where conversion starts.
  618|       |    /// - `output`: Complete output unit slice visible to the converter.
  619|       |    /// - `output_index`: Absolute output index where writing starts.
  620|       |    ///
  621|       |    /// # Returns
  622|       |    ///
  623|       |    /// Returns conversion progress describing input units consumed, target
  624|       |    /// units written, and why conversion stopped.
  625|       |    ///
  626|       |    /// # Errors
  627|       |    ///
  628|       |    /// Returns [`TranscodeConvertError::Failure`] with
  629|       |    /// [`TranscodeFailure::InvalidInputIndex`] for an invalid input index or
  630|       |    /// [`TranscodeFailure::InvalidOutputIndex`] for an invalid output index.
  631|       |    /// Concrete codec or hook conversion errors retain their decode-side or
  632|       |    /// encode-side classification. Returns
  633|       |    /// [`TranscodeFailure::TranscodeBeforeReset`] when the
  634|       |    /// engine has not completed its first reset. Returns
  635|       |    /// [`TranscodeFailure::TranscodeAfterFinish`] when the logical stream was
  636|       |    /// already finished and has not been reset, or
  637|       |    /// [`TranscodeFailure::LifecyclePoisoned`] when an earlier reset or finish
  638|       |    /// failed after execution started.
  639|       |    #[inline]
  640|  2.81k|    pub fn transcode(
  641|  2.81k|        &mut self,
  642|  2.81k|        input: &[D::Unit],
  643|  2.81k|        input_index: usize,
  644|  2.81k|        output: &mut [E::Unit],
  645|  2.81k|        output_index: usize,
  646|  2.81k|    ) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>> {
  647|  2.81k|        self.transcode_with_eof(input, input_index, output, output_index, false)
  648|  2.81k|    }
  649|       |
  650|       |    /// Converts source units after the caller has established end of input.
  651|       |    ///
  652|       |    /// # Parameters
  653|       |    ///
  654|       |    /// - `input`: Source units visible to this call.
  655|       |    /// - `input_index`: Starting index in `input`.
  656|       |    /// - `output`: Destination for target units.
  657|       |    /// - `output_index`: Starting index in `output`.
  658|       |    ///
  659|       |    /// # Returns
  660|       |    ///
  661|       |    /// Consumed source units, written target units, and the stopping reason.
  662|       |    ///
  663|       |    /// # Errors
  664|       |    ///
  665|       |    /// Returns an error for invalid indices, a missing reset, a finished or
  666|       |    /// poisoned stream, or codec/hook conversion failure.
  667|       |    #[inline]
  668|  1.14k|    pub fn transcode_eof(
  669|  1.14k|        &mut self,
  670|  1.14k|        input: &[D::Unit],
  671|  1.14k|        input_index: usize,
  672|  1.14k|        output: &mut [E::Unit],
  673|  1.14k|        output_index: usize,
  674|  1.14k|    ) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>> {
  675|  1.14k|        self.transcode_with_eof(input, input_index, output, output_index, true)
  676|  1.14k|    }
  677|       |
  678|       |    /// Finishes retained output after EOF.
  679|       |    ///
  680|       |    /// Finalization drains a pending decoded value first, then lets the
  681|       |    /// source-side decode hooks emit final values, encodes those values through
  682|       |    /// the target-side encode hooks, and finally finishes target-side encode
  683|       |    /// hook state. The decode-finish value buffer used for this cold path
  684|       |    /// requires `D::Value: Default`; the normal [`Self::transcode`] loop does
  685|       |    /// not.
  686|       |    ///
  687|       |    /// # Parameters
  688|       |    ///
  689|       |    /// - `output`: Complete output unit slice visible to the converter.
  690|       |    /// - `output_index`: Absolute output index where writing starts.
  691|       |    ///
  692|       |    /// # Returns
  693|       |    ///
  694|       |    /// Returns the number of target units written during finalization.
  695|       |    ///
  696|       |    /// # Errors
  697|       |    ///
  698|       |    /// Returns a converter error when output capacity checks fail, the
  699|       |    /// internal lifecycle value buffer cannot be reserved, or hook
  700|       |    /// finalization fails. Returns
  701|       |    /// [`TranscodeFailure::FinishBeforeReset`] when the engine has not
  702|       |    /// completed its first reset. Returns
  703|       |    /// [`TranscodeFailure::FinishAfterFinish`] when the logical stream was
  704|       |    /// already finished and has not been reset, or
  705|       |    /// [`TranscodeFailure::LifecyclePoisoned`] when an earlier reset or finish
  706|       |    /// failed after execution started. Capacity and index failures occur before
  707|       |    /// finish execution and remain retryable; later failures poison the
  708|       |    /// converter until reset succeeds. Once lifecycle execution starts,
  709|       |    /// failure leaves the converter poisoned until a later reset succeeds.
  710|       |    ///
  711|       |    /// # Panics
  712|       |    ///
  713|       |    /// Panics when a retained pending value or decode-finish value cannot be
  714|       |    /// encoded within the planned finish capacity.
  715|  1.17k|    pub fn finish(
  716|  1.17k|        &mut self,
  717|  1.17k|        output: &mut [E::Unit],
  718|  1.17k|        output_index: usize,
  719|  1.17k|    ) -> Result<usize, TranscodeConvertErrorOf<D, E>>
  720|  1.17k|    where
  721|  1.17k|        D::Value: Default,
  722|       |    {
  723|  1.17k|        self.lifecycle.on_finish_attempt()?;
                                                        ^3
  724|  1.17k|        let required = self.current_finish_output_len()?;
                          ^1.16k                                     ^3
  725|  1.16k|        TranscodeFailure::ensure_output_capacity(output.len(), output_index, required)?;
                                                                                                    ^6
  726|  1.16k|        self.lifecycle.on_finish_start();
  727|       |
  728|  1.16k|        let empty_input: &[D::Unit] = &[];
  729|  1.16k|        let mut state = ConvertState::new(empty_input, 0, output, output_index);
  730|       |        // Finish keeps the same priority as transcode: output any retained
  731|       |        // decoded value before asking source-side hooks for final values.
  732|  1.16k|        let progress = self.drain_pending(&mut state)?;
                          ^1.16k                                   ^1
  733|  1.16k|        assert_reserved_output_drained(progress, "converter finish bound must reserve space for pending values");
  734|       |
  735|       |        // Source-side finish may emit one or more final values. Drain them into
  736|       |        // the target encoder before finishing target-side hook state.
  737|  1.16k|        self.drain_decoder_finish(&mut state)?;
                                                           ^8
  738|       |
  739|  1.15k|        let output_cursor = state.output_cursor();
  740|  1.15k|        let written = self.encode_engine.finish(state.output_mut(), output_cursor)?;
                          ^1.15k                                                                ^1
  741|  1.15k|        state.advance_output(written);
  742|  1.15k|        self.lifecycle.on_finish_success();
  743|  1.15k|        Ok(state.written())
  744|  1.17k|    }
  745|       |
  746|       |    /// Runs a complete one-shot `reset -> transcode -> finish` conversion.
  747|       |    ///
  748|       |    /// The complete input is supplied as `input`, and output starts at index
  749|       |    /// `0` in `output`. Callers that need subranges should slice their
  750|       |    /// buffers before calling this method. Downstream one-shot converter
  751|       |    /// helpers should call this engine method instead of reproducing the
  752|       |    /// reset, transcode, and finish sequence themselves.
  753|       |    ///
  754|       |    /// # Parameters
  755|       |    ///
  756|       |    /// - `input`: Complete source unit slice.
  757|       |    /// - `output`: Target unit slice for the whole converted stream.
  758|       |    ///
  759|       |    /// # Returns
  760|       |    ///
  761|       |    /// Returns the number of target units written.
  762|       |    ///
  763|       |    /// # Errors
  764|       |    ///
  765|       |    /// Returns framework errors for insufficient output, capacity overflow, or
  766|       |    /// an incomplete EOF tail, and domain errors from reset, conversion, or
  767|       |    /// finish.
  768|       |    ///
  769|       |    /// # Panics
  770|       |    ///
  771|       |    /// Panics if codec/hook lifecycle output exceeds its declared capacity
  772|       |    /// bound.
  773|       |    #[inline]
  774|    531|    pub fn transcode_complete_into(
  775|    531|        &mut self,
  776|    531|        input: &[D::Unit],
  777|    531|        output: &mut [E::Unit],
  778|    531|    ) -> Result<usize, TranscodeConvertErrorOf<D, E>>
  779|    531|    where
  780|    531|        D::Value: Default,
  781|       |    {
  782|    531|        <Self as Transcoder>::transcode_complete_into(self, input, output)
  783|    531|    }
  784|       |
  785|       |    /// Returns the finish-output bound for the converter's current pending
  786|       |    /// state.
  787|       |    ///
  788|       |    /// Public capacity methods expose global bounds across every transient
  789|       |    /// state. A concrete finish call can use this narrower checked bound while
  790|       |    /// still relying on global component bounds for decoder and encoder
  791|       |    /// finalization.
  792|       |    ///
  793|       |    /// # Returns
  794|       |    ///
  795|       |    /// Returns the current finish-output bound, or a capacity error on
  796|       |    /// arithmetic overflow.
  797|       |    ///
  798|       |    /// # Errors
  799|       |    ///
  800|       |    /// Returns [`CapacityError`] when a component bound or their sum overflows
  801|       |    /// `usize`.
  802|  1.17k|    fn current_finish_output_len(&self) -> Result<usize, CapacityError> {
  803|  1.17k|        let pending_units = self.pending.current_output_len(&self.encode_engine)?;
                                                                                              ^0
  804|  1.17k|        let decoder_finish_values = self.decode_engine.max_finish_output_len()?;
                          ^1.17k                                                            ^1
  805|  1.17k|        let decoder_finish_units = self.encode_engine.max_transcode_output_len(decoder_finish_values)?;
                          ^1.16k                                                                                   ^1
  806|  1.16k|        let encoder_finish_units = self.encode_engine.max_finish_output_len()?;
                          ^1.16k                                                           ^1
  807|  1.16k|        sum_convert_output_bounds(pending_units, decoder_finish_units, encoder_finish_units)
  808|  1.17k|    }
  809|       |
  810|       |    /// Runs one conversion step with an explicit end-of-input flag.
  811|       |    ///
  812|       |    /// The helper owns the shared lifecycle and cursor checks used by both
  813|       |    /// streaming and EOF-aware public entry points.
  814|       |    ///
  815|       |    /// # Parameters
  816|       |    ///
  817|       |    /// - `input`: Source units visible to this call.
  818|       |    /// - `input_index`: Starting index in `input`.
  819|       |    /// - `output`: Destination for target units.
  820|       |    /// - `output_index`: Starting index in `output`.
  821|       |    /// - `end_of_input`: Whether the source is closed and EOF decode rules
  822|       |    ///   apply.
  823|       |    ///
  824|       |    /// # Returns
  825|       |    ///
  826|       |    /// Consumed source units, written target units, and the stopping reason.
  827|       |    ///
  828|       |    /// # Errors
  829|       |    ///
  830|       |    /// Returns an error for invalid indices, a missing reset, a finished or
  831|       |    /// poisoned stream, or codec/hook conversion failure.
  832|  3.95k|    fn transcode_with_eof(
  833|  3.95k|        &mut self,
  834|  3.95k|        input: &[D::Unit],
  835|  3.95k|        input_index: usize,
  836|  3.95k|        output: &mut [E::Unit],
  837|  3.95k|        output_index: usize,
  838|  3.95k|        end_of_input: bool,
  839|  3.95k|    ) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>> {
  840|  3.95k|        self.lifecycle.on_transcode()?;
                                                   ^5
  841|  3.95k|        TranscodeFailure::ensure_transcode_indices(input.len(), input_index, output.len(), output_index)?;
                                                                                                                      ^4
  842|       |
  843|  3.94k|        let mut state = ConvertState::new(input, input_index, output, output_index);
  844|       |
  845|       |        // A retained decoded value must be written before consuming more input,
  846|       |        // otherwise callers could observe output reordered across buffer turns.
  847|  3.94k|        if let Some(progress) = self.drain_pending(&mut state)? {
                                  ^1                                        ^1
  848|      1|            return Ok(progress);
  849|  3.94k|        }
  850|       |
  851|  3.94k|        let min_input_units =
  852|  3.94k|            NonZeroUsize::new(D::MIN_UNITS_PER_VALUE).expect("Codec::MIN_UNITS_PER_VALUE is non-zero");
  853|  3.94k|        let min_input_len = min_input_units.get();
  854|  7.40k|        while state.has_input() {
  855|  5.58k|            let available = state.available_input();
  856|  5.58k|            if available < min_input_len && !end_of_input {
                                                          ^1.18k
  857|  1.14k|                return Ok(state.need_input_progress(min_input_units));
  858|  4.43k|            }
  859|       |
  860|  4.43k|            let previous_read = state.read();
  861|       |            // Each hot-path step decodes one source value and immediately tries
  862|       |            // to encode it, preserving backpressure at the target output.
  863|  4.43k|            if let Some(progress) = self.convert_next(&mut state, end_of_input)? {
                                      ^961                                                   ^10
  864|    961|                return Ok(progress);
  865|  3.46k|            }
  866|  3.46k|            debug_assert!(
  867|  3.46k|                state.read() > previous_read,
  868|       |                "TranscodeConvertEngine conversion step must consume input or stop",
  869|       |            );
  870|       |        }
  871|       |
  872|  1.82k|        Ok(state.complete_progress())
  873|  3.95k|    }
  874|       |
  875|       |    /// Drains source-side decode reset output and encodes emitted reset
  876|       |    /// values.
  877|       |    ///
  878|       |    /// Stateless decoders still call [`TranscodeDecodeEngine::reset`] so hook
  879|       |    /// teardown side effects run even when no reset values are emitted.
  880|       |    ///
  881|       |    /// # Parameters
  882|       |    ///
  883|       |    /// - `state`: Current conversion cursors and output buffer.
  884|       |    ///
  885|       |    /// # Returns
  886|       |    ///
  887|       |    /// Returns `Ok(())` after all decode-reset values have been encoded.
  888|       |    ///
  889|       |    /// # Errors
  890|       |    ///
  891|       |    /// Returns a converter error when decode reset or encode reset handling
  892|       |    /// fails.
  893|       |    ///
  894|       |    /// # Panics
  895|       |    ///
  896|       |    /// Panics when a decode-reset value cannot be encoded within the capacity
  897|       |    /// reserved by [`Self::max_reset_output_len`].
  898|  1.22k|    fn drain_decoder_reset(
  899|  1.22k|        &mut self,
  900|  1.22k|        state: &mut ConvertState<'_, D::Unit, E::Unit>,
  901|  1.22k|    ) -> Result<(), TranscodeConvertErrorOf<D, E>>
  902|  1.22k|    where
  903|  1.22k|        D::Value: Default,
  904|       |    {
  905|  1.22k|        let value_count = D::MAX_DECODE_RESET_VALUES;
  906|  1.22k|        if value_count == 0 {
  907|       |            // Stateless decoder: still call decode_reset so codecs whose
  908|       |            // hooks own teardown side effects (e.g. clearing accumulators)
  909|       |            // run them. The empty slice is safe because the capacity check
  910|       |            // inside `reset` accepts `required == 0` against any slice.
  911|  1.21k|            self.decode_engine.reset(&mut [], 0)?;
                                                              ^1
  912|  1.21k|            return Ok(());
  913|      7|        }
  914|       |        // `D::Value: Default` is only consulted when the decoder declares
  915|       |        // reset output. Stateless codecs never reach this branch.
  916|      7|        let mut reset_values = Vec::new();
  917|      7|        try_reserve_vec(&mut reset_values, value_count).map_err(|_| TranscodeFailure::allocation_failed())?;
                                                                                  ^1                                    ^1
  918|      6|        reset_values.resize_with(value_count, D::Value::default);
  919|      6|        let written = self.decode_engine.reset(&mut reset_values, 0)?;
                          ^4                                                      ^2
  920|      4|        for value in reset_values.into_iter().take(written) {
  921|      4|            let pending = PendingValue::new(value, 0);
  922|      4|            let progress = self.encode_pending(pending, state)?;
                              ^2                                            ^2
  923|      2|            assert_reserved_output_drained(
  924|      2|                progress,
  925|       |                "converter reset bound must reserve space for decode reset values",
  926|       |            );
  927|       |        }
  928|      2|        Ok(())
  929|  1.22k|    }
  930|       |
  931|       |    /// Converts one source value from the current state cursors.
  932|       |    ///
  933|       |    /// Decodes one value through the source engine, then immediately attempts
  934|       |    /// to encode it through the target engine.
  935|       |    ///
  936|       |    /// # Parameters
  937|       |    ///
  938|       |    /// - `state`: Current conversion cursors and output buffer.
  939|       |    /// - `end_of_input`: Whether to apply EOF-aware decode rules.
  940|       |    ///
  941|       |    /// # Returns
  942|       |    ///
  943|       |    /// Returns conversion progress when the step stops early, or `None` when
  944|       |    /// the value was consumed and either encoded or skipped by policy.
  945|       |    ///
  946|       |    /// # Errors
  947|       |    ///
  948|       |    /// Returns a converter error when decode or encode handling fails.
  949|       |    #[inline]
  950|  4.43k|    fn convert_next(
  951|  4.43k|        &mut self,
  952|  4.43k|        state: &mut ConvertState<'_, D::Unit, E::Unit>,
  953|  4.43k|        end_of_input: bool,
  954|  4.43k|    ) -> Result<Option<TranscodeProgress>, TranscodeConvertErrorOf<D, E>> {
  955|  4.42k|        let (outcome, pending) =
  956|  4.43k|            self.decode_engine
  957|  4.43k|                .decode_one(state.input(), state.decode_context(), end_of_input, PendingValue::new)?;
                                                                                                                 ^6
  958|  4.42k|        if let Some(pending) = pending {
                                  ^3.89k
  959|  3.89k|            self.pending.put(pending);
  960|  3.89k|        }
                      ^536
  961|  4.42k|        if let Some(progress) = state.apply_decode_outcome(outcome) {
                                  ^3
  962|      3|            return Ok(Some(progress));
  963|  4.42k|        }
  964|  4.42k|        let Some(pending) = self.pending.take() else {
                               ^3.89k
  965|    533|            return Ok(None);
  966|       |        };
  967|  3.89k|        self.encode_pending(pending, state)
  968|  4.43k|    }
  969|       |
  970|       |    /// Writes a retained decoded value before new input is consumed.
  971|       |    ///
  972|       |    /// # Parameters
  973|       |    ///
  974|       |    /// - `state`: Current conversion cursors and output buffer.
  975|       |    ///
  976|       |    /// # Returns
  977|       |    ///
  978|       |    /// Returns conversion progress when the pending value needs more output
  979|       |    /// capacity, or `None` when no value was pending or the value was encoded
  980|       |    /// or skipped by policy.
  981|       |    ///
  982|       |    /// # Errors
  983|       |    ///
  984|       |    /// Returns a converter error when encode handling fails.
  985|       |    #[inline]
  986|  5.10k|    fn drain_pending(
  987|  5.10k|        &mut self,
  988|  5.10k|        state: &mut ConvertState<'_, D::Unit, E::Unit>,
  989|  5.10k|    ) -> Result<Option<TranscodeProgress>, TranscodeConvertErrorOf<D, E>> {
  990|  5.10k|        let Some(pending) = self.pending.take() else {
                               ^954
  991|  4.15k|            return Ok(None);
  992|       |        };
  993|    954|        self.encode_pending(pending, state)
  994|  5.10k|    }
  995|       |
  996|       |    /// Drains source-side decode finish output and encodes emitted final
  997|       |    /// values.
  998|       |    ///
  999|       |    /// When the decoder declares no finish output, still calls
 1000|       |    /// [`TranscodeDecodeEngine::finish`] so codec finish and hook teardown can
 1001|       |    /// run and fail even when zero values are emitted.
 1002|       |    ///
 1003|       |    /// # Parameters
 1004|       |    ///
 1005|       |    /// - `state`: Current conversion cursors and output buffer.
 1006|       |    ///
 1007|       |    /// # Returns
 1008|       |    ///
 1009|       |    /// Returns `Ok(())` after all decode-finish values have been encoded.
 1010|       |    ///
 1011|       |    /// # Errors
 1012|       |    ///
 1013|       |    /// Returns a converter error when decode finish or encode handling fails.
 1014|       |    ///
 1015|       |    /// # Panics
 1016|       |    ///
 1017|       |    /// Panics when a decode-finish value cannot be encoded within the
 1018|       |    /// capacity reserved by [`Self::max_finish_output_len`].
 1019|  1.16k|    fn drain_decoder_finish(
 1020|  1.16k|        &mut self,
 1021|  1.16k|        state: &mut ConvertState<'_, D::Unit, E::Unit>,
 1022|  1.16k|    ) -> Result<(), TranscodeConvertErrorOf<D, E>>
 1023|  1.16k|    where
 1024|  1.16k|        D::Value: Default,
 1025|       |    {
 1026|  1.16k|        let value_count = self.decode_engine.max_finish_output_len()?;
                          ^1.15k                                                  ^1
 1027|  1.15k|        if value_count == 0 {
 1028|       |            // Skip the Vec allocation when the decoder declares no finish
 1029|       |            // output. We still call finish() so that
 1030|       |            // codec.decode_finish and hooks.finish_hooks both run —
 1031|       |            // hooks may do validation or teardown (e.g. checksum
 1032|       |            // verification) that can fail even when emitting zero
 1033|       |            // values. Passing an empty slice is safe here because the capacity
 1034|       |            // check inside finish() accepts required == 0 against any slice.
 1035|  1.15k|            self.decode_engine.finish(&mut [], 0)?;
                                                               ^3
 1036|  1.14k|            return Ok(());
 1037|      9|        }
 1038|       |        // D::Value: Default is required only when value_count > 0. The bound
 1039|       |        // remains on the method signature for the general case; stateless
 1040|       |        // codecs never reach this branch.
 1041|      9|        let mut decoded = Vec::new();
 1042|      9|        try_reserve_vec(&mut decoded, value_count).map_err(|_| TranscodeFailure::allocation_failed())?;
                                                                             ^1                                    ^1
 1043|      8|        decoded.resize_with(value_count, D::Value::default);
 1044|      8|        let written = self.decode_engine.finish(&mut decoded, 0)?;
                          ^6                                                  ^2
 1045|      7|        for value in decoded.into_iter().take(written) {
                                   ^6      ^6          ^6   ^6
 1046|      7|            let pending = PendingValue::new(value, 0);
 1047|      7|            let progress = self.encode_pending(pending, state)?;
                              ^5                                            ^2
 1048|      5|            assert_reserved_output_drained(
 1049|      5|                progress,
 1050|       |                "converter finish bound must reserve space for decode finish values",
 1051|       |            );
 1052|       |        }
 1053|      4|        Ok(())
 1054|  1.16k|    }
 1055|       |
 1056|       |    /// Encodes one pending value and applies output/pending state changes.
 1057|       |    ///
 1058|       |    /// When the target buffer lacks capacity, the value is put back into the
 1059|       |    /// pending slot and progress reports
 1060|       |    /// [`crate::TranscodeStatus::NeedOutput`].
 1061|       |    ///
 1062|       |    /// # Parameters
 1063|       |    ///
 1064|       |    /// - `pending`: Decoded value waiting for target output capacity.
 1065|       |    /// - `state`: Current conversion cursors and output buffer.
 1066|       |    ///
 1067|       |    /// # Returns
 1068|       |    ///
 1069|       |    /// Returns conversion progress when the value needs more output capacity,
 1070|       |    /// or `None` when the value was encoded or skipped by policy.
 1071|       |    ///
 1072|       |    /// # Errors
 1073|       |    ///
 1074|       |    /// Returns a converter error when encode hook handling fails.
 1075|  4.85k|    fn encode_pending(
 1076|  4.85k|        &mut self,
 1077|  4.85k|        pending: PendingValue<D::Value>,
 1078|  4.85k|        state: &mut ConvertState<'_, D::Unit, E::Unit>,
 1079|  4.85k|    ) -> Result<Option<TranscodeProgress>, TranscodeConvertErrorOf<D, E>> {
 1080|  4.85k|        let input_index = pending.input_index();
 1081|  4.85k|        let output_index = state.output_cursor();
 1082|  4.85k|        let attempt = EncodeAttempt::new(pending.value(), input_index, state.output_mut(), output_index);
 1083|  4.85k|        let outcome = match self.encode_engine.encode_one(attempt) {
                          ^4.84k
 1084|  4.84k|            Ok(outcome) => outcome,
 1085|      8|            Err(error) => {
 1086|      8|                return Err(TranscodeConvertError::from_encode_error_with_value(
 1087|      8|                    error,
 1088|      8|                    pending.into_value(),
 1089|      8|                ));
 1090|       |            }
 1091|       |        };
 1092|  4.84k|        let progress = state.apply_encode_outcome(outcome);
 1093|  4.84k|        if progress.is_some() {
 1094|    962|            self.pending.put(pending);
 1095|  3.88k|        }
 1096|  4.84k|        Ok(progress)
 1097|  4.85k|    }
 1098|       |}
 1099|       |
 1100|       |impl<D, E, DH, EH> Default for TranscodeConvertEngine<D, E, DH, EH>
 1101|       |where
 1102|       |    D: Codec + Default,
 1103|       |    E: Codec<Value = D::Value> + Default,
 1104|       |    DH: TranscodeDecodeHooks<D> + Default,
 1105|       |    EH: TranscodeEncodeHooks<E> + Default,
 1106|       |{
 1107|       |    /// Creates a default buffered converter engine.
 1108|       |    ///
 1109|       |    /// # Returns
 1110|       |    ///
 1111|       |    /// Returns a converter engine constructed from default codecs and hooks.
 1112|      2|    fn default() -> Self {
 1113|      2|        Self::new(D::default(), E::default(), DH::default(), EH::default())
 1114|      2|    }
 1115|       |}
 1116|       |
 1117|       |impl<D, E, DH, EH> Transcoder for TranscodeConvertEngine<D, E, DH, EH>
 1118|       |where
 1119|       |    D: Codec,
 1120|       |    E: Codec<Value = D::Value>,
 1121|       |    D::Value: Default,
 1122|       |    DH: TranscodeDecodeHooks<D>,
 1123|       |    EH: TranscodeEncodeHooks<E>,
 1124|       |{
 1125|       |    /// Source representation units accepted by the decoder.
 1126|       |    type Input = D::Unit;
 1127|       |    /// Target representation units emitted by the encoder.
 1128|       |    type Output = E::Unit;
 1129|       |    /// Source, target, contract, and capacity failures in the pipeline.
 1130|       |    type Error = TranscodeConvertErrorOf<D, E>;
 1131|       |
 1132|       |    /// Returns an upper bound for target units produced from `input_len`
 1133|       |    /// units.
 1134|       |    ///
 1135|       |    /// # Parameters
 1136|       |    ///
 1137|       |    /// - `input_len`: Number of source units to include in capacity planning.
 1138|       |    ///
 1139|       |    /// # Returns
 1140|       |    ///
 1141|       |    /// The target-unit capacity bound, including the applicable retained state.
 1142|       |    ///
 1143|       |    /// # Errors
 1144|       |    ///
 1145|       |    /// Returns [`CapacityError`] when a component bound or their sum overflows
 1146|       |    /// `usize`.
 1147|       |    #[inline]
 1148|    532|    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
 1149|    532|        TranscodeConvertEngine::max_transcode_output_len(self, input_len)
 1150|    532|    }
 1151|       |
 1152|       |    /// Returns an upper bound for target units emitted by finishing retained
 1153|       |    /// state.
 1154|       |    ///
 1155|       |    /// # Returns
 1156|       |    ///
 1157|       |    /// The target-unit capacity bound, including the applicable retained state.
 1158|       |    ///
 1159|       |    /// # Errors
 1160|       |    ///
 1161|       |    /// Returns [`CapacityError`] when a component bound or their sum overflows
 1162|       |    /// `usize`.
 1163|       |    #[inline]
 1164|    532|    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
 1165|    532|        TranscodeConvertEngine::max_finish_output_len(self)
 1166|    532|    }
 1167|       |
 1168|       |    /// Returns an upper bound for target units emitted when resetting stream
 1169|       |    /// state.
 1170|       |    ///
 1171|       |    /// # Returns
 1172|       |    ///
 1173|       |    /// The target-unit capacity bound, including the applicable retained state.
 1174|       |    ///
 1175|       |    /// # Errors
 1176|       |    ///
 1177|       |    /// Returns [`CapacityError`] when a component bound or their sum overflows
 1178|       |    /// `usize`.
 1179|       |    #[inline]
 1180|    532|    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
 1181|    532|        TranscodeConvertEngine::max_reset_output_len(self)
 1182|    532|    }
 1183|       |
 1184|       |    /// Clears retained conversion state and emits target reset output.
 1185|       |    ///
 1186|       |    /// # Parameters
 1187|       |    ///
 1188|       |    /// - `output`: Destination for lifecycle output units.
 1189|       |    /// - `output_index`: Starting index in `output`.
 1190|       |    ///
 1191|       |    /// # Returns
 1192|       |    ///
 1193|       |    /// The number of lifecycle output units written.
 1194|       |    ///
 1195|       |    /// # Errors
 1196|       |    ///
 1197|       |    /// Returns the engine error for invalid indices, insufficient capacity,
 1198|       |    /// an invalid lifecycle transition, or codec/hook failure. See the inherent
 1199|       |    /// [`TranscodeConvertEngine::reset`] method for retry and poisoning
 1200|       |    /// semantics.
 1201|       |    ///
 1202|       |    /// # Panics
 1203|       |    ///
 1204|       |    /// Panics if a codec/hook output bound fails to reserve enough space for
 1205|       |    /// emitted values.
 1206|       |    #[inline]
 1207|    532|    fn reset(&mut self, output: &mut [E::Unit], output_index: usize) -> Result<usize, TranscodeConvertErrorOf<D, E>> {
 1208|    532|        TranscodeConvertEngine::reset(self, output, output_index)
 1209|    532|    }
 1210|       |
 1211|       |    /// Converts source units into target units.
 1212|       |    ///
 1213|       |    /// # Parameters
 1214|       |    ///
 1215|       |    /// - `input`: Source units visible to this call.
 1216|       |    /// - `input_index`: Starting index in `input`.
 1217|       |    /// - `output`: Destination for target units.
 1218|       |    /// - `output_index`: Starting index in `output`.
 1219|       |    ///
 1220|       |    /// # Returns
 1221|       |    ///
 1222|       |    /// Consumed source units, written target units, and the stopping reason.
 1223|       |    ///
 1224|       |    /// # Errors
 1225|       |    ///
 1226|       |    /// Returns an error for invalid indices, a missing reset, a finished or
 1227|       |    /// poisoned stream, or codec/hook conversion failure.
 1228|       |    #[inline]
 1229|      1|    fn transcode(
 1230|      1|        &mut self,
 1231|      1|        input: &[D::Unit],
 1232|      1|        input_index: usize,
 1233|      1|        output: &mut [E::Unit],
 1234|      1|        output_index: usize,
 1235|      1|    ) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>> {
 1236|      1|        TranscodeConvertEngine::transcode(self, input, input_index, output, output_index)
 1237|      1|    }
 1238|       |
 1239|       |    /// Transcodes an input segment while marking it as the end of the stream.
 1240|       |    ///
 1241|       |    /// This forwarding implementation delegates to the engine's EOF-aware
 1242|       |    /// operation so the decoder can resolve trailing input using EOF rules.
 1243|       |    /// Call `finish` separately to drain retained lifecycle output.
 1244|       |    ///
 1245|       |    /// # Parameters
 1246|       |    ///
 1247|       |    /// - `input`: Source units visible to this call.
 1248|       |    /// - `input_index`: Starting index in `input`.
 1249|       |    /// - `output`: Destination for target units.
 1250|       |    /// - `output_index`: Starting index in `output`.
 1251|       |    ///
 1252|       |    /// # Returns
 1253|       |    ///
 1254|       |    /// Consumed source units, written target units, and the stopping reason.
 1255|       |    ///
 1256|       |    /// # Errors
 1257|       |    ///
 1258|       |    /// Returns an error for invalid indices, a missing reset, a finished or
 1259|       |    /// poisoned stream, or codec/hook conversion failure.
 1260|       |    #[inline]
 1261|    531|    fn transcode_eof(
 1262|    531|        &mut self,
 1263|    531|        input: &[D::Unit],
 1264|    531|        input_index: usize,
 1265|    531|        output: &mut [E::Unit],
 1266|    531|        output_index: usize,
 1267|    531|    ) -> Result<TranscodeProgress, TranscodeConvertErrorOf<D, E>> {
 1268|    531|        TranscodeConvertEngine::transcode_eof(self, input, input_index, output, output_index)
 1269|    531|    }
 1270|       |
 1271|       |    /// Finishes retained converter output after EOF.
 1272|       |    ///
 1273|       |    /// # Parameters
 1274|       |    ///
 1275|       |    /// - `output`: Destination for lifecycle output units.
 1276|       |    /// - `output_index`: Starting index in `output`.
 1277|       |    ///
 1278|       |    /// # Returns
 1279|       |    ///
 1280|       |    /// The number of lifecycle output units written.
 1281|       |    ///
 1282|       |    /// # Errors
 1283|       |    ///
 1284|       |    /// Returns the engine error for invalid indices, insufficient capacity,
 1285|       |    /// an invalid lifecycle transition, or codec/hook failure. See the inherent
 1286|       |    /// [`TranscodeConvertEngine::finish`] method for retry and poisoning
 1287|       |    /// semantics.
 1288|       |    ///
 1289|       |    /// # Panics
 1290|       |    ///
 1291|       |    /// Panics if a codec/hook output bound fails to reserve enough space for
 1292|       |    /// emitted values.
 1293|       |    #[inline]
 1294|    532|    fn finish(&mut self, output: &mut [E::Unit], output_index: usize) -> Result<usize, TranscodeConvertErrorOf<D, E>> {
 1295|    532|        TranscodeConvertEngine::finish(self, output, output_index)
 1296|    532|    }
 1297|       |}
 1298|       |
 1299|       |impl<D, E, DH, EH> TranscodeConverter for TranscodeConvertEngine<D, E, DH, EH>
 1300|       |where
 1301|       |    D: Codec,
 1302|       |    E: Codec<Value = D::Value>,
 1303|       |    D::Value: Default,
 1304|       |    DH: TranscodeDecodeHooks<D>,
 1305|       |    EH: TranscodeEncodeHooks<E>,
 1306|       |{
 1307|       |    /// Source codec failures retained in conversion errors.
 1308|       |    type DecodeError = D::DecodeError;
 1309|       |    /// Target codec failures retained in conversion errors.
 1310|       |    type EncodeError = E::EncodeError;
 1311|       |    /// Shared logical value transferred between the two codecs.
 1312|       |    type Value = D::Value;
 1313|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/engine/transcode_decode_engine.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Reusable buffered decoder engine.
    9|       |
   10|       |use core::num::NonZeroUsize;
   11|       |
   12|       |use super::super::internal::applied_decode_invalid_action::AppliedDecodeInvalidAction;
   13|       |use super::super::internal::decode_state::DecodeState;
   14|       |use super::super::internal::lifecycle_guard::LifecycleGuard;
   15|       |use super::DecodeContext;
   16|       |use super::DecodeIncompleteAction;
   17|       |use super::DecodeInvalidAction;
   18|       |use super::DecodeOutcome;
   19|       |use super::TranscodeDecodeHooks;
   20|       |use crate::CapacityError;
   21|       |use crate::Codec;
   22|       |use crate::DecodeFailure;
   23|       |use crate::TranscodeDecodeError;
   24|       |use crate::TranscodeDecodeErrorOf;
   25|       |use crate::TranscodeDecoder;
   26|       |use crate::TranscodeFailure;
   27|       |use crate::TranscodeProgress;
   28|       |use crate::Transcoder;
   29|       |use crate::codec::assert_unit_bounds;
   30|       |
   31|       |/// Reusable buffered decoding engine for codec-backed decoders.
   32|       |///
   33|       |/// The engine owns the low-level codec and hook object. It keeps the common
   34|       |/// buffered decoding loop private: input-index validation, output-capacity
   35|       |/// checks, calls to [`Codec::decode`], hook dispatch, and
   36|       |/// [`crate::TranscodeStatus`] reporting. Incomplete input tails are left in the
   37|       |/// caller-provided input slice; callers own input-buffer refill.
   38|       |///
   39|       |/// Use this type to build a streaming decoder over a one-value [`Codec`]. The
   40|       |/// engine decodes into a caller-provided output slice and returns
   41|       |/// [`TranscodeProgress`] instead of allocating. On success it writes decoded
   42|       |/// values directly to output. On codec errors it delegates to
   43|       |/// [`crate::engine::TranscodeDecodeHooks`], allowing a policy to skip invalid
   44|       |/// units, emit a replacement value, or fail.
   45|       |///
   46|       |/// The engine stops before reading an incomplete value when fewer than
   47|       |/// [`Codec::MIN_UNITS_PER_VALUE`] units are available. For variable-width
   48|       |/// codecs, the codec may still return an incomplete decode error after that
   49|       |/// minimum is satisfied. Both conditions return
   50|       |/// [`crate::TranscodeStatus::NeedInput`] while the stream is open; at EOF the
   51|       |/// hook selects whether to reject, skip, or replace the remaining tail.
   52|       |///
   53|       |/// For strict decoding that wraps codec errors, use
   54|       |/// [`crate::CodecTranscodeDecoder`]. Use `TranscodeDecodeEngine` directly when
   55|       |/// invalid input should be repaired, skipped, counted, or otherwise handled by
   56|       |/// policy.
   57|       |///
   58|       |/// # Examples
   59|       |///
   60|       |/// ```rust
   61|       |/// use core::num::NonZeroUsize;
   62|       |/// use qubit_codec::{
   63|       |///     Codec,
   64|       |///     TranscodeDecodeErrorOf,
   65|       |///     DecodeFailure,
   66|       |///     TranscodeStatus,
   67|       |/// };
   68|       |/// use qubit_codec::engine::{
   69|       |///     DecodeContext,
   70|       |///     DecodeInvalidAction,
   71|       |///     TranscodeDecodeEngine,
   72|       |///     TranscodeDecodeHooks,
   73|       |/// };
   74|       |///
   75|       |/// #[derive(Clone, Copy)]
   76|       |/// struct ByteCodec;
   77|       |///
   78|       |/// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
   79|       |/// enum ByteDecodeError {
   80|       |///     Malformed { consumed: NonZeroUsize },
   81|       |/// }
   82|       |///
   83|       |/// impl Codec for ByteCodec {
   84|       |///     type Value = u8;
   85|       |///     type Unit = u8;
   86|       |///     type DecodeError = ByteDecodeError;
   87|       |///     type EncodeError = core::convert::Infallible;
   88|       |///
   89|       |///     const MIN_UNITS_PER_VALUE: usize = 1;
   90|       |///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
   91|       |///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
   92|       |///
   93|       |///     unsafe fn decode(
   94|       |///         &mut self,
   95|       |///         input: &[u8],
   96|       |///         index: usize,
   97|       |///     ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
   98|       |///         match input[index] {
   99|       |///             0xff => Err(DecodeFailure::invalid(
  100|       |///                 ByteDecodeError::Malformed {
  101|       |///                     consumed: NonZeroUsize::MIN,
  102|       |///                 },
  103|       |///                 NonZeroUsize::MIN,
  104|       |///             )),
  105|       |///             value => Ok((value, NonZeroUsize::MIN)),
  106|       |///         }
  107|       |///     }
  108|       |///
  109|       |///     unsafe fn encode(
  110|       |///         &mut self,
  111|       |///         value: &u8,
  112|       |///         output: &mut [u8],
  113|       |///         index: usize,
  114|       |///     ) -> Result<usize, Self::EncodeError> {
  115|       |///         output[index] = *value;
  116|       |///         Ok(1)
  117|       |///     }
  118|       |/// }
  119|       |///
  120|       |/// struct ReplacementHooks;
  121|       |///
  122|       |/// impl TranscodeDecodeHooks<ByteCodec> for ReplacementHooks {
  123|       |///     fn handle_invalid_decode(
  124|       |///         &mut self,
  125|       |///         _codec: &mut ByteCodec,
  126|       |///         error: &ByteDecodeError,
  127|       |///         consumed: Option<NonZeroUsize>,
  128|       |///         _context: DecodeContext,
  129|       |///     ) -> Result<DecodeInvalidAction<u8>, TranscodeDecodeErrorOf<ByteCodec>> {
  130|       |///         match error {
  131|       |///             ByteDecodeError::Malformed { .. } => {
  132|       |///                 Ok(DecodeInvalidAction::Emit {
  133|       |///                     value: b'?',
  134|       |///                     consumed: consumed.expect("codec reported malformed width"),
  135|       |///                 })
  136|       |///             }
  137|       |///         }
  138|       |///     }
  139|       |/// }
  140|       |///
  141|       |/// let mut engine = TranscodeDecodeEngine::<_, _>::new(ByteCodec, ReplacementHooks);
  142|       |/// let input = [b'a', 0xff, b'b'];
  143|       |/// let mut output = [0_u8; 3];
  144|       |/// let mut reset_output = [];
  145|       |/// engine.reset(&mut reset_output, 0)?;
  146|       |///
  147|       |/// let progress = engine.transcode(&input, 0, &mut output, 0)?;
  148|       |/// match progress.status() {
  149|       |///     TranscodeStatus::Complete => assert_eq!(&output[..progress.written()], b"a?b"),
  150|       |///     TranscodeStatus::NeedInput { .. } => {
  151|       |///         // Keep `input[input_index + progress.read()..]`, append more source
  152|       |///         // units, and resume.
  153|       |///     }
  154|       |///     TranscodeStatus::NeedOutput { .. } => {
  155|       |///         // Drain `output[..progress.written()]`, then resume with more output
  156|       |///         // room.
  157|       |///     }
  158|       |/// }
  159|       |/// # Ok::<(), qubit_codec::TranscodeDecodeError<ByteDecodeError>>(())
  160|       |/// ```
  161|       |///
  162|       |/// # Type Parameters
  163|       |///
  164|       |/// - `C`: Low-level codec used by the engine.
  165|       |/// - `H`: Policy hook object used by the engine.
  166|       |#[derive(Debug)]
  167|       |pub struct TranscodeDecodeEngine<C, H> {
  168|       |    /// Low-level codec used for one-value decoding.
  169|       |    pub(super) codec: C,
  170|       |    /// Policy hooks used for decode failures.
  171|       |    pub(super) hooks: H,
  172|       |    /// Guard for the `reset → transcode* → finish` lifecycle in every build
  173|       |    /// profile.
  174|       |    lifecycle: LifecycleGuard,
  175|       |}
  176|       |
  177|       |impl<C, H> Default for TranscodeDecodeEngine<C, H>
  178|       |where
  179|       |    C: Codec + Default,
  180|       |    H: TranscodeDecodeHooks<C> + Default,
  181|       |{
  182|       |    /// Creates an engine with the same codec-bound checks as [`Self::new`].
  183|       |    ///
  184|       |    /// # Returns
  185|       |    ///
  186|       |    /// Returns a fresh engine containing default codec and hook values.
  187|       |    ///
  188|       |    /// # Compile-Time Checks
  189|       |    ///
  190|       |    /// Fails to compile when the codec declares a zero decode unit bound or
  191|       |    /// when [`Codec::MIN_UNITS_PER_VALUE`] exceeds
  192|       |    /// [`Codec::MAX_DECODE_UNITS_PER_VALUE`], matching [`Self::new`].
  193|       |    ///
  194|       |    /// A legal codec and hook can use the default constructor:
  195|       |    ///
  196|       |    /// ```rust
  197|       |    /// use core::convert::Infallible;
  198|       |    /// use core::num::NonZeroUsize;
  199|       |    /// use qubit_codec::{Codec, DecodeFailure};
  200|       |    /// use qubit_codec::engine::{DecodeContext, DecodeInvalidAction, TranscodeDecodeEngine, TranscodeDecodeHooks};
  201|       |    ///
  202|       |    /// #[derive(Default)]
  203|       |    /// struct ByteCodec;
  204|       |    /// impl Codec for ByteCodec {
  205|       |    ///     type Value = u8;
  206|       |    ///     type Unit = u8;
  207|       |    ///     type DecodeError = Infallible;
  208|       |    ///     type EncodeError = Infallible;
  209|       |    ///     const MIN_UNITS_PER_VALUE: usize = 1;
  210|       |    ///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
  211|       |    ///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
  212|       |    ///     unsafe fn decode(&mut self, input: &[u8], index: usize) -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
  213|       |    ///         Ok((input[index], NonZeroUsize::MIN))
  214|       |    ///     }
  215|       |    ///     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Infallible> {
  216|       |    ///         output[index] = *value;
  217|       |    ///         Ok(1)
  218|       |    ///     }
  219|       |    /// }
  220|       |    /// #[derive(Default)]
  221|       |    /// struct Hooks;
  222|       |    /// impl TranscodeDecodeHooks<ByteCodec> for Hooks {
  223|       |    ///     fn handle_invalid_decode(&mut self, _codec: &mut ByteCodec, error: &Infallible, _consumed: Option<NonZeroUsize>, _context: DecodeContext) -> Result<DecodeInvalidAction<u8>, qubit_codec::TranscodeDecodeError<Infallible>> {
  224|       |    ///         match *error {}
  225|       |    ///     }
  226|       |    /// }
  227|       |    /// let _engine = TranscodeDecodeEngine::<ByteCodec, Hooks>::default();
  228|       |    /// ```
  229|       |    ///
  230|       |    /// A zero minimum decode bound is rejected:
  231|       |    ///
  232|       |    /// ```compile_fail
  233|       |    /// use core::convert::Infallible;
  234|       |    /// use core::num::NonZeroUsize;
  235|       |    /// use qubit_codec::{Codec, DecodeFailure};
  236|       |    /// use qubit_codec::engine::{DecodeContext, DecodeInvalidAction, TranscodeDecodeEngine, TranscodeDecodeHooks};
  237|       |    /// #[derive(Default)] struct BadCodec;
  238|       |    /// impl Codec for BadCodec {
  239|       |    ///     type Value = u8; type Unit = u8; type DecodeError = Infallible; type EncodeError = Infallible;
  240|       |    ///     const MIN_UNITS_PER_VALUE: usize = 0; const MAX_ENCODE_UNITS_PER_VALUE: usize = 1; const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
  241|       |    ///     unsafe fn decode(&mut self, input: &[u8], index: usize) -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> { Ok((input[index], NonZeroUsize::MIN)) }
  242|       |    ///     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Infallible> { output[index] = *value; Ok(1) }
  243|       |    /// }
  244|       |    /// #[derive(Default)] struct Hooks;
  245|       |    /// impl TranscodeDecodeHooks<BadCodec> for Hooks {
  246|       |    ///     fn handle_invalid_decode(&mut self, _codec: &mut BadCodec, error: &Infallible, _consumed: Option<NonZeroUsize>, _context: DecodeContext) -> Result<DecodeInvalidAction<u8>, qubit_codec::TranscodeDecodeError<Infallible>> { match *error {} }
  247|       |    /// }
  248|       |    /// let _ = TranscodeDecodeEngine::<BadCodec, Hooks>::default();
  249|       |    /// ```
  250|       |    ///
  251|       |    /// A minimum larger than the maximum decode width is also rejected:
  252|       |    ///
  253|       |    /// ```compile_fail
  254|       |    /// use core::convert::Infallible;
  255|       |    /// use core::num::NonZeroUsize;
  256|       |    /// use qubit_codec::{Codec, DecodeFailure};
  257|       |    /// use qubit_codec::engine::{DecodeContext, DecodeInvalidAction, TranscodeDecodeEngine, TranscodeDecodeHooks};
  258|       |    /// #[derive(Default)] struct BadCodec;
  259|       |    /// impl Codec for BadCodec {
  260|       |    ///     type Value = u8; type Unit = u8; type DecodeError = Infallible; type EncodeError = Infallible;
  261|       |    ///     const MIN_UNITS_PER_VALUE: usize = 2; const MAX_ENCODE_UNITS_PER_VALUE: usize = 1; const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
  262|       |    ///     unsafe fn decode(&mut self, input: &[u8], index: usize) -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> { Ok((input[index], NonZeroUsize::MIN)) }
  263|       |    ///     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Infallible> { output[index] = *value; Ok(1) }
  264|       |    /// }
  265|       |    /// #[derive(Default)] struct Hooks;
  266|       |    /// impl TranscodeDecodeHooks<BadCodec> for Hooks {
  267|       |    ///     fn handle_invalid_decode(&mut self, _codec: &mut BadCodec, error: &Infallible, _consumed: Option<NonZeroUsize>, _context: DecodeContext) -> Result<DecodeInvalidAction<u8>, qubit_codec::TranscodeDecodeError<Infallible>> { match *error {} }
  268|       |    /// }
  269|       |    /// let _ = TranscodeDecodeEngine::<BadCodec, Hooks>::default();
  270|       |    /// ```
  271|       |    #[inline]
  272|      1|    fn default() -> Self {
  273|      1|        Self::new(C::default(), H::default())
  274|      1|    }
  275|       |}
  276|       |
  277|       |/// Adds two independent decoded-value capacity bounds.
  278|       |///
  279|       |/// # Parameters
  280|       |///
  281|       |/// - `first`: First decoded-value bound.
  282|       |/// - `second`: Independent decoded-value bound to add.
  283|       |///
  284|       |/// # Returns
  285|       |///
  286|       |/// Returns the combined decoded-value bound.
  287|       |///
  288|       |/// # Errors
  289|       |///
  290|       |/// Returns `CapacityError::OutputLengthOverflow` if the sum exceeds
  291|       |/// `usize::MAX`.
  292|       |#[inline]
  293|  4.05k|fn add_decode_output_bounds(first: usize, second: usize) -> Result<usize, CapacityError> {
  294|  4.05k|    first.checked_add(second).ok_or(CapacityError::OutputLengthOverflow)
  295|  4.05k|}
  296|       |
  297|       |/// Adds reset, transcode, and finish decoded-value capacity bounds.
  298|       |///
  299|       |/// # Parameters
  300|       |///
  301|       |/// - `reset`: Maximum reset output.
  302|       |/// - `transcode`: Maximum streaming output.
  303|       |/// - `finish`: Maximum finalization output.
  304|       |///
  305|       |/// # Returns
  306|       |///
  307|       |/// Returns the complete-stream decoded-value bound.
  308|       |///
  309|       |/// # Errors
  310|       |///
  311|       |/// Returns `CapacityError::OutputLengthOverflow` if the sum exceeds
  312|       |/// `usize::MAX`.
  313|       |#[inline]
  314|      3|fn sum_decode_output_bounds(reset: usize, transcode: usize, finish: usize) -> Result<usize, CapacityError> {
  315|      3|    let before_finish = add_decode_output_bounds(reset, transcode)?;
                      ^2                                                        ^1
  316|      2|    add_decode_output_bounds(before_finish, finish)
  317|      3|}
  318|       |
  319|       |impl<C, H> TranscodeDecodeEngine<C, H>
  320|       |where
  321|       |    C: Codec,
  322|       |    H: TranscodeDecodeHooks<C>,
  323|       |{
  324|       |    /// Creates a buffered decoder engine.
  325|       |    ///
  326|       |    /// # Parameters
  327|       |    ///
  328|       |    /// - `codec`: Low-level codec used for one-value decoding.
  329|       |    /// - `hooks`: Policy hooks used for decode failures.
  330|       |    ///
  331|       |    /// # Returns
  332|       |    ///
  333|       |    /// Returns a buffered decoder engine.
  334|       |    ///
  335|       |    /// # Compile-Time Checks
  336|       |    ///
  337|       |    /// Fails to compile when the supplied codec declares a zero decode unit
  338|       |    /// bound or when [`Codec::MIN_UNITS_PER_VALUE`] exceeds
  339|       |    /// [`Codec::MAX_DECODE_UNITS_PER_VALUE`].
  340|       |    #[inline]
  341|       |    #[must_use]
  342|  1.32k|    pub fn new(codec: C, hooks: H) -> Self {
  343|  1.32k|        assert_unit_bounds::<C>();
  344|  1.32k|        Self {
  345|  1.32k|            codec,
  346|  1.32k|            hooks,
  347|  1.32k|            lifecycle: LifecycleGuard::new(),
  348|  1.32k|        }
  349|  1.32k|    }
  350|       |
  351|       |    /// Returns the wrapped low-level codec.
  352|       |    ///
  353|       |    /// # Returns
  354|       |    ///
  355|       |    /// Returns a shared reference to the codec owned by this engine.
  356|       |    #[inline]
  357|       |    #[must_use]
  358|      4|    pub const fn codec(&self) -> &C {
  359|      4|        &self.codec
  360|      4|    }
  361|       |
  362|       |    /// Returns the wrapped low-level codec mutably.
  363|       |    ///
  364|       |    /// # Returns
  365|       |    ///
  366|       |    /// Returns a mutable reference to the codec owned by this engine.
  367|       |    #[inline]
  368|       |    #[must_use]
  369|      5|    pub fn codec_mut(&mut self) -> &mut C {
  370|      5|        &mut self.codec
  371|      5|    }
  372|       |
  373|       |    /// Returns the decode hooks used by this engine.
  374|       |    ///
  375|       |    /// # Returns
  376|       |    ///
  377|       |    /// Returns a shared reference to the hook object owned by this engine.
  378|       |    #[inline]
  379|       |    #[must_use]
  380|     11|    pub const fn hooks(&self) -> &H {
  381|     11|        &self.hooks
  382|     11|    }
  383|       |
  384|       |    /// Returns the decode hooks mutably.
  385|       |    ///
  386|       |    /// # Returns
  387|       |    ///
  388|       |    /// Returns a mutable reference to the hook object owned by this engine.
  389|       |    #[inline]
  390|       |    #[must_use]
  391|      2|    pub fn hooks_mut(&mut self) -> &mut H {
  392|      2|        &mut self.hooks
  393|      2|    }
  394|       |
  395|       |    /// Consumes the engine and returns its codec and hooks.
  396|       |    ///
  397|       |    /// Any lifecycle state owned by the engine is discarded.
  398|       |    ///
  399|       |    /// # Returns
  400|       |    ///
  401|       |    /// Returns the wrapped codec followed by the decode hooks.
  402|       |    #[inline]
  403|       |    #[must_use]
  404|      4|    pub fn into_parts(self) -> (C, H) {
  405|      4|        let Self { codec, hooks, .. } = self;
  406|      4|        (codec, hooks)
  407|      4|    }
  408|       |
  409|       |    /// Returns an upper bound for decoded values produced from `input_len`
  410|       |    /// units.
  411|       |    ///
  412|       |    /// This bound covers only the streaming decode phase. It is delegated to
  413|       |    /// [`TranscodeDecodeHooks::max_transcode_output_len`], so it includes hook
  414|       |    /// policy and is valid for every reachable transient codec and hook state.
  415|       |    /// Downstream decoders must use this engine-level API for capacity planning
  416|       |    /// instead of recomputing the bound from [`Codec`] constants.
  417|       |    ///
  418|       |    /// # Parameters
  419|       |    ///
  420|       |    /// - `input_len`: Number of source units the caller plans to decode.
  421|       |    ///
  422|       |    /// # Returns
  423|       |    ///
  424|       |    /// Returns a conservative upper bound, or a capacity error on arithmetic
  425|       |    /// overflow.
  426|       |    ///
  427|       |    /// # Errors
  428|       |    ///
  429|       |    /// Returns a capacity error when hook capacity arithmetic overflows.
  430|       |    #[inline]
  431|       |    #[must_use = "capacity planning can fail on overflow"]
  432|  1.11k|    pub fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
  433|  1.11k|        self.hooks.max_transcode_output_len(&self.codec, input_len)
  434|  1.11k|    }
  435|       |
  436|       |    /// Returns the global maximum values emitted by finishing codec state and
  437|       |    /// finishing hook-owned state.
  438|       |    ///
  439|       |    /// # Returns
  440|       |    ///
  441|       |    /// Returns the sum of [`Codec::MAX_DECODE_FINISH_VALUES`] and the
  442|       |    /// hook-provided final-output bound. The codec finish portion covers values
  443|       |    /// written by [`Codec::decode_finish`]; hook implementations must not
  444|       |    /// include that portion in
  445|       |    /// [`TranscodeDecodeHooks::max_finish_output_len`]. Both component bounds
  446|       |    /// cover every reachable transient state.
  447|       |    ///
  448|       |    /// # Errors
  449|       |    ///
  450|       |    /// Returns a capacity error when the combined finish bound exceeds
  451|       |    /// `usize::MAX`.
  452|       |    #[inline]
  453|       |    #[must_use = "capacity planning can fail on overflow"]
  454|  4.05k|    pub fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
  455|  4.05k|        add_decode_output_bounds(
  456|       |            C::MAX_DECODE_FINISH_VALUES,
  457|  4.05k|            self.hooks.max_finish_output_len(&self.codec),
  458|       |        )
  459|  4.05k|    }
  460|       |
  461|       |    /// Returns the global maximum values emitted when resetting stream state.
  462|       |    ///
  463|       |    /// # Returns
  464|       |    ///
  465|       |    /// Returns `Ok(Codec::MAX_DECODE_RESET_VALUES)` for the wrapped codec.
  466|       |    /// This query cannot fail.
  467|       |    /// Stateless decoders always return `0`.
  468|       |    #[inline]
  469|     14|    pub fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
  470|     14|        Ok(C::MAX_DECODE_RESET_VALUES)
  471|     14|    }
  472|       |
  473|       |    /// Returns the maximum values needed by a complete one-shot decode stream.
  474|       |    ///
  475|       |    /// The returned bound covers reset output, the streaming decode phase for
  476|       |    /// `input_len` units, and finish output. Its components are global across
  477|       |    /// transient state. Higher-level complete decode helpers should use this
  478|       |    /// engine-level bound instead of recomputing capacity from [`Codec`]
  479|       |    /// constants, because hook policy may change streaming or finish output.
  480|       |    ///
  481|       |    /// # Parameters
  482|       |    ///
  483|       |    /// - `input_len`: Number of source units in the complete stream.
  484|       |    ///
  485|       |    /// # Returns
  486|       |    ///
  487|       |    /// Returns the complete-stream output bound, or a capacity error on
  488|       |    /// arithmetic overflow.
  489|       |    ///
  490|       |    /// # Errors
  491|       |    ///
  492|       |    /// Returns a capacity error when a component or their sum overflows.
  493|       |    #[inline]
  494|       |    #[must_use = "capacity planning can fail on overflow"]
  495|      5|    pub fn max_total_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
  496|      5|        let transcode = self.max_transcode_output_len(input_len)?;
                          ^4                                                  ^1
  497|      4|        let finish = self.max_finish_output_len()?;
                          ^3                                   ^1
  498|      3|        sum_decode_output_bounds(C::MAX_DECODE_RESET_VALUES, transcode, finish)
  499|      5|    }
  500|       |
  501|       |    /// Resets codec decode state, runs reset hooks, and emits stream-start
  502|       |    /// values.
  503|       |    ///
  504|       |    /// The sequence is: validate capacity → run `reset_hooks` → call
  505|       |    /// [`Codec::decode_reset`]. Stateless decoders (`MAX_DECODE_RESET_VALUES
  506|       |    /// == 0`) write nothing and return `Ok(0)`.
  507|       |    ///
  508|       |    /// # Parameters
  509|       |    ///
  510|       |    /// - `output`: Complete output value slice visible to the decoder.
  511|       |    /// - `output_index`: Absolute output value index where writing starts.
  512|       |    ///
  513|       |    /// # Returns
  514|       |    ///
  515|       |    /// Returns the number of reset values written.
  516|       |    ///
  517|       |    /// # Errors
  518|       |    ///
  519|       |    /// Returns framework errors when the caller provides invalid or
  520|       |    /// insufficient output capacity. Returns domain errors when codec reset
  521|       |    /// or hook reset handling fails. Capacity and index failures occur before
  522|       |    /// any reset state is changed. Once reset execution starts, an error
  523|       |    /// poisons the engine until a later reset succeeds.
  524|       |    ///
  525|       |    /// # Panics
  526|       |    ///
  527|       |    /// Panics if the codec reports more than `MAX_DECODE_RESET_VALUES` written.
  528|  1.31k|    pub fn reset(&mut self, output: &mut [C::Value], output_index: usize) -> Result<usize, TranscodeDecodeErrorOf<C>> {
  529|  1.31k|        let required = C::MAX_DECODE_RESET_VALUES;
  530|  1.31k|        TranscodeFailure::ensure_output_capacity(output.len(), output_index, required)?;
                                                                                                    ^3
  531|  1.31k|        self.lifecycle.on_reset_start();
  532|  1.31k|        self.hooks.reset_hooks(&mut self.codec);
  533|  1.30k|        let written = unsafe {
  534|       |            // SAFETY: The capacity check above reserves the codec's declared
  535|       |            // reset-output bound at `output_index`.
  536|  1.31k|            self.codec.decode_reset(output, output_index)
  537|       |        }
  538|  1.31k|        .map_err(TranscodeDecodeError::domain_reset)?;
                                                                  ^6
  539|  1.30k|        assert!(written <= required, "Codec::decode_reset wrote beyond its reset bound",);
  540|  1.30k|        self.lifecycle.on_reset_success();
  541|  1.30k|        Ok(written)
  542|  1.31k|    }
  543|       |
  544|       |    /// Decodes source units into caller-provided output values.
  545|       |    ///
  546|       |    /// # Parameters
  547|       |    ///
  548|       |    /// - `input`: Complete input unit slice visible to the decoder.
  549|       |    /// - `input_index`: Absolute input unit index where decoding starts.
  550|       |    /// - `output`: Complete output value slice visible to the decoder.
  551|       |    /// - `output_index`: Absolute output value index where writing starts.
  552|       |    ///
  553|       |    /// # Returns
  554|       |    ///
  555|       |    /// Returns progress describing input units consumed, output values written,
  556|       |    /// and why decoding stopped.
  557|       |    ///
  558|       |    /// # Errors
  559|       |    ///
  560|       |    /// Returns framework errors when either index is outside its slice, and
  561|       |    /// domain errors when a concrete policy hook rejects a value. Returns
  562|       |    /// [`TranscodeFailure::TranscodeBeforeReset`] when the engine has not
  563|       |    /// completed its first reset. Returns
  564|       |    /// [`TranscodeFailure::TranscodeAfterFinish`] when the logical stream was
  565|       |    /// already finished and has not been reset, or
  566|       |    /// [`TranscodeFailure::LifecyclePoisoned`] when an earlier reset or finish
  567|       |    /// failed after execution started.
  568|       |    ///
  569|       |    /// # Panics
  570|       |    ///
  571|       |    /// Panics when codec or hook progress violates declared decode bounds.
  572|       |    #[inline]
  573|     50|    pub fn transcode(
  574|     50|        &mut self,
  575|     50|        input: &[C::Unit],
  576|     50|        input_index: usize,
  577|     50|        output: &mut [C::Value],
  578|     50|        output_index: usize,
  579|     50|    ) -> Result<TranscodeProgress, TranscodeDecodeErrorOf<C>> {
  580|     50|        self.transcode_with_eof(input, input_index, output, output_index, false)
  581|     50|    }
  582|       |
  583|       |    /// Decodes source units after the caller has established end of input.
  584|       |    ///
  585|       |    /// This follows the normal streaming contract, except that codec attempts
  586|       |    /// use [`Codec::decode_eof`]. A codec may therefore resolve a trailing
  587|       |    /// prefix that would remain incomplete in an open stream.
  588|       |    ///
  589|       |    /// # Parameters
  590|       |    ///
  591|       |    /// - `input`: Complete source-unit slice.
  592|       |    /// - `input_index`: Absolute index of the next source unit.
  593|       |    /// - `output`: Destination for decoded values.
  594|       |    /// - `output_index`: Absolute index of the next destination slot.
  595|       |    ///
  596|       |    /// # Returns
  597|       |    ///
  598|       |    /// Returns consumed/written counts and the reason decoding stopped.
  599|       |    ///
  600|       |    /// # Errors
  601|       |    ///
  602|       |    /// At confirmed EOF, an incomplete tail requires one writable output slot
  603|       |    /// before the incomplete-input hook runs, including when that hook might
  604|       |    /// skip or reject. A capacity shortage leaves both the tail and hook state
  605|       |    /// untouched; errors after hook execution do not roll back hook mutations.
  606|       |    ///
  607|       |    /// Returns framework errors for invalid lifecycle state or slice indices,
  608|       |    /// and policy/domain errors for rejected input, including incomplete EOF
  609|       |    /// tails.
  610|       |    ///
  611|       |    /// # Panics
  612|       |    ///
  613|       |    /// Panics when codec or hook progress violates declared decode bounds.
  614|       |    #[inline]
  615|     26|    pub fn transcode_eof(
  616|     26|        &mut self,
  617|     26|        input: &[C::Unit],
  618|     26|        input_index: usize,
  619|     26|        output: &mut [C::Value],
  620|     26|        output_index: usize,
  621|     26|    ) -> Result<TranscodeProgress, TranscodeDecodeErrorOf<C>> {
  622|     26|        self.transcode_with_eof(input, input_index, output, output_index, true)
  623|     26|    }
  624|       |
  625|       |    /// Finishes codec and hook-owned output after EOF.
  626|       |    ///
  627|       |    /// Finalization first finishes decode-side codec state through
  628|       |    /// [`Codec::decode_finish`], then lets hook implementations finish their
  629|       |    /// own retained state. The caller must provide enough output capacity for
  630|       |    /// [`TranscodeDecodeEngine::max_finish_output_len`], which includes both
  631|       |    /// the codec finish bound and the hook-owned finish bound.
  632|       |    ///
  633|       |    /// # Parameters
  634|       |    ///
  635|       |    /// - `output`: Complete output value slice visible to the decoder.
  636|       |    /// - `output_index`: Absolute output value index where writing starts.
  637|       |    ///
  638|       |    /// # Returns
  639|       |    ///
  640|       |    /// Returns the number of values written by finalization.
  641|       |    ///
  642|       |    /// # Errors
  643|       |    ///
  644|       |    /// Returns framework errors when the caller provides invalid or
  645|       |    /// insufficient output capacity. Returns domain errors when codec finish or
  646|       |    /// hook finalization fails. Returns
  647|       |    /// [`TranscodeFailure::FinishBeforeReset`] when the engine has not
  648|       |    /// completed its first reset. Returns
  649|       |    /// [`TranscodeFailure::FinishAfterFinish`] when the logical stream was
  650|       |    /// already finished and has not been reset, or
  651|       |    /// [`TranscodeFailure::LifecyclePoisoned`] when an earlier reset or finish
  652|       |    /// failed after execution started. Capacity and index failures occur before
  653|       |    /// finish execution and remain retryable; later failures poison the engine
  654|       |    /// until reset succeeds.
  655|       |    ///
  656|       |    /// # Panics
  657|       |    ///
  658|       |    /// Panics when the codec finish or finish hooks write beyond their
  659|       |    /// independently declared output bounds.
  660|  1.19k|    pub fn finish(&mut self, output: &mut [C::Value], output_index: usize) -> Result<usize, TranscodeDecodeErrorOf<C>> {
  661|  1.19k|        self.lifecycle.on_finish_attempt()?;
                                                        ^3
  662|  1.19k|        let hook_required = self.hooks.max_finish_output_len(&self.codec);
  663|  1.19k|        let required = C::MAX_DECODE_FINISH_VALUES
  664|  1.19k|            .checked_add(hook_required)
  665|  1.19k|            .ok_or(CapacityError::OutputLengthOverflow)?;
                                                                     ^1
  666|  1.19k|        TranscodeFailure::ensure_output_capacity(output.len(), output_index, required)?;
                                                                                                    ^5
  667|  1.19k|        self.lifecycle.on_finish_start();
  668|  1.18k|        let finished =
  669|  1.19k|            unsafe { self.codec.decode_finish(output, output_index) }.map_err(TranscodeDecodeError::domain_finish)?;
                                                                                                                                ^7
  670|  1.18k|        assert!(
  671|  1.18k|            finished <= C::MAX_DECODE_FINISH_VALUES,
  672|       |            "Codec::decode_finish wrote beyond its finish bound",
  673|       |        );
  674|  1.18k|        let written = self
                          ^1.18k
  675|  1.18k|            .hooks
  676|  1.18k|            .finish_hooks(&mut self.codec, output, output_index + finished)?;
                                                                                         ^2
  677|  1.18k|        assert!(
  678|  1.18k|            written <= hook_required,
  679|       |            "TranscodeDecodeEngine hook wrote beyond its finish bound",
  680|       |        );
  681|  1.17k|        self.lifecycle.on_finish_success();
  682|  1.17k|        Ok(finished + written)
  683|  1.19k|    }
  684|       |
  685|       |    /// Runs a complete one-shot `reset -> transcode -> finish` decode stream.
  686|       |    ///
  687|       |    /// The complete input is supplied as `input`, and output starts at index
  688|       |    /// `0` in `output`. Callers that need subranges should slice their
  689|       |    /// buffers before calling this method. Downstream one-shot decoder
  690|       |    /// helpers should call this engine method instead of reproducing the
  691|       |    /// reset, transcode, and finish sequence themselves.
  692|       |    ///
  693|       |    /// # Parameters
  694|       |    ///
  695|       |    /// - `input`: Complete source unit slice.
  696|       |    /// - `output`: Output value slice for the whole decoded stream.
  697|       |    ///
  698|       |    /// # Returns
  699|       |    ///
  700|       |    /// Returns the number of output values written.
  701|       |    ///
  702|       |    /// # Errors
  703|       |    ///
  704|       |    /// Returns framework errors for insufficient output, capacity overflow, or
  705|       |    /// an incomplete EOF tail, and domain errors from reset, decode, or
  706|       |    /// finish.
  707|       |    ///
  708|       |    /// # Panics
  709|       |    ///
  710|       |    /// Panics when codec or hook output violates reset, decode, or finish
  711|       |    /// bounds.
  712|       |    #[inline]
  713|      1|    pub fn transcode_complete_into(
  714|      1|        &mut self,
  715|      1|        input: &[C::Unit],
  716|      1|        output: &mut [C::Value],
  717|      1|    ) -> Result<usize, TranscodeDecodeErrorOf<C>> {
  718|      1|        <Self as Transcoder>::transcode_complete_into(self, input, output)
  719|      1|    }
  720|       |
  721|       |    /// Decodes one source value attempt and delivers emitted values.
  722|       |    ///
  723|       |    /// # Parameters
  724|       |    ///
  725|       |    /// - `input`: Complete input unit slice visible to the caller.
  726|       |    /// - `context`: Decode context describing the current source and output
  727|       |    ///   cursors.
  728|       |    /// - `end_of_input`: Whether the source is final and EOF policy applies.
  729|       |    /// - `consume`: Callback invoked exactly once when this attempt emits a
  730|       |    ///   logical value.
  731|       |    ///
  732|       |    /// # Type Parameters
  733|       |    ///
  734|       |    /// - `R`: Value returned by the consumer when a decoded value is emitted.
  735|       |    /// - `F`: Consumer callback type.
  736|       |    ///
  737|       |    /// # Returns
  738|       |    ///
  739|       |    /// Returns the decode outcome and `Some` consumer result when a value was
  740|       |    /// emitted; returns `None` when input was skipped or more input is needed.
  741|       |    ///
  742|       |    /// # Errors
  743|       |    ///
  744|       |    /// Returns hook errors when the decode policy rejects the input.
  745|       |    ///
  746|       |    /// # Panics
  747|       |    ///
  748|       |    /// Panics when the codec reports consumption beyond the available input,
  749|       |    /// an incomplete-input requirement beyond
  750|       |    /// [`Codec::MAX_DECODE_UNITS_PER_VALUE`], or hooks return an action that
  751|       |    /// consumes beyond the available input. Also panics when reported consumed
  752|       |    /// width exceeds the codec maximum or an incomplete requirement does not
  753|       |    /// exceed available input. Debug builds reject an empty context.
  754|  4.49k|    pub(crate) fn decode_one<R, F>(
  755|  4.49k|        &mut self,
  756|  4.49k|        input: &[C::Unit],
  757|  4.49k|        context: DecodeContext,
  758|  4.49k|        end_of_input: bool,
  759|  4.49k|        consume: F,
  760|  4.49k|    ) -> Result<(DecodeOutcome, Option<R>), TranscodeDecodeErrorOf<C>>
  761|  4.49k|    where
  762|  4.49k|        F: FnOnce(C::Value, usize) -> R,
  763|       |    {
  764|  4.49k|        debug_assert!(
  765|  4.49k|            context.available() > 0,
  766|       |            "decode_one requires at least one source input unit",
  767|       |        );
  768|       |
  769|  4.49k|        if context.available() < C::MIN_UNITS_PER_VALUE {
  770|     46|            let required_total =
  771|     46|                NonZeroUsize::new(C::MIN_UNITS_PER_VALUE).expect("Codec::MIN_UNITS_PER_VALUE is non-zero");
  772|     46|            if !end_of_input {
  773|      0|                return Ok((DecodeOutcome::need_input(required_total), None));
  774|     46|            }
  775|     46|            return self.apply_incomplete_decode(None, required_total, context, consume);
  776|  4.45k|        }
  777|       |
  778|       |        // SAFETY: The context reports at least `MIN_UNITS_PER_VALUE` source
  779|       |        // units available from `context.input_index()`.
  780|  4.45k|        let result = unsafe {
  781|  4.45k|            if end_of_input {
  782|  2.09k|                self.codec.decode_eof(input, context.input_index())
  783|       |            } else {
  784|  2.35k|                self.codec.decode(input, context.input_index())
  785|       |            }
  786|       |        };
  787|  1.05k|        match result {
  788|  3.40k|            Ok((value, consumed)) => {
  789|  3.40k|                assert!(
  790|  3.40k|                    consumed.get() <= context.available(),
  791|       |                    "Codec::decode consumed beyond available input",
  792|       |                );
  793|  3.40k|                assert!(
  794|  3.40k|                    consumed.get() <= C::MAX_DECODE_UNITS_PER_VALUE,
  795|       |                    "Codec::decode consumed beyond Codec::MAX_DECODE_UNITS_PER_VALUE",
  796|       |                );
  797|  3.39k|                let consumed_value = consume(value, context.input_index());
  798|  3.39k|                Ok((
  799|  3.39k|                    DecodeOutcome::emitted(consumed, NonZeroUsize::MIN),
  800|  3.39k|                    Some(consumed_value),
  801|  3.39k|                ))
  802|       |            }
  803|     10|            Err(DecodeFailure::Incomplete { source, required_total }) => {
  804|     10|                assert!(
  805|     10|                    required_total.get() > context.available(),
  806|       |                    "Codec::decode incomplete required_total must exceed available input",
  807|       |                );
  808|     10|                assert!(
  809|     10|                    required_total.get() <= C::MAX_DECODE_UNITS_PER_VALUE,
  810|       |                    "Codec::decode incomplete required_total exceeded Codec::MAX_DECODE_UNITS_PER_VALUE",
  811|       |                );
  812|      9|                if !end_of_input {
  813|      6|                    return Ok((DecodeOutcome::need_input(required_total), None));
  814|      3|                }
  815|      3|                self.apply_incomplete_decode(source, required_total, context, consume)
  816|       |            }
  817|  1.04k|            Err(DecodeFailure::Invalid { source, consumed }) => {
  818|  1.04k|                let action = match self
                                  ^1.03k
  819|  1.04k|                    .hooks
  820|  1.04k|                    .handle_invalid_decode(&mut self.codec, &source, consumed, context)?
                                                                                                     ^2
  821|       |                {
  822|       |                    DecodeInvalidAction::Reject => {
  823|      6|                        return Err(TranscodeDecodeError::domain_main_with_consumed(
  824|      6|                            source,
  825|      6|                            context.input_index(),
  826|      6|                            consumed,
  827|      6|                        ));
  828|       |                    }
  829|    518|                    DecodeInvalidAction::Skip { consumed } => AppliedDecodeInvalidAction::Skip { consumed },
  830|    516|                    DecodeInvalidAction::Emit { value, consumed } => {
  831|    516|                        AppliedDecodeInvalidAction::Emit { value, consumed }
  832|       |                    }
  833|       |                };
  834|  1.03k|                Ok(Self::apply_invalid_decode_action(action, context, consume))
  835|       |            }
  836|       |        }
  837|  4.49k|    }
  838|       |
  839|       |    /// Runs one decode step with an explicit end-of-input flag.
  840|       |    ///
  841|       |    /// The helper centralizes lifecycle validation and shared cursor handling
  842|       |    /// for the streaming and EOF-aware entry points.
  843|       |    ///
  844|       |    /// # Parameters
  845|       |    ///
  846|       |    /// - `input`: Complete source-unit slice.
  847|       |    /// - `input_index`: Absolute index of the next source unit.
  848|       |    /// - `output`: Destination for decoded values.
  849|       |    /// - `output_index`: Absolute index of the next destination slot.
  850|       |    /// - `end_of_input`: Whether to resolve incomplete tails through EOF
  851|       |    ///   policy.
  852|       |    ///
  853|       |    /// # Returns
  854|       |    ///
  855|       |    /// Returns consumed/written counts and the reason decoding stopped.
  856|       |    ///
  857|       |    /// # Errors
  858|       |    ///
  859|       |    /// Returns framework errors for invalid lifecycle state or slice indices,
  860|       |    /// and policy/domain errors for rejected input, including incomplete EOF
  861|       |    /// tails.
  862|       |    ///
  863|       |    /// # Panics
  864|       |    ///
  865|       |    /// Panics when codec or hook progress violates declared decode bounds.
  866|     76|    fn transcode_with_eof(
  867|     76|        &mut self,
  868|     76|        input: &[C::Unit],
  869|     76|        input_index: usize,
  870|     76|        output: &mut [C::Value],
  871|     76|        output_index: usize,
  872|     76|        end_of_input: bool,
  873|     76|    ) -> Result<TranscodeProgress, TranscodeDecodeErrorOf<C>> {
  874|     76|        self.lifecycle.on_transcode()?;
                                                   ^7
  875|     69|        TranscodeFailure::ensure_transcode_indices(input.len(), input_index, output.len(), output_index)?;
                                                                                                                      ^4
  876|       |
  877|     65|        let min_units = NonZeroUsize::new(C::MIN_UNITS_PER_VALUE).expect("Codec::MIN_UNITS_PER_VALUE is non-zero");
  878|     65|        let min_units_len = min_units.get();
  879|     65|        let mut state = DecodeState::new(input, input_index, output, output_index);
  880|    114|        while state.has_input() {
  881|     83|            let context = state.context();
  882|     83|            let available = context.available();
  883|     83|            if available < min_units_len {
  884|     18|                if !end_of_input {
  885|      4|                    return Ok(state.need_input_progress_with(min_units));
  886|     14|                }
  887|     14|                if state.needs_output() {
  888|      7|                    return Ok(state.need_output_progress());
  889|      7|                }
  890|      7|                let output_index = state.output_cursor();
  891|      7|                let output = state.output_mut();
  892|      7|                let (outcome, _) = self.decode_one(input, context, true, |value, _input_index| {
                                   ^4                                                                        ^1
  893|       |                    // SAFETY: The empty-output check above proved this slot writable.
  894|      1|                    unsafe {
  895|      1|                        *output.get_unchecked_mut(output_index) = value;
  896|      1|                    }
  897|      3|                })?;
                              ^1
  898|      4|                if let Some(progress) = state.apply_decode_outcome(outcome) {
                                          ^0
  899|      0|                    return Ok(progress);
  900|      4|                }
  901|      4|                continue;
  902|     65|            }
  903|     65|            if state.needs_output() {
  904|      5|                return Ok(state.need_output_progress());
  905|     60|            }
  906|     60|            let output_index = state.output_cursor();
  907|     60|            let output = state.output_mut();
  908|     60|            let (outcome, _) = self.decode_one(input, context, end_of_input, |value, _input_index| {
                               ^53                                                                               ^43
  909|       |                // SAFETY: `needs_output()` returned false, so the output
  910|       |                // cursor points at a writable initialized slot.
  911|     43|                unsafe {
  912|     43|                    *output.get_unchecked_mut(output_index) = value;
  913|     43|                }
  914|     43|            })?;
                            ^7
  915|     53|            if let Some(progress) = state.apply_decode_outcome(outcome) {
                                      ^8
  916|      8|                return Ok(progress);
  917|     45|            }
  918|       |        }
  919|       |
  920|     31|        Ok(state.complete_progress())
  921|     76|    }
  922|       |
  923|       |    /// Applies a hook-selected invalid-decode action after reject handling.
  924|       |    ///
  925|       |    /// # Type Parameters
  926|       |    ///
  927|       |    /// - `R`: Consumer result for an emitted value.
  928|       |    /// - `F`: Callback consuming an emitted value and its source index.
  929|       |    ///
  930|       |    /// # Parameters
  931|       |    ///
  932|       |    /// - `action`: Validated policy action to skip or emit.
  933|       |    /// - `context`: Source cursor and available input.
  934|       |    /// - `consume`: Callback invoked only for an emitted value.
  935|       |    ///
  936|       |    /// # Returns
  937|       |    ///
  938|       |    /// Returns progress with `Some` callback result for Emit, or `None` for
  939|       |    /// Skip.
  940|       |    ///
  941|       |    /// # Panics
  942|       |    ///
  943|       |    /// Panics if the action consumes more units than the context makes
  944|       |    /// available.
  945|  1.03k|    fn apply_invalid_decode_action<R, F>(
  946|  1.03k|        action: AppliedDecodeInvalidAction<C::Value>,
  947|  1.03k|        context: DecodeContext,
  948|  1.03k|        consume: F,
  949|  1.03k|    ) -> (DecodeOutcome, Option<R>)
  950|  1.03k|    where
  951|  1.03k|        F: FnOnce(C::Value, usize) -> R,
  952|       |    {
  953|  1.03k|        match action {
  954|    518|            AppliedDecodeInvalidAction::Skip { consumed } => {
  955|    518|                let read = DecodeInvalidAction::<C::Value>::bound_consumed(consumed, context.available());
  956|    518|                (DecodeOutcome::skipped(read), None)
  957|       |            }
  958|    516|            AppliedDecodeInvalidAction::Emit { value, consumed } => {
  959|    516|                let read = DecodeInvalidAction::<C::Value>::bound_consumed(consumed, context.available());
  960|    516|                let consumed_value = consume(value, context.input_index());
  961|    516|                (DecodeOutcome::emitted(read, NonZeroUsize::MIN), Some(consumed_value))
  962|       |            }
  963|       |        }
  964|  1.03k|    }
  965|       |
  966|       |    /// Applies the hook policy to an incomplete input tail.
  967|       |    ///
  968|       |    /// # Parameters
  969|       |    ///
  970|       |    /// - `source`: Optional codec error associated with the incomplete tail.
  971|       |    /// - `required_total`: Minimum total input width needed to decode a value.
  972|       |    /// - `context`: Decode cursors and the number of visible source units.
  973|       |    /// - `consume`: Callback used once when the policy emits a value.
  974|       |    ///
  975|       |    /// # Returns
  976|       |    ///
  977|       |    /// Returns the resulting decode outcome and an optional consumer result.
  978|       |    ///
  979|       |    /// # Errors
  980|       |    ///
  981|       |    /// Returns the hook error or the appropriate incomplete-input rejection,
  982|       |    /// preserving a codec error source when one was supplied.
  983|     49|    fn apply_incomplete_decode<R, F>(
  984|     49|        &mut self,
  985|     49|        source: Option<C::DecodeError>,
  986|     49|        required_total: NonZeroUsize,
  987|     49|        context: DecodeContext,
  988|     49|        consume: F,
  989|     49|    ) -> Result<(DecodeOutcome, Option<R>), TranscodeDecodeErrorOf<C>>
  990|     49|    where
  991|     49|        F: FnOnce(C::Value, usize) -> R,
  992|       |    {
  993|     49|        match self
  994|     49|            .hooks
  995|     49|            .handle_incomplete_decode(&mut self.codec, source.as_ref(), required_total, context)?
                                                                                                              ^0
  996|       |        {
  997|      8|            DecodeIncompleteAction::Reject => match source {
  998|      1|                Some(source) => Err(TranscodeDecodeError::domain_main(source, context.input_index())),
  999|      7|                None => Err(TranscodeFailure::incomplete_input(
 1000|      7|                    context.input_index(),
 1001|      7|                    required_total.get(),
 1002|      7|                    context.available(),
 1003|      7|                )
 1004|      7|                .into()),
 1005|       |            },
 1006|       |            DecodeIncompleteAction::Skip => {
 1007|     21|                let read =
 1008|     21|                    NonZeroUsize::new(context.available()).expect("incomplete decode tail must contain source units");
 1009|     21|                Ok((DecodeOutcome::skipped(read), None))
 1010|       |            }
 1011|     20|            DecodeIncompleteAction::Emit { value } => {
 1012|     20|                let read =
 1013|     20|                    NonZeroUsize::new(context.available()).expect("incomplete decode tail must contain source units");
 1014|     20|                let result = consume(value, context.input_index());
 1015|     20|                Ok((DecodeOutcome::emitted(read, NonZeroUsize::MIN), Some(result)))
 1016|       |            }
 1017|       |        }
 1018|     49|    }
 1019|       |}
 1020|       |
 1021|       |impl<C, H> Transcoder for TranscodeDecodeEngine<C, H>
 1022|       |where
 1023|       |    C: Codec,
 1024|       |    H: TranscodeDecodeHooks<C>,
 1025|       |{
 1026|       |    /// Source units accepted by the wrapped codec.
 1027|       |    type Input = C::Unit;
 1028|       |    /// Logical values decoded by the wrapped codec.
 1029|       |    type Output = C::Value;
 1030|       |    /// Framework failures and the wrapped codec's decode errors.
 1031|       |    type Error = TranscodeDecodeErrorOf<C>;
 1032|       |
 1033|       |    /// Returns an upper bound for decoded values produced from `input_len`
 1034|       |    /// units.
 1035|       |    ///
 1036|       |    /// # Parameters
 1037|       |    ///
 1038|       |    /// - `input_len`: Number of source units to decode.
 1039|       |    ///
 1040|       |    /// # Returns
 1041|       |    ///
 1042|       |    /// Returns the conservative streaming output bound.
 1043|       |    ///
 1044|       |    /// # Errors
 1045|       |    ///
 1046|       |    /// Returns a capacity error on hook arithmetic overflow.
 1047|       |    #[inline]
 1048|      3|    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
 1049|      3|        TranscodeDecodeEngine::max_transcode_output_len(self, input_len)
 1050|      3|    }
 1051|       |
 1052|       |    /// Returns an upper bound for values produced by finishing codec and hook
 1053|       |    /// state.
 1054|       |    ///
 1055|       |    /// # Returns
 1056|       |    ///
 1057|       |    /// Returns the combined codec and hook finish bound.
 1058|       |    ///
 1059|       |    /// # Errors
 1060|       |    ///
 1061|       |    /// Returns a capacity error if that sum overflows.
 1062|       |    #[inline]
 1063|      2|    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
 1064|      2|        TranscodeDecodeEngine::max_finish_output_len(self)
 1065|      2|    }
 1066|       |
 1067|       |    /// Returns an upper bound for values emitted when resetting stream state.
 1068|       |    ///
 1069|       |    /// # Returns
 1070|       |    ///
 1071|       |    /// Returns the codec reset bound as `Ok`; this query cannot fail.
 1072|       |    #[inline]
 1073|      2|    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
 1074|      2|        TranscodeDecodeEngine::max_reset_output_len(self)
 1075|      2|    }
 1076|       |
 1077|       |    /// Resets hooks and codec state, emitting any stream-start values.
 1078|       |    ///
 1079|       |    /// # Parameters
 1080|       |    ///
 1081|       |    /// - `output`: Destination value slice.
 1082|       |    /// - `output_index`: First writable value index.
 1083|       |    ///
 1084|       |    /// # Returns
 1085|       |    ///
 1086|       |    /// Returns the number of stream-start values written.
 1087|       |    ///
 1088|       |    /// # Errors
 1089|       |    ///
 1090|       |    /// Returns invalid-capacity/index errors or codec reset domain errors.
 1091|       |    ///
 1092|       |    /// # Panics
 1093|       |    ///
 1094|       |    /// Panics if codec reset output exceeds its declared bound.
 1095|      2|    fn reset(&mut self, output: &mut [C::Value], output_index: usize) -> Result<usize, TranscodeDecodeErrorOf<C>> {
 1096|      2|        TranscodeDecodeEngine::reset(self, output, output_index)
 1097|      2|    }
 1098|       |
 1099|       |    /// Decodes source units into logical values.
 1100|       |    ///
 1101|       |    /// # Parameters
 1102|       |    ///
 1103|       |    /// - `input`: Source unit slice.
 1104|       |    /// - `input_index`: First source index.
 1105|       |    /// - `output`: Destination value slice.
 1106|       |    /// - `output_index`: First destination index.
 1107|       |    ///
 1108|       |    /// # Returns
 1109|       |    ///
 1110|       |    /// Returns consumed/written counts and the stop status.
 1111|       |    ///
 1112|       |    /// # Errors
 1113|       |    ///
 1114|       |    /// Returns lifecycle/index errors or rejected-input domain errors.
 1115|       |    ///
 1116|       |    /// # Panics
 1117|       |    ///
 1118|       |    /// Panics if codec or hook progress violates decode bounds.
 1119|      1|    fn transcode(
 1120|      1|        &mut self,
 1121|      1|        input: &[C::Unit],
 1122|      1|        input_index: usize,
 1123|      1|        output: &mut [C::Value],
 1124|      1|        output_index: usize,
 1125|      1|    ) -> Result<TranscodeProgress, TranscodeDecodeErrorOf<C>> {
 1126|      1|        TranscodeDecodeEngine::transcode(self, input, input_index, output, output_index)
 1127|      1|    }
 1128|       |
 1129|       |    /// Decodes source units after end of input is known.
 1130|       |    ///
 1131|       |    /// # Parameters
 1132|       |    ///
 1133|       |    /// - `input`: Final source unit slice.
 1134|       |    /// - `input_index`: First source index.
 1135|       |    /// - `output`: Destination value slice.
 1136|       |    /// - `output_index`: First destination index.
 1137|       |    ///
 1138|       |    /// # Returns
 1139|       |    ///
 1140|       |    /// Returns consumed/written counts and the stop status.
 1141|       |    ///
 1142|       |    /// # Errors
 1143|       |    ///
 1144|       |    /// Returns lifecycle/index errors or rejected input and incomplete-tail
 1145|       |    /// errors.
 1146|       |    ///
 1147|       |    /// # Panics
 1148|       |    ///
 1149|       |    /// Panics if codec or hook progress violates decode bounds.
 1150|      1|    fn transcode_eof(
 1151|      1|        &mut self,
 1152|      1|        input: &[C::Unit],
 1153|      1|        input_index: usize,
 1154|      1|        output: &mut [C::Value],
 1155|      1|        output_index: usize,
 1156|      1|    ) -> Result<TranscodeProgress, TranscodeDecodeErrorOf<C>> {
 1157|      1|        TranscodeDecodeEngine::transcode_eof(self, input, input_index, output, output_index)
 1158|      1|    }
 1159|       |
 1160|       |    /// Finishes internally retained output after EOF.
 1161|       |    ///
 1162|       |    /// # Parameters
 1163|       |    ///
 1164|       |    /// - `output`: Destination value slice.
 1165|       |    /// - `output_index`: First writable value index.
 1166|       |    ///
 1167|       |    /// # Returns
 1168|       |    ///
 1169|       |    /// Returns the number of final values written.
 1170|       |    ///
 1171|       |    /// # Errors
 1172|       |    ///
 1173|       |    /// Returns lifecycle/capacity/index errors or codec/hook finish errors.
 1174|       |    ///
 1175|       |    /// # Panics
 1176|       |    ///
 1177|       |    /// Panics if finalization output exceeds its declared bounds.
 1178|      2|    fn finish(&mut self, output: &mut [C::Value], output_index: usize) -> Result<usize, TranscodeDecodeErrorOf<C>> {
 1179|      2|        TranscodeDecodeEngine::finish(self, output, output_index)
 1180|      2|    }
 1181|       |}
 1182|       |
 1183|       |impl<C, H> TranscodeDecoder for TranscodeDecodeEngine<C, H>
 1184|       |where
 1185|       |    C: Codec,
 1186|       |    H: TranscodeDecodeHooks<C>,
 1187|       |{
 1188|       |    /// Domain error reported by the wrapped codec while decoding.
 1189|       |    type DecodeError = C::DecodeError;
 1190|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/engine/transcode_decode_hooks.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Policy hooks used by buffered decoder engines.
    9|       |
   10|       |use core::num::NonZeroUsize;
   11|       |
   12|       |use super::decode_context::DecodeContext;
   13|       |use super::decode_incomplete_action::DecodeIncompleteAction;
   14|       |use super::decode_invalid_action::DecodeInvalidAction;
   15|       |use crate::CapacityError;
   16|       |use crate::Codec;
   17|       |use crate::TranscodeDecodeErrorOf;
   18|       |
   19|       |/// Policy hooks for [`crate::engine::TranscodeDecodeEngine`].
   20|       |///
   21|       |/// Hooks own policy state, such as malformed-input replacement behavior. The
   22|       |/// engine passes the codec into hook methods when policy code needs codec
   23|       |/// metadata.
   24|       |///
   25|       |/// Implement this trait when a buffered decoder needs policy decisions after
   26|       |/// the low-level codec reports an error. The engine handles input/output cursor
   27|       |/// bookkeeping, output-capacity checks, and successful one-value decodes; hooks
   28|       |/// decide whether invalid or EOF-incomplete input should be skipped, replaced,
   29|       |/// or returned as an error.
   30|       |///
   31|       |/// The hook receives a [`DecodeContext`] with absolute input/output cursors, so
   32|       |/// errors can include useful positions without duplicating engine arithmetic.
   33|       |/// Stateful hooks may also use [`finish_hooks`](Self::finish_hooks) to emit
   34|       |/// final values after the caller has supplied all input and completed EOF
   35|       |/// decoding.
   36|       |///
   37|       |/// # Examples
   38|       |///
   39|       |/// This hook replaces malformed units with `b'?'` and otherwise lets the engine
   40|       |/// keep decoding. At EOF, implementations may also override
   41|       |/// [`Self::handle_incomplete_decode`] to repair or discard a trailing partial
   42|       |/// value.
   43|       |///
   44|       |/// ```rust
   45|       |/// use core::convert::Infallible;
   46|       |/// use core::num::NonZeroUsize;
   47|       |///
   48|       |/// use qubit_codec::Codec;
   49|       |/// use qubit_codec::DecodeFailure;
   50|       |/// use qubit_codec::TranscodeDecodeError;
   51|       |/// use qubit_codec::engine::DecodeContext;
   52|       |/// use qubit_codec::engine::DecodeInvalidAction;
   53|       |/// use qubit_codec::engine::TranscodeDecodeHooks;
   54|       |///
   55|       |/// #[derive(Clone, Copy)]
   56|       |/// struct MyCodec;
   57|       |///
   58|       |/// #[derive(Clone, Copy, Debug, Eq, PartialEq)]
   59|       |/// enum MyDecodeError {
   60|       |///     Malformed { consumed: NonZeroUsize },
   61|       |/// }
   62|       |///
   63|       |/// impl Codec for MyCodec {
   64|       |///     type Value = u8;
   65|       |///     type Unit = u8;
   66|       |///     type DecodeError = MyDecodeError;
   67|       |///     type EncodeError = Infallible;
   68|       |///
   69|       |///     const MIN_UNITS_PER_VALUE: usize = 1;
   70|       |///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
   71|       |///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
   72|       |///
   73|       |///     unsafe fn decode(
   74|       |///         &mut self,
   75|       |///         input: &[u8],
   76|       |///         index: usize,
   77|       |///     ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
   78|       |///         match input[index] {
   79|       |///             0xff => Err(DecodeFailure::invalid(
   80|       |///                 MyDecodeError::Malformed {
   81|       |///                     consumed: NonZeroUsize::MIN,
   82|       |///                 },
   83|       |///                 NonZeroUsize::MIN,
   84|       |///             )),
   85|       |///             value => Ok((value, NonZeroUsize::MIN)),
   86|       |///         }
   87|       |///     }
   88|       |///
   89|       |///     unsafe fn encode(
   90|       |///         &mut self,
   91|       |///         value: &u8,
   92|       |///         output: &mut [u8],
   93|       |///         index: usize,
   94|       |///     ) -> Result<usize, Self::EncodeError> {
   95|       |///         output[index] = *value;
   96|       |///         Ok(1)
   97|       |///     }
   98|       |/// }
   99|       |///
  100|       |/// #[derive(Default)]
  101|       |/// struct ReplacementHooks;
  102|       |///
  103|       |/// impl TranscodeDecodeHooks<MyCodec> for ReplacementHooks {
  104|       |///     fn handle_invalid_decode(
  105|       |///         &mut self,
  106|       |///         _codec: &mut MyCodec,
  107|       |///         error: &MyDecodeError,
  108|       |///         consumed: Option<NonZeroUsize>,
  109|       |///         _context: DecodeContext,
  110|       |///     ) -> Result<DecodeInvalidAction<u8>, TranscodeDecodeError<MyDecodeError>> {
  111|       |///         match error {
  112|       |///             MyDecodeError::Malformed { .. } => {
  113|       |///                 Ok(DecodeInvalidAction::Emit {
  114|       |///                     value: b'?',
  115|       |///                     consumed: consumed.expect("codec reported malformed width"),
  116|       |///                 })
  117|       |///             }
  118|       |///         }
  119|       |///     }
  120|       |/// }
  121|       |///
  122|       |/// let mut hooks = ReplacementHooks::default();
  123|       |/// let mut codec = MyCodec;
  124|       |/// let context = DecodeContext::new(0, 0, 0, 0, 1);
  125|       |/// let error = MyDecodeError::Malformed { consumed: NonZeroUsize::MIN };
  126|       |/// let action = hooks.handle_invalid_decode(
  127|       |///     &mut codec, &error, Some(NonZeroUsize::MIN), context,
  128|       |/// ).expect("replacement policy accepts malformed input");
  129|       |/// assert!(matches!(action, DecodeInvalidAction::Emit { value: b'?', consumed }
  130|       |///     if consumed == NonZeroUsize::MIN));
  131|       |/// ```
  132|       |///
  133|       |/// # Type Parameters
  134|       |///
  135|       |/// - `C`: Low-level codec owned by the engine.
  136|       |pub trait TranscodeDecodeHooks<C>
  137|       |where
  138|       |    C: Codec,
  139|       |{
  140|       |    /// Returns an upper bound for decoded values produced from `input_len`
  141|       |    /// units.
  142|       |    ///
  143|       |    /// This bound covers the streaming decode phase, including conservative
  144|       |    /// capacity preflight in streaming and EOF operations, as required by
  145|       |    /// [`crate::Transcoder::max_transcode_output_len`]. It describes required
  146|       |    /// writable capacity, not only values actually emitted. It must not include
  147|       |    /// codec reset values, codec finish values, or hook finish output.
  148|       |    /// Streaming actions can emit at most one value for the current input
  149|       |    /// event or skip it; an emitted replacement consumes a non-zero number of
  150|       |    /// source units. The hook API has no separate operation for draining
  151|       |    /// hook-owned output without processing input. Use a custom
  152|       |    /// [`crate::Transcoder`] for streaming behavior beyond this action model,
  153|       |    /// such as one-to-many output or draining output without new input. The
  154|       |    /// bound may depend on immutable hook and codec configuration, but must
  155|       |    /// cover every reachable transient hook and codec state.
  156|       |    ///
  157|       |    /// The bound must cover the preflight required by
  158|       |    /// [`handle_invalid_decode`](Self::handle_invalid_decode) and
  159|       |    /// [`handle_incomplete_decode`](Self::handle_incomplete_decode), even when
  160|       |    /// the chosen action skips input and emits no value. Zero actual output
  161|       |    /// does not imply zero required writable capacity. See the
  162|       |    /// [trait example](#examples) for the hook action model.
  163|       |    ///
  164|       |    /// The default implementation returns `input_len`, which remains
  165|       |    /// conservative when a replacement policy consumes fewer units than a
  166|       |    /// valid codec value. Override it when codec or hook policy can produce a
  167|       |    /// larger streaming output bound than this default.
  168|       |    ///
  169|       |    /// # Parameters
  170|       |    ///
  171|       |    /// - `codec`: Low-level codec owned by the engine.
  172|       |    /// - `input_len`: Number of source units the caller plans to decode.
  173|       |    ///
  174|       |    /// # Returns
  175|       |    ///
  176|       |    /// Returns a conservative upper bound for streaming output.
  177|       |    ///
  178|       |    /// # Errors
  179|       |    ///
  180|       |    /// Returns [`CapacityError::OutputLengthOverflow`] when the bound cannot
  181|       |    /// be represented as `usize`.
  182|       |    #[inline]
  183|  1.07k|    fn max_transcode_output_len(&self, _codec: &C, input_len: usize) -> Result<usize, CapacityError> {
  184|  1.07k|        Ok(input_len)
  185|  1.07k|    }
  186|       |
  187|       |    /// Returns an upper bound for values emitted by finishing hook-owned state.
  188|       |    ///
  189|       |    /// `finish` never receives more input. Implementations must report the
  190|       |    /// maximum output that could be derived from hook-owned state in any
  191|       |    /// reachable transient stream state, not only what happens to remain when
  192|       |    /// this method is called.
  193|       |    ///
  194|       |    /// The default implementation returns `0`. Override it when
  195|       |    /// [`finish_hooks`](Self::finish_hooks) can write trailers, delayed
  196|       |    /// replacements, or any other hook-owned final output. Do not include
  197|       |    /// [`Codec::MAX_DECODE_FINISH_VALUES`]; the engine adds the codec finish
  198|       |    /// bound separately.
  199|       |    ///
  200|       |    /// # Parameters
  201|       |    ///
  202|       |    /// - `codec`: Low-level codec owned by the engine.
  203|       |    ///
  204|       |    /// # Returns
  205|       |    ///
  206|       |    /// Returns the finite, transient-state-independent final-output upper
  207|       |    /// bound.
  208|       |    #[must_use]
  209|       |    #[inline]
  210|  5.17k|    fn max_finish_output_len(&self, _codec: &C) -> usize {
  211|  5.17k|        0
  212|  5.17k|    }
  213|       |
  214|       |    /// Handles a codec-domain invalid decode error during `transcode`.
  215|       |    ///
  216|       |    /// # Parameters
  217|       |    ///
  218|       |    /// - `codec`: Low-level codec owned by the engine.
  219|       |    /// - `error`: Invalid domain error returned by the codec.
  220|       |    /// - `consumed`: Known invalid input units that may be consumed by
  221|       |    ///   non-strict policies, or `None` when the codec cannot state an invalid
  222|       |    ///   span length.
  223|       |    /// - `context`: Decode attempt context.
  224|       |    ///
  225|       |    /// # Returns
  226|       |    ///
  227|       |    /// Returns the action selected by this hook policy.
  228|       |    ///
  229|       |    /// Returned consuming actions must not consume more than
  230|       |    /// `context.available()` input units.
  231|       |    ///
  232|       |    /// The engine treats violations as hook bugs and panics.
  233|       |    ///
  234|       |    /// # Errors
  235|       |    ///
  236|       |    /// Returns [`crate::TranscodeDecodeError`] when the policy rejects the
  237|       |    /// input with a codec-domain error.
  238|       |    fn handle_invalid_decode(
  239|       |        &mut self,
  240|       |        codec: &mut C,
  241|       |        error: &C::DecodeError,
  242|       |        consumed: Option<NonZeroUsize>,
  243|       |        context: DecodeContext,
  244|       |    ) -> Result<DecodeInvalidAction<C::Value>, TranscodeDecodeErrorOf<C>>;
  245|       |
  246|       |    /// Handles a codec value left incomplete after end of input.
  247|       |    ///
  248|       |    /// The engine calls this method only from `transcode_eof`. `source` is
  249|       |    /// `Some` only when the codec returned an incomplete failure carrying a
  250|       |    /// source, such as through
  251|       |    /// [`crate::DecodeFailure::incomplete_with_source`]. It is `None` when
  252|       |    /// the available tail is shorter than [`Codec::MIN_UNITS_PER_VALUE`]
  253|       |    /// and the codec was not called, or when the codec was called but
  254|       |    /// returned [`crate::DecodeFailure::incomplete`] without a source.
  255|       |    ///
  256|       |    /// # Parameters
  257|       |    ///
  258|       |    /// - `codec`: Low-level codec owned by the engine.
  259|       |    /// - `source`: Optional domain error explaining the incomplete tail.
  260|       |    /// - `required_total`: Minimum source units required to retry decoding.
  261|       |    /// - `context`: Decode attempt context.
  262|       |    ///
  263|       |    /// # Returns
  264|       |    ///
  265|       |    /// Returns the EOF policy action. `Skip` and `Emit` consume every source
  266|       |    /// unit available in `context`. At confirmed EOF,
  267|       |    /// [`crate::engine::TranscodeDecodeEngine`] requires one writable value
  268|       |    /// slot before invoking this method, even when the eventual action is
  269|       |    /// `Skip` or `Reject`; a capacity shortage does not invoke the hook or
  270|       |    /// change its state. [`crate::engine::TranscodeConvertEngine`] may invoke
  271|       |    /// the hook without target capacity: `Skip` can consume the tail, while
  272|       |    /// `Emit` can retain a pending value for encoding. A retry drains that
  273|       |    /// pending value without invoking the hook again for the same tail.
  274|       |    ///
  275|       |    /// # Errors
  276|       |    ///
  277|       |    /// Returns [`crate::TranscodeDecodeError`] when policy evaluation fails.
  278|       |    #[inline]
  279|      4|    fn handle_incomplete_decode(
  280|      4|        &mut self,
  281|      4|        _codec: &mut C,
  282|      4|        _source: Option<&C::DecodeError>,
  283|      4|        _required_total: NonZeroUsize,
  284|      4|        _context: DecodeContext,
  285|      4|    ) -> Result<DecodeIncompleteAction<C::Value>, TranscodeDecodeErrorOf<C>> {
  286|      4|        Ok(DecodeIncompleteAction::Reject)
  287|      4|    }
  288|       |
  289|       |    /// Runs hook-owned cleanup as part of stream reset.
  290|       |    ///
  291|       |    /// Called before [`Codec::decode_reset`]
  292|       |    /// writes its own reset output. Stateless hooks may use the default no-op.
  293|       |    ///
  294|       |    /// # Parameters
  295|       |    ///
  296|       |    /// - `codec`: Low-level codec owned by the engine.
  297|       |    #[inline]
  298|  1.31k|    fn reset_hooks(&mut self, _codec: &mut C) {}
  299|       |
  300|       |    /// Finishes hook-owned state and writes any retained output.
  301|       |    ///
  302|       |    /// The default implementation is a no-op for stateless decode hooks.
  303|       |    /// Stateful hooks may emit final values such as checksums, reset markers,
  304|       |    /// or other trailer data. The caller must provide at least
  305|       |    /// [`TranscodeDecodeHooks::max_finish_output_len`] writable slots from
  306|       |    /// `output_index`. Implementations must not write beyond that declared
  307|       |    /// final-output bound.
  308|       |    ///
  309|       |    /// Called after [`Codec::decode_finish`] has
  310|       |    /// written its own finish output.
  311|       |    ///
  312|       |    /// # Parameters
  313|       |    ///
  314|       |    /// - `codec`: Low-level codec owned by the engine.
  315|       |    /// - `output`: Output value slice visible to the hook.
  316|       |    /// - `output_index`: Absolute output value index where writing starts.
  317|       |    ///
  318|       |    /// # Returns
  319|       |    ///
  320|       |    /// Returns the number of values written by finalization. This count must
  321|       |    /// not exceed [`TranscodeDecodeHooks::max_finish_output_len`].
  322|       |    ///
  323|       |    /// # Errors
  324|       |    ///
  325|       |    /// Returns a transcode decode error when hook-owned state cannot be
  326|       |    /// finalized.
  327|       |    #[inline]
  328|  1.16k|    fn finish_hooks(
  329|  1.16k|        &mut self,
  330|  1.16k|        _codec: &mut C,
  331|  1.16k|        _output: &mut [C::Value],
  332|  1.16k|        _output_index: usize,
  333|  1.16k|    ) -> Result<usize, TranscodeDecodeErrorOf<C>> {
  334|  1.16k|        Ok(0)
  335|  1.16k|    }
  336|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/engine/transcode_encode_engine.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Reusable buffered encoder engine.
    9|       |
   10|       |use core::num::NonZeroUsize;
   11|       |
   12|       |use super::super::internal::encode_attempt::EncodeAttempt;
   13|       |use super::super::internal::encode_state::EncodeState;
   14|       |use super::super::internal::lifecycle_guard::LifecycleGuard;
   15|       |use super::EncodeOutcome;
   16|       |use super::EncodeUnencodableAction;
   17|       |use super::TranscodeEncodeHooks;
   18|       |use crate::CapacityError;
   19|       |use crate::Codec;
   20|       |use crate::TranscodeEncodeError;
   21|       |use crate::TranscodeEncodeErrorOf;
   22|       |use crate::TranscodeEncoder;
   23|       |use crate::TranscodeFailure;
   24|       |use crate::TranscodeProgress;
   25|       |use crate::Transcoder;
   26|       |use crate::codec::assert_unit_bounds;
   27|       |
   28|       |/// Reusable buffered encoding engine for codec-backed encoders.
   29|       |///
   30|       |/// The engine owns the low-level codec and hook object. It keeps the common
   31|       |/// buffered encoding loop private: input-index validation, output-capacity
   32|       |/// checks, input consumption, output progress, and [`crate::TranscodeStatus`]
   33|       |/// reporting.
   34|       |///
   35|       |/// Use this type to build a streaming encoder over a one-value [`Codec`]. The
   36|       |/// engine does not allocate output. It encodes values accepted by
   37|       |/// [`Codec::can_encode_value`] directly through the codec. When a value is
   38|       |/// outside the codec's encodable domain, the engine asks hooks whether to
   39|       |/// reject, skip, or replace it. If the current output buffer is too small for
   40|       |/// the selected value, the engine returns
   41|       |/// [`crate::TranscodeStatus::NeedOutput`] without consuming that input; the
   42|       |/// caller can provide a larger or fresh output buffer and resume with the
   43|       |/// returned input index.
   44|       |///
   45|       |/// For the common strict policy that simply wraps codec errors, use
   46|       |/// [`crate::CodecTranscodeEncoder`]. Use `TranscodeEncodeEngine` directly when
   47|       |/// the encode policy needs custom planning, replacement, skipped values, or
   48|       |/// finish-time output.
   49|       |///
   50|       |/// # Examples
   51|       |///
   52|       |/// ```rust
   53|       |/// use core::convert::Infallible;
   54|       |/// use core::num::NonZeroUsize;
   55|       |///
   56|       |/// use qubit_codec::Codec;
   57|       |/// use qubit_codec::DecodeFailure;
   58|       |/// use qubit_codec::TranscodeEncodeErrorOf;
   59|       |/// use qubit_codec::TranscodeStatus;
   60|       |/// use qubit_codec::engine::EncodeContext;
   61|       |/// use qubit_codec::engine::EncodeUnencodableAction;
   62|       |/// use qubit_codec::engine::TranscodeEncodeEngine;
   63|       |/// use qubit_codec::engine::TranscodeEncodeHooks;
   64|       |///
   65|       |/// #[derive(Clone, Copy)]
   66|       |/// struct ByteCodec;
   67|       |///
   68|       |/// impl Codec for ByteCodec {
   69|       |///     type Value = u8;
   70|       |///     type Unit = u8;
   71|       |///     type DecodeError = Infallible;
   72|       |///     type EncodeError = Infallible;
   73|       |///
   74|       |///     const MIN_UNITS_PER_VALUE: usize = 1;
   75|       |///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
   76|       |///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
   77|       |///
   78|       |///     unsafe fn decode(
   79|       |///         &mut self,
   80|       |///         input: &[u8],
   81|       |///         index: usize,
   82|       |///     ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
   83|       |///         Ok((input[index], NonZeroUsize::MIN))
   84|       |///     }
   85|       |///
   86|       |///     unsafe fn encode(
   87|       |///         &mut self,
   88|       |///         value: &u8,
   89|       |///         output: &mut [u8],
   90|       |///         index: usize,
   91|       |///     ) -> Result<usize, Self::EncodeError> {
   92|       |///         output[index] = *value;
   93|       |///         Ok(1)
   94|       |///     }
   95|       |/// }
   96|       |///
   97|       |/// struct StrictHooks;
   98|       |///
   99|       |/// impl TranscodeEncodeHooks<ByteCodec> for StrictHooks {
  100|       |///     fn handle_unencodable_encode(
  101|       |///         &mut self,
  102|       |///         _codec: &mut ByteCodec,
  103|       |///         _context: &EncodeContext<'_, u8>,
  104|       |///     ) -> Result<EncodeUnencodableAction<u8>, TranscodeEncodeErrorOf<ByteCodec>> {
  105|       |///         unreachable!("ByteCodec accepts every u8")
  106|       |///     }
  107|       |/// }
  108|       |///
  109|       |/// let mut engine = TranscodeEncodeEngine::new(ByteCodec, StrictHooks);
  110|       |/// let input = [1_u8, 2, 3];
  111|       |/// let mut output = [0_u8; 2];
  112|       |/// let mut reset_output = [];
  113|       |/// engine.reset(&mut reset_output, 0)?;
  114|       |///
  115|       |/// let progress = engine.transcode(&input, 0, &mut output, 0)?;
  116|       |/// match progress.status() {
  117|       |///     TranscodeStatus::Complete => unreachable!("output is intentionally short"),
  118|       |///     TranscodeStatus::NeedOutput { .. } => {
  119|       |///         assert_eq!(2, progress.written());
  120|       |///         assert_eq!([1, 2], output);
  121|       |///         // Write out `output[..progress.written()]`, then resume at
  122|       |///         // `progress.read()` with fresh output capacity.
  123|       |///     }
  124|       |///     TranscodeStatus::NeedInput { .. } => unreachable!("encoders do not read encoded input"),
  125|       |/// }
  126|       |/// # Ok::<(), TranscodeEncodeErrorOf<ByteCodec>>(())
  127|       |/// ```
  128|       |///
  129|       |/// # Type Parameters
  130|       |///
  131|       |/// - `C`: Low-level codec used by the engine.
  132|       |/// - `H`: Policy hook object used by the engine.
  133|       |#[derive(Debug)]
  134|       |pub struct TranscodeEncodeEngine<C, H> {
  135|       |    /// Owned codec carrying the encode-side stream state.
  136|       |    codec: C,
  137|       |    /// Owned policy hooks for unsupported values and lifecycle output.
  138|       |    hooks: H,
  139|       |    /// Guard for the `reset → transcode* → finish` lifecycle in every build
  140|       |    /// profile.
  141|       |    lifecycle: LifecycleGuard,
  142|       |}
  143|       |
  144|       |impl<C, H> TranscodeEncodeEngine<C, H>
  145|       |where
  146|       |    C: Codec,
  147|       |    H: TranscodeEncodeHooks<C>,
  148|       |{
  149|       |    /// Creates a buffered encoder engine.
  150|       |    ///
  151|       |    /// # Parameters
  152|       |    ///
  153|       |    /// - `codec`: Low-level codec used for one-value encoding.
  154|       |    /// - `hooks`: Policy hooks used for planning and writing values.
  155|       |    ///
  156|       |    /// # Returns
  157|       |    ///
  158|       |    /// Returns a buffered encoder engine.
  159|       |    ///
  160|       |    /// # Compile-Time Checks
  161|       |    ///
  162|       |    /// Fails to compile when the supplied codec declares a zero decode unit
  163|       |    /// bound or when [`Codec::MIN_UNITS_PER_VALUE`] exceeds
  164|       |    /// [`Codec::MAX_DECODE_UNITS_PER_VALUE`]. The encode bound may be zero for
  165|       |    /// a fully buffered codec.
  166|       |    #[inline]
  167|       |    #[must_use]
  168|  1.28k|    pub fn new(codec: C, hooks: H) -> Self {
  169|  1.28k|        assert_unit_bounds::<C>();
  170|  1.28k|        Self {
  171|  1.28k|            codec,
  172|  1.28k|            hooks,
  173|  1.28k|            lifecycle: LifecycleGuard::new(),
  174|  1.28k|        }
  175|  1.28k|    }
  176|       |
  177|       |    /// Returns the wrapped low-level codec.
  178|       |    ///
  179|       |    /// # Returns
  180|       |    ///
  181|       |    /// Returns a shared reference to the codec owned by this engine.
  182|       |    #[inline]
  183|       |    #[must_use = "use the codec reference to inspect the wrapped codec"]
  184|      4|    pub const fn codec(&self) -> &C {
  185|      4|        &self.codec
  186|      4|    }
  187|       |
  188|       |    /// Returns the wrapped low-level codec mutably.
  189|       |    ///
  190|       |    /// # Returns
  191|       |    ///
  192|       |    /// Returns a mutable reference to the codec owned by this engine.
  193|       |    #[inline]
  194|       |    #[must_use = "use the codec reference to configure the wrapped codec"]
  195|      4|    pub fn codec_mut(&mut self) -> &mut C {
  196|      4|        &mut self.codec
  197|      4|    }
  198|       |
  199|       |    /// Returns the encode hooks used by this engine.
  200|       |    ///
  201|       |    /// # Returns
  202|       |    ///
  203|       |    /// Returns a shared reference to the hook object owned by this engine.
  204|       |    #[inline]
  205|       |    #[must_use = "use the hook reference to inspect the encode policy"]
  206|     15|    pub const fn hooks(&self) -> &H {
  207|     15|        &self.hooks
  208|     15|    }
  209|       |
  210|       |    /// Returns the encode hooks mutably.
  211|       |    ///
  212|       |    /// # Returns
  213|       |    ///
  214|       |    /// Returns a mutable reference to the hook object owned by this engine.
  215|       |    #[inline]
  216|       |    #[must_use = "use the hook reference to configure the encode policy"]
  217|      2|    pub fn hooks_mut(&mut self) -> &mut H {
  218|      2|        &mut self.hooks
  219|      2|    }
  220|       |
  221|       |    /// Consumes the engine and returns its codec and hooks.
  222|       |    ///
  223|       |    /// Any lifecycle state owned by the engine is discarded.
  224|       |    ///
  225|       |    /// # Returns
  226|       |    ///
  227|       |    /// Returns the wrapped codec followed by the encode hooks.
  228|       |    #[inline]
  229|       |    #[must_use]
  230|      4|    pub fn into_parts(self) -> (C, H) {
  231|      4|        let Self { codec, hooks, .. } = self;
  232|      4|        (codec, hooks)
  233|      4|    }
  234|       |
  235|       |    /// Gets a conservative upper bound for output units needed for
  236|       |    /// `input_len` values.
  237|       |    ///
  238|       |    /// This bound covers only the streaming encode phase. It is delegated to
  239|       |    /// [`TranscodeEncodeHooks::max_transcode_output_len`], so it includes hook
  240|       |    /// policy and is valid for every reachable transient codec and hook state.
  241|       |    /// Downstream encoders must use this engine-level API for capacity planning
  242|       |    /// instead of recomputing the bound from [`Codec`] constants.
  243|       |    ///
  244|       |    /// # Parameters
  245|       |    ///
  246|       |    /// - `input_len`: Number of input values the caller plans to encode.
  247|       |    ///
  248|       |    /// # Returns
  249|       |    ///
  250|       |    /// Returns a conservative upper bound for output units.
  251|       |    ///
  252|       |    /// # Errors
  253|       |    ///
  254|       |    /// Returns a capacity error when hook planning overflows.
  255|       |    #[inline]
  256|  9.67k|    pub fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
  257|  9.67k|        self.hooks.max_transcode_output_len(&self.codec, input_len)
  258|  9.67k|    }
  259|       |
  260|       |    /// Gets the global maximum output units emitted by stream reset.
  261|       |    ///
  262|       |    /// # Returns
  263|       |    ///
  264|       |    /// Returns the codec's reset-output upper bound. This query never fails.
  265|       |    #[inline]
  266|  1.29k|    pub fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
  267|  1.29k|        Ok(C::MAX_ENCODE_RESET_UNITS)
  268|  1.29k|    }
  269|       |
  270|       |    /// Gets the global maximum output units emitted by finishing codec and hook
  271|       |    /// state.
  272|       |    ///
  273|       |    /// Returns the sum of [`Codec::MAX_ENCODE_FINISH_UNITS`] and the
  274|       |    /// hook-provided final-output bound. The codec finish portion covers units
  275|       |    /// written by [`Codec::encode_finish`]; hook implementations must not
  276|       |    /// include that portion in
  277|       |    /// [`TranscodeEncodeHooks::max_finish_output_len`]. Both component bounds
  278|       |    /// cover every reachable transient state.
  279|       |    ///
  280|       |    /// # Returns
  281|       |    ///
  282|       |    /// Returns the combined codec-finish and hook-finish output bound.
  283|       |    ///
  284|       |    /// # Errors
  285|       |    ///
  286|       |    /// Returns [`CapacityError::OutputLengthOverflow`] when the sum overflows.
  287|       |    #[inline]
  288|  2.87k|    pub fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
  289|       |        C::MAX_ENCODE_FINISH_UNITS
  290|  2.87k|            .checked_add(self.hooks.max_finish_output_len(&self.codec))
  291|  2.87k|            .ok_or(CapacityError::OutputLengthOverflow)
  292|  2.87k|    }
  293|       |
  294|       |    /// Gets the maximum output units needed by a complete one-shot encode
  295|       |    /// stream.
  296|       |    ///
  297|       |    /// The returned bound covers reset output, the streaming encode phase for
  298|       |    /// `input_len` values, and finish output. Its components are global across
  299|       |    /// transient state. Higher-level complete encode helpers should use this
  300|       |    /// engine-level bound instead of recomputing capacity from [`Codec`]
  301|       |    /// constants, because hook policy may change streaming or finish output.
  302|       |    ///
  303|       |    /// # Parameters
  304|       |    ///
  305|       |    /// - `input_len`: Number of input values in the complete stream.
  306|       |    ///
  307|       |    /// # Returns
  308|       |    ///
  309|       |    /// Returns the complete-stream output bound.
  310|       |    ///
  311|       |    /// # Errors
  312|       |    ///
  313|       |    /// Returns a capacity error if a component query or their sum overflows.
  314|       |    #[inline]
  315|      4|    pub fn max_total_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
  316|      4|        let reset = self.max_reset_output_len()?;
                                                             ^0
  317|      4|        let transcode = self.max_transcode_output_len(input_len)?;
                          ^3                                                  ^1
  318|      3|        let finish = self.max_finish_output_len()?;
                                                               ^0
  319|      3|        reset
  320|      3|            .checked_add(transcode)
  321|      3|            .and_then(|len| len.checked_add(finish))
                                          ^2  ^2          ^2
  322|      3|            .ok_or(CapacityError::OutputLengthOverflow)
  323|      4|    }
  324|       |
  325|       |    /// Resets codec encode state, hook-owned state, and stream-start output.
  326|       |    ///
  327|       |    /// # Parameters
  328|       |    ///
  329|       |    /// - `output`: Complete output unit slice visible to the encoder.
  330|       |    /// - `output_index`: Absolute output unit index where writing starts.
  331|       |    ///
  332|       |    /// # Returns
  333|       |    ///
  334|       |    /// Returns the number of reset units written.
  335|       |    ///
  336|       |    /// # Errors
  337|       |    ///
  338|       |    /// Returns framework errors when the caller provides invalid or
  339|       |    /// insufficient output capacity. Returns domain errors when codec reset or
  340|       |    /// hook reset handling fails. Capacity and index failures occur before any
  341|       |    /// reset state is changed. Once reset execution starts, an error poisons
  342|       |    /// the engine until a later reset succeeds.
  343|       |    ///
  344|       |    /// # Panics
  345|       |    ///
  346|       |    /// Panics if the codec reports reset output beyond its declared bound.
  347|  1.28k|    pub fn reset(&mut self, output: &mut [C::Unit], output_index: usize) -> Result<usize, TranscodeEncodeErrorOf<C>> {
  348|  1.28k|        let required = self.max_reset_output_len()?;
                                                                ^0
  349|  1.28k|        TranscodeFailure::ensure_output_capacity(output.len(), output_index, required)?;
                                                                                                    ^3
  350|  1.28k|        self.lifecycle.on_reset_start();
  351|  1.28k|        self.hooks.reset_hooks(&mut self.codec);
  352|  1.28k|        let written = unsafe {
  353|       |            // SAFETY: The capacity check above reserves the codec's declared
  354|       |            // reset-output bound at `output_index`.
  355|  1.28k|            self.codec.encode_reset(output, output_index)
  356|       |        }
  357|  1.28k|        .map_err(TranscodeEncodeError::domain_reset)?;
                                                                  ^4
  358|  1.28k|        assert!(written <= required, "Codec::encode_reset wrote beyond its reset bound",);
  359|  1.28k|        self.lifecycle.on_reset_success();
  360|  1.28k|        Ok(written)
  361|  1.28k|    }
  362|       |
  363|       |    /// Encodes values into a caller-provided output buffer.
  364|       |    ///
  365|       |    /// The engine stops before consuming the next input value when the current
  366|       |    /// output buffer does not satisfy that value's planned capacity bound.
  367|       |    ///
  368|       |    /// # Parameters
  369|       |    ///
  370|       |    /// - `input`: Complete input value slice visible to the encoder.
  371|       |    /// - `input_index`: Absolute input value index where encoding starts.
  372|       |    /// - `output`: Complete output unit slice visible to the encoder.
  373|       |    /// - `output_index`: Absolute output unit index where writing starts.
  374|       |    ///
  375|       |    /// # Returns
  376|       |    ///
  377|       |    /// Returns progress describing input values consumed, output units written,
  378|       |    /// and why encoding stopped.
  379|       |    ///
  380|       |    /// # Errors
  381|       |    ///
  382|       |    /// Returns framework errors when `input_index` is outside `input` or
  383|       |    /// `output_index` is outside `output`, and domain errors when codec or
  384|       |    /// hook processing fails or rejects a value. Returns
  385|       |    /// [`TranscodeFailure::TranscodeBeforeReset`] when the engine has not
  386|       |    /// completed its first reset. Returns
  387|       |    /// [`TranscodeFailure::TranscodeAfterFinish`] when the logical stream was
  388|       |    /// already finished and has not been reset, or
  389|       |    /// [`TranscodeFailure::LifecyclePoisoned`] when an earlier reset or finish
  390|       |    /// failed after execution started.
  391|       |    ///
  392|       |    /// # Panics
  393|       |    ///
  394|       |    /// Panics if the codec violates its declared encode length or hooks
  395|       |    /// return an unencodable replacement.
  396|     49|    pub fn transcode(
  397|     49|        &mut self,
  398|     49|        input: &[C::Value],
  399|     49|        input_index: usize,
  400|     49|        output: &mut [C::Unit],
  401|     49|        output_index: usize,
  402|     49|    ) -> Result<TranscodeProgress, TranscodeEncodeErrorOf<C>> {
  403|     49|        self.lifecycle.on_transcode()?;
                                                   ^3
  404|     46|        TranscodeFailure::ensure_transcode_indices(input.len(), input_index, output.len(), output_index)?;
                                                                                                                      ^4
  405|     42|        let mut state = EncodeState::new(input, input_index, output, output_index);
  406|       |
  407|     70|        while state.has_input() {
  408|       |            // SAFETY: The loop condition proves that the current input cursor
  409|       |            // points at an available value.
  410|     54|            let attempt = unsafe { state.context_unchecked() };
  411|     54|            let outcome = self.encode_one(attempt)?;
                              ^49                               ^5
  412|     49|            if let Some(progress) = state.apply_encode_outcome(outcome) {
                                      ^21
  413|     21|                return Ok(progress);
  414|     28|            }
  415|       |        }
  416|       |
  417|     16|        Ok(state.complete_progress())
  418|     49|    }
  419|       |
  420|       |    /// Finishes codec and hook-owned output after EOF.
  421|       |    ///
  422|       |    /// Finalization first finishes encode-side codec state through
  423|       |    /// [`Codec::encode_finish`], then lets hook implementations finish their
  424|       |    /// own retained state. The caller must provide enough output capacity for
  425|       |    /// [`TranscodeEncodeEngine::max_finish_output_len`], which includes both
  426|       |    /// the codec finish bound and the hook-owned finish bound.
  427|       |    ///
  428|       |    /// # Parameters
  429|       |    ///
  430|       |    /// - `output`: Complete output unit slice visible to the encoder.
  431|       |    /// - `output_index`: Absolute output unit index where writing starts.
  432|       |    ///
  433|       |    /// # Returns
  434|       |    ///
  435|       |    /// Returns the number of units written by finalization.
  436|       |    ///
  437|       |    /// # Errors
  438|       |    ///
  439|       |    /// Returns framework errors when the caller provides invalid or
  440|       |    /// insufficient output capacity. Returns domain errors when codec finish or
  441|       |    /// hook finalization fails. Returns
  442|       |    /// [`TranscodeFailure::FinishBeforeReset`] when the engine has not
  443|       |    /// completed its first reset. Returns
  444|       |    /// [`TranscodeFailure::FinishAfterFinish`] when the logical stream was
  445|       |    /// already finished and has not been reset, or
  446|       |    /// [`TranscodeFailure::LifecyclePoisoned`] when an earlier reset or finish
  447|       |    /// failed after execution started. Capacity and index failures occur before
  448|       |    /// finish execution and remain retryable; later failures poison the engine
  449|       |    /// until reset succeeds.
  450|       |    ///
  451|       |    /// # Panics
  452|       |    ///
  453|       |    /// Panics when the codec finish or finish hooks write beyond their
  454|       |    /// independently declared output bounds.
  455|  1.17k|    pub fn finish(&mut self, output: &mut [C::Unit], output_index: usize) -> Result<usize, TranscodeEncodeErrorOf<C>> {
  456|  1.17k|        self.lifecycle.on_finish_attempt()?;
                                                        ^2
  457|  1.17k|        let hook_required = self.hooks.max_finish_output_len(&self.codec);
  458|  1.17k|        let required = C::MAX_ENCODE_FINISH_UNITS
  459|  1.17k|            .checked_add(hook_required)
  460|  1.17k|            .ok_or(CapacityError::OutputLengthOverflow)?;
                                                                     ^0
  461|  1.17k|        TranscodeFailure::ensure_output_capacity(output.len(), output_index, required)?;
                                                                                                    ^4
  462|  1.16k|        self.lifecycle.on_finish_start();
  463|  1.16k|        let finished = unsafe {
  464|       |            // SAFETY: The capacity check above reserves the codec's declared
  465|       |            // finish-output bound at `output_index`.
  466|  1.16k|            self.codec.encode_finish(output, output_index)
  467|       |        }
  468|  1.16k|        .map_err(TranscodeEncodeError::domain_finish)?;
                                                                   ^1
  469|  1.16k|        assert!(
  470|  1.16k|            finished <= C::MAX_ENCODE_FINISH_UNITS,
  471|       |            "Codec::encode_finish wrote beyond its finish bound",
  472|       |        );
  473|  1.16k|        let written = self
                          ^1.16k
  474|  1.16k|            .hooks
  475|  1.16k|            .finish_hooks(&mut self.codec, output, output_index + finished)?;
                                                                                         ^3
  476|  1.16k|        assert!(
  477|  1.16k|            written <= hook_required,
  478|       |            "TranscodeEncodeEngine hook wrote beyond its finish bound",
  479|       |        );
  480|  1.16k|        self.lifecycle.on_finish_success();
  481|  1.16k|        Ok(finished + written)
  482|  1.17k|    }
  483|       |
  484|       |    /// Runs a complete one-shot `reset -> transcode -> finish` encode stream.
  485|       |    ///
  486|       |    /// The complete input is supplied as `input`, and output starts at index
  487|       |    /// `0` in `output`. Callers that need subranges should slice their
  488|       |    /// buffers before calling this method. Downstream one-shot encoder
  489|       |    /// helpers should call this engine method instead of reproducing the
  490|       |    /// reset, transcode, and finish sequence themselves.
  491|       |    ///
  492|       |    /// # Parameters
  493|       |    ///
  494|       |    /// - `input`: Complete input values.
  495|       |    /// - `output`: Output unit slice for the whole encoded stream.
  496|       |    ///
  497|       |    /// # Returns
  498|       |    ///
  499|       |    /// Returns the number of output units written.
  500|       |    ///
  501|       |    /// # Errors
  502|       |    ///
  503|       |    /// Returns framework errors for insufficient output or capacity overflow,
  504|       |    /// and domain errors from reset, encode, or finish.
  505|       |    ///
  506|       |    /// # Panics
  507|       |    ///
  508|       |    /// Panics if codec or hook implementations violate the output or
  509|       |    /// replacement contracts of [`Self::reset`], [`Self::transcode`], or
  510|       |    /// [`Self::finish`].
  511|       |    #[inline]
  512|      1|    pub fn transcode_complete_into(
  513|      1|        &mut self,
  514|      1|        input: &[C::Value],
  515|      1|        output: &mut [C::Unit],
  516|      1|    ) -> Result<usize, TranscodeEncodeErrorOf<C>> {
  517|      1|        <Self as Transcoder>::transcode_complete_into(self, input, output)
  518|      1|    }
  519|       |
  520|       |    /// Encodes one value through the codec and unencodable-value hooks.
  521|       |    ///
  522|       |    /// This is the single entry point for `TranscodeConvertEngine` to drive
  523|       |    /// the encode side without accessing `codec` and `hooks` directly.
  524|       |    ///
  525|       |    /// # Parameters
  526|       |    ///
  527|       |    /// - `attempt`: Current value, absolute indices, and writable output
  528|       |    ///   buffer.
  529|       |    ///
  530|       |    /// # Returns
  531|       |    ///
  532|       |    /// Returns whether the value was consumed or needs more output capacity.
  533|       |    ///
  534|       |    /// For an unencodable value, this checks capacity for the codec's maximum
  535|       |    /// value width before invoking policy hooks. A shortage returns
  536|       |    /// `NeedOutput` without changing hook state; this conservative requirement
  537|       |    /// also applies when the hook would skip or reject. Errors after a hook
  538|       |    /// runs do not roll back mutations made by that hook.
  539|       |    ///
  540|       |    /// # Errors
  541|       |    ///
  542|       |    /// Returns an engine-domain error when the codec fails or when the hook
  543|       |    /// rejects an unencodable value.
  544|       |    ///
  545|       |    /// # Panics
  546|       |    ///
  547|       |    /// Panics if the codec violates its encode length contract or hooks
  548|       |    /// supply an unencodable replacement.
  549|  4.90k|    pub(in crate::transcode) fn encode_one(
  550|  4.90k|        &mut self,
  551|  4.90k|        attempt: EncodeAttempt<'_, C::Value, C::Unit>,
  552|  4.90k|    ) -> Result<EncodeOutcome, TranscodeEncodeErrorOf<C>> {
  553|  4.90k|        let encodable = {
  554|  4.90k|            let context = attempt.context();
  555|  4.90k|            self.codec.can_encode_value(context.input_value())
  556|       |        };
  557|  4.90k|        if encodable {
  558|  3.64k|            return self.encode_encodable_value(attempt);
  559|  1.26k|        }
  560|  1.26k|        let required = C::MAX_ENCODE_UNITS_PER_VALUE;
  561|  1.26k|        if attempt.available_output() < required {
  562|    220|            return Ok(EncodeOutcome::need_output(
  563|    220|                NonZeroUsize::new(required).expect("required output is non-zero when capacity is insufficient"),
  564|    220|            ));
  565|  1.04k|        }
  566|  1.04k|        let action = {
  567|  1.04k|            let context = attempt.context();
  568|  1.04k|            self.hooks.handle_unencodable_encode(&mut self.codec, &context)?
                                                                                         ^1
  569|       |        };
  570|  1.04k|        self.apply_unencodable_action(action, attempt)
  571|  4.90k|    }
  572|       |
  573|       |    /// Encodes an encodable input value.
  574|       |    ///
  575|       |    /// # Parameters
  576|       |    ///
  577|       |    /// - `attempt`: Current value, absolute indices, and writable output
  578|       |    ///   buffer.
  579|       |    ///
  580|       |    /// # Returns
  581|       |    ///
  582|       |    /// Returns the encode outcome for the current value.
  583|       |    ///
  584|       |    /// # Errors
  585|       |    ///
  586|       |    /// Returns a codec error when encoding fails.
  587|       |    ///
  588|       |    /// # Panics
  589|       |    ///
  590|       |    /// Panics when [`Codec::encode_len`] exceeds
  591|       |    /// [`Codec::MAX_ENCODE_UNITS_PER_VALUE`] or [`Codec::encode`] writes a
  592|       |    /// different number of units than planned.
  593|  3.64k|    fn encode_encodable_value(
  594|  3.64k|        &mut self,
  595|  3.64k|        attempt: EncodeAttempt<'_, C::Value, C::Unit>,
  596|  3.64k|    ) -> Result<EncodeOutcome, TranscodeEncodeErrorOf<C>> {
  597|  3.64k|        let required = self.codec.encode_len(attempt.value());
  598|  3.64k|        assert!(
  599|  3.64k|            required <= C::MAX_ENCODE_UNITS_PER_VALUE,
  600|       |            "Codec::encode_len exceeded Codec::MAX_ENCODE_UNITS_PER_VALUE",
  601|       |        );
  602|  3.64k|        if attempt.available_output() < required {
  603|    758|            return Ok(EncodeOutcome::need_output(
  604|    758|                NonZeroUsize::new(required).expect("required output is non-zero when capacity is insufficient"),
  605|    758|            ));
  606|  2.88k|        }
  607|  2.88k|        let (value, input_index, output, output_index) = attempt.into_parts();
  608|  2.88k|        let written = unsafe {
  609|       |            // SAFETY: The capacity check above reserves the exact value width.
  610|  2.88k|            self.codec.encode(value, output, output_index)
  611|       |        }
  612|  2.88k|        .map_err(|error| TranscodeEncodeError::domain_main(error, input_index))?;
                                       ^5                                ^5     ^5           ^5
  613|  2.88k|        assert!(
  614|  2.88k|            written == required,
  615|       |            "Codec::encode wrote a different length than Codec::encode_len",
  616|       |        );
  617|  2.88k|        Ok(EncodeOutcome::consumed(written))
  618|  3.64k|    }
  619|       |
  620|       |    /// Applies the hook-selected unencodable-value action.
  621|       |    ///
  622|       |    /// # Parameters
  623|       |    ///
  624|       |    /// - `action`: Policy action selected by the encode hooks.
  625|       |    /// - `attempt`: Rejected value, absolute indices, and writable output
  626|       |    ///   buffer.
  627|       |    ///
  628|       |    /// # Returns
  629|       |    ///
  630|       |    /// Returns the encode outcome for the current input value.
  631|       |    ///
  632|       |    /// # Errors
  633|       |    ///
  634|       |    /// Returns an unencodable-value error for rejection, or a codec-domain
  635|       |    /// error when replacement encoding fails.
  636|       |    ///
  637|       |    /// # Panics
  638|       |    ///
  639|       |    /// Panics if replacement encoding violates the codec length contract or
  640|       |    /// the replacement cannot be encoded.
  641|       |    #[inline]
  642|  1.04k|    fn apply_unencodable_action(
  643|  1.04k|        &mut self,
  644|  1.04k|        action: EncodeUnencodableAction<C::Value>,
  645|  1.04k|        attempt: EncodeAttempt<'_, C::Value, C::Unit>,
  646|  1.04k|    ) -> Result<EncodeOutcome, TranscodeEncodeErrorOf<C>> {
  647|  1.04k|        match action {
  648|       |            EncodeUnencodableAction::Reject => {
  649|       |                // Keep the streaming encoder usable for non-Clone values.
  650|       |                // The failure still carries the absolute input index; callers
  651|       |                // that need value context can add it in a higher-level facade.
  652|      6|                Err(TranscodeEncodeError::unencodable_without_context(attempt.input_index()))
  653|       |            }
  654|    517|            EncodeUnencodableAction::Skip => Ok(EncodeOutcome::consumed(0)),
  655|    519|            EncodeUnencodableAction::Replace { value } => self.encode_replacement_value(value, attempt),
  656|       |        }
  657|  1.04k|    }
  658|       |
  659|       |    /// Encodes a hook-provided replacement value.
  660|       |    ///
  661|       |    /// # Parameters
  662|       |    ///
  663|       |    /// - `value`: Replacement value selected by hooks.
  664|       |    /// - `attempt`: Original value, absolute indices, and writable output
  665|       |    ///   buffer.
  666|       |    ///
  667|       |    /// # Returns
  668|       |    ///
  669|       |    /// Returns the encode outcome for the replacement.
  670|       |    ///
  671|       |    /// # Errors
  672|       |    ///
  673|       |    /// Returns a codec error when replacement encoding fails.
  674|       |    ///
  675|       |    /// # Panics
  676|       |    ///
  677|       |    /// Panics when hooks return a replacement value that the codec cannot
  678|       |    /// encode, [`Codec::encode_len`] exceeds
  679|       |    /// [`Codec::MAX_ENCODE_UNITS_PER_VALUE`], or [`Codec::encode`] writes a
  680|       |    /// different number of units than planned.
  681|    519|    fn encode_replacement_value(
  682|    519|        &mut self,
  683|    519|        value: C::Value,
  684|    519|        attempt: EncodeAttempt<'_, C::Value, C::Unit>,
  685|    519|    ) -> Result<EncodeOutcome, TranscodeEncodeErrorOf<C>> {
  686|    519|        assert!(
  687|    519|            self.codec.can_encode_value(&value),
  688|       |            "EncodeUnencodableAction::Replace returned an unencodable replacement value",
  689|       |        );
  690|    518|        let required = self.codec.encode_len(&value);
  691|    518|        assert!(
  692|    518|            required <= C::MAX_ENCODE_UNITS_PER_VALUE,
  693|       |            "Codec::encode_len exceeded Codec::MAX_ENCODE_UNITS_PER_VALUE",
  694|       |        );
  695|    517|        if attempt.available_output() < required {
  696|      0|            return Ok(EncodeOutcome::need_output(
  697|      0|                NonZeroUsize::new(required).expect("required output is non-zero when capacity is insufficient"),
  698|      0|            ));
  699|    517|        }
  700|    517|        let (_, input_index, output, output_index) = attempt.into_parts();
  701|    516|        let written = unsafe {
  702|       |            // SAFETY: The capacity check above reserves the exact replacement
  703|       |            // value width, and the hook contract requires encodability.
  704|    517|            self.codec.encode(&value, output, output_index)
  705|       |        }
  706|    517|        .map_err(|error| TranscodeEncodeError::domain_main(error, input_index))?;
                                       ^1                                ^1     ^1           ^1
  707|    516|        assert!(
  708|    516|            written == required,
  709|       |            "Codec::encode wrote a different length than Codec::encode_len",
  710|       |        );
  711|    516|        Ok(EncodeOutcome::consumed(written))
  712|    517|    }
  713|       |}
  714|       |
  715|       |impl<C, H> Default for TranscodeEncodeEngine<C, H>
  716|       |where
  717|       |    C: Codec + Default,
  718|       |    H: TranscodeEncodeHooks<C> + Default,
  719|       |{
  720|       |    /// Creates a default buffered encoder engine.
  721|       |    ///
  722|       |    /// # Returns
  723|       |    ///
  724|       |    /// Returns an engine with default codec and hooks.
  725|       |    #[inline]
  726|      1|    fn default() -> Self {
  727|      1|        Self::new(C::default(), H::default())
  728|      1|    }
  729|       |}
  730|       |
  731|       |impl<C, H> Transcoder for TranscodeEncodeEngine<C, H>
  732|       |where
  733|       |    C: Codec,
  734|       |    H: TranscodeEncodeHooks<C>,
  735|       |{
  736|       |    /// Logical values borrowed from the caller during encoding.
  737|       |    type Input = C::Value;
  738|       |    /// Encoded storage units written into caller-owned buffers.
  739|       |    type Output = C::Unit;
  740|       |    /// Framework failures or codec-domain errors from encoding.
  741|       |    type Error = TranscodeEncodeErrorOf<C>;
  742|       |
  743|       |    /// Gets a conservative upper bound for output units needed for
  744|       |    /// `input_len` values.
  745|       |    ///
  746|       |    /// This bound covers only the streaming encode phase. It is delegated to
  747|       |    /// [`TranscodeEncodeHooks::max_transcode_output_len`], so it includes hook
  748|       |    /// policy and is valid for every reachable transient codec and hook state.
  749|       |    /// Downstream encoders must use this engine-level API for capacity planning
  750|       |    /// instead of recomputing the bound from [`Codec`] constants.
  751|       |    ///
  752|       |    /// # Parameters
  753|       |    ///
  754|       |    /// - `input_len`: Number of input values the caller plans to encode.
  755|       |    ///
  756|       |    /// # Returns
  757|       |    ///
  758|       |    /// Returns a conservative upper bound for output units.
  759|       |    ///
  760|       |    /// # Errors
  761|       |    ///
  762|       |    /// Returns a capacity error when hook planning overflows.
  763|       |    #[inline]
  764|      3|    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
  765|      3|        TranscodeEncodeEngine::max_transcode_output_len(self, input_len)
  766|      3|    }
  767|       |
  768|       |    /// Gets the global maximum output units emitted by stream reset.
  769|       |    ///
  770|       |    /// # Returns
  771|       |    ///
  772|       |    /// Returns the codec's reset-output upper bound. This query never fails.
  773|       |    #[inline]
  774|      2|    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
  775|      2|        TranscodeEncodeEngine::max_reset_output_len(self)
  776|      2|    }
  777|       |
  778|       |    /// Gets the global maximum output units emitted by finishing codec and hook
  779|       |    /// state.
  780|       |    ///
  781|       |    /// Returns the sum of [`Codec::MAX_ENCODE_FINISH_UNITS`] and the
  782|       |    /// hook-provided final-output bound. The codec finish portion covers units
  783|       |    /// written by [`Codec::encode_finish`]; hook implementations must not
  784|       |    /// include that portion in
  785|       |    /// [`TranscodeEncodeHooks::max_finish_output_len`]. Both component bounds
  786|       |    /// cover every reachable transient state.
  787|       |    ///
  788|       |    /// # Returns
  789|       |    ///
  790|       |    /// Returns the combined codec-finish and hook-finish output bound.
  791|       |    ///
  792|       |    /// # Errors
  793|       |    ///
  794|       |    /// Returns [`CapacityError::OutputLengthOverflow`] when the sum overflows.
  795|       |    #[inline]
  796|      2|    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
  797|      2|        TranscodeEncodeEngine::max_finish_output_len(self)
  798|      2|    }
  799|       |
  800|       |    /// Resets codec encode state, hook-owned state, and stream-start output.
  801|       |    ///
  802|       |    /// # Parameters
  803|       |    ///
  804|       |    /// - `output`: Complete output unit slice visible to the encoder.
  805|       |    /// - `output_index`: Absolute output unit index where writing starts.
  806|       |    ///
  807|       |    /// # Returns
  808|       |    ///
  809|       |    /// Returns the number of reset units written.
  810|       |    ///
  811|       |    /// # Errors
  812|       |    ///
  813|       |    /// Returns framework errors when the caller provides invalid or
  814|       |    /// insufficient output capacity. Returns domain errors when codec reset or
  815|       |    /// hook reset handling fails. Capacity and index failures occur before any
  816|       |    /// reset state is changed. Once reset execution starts, an error poisons
  817|       |    /// the engine until a later reset succeeds.
  818|       |    ///
  819|       |    /// # Panics
  820|       |    ///
  821|       |    /// Panics if the codec reports reset output beyond its declared bound.
  822|       |    #[inline]
  823|      3|    fn reset(&mut self, output: &mut [C::Unit], output_index: usize) -> Result<usize, TranscodeEncodeErrorOf<C>> {
  824|      3|        TranscodeEncodeEngine::reset(self, output, output_index)
  825|      3|    }
  826|       |
  827|       |    /// Encodes values into a caller-provided output buffer.
  828|       |    ///
  829|       |    /// The engine stops before consuming the next input value when the current
  830|       |    /// output buffer does not satisfy that value's planned capacity bound.
  831|       |    ///
  832|       |    /// # Parameters
  833|       |    ///
  834|       |    /// - `input`: Complete input value slice visible to the encoder.
  835|       |    /// - `input_index`: Absolute input value index where encoding starts.
  836|       |    /// - `output`: Complete output unit slice visible to the encoder.
  837|       |    /// - `output_index`: Absolute output unit index where writing starts.
  838|       |    ///
  839|       |    /// # Returns
  840|       |    ///
  841|       |    /// Returns progress describing input values consumed, output units written,
  842|       |    /// and why encoding stopped.
  843|       |    ///
  844|       |    /// # Errors
  845|       |    ///
  846|       |    /// Returns framework errors when `input_index` is outside `input` or
  847|       |    /// `output_index` is outside `output`, and domain errors when codec or
  848|       |    /// hook processing fails or rejects a value. Returns
  849|       |    /// [`TranscodeFailure::TranscodeBeforeReset`] when the engine has not
  850|       |    /// completed its first reset. Returns
  851|       |    /// [`TranscodeFailure::TranscodeAfterFinish`] when the logical stream was
  852|       |    /// already finished and has not been reset, or
  853|       |    /// [`TranscodeFailure::LifecyclePoisoned`] when an earlier reset or finish
  854|       |    /// failed after execution started.
  855|       |    ///
  856|       |    /// # Panics
  857|       |    ///
  858|       |    /// Panics if the codec violates its declared encode length or hooks
  859|       |    /// return an unencodable replacement.
  860|       |    #[inline]
  861|      2|    fn transcode(
  862|      2|        &mut self,
  863|      2|        input: &[C::Value],
  864|      2|        input_index: usize,
  865|      2|        output: &mut [C::Unit],
  866|      2|        output_index: usize,
  867|      2|    ) -> Result<TranscodeProgress, TranscodeEncodeErrorOf<C>> {
  868|      2|        TranscodeEncodeEngine::transcode(self, input, input_index, output, output_index)
  869|      2|    }
  870|       |
  871|       |    /// Finishes codec and hook-owned output after EOF.
  872|       |    ///
  873|       |    /// Finalization first finishes encode-side codec state through
  874|       |    /// [`Codec::encode_finish`], then lets hook implementations finish their
  875|       |    /// own retained state. The caller must provide enough output capacity for
  876|       |    /// [`TranscodeEncodeEngine::max_finish_output_len`], which includes both
  877|       |    /// the codec finish bound and the hook-owned finish bound.
  878|       |    ///
  879|       |    /// # Parameters
  880|       |    ///
  881|       |    /// - `output`: Complete output unit slice visible to the encoder.
  882|       |    /// - `output_index`: Absolute output unit index where writing starts.
  883|       |    ///
  884|       |    /// # Returns
  885|       |    ///
  886|       |    /// Returns the number of units written by finalization.
  887|       |    ///
  888|       |    /// # Errors
  889|       |    ///
  890|       |    /// Returns framework errors when the caller provides invalid or
  891|       |    /// insufficient output capacity. Returns domain errors when codec finish or
  892|       |    /// hook finalization fails. Returns
  893|       |    /// [`TranscodeFailure::FinishBeforeReset`] when the engine has not
  894|       |    /// completed its first reset. Returns
  895|       |    /// [`TranscodeFailure::FinishAfterFinish`] when the logical stream was
  896|       |    /// already finished and has not been reset, or
  897|       |    /// [`TranscodeFailure::LifecyclePoisoned`] when an earlier reset or finish
  898|       |    /// failed after execution started. Capacity and index failures occur before
  899|       |    /// finish execution and remain retryable; later failures poison the engine
  900|       |    /// until reset succeeds.
  901|       |    ///
  902|       |    /// # Panics
  903|       |    ///
  904|       |    /// Panics when the codec finish writes beyond
  905|       |    /// [`Codec::MAX_ENCODE_FINISH_UNITS`] or when the combined codec and hook
  906|       |    /// finalization writes beyond
  907|       |    /// [`TranscodeEncodeEngine::max_finish_output_len`].
  908|       |    #[inline]
  909|      2|    fn finish(&mut self, output: &mut [C::Unit], output_index: usize) -> Result<usize, TranscodeEncodeErrorOf<C>> {
  910|      2|        TranscodeEncodeEngine::finish(self, output, output_index)
  911|      2|    }
  912|       |}
  913|       |
  914|       |impl<C, H> TranscodeEncoder for TranscodeEncodeEngine<C, H>
  915|       |where
  916|       |    C: Codec,
  917|       |    H: TranscodeEncodeHooks<C>,
  918|       |{
  919|       |    /// Codec-specific source error retained by domain failures.
  920|       |    type EncodeError = C::EncodeError;
  921|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/engine/transcode_encode_hooks.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Policy hooks used by buffered encoder engines.
    9|       |
   10|       |use super::encode_context::EncodeContext;
   11|       |use super::encode_unencodable_action::EncodeUnencodableAction;
   12|       |use crate::CapacityError;
   13|       |use crate::Codec;
   14|       |use crate::TranscodeEncodeErrorOf;
   15|       |
   16|       |/// Policy hooks for [`crate::engine::TranscodeEncodeEngine`].
   17|       |///
   18|       |/// Hooks own policy state, such as replacement or ignore behavior, but not the
   19|       |/// codec or engine cursor state. The engine owns the normal one-value encode
   20|       |/// operation and calls hooks only when policy decisions are needed.
   21|       |///
   22|       |/// Implement this trait when a buffered encoder needs policy decisions around
   23|       |/// individual values while reusing the common engine loop. Examples include
   24|       |/// rejecting unsupported values with adapter-level context, skipping
   25|       |/// unsupported values, replacing them with encodable values, resetting
   26|       |/// hook-owned state in [`reset_hooks`](Self::reset_hooks), or emitting final
   27|       |/// state in [`finish_hooks`](Self::finish_hooks).
   28|       |///
   29|       |/// The engine calls
   30|       |/// [`handle_unencodable_encode`](Self::handle_unencodable_encode) only after
   31|       |/// [`Codec::can_encode_value`] returns `false` for the current input
   32|       |/// value. Encodable values are encoded directly by the engine with
   33|       |/// [`Codec::encode_len`] and [`Codec::encode`].
   34|       |///
   35|       |/// # Examples
   36|       |///
   37|       |/// This hook reports unsupported values as adapter-level errors.
   38|       |///
   39|       |/// ```rust
   40|       |/// use core::convert::Infallible;
   41|       |/// use core::num::NonZeroUsize;
   42|       |///
   43|       |/// use qubit_codec::Codec;
   44|       |/// use qubit_codec::DecodeFailure;
   45|       |/// use qubit_codec::TranscodeEncodeErrorOf;
   46|       |/// use qubit_codec::engine::EncodeContext;
   47|       |/// use qubit_codec::engine::EncodeUnencodableAction;
   48|       |/// use qubit_codec::engine::TranscodeEncodeEngine;
   49|       |/// use qubit_codec::engine::TranscodeEncodeHooks;
   50|       |///
   51|       |/// #[derive(Clone, Copy)]
   52|       |/// struct ByteCodec;
   53|       |///
   54|       |/// impl Codec for ByteCodec {
   55|       |///     type Value = u8;
   56|       |///     type Unit = u8;
   57|       |///     type DecodeError = Infallible;
   58|       |///     type EncodeError = Infallible;
   59|       |///
   60|       |///     const MIN_UNITS_PER_VALUE: usize = 1;
   61|       |///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
   62|       |///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
   63|       |///
   64|       |///     unsafe fn decode(
   65|       |///         &mut self,
   66|       |///         input: &[u8],
   67|       |///         index: usize,
   68|       |///     ) -> Result<(u8, NonZeroUsize), DecodeFailure<Self::DecodeError>> {
   69|       |///         Ok((input[index], NonZeroUsize::MIN))
   70|       |///     }
   71|       |///
   72|       |///     unsafe fn encode(
   73|       |///         &mut self,
   74|       |///         value: &u8,
   75|       |///         output: &mut [u8],
   76|       |///         index: usize,
   77|       |///     ) -> Result<usize, Self::EncodeError> {
   78|       |///         output[index] = *value;
   79|       |///         Ok(1)
   80|       |///     }
   81|       |/// }
   82|       |///
   83|       |/// struct StrictHooks;
   84|       |///
   85|       |/// impl<C> TranscodeEncodeHooks<C> for StrictHooks
   86|       |/// where
   87|       |///     C: Codec,
   88|       |/// {
   89|       |///     fn handle_unencodable_encode(
   90|       |///         &mut self,
   91|       |///         _codec: &mut C,
   92|       |///         _context: &EncodeContext<'_, C::Value>,
   93|       |///     ) -> Result<EncodeUnencodableAction<C::Value>, TranscodeEncodeErrorOf<C>> {
   94|       |///         Ok(EncodeUnencodableAction::Reject)
   95|       |///     }
   96|       |/// }
   97|       |///
   98|       |/// let codec = ByteCodec;
   99|       |/// let hooks = StrictHooks;
  100|       |/// let bound = hooks.max_transcode_output_len(&codec, 1).unwrap();
  101|       |/// assert!(bound >= ByteCodec::MAX_ENCODE_UNITS_PER_VALUE);
  102|       |/// let mut engine = TranscodeEncodeEngine::new(codec, hooks);
  103|       |/// engine.reset(&mut [], 0).unwrap();
  104|       |/// let mut output = [0_u8; 1];
  105|       |/// let progress = engine.transcode(&[42], 0, &mut output, 0).unwrap();
  106|       |/// assert_eq!(progress.read(), 1);
  107|       |/// assert_eq!(progress.written(), 1);
  108|       |/// assert_eq!(output, [42]);
  109|       |/// ```
  110|       |///
  111|       |/// # Type Parameters
  112|       |///
  113|       |/// - `C`: Low-level codec owned by the engine.
  114|       |pub trait TranscodeEncodeHooks<C>
  115|       |where
  116|       |    C: Codec,
  117|       |{
  118|       |    /// Returns the maximum output units needed for `input_len` values.
  119|       |    ///
  120|       |    /// This bound covers the streaming encode phase, including conservative
  121|       |    /// capacity preflight in streaming and EOF operations, as required by
  122|       |    /// [`crate::Transcoder::max_transcode_output_len`]. It describes required
  123|       |    /// writable capacity, not only units actually emitted. It must not include
  124|       |    /// codec reset output, codec finish output, or hook finish output.
  125|       |    /// It may depend on immutable hook and codec configuration, but it must
  126|       |    /// cover every reachable transient hook and codec state. Each streaming
  127|       |    /// action handles one current input value by skipping it or encoding at
  128|       |    /// most one replacement; the hook API has no separate operation for
  129|       |    /// draining hook-owned output without input. Use a custom
  130|       |    /// [`crate::Transcoder`] for streaming behavior beyond this action model,
  131|       |    /// such as one-to-many output or draining output without new input.
  132|       |    ///
  133|       |    /// Before [`handle_unencodable_encode`](Self::handle_unencodable_encode),
  134|       |    /// the engine may preflight [`Codec::MAX_ENCODE_UNITS_PER_VALUE`], even
  135|       |    /// when the selected action is `Skip` or `Reject` and emits no units.
  136|       |    /// Thus the bound must leave room for that preflight; zero actual output
  137|       |    /// does not imply zero required writable capacity. See the
  138|       |    /// [trait example](#examples) for a capacity check before constructing the
  139|       |    /// engine.
  140|       |    ///
  141|       |    /// The default implementation multiplies `input_len` by
  142|       |    /// [`Codec::MAX_ENCODE_UNITS_PER_VALUE`]. This bound is valid for direct
  143|       |    /// encoding, skipped unencodable values, and single-value replacements.
  144|       |    /// Override it when the codec's streaming output bound requires a larger
  145|       |    /// value.
  146|       |    ///
  147|       |    /// # Parameters
  148|       |    ///
  149|       |    /// - `codec`: Low-level codec owned by the engine.
  150|       |    /// - `input_len`: Number of input values the caller plans to encode.
  151|       |    ///
  152|       |    /// # Returns
  153|       |    ///
  154|       |    /// Returns a conservative upper bound for streaming output.
  155|       |    ///
  156|       |    /// # Errors
  157|       |    ///
  158|       |    /// Returns [`CapacityError::OutputLengthOverflow`] when the bound cannot
  159|       |    /// be represented as `usize`.
  160|       |    #[inline]
  161|  9.61k|    fn max_transcode_output_len(&self, _codec: &C, input_len: usize) -> Result<usize, CapacityError> {
  162|  9.61k|        input_len
  163|  9.61k|            .checked_mul(C::MAX_ENCODE_UNITS_PER_VALUE)
  164|  9.61k|            .ok_or(CapacityError::OutputLengthOverflow)
  165|  9.61k|    }
  166|       |
  167|       |    /// Returns an upper bound for units emitted by finishing hook-owned state.
  168|       |    ///
  169|       |    /// `finish` never receives more input values. Implementations must report
  170|       |    /// the maximum output that could be derived from hook-owned state in any
  171|       |    /// reachable transient stream state, not only what happens to remain when
  172|       |    /// this method is called.
  173|       |    ///
  174|       |    /// The default implementation returns `0`. Override it when
  175|       |    /// [`finish_hooks`](Self::finish_hooks) can write trailers, checksums,
  176|       |    /// delayed replacements, or any other hook-owned final output. Do not
  177|       |    /// include [`Codec::MAX_ENCODE_FINISH_UNITS`]; the engine adds the codec
  178|       |    /// finish bound separately.
  179|       |    ///
  180|       |    /// # Parameters
  181|       |    ///
  182|       |    /// - `codec`: Low-level codec owned by the engine.
  183|       |    ///
  184|       |    /// # Returns
  185|       |    ///
  186|       |    /// Returns the finite, transient-state-independent final-output upper
  187|       |    /// bound.
  188|       |    #[inline]
  189|       |    #[must_use]
  190|  4.01k|    fn max_finish_output_len(&self, _codec: &C) -> usize {
  191|  4.01k|        0
  192|  4.01k|    }
  193|       |
  194|       |    /// Handles one unencodable input value.
  195|       |    ///
  196|       |    /// # Parameters
  197|       |    ///
  198|       |    /// - `codec`: Low-level codec owned by the engine.
  199|       |    /// - `context`: Encode context for the input value that
  200|       |    ///   [`Codec::can_encode_value`] rejected.
  201|       |    ///
  202|       |    /// # Returns
  203|       |    ///
  204|       |    /// Returns the action selected by this hook policy.
  205|       |    ///
  206|       |    /// Replacement actions must contain a value that the same codec can encode.
  207|       |    /// The engine treats unencodable replacements as hook contract violations
  208|       |    /// and panics.
  209|       |    ///
  210|       |    /// Before invoking this method, the engine requires writable capacity for
  211|       |    /// [`Codec::MAX_ENCODE_UNITS_PER_VALUE`]. This conservative bound also
  212|       |    /// applies when the selected action is `Skip` or `Reject`; it may exceed
  213|       |    /// the replacement's actual encoded width. A capacity shortage does not
  214|       |    /// invoke the hook, so hook state is not changed by output backpressure.
  215|       |    ///
  216|       |    /// # Errors
  217|       |    ///
  218|       |    /// Returns [`crate::TranscodeEncodeError`] when the policy rejects the
  219|       |    /// value with a codec-domain error.
  220|       |    fn handle_unencodable_encode(
  221|       |        &mut self,
  222|       |        codec: &mut C,
  223|       |        context: &EncodeContext<'_, C::Value>,
  224|       |    ) -> Result<EncodeUnencodableAction<C::Value>, TranscodeEncodeErrorOf<C>>;
  225|       |
  226|       |    /// Runs hook-owned cleanup as part of stream reset.
  227|       |    ///
  228|       |    /// Called before [`Codec::encode_reset`] writes
  229|       |    /// its own reset output. Stateless hooks may use the default no-op.
  230|       |    ///
  231|       |    /// # Parameters
  232|       |    ///
  233|       |    /// - `codec`: Low-level codec owned by the engine.
  234|       |    #[inline]
  235|  1.28k|    fn reset_hooks(&mut self, _codec: &mut C) {}
  236|       |
  237|       |    /// Finishes hook-owned state and writes any retained output units.
  238|       |    ///
  239|       |    /// The default implementation is a no-op for stateless encode hooks.
  240|       |    /// Stateful hooks may emit final units such as reset sequences, checksums,
  241|       |    /// or trailers. The caller must provide at least
  242|       |    /// [`TranscodeEncodeHooks::max_finish_output_len`] writable units from
  243|       |    /// `output_index`. Implementations must not write beyond that declared
  244|       |    /// final-output bound.
  245|       |    ///
  246|       |    /// Called after [`Codec::encode_finish`] has
  247|       |    /// written its own finish output.
  248|       |    ///
  249|       |    /// # Parameters
  250|       |    ///
  251|       |    /// - `codec`: Low-level codec owned by the engine.
  252|       |    /// - `output`: Output unit slice visible to the hook.
  253|       |    /// - `output_index`: Absolute output unit index where writing starts.
  254|       |    ///
  255|       |    /// # Returns
  256|       |    ///
  257|       |    /// Returns the number of units written by finalization. This count must not
  258|       |    /// exceed [`TranscodeEncodeHooks::max_finish_output_len`].
  259|       |    ///
  260|       |    /// # Errors
  261|       |    ///
  262|       |    /// Returns a transcode encode error when hook-owned state cannot be
  263|       |    /// finalized.
  264|       |    #[inline]
  265|  1.15k|    fn finish_hooks(
  266|  1.15k|        &mut self,
  267|  1.15k|        _codec: &mut C,
  268|  1.15k|        _output: &mut [C::Unit],
  269|  1.15k|        _output_index: usize,
  270|  1.15k|    ) -> Result<usize, TranscodeEncodeErrorOf<C>> {
  271|  1.15k|        Ok(0)
  272|  1.15k|    }
  273|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/internal/convert_state.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Mutable state for one buffered conversion call.
    9|       |
   10|       |use core::num::NonZeroUsize;
   11|       |
   12|       |use super::super::engine::DecodeContext;
   13|       |use super::super::engine::DecodeOutcome;
   14|       |use super::super::engine::EncodeOutcome;
   15|       |use super::super::transcode_progress::TranscodeProgress;
   16|       |use super::transcode_state::TranscodeState;
   17|       |
   18|       |/// Mutable state for one buffered conversion call.
   19|       |///
   20|       |/// `ConvertState` is an internal cursor helper owned by
   21|       |/// [`crate::engine::TranscodeConvertEngine`]. Hook implementations receive
   22|       |/// narrower context objects and never own converter cursor state.
   23|       |///
   24|       |/// # Type Parameters
   25|       |///
   26|       |/// - `Input`: Source unit type borrowed for this call.
   27|       |/// - `Output`: Target unit type written through the exclusively borrowed
   28|       |///   output.
   29|       |pub(crate) struct ConvertState<'a, Input, Output> {
   30|       |    /// Shared input/output state for this conversion call.
   31|       |    state: TranscodeState<'a, Input, Output>,
   32|       |}
   33|       |
   34|       |impl<'a, Input, Output> ConvertState<'a, Input, Output> {
   35|       |    /// Creates mutable conversion state.
   36|       |    ///
   37|       |    /// # Parameters
   38|       |    ///
   39|       |    /// - `input`: Complete input unit slice visible to the converter.
   40|       |    /// - `input_index`: Absolute input index where conversion starts.
   41|       |    /// - `output`: Complete output unit slice visible to the converter.
   42|       |    /// - `output_index`: Absolute output index where writing starts.
   43|       |    ///
   44|       |    /// # Returns
   45|       |    ///
   46|       |    /// Returns initialized conversion state with cursors at the requested start
   47|       |    /// positions.
   48|       |    #[inline]
   49|       |    #[must_use]
   50|  6.33k|    pub(crate) fn new(input: &'a [Input], input_index: usize, output: &'a mut [Output], output_index: usize) -> Self {
   51|  6.33k|        Self {
   52|  6.33k|            state: TranscodeState::new(input, input_index, output, output_index),
   53|  6.33k|        }
   54|  6.33k|    }
   55|       |
   56|       |    /// Returns the complete input slice.
   57|       |    ///
   58|       |    /// # Returns
   59|       |    ///
   60|       |    /// Returns the full input slice.
   61|       |    #[inline]
   62|       |    #[must_use]
   63|  4.43k|    pub(crate) fn input(&self) -> &[Input] {
   64|  4.43k|        self.state.input()
   65|  4.43k|    }
   66|       |
   67|       |    /// Returns the complete output slice mutably.
   68|       |    ///
   69|       |    /// # Returns
   70|       |    ///
   71|       |    /// Returns the full mutable output slice.
   72|       |    #[inline]
   73|       |    #[must_use]
   74|  7.23k|    pub(crate) fn output_mut(&mut self) -> &mut [Output] {
   75|  7.23k|        self.state.output_mut()
   76|  7.23k|    }
   77|       |
   78|       |    /// Returns the current output cursor.
   79|       |    ///
   80|       |    /// # Returns
   81|       |    ///
   82|       |    /// Returns current output cursor.
   83|       |    #[inline]
   84|       |    #[must_use]
   85|  7.23k|    pub(crate) fn output_cursor(&self) -> usize {
   86|  7.23k|        self.state.output_cursor()
   87|  7.23k|    }
   88|       |
   89|       |    /// Returns whether there is still input to convert.
   90|       |    ///
   91|       |    /// # Returns
   92|       |    ///
   93|       |    /// Returns `true` when more input units remain.
   94|       |    #[inline]
   95|       |    #[must_use]
   96|  7.40k|    pub(crate) fn has_input(&self) -> bool {
   97|  7.40k|        self.state.has_input()
   98|  7.40k|    }
   99|       |
  100|       |    /// Returns input units visible from the current input cursor.
  101|       |    ///
  102|       |    /// # Returns
  103|       |    ///
  104|       |    /// Returns remaining input units visible from `input_cursor`.
  105|       |    #[inline]
  106|       |    #[must_use]
  107|  14.4k|    pub(crate) fn available_input(&self) -> usize {
  108|  14.4k|        self.state.available_input()
  109|  14.4k|    }
  110|       |
  111|       |    /// Returns writable output units visible from the current output cursor.
  112|       |    ///
  113|       |    /// # Returns
  114|       |    ///
  115|       |    /// Returns remaining writable output capacity from `output_cursor`.
  116|       |    #[inline]
  117|       |    #[must_use]
  118|  7.22k|    pub(crate) fn available_output(&self) -> usize {
  119|  7.22k|        self.state.available_output()
  120|  7.22k|    }
  121|       |
  122|       |    /// Returns a public decode context snapshot at the current cursors.
  123|       |    ///
  124|       |    /// # Returns
  125|       |    ///
  126|       |    /// Returns context values suitable for decode-error hook dispatch.
  127|       |    #[inline]
  128|       |    #[must_use]
  129|  4.43k|    pub(crate) fn decode_context(&self) -> DecodeContext {
  130|  4.43k|        DecodeContext::new(
  131|  4.43k|            self.state.input_start(),
  132|  4.43k|            self.state.input_cursor(),
  133|  4.43k|            self.state.output_start(),
  134|  4.43k|            self.state.output_cursor(),
  135|  4.43k|            self.available_input(),
  136|       |        )
  137|  4.43k|    }
  138|       |
  139|       |    /// Returns input units consumed since this call started.
  140|       |    ///
  141|       |    /// # Returns
  142|       |    ///
  143|       |    /// Returns consumed input units relative to `input_start`.
  144|       |    #[inline]
  145|       |    #[must_use]
  146|  7.89k|    pub(crate) fn read(&self) -> usize {
  147|  7.89k|        self.state.read()
  148|  7.89k|    }
  149|       |
  150|       |    /// Returns output units written since this call started.
  151|       |    ///
  152|       |    /// # Returns
  153|       |    ///
  154|       |    /// Returns written output units relative to `output_start`.
  155|       |    #[inline]
  156|       |    #[must_use]
  157|  2.37k|    pub(crate) fn written(&self) -> usize {
  158|  2.37k|        self.state.written()
  159|  2.37k|    }
  160|       |
  161|       |    /// Returns completed progress for the current cursors.
  162|       |    ///
  163|       |    /// # Returns
  164|       |    ///
  165|       |    /// Returns [`TranscodeProgress::complete`]-style state.
  166|       |    #[inline]
  167|  1.82k|    pub(crate) fn complete_progress(&self) -> TranscodeProgress {
  168|  1.82k|        self.state.complete_progress()
  169|  1.82k|    }
  170|       |
  171|       |    /// Returns progress for missing input.
  172|       |    ///
  173|       |    /// # Parameters
  174|       |    ///
  175|       |    /// - `required`: Current minimum total input units required before retrying
  176|       |    ///   from the current input position. A later retry may raise this lower
  177|       |    ///   bound.
  178|       |    /// # Returns
  179|       |    ///
  180|       |    /// Returns [`TranscodeProgress`] with
  181|       |    /// [`crate::TranscodeStatus::NeedInput`].
  182|       |    #[inline]
  183|  1.15k|    pub(crate) fn need_input_progress(&self, required: NonZeroUsize) -> TranscodeProgress {
  184|  1.15k|        self.state.need_input_progress(required)
  185|  1.15k|    }
  186|       |
  187|       |    /// Returns progress for missing output.
  188|       |    ///
  189|       |    /// # Parameters
  190|       |    ///
  191|       |    /// - `required`: Total output units required from the current output
  192|       |    ///   position.
  193|       |    /// # Returns
  194|       |    ///
  195|       |    /// Returns [`TranscodeProgress`] with
  196|       |    /// [`crate::TranscodeStatus::NeedOutput`].
  197|       |    #[inline]
  198|    962|    pub(crate) fn need_output_progress(&self, required: NonZeroUsize) -> TranscodeProgress {
  199|    962|        self.state.need_output_progress(required)
  200|    962|    }
  201|       |
  202|       |    /// Advances the input cursor.
  203|       |    ///
  204|       |    /// # Parameters
  205|       |    ///
  206|       |    /// - `read`: Number of input units consumed by the conversion step.
  207|       |    ///
  208|       |    /// # Panics
  209|       |    ///
  210|       |    /// Panics when `read` exceeds the remaining input units.
  211|       |    #[inline]
  212|  4.42k|    pub(crate) fn advance_input(&mut self, read: usize) {
  213|  4.42k|        assert!(read <= self.available_input(), "conversion step read beyond input");
  214|  4.42k|        self.state.advance_input(read);
  215|  4.42k|    }
  216|       |
  217|       |    /// Advances the output cursor.
  218|       |    ///
  219|       |    /// # Parameters
  220|       |    ///
  221|       |    /// - `written`: Number of output units written by the conversion step.
  222|       |    ///
  223|       |    /// # Panics
  224|       |    ///
  225|       |    /// Panics when `written` exceeds the remaining output capacity.
  226|       |    #[inline]
  227|  6.26k|    pub(crate) fn advance_output(&mut self, written: usize) {
  228|  6.26k|        assert!(
  229|  6.26k|            written <= self.available_output(),
  230|       |            "conversion step wrote beyond output",
  231|       |        );
  232|  6.26k|        self.state.advance_output(written);
  233|  6.26k|    }
  234|       |
  235|       |    /// Applies one decode outcome to this conversion state.
  236|       |    ///
  237|       |    /// # Parameters
  238|       |    ///
  239|       |    /// - `outcome`: Decode outcome produced by the decode engine.
  240|       |    ///
  241|       |    /// # Returns
  242|       |    ///
  243|       |    /// Returns stop progress for missing input, otherwise `None`.
  244|       |    ///
  245|       |    /// # Panics
  246|       |    ///
  247|       |    /// Panics if an emitted outcome contains other than one value, or if the
  248|       |    /// outcome consumes more input units than remain.
  249|       |    #[inline]
  250|  4.42k|    pub(crate) fn apply_decode_outcome(&mut self, outcome: DecodeOutcome) -> Option<TranscodeProgress> {
  251|  4.42k|        match outcome {
  252|  3.89k|            DecodeOutcome::Emitted { read, emitted } => {
  253|  3.89k|                assert_eq!(
  254|       |                    emitted,
  255|       |                    NonZeroUsize::MIN,
  256|       |                    "converter decode outcome can carry only one pending value",
  257|       |                );
  258|  3.89k|                self.advance_input(read.get());
  259|  3.89k|                None
  260|       |            }
  261|    533|            DecodeOutcome::Skipped { read } => {
  262|    533|                self.advance_input(read.get());
  263|    533|                None
  264|       |            }
  265|      3|            DecodeOutcome::NeedInput { required } => Some(self.need_input_progress(required)),
  266|       |        }
  267|  4.42k|    }
  268|       |
  269|       |    /// Applies one normalized encode outcome to this conversion state.
  270|       |    ///
  271|       |    /// # Parameters
  272|       |    ///
  273|       |    /// - `outcome`: Encode outcome produced by the target encoder hooks.
  274|       |    ///
  275|       |    /// # Returns
  276|       |    ///
  277|       |    /// Returns `Some(progress)` when target output is insufficient and
  278|       |    /// conversion must stop, otherwise returns `None` after advancing the
  279|       |    /// output cursor.
  280|       |    ///
  281|       |    /// # Panics
  282|       |    ///
  283|       |    /// Panics if an outcome writes beyond the remaining output or requests
  284|       |    /// output capacity that is already available.
  285|       |    #[inline]
  286|  4.84k|    pub(crate) fn apply_encode_outcome(&mut self, outcome: EncodeOutcome) -> Option<TranscodeProgress> {
  287|  4.84k|        match outcome {
  288|  3.88k|            EncodeOutcome::Consumed { written } => {
  289|  3.88k|                self.advance_output(written);
  290|  3.88k|                None
  291|       |            }
  292|    962|            EncodeOutcome::NeedOutput { required } => {
  293|    962|                let available = self.available_output();
  294|    962|                assert!(
  295|    962|                    required.get() > available,
  296|       |                    "EncodeOutcome::NeedOutput required capacity must exceed available output",
  297|       |                );
  298|    962|                Some(self.need_output_progress(required))
  299|       |            }
  300|       |        }
  301|  4.84k|    }
  302|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/internal/decode_state.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Mutable state for one buffered decode call.
    9|       |
   10|       |use core::num::NonZeroUsize;
   11|       |
   12|       |use super::super::engine::DecodeContext;
   13|       |use super::super::engine::DecodeOutcome;
   14|       |use super::super::transcode_progress::TranscodeProgress;
   15|       |use super::transcode_state::TranscodeState;
   16|       |
   17|       |/// Mutable state for one buffered decode call.
   18|       |///
   19|       |/// Borrows input and exclusively borrows output for the call lifetime `'a`.
   20|       |/// Cursor updates do not allocate or perform I/O.
   21|       |///
   22|       |/// # Type Parameters
   23|       |///
   24|       |/// - `'a`: Lifetime of the input and exclusive output borrows.
   25|       |/// - `Unit`: Encoded input element.
   26|       |/// - `Value`: Decoded output element.
   27|       |pub(in crate::transcode) struct DecodeState<'a, Unit, Value> {
   28|       |    /// Shared input/output state for this decode call.
   29|       |    state: TranscodeState<'a, Unit, Value>,
   30|       |}
   31|       |
   32|       |impl<'a, Unit, Value> DecodeState<'a, Unit, Value> {
   33|       |    /// Creates mutable decode state.
   34|       |    ///
   35|       |    /// # Parameters
   36|       |    ///
   37|       |    /// - `input`: Complete input unit slice visible to the decoder.
   38|       |    /// - `input_index`: Absolute source index where decoding starts.
   39|       |    /// - `output`: Complete output value slice visible to the decoder.
   40|       |    /// - `output_index`: Absolute output index where writing starts.
   41|       |    ///
   42|       |    /// # Returns
   43|       |    ///
   44|       |    /// Returns initialized decode state with cursors at the requested start
   45|       |    /// positions.
   46|       |    #[inline]
   47|       |    #[must_use]
   48|     65|    pub(in crate::transcode) fn new(
   49|     65|        input: &'a [Unit],
   50|     65|        input_index: usize,
   51|     65|        output: &'a mut [Value],
   52|     65|        output_index: usize,
   53|     65|    ) -> Self {
   54|     65|        Self {
   55|     65|            state: TranscodeState::new(input, input_index, output, output_index),
   56|     65|        }
   57|     65|    }
   58|       |
   59|       |    /// Returns the complete output slice mutably.
   60|       |    ///
   61|       |    /// # Returns
   62|       |    ///
   63|       |    /// Returns the full mutable output slice visible to this decode call.
   64|       |    #[inline]
   65|       |    #[must_use]
   66|     67|    pub(in crate::transcode) fn output_mut(&mut self) -> &mut [Value] {
   67|     67|        self.state.output_mut()
   68|     67|    }
   69|       |
   70|       |    /// Returns the current output cursor.
   71|       |    ///
   72|       |    /// # Returns
   73|       |    ///
   74|       |    /// Returns the absolute output cursor for the next decoded value.
   75|       |    #[inline]
   76|       |    #[must_use]
   77|     67|    pub(in crate::transcode) fn output_cursor(&self) -> usize {
   78|     67|        self.state.output_cursor()
   79|     67|    }
   80|       |
   81|       |    /// Returns whether there is still input to decode.
   82|       |    ///
   83|       |    /// # Returns
   84|       |    ///
   85|       |    /// Returns `true` when there are input units remaining.
   86|       |    #[inline]
   87|       |    #[must_use]
   88|    114|    pub(in crate::transcode) fn has_input(&self) -> bool {
   89|    114|        self.state.has_input()
   90|    114|    }
   91|       |
   92|       |    /// Returns whether the output slice has no slot for the next value.
   93|       |    ///
   94|       |    /// # Returns
   95|       |    ///
   96|       |    /// Returns `true` when no output slot remains for the next value.
   97|       |    #[inline]
   98|       |    #[must_use]
   99|     79|    pub(in crate::transcode) fn needs_output(&self) -> bool {
  100|     79|        self.state.needs_output()
  101|     79|    }
  102|       |
  103|       |    /// Returns a public decode context snapshot.
  104|       |    ///
  105|       |    /// # Returns
  106|       |    ///
  107|       |    /// Returns the current [`DecodeContext`].
  108|       |    #[inline]
  109|       |    #[must_use]
  110|     83|    pub(in crate::transcode) fn context(&self) -> DecodeContext {
  111|     83|        DecodeContext::new(
  112|     83|            self.state.input_start(),
  113|     83|            self.state.input_cursor(),
  114|     83|            self.state.output_start(),
  115|     83|            self.state.output_cursor(),
  116|     83|            self.available(),
  117|       |        )
  118|     83|    }
  119|       |
  120|       |    /// Returns completed progress for the current cursors.
  121|       |    ///
  122|       |    /// # Returns
  123|       |    ///
  124|       |    /// Returns a completed [`TranscodeProgress`].
  125|       |    #[inline]
  126|     31|    pub(in crate::transcode) fn complete_progress(&self) -> TranscodeProgress {
  127|     31|        self.state.complete_progress()
  128|     31|    }
  129|       |
  130|       |    /// Returns progress for a missing output slot.
  131|       |    ///
  132|       |    /// # Returns
  133|       |    ///
  134|       |    /// Returns progress with [`crate::TranscodeStatus::NeedOutput`].
  135|       |    #[inline]
  136|     12|    pub(in crate::transcode) fn need_output_progress(&self) -> TranscodeProgress {
  137|     12|        self.state.need_output_progress(NonZeroUsize::MIN)
  138|     12|    }
  139|       |
  140|       |    /// Returns progress for a policy-selected need-input stop.
  141|       |    ///
  142|       |    /// # Parameters
  143|       |    ///
  144|       |    /// - `required`: Current minimum total source units required before
  145|       |    ///   retrying from the current input position. A later retry may raise this
  146|       |    ///   lower bound.
  147|       |    /// # Returns
  148|       |    ///
  149|       |    /// Returns progress at the current decode cursor.
  150|       |    #[inline]
  151|      7|    pub(in crate::transcode) fn need_input_progress_with(&self, required: NonZeroUsize) -> TranscodeProgress {
  152|      7|        self.state.need_input_progress(required)
  153|      7|    }
  154|       |
  155|       |    /// Advances the input cursor without emitted output.
  156|       |    ///
  157|       |    /// # Parameters
  158|       |    ///
  159|       |    /// - `read`: Non-zero input units consumed by the current operation.
  160|       |    ///
  161|       |    /// # Panics
  162|       |    ///
  163|       |    /// Panics when `read` exceeds the input units available at the current
  164|       |    /// cursor.
  165|       |    #[inline]
  166|      5|    pub(in crate::transcode) fn skip(&mut self, read: NonZeroUsize) {
  167|      5|        let read = read.get();
  168|      5|        assert!(
  169|      5|            read <= self.available(),
  170|       |            "DecodeOutcome::Skipped read beyond available input",
  171|       |        );
  172|      5|        self.state.advance_input(read);
  173|      5|    }
  174|       |
  175|       |    /// Accepts emitted output and advances both cursors.
  176|       |    ///
  177|       |    /// # Parameters
  178|       |    ///
  179|       |    /// - `read`: Input units consumed by this decode outcome.
  180|       |    /// - `emitted`: Logical output values emitted by this decode outcome.
  181|       |    ///
  182|       |    /// # Panics
  183|       |    ///
  184|       |    /// Panics when `read` exceeds available input or `emitted` exceeds
  185|       |    /// available output.
  186|       |    #[inline]
  187|     44|    pub(in crate::transcode) fn accept_emitted(&mut self, read: NonZeroUsize, emitted: NonZeroUsize) {
  188|     44|        let read = read.get();
  189|     44|        let emitted = emitted.get();
  190|     44|        assert!(
  191|     44|            read <= self.available(),
  192|       |            "DecodeOutcome::Emitted read beyond available input",
  193|       |        );
  194|     44|        assert!(
  195|     44|            emitted <= self.state.available_output(),
  196|       |            "DecodeOutcome::Emitted exceeded available output",
  197|       |        );
  198|     44|        self.state.advance(read, emitted);
  199|     44|    }
  200|       |
  201|       |    /// Applies one decode outcome to this decode state.
  202|       |    ///
  203|       |    /// # Parameters
  204|       |    ///
  205|       |    /// - `outcome`: Decode outcome produced by the decode engine.
  206|       |    ///
  207|       |    /// # Returns
  208|       |    ///
  209|       |    /// Returns `Some(progress)` when more input is required and this call must
  210|       |    /// stop, or `None` after applying emitted/skipped progress so decoding can
  211|       |    /// continue.
  212|       |    ///
  213|       |    /// # Panics
  214|       |    ///
  215|       |    /// Panics when an emitted or skipped outcome exceeds the input or output
  216|       |    /// units available at the current cursors.
  217|       |    #[inline]
  218|     52|    pub(in crate::transcode) fn apply_decode_outcome(&mut self, outcome: DecodeOutcome) -> Option<TranscodeProgress> {
  219|     52|        match outcome {
  220|     44|            DecodeOutcome::Emitted { read, emitted } => {
  221|     44|                self.accept_emitted(read, emitted);
  222|     44|                None
  223|       |            }
  224|      5|            DecodeOutcome::Skipped { read } => {
  225|      5|                self.skip(read);
  226|      5|                None
  227|       |            }
  228|      3|            DecodeOutcome::NeedInput { required } => Some(self.need_input_progress_with(required)),
  229|       |        }
  230|     52|    }
  231|       |
  232|       |    /// Returns input units visible from the current input cursor.
  233|       |    ///
  234|       |    /// # Returns
  235|       |    ///
  236|       |    /// The remaining readable unit count, without advancing the cursor.
  237|       |    #[inline]
  238|       |    #[must_use]
  239|    132|    fn available(&self) -> usize {
  240|    132|        self.state.available_input()
  241|    132|    }
  242|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/internal/encode_attempt.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Borrowed state for one value-level encode attempt.
    9|       |
   10|       |use super::super::engine::EncodeContext;
   11|       |
   12|       |/// Engine-owned mutable state for one encode attempt.
   13|       |///
   14|       |/// Borrows the input and exclusively borrows output for `'a`; creating this
   15|       |/// state does not encode a value or validate the supplied indices.
   16|       |///
   17|       |/// # Type Parameters
   18|       |///
   19|       |/// - `Value`: Logical input value type.
   20|       |/// - `Unit`: Encoded output unit type.
   21|       |pub(in crate::transcode) struct EncodeAttempt<'a, Value, Unit> {
   22|       |    /// Input value being encoded.
   23|       |    value: &'a Value,
   24|       |    /// Absolute input index of `value`.
   25|       |    input_index: usize,
   26|       |    /// Complete mutable output slice owned by the engine.
   27|       |    output: &'a mut [Unit],
   28|       |    /// Absolute output index where writing begins.
   29|       |    output_index: usize,
   30|       |}
   31|       |
   32|       |impl<'a, Value, Unit> EncodeAttempt<'a, Value, Unit> {
   33|       |    /// Creates an engine-owned encode attempt.
   34|       |    ///
   35|       |    /// # Parameters
   36|       |    ///
   37|       |    /// - `value`: Input borrowed for the attempt lifetime `'a`.
   38|       |    /// - `input_index`: Absolute position of this input value.
   39|       |    /// - `output`: Complete output slice exclusively borrowed for `'a`.
   40|       |    /// - `output_index`: First writable offset in `output`, retained unchecked.
   41|       |    ///
   42|       |    /// # Returns
   43|       |    ///
   44|       |    /// A borrowed state holder without encoding or allocating output.
   45|       |    #[must_use]
   46|       |    #[inline]
   47|  4.90k|    pub(in crate::transcode) fn new(
   48|  4.90k|        value: &'a Value,
   49|  4.90k|        input_index: usize,
   50|  4.90k|        output: &'a mut [Unit],
   51|  4.90k|        output_index: usize,
   52|  4.90k|    ) -> Self {
   53|  4.90k|        Self {
   54|  4.90k|            value,
   55|  4.90k|            input_index,
   56|  4.90k|            output,
   57|  4.90k|            output_index,
   58|  4.90k|        }
   59|  4.90k|    }
   60|       |
   61|       |    /// Returns the input value being encoded.
   62|       |    ///
   63|       |    /// # Returns
   64|       |    ///
   65|       |    /// The original input borrowed through this attempt, without cloning it.
   66|       |    #[must_use = "the borrowed input value should be inspected"]
   67|       |    #[inline]
   68|  3.64k|    pub(in crate::transcode) fn value(&self) -> &Value {
   69|  3.64k|        self.value
   70|  3.64k|    }
   71|       |
   72|       |    /// Returns the absolute input index.
   73|       |    ///
   74|       |    /// # Returns
   75|       |    ///
   76|       |    /// The input position supplied to the constructor, unchanged by this
   77|       |    /// holder.
   78|       |    #[must_use]
   79|       |    #[inline]
   80|      6|    pub(in crate::transcode) const fn input_index(&self) -> usize {
   81|      6|        self.input_index
   82|      6|    }
   83|       |
   84|       |    /// Returns writable output capacity.
   85|       |    ///
   86|       |    /// # Returns
   87|       |    ///
   88|       |    /// The units remaining after the output offset, or zero if it exceeds the
   89|       |    /// output length. This query does not modify the output slice.
   90|       |    #[must_use]
   91|       |    #[inline]
   92|  11.3k|    pub(in crate::transcode) fn available_output(&self) -> usize {
   93|  11.3k|        self.output.len().saturating_sub(self.output_index)
   94|  11.3k|    }
   95|       |
   96|       |    /// Returns the read-only policy view for this attempt.
   97|       |    ///
   98|       |    /// # Returns
   99|       |    ///
  100|       |    /// A context borrowing the input while this attempt is borrowed, carrying
  101|       |    /// both absolute indices and the current saturating output capacity.
  102|       |    #[must_use]
  103|       |    #[inline]
  104|  5.95k|    pub(in crate::transcode) fn context(&self) -> EncodeContext<'_, Value> {
  105|  5.95k|        EncodeContext::new(self.value, self.input_index, self.output_index, self.available_output())
  106|  5.95k|    }
  107|       |
  108|       |    /// Returns all mutable engine state for the encode operation.
  109|       |    ///
  110|       |    /// # Returns
  111|       |    ///
  112|       |    /// The original input reference, input index, exclusive output slice and
  113|       |    /// output index, in that order. Consumes this holder and transfers its
  114|       |    /// original `'a` borrows without encoding or allocating.
  115|       |    #[must_use = "the encode attempt parts must be used"]
  116|       |    #[inline]
  117|  3.40k|    pub(in crate::transcode) fn into_parts(self) -> (&'a Value, usize, &'a mut [Unit], usize) {
  118|  3.40k|        (self.value, self.input_index, self.output, self.output_index)
  119|  3.40k|    }
  120|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/internal/encode_state.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Mutable state for one buffered encode call.
    9|       |
   10|       |use core::num::NonZeroUsize;
   11|       |
   12|       |use super::super::engine::EncodeOutcome;
   13|       |use super::super::transcode_progress::TranscodeProgress;
   14|       |use super::encode_attempt::EncodeAttempt;
   15|       |use super::transcode_state::TranscodeState;
   16|       |
   17|       |/// Tracks borrowed input and output cursors for one buffered encode call.
   18|       |///
   19|       |/// # Type Parameters
   20|       |///
   21|       |/// - `'a`: Lifetime of the input and exclusively borrowed output buffers.
   22|       |/// - `Value`: Logical values read from input.
   23|       |/// - `Unit`: Representation units written to output.
   24|       |pub(in crate::transcode) struct EncodeState<'a, Value, Unit> {
   25|       |    /// Shared input/output state for this encode call.
   26|       |    state: TranscodeState<'a, Value, Unit>,
   27|       |}
   28|       |
   29|       |impl<'a, Value, Unit> EncodeState<'a, Value, Unit> {
   30|       |    /// Creates mutable encode state.
   31|       |    ///
   32|       |    /// # Parameters
   33|       |    ///
   34|       |    /// - `input`: Complete input value slice visible to the encoder.
   35|       |    /// - `input_index`: Absolute input value index where encoding starts.
   36|       |    /// - `output`: Complete output unit slice visible to the encoder.
   37|       |    /// - `output_index`: Absolute output unit index where writing starts.
   38|       |    ///
   39|       |    /// # Returns
   40|       |    ///
   41|       |    /// Returns initialized encode state with cursors at the requested start
   42|       |    /// positions.
   43|       |    #[must_use]
   44|       |    #[inline]
   45|     42|    pub(in crate::transcode) fn new(
   46|     42|        input: &'a [Value],
   47|     42|        input_index: usize,
   48|     42|        output: &'a mut [Unit],
   49|     42|        output_index: usize,
   50|     42|    ) -> Self {
   51|     42|        Self {
   52|     42|            state: TranscodeState::new(input, input_index, output, output_index),
   53|     42|        }
   54|     42|    }
   55|       |
   56|       |    /// Returns whether there is still input to encode.
   57|       |    ///
   58|       |    /// # Returns
   59|       |    ///
   60|       |    /// Returns `true` when more input values remain.
   61|       |    #[must_use]
   62|       |    #[inline]
   63|     70|    pub(in crate::transcode) fn has_input(&self) -> bool {
   64|     70|        self.state.has_input()
   65|     70|    }
   66|       |
   67|       |    /// Returns an encode context borrowing the current value and output.
   68|       |    ///
   69|       |    /// # Returns
   70|       |    ///
   71|       |    /// An attempt borrowing the current value and the output buffer until the
   72|       |    /// mutable borrow of this state ends. The cursors are not advanced.
   73|       |    ///
   74|       |    /// # Safety
   75|       |    ///
   76|       |    /// The caller must guarantee that `self.has_input()` returned `true`.
   77|       |    #[must_use]
   78|       |    #[inline]
   79|     54|    pub(in crate::transcode) unsafe fn context_unchecked(&mut self) -> EncodeAttempt<'_, Value, Unit> {
   80|     54|        let input_index = self.state.input_cursor();
   81|     54|        let output_index = self.state.output_cursor();
   82|     54|        let (input, output) = self.state.input_output_mut();
   83|       |        // SAFETY: Guaranteed by the caller.
   84|     54|        let value = unsafe { input.get_unchecked(input_index) };
   85|     54|        EncodeAttempt::new(value, input_index, output, output_index)
   86|     54|    }
   87|       |
   88|       |    /// Returns completed progress for the current cursors.
   89|       |    ///
   90|       |    /// # Returns
   91|       |    ///
   92|       |    /// Returns a completed [`TranscodeProgress`] with consumed input and output
   93|       |    /// counters.
   94|       |    #[inline]
   95|     16|    pub(in crate::transcode) fn complete_progress(&self) -> TranscodeProgress {
   96|     16|        self.state.complete_progress()
   97|     16|    }
   98|       |
   99|       |    /// Returns progress for a known missing output capacity.
  100|       |    ///
  101|       |    /// # Parameters
  102|       |    ///
  103|       |    /// - `required`: Total output units required from the current output
  104|       |    ///   position.
  105|       |    ///
  106|       |    /// # Returns
  107|       |    ///
  108|       |    /// Returns [`TranscodeProgress::need_output`] with missing-capacity
  109|       |    /// counters.
  110|       |    #[inline]
  111|     16|    pub(in crate::transcode) fn need_output_progress_with(&self, required: NonZeroUsize) -> TranscodeProgress {
  112|     16|        self.state.need_output_progress(required)
  113|     16|    }
  114|       |
  115|       |    /// Accepts a completed one-value write and advances both cursors.
  116|       |    ///
  117|       |    /// # Parameters
  118|       |    ///
  119|       |    /// - `written`: Output units written by the last encode call.
  120|       |    ///
  121|       |    /// # Returns
  122|       |    ///
  123|       |    /// Returns unit `()`, while advancing `input_cursor` and `output_cursor`.
  124|       |    ///
  125|       |    /// # Panics
  126|       |    ///
  127|       |    /// Panics if `written` exceeds the remaining output capacity.
  128|       |    #[inline]
  129|     28|    pub(in crate::transcode) fn accept_written_value(&mut self, written: usize) {
  130|     28|        assert!(
  131|     28|            written <= self.available_output(),
  132|       |            "EncodeOutcome::Consumed wrote beyond available output",
  133|       |        );
  134|     28|        self.state.advance(1, written);
  135|     28|    }
  136|       |
  137|       |    /// Applies one encode outcome to this encode state.
  138|       |    ///
  139|       |    /// # Parameters
  140|       |    ///
  141|       |    /// - `outcome`: Encode outcome produced by the encode engine.
  142|       |    ///
  143|       |    /// # Returns
  144|       |    ///
  145|       |    /// Returns stop progress when output is insufficient, otherwise `None`.
  146|       |    ///
  147|       |    /// # Panics
  148|       |    ///
  149|       |    /// Panics if a consumed outcome writes beyond remaining capacity, or a
  150|       |    /// need-output outcome does not request more than the available capacity.
  151|       |    #[inline]
  152|     44|    pub(in crate::transcode) fn apply_encode_outcome(&mut self, outcome: EncodeOutcome) -> Option<TranscodeProgress> {
  153|     44|        match outcome {
  154|     28|            EncodeOutcome::Consumed { written } => {
  155|     28|                self.accept_written_value(written);
  156|     28|                None
  157|       |            }
  158|     16|            EncodeOutcome::NeedOutput { required } => {
  159|     16|                let available = self.available_output();
  160|     16|                assert!(
  161|     16|                    required.get() > available,
  162|       |                    "EncodeOutcome::NeedOutput required capacity must exceed available output",
  163|       |                );
  164|     16|                Some(self.need_output_progress_with(required))
  165|       |            }
  166|       |        }
  167|     44|    }
  168|       |
  169|       |    /// Returns the number of writable output units from the current cursor.
  170|       |    ///
  171|       |    /// # Returns
  172|       |    ///
  173|       |    /// Returns writable output capacity from the current output cursor.
  174|       |    #[must_use]
  175|       |    #[inline]
  176|     44|    fn available_output(&self) -> usize {
  177|     44|        self.state.available_output()
  178|     44|    }
  179|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/internal/lifecycle_guard.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Lifecycle guard shared by transcode engines.
    9|       |//!
   10|       |//! [`Transcoder`](crate::Transcoder) documents a lifecycle of
   11|       |//! `reset → transcode* → finish` and then `reset` again before reusing the
   12|       |//! instance for another logical stream. The trait itself does not enforce
   13|       |//! this; engines historically rely on caller discipline. `LifecycleGuard`
   14|       |//! rejects common misuse (calling `transcode` or `finish` before `reset`,
   15|       |//! calling `transcode` after `finish` without an intervening `reset`, or
   16|       |//! calling `finish` twice in a row) in every build profile. It also prevents
   17|       |//! callers from continuing after a reset or finish operation failed after
   18|       |//! potentially mutating codec or hook state.
   19|       |
   20|       |use super::lifecycle_phase::LifecyclePhase;
   21|       |use crate::TranscodeFailure;
   22|       |
   23|       |/// Lifecycle guard for transcode engines.
   24|       |///
   25|       |/// Lifecycle rules:
   26|       |///
   27|       |/// - `transcode` is rejected when the engine is `Uninitialized`, `Finished`, or
   28|       |///   `Poisoned`.
   29|       |/// - `finish` is rejected when the engine is `Uninitialized`, `Finished`, or
   30|       |///   `Poisoned`.
   31|       |/// - reset preflight never changes the phase.
   32|       |/// - starting reset or finish marks the engine `Poisoned`; success commits the
   33|       |///   corresponding `Fresh` or `Finished` phase.
   34|       |#[derive(Debug, Default)]
   35|       |pub(crate) struct LifecycleGuard {
   36|       |    /// Current lifecycle phase.
   37|       |    phase: LifecyclePhase,
   38|       |}
   39|       |
   40|       |impl LifecycleGuard {
   41|       |    /// Creates a guard in the [`LifecyclePhase::Uninitialized`] phase.
   42|       |    ///
   43|       |    /// # Returns
   44|       |    ///
   45|       |    /// Returns a guard ready to observe the first lifecycle event.
   46|       |    #[inline]
   47|       |    #[must_use]
   48|  3.83k|    pub(crate) const fn new() -> Self {
   49|  3.83k|        Self {
   50|  3.83k|            phase: LifecyclePhase::Uninitialized,
   51|  3.83k|        }
   52|  3.83k|    }
   53|       |
   54|       |    /// Records that reset execution is starting and state may be mutated.
   55|       |    ///
   56|       |    /// Reset is legal from every phase. The guard remains poisoned unless the
   57|       |    /// caller later invokes [`Self::on_reset_success`].
   58|       |    #[inline]
   59|  3.82k|    pub(crate) fn on_reset_start(&mut self) {
   60|  3.82k|        self.phase = LifecyclePhase::Poisoned;
   61|  3.82k|    }
   62|       |
   63|       |    /// Commits the [`LifecyclePhase::Fresh`] phase after reset succeeds.
   64|       |    #[inline]
   65|  3.80k|    pub(crate) fn on_reset_success(&mut self) {
   66|  3.80k|        self.phase = LifecyclePhase::Fresh;
   67|  3.80k|    }
   68|       |
   69|       |    /// Records a `transcode` entry.
   70|       |    ///
   71|       |    /// # Returns
   72|       |    ///
   73|       |    /// Returns `Ok(())` after accepting the entry and entering streaming state.
   74|       |    ///
   75|       |    /// # Errors
   76|       |    ///
   77|       |    /// Returns [`TranscodeFailure::TranscodeAfterFinish`] when `transcode` is
   78|       |    /// called after `finish` without an intervening reset. Returns
   79|       |    /// [`TranscodeFailure::TranscodeBeforeReset`] when called before the first
   80|       |    /// successful reset, or
   81|       |    /// [`TranscodeFailure::LifecyclePoisoned`] after partial reset or finish
   82|       |    /// failure.
   83|       |    #[inline]
   84|  4.08k|    pub(crate) fn on_transcode(&mut self) -> Result<(), TranscodeFailure> {
   85|  4.08k|        match self.phase {
   86|       |            LifecyclePhase::Uninitialized => {
   87|      3|                return Err(TranscodeFailure::TranscodeBeforeReset);
   88|       |            }
   89|       |            LifecyclePhase::Finished => {
   90|      6|                return Err(TranscodeFailure::TranscodeAfterFinish);
   91|       |            }
   92|       |            LifecyclePhase::Poisoned => {
   93|      6|                return Err(TranscodeFailure::LifecyclePoisoned);
   94|       |            }
   95|  4.06k|            LifecyclePhase::Fresh | LifecyclePhase::Streaming => {}
   96|       |        }
   97|  4.06k|        if self.phase == LifecyclePhase::Fresh {
   98|  1.27k|            self.phase = LifecyclePhase::Streaming;
   99|  2.79k|        }
  100|  4.06k|        Ok(())
  101|  4.08k|    }
  102|       |
  103|       |    /// Validates that the guard may enter `finish`. Does not change
  104|       |    /// state, so callers that fail before completing finish (for example,
  105|       |    /// capacity checks rejecting the supplied output) can retry without
  106|       |    /// being marked closed.
  107|       |    ///
  108|       |    /// # Returns
  109|       |    ///
  110|       |    /// Returns `Ok(())` when finish may begin; the phase remains unchanged.
  111|       |    ///
  112|       |    /// # Errors
  113|       |    ///
  114|       |    /// Returns [`TranscodeFailure::FinishAfterFinish`] when `finish` is called
  115|       |    /// twice without an intervening reset. Returns
  116|       |    /// [`TranscodeFailure::FinishBeforeReset`] when called before the first
  117|       |    /// successful reset, or
  118|       |    /// [`TranscodeFailure::LifecyclePoisoned`] after partial reset or finish
  119|       |    /// failure.
  120|       |    #[inline]
  121|  3.54k|    pub(crate) fn on_finish_attempt(&self) -> Result<(), TranscodeFailure> {
  122|  3.54k|        match self.phase {
  123|       |            LifecyclePhase::Uninitialized => {
  124|      1|                return Err(TranscodeFailure::FinishBeforeReset);
  125|       |            }
  126|       |            LifecyclePhase::Finished => {
  127|      3|                return Err(TranscodeFailure::FinishAfterFinish);
  128|       |            }
  129|       |            LifecyclePhase::Poisoned => {
  130|      4|                return Err(TranscodeFailure::LifecyclePoisoned);
  131|       |            }
  132|  3.53k|            LifecyclePhase::Fresh | LifecyclePhase::Streaming => {}
  133|       |        }
  134|  3.53k|        Ok(())
  135|  3.54k|    }
  136|       |
  137|       |    /// Records that finish execution is starting and state may be mutated.
  138|       |    ///
  139|       |    /// The guard remains poisoned unless the caller later invokes
  140|       |    /// [`Self::on_finish_success`].
  141|       |    #[inline]
  142|  3.52k|    pub(crate) fn on_finish_start(&mut self) {
  143|  3.52k|        self.phase = LifecyclePhase::Poisoned;
  144|  3.52k|    }
  145|       |
  146|       |    /// Commits the `Finished` state after `finish` actually completed. Call
  147|       |    /// only on the success path.
  148|       |    #[inline]
  149|  3.48k|    pub(crate) fn on_finish_success(&mut self) {
  150|  3.48k|        self.phase = LifecyclePhase::Finished;
  151|  3.48k|    }
  152|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/internal/pending_value.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Pending decoded value retained by buffered converters.
    9|       |
   10|       |/// Decoded value retained after source input has been consumed.
   11|       |///
   12|       |/// # Type Parameters
   13|       |///
   14|       |/// - `Value`: Owned decoded value retained until downstream encoding succeeds.
   15|       |#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
   16|       |pub(in crate::transcode) struct PendingValue<Value> {
   17|       |    /// Decoded logical value.
   18|       |    value: Value,
   19|       |    /// Source input index that produced this value.
   20|       |    input_index: usize,
   21|       |}
   22|       |
   23|       |impl<Value> PendingValue<Value> {
   24|       |    /// Creates a retained decoded value.
   25|       |    ///
   26|       |    /// # Parameters
   27|       |    ///
   28|       |    /// - `value`: Decoded logical value.
   29|       |    /// - `input_index`: Source input index that produced the value.
   30|       |    ///
   31|       |    /// # Returns
   32|       |    ///
   33|       |    /// Returns pending-value state owned by the converter engine.
   34|       |    #[inline]
   35|       |    #[must_use]
   36|  3.90k|    pub(in crate::transcode) const fn new(value: Value, input_index: usize) -> Self {
   37|  3.90k|        Self { value, input_index }
   38|  3.90k|    }
   39|       |
   40|       |    /// Returns the source input index that produced this value.
   41|       |    ///
   42|       |    /// # Returns
   43|       |    ///
   44|       |    /// Returns the absolute source input index used for downstream encode
   45|       |    /// errors.
   46|       |    #[inline]
   47|       |    #[must_use]
   48|  4.85k|    pub(in crate::transcode) const fn input_index(&self) -> usize {
   49|  4.85k|        self.input_index
   50|  4.85k|    }
   51|       |
   52|       |    /// Returns the decoded value.
   53|       |    ///
   54|       |    /// # Returns
   55|       |    ///
   56|       |    /// Returns the retained decoded value by shared reference.
   57|       |    #[inline]
   58|       |    #[must_use = "the decoded value must be inspected before encoding"]
   59|  4.85k|    pub(in crate::transcode) const fn value(&self) -> &Value {
   60|  4.85k|        &self.value
   61|  4.85k|    }
   62|       |
   63|       |    /// Consumes the pending state and returns the decoded value.
   64|       |    ///
   65|       |    /// # Returns
   66|       |    ///
   67|       |    /// Returns the owned decoded value.
   68|       |    #[inline]
   69|       |    #[must_use = "the decoded value must be retained for downstream encoding"]
   70|      8|    pub(in crate::transcode) fn into_value(self) -> Value {
   71|      8|        self.value
   72|      8|    }
   73|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/internal/pending_value_slot.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Slot that owns the converter's retained decoded value.
    9|       |
   10|       |use super::super::engine::TranscodeEncodeEngine;
   11|       |use super::super::engine::TranscodeEncodeHooks;
   12|       |use super::pending_value::PendingValue;
   13|       |use crate::CapacityError;
   14|       |use crate::Codec;
   15|       |
   16|       |/// Slot that owns the converter's retained decoded value.
   17|       |///
   18|       |/// # Type Parameters
   19|       |///
   20|       |/// - `Value`: Decoded value retained until the target encoder has capacity.
   21|       |#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
   22|       |pub(in crate::transcode) struct PendingValueSlot<Value> {
   23|       |    /// Retained decoded value waiting for output capacity.
   24|       |    value: Option<PendingValue<Value>>,
   25|       |}
   26|       |
   27|       |impl<Value> PendingValueSlot<Value> {
   28|       |    /// Creates an empty pending-value slot.
   29|       |    ///
   30|       |    /// # Returns
   31|       |    ///
   32|       |    /// Returns a slot with no retained value.
   33|       |    #[must_use]
   34|       |    #[inline]
   35|  1.22k|    pub(in crate::transcode) const fn empty() -> Self {
   36|  1.22k|        Self { value: None }
   37|  1.22k|    }
   38|       |
   39|       |    /// Returns the current target-output bound for the retained value.
   40|       |    ///
   41|       |    /// Unlike public transcoder capacity methods, this helper intentionally
   42|       |    /// examines transient state so a concrete finish call can accept the
   43|       |    /// capacity required by its current pending value.
   44|       |    ///
   45|       |    /// # Type Parameters
   46|       |    ///
   47|       |    /// - `E`: Encoder codec type used to query output bounds.
   48|       |    /// - `H`: Encoder hook type used by the encoder engine.
   49|       |    ///
   50|       |    /// # Parameters
   51|       |    ///
   52|       |    /// - `engine`: Target encode engine for one-value output planning.
   53|       |    ///
   54|       |    /// # Returns
   55|       |    ///
   56|       |    /// Returns zero for an empty slot, or the encoder bound for one value.
   57|       |    ///
   58|       |    /// # Errors
   59|       |    ///
   60|       |    /// Returns a capacity error when encoder planning overflows.
   61|       |    #[inline]
   62|  1.17k|    pub(in crate::transcode) fn current_output_len<E, H>(
   63|  1.17k|        &self,
   64|  1.17k|        engine: &TranscodeEncodeEngine<E, H>,
   65|  1.17k|    ) -> Result<usize, CapacityError>
   66|  1.17k|    where
   67|  1.17k|        E: Codec<Value = Value>,
   68|  1.17k|        H: TranscodeEncodeHooks<E>,
   69|       |    {
   70|  1.17k|        if self.value.is_some() {
   71|      9|            engine.max_transcode_output_len(1)
   72|       |        } else {
   73|  1.16k|            Ok(0)
   74|       |        }
   75|  1.17k|    }
   76|       |
   77|       |    /// Stores a decoded value that could not be encoded yet.
   78|       |    ///
   79|       |    /// # Parameters
   80|       |    ///
   81|       |    /// - `pending`: Decoded value and its source input position.
   82|       |    #[inline]
   83|  4.85k|    pub(in crate::transcode) fn put(&mut self, pending: PendingValue<Value>) {
   84|  4.85k|        self.value = Some(pending);
   85|  4.85k|    }
   86|       |
   87|       |    /// Takes the retained decoded value, if any.
   88|       |    ///
   89|       |    /// # Returns
   90|       |    ///
   91|       |    /// Returns the retained value when present, otherwise `None`.
   92|       |    #[inline]
   93|  9.53k|    pub(in crate::transcode) fn take(&mut self) -> Option<PendingValue<Value>> {
   94|  9.53k|        self.value.take()
   95|  9.53k|    }
   96|       |
   97|       |    /// Removes any retained decoded value.
   98|       |    ///
   99|       |    /// # Returns
  100|       |    ///
  101|       |    /// Returns unit `()`.
  102|       |    #[inline]
  103|  1.22k|    pub(in crate::transcode) fn clear(&mut self) {
  104|  1.22k|        self.value = None;
  105|  1.22k|    }
  106|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/internal/transcode_state.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Shared input/output state for one transcode call.
    9|       |
   10|       |use core::num::NonZeroUsize;
   11|       |
   12|       |use super::super::transcode_progress::TranscodeProgress;
   13|       |
   14|       |/// Shared input/output state for one transcode call.
   15|       |///
   16|       |/// `TranscodeState` borrows the input/output slices visible to a single
   17|       |/// buffered transcode call and tracks absolute input/output cursors. Concrete
   18|       |/// encode/decode/convert states wrap it and keep domain-specific operations in
   19|       |/// their own types.
   20|       |///
   21|       |/// Callers must keep input cursors within the input slice and prevent cursor
   22|       |/// arithmetic from overflowing. An output cursor beyond the slice is tolerated
   23|       |/// by capacity queries, which report zero remaining capacity.
   24|       |///
   25|       |/// # Type Parameters
   26|       |///
   27|       |/// - `'a`: Lifetime of the shared input and exclusive output borrows.
   28|       |/// - `Input`: Source unit type read by the current transcode call.
   29|       |/// - `Output`: Target unit type written by the current transcode call.
   30|       |pub(in crate::transcode) struct TranscodeState<'a, Input, Output> {
   31|       |    /// Complete input slice visible to the current call.
   32|       |    input: &'a [Input],
   33|       |    /// Complete output slice visible to the current call.
   34|       |    output: &'a mut [Output],
   35|       |    /// Absolute input index where this call starts.
   36|       |    input_start: usize,
   37|       |    /// Absolute output index where this call starts.
   38|       |    output_start: usize,
   39|       |    /// Absolute input index for the next operation.
   40|       |    input_cursor: usize,
   41|       |    /// Absolute output index for the next write.
   42|       |    output_cursor: usize,
   43|       |}
   44|       |
   45|       |impl<'a, Input, Output> TranscodeState<'a, Input, Output> {
   46|       |    /// Creates shared transcode state.
   47|       |    ///
   48|       |    /// # Parameters
   49|       |    ///
   50|       |    /// - `input`: Complete input slice visible to the current call.
   51|       |    /// - `input_index`: Absolute input index where this call starts.
   52|       |    /// - `output`: Complete output slice visible to the current call.
   53|       |    /// - `output_index`: Absolute output index where this call starts.
   54|       |    ///
   55|       |    /// # Returns
   56|       |    ///
   57|       |    /// Returns initialized state with both cursors at their call starts.
   58|       |    ///
   59|       |    /// # Panics
   60|       |    ///
   61|       |    /// Panics in debug builds if `input_index` exceeds `input.len()`. Callers
   62|       |    /// must uphold this input bound in all builds.
   63|       |    #[inline]
   64|       |    #[must_use]
   65|  6.44k|    pub(in crate::transcode) fn new(
   66|  6.44k|        input: &'a [Input],
   67|  6.44k|        input_index: usize,
   68|  6.44k|        output: &'a mut [Output],
   69|  6.44k|        output_index: usize,
   70|  6.44k|    ) -> Self {
   71|  6.44k|        debug_assert!(input_index <= input.len(), "input index must be within the input slice",);
   72|       |
   73|  6.44k|        Self {
   74|  6.44k|            input,
   75|  6.44k|            output,
   76|  6.44k|            input_start: input_index,
   77|  6.44k|            output_start: output_index,
   78|  6.44k|            input_cursor: input_index,
   79|  6.44k|            output_cursor: output_index,
   80|  6.44k|        }
   81|  6.44k|    }
   82|       |
   83|       |    /// Returns the complete input slice.
   84|       |    ///
   85|       |    /// # Returns
   86|       |    ///
   87|       |    /// Returns the full input slice visible to this call.
   88|       |    #[inline]
   89|       |    #[must_use]
   90|  4.43k|    pub(in crate::transcode) fn input(&self) -> &[Input] {
   91|  4.43k|        self.input
   92|  4.43k|    }
   93|       |
   94|       |    /// Returns the complete output slice mutably.
   95|       |    ///
   96|       |    /// # Returns
   97|       |    ///
   98|       |    /// Returns the full mutable output slice visible to this call.
   99|       |    #[inline]
  100|       |    #[must_use]
  101|  7.30k|    pub(in crate::transcode) fn output_mut(&mut self) -> &mut [Output] {
  102|  7.30k|        self.output
  103|  7.30k|    }
  104|       |
  105|       |    /// Returns input and output slices together.
  106|       |    ///
  107|       |    /// # Returns
  108|       |    ///
  109|       |    /// Returns the immutable input slice and mutable output slice. This helper
  110|       |    /// lets callers borrow the two disjoint fields at the same time.
  111|       |    #[inline]
  112|       |    #[must_use]
  113|     54|    pub(in crate::transcode) fn input_output_mut(&mut self) -> (&[Input], &mut [Output]) {
  114|     54|        (self.input, self.output)
  115|     54|    }
  116|       |
  117|       |    /// Returns the absolute input start index.
  118|       |    ///
  119|       |    /// # Returns
  120|       |    ///
  121|       |    /// Returns the input index recorded when this call began.
  122|       |    #[inline]
  123|       |    #[must_use]
  124|  4.51k|    pub(in crate::transcode) const fn input_start(&self) -> usize {
  125|  4.51k|        self.input_start
  126|  4.51k|    }
  127|       |
  128|       |    /// Returns the absolute output start index.
  129|       |    ///
  130|       |    /// # Returns
  131|       |    ///
  132|       |    /// Returns the output index recorded when this call began.
  133|       |    #[inline]
  134|       |    #[must_use]
  135|  4.51k|    pub(in crate::transcode) const fn output_start(&self) -> usize {
  136|  4.51k|        self.output_start
  137|  4.51k|    }
  138|       |
  139|       |    /// Returns the current absolute input cursor.
  140|       |    ///
  141|       |    /// # Returns
  142|       |    ///
  143|       |    /// Returns the absolute input index for the next read or consume operation.
  144|       |    #[inline]
  145|       |    #[must_use]
  146|  4.56k|    pub(in crate::transcode) const fn input_cursor(&self) -> usize {
  147|  4.56k|        self.input_cursor
  148|  4.56k|    }
  149|       |
  150|       |    /// Returns the current absolute output cursor.
  151|       |    ///
  152|       |    /// # Returns
  153|       |    ///
  154|       |    /// Returns the absolute output index for the next write operation.
  155|       |    #[inline]
  156|       |    #[must_use]
  157|  11.8k|    pub(in crate::transcode) const fn output_cursor(&self) -> usize {
  158|  11.8k|        self.output_cursor
  159|  11.8k|    }
  160|       |
  161|       |    /// Returns whether input remains.
  162|       |    ///
  163|       |    /// # Returns
  164|       |    ///
  165|       |    /// Returns `true` when the input cursor has not reached the input end.
  166|       |    #[inline]
  167|       |    #[must_use]
  168|  7.59k|    pub(in crate::transcode) fn has_input(&self) -> bool {
  169|  7.59k|        self.input_cursor < self.input.len()
  170|  7.59k|    }
  171|       |
  172|       |    /// Returns whether output has no room for another value.
  173|       |    ///
  174|       |    /// # Returns
  175|       |    ///
  176|       |    /// Returns `true` when the output cursor is at the output end.
  177|       |    #[inline]
  178|       |    #[must_use]
  179|     79|    pub(in crate::transcode) fn needs_output(&self) -> bool {
  180|     79|        self.output_cursor == self.output.len()
  181|     79|    }
  182|       |
  183|       |    /// Returns input units visible from the current input cursor.
  184|       |    ///
  185|       |    /// # Returns
  186|       |    ///
  187|       |    /// Returns remaining input units visible from `input_cursor`.
  188|       |    #[inline]
  189|       |    #[must_use]
  190|  14.5k|    pub(in crate::transcode) fn available_input(&self) -> usize {
  191|  14.5k|        self.input.len() - self.input_cursor
  192|  14.5k|    }
  193|       |
  194|       |    /// Returns output units writable from the current output cursor.
  195|       |    ///
  196|       |    /// # Returns
  197|       |    ///
  198|       |    /// Returns remaining output capacity from `output_cursor`.
  199|       |    #[inline]
  200|       |    #[must_use]
  201|  7.31k|    pub(in crate::transcode) fn available_output(&self) -> usize {
  202|  7.31k|        self.output.len().saturating_sub(self.output_cursor)
  203|  7.31k|    }
  204|       |
  205|       |    /// Returns input units consumed since this call started.
  206|       |    ///
  207|       |    /// # Returns
  208|       |    ///
  209|       |    /// Returns consumed input units relative to `input_start`.
  210|       |    #[inline]
  211|       |    #[must_use]
  212|  11.9k|    pub(in crate::transcode) const fn read(&self) -> usize {
  213|  11.9k|        self.input_cursor - self.input_start
  214|  11.9k|    }
  215|       |
  216|       |    /// Returns output units written since this call started.
  217|       |    ///
  218|       |    /// # Returns
  219|       |    ///
  220|       |    /// Returns written output units relative to `output_start`.
  221|       |    #[inline]
  222|       |    #[must_use]
  223|  6.39k|    pub(in crate::transcode) const fn written(&self) -> usize {
  224|  6.39k|        self.output_cursor - self.output_start
  225|  6.39k|    }
  226|       |
  227|       |    /// Returns completed progress for the current cursors.
  228|       |    ///
  229|       |    /// # Returns
  230|       |    ///
  231|       |    /// Returns completed progress with consumed input and written output
  232|       |    /// counters.
  233|       |    #[inline]
  234|  1.87k|    pub(in crate::transcode) fn complete_progress(&self) -> TranscodeProgress {
  235|  1.87k|        TranscodeProgress::complete(self.read(), self.written())
  236|  1.87k|    }
  237|       |
  238|       |    /// Returns progress for missing input.
  239|       |    ///
  240|       |    /// # Parameters
  241|       |    ///
  242|       |    /// - `required`: Current minimum total input units required before retrying
  243|       |    ///   from the current input position. A later retry may raise this lower
  244|       |    ///   bound.
  245|       |    ///
  246|       |    /// # Returns
  247|       |    ///
  248|       |    /// Returns [`TranscodeProgress`] with need-input status.
  249|       |    #[inline]
  250|  1.15k|    pub(in crate::transcode) fn need_input_progress(&self, required: NonZeroUsize) -> TranscodeProgress {
  251|  1.15k|        TranscodeProgress::need_input(required, self.read(), self.written())
  252|  1.15k|    }
  253|       |
  254|       |    /// Returns progress for missing output.
  255|       |    ///
  256|       |    /// # Parameters
  257|       |    ///
  258|       |    /// - `required`: Total output units required from the current output
  259|       |    ///   position.
  260|       |    ///
  261|       |    /// # Returns
  262|       |    ///
  263|       |    /// Returns [`TranscodeProgress`] with need-output status.
  264|       |    #[inline]
  265|    990|    pub(in crate::transcode) fn need_output_progress(&self, required: NonZeroUsize) -> TranscodeProgress {
  266|    990|        TranscodeProgress::need_output(required, self.read(), self.written())
  267|    990|    }
  268|       |
  269|       |    /// Advances the input cursor.
  270|       |    ///
  271|       |    /// # Parameters
  272|       |    ///
  273|       |    /// - `read`: Number of input units consumed by the last operation; must not
  274|       |    ///   exceed the available input.
  275|       |    ///
  276|       |    /// # Panics
  277|       |    ///
  278|       |    /// Panics when overflow checks are enabled if cursor addition overflows.
  279|       |    #[inline]
  280|  4.50k|    pub(in crate::transcode) fn advance_input(&mut self, read: usize) {
  281|  4.50k|        self.input_cursor += read;
  282|  4.50k|    }
  283|       |
  284|       |    /// Advances the output cursor.
  285|       |    ///
  286|       |    /// # Parameters
  287|       |    ///
  288|       |    /// - `written`: Number of output units written by the last operation.
  289|       |    ///   Callers must ensure cursor addition does not overflow.
  290|       |    ///
  291|       |    /// # Panics
  292|       |    ///
  293|       |    /// Panics when overflow checks are enabled if cursor addition overflows.
  294|       |    #[inline]
  295|  6.33k|    pub(in crate::transcode) fn advance_output(&mut self, written: usize) {
  296|  6.33k|        self.output_cursor += written;
  297|  6.33k|    }
  298|       |
  299|       |    /// Advances both cursors.
  300|       |    ///
  301|       |    /// `read` must not exceed the available input. Neither cursor addition may
  302|       |    /// overflow; this method delegates to the individual cursor operations.
  303|       |    ///
  304|       |    /// # Parameters
  305|       |    ///
  306|       |    /// - `read`: Number of input units consumed by the last operation.
  307|       |    /// - `written`: Number of output units written by the last operation.
  308|       |    ///   Callers must ensure cursor addition does not overflow.
  309|       |    ///
  310|       |    /// # Panics
  311|       |    ///
  312|       |    /// Panics when overflow checks are enabled if cursor addition overflows.
  313|       |    #[inline]
  314|     72|    pub(in crate::transcode) fn advance(&mut self, read: usize, written: usize) {
  315|     72|        self.advance_input(read);
  316|     72|        self.advance_output(written);
  317|     72|    }
  318|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/io/async_transcode_decode_input.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Buffered asynchronous input driver that decodes units into values.
    9|       |
   10|       |use core::fmt;
   11|       |use std::collections::TryReserveError;
   12|       |use std::future::poll_fn;
   13|       |use std::io::Error;
   14|       |use std::io::ErrorKind;
   15|       |use std::io::Result;
   16|       |use std::pin::Pin;
   17|       |use std::result::Result as StdResult;
   18|       |use std::task::Context;
   19|       |use std::task::Poll;
   20|       |
   21|       |use qubit_io::AsyncBufferedInput;
   22|       |use qubit_io::AsyncInput;
   23|       |use qubit_io::Buffer;
   24|       |use qubit_utils::SliceRange;
   25|       |use qubit_utils::allocation_error;
   26|       |
   27|       |use super::async_transcode_decode_step::AsyncTranscodeDecodeStep;
   28|       |use super::internal::validate_decode_progress;
   29|       |use super::internal::validate_eof_decode_progress;
   30|       |use crate::CapacityError;
   31|       |use crate::TranscodeProgress;
   32|       |use crate::Transcoder;
   33|       |
   34|       |/// Decodes an asynchronous unit stream into values.
   35|       |///
   36|       |/// The adapter owns unit buffering but not decoder state. Callers supply a
   37|       |/// streaming decoder and error mapper for each operation.
   38|       |///
   39|       |/// # Driving input through EOF
   40|       |///
   41|       |/// Reset the decoder first and collect the number of values it writes. For
   42|       |/// each [`Self::poll_transcode`] progress result, add `written()` to the output
   43|       |/// cursor; the bridge has already consumed `read()` from its unread buffer.
   44|       |/// When the status is `NeedInput { required }`, call
   45|       |/// [`Self::fill_until_async`] with `required.get()`. If it returns `true`, poll
   46|       |/// the decoder again. If it returns `false`, EOF has been confirmed and any
   47|       |/// short tail remains unread; pass that tail to [`Self::transcode_eof_step`].
   48|       |/// Keep stepping on `NeedOutput` with a sufficiently large output range. Once
   49|       |/// EOF processing returns `Complete`, call [`Self::finish`] and collect its
   50|       |/// returned output count. `EndOfInput` from polling means the unread buffer was
   51|       |/// already empty when refill confirmed EOF. In both cases, call
   52|       |/// [`Self::transcode_eof_step`] to dispatch the EOF event, including when no
   53|       |/// unread units remain.
   54|       |///
   55|       |/// # Type Parameters
   56|       |///
   57|       |/// - `I`: Wrapped asynchronous unit input.
   58|       |///
   59|       |/// # Examples
   60|       |///
   61|       |/// ```
   62|       |/// use qubit_codec::AsyncTranscodeDecodeInput;
   63|       |/// use qubit_io::AsyncInput;
   64|       |///
   65|       |/// # use std::io::Result;
   66|       |/// # use std::pin::Pin;
   67|       |/// # use std::task::Context;
   68|       |/// # use std::task::Poll;
   69|       |/// # struct EmptyInput;
   70|       |/// # impl AsyncInput for EmptyInput {
   71|       |/// #     type Item = u8;
   72|       |/// #     unsafe fn poll_read_unchecked(self: Pin<&mut Self>, _cx: &mut Context<'_>, _output: &mut [u8], _index: usize, _count: usize) -> Poll<Result<usize>> {
   73|       |/// #         Poll::Ready(Ok(0))
   74|       |/// #     }
   75|       |/// # }
   76|       |/// let mut input = AsyncTranscodeDecodeInput::with_capacity(EmptyInput, 8);
   77|       |/// assert!(input.capacity() >= 8);
   78|       |/// assert_eq!(input.unread_len(), 0);
   79|       |/// input.consume(0);
   80|       |/// assert!(input.unread().is_empty());
   81|       |/// ```
   82|       |///
   83|       |/// The following driver decodes one complete pair, preserves a one-unit tail,
   84|       |/// retries EOF processing after `NeedOutput`, and collects lifecycle output
   85|       |/// without an async runtime. The source and decoder are small fixtures; real
   86|       |/// applications use their own `AsyncInput` and `Transcoder` implementations.
   87|       |///
   88|       |/// ```
   89|       |/// use qubit_codec::{AsyncTranscodeDecodeInput, AsyncTranscodeDecodeStep, CapacityError, TranscodeDecodeError, TranscodeProgress, Transcoder};
   90|       |/// use qubit_io::AsyncInput;
   91|       |/// use std::convert::Infallible;
   92|       |/// use std::future::{poll_fn, Future};
   93|       |/// use std::io;
   94|       |/// use std::num::NonZeroUsize;
   95|       |/// use std::pin::{pin, Pin};
   96|       |/// use std::task::{Context, Poll, Waker};
   97|       |///
   98|       |/// # #[derive(Debug)] struct ThreeBytes { bytes: [u8; 3], position: usize }
   99|       |/// # impl AsyncInput for ThreeBytes {
  100|       |/// #     type Item = u8;
  101|       |/// #     unsafe fn poll_read_unchecked(mut self: Pin<&mut Self>, _cx: &mut Context<'_>, output: &mut [u8], index: usize, count: usize) -> Poll<io::Result<usize>> {
  102|       |/// #         let read = count.min(self.bytes.len() - self.position);
  103|       |/// #         output[index..index + read].copy_from_slice(&self.bytes[self.position..self.position + read]);
  104|       |/// #         self.position += read;
  105|       |/// #         Poll::Ready(Ok(read))
  106|       |/// #     }
  107|       |/// # }
  108|       |/// # #[derive(Debug)] struct TailDecoder;
  109|       |/// # impl Transcoder for TailDecoder {
  110|       |/// #     type Input = u8; type Output = u16; type Error = TranscodeDecodeError<Infallible>;
  111|       |/// #     fn max_transcode_output_len(&self, n: usize) -> Result<usize, CapacityError> { Ok(n) }
  112|       |/// #     fn max_reset_output_len(&self) -> Result<usize, CapacityError> { Ok(1) }
  113|       |/// #     fn reset(&mut self, out: &mut [u16], index: usize) -> Result<usize, Self::Error> { out[index] = 0xaaaa; Ok(1) }
  114|       |/// #     fn transcode(&mut self, input: &[u8], ii: usize, out: &mut [u16], oi: usize) -> Result<TranscodeProgress, Self::Error> {
  115|       |/// #         let available = input.len() - ii;
  116|       |/// #         if available < 2 { return Ok(TranscodeProgress::need_input(NonZeroUsize::new(2).unwrap(), 0, 0)); }
  117|       |/// #         if out.len() == oi { return Ok(TranscodeProgress::need_output(NonZeroUsize::new(1).unwrap(), 0, 0)); }
  118|       |/// #         out[oi] = u16::from_be_bytes([input[ii], input[ii + 1]]);
  119|       |/// #         Ok(TranscodeProgress::complete(2, 1))
  120|       |/// #     }
  121|       |/// #     fn transcode_eof(&mut self, input: &[u8], ii: usize, out: &mut [u16], oi: usize) -> Result<TranscodeProgress, Self::Error> {
  122|       |/// #         let available = input.len() - ii;
  123|       |/// #         if available == 0 { return Ok(TranscodeProgress::complete(0, 0)); }
  124|       |/// #         if available == 1 {
  125|       |/// #             if out.len() - oi < 2 { return Ok(TranscodeProgress::need_output(NonZeroUsize::new(2).unwrap(), 0, 0)); }
  126|       |/// #             out[oi] = u16::from(input[ii]);
  127|       |/// #             return Ok(TranscodeProgress::complete(1, 1));
  128|       |/// #         }
  129|       |/// #         self.transcode(input, ii, out, oi)
  130|       |/// #     }
  131|       |/// #     fn max_finish_output_len(&self) -> Result<usize, CapacityError> { Ok(1) }
  132|       |/// #     fn finish(&mut self, out: &mut [u16], index: usize) -> Result<usize, Self::Error> { out[index] = 0xbbbb; Ok(1) }
  133|       |/// # }
  134|       |/// # fn block_on<F: Future>(future: F) -> F::Output {
  135|       |/// #     let mut future = pin!(future); let mut cx = Context::from_waker(Waker::noop());
  136|       |/// #     loop { if let Poll::Ready(value) = future.as_mut().poll(&mut cx) { return value; } }
  137|       |/// # }
  138|       |/// let source = ThreeBytes { bytes: [0x12, 0x34, 0x56], position: 0 };
  139|       |/// let mut input = AsyncTranscodeDecodeInput::with_capacity(source, 2);
  140|       |/// let mut decoder = TailDecoder;
  141|       |/// let mut output = [0_u16; 4];
  142|       |/// let output_len = output.len();
  143|       |/// let mut map_error = |error| io::Error::new(io::ErrorKind::InvalidData, error);
  144|       |/// let mut written = input.reset(&mut decoder, &mut map_error, &mut output, 0, output_len)?;
  145|       |/// assert_eq!(1, written);
  146|       |/// let first = block_on(poll_fn(|cx| Pin::new(&mut input).poll_transcode(cx, &mut decoder, &mut map_error, &mut output, written, output_len - written)))?;
  147|       |/// let AsyncTranscodeDecodeStep::Progress(progress) = first else { panic!("expected decoded pair") };
  148|       |/// assert_eq!(TranscodeProgress::complete(2, 1), progress);
  149|       |/// written += progress.written();
  150|       |/// assert_eq!(0x1234, output[1]);
  151|       |/// let second = block_on(poll_fn(|cx| Pin::new(&mut input).poll_transcode(cx, &mut decoder, &mut map_error, &mut output, written, output_len - written)))?;
  152|       |/// assert!(matches!(second, AsyncTranscodeDecodeStep::Progress(progress) if matches!(progress.status(), qubit_codec::TranscodeStatus::NeedInput { .. })));
  153|       |/// assert!(!block_on(input.fill_until_async(2))?);
  154|       |/// assert_eq!([0x56], input.unread());
  155|       |/// let eof = input.transcode_eof_step(&mut decoder, &mut map_error, &mut output, written, 1)?;
  156|       |/// assert_eq!(TranscodeProgress::need_output(NonZeroUsize::new(2).unwrap(), 0, 0), eof);
  157|       |/// let eof = input.transcode_eof_step(&mut decoder, &mut map_error, &mut output, written, 2)?;
  158|       |/// assert_eq!(TranscodeProgress::complete(1, 1), eof);
  159|       |/// written += eof.written();
  160|       |/// let finished = input.finish(&mut decoder, &mut map_error, &mut output, written, output_len - written)?;
  161|       |/// written += finished;
  162|       |/// assert_eq!([0xaaaa, 0x1234, 0x56, 0xbbbb], output);
  163|       |/// assert_eq!(4, written);
  164|       |/// # Ok::<(), io::Error>(())
  165|       |/// ```
  166|       |#[must_use]
  167|       |pub struct AsyncTranscodeDecodeInput<I>
  168|       |where
  169|       |    I: AsyncInput,
  170|       |    I::Item: Copy + Default,
  171|       |{
  172|       |    /// Buffered asynchronous unit input.
  173|       |    input: AsyncBufferedInput<I>,
  174|       |}
  175|       |
  176|       |impl<I> AsyncTranscodeDecodeInput<I>
  177|       |where
  178|       |    I: AsyncInput,
  179|       |    I::Item: Copy + Default,
  180|       |{
  181|       |    /// Creates an adapter with the default unit-buffer capacity.
  182|       |    ///
  183|       |    /// # Parameters
  184|       |    ///
  185|       |    /// - `inner`: Unit input read by this adapter.
  186|       |    ///
  187|       |    /// # Returns
  188|       |    ///
  189|       |    /// Returns a new buffered asynchronous decoder input.
  190|       |    #[inline]
  191|     13|    pub fn new(inner: I) -> Self {
  192|     13|        Self {
  193|     13|            input: AsyncBufferedInput::new(inner),
  194|     13|        }
  195|     13|    }
  196|       |
  197|       |    /// Creates an adapter with an internal unit buffer of at least capacity.
  198|       |    ///
  199|       |    /// # Parameters
  200|       |    ///
  201|       |    /// - `inner`: Unit input read by this adapter.
  202|       |    /// - `capacity`: Requested internal unit-buffer capacity.
  203|       |    ///
  204|       |    /// # Returns
  205|       |    ///
  206|       |    /// Returns a new buffered asynchronous decoder input.
  207|       |    #[inline]
  208|     15|    pub fn with_capacity(inner: I, capacity: usize) -> Self {
  209|     15|        Self {
  210|     15|            input: AsyncBufferedInput::with_capacity(inner, capacity),
  211|     15|        }
  212|     15|    }
  213|       |
  214|       |    /// Tries to create an adapter with an internal unit buffer of at least
  215|       |    /// capacity.
  216|       |    ///
  217|       |    /// # Parameters
  218|       |    ///
  219|       |    /// - `inner`: Asynchronous unit source owned by the adapter.
  220|       |    /// - `capacity`: Minimum requested unit-buffer capacity.
  221|       |    ///
  222|       |    /// # Returns
  223|       |    ///
  224|       |    /// The adapter with an empty unread window on successful allocation.
  225|       |    ///
  226|       |    /// # Errors
  227|       |    ///
  228|       |    /// Returns the allocation error when the internal buffer cannot be
  229|       |    /// allocated.
  230|      2|    pub fn try_with_capacity(inner: I, capacity: usize) -> StdResult<Self, TryReserveError> {
  231|       |        Ok(Self {
  232|      2|            input: AsyncBufferedInput::try_with_capacity(inner, capacity)?,
                                                                                       ^1
  233|       |        })
  234|      2|    }
  235|       |
  236|       |    /// Returns a shared reference to the wrapped asynchronous input.
  237|       |    ///
  238|       |    /// The input can be physically positioned after units retained in this
  239|       |    /// adapter's unread buffer.
  240|       |    ///
  241|       |    /// # Returns
  242|       |    ///
  243|       |    /// A shared borrow of the underlying source, excluding buffered units.
  244|       |    #[must_use]
  245|       |    #[inline]
  246|      1|    pub const fn inner(&self) -> &I {
  247|      1|        self.input.inner()
  248|      1|    }
  249|       |
  250|       |    /// Returns a mutable reference to the wrapped asynchronous input.
  251|       |    ///
  252|       |    /// Direct reads can invalidate the logical stream position represented by
  253|       |    /// unread buffered units.
  254|       |    ///
  255|       |    /// # Returns
  256|       |    ///
  257|       |    /// An exclusive borrow of the source; direct reads bypass buffered units.
  258|       |    #[must_use]
  259|       |    #[inline]
  260|      1|    pub fn inner_mut(&mut self) -> &mut I {
  261|      1|        self.input.inner_mut()
  262|      1|    }
  263|       |
  264|       |    /// Returns the number of unread units currently buffered.
  265|       |    ///
  266|       |    /// # Returns
  267|       |    ///
  268|       |    /// The number of buffered units not yet consumed.
  269|       |    #[must_use]
  270|       |    #[inline]
  271|     66|    pub const fn unread_len(&self) -> usize {
  272|     66|        self.input.unread_len()
  273|     66|    }
  274|       |
  275|       |    /// Returns the unread buffered unit window.
  276|       |    ///
  277|       |    /// # Returns
  278|       |    ///
  279|       |    /// The currently retained units borrowed without allocation.
  280|       |    #[must_use]
  281|       |    #[inline]
  282|     33|    pub fn unread(&self) -> &[I::Item] {
  283|     33|        self.input.unread()
  284|     33|    }
  285|       |
  286|       |    /// Returns the total internal unit-buffer capacity.
  287|       |    ///
  288|       |    /// # Returns
  289|       |    ///
  290|       |    /// The total number of units the internal buffer can hold.
  291|       |    #[must_use]
  292|       |    #[inline]
  293|      1|    pub fn capacity(&self) -> usize {
  294|      1|        self.input.capacity()
  295|      1|    }
  296|       |
  297|       |    /// Consumes unread units from the current buffer window.
  298|       |    ///
  299|       |    /// # Parameters
  300|       |    ///
  301|       |    /// - `count`: Number of retained units to discard from the unread prefix.
  302|       |    ///
  303|       |    /// # Panics
  304|       |    ///
  305|       |    /// Panics when count exceeds the unread unit count.
  306|       |    #[inline]
  307|     17|    pub fn consume(&mut self, count: usize) {
  308|     17|        assert!(count <= self.unread_len(), "cannot consume beyond buffered input",);
  309|       |        // SAFETY: The asserted bound proves count fits the unread window.
  310|     16|        unsafe {
  311|     16|            self.input.consume(count);
  312|     16|        }
  313|     16|    }
  314|       |
  315|       |    /// Copies unread units into an indexed output range without consuming them.
  316|       |    ///
  317|       |    /// # Parameters
  318|       |    ///
  319|       |    /// - `output`: Destination slice receiving copies of retained units.
  320|       |    /// - `output_index`: First destination index.
  321|       |    /// - `count`: Number of unread units to copy without consuming them.
  322|       |    ///
  323|       |    /// # Safety
  324|       |    ///
  325|       |    /// The caller must guarantee that the indexed range fits in output, does
  326|       |    /// not overflow, holds no overlapping unread units, and count is no
  327|       |    /// greater than the unread unit count.
  328|      1|    pub unsafe fn copy_unread_to(&self, output: &mut [I::Item], output_index: usize, count: usize) {
  329|       |        // SAFETY: The caller upholds the delegated buffer-copy contract.
  330|      1|        unsafe {
  331|      1|            self.input.copy_unread_to(output, output_index, count);
  332|      1|        }
  333|      1|    }
  334|       |
  335|       |    /// Consumes this adapter and returns the input plus its unread buffer.
  336|       |    ///
  337|       |    /// # Returns
  338|       |    ///
  339|       |    /// The owned source and buffer, preserving all unread units.
  340|       |    #[must_use = "the returned input and unread buffer must be handled"]
  341|       |    #[inline]
  342|      1|    pub fn into_parts(self) -> (I, Buffer<I::Item>) {
  343|      1|        self.input.into_parts()
  344|      1|    }
  345|       |
  346|       |    /// Runs decoder reset into an indexed output range without I/O.
  347|       |    ///
  348|       |    /// # Type Parameters
  349|       |    ///
  350|       |    /// - `D`: Stateful decoder from source units into values.
  351|       |    /// - `M`: Callback mapping decoder errors to I/O errors.
  352|       |    /// - `Value`: Destination value type.
  353|       |    ///
  354|       |    /// # Parameters
  355|       |    ///
  356|       |    /// - `decoder`: Decoder whose state may change during this operation.
  357|       |    /// - `map_error`: Callback invoked for decoder errors.
  358|       |    /// - `output`: Destination value slice.
  359|       |    /// - `output_index`: First destination index.
  360|       |    /// - `count`: Number of writable destination positions.
  361|       |    ///
  362|       |    /// # Returns
  363|       |    ///
  364|       |    /// Number of values emitted during decoder reset.
  365|       |    ///
  366|       |    /// # Errors
  367|       |    ///
  368|       |    /// Returns invalid output-range, capacity, or mapped decoder-reset errors.
  369|       |    ///
  370|       |    /// # Panics
  371|       |    ///
  372|       |    /// Panics if the decoder writes more values than its reset bound.
  373|      6|    pub fn reset<D, M, Value>(
  374|      6|        &self,
  375|      6|        decoder: &mut D,
  376|      6|        map_error: &mut M,
  377|      6|        output: &mut [Value],
  378|      6|        output_index: usize,
  379|      6|        count: usize,
  380|      6|    ) -> Result<usize>
  381|      6|    where
  382|      6|        D: Transcoder<Input = I::Item, Output = Value>,
  383|      6|        M: FnMut(D::Error) -> Error,
  384|       |    {
  385|      6|        let required = decoder.max_reset_output_len().map_err(capacity_to_io_error)?;
                          ^5                                                                     ^1
  386|      5|        let output_end = SliceRange::checked_range_end(
                          ^4
  387|      5|            output.len(),
  388|      5|            output_index,
  389|      5|            count,
  390|       |            "reset output range exceeds destination buffer",
  391|      1|        )?;
  392|      4|        if count < required {
  393|      1|            return Err(Error::new(
  394|      1|                ErrorKind::InvalidData,
  395|      1|                "insufficient output for decoder reset bound",
  396|      1|            ));
  397|      3|        }
  398|      3|        let output = &mut output[..output_end];
  399|      3|        let written = decoder.reset(output, output_index).map_err(&mut *map_error)?;
                                                                                                ^0
  400|      3|        assert!(written <= required, "reset wrote beyond its bound");
  401|      2|        Ok(written)
  402|      5|    }
  403|       |
  404|       |    /// Finishes a decoder into an indexed output range without I/O.
  405|       |    ///
  406|       |    /// # Type Parameters
  407|       |    ///
  408|       |    /// - `D`: Stateful decoder from source units into values.
  409|       |    /// - `M`: Callback mapping decoder errors to I/O errors.
  410|       |    /// - `Value`: Destination value type.
  411|       |    ///
  412|       |    /// # Parameters
  413|       |    ///
  414|       |    /// - `decoder`: Decoder whose state may change during this operation.
  415|       |    /// - `map_error`: Callback invoked for decoder errors.
  416|       |    /// - `output`: Destination value slice.
  417|       |    /// - `output_index`: First destination index.
  418|       |    /// - `count`: Number of writable destination positions.
  419|       |    ///
  420|       |    /// # Returns
  421|       |    ///
  422|       |    /// Number of values emitted while finishing the decoder.
  423|       |    ///
  424|       |    /// # Errors
  425|       |    ///
  426|       |    /// Returns invalid output-range, capacity, or mapped decoder-finish
  427|       |    /// errors.
  428|       |    ///
  429|       |    /// # Panics
  430|       |    ///
  431|       |    /// Panics if the decoder writes more values than its finish bound.
  432|      6|    pub fn finish<D, M, Value>(
  433|      6|        &self,
  434|      6|        decoder: &mut D,
  435|      6|        map_error: &mut M,
  436|      6|        output: &mut [Value],
  437|      6|        output_index: usize,
  438|      6|        count: usize,
  439|      6|    ) -> Result<usize>
  440|      6|    where
  441|      6|        D: Transcoder<Input = I::Item, Output = Value>,
  442|      6|        M: FnMut(D::Error) -> Error,
  443|       |    {
  444|      6|        let required = decoder.max_finish_output_len().map_err(capacity_to_io_error)?;
                          ^5                                                                      ^1
  445|      5|        let output_end = SliceRange::checked_range_end(
                          ^4
  446|      5|            output.len(),
  447|      5|            output_index,
  448|      5|            count,
  449|       |            "finish output range exceeds destination buffer",
  450|      1|        )?;
  451|      4|        if count < required {
  452|      1|            return Err(Error::new(
  453|      1|                ErrorKind::InvalidData,
  454|      1|                "insufficient output for decoder finish bound",
  455|      1|            ));
  456|      3|        }
  457|      3|        let output = &mut output[..output_end];
  458|      3|        let written = decoder.finish(output, output_index).map_err(&mut *map_error)?;
                                                                                                 ^0
  459|      3|        assert!(written <= required, "finish wrote beyond its bound");
  460|      2|        Ok(written)
  461|      5|    }
  462|       |}
  463|       |
  464|       |impl<I> AsyncTranscodeDecodeInput<I>
  465|       |where
  466|       |    I: AsyncInput + Unpin,
  467|       |    I::Item: Copy + Default + Unpin,
  468|       |{
  469|       |    /// Appends at least one unit to the unread buffer.
  470|       |    ///
  471|       |    /// # Returns
  472|       |    ///
  473|       |    /// Returns true when data was appended, or false at end of input.
  474|       |    ///
  475|       |    /// # Errors
  476|       |    ///
  477|       |    /// Returns input or buffer-management errors while refilling.
  478|       |    #[inline]
  479|      9|    pub async fn fill_more_async(&mut self) -> Result<bool> {
  480|      9|        self.input.fill_more_async().await
  481|      9|    }
  482|       |
  483|       |    /// Refills until at least count unread units are available.
  484|       |    ///
  485|       |    /// # Parameters
  486|       |    ///
  487|       |    /// - `count`: Minimum desired unread unit count; grows the buffer if
  488|       |    ///   needed.
  489|       |    ///
  490|       |    /// # Returns
  491|       |    ///
  492|       |    /// Returns true when count units are available, or false when EOF occurs
  493|       |    /// first.
  494|       |    ///
  495|       |    /// # Errors
  496|       |    ///
  497|       |    /// Returns allocation, input, or buffer-management errors while refilling.
  498|     10|    pub async fn fill_until_async(&mut self, count: usize) -> Result<bool> {
  499|     10|        if count > self.input.capacity() {
  500|      2|            self.input.try_reserve_capacity(count).map_err(allocation_error)?;
                                                                                          ^1
  501|      8|        }
  502|      9|        self.input.fill_until_async(count).await
  503|     10|    }
  504|       |
  505|       |    /// Polls one cancellation-safe decoder operation.
  506|       |    ///
  507|       |    /// The operation first refills only when no unread unit is buffered. After
  508|       |    /// a decoder call changes either the decoder or the buffered input, it
  509|       |    /// immediately returns [`AsyncTranscodeDecodeStep::Progress`] and never
  510|       |    /// polls the input again. This makes the returned progress the commit
  511|       |    /// boundary for cancellation: resume with the adapter's current state,
  512|       |    /// rather than replaying the previous source range.
  513|       |    ///
  514|       |    /// # Type Parameters
  515|       |    ///
  516|       |    /// - `D`: Stateful decoder from source units into values.
  517|       |    /// - `M`: Callback mapping decoder errors to I/O errors.
  518|       |    /// - `Value`: Destination value type.
  519|       |    ///
  520|       |    /// # Parameters
  521|       |    ///
  522|       |    /// - `cx`: Task context used when polling the source.
  523|       |    /// - `decoder`: Decoder whose state may change during this operation.
  524|       |    /// - `map_error`: Callback invoked for decoder errors.
  525|       |    /// - `output`: Destination value slice.
  526|       |    /// - `output_index`: First destination index.
  527|       |    /// - `count`: Number of writable destination positions.
  528|       |    ///
  529|       |    /// # Returns
  530|       |    ///
  531|       |    /// `Pending` only while awaiting source input before decoding; otherwise
  532|       |    /// ready progress or end-of-input. Successful progress has already
  533|       |    /// consumed source units.
  534|       |    ///
  535|       |    /// # Errors
  536|       |    ///
  537|       |    /// Returns input, allocation, output-range, invalid-progress, or mapped
  538|       |    /// decoder errors.
  539|       |    /// Successful validated progress commits its `read()` count. If decoding
  540|       |    /// or progress validation fails, this call does not consume the unread
  541|       |    /// range. The decoder and destination may already have changed; neither is
  542|       |    /// rolled back, and no complete progress report is available. A refill
  543|       |    /// performed before the failure is not rolled back at the wrapped input.
  544|       |    /// Cancellation safety applies only to cancellation while this method is
  545|       |    /// pending; it does not imply rollback after an error. Do not replay the
  546|       |    /// original input unless the decoder defines an explicit recovery protocol.
  547|     24|    pub fn poll_transcode<D, M, Value>(
  548|     24|        mut self: Pin<&mut Self>,
  549|     24|        cx: &mut Context<'_>,
  550|     24|        decoder: &mut D,
  551|     24|        map_error: &mut M,
  552|     24|        output: &mut [Value],
  553|     24|        output_index: usize,
  554|     24|        count: usize,
  555|     24|    ) -> Poll<Result<AsyncTranscodeDecodeStep>>
  556|     24|    where
  557|     24|        D: Transcoder<Input = I::Item, Output = Value>,
  558|     24|        M: FnMut(D::Error) -> Error,
  559|       |    {
  560|     24|        let output_end = SliceRange::checked_range_end(
                          ^23
  561|     24|            output.len(),
  562|     24|            output_index,
  563|     24|            count,
  564|       |            "decoded output range exceeds destination buffer",
  565|      1|        )?;
  566|     23|        if count == 0 {
  567|      2|            return Poll::Ready(Ok(AsyncTranscodeDecodeStep::Progress(TranscodeProgress::complete(
  568|      2|                0, 0,
  569|      2|            ))));
  570|     21|        }
  571|     21|        let output = &mut output[..output_end];
  572|     21|        if self.as_ref().get_ref().unread_len() == 0 {
  573|     16|            let this = self.as_mut().get_mut();
  574|     16|            match Pin::new(&mut this.input).poll_fill_more(cx) {
  575|      6|                Poll::Ready(Ok(true)) => {}
  576|       |                Poll::Ready(Ok(false)) => {
  577|      2|                    return Poll::Ready(Ok(AsyncTranscodeDecodeStep::EndOfInput));
  578|       |                }
  579|      1|                Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
  580|      7|                Poll::Pending => return Poll::Pending,
  581|       |            }
  582|      5|        }
  583|     11|        let this = self.as_mut().get_mut();
  584|     11|        let available_input = this.unread_len();
  585|     11|        let progress = decoder
  586|     11|            .transcode(this.unread(), 0, output, output_index)
  587|     11|            .map_err(&mut *map_error)
  588|     11|            .and_then(|progress| validate_decode_progress(progress, 0, available_input, output_index, count));
                                               ^10                      ^10          ^10              ^10           ^10
  589|     11|        let progress = match progress {
                          ^8
  590|      8|            Ok(progress) => progress,
  591|      3|            Err(error) => return Poll::Ready(Err(error)),
  592|       |        };
  593|      8|        this.consume(progress.read());
  594|      8|        Poll::Ready(Ok(AsyncTranscodeDecodeStep::Progress(progress)))
  595|     24|    }
  596|       |
  597|       |    /// Performs one decode step after the caller has established EOF.
  598|       |    ///
  599|       |    /// This method performs no I/O. It passes the current unread buffer to
  600|       |    /// [`Transcoder::transcode_eof`], validates progress, and commits consumed
  601|       |    /// source units before returning.
  602|       |    ///
  603|       |    /// # Type Parameters
  604|       |    ///
  605|       |    /// - `D`: Stateful decoder from source units into values.
  606|       |    /// - `M`: Callback mapping decoder errors to I/O errors.
  607|       |    /// - `Value`: Destination value type.
  608|       |    ///
  609|       |    /// # Parameters
  610|       |    ///
  611|       |    /// - `decoder`: Decoder whose state may change during this operation.
  612|       |    /// - `map_error`: Callback invoked for decoder errors.
  613|       |    /// - `output`: Destination value slice.
  614|       |    /// - `output_index`: First destination index.
  615|       |    /// - `count`: Number of writable destination positions.
  616|       |    ///
  617|       |    /// # Returns
  618|       |    ///
  619|       |    /// Validated progress with source consumption committed. A zero output
  620|       |    /// count returns zero progress without calling the decoder; an empty input
  621|       |    /// is still dispatched to `transcode_eof`.
  622|       |    ///
  623|       |    /// # Errors
  624|       |    ///
  625|       |    /// Returns invalid output-range errors, decoder errors mapped by
  626|       |    /// `map_error`, or `InvalidData` if the decoder requests more input after
  627|       |    /// EOF. Invalid progress does not commit source consumption, but decoder
  628|       |    /// state and caller-owned output are not rolled back.
  629|     13|    pub fn transcode_eof_step<D, M, Value>(
  630|     13|        &mut self,
  631|     13|        decoder: &mut D,
  632|     13|        map_error: &mut M,
  633|     13|        output: &mut [Value],
  634|     13|        output_index: usize,
  635|     13|        count: usize,
  636|     13|    ) -> Result<TranscodeProgress>
  637|     13|    where
  638|     13|        D: Transcoder<Input = I::Item, Output = Value>,
  639|     13|        M: FnMut(D::Error) -> Error,
  640|       |    {
  641|     13|        let output_end = SliceRange::checked_range_end(
                          ^12
  642|     13|            output.len(),
  643|     13|            output_index,
  644|     13|            count,
  645|       |            "decoded EOF output range exceeds destination buffer",
  646|      1|        )?;
  647|     12|        if count == 0 {
  648|      2|            return Ok(TranscodeProgress::complete(0, 0));
  649|     10|        }
  650|     10|        let available_input = self.unread_len();
  651|     10|        let progress = decoder
                          ^7
  652|     10|            .transcode_eof(self.unread(), 0, &mut output[..output_end], output_index)
  653|     10|            .map_err(&mut *map_error)
  654|     10|            .and_then(|progress| validate_eof_decode_progress(progress, 0, available_input, output_index, count))?;
                                               ^9                           ^9           ^9               ^9            ^9     ^3
  655|      7|        self.consume(progress.read());
  656|      7|        Ok(progress)
  657|     13|    }
  658|       |
  659|       |    /// Decodes one cancellation-safe progress step into an indexed output
  660|       |    /// range.
  661|       |    ///
  662|       |    /// This is the async wrapper for [`Self::poll_transcode`]. It returns after
  663|       |    /// one decoder invocation; callers that require a complete destination
  664|       |    /// range must drive successive steps and retain each returned progress.
  665|       |    ///
  666|       |    /// # Type Parameters
  667|       |    ///
  668|       |    /// - `D`: Stateful decoder from source units into values.
  669|       |    /// - `M`: Callback mapping decoder errors to I/O errors.
  670|       |    /// - `Value`: Destination value type.
  671|       |    ///
  672|       |    /// # Parameters
  673|       |    ///
  674|       |    /// - `decoder`: Decoder whose state may change during this operation.
  675|       |    /// - `map_error`: Callback invoked for decoder errors.
  676|       |    /// - `output`: Destination value slice.
  677|       |    /// - `output_index`: First destination index.
  678|       |    /// - `count`: Number of writable destination positions.
  679|       |    ///
  680|       |    /// # Returns
  681|       |    ///
  682|       |    /// One committed progress step, or end-of-input when the source is
  683|       |    /// exhausted.
  684|       |    ///
  685|       |    /// # Errors
  686|       |    ///
  687|       |    /// Returns input, allocation, output-range, invalid-progress, or mapped
  688|       |    /// decoder errors from `poll_transcode`.
  689|       |    #[inline]
  690|     14|    pub async fn transcode_async<D, M, Value>(
  691|     14|        &mut self,
  692|     14|        decoder: &mut D,
  693|     14|        map_error: &mut M,
  694|     14|        output: &mut [Value],
  695|     14|        output_index: usize,
  696|     14|        count: usize,
  697|     14|    ) -> Result<AsyncTranscodeDecodeStep>
  698|     14|    where
  699|     14|        D: Transcoder<Input = I::Item, Output = Value>,
  700|     14|        M: FnMut(D::Error) -> Error,
  701|     14|    {
  702|     20|        poll_fn(|cx| Pin::new(&mut *self).poll_transcode(cx, decoder, map_error, output, output_index, count)).await
                      ^14                                                                                                    ^14
  703|     14|    }
  704|       |}
  705|       |
  706|       |impl<I> AsyncInput for AsyncTranscodeDecodeInput<I>
  707|       |where
  708|       |    I: AsyncInput,
  709|       |    I::Item: Copy + Default,
  710|       |{
  711|       |    /// Item type read through the retained unit buffer.
  712|       |    type Item = I::Item;
  713|       |
  714|       |    /// Reports that this input retains unread units.
  715|       |    ///
  716|       |    /// # Returns
  717|       |    ///
  718|       |    /// Always `true`, because this adapter retains unread units.
  719|       |    #[inline]
  720|      1|    fn is_buffered(&self) -> bool {
  721|      1|        true
  722|      1|    }
  723|       |
  724|       |    /// Polls one read through the retained unit buffer.
  725|       |    ///
  726|       |    /// # Parameters
  727|       |    ///
  728|       |    /// - `cx`: Task context used to poll the buffered source.
  729|       |    /// - `output`: Destination unit slice.
  730|       |    /// - `index`: First destination index.
  731|       |    /// - `count`: Maximum number of units to read.
  732|       |    ///
  733|       |    /// # Returns
  734|       |    ///
  735|       |    /// `Pending` while awaiting source data, or a ready read count; zero
  736|       |    /// indicates EOF.
  737|       |    ///
  738|       |    /// # Errors
  739|       |    ///
  740|       |    /// Returns errors from the buffered source or its buffer management.
  741|       |    ///
  742|       |    /// # Safety
  743|       |    ///
  744|       |    /// The caller must provide a valid indexed output range.
  745|      2|    unsafe fn poll_read_unchecked(
  746|      2|        mut self: Pin<&mut Self>,
  747|      2|        cx: &mut Context<'_>,
  748|      2|        output: &mut [Self::Item],
  749|      2|        index: usize,
  750|      2|        count: usize,
  751|      2|    ) -> Poll<Result<usize>> {
  752|       |        // SAFETY: Pinning this wrapper pins its buffered input field.
  753|      2|        let input = unsafe { Pin::new_unchecked(&mut self.as_mut().get_unchecked_mut().input) };
  754|       |        // SAFETY: The caller upholds the delegated indexed-read contract.
  755|      2|        unsafe { input.poll_read_unchecked(cx, output, index, count) }
  756|      2|    }
  757|       |}
  758|       |
  759|       |impl<I> fmt::Debug for AsyncTranscodeDecodeInput<I>
  760|       |where
  761|       |    I: AsyncInput,
  762|       |    I::Item: Copy + Default,
  763|       |    AsyncBufferedInput<I>: fmt::Debug,
  764|       |{
  765|       |    /// Formats this asynchronous decoder input for debugging.
  766|       |    ///
  767|       |    /// # Parameters
  768|       |    ///
  769|       |    /// - `formatter`: Debug output destination.
  770|       |    ///
  771|       |    /// # Returns
  772|       |    ///
  773|       |    /// The result of formatting the buffered input.
  774|       |    ///
  775|       |    /// # Errors
  776|       |    ///
  777|       |    /// Returns a formatting error if the destination rejects output.
  778|      1|    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
  779|      1|        formatter
  780|      1|            .debug_struct("AsyncTranscodeDecodeInput")
  781|      1|            .field("input", &self.input)
  782|      1|            .finish()
  783|      1|    }
  784|       |}
  785|       |
  786|       |/// Converts decoder capacity failures into invalid stream data.
  787|       |///
  788|       |/// # Parameters
  789|       |///
  790|       |/// - `error`: Capacity failure retained as the I/O error payload.
  791|       |///
  792|       |/// # Returns
  793|       |///
  794|       |/// An invalid-data error owning the original capacity failure.
  795|      2|fn capacity_to_io_error(error: CapacityError) -> Error {
  796|      2|    Error::new(ErrorKind::InvalidData, error)
  797|      2|}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/io/async_transcode_encode_output.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Buffered asynchronous output driver for streaming transcoders.
    9|       |
   10|       |use core::fmt;
   11|       |use std::collections::TryReserveError;
   12|       |use std::future::poll_fn;
   13|       |use std::io::Error;
   14|       |use std::io::ErrorKind;
   15|       |use std::io::Result;
   16|       |use std::pin::Pin;
   17|       |use std::task::Context;
   18|       |use std::task::Poll;
   19|       |
   20|       |use qubit_io::AsyncBufferedOutput;
   21|       |use qubit_io::AsyncOutput;
   22|       |use qubit_io::Buffer;
   23|       |use qubit_utils::SliceRange;
   24|       |use qubit_utils::allocation_error;
   25|       |
   26|       |use super::internal::validate_encode_progress;
   27|       |use crate::CapacityError;
   28|       |use crate::TranscodeEncoder;
   29|       |use crate::TranscodeProgress;
   30|       |use crate::TranscodeStatus;
   31|       |use crate::Transcoder;
   32|       |
   33|       |/// Buffers asynchronous transcoder output while preserving pending units.
   34|       |///
   35|       |/// This adapter is the asynchronous counterpart to
   36|       |/// [`super::TranscodeEncodeOutput`]. It owns the unit-level buffer, so a
   37|       |/// future cancelled while the wrapped output is pending retains every unit
   38|       |/// that the transcoder has already produced. Each transcode operation returns
   39|       |/// after exactly one encoder invocation, so its progress is the explicit
   40|       |/// source-consumption commit boundary.
   41|       |///
   42|       |/// Cancellation and pending-output recovery assume that wrapped I/O calls
   43|       |/// obey their normal return contracts. This adapter does not track poisoning
   44|       |/// or indeterminate delivery after an inner I/O panic. If such a panic is
   45|       |/// caught, reconcile external side effects before retrying or replaying
   46|       |/// pending units.
   47|       |///
   48|       |/// # Type Parameters
   49|       |///
   50|       |/// * `O` - Wrapped asynchronous unit output.
   51|       |///
   52|       |/// # Examples
   53|       |///
   54|       |/// ```
   55|       |/// use core::convert::Infallible;
   56|       |/// use core::num::NonZeroUsize;
   57|       |/// use std::future::Future;
   58|       |/// use std::io::Result;
   59|       |/// use std::pin::{pin, Pin};
   60|       |/// use std::task::{Context, Poll, Waker};
   61|       |///
   62|       |/// use qubit_codec::{AsyncTranscodeEncodeOutput, Codec, CodecTranscodeEncoder};
   63|       |/// use qubit_codec::{DecodeFailure, TranscodeEncoder};
   64|       |/// use qubit_io::AsyncOutput;
   65|       |///
   66|       |/// struct ByteCodec;
   67|       |/// impl Codec for ByteCodec {
   68|       |///     type Value = u8;
   69|       |///     type Unit = u8;
   70|       |///     type DecodeError = Infallible;
   71|       |///     type EncodeError = Infallible;
   72|       |///     const MIN_UNITS_PER_VALUE: usize = 1;
   73|       |///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
   74|       |///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
   75|       |///     unsafe fn decode(&mut self, input: &[u8], index: usize)
   76|       |///         -> std::result::Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
   77|       |///         Ok((input[index], NonZeroUsize::MIN))
   78|       |///     }
   79|       |///     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize)
   80|       |///         -> std::result::Result<usize, Infallible> {
   81|       |///         output[index] = *value;
   82|       |///         Ok(1)
   83|       |///     }
   84|       |/// }
   85|       |///
   86|       |/// #[derive(Default)]
   87|       |/// struct Sink { bytes: Vec<u8>, flushes: usize }
   88|       |/// impl AsyncOutput for Sink {
   89|       |///     type Item = u8;
   90|       |///     unsafe fn poll_write_unchecked(
   91|       |///         self: Pin<&mut Self>, _: &mut Context<'_>, units: &[u8], index: usize, count: usize,
   92|       |///     ) -> Poll<Result<usize>> {
   93|       |///         self.get_mut().bytes.extend_from_slice(&units[index..index + count]);
   94|       |///         Poll::Ready(Ok(count))
   95|       |///     }
   96|       |///     fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<()>> {
   97|       |///         self.get_mut().flushes += 1;
   98|       |///         Poll::Ready(Ok(()))
   99|       |///     }
  100|       |/// }
  101|       |///
  102|       |/// async fn run_example() -> Result<()> {
  103|       |///     let mut encoder = CodecTranscodeEncoder::new(ByteCodec);
  104|       |///     let mut output = AsyncTranscodeEncodeOutput::with_capacity(Sink::default(), 8);
  105|       |///     let mut map_error = |_| std::io::Error::other("encode error");
  106|       |///     output.reset_async(&mut encoder, &mut map_error).await?;
  107|       |///     let values = [1_u8, 2, 3];
  108|       |///     let mut offset = 0;
  109|       |///     while offset < values.len() {
  110|       |///         let progress = output.transcode_async(
  111|       |///             &mut encoder, &mut map_error, &values, offset, values.len() - offset,
  112|       |///         ).await?;
  113|       |///         assert!(progress.read() > 0);
  114|       |///         offset += progress.read();
  115|       |///     }
  116|       |///     output.finish_async(&mut encoder, &mut map_error).await?;
  117|       |///     assert_eq!(output.pending_len(), values.len());
  118|       |///     output.flush_async().await?;
  119|       |///     let (sink, pending) = output.into_parts();
  120|       |///     assert!(pending.is_empty());
  121|       |///     assert_eq!(sink.bytes, vec![1, 2, 3]);
  122|       |///     assert_eq!(sink.flushes, 1);
  123|       |///     Ok(())
  124|       |/// }
  125|       |///
  126|       |/// let mut future = pin!(run_example());
  127|       |/// let mut context = Context::from_waker(Waker::noop());
  128|       |/// match future.as_mut().poll(&mut context) {
  129|       |///     Poll::Ready(result) => result.unwrap(),
  130|       |///     Poll::Pending => panic!("the example sink always completes immediately"),
  131|       |/// }
  132|       |/// ```
  133|       |///
  134|       |/// `finish_async` commits final units to the adapter buffer. `flush_async`
  135|       |/// delivers those units and flushes the wrapped sink.
  136|       |#[must_use]
  137|       |pub struct AsyncTranscodeEncodeOutput<O>
  138|       |where
  139|       |    O: AsyncOutput,
  140|       |    O::Item: Clone + Default,
  141|       |{
  142|       |    /// Owned output buffer retaining units until the wrapped sink accepts them.
  143|       |    output: AsyncBufferedOutput<O>,
  144|       |    /// Minimum spare units requested by the previous encoder progress report.
  145|       |    required_spare: usize,
  146|       |}
  147|       |
  148|       |impl<O> AsyncTranscodeEncodeOutput<O>
  149|       |where
  150|       |    O: AsyncOutput,
  151|       |    O::Item: Clone + Default,
  152|       |{
  153|       |    /// Creates an asynchronous encoder output with the default unit capacity.
  154|       |    ///
  155|       |    /// # Parameters
  156|       |    ///
  157|       |    /// * `inner` - Unit output receiving transcoder data.
  158|       |    ///
  159|       |    /// # Returns
  160|       |    ///
  161|       |    /// Returns a buffered asynchronous output adapter.
  162|       |    ///
  163|       |    /// # Panics
  164|       |    ///
  165|       |    /// Panics if initializing the default buffer requires `O::Item::default()`
  166|       |    /// or `O::Item::clone()` and either operation panics, or if the default
  167|       |    /// backing length exceeds `Vec`'s supported capacity.
  168|     12|    pub fn new(inner: O) -> Self {
  169|     12|        Self {
  170|     12|            output: AsyncBufferedOutput::new(inner),
  171|     12|            required_spare: 1,
  172|     12|        }
  173|     12|    }
  174|       |
  175|       |    /// Creates an asynchronous encoder output with at least `capacity` units.
  176|       |    ///
  177|       |    /// # Parameters
  178|       |    ///
  179|       |    /// * `inner` - Unit output receiving transcoder data.
  180|       |    /// * `capacity` - Requested internal unit capacity.
  181|       |    ///
  182|       |    /// # Returns
  183|       |    ///
  184|       |    /// Returns a buffered asynchronous output adapter.
  185|       |    ///
  186|       |    /// # Panics
  187|       |    ///
  188|       |    /// Panics if initializing the buffer requires `O::Item::default()` or
  189|       |    /// `O::Item::clone()` and either operation panics, or if the requested
  190|       |    /// backing length exceeds `Vec`'s supported capacity.
  191|     14|    pub fn with_capacity(inner: O, capacity: usize) -> Self {
  192|     14|        Self {
  193|     14|            output: AsyncBufferedOutput::with_capacity(inner, capacity),
  194|     14|            required_spare: 1,
  195|     14|        }
  196|     14|    }
  197|       |
  198|       |    /// Tries to create an adapter with at least `capacity` buffered units.
  199|       |    ///
  200|       |    /// # Parameters
  201|       |    ///
  202|       |    /// * `inner` - Output moved into the adapter, including on allocation
  203|       |    ///   failure.
  204|       |    /// * `capacity` - Minimum number of units to reserve.
  205|       |    ///
  206|       |    /// # Returns
  207|       |    ///
  208|       |    /// Returns an empty buffered adapter on successful allocation.
  209|       |    ///
  210|       |    /// # Errors
  211|       |    ///
  212|       |    /// Returns an allocation error when the internal unit buffer cannot be
  213|       |    /// allocated.
  214|       |    ///
  215|       |    /// # Panics
  216|       |    ///
  217|       |    /// Panics if initializing the buffer requires `O::Item::default()` or
  218|       |    /// `O::Item::clone()` and either operation panics.
  219|      2|    pub fn try_with_capacity(inner: O, capacity: usize) -> std::result::Result<Self, TryReserveError> {
  220|       |        Ok(Self {
  221|      2|            output: AsyncBufferedOutput::try_with_capacity(inner, capacity)?,
                                                                                         ^1
  222|       |            required_spare: 1,
  223|       |        })
  224|      2|    }
  225|       |
  226|       |    /// Returns the total internal unit-buffer capacity.
  227|       |    ///
  228|       |    /// # Returns
  229|       |    ///
  230|       |    /// The allocated unit capacity, including occupied and spare slots.
  231|       |    #[must_use]
  232|       |    #[inline]
  233|      6|    pub fn capacity(&self) -> usize {
  234|      6|        self.output.capacity()
  235|      6|    }
  236|       |
  237|       |    /// Returns a shared reference to the wrapped asynchronous output.
  238|       |    ///
  239|       |    /// # Returns
  240|       |    ///
  241|       |    /// Returns the wrapped output. Units can still be pending in this adapter.
  242|       |    #[must_use]
  243|       |    #[inline]
  244|     21|    pub const fn inner(&self) -> &O {
  245|     21|        self.output.inner()
  246|     21|    }
  247|       |
  248|       |    /// Returns a mutable reference to the wrapped asynchronous output.
  249|       |    ///
  250|       |    /// Pending adapter units are not automatically delivered before direct
  251|       |    /// output operations. Call [`Self::flush_async`] first when ordering with
  252|       |    /// previously transcoded data matters.
  253|       |    ///
  254|       |    /// # Returns
  255|       |    ///
  256|       |    /// Returns the wrapped output.
  257|       |    #[must_use]
  258|       |    #[inline]
  259|     15|    pub fn inner_mut(&mut self) -> &mut O {
  260|     15|        self.output.inner_mut()
  261|     15|    }
  262|       |
  263|       |    /// Returns the number of units retained pending delivery.
  264|       |    ///
  265|       |    /// # Returns
  266|       |    ///
  267|       |    /// Returns the length of the pending unit window.
  268|       |    #[must_use]
  269|       |    #[inline]
  270|     21|    pub const fn pending_len(&self) -> usize {
  271|     21|        self.output.pending_len()
  272|     21|    }
  273|       |
  274|       |    /// Consumes this adapter without flushing the wrapped output.
  275|       |    ///
  276|       |    /// This method performs no asynchronous I/O. The returned buffer holds
  277|       |    /// units that must be delivered before the logical stream can continue.
  278|       |    ///
  279|       |    /// Recovery assumes that wrapped I/O calls obey their normal return
  280|       |    /// contracts. If an inner I/O panic is caught, the pending units may
  281|       |    /// overlap a prefix already delivered. The buffer does not track this
  282|       |    /// indeterminate delivery; reconcile external side effects before
  283|       |    /// retrying or replaying those units.
  284|       |    ///
  285|       |    /// # Returns
  286|       |    ///
  287|       |    /// Returns the wrapped output and its pending unit buffer, assuming
  288|       |    /// wrapped I/O calls obey their normal return contracts.
  289|       |    #[must_use = "the returned output and pending buffer must be handled"]
  290|       |    #[inline]
  291|      4|    pub fn into_parts(self) -> (O, Buffer<O::Item>) {
  292|      4|        self.output.into_parts()
  293|      4|    }
  294|       |}
  295|       |
  296|       |impl<O> AsyncTranscodeEncodeOutput<O>
  297|       |where
  298|       |    O: AsyncOutput + Unpin,
  299|       |    O::Item: Clone + Default + Unpin,
  300|       |{
  301|       |    /// Flushes pending transcoded units and the wrapped asynchronous output.
  302|       |    ///
  303|       |    /// # Returns
  304|       |    ///
  305|       |    /// Returns `Ok(())` after pending units are delivered and the sink is
  306|       |    /// flushed.
  307|       |    ///
  308|       |    /// # Errors
  309|       |    ///
  310|       |    /// Returns output-delivery or flush errors from the wrapped output. A
  311|       |    /// failed delivery retains every unit that was not accepted.
  312|     11|    pub async fn flush_async(&mut self) -> Result<()> {
  313|     11|        self.output.flush_async().await
  314|     10|    }
  315|       |
  316|       |    /// Delivers pending transcoded units without flushing the wrapped output.
  317|       |    ///
  318|       |    /// # Returns
  319|       |    ///
  320|       |    /// Returns `Ok(())` when the pending buffer is empty.
  321|       |    ///
  322|       |    /// # Errors
  323|       |    ///
  324|       |    /// Returns output-delivery errors from the wrapped output. A failed
  325|       |    /// delivery retains every unit that was not accepted.
  326|      8|    pub async fn drain_async(&mut self) -> Result<()> {
  327|      8|        let capacity = self.output.capacity();
  328|      8|        self.output.ensure_spare_capacity_async(capacity).await
  329|      8|    }
  330|       |
  331|       |    /// Runs encoder reset and buffers its stream-prefix units.
  332|       |    ///
  333|       |    /// # Type Parameters
  334|       |    ///
  335|       |    /// * `E` - Encoder producing the wrapped output item type.
  336|       |    /// * `M` - Error mapper invoked for encoder failures.
  337|       |    /// * `Value` - Logical input value type accepted by the encoder.
  338|       |    ///
  339|       |    /// # Parameters
  340|       |    ///
  341|       |    /// * `encoder` - Streaming encoder whose lifecycle is being started.
  342|       |    /// * `map_error` - Maps transcoder errors into I/O errors.
  343|       |    ///
  344|       |    /// # Returns
  345|       |    ///
  346|       |    /// Returns `Ok(())` with the reset prefix retained in the buffer.
  347|       |    /// A successful reset discards the previous stream's spare-capacity hint.
  348|       |    /// Previously committed pending units remain buffered in delivery order.
  349|       |    ///
  350|       |    /// # Errors
  351|       |    ///
  352|       |    /// Returns capacity planning, output delivery, allocation, or mapped reset
  353|       |    /// errors. Pending units remain retained if asynchronous delivery fails.
  354|       |    ///
  355|       |    /// # Panics
  356|       |    ///
  357|       |    /// Panics when `encoder` writes more units than its declared reset bound.
  358|     18|    pub async fn reset_async<E, M, Value>(&mut self, encoder: &mut E, map_error: &mut M) -> Result<()>
  359|     18|    where
  360|     18|        E: Transcoder<Input = Value, Output = O::Item>,
  361|     18|        M: FnMut(E::Error) -> Error,
  362|     18|    {
  363|     18|        let required = encoder.max_reset_output_len().map_err(capacity_error_to_invalid_data)?;
                          ^17                                                                              ^1
  364|     17|        self.ensure_spare_capacity_async(required).await?;
                                                                      ^1
  365|     15|        let (units, output_index, available) = self.output.spare_raw_parts_mut();
  366|     15|        debug_assert!(available >= required);
  367|     15|        let written = encoder.reset(units, output_index).map_err(&mut *map_error)?;
                          ^12                                                                  ^3
  368|     12|        assert!(written <= required, "reset wrote beyond its bound");
  369|       |        // SAFETY: `written` is bounded by the reserved spare range above.
  370|     12|        unsafe {
  371|     12|            self.output.advance(written);
  372|     12|        }
  373|     12|        self.required_spare = 1;
  374|     12|        Ok(())
  375|     17|    }
  376|       |
  377|       |    /// Polls one cancellation-safe encoder operation.
  378|       |    ///
  379|       |    /// The adapter may first deliver previously pending output to make a
  380|       |    /// conservative spare range available. Once it invokes `encoder`, it
  381|       |    /// advances the produced units and returns immediately without another
  382|       |    /// poll. The returned [`TranscodeProgress`] is therefore the exact source
  383|       |    /// range committed by this call.
  384|       |    ///
  385|       |    /// # Type Parameters
  386|       |    ///
  387|       |    /// * `E` - Encoder producing the wrapped output item type.
  388|       |    /// * `M` - Error mapper invoked for encoder failures.
  389|       |    /// * `Value` - Logical input value type accepted by the encoder.
  390|       |    ///
  391|       |    /// # Parameters
  392|       |    ///
  393|       |    /// * `cx` - Task context used by the sink to register readiness.
  394|       |    /// * `encoder` - Streaming encoder used for the conversion.
  395|       |    /// * `map_error` - Maps transcoder errors into I/O errors.
  396|       |    /// * `input` - Source values.
  397|       |    /// * `input_index` - First source value to transcode.
  398|       |    /// * `count` - Number of source values available from `input_index`.
  399|       |    ///
  400|       |    /// # Returns
  401|       |    ///
  402|       |    /// Returns `Pending` while making output space, without consuming input,
  403|       |    /// or `Ready` with one validated encoder progress report or an error.
  404|       |    ///
  405|       |    /// # Errors
  406|       |    ///
  407|       |    /// Returns invalid input ranges, output delivery, allocation, contract, or
  408|       |    /// mapped transcoder errors. Pending units remain retained after a failed
  409|       |    /// asynchronous delivery. An encoder error does not provide general
  410|       |    /// transactional rollback or complete progress: the encoder may already
  411|       |    /// have advanced, but units written by its failing call are not committed
  412|       |    /// to this adapter's pending output. Do not replay the original input
  413|       |    /// range unless the concrete encoder provides an explicit recovery
  414|       |    /// protocol. Previously committed pending units follow the wrapped output's
  415|       |    /// recovery contract; reset starts a new logical stream and does not undo
  416|       |    /// output already delivered.
  417|     29|    pub fn poll_transcode<E, M, Value>(
  418|     29|        mut self: Pin<&mut Self>,
  419|     29|        cx: &mut Context<'_>,
  420|     29|        encoder: &mut E,
  421|     29|        map_error: &mut M,
  422|     29|        input: &[Value],
  423|     29|        input_index: usize,
  424|     29|        count: usize,
  425|     29|    ) -> Poll<Result<TranscodeProgress>>
  426|     29|    where
  427|     29|        E: TranscodeEncoder<Input = Value, Output = O::Item>,
  428|     29|        M: FnMut(E::Error) -> Error,
  429|       |    {
  430|     29|        let input_end = SliceRange::checked_range_end(
                          ^28
  431|     29|            input.len(),
  432|     29|            input_index,
  433|     29|            count,
  434|       |            "encode input range exceeds source buffer",
  435|      1|        )?;
  436|     28|        if count == 0 {
  437|      1|            return Poll::Ready(Ok(TranscodeProgress::complete(0, 0)));
  438|     27|        }
  439|     27|        let input = &input[..input_end];
  440|     27|        let this = self.as_mut().get_mut();
  441|     27|        let per_value = match encoder.max_transcode_output_len(1) {
                          ^26
  442|     26|            Ok(required) => required.max(1),
  443|      1|            Err(error) => {
  444|      1|                return Poll::Ready(Err(capacity_error_to_invalid_data(error)));
  445|       |            }
  446|       |        };
  447|     26|        let required_spare = this.required_spare.max(per_value);
  448|     26|        if let Err(error) = this.output.try_reserve_capacity(required_spare) {
                                 ^1
  449|      1|            return Poll::Ready(Err(allocation_error(error)));
  450|     25|        }
  451|     25|        match Pin::new(&mut this.output).poll_ensure_spare_capacity(cx, required_spare) {
  452|     20|            Poll::Ready(Ok(())) => {}
  453|      1|            Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
  454|      4|            Poll::Pending => return Poll::Pending,
  455|       |        }
  456|     20|        let (units, output_index, available_output) = this.output.spare_raw_parts_mut();
  457|     20|        let progress = encoder
  458|     20|            .transcode(input, input_index, units, output_index)
  459|     20|            .map_err(&mut *map_error)
  460|     20|            .and_then(|progress| {
                                               ^18
  461|     18|                validate_encode_progress(progress, input_index, count, output_index, available_output)
  462|     18|            });
  463|     20|        let progress = match progress {
                          ^17
  464|     17|            Ok(progress) => progress,
  465|      3|            Err(error) => return Poll::Ready(Err(error)),
  466|       |        };
  467|       |        // SAFETY: Progress validation proves the initialized output count fits
  468|       |        // the spare window provided to the encoder.
  469|     17|        unsafe {
  470|     17|            this.output.advance(progress.written());
  471|     17|        }
  472|     17|        this.required_spare = if let TranscodeStatus::NeedOutput { required, .. } = progress.status() {
                                                                                 ^5
  473|      5|            required.get()
  474|       |        } else {
  475|       |            // `validate_encode_progress` rejects `NeedInput` before this
  476|       |            // point.
  477|     12|            debug_assert!(progress.is_complete());
  478|     12|            1
  479|       |        };
  480|     17|        Poll::Ready(Ok(progress))
  481|     29|    }
  482|       |
  483|       |    /// Transcodes one cancellation-safe progress step into the retained buffer.
  484|       |    ///
  485|       |    /// This is the async wrapper for [`Self::poll_transcode`]. It returns after
  486|       |    /// one encoder invocation; callers that need to consume a whole source
  487|       |    /// range must advance the source index by [`TranscodeProgress::read`] and
  488|       |    /// call it again.
  489|       |    ///
  490|       |    /// # Type Parameters
  491|       |    ///
  492|       |    /// * `E` - Encoder producing the wrapped output item type.
  493|       |    /// * `M` - Error mapper invoked for encoder failures.
  494|       |    /// * `Value` - Logical input value type accepted by the encoder.
  495|       |    ///
  496|       |    /// # Parameters
  497|       |    ///
  498|       |    /// * `encoder` - Encoder invoked once after output space is available.
  499|       |    /// * `map_error` - Maps encoder errors to I/O errors.
  500|       |    /// * `input` - Source values borrowed for this operation.
  501|       |    /// * `input_index` - Starting source offset.
  502|       |    /// * `count` - Number of source values available at that offset.
  503|       |    ///
  504|       |    /// # Returns
  505|       |    ///
  506|       |    /// One validated progress report describing committed source consumption.
  507|       |    ///
  508|       |    /// # Errors
  509|       |    ///
  510|       |    /// Returns range, capacity, allocation, delivery, contract or mapped
  511|       |    /// encoder errors, with pending units retained as described by
  512|       |    /// [`Self::poll_transcode`].
  513|     25|    pub async fn transcode_async<E, M, Value>(
  514|     25|        &mut self,
  515|     25|        encoder: &mut E,
  516|     25|        map_error: &mut M,
  517|     25|        input: &[Value],
  518|     25|        input_index: usize,
  519|     25|        count: usize,
  520|     25|    ) -> Result<TranscodeProgress>
  521|     25|    where
  522|     25|        E: TranscodeEncoder<Input = Value, Output = O::Item>,
  523|     25|        M: FnMut(E::Error) -> Error,
  524|     25|    {
  525|     28|        poll_fn(|cx| Pin::new(&mut *self).poll_transcode(cx, encoder, map_error, input, input_index, count)).await
                      ^25                                                                                                  ^25
  526|     24|    }
  527|       |
  528|       |    /// Finishes the encoder and retains its final output for delivery.
  529|       |    ///
  530|       |    /// # Type Parameters
  531|       |    ///
  532|       |    /// * `E` - Encoder producing the wrapped output item type.
  533|       |    /// * `M` - Error mapper invoked for encoder failures.
  534|       |    /// * `Value` - Logical input value type accepted by the encoder.
  535|       |    ///
  536|       |    /// # Parameters
  537|       |    ///
  538|       |    /// * `encoder` - Streaming encoder whose lifecycle is being finished.
  539|       |    /// * `map_error` - Maps transcoder errors into I/O errors.
  540|       |    ///
  541|       |    /// # Returns
  542|       |    ///
  543|       |    /// Returns `Ok(())` after final units are committed to the buffer.
  544|       |    ///
  545|       |    /// # Errors
  546|       |    ///
  547|       |    /// Returns capacity planning, output delivery, allocation, or mapped finish
  548|       |    /// errors. Call [`Self::flush_async`] after this method to deliver
  549|       |    /// final units. Keeping finalization separate from delivery lets a
  550|       |    /// caller record the completed lifecycle before any later suspension
  551|       |    /// point.
  552|       |    ///
  553|       |    /// # Panics
  554|       |    ///
  555|       |    /// Panics when `encoder` writes more units than its declared finish bound.
  556|      7|    pub async fn finish_async<E, M, Value>(&mut self, encoder: &mut E, map_error: &mut M) -> Result<()>
  557|      7|    where
  558|      7|        E: Transcoder<Input = Value, Output = O::Item>,
  559|      7|        M: FnMut(E::Error) -> Error,
  560|      7|    {
  561|      7|        let required = encoder
                          ^6
  562|      7|            .max_finish_output_len()
  563|      7|            .map_err(capacity_error_to_invalid_data)?;
                                                                  ^1
  564|      6|        self.ensure_spare_capacity_async(required).await?;
                                                                      ^1
  565|      4|        let (units, output_index, available) = self.output.spare_raw_parts_mut();
  566|      4|        debug_assert!(available >= required);
  567|      4|        let written = encoder.finish(units, output_index).map_err(&mut *map_error)?;
                          ^3                                                                    ^1
  568|      3|        assert!(written <= required, "finish wrote beyond its bound");
  569|       |        // SAFETY: `written` is bounded by the reserved spare range above.
  570|      3|        unsafe {
  571|      3|            self.output.advance(written);
  572|      3|        }
  573|      3|        Ok(())
  574|      6|    }
  575|       |
  576|       |    /// Reserves enough total buffer capacity and makes `count` spare slots
  577|       |    /// available for one transcoder operation. Pending units may be delivered
  578|       |    /// before the codec operation, and an async poll can return `Pending` until
  579|       |    /// the wrapped output accepts them.
  580|       |    ///
  581|       |    /// # Parameters
  582|       |    ///
  583|       |    /// * `count` - Minimum spare units needed after any pending delivery.
  584|       |    ///
  585|       |    /// # Returns
  586|       |    ///
  587|       |    /// `Ok(())` when the requested spare capacity is available.
  588|       |    ///
  589|       |    /// # Errors
  590|       |    ///
  591|       |    /// Returns allocation or wrapped output errors; undelivered units remain
  592|       |    /// buffered.
  593|     23|    async fn ensure_spare_capacity_async(&mut self, count: usize) -> Result<()> {
  594|     23|        self.output.try_reserve_capacity(count).map_err(allocation_error)?;
                                                                                       ^2
  595|     21|        self.output.ensure_spare_capacity_async(count).await
  596|     21|    }
  597|       |}
  598|       |
  599|       |/// Converts capacity planning failures into invalid asynchronous stream data.
  600|       |///
  601|       |/// # Parameters
  602|       |///
  603|       |/// * `error` - Capacity failure retained as the source of the I/O error.
  604|       |///
  605|       |/// # Returns
  606|       |///
  607|       |/// An `InvalidData` error owning the capacity failure.
  608|       |#[must_use]
  609|      3|fn capacity_error_to_invalid_data(error: CapacityError) -> Error {
  610|      3|    Error::new(ErrorKind::InvalidData, error)
  611|      3|}
  612|       |
  613|       |impl<O> fmt::Debug for AsyncTranscodeEncodeOutput<O>
  614|       |where
  615|       |    O: AsyncOutput,
  616|       |    O::Item: Clone + Default,
  617|       |    AsyncBufferedOutput<O>: fmt::Debug,
  618|       |{
  619|       |    /// Formats this asynchronous output adapter for debugging.
  620|       |    ///
  621|       |    /// # Parameters
  622|       |    ///
  623|       |    /// * `formatter` - Destination for the adapter and wrapped buffer
  624|       |    ///   description.
  625|       |    ///
  626|       |    /// # Returns
  627|       |    ///
  628|       |    /// `Ok(())` after the description is written.
  629|       |    ///
  630|       |    /// # Errors
  631|       |    ///
  632|       |    /// Propagates errors reported by the formatter.
  633|      1|    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
  634|      1|        formatter
  635|      1|            .debug_struct("AsyncTranscodeEncodeOutput")
  636|      1|            .field("output", &self.output)
  637|      1|            .finish()
  638|      1|    }
  639|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/io/internal/codec_decode_driver.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Private driver for decoding one codec value from buffered input.
    9|       |
   10|       |use core::num::NonZeroUsize;
   11|       |use std::io::Error;
   12|       |use std::io::ErrorKind;
   13|       |use std::io::Result;
   14|       |
   15|       |use qubit_io::BufferedInput;
   16|       |use qubit_io::Input;
   17|       |use qubit_utils::allocation_error;
   18|       |
   19|       |use crate::Codec;
   20|       |use crate::DecodeFailure;
   21|       |
   22|       |/// Drives one codec decode operation against persistent buffered input.
   23|       |///
   24|       |/// # Type Parameters
   25|       |///
   26|       |/// - `'a`: Lifetime of the mutable borrow of the shared input buffer.
   27|       |/// - `I`: Input implementation whose items are the codec's decode units.
   28|       |pub(in crate::transcode::io) struct CodecDecodeDriver<'a, I>
   29|       |where
   30|       |    I: Input,
   31|       |    I::Item: Copy + Default,
   32|       |{
   33|       |    /// Buffered unit input shared with the public adapter.
   34|       |    input: &'a mut BufferedInput<I>,
   35|       |}
   36|       |
   37|       |impl<'a, I> CodecDecodeDriver<'a, I>
   38|       |where
   39|       |    I: Input,
   40|       |    I::Item: Copy + Default,
   41|       |{
   42|       |    /// Creates a one-value decode driver over `input`.
   43|       |    ///
   44|       |    /// # Parameters
   45|       |    ///
   46|       |    /// - `input`: Buffered unit input shared with the public adapter. The
   47|       |    ///   borrow keeps every unit this driver consumes inside that buffer.
   48|       |    ///
   49|       |    /// # Returns
   50|       |    ///
   51|       |    /// Returns a driver that decodes exactly one value per `read_one` call.
   52|       |    #[inline]
   53|       |    #[must_use]
   54|     79|    pub(in crate::transcode::io) const fn new(input: &'a mut BufferedInput<I>) -> Self {
   55|     79|        Self { input }
   56|     79|    }
   57|       |
   58|       |    /// Reads one decoded value after codec lifecycle reset completed.
   59|       |    ///
   60|       |    /// The driver fills the shared buffer until the codec minimum is
   61|       |    /// available, retries after an incomplete report, and switches to
   62|       |    /// `Codec::decode_eof` once the underlying input is exhausted.
   63|       |    ///
   64|       |    /// # Type Parameters
   65|       |    ///
   66|       |    /// - `C`: Codec that decodes units from the shared input buffer.
   67|       |    /// - `M`: Callback that converts codec errors to I/O errors.
   68|       |    ///
   69|       |    /// # Parameters
   70|       |    ///
   71|       |    /// - `codec`: Codec whose decode entry points read the prepared window.
   72|       |    /// - `map_error`: Converts a codec decode error into the reported I/O
   73|       |    ///   error; it is called at most once per call.
   74|       |    ///
   75|       |    /// # Returns
   76|       |    ///
   77|       |    /// Returns the decoded value after consuming exactly the units the codec
   78|       |    /// reported as read.
   79|       |    ///
   80|       |    /// # Errors
   81|       |    ///
   82|       |    /// Returns `ErrorKind::UnexpectedEof` when the input ends before one
   83|       |    /// complete value is available, `ErrorKind::InvalidData` when the codec
   84|       |    /// reports consumption beyond the unread window or reports incomplete
   85|       |    /// input inside a window that is already large enough, the allocation
   86|       |    /// error when the buffer cannot grow, and the mapped codec error when the
   87|       |    /// codec rejects the input.
   88|       |    ///
   89|       |    /// # Panics
   90|       |    ///
   91|       |    /// Panics when a codec reports `DecodeFailure::Incomplete` with a
   92|       |    /// `required_total` above `Codec::MAX_DECODE_UNITS_PER_VALUE`, which
   93|       |    /// violates the codec contract.
   94|     79|    pub(in crate::transcode::io) fn read_one<C, M>(&mut self, codec: &mut C, map_error: &mut M) -> Result<C::Value>
   95|     79|    where
   96|     79|        C: Codec<Unit = I::Item>,
   97|     79|        M: FnMut(C::DecodeError) -> Error,
   98|       |    {
   99|     79|        let min_units_per_value = C::MIN_UNITS_PER_VALUE;
  100|     79|        let max_units_per_value = C::MAX_DECODE_UNITS_PER_VALUE.max(min_units_per_value);
  101|     79|        self.input
  102|     79|            .try_reserve_capacity(min_units_per_value)
  103|     79|            .map_err(allocation_error)?;
                                                    ^1
  104|       |        loop {
  105|     89|            let (available, end_of_input) = self.prepare_buffered_window(min_units_per_value, max_units_per_value)?;
                               ^78        ^78                                                                                   ^11
  106|     78|            let units = &self.input.unread()[..available];
  107|     78|            debug_assert!(units.len() >= min_units_per_value);
  108|     78|            let decode_result = Self::decode_window(codec, units, end_of_input);
  109|     38|            match decode_result {
  110|     40|                Ok((value, consumed)) => {
  111|     40|                    return self.accept(value, consumed, available);
  112|       |                }
  113|     24|                Err(DecodeFailure::Incomplete { source, required_total }) => {
  114|     24|                    assert!(
  115|     24|                        required_total.get() <= C::MAX_DECODE_UNITS_PER_VALUE,
  116|       |                        "Codec::decode incomplete required_total exceeded Codec::MAX_DECODE_UNITS_PER_VALUE",
  117|       |                    );
  118|     23|                    if end_of_input {
  119|      0|                        return self.handle_eof_incomplete::<C, M>(source, required_total, map_error);
  120|     23|                    }
  121|     23|                    if !self.refill_after_incomplete(required_total, available)? {
                                                                                             ^7
  122|      5|                        return self.retry_decode_at_eof::<C, M>(codec, map_error);
  123|     11|                    }
  124|       |                }
  125|     14|                Err(DecodeFailure::Invalid { source, consumed }) => {
  126|     14|                    return self.reject::<C, M>(source, consumed, available, map_error);
  127|       |                }
  128|       |            }
  129|       |        }
  130|     78|    }
  131|       |
  132|       |    /// Decodes the prepared window using the entry point selected by EOF state.
  133|       |    ///
  134|       |    /// # Parameters
  135|       |    ///
  136|       |    /// - `codec`: Codec that decodes the prepared unit window.
  137|       |    /// - `units`: Readable units starting at the current value boundary.
  138|       |    /// - `end_of_input`: Whether the underlying input has been exhausted.
  139|       |    ///
  140|       |    /// # Returns
  141|       |    ///
  142|       |    /// Returns the codec's decode result without changing the buffered input.
  143|     78|    fn decode_window<C>(
  144|     78|        codec: &mut C,
  145|     78|        units: &[I::Item],
  146|     78|        end_of_input: bool,
  147|     78|    ) -> core::result::Result<(C::Value, NonZeroUsize), DecodeFailure<C::DecodeError>>
  148|     78|    where
  149|     78|        C: Codec<Unit = I::Item>,
  150|       |    {
  151|       |        // SAFETY: the caller prepared at least the codec minimum number of
  152|       |        // readable units, satisfying either decode entry point's preconditions.
  153|       |        unsafe {
  154|     78|            if end_of_input {
  155|      0|                codec.decode_eof(units, 0)
  156|       |            } else {
  157|     78|                codec.decode(units, 0)
  158|       |            }
  159|       |        }
  160|     78|    }
  161|       |
  162|       |    /// Retries decoding with EOF semantics after refill reports exhaustion.
  163|       |    ///
  164|       |    /// # Type Parameters
  165|       |    ///
  166|       |    /// - `C`: Codec that decodes the final buffered window.
  167|       |    /// - `M`: Callback that converts codec errors to I/O errors.
  168|       |    ///
  169|       |    /// # Parameters
  170|       |    ///
  171|       |    /// - `codec`: Codec whose EOF-aware entry point decodes the final window.
  172|       |    /// - `map_error`: Converts codec errors into reported I/O errors.
  173|       |    ///
  174|       |    /// # Errors
  175|       |    ///
  176|       |    /// Returns `ErrorKind::UnexpectedEof` for an unresolved incomplete value
  177|       |    /// without codec context, the mapped codec error when context is present,
  178|       |    /// or `ErrorKind::InvalidData` when the codec reports an invalid
  179|       |    /// consumption count or an already available required width.
  180|      5|    fn retry_decode_at_eof<C, M>(&mut self, codec: &mut C, map_error: &mut M) -> Result<C::Value>
  181|      5|    where
  182|      5|        C: Codec<Unit = I::Item>,
  183|      5|        M: FnMut(C::DecodeError) -> Error,
  184|       |    {
  185|      5|        let available = self.input.unread_len();
  186|      5|        let units = &self.input.unread()[..available];
  187|       |        // SAFETY: the failed streaming decode established that at least the
  188|       |        // codec minimum units remain readable in this unchanged buffer.
  189|      5|        let eof_result = unsafe { codec.decode_eof(units, 0) };
  190|      4|        match eof_result {
  191|      1|            Ok((value, consumed)) => self.accept(value, consumed, available),
  192|      0|            Err(DecodeFailure::Invalid { source, consumed }) => {
  193|      0|                self.reject::<C, M>(source, consumed, available, map_error)
  194|       |            }
  195|      4|            Err(DecodeFailure::Incomplete { source, required_total }) => {
  196|      4|                self.handle_eof_incomplete::<C, M>(source, required_total, map_error)
  197|       |            }
  198|       |        }
  199|      5|    }
  200|       |
  201|       |    /// Prepares the buffered window for one codec decode attempt.
  202|       |    ///
  203|       |    /// The buffer is filled only up to the codec minimum before decoding.
  204|       |    /// Additional units are read only when the codec reports that its current
  205|       |    /// window is incomplete. All unread units are consumed before reporting
  206|       |    /// the unexpected end of input, so the caller never observes a partially
  207|       |    /// drained window.
  208|       |    ///
  209|       |    /// # Parameters
  210|       |    ///
  211|       |    /// - `min_units_per_value`: Smallest unit count one codec attempt may read.
  212|       |    /// - `max_units_per_value`: Largest unit count one codec attempt may read.
  213|       |    ///
  214|       |    /// # Returns
  215|       |    ///
  216|       |    /// Returns the number of leading unread units the codec may read together
  217|       |    /// with whether the underlying input is exhausted, which selects
  218|       |    /// `Codec::decode` or `Codec::decode_eof`.
  219|       |    ///
  220|       |    /// # Errors
  221|       |    ///
  222|       |    /// Returns `ErrorKind::UnexpectedEof` when fewer than `min_units_per_value`
  223|       |    /// units remain after refilling, and propagates buffer fill or allocation
  224|       |    /// failures from the underlying input.
  225|     89|    fn prepare_buffered_window(
  226|     89|        &mut self,
  227|     89|        min_units_per_value: usize,
  228|     89|        max_units_per_value: usize,
  229|     89|    ) -> Result<(usize, bool)> {
  230|     89|        let mut end_of_input = false;
  231|     89|        let available = self.input.unread_len();
  232|     89|        if available < min_units_per_value && !self.input.fill_until(min_units_per_value)? {
                                                             ^69        ^69        ^69                 ^6
  233|      5|            end_of_input = true;
  234|     78|        }
  235|     83|        if self.input.unread_len() < min_units_per_value {
  236|      5|            let available = self.input.unread_len();
  237|       |            // SAFETY: `available` is the current unread length.
  238|      5|            unsafe {
  239|      5|                self.input.consume(available);
  240|      5|            }
  241|      5|            return Err(Error::new(ErrorKind::UnexpectedEof, "failed to decode complete value"));
  242|     78|        }
  243|     78|        Ok((self.input.unread_len().min(max_units_per_value), end_of_input))
  244|     89|    }
  245|       |
  246|       |    /// Accepts a decoded value and consumes its source units.
  247|       |    ///
  248|       |    /// # Type Parameters
  249|       |    ///
  250|       |    /// - `Value`: Decoded value returned after consuming its source units.
  251|       |    ///
  252|       |    /// # Parameters
  253|       |    ///
  254|       |    /// - `value`: Value produced by the codec for the prepared window.
  255|       |    /// - `consumed`: Non-zero unit count the codec reported as read.
  256|       |    /// - `available`: Unit count the codec was allowed to read.
  257|       |    ///
  258|       |    /// # Returns
  259|       |    ///
  260|       |    /// Returns `value` after the consumed units are removed from the buffer.
  261|       |    ///
  262|       |    /// # Errors
  263|       |    ///
  264|       |    /// Returns `ErrorKind::InvalidData` when `consumed` exceeds `available`,
  265|       |    /// because that consumption would move the window past its own bounds.
  266|     41|    fn accept<Value>(&mut self, value: Value, consumed: NonZeroUsize, available: usize) -> Result<Value> {
  267|     41|        if consumed.get() > available {
  268|      5|            return Err(Error::new(
  269|      5|                ErrorKind::InvalidData,
  270|      5|                "codec consumed units exceed unread window",
  271|      5|            ));
  272|     36|        }
  273|       |        // SAFETY: The check above proves `consumed <= available`, and
  274|       |        // `available` came from the current unread window.
  275|     36|        unsafe {
  276|     36|            self.input.consume(consumed.get());
  277|     36|        }
  278|     36|        Ok(value)
  279|     41|    }
  280|       |
  281|       |    /// Refills after the codec reports incomplete input.
  282|       |    ///
  283|       |    /// # Parameters
  284|       |    ///
  285|       |    /// - `required_total`: Total unit count the codec needs to make progress.
  286|       |    /// - `available`: Unit count already readable before this refill.
  287|       |    ///
  288|       |    /// # Returns
  289|       |    ///
  290|       |    /// Returns `true` when the buffer now holds `required_total` units and the
  291|       |    /// decode attempt should be retried, and `false` when the input is
  292|       |    /// exhausted first.
  293|       |    ///
  294|       |    /// # Errors
  295|       |    ///
  296|       |    /// Returns `ErrorKind::InvalidData` when the codec already had
  297|       |    /// `required_total` units readable, which contradicts an incomplete report,
  298|       |    /// and propagates buffer growth and fill failures from the underlying
  299|       |    /// input.
  300|     23|    fn refill_after_incomplete(&mut self, required_total: NonZeroUsize, available: usize) -> Result<bool> {
  301|     23|        let required_total = required_total.get();
  302|     23|        if available >= required_total {
  303|      4|            return Err(Error::new(
  304|      4|                ErrorKind::InvalidData,
  305|      4|                "codec reported incomplete input within available window",
  306|      4|            ));
  307|     19|        }
  308|     19|        self.input
  309|     19|            .try_reserve_capacity(required_total)
  310|     19|            .map_err(allocation_error)?;
                                                    ^1
  311|     18|        self.input.fill_until(required_total)
  312|     23|    }
  313|       |
  314|       |    /// Consumes the unread tail and reports an incomplete value at EOF.
  315|       |    ///
  316|       |    /// # Type Parameters
  317|       |    ///
  318|       |    /// - `C`: Codec whose incomplete decode report is being handled.
  319|       |    /// - `M`: Callback that converts a present codec error to an I/O error.
  320|       |    ///
  321|       |    /// # Parameters
  322|       |    ///
  323|       |    /// - `source`: Optional codec error associated with the incomplete value.
  324|       |    /// - `required_total`: Total unit count the codec needs to decode a value.
  325|       |    /// - `map_error`: Converts a present codec error into an I/O error.
  326|       |    ///
  327|       |    /// # Errors
  328|       |    ///
  329|       |    /// Returns `ErrorKind::InvalidData` if the codec's required width is
  330|       |    /// already available, `ErrorKind::UnexpectedEof` when no codec error is
  331|       |    /// present, or the mapped codec error otherwise.
  332|       |    ///
  333|       |    /// # Panics
  334|       |    ///
  335|       |    /// Panics when `required_total` exceeds the codec's declared maximum.
  336|      4|    fn handle_eof_incomplete<C, M>(
  337|      4|        &mut self,
  338|      4|        source: Option<C::DecodeError>,
  339|      4|        required_total: NonZeroUsize,
  340|      4|        map_error: &mut M,
  341|      4|    ) -> Result<C::Value>
  342|      4|    where
  343|      4|        C: Codec<Unit = I::Item>,
  344|      4|        M: FnMut(C::DecodeError) -> Error,
  345|       |    {
  346|      4|        assert!(
  347|      4|            required_total.get() <= C::MAX_DECODE_UNITS_PER_VALUE,
  348|       |            "Codec::decode_eof incomplete required_total exceeded Codec::MAX_DECODE_UNITS_PER_VALUE",
  349|       |        );
  350|      4|        let available = self.input.unread_len();
  351|      4|        if required_total.get() <= available {
  352|      1|            return Err(Error::new(
  353|      1|                ErrorKind::InvalidData,
  354|      1|                "codec reported incomplete input within available window",
  355|      1|            ));
  356|      3|        }
  357|       |        // SAFETY: `available` is the current unread length.
  358|      3|        unsafe {
  359|      3|            self.input.consume(available);
  360|      3|        }
  361|      3|        match source {
  362|      1|            Some(source) => Err(map_error(source)),
  363|      2|            None => Err(Error::new(ErrorKind::UnexpectedEof, "failed to decode complete value")),
  364|       |        }
  365|      4|    }
  366|       |
  367|       |    /// Rejects invalid codec input and applies its consumption hint.
  368|       |    ///
  369|       |    /// The optional consumption hint is honored first so the mapped error
  370|       |    /// describes a window that already advanced past the rejected units.
  371|       |    ///
  372|       |    /// # Type Parameters
  373|       |    ///
  374|       |    /// - `C`: Codec that reported invalid input.
  375|       |    /// - `M`: Callback that converts the codec error to an I/O error.
  376|       |    ///
  377|       |    /// # Parameters
  378|       |    ///
  379|       |    /// - `source`: Codec decode error reported for the prepared window.
  380|       |    /// - `consumed`: Optional unit count the codec reported as read before
  381|       |    ///   failing.
  382|       |    /// - `available`: Unit count the codec was allowed to read.
  383|       |    /// - `map_error`: Converts `source` into the reported I/O error.
  384|       |    ///
  385|       |    /// # Errors
  386|       |    ///
  387|       |    /// Always returns an error. Returns `ErrorKind::InvalidData` when the
  388|       |    /// consumption hint exceeds `available` without calling `map_error`,
  389|       |    /// otherwise the error produced by `map_error`.
  390|     14|    fn reject<C, M>(
  391|     14|        &mut self,
  392|     14|        source: C::DecodeError,
  393|     14|        consumed: Option<NonZeroUsize>,
  394|     14|        available: usize,
  395|     14|        map_error: &mut M,
  396|     14|    ) -> Result<C::Value>
  397|     14|    where
  398|     14|        C: Codec<Unit = I::Item>,
  399|     14|        M: FnMut(C::DecodeError) -> Error,
  400|       |    {
  401|     14|        if let Some(consumed) = consumed {
                                  ^10
  402|     10|            if consumed.get() > available {
  403|      4|                return Err(Error::new(
  404|      4|                    ErrorKind::InvalidData,
  405|      4|                    "decode error consumed units exceed unread window",
  406|      4|                ));
  407|      6|            }
  408|       |            // SAFETY: The check above proves `consumed <= available`, and
  409|       |            // `available` came from the current unread window.
  410|      6|            unsafe {
  411|      6|                self.input.consume(consumed.get());
  412|      6|            }
  413|      4|        }
  414|     10|        Err(map_error(source))
  415|     14|    }
  416|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/io/internal/encode_output_error.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Error translation helpers for buffered encoder outputs.
    9|       |
   10|       |use core::result::Result as CoreResult;
   11|       |use std::io::Error;
   12|       |use std::io::ErrorKind;
   13|       |use std::io::Result;
   14|       |
   15|       |use crate::CapacityError;
   16|       |use crate::TranscodeEncodeError;
   17|       |
   18|       |/// Wraps a streaming capacity failure as `InvalidData`, retaining its source.
   19|       |///
   20|       |/// # Parameters
   21|       |///
   22|       |/// * `error` - Capacity planning failure from a streaming transcoder.
   23|       |///
   24|       |/// # Returns
   25|       |///
   26|       |/// An owned I/O error carrying the original capacity error.
   27|       |#[must_use]
   28|      1|pub(in crate::transcode::io) fn capacity_error_to_invalid_data(error: CapacityError) -> Error {
   29|      1|    Error::new(ErrorKind::InvalidData, error)
   30|      1|}
   31|       |
   32|       |/// Preserves successful values and maps one-value failures into I/O errors.
   33|       |///
   34|       |/// # Type Parameters
   35|       |///
   36|       |/// * `T` - Successful result value.
   37|       |/// * `E` - Codec-domain error passed to the mapper.
   38|       |/// * `Value` - Source value type carried by an unencodable error.
   39|       |///
   40|       |/// # Parameters
   41|       |///
   42|       |/// * `result` - Codec lifecycle result to translate.
   43|       |/// * `map_error` - Called only for a codec-domain failure.
   44|       |///
   45|       |/// # Returns
   46|       |///
   47|       |/// The original successful value without transformation.
   48|       |///
   49|       |/// # Errors
   50|       |///
   51|       |/// Returns mapped domain errors or `InvalidInput` for bounds and domain
   52|       |/// rejection.
   53|     31|pub(in crate::transcode::io) fn map_encode_value_result<T, E, Value>(
   54|     31|    result: CoreResult<T, TranscodeEncodeError<E, Value>>,
   55|     31|    map_error: &mut dyn FnMut(E) -> Error,
   56|     31|) -> Result<T> {
   57|     31|    match result {
   58|     26|        Ok(value) => Ok(value),
   59|      5|        Err(error) => Err(map_encode_value_error(error, map_error)),
   60|       |    }
   61|     31|}
   62|       |
   63|       |/// Maps domain failures through the caller and other failures to
   64|       |/// `InvalidInput`.
   65|       |///
   66|       |/// # Type Parameters
   67|       |///
   68|       |/// * `E` - Codec-domain error passed to the mapper.
   69|       |/// * `Value` - Rejected source value type, discarded during I/O translation.
   70|       |///
   71|       |/// # Parameters
   72|       |///
   73|       |/// * `error` - Encoding failure consumed by this translation.
   74|       |/// * `map_error` - Invoked only for the `Domain` variant.
   75|       |///
   76|       |/// # Returns
   77|       |///
   78|       |/// An I/O error retaining mapped domain detail or a bound/rejection message.
   79|       |#[must_use]
   80|       |#[inline(never)]
   81|      5|fn map_encode_value_error<E, Value>(
   82|      5|    error: TranscodeEncodeError<E, Value>,
   83|      5|    map_error: &mut dyn FnMut(E) -> Error,
   84|      5|) -> Error {
   85|      5|    match error {
   86|      3|        TranscodeEncodeError::Domain(error) => map_error(error.into_source()),
   87|      1|        TranscodeEncodeError::Unencodable { .. } => Error::new(ErrorKind::InvalidInput, "codec cannot encode value"),
   88|      1|        TranscodeEncodeError::Failure(_) => Error::new(ErrorKind::InvalidInput, "codec output bound overflow"),
   89|       |    }
   90|      5|}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/io/internal/transcode_progress_validation.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Validation shared by transcode I/O adapters.
    9|       |
   10|       |use std::io::Error;
   11|       |use std::io::ErrorKind;
   12|       |use std::io::Result;
   13|       |
   14|       |use crate::TranscodeProgress;
   15|       |use crate::TranscodeStatus;
   16|       |
   17|       |/// Validates a decoder progress report before the input adapter commits it.
   18|       |///
   19|       |/// # Parameters
   20|       |///
   21|       |/// - `progress`: Report returned by the transcoder for the current call.
   22|       |/// - `input_index`: Absolute input index at the start of that call.
   23|       |/// - `available_input`: Input units available from the starting index.
   24|       |/// - `output_index`: Absolute output index at the start of that call.
   25|       |/// - `available_output`: Writable units available from the starting index.
   26|       |///
   27|       |/// # Returns
   28|       |///
   29|       |/// Returns the unchanged report when its counts and status satisfy the
   30|       |/// contract.
   31|       |///
   32|       |/// # Errors
   33|       |///
   34|       |/// Returns [`ErrorKind::InvalidData`] if the report violates progress bounds
   35|       |/// or status invariants. No adapter state is changed by this validation.
   36|     57|pub(in crate::transcode::io) fn validate_decode_progress(
   37|     57|    progress: TranscodeProgress,
   38|     57|    input_index: usize,
   39|     57|    available_input: usize,
   40|     57|    output_index: usize,
   41|     57|    available_output: usize,
   42|     57|) -> Result<TranscodeProgress> {
   43|     57|    progress
   44|     57|        .validate(input_index, available_input, output_index, available_output)
   45|     57|        .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
                                       ^7         ^7                      ^7     ^7
   46|     50|    Ok(progress)
   47|     57|}
   48|       |
   49|       |/// Validates EOF progress before the input adapter commits source consumption.
   50|       |///
   51|       |/// # Parameters
   52|       |///
   53|       |/// - `progress`: Report returned by the transcoder for the EOF call.
   54|       |/// - `input_index`: Absolute input index at the start of that call.
   55|       |/// - `available_input`: Input units available from the starting index.
   56|       |/// - `output_index`: Absolute output index at the start of that call.
   57|       |/// - `available_output`: Writable units available from the starting index.
   58|       |///
   59|       |/// # Returns
   60|       |///
   61|       |/// Returns the unchanged report when its counts and status satisfy the EOF
   62|       |/// contract.
   63|       |///
   64|       |/// # Errors
   65|       |///
   66|       |/// Returns `InvalidData` for invalid counts, status invariants, or `NeedInput`.
   67|       |/// The adapter has not committed source consumption, but this check does not
   68|       |/// roll back decoder state or caller-owned output.
   69|     24|pub(in crate::transcode::io) fn validate_eof_decode_progress(
   70|     24|    progress: TranscodeProgress,
   71|     24|    input_index: usize,
   72|     24|    available_input: usize,
   73|     24|    output_index: usize,
   74|     24|    available_output: usize,
   75|     24|) -> Result<TranscodeProgress> {
   76|     24|    let progress = validate_decode_progress(progress, input_index, available_input, output_index, available_output)?;
                                                                                                                                 ^0
   77|     24|    if matches!(progress.status(), TranscodeStatus::NeedInput { .. }) {
                     ^19
   78|      5|        return Err(Error::new(
   79|      5|            ErrorKind::InvalidData,
   80|      5|            "decoder violated the EOF contract by requesting more input",
   81|      5|        ));
   82|     19|    }
   83|     19|    Ok(progress)
   84|     24|}
   85|       |
   86|       |/// Validates an encoder progress report before the output adapter commits it.
   87|       |///
   88|       |/// # Parameters
   89|       |///
   90|       |/// - `progress`: Report returned by the transcoder for the current call.
   91|       |/// - `input_index`: Absolute input index at the start of that call.
   92|       |/// - `available_input`: Input units available from the starting index.
   93|       |/// - `output_index`: Absolute output index at the start of that call.
   94|       |/// - `available_output`: Writable units available from the starting index.
   95|       |///
   96|       |/// # Returns
   97|       |///
   98|       |/// Returns the unchanged report when its counts and status satisfy the
   99|       |/// contract.
  100|       |///
  101|       |/// # Errors
  102|       |///
  103|       |/// Returns [`ErrorKind::InvalidData`] if the report violates progress bounds
  104|       |/// or status invariants, or requests more input. This changes no adapter state.
  105|  9.25k|pub(in crate::transcode::io) fn validate_encode_progress(
  106|  9.25k|    progress: TranscodeProgress,
  107|  9.25k|    input_index: usize,
  108|  9.25k|    available_input: usize,
  109|  9.25k|    output_index: usize,
  110|  9.25k|    available_output: usize,
  111|  9.25k|) -> Result<TranscodeProgress> {
  112|  9.25k|    progress
  113|  9.25k|        .validate(input_index, available_input, output_index, available_output)
  114|  9.25k|        .map_err(|error| Error::new(ErrorKind::InvalidData, error))?;
                                       ^6         ^6                      ^6     ^6
  115|  9.24k|    if matches!(progress.status(), TranscodeStatus::NeedInput { .. }) {
                     ^9.24k
  116|      3|        return Err(Error::new(
  117|      3|            ErrorKind::InvalidData,
  118|      3|            "encoder violated the TranscodeEncoder contract by requesting more input",
  119|      3|        ));
  120|  9.24k|    }
  121|  9.24k|    Ok(progress)
  122|  9.25k|}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/io/transcode_decode_input.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Buffered input driver that decodes units into values.
    9|       |
   10|       |use core::fmt::Debug;
   11|       |use core::fmt::Formatter;
   12|       |use core::fmt::Result as FormatResult;
   13|       |use std::collections::TryReserveError;
   14|       |use std::io::Error;
   15|       |use std::io::ErrorKind;
   16|       |use std::io::Read;
   17|       |use std::io::Result;
   18|       |use std::io::Seek;
   19|       |use std::io::SeekFrom;
   20|       |use std::result::Result as StdResult;
   21|       |
   22|       |use qubit_io::Buffer;
   23|       |use qubit_io::BufferedInput;
   24|       |use qubit_io::ByteInput;
   25|       |use qubit_io::Input;
   26|       |use qubit_io::SeekableByteInput;
   27|       |use qubit_utils::SliceRange;
   28|       |use qubit_utils::UncheckedSlice;
   29|       |use qubit_utils::allocation_error;
   30|       |use qubit_utils::try_reserve_vec;
   31|       |
   32|       |use super::internal::CodecDecodeDriver;
   33|       |use super::internal::validate_decode_progress;
   34|       |use super::internal::validate_eof_decode_progress;
   35|       |use crate::CapacityError;
   36|       |use crate::Codec;
   37|       |use crate::DecodeLifecycleOutput;
   38|       |use crate::DecodeLifecycleProgress;
   39|       |use crate::TranscodeFailure;
   40|       |use crate::TranscodeProgress;
   41|       |use crate::TranscodeStatus;
   42|       |use crate::Transcoder;
   43|       |use crate::codec::assert_unit_bounds;
   44|       |
   45|       |/// Decodes an [`Input`] unit stream into an [`Input`] value stream.
   46|       |///
   47|       |/// This type owns only the unit-level [`qubit_io::BufferedInput`]. Callers pass
   48|       |/// a streaming [`Transcoder`] and error mapper to each decode operation, which
   49|       |/// lets one buffered input drive different decoders without nesting buffers or
   50|       |/// storing decoder-specific state in the buffer owner.
   51|       |///
   52|       |/// # Type Parameters
   53|       |///
   54|       |/// * `I` - Wrapped unit input.
   55|       |///
   56|       |/// # Examples
   57|       |///
   58|       |/// ```
   59|       |/// use core::convert::Infallible;
   60|       |/// use core::num::NonZeroUsize;
   61|       |/// use std::io::Cursor;
   62|       |/// use qubit_codec::{Codec, DecodeFailure, TranscodeDecodeInput};
   63|       |///
   64|       |/// struct Identity;
   65|       |/// impl Codec for Identity {
   66|       |///     type Value = u8;
   67|       |///     type Unit = u8;
   68|       |///     type DecodeError = Infallible;
   69|       |///     type EncodeError = Infallible;
   70|       |///     const MIN_UNITS_PER_VALUE: usize = 1;
   71|       |///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
   72|       |///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
   73|       |///     unsafe fn decode(&mut self, input: &[u8], index: usize)
   74|       |///         -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
   75|       |///         Ok((input[index], NonZeroUsize::MIN))
   76|       |///     }
   77|       |///     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize)
   78|       |///         -> Result<usize, Infallible> {
   79|       |///         output[index] = *value;
   80|       |///         Ok(1)
   81|       |///     }
   82|       |/// }
   83|       |///
   84|       |/// let mut input = TranscodeDecodeInput::with_capacity(Cursor::new(vec![42_u8]), 8);
   85|       |/// assert!(input.fill_until(1).expect("refilling the cursor input should succeed"));
   86|       |/// assert_eq!(input.unread(), &[42]);
   87|       |/// let value = input.read_decoded_with(&mut Identity, |error| match error {})?;
   88|       |/// assert_eq!(value, 42);
   89|       |/// # Ok::<(), std::io::Error>(())
   90|       |/// ```
   91|       |///
   92|       |/// `read_decoded_with` reads one complete value and owns its strict
   93|       |/// single-value lifecycle; use the step methods when the caller needs to
   94|       |/// coordinate streaming input and output itself.
   95|       |pub struct TranscodeDecodeInput<I>
   96|       |where
   97|       |    I: Input,
   98|       |    I::Item: Copy + Default,
   99|       |{
  100|       |    /// Owns the unit source and retains unread units across decode operations.
  101|       |    input: BufferedInput<I>,
  102|       |}
  103|       |
  104|       |impl<I> TranscodeDecodeInput<I>
  105|       |where
  106|       |    I: Input,
  107|       |    I::Item: Copy + Default,
  108|       |{
  109|       |    /// Creates a decoder input with the default unit buffer capacity.
  110|       |    ///
  111|       |    /// # Parameters
  112|       |    ///
  113|       |    /// * `inner` - Unit input read by this adapter.
  114|       |    ///
  115|       |    /// # Returns
  116|       |    ///
  117|       |    /// A new buffered decoder input.
  118|       |    ///
  119|       |    /// # Panics
  120|       |    ///
  121|       |    /// Panics if `I::Item::default()` panics or the default buffer capacity
  122|       |    /// exceeds the supported allocation size.
  123|       |    #[must_use]
  124|       |    #[inline]
  125|     13|    pub fn new(inner: I) -> Self {
  126|     13|        Self {
  127|     13|            input: BufferedInput::new(inner),
  128|     13|        }
  129|     13|    }
  130|       |
  131|       |    /// Creates a decoder input with a unit buffer of at least `capacity`.
  132|       |    ///
  133|       |    /// # Parameters
  134|       |    ///
  135|       |    /// * `inner` - Unit input read by this adapter.
  136|       |    /// * `capacity` - Requested internal unit buffer capacity.
  137|       |    ///
  138|       |    /// # Returns
  139|       |    ///
  140|       |    /// A new buffered decoder input.
  141|       |    ///
  142|       |    /// # Panics
  143|       |    ///
  144|       |    /// Panics if `I::Item::default()` panics or the requested capacity exceeds
  145|       |    /// the supported allocation size.
  146|       |    #[must_use]
  147|       |    #[inline]
  148|    118|    pub fn with_capacity(inner: I, capacity: usize) -> Self {
  149|    118|        Self {
  150|    118|            input: BufferedInput::with_capacity(inner, capacity),
  151|    118|        }
  152|    118|    }
  153|       |
  154|       |    /// Creates a decoder input with a unit buffer of at least `capacity`.
  155|       |    ///
  156|       |    /// # Parameters
  157|       |    ///
  158|       |    /// * `inner` - Unit input read by this adapter.
  159|       |    /// * `capacity` - Requested internal unit buffer capacity.
  160|       |    ///
  161|       |    /// # Returns
  162|       |    ///
  163|       |    /// Returns a buffered adapter owning `inner` with at least the requested
  164|       |    /// capacity.
  165|       |    ///
  166|       |    /// # Errors
  167|       |    ///
  168|       |    /// Returns an allocation error when the requested buffer cannot be
  169|       |    /// allocated.
  170|       |    ///
  171|       |    /// # Panics
  172|       |    ///
  173|       |    /// Panics if `I::Item::default()` panics while initializing the buffer.
  174|      1|    pub fn try_with_capacity(inner: I, capacity: usize) -> StdResult<Self, TryReserveError> {
  175|       |        Ok(Self {
  176|      1|            input: BufferedInput::try_with_capacity(inner, capacity)?,
                                                                                  ^0
  177|       |        })
  178|      1|    }
  179|       |
  180|       |    /// Returns a shared reference to the wrapped unit input.
  181|       |    ///
  182|       |    /// The wrapped input can be physically positioned after units that remain
  183|       |    /// unread in this adapter's internal buffer.
  184|       |    ///
  185|       |    /// # Returns
  186|       |    ///
  187|       |    /// A shared reference to the wrapped unit input.
  188|       |    #[must_use]
  189|       |    #[inline]
  190|     11|    pub const fn inner(&self) -> &I {
  191|     11|        self.input.inner()
  192|     11|    }
  193|       |
  194|       |    /// Returns the number of unread units currently buffered.
  195|       |    ///
  196|       |    /// # Returns
  197|       |    ///
  198|       |    /// The number of unread units in the internal buffer.
  199|       |    #[must_use]
  200|       |    #[inline]
  201|    103|    pub fn unread_len(&self) -> usize {
  202|    103|        self.input.unread_len()
  203|    103|    }
  204|       |
  205|       |    /// Returns the currently buffered unread units.
  206|       |    ///
  207|       |    /// # Returns
  208|       |    ///
  209|       |    /// Returns a shared slice over the unread portion of the internal unit
  210|       |    /// buffer. The slice is valid until this adapter is mutated.
  211|       |    #[must_use]
  212|       |    #[inline]
  213|     98|    pub fn unread(&self) -> &[I::Item] {
  214|     98|        self.input.unread()
  215|     98|    }
  216|       |
  217|       |    /// Returns the internal unit buffer capacity.
  218|       |    ///
  219|       |    /// # Returns
  220|       |    ///
  221|       |    /// The maximum number of units retained in the internal buffer.
  222|       |    #[must_use]
  223|       |    #[inline]
  224|      3|    pub fn capacity(&self) -> usize {
  225|      3|        self.input.capacity()
  226|      3|    }
  227|       |
  228|       |    /// Refills the internal buffer until at least `count` unread units are
  229|       |    /// available.
  230|       |    ///
  231|       |    /// # Parameters
  232|       |    ///
  233|       |    /// * `count` - Minimum number of unread units required.
  234|       |    ///
  235|       |    /// # Returns
  236|       |    ///
  237|       |    /// Returns `true` when at least `count` units are buffered; `false` if EOF
  238|       |    /// prevents reaching that count. Buffered units remain available in either
  239|       |    /// case.
  240|       |    ///
  241|       |    /// # Errors
  242|       |    ///
  243|       |    /// Returns allocation errors mapped to [`ErrorKind::OutOfMemory`], or I/O
  244|       |    /// errors from the wrapped input while refilling.
  245|     30|    pub fn fill_until(&mut self, count: usize) -> Result<bool> {
  246|     30|        if count > self.input.capacity() {
  247|      4|            self.input.try_reserve_capacity(count).map_err(allocation_error)?;
                                                                                          ^1
  248|     26|        }
  249|     29|        self.input.fill_until(count)
  250|     30|    }
  251|       |
  252|       |    /// Performs one decode step and preserves the transcoder stop status.
  253|       |    ///
  254|       |    /// `None` means that the wrapped input reached EOF with no buffered units.
  255|       |    /// Consumed input is committed before this method returns. Callers can use
  256|       |    /// the returned [`TranscodeProgress`] to distinguish `NeedOutput` from an
  257|       |    /// incomplete input tail without guessing from the unread buffer alone.
  258|       |    ///
  259|       |    /// # Type Parameters
  260|       |    ///
  261|       |    /// - `D`: Streaming decoder accepting the source unit type.
  262|       |    /// - `M`: Mapper from decoder errors to I/O errors.
  263|       |    /// - `Value`: Decoded destination value type.
  264|       |    ///
  265|       |    /// # Parameters
  266|       |    ///
  267|       |    /// - `decoder`: Decoder for the current logical stream.
  268|       |    /// - `map_error`: Mapper invoked for decoder errors.
  269|       |    /// - `output`: Destination value storage.
  270|       |    /// - `output_index`: Start of the writable range.
  271|       |    /// - `count`: Maximum number of destination values available.
  272|       |    ///
  273|       |    /// # Returns
  274|       |    ///
  275|       |    /// Returns `Some` validated progress after a decode step, or `None` at EOF
  276|       |    /// with no buffered units. A zero count returns `Some` zero progress.
  277|       |    ///
  278|       |    /// # Errors
  279|       |    ///
  280|       |    /// Returns invalid output-range or progress errors, input/refill errors,
  281|       |    /// or decoder errors translated by `map_error`.
  282|       |    /// A successful validated progress commits its `read()` count. If decoding
  283|       |    /// or progress validation fails, this call does not consume the unread
  284|       |    /// range. The decoder and destination may already have changed; neither is
  285|       |    /// rolled back, and no complete progress report is available. A refill
  286|       |    /// performed before the failure is not rolled back at the wrapped input.
  287|       |    /// Do not replay the original input unless the decoder defines an explicit
  288|       |    /// recovery protocol.
  289|      6|    pub fn transcode_step<D, M, Value>(
  290|      6|        &mut self,
  291|      6|        decoder: &mut D,
  292|      6|        map_error: &mut M,
  293|      6|        output: &mut [Value],
  294|      6|        output_index: usize,
  295|      6|        count: usize,
  296|      6|    ) -> Result<Option<TranscodeProgress>>
  297|      6|    where
  298|      6|        D: Transcoder<Input = I::Item, Output = Value>,
  299|      6|        M: FnMut(D::Error) -> Error,
  300|       |    {
  301|      6|        let output_end = SliceRange::checked_range_end(
                          ^5
  302|      6|            output.len(),
  303|      6|            output_index,
  304|      6|            count,
  305|       |            "decoded output range exceeds destination buffer",
  306|      1|        )?;
  307|      5|        if count == 0 {
  308|      1|            return Ok(Some(TranscodeProgress::complete(0, 0)));
  309|      4|        }
  310|      4|        if self.unread_len() == 0 && !self.fill_more()? {
                                                    ^2   ^2         ^0
  311|      1|            return Ok(None);
  312|      3|        }
  313|      3|        let available_input = self.unread_len();
  314|      3|        let progress = decoder
                          ^1
  315|      3|            .transcode(self.unread(), 0, &mut output[..output_end], output_index)
  316|      3|            .map_err(&mut *map_error)
  317|      3|            .and_then(|progress| validate_decode_progress(progress, 0, available_input, output_index, count))?;
                                               ^2                       ^2           ^2               ^2            ^2     ^2
  318|      1|        self.consume(progress.read());
  319|      1|        Ok(Some(progress))
  320|      6|    }
  321|       |
  322|       |    /// Performs one decode step after the caller has established EOF.
  323|       |    ///
  324|       |    /// This method does not read from the wrapped input. It passes the current
  325|       |    /// unread buffer to [`Transcoder::transcode_eof`], validates the returned
  326|       |    /// progress, and commits consumed source units before returning.
  327|       |    ///
  328|       |    /// # Type Parameters
  329|       |    ///
  330|       |    /// - `D`: Streaming decoder accepting the source unit type.
  331|       |    /// - `M`: Mapper from decoder errors to I/O errors.
  332|       |    /// - `Value`: Decoded destination value type.
  333|       |    ///
  334|       |    /// # Parameters
  335|       |    ///
  336|       |    /// - `decoder`: Decoder for the current logical stream.
  337|       |    /// - `map_error`: Mapper invoked for decoder errors.
  338|       |    /// - `output`: Destination value storage.
  339|       |    /// - `output_index`: Start of the writable range.
  340|       |    /// - `count`: Maximum number of destination values available.
  341|       |    ///
  342|       |    /// # Returns
  343|       |    ///
  344|       |    /// Returns validated EOF progress. A zero output count returns zero
  345|       |    /// progress without calling the decoder; an empty input is still passed to
  346|       |    /// `transcode_eof` so decoders can handle EOF with retained state.
  347|       |    ///
  348|       |    /// # Errors
  349|       |    ///
  350|       |    /// Returns invalid output-range errors, decoder errors mapped by
  351|       |    /// `map_error`, or `InvalidData` if the decoder requests more input after
  352|       |    /// EOF. Invalid progress does not commit source consumption, but decoder
  353|       |    /// state and caller-owned output are not rolled back.
  354|     21|    pub fn transcode_eof_step<D, M, Value>(
  355|     21|        &mut self,
  356|     21|        decoder: &mut D,
  357|     21|        map_error: &mut M,
  358|     21|        output: &mut [Value],
  359|     21|        output_index: usize,
  360|     21|        count: usize,
  361|     21|    ) -> Result<TranscodeProgress>
  362|     21|    where
  363|     21|        D: Transcoder<Input = I::Item, Output = Value>,
  364|     21|        M: FnMut(D::Error) -> Error,
  365|       |    {
  366|     21|        let output_end = SliceRange::checked_range_end(
                          ^20
  367|     21|            output.len(),
  368|     21|            output_index,
  369|     21|            count,
  370|       |            "decoded EOF output range exceeds destination buffer",
  371|      1|        )?;
  372|     20|        if count == 0 {
  373|      2|            return Ok(TranscodeProgress::complete(0, 0));
  374|     18|        }
  375|     18|        let available_input = self.unread_len();
  376|     18|        let progress = decoder
                          ^12
  377|     18|            .transcode_eof(self.unread(), 0, &mut output[..output_end], output_index)
  378|     18|            .map_err(&mut *map_error)
  379|     18|            .and_then(|progress| validate_eof_decode_progress(progress, 0, available_input, output_index, count))?;
                                               ^15                          ^15          ^15              ^15           ^15    ^6
  380|     12|        self.consume(progress.read());
  381|     12|        Ok(progress)
  382|     21|    }
  383|       |
  384|       |    /// Consumes unread units from the current buffer window.
  385|       |    ///
  386|       |    /// # Parameters
  387|       |    ///
  388|       |    /// * `count` - Number of unread units to discard.
  389|       |    ///
  390|       |    /// # Panics
  391|       |    ///
  392|       |    /// Panics when `count` exceeds [`Self::unread_len`].
  393|       |    #[inline]
  394|     36|    pub fn consume(&mut self, count: usize) {
  395|     36|        assert!(count <= self.unread_len(), "cannot consume beyond buffered input",);
  396|       |        // SAFETY: The caller-provided count is within the unread window.
  397|     35|        unsafe {
  398|     35|            self.input.consume(count);
  399|     35|        }
  400|     35|    }
  401|       |
  402|       |    /// Copies unread units into an indexed output range without consuming them.
  403|       |    ///
  404|       |    /// # Parameters
  405|       |    ///
  406|       |    /// * `output` - Destination storage that receives a copy of unread units.
  407|       |    /// * `output_index` - Start index inside `output`.
  408|       |    /// * `count` - Number of unread units to copy.
  409|       |    ///
  410|       |    /// # Safety
  411|       |    ///
  412|       |    /// The caller must guarantee that `output_index..output_index + count` is
  413|       |    /// a valid range inside `output`, that the addition does not overflow, that
  414|       |    /// `count <= self.unread_len()`, and that the destination range does not
  415|       |    /// overlap with the unread units stored inside this buffer.
  416|      2|    pub unsafe fn copy_unread_to(&self, output: &mut [I::Item], output_index: usize, count: usize) {
  417|       |        // SAFETY: The caller guarantees the destination range and non-overlap
  418|       |        // requirements for the unread copy.
  419|      2|        let unread = self.unread();
  420|      2|        unsafe {
  421|      2|            UncheckedSlice::copy_nonoverlapping(unread, 0, output, output_index, count);
  422|      2|        }
  423|      2|    }
  424|       |
  425|       |    /// Consumes this adapter and returns its parts.
  426|       |    ///
  427|       |    /// # Returns
  428|       |    ///
  429|       |    /// The wrapped input and the buffer holding unread units.
  430|       |    #[must_use = "the returned input and unread buffer must be handled"]
  431|       |    #[inline]
  432|      4|    pub fn into_parts(self) -> (I, Buffer<I::Item>) {
  433|      4|        self.input.into_parts()
  434|      4|    }
  435|       |
  436|       |    /// Reads buffered units into an indexed output range.
  437|       |    ///
  438|       |    /// # Parameters
  439|       |    ///
  440|       |    /// * `output` - Destination unit storage.
  441|       |    /// * `output_index` - Start index inside `output`.
  442|       |    /// * `count` - Maximum number of units to read.
  443|       |    ///
  444|       |    /// # Returns
  445|       |    ///
  446|       |    /// The number of units copied into `output`.
  447|       |    ///
  448|       |    /// # Errors
  449|       |    ///
  450|       |    /// Returns input or buffer validation errors from the wrapped
  451|       |    /// [`qubit_io::BufferedInput`].
  452|       |    ///
  453|       |    /// # Panics
  454|       |    ///
  455|       |    /// Debug builds panic if the destination range violates the safety
  456|       |    /// contract.
  457|       |    ///
  458|       |    /// # Safety
  459|       |    ///
  460|       |    /// The caller must guarantee that `output_index..output_index + count` is
  461|       |    /// a valid range inside `output` and that the addition does not overflow.
  462|      9|    pub unsafe fn read_unchecked(
  463|      9|        &mut self,
  464|      9|        output: &mut [I::Item],
  465|      9|        output_index: usize,
  466|      9|        count: usize,
  467|      9|    ) -> Result<usize> {
  468|      9|        debug_assert!(
  469|      9|            SliceRange::range_fits(output.len(), output_index, count),
  470|       |            "unchecked read output range exceeds destination buffer",
  471|       |        );
  472|      9|        if count == 0 {
  473|      1|            return Ok(0);
  474|      8|        }
  475|       |        // SAFETY: The caller guarantees the destination range is valid.
  476|      8|        unsafe { self.input.read_unchecked(output, output_index, count) }
  477|      9|    }
  478|       |
  479|       |    /// Decodes one codec value from the buffered unit input.
  480|       |    ///
  481|       |    /// This strict convenience method supports codecs whose decode reset and
  482|       |    /// finish phases do not emit values. Use
  483|       |    /// [`Self::read_decoded_lifecycle_with`] when those phases have output.
  484|       |    ///
  485|       |    /// # Type Parameters
  486|       |    ///
  487|       |    /// - `C`: Codec accepting the wrapped input's unit type.
  488|       |    /// - `M`: Mapper from codec decode errors to I/O errors.
  489|       |    ///
  490|       |    /// # Parameters
  491|       |    ///
  492|       |    /// * `codec` - Codec used for this single-value decode.
  493|       |    /// * `map_error` - Mapper for codec-domain invalid-input errors.
  494|       |    ///
  495|       |    /// # Returns
  496|       |    ///
  497|       |    /// Returns one decoded codec value.
  498|       |    ///
  499|       |    /// # Errors
  500|       |    ///
  501|       |    /// Returns [`ErrorKind::Unsupported`] before invoking the codec or reading
  502|       |    /// input when reset or finish may emit values. Otherwise, returns I/O
  503|       |    /// errors from the wrapped input, `UnexpectedEof` when EOF occurs before a
  504|       |    /// complete value is available, `InvalidData` when the codec reports an
  505|       |    /// impossible incomplete state, or the error returned by `map_error` for
  506|       |    /// codec reset, invalid input, or codec finish failures.
  507|       |    ///
  508|       |    /// This operation is not transactional. A successful main decode consumes
  509|       |    /// its source units before codec finalization. If finalization fails, the
  510|       |    /// decoded value is not returned and the consumed input is not restored.
  511|       |    /// A later read must not be treated as a retry of that value. Truncated
  512|       |    /// input at EOF is consumed before reporting the error. Invalid-input
  513|       |    /// consumption follows the codec's validated consumption hint. Codec
  514|       |    /// state is not rolled back. The `Unsupported` preflight error above is
  515|       |    /// returned before invoking the codec or reading input.
  516|       |    ///
  517|       |    /// # Panics
  518|       |    ///
  519|       |    /// Panics when the codec reports more reset or finish values than its
  520|       |    /// declared bounds.
  521|     66|    pub fn read_decoded_with<C, M>(&mut self, codec: &mut C, map_error: M) -> Result<C::Value>
  522|     66|    where
  523|     66|        C: Codec<Unit = I::Item>,
  524|     66|        M: FnMut(C::DecodeError) -> Error,
  525|       |    {
  526|     66|        TranscodeFailure::ensure_no_decode_lifecycle_output::<C>()
  527|     66|            .map_err(|error| Error::new(ErrorKind::Unsupported, error))?;
                                           ^1         ^1                      ^1     ^1
  528|     65|        let progress = self.read_decoded_lifecycle_with_scratch_impl(codec, &mut [], &mut [], map_error)?;
                          ^23                                                                                         ^42
  529|     23|        let (value, reset_written, finish_written) = progress.into_parts();
  530|     23|        debug_assert_eq!(0, reset_written);
  531|     22|        debug_assert_eq!(0, finish_written);
  532|     22|        Ok(value)
  533|     65|    }
  534|       |
  535|       |    /// Decodes one complete codec lifecycle and preserves every output phase.
  536|       |    ///
  537|       |    /// The method allocates independent storage for values emitted by
  538|       |    /// `decode_reset` and `decode_finish`, so neither phase can overwrite the
  539|       |    /// other.
  540|       |    ///
  541|       |    /// # Type Parameters
  542|       |    ///
  543|       |    /// - `C`: Codec accepting the wrapped input's unit type.
  544|       |    /// - `M`: Mapper from codec decode errors to I/O errors.
  545|       |    ///
  546|       |    /// # Parameters
  547|       |    ///
  548|       |    /// - `codec`: Codec used to decode one complete lifecycle.
  549|       |    /// - `map_error`: Maps codec decode errors to I/O errors.
  550|       |    ///
  551|       |    /// # Returns
  552|       |    ///
  553|       |    /// Returns reset output, the main decoded value, and finish output.
  554|       |    ///
  555|       |    /// # Errors
  556|       |    ///
  557|       |    /// Returns I/O errors from the wrapped input, `UnexpectedEof` when EOF
  558|       |    /// occurs before a complete value is available, `InvalidData` when the
  559|       |    /// codec reports an impossible incomplete state, or the error returned
  560|       |    /// by `map_error` for codec reset, invalid input, or codec finish failures.
  561|       |    /// Returns `OutOfMemory` when lifecycle output storage cannot be reserved.
  562|       |    ///
  563|       |    /// This operation is not transactional. A successful main decode consumes
  564|       |    /// its source units before codec finalization. If finalization fails, the
  565|       |    /// decoded value is not returned and the consumed input is not restored.
  566|       |    /// A later read must not be treated as a retry of that value. Truncated
  567|       |    /// input at EOF is consumed before reporting the error. Invalid-input
  568|       |    /// consumption follows the codec's validated consumption hint. Codec
  569|       |    /// state is not rolled back.
  570|       |    ///
  571|       |    /// # Panics
  572|       |    ///
  573|       |    /// Panics when the codec reports more reset or finish values than its
  574|       |    /// declared bounds.
  575|     14|    pub fn read_decoded_lifecycle_with<C, M>(
  576|     14|        &mut self,
  577|     14|        codec: &mut C,
  578|     14|        map_error: M,
  579|     14|    ) -> Result<DecodeLifecycleOutput<C::Value>>
  580|     14|    where
  581|     14|        C: Codec<Unit = I::Item>,
  582|     14|        C::Value: Default,
  583|     14|        M: FnMut(C::DecodeError) -> Error,
  584|       |    {
  585|     14|        assert_unit_bounds::<C>();
  586|     14|        let mut reset_output = Vec::new();
  587|     14|        try_reserve_vec(&mut reset_output, C::MAX_DECODE_RESET_VALUES).map_err(allocation_error)?;
                                                                                                              ^1
  588|     13|        reset_output.resize_with(C::MAX_DECODE_RESET_VALUES, C::Value::default);
  589|     13|        let mut finish_output = Vec::new();
  590|     13|        try_reserve_vec(&mut finish_output, C::MAX_DECODE_FINISH_VALUES).map_err(allocation_error)?;
                                                                                                                ^1
  591|     12|        finish_output.resize_with(C::MAX_DECODE_FINISH_VALUES, C::Value::default);
  592|      9|        let progress =
  593|     12|            self.read_decoded_lifecycle_with_scratch_impl(codec, &mut reset_output, &mut finish_output, map_error)?;
                                                                                                                                ^3
  594|      9|        let (value, reset_written, finish_written) = progress.into_parts();
  595|      9|        reset_output.truncate(reset_written);
  596|      9|        finish_output.truncate(finish_written);
  597|      9|        Ok(DecodeLifecycleOutput::new(reset_output, value, finish_output))
  598|     14|    }
  599|       |
  600|       |    /// Decodes one complete codec lifecycle into separate caller storage.
  601|       |    ///
  602|       |    /// # Type Parameters
  603|       |    ///
  604|       |    /// - `C`: Codec accepting the wrapped input's unit type.
  605|       |    /// - `M`: Mapper from codec decode errors to I/O errors.
  606|       |    ///
  607|       |    /// # Parameters
  608|       |    ///
  609|       |    /// - `codec`: Codec used to decode one complete lifecycle.
  610|       |    /// - `reset_output`: Reusable storage for values emitted by
  611|       |    ///   [`Codec::decode_reset`].
  612|       |    /// - `finish_output`: Reusable storage for values emitted by
  613|       |    ///   [`Codec::decode_finish`].
  614|       |    /// - `map_error`: Maps codec decode errors to I/O errors.
  615|       |    ///
  616|       |    /// # Returns
  617|       |    ///
  618|       |    /// Returns the main decoded value and the initialized lengths of both
  619|       |    /// lifecycle output buffers.
  620|       |    ///
  621|       |    /// # Errors
  622|       |    ///
  623|       |    /// Returns [`ErrorKind::InvalidInput`] when either output buffer is shorter
  624|       |    /// than the corresponding codec bound. Returns I/O errors from the wrapped
  625|       |    /// input, `UnexpectedEof` when EOF occurs before a complete value is
  626|       |    /// available, `InvalidData` when the codec reports an impossible
  627|       |    /// incomplete state, or the error returned by `map_error` for codec reset,
  628|       |    /// invalid input, or codec finish failures.
  629|       |    ///
  630|       |    /// This operation is not transactional. A successful main decode consumes
  631|       |    /// its source units before codec finalization. If finalization fails, the
  632|       |    /// decoded value is not returned and the consumed input is not restored.
  633|       |    /// A later read must not be treated as a retry of that value. Truncated
  634|       |    /// input at EOF is consumed before reporting the error. Invalid-input
  635|       |    /// consumption follows the codec's validated consumption hint. Codec
  636|       |    /// state is not rolled back. A short scratch buffer is rejected before
  637|       |    /// invoking the codec or reading input. Reset and finish output buffers
  638|       |    /// may already have changed when an error is returned; the error carries
  639|       |    /// no valid output lengths, so their full contents must not be treated as
  640|       |    /// this operation's successful output. Recovery of codec state after an
  641|       |    /// error is codec-specific, and consumed input is never restored.
  642|       |    ///
  643|       |    /// # Panics
  644|       |    ///
  645|       |    /// Panics when the codec reports more reset or finish values than its
  646|       |    /// declared bounds.
  647|      6|    pub fn read_decoded_lifecycle_with_scratch<C, M>(
  648|      6|        &mut self,
  649|      6|        codec: &mut C,
  650|      6|        reset_output: &mut [C::Value],
  651|      6|        finish_output: &mut [C::Value],
  652|      6|        map_error: M,
  653|      6|    ) -> Result<DecodeLifecycleProgress<C::Value>>
  654|      6|    where
  655|      6|        C: Codec<Unit = I::Item>,
  656|      6|        M: FnMut(C::DecodeError) -> Error,
  657|       |    {
  658|      6|        self.read_decoded_lifecycle_with_scratch_impl(codec, reset_output, finish_output, map_error)
  659|      6|    }
  660|       |
  661|       |    /// Runs decoder reset into an indexed output range.
  662|       |    ///
  663|       |    /// This method does not read or consume buffered input units.
  664|       |    ///
  665|       |    /// # Type Parameters
  666|       |    ///
  667|       |    /// - `D`: Streaming decoder accepting the source unit type.
  668|       |    /// - `M`: Mapper from decoder errors to I/O errors.
  669|       |    /// - `Value`: Decoded destination value type.
  670|       |    ///
  671|       |    /// # Parameters
  672|       |    ///
  673|       |    /// - `decoder`: Decoder for the current logical stream.
  674|       |    /// - `map_error`: Mapper invoked for decoder errors.
  675|       |    /// - `output`: Destination value storage.
  676|       |    /// - `output_index`: Start of the writable range.
  677|       |    /// - `count`: Maximum number of destination values available.
  678|       |    ///
  679|       |    /// # Returns
  680|       |    ///
  681|       |    /// Returns the number of reset values initialized in the requested range.
  682|       |    ///
  683|       |    /// # Errors
  684|       |    ///
  685|       |    /// Returns invalid output ranges, insufficient declared output capacity,
  686|       |    /// capacity errors, or reset errors mapped by `map_error`.
  687|       |    ///
  688|       |    /// # Panics
  689|       |    ///
  690|       |    /// Panics when the decoder reports writing more values than its declared
  691|       |    /// reset bound.
  692|      3|    pub fn reset<D, M, Value>(
  693|      3|        &self,
  694|      3|        decoder: &mut D,
  695|      3|        map_error: &mut M,
  696|      3|        output: &mut [Value],
  697|      3|        output_index: usize,
  698|      3|        count: usize,
  699|      3|    ) -> Result<usize>
  700|      3|    where
  701|      3|        D: Transcoder<Input = I::Item, Output = Value>,
  702|      3|        M: FnMut(D::Error) -> Error,
  703|       |    {
  704|      3|        let required = decoder.max_reset_output_len().map_err(capacity_to_io_error)?;
                                                                                                 ^0
  705|      3|        let output_end = SliceRange::checked_range_end(
                          ^2
  706|      3|            output.len(),
  707|      3|            output_index,
  708|      3|            count,
  709|       |            "reset output range exceeds destination buffer",
  710|      1|        )?;
  711|      2|        if count < required {
  712|      1|            return Err(Error::new(
  713|      1|                ErrorKind::InvalidData,
  714|      1|                "insufficient output for decoder reset bound",
  715|      1|            ));
  716|      1|        }
  717|      1|        let output = &mut output[..output_end];
  718|      1|        let written = decoder.reset(output, output_index).map_err(&mut *map_error)?;
                                                                                                ^0
  719|      1|        assert!(written <= required, "reset wrote beyond its bound");
  720|      1|        Ok(written)
  721|      3|    }
  722|       |
  723|       |    /// Decodes values into an indexed output range using a streaming
  724|       |    /// [`Transcoder`].
  725|       |    ///
  726|       |    /// # Type Parameters
  727|       |    ///
  728|       |    /// - `D`: Streaming decoder accepting the source unit type.
  729|       |    /// - `M`: Mapper from decoder errors to I/O errors.
  730|       |    /// - `Value`: Decoded destination value type.
  731|       |    ///
  732|       |    /// # Parameters
  733|       |    ///
  734|       |    /// * `decoder` - Streaming decoder used for this operation.
  735|       |    /// * `map_error` - Function mapping transcode errors into I/O errors.
  736|       |    /// * `output` - Destination value storage.
  737|       |    /// * `output_index` - Start index inside `output`.
  738|       |    /// * `count` - Maximum number of values to write.
  739|       |    ///
  740|       |    /// # Returns
  741|       |    ///
  742|       |    /// The number of values written. When this method is driven through EOF
  743|       |    /// with output capacity remaining, it passes the retained tail—or an empty
  744|       |    /// input when no tail remains—to [`Transcoder::transcode_eof`] before
  745|       |    /// returning.
  746|       |    /// A short return, including zero, may mean the decoder requires a larger
  747|       |    /// contiguous output range. It does not prove conversion is complete. Use
  748|       |    /// [`Self::transcode_step`] and [`Self::transcode_eof_step`] when the stop
  749|       |    /// reason and required capacity must remain observable.
  750|       |    ///
  751|       |    /// # Errors
  752|       |    ///
  753|       |    /// Returns input errors, invalid output ranges, capacity errors from the
  754|       |    /// internal buffer, or transcode errors mapped by `map_error`.
  755|       |    /// This operation is not transactional. An error may be returned after
  756|       |    /// source units have been consumed and values have been written to
  757|       |    /// output. Progress is not rolled back, and the error does not report
  758|       |    /// the accumulated output length. Use `transcode_step` and commit each
  759|       |    /// successful progress result when previously decoded values must
  760|       |    /// remain observable after a later failure.
  761|       |    /// The method does not refill for later values once the requested output
  762|       |    /// range is full; the underlying input may still prefetch units beyond
  763|       |    /// that range during an earlier read.
  764|     27|    pub fn transcode<D, M, Value>(
  765|     27|        &mut self,
  766|     27|        decoder: &mut D,
  767|     27|        map_error: &mut M,
  768|     27|        output: &mut [Value],
  769|     27|        output_index: usize,
  770|     27|        count: usize,
  771|     27|    ) -> Result<usize>
  772|     27|    where
  773|     27|        D: Transcoder<Input = I::Item, Output = Value>,
  774|     27|        M: FnMut(D::Error) -> Error,
  775|       |    {
  776|     27|        let output_end = SliceRange::checked_range_end(
                          ^26
  777|     27|            output.len(),
  778|     27|            output_index,
  779|     27|            count,
  780|       |            "decoded output range exceeds destination buffer",
  781|      1|        )?;
  782|     26|        if count == 0 {
  783|      2|            return Ok(0);
  784|     24|        }
  785|     24|        let output = &mut output[..output_end];
  786|     24|        let mut written_total = 0;
  787|       |        loop {
  788|     28|            if self.unread_len() == 0 && !self.fill_more()? {
                                                        ^25  ^25        ^1
  789|      5|                let remaining_output = count - written_total;
  790|      5|                let eof_progress = self.transcode_eof_step(
  791|      5|                    decoder,
  792|      5|                    map_error,
  793|      5|                    output,
  794|      5|                    output_index + written_total,
  795|      5|                    remaining_output,
  796|      0|                )?;
  797|      5|                written_total += eof_progress.written();
  798|      5|                return Ok(written_total);
  799|     22|            }
  800|     22|            let units = self.unread();
  801|     22|            let available_input = units.len();
  802|     22|            let remaining_output = count - written_total;
  803|     22|            let progress = decoder
                              ^21
  804|     22|                .transcode(units, 0, output, output_index + written_total)
  805|     22|                .map_err(&mut *map_error)?;
                                                       ^1
  806|     21|            let progress = validate_decode_progress(
                              ^17
  807|     21|                progress,
  808|       |                0,
  809|     21|                available_input,
  810|     21|                output_index + written_total,
  811|     21|                remaining_output,
  812|      4|            )?;
  813|     17|            let consumed = progress.read();
  814|     17|            let written = progress.written();
  815|     17|            self.consume(consumed);
  816|     17|            written_total += written;
  817|     17|            if written_total == count {
  818|      7|                return Ok(written_total);
  819|     10|            }
  820|     10|            match progress.status() {
  821|      2|                TranscodeStatus::Complete => {}
  822|       |                TranscodeStatus::NeedOutput { .. } => {
  823|      0|                    return Ok(written_total);
  824|       |                }
  825|      8|                TranscodeStatus::NeedInput { required, .. } => {
  826|      8|                    if self.fill_until(required.get())? {
                                                                    ^1
  827|      2|                        continue;
  828|      5|                    }
  829|      5|                    let remaining_output = count - written_total;
  830|      5|                    let eof_progress = self.transcode_eof_step(
                                      ^1
  831|      5|                        decoder,
  832|      5|                        map_error,
  833|      5|                        output,
  834|      5|                        output_index + written_total,
  835|      5|                        remaining_output,
  836|      4|                    )?;
  837|      1|                    written_total += eof_progress.written();
  838|      1|                    return Ok(written_total);
  839|       |                }
  840|       |            }
  841|       |        }
  842|     27|    }
  843|       |
  844|       |    /// Finishes a streaming decoder into an indexed output range.
  845|       |    ///
  846|       |    /// # Type Parameters
  847|       |    ///
  848|       |    /// - `D`: Streaming decoder accepting the source unit type.
  849|       |    /// - `M`: Mapper from decoder errors to I/O errors.
  850|       |    /// - `Value`: Decoded destination value type.
  851|       |    ///
  852|       |    /// # Parameters
  853|       |    ///
  854|       |    /// * `decoder` - Streaming decoder whose final output is being collected.
  855|       |    /// * `map_error` - Function mapping transcode errors into I/O errors.
  856|       |    /// * `output` - Destination value storage.
  857|       |    /// * `output_index` - Start index inside `output`.
  858|       |    /// * `count` - Maximum number of finish values to write.
  859|       |    ///
  860|       |    /// # Returns
  861|       |    ///
  862|       |    /// The number of values written by the decoder finish operation.
  863|       |    ///
  864|       |    /// # Errors
  865|       |    ///
  866|       |    /// Returns invalid output ranges, capacity errors, or transcode
  867|       |    /// finalization errors mapped to I/O errors.
  868|       |    ///
  869|       |    /// # Panics
  870|       |    ///
  871|       |    /// Panics when the decoder reports writing more values than its
  872|       |    /// [`Transcoder::max_finish_output_len`] bound.
  873|     10|    pub fn finish<D, M, Value>(
  874|     10|        &self,
  875|     10|        decoder: &mut D,
  876|     10|        map_error: &mut M,
  877|     10|        output: &mut [Value],
  878|     10|        output_index: usize,
  879|     10|        count: usize,
  880|     10|    ) -> Result<usize>
  881|     10|    where
  882|     10|        D: Transcoder<Input = I::Item, Output = Value>,
  883|     10|        M: FnMut(D::Error) -> Error,
  884|       |    {
  885|     10|        let required = decoder.max_finish_output_len().map_err(capacity_to_io_error)?;
                          ^9                                                                      ^1
  886|       |        // Validate the caller-supplied count range first (InvalidInput).
  887|      9|        let output_end = SliceRange::checked_range_end(
                          ^8
  888|      9|            output.len(),
  889|      9|            output_index,
  890|      9|            count,
  891|       |            "finish output range exceeds destination buffer",
  892|      1|        )?;
  893|       |        // `count` is the caller's declared writable finish range.  The
  894|       |        // destination slice may be larger, but passing extra capacity to the
  895|       |        // decoder would allow it to write beyond the range the caller granted.
  896|      8|        if count < required {
  897|      2|            return Err(Error::new(
  898|      2|                ErrorKind::InvalidData,
  899|      2|                "insufficient output for decoder finish bound",
  900|      2|            ));
  901|      6|        }
  902|      6|        let output = &mut output[..output_end];
  903|      6|        let written = decoder.finish(output, output_index).map_err(&mut *map_error)?;
                          ^3                                                                     ^3
  904|      3|        assert!(written <= required, "finish wrote beyond its bound");
  905|      2|        Ok(written)
  906|      9|    }
  907|       |}
  908|       |
  909|       |impl<I> TranscodeDecodeInput<I>
  910|       |where
  911|       |    I: SeekableByteInput,
  912|       |{
  913|       |    /// Seeks the wrapped byte input and discards buffered bytes after success.
  914|       |    ///
  915|       |    /// # Parameters
  916|       |    ///
  917|       |    /// * `position` - Target seek position.
  918|       |    ///
  919|       |    /// # Returns
  920|       |    ///
  921|       |    /// The new stream position reported by the wrapped input.
  922|       |    ///
  923|       |    /// # Errors
  924|       |    ///
  925|       |    /// Returns seek errors from the wrapped input.
  926|       |    #[inline]
  927|      4|    pub fn seek(&mut self, position: SeekFrom) -> Result<u64> {
  928|      4|        self.input.seek_to(position)
  929|      4|    }
  930|       |}
  931|       |
  932|       |impl<I> TranscodeDecodeInput<I>
  933|       |where
  934|       |    I: Input,
  935|       |    I::Item: Copy + Default,
  936|       |{
  937|       |    /// Reads and decodes reset, body, and finish output using caller scratch
  938|       |    /// buffers.
  939|       |    ///
  940|       |    /// This internal path keeps lifecycle allocation and error mapping in one
  941|       |    /// place for the public convenience methods.
  942|       |    ///
  943|       |    /// # Type Parameters
  944|       |    ///
  945|       |    /// - `C`: Codec accepting the wrapped input's unit type.
  946|       |    /// - `M`: Mapper from codec decode errors to I/O errors.
  947|       |    ///
  948|       |    /// # Parameters
  949|       |    ///
  950|       |    /// - `codec`: Codec whose reset, decode, and finish phases are executed.
  951|       |    /// - `reset_output`: Separate storage for reset values.
  952|       |    /// - `finish_output`: Separate storage for finish values.
  953|       |    /// - `map_error`: Mapper for codec decode errors.
  954|       |    ///
  955|       |    /// # Returns
  956|       |    ///
  957|       |    /// Returns the decoded value and initialized lengths of both output
  958|       |    /// buffers.
  959|       |    ///
  960|       |    /// # Errors
  961|       |    ///
  962|       |    /// Returns InvalidInput for insufficient scratch capacity, input I/O
  963|       |    /// errors, UnexpectedEof for a truncated value, InvalidData for invalid
  964|       |    /// codec progress, or mapped codec reset, decode, and finish errors.
  965|       |    ///
  966|       |    /// # Panics
  967|       |    ///
  968|       |    /// Panics if reset or finish reports output beyond its declared bound.
  969|     83|    fn read_decoded_lifecycle_with_scratch_impl<C, M>(
  970|     83|        &mut self,
  971|     83|        codec: &mut C,
  972|     83|        reset_output: &mut [C::Value],
  973|     83|        finish_output: &mut [C::Value],
  974|     83|        mut map_error: M,
  975|     83|    ) -> Result<DecodeLifecycleProgress<C::Value>>
  976|     83|    where
  977|     83|        C: Codec<Unit = I::Item>,
  978|     83|        M: FnMut(C::DecodeError) -> Error,
  979|       |    {
  980|     83|        assert_unit_bounds::<C>();
  981|     83|        if reset_output.len() < C::MAX_DECODE_RESET_VALUES {
  982|      1|            return Err(Error::new(
  983|      1|                ErrorKind::InvalidInput,
  984|      1|                "decode reset output is shorter than the codec reset bound",
  985|      1|            ));
  986|     82|        }
  987|     82|        if finish_output.len() < C::MAX_DECODE_FINISH_VALUES {
  988|      1|            return Err(Error::new(
  989|      1|                ErrorKind::InvalidInput,
  990|      1|                "decode finish output is shorter than the codec finish bound",
  991|      1|            ));
  992|     81|        }
  993|     80|        let reset_written = unsafe {
  994|       |            // SAFETY: The reset output length check above reserves the codec's
  995|       |            // declared decode-reset output bound.
  996|     81|            codec.decode_reset(reset_output, 0)
  997|       |        }
  998|     81|        .map_err(&mut map_error)?;
                                              ^1
  999|     80|        assert!(
 1000|     80|            reset_written <= C::MAX_DECODE_RESET_VALUES,
 1001|       |            "Codec::decode_reset wrote beyond its reset bound",
 1002|       |        );
 1003|       |
 1004|     79|        let value = CodecDecodeDriver::new(&mut self.input).read_one(codec, &mut map_error)?;
                          ^37                                                                            ^42
 1005|       |
 1006|     34|        let finish_written = unsafe {
 1007|       |            // SAFETY: The finish output length check above reserves the
 1008|       |            // codec's declared decode-finish output bound.
 1009|     37|            codec.decode_finish(finish_output, 0)
 1010|       |        }
 1011|     37|        .map_err(&mut map_error)?;
                                              ^3
 1012|     34|        assert!(
 1013|     34|            finish_written <= C::MAX_DECODE_FINISH_VALUES,
 1014|       |            "Codec::decode_finish wrote beyond its finish bound",
 1015|       |        );
 1016|     32|        Ok(DecodeLifecycleProgress::new(value, reset_written, finish_written))
 1017|     80|    }
 1018|       |
 1019|       |    /// Refills the underlying buffer.
 1020|       |    ///
 1021|       |    /// # Returns
 1022|       |    ///
 1023|       |    /// Returns `true` if unread input is available after refill, or `false` at
 1024|       |    /// EOF.
 1025|       |    ///
 1026|       |    /// # Errors
 1027|       |    ///
 1028|       |    /// Propagates errors from the wrapped input while refilling.
 1029|       |    #[inline]
 1030|     27|    fn fill_more(&mut self) -> Result<bool> {
 1031|     27|        self.input.fill_more()
 1032|     27|    }
 1033|       |}
 1034|       |
 1035|       |impl<I> Read for TranscodeDecodeInput<I>
 1036|       |where
 1037|       |    I: ByteInput,
 1038|       |{
 1039|       |    /// Reads raw bytes through the internal buffer.
 1040|       |    ///
 1041|       |    /// # Parameters
 1042|       |    ///
 1043|       |    /// - `output`: Destination for raw buffered bytes.
 1044|       |    ///
 1045|       |    /// # Returns
 1046|       |    ///
 1047|       |    /// Returns the number of bytes read; zero indicates EOF or an empty
 1048|       |    /// destination.
 1049|       |    ///
 1050|       |    /// # Errors
 1051|       |    ///
 1052|       |    /// Propagates wrapped input or buffer errors.
 1053|       |    #[inline]
 1054|      5|    fn read(&mut self, output: &mut [u8]) -> Result<usize> {
 1055|       |        // SAFETY: The full output slice is a valid destination range.
 1056|      5|        unsafe { self.read_unchecked(output, 0, output.len()) }
 1057|      5|    }
 1058|       |}
 1059|       |
 1060|       |impl<I> Seek for TranscodeDecodeInput<I>
 1061|       |where
 1062|       |    I: SeekableByteInput,
 1063|       |{
 1064|       |    /// Seeks the wrapped byte input and discards buffered bytes after success.
 1065|       |    ///
 1066|       |    /// # Parameters
 1067|       |    ///
 1068|       |    /// - `position`: Target byte-stream position.
 1069|       |    ///
 1070|       |    /// # Returns
 1071|       |    ///
 1072|       |    /// Returns the new byte offset after clearing buffered data.
 1073|       |    ///
 1074|       |    /// # Errors
 1075|       |    ///
 1076|       |    /// Propagates wrapped seek errors, preserving the buffer when seeking
 1077|       |    /// fails.
 1078|       |    #[inline]
 1079|      4|    fn seek(&mut self, position: SeekFrom) -> Result<u64> {
 1080|      4|        self.seek(position)
 1081|      4|    }
 1082|       |}
 1083|       |
 1084|       |impl<I> Debug for TranscodeDecodeInput<I>
 1085|       |where
 1086|       |    I: Input,
 1087|       |    I::Item: Copy + Default,
 1088|       |    BufferedInput<I>: Debug,
 1089|       |{
 1090|       |    /// Formats this buffered decode input for debugging.
 1091|       |    ///
 1092|       |    /// # Parameters
 1093|       |    ///
 1094|       |    /// - `formatter`: Formatting sink receiving the struct and owned input.
 1095|       |    ///
 1096|       |    /// # Returns
 1097|       |    ///
 1098|       |    /// Returns success after writing the debug representation.
 1099|       |    ///
 1100|       |    /// # Errors
 1101|       |    ///
 1102|       |    /// Propagates a formatting error from the sink or wrapped input.
 1103|      2|    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
 1104|      2|        formatter
 1105|      2|            .debug_struct("TranscodeDecodeInput")
 1106|      2|            .field("input", &self.input)
 1107|      2|            .finish()
 1108|      2|    }
 1109|       |}
 1110|       |
 1111|       |/// Converts a capacity planning failure into an I/O error.
 1112|       |///
 1113|       |/// # Parameters
 1114|       |///
 1115|       |/// - `error`: Capacity failure preserved as the I/O error source.
 1116|       |///
 1117|       |/// # Returns
 1118|       |///
 1119|       |/// Returns an `InvalidData` I/O error owning the capacity failure.
 1120|      1|fn capacity_to_io_error(error: CapacityError) -> Error {
 1121|      1|    Error::new(ErrorKind::InvalidData, error)
 1122|      1|}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/io/transcode_encode_output.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Buffered output driver that encodes values into units.
    9|       |
   10|       |use core::fmt;
   11|       |use std::collections::TryReserveError;
   12|       |use std::io::Error;
   13|       |use std::io::ErrorKind;
   14|       |use std::io::Result;
   15|       |use std::io::Seek;
   16|       |use std::io::SeekFrom;
   17|       |use std::io::Write;
   18|       |use std::result::Result as CoreResult;
   19|       |
   20|       |use qubit_io::BufferedOutput;
   21|       |use qubit_io::BufferedOutputRecovery;
   22|       |use qubit_io::ByteOutput;
   23|       |use qubit_io::Output;
   24|       |use qubit_io::SeekableByteOutput;
   25|       |use qubit_utils::SliceRange;
   26|       |use qubit_utils::allocation_error;
   27|       |
   28|       |use super::internal::capacity_error_to_invalid_data;
   29|       |use super::internal::map_encode_value_result;
   30|       |use super::internal::validate_encode_progress;
   31|       |use crate::CapacityError;
   32|       |use crate::Codec;
   33|       |use crate::TranscodeEncodeErrorOf;
   34|       |use crate::TranscodeEncoder;
   35|       |use crate::TranscodeProgress;
   36|       |use crate::TranscodeStatus;
   37|       |use crate::Transcoder;
   38|       |use crate::value::codec_value_lifecycle::encode_complete_value_into_reserved;
   39|       |use crate::value::codec_value_lifecycle::max_complete_encode_units;
   40|       |
   41|       |/// Encodes an [`Output`] value stream into an [`Output`] unit stream.
   42|       |///
   43|       |/// This type owns only the unit-level [`qubit_io::BufferedOutput`]. Callers
   44|       |/// pass a [`crate::Codec`] and error mapper to each encode operation, which
   45|       |/// lets one buffered output drive different encoders without nesting buffers or
   46|       |/// storing codec-specific state in the buffer owner.
   47|       |///
   48|       |/// [`Self::flush`] only drains already buffered units. State-aware streaming
   49|       |/// encoders can use [`Self::reset`], [`Self::transcode`], and [`Self::finish`]
   50|       |/// explicitly.
   51|       |///
   52|       |/// # Type Parameters
   53|       |///
   54|       |/// * `O` - Wrapped unit output.
   55|       |///
   56|       |/// # Examples
   57|       |///
   58|       |/// ```
   59|       |/// use core::convert::Infallible;
   60|       |/// use core::num::NonZeroUsize;
   61|       |/// use std::io::Cursor;
   62|       |/// use qubit_codec::{Codec, CodecTranscodeEncoder, DecodeFailure, TranscodeEncodeOutput};
   63|       |/// use qubit_codec::{TranscodeEncoder, TranscodeStatus};
   64|       |///
   65|       |/// struct ByteCodec;
   66|       |/// impl Codec for ByteCodec {
   67|       |///     type Value = u8;
   68|       |///     type Unit = u8;
   69|       |///     type DecodeError = Infallible;
   70|       |///     type EncodeError = Infallible;
   71|       |///     const MIN_UNITS_PER_VALUE: usize = 1;
   72|       |///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
   73|       |///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
   74|       |///     unsafe fn decode(&mut self, input: &[u8], index: usize)
   75|       |///         -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
   76|       |///         Ok((input[index], NonZeroUsize::MIN))
   77|       |///     }
   78|       |///     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize)
   79|       |///         -> Result<usize, Infallible> {
   80|       |///         output[index] = *value;
   81|       |///         Ok(1)
   82|       |///     }
   83|       |/// }
   84|       |///
   85|       |/// let mut encoder = CodecTranscodeEncoder::new(ByteCodec);
   86|       |/// let mut output = TranscodeEncodeOutput::with_capacity(Cursor::new(Vec::new()), 8);
   87|       |/// let mut map_error = |_| std::io::Error::other("encode error");
   88|       |/// output.reset(&mut encoder, &mut map_error)?;
   89|       |/// let values = [1_u8, 2, 3];
   90|       |/// let mut offset = 0;
   91|       |/// while offset < values.len() {
   92|       |///     let progress = output.transcode_step(
   93|       |///         &mut encoder, &mut map_error, &values, offset, values.len() - offset,
   94|       |///     )?;
   95|       |///     offset += progress.read();
   96|       |///     if let TranscodeStatus::NeedOutput { required, .. } = progress.status() {
   97|       |///         output.ensure_spare_capacity(required.get())?;
   98|       |///     }
   99|       |/// }
  100|       |/// output.finish(&mut encoder, &mut map_error)?;
  101|       |/// assert_eq!(output.inner().get_ref(), &[1, 2, 3]);
  102|       |/// # Ok::<(), std::io::Error>(())
  103|       |/// ```
  104|       |///
  105|       |/// `finish` commits final units, drains the buffered output, and flushes the
  106|       |/// wrapped sink. Use `finish_to_buffer` followed by `flush` when those steps
  107|       |/// need to be controlled separately.
  108|       |pub struct TranscodeEncodeOutput<O>
  109|       |where
  110|       |    O: Output,
  111|       |    O::Item: Copy + Default,
  112|       |{
  113|       |    /// Owns the wrapped unit sink and pending encoded units awaiting delivery.
  114|       |    output: BufferedOutput<O>,
  115|       |}
  116|       |
  117|       |impl<O> TranscodeEncodeOutput<O>
  118|       |where
  119|       |    O: Output,
  120|       |    O::Item: Copy + Default,
  121|       |{
  122|       |    /// Creates an encoder output with the default unit buffer capacity.
  123|       |    ///
  124|       |    /// # Parameters
  125|       |    ///
  126|       |    /// * `inner` - Unit output written by this adapter.
  127|       |    ///
  128|       |    /// # Returns
  129|       |    ///
  130|       |    /// A new buffered encoder output.
  131|       |    #[must_use]
  132|      2|    pub fn new(inner: O) -> Self {
  133|      2|        Self {
  134|      2|            output: BufferedOutput::new(inner),
  135|      2|        }
  136|      2|    }
  137|       |
  138|       |    /// Creates an encoder output with a unit buffer of at least `capacity`.
  139|       |    ///
  140|       |    /// # Parameters
  141|       |    ///
  142|       |    /// * `inner` - Unit output written by this adapter.
  143|       |    /// * `capacity` - Requested internal unit buffer capacity.
  144|       |    ///
  145|       |    /// # Returns
  146|       |    ///
  147|       |    /// A new buffered encoder output.
  148|       |    #[must_use]
  149|     75|    pub fn with_capacity(inner: O, capacity: usize) -> Self {
  150|     75|        Self {
  151|     75|            output: BufferedOutput::with_capacity(inner, capacity),
  152|     75|        }
  153|     75|    }
  154|       |
  155|       |    /// Creates an encoder output with a unit buffer of at least `capacity`.
  156|       |    ///
  157|       |    /// # Parameters
  158|       |    ///
  159|       |    /// * `inner` - Unit output written by this adapter.
  160|       |    /// * `capacity` - Requested internal unit buffer capacity.
  161|       |    ///
  162|       |    /// # Errors
  163|       |    ///
  164|       |    /// Returns an allocation error when the requested buffer cannot be
  165|       |    /// allocated.
  166|       |    ///
  167|       |    /// # Returns
  168|       |    ///
  169|       |    /// A buffered adapter owning `inner` on success.
  170|      2|    pub fn try_with_capacity(inner: O, capacity: usize) -> CoreResult<Self, TryReserveError> {
  171|       |        Ok(Self {
  172|      2|            output: BufferedOutput::try_with_capacity(inner, capacity)?,
                                                                                    ^1
  173|       |        })
  174|      2|    }
  175|       |
  176|       |    /// Returns a shared reference to the wrapped unit output.
  177|       |    ///
  178|       |    /// Pending units remain in this adapter's internal buffer and are not
  179|       |    /// visible through the returned output until [`Self::flush`] succeeds.
  180|       |    ///
  181|       |    /// # Returns
  182|       |    ///
  183|       |    /// A shared reference to the wrapped unit output.
  184|       |    #[must_use]
  185|       |    #[inline]
  186|     51|    pub const fn inner(&self) -> &O {
  187|     51|        self.output.inner()
  188|     51|    }
  189|       |
  190|       |    /// Returns the available capacity of the spare output buffer.
  191|       |    ///
  192|       |    /// # Returns
  193|       |    ///
  194|       |    /// The number of output units that can still be appended without flushing.
  195|       |    #[must_use]
  196|       |    #[inline]
  197|      6|    pub fn spare_capacity(&self) -> usize {
  198|      6|        self.output.spare_capacity()
  199|      6|    }
  200|       |
  201|       |    /// Returns raw spare-buffer parts for the internal output buffer.
  202|       |    ///
  203|       |    /// # Returns
  204|       |    ///
  205|       |    /// The full backing storage, the spare start index, and the spare unit
  206|       |    /// count.
  207|       |    #[must_use]
  208|       |    #[inline]
  209|  1.00k|    pub fn spare_raw_parts_mut(&mut self) -> (&mut [O::Item], usize, usize) {
  210|  1.00k|        self.output.spare_raw_parts_mut()
  211|  1.00k|    }
  212|       |
  213|       |    /// Marks `count` units from [`Self::spare_raw_parts_mut`] as written.
  214|       |    ///
  215|       |    /// # Parameters
  216|       |    ///
  217|       |    /// * `count` - Initialized units to commit from the spare window.
  218|       |    ///
  219|       |    /// # Safety
  220|       |    ///
  221|       |    /// The caller must guarantee that `count <= Self::spare_capacity()` and
  222|       |    /// that the corresponding units in the returned spare slice have been
  223|       |    /// initialized.
  224|       |    #[inline]
  225|      6|    pub unsafe fn advance(&mut self, count: usize) {
  226|       |        // SAFETY: The caller guarantees `count` and initialization invariants.
  227|      6|        unsafe { self.output.advance(count) }
  228|      6|    }
  229|       |
  230|       |    /// Ensures that at least `count` spare units are available.
  231|       |    ///
  232|       |    /// # Parameters
  233|       |    ///
  234|       |    /// * `count` - Number of spare units required.
  235|       |    ///
  236|       |    /// # Errors
  237|       |    ///
  238|       |    /// Returns allocation errors mapped to [`ErrorKind::OutOfMemory`], or I/O
  239|       |    /// errors from the wrapped output while delivering pending units.
  240|  17.4k|    pub fn ensure_spare_capacity(&mut self, count: usize) -> Result<()> {
  241|  17.4k|        self.output.try_reserve_capacity(count).map_err(allocation_error)?;
                                                                                       ^4
  242|  17.4k|        self.output.ensure_spare_capacity(count)
  243|  17.4k|    }
  244|       |
  245|       |    /// Consumes this adapter without flushing the wrapped output.
  246|       |    ///
  247|       |    /// This method does not call [`Self::flush`] and performs no I/O. `Pending`
  248|       |    /// units were not reported as written and may be retried. `Indeterminate`
  249|       |    /// means an inner write panicked and may have produced an external prefix;
  250|       |    /// do not replay it automatically.
  251|       |    ///
  252|       |    /// # Returns
  253|       |    ///
  254|       |    /// The wrapped output and pending or indeterminate units.
  255|       |    #[must_use = "the returned output and recovery status must be handled"]
  256|       |    #[inline]
  257|      4|    pub fn into_parts(self) -> (O, BufferedOutputRecovery<O::Item>) {
  258|      4|        self.output.into_parts()
  259|      4|    }
  260|       |
  261|       |    /// Flushes buffered units without finishing any encoder stream.
  262|       |    ///
  263|       |    /// # Errors
  264|       |    ///
  265|       |    /// Returns errors from the wrapped output while flushing pending units.
  266|       |    #[inline]
  267|     29|    pub fn flush(&mut self) -> Result<()> {
  268|     29|        self.output.flush()
  269|     29|    }
  270|       |
  271|       |    /// Encodes one codec value into this buffered unit output.
  272|       |    ///
  273|       |    /// The method grows the persistent internal buffer when necessary, then
  274|       |    /// writes the complete encoded value into its spare window.
  275|       |    ///
  276|       |    /// # Type Parameters
  277|       |    ///
  278|       |    /// * `C` - Codec producing the wrapped output unit type.
  279|       |    /// * `M` - Mapper from codec errors into I/O errors.
  280|       |    ///
  281|       |    /// # Parameters
  282|       |    ///
  283|       |    /// * `codec` - Codec used for this single-value encode.
  284|       |    /// * `value` - Value to encode.
  285|       |    /// * `map_error` - Mapper for codec-domain encode errors.
  286|       |    ///
  287|       |    /// # Errors
  288|       |    ///
  289|       |    /// Returns I/O errors from the wrapped output, `InvalidInput` when the
  290|       |    /// codec output bound overflows or the value is outside the codec domain,
  291|       |    /// or the error returned by `map_error` for codec encode, reset, and finish
  292|       |    /// failures.
  293|       |    ///
  294|       |    /// # Panics
  295|       |    ///
  296|       |    /// Panics when the codec violates its declared reset, value-width, or
  297|       |    /// finish bounds, or when `encode` writes a length different from
  298|       |    /// `encode_len` in the same reset state.
  299|     18|    pub fn write_encoded_with<C, M>(&mut self, codec: &mut C, value: &C::Value, mut map_error: M) -> Result<()>
  300|     18|    where
  301|     18|        C: Codec<Unit = O::Item>,
  302|     18|        M: FnMut(C::EncodeError) -> Error,
  303|       |    {
  304|     18|        let max_units = map_encode_value_result(
                          ^17
  305|     18|            max_complete_encode_units::<C>().map_err(TranscodeEncodeErrorOf::<C>::from),
  306|     18|            &mut map_error,
  307|      1|        )?;
  308|     17|        self.ensure_spare_capacity(max_units)?;
                                                           ^2
  309|     15|        let (units, output_index, available) = self.output.spare_raw_parts_mut();
  310|     15|        debug_assert!(
  311|     15|            available >= max_units,
  312|       |            "reserved spare buffer is smaller than codec upper bound",
  313|       |        );
  314|     15|        let written = map_encode_value_result(
                          ^11
  315|     15|            encode_complete_value_into_reserved(codec, value, units, output_index, max_units),
  316|     15|            &mut map_error,
  317|      4|        )?;
  318|       |        // SAFETY: The spare buffer has the conservative complete lifecycle
  319|       |        // capacity, and the helper reports how many initialized units it
  320|       |        // actually wrote.
  321|     11|        unsafe {
  322|     11|            self.output.advance(written);
  323|     11|        }
  324|     11|        Ok(())
  325|     18|    }
  326|       |
  327|       |    /// Runs encoder reset and buffers any stream-prefix units.
  328|       |    ///
  329|       |    /// This method reserves enough persistent buffer capacity for all reset
  330|       |    /// output before calling `encoder`. Preparing that space may deliver
  331|       |    /// previously buffered units to the wrapped output and can return an I/O
  332|       |    /// error before `encoder.reset` runs. This delivery does not call the
  333|       |    /// wrapped output's `flush` method.
  334|       |    ///
  335|       |    /// # Type Parameters
  336|       |    ///
  337|       |    /// * `E` - Transcoder producing the wrapped output unit type.
  338|       |    /// * `M` - Mapper from transcoder errors into I/O errors.
  339|       |    /// * `Value` - Source value type accepted by the transcoder.
  340|       |    ///
  341|       |    /// # Parameters
  342|       |    ///
  343|       |    /// * `encoder` - Transcoder whose stream prefix is being collected.
  344|       |    /// * `map_error` - Mapper for reset failures.
  345|       |    ///
  346|       |    /// # Errors
  347|       |    ///
  348|       |    /// Returns capacity errors, allocation errors, or reset errors mapped by
  349|       |    /// `map_error`.
  350|       |    ///
  351|       |    /// # Panics
  352|       |    ///
  353|       |    /// Panics when `encoder` writes more than its declared reset bound.
  354|      4|    pub fn reset<E, M, Value>(&mut self, encoder: &mut E, map_error: &mut M) -> Result<()>
  355|      4|    where
  356|      4|        E: Transcoder<Input = Value, Output = O::Item>,
  357|      4|        M: FnMut(E::Error) -> Error,
  358|       |    {
  359|      4|        let required = encoder.max_reset_output_len().map_err(capacity_error_to_invalid_data)?;
                                                                                                           ^0
  360|      4|        self.ensure_spare_capacity(required)?;
                                                          ^1
  361|      3|        let (units, output_index, available) = self.output.spare_raw_parts_mut();
  362|      3|        debug_assert!(
  363|      3|            available >= required,
  364|       |            "insufficient reset capacity reserved in spare output buffer",
  365|       |        );
  366|      3|        let written = encoder.reset(units, output_index).map_err(&mut *map_error)?;
                                                                                               ^0
  367|      3|        assert!(written <= required, "reset wrote beyond its bound");
  368|       |        // SAFETY: The encoder reported initialized units within the spare
  369|       |        // range reserved above.
  370|      3|        unsafe {
  371|      3|            self.output.advance(written);
  372|      3|        }
  373|      3|        Ok(())
  374|      4|    }
  375|       |
  376|       |    /// Encodes at most one step of the selected input range.
  377|       |    ///
  378|       |    /// Unlike [`Self::transcode`], this method does not reserve output space
  379|       |    /// for the entire range and calls the encoder exactly once. The returned
  380|       |    /// progress is relative to this call's input and output windows. Advance
  381|       |    /// the input index by [`TranscodeProgress::read`]; if the status is
  382|       |    /// `NeedOutput`, provide the requested spare capacity before retrying.
  383|       |    /// Written units have already been committed to this adapter's pending
  384|       |    /// buffer and must not be advanced again. A zero count validates the range
  385|       |    /// and returns complete progress without delivering pending units or
  386|       |    /// calling the encoder.
  387|       |    ///
  388|       |    /// # Parameters
  389|       |    ///
  390|       |    /// * `encoder` - Encoder that processes one progress step.
  391|       |    /// * `map_error` - Maps encoder errors to I/O errors.
  392|       |    /// * `input` - Source values.
  393|       |    /// * `input_index` - Start index inside `input`.
  394|       |    /// * `count` - Number of values available from `input_index`.
  395|       |    ///
  396|       |    /// # Returns
  397|       |    ///
  398|       |    /// Progress for the single encoder call, including consumed input,
  399|       |    /// committed output, and whether more output capacity is needed.
  400|       |    ///
  401|       |    /// # Errors
  402|       |    ///
  403|       |    /// Returns invalid input ranges, capacity, encoder, or output errors. An
  404|       |    /// invalid encoder progress report is returned as `ErrorKind::InvalidData`.
  405|       |    /// An error does not provide general transactional rollback or complete
  406|       |    /// progress. The encoder may already have advanced, but units written by
  407|       |    /// its failing call are not committed to this adapter's pending output.
  408|       |    /// Do not replay the original input range without an encoder-specific
  409|       |    /// recovery protocol. Earlier pending units retain the wrapped output's
  410|       |    /// recovery contract; reset starts a new logical stream and does not undo
  411|       |    /// output already delivered.
  412|       |    ///
  413|       |    /// # Examples
  414|       |    ///
  415|       |    /// ```
  416|       |    /// use std::io::Cursor;
  417|       |    ///
  418|       |    /// # use core::convert::Infallible;
  419|       |    /// # use core::num::NonZeroUsize;
  420|       |    /// # use qubit_codec::Codec;
  421|       |    /// # use qubit_codec::CodecTranscodeEncoder;
  422|       |    /// # use qubit_codec::DecodeFailure;
  423|       |    /// use qubit_codec::TranscodeEncodeOutput;
  424|       |    /// use qubit_codec::TranscodeEncoder;
  425|       |    /// use qubit_codec::TranscodeStatus;
  426|       |    ///
  427|       |    /// # struct ByteCodec;
  428|       |    /// # impl Codec for ByteCodec {
  429|       |    /// #     type Value = u8;
  430|       |    /// #     type Unit = u8;
  431|       |    /// #     type DecodeError = Infallible;
  432|       |    /// #     type EncodeError = Infallible;
  433|       |    /// #     const MIN_UNITS_PER_VALUE: usize = 1;
  434|       |    /// #     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
  435|       |    /// #     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
  436|       |    /// #     unsafe fn decode(
  437|       |    /// #         &mut self,
  438|       |    /// #         input: &[u8],
  439|       |    /// #         index: usize,
  440|       |    /// #     ) -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
  441|       |    /// #         Ok((input[index], NonZeroUsize::MIN))
  442|       |    /// #     }
  443|       |    /// #     unsafe fn encode(
  444|       |    /// #         &mut self,
  445|       |    /// #         value: &u8,
  446|       |    /// #         output: &mut [u8],
  447|       |    /// #         index: usize,
  448|       |    /// #     ) -> Result<usize, Infallible> {
  449|       |    /// #         output[index] = *value;
  450|       |    /// #         Ok(1)
  451|       |    /// #     }
  452|       |    /// # }
  453|       |    /// fn encode<E>(encoder: &mut E, values: &[E::Input]) -> std::io::Result<Vec<u8>>
  454|       |    /// where
  455|       |    ///     E: TranscodeEncoder<Output = u8>,
  456|       |    /// {
  457|       |    ///     let mut output = TranscodeEncodeOutput::with_capacity(Cursor::new(Vec::new()), 8);
  458|       |    ///     let mut map_error = |_| std::io::Error::other("encode error");
  459|       |    ///     output.reset(encoder, &mut map_error)?;
  460|       |    ///     let mut offset = 0;
  461|       |    ///     while offset < values.len() {
  462|       |    ///         let progress = output.transcode_step(
  463|       |    ///             encoder,
  464|       |    ///             &mut map_error,
  465|       |    ///             values,
  466|       |    ///             offset,
  467|       |    ///             values.len() - offset,
  468|       |    ///         )?;
  469|       |    ///         offset += progress.read();
  470|       |    ///         if let TranscodeStatus::NeedOutput { required, .. } = progress.status() {
  471|       |    ///             output.ensure_spare_capacity(required.get())?;
  472|       |    ///         }
  473|       |    ///     }
  474|       |    ///     output.finish(encoder, &mut map_error)?;
  475|       |    ///     let (cursor, _) = output.into_parts();
  476|       |    ///     Ok(cursor.into_inner())
  477|       |    /// }
  478|       |    ///
  479|       |    /// let mut encoder = CodecTranscodeEncoder::new(ByteCodec);
  480|       |    /// assert_eq!(encode(&mut encoder, &[1, 2, 3]).unwrap(), vec![1, 2, 3]);
  481|       |    /// let mut empty_encoder = CodecTranscodeEncoder::new(ByteCodec);
  482|       |    /// assert_eq!(encode(&mut empty_encoder, &[]).unwrap(), Vec::<u8>::new());
  483|       |    /// ```
  484|  8.21k|    pub fn transcode_step<E, M, Value>(
  485|  8.21k|        &mut self,
  486|  8.21k|        encoder: &mut E,
  487|  8.21k|        map_error: &mut M,
  488|  8.21k|        input: &[Value],
  489|  8.21k|        input_index: usize,
  490|  8.21k|        count: usize,
  491|  8.21k|    ) -> Result<TranscodeProgress>
  492|  8.21k|    where
  493|  8.21k|        E: TranscodeEncoder<Input = Value, Output = O::Item>,
  494|  8.21k|        M: FnMut(E::Error) -> Error,
  495|       |    {
  496|  8.21k|        let input_end = SliceRange::checked_range_end(
                          ^8.21k
  497|  8.21k|            input.len(),
  498|  8.21k|            input_index,
  499|  8.21k|            count,
  500|       |            "encode input range exceeds source buffer",
  501|      1|        )?;
  502|  8.21k|        if count == 0 {
  503|      1|            return Ok(TranscodeProgress::complete(0, 0));
  504|  8.21k|        }
  505|  8.21k|        let required = encoder
  506|  8.21k|            .max_transcode_output_len(1)
  507|  8.21k|            .map_err(capacity_error_to_invalid_data)?
                                                                  ^0
  508|  8.21k|            .max(1);
  509|  8.21k|        self.ensure_spare_capacity(required)?;
                                                          ^1
  510|  8.21k|        let (units, output_index, available_output) = self.output.spare_raw_parts_mut();
  511|  8.21k|        let progress = encoder
                          ^8.21k
  512|  8.21k|            .transcode(&input[..input_end], input_index, units, output_index)
  513|  8.21k|            .map_err(&mut *map_error)?;
                                                   ^1
  514|  8.21k|        let progress = validate_encode_progress(progress, input_index, count, output_index, available_output)?;
                          ^8.21k                                                                                           ^2
  515|       |        // SAFETY: Validated progress fits the spare output window.
  516|  8.21k|        unsafe {
  517|  8.21k|            self.output.advance(progress.written());
  518|  8.21k|        }
  519|  8.21k|        Ok(progress)
  520|  8.21k|    }
  521|       |
  522|       |    /// Encodes values from an indexed input range using a streaming
  523|       |    /// [`Transcoder`].
  524|       |    ///
  525|       |    /// This streaming path reserves the encoder's conservative output bound
  526|       |    /// for the whole batch before calling it. Preparing that space may deliver
  527|       |    /// previously buffered units to the wrapped output and can return an I/O
  528|       |    /// error before consuming source values. Split large batches into smaller
  529|       |    /// calls when you need to limit the memory required for one operation.
  530|       |    ///
  531|       |    /// # Type Parameters
  532|       |    ///
  533|       |    /// * `E` - Encoder producing the wrapped output unit type.
  534|       |    /// * `M` - Mapper from encoder errors into I/O errors.
  535|       |    /// * `Value` - Source value type accepted by the encoder.
  536|       |    ///
  537|       |    /// # Parameters
  538|       |    ///
  539|       |    /// * `encoder` - Streaming encoder used for this operation.
  540|       |    /// * `map_error` - Function mapping transcode errors into I/O errors.
  541|       |    /// * `input` - Source values.
  542|       |    /// * `input_index` - Start index inside `input`.
  543|       |    /// * `count` - Maximum number of values to encode.
  544|       |    ///
  545|       |    /// # Returns
  546|       |    ///
  547|       |    /// The number of source values consumed.
  548|       |    ///
  549|       |    /// # Errors
  550|       |    ///
  551|       |    /// Returns invalid input ranges, capacity, transcode, or output errors. An
  552|       |    /// output-bound contract violation is reported as `ErrorKind::InvalidData`.
  553|       |    /// An error does not provide general transactional rollback or complete
  554|       |    /// progress. The encoder may already have advanced, while units written by
  555|       |    /// its failing call are not committed to this adapter's pending output.
  556|       |    /// Do not replay the original input range unless the concrete encoder
  557|       |    /// provides an explicit recovery protocol. Previously committed pending
  558|       |    /// output remains subject to the wrapped output's recovery contract, and
  559|       |    /// reset starts a new logical stream without undoing output already
  560|       |    /// delivered.
  561|       |    ///
  562|       |    /// # Examples
  563|       |    ///
  564|       |    /// A value that only implements [`Transcoder`] is not an encoder and cannot
  565|       |    /// be passed to this method.
  566|       |    ///
  567|       |    /// ```compile_fail
  568|       |    /// use std::io::Error;
  569|       |    /// use std::io::sink;
  570|       |    ///
  571|       |    /// use qubit_codec::CapacityError;
  572|       |    /// use qubit_codec::TranscodeEncodeError;
  573|       |    /// use qubit_codec::TranscodeEncodeOutput;
  574|       |    /// use qubit_codec::TranscodeProgress;
  575|       |    /// use qubit_codec::Transcoder;
  576|       |    ///
  577|       |    /// struct GenericTranscoder;
  578|       |    ///
  579|       |    /// impl Transcoder for GenericTranscoder {
  580|       |    ///     type Input = u8;
  581|       |    ///     type Output = u8;
  582|       |    ///     type Error = TranscodeEncodeError<(), u8>;
  583|       |    ///
  584|       |    ///     fn max_transcode_output_len(
  585|       |    ///         &self,
  586|       |    ///         input_len: usize,
  587|       |    ///     ) -> Result<usize, CapacityError> {
  588|       |    ///         Ok(input_len)
  589|       |    ///     }
  590|       |    ///
  591|       |    ///     fn reset(
  592|       |    ///         &mut self,
  593|       |    ///         _output: &mut [u8],
  594|       |    ///         _output_index: usize,
  595|       |    ///     ) -> Result<usize, Self::Error> {
  596|       |    ///         Ok(0)
  597|       |    ///     }
  598|       |    ///
  599|       |    ///     fn transcode(
  600|       |    ///         &mut self,
  601|       |    ///         input: &[u8],
  602|       |    ///         input_index: usize,
  603|       |    ///         _output: &mut [u8],
  604|       |    ///         _output_index: usize,
  605|       |    ///     ) -> Result<TranscodeProgress, Self::Error> {
  606|       |    ///         Ok(TranscodeProgress::complete(input.len() - input_index, 0))
  607|       |    ///     }
  608|       |    ///
  609|       |    ///     fn finish(
  610|       |    ///         &mut self,
  611|       |    ///         _output: &mut [u8],
  612|       |    ///         _output_index: usize,
  613|       |    ///     ) -> Result<usize, Self::Error> {
  614|       |    ///         Ok(0)
  615|       |    ///     }
  616|       |    /// }
  617|       |    ///
  618|       |    /// let mut output = TranscodeEncodeOutput::with_capacity(sink(), 1);
  619|       |    /// let mut transcoder = GenericTranscoder;
  620|       |    /// let mut map_error = |_| Error::other("transcode error");
  621|       |    /// let _ = output.transcode(
  622|       |    ///     &mut transcoder,
  623|       |    ///     &mut map_error,
  624|       |    ///     &[0_u8],
  625|       |    ///     0,
  626|       |    ///     1,
  627|       |    /// );
  628|       |    /// ```
  629|  1.02k|    pub fn transcode<E, M, Value>(
  630|  1.02k|        &mut self,
  631|  1.02k|        encoder: &mut E,
  632|  1.02k|        map_error: &mut M,
  633|  1.02k|        input: &[Value],
  634|  1.02k|        input_index: usize,
  635|  1.02k|        count: usize,
  636|  1.02k|    ) -> Result<usize>
  637|  1.02k|    where
  638|  1.02k|        E: TranscodeEncoder<Input = Value, Output = O::Item>,
  639|  1.02k|        M: FnMut(E::Error) -> Error,
  640|       |    {
  641|  1.02k|        let input_end = SliceRange::checked_range_end(
                          ^1.02k
  642|  1.02k|            input.len(),
  643|  1.02k|            input_index,
  644|  1.02k|            count,
  645|       |            "encode input range exceeds source buffer",
  646|      1|        )?;
  647|  1.02k|        if count == 0 {
  648|      1|            return Ok(0);
  649|  1.02k|        }
  650|  1.02k|        let required = encoder
  651|  1.02k|            .max_transcode_output_len(count)
  652|  1.02k|            .map_err(capacity_error_to_invalid_data)?
                                                                  ^0
  653|  1.02k|            .max(1);
  654|  1.02k|        self.ensure_spare_capacity(required)?;
                                                          ^3
  655|  1.02k|        let input = &input[..input_end];
  656|  1.02k|        let mut read_total = 0;
  657|  1.02k|        while read_total < count {
  658|  1.02k|            let (units, output_index, available_output) = self.output.spare_raw_parts_mut();
  659|  1.02k|            let remaining_input = count - read_total;
  660|  1.02k|            let progress = encoder
                              ^1.02k
  661|  1.02k|                .transcode(input, input_index + read_total, units, output_index)
  662|  1.02k|                .map_err(&mut *map_error)?;
                                                       ^2
  663|  1.02k|            let progress = validate_encode_progress(
                              ^1.01k
  664|  1.02k|                progress,
  665|  1.02k|                input_index + read_total,
  666|  1.02k|                remaining_input,
  667|  1.02k|                output_index,
  668|  1.02k|                available_output,
  669|      6|            )?;
  670|  1.01k|            let read = progress.read();
  671|  1.01k|            let written = progress.written();
  672|       |            // SAFETY: The progress bounds check above proved that the encoder
  673|       |            // initialized no more than the available spare output window.
  674|  1.01k|            unsafe {
  675|  1.01k|                self.output.advance(written);
  676|  1.01k|            }
  677|  1.01k|            read_total += read;
  678|  1.01k|            if progress.is_complete() {
  679|  1.01k|                return Ok(read_total);
  680|      2|            }
  681|      2|            if let TranscodeStatus::NeedOutput { required, .. } = progress.status()
  682|      2|                && (read_total == count || available_output - written < required.get() || (read == 0 && written == 0))
                                                         ^0                                              ^0           ^0
  683|       |            {
  684|      2|                return Err(Error::new(
  685|      2|                    ErrorKind::InvalidData,
  686|      2|                    "encoder output bound was insufficient",
  687|      2|                ));
  688|      0|            }
  689|       |        }
  690|      0|        Ok(read_total)
  691|  1.02k|    }
  692|       |
  693|       |    /// Finishes the encoder and flushes the wrapped unit output.
  694|       |    ///
  695|       |    /// This method writes final units directly into the internal spare buffer,
  696|       |    /// growing it when [`Transcoder::max_finish_output_len`] exceeds the
  697|       |    /// current capacity.
  698|       |    ///
  699|       |    /// # Type Parameters
  700|       |    ///
  701|       |    /// * `E` - Transcoder producing the wrapped output unit type.
  702|       |    /// * `M` - Mapper from transcoder errors into I/O errors.
  703|       |    /// * `Value` - Source value type accepted by the transcoder.
  704|       |    ///
  705|       |    /// # Parameters
  706|       |    ///
  707|       |    /// * `encoder` - Encoder whose final units are being collected.
  708|       |    /// * `map_error` - Function mapping transcode errors into I/O errors.
  709|       |    ///
  710|       |    /// # Errors
  711|       |    ///
  712|       |    /// Returns capacity, transcode finalization, or wrapped output flush
  713|       |    /// errors.
  714|       |    ///
  715|       |    /// # Panics
  716|       |    ///
  717|       |    /// Panics when the encoder exceeds its declared finish-output bound.
  718|     12|    pub fn finish<E, M, Value>(&mut self, encoder: &mut E, map_error: &mut M) -> Result<()>
  719|     12|    where
  720|     12|        E: Transcoder<Input = Value, Output = O::Item>,
  721|     12|        M: FnMut(E::Error) -> Error,
  722|       |    {
  723|     12|        self.finish_to_buffer(encoder, map_error)?;
                                                               ^6
  724|      6|        self.output.flush()
  725|     12|    }
  726|       |
  727|       |    /// Finishes the encoder while retaining final units in the output buffer.
  728|       |    ///
  729|       |    /// This method separates encoder finalization from output delivery. It is
  730|       |    /// useful when a caller must record successful finalization before a later
  731|       |    /// flush can fail. Call [`Self::flush`] to deliver the retained units.
  732|       |    ///
  733|       |    /// # Type Parameters
  734|       |    ///
  735|       |    /// * `E` - Transcoder producing the wrapped output unit type.
  736|       |    /// * `M` - Mapper from transcoder errors into I/O errors.
  737|       |    /// * `Value` - Source value type accepted by the transcoder.
  738|       |    ///
  739|       |    /// # Parameters
  740|       |    ///
  741|       |    /// * `encoder` - Encoder whose final units are being collected.
  742|       |    /// * `map_error` - Function mapping transcode errors into I/O errors.
  743|       |    ///
  744|       |    /// # Errors
  745|       |    ///
  746|       |    /// Returns capacity planning, allocation, or transcoder finalization
  747|       |    /// errors. It does not perform output I/O; use [`Self::finish`] to deliver
  748|       |    /// the buffered final units and flush the wrapped output.
  749|       |    ///
  750|       |    /// # Panics
  751|       |    ///
  752|       |    /// Panics when `encoder` writes more units than its declared finish bound.
  753|     13|    pub fn finish_to_buffer<E, M, Value>(&mut self, encoder: &mut E, map_error: &mut M) -> Result<()>
  754|     13|    where
  755|     13|        E: Transcoder<Input = Value, Output = O::Item>,
  756|     13|        M: FnMut(E::Error) -> Error,
  757|       |    {
  758|     13|        let required = match encoder.max_finish_output_len() {
                          ^12
  759|     12|            Ok(required) => required,
  760|      1|            Err(error) => return Err(capacity_error_to_invalid_data(error)),
  761|       |        };
  762|     12|        let pending = self.output.capacity() - self.output.spare_capacity();
  763|     12|        let capacity = pending
  764|     12|            .checked_add(required)
  765|     12|            .ok_or_else(|| capacity_error_to_invalid_data(CapacityError::OutputLengthOverflow))?;
                                         ^0                             ^0                                   ^0
  766|     12|        self.output.try_reserve_capacity(capacity).map_err(allocation_error)?;
                                                                                          ^1
  767|     11|        let (units, output_index, available) = self.output.spare_raw_parts_mut();
  768|     11|        debug_assert!(
  769|     11|            available >= required,
  770|       |            "insufficient finish capacity reserved in spare output buffer",
  771|       |        );
  772|     11|        let written = encoder.finish(units, output_index).map_err(&mut *map_error)?;
                          ^7                                                                    ^4
  773|      7|        assert!(written <= required, "finish wrote beyond its bound");
  774|       |        // SAFETY: The encoder reported initialized units within the spare
  775|       |        // range that was reserved above.
  776|      6|        unsafe {
  777|      6|            self.output.advance(written);
  778|      6|        }
  779|      6|        Ok(())
  780|     12|    }
  781|       |}
  782|       |
  783|       |impl<O> TranscodeEncodeOutput<O>
  784|       |where
  785|       |    O: SeekableByteOutput,
  786|       |{
  787|       |    /// Flushes pending bytes, then seeks the wrapped byte output.
  788|       |    ///
  789|       |    /// # Parameters
  790|       |    ///
  791|       |    /// * `position` - Target seek position.
  792|       |    ///
  793|       |    /// # Returns
  794|       |    ///
  795|       |    /// The new stream position reported by the wrapped output.
  796|       |    ///
  797|       |    /// # Errors
  798|       |    ///
  799|       |    /// Returns flush or seek errors from the wrapped output.
  800|       |    #[inline]
  801|      1|    pub fn seek(&mut self, position: SeekFrom) -> Result<u64> {
  802|      1|        self.output.seek_to(position)
  803|      1|    }
  804|       |}
  805|       |
  806|       |impl<O> Write for TranscodeEncodeOutput<O>
  807|       |where
  808|       |    O: ByteOutput,
  809|       |{
  810|       |    /// Writes raw bytes through the internal buffer without encoding them.
  811|       |    ///
  812|       |    /// # Parameters
  813|       |    ///
  814|       |    /// * `input` - Bytes to append to the buffered unit output.
  815|       |    ///
  816|       |    /// # Returns
  817|       |    ///
  818|       |    /// The number of bytes accepted, which may be shorter than `input`.
  819|       |    ///
  820|       |    /// # Errors
  821|       |    ///
  822|       |    /// Propagates wrapped-output errors when buffer delivery is needed.
  823|       |    #[inline]
  824|      3|    fn write(&mut self, input: &[u8]) -> Result<usize> {
  825|      3|        Output::write(&mut self.output, input)
  826|      3|    }
  827|       |
  828|       |    /// Writes all raw bytes through the internal buffer without encoding them.
  829|       |    ///
  830|       |    /// # Parameters
  831|       |    ///
  832|       |    /// * `input` - Bytes that must all be accepted before success.
  833|       |    ///
  834|       |    /// # Errors
  835|       |    ///
  836|       |    /// Propagates output errors and returns `WriteZero` if output stops
  837|       |    /// progressing.
  838|       |    ///
  839|       |    /// # Panics
  840|       |    ///
  841|       |    /// Panics when the wrapped output reports more bytes than supplied.
  842|      9|    fn write_all(&mut self, input: &[u8]) -> Result<()> {
  843|      9|        let mut written = 0;
  844|     18|        while written < input.len() {
  845|     12|            let count = match Output::write(&mut self.output, &input[written..]) {
                              ^8
  846|      8|                Ok(count) => count,
  847|      4|                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                                  ^2                                             ^2   ^2
  848|      2|                Err(error) => return Err(error),
  849|       |            };
  850|      8|            if count == 0 {
  851|      1|                return Err(Error::from(ErrorKind::WriteZero));
  852|      7|            }
  853|      7|            assert!(
  854|      7|                count <= input.len() - written,
  855|       |                "Output::write returned a count beyond the input length",
  856|       |            );
  857|      7|            written += count;
  858|       |        }
  859|      6|        Ok(())
  860|      9|    }
  861|       |
  862|       |    /// Flushes buffered bytes to the wrapped output without finishing an
  863|       |    /// encoder.
  864|       |    ///
  865|       |    /// # Errors
  866|       |    ///
  867|       |    /// Propagates wrapped-output flush errors.
  868|       |    #[inline]
  869|      2|    fn flush(&mut self) -> Result<()> {
  870|      2|        TranscodeEncodeOutput::flush(self)
  871|      2|    }
  872|       |}
  873|       |
  874|       |impl<O> Seek for TranscodeEncodeOutput<O>
  875|       |where
  876|       |    O: SeekableByteOutput,
  877|       |{
  878|       |    /// Flushes pending bytes, then seeks the wrapped byte output.
  879|       |    ///
  880|       |    /// # Parameters
  881|       |    ///
  882|       |    /// * `position` - Target position relative to the selected seek origin.
  883|       |    ///
  884|       |    /// # Returns
  885|       |    ///
  886|       |    /// The new absolute byte position.
  887|       |    ///
  888|       |    /// # Errors
  889|       |    ///
  890|       |    /// Propagates flush or seek errors from the wrapped output.
  891|       |    #[inline]
  892|      1|    fn seek(&mut self, position: SeekFrom) -> Result<u64> {
  893|      1|        self.seek(position)
  894|      1|    }
  895|       |}
  896|       |
  897|       |impl<O> fmt::Debug for TranscodeEncodeOutput<O>
  898|       |where
  899|       |    O: Output,
  900|       |    O::Item: Copy + Default,
  901|       |    BufferedOutput<O>: fmt::Debug,
  902|       |{
  903|       |    /// Formats the owned buffer and underlying output for diagnostics.
  904|       |    ///
  905|       |    /// # Parameters
  906|       |    ///
  907|       |    /// * `formatter` - Destination and formatting options.
  908|       |    ///
  909|       |    /// # Errors
  910|       |    ///
  911|       |    /// Returns a formatting error if the destination rejects output.
  912|      2|    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
  913|      2|        formatter
  914|      2|            .debug_struct("TranscodeEncodeOutput")
  915|      2|            .field("output", &self.output)
  916|      2|            .finish()
  917|      2|    }
  918|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/transcode_convert_error.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Errors reported by transcode converters.
    9|       |
   10|       |use thiserror::Error;
   11|       |
   12|       |use super::capacity_error::CapacityError;
   13|       |use super::transcode_decode_errors::TranscodeDecodeError;
   14|       |use super::transcode_domain_error::TranscodeDomainError;
   15|       |use super::transcode_encode_error::TranscodeEncodeError;
   16|       |use super::transcode_failure::TranscodeFailure;
   17|       |use crate::Codec;
   18|       |
   19|       |/// Convert transcode error for a codec-backed converter.
   20|       |///
   21|       |/// # Type Parameters
   22|       |///
   23|       |/// - `D`: Source codec providing decode errors and the intermediate value type.
   24|       |/// - `E`: Target codec providing encode errors for the same value type.
   25|       |pub type TranscodeConvertErrorOf<D, E> =
   26|       |    TranscodeConvertError<<D as Codec>::DecodeError, <E as Codec>::EncodeError, <D as Codec>::Value>;
   27|       |
   28|       |/// Error reported by a unit-to-unit transcode conversion.
   29|       |///
   30|       |/// # Type Parameters
   31|       |///
   32|       |/// - `DE`: Source decoder domain error.
   33|       |/// - `EE`: Target encoder domain error.
   34|       |/// - `V`: Owned intermediate value retained when encoding is rejected.
   35|       |///
   36|       |/// # Examples
   37|       |///
   38|       |/// ```
   39|       |/// use qubit_codec::TranscodeConvertError;
   40|       |///
   41|       |/// let error = TranscodeConvertError::<&str, &str, u8>::unencodable(3, 42);
   42|       |/// assert_eq!(error.unencodable_ref(), Some((3, Some(&42))));
   43|       |/// assert_eq!(error.failure_ref(), None);
   44|       |/// ```
   45|       |#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
   46|       |#[must_use]
   47|       |pub enum TranscodeConvertError<DE, EE, V> {
   48|       |    /// Framework-level transcode failure.
   49|       |    #[error(transparent)]
   50|       |    Failure(
   51|       |        /// Framework contract, capacity, or lifecycle failure.
   52|       |        #[from]
   53|       |        TranscodeFailure,
   54|       |    ),
   55|       |
   56|       |    /// Source-side domain error.
   57|       |    #[error("decode side failed: {0}")]
   58|       |    DecodeDomain(
   59|       |        /// Owned source error with its lifecycle phase and input context.
   60|       |        #[source]
   61|       |        TranscodeDomainError<DE>,
   62|       |    ),
   63|       |
   64|       |    /// Target-side domain error.
   65|       |    #[error("encode side failed: {0}")]
   66|       |    EncodeDomain(
   67|       |        /// Owned target error with its lifecycle phase and input context.
   68|       |        #[source]
   69|       |        TranscodeDomainError<EE>,
   70|       |    ),
   71|       |
   72|       |    /// The decoded intermediate value cannot be encoded by the target codec and
   73|       |    /// policy.
   74|       |    #[error("unencodable value at input index {input_index}")]
   75|       |    Unencodable {
   76|       |        /// Absolute source input index of the value being encoded.
   77|       |        input_index: usize,
   78|       |        /// `Some` retains the decoded intermediate value; `None` means value
   79|       |        /// context was unavailable or intentionally omitted.
   80|       |        value: Option<V>,
   81|       |    },
   82|       |}
   83|       |
   84|       |impl<DE, EE, V> TranscodeConvertError<DE, EE, V> {
   85|       |    /// Creates a source reset-phase domain-specific converter error.
   86|       |    ///
   87|       |    /// # Parameters
   88|       |    ///
   89|       |    /// - `source`: Owned source error raised during reset.
   90|       |    ///
   91|       |    /// # Returns
   92|       |    ///
   93|       |    /// Returns a source-domain reset error.
   94|       |    #[inline]
   95|      2|    pub const fn decode_domain_reset(source: DE) -> Self {
   96|      2|        Self::DecodeDomain(TranscodeDomainError::reset(source))
   97|      2|    }
   98|       |
   99|       |    /// Creates a source main-phase domain-specific converter error.
  100|       |    ///
  101|       |    /// # Parameters
  102|       |    ///
  103|       |    /// - `source`: Owned source error raised during conversion.
  104|       |    /// - `input_index`: Absolute source position of the failed operation.
  105|       |    ///
  106|       |    /// # Returns
  107|       |    ///
  108|       |    /// Returns a source-domain main-phase error at the supplied input index.
  109|       |    #[inline]
  110|      7|    pub const fn decode_domain_main(source: DE, input_index: usize) -> Self {
  111|      7|        Self::DecodeDomain(TranscodeDomainError::main(source, input_index))
  112|      7|    }
  113|       |
  114|       |    /// Creates a source main-phase converter error with decode consumption
  115|       |    /// context.
  116|       |    ///
  117|       |    /// # Parameters
  118|       |    ///
  119|       |    /// - `source`: Owned source error raised during conversion.
  120|       |    /// - `input_index`: Absolute source position of the failed operation.
  121|       |    /// - `input_consumed`: `Some` records invalid units consumed; `None` means
  122|       |    ///   unknown.
  123|       |    ///
  124|       |    /// # Returns
  125|       |    ///
  126|       |    /// Returns a source-domain main-phase error retaining consumption context.
  127|       |    #[inline]
  128|      3|    pub const fn decode_domain_main_with_consumed(
  129|      3|        source: DE,
  130|      3|        input_index: usize,
  131|      3|        input_consumed: Option<core::num::NonZeroUsize>,
  132|      3|    ) -> Self {
  133|      3|        Self::DecodeDomain(TranscodeDomainError::main_with_consumed(
  134|      3|            source,
  135|      3|            input_index,
  136|      3|            input_consumed,
  137|      3|        ))
  138|      3|    }
  139|       |
  140|       |    /// Creates a source finish-phase domain-specific converter error.
  141|       |    ///
  142|       |    /// # Parameters
  143|       |    ///
  144|       |    /// - `source`: Owned source error raised during finalization.
  145|       |    ///
  146|       |    /// # Returns
  147|       |    ///
  148|       |    /// Returns a source-domain finish error.
  149|       |    #[inline]
  150|      3|    pub const fn decode_domain_finish(source: DE) -> Self {
  151|      3|        Self::DecodeDomain(TranscodeDomainError::finish(source))
  152|      3|    }
  153|       |
  154|       |    /// Creates a target reset-phase domain-specific converter error.
  155|       |    ///
  156|       |    /// # Parameters
  157|       |    ///
  158|       |    /// - `source`: Owned target error raised during reset.
  159|       |    ///
  160|       |    /// # Returns
  161|       |    ///
  162|       |    /// Returns a target-domain reset error.
  163|       |    #[inline]
  164|      4|    pub const fn encode_domain_reset(source: EE) -> Self {
  165|      4|        Self::EncodeDomain(TranscodeDomainError::reset(source))
  166|      4|    }
  167|       |
  168|       |    /// Creates a target main-phase domain-specific converter error.
  169|       |    ///
  170|       |    /// # Parameters
  171|       |    ///
  172|       |    /// - `source`: Owned target error raised during conversion.
  173|       |    /// - `input_index`: Absolute source position of the failed operation.
  174|       |    ///
  175|       |    /// # Returns
  176|       |    ///
  177|       |    /// Returns a target-domain main-phase error at the supplied input index.
  178|       |    #[inline]
  179|      4|    pub const fn encode_domain_main(source: EE, input_index: usize) -> Self {
  180|      4|        Self::EncodeDomain(TranscodeDomainError::main(source, input_index))
  181|      4|    }
  182|       |
  183|       |    /// Creates a target finish-phase domain-specific converter error.
  184|       |    ///
  185|       |    /// # Parameters
  186|       |    ///
  187|       |    /// - `source`: Owned target error raised during finalization.
  188|       |    ///
  189|       |    /// # Returns
  190|       |    ///
  191|       |    /// Returns a target-domain finish error.
  192|       |    #[inline]
  193|      3|    pub const fn encode_domain_finish(source: EE) -> Self {
  194|      3|        Self::EncodeDomain(TranscodeDomainError::finish(source))
  195|      3|    }
  196|       |
  197|       |    /// Creates an unencodable intermediate-value error with value context.
  198|       |    ///
  199|       |    /// # Parameters
  200|       |    ///
  201|       |    /// - `input_index`: Absolute source index of the rejected value.
  202|       |    /// - `value`: Intermediate value moved into the error for recovery.
  203|       |    ///
  204|       |    /// # Returns
  205|       |    ///
  206|       |    /// Returns an unencodable error owning `Some(value)`.
  207|       |    #[inline]
  208|     10|    pub fn unencodable(input_index: usize, value: V) -> Self {
  209|     10|        Self::Unencodable {
  210|     10|            input_index,
  211|     10|            value: Some(value),
  212|     10|        }
  213|     10|    }
  214|       |
  215|       |    /// Creates an unencodable intermediate-value error without value context.
  216|       |    ///
  217|       |    /// # Parameters
  218|       |    ///
  219|       |    /// - `input_index`: Absolute source index of the rejected value.
  220|       |    ///
  221|       |    /// # Returns
  222|       |    ///
  223|       |    /// Returns an unencodable error with absent (`None`) value context.
  224|       |    #[inline]
  225|      3|    pub const fn unencodable_without_context(input_index: usize) -> Self {
  226|      3|        Self::Unencodable {
  227|      3|            input_index,
  228|      3|            value: None,
  229|      3|        }
  230|      3|    }
  231|       |
  232|       |    /// Converts an encode error into a converter error while adding fallback
  233|       |    /// value context to unencodable errors that lack it.
  234|       |    ///
  235|       |    /// # Parameters
  236|       |    ///
  237|       |    /// - `error`: Owned target-side error to convert.
  238|       |    /// - `fallback_value`: Value moved into absent context, or dropped if not
  239|       |    ///   needed.
  240|       |    ///
  241|       |    /// # Returns
  242|       |    ///
  243|       |    /// Returns the converted error, adding the fallback only when unencodable
  244|       |    /// value context is absent.
  245|       |    #[inline]
  246|     11|    pub fn from_encode_error_with_value(error: TranscodeEncodeError<EE, V>, fallback_value: V) -> Self {
  247|      4|        match error {
  248|       |            TranscodeEncodeError::Unencodable {
  249|      3|                input_index,
  250|       |                value: None,
  251|      3|            } => Self::Unencodable {
  252|      3|                input_index,
  253|      3|                value: Some(fallback_value),
  254|      3|            },
  255|      8|            other => Self::from(other),
  256|       |        }
  257|     11|    }
  258|       |
  259|       |    /// Returns the framework failure carried by this error.
  260|       |    ///
  261|       |    /// # Returns
  262|       |    ///
  263|       |    /// Returns `Some` for a framework failure, or `None` for domain and
  264|       |    /// unencodable errors.
  265|       |    #[inline]
  266|      5|    pub const fn failure_ref(&self) -> Option<&TranscodeFailure> {
  267|      5|        match self {
  268|      2|            Self::Failure(failure) => Some(failure),
  269|      3|            Self::DecodeDomain(_) | Self::EncodeDomain(_) | Self::Unencodable { .. } => None,
                          ^1                      ^1
  270|       |        }
  271|      5|    }
  272|       |
  273|       |    /// Borrows the unencodable value context carried by this error.
  274|       |    ///
  275|       |    /// # Returns
  276|       |    ///
  277|       |    /// Returns `Some((index, value))` for an unencodable error and `None`
  278|       |    /// otherwise. The inner `Some` borrows retained value context; inner
  279|       |    /// `None` means it is absent.
  280|       |    #[inline]
  281|      5|    pub const fn unencodable_ref(&self) -> Option<(usize, Option<&V>)> {
  282|      5|        match self {
  283|      2|            Self::Unencodable { input_index, value } => Some((*input_index, value.as_ref())),
  284|      3|            Self::Failure(_) | Self::DecodeDomain(_) | Self::EncodeDomain(_) => None,
                          ^1                 ^1                      ^1
  285|       |        }
  286|      5|    }
  287|       |
  288|       |    /// Maps the source domain error while preserving other errors.
  289|       |    ///
  290|       |    /// # Type Parameters
  291|       |    ///
  292|       |    /// - `F`: One-shot transformation from `DE` to `T`.
  293|       |    /// - `T`: Replacement payload type.
  294|       |    ///
  295|       |    /// # Parameters
  296|       |    ///
  297|       |    /// - `f`: Invoked only for a source-domain error; otherwise dropped without
  298|       |    ///   being called.
  299|       |    ///
  300|       |    /// # Returns
  301|       |    ///
  302|       |    /// Returns an owned error preserving all other variants and positional
  303|       |    /// context.
  304|       |    ///
  305|       |    /// # Panics
  306|       |    ///
  307|       |    /// Propagates a panic if the invoked transformation panics.
  308|       |    #[inline]
  309|      4|    pub fn map_decode_domain<F, T>(self, f: F) -> TranscodeConvertError<T, EE, V>
  310|      4|    where
  311|      4|        F: FnOnce(DE) -> T,
  312|       |    {
  313|      4|        match self {
  314|      1|            Self::Failure(failure) => TranscodeConvertError::Failure(failure),
  315|      1|            Self::DecodeDomain(error) => TranscodeConvertError::DecodeDomain(error.map_source(f)),
  316|      1|            Self::EncodeDomain(error) => TranscodeConvertError::EncodeDomain(error),
  317|      1|            Self::Unencodable { input_index, value } => TranscodeConvertError::Unencodable { input_index, value },
  318|       |        }
  319|      4|    }
  320|       |
  321|       |    /// Maps the target domain error while preserving other errors.
  322|       |    ///
  323|       |    /// # Type Parameters
  324|       |    ///
  325|       |    /// - `F`: One-shot transformation from `EE` to `T`.
  326|       |    /// - `T`: Replacement payload type.
  327|       |    ///
  328|       |    /// # Parameters
  329|       |    ///
  330|       |    /// - `f`: Invoked only for a target-domain error; otherwise dropped without
  331|       |    ///   being called.
  332|       |    ///
  333|       |    /// # Returns
  334|       |    ///
  335|       |    /// Returns an owned error preserving all other variants and positional
  336|       |    /// context.
  337|       |    ///
  338|       |    /// # Panics
  339|       |    ///
  340|       |    /// Propagates a panic if the invoked transformation panics.
  341|       |    #[inline]
  342|      4|    pub fn map_encode_domain<F, T>(self, f: F) -> TranscodeConvertError<DE, T, V>
  343|      4|    where
  344|      4|        F: FnOnce(EE) -> T,
  345|       |    {
  346|      4|        match self {
  347|      1|            Self::Failure(failure) => TranscodeConvertError::Failure(failure),
  348|      1|            Self::DecodeDomain(error) => TranscodeConvertError::DecodeDomain(error),
  349|      1|            Self::EncodeDomain(error) => TranscodeConvertError::EncodeDomain(error.map_source(f)),
  350|      1|            Self::Unencodable { input_index, value } => TranscodeConvertError::Unencodable { input_index, value },
  351|       |        }
  352|      4|    }
  353|       |
  354|       |    /// Maps value context carried by unencodable-value errors.
  355|       |    ///
  356|       |    /// # Type Parameters
  357|       |    ///
  358|       |    /// - `F`: One-shot transformation from `V` to `W`.
  359|       |    /// - `W`: Replacement payload type.
  360|       |    ///
  361|       |    /// # Parameters
  362|       |    ///
  363|       |    /// - `f`: Invoked only for present unencodable value context; otherwise
  364|       |    ///   dropped without being called.
  365|       |    ///
  366|       |    /// # Returns
  367|       |    ///
  368|       |    /// Returns an owned error preserving all other variants and positional
  369|       |    /// context.
  370|       |    ///
  371|       |    /// # Panics
  372|       |    ///
  373|       |    /// Propagates a panic if the invoked transformation panics.
  374|       |    #[inline]
  375|      5|    pub fn map_value<F, W>(self, f: F) -> TranscodeConvertError<DE, EE, W>
  376|      5|    where
  377|      5|        F: FnOnce(V) -> W,
  378|       |    {
  379|      5|        match self {
  380|      1|            Self::Failure(failure) => TranscodeConvertError::Failure(failure),
  381|      1|            Self::DecodeDomain(error) => TranscodeConvertError::DecodeDomain(error),
  382|      1|            Self::EncodeDomain(error) => TranscodeConvertError::EncodeDomain(error),
  383|      2|            Self::Unencodable { input_index, value } => TranscodeConvertError::Unencodable {
  384|      2|                input_index,
  385|      2|                value: value.map(f),
  386|      2|            },
  387|       |        }
  388|      5|    }
  389|       |}
  390|       |
  391|       |impl<DE, EE, V> From<TranscodeDecodeError<DE>> for TranscodeConvertError<DE, EE, V> {
  392|       |    /// Converts a source-side decode error into a converter error.
  393|       |    ///
  394|       |    /// # Parameters
  395|       |    ///
  396|       |    /// - `error`: Owned failure to convert without discarding its context.
  397|       |    ///
  398|       |    /// # Returns
  399|       |    ///
  400|       |    /// Returns the corresponding converter variant preserving the original
  401|       |    /// cause.
  402|       |    #[inline]
  403|     17|    fn from(error: TranscodeDecodeError<DE>) -> Self {
  404|     17|        match error {
  405|      4|            TranscodeDecodeError::Failure(failure) => Self::Failure(failure),
  406|     13|            TranscodeDecodeError::Domain(error) => Self::DecodeDomain(error),
  407|       |        }
  408|     17|    }
  409|       |}
  410|       |
  411|       |impl<DE, EE, V> From<TranscodeEncodeError<EE, V>> for TranscodeConvertError<DE, EE, V> {
  412|       |    /// Converts a target-side encode error into a converter error.
  413|       |    ///
  414|       |    /// # Parameters
  415|       |    ///
  416|       |    /// - `error`: Owned failure to convert without discarding its context.
  417|       |    ///
  418|       |    /// # Returns
  419|       |    ///
  420|       |    /// Returns the corresponding converter variant preserving the original
  421|       |    /// cause.
  422|       |    #[inline]
  423|     15|    fn from(error: TranscodeEncodeError<EE, V>) -> Self {
  424|     15|        match error {
  425|      1|            TranscodeEncodeError::Failure(failure) => Self::Failure(failure),
  426|      3|            TranscodeEncodeError::Unencodable { input_index, value } => Self::Unencodable { input_index, value },
  427|     11|            TranscodeEncodeError::Domain(error) => Self::EncodeDomain(error),
  428|       |        }
  429|     15|    }
  430|       |}
  431|       |
  432|       |impl<DE, EE, V> From<CapacityError> for TranscodeConvertError<DE, EE, V> {
  433|       |    /// Converts capacity planning errors into transcode framework errors.
  434|       |    ///
  435|       |    /// # Parameters
  436|       |    ///
  437|       |    /// - `error`: Owned failure to convert without discarding its context.
  438|       |    ///
  439|       |    /// # Returns
  440|       |    ///
  441|       |    /// Returns the corresponding converter variant preserving the original
  442|       |    /// cause.
  443|       |    #[inline]
  444|      6|    fn from(error: CapacityError) -> Self {
  445|      6|        TranscodeFailure::from(error).into()
  446|      6|    }
  447|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/transcode_decode_error.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Errors reported by transcode decoders.
    9|       |
   10|       |use core::num::NonZeroUsize;
   11|       |
   12|       |use thiserror::Error;
   13|       |
   14|       |use super::super::capacity_error::CapacityError;
   15|       |use super::super::transcode_domain_error::TranscodeDomainError;
   16|       |use super::super::transcode_failure::TranscodeFailure;
   17|       |use crate::Codec;
   18|       |use crate::DecodeFailure;
   19|       |
   20|       |/// Decode transcode error for a codec-backed decoder.
   21|       |///
   22|       |/// # Type Parameters
   23|       |///
   24|       |/// - `C`: Codec whose decode error becomes the domain failure.
   25|       |pub type TranscodeDecodeErrorOf<C> = TranscodeDecodeError<<C as Codec>::DecodeError>;
   26|       |
   27|       |/// Error reported by a decode-oriented transcode operation.
   28|       |///
   29|       |/// # Type Parameters
   30|       |///
   31|       |/// - `E`: Owned codec or policy failure preserved with transcode context.
   32|       |///
   33|       |/// # Examples
   34|       |///
   35|       |/// ```
   36|       |/// use qubit_codec::TranscodeDecodeError;
   37|       |///
   38|       |/// let error = TranscodeDecodeError::domain_main("invalid byte", 3);
   39|       |/// assert!(error.is_domain());
   40|       |/// assert_eq!(error.domain_ref(), Some(&"invalid byte"));
   41|       |/// assert!(error.failure_ref().is_none());
   42|       |/// ```
   43|       |#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
   44|       |#[must_use]
   45|       |pub enum TranscodeDecodeError<E> {
   46|       |    /// Framework-level transcode failure.
   47|       |    #[error(transparent)]
   48|       |    Failure(
   49|       |        /// Framework contract, capacity, or incomplete-input failure.
   50|       |        #[from]
   51|       |        TranscodeFailure,
   52|       |    ),
   53|       |
   54|       |    /// Domain-specific codec, charset, or policy error.
   55|       |    #[error(transparent)]
   56|       |    Domain(
   57|       |        /// Codec failure retaining its phase and source position.
   58|       |        #[from]
   59|       |        TranscodeDomainError<E>,
   60|       |    ),
   61|       |}
   62|       |
   63|       |impl<E> TranscodeDecodeError<E> {
   64|       |    /// Creates a reset-phase domain-specific transcode error.
   65|       |    ///
   66|       |    /// # Parameters
   67|       |    ///
   68|       |    /// - `source`: Owned codec or policy failure.
   69|       |    ///
   70|       |    /// # Returns
   71|       |    ///
   72|       |    /// The domain failure tagged as reset.
   73|       |    #[inline]
   74|     15|    pub const fn domain_reset(source: E) -> Self {
   75|     15|        Self::Domain(TranscodeDomainError::reset(source))
   76|     15|    }
   77|       |
   78|       |    /// Creates a main-phase domain-specific transcode error.
   79|       |    ///
   80|       |    /// # Parameters
   81|       |    ///
   82|       |    /// - `source`: Owned codec or policy failure.
   83|       |    /// - `input_index`: Absolute position of the failing input.
   84|       |    ///
   85|       |    /// # Returns
   86|       |    ///
   87|       |    /// The domain failure tagged with its main-phase input position.
   88|       |    #[inline]
   89|     21|    pub const fn domain_main(source: E, input_index: usize) -> Self {
   90|     21|        Self::Domain(TranscodeDomainError::main(source, input_index))
   91|     21|    }
   92|       |
   93|       |    /// Creates a main-phase domain-specific transcode error with decode
   94|       |    /// consumption context.
   95|       |    ///
   96|       |    /// # Parameters
   97|       |    ///
   98|       |    /// - `source`: Owned codec or policy failure.
   99|       |    /// - `input_index`: Absolute position of the failing input.
  100|       |    /// - `input_consumed`: Some(non-zero span) when known, or None for an
  101|       |    ///   unknown span.
  102|       |    ///
  103|       |    /// # Returns
  104|       |    ///
  105|       |    /// The main-phase failure retaining the known or unknown consumed span.
  106|       |    #[inline]
  107|     13|    pub const fn domain_main_with_consumed(
  108|     13|        source: E,
  109|     13|        input_index: usize,
  110|     13|        input_consumed: Option<NonZeroUsize>,
  111|     13|    ) -> Self {
  112|     13|        Self::Domain(TranscodeDomainError::main_with_consumed(
  113|     13|            source,
  114|     13|            input_index,
  115|     13|            input_consumed,
  116|     13|        ))
  117|     13|    }
  118|       |
  119|       |    /// Creates a finish-phase domain-specific transcode error.
  120|       |    ///
  121|       |    /// # Parameters
  122|       |    ///
  123|       |    /// - `source`: Owned codec or policy failure.
  124|       |    ///
  125|       |    /// # Returns
  126|       |    ///
  127|       |    /// The domain failure tagged as finish.
  128|       |    #[inline]
  129|     20|    pub const fn domain_finish(source: E) -> Self {
  130|     20|        Self::Domain(TranscodeDomainError::finish(source))
  131|     20|    }
  132|       |
  133|       |    /// Converts a low-level decode failure into a decode transcode error.
  134|       |    ///
  135|       |    /// # Parameters
  136|       |    ///
  137|       |    /// - `failure`: Owned incomplete or invalid low-level failure.
  138|       |    /// - `input_index`: Absolute failing input position.
  139|       |    /// - `available`: Number of readable units at that position.
  140|       |    ///
  141|       |    /// # Returns
  142|       |    ///
  143|       |    /// A domain error when a codec source exists; otherwise an incomplete-input
  144|       |    /// framework failure.
  145|       |    #[inline]
  146|      6|    pub fn from_decode_failure(failure: DecodeFailure<E>, input_index: usize, available: usize) -> Self {
  147|      4|        match failure {
  148|       |            DecodeFailure::Incomplete {
  149|      2|                source: Some(source), ..
  150|      2|            } => Self::domain_main(source, input_index),
  151|      2|            DecodeFailure::Incomplete { required_total, .. } => {
  152|      2|                TranscodeFailure::incomplete_input(input_index, required_total.get(), available).into()
  153|       |            }
  154|      2|            DecodeFailure::Invalid { source, consumed } => {
  155|      2|                Self::domain_main_with_consumed(source, input_index, consumed)
  156|       |            }
  157|       |        }
  158|      6|    }
  159|       |
  160|       |    /// Returns whether this error wraps a domain error.
  161|       |    ///
  162|       |    /// # Returns
  163|       |    ///
  164|       |    /// True for Domain and false for a framework Failure.
  165|       |    #[inline]
  166|       |    #[must_use]
  167|      2|    pub const fn is_domain(&self) -> bool {
  168|      2|        matches!(self, Self::Domain(_))
                      ^1             ^1
  169|      2|    }
  170|       |
  171|       |    /// Returns the framework failure carried by this error.
  172|       |    ///
  173|       |    /// # Returns
  174|       |    ///
  175|       |    /// Some(borrowed failure) for Failure, or None for Domain.
  176|       |    #[inline]
  177|      5|    pub const fn failure_ref(&self) -> Option<&TranscodeFailure> {
  178|      5|        match self {
  179|      4|            Self::Failure(failure) => Some(failure),
  180|      1|            Self::Domain(_) => None,
  181|       |        }
  182|      5|    }
  183|       |
  184|       |    /// Borrows the wrapped domain error and transcode context.
  185|       |    ///
  186|       |    /// # Returns
  187|       |    ///
  188|       |    /// Some(borrowed context) for Domain, or None for Failure.
  189|       |    #[inline]
  190|      3|    pub const fn domain_error_ref(&self) -> Option<&TranscodeDomainError<E>> {
  191|      3|        match self {
  192|      2|            Self::Domain(error) => Some(error),
  193|      1|            Self::Failure(_) => None,
  194|       |        }
  195|      3|    }
  196|       |
  197|       |    /// Borrows the wrapped domain error.
  198|       |    ///
  199|       |    /// # Returns
  200|       |    ///
  201|       |    /// Some(borrowed source) for Domain, or None for Failure.
  202|       |    #[inline]
  203|      2|    pub const fn domain_ref(&self) -> Option<&E> {
  204|      2|        match self {
  205|      1|            Self::Domain(error) => Some(error.source()),
  206|      1|            Self::Failure(_) => None,
  207|       |        }
  208|      2|    }
  209|       |
  210|       |    /// Maps the wrapped domain error while preserving framework failures.
  211|       |    ///
  212|       |    /// # Type Parameters
  213|       |    ///
  214|       |    /// - `F`: One-shot mapping closure, invoked only for a domain failure.
  215|       |    /// - `T`: Replacement domain error type.
  216|       |    ///
  217|       |    /// # Parameters
  218|       |    ///
  219|       |    /// - `f`: Consumes the domain source, if present; may perform
  220|       |    ///   caller-defined side effects. Framework failures do not invoke it.
  221|       |    ///
  222|       |    /// # Returns
  223|       |    ///
  224|       |    /// The mapped error retaining phase and input context, or the original
  225|       |    /// framework failure unchanged.
  226|       |    #[inline]
  227|      2|    pub fn map_domain<F, T>(self, f: F) -> TranscodeDecodeError<T>
  228|      2|    where
  229|      2|        F: FnOnce(E) -> T,
  230|       |    {
  231|      2|        match self {
  232|      1|            Self::Failure(failure) => TranscodeDecodeError::Failure(failure),
  233|      1|            Self::Domain(error) => TranscodeDecodeError::Domain(error.map_source(f)),
  234|       |        }
  235|      2|    }
  236|       |}
  237|       |
  238|       |impl<E> From<CapacityError> for TranscodeDecodeError<E> {
  239|       |    /// Converts capacity planning errors into transcode framework errors.
  240|       |    ///
  241|       |    /// # Parameters
  242|       |    ///
  243|       |    /// - `error`: Owned capacity planning failure.
  244|       |    ///
  245|       |    /// # Returns
  246|       |    ///
  247|       |    /// The corresponding framework failure wrapped in this decode error.
  248|       |    #[inline]
  249|      2|    fn from(error: CapacityError) -> Self {
  250|      2|        TranscodeFailure::from(error).into()
  251|      2|    }
  252|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/transcode_domain_error.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Domain-specific transcode errors with codec phase context.
    9|       |
   10|       |use core::num::NonZeroUsize;
   11|       |
   12|       |use thiserror::Error;
   13|       |
   14|       |/// Domain-specific codec, charset, or policy error with transcode context.
   15|       |///
   16|       |/// Reset and finish failures are lifecycle failures and therefore cannot carry
   17|       |/// an input index. Main-phase failures always carry the absolute input index
   18|       |/// where the domain error occurred.
   19|       |///
   20|       |/// # Type Parameters
   21|       |///
   22|       |/// - `E`: Domain error reported by the concrete codec, hook, or facade.
   23|       |///
   24|       |/// # Examples
   25|       |///
   26|       |/// ```
   27|       |/// use qubit_codec::TranscodeDomainError;
   28|       |///
   29|       |/// let error = TranscodeDomainError::main("invalid byte", 4);
   30|       |/// assert_eq!(error.input_index(), Some(4));
   31|       |/// assert_eq!(error.to_string(), "codec main error at input index 4: invalid byte");
   32|       |/// ```
   33|       |#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
   34|       |#[must_use]
   35|       |pub enum TranscodeDomainError<E> {
   36|       |    /// Domain error reported while resetting codec or hook state.
   37|       |    #[error("codec reset error: {source}")]
   38|       |    Reset {
   39|       |        /// Domain error returned by the codec or policy facade.
   40|       |        #[source]
   41|       |        source: E,
   42|       |    },
   43|       |
   44|       |    /// Domain error reported while processing input.
   45|       |    #[error("codec main error at input index {input_index}: {source}")]
   46|       |    Main {
   47|       |        /// Domain error returned by the codec or policy facade.
   48|       |        #[source]
   49|       |        source: E,
   50|       |        /// Absolute input index where the error occurred.
   51|       |        input_index: usize,
   52|       |        /// Invalid input units consumed by the codec-domain error, when known.
   53|       |        input_consumed: Option<NonZeroUsize>,
   54|       |    },
   55|       |
   56|       |    /// Domain error reported while finishing codec or hook state.
   57|       |    #[error("codec finish error: {source}")]
   58|       |    Finish {
   59|       |        /// Domain error returned by the codec or policy facade.
   60|       |        #[source]
   61|       |        source: E,
   62|       |    },
   63|       |}
   64|       |
   65|       |impl<E> TranscodeDomainError<E> {
   66|       |    /// Creates a reset-phase domain error.
   67|       |    ///
   68|       |    /// # Parameters
   69|       |    ///
   70|       |    /// - `source`: Domain error returned by reset handling.
   71|       |    ///
   72|       |    /// # Returns
   73|       |    ///
   74|       |    /// Returns a reset-phase domain error.
   75|       |    #[inline]
   76|     37|    pub const fn reset(source: E) -> Self {
   77|     37|        Self::Reset { source }
   78|     37|    }
   79|       |
   80|       |    /// Creates a main-phase domain error.
   81|       |    ///
   82|       |    /// # Parameters
   83|       |    ///
   84|       |    /// - `source`: Domain error returned by value processing.
   85|       |    /// - `input_index`: Absolute input index where the error occurred.
   86|       |    ///
   87|       |    /// # Returns
   88|       |    ///
   89|       |    /// Returns a main-phase domain error.
   90|       |    #[inline]
   91|     54|    pub const fn main(source: E, input_index: usize) -> Self {
   92|     54|        Self::Main {
   93|     54|            source,
   94|     54|            input_index,
   95|     54|            input_consumed: None,
   96|     54|        }
   97|     54|    }
   98|       |
   99|       |    /// Creates a main-phase domain error with decode consumption context.
  100|       |    ///
  101|       |    /// # Parameters
  102|       |    ///
  103|       |    /// - `source`: Domain error returned by value processing.
  104|       |    /// - `input_index`: Absolute input index where the error occurred.
  105|       |    /// - `input_consumed`: Invalid input units consumed, when known.
  106|       |    ///
  107|       |    /// # Returns
  108|       |    ///
  109|       |    /// Returns a main-phase domain error with consumption context.
  110|       |    #[inline]
  111|     17|    pub const fn main_with_consumed(source: E, input_index: usize, input_consumed: Option<NonZeroUsize>) -> Self {
  112|     17|        Self::Main {
  113|     17|            source,
  114|     17|            input_index,
  115|     17|            input_consumed,
  116|     17|        }
  117|     17|    }
  118|       |
  119|       |    /// Creates a finish-phase domain error.
  120|       |    ///
  121|       |    /// # Parameters
  122|       |    ///
  123|       |    /// - `source`: Domain error returned by finish handling.
  124|       |    ///
  125|       |    /// # Returns
  126|       |    ///
  127|       |    /// Returns a finish-phase domain error.
  128|       |    #[inline]
  129|     45|    pub const fn finish(source: E) -> Self {
  130|     45|        Self::Finish { source }
  131|     45|    }
  132|       |
  133|       |    /// Returns the wrapped source error.
  134|       |    ///
  135|       |    /// # Returns
  136|       |    ///
  137|       |    /// Returns a shared reference to the domain error.
  138|       |    #[inline]
  139|       |    #[must_use]
  140|      7|    pub const fn source(&self) -> &E {
  141|      7|        match self {
  142|      7|            Self::Reset { source } | Self::Main { source, .. } | Self::Finish { source } => source,
                                        ^2                      ^4                            ^1
  143|       |        }
  144|      7|    }
  145|       |
  146|       |    /// Consumes this error and returns the wrapped source error.
  147|       |    ///
  148|       |    /// # Returns
  149|       |    ///
  150|       |    /// Returns the domain error.
  151|       |    #[inline]
  152|       |    #[must_use]
  153|      5|    pub fn into_source(self) -> E {
  154|      5|        match self {
  155|      5|            Self::Reset { source } | Self::Main { source, .. } | Self::Finish { source } => source,
                                        ^1                      ^2                            ^2
  156|       |        }
  157|      5|    }
  158|       |
  159|       |    /// Returns the absolute input index associated with the error.
  160|       |    ///
  161|       |    /// # Returns
  162|       |    ///
  163|       |    /// Returns `Some(index)` for main-phase errors and `None` for reset or
  164|       |    /// finish errors.
  165|       |    #[inline]
  166|      2|    pub const fn input_index(&self) -> Option<usize> {
  167|      2|        match self {
  168|      1|            Self::Main { input_index, .. } => Some(*input_index),
  169|      1|            Self::Reset { .. } | Self::Finish { .. } => None,
  170|       |        }
  171|      2|    }
  172|       |
  173|       |    /// Returns invalid input consumption associated with the error.
  174|       |    ///
  175|       |    /// # Returns
  176|       |    ///
  177|       |    /// Returns `Some(consumed)` when a main-phase decode error reports the
  178|       |    /// invalid input width. Returns `None` otherwise.
  179|       |    #[inline]
  180|      2|    pub const fn input_consumed(&self) -> Option<NonZeroUsize> {
  181|      2|        match self {
  182|      1|            Self::Main { input_consumed, .. } => *input_consumed,
  183|      1|            Self::Reset { .. } | Self::Finish { .. } => None,
  184|       |        }
  185|      2|    }
  186|       |
  187|       |    /// Maps the wrapped source error while preserving phase context.
  188|       |    ///
  189|       |    /// # Type Parameters
  190|       |    ///
  191|       |    /// - `T`: Target source error type.
  192|       |    /// - `F`: Mapping function type.
  193|       |    ///
  194|       |    /// # Parameters
  195|       |    ///
  196|       |    /// - `f`: Function applied to the wrapped source error.
  197|       |    ///
  198|       |    /// # Returns
  199|       |    ///
  200|       |    /// Returns the mapped domain error.
  201|       |    #[inline]
  202|      7|    pub fn map_source<T, F>(self, f: F) -> TranscodeDomainError<T>
  203|      7|    where
  204|      7|        F: FnOnce(E) -> T,
  205|       |    {
  206|      7|        match self {
  207|      1|            Self::Reset { source } => TranscodeDomainError::Reset { source: f(source) },
  208|       |            Self::Main {
  209|      4|                source,
  210|      4|                input_index,
  211|      4|                input_consumed,
  212|      4|            } => TranscodeDomainError::Main {
  213|      4|                source: f(source),
  214|      4|                input_index,
  215|      4|                input_consumed,
  216|      4|            },
  217|      2|            Self::Finish { source } => TranscodeDomainError::Finish { source: f(source) },
  218|       |        }
  219|      7|    }
  220|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/transcode_encode_error.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Errors reported by transcode encoders.
    9|       |
   10|       |use thiserror::Error;
   11|       |
   12|       |use super::capacity_error::CapacityError;
   13|       |use super::transcode_domain_error::TranscodeDomainError;
   14|       |use super::transcode_failure::TranscodeFailure;
   15|       |use crate::Codec;
   16|       |
   17|       |/// Encode transcode error for a codec-backed encoder.
   18|       |///
   19|       |/// # Type Parameters
   20|       |///
   21|       |/// - `C`: Codec supplying the domain error and input-value types.
   22|       |pub type TranscodeEncodeErrorOf<C> = TranscodeEncodeError<<C as Codec>::EncodeError, <C as Codec>::Value>;
   23|       |
   24|       |/// Error reported by an encode-oriented transcode operation.
   25|       |///
   26|       |/// # Type Parameters
   27|       |///
   28|       |/// - `E`: Domain error preserved from the codec or policy.
   29|       |/// - `V`: Optional owned context for an unencodable input value.
   30|       |///
   31|       |/// # Examples
   32|       |///
   33|       |/// ```
   34|       |/// use qubit_codec::TranscodeEncodeError;
   35|       |///
   36|       |/// let error = TranscodeEncodeError::<(), u8>::unencodable(3, 255);
   37|       |/// assert_eq!(error.unencodable_ref(), Some((3, Some(&255))));
   38|       |/// ```
   39|       |#[derive(Clone, Debug, Eq, Error, Hash, PartialEq)]
   40|       |#[must_use]
   41|       |pub enum TranscodeEncodeError<E, V> {
   42|       |    /// Framework-level transcode failure.
   43|       |    #[error(transparent)]
   44|       |    Failure(
   45|       |        /// Framework contract failure preserved as the error source.
   46|       |        #[from]
   47|       |        TranscodeFailure,
   48|       |    ),
   49|       |
   50|       |    /// The input value cannot be encoded by the target codec and policy.
   51|       |    #[error("unencodable value at input index {input_index}")]
   52|       |    Unencodable {
   53|       |        /// Absolute input index of the value being encoded.
   54|       |        input_index: usize,
   55|       |        /// `Some` owns the rejected value; `None` means value context was
   56|       |        /// unavailable.
   57|       |        value: Option<V>,
   58|       |    },
   59|       |
   60|       |    /// Domain-specific codec, charset, or policy error.
   61|       |    #[error(transparent)]
   62|       |    Domain(
   63|       |        /// Owned domain source and its reset, main, or finish context.
   64|       |        #[from]
   65|       |        TranscodeDomainError<E>,
   66|       |    ),
   67|       |}
   68|       |
   69|       |impl<E, V> TranscodeEncodeError<E, V> {
   70|       |    /// Creates a reset-phase domain-specific transcode error.
   71|       |    ///
   72|       |    /// # Parameters
   73|       |    ///
   74|       |    /// - `source`: Owned domain error from reset.
   75|       |    ///
   76|       |    /// # Returns
   77|       |    ///
   78|       |    /// Returns a domain error carrying reset-phase context.
   79|       |    #[inline]
   80|     13|    pub const fn domain_reset(source: E) -> Self {
   81|     13|        Self::Domain(TranscodeDomainError::reset(source))
   82|     13|    }
   83|       |
   84|       |    /// Creates a main-phase domain-specific transcode error.
   85|       |    ///
   86|       |    /// # Parameters
   87|       |    ///
   88|       |    /// - `source`: Owned domain error during encoding.
   89|       |    /// - `input_index`: Absolute index of the failed input value.
   90|       |    ///
   91|       |    /// # Returns
   92|       |    ///
   93|       |    /// Returns a domain error carrying the input index and main-phase context.
   94|       |    #[inline]
   95|     21|    pub const fn domain_main(source: E, input_index: usize) -> Self {
   96|     21|        Self::Domain(TranscodeDomainError::main(source, input_index))
   97|     21|    }
   98|       |
   99|       |    /// Creates a finish-phase domain-specific transcode error.
  100|       |    ///
  101|       |    /// # Parameters
  102|       |    ///
  103|       |    /// - `source`: Owned domain error from finalization.
  104|       |    ///
  105|       |    /// # Returns
  106|       |    ///
  107|       |    /// Returns a domain error carrying finish-phase context.
  108|       |    #[inline]
  109|     16|    pub const fn domain_finish(source: E) -> Self {
  110|     16|        Self::Domain(TranscodeDomainError::finish(source))
  111|     16|    }
  112|       |
  113|       |    /// Creates an unencodable-value error with value context.
  114|       |    ///
  115|       |    /// # Parameters
  116|       |    ///
  117|       |    /// - `input_index`: Absolute index of the rejected value.
  118|       |    /// - `value`: Rejected value transferred into the error.
  119|       |    ///
  120|       |    /// # Returns
  121|       |    ///
  122|       |    /// Returns an unencodable error with owned value context.
  123|       |    #[inline]
  124|      4|    pub fn unencodable(input_index: usize, value: V) -> Self {
  125|      4|        Self::Unencodable {
  126|      4|            input_index,
  127|      4|            value: Some(value),
  128|      4|        }
  129|      4|    }
  130|       |
  131|       |    /// Creates an unencodable-value error without value context.
  132|       |    ///
  133|       |    /// # Parameters
  134|       |    ///
  135|       |    /// - `input_index`: Absolute index of the rejected input.
  136|       |    ///
  137|       |    /// # Returns
  138|       |    ///
  139|       |    /// Returns an unencodable error with no retained value.
  140|       |    #[inline]
  141|     19|    pub const fn unencodable_without_context(input_index: usize) -> Self {
  142|     19|        Self::Unencodable {
  143|     19|            input_index,
  144|     19|            value: None,
  145|     19|        }
  146|     19|    }
  147|       |
  148|       |    /// Returns whether this error wraps a domain error.
  149|       |    ///
  150|       |    /// # Returns
  151|       |    ///
  152|       |    /// Returns `true` only for the Domain variant.
  153|       |    #[inline]
  154|       |    #[must_use]
  155|      2|    pub const fn is_domain(&self) -> bool {
  156|      2|        matches!(self, Self::Domain(_))
                      ^1             ^1
  157|      2|    }
  158|       |
  159|       |    /// Returns the framework failure carried by this error.
  160|       |    ///
  161|       |    /// # Returns
  162|       |    ///
  163|       |    /// Returns `Some` borrowed framework failure for Failure, otherwise `None`.
  164|       |    #[inline]
  165|      2|    pub const fn failure_ref(&self) -> Option<&TranscodeFailure> {
  166|      2|        match self {
  167|      1|            Self::Failure(failure) => Some(failure),
  168|      1|            Self::Unencodable { .. } | Self::Domain(_) => None,
  169|       |        }
  170|      2|    }
  171|       |
  172|       |    /// Borrows the wrapped domain error and transcode context.
  173|       |    ///
  174|       |    /// # Returns
  175|       |    ///
  176|       |    /// Returns `Some` borrowed domain error and phase context for Domain,
  177|       |    /// or `None` for framework and unencodable errors.
  178|       |    #[inline]
  179|      2|    pub const fn domain_error_ref(&self) -> Option<&TranscodeDomainError<E>> {
  180|      2|        match self {
  181|      1|            Self::Domain(error) => Some(error),
  182|      1|            Self::Failure(_) | Self::Unencodable { .. } => None,
  183|       |        }
  184|      2|    }
  185|       |
  186|       |    /// Borrows the wrapped domain error.
  187|       |    ///
  188|       |    /// # Returns
  189|       |    ///
  190|       |    /// Returns `Some` borrowed domain source for Domain, otherwise `None`.
  191|       |    #[inline]
  192|      5|    pub const fn domain_ref(&self) -> Option<&E> {
  193|      5|        match self {
  194|      4|            Self::Domain(error) => Some(error.source()),
  195|      1|            Self::Failure(_) | Self::Unencodable { .. } => None,
  196|       |        }
  197|      5|    }
  198|       |
  199|       |    /// Borrows the unencodable value context carried by this error.
  200|       |    ///
  201|       |    /// # Returns
  202|       |    ///
  203|       |    /// Returns `Some((index, value))` for Unencodable, otherwise `None`.
  204|       |    /// The nested option is `Some` when a retained value can be borrowed,
  205|       |    /// and `None` when the error has only an input index.
  206|       |    #[inline]
  207|      4|    pub const fn unencodable_ref(&self) -> Option<(usize, Option<&V>)> {
  208|      4|        match self {
  209|      2|            Self::Unencodable { input_index, value } => Some((*input_index, value.as_ref())),
  210|      2|            Self::Failure(_) | Self::Domain(_) => None,
                          ^1                 ^1
  211|       |        }
  212|      4|    }
  213|       |
  214|       |    /// Maps the wrapped domain error while preserving other errors.
  215|       |    ///
  216|       |    /// # Type Parameters
  217|       |    ///
  218|       |    /// - `F`: Callback consuming the domain source.
  219|       |    /// - `T`: Replacement domain error type.
  220|       |    ///
  221|       |    /// # Parameters
  222|       |    ///
  223|       |    /// - `f`: Called exactly once for Domain, never for other variants.
  224|       |    ///
  225|       |    /// # Returns
  226|       |    ///
  227|       |    /// Returns an error with the transformed domain source while preserving
  228|       |    /// phase, input position, framework failures, and unencodable value
  229|       |    /// context.
  230|       |    #[inline]
  231|      3|    pub fn map_domain<F, T>(self, f: F) -> TranscodeEncodeError<T, V>
  232|      3|    where
  233|      3|        F: FnOnce(E) -> T,
  234|       |    {
  235|      3|        match self {
  236|      1|            Self::Failure(failure) => TranscodeEncodeError::Failure(failure),
  237|      1|            Self::Unencodable { input_index, value } => TranscodeEncodeError::Unencodable { input_index, value },
  238|      1|            Self::Domain(error) => TranscodeEncodeError::Domain(error.map_source(f)),
  239|       |        }
  240|      3|    }
  241|       |
  242|       |    /// Maps value context carried by unencodable-value errors.
  243|       |    ///
  244|       |    /// # Type Parameters
  245|       |    ///
  246|       |    /// - `F`: Callback consuming a retained value.
  247|       |    /// - `W`: Replacement value-context type.
  248|       |    ///
  249|       |    /// # Parameters
  250|       |    ///
  251|       |    /// - `f`: Called once for a retained unencodable value, otherwise not
  252|       |    ///   called.
  253|       |    ///
  254|       |    /// # Returns
  255|       |    ///
  256|       |    /// Returns an error with transformed value context; absent context and
  257|       |    /// other variants are preserved.
  258|       |    #[inline]
  259|      3|    pub fn map_value<F, W>(self, f: F) -> TranscodeEncodeError<E, W>
  260|      3|    where
  261|      3|        F: FnOnce(V) -> W,
  262|       |    {
  263|      3|        match self {
  264|      1|            Self::Failure(failure) => TranscodeEncodeError::Failure(failure),
  265|      1|            Self::Unencodable { input_index, value } => TranscodeEncodeError::Unencodable {
  266|      1|                input_index,
  267|      1|                value: value.map(f),
  268|      1|            },
  269|      1|            Self::Domain(error) => TranscodeEncodeError::Domain(error),
  270|       |        }
  271|      3|    }
  272|       |}
  273|       |
  274|       |impl<E, V> From<CapacityError> for TranscodeEncodeError<E, V> {
  275|       |    /// Converts capacity planning errors into transcode framework errors.
  276|       |    ///
  277|       |    /// # Parameters
  278|       |    ///
  279|       |    /// - `error`: Capacity failure from output planning.
  280|       |    ///
  281|       |    /// # Returns
  282|       |    ///
  283|       |    /// Returns the corresponding framework failure variant.
  284|       |    #[inline]
  285|      5|    fn from(error: CapacityError) -> Self {
  286|      5|        TranscodeFailure::from(error).into()
  287|      5|    }
  288|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/transcode_failure.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Framework-level failures reported by safe transcode APIs.
    9|       |
   10|       |use thiserror::Error;
   11|       |
   12|       |use super::capacity_error::CapacityError;
   13|       |use super::transcode_contract_error::TranscodeContractError;
   14|       |use crate::Codec;
   15|       |use crate::codec::assert_unit_bounds;
   16|       |
   17|       |/// Framework-level failure reported by a transcode operation.
   18|       |///
   19|       |/// These failures are part of the safe public transcode API. They describe
   20|       |/// caller-supplied buffer ranges, capacity planning, complete-input shape, and
   21|       |/// stream lifecycle misuse that the transcode layer can detect without
   22|       |/// interpreting a concrete codec error.
   23|       |///
   24|       |/// # Examples
   25|       |///
   26|       |/// ```
   27|       |/// use qubit_codec::TranscodeFailure;
   28|       |///
   29|       |/// let failure = TranscodeFailure::invalid_input_index(4, 3);
   30|       |/// assert_eq!(failure.to_string(), "invalid input index 4 for input length 3");
   31|       |/// assert_eq!(TranscodeFailure::ensure_input_index(3, 4), Err(failure));
   32|       |/// ```
   33|       |#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
   34|       |#[non_exhaustive]
   35|       |#[must_use]
   36|       |pub enum TranscodeFailure {
   37|       |    /// The caller supplied an input index outside the input slice.
   38|       |    #[error("invalid input index {index} for input length {input_len}")]
   39|       |    InvalidInputIndex {
   40|       |        /// Invalid input index supplied by the caller.
   41|       |        index: usize,
   42|       |        /// Length of the input slice.
   43|       |        input_len: usize,
   44|       |    },
   45|       |
   46|       |    /// The caller supplied an output index outside the output slice.
   47|       |    #[error("invalid output index {index} for output length {output_len}")]
   48|       |    InvalidOutputIndex {
   49|       |        /// Invalid output index supplied by the caller.
   50|       |        index: usize,
   51|       |        /// Length of the output slice.
   52|       |        output_len: usize,
   53|       |    },
   54|       |
   55|       |    /// The caller supplied an output range extending beyond the output slice.
   56|       |    #[error("invalid output range at index {output_index} with length {range_len} for output length {output_len}")]
   57|       |    InvalidOutputRange {
   58|       |        /// Absolute output index where the range starts.
   59|       |        output_index: usize,
   60|       |        /// Length of the caller-supplied output range.
   61|       |        range_len: usize,
   62|       |        /// Length of the output slice.
   63|       |        output_len: usize,
   64|       |    },
   65|       |
   66|       |    /// The output slice cannot hold all required output.
   67|       |    #[error("insufficient output at index {output_index}: required {required} units, available {available}")]
   68|       |    InsufficientOutput {
   69|       |        /// Absolute output index where writing would start.
   70|       |        output_index: usize,
   71|       |        /// Output units required from `output_index`.
   72|       |        required: usize,
   73|       |        /// Output units available from `output_index`.
   74|       |        available: usize,
   75|       |    },
   76|       |
   77|       |    /// Output length arithmetic overflowed.
   78|       |    #[error("output length arithmetic overflow")]
   79|       |    OutputLengthOverflow,
   80|       |
   81|       |    /// The allocator could not reserve the requested output storage.
   82|       |    #[error("output allocation failed")]
   83|       |    AllocationFailed,
   84|       |
   85|       |    /// The complete input ended with an incomplete value.
   86|       |    #[error(
   87|       |        "incomplete input at index {input_index}: at least {required} units required to retry, {available} available"
   88|       |    )]
   89|       |    IncompleteInput {
   90|       |        /// Absolute input index where the incomplete value starts.
   91|       |        input_index: usize,
   92|       |        /// Current minimum total input units required from `input_index`
   93|       |        /// before retrying. A later retry may raise this lower bound.
   94|       |        required: usize,
   95|       |        /// Input units available from `input_index`.
   96|       |        available: usize,
   97|       |    },
   98|       |
   99|       |    /// A transcoder returned progress inconsistent with the supplied buffers.
  100|       |    #[error("invalid transcoder progress: {source}")]
  101|       |    InvalidProgress {
  102|       |        /// Contract error describing the invalid progress.
  103|       |        #[source]
  104|       |        source: TranscodeContractError,
  105|       |    },
  106|       |
  107|       |    /// The input contains exactly one decoded value plus trailing units.
  108|       |    #[error("trailing input after value: consumed {consumed} units, remaining {remaining}")]
  109|       |    TrailingInput {
  110|       |        /// Units consumed by the decoded value.
  111|       |        consumed: usize,
  112|       |        /// Extra units left after the decoded value.
  113|       |        remaining: usize,
  114|       |    },
  115|       |
  116|       |    /// A strict single-value decoder cannot expose codec lifecycle output.
  117|       |    #[error(
  118|       |        "strict single-value decoding does not support lifecycle output: reset bound {reset_bound}, finish bound {finish_bound}"
  119|       |    )]
  120|       |    UnsupportedDecodeLifecycleOutput {
  121|       |        /// Maximum values the codec may emit while resetting decode state.
  122|       |        reset_bound: usize,
  123|       |        /// Maximum values the codec may emit while finishing decode state.
  124|       |        finish_bound: usize,
  125|       |    },
  126|       |
  127|       |    /// `transcode` was called after the logical stream was finished.
  128|       |    #[error("transcode called after finish without an intervening reset")]
  129|       |    TranscodeAfterFinish,
  130|       |
  131|       |    /// `transcode` was called before the first successful reset.
  132|       |    #[error("transcode called before reset")]
  133|       |    TranscodeBeforeReset,
  134|       |
  135|       |    /// `finish` was called after the logical stream was already finished.
  136|       |    #[error("finish called twice without an intervening reset")]
  137|       |    FinishAfterFinish,
  138|       |
  139|       |    /// `finish` was called before the first successful reset.
  140|       |    #[error("finish called before reset")]
  141|       |    FinishBeforeReset,
  142|       |
  143|       |    /// Reset or finish failed after codec, hook, or buffered state may have
  144|       |    /// changed. Continuing could repeat side effects or observe inconsistent
  145|       |    /// state; a successful reset is required before further use.
  146|       |    #[error("transcoder lifecycle is poisoned; a successful reset is required")]
  147|       |    LifecyclePoisoned,
  148|       |}
  149|       |
  150|       |impl TranscodeFailure {
  151|       |    /// Creates an invalid-input-index error.
  152|       |    ///
  153|       |    /// # Parameters
  154|       |    ///
  155|       |    /// - `index`: Absolute input index rejected by the caller boundary.
  156|       |    /// - `len`: Total number of input units.
  157|       |    ///
  158|       |    /// # Returns
  159|       |    ///
  160|       |    /// Returns a failure retaining the supplied index and input length.
  161|       |    #[inline]
  162|     25|    pub const fn invalid_input_index(index: usize, len: usize) -> Self {
  163|     25|        Self::InvalidInputIndex { index, input_len: len }
  164|     25|    }
  165|       |
  166|       |    /// Creates an invalid-output-index error.
  167|       |    ///
  168|       |    /// # Parameters
  169|       |    ///
  170|       |    /// - `index`: Absolute output index rejected by the caller boundary.
  171|       |    /// - `len`: Total number of writable output units.
  172|       |    ///
  173|       |    /// # Returns
  174|       |    ///
  175|       |    /// Returns a failure retaining the supplied index and output length.
  176|       |    #[inline]
  177|     49|    pub const fn invalid_output_index(index: usize, len: usize) -> Self {
  178|     49|        Self::InvalidOutputIndex { index, output_len: len }
  179|     49|    }
  180|       |
  181|       |    /// Creates an invalid-output-range error.
  182|       |    ///
  183|       |    /// # Parameters
  184|       |    ///
  185|       |    /// - `output_index`: Absolute start of the requested output range.
  186|       |    /// - `range_len`: Requested range length.
  187|       |    /// - `output_len`: Total output slice length.
  188|       |    ///
  189|       |    /// # Returns
  190|       |    ///
  191|       |    /// Returns a failure describing the range that exceeds the output slice.
  192|       |    #[inline]
  193|      1|    pub const fn invalid_output_range(output_index: usize, range_len: usize, output_len: usize) -> Self {
  194|      1|        Self::InvalidOutputRange {
  195|      1|            output_index,
  196|      1|            range_len,
  197|      1|            output_len,
  198|      1|        }
  199|      1|    }
  200|       |
  201|       |    /// Creates an insufficient-output error.
  202|       |    ///
  203|       |    /// # Parameters
  204|       |    ///
  205|       |    /// - `output_index`: Absolute start of the writable output.
  206|       |    /// - `required`: Units needed from that index.
  207|       |    /// - `available`: Units actually available from that index.
  208|       |    ///
  209|       |    /// # Returns
  210|       |    ///
  211|       |    /// Returns a failure retaining the required and available capacities.
  212|       |    #[inline]
  213|     44|    pub const fn insufficient_output(output_index: usize, required: usize, available: usize) -> Self {
  214|     44|        Self::InsufficientOutput {
  215|     44|            output_index,
  216|     44|            required,
  217|     44|            available,
  218|     44|        }
  219|     44|    }
  220|       |
  221|       |    /// Creates an output-length-overflow error.
  222|       |    ///
  223|       |    /// # Returns
  224|       |    ///
  225|       |    /// Returns the framework marker for unrepresentable output capacity.
  226|       |    #[inline]
  227|     17|    pub const fn output_length_overflow() -> Self {
  228|     17|        Self::OutputLengthOverflow
  229|     17|    }
  230|       |
  231|       |    /// Creates an output-allocation failure.
  232|       |    ///
  233|       |    /// # Returns
  234|       |    ///
  235|       |    /// Returns the framework marker for a failed output allocation.
  236|       |    #[inline]
  237|      9|    pub const fn allocation_failed() -> Self {
  238|      9|        Self::AllocationFailed
  239|      9|    }
  240|       |
  241|       |    /// Creates an incomplete-input error.
  242|       |    ///
  243|       |    /// # Parameters
  244|       |    ///
  245|       |    /// - `input_index`: Absolute index where the incomplete value starts.
  246|       |    /// - `required`: Current minimum total units needed from that index.
  247|       |    /// - `available`: Units currently available from that index.
  248|       |    ///
  249|       |    /// # Returns
  250|       |    ///
  251|       |    /// Returns a failure preserving the incomplete tail and retry requirement.
  252|       |    #[inline]
  253|     41|    pub const fn incomplete_input(input_index: usize, required: usize, available: usize) -> Self {
  254|     41|        Self::IncompleteInput {
  255|     41|            input_index,
  256|     41|            required,
  257|     41|            available,
  258|     41|        }
  259|     41|    }
  260|       |
  261|       |    /// Creates an invalid-progress error.
  262|       |    ///
  263|       |    /// # Parameters
  264|       |    ///
  265|       |    /// - `source`: Contract violation detected in a transcoder report.
  266|       |    ///
  267|       |    /// # Returns
  268|       |    ///
  269|       |    /// Returns a framework failure that retains the contract error as its
  270|       |    /// source.
  271|       |    #[inline]
  272|      6|    pub const fn invalid_progress(source: TranscodeContractError) -> Self {
  273|      6|        Self::InvalidProgress { source }
  274|      6|    }
  275|       |
  276|       |    /// Creates a trailing-input error.
  277|       |    ///
  278|       |    /// # Parameters
  279|       |    ///
  280|       |    /// - `consumed`: Units used by the decoded value.
  281|       |    /// - `remaining`: Extra units after that value.
  282|       |    ///
  283|       |    /// # Returns
  284|       |    ///
  285|       |    /// Returns a failure describing the otherwise successful single-value
  286|       |    /// decode.
  287|       |    #[inline]
  288|      9|    pub const fn trailing_input(consumed: usize, remaining: usize) -> Self {
  289|      9|        Self::TrailingInput { consumed, remaining }
  290|      9|    }
  291|       |
  292|       |    /// Creates an unsupported decode-lifecycle-output error.
  293|       |    ///
  294|       |    /// # Parameters
  295|       |    ///
  296|       |    /// - `reset_bound`: Maximum values produced by decode reset.
  297|       |    /// - `finish_bound`: Maximum values produced by decode finish.
  298|       |    ///
  299|       |    /// # Returns
  300|       |    ///
  301|       |    /// Returns a failure retaining the lifecycle output bounds rejected by
  302|       |    /// strict decoding.
  303|       |    #[inline]
  304|      3|    pub const fn unsupported_decode_lifecycle_output(reset_bound: usize, finish_bound: usize) -> Self {
  305|      3|        Self::UnsupportedDecodeLifecycleOutput {
  306|      3|            reset_bound,
  307|      3|            finish_bound,
  308|      3|        }
  309|      3|    }
  310|       |
  311|       |    /// Validates that `input_index` is within an input slice.
  312|       |    ///
  313|       |    /// # Parameters
  314|       |    ///
  315|       |    /// - `input_len`: Total input slice length.
  316|       |    /// - `input_index`: Absolute start index, including the end boundary.
  317|       |    ///
  318|       |    /// # Returns
  319|       |    ///
  320|       |    /// Returns `Ok(())` when the index is at most the input length.
  321|       |    ///
  322|       |    /// # Errors
  323|       |    ///
  324|       |    /// Returns [`Self::InvalidInputIndex`] when the index exceeds the length.
  325|       |    #[inline]
  326|  4.16k|    pub fn ensure_input_index(input_len: usize, input_index: usize) -> Result<(), Self> {
  327|  4.16k|        if input_index > input_len {
  328|     14|            return Err(Self::invalid_input_index(input_index, input_len));
  329|  4.15k|        }
  330|  4.15k|        Ok(())
  331|  4.16k|    }
  332|       |
  333|       |    /// Validates that enough input units are available from `input_index`.
  334|       |    ///
  335|       |    /// # Parameters
  336|       |    ///
  337|       |    /// - `input_len`: Total input slice length.
  338|       |    /// - `input_index`: Absolute start of the input region.
  339|       |    /// - `min_required`: Minimum readable units required from that start.
  340|       |    ///
  341|       |    /// # Returns
  342|       |    ///
  343|       |    /// Returns `Ok(())` when the start index and minimum readable capacity are
  344|       |    /// valid.
  345|       |    ///
  346|       |    /// # Errors
  347|       |    ///
  348|       |    /// Returns [`Self::InvalidInputIndex`] for an invalid start, or
  349|       |    /// [`Self::IncompleteInput`] when fewer than `min_required` units remain.
  350|       |    #[inline]
  351|     34|    pub fn ensure_min_input(input_len: usize, input_index: usize, min_required: usize) -> Result<(), Self> {
  352|     34|        Self::ensure_input_index(input_len, input_index)?;
                                                                      ^1
  353|     33|        let available = input_len - input_index;
  354|     33|        if available < min_required {
  355|      3|            return Err(Self::incomplete_input(input_index, min_required, available));
  356|     30|        }
  357|     30|        Ok(())
  358|     34|    }
  359|       |
  360|       |    /// Validates a consumed input count and rejects trailing input.
  361|       |    ///
  362|       |    /// # Parameters
  363|       |    ///
  364|       |    /// - `consumed`: Number of input units consumed by the decoded value.
  365|       |    /// - `input_len`: Total number of input units supplied.
  366|       |    ///
  367|       |    /// # Returns
  368|       |    ///
  369|       |    /// Returns `Ok(())` when `consumed == input_len`.
  370|       |    ///
  371|       |    /// # Errors
  372|       |    ///
  373|       |    /// Returns [`TranscodeFailure::InvalidInputIndex`] when `consumed` exceeds
  374|       |    /// `input_len`, or [`TranscodeFailure::TrailingInput`] when unconsumed
  375|       |    /// input remains.
  376|       |    #[inline]
  377|     29|    pub fn ensure_no_trailing_input(consumed: usize, input_len: usize) -> Result<(), Self> {
  378|     29|        Self::ensure_input_index(input_len, consumed)?;
                                                                   ^2
  379|     27|        let remaining = input_len - consumed;
  380|     27|        if remaining != 0 {
  381|      3|            return Err(Self::trailing_input(consumed, remaining));
  382|     24|        }
  383|     24|        Ok(())
  384|     29|    }
  385|       |
  386|       |    /// Validates that `output_index` is within an output slice.
  387|       |    ///
  388|       |    /// # Parameters
  389|       |    ///
  390|       |    /// - `output_len`: Total output slice length.
  391|       |    /// - `output_index`: Absolute start index, including the end boundary.
  392|       |    ///
  393|       |    /// # Returns
  394|       |    ///
  395|       |    /// Returns `Ok(())` when the index is at most the output length.
  396|       |    ///
  397|       |    /// # Errors
  398|       |    ///
  399|       |    /// Returns [`Self::InvalidOutputIndex`] when the index exceeds the length.
  400|       |    #[inline]
  401|  12.1k|    pub fn ensure_output_index(output_len: usize, output_index: usize) -> Result<(), Self> {
  402|  12.1k|        if output_index > output_len {
  403|     24|            return Err(Self::invalid_output_index(output_index, output_len));
  404|  12.1k|        }
  405|  12.1k|        Ok(())
  406|  12.1k|    }
  407|       |
  408|       |    /// Validates input and output start indices for a transcode call.
  409|       |    ///
  410|       |    /// # Parameters
  411|       |    ///
  412|       |    /// - `input_len`: Total input slice length.
  413|       |    /// - `input_index`: Absolute starting input index.
  414|       |    /// - `output_len`: Total output slice length.
  415|       |    /// - `output_index`: Absolute starting output index.
  416|       |    ///
  417|       |    /// # Returns
  418|       |    ///
  419|       |    /// Returns `Ok(())` when both indices lie within their slices or at their
  420|       |    /// ends.
  421|       |    ///
  422|       |    /// # Errors
  423|       |    ///
  424|       |    /// Returns [`Self::InvalidInputIndex`] first for invalid input, otherwise
  425|       |    /// [`Self::InvalidOutputIndex`] for invalid output.
  426|       |    #[inline]
  427|  4.09k|    pub fn ensure_transcode_indices(
  428|  4.09k|        input_len: usize,
  429|  4.09k|        input_index: usize,
  430|  4.09k|        output_len: usize,
  431|  4.09k|        output_index: usize,
  432|  4.09k|    ) -> Result<(), Self> {
  433|  4.09k|        Self::ensure_input_index(input_len, input_index)?;
                                                                      ^9
  434|  4.09k|        Self::ensure_output_index(output_len, output_index)
  435|  4.09k|    }
  436|       |
  437|       |    /// Validates that an output slice can hold one-shot finalization output.
  438|       |    ///
  439|       |    /// # Parameters
  440|       |    ///
  441|       |    /// - `output_len`: Total output slice length.
  442|       |    /// - `output_index`: Absolute start of the writable region.
  443|       |    /// - `required`: Minimum output units needed from that start.
  444|       |    ///
  445|       |    /// # Returns
  446|       |    ///
  447|       |    /// Returns `Ok(())` when the output region has at least `required` units.
  448|       |    ///
  449|       |    /// # Errors
  450|       |    ///
  451|       |    /// Returns [`Self::InvalidOutputIndex`] for an invalid start, or
  452|       |    /// [`Self::InsufficientOutput`] when the remaining capacity is too small.
  453|       |    #[inline]
  454|  8.03k|    pub fn ensure_output_capacity(output_len: usize, output_index: usize, required: usize) -> Result<(), Self> {
  455|  8.03k|        Self::ensure_output_index(output_len, output_index)?;
                                                                         ^12
  456|  8.02k|        let available = output_len - output_index;
  457|  8.02k|        if available < required {
  458|     22|            return Err(Self::insufficient_output(output_index, required, available));
  459|  8.00k|        }
  460|  8.00k|        Ok(())
  461|  8.03k|    }
  462|       |
  463|       |    /// Validates an indexed output range and its minimum writable capacity.
  464|       |    ///
  465|       |    /// # Parameters
  466|       |    ///
  467|       |    /// - `output_len`: Total output slice length.
  468|       |    /// - `output_index`: Absolute start of the writable range.
  469|       |    /// - `range_len`: Length of the caller-selected writable range.
  470|       |    /// - `required`: Minimum capacity required within the selected range.
  471|       |    ///
  472|       |    /// # Returns
  473|       |    ///
  474|       |    /// Returns `Ok(())` when the selected range fits and has enough capacity.
  475|       |    ///
  476|       |    /// # Errors
  477|       |    ///
  478|       |    /// Returns [`Self::InvalidOutputIndex`] for an invalid start,
  479|       |    /// [`Self::InvalidOutputRange`] when the range exceeds the slice, or
  480|       |    /// [`Self::InsufficientOutput`] when the selected range is too short.
  481|       |    #[inline]
  482|      8|    pub fn ensure_output_range(
  483|      8|        output_len: usize,
  484|      8|        output_index: usize,
  485|      8|        range_len: usize,
  486|      8|        required: usize,
  487|      8|    ) -> Result<(), Self> {
  488|      8|        Self::ensure_output_index(output_len, output_index)?;
                                                                         ^1
  489|      7|        let available = output_len - output_index;
  490|      7|        if range_len > available {
  491|      1|            return Err(Self::invalid_output_range(output_index, range_len, output_len));
  492|      6|        }
  493|      6|        if range_len < required {
  494|      3|            return Err(Self::insufficient_output(output_index, required, range_len));
  495|      3|        }
  496|      3|        Ok(())
  497|      8|    }
  498|       |
  499|       |    /// Rejects lifecycle output for a strict single-value decode.
  500|       |    ///
  501|       |    /// # Type Parameters
  502|       |    ///
  503|       |    /// - `C`: Codec whose decode lifecycle bounds are checked.
  504|       |    ///
  505|       |    /// # Returns
  506|       |    ///
  507|       |    /// Returns `Ok(())` when both decode lifecycle output bounds are zero.
  508|       |    ///
  509|       |    /// # Errors
  510|       |    ///
  511|       |    /// Returns [`Self::UnsupportedDecodeLifecycleOutput`] when reset or finish
  512|       |    /// may emit values.
  513|       |    ///
  514|       |    /// # Panics
  515|       |    ///
  516|       |    /// Panics at compile time when `C` violates the required codec unit bounds.
  517|       |    #[inline]
  518|     83|    pub(crate) fn ensure_no_decode_lifecycle_output<C>() -> Result<(), Self>
  519|     83|    where
  520|     83|        C: Codec,
  521|       |    {
  522|     83|        assert_unit_bounds::<C>();
  523|     83|        if C::MAX_DECODE_RESET_VALUES != 0 || C::MAX_DECODE_FINISH_VALUES != 0 {
                                                            ^81
  524|      2|            return Err(Self::unsupported_decode_lifecycle_output(
  525|      2|                C::MAX_DECODE_RESET_VALUES,
  526|      2|                C::MAX_DECODE_FINISH_VALUES,
  527|      2|            ));
  528|     81|        }
  529|     81|        Ok(())
  530|     83|    }
  531|       |}
  532|       |
  533|       |impl From<CapacityError> for TranscodeFailure {
  534|       |    /// Converts capacity planning errors into transcode framework failures.
  535|       |    ///
  536|       |    /// # Parameters
  537|       |    ///
  538|       |    /// - `error`: Capacity-planning failure to translate.
  539|       |    ///
  540|       |    /// # Returns
  541|       |    ///
  542|       |    /// Returns the corresponding framework output-length-overflow failure.
  543|       |    #[inline]
  544|     17|    fn from(error: CapacityError) -> Self {
  545|     17|        match error {
  546|     17|            CapacityError::OutputLengthOverflow => Self::OutputLengthOverflow,
  547|       |        }
  548|     17|    }
  549|       |}
  550|       |
  551|       |impl From<TranscodeContractError> for TranscodeFailure {
  552|       |    /// Converts a broken transcoder progress report into a framework failure.
  553|       |    ///
  554|       |    /// # Parameters
  555|       |    ///
  556|       |    /// - `error`: Progress-contract violation to retain as the error source.
  557|       |    ///
  558|       |    /// # Returns
  559|       |    ///
  560|       |    /// Returns [`Self::InvalidProgress`] carrying the original violation.
  561|       |    #[inline]
  562|      0|    fn from(error: TranscodeContractError) -> Self {
  563|      0|        Self::invalid_progress(error)
  564|      0|    }
  565|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/transcode_progress.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Progress reported by transcode operations.
    9|       |
   10|       |use core::num::NonZeroUsize;
   11|       |
   12|       |use super::super::TranscodeContractError;
   13|       |use super::super::TranscodeStatus;
   14|       |
   15|       |/// Counts how much work a [`crate::Transcoder`] completed before
   16|       |/// returning.
   17|       |///
   18|       |/// # Examples
   19|       |///
   20|       |/// ```
   21|       |/// use qubit_codec::TranscodeProgress;
   22|       |/// use qubit_codec::TranscodeStatus;
   23|       |///
   24|       |/// let progress = TranscodeProgress::complete(3, 6);
   25|       |/// assert_eq!(progress.read(), 3);
   26|       |/// assert_eq!(progress.written(), 6);
   27|       |/// assert!(matches!(progress.status(), TranscodeStatus::Complete));
   28|       |/// ```
   29|       |#[derive(Clone, Copy, Debug, Eq, PartialEq)]
   30|       |#[must_use]
   31|       |pub struct TranscodeProgress {
   32|       |    /// Stop reason reported by the transcoder.
   33|       |    status: TranscodeStatus,
   34|       |    /// Number of input units consumed from the requested input index.
   35|       |    read: usize,
   36|       |    /// Number of output units written from the requested output index.
   37|       |    written: usize,
   38|       |}
   39|       |
   40|       |impl TranscodeProgress {
   41|       |    /// Creates a progress value.
   42|       |    ///
   43|       |    /// # Parameters
   44|       |    ///
   45|       |    /// - `status`: The reason conversion stopped.
   46|       |    /// - `read`: Number of input units consumed from the call's input index.
   47|       |    /// - `written`: Number of output units written from the call's output
   48|       |    ///   index.
   49|       |    ///
   50|       |    /// # Returns
   51|       |    ///
   52|       |    /// Returns a progress value carrying the supplied counters.
   53|       |    #[inline]
   54|  13.4k|    pub const fn new(status: TranscodeStatus, read: usize, written: usize) -> Self {
   55|  13.4k|        Self { status, read, written }
   56|  13.4k|    }
   57|       |
   58|       |    /// Creates a completed progress value.
   59|       |    ///
   60|       |    /// # Parameters
   61|       |    ///
   62|       |    /// - `read`: Number of consumed input units.
   63|       |    /// - `written`: Number of produced output units.
   64|       |    ///
   65|       |    /// # Returns
   66|       |    ///
   67|       |    /// Returns a progress value whose status is [`TranscodeStatus::Complete`].
   68|       |    #[inline]
   69|  3.00k|    pub const fn complete(read: usize, written: usize) -> Self {
   70|  3.00k|        Self::new(TranscodeStatus::Complete, read, written)
   71|  3.00k|    }
   72|       |
   73|       |    /// Creates progress that stopped because more input is needed.
   74|       |    ///
   75|       |    /// # Parameters
   76|       |    ///
   77|       |    /// - `required`: Current minimum total input units required before retrying
   78|       |    ///   from the current input position. A later retry may raise this lower
   79|       |    ///   bound.
   80|       |    /// - `read`: Number of consumed input units.
   81|       |    /// - `written`: Number of produced output units.
   82|       |    ///
   83|       |    /// # Returns
   84|       |    ///
   85|       |    /// Returns a progress value with [`TranscodeStatus::NeedInput`].
   86|       |    #[inline]
   87|  1.19k|    pub const fn need_input(required: NonZeroUsize, read: usize, written: usize) -> Self {
   88|  1.19k|        Self::new(TranscodeStatus::need_input(required), read, written)
   89|  1.19k|    }
   90|       |
   91|       |    /// Creates progress that stopped because more output capacity is needed.
   92|       |    ///
   93|       |    /// # Parameters
   94|       |    ///
   95|       |    /// - `required`: Total output units required from the current output
   96|       |    ///   position.
   97|       |    /// - `read`: Number of consumed input units.
   98|       |    /// - `written`: Number of produced output units.
   99|       |    ///
  100|       |    /// # Returns
  101|       |    ///
  102|       |    /// Returns a progress value with [`TranscodeStatus::NeedOutput`].
  103|       |    #[inline]
  104|  9.22k|    pub const fn need_output(required: NonZeroUsize, read: usize, written: usize) -> Self {
  105|  9.22k|        Self::new(TranscodeStatus::need_output(required), read, written)
  106|  9.22k|    }
  107|       |
  108|       |    /// Returns the status that stopped conversion.
  109|       |    ///
  110|       |    /// # Returns
  111|       |    ///
  112|       |    /// Returns the stored [`TranscodeStatus`].
  113|       |    #[inline]
  114|  21.5k|    pub const fn status(self) -> TranscodeStatus {
  115|  21.5k|        self.status
  116|  21.5k|    }
  117|       |
  118|       |    /// Returns whether conversion consumed all currently supplied input.
  119|       |    ///
  120|       |    /// # Returns
  121|       |    ///
  122|       |    /// Returns `true` when the stored status is
  123|       |    /// [`TranscodeStatus::Complete`].
  124|       |    #[inline]
  125|       |    #[must_use]
  126|  1.03k|    pub const fn is_complete(self) -> bool {
  127|  1.03k|        matches!(self.status, TranscodeStatus::Complete)
                      ^4
  128|  1.03k|    }
  129|       |
  130|       |    /// Returns whether conversion stopped because more input is needed.
  131|       |    ///
  132|       |    /// # Returns
  133|       |    ///
  134|       |    /// Returns `true` when the stored status is
  135|       |    /// [`TranscodeStatus::NeedInput`].
  136|       |    #[inline]
  137|       |    #[must_use]
  138|      3|    pub const fn is_need_input(self) -> bool {
  139|      3|        matches!(self.status, TranscodeStatus::NeedInput { .. })
                      ^2
  140|      3|    }
  141|       |
  142|       |    /// Returns whether conversion stopped because more output capacity is
  143|       |    /// needed.
  144|       |    ///
  145|       |    /// # Returns
  146|       |    ///
  147|       |    /// Returns `true` when the stored status is
  148|       |    /// [`TranscodeStatus::NeedOutput`].
  149|       |    #[inline]
  150|       |    #[must_use]
  151|      5|    pub const fn is_need_output(self) -> bool {
  152|      5|        matches!(self.status, TranscodeStatus::NeedOutput { .. })
                      ^2
  153|      5|    }
  154|       |
  155|       |    /// Returns the number of input units consumed by the call.
  156|       |    ///
  157|       |    /// # Returns
  158|       |    ///
  159|       |    /// Returns a count relative to the input index passed to the conversion
  160|       |    /// call.
  161|       |    #[inline]
  162|       |    #[must_use]
  163|  12.7k|    pub const fn read(self) -> usize {
  164|  12.7k|        self.read
  165|  12.7k|    }
  166|       |
  167|       |    /// Returns the number of output units written by the call.
  168|       |    ///
  169|       |    /// # Returns
  170|       |    ///
  171|       |    /// Returns a count relative to the output index passed to the conversion
  172|       |    /// call.
  173|       |    #[inline]
  174|       |    #[must_use]
  175|  13.2k|    pub const fn written(self) -> usize {
  176|  13.2k|        self.written
  177|  13.2k|    }
  178|       |
  179|       |    /// Validates this progress against the call bounds supplied to a
  180|       |    /// transcoder.
  181|       |    ///
  182|       |    /// Buffered drivers should call this before using [`Self::read`] or
  183|       |    /// [`Self::written`] to advance unchecked input or output cursors. The
  184|       |    /// method checks relative counters, unsatisfied `NeedInput` / `NeedOutput`
  185|       |    /// requirements, and the `Complete` invariant that all visible input was
  186|       |    /// consumed.
  187|       |    ///
  188|       |    /// This is a contract checker, not a semantic recovery policy. It performs
  189|       |    /// the same validation in every build profile. Drivers that advance unsafe
  190|       |    /// cursors may convert failures into their own error type, while
  191|       |    /// convenience helpers may assert that validation succeeds. Custom
  192|       |    /// [`crate::Transcoder`] implementations must therefore return progress
  193|       |    /// that satisfies the documented contract in all builds.
  194|       |    ///
  195|       |    /// # Parameters
  196|       |    ///
  197|       |    /// - `input_index`: Input index originally passed to the transcoder.
  198|       |    /// - `available_input`: Number of input units visible from `input_index`.
  199|       |    /// - `output_index`: Output index originally passed to the transcoder.
  200|       |    /// - `available_output`: Number of output slots visible from
  201|       |    ///   `output_index`.
  202|       |    ///
  203|       |    /// # Returns
  204|       |    ///
  205|       |    /// Returns `Ok(())` when progress is internally consistent with the
  206|       |    /// supplied call bounds.
  207|       |    ///
  208|       |    /// # Errors
  209|       |    ///
  210|       |    /// Returns [`TranscodeContractError`] when a custom transcoder reports
  211|       |    /// counters or missing-capacity requirements that do not match the buffers
  212|       |    /// supplied by the caller.
  213|  9.88k|    pub fn validate(
  214|  9.88k|        &self,
  215|  9.88k|        input_index: usize,
  216|  9.88k|        available_input: usize,
  217|  9.88k|        output_index: usize,
  218|  9.88k|        available_output: usize,
  219|  9.88k|    ) -> Result<(), TranscodeContractError> {
  220|  9.88k|        if self.read > available_input {
  221|      7|            return Err(TranscodeContractError::OverRead {
  222|      7|                read: self.read,
  223|      7|                available: available_input,
  224|      7|            });
  225|  9.87k|        }
  226|  9.87k|        if self.written > available_output {
  227|      4|            return Err(TranscodeContractError::OverWritten {
  228|      4|                written: self.written,
  229|      4|                available: available_output,
  230|      4|            });
  231|  9.87k|        }
  232|       |
  233|  9.87k|        input_index
  234|  9.87k|            .checked_add(self.read)
  235|  9.87k|            .ok_or(TranscodeContractError::ProgressIndexOverflow {
  236|  9.87k|                index: input_index,
  237|  9.87k|                advanced: self.read,
  238|  9.87k|            })?;
                            ^1
  239|  9.87k|        output_index
  240|  9.87k|            .checked_add(self.written)
  241|  9.87k|            .ok_or(TranscodeContractError::ProgressIndexOverflow {
  242|  9.87k|                index: output_index,
  243|  9.87k|                advanced: self.written,
  244|  9.87k|            })?;
                            ^1
  245|  9.87k|        match self.status {
  246|       |            TranscodeStatus::Complete => {
  247|  1.61k|                if self.read != available_input {
  248|      4|                    return Err(TranscodeContractError::CompleteWithRemainingInput {
  249|      4|                        read: self.read,
  250|      4|                        available: available_input,
  251|      4|                    });
  252|  1.61k|                }
  253|  1.61k|                Ok(())
  254|       |            }
  255|     34|            TranscodeStatus::NeedInput { required } => {
  256|     34|                let available = available_input - self.read;
  257|     34|                if required.get() <= available {
  258|      2|                    return Err(TranscodeContractError::SatisfiedNeed {
  259|      2|                        required: required.get(),
  260|      2|                        available,
  261|      2|                    });
  262|     32|                }
  263|     32|                Ok(())
  264|       |            }
  265|  8.22k|            TranscodeStatus::NeedOutput { required } => {
  266|  8.22k|                let available = available_output - self.written;
  267|  8.22k|                if required.get() <= available {
  268|      4|                    return Err(TranscodeContractError::SatisfiedNeed {
  269|      4|                        required: required.get(),
  270|      4|                        available,
  271|      4|                    });
  272|  8.22k|                }
  273|  8.22k|                Ok(())
  274|       |            }
  275|       |        }
  276|  9.88k|    }
  277|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/transcode_status.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |use core::num::NonZeroUsize;
    9|       |
   10|       |/// Reports why a [`crate::Transcoder`] stopped converting input.
   11|       |///
   12|       |/// # Examples
   13|       |///
   14|       |/// ```
   15|       |/// use core::num::NonZeroUsize;
   16|       |/// use qubit_codec::TranscodeStatus;
   17|       |///
   18|       |/// let status = TranscodeStatus::need_output(NonZeroUsize::new(2).unwrap());
   19|       |/// assert!(matches!(status, TranscodeStatus::NeedOutput { required } if required.get() == 2));
   20|       |/// ```
   21|       |#[must_use]
   22|       |#[derive(Clone, Copy, Debug, Eq, PartialEq)]
   23|       |pub enum TranscodeStatus {
   24|       |    /// All currently supplied input was consumed.
   25|       |    ///
   26|       |    /// A `Complete` status means the current [`crate::Transcoder::transcode`]
   27|       |    /// call consumed every input unit visible from the supplied `input_index`.
   28|       |    /// Implementations must report [`NeedInput`](Self::NeedInput) for an
   29|       |    /// incomplete input tail and [`NeedOutput`](Self::NeedOutput) when output
   30|       |    /// capacity prevents further input consumption.
   31|       |    Complete,
   32|       |
   33|       |    /// More input is needed to complete the next output value.
   34|       |    ///
   35|       |    /// The transcoder does not consume incomplete input tails. If the call
   36|       |    /// started at `start` and returned `progress`, preserve
   37|       |    /// `input[start + progress.read()..]` before refilling the input buffer.
   38|       |    /// If the caller has already advanced its input cursor by
   39|       |    /// `progress.read()`, it can slice from that updated cursor directly.
   40|       |    /// Apply the EOF policy when the upstream source is closed. Calling
   41|       |    /// [`crate::Transcoder::finish`] does not pass this tail back to the
   42|       |    /// transcoder.
   43|       |    ///
   44|       |    /// - `required`: Current minimum total input units required before retrying
   45|       |    ///   from the current input position. A later retry may raise this lower
   46|       |    ///   bound.
   47|       |    NeedInput {
   48|       |        /// Current minimum total input units required before retrying from the
   49|       |        /// current input position. A later retry may raise this lower bound.
   50|       |        required: NonZeroUsize,
   51|       |    },
   52|       |
   53|       |    /// More output capacity is needed before conversion can continue.
   54|       |    ///
   55|       |    /// - `required`: Total output units required from the current output
   56|       |    ///   position.
   57|       |    NeedOutput {
   58|       |        /// Total output units required from the current output position.
   59|       |        required: NonZeroUsize,
   60|       |    },
   61|       |}
   62|       |
   63|       |impl TranscodeStatus {
   64|       |    /// Creates a status that requests more input.
   65|       |    ///
   66|       |    /// # Parameters
   67|       |    ///
   68|       |    /// - `required`: Current minimum total input units required before retrying
   69|       |    ///   from the current input position. A later retry may raise this lower
   70|       |    ///   bound.
   71|       |    ///
   72|       |    /// # Returns
   73|       |    ///
   74|       |    /// Returns a [`TranscodeStatus::NeedInput`] value.
   75|       |    #[inline]
   76|  1.19k|    pub const fn need_input(required: NonZeroUsize) -> Self {
   77|  1.19k|        Self::NeedInput { required }
   78|  1.19k|    }
   79|       |
   80|       |    /// Creates a status that requests more output capacity.
   81|       |    ///
   82|       |    /// # Parameters
   83|       |    ///
   84|       |    /// - `required`: Total output units required from the current output
   85|       |    ///   position.
   86|       |    ///
   87|       |    /// # Returns
   88|       |    ///
   89|       |    /// Returns a [`TranscodeStatus::NeedOutput`] value.
   90|       |    #[inline]
   91|  9.22k|    pub const fn need_output(required: NonZeroUsize) -> Self {
   92|  9.22k|        Self::NeedOutput { required }
   93|  9.22k|    }
   94|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/transcode/transcoder.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |use super::capacity_error::CapacityError;
    9|       |use super::transcode_failure::TranscodeFailure;
   10|       |use super::transcode_progress::TranscodeProgress;
   11|       |use super::transcode_status::TranscodeStatus;
   12|       |
   13|       |/// Validates one-shot transcode progress and returns completed output length.
   14|       |///
   15|       |/// Keeping progress classification independent of the concrete transcoder
   16|       |/// ensures every implementation receives identical trailing-input and
   17|       |/// streaming-stop handling.
   18|       |///
   19|       |/// # Parameters
   20|       |///
   21|       |/// - `progress`: Progress returned by the streaming transcode phase.
   22|       |/// - `input_len`: Total complete-input length supplied to the one-shot call.
   23|       |/// - `output_index`: Absolute output index used for the transcode phase.
   24|       |/// - `output_len`: Total output slice length.
   25|       |///
   26|       |/// # Returns
   27|       |///
   28|       |/// Returns the number of output units written when progress is complete.
   29|       |///
   30|       |/// # Errors
   31|       |///
   32|       |/// Returns a trailing-input, incomplete-input, or insufficient-output failure
   33|       |/// when progress reports a non-complete one-shot result.
   34|       |///
   35|       |/// # Panics
   36|       |///
   37|       |/// Panics when the transcoder reports progress inconsistent with the supplied
   38|       |/// input and output bounds.
   39|    548|fn complete_progress_written(
   40|    548|    progress: TranscodeProgress,
   41|    548|    input_len: usize,
   42|    548|    output_index: usize,
   43|    548|    output_len: usize,
   44|    548|) -> Result<usize, TranscodeFailure> {
   45|    548|    assert!(
   46|    548|        progress
   47|    548|            .validate(0, input_len, output_index, output_len.saturating_sub(output_index),)
   48|    548|            .is_ok(),
   49|       |        "Transcoder::transcode returned invalid progress",
   50|       |    );
   51|    548|    match progress.status() {
   52|    546|        TranscodeStatus::Complete => Ok(progress.written()),
   53|      1|        TranscodeStatus::NeedOutput { required } => Err(TranscodeFailure::insufficient_output(
   54|      1|            output_index + progress.written(),
   55|      1|            required.get(),
   56|      1|            output_len - output_index - progress.written(),
   57|      1|        )),
   58|      1|        TranscodeStatus::NeedInput { required } => Err(TranscodeFailure::incomplete_input(
   59|      1|            progress.read(),
   60|      1|            required.get(),
   61|      1|            input_len - progress.read(),
   62|      1|        )),
   63|       |    }
   64|    548|}
   65|       |
   66|       |/// Adds two independent output-capacity bounds.
   67|       |///
   68|       |/// # Parameters
   69|       |///
   70|       |/// - `first`: First output-capacity bound.
   71|       |/// - `second`: Second output-capacity bound.
   72|       |///
   73|       |/// # Returns
   74|       |///
   75|       |/// Returns the sum of both bounds.
   76|       |///
   77|       |/// # Errors
   78|       |///
   79|       |/// Returns [`CapacityError::OutputLengthOverflow`] when the sum overflows.
   80|       |#[inline]
   81|  1.13k|fn add_output_bounds(first: usize, second: usize) -> Result<usize, CapacityError> {
   82|  1.13k|    first.checked_add(second).ok_or(CapacityError::OutputLengthOverflow)
   83|  1.13k|}
   84|       |
   85|       |/// Adds the reset, transcode, and finish output-capacity bounds.
   86|       |///
   87|       |/// # Parameters
   88|       |///
   89|       |/// - `reset`: Reset-phase output-capacity bound.
   90|       |/// - `transcode`: Streaming transcode-phase output-capacity bound.
   91|       |/// - `finish`: Finish-phase output-capacity bound.
   92|       |///
   93|       |/// # Returns
   94|       |///
   95|       |/// Returns the complete lifecycle output-capacity bound.
   96|       |///
   97|       |/// # Errors
   98|       |///
   99|       |/// Returns [`CapacityError::OutputLengthOverflow`] when either addition
  100|       |/// overflows.
  101|       |#[inline]
  102|    570|fn sum_output_bounds(reset: usize, transcode: usize, finish: usize) -> Result<usize, CapacityError> {
  103|    570|    let before_finish = add_output_bounds(reset, transcode)?;
                      ^569                                               ^1
  104|    569|    add_output_bounds(before_finish, finish)
  105|    570|}
  106|       |
  107|       |/// Converts one logical stream of input units into one logical stream of output
  108|       |/// units.
  109|       |///
  110|       |/// `transcode` is the main streaming API. It transforms a provided input
  111|       |/// segment and writes as much output as available buffer space allows. A
  112|       |/// successful `transcode` call must stop with exactly one of three meanings:
  113|       |/// all visible input was consumed (`Complete`), an incomplete tail remains
  114|       |/// caller-owned (`NeedInput`), or output capacity prevented further progress
  115|       |/// (`NeedOutput`).
  116|       |///
  117|       |/// A transcoder instance has a simple lifecycle:
  118|       |///
  119|       |/// 1. A newly created instance is uninitialized; call [`Transcoder::reset`]
  120|       |///    before the first stream. A reset instance is ready for a new logical
  121|       |///    stream.
  122|       |/// 2. Call [`Transcoder::transcode`] zero or more times while input is
  123|       |///    available.
  124|       |/// 3. Preserve any tail reported by [`crate::TranscodeStatus::NeedInput`] in
  125|       |///    the caller-owned input buffer.
  126|       |/// 4. Call [`Transcoder::finish`] after the caller knows no more input remains
  127|       |///    and has handled any incomplete tail. Size this final output with
  128|       |///    [`Transcoder::max_finish_output_len`].
  129|       |/// 5. After [`Transcoder::finish`] succeeds, call [`Transcoder::reset`] with a
  130|       |///    buffer sized by [`Transcoder::max_reset_output_len`] before starting
  131|       |///    another logical stream with the same instance.
  132|       |///
  133|       |/// The method is suitable for:
  134|       |/// - pull-style consumers that call conversion repeatedly as buffers arrive;
  135|       |/// - bounded output sinks that use `NeedOutput` progress during `transcode`;
  136|       |/// - stateless and stateful codecs that all return progress-oriented stopping
  137|       |///   reasons.
  138|       |///
  139|       |/// `finish` finalizes retained state only; it does not receive source input and
  140|       |/// does not reinterpret a tail previously reported by `NeedInput`. For
  141|       |/// codec-backed decoders, streaming uses
  142|       |/// [`Codec::decode`](crate::Codec::decode) and an explicit EOF call may use
  143|       |/// [`Codec::decode_eof`](crate::Codec::decode_eof)
  144|       |/// through [`Transcoder::transcode_eof`]. If a format needs EOF-aware
  145|       |/// maximal-munch parsing or must delay whether a prefix is complete until the
  146|       |/// next chunk or EOF, implement that policy in the codec or override
  147|       |/// `transcode_eof`.
  148|       |///
  149|       |/// `Transcoder` is intentionally independent from any charset
  150|       |/// semantics:
  151|       |///
  152|       |/// - Use `Transcoder` directly for custom, policy-free unit transforms.
  153|       |/// - Use `Transcoder` when you want to own malformed/unmappable decisions at
  154|       |///   the call site.
  155|       |///
  156|       |/// # Examples
  157|       |///
  158|       |/// Streaming byte-to-word decoding:
  159|       |///
  160|       |/// ```rust
  161|       |/// use core::convert::Infallible;
  162|       |/// use core::num::NonZeroUsize;
  163|       |///
  164|       |/// use qubit_codec::CapacityError;
  165|       |/// use qubit_codec::TranscodeDecodeError;
  166|       |/// use qubit_codec::TranscodeFailure;
  167|       |/// use qubit_codec::TranscodeProgress;
  168|       |/// use qubit_codec::TranscodeStatus;
  169|       |/// use qubit_codec::Transcoder;
  170|       |///
  171|       |/// #[derive(Default)]
  172|       |/// struct U16BeBytesDecoder;
  173|       |///
  174|       |/// impl Transcoder for U16BeBytesDecoder {
  175|       |///     type Input = u8;
  176|       |///     type Output = u16;
  177|       |///     type Error = TranscodeDecodeError<Infallible>;
  178|       |///
  179|       |///     fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
  180|       |///         Ok(input_len / 2)
  181|       |///     }
  182|       |///
  183|       |///     fn reset(
  184|       |///         &mut self,
  185|       |///         output: &mut [u16],
  186|       |///         output_index: usize,
  187|       |///     ) -> Result<usize, Self::Error> {
  188|       |///         TranscodeFailure::ensure_output_index(output.len(), output_index)?;
  189|       |///         Ok(0)
  190|       |///     }
  191|       |///
  192|       |///     fn transcode(
  193|       |///         &mut self,
  194|       |///         input: &[u8],
  195|       |///         input_index: usize,
  196|       |///         output: &mut [u16],
  197|       |///         output_index: usize,
  198|       |///     ) -> Result<TranscodeProgress, Self::Error> {
  199|       |///         TranscodeFailure::ensure_transcode_indices(
  200|       |///             input.len(),
  201|       |///             input_index,
  202|       |///             output.len(),
  203|       |///             output_index,
  204|       |///         )?;
  205|       |///
  206|       |///         let mut read = 0;
  207|       |///         let mut written = 0;
  208|       |///         while input_index + read + 1 < input.len() {
  209|       |///             if output_index + written == output.len() {
  210|       |///                 let status = TranscodeStatus::NeedOutput {
  211|       |///                     required: NonZeroUsize::MIN,
  212|       |///                 };
  213|       |///                 return Ok(TranscodeProgress::new(status, read, written));
  214|       |///             }
  215|       |///             let high = input[input_index + read] as u16;
  216|       |///             let low = input[input_index + read + 1] as u16;
  217|       |///             output[output_index + written] = (high << 8) | low;
  218|       |///             read += 2;
  219|       |///             written += 1;
  220|       |///         }
  221|       |///         if input_index + read == input.len() {
  222|       |///             Ok(TranscodeProgress::complete(read, written))
  223|       |///         } else {
  224|       |///             let status = TranscodeStatus::NeedInput {
  225|       |///                 required: NonZeroUsize::new(2).expect("two is non-zero"),
  226|       |///             };
  227|       |///             Ok(TranscodeProgress::new(status, read, written))
  228|       |///         }
  229|       |///     }
  230|       |///
  231|       |///     fn finish(
  232|       |///         &mut self,
  233|       |///         output: &mut [u16],
  234|       |///         output_index: usize,
  235|       |///     ) -> Result<usize, Self::Error> {
  236|       |///         TranscodeFailure::ensure_output_index(output.len(), output_index)?;
  237|       |///         Ok(0)
  238|       |///     }
  239|       |/// }
  240|       |///
  241|       |/// let mut transcoder = U16BeBytesDecoder::default();
  242|       |/// let mut reset_output = [];
  243|       |/// transcoder
  244|       |///     .reset(&mut reset_output, 0)
  245|       |///     .expect("stateless decoder reset cannot fail");
  246|       |/// let mut output = [0_u16; 1];
  247|       |/// let progress = transcoder
  248|       |///     .transcode(&[0x12, 0x34, 0xab, 0xcd], 0, &mut output, 0)
  249|       |///     .expect("decoding cannot fail");
  250|       |/// assert_eq!(TranscodeStatus::NeedOutput {
  251|       |///     required: NonZeroUsize::MIN,
  252|       |/// }, progress.status());
  253|       |/// assert_eq!(2, progress.read());
  254|       |/// assert_eq!(1, progress.written());
  255|       |/// assert_eq!([0x1234], output);
  256|       |///
  257|       |/// let mut output = [0_u16; 2];
  258|       |/// let progress = transcoder
  259|       |///     .transcode(&[0x12, 0x34, 0xab], 0, &mut output, 0)
  260|       |///     .expect("decoding cannot fail");
  261|       |/// assert_eq!(TranscodeStatus::NeedInput {
  262|       |///     required: NonZeroUsize::new(2).expect("two is non-zero"),
  263|       |/// }, progress.status());
  264|       |/// assert_eq!(2, progress.read());
  265|       |/// assert_eq!(1, progress.written());
  266|       |/// assert_eq!([0x1234, 0], output);
  267|       |///
  268|       |/// assert!(matches!(
  269|       |///     transcoder.transcode(&[0x12], 2, &mut output, 0),
  270|       |///     Err(TranscodeDecodeError::Failure(
  271|       |///         TranscodeFailure::InvalidInputIndex { .. }
  272|       |///     )),
  273|       |/// ));
  274|       |/// assert!(matches!(
  275|       |///     transcoder.transcode(&[0x12], 0, &mut output, 3),
  276|       |///     Err(TranscodeDecodeError::Failure(
  277|       |///         TranscodeFailure::InvalidOutputIndex { .. }
  278|       |///     )),
  279|       |/// ));
  280|       |/// ```
  281|       |///
  282|       |/// The trait is intentionally independent from charset concepts. Implementors
  283|       |/// use `input_index` and `output_index` as absolute positions in the supplied
  284|       |/// slices. Returned progress counters are relative counts from those positions.
  285|       |/// For raw codecs this gives a compact API; higher-level workflows can wrap
  286|       |/// this trait with their own semantic policies.
  287|       |pub trait Transcoder {
  288|       |    /// Input unit type accepted by this transcoder.
  289|       |    type Input;
  290|       |
  291|       |    /// Output unit type produced by this transcoder.
  292|       |    type Output;
  293|       |
  294|       |    /// Complete error type produced by this transcoder.
  295|       |    type Error: From<TranscodeFailure>;
  296|       |
  297|       |    /// Returns an upper bound for output units emitted when resetting stream
  298|       |    /// state.
  299|       |    ///
  300|       |    /// Stateful encoders may need a stream-start sequence, such as a byte
  301|       |    /// order mark, before the first encoded value. Callers use this bound to
  302|       |    /// size the output buffer passed to [`Transcoder::reset`].
  303|       |    ///
  304|       |    /// The bound may depend on immutable transcoder configuration, but it must
  305|       |    /// cover reset output from every reachable transient stream state. It must
  306|       |    /// not shrink merely because the current stream has already been reset or
  307|       |    /// finished.
  308|       |    ///
  309|       |    /// # Returns
  310|       |    ///
  311|       |    /// Returns `Ok(bound)` when the upper bound can be represented as `usize`.
  312|       |    /// Stateless transcoders default to `Ok(0)`.
  313|       |    ///
  314|       |    /// # Errors
  315|       |    ///
  316|       |    /// Implementations return [`CapacityError::OutputLengthOverflow`] when
  317|       |    /// capacity arithmetic overflows. The default never fails.
  318|       |    #[inline]
  319|     25|    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
  320|     25|        Ok(0)
  321|     25|    }
  322|       |
  323|       |    /// Returns an upper bound for the output capacity needed to transcode
  324|       |    /// `input_len` units during the streaming phase.
  325|       |    ///
  326|       |    /// This bound excludes stream-start output emitted by
  327|       |    /// [`Transcoder::reset`] and final output emitted by
  328|       |    /// [`Transcoder::finish`]. Callers that need a complete one-shot stream
  329|       |    /// bound should use [`Transcoder::max_total_output_len`].
  330|       |    ///
  331|       |    /// The bound covers both [`Transcoder::transcode`] and
  332|       |    /// [`Transcoder::transcode_eof`], including any conservative capacity
  333|       |    /// preflight required by those operations. It must provide enough writable
  334|       |    /// capacity for the implementation to complete its preflight, even when
  335|       |    /// the actual output would use fewer units.
  336|       |    ///
  337|       |    /// The bound may depend on immutable transcoder configuration and
  338|       |    /// `input_len`, but it must cover the streaming output reachable from every
  339|       |    /// transient stream state. In particular, it must include the maximum
  340|       |    /// retained output that may be emitted before or alongside output derived
  341|       |    /// from the supplied input, even when no output is currently retained.
  342|       |    ///
  343|       |    /// # Parameters
  344|       |    ///
  345|       |    /// - `input_len`: Number of input units the caller plans to transcode.
  346|       |    ///
  347|       |    /// # Returns
  348|       |    ///
  349|       |    /// Returns `Ok(bound)` when the upper bound can be represented as `usize`.
  350|       |    ///
  351|       |    /// # Errors
  352|       |    ///
  353|       |    /// Returns [`CapacityError::OutputLengthOverflow`] when capacity arithmetic
  354|       |    /// overflows.
  355|       |    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError>;
  356|       |
  357|       |    /// Returns an upper bound for a complete `reset -> transcode_eof -> finish`
  358|       |    /// stream.
  359|       |    ///
  360|       |    /// This is the one-shot bound for [`Transcoder::transcode_complete_into`]
  361|       |    /// and a convenience sum of [`Transcoder::max_reset_output_len`],
  362|       |    /// [`Transcoder::max_transcode_output_len`], and
  363|       |    /// [`Transcoder::max_finish_output_len`]. Each component is independent of
  364|       |    /// transient stream state, so the sum is valid before a full one-shot
  365|       |    /// stream regardless of how the previous logical stream ended.
  366|       |    ///
  367|       |    /// # Parameters
  368|       |    ///
  369|       |    /// - `input_len`: Number of input units in the complete stream.
  370|       |    ///
  371|       |    /// # Returns
  372|       |    ///
  373|       |    /// Returns `Ok(bound)` when the full-stream upper bound can be represented
  374|       |    /// as `usize`.
  375|       |    ///
  376|       |    /// # Errors
  377|       |    ///
  378|       |    /// Returns [`CapacityError::OutputLengthOverflow`] when a component bound
  379|       |    /// or the sum of lifecycle bounds overflows.
  380|       |    #[inline]
  381|     14|    fn max_total_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
  382|     14|        let reset = self.max_reset_output_len()?;
                          ^13                                ^1
  383|     13|        let transcode = self.max_transcode_output_len(input_len)?;
                          ^12                                                 ^1
  384|     12|        let finish = self.max_finish_output_len()?;
                          ^11                                  ^1
  385|     11|        sum_output_bounds(reset, transcode, finish)
  386|     14|    }
  387|       |
  388|       |    /// Returns an upper bound for output units produced by stream finalization.
  389|       |    ///
  390|       |    /// The bound may depend on immutable transcoder configuration, but it must
  391|       |    /// cover final output from every reachable transient stream state. For
  392|       |    /// example, an encoder that can emit one checksum byte must return at least
  393|       |    /// `1` before input, while accumulating the checksum, and after finishing a
  394|       |    /// previous stream. It must not shrink merely because no finish output is
  395|       |    /// currently pending.
  396|       |    ///
  397|       |    /// # Returns
  398|       |    ///
  399|       |    /// Returns `Ok(bound)` when the upper bound can be represented as `usize`.
  400|       |    /// Stateless transcoders default to `Ok(0)`.
  401|       |    ///
  402|       |    /// # Errors
  403|       |    ///
  404|       |    /// Implementations return [`CapacityError::OutputLengthOverflow`] when
  405|       |    /// capacity arithmetic overflows. The default never fails.
  406|       |    #[inline]
  407|     12|    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
  408|     12|        Ok(0)
  409|     12|    }
  410|       |
  411|       |    /// Resets stream state and emits stream-start output into `output`.
  412|       |    ///
  413|       |    /// This starts a new logical stream while keeping configuration such as
  414|       |    /// byte order, charset policy, replacement values, and cryptographic keys.
  415|       |    /// Pending input, pending output, and completed-stream state must be
  416|       |    /// discarded by stateful implementations. The caller must provide enough
  417|       |    /// output capacity for [`Transcoder::max_reset_output_len`].
  418|       |    ///
  419|       |    /// # Parameters
  420|       |    ///
  421|       |    /// - `output`: Complete output unit slice visible to the transcoder.
  422|       |    /// - `output_index`: Absolute output unit index where writing starts.
  423|       |    ///
  424|       |    /// # Returns
  425|       |    ///
  426|       |    /// Returns the number of units written while resetting stream state.
  427|       |    /// Stateless transcoders return `0`. Implementations must not return a
  428|       |    /// count greater than [`Transcoder::max_reset_output_len`] or the output
  429|       |    /// units available from `output_index`.
  430|       |    ///
  431|       |    /// # Errors
  432|       |    ///
  433|       |    /// Returns contract errors (`invalid_output_index`, `insufficient_output`)
  434|       |    /// when capacity checks fail, or policy errors when reset itself fails.
  435|       |    fn reset(&mut self, output: &mut [Self::Output], output_index: usize) -> Result<usize, Self::Error>;
  436|       |
  437|       |    /// Converts available input units into output units.
  438|       |    ///
  439|       |    /// This method processes an input segment without closing the logical input
  440|       |    /// stream. When the current segment ends in a partial value, the transcoder
  441|       |    /// reports [`crate::TranscodeStatus::NeedInput`] without consuming that
  442|       |    /// tail. The caller owns input-buffer refill and EOF incomplete-tail
  443|       |    /// policy.
  444|       |    ///
  445|       |    /// Returning [`crate::TranscodeStatus::Complete`] means all input visible
  446|       |    /// from `input_index` was consumed. Implementations must not return
  447|       |    /// `Complete` after consuming only a prefix of the supplied input; they
  448|       |    /// must instead continue processing, report `NeedInput` for an
  449|       |    /// incomplete tail, or report `NeedOutput` if output capacity prevents
  450|       |    /// progress. This invariant is checked by
  451|       |    /// [`crate::TranscodeProgress::validate`].
  452|       |    ///
  453|       |    /// # Parameters
  454|       |    ///
  455|       |    /// - `input`: Complete input unit slice visible to the transcoder.
  456|       |    /// - `input_index`: Absolute input unit index where conversion starts.
  457|       |    /// - `output`: Complete output unit slice visible to the transcoder.
  458|       |    /// - `output_index`: Absolute output unit index where writing starts.
  459|       |    ///
  460|       |    /// # Returns
  461|       |    ///
  462|       |    /// Returns progress describing how many units were consumed and produced
  463|       |    /// and why conversion stopped. Implementations must keep the returned
  464|       |    /// counters and status fields consistent with the supplied input and
  465|       |    /// output ranges. For `Complete`, `read` must equal `input.len() -
  466|       |    /// input_index`. The default one-shot helper rejects incomplete `Complete`
  467|       |    /// progress and asserts the remaining progress contract in all builds;
  468|       |    /// streaming I/O drivers may convert validation failures before advancing
  469|       |    /// unsafe cursors.
  470|       |    ///
  471|       |    /// # Errors
  472|       |    ///
  473|       |    /// Returns framework failures for invalid indices, insufficient output,
  474|       |    /// incomplete input, and capacity overflow. Returns domain errors for
  475|       |    /// semantic conversion failures that the transcoder's policy does not
  476|       |    /// absorb. An error does not provide general transactional rollback or
  477|       |    /// complete progress: the transcoder may already have advanced. Callers
  478|       |    /// must not replay the original input range unless the concrete
  479|       |    /// transcoder provides an explicit recovery protocol. Reset may begin a
  480|       |    /// new logical stream but cannot undo output already delivered.
  481|       |    fn transcode(
  482|       |        &mut self,
  483|       |        input: &[Self::Input],
  484|       |        input_index: usize,
  485|       |        output: &mut [Self::Output],
  486|       |        output_index: usize,
  487|       |    ) -> Result<TranscodeProgress, Self::Error>;
  488|       |
  489|       |    /// Converts source units after the caller has established end of input.
  490|       |    ///
  491|       |    /// The default preserves ordinary streaming behavior. Implementations that
  492|       |    /// can resolve a trailing prefix using EOF-aware format rules should
  493|       |    /// override it. If the default still reports
  494|       |    /// [`TranscodeStatus::NeedInput`], this method returns
  495|       |    /// [`TranscodeFailure::IncompleteInput`] instead. The default validates
  496|       |    /// its input and output indices before delegating, then validates returned
  497|       |    /// progress before calculating the incomplete-input context. A broken
  498|       |    /// implementation therefore returns [`TranscodeFailure::InvalidProgress`]
  499|       |    /// rather than causing arithmetic overflow or advancing invalid cursors.
  500|       |    ///
  501|       |    /// # Parameters
  502|       |    ///
  503|       |    /// - `input`: Complete input slice, including the final source segment.
  504|       |    /// - `input_index`: Absolute position where final decoding starts.
  505|       |    /// - `output`: Destination output slice.
  506|       |    /// - `output_index`: Absolute output position where writing starts.
  507|       |    ///
  508|       |    /// # Returns
  509|       |    ///
  510|       |    /// Validated complete or output-limited progress. EOF results should be
  511|       |    /// `Complete`, `NeedOutput`, or an error; the default never returns a
  512|       |    /// successful `NeedInput` status at EOF.
  513|       |    ///
  514|       |    /// # Errors
  515|       |    ///
  516|       |    /// Returns invalid-index errors before calling `transcode`, forwards its
  517|       |    /// errors, and rejects invalid progress or an incomplete final input tail.
  518|       |    /// It has the same non-transactional error boundary as [`Self::transcode`]:
  519|       |    /// the transcoder may have advanced, and callers must not replay the
  520|       |    /// original input range without an explicit recovery protocol. Reset only
  521|       |    /// begins a new logical stream; it cannot undo output already delivered.
  522|       |    #[inline]
  523|     19|    fn transcode_eof(
  524|     19|        &mut self,
  525|     19|        input: &[Self::Input],
  526|     19|        input_index: usize,
  527|     19|        output: &mut [Self::Output],
  528|     19|        output_index: usize,
  529|     19|    ) -> Result<TranscodeProgress, Self::Error> {
  530|     19|        TranscodeFailure::ensure_transcode_indices(input.len(), input_index, output.len(), output_index)?;
                                                                                                                      ^0
  531|     19|        let available_input = input.len() - input_index;
  532|     19|        let available_output = output.len() - output_index;
  533|     19|        let progress = self.transcode(input, input_index, output, output_index)?;
                          ^18                                                                ^1
  534|     18|        progress
  535|     18|            .validate(input_index, available_input, output_index, available_output)
  536|     18|            .map_err(TranscodeFailure::invalid_progress)?;
                                                                      ^3
  537|     15|        match progress.status() {
  538|      6|            TranscodeStatus::NeedInput { required } => Err(TranscodeFailure::incomplete_input(
  539|      6|                input_index + progress.read(),
  540|      6|                required.get(),
  541|      6|                available_input - progress.read(),
  542|      6|            )
  543|      6|            .into()),
  544|      9|            _ => Ok(progress),
  545|       |        }
  546|     19|    }
  547|       |
  548|       |    /// Finishes internally retained output after all input has been supplied.
  549|       |    ///
  550|       |    /// `transcode` handles ordinary input consumption. `finish` is called once
  551|       |    /// after the caller knows no more input remains and has handled any
  552|       |    /// incomplete input tail reported by `transcode`. It emits final output
  553|       |    /// derived from internal state, such as reset bytes, checksums, digests, or
  554|       |    /// trailers. The caller must provide enough output capacity for
  555|       |    /// [`Transcoder::max_finish_output_len`].
  556|       |    ///
  557|       |    /// After `finish` succeeds, the logical stream is closed. Portable callers
  558|       |    /// should call [`Transcoder::reset`] with a buffer sized by
  559|       |    /// [`Transcoder::max_reset_output_len`] before passing input for another
  560|       |    /// logical stream to the same instance.
  561|       |    ///
  562|       |    /// # Examples
  563|       |    ///
  564|       |    /// ```rust
  565|       |    /// use core::convert::Infallible;
  566|       |    /// use core::num::NonZeroUsize;
  567|       |    ///
  568|       |    /// use qubit_codec::CapacityError;
  569|       |    /// use qubit_codec::TranscodeDecodeError;
  570|       |    /// use qubit_codec::TranscodeFailure;
  571|       |    /// use qubit_codec::TranscodeProgress;
  572|       |    /// use qubit_codec::TranscodeStatus;
  573|       |    /// use qubit_codec::Transcoder;
  574|       |    ///
  575|       |    /// #[derive(Default)]
  576|       |    /// struct ByteCopy;
  577|       |    ///
  578|       |    /// impl Transcoder for ByteCopy {
  579|       |    ///     type Input = u8;
  580|       |    ///     type Output = u8;
  581|       |    ///     type Error = TranscodeDecodeError<Infallible>;
  582|       |    ///
  583|       |    ///     fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
  584|       |    ///         Ok(input_len)
  585|       |    ///     }
  586|       |    ///
  587|       |    ///     fn reset(
  588|       |    ///         &mut self,
  589|       |    ///         output: &mut [u8],
  590|       |    ///         output_index: usize,
  591|       |    ///     ) -> Result<usize, Self::Error> {
  592|       |    ///         TranscodeFailure::ensure_output_index(output.len(), output_index)?;
  593|       |    ///         Ok(0)
  594|       |    ///     }
  595|       |    ///
  596|       |    ///     fn transcode(
  597|       |    ///         &mut self,
  598|       |    ///         input: &[u8],
  599|       |    ///         input_index: usize,
  600|       |    ///         output: &mut [u8],
  601|       |    ///         output_index: usize,
  602|       |    ///     ) -> Result<TranscodeProgress, Self::Error> {
  603|       |    ///         let mut read = 0;
  604|       |    ///         let mut written = 0;
  605|       |    ///         while input_index + read < input.len() && output_index + written < output.len() {
  606|       |    ///             output[output_index + written] = input[input_index + read];
  607|       |    ///             read += 1;
  608|       |    ///             written += 1;
  609|       |    ///         }
  610|       |    ///         if input_index + read == input.len() {
  611|       |    ///             Ok(TranscodeProgress::complete(read, written))
  612|       |    ///         } else {
  613|       |    ///             let status = TranscodeStatus::NeedOutput {
  614|       |    ///                 required: NonZeroUsize::MIN,
  615|       |    ///             };
  616|       |    ///             Ok(TranscodeProgress::new(
  617|       |    ///                 status,
  618|       |    ///                 read,
  619|       |    ///                 written,
  620|       |    ///             ))
  621|       |    ///         }
  622|       |    ///     }
  623|       |    ///
  624|       |    ///     fn finish(
  625|       |    ///         &mut self,
  626|       |    ///         output: &mut [u8],
  627|       |    ///         output_index: usize,
  628|       |    ///     ) -> Result<usize, Self::Error> {
  629|       |    ///         TranscodeFailure::ensure_output_index(output.len(), output_index)?;
  630|       |    ///         Ok(0)
  631|       |    ///     }
  632|       |    /// }
  633|       |    ///
  634|       |    /// let mut transcoder = ByteCopy::default();
  635|       |    /// let mut reset_output = [];
  636|       |    /// transcoder
  637|       |    ///     .reset(&mut reset_output, 0)
  638|       |    ///     .expect("stateless transcoder reset cannot fail");
  639|       |    /// let mut output = [1_u8; 1];
  640|       |    /// let progress = transcoder
  641|       |    ///     .transcode(&[7], 0, &mut output, 0)
  642|       |    ///     .expect("writer consumes one unit");
  643|       |    /// assert_eq!(TranscodeStatus::Complete, progress.status());
  644|       |    ///
  645|       |    /// let written = transcoder
  646|       |    ///     .finish(&mut output, 1)
  647|       |    ///     .expect("finish does not emit final state for no-op transcoders");
  648|       |    /// assert_eq!(0, written);
  649|       |    /// ```
  650|       |    ///
  651|       |    /// # Parameters
  652|       |    ///
  653|       |    /// - `output`: Complete output unit slice visible to the transcoder.
  654|       |    /// - `output_index`: Absolute output unit index where writing starts.
  655|       |    ///
  656|       |    /// # Returns
  657|       |    ///
  658|       |    /// Returns the number of units written during finalization. Stateless
  659|       |    /// transcoders return `0`. Implementations must not return a count greater
  660|       |    /// than [`Transcoder::max_finish_output_len`] or the output units available
  661|       |    /// from `output_index`.
  662|       |    ///
  663|       |    /// # Errors
  664|       |    ///
  665|       |    /// Returns contract errors (`invalid_output_index`, `insufficient_output`)
  666|       |    /// when capacity checks fail, or policy errors when finish itself
  667|       |    /// fails.
  668|       |    fn finish(&mut self, output: &mut [Self::Output], output_index: usize) -> Result<usize, Self::Error>;
  669|       |
  670|       |    /// Runs a complete one-shot `reset -> transcode -> finish` stream.
  671|       |    ///
  672|       |    /// The `input` slice is treated as complete input at EOF, and output is
  673|       |    /// written from the beginning of `output`. Callers that need to operate on
  674|       |    /// a range inside a larger buffer should slice the input or output
  675|       |    /// before calling this method.
  676|       |    ///
  677|       |    /// Before invoking any lifecycle method, this method queries the
  678|       |    /// state-independent reset, transcode, and finish bounds and requires
  679|       |    /// `output` to provide at least the complete upper bound returned by
  680|       |    /// [`Transcoder::max_total_output_len`] for `input.len()` units.
  681|       |    ///
  682|       |    /// # Parameters
  683|       |    ///
  684|       |    /// - `input`: Complete input unit slice.
  685|       |    /// - `output`: Complete output unit slice where the stream starts at index
  686|       |    ///   `0`.
  687|       |    ///
  688|       |    /// # Returns
  689|       |    ///
  690|       |    /// Returns the number of output units written to `output`.
  691|       |    ///
  692|       |    /// # Errors
  693|       |    ///
  694|       |    /// Returns framework errors when the output buffer is too small, when
  695|       |    /// capacity arithmetic overflows, or when the complete input ends with
  696|       |    /// an incomplete value. Capacity-bound overflow and insufficient
  697|       |    /// complete-stream capacity are reported before `reset`, `transcode`, or
  698|       |    /// `finish` is called, so those preflight failures do not write to
  699|       |    /// `output` or advance transient stream state through lifecycle methods.
  700|       |    /// Once reset begins, domain errors, incomplete input, runtime
  701|       |    /// backpressure caused by an invalid bound implementation, or contract
  702|       |    /// violations may leave partial output and advanced state; this method
  703|       |    /// does not provide general transactional rollback.
  704|       |    ///
  705|       |    /// # Panics
  706|       |    ///
  707|       |    /// Panics when `reset` or `finish` reports writing beyond its declared
  708|       |    /// bound or the available output, when an overridden `transcode_eof`
  709|       |    /// returns invalid progress, or when a returned write count cannot be
  710|       |    /// added to the output cursor. The default `transcode_eof` converts
  711|       |    /// invalid `transcode` progress into [`TranscodeFailure::InvalidProgress`].
  712|    562|    fn transcode_complete_into(
  713|    562|        &mut self,
  714|    562|        input: &[Self::Input],
  715|    562|        output: &mut [Self::Output],
  716|    562|    ) -> Result<usize, Self::Error> {
  717|    562|        let reset_required = self.max_reset_output_len().map_err(TranscodeFailure::from)?;
                          ^561                                                                        ^1
  718|    561|        let transcode_required = self
                          ^560
  719|    561|            .max_transcode_output_len(input.len())
  720|    561|            .map_err(TranscodeFailure::from)?;
                                                          ^1
  721|    560|        let finish_required = self.max_finish_output_len().map_err(TranscodeFailure::from)?;
                          ^559                                                                          ^1
  722|    558|        let total_required =
  723|    559|            sum_output_bounds(reset_required, transcode_required, finish_required).map_err(TranscodeFailure::from)?;
                                                                                                                                ^1
  724|    558|        TranscodeFailure::ensure_output_capacity(output.len(), 0, total_required)?;
                                                                                               ^3
  725|       |
  726|    555|        let reset_written = self.reset(output, 0)?;
                          ^554                                 ^1
  727|    554|        assert!(
  728|    554|            reset_written <= reset_required,
  729|       |            "Transcoder::reset wrote beyond its bound",
  730|       |        );
  731|    553|        assert!(
  732|    553|            reset_written <= output.len(),
  733|       |            "Transcoder::reset wrote beyond available output",
  734|       |        );
  735|    553|        let mut output_cursor = reset_written;
  736|       |
  737|    553|        let progress = self.transcode_eof(input, 0, output, output_cursor)?;
                          ^548                                                          ^5
  738|    548|        let transcode_written = complete_progress_written(progress, input.len(), output_cursor, output.len())?;
                          ^546                                                                                             ^2
  739|    546|        output_cursor = output_cursor
  740|    546|            .checked_add(transcode_written)
  741|    546|            .expect("Transcoder::transcode write count overflowed the output cursor");
  742|       |
  743|    546|        let finish_available = output.len() - output_cursor;
  744|    546|        let finish_written = self.finish(output, output_cursor)?;
                          ^545                                               ^1
  745|    545|        assert!(
  746|    545|            finish_written <= finish_required,
  747|       |            "Transcoder::finish wrote beyond its bound",
  748|       |        );
  749|    544|        assert!(
  750|    544|            finish_written <= finish_available,
  751|       |            "Transcoder::finish wrote beyond available output",
  752|       |        );
  753|    543|        output_cursor = output_cursor
  754|    543|            .checked_add(finish_written)
  755|    543|            .expect("Transcoder::finish write count overflowed the output cursor");
  756|    543|        Ok(output_cursor)
  757|    559|    }
  758|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/codec_value_decoder.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Value decoder adapter backed by a low-level codec.
    9|       |
   10|       |use core::fmt;
   11|       |
   12|       |use qubit_utils::try_reserve_vec;
   13|       |
   14|       |use super::DecodeLifecycleOutput;
   15|       |use super::DecodeLifecycleProgress;
   16|       |use super::ValueDecoder;
   17|       |use crate::Codec;
   18|       |use crate::TranscodeDecodeErrorOf;
   19|       |use crate::TranscodeFailure;
   20|       |use crate::codec::assert_unit_bounds;
   21|       |use crate::value::codec_value_lifecycle::decode_exact_complete_value;
   22|       |
   23|       |/// Decodes one encoded unit slice into one owned value by using a [`Codec`].
   24|       |///
   25|       |/// `CodecValueDecoder` is the default bridge from the low-level unchecked
   26|       |/// [`Codec`] contract to the convenience-layer [`ValueDecoder`] contract. The
   27|       |/// supplied input slice must contain exactly one encoded value. The adapter
   28|       |/// runs the complete decode lifecycle, including [`Codec::decode_finish`],
   29|       |/// before returning the decoded value.
   30|       |///
   31|       |/// # Type Parameters
   32|       |///
   33|       |/// - `C`: Low-level codec used to decode one value.
   34|       |///
   35|       |/// # Examples
   36|       |///
   37|       |/// ```
   38|       |/// use std::convert::Infallible;
   39|       |/// use std::num::NonZeroUsize;
   40|       |///
   41|       |/// use qubit_codec::Codec;
   42|       |/// use qubit_codec::CodecValueDecoder;
   43|       |/// use qubit_codec::DecodeFailure;
   44|       |///
   45|       |/// struct Identity;
   46|       |/// impl Codec for Identity {
   47|       |///     type Value = u8;
   48|       |///     type Unit = u8;
   49|       |///     type DecodeError = Infallible;
   50|       |///     type EncodeError = Infallible;
   51|       |///     const MIN_UNITS_PER_VALUE: usize = 1;
   52|       |///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
   53|       |///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
   54|       |///     unsafe fn decode(&mut self, input: &[u8], index: usize)
   55|       |///         -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
   56|       |///         Ok((input[index], NonZeroUsize::new(1).unwrap()))
   57|       |///     }
   58|       |///     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize)
   59|       |///         -> Result<usize, Infallible> {
   60|       |///         output[index] = *value;
   61|       |///         Ok(1)
   62|       |///     }
   63|       |/// }
   64|       |/// let mut decoder = CodecValueDecoder::new(Identity);
   65|       |/// assert_eq!(decoder.decode(&[7]).unwrap(), 7);
   66|       |/// ```
   67|       |pub struct CodecValueDecoder<C>
   68|       |where
   69|       |    C: Codec,
   70|       |{
   71|       |    /// Low-level codec used for one-value decoding.
   72|       |    codec: C,
   73|       |}
   74|       |
   75|       |impl<C> CodecValueDecoder<C>
   76|       |where
   77|       |    C: Codec,
   78|       |{
   79|       |    /// Creates a decoder backed by `codec`.
   80|       |    ///
   81|       |    /// # Parameters
   82|       |    ///
   83|       |    /// - `codec`: Low-level codec used to decode one value.
   84|       |    ///
   85|       |    /// # Returns
   86|       |    ///
   87|       |    /// Returns a value decoder adapter for the supplied codec.
   88|       |    ///
   89|       |    /// # Panics
   90|       |    ///
   91|       |    /// Fails at compile time if codec decode bounds are zero or the minimum
   92|       |    /// exceeds the maximum.
   93|       |    #[inline]
   94|       |    #[must_use]
   95|     33|    pub fn new(codec: C) -> Self {
   96|     33|        assert_unit_bounds::<C>();
   97|     33|        Self { codec }
   98|     33|    }
   99|       |
  100|       |    /// Returns a shared reference to the wrapped codec.
  101|       |    ///
  102|       |    /// # Returns
  103|       |    ///
  104|       |    /// The codec borrowed for as long as the adapter is borrowed.
  105|       |    #[inline]
  106|       |    #[must_use]
  107|      1|    pub const fn codec(&self) -> &C {
  108|      1|        &self.codec
  109|      1|    }
  110|       |
  111|       |    /// Returns a mutable reference to the wrapped codec.
  112|       |    ///
  113|       |    /// The next value operation starts a fresh codec lifecycle, so mutations
  114|       |    /// are applied to the following operation.
  115|       |    ///
  116|       |    /// # Returns
  117|       |    ///
  118|       |    /// An exclusive borrow of the codec without allocating.
  119|       |    #[inline]
  120|       |    #[must_use]
  121|      1|    pub fn codec_mut(&mut self) -> &mut C {
  122|      1|        &mut self.codec
  123|      1|    }
  124|       |
  125|       |    /// Consumes the adapter and returns its wrapped codec.
  126|       |    ///
  127|       |    /// # Returns
  128|       |    ///
  129|       |    /// The owned codec, preserving its most recent lifecycle state.
  130|       |    #[inline]
  131|       |    #[must_use]
  132|      1|    pub fn into_codec(self) -> C {
  133|      1|        self.codec
  134|      1|    }
  135|       |
  136|       |    /// Decodes exactly one encoded value from `input`.
  137|       |    ///
  138|       |    /// # Parameters
  139|       |    ///
  140|       |    /// - `input`: Encoded units for exactly one value.
  141|       |    ///
  142|       |    /// # Returns
  143|       |    ///
  144|       |    /// Returns the decoded value.
  145|       |    ///
  146|       |    /// Codecs that may emit decode reset or finish values must use
  147|       |    /// [`Self::decode_lifecycle`] or [`Self::decode_lifecycle_with_scratch`].
  148|       |    ///
  149|       |    /// # Errors
  150|       |    ///
  151|       |    /// Returns [`crate::TranscodeFailure::UnsupportedDecodeLifecycleOutput`]
  152|       |    /// before inspecting `input` or running codec hooks when reset or finish
  153|       |    /// may emit values. Returns
  154|       |    /// [`crate::TranscodeFailure::IncompleteInput`] when fewer than
  155|       |    /// [`Codec::MIN_UNITS_PER_VALUE`] units are available or when the codec
  156|       |    /// reports [`crate::DecodeFailure::Incomplete`] without a source.
  157|       |    /// An incomplete failure carrying a codec source is preserved as
  158|       |    /// [`crate::TranscodeDecodeError::Domain`] with
  159|       |    /// [`crate::TranscodeDomainError::Main`] phase information.
  160|       |    /// In this one-shot API, either form of incomplete input is a terminal
  161|       |    /// error rather than a resumable streaming status. Returns a domain error
  162|       |    /// when the wrapped codec rejects or cannot finish the input. Returns
  163|       |    /// [`crate::TranscodeFailure::TrailingInput`] when a value is decoded but
  164|       |    /// extra input remains.
  165|       |    ///
  166|       |    /// # Panics
  167|       |    ///
  168|       |    /// Panics when the wrapped codec reports a consumed unit count larger than
  169|       |    /// the input slice length or [`Codec::MAX_DECODE_UNITS_PER_VALUE`].
  170|     17|    pub fn decode(&mut self, input: &[C::Unit]) -> Result<C::Value, TranscodeDecodeErrorOf<C>> {
  171|     17|        TranscodeFailure::ensure_no_decode_lifecycle_output::<C>()?;
                                                                                ^1
  172|     10|        let (value, reset_written, finish_written) =
  173|     16|            decode_exact_complete_value(&mut self.codec, input, &mut [], &mut [])?;
                                                                                               ^6
  174|     10|        debug_assert_eq!(0, reset_written);
  175|      6|        debug_assert_eq!(0, finish_written);
  176|      6|        Ok(value)
  177|     13|    }
  178|       |
  179|       |    /// Decodes exactly one value and preserves all lifecycle output.
  180|       |    ///
  181|       |    /// # Parameters
  182|       |    ///
  183|       |    /// - `input`: Encoded units for exactly one value.
  184|       |    ///
  185|       |    /// # Returns
  186|       |    ///
  187|       |    /// Returns reset values, the main decoded value, and finish values in
  188|       |    /// separate owned buffers.
  189|       |    ///
  190|       |    /// # Errors
  191|       |    ///
  192|       |    /// Returns a framework error for input shorter than the codec minimum,
  193|       |    /// incomplete input without a codec source, or trailing units. Incomplete
  194|       |    /// input carrying a codec source is preserved as a main-phase domain
  195|       |    /// error. Returns a phase-aware domain error when reset, decode, or finish
  196|       |    /// fails. Returns an allocation failure when either lifecycle buffer cannot
  197|       |    /// reserve its declared capacity.
  198|       |    ///
  199|       |    /// # Panics
  200|       |    ///
  201|       |    /// Panics when the wrapped codec violates its declared reset, decode, or
  202|       |    /// finish bounds.
  203|     12|    pub fn decode_lifecycle(
  204|     12|        &mut self,
  205|     12|        input: &[C::Unit],
  206|     12|    ) -> Result<DecodeLifecycleOutput<C::Value>, TranscodeDecodeErrorOf<C>>
  207|     12|    where
  208|     12|        C::Value: Default,
  209|       |    {
  210|     12|        let mut reset = Vec::new();
  211|     12|        try_reserve_vec(&mut reset, C::MAX_DECODE_RESET_VALUES).map_err(|_| TranscodeFailure::allocation_failed())?;
                                                                                          ^1                                    ^1
  212|     11|        reset.resize_with(C::MAX_DECODE_RESET_VALUES, C::Value::default);
  213|     11|        let mut finish = Vec::new();
  214|     11|        try_reserve_vec(&mut finish, C::MAX_DECODE_FINISH_VALUES).map_err(|_| TranscodeFailure::allocation_failed())?;
                                                                                            ^1                                    ^1
  215|     10|        finish.resize_with(C::MAX_DECODE_FINISH_VALUES, C::Value::default);
  216|     10|        let (value, reset_written, finish_written) = self
                           ^8     ^8             ^8
  217|     10|            .decode_lifecycle_with_scratch(input, &mut reset, &mut finish)?
                                                                                        ^2
  218|      8|            .into_parts();
  219|      8|        reset.truncate(reset_written);
  220|      8|        finish.truncate(finish_written);
  221|      8|        Ok(DecodeLifecycleOutput::new(reset, value, finish))
  222|     12|    }
  223|       |
  224|       |    /// Decodes exactly one value into separate lifecycle output buffers.
  225|       |    ///
  226|       |    /// # Parameters
  227|       |    ///
  228|       |    /// - `input`: Encoded units for exactly one value.
  229|       |    /// - `reset_output`: Destination for values emitted by decode reset.
  230|       |    /// - `finish_output`: Destination for values emitted by decode finish.
  231|       |    ///
  232|       |    /// # Returns
  233|       |    ///
  234|       |    /// Returns the main decoded value and the initialized lengths of both
  235|       |    /// lifecycle output buffers.
  236|       |    ///
  237|       |    /// # Errors
  238|       |    ///
  239|       |    /// Returns a framework error before running any lifecycle hook when either
  240|       |    /// output buffer is shorter than its corresponding codec bound. Also
  241|       |    /// returns a framework error for input shorter than the codec minimum,
  242|       |    /// incomplete input without a codec source, or trailing units. Incomplete
  243|       |    /// input carrying a codec source is preserved as a main-phase domain
  244|       |    /// error. Returns phase-aware domain errors when reset, decode, or finish
  245|       |    /// fails.
  246|       |    ///
  247|       |    /// # Panics
  248|       |    ///
  249|       |    /// Panics when the wrapped codec violates its declared reset, decode, or
  250|       |    /// finish bounds.
  251|     18|    pub fn decode_lifecycle_with_scratch(
  252|     18|        &mut self,
  253|     18|        input: &[C::Unit],
  254|     18|        reset_output: &mut [C::Value],
  255|     18|        finish_output: &mut [C::Value],
  256|     18|    ) -> Result<DecodeLifecycleProgress<C::Value>, TranscodeDecodeErrorOf<C>> {
  257|     13|        let (value, reset_written, finish_written) =
  258|     18|            decode_exact_complete_value(&mut self.codec, input, reset_output, finish_output)?;
                                                                                                          ^5
  259|     13|        Ok(DecodeLifecycleProgress::new(value, reset_written, finish_written))
  260|     18|    }
  261|       |}
  262|       |
  263|       |impl<C> ValueDecoder<[C::Unit]> for CodecValueDecoder<C>
  264|       |where
  265|       |    C: Codec,
  266|       |{
  267|       |    /// Owned value returned after the complete decode lifecycle.
  268|       |    type Output = C::Value;
  269|       |    /// Framework or phase-aware codec failure from one-value decoding.
  270|       |    type Error = TranscodeDecodeErrorOf<C>;
  271|       |
  272|       |    /// Decodes one value through the owned-value adapter.
  273|       |    ///
  274|       |    /// # Parameters
  275|       |    ///
  276|       |    /// - `input`: Complete encoded representation of exactly one value.
  277|       |    ///
  278|       |    /// # Returns
  279|       |    ///
  280|       |    /// The owned decoded value after finishing the codec lifecycle.
  281|       |    ///
  282|       |    /// # Errors
  283|       |    ///
  284|       |    /// Returns the same unsupported-lifecycle, incomplete-input,
  285|       |    /// trailing-input, and codec errors as [`Self::decode`].
  286|       |    ///
  287|       |    /// # Panics
  288|       |    ///
  289|       |    /// Panics if the codec reports consumption beyond its bounds or the input.
  290|       |    #[inline]
  291|     14|    fn decode(&mut self, input: &[C::Unit]) -> Result<Self::Output, Self::Error> {
  292|     14|        self.decode(input)
  293|     14|    }
  294|       |}
  295|       |
  296|       |impl<C> fmt::Debug for CodecValueDecoder<C>
  297|       |where
  298|       |    C: Codec + fmt::Debug,
  299|       |{
  300|       |    /// Formats the decoder without requiring finished values to be printable.
  301|       |    ///
  302|       |    /// # Parameters
  303|       |    ///
  304|       |    /// - `formatter`: Destination and formatting options.
  305|       |    ///
  306|       |    /// # Returns
  307|       |    ///
  308|       |    /// Unit on successful formatting.
  309|       |    ///
  310|       |    /// # Errors
  311|       |    ///
  312|       |    /// Returns the formatter error if writing the codec representation fails.
  313|      2|    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
  314|      2|        formatter
  315|      2|            .debug_struct("CodecValueDecoder")
  316|      2|            .field("codec", &self.codec)
  317|      2|            .finish()
  318|      2|    }
  319|       |}
  320|       |
  321|       |impl<C> Default for CodecValueDecoder<C>
  322|       |where
  323|       |    C: Codec + Default,
  324|       |{
  325|       |    /// Creates a decoder from the default codec.
  326|       |    ///
  327|       |    /// # Returns
  328|       |    ///
  329|       |    /// An adapter owning `C::default()`.
  330|       |    ///
  331|       |    /// # Panics
  332|       |    ///
  333|       |    /// Fails at compile time for invalid codec decode bounds, as [`Self::new`].
  334|       |    /// May panic if the codec's own default constructor panics.
  335|      2|    fn default() -> Self {
  336|      2|        Self::new(C::default())
  337|      2|    }
  338|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/codec_value_encoder.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Value encoder adapter backed by a low-level codec.
    9|       |
   10|       |use core::fmt;
   11|       |
   12|       |use qubit_utils::try_reserve_vec;
   13|       |
   14|       |use super::ValueEncoder;
   15|       |use crate::CapacityError;
   16|       |use crate::Codec;
   17|       |use crate::TranscodeEncodeErrorOf;
   18|       |use crate::TranscodeFailure;
   19|       |use crate::codec::assert_unit_bounds;
   20|       |use crate::value::codec_value_lifecycle::encode_complete_value_into_reserved;
   21|       |use crate::value::codec_value_lifecycle::max_complete_encode_units;
   22|       |
   23|       |/// Encodes one borrowed value into owned units by using a [`Codec`].
   24|       |///
   25|       |/// `CodecValueEncoder` is the default bridge from the low-level unchecked
   26|       |/// [`Codec`] contract to the convenience-layer [`ValueEncoder`] contract. Each
   27|       |/// call emits stream-start output through [`Codec::encode_reset`], encodes one
   28|       |/// value through [`Codec::encode`], finishes encode-side state through
   29|       |/// [`Codec::encode_finish`], and returns the owned output truncated to the
   30|       |/// units actually written.
   31|       |///
   32|       |/// # Type Parameters
   33|       |///
   34|       |/// - `C`: Low-level codec used to encode one value.
   35|       |///
   36|       |/// # Examples
   37|       |///
   38|       |/// ```
   39|       |/// use core::convert::Infallible;
   40|       |/// use core::num::NonZeroUsize;
   41|       |///
   42|       |/// use qubit_codec::Codec;
   43|       |/// use qubit_codec::CodecValueEncoder;
   44|       |/// use qubit_codec::DecodeFailure;
   45|       |/// use qubit_codec::ValueEncoder;
   46|       |///
   47|       |/// # struct IdentityCodec;
   48|       |/// # impl Codec for IdentityCodec {
   49|       |/// #     type Value = u8;
   50|       |/// #     type Unit = u8;
   51|       |/// #     type DecodeError = Infallible;
   52|       |/// #     type EncodeError = Infallible;
   53|       |/// #     const MIN_UNITS_PER_VALUE: usize = 1;
   54|       |/// #     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
   55|       |/// #     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
   56|       |/// #     unsafe fn decode(&mut self, input: &[u8], index: usize)
   57|       |/// #         -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>> {
   58|       |/// #         Ok((input[index], NonZeroUsize::MIN))
   59|       |/// #     }
   60|       |/// #     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize)
   61|       |/// #         -> Result<usize, Infallible> {
   62|       |/// #         output[index] = *value;
   63|       |/// #         Ok(1)
   64|       |/// #     }
   65|       |/// # }
   66|       |/// let mut encoder = CodecValueEncoder::new(IdentityCodec);
   67|       |/// assert_eq!(encoder.encode(&7_u8).unwrap(), vec![7]);
   68|       |/// ```
   69|       |pub struct CodecValueEncoder<C> {
   70|       |    /// Low-level codec used for one-value encoding.
   71|       |    codec: C,
   72|       |}
   73|       |
   74|       |impl<C> CodecValueEncoder<C>
   75|       |where
   76|       |    C: Codec,
   77|       |{
   78|       |    /// Creates an encoder backed by `codec`.
   79|       |    ///
   80|       |    /// # Parameters
   81|       |    ///
   82|       |    /// - `codec`: Low-level codec used to encode one value.
   83|       |    ///
   84|       |    /// # Returns
   85|       |    ///
   86|       |    /// Returns a value encoder adapter for the supplied codec.
   87|       |    ///
   88|       |    /// # Compile-Time Checks
   89|       |    ///
   90|       |    /// Fails to compile when the supplied codec declares a zero decode unit
   91|       |    /// bound or when [`Codec::MIN_UNITS_PER_VALUE`] exceeds
   92|       |    /// [`Codec::MAX_DECODE_UNITS_PER_VALUE`]. The encode bound may be zero for
   93|       |    /// a fully buffered codec.
   94|       |    #[inline]
   95|       |    #[must_use]
   96|     24|    pub fn new(codec: C) -> Self {
   97|     24|        assert_unit_bounds::<C>();
   98|     24|        Self { codec }
   99|     24|    }
  100|       |
  101|       |    /// Returns a shared reference to the wrapped codec.
  102|       |    ///
  103|       |    /// # Returns
  104|       |    ///
  105|       |    /// The codec borrowed for as long as this adapter is shared, without
  106|       |    /// cloning.
  107|       |    #[inline]
  108|       |    #[must_use]
  109|      1|    pub const fn codec(&self) -> &C {
  110|      1|        &self.codec
  111|      1|    }
  112|       |
  113|       |    /// Returns a mutable reference to the wrapped codec.
  114|       |    ///
  115|       |    /// The next value operation starts a fresh codec lifecycle, so mutations
  116|       |    /// are applied to the following operation.
  117|       |    ///
  118|       |    /// # Returns
  119|       |    ///
  120|       |    /// An exclusive codec borrow that prevents simultaneous adapter operations.
  121|       |    #[inline]
  122|       |    #[must_use]
  123|      1|    pub fn codec_mut(&mut self) -> &mut C {
  124|      1|        &mut self.codec
  125|      1|    }
  126|       |
  127|       |    /// Consumes the adapter and returns its wrapped codec.
  128|       |    ///
  129|       |    /// # Returns
  130|       |    ///
  131|       |    /// The owned codec in its current lifecycle state, without resetting it.
  132|       |    #[inline]
  133|       |    #[must_use]
  134|      1|    pub fn into_codec(self) -> C {
  135|      1|        self.codec
  136|      1|    }
  137|       |
  138|       |    /// Encodes one borrowed value and appends the emitted units to `output`.
  139|       |    ///
  140|       |    /// This method is the reusable-buffer counterpart of
  141|       |    /// [`ValueEncoder::encode`]. It emits stream-start output through
  142|       |    /// [`Codec::encode_reset`], encodes `input` through [`Codec::encode`], and
  143|       |    /// finishes encode-side state through [`Codec::encode_finish`], appending
  144|       |    /// only the units actually written. When encoding fails, the vector length
  145|       |    /// is restored to its original value.
  146|       |    ///
  147|       |    /// # Parameters
  148|       |    ///
  149|       |    /// - `input`: Value to encode.
  150|       |    /// - `output`: Destination vector receiving appended encoded units.
  151|       |    ///
  152|       |    /// # Returns
  153|       |    ///
  154|       |    /// Returns the number of units appended to `output`.
  155|       |    ///
  156|       |    /// # Errors
  157|       |    ///
  158|       |    /// Returns the wrapped codec's encode error when reset output, `input`, or
  159|       |    /// finish output cannot be represented. Returns a framework error when
  160|       |    /// output length arithmetic overflows or the output allocation fails.
  161|       |    ///
  162|       |    /// # Panics
  163|       |    ///
  164|       |    /// Panics when the wrapped codec reports more reset or finish output than
  165|       |    /// its declared bounds, when [`Codec::encode_len`] exceeds
  166|       |    /// [`Codec::MAX_ENCODE_UNITS_PER_VALUE`], or when encoding writes a
  167|       |    /// different value width than [`Codec::encode_len`].
  168|     25|    pub fn encode_into(
  169|     25|        &mut self,
  170|     25|        input: &C::Value,
  171|     25|        output: &mut Vec<C::Unit>,
  172|     25|    ) -> Result<usize, TranscodeEncodeErrorOf<C>>
  173|     25|    where
  174|     25|        C::Unit: Default,
  175|       |    {
  176|     25|        let units = max_complete_encode_units::<C>()?;
                          ^23                                     ^2
  177|     23|        let original_len = output.len();
  178|     23|        let target_len = original_len
                          ^22
  179|     23|            .checked_add(units)
  180|     23|            .ok_or(CapacityError::OutputLengthOverflow)?;
                                                                     ^1
  181|     22|        try_reserve_vec(output, units).map_err(|_| TranscodeFailure::allocation_failed())?;
                                                                 ^1                                    ^1
  182|     21|        output.resize_with(target_len, C::Unit::default);
  183|       |
  184|     21|        match encode_complete_value_into_reserved(&mut self.codec, input, output, original_len, units) {
  185|     16|            Ok(written) => {
  186|     16|                output.truncate(original_len + written);
  187|     16|                Ok(written)
  188|       |            }
  189|      5|            Err(error) => {
  190|      5|                output.truncate(original_len);
  191|      5|                Err(error)
  192|       |            }
  193|       |        }
  194|     25|    }
  195|       |}
  196|       |
  197|       |impl<C> ValueEncoder<C::Value> for CodecValueEncoder<C>
  198|       |where
  199|       |    C: Codec,
  200|       |    C::Unit: Default,
  201|       |{
  202|       |    /// Owned units containing the complete reset/value/finish representation.
  203|       |    type Output = Vec<C::Unit>;
  204|       |    /// Codec-domain failures or framework capacity/allocation failures.
  205|       |    type Error = TranscodeEncodeErrorOf<C>;
  206|       |
  207|       |    /// Encodes one borrowed value into owned units.
  208|       |    ///
  209|       |    /// # Parameters
  210|       |    ///
  211|       |    /// - `input`: Value to encode.
  212|       |    ///
  213|       |    /// # Returns
  214|       |    ///
  215|       |    /// Returns stream-start output followed by the units written for `input`
  216|       |    /// and any encode-finish output.
  217|       |    ///
  218|       |    /// # Errors
  219|       |    ///
  220|       |    /// Returns the wrapped codec's encode error when reset output, `input`, or
  221|       |    /// finish output cannot be represented. Also returns framework capacity
  222|       |    /// overflow or allocation failures while preparing the output vector.
  223|       |    ///
  224|       |    /// # Panics
  225|       |    ///
  226|       |    /// Panics when the wrapped codec reports more reset or finish output than
  227|       |    /// its declared bounds, or a value width different from
  228|       |    /// [`Codec::encode_len`].
  229|     18|    fn encode(&mut self, input: &C::Value) -> Result<Self::Output, Self::Error> {
  230|     18|        let mut output = Vec::new();
  231|     18|        self.encode_into(input, &mut output)?;
                                                          ^5
  232|     13|        Ok(output)
  233|     18|    }
  234|       |}
  235|       |
  236|       |impl<C> fmt::Debug for CodecValueEncoder<C>
  237|       |where
  238|       |    C: Codec + fmt::Debug,
  239|       |{
  240|       |    /// Formats the encoder without requiring finished values to be printable.
  241|       |    ///
  242|       |    /// # Parameters
  243|       |    ///
  244|       |    /// - `formatter`: Destination receiving the wrapped codec description.
  245|       |    ///
  246|       |    /// # Returns
  247|       |    ///
  248|       |    /// `Ok(())` when the description has been written.
  249|       |    ///
  250|       |    /// # Errors
  251|       |    ///
  252|       |    /// Propagates errors from the destination formatter.
  253|      1|    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
  254|      1|        formatter
  255|      1|            .debug_struct("CodecValueEncoder")
  256|      1|            .field("codec", &self.codec)
  257|      1|            .finish()
  258|      1|    }
  259|       |}
  260|       |
  261|       |impl<C> Default for CodecValueEncoder<C>
  262|       |where
  263|       |    C: Codec + Default,
  264|       |{
  265|       |    /// Creates an encoder from the default codec.
  266|       |    ///
  267|       |    /// # Returns
  268|       |    ///
  269|       |    /// An adapter owning `C::default()`, with the same compile-time unit-bound
  270|       |    /// checks as [`Self::new`].
  271|       |    #[inline]
  272|      1|    fn default() -> Self {
  273|      1|        Self::new(C::default())
  274|      1|    }
  275|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/codec_value_lifecycle.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Internal helpers for complete single-value codec lifecycles.
    9|       |
   10|       |use crate::CapacityError;
   11|       |use crate::Codec;
   12|       |use crate::TranscodeDecodeError;
   13|       |use crate::TranscodeDecodeErrorOf;
   14|       |use crate::TranscodeEncodeError;
   15|       |use crate::TranscodeEncodeErrorOf;
   16|       |use crate::TranscodeFailure;
   17|       |use crate::codec::assert_unit_bounds;
   18|       |
   19|       |/// Returns the conservative maximum unit count for a complete encode lifecycle.
   20|       |///
   21|       |/// # Type Parameters
   22|       |///
   23|       |/// - `C`: Codec whose encode bounds are queried.
   24|       |///
   25|       |/// # Returns
   26|       |///
   27|       |/// Returns the checked sum of encode reset, value, and finish unit bounds.
   28|       |///
   29|       |/// # Errors
   30|       |///
   31|       |/// Returns [`CapacityError::OutputLengthOverflow`] when the sum cannot be
   32|       |/// represented as `usize`.
   33|       |#[inline]
   34|     43|pub(crate) fn max_complete_encode_units<C>() -> Result<usize, CapacityError>
   35|     43|where
   36|     43|    C: Codec,
   37|       |{
   38|     43|    assert_unit_bounds::<C>();
   39|       |    C::MAX_ENCODE_RESET_UNITS
   40|     43|        .checked_add(C::MAX_ENCODE_UNITS_PER_VALUE)
   41|     43|        .and_then(|units| units.checked_add(C::MAX_ENCODE_FINISH_UNITS))
                                        ^40   ^40
   42|     43|        .ok_or(CapacityError::OutputLengthOverflow)
   43|     43|}
   44|       |
   45|       |/// Encodes one value through reset, encode, and finish into reserved output.
   46|       |///
   47|       |/// # Parameters
   48|       |///
   49|       |/// - `codec`: Codec used for the complete encode lifecycle.
   50|       |/// - `value`: Value to encode.
   51|       |/// - `output`: Destination unit buffer.
   52|       |/// - `output_index`: Start index in `output`.
   53|       |/// - `reserved`: Conservative complete output length returned by
   54|       |///   [`max_complete_encode_units`].
   55|       |///
   56|       |/// # Returns
   57|       |///
   58|       |/// Returns the total number of units written.
   59|       |///
   60|       |/// # Errors
   61|       |///
   62|       |/// Returns [`TranscodeEncodeError`] when the reset-state codec domain rejects
   63|       |/// `value`, output length arithmetic overflows, reserved output is
   64|       |/// insufficient for a codec-reported exact width, or a codec reset, encode, or
   65|       |/// finish hook reports a domain error.
   66|       |///
   67|       |/// # Panics
   68|       |///
   69|       |/// Panics when the caller did not reserve `reserved` units at `output_index`,
   70|       |/// or when the codec violates its reset, encoded-value, or finish write-count
   71|       |/// contract.
   72|     36|pub(crate) fn encode_complete_value_into_reserved<C>(
   73|     36|    codec: &mut C,
   74|     36|    value: &C::Value,
   75|     36|    output: &mut [C::Unit],
   76|     36|    output_index: usize,
   77|     36|    reserved: usize,
   78|     36|) -> Result<usize, TranscodeEncodeErrorOf<C>>
   79|     36|where
   80|     36|    C: Codec,
   81|       |{
   82|     36|    assert!(
   83|     36|        output
   84|     36|            .len()
   85|     36|            .checked_sub(output_index)
   86|     36|            .is_some_and(|available| available >= reserved),
   87|       |        "complete encode output was not reserved",
   88|       |    );
   89|       |
   90|     35|    let reset_written = unsafe {
   91|       |        // SAFETY: The caller reserved `reserved` units, which includes the
   92|       |        // codec-declared reset bound at `output_index`.
   93|     36|        codec.encode_reset(output, output_index)
   94|       |    }
   95|     36|    .map_err(TranscodeEncodeError::domain_reset)?;
                                                              ^1
   96|     35|    assert!(
   97|     35|        reset_written <= C::MAX_ENCODE_RESET_UNITS,
   98|       |        "Codec::encode_reset wrote beyond its reset bound",
   99|       |    );
  100|       |
  101|     35|    if !codec.can_encode_value(value) {
  102|      2|        return Err(TranscodeEncodeError::unencodable_without_context(0));
  103|     33|    }
  104|     33|    let value_units = codec.encode_len(value);
  105|     33|    assert!(
  106|     33|        value_units <= C::MAX_ENCODE_UNITS_PER_VALUE,
  107|       |        "Codec::encode_len exceeded Codec::MAX_ENCODE_UNITS_PER_VALUE",
  108|       |    );
  109|     30|    let value_and_finish = value_units
  110|     30|        .checked_add(C::MAX_ENCODE_FINISH_UNITS)
  111|     30|        .ok_or(CapacityError::OutputLengthOverflow)?;
                                                                 ^0
  112|     30|    let value_index = output_index + reset_written;
  113|     30|    TranscodeFailure::ensure_output_capacity(output.len(), value_index, value_and_finish)?;
                                                                                                       ^0
  114|     27|    let value_written = unsafe {
  115|       |        // SAFETY: The capacity check above leaves the reset-state exact value
  116|       |        // width writable after reset output, and the reset-state domain check
  117|       |        // establishes the codec's value precondition.
  118|     30|        codec.encode(value, output, value_index)
  119|       |    }
  120|     30|    .map_err(|error| TranscodeEncodeError::domain_main(error, 0))?;
                                   ^3                                ^3        ^3
  121|     27|    assert!(
  122|     27|        value_written == value_units,
  123|       |        "Codec::encode wrote a different length than Codec::encode_len",
  124|       |    );
  125|       |
  126|     26|    let finish_index = value_index + value_written;
  127|     23|    let finish_written = unsafe {
  128|       |        // SAFETY: The reserved complete lifecycle output includes the
  129|       |        // codec-declared finish bound after reset and value output.
  130|     26|        codec.encode_finish(output, finish_index)
  131|       |    }
  132|     26|    .map_err(TranscodeEncodeError::domain_finish)?;
                                                               ^3
  133|     23|    assert!(
  134|     23|        finish_written <= C::MAX_ENCODE_FINISH_UNITS,
  135|       |        "Codec::encode_finish wrote beyond its finish bound",
  136|       |    );
  137|     23|    Ok(reset_written + value_written + finish_written)
  138|     32|}
  139|       |
  140|       |/// Decodes exactly one value through reset, decode, and finish.
  141|       |///
  142|       |/// Reset and finish output values are written into separate caller-provided
  143|       |/// buffers so both phases remain observable.
  144|       |///
  145|       |/// # Parameters
  146|       |///
  147|       |/// - `codec`: Codec used for the complete decode lifecycle.
  148|       |/// - `input`: Source units for exactly one encoded value.
  149|       |/// - `reset_output`: Destination for reset output values.
  150|       |/// - `finish_output`: Destination for finish output values.
  151|       |///
  152|       |/// # Returns
  153|       |///
  154|       |/// Returns the main decoded value and the actual reset and finish output
  155|       |/// lengths.
  156|       |///
  157|       |/// # Errors
  158|       |///
  159|       |/// Returns [`TranscodeDecodeError`] when either lifecycle output buffer is
  160|       |/// shorter than its declared bound, decoding fails, trailing input remains, or
  161|       |/// reset or finish fails.
  162|       |///
  163|       |/// # Panics
  164|       |///
  165|       |/// Panics when the codec consumes beyond available input or its declared decode
  166|       |/// maximum, or writes more reset or finish values than its declared bounds.
  167|     34|pub(crate) fn decode_exact_complete_value<C>(
  168|     34|    codec: &mut C,
  169|     34|    input: &[C::Unit],
  170|     34|    reset_output: &mut [C::Value],
  171|     34|    finish_output: &mut [C::Value],
  172|     34|) -> Result<(C::Value, usize, usize), TranscodeDecodeErrorOf<C>>
  173|     34|where
  174|     34|    C: Codec,
  175|       |{
  176|     34|    assert_unit_bounds::<C>();
  177|     34|    TranscodeFailure::ensure_output_capacity(reset_output.len(), 0, C::MAX_DECODE_RESET_VALUES)?;
                                                                                                             ^1
  178|     33|    TranscodeFailure::ensure_output_capacity(finish_output.len(), 0, C::MAX_DECODE_FINISH_VALUES)?;
                                                                                                               ^1
  179|     30|    let reset_written = unsafe {
  180|       |        // SAFETY: The capacity check above reserves the codec's declared
  181|       |        // decode-reset output bound.
  182|     32|        codec.decode_reset(reset_output, 0)
  183|       |    }
  184|     32|    .map_err(TranscodeDecodeError::domain_reset)?;
                                                              ^2
  185|     30|    assert!(
  186|     30|        reset_written <= C::MAX_DECODE_RESET_VALUES,
  187|       |        "Codec::decode_reset wrote beyond its reset bound",
  188|       |    );
  189|       |
  190|     29|    TranscodeFailure::ensure_min_input(input.len(), 0, C::MIN_UNITS_PER_VALUE)?;
                                                                                            ^1
  191|     25|    let (value, consumed) = unsafe {
  192|       |        // SAFETY: The input check above guarantees the minimum readable units
  193|       |        // required by `Codec::decode_eof` at index 0.
  194|     28|        codec.decode_eof(input, 0)
  195|       |    }
  196|     28|    .map_err(|failure| TranscodeDecodeError::from_decode_failure(failure, 0, input.len()))?;
                                     ^3                                        ^3          ^3    ^3     ^3
  197|     25|    assert!(
  198|     25|        consumed.get() <= input.len(),
  199|       |        "Codec::decode consumed beyond available input",
  200|       |    );
  201|     24|    assert!(
  202|     24|        consumed.get() <= C::MAX_DECODE_UNITS_PER_VALUE,
  203|       |        "Codec::decode consumed beyond Codec::MAX_DECODE_UNITS_PER_VALUE",
  204|       |    );
  205|     23|    TranscodeFailure::ensure_no_trailing_input(consumed.get(), input.len())?;
                                                                                         ^1
  206|       |
  207|     20|    let finish_written = unsafe {
  208|       |        // SAFETY: The capacity check above reserves the codec's declared
  209|       |        // decode-finish output bound.
  210|     22|        codec.decode_finish(finish_output, 0)
  211|       |    }
  212|     22|    .map_err(TranscodeDecodeError::domain_finish)?;
                                                               ^2
  213|     20|    assert!(
  214|     20|        finish_written <= C::MAX_DECODE_FINISH_VALUES,
  215|       |        "Codec::decode_finish wrote beyond its finish bound",
  216|       |    );
  217|     19|    Ok((value, reset_written, finish_written))
  218|     30|}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/decode_lifecycle_output.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Owned output from a complete single-value decode lifecycle.
    9|       |
   10|       |/// Values produced by decode reset, the main decode, and decode finish.
   11|       |///
   12|       |/// This result keeps lifecycle output in phase order without allowing finish
   13|       |/// output to overwrite reset output.
   14|       |///
   15|       |/// # Type Parameters
   16|       |///
   17|       |/// - `V`: Logical value type produced by the codec.
   18|       |///
   19|       |/// # Examples
   20|       |///
   21|       |/// A stateless identity codec produces one lifecycle whose reset and finish
   22|       |/// phases stay empty:
   23|       |///
   24|       |/// ```
   25|       |/// use core::convert::Infallible;
   26|       |/// use core::num::NonZeroUsize;
   27|       |///
   28|       |/// use qubit_codec::Codec;
   29|       |/// use qubit_codec::CodecValueDecoder;
   30|       |/// use qubit_codec::DecodeFailure;
   31|       |/// use qubit_codec::DecodeLifecycleOutput;
   32|       |///
   33|       |/// struct Identity;
   34|       |///
   35|       |/// impl Codec for Identity {
   36|       |///     type Value = u8;
   37|       |///     type Unit = u8;
   38|       |///     type DecodeError = Infallible;
   39|       |///     type EncodeError = Infallible;
   40|       |///     const MIN_UNITS_PER_VALUE: usize = 1;
   41|       |///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
   42|       |///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
   43|       |///
   44|       |///     unsafe fn decode(&mut self, input: &[u8], index: usize)
   45|       |///         -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>>
   46|       |///     {
   47|       |///         Ok((input[index], NonZeroUsize::MIN))
   48|       |///     }
   49|       |///
   50|       |///     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize)
   51|       |///         -> Result<usize, Infallible>
   52|       |///     {
   53|       |///         output[index] = *value;
   54|       |///         Ok(1)
   55|       |///     }
   56|       |/// }
   57|       |///
   58|       |/// let output: DecodeLifecycleOutput<u8> =
   59|       |///     CodecValueDecoder::new(Identity).decode_lifecycle(&[7]).unwrap();
   60|       |/// assert_eq!(*output.value(), 7);
   61|       |/// assert!(output.reset().is_empty());
   62|       |/// assert!(output.finish().is_empty());
   63|       |///
   64|       |/// let (reset, value, finish) = output.into_parts();
   65|       |/// assert_eq!(value, 7);
   66|       |/// assert!(reset.is_empty() && finish.is_empty());
   67|       |/// ```
   68|       |#[derive(Clone, Debug, Eq, Hash, PartialEq)]
   69|       |#[must_use]
   70|       |pub struct DecodeLifecycleOutput<V> {
   71|       |    /// Values emitted while resetting decode state.
   72|       |    reset: Vec<V>,
   73|       |    /// Main value decoded from the supplied input.
   74|       |    value: V,
   75|       |    /// Values emitted while finishing decode state.
   76|       |    finish: Vec<V>,
   77|       |}
   78|       |
   79|       |impl<V> DecodeLifecycleOutput<V> {
   80|       |    /// Creates owned output from the three decode lifecycle phases.
   81|       |    ///
   82|       |    /// # Parameters
   83|       |    ///
   84|       |    /// - `reset`: Values emitted while resetting decode state.
   85|       |    /// - `value`: Main value decoded from the supplied input.
   86|       |    /// - `finish`: Values emitted while finishing decode state.
   87|       |    ///
   88|       |    /// # Returns
   89|       |    ///
   90|       |    /// Returns output preserving all three lifecycle phases.
   91|       |    #[inline]
   92|     15|    pub(crate) fn new(reset: Vec<V>, value: V, finish: Vec<V>) -> Self {
   93|     15|        Self { reset, value, finish }
   94|     15|    }
   95|       |
   96|       |    /// Returns values emitted while resetting decode state.
   97|       |    ///
   98|       |    /// # Returns
   99|       |    ///
  100|       |    /// Returns the reset output in emission order.
  101|       |    #[inline]
  102|       |    #[must_use]
  103|      3|    pub fn reset(&self) -> &[V] {
  104|      3|        &self.reset
  105|      3|    }
  106|       |
  107|       |    /// Returns the main value decoded from the supplied input.
  108|       |    ///
  109|       |    /// # Returns
  110|       |    ///
  111|       |    /// Returns a shared reference to the main decoded value.
  112|       |    #[inline]
  113|       |    #[must_use]
  114|     15|    pub const fn value(&self) -> &V {
  115|     15|        &self.value
  116|     15|    }
  117|       |
  118|       |    /// Returns values emitted while finishing decode state.
  119|       |    ///
  120|       |    /// # Returns
  121|       |    ///
  122|       |    /// Returns the finish output in emission order.
  123|       |    #[inline]
  124|       |    #[must_use]
  125|      5|    pub fn finish(&self) -> &[V] {
  126|      5|        &self.finish
  127|      5|    }
  128|       |
  129|       |    /// Consumes this result and returns all lifecycle phases.
  130|       |    ///
  131|       |    /// # Returns
  132|       |    ///
  133|       |    /// Returns reset values, the main decoded value, and finish values.
  134|       |    #[inline]
  135|       |    #[must_use]
  136|      2|    pub fn into_parts(self) -> (Vec<V>, V, Vec<V>) {
  137|      2|        (self.reset, self.value, self.finish)
  138|      2|    }
  139|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/decode_lifecycle_progress.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Progress from a scratch-backed single-value decode lifecycle.
    9|       |
   10|       |/// Main decoded value and lifecycle output lengths written to caller storage.
   11|       |///
   12|       |/// # Type Parameters
   13|       |///
   14|       |/// - `V`: Logical value type produced by the codec.
   15|       |///
   16|       |/// # Examples
   17|       |///
   18|       |/// ```
   19|       |/// use core::convert::Infallible;
   20|       |/// use core::num::NonZeroUsize;
   21|       |///
   22|       |/// use qubit_codec::Codec;
   23|       |/// use qubit_codec::CodecValueDecoder;
   24|       |/// use qubit_codec::DecodeFailure;
   25|       |/// use qubit_codec::DecodeLifecycleProgress;
   26|       |///
   27|       |/// struct ByteCodec;
   28|       |/// impl Codec for ByteCodec {
   29|       |///     type Value = u8;
   30|       |///     type Unit = u8;
   31|       |///     type DecodeError = Infallible;
   32|       |///     type EncodeError = Infallible;
   33|       |///     const MIN_UNITS_PER_VALUE: usize = 1;
   34|       |///     const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
   35|       |///     const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
   36|       |///
   37|       |///     unsafe fn decode(&mut self, input: &[u8], index: usize)
   38|       |///         -> Result<(u8, NonZeroUsize), DecodeFailure<Infallible>>
   39|       |///     {
   40|       |///         Ok((input[index], NonZeroUsize::MIN))
   41|       |///     }
   42|       |///
   43|       |///     unsafe fn encode(&mut self, value: &u8, output: &mut [u8], index: usize)
   44|       |///         -> Result<usize, Infallible>
   45|       |///     {
   46|       |///         output[index] = *value;
   47|       |///         Ok(1)
   48|       |///     }
   49|       |/// }
   50|       |///
   51|       |/// let mut decoder = CodecValueDecoder::new(ByteCodec);
   52|       |/// let progress: DecodeLifecycleProgress<u8> = decoder
   53|       |///     .decode_lifecycle_with_scratch(&[42], &mut [], &mut [])
   54|       |///     .unwrap();
   55|       |/// assert_eq!(*progress.value(), 42);
   56|       |/// assert_eq!(progress.reset_written(), 0);
   57|       |/// assert_eq!(progress.finish_written(), 0);
   58|       |/// ```
   59|       |#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
   60|       |#[must_use]
   61|       |pub struct DecodeLifecycleProgress<V> {
   62|       |    /// Main value decoded from the supplied input.
   63|       |    value: V,
   64|       |    /// Number of reset values written to caller storage.
   65|       |    reset_written: usize,
   66|       |    /// Number of finish values written to caller storage.
   67|       |    finish_written: usize,
   68|       |}
   69|       |
   70|       |impl<V> DecodeLifecycleProgress<V> {
   71|       |    /// Creates progress for a completed decode lifecycle.
   72|       |    ///
   73|       |    /// # Parameters
   74|       |    ///
   75|       |    /// - `value`: Main value decoded from the supplied input.
   76|       |    /// - `reset_written`: Number of reset values written to caller storage.
   77|       |    /// - `finish_written`: Number of finish values written to caller storage.
   78|       |    ///
   79|       |    /// # Returns
   80|       |    ///
   81|       |    /// Returns completed lifecycle progress.
   82|       |    #[inline]
   83|     45|    pub(crate) const fn new(value: V, reset_written: usize, finish_written: usize) -> Self {
   84|     45|        Self {
   85|     45|            value,
   86|     45|            reset_written,
   87|     45|            finish_written,
   88|     45|        }
   89|     45|    }
   90|       |
   91|       |    /// Returns the main value decoded from the supplied input.
   92|       |    ///
   93|       |    /// # Returns
   94|       |    ///
   95|       |    /// Returns a shared reference to the main decoded value.
   96|       |    #[inline]
   97|       |    #[must_use]
   98|      6|    pub const fn value(&self) -> &V {
   99|      6|        &self.value
  100|      6|    }
  101|       |
  102|       |    /// Returns the number of reset values written to caller storage.
  103|       |    ///
  104|       |    /// # Returns
  105|       |    ///
  106|       |    /// Returns the initialized reset output length.
  107|       |    #[inline]
  108|       |    #[must_use]
  109|      2|    pub const fn reset_written(&self) -> usize {
  110|      2|        self.reset_written
  111|      2|    }
  112|       |
  113|       |    /// Returns the number of finish values written to caller storage.
  114|       |    ///
  115|       |    /// # Returns
  116|       |    ///
  117|       |    /// Returns the initialized finish output length.
  118|       |    #[inline]
  119|       |    #[must_use]
  120|      4|    pub const fn finish_written(&self) -> usize {
  121|      4|        self.finish_written
  122|      4|    }
  123|       |
  124|       |    /// Consumes this progress and returns its components.
  125|       |    ///
  126|       |    /// # Returns
  127|       |    ///
  128|       |    /// Returns the main value, reset output length, and finish output length.
  129|       |    #[inline]
  130|       |    #[must_use]
  131|     40|    pub fn into_parts(self) -> (V, usize, usize) {
  132|     40|        (self.value, self.reset_written, self.finish_written)
  133|     40|    }
  134|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/json_bytes_value_codec.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Bidirectional Serde JSON bytes codec for one owned value.
    9|       |
   10|       |use core::marker::PhantomData;
   11|       |
   12|       |use serde::Serialize;
   13|       |use serde::de::DeserializeOwned;
   14|       |use serde_json::Error as JsonError;
   15|       |
   16|       |use super::JsonBytesValueDecoder;
   17|       |use super::JsonBytesValueEncoder;
   18|       |use super::ValueDecoder;
   19|       |use super::ValueEncoder;
   20|       |
   21|       |/// Encodes and decodes one serializable value through JSON bytes.
   22|       |///
   23|       |/// This type combines [`JsonBytesValueEncoder`] and [`JsonBytesValueDecoder`]
   24|       |/// for registry-backed or bidirectional call sites. It holds no state between
   25|       |/// calls, so it is `Copy` and `Default` without requiring those traits on `T`;
   26|       |/// encoding and decoding can share one value through [`ValueEncoder`] and
   27|       |/// [`ValueDecoder`].
   28|       |///
   29|       |/// # Type Parameters
   30|       |///
   31|       |/// - `T`: Value type serialized to JSON bytes and recovered from them.
   32|       |///
   33|       |/// # Examples
   34|       |///
   35|       |/// ```
   36|       |/// use qubit_codec::JsonBytesValueCodec;
   37|       |/// use qubit_codec::ValueDecoder;
   38|       |/// use qubit_codec::ValueEncoder;
   39|       |///
   40|       |/// let mut codec = JsonBytesValueCodec::<String>::new();
   41|       |/// assert_eq!(codec.encode(&"hello".to_string()).unwrap(), br#""hello""#.to_vec());
   42|       |/// assert_eq!(codec.decode(br#""hello""#).unwrap(), "hello");
   43|       |/// ```
   44|       |#[must_use]
   45|       |#[derive(Debug)]
   46|       |pub struct JsonBytesValueCodec<T> {
   47|       |    /// Associates the codec with `T` without retaining a value.
   48|       |    _marker: PhantomData<T>,
   49|       |}
   50|       |
   51|       |impl<T> Copy for JsonBytesValueCodec<T> {}
   52|       |
   53|       |impl<T> Clone for JsonBytesValueCodec<T> {
   54|       |    /// Returns an identical stateless codec without requiring `T: Clone`.
   55|       |    ///
   56|       |    /// # Returns
   57|       |    ///
   58|       |    /// Returns a codec associated with the same value type.
   59|       |    #[inline]
   60|      0|    fn clone(&self) -> Self {
   61|      0|        *self
   62|      0|    }
   63|       |}
   64|       |
   65|       |impl<T> Default for JsonBytesValueCodec<T> {
   66|       |    /// Creates a codec for `T` without constructing a value of that type.
   67|       |    ///
   68|       |    /// # Returns
   69|       |    ///
   70|       |    /// Returns a stateless codec associated with `T`.
   71|       |    #[inline]
   72|      6|    fn default() -> Self {
   73|      6|        Self::new()
   74|      6|    }
   75|       |}
   76|       |
   77|       |impl<T> JsonBytesValueCodec<T> {
   78|       |    /// Creates a bidirectional JSON bytes codec for values of type `T`.
   79|       |    ///
   80|       |    /// # Returns
   81|       |    ///
   82|       |    /// Returns a stateless codec bound to values of type `T`.
   83|       |    #[inline]
   84|     19|    pub const fn new() -> Self {
   85|     19|        Self { _marker: PhantomData }
   86|     19|    }
   87|       |}
   88|       |
   89|       |impl<T> ValueEncoder<T> for JsonBytesValueCodec<T>
   90|       |where
   91|       |    T: Serialize,
   92|       |{
   93|       |    /// Serialization failure reported by Serde JSON.
   94|       |    type Error = JsonError;
   95|       |    /// Newly allocated UTF-8 JSON bytes owned by the caller.
   96|       |    type Output = Vec<u8>;
   97|       |
   98|       |    /// Serializes `input` into UTF-8 JSON bytes.
   99|       |    ///
  100|       |    /// # Parameters
  101|       |    ///
  102|       |    /// - `input`: Value borrowed for the duration of serialization.
  103|       |    ///
  104|       |    /// # Returns
  105|       |    ///
  106|       |    /// Returns a newly allocated vector containing the serialized JSON.
  107|       |    ///
  108|       |    /// # Errors
  109|       |    ///
  110|       |    /// Returns a JSON error if `T` rejects serialization or contains a map key
  111|       |    /// that cannot be represented as a JSON object key.
  112|       |    #[inline]
  113|     10|    fn encode(&mut self, input: &T) -> Result<Self::Output, Self::Error> {
  114|     10|        JsonBytesValueEncoder::<T>::new().encode(input)
  115|     10|    }
  116|       |}
  117|       |
  118|       |impl<T> ValueDecoder<[u8]> for JsonBytesValueCodec<T>
  119|       |where
  120|       |    T: DeserializeOwned,
  121|       |{
  122|       |    /// JSON syntax or target-value deserialization failure.
  123|       |    type Error = JsonError;
  124|       |    /// Owned value deserialized independently of the input byte slice.
  125|       |    type Output = T;
  126|       |
  127|       |    /// Deserializes `input` using [`serde_json::from_slice`].
  128|       |    ///
  129|       |    /// # Parameters
  130|       |    ///
  131|       |    /// - `input`: One complete JSON value, optionally surrounded by whitespace.
  132|       |    ///   String-content validation follows `T`'s Serde deserialization
  133|       |    ///   implementation.
  134|       |    ///
  135|       |    /// # Returns
  136|       |    ///
  137|       |    /// Returns an owned `T`; decoding may allocate according to the target
  138|       |    /// type.
  139|       |    ///
  140|       |    /// # Errors
  141|       |    ///
  142|       |    /// Returns a JSON error for malformed or incomplete input, trailing
  143|       |    /// non-whitespace data, or a value incompatible with `T`. Validation
  144|       |    /// follows serde_json and `T`'s `Deserialize` implementation; byte-oriented
  145|       |    /// targets may accept non-UTF-8 string contents.
  146|       |    #[inline]
  147|     10|    fn decode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
  148|     10|        JsonBytesValueDecoder::<T>::new().decode(input)
  149|     10|    }
  150|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/json_bytes_value_decoder.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Serde JSON bytes decoder for one owned value.
    9|       |
   10|       |use core::marker::PhantomData;
   11|       |
   12|       |use serde::de::DeserializeOwned;
   13|       |use serde_json::Error as JsonError;
   14|       |use serde_json::from_slice;
   15|       |
   16|       |use super::ValueDecoder;
   17|       |
   18|       |/// Decodes JSON bytes into one owned value via [`serde_json`].
   19|       |/// Its stateless marker implements `Clone`, `Copy`, and `Default` without
   20|       |/// requiring those traits on `T`.
   21|       |///
   22|       |/// # Type Parameters
   23|       |///
   24|       |/// - `T`: Target value type deserialized through Serde.
   25|       |///
   26|       |/// # Examples
   27|       |///
   28|       |/// ```rust
   29|       |/// use qubit_codec::{JsonBytesValueDecoder, ValueDecoder};
   30|       |/// use serde::Deserialize;
   31|       |///
   32|       |/// #[derive(Debug, Deserialize, PartialEq)]
   33|       |/// struct Point {
   34|       |///     x: i32,
   35|       |///     y: i32,
   36|       |/// }
   37|       |///
   38|       |/// let mut decoder = JsonBytesValueDecoder::<Point>::new();
   39|       |/// let point = decoder.decode(br#"{"x":1,"y":2}"#).unwrap();
   40|       |/// assert_eq!(point, Point { x: 1, y: 2 });
   41|       |/// ```
   42|       |#[must_use]
   43|       |#[derive(Debug)]
   44|       |pub struct JsonBytesValueDecoder<T> {
   45|       |    /// Associates the decoder with `T` without retaining a value.
   46|       |    _marker: PhantomData<T>,
   47|       |}
   48|       |
   49|       |impl<T> Copy for JsonBytesValueDecoder<T> {}
   50|       |
   51|       |impl<T> Clone for JsonBytesValueDecoder<T> {
   52|       |    /// Returns an identical stateless decoder without requiring `T: Clone`.
   53|       |    ///
   54|       |    /// # Returns
   55|       |    ///
   56|       |    /// Returns a decoder associated with the same target type.
   57|       |    #[inline]
   58|      0|    fn clone(&self) -> Self {
   59|      0|        *self
   60|      0|    }
   61|       |}
   62|       |
   63|       |impl<T> Default for JsonBytesValueDecoder<T> {
   64|       |    /// Creates a decoder for `T` without constructing a value of that type.
   65|       |    ///
   66|       |    /// # Returns
   67|       |    ///
   68|       |    /// Returns an uninitialized stateless decoder associated with `T`.
   69|       |    #[inline]
   70|      1|    fn default() -> Self {
   71|      1|        Self::new()
   72|      1|    }
   73|       |}
   74|       |
   75|       |impl<T> JsonBytesValueDecoder<T> {
   76|       |    /// Creates a JSON bytes decoder for values of type `T`.
   77|       |    ///
   78|       |    /// # Returns
   79|       |    ///
   80|       |    /// Returns a stateless decoder without allocating or constructing a `T`.
   81|       |    #[inline]
   82|     18|    pub const fn new() -> Self {
   83|     18|        Self { _marker: PhantomData }
   84|     18|    }
   85|       |}
   86|       |
   87|       |impl<T> ValueDecoder<[u8]> for JsonBytesValueDecoder<T>
   88|       |where
   89|       |    T: DeserializeOwned,
   90|       |{
   91|       |    /// JSON syntax or target-value deserialization failure.
   92|       |    type Error = JsonError;
   93|       |    /// Owned value deserialized independently of the input byte slice.
   94|       |    type Output = T;
   95|       |
   96|       |    /// Deserializes `input` using [`serde_json::from_slice`].
   97|       |    ///
   98|       |    /// # Parameters
   99|       |    ///
  100|       |    /// - `input`: One complete JSON value, optionally surrounded by whitespace.
  101|       |    ///   String-content validation follows `T`'s Serde deserialization
  102|       |    ///   implementation.
  103|       |    ///
  104|       |    /// # Returns
  105|       |    ///
  106|       |    /// Returns an owned `T`; decoding may allocate according to the target
  107|       |    /// type.
  108|       |    ///
  109|       |    /// # Errors
  110|       |    ///
  111|       |    /// Returns a JSON error for malformed or incomplete input, trailing
  112|       |    /// non-whitespace data, or a value incompatible with `T`. Validation
  113|       |    /// follows serde_json and `T`'s `Deserialize` implementation; byte-oriented
  114|       |    /// targets may accept non-UTF-8 string contents.
  115|       |    #[inline]
  116|     18|    fn decode(&mut self, input: &[u8]) -> Result<Self::Output, Self::Error> {
  117|     18|        from_slice(input)
  118|     18|    }
  119|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/json_bytes_value_encoder.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Serde JSON bytes encoder for one owned value.
    9|       |
   10|       |use core::marker::PhantomData;
   11|       |
   12|       |use serde::Serialize;
   13|       |use serde_json::Error as JsonError;
   14|       |use serde_json::to_vec;
   15|       |
   16|       |use super::ValueEncoder;
   17|       |
   18|       |/// Encodes one serializable value into UTF-8 JSON bytes via `serde_json`.
   19|       |/// Its stateless marker implements `Clone`, `Copy`, and `Default` without
   20|       |/// requiring those traits on `T`.
   21|       |///
   22|       |/// # Type Parameters
   23|       |///
   24|       |/// - `T`: Source value type serialized through Serde.
   25|       |///
   26|       |/// # Examples
   27|       |///
   28|       |/// ```rust
   29|       |/// use qubit_codec::JsonBytesValueEncoder;
   30|       |/// use qubit_codec::ValueEncoder;
   31|       |/// use serde::Serialize;
   32|       |///
   33|       |/// #[derive(Serialize)]
   34|       |/// struct Point {
   35|       |///     x: i32,
   36|       |///     y: i32,
   37|       |/// }
   38|       |///
   39|       |/// let mut encoder = JsonBytesValueEncoder::<Point>::new();
   40|       |/// let json = encoder.encode(&Point { x: 1, y: 2 }).unwrap();
   41|       |/// assert_eq!(json, br#"{"x":1,"y":2}"#);
   42|       |/// ```
   43|       |#[must_use]
   44|       |#[derive(Debug)]
   45|       |pub struct JsonBytesValueEncoder<T> {
   46|       |    /// Associates the encoder with `T` without retaining a source value.
   47|       |    _marker: PhantomData<T>,
   48|       |}
   49|       |
   50|       |impl<T> Copy for JsonBytesValueEncoder<T> {}
   51|       |
   52|       |impl<T> Clone for JsonBytesValueEncoder<T> {
   53|       |    #[inline]
   54|      0|    fn clone(&self) -> Self {
   55|      0|        *self
   56|      0|    }
   57|       |}
   58|       |
   59|       |impl<T> Default for JsonBytesValueEncoder<T> {
   60|       |    #[inline]
   61|      2|    fn default() -> Self {
   62|      2|        Self::new()
   63|      2|    }
   64|       |}
   65|       |
   66|       |impl<T> JsonBytesValueEncoder<T> {
   67|       |    /// Creates a JSON bytes encoder for values of type `T`.
   68|       |    ///
   69|       |    /// # Returns
   70|       |    ///
   71|       |    /// Returns a stateless encoder without allocating an output buffer.
   72|       |    #[inline]
   73|     19|    pub const fn new() -> Self {
   74|     19|        Self { _marker: PhantomData }
   75|     19|    }
   76|       |}
   77|       |
   78|       |impl<T> ValueEncoder<T> for JsonBytesValueEncoder<T>
   79|       |where
   80|       |    T: Serialize,
   81|       |{
   82|       |    /// Serialization failure reported by Serde JSON.
   83|       |    type Error = JsonError;
   84|       |    /// Newly allocated UTF-8 JSON bytes owned by the caller.
   85|       |    type Output = Vec<u8>;
   86|       |
   87|       |    /// Serializes `input` into UTF-8 JSON bytes.
   88|       |    ///
   89|       |    /// # Parameters
   90|       |    ///
   91|       |    /// - `input`: Value borrowed for the duration of serialization.
   92|       |    ///
   93|       |    /// # Returns
   94|       |    ///
   95|       |    /// Returns a newly allocated vector containing the serialized JSON.
   96|       |    ///
   97|       |    /// # Errors
   98|       |    ///
   99|       |    /// Returns a JSON error if `T` rejects serialization or contains a map key
  100|       |    /// that cannot be represented as a JSON object key.
  101|       |    #[inline]
  102|     19|    fn encode(&mut self, input: &T) -> Result<Self::Output, Self::Error> {
  103|     19|        to_vec(input)
  104|     19|    }
  105|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/json_string_value_codec.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Bidirectional Serde JSON string codec for one owned value.
    9|       |
   10|       |use core::marker::PhantomData;
   11|       |
   12|       |use serde::Serialize;
   13|       |use serde::de::DeserializeOwned;
   14|       |
   15|       |use super::JsonStringValueDecoder;
   16|       |use super::JsonStringValueEncoder;
   17|       |use super::ValueDecoder;
   18|       |use super::ValueEncoder;
   19|       |
   20|       |/// Encodes and decodes one serializable value through UTF-8 JSON strings.
   21|       |///
   22|       |/// This type combines [`JsonStringValueEncoder`] and [`JsonStringValueDecoder`]
   23|       |/// for registry-backed or bidirectional call sites. It holds no state between
   24|       |/// calls, so it is `Copy` and `Default` without requiring those traits on `T`;
   25|       |/// encoding and decoding can share one value through [`ValueEncoder`] and
   26|       |/// [`ValueDecoder`].
   27|       |///
   28|       |/// # Type Parameters
   29|       |///
   30|       |/// - `T`: Value type serialized to a JSON string and recovered from it.
   31|       |///
   32|       |/// # Examples
   33|       |///
   34|       |/// ```
   35|       |/// use qubit_codec::JsonStringValueCodec;
   36|       |/// use qubit_codec::ValueDecoder;
   37|       |/// use qubit_codec::ValueEncoder;
   38|       |///
   39|       |/// let mut codec = JsonStringValueCodec::<String>::new();
   40|       |/// assert_eq!(codec.encode(&"hello".to_string()).unwrap(), r#""hello""#.to_string());
   41|       |/// assert_eq!(codec.decode(r#""hello""#).unwrap(), "hello");
   42|       |/// ```
   43|       |#[must_use]
   44|       |#[derive(Debug)]
   45|       |pub struct JsonStringValueCodec<T> {
   46|       |    /// Associates the codec with `T` without retaining a value.
   47|       |    _marker: PhantomData<T>,
   48|       |}
   49|       |
   50|       |impl<T> Copy for JsonStringValueCodec<T> {}
   51|       |
   52|       |impl<T> Clone for JsonStringValueCodec<T> {
   53|       |    /// Copies the stateless marker without requiring `T: Clone`.
   54|       |    ///
   55|       |    /// # Returns
   56|       |    ///
   57|       |    /// Returns another marker bound to the same value type.
   58|       |    #[inline]
   59|      0|    fn clone(&self) -> Self {
   60|      0|        *self
   61|      0|    }
   62|       |}
   63|       |
   64|       |impl<T> Default for JsonStringValueCodec<T> {
   65|       |    /// Creates the stateless marker without requiring `T: Default`.
   66|       |    ///
   67|       |    /// # Returns
   68|       |    ///
   69|       |    /// Returns a codec bound to the value type `T`.
   70|       |    #[inline]
   71|      6|    fn default() -> Self {
   72|      6|        Self::new()
   73|      6|    }
   74|       |}
   75|       |
   76|       |impl<T> JsonStringValueCodec<T> {
   77|       |    /// Creates a bidirectional JSON string codec for values of type `T`.
   78|       |    ///
   79|       |    /// # Returns
   80|       |    ///
   81|       |    /// Returns a stateless codec bound to values of type `T`.
   82|       |    #[inline]
   83|     18|    pub const fn new() -> Self {
   84|     18|        Self { _marker: PhantomData }
   85|     18|    }
   86|       |}
   87|       |
   88|       |impl<T> ValueEncoder<T> for JsonStringValueCodec<T>
   89|       |where
   90|       |    T: Serialize,
   91|       |{
   92|       |    /// Serialization failure reported by Serde JSON.
   93|       |    type Error = serde_json::Error;
   94|       |    /// Newly allocated UTF-8 JSON string owned by the caller.
   95|       |    type Output = String;
   96|       |
   97|       |    /// Serializes `input` into a JSON string.
   98|       |    ///
   99|       |    /// The codec holds no state, so this delegates to a freshly created
  100|       |    /// [`JsonStringValueEncoder`] and `self` stays usable for further calls.
  101|       |    ///
  102|       |    /// # Parameters
  103|       |    ///
  104|       |    /// - `input`: Value to serialize as JSON.
  105|       |    ///
  106|       |    /// # Returns
  107|       |    ///
  108|       |    /// Returns the serialized JSON text as an owned UTF-8 string.
  109|       |    ///
  110|       |    /// # Errors
  111|       |    ///
  112|       |    /// Returns [`serde_json::Error`] when `input` cannot be serialized to
  113|       |    /// JSON, including when a custom `Serialize` implementation fails or a map
  114|       |    /// key cannot be represented as a JSON object key.
  115|       |    #[inline]
  116|     10|    fn encode(&mut self, input: &T) -> Result<Self::Output, Self::Error> {
  117|     10|        JsonStringValueEncoder::<T>::new().encode(input)
  118|     10|    }
  119|       |}
  120|       |
  121|       |impl<T> ValueDecoder<str> for JsonStringValueCodec<T>
  122|       |where
  123|       |    T: DeserializeOwned,
  124|       |{
  125|       |    /// JSON syntax or target-value deserialization failure.
  126|       |    type Error = serde_json::Error;
  127|       |    /// Owned value deserialized independently of the input string lifetime.
  128|       |    type Output = T;
  129|       |
  130|       |    /// Deserializes `input` from a JSON string.
  131|       |    ///
  132|       |    /// The codec holds no state, so this delegates to a freshly created
  133|       |    /// [`JsonStringValueDecoder`] and `self` stays usable for further calls.
  134|       |    ///
  135|       |    /// # Parameters
  136|       |    ///
  137|       |    /// - `input`: Complete JSON value to deserialize.
  138|       |    ///
  139|       |    /// # Returns
  140|       |    ///
  141|       |    /// Returns an owned `T` deserialized independently of `input`'s lifetime.
  142|       |    ///
  143|       |    /// # Errors
  144|       |    ///
  145|       |    /// Returns [`serde_json::Error`] when `input` is not valid JSON or when a
  146|       |    /// valid JSON document does not match the shape required by `T`.
  147|       |    #[inline]
  148|      9|    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
  149|      9|        JsonStringValueDecoder::<T>::new().decode(input)
  150|      9|    }
  151|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/json_string_value_decoder.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Serde JSON string decoder for one owned value.
    9|       |
   10|       |use core::marker::PhantomData;
   11|       |
   12|       |use serde::de::DeserializeOwned;
   13|       |use serde_json::Error as JsonError;
   14|       |use serde_json::from_str;
   15|       |
   16|       |use super::ValueDecoder;
   17|       |
   18|       |/// Decodes one JSON string into an owned value via `serde_json`.
   19|       |/// Its stateless marker implements `Clone`, `Copy`, and `Default` without
   20|       |/// requiring those traits on `T`.
   21|       |///
   22|       |/// # Type Parameters
   23|       |///
   24|       |/// - `T`: Target value type deserialized through Serde.
   25|       |///
   26|       |/// # Examples
   27|       |///
   28|       |/// ```rust
   29|       |/// use qubit_codec::JsonStringValueDecoder;
   30|       |/// use qubit_codec::ValueDecoder;
   31|       |/// use serde::Deserialize;
   32|       |///
   33|       |/// #[derive(Debug, Deserialize, PartialEq)]
   34|       |/// struct Point {
   35|       |///     x: i32,
   36|       |///     y: i32,
   37|       |/// }
   38|       |///
   39|       |/// let mut decoder = JsonStringValueDecoder::<Point>::new();
   40|       |/// let point = decoder.decode(r#"{"x":1,"y":2}"#).unwrap();
   41|       |/// assert_eq!(point, Point { x: 1, y: 2 });
   42|       |/// ```
   43|       |#[must_use]
   44|       |#[derive(Debug)]
   45|       |pub struct JsonStringValueDecoder<T> {
   46|       |    /// Tracks the owned output type without storing a value between calls.
   47|       |    _marker: PhantomData<T>,
   48|       |}
   49|       |
   50|       |impl<T> Copy for JsonStringValueDecoder<T> {}
   51|       |
   52|       |impl<T> Clone for JsonStringValueDecoder<T> {
   53|       |    /// Copies the stateless decoder marker without requiring `T: Clone`.
   54|       |    #[inline]
   55|      0|    fn clone(&self) -> Self {
   56|      0|        *self
   57|      0|    }
   58|       |}
   59|       |
   60|       |impl<T> Default for JsonStringValueDecoder<T> {
   61|       |    /// Creates the stateless decoder through its const constructor.
   62|       |    #[inline]
   63|      1|    fn default() -> Self {
   64|      1|        Self::new()
   65|      1|    }
   66|       |}
   67|       |
   68|       |impl<T> JsonStringValueDecoder<T> {
   69|       |    /// Creates a JSON string decoder for values of type `T`.
   70|       |    ///
   71|       |    /// # Returns
   72|       |    ///
   73|       |    /// Returns a stateless decoder without allocating or constructing a `T`.
   74|       |    #[inline]
   75|     14|    pub const fn new() -> Self {
   76|     14|        Self { _marker: PhantomData }
   77|     14|    }
   78|       |}
   79|       |
   80|       |impl<T> ValueDecoder<str> for JsonStringValueDecoder<T>
   81|       |where
   82|       |    T: DeserializeOwned,
   83|       |{
   84|       |    /// JSON syntax or target-value deserialization failure.
   85|       |    type Error = JsonError;
   86|       |    /// Owned value deserialized independently of the input string lifetime.
   87|       |    type Output = T;
   88|       |
   89|       |    /// Deserializes `input` from a JSON string.
   90|       |    ///
   91|       |    /// # Parameters
   92|       |    ///
   93|       |    /// - `input`: One complete JSON value, optionally surrounded by whitespace.
   94|       |    ///
   95|       |    /// # Returns
   96|       |    ///
   97|       |    /// Returns an owned `T`; decoding may allocate according to the target
   98|       |    /// type.
   99|       |    ///
  100|       |    /// # Errors
  101|       |    ///
  102|       |    /// Returns a JSON error for malformed or incomplete input, trailing
  103|       |    /// non-whitespace data, or a value incompatible with `T`.
  104|       |    #[inline]
  105|     14|    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
  106|     14|        from_str(input)
  107|     14|    }
  108|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/json_string_value_encoder.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |//! Serde JSON string encoder for one owned value.
    9|       |
   10|       |use core::marker::PhantomData;
   11|       |
   12|       |use serde::Serialize;
   13|       |
   14|       |use super::ValueEncoder;
   15|       |
   16|       |/// Encodes one serializable value into a UTF-8 JSON string via `serde_json`.
   17|       |/// Its stateless marker implements `Clone`, `Copy`, and `Default` without
   18|       |/// requiring those traits on `T`.
   19|       |///
   20|       |/// # Type Parameters
   21|       |///
   22|       |/// - `T`: Source value type serialized through Serde.
   23|       |///
   24|       |/// # Examples
   25|       |///
   26|       |/// ```rust
   27|       |/// use qubit_codec::{JsonStringValueEncoder, ValueEncoder};
   28|       |/// use serde::Serialize;
   29|       |///
   30|       |/// #[derive(Serialize)]
   31|       |/// struct Point {
   32|       |///     x: i32,
   33|       |///     y: i32,
   34|       |/// }
   35|       |///
   36|       |/// let mut encoder = JsonStringValueEncoder::<Point>::new();
   37|       |/// let json = encoder.encode(&Point { x: 1, y: 2 }).unwrap();
   38|       |/// assert_eq!(json, r#"{"x":1,"y":2}"#);
   39|       |/// ```
   40|       |#[must_use]
   41|       |#[derive(Debug)]
   42|       |pub struct JsonStringValueEncoder<T> {
   43|       |    /// Associates the encoder with `T` without retaining a source value.
   44|       |    _marker: PhantomData<T>,
   45|       |}
   46|       |
   47|       |impl<T> Copy for JsonStringValueEncoder<T> {}
   48|       |
   49|       |impl<T> Clone for JsonStringValueEncoder<T> {
   50|       |    /// Copies the stateless encoder marker without requiring `T: Clone`.
   51|       |    #[inline]
   52|      0|    fn clone(&self) -> Self {
   53|      0|        *self
   54|      0|    }
   55|       |}
   56|       |
   57|       |impl<T> Default for JsonStringValueEncoder<T> {
   58|       |    /// Creates the stateless encoder through its const constructor.
   59|       |    #[inline]
   60|      2|    fn default() -> Self {
   61|      2|        Self::new()
   62|      2|    }
   63|       |}
   64|       |
   65|       |impl<T> JsonStringValueEncoder<T> {
   66|       |    /// Creates a JSON string encoder for values of type `T`.
   67|       |    ///
   68|       |    /// # Returns
   69|       |    ///
   70|       |    /// Returns a stateless encoder without allocating an output string.
   71|       |    #[inline]
   72|     17|    pub const fn new() -> Self {
   73|     17|        Self { _marker: PhantomData }
   74|     17|    }
   75|       |}
   76|       |
   77|       |impl<T> ValueEncoder<T> for JsonStringValueEncoder<T>
   78|       |where
   79|       |    T: Serialize,
   80|       |{
   81|       |    type Error = serde_json::Error;
   82|       |    type Output = String;
   83|       |
   84|       |    /// Serializes `input` into a compact JSON string.
   85|       |    ///
   86|       |    /// # Parameters
   87|       |    /// - `input`: Value to serialize.
   88|       |    ///
   89|       |    /// # Returns
   90|       |    /// The serialized JSON text as an owned UTF-8 string.
   91|       |    ///
   92|       |    /// # Errors
   93|       |    /// Returns the `serde_json` error if serialization fails, such as when a
   94|       |    /// custom serializer reports an error or the value contains unsupported
   95|       |    /// data.
   96|       |    #[inline]
   97|     17|    fn encode(&mut self, input: &T) -> Result<Self::Output, Self::Error> {
   98|     17|        serde_json::to_string(input)
   99|     17|    }
  100|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/value_bytes_codec_descriptor.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |
    9|       |//! Safe type-erased bidirectional bytes value-codec descriptors.
   10|       |
   11|       |use core::any::type_name;
   12|       |use core::fmt::Debug;
   13|       |use core::fmt::Formatter;
   14|       |use core::fmt::Result as FmtResult;
   15|       |use std::any::Any;
   16|       |use std::any::TypeId;
   17|       |use std::error::Error;
   18|       |
   19|       |use crate::ValueCodecExecutionError;
   20|       |use crate::ValueDecoder;
   21|       |use crate::ValueEncoder;
   22|       |
   23|       |/// Checked type-erased encoding entry point returning caller-owned bytes.
   24|       |type EncodeFn = fn(&dyn Any) -> Result<Vec<u8>, ValueCodecExecutionError>;
   25|       |/// Type-erased decoding entry point returning a boxed owned value.
   26|       |type DecodeFn = fn(&[u8]) -> Result<Box<dyn Any>, ValueCodecExecutionError>;
   27|       |
   28|       |/// An immutable, safely erased bidirectional bytes codec for one value type.
   29|       |///
   30|       |/// Each operation creates a fresh default codec; no codec state is retained.
   31|       |/// Type identities are process-local and names are diagnostic, not stable IDs.
   32|       |///
   33|       |/// # Examples
   34|       |///
   35|       |/// ```
   36|       |/// use core::convert::Infallible;
   37|       |///
   38|       |/// use qubit_codec::ValueBytesCodecDescriptor;
   39|       |/// use qubit_codec::ValueDecoder;
   40|       |/// use qubit_codec::ValueEncoder;
   41|       |///
   42|       |/// #[derive(Default)]
   43|       |/// struct Bytes;
   44|       |/// impl ValueEncoder<Vec<u8>> for Bytes {
   45|       |///     type Error = Infallible;
   46|       |///     type Output = Vec<u8>;
   47|       |///     fn encode(&mut self, value: &Vec<u8>) -> Result<Vec<u8>, Infallible> {
   48|       |///         Ok(value.clone())
   49|       |///     }
   50|       |/// }
   51|       |/// impl ValueDecoder<[u8]> for Bytes {
   52|       |///     type Error = Infallible;
   53|       |///     type Output = Vec<u8>;
   54|       |///     fn decode(&mut self, input: &[u8]) -> Result<Vec<u8>, Infallible> {
   55|       |///         Ok(input.to_vec())
   56|       |///     }
   57|       |/// }
   58|       |/// let descriptor = ValueBytesCodecDescriptor::of::<Bytes, Vec<u8>>();
   59|       |/// let bytes = descriptor.encode(&vec![1_u8, 2]).unwrap();
   60|       |/// let decoded = descriptor.decode(&bytes).unwrap();
   61|       |/// assert_eq!(*decoded.downcast::<Vec<u8>>().unwrap(), vec![1, 2]);
   62|       |/// ```
   63|       |#[derive(Clone, Copy)]
   64|       |pub struct ValueBytesCodecDescriptor {
   65|       |    /// Returns the process-local identity of the concrete codec.
   66|       |    codec_type_id: fn() -> TypeId,
   67|       |    /// Returns the diagnostic name of the concrete codec.
   68|       |    codec_type_name: fn() -> &'static str,
   69|       |    /// Returns the process-local identity of the decoded value.
   70|       |    value_type_id: fn() -> TypeId,
   71|       |    /// Returns the diagnostic name of the decoded value.
   72|       |    value_type_name: fn() -> &'static str,
   73|       |    /// Checks the erased input type and invokes a fresh default encoder.
   74|       |    encode: EncodeFn,
   75|       |    /// Invokes a fresh default decoder and boxes its owned output.
   76|       |    decode: DecodeFn,
   77|       |}
   78|       |
   79|       |impl ValueBytesCodecDescriptor {
   80|       |    /// Creates a descriptor for codec `C` and value type `V`.
   81|       |    ///
   82|       |    /// Both codec error types must implement `Send + Sync` so execution errors
   83|       |    /// can retain their source across thread boundaries. The codec and value
   84|       |    /// types themselves need not implement these traits; `V` only needs to be
   85|       |    /// `'static` because decoded values are returned through `Any`.
   86|       |    ///
   87|       |    /// # Type Parameters
   88|       |    ///
   89|       |    /// - `C`: Default-constructible bytes encoder/decoder with owned errors.
   90|       |    /// - `V`: Owned value type accepted and produced by `C`.
   91|       |    ///
   92|       |    /// # Returns
   93|       |    ///
   94|       |    /// Returns immutable function metadata without constructing a codec.
   95|       |    #[must_use]
   96|       |    #[inline]
   97|      0|    pub const fn of<C, V>() -> Self
   98|      0|    where
   99|      0|        C: Default + ValueEncoder<V, Output = Vec<u8>> + ValueDecoder<[u8], Output = V> + 'static,
  100|      0|        <C as ValueEncoder<V>>::Error: Error + Send + Sync + 'static,
  101|      0|        <C as ValueDecoder<[u8]>>::Error: Error + Send + Sync + 'static,
  102|      0|        V: 'static,
  103|       |    {
  104|      0|        Self {
  105|      0|            codec_type_id: TypeId::of::<C>,
  106|      0|            codec_type_name: type_name::<C>,
  107|      0|            value_type_id: TypeId::of::<V>,
  108|      0|            value_type_name: type_name::<V>,
  109|      0|            encode: encode::<C, V>,
  110|      0|            decode: decode::<C, V>,
  111|      0|        }
  112|      0|    }
  113|       |
  114|       |    /// Returns the process-local codec type identity.
  115|       |    ///
  116|       |    /// # Returns
  117|       |    ///
  118|       |    /// Returns metadata for the type selected by [`Self::of`], without
  119|       |    /// allocation.
  120|       |    #[must_use]
  121|       |    #[inline]
  122|     12|    pub fn codec_type_id(&self) -> TypeId {
  123|     12|        (self.codec_type_id)()
  124|     12|    }
  125|       |
  126|       |    /// Returns the diagnostic codec type name.
  127|       |    ///
  128|       |    /// # Returns
  129|       |    ///
  130|       |    /// Returns metadata for the type selected by [`Self::of`], without
  131|       |    /// allocation.
  132|       |    #[must_use]
  133|       |    #[inline]
  134|      3|    pub fn codec_type_name(&self) -> &'static str {
  135|      3|        (self.codec_type_name)()
  136|      3|    }
  137|       |
  138|       |    /// Returns the process-local value type identity.
  139|       |    ///
  140|       |    /// # Returns
  141|       |    ///
  142|       |    /// Returns metadata for the type selected by [`Self::of`], without
  143|       |    /// allocation.
  144|       |    #[must_use]
  145|       |    #[inline]
  146|      3|    pub fn value_type_id(&self) -> TypeId {
  147|      3|        (self.value_type_id)()
  148|      3|    }
  149|       |
  150|       |    /// Returns the diagnostic value type name.
  151|       |    ///
  152|       |    /// # Returns
  153|       |    ///
  154|       |    /// Returns metadata for the type selected by [`Self::of`], without
  155|       |    /// allocation.
  156|       |    #[must_use]
  157|       |    #[inline]
  158|      3|    pub fn value_type_name(&self) -> &'static str {
  159|      3|        (self.value_type_name)()
  160|      3|    }
  161|       |
  162|       |    /// Encodes one erased value after checking its concrete type.
  163|       |    ///
  164|       |    /// # Parameters
  165|       |    ///
  166|       |    /// - `value`: Borrowed value whose concrete type must match the descriptor.
  167|       |    ///
  168|       |    /// # Returns
  169|       |    ///
  170|       |    /// Returns owned bytes produced by a fresh default codec.
  171|       |    ///
  172|       |    /// # Errors
  173|       |    ///
  174|       |    /// Returns a type mismatch or the typed encoder source error.
  175|       |    #[inline]
  176|     20|    pub fn encode(&self, value: &dyn Any) -> Result<Vec<u8>, ValueCodecExecutionError> {
  177|     20|        (self.encode)(value)
  178|     20|    }
  179|       |
  180|       |    /// Decodes one byte slice into a safely erased value of the declared type.
  181|       |    ///
  182|       |    /// # Parameters
  183|       |    ///
  184|       |    /// - `input`: Bytes borrowed for one decoding operation.
  185|       |    ///
  186|       |    /// # Returns
  187|       |    ///
  188|       |    /// Returns a boxed owned value that can be downcast to the declared type.
  189|       |    ///
  190|       |    /// # Errors
  191|       |    ///
  192|       |    /// Returns the typed decoder source error.
  193|       |    #[inline]
  194|     14|    pub fn decode(&self, input: &[u8]) -> Result<Box<dyn Any>, ValueCodecExecutionError> {
  195|     14|        (self.decode)(input)
  196|     14|    }
  197|       |}
  198|       |
  199|       |impl Debug for ValueBytesCodecDescriptor {
  200|       |    /// Formats descriptor metadata without invoking either codec function.
  201|       |    ///
  202|       |    /// # Parameters
  203|       |    ///
  204|       |    /// - `formatter`: Destination receiving the diagnostic type names.
  205|       |    ///
  206|       |    /// # Returns
  207|       |    ///
  208|       |    /// Returns success after both names have been formatted.
  209|       |    ///
  210|       |    /// # Errors
  211|       |    ///
  212|       |    /// Returns the formatting error if the destination rejects output.
  213|      2|    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
  214|      2|        formatter
  215|      2|            .debug_struct("ValueBytesCodecDescriptor")
  216|      2|            .field("codec_type_name", &self.codec_type_name())
  217|      2|            .field("value_type_name", &self.value_type_name())
  218|      2|            .finish_non_exhaustive()
  219|      2|    }
  220|       |}
  221|       |
  222|       |/// Downcasts and encodes one typed value using a fresh default codec.
  223|       |///
  224|       |/// # Type Parameters
  225|       |///
  226|       |/// - `C`: Default codec with matching bytes encode/decode contracts.
  227|       |/// - `V`: Concrete owned value type required by the descriptor.
  228|       |///
  229|       |/// # Parameters
  230|       |///
  231|       |/// - `value`: Erased input borrowed until serialization completes.
  232|       |///
  233|       |/// # Returns
  234|       |///
  235|       |/// Returns caller-owned encoded bytes.
  236|       |///
  237|       |/// # Errors
  238|       |///
  239|       |/// Returns `TypeMismatch` for a different concrete type, or `EncodeFailed`
  240|       |/// retaining the codec source error if encoding fails.
  241|     20|fn encode<C, V>(value: &dyn Any) -> Result<Vec<u8>, ValueCodecExecutionError>
  242|     20|where
  243|     20|    C: Default + ValueEncoder<V, Output = Vec<u8>> + ValueDecoder<[u8], Output = V> + 'static,
  244|     20|    <C as ValueEncoder<V>>::Error: Error + Send + Sync + 'static,
  245|     20|    <C as ValueDecoder<[u8]>>::Error: Error + Send + Sync + 'static,
  246|     20|    V: 'static,
  247|       |{
  248|     20|    let value = value
                      ^18
  249|     20|        .downcast_ref::<V>()
  250|     20|        .ok_or_else(|| ValueCodecExecutionError::TypeMismatch {
  251|      2|            expected_type: type_name::<V>(),
  252|      2|            actual_type: value.type_id(),
  253|      2|        })?;
  254|     18|    C::default()
  255|     18|        .encode(value)
  256|     18|        .map_err(|source| ValueCodecExecutionError::EncodeFailed {
  257|      1|            codec_type: type_name::<C>(),
  258|      1|            source: Box::new(source),
  259|      1|        })
  260|     20|}
  261|       |
  262|       |/// Decodes one byte slice and erases the typed output safely.
  263|       |///
  264|       |/// # Type Parameters
  265|       |///
  266|       |/// - `C`: Default codec with matching bytes encode/decode contracts.
  267|       |/// - `V`: Concrete owned output type placed in the returned box.
  268|       |///
  269|       |/// # Parameters
  270|       |///
  271|       |/// - `input`: Borrowed bytes passed to a fresh default decoder.
  272|       |///
  273|       |/// # Returns
  274|       |///
  275|       |/// Returns a newly boxed decoded value with no borrow of the input.
  276|       |///
  277|       |/// # Errors
  278|       |///
  279|       |/// Returns `DecodeFailed` retaining the codec source error on decode failure.
  280|     14|fn decode<C, V>(input: &[u8]) -> Result<Box<dyn Any>, ValueCodecExecutionError>
  281|     14|where
  282|     14|    C: Default + ValueEncoder<V, Output = Vec<u8>> + ValueDecoder<[u8], Output = V> + 'static,
  283|     14|    <C as ValueEncoder<V>>::Error: Error + Send + Sync + 'static,
  284|     14|    <C as ValueDecoder<[u8]>>::Error: Error + Send + Sync + 'static,
  285|     14|    V: 'static,
  286|       |{
  287|     14|    C::default()
  288|     14|        .decode(input)
  289|     14|        .map(|value| Box::new(value) as Box<dyn Any>)
                                   ^11      ^11       ^11
  290|     14|        .map_err(|source| ValueCodecExecutionError::DecodeFailed {
  291|      3|            codec_type: type_name::<C>(),
  292|      3|            source: Box::new(source),
  293|      3|        })
  294|     14|}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/value_codec_id.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |
    9|       |//! Stable value-codec identifiers.
   10|       |
   11|       |use core::borrow::Borrow;
   12|       |
   13|       |use crate::ValueCodecIdError;
   14|       |
   15|       |/// A validated, process-independent value-codec identifier.
   16|       |///
   17|       |/// # Examples
   18|       |///
   19|       |/// ```
   20|       |/// use qubit_codec::ValueCodecId;
   21|       |///
   22|       |/// let id = ValueCodecId::new("example.u16");
   23|       |/// assert_eq!(id.as_str(), "example.u16");
   24|       |/// ```
   25|       |#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
   26|       |pub struct ValueCodecId(
   27|       |    /// Validated static identifier, borrowed without allocation.
   28|       |    &'static str,
   29|       |);
   30|       |
   31|       |impl ValueCodecId {
   32|       |    /// Creates an ID from a static string.
   33|       |    ///
   34|       |    /// # Parameters
   35|       |    ///
   36|       |    /// - `value`: Non-empty dot-separated ASCII segments; each starts with a
   37|       |    ///   letter and continues with letters, digits, or underscores.
   38|       |    ///
   39|       |    /// # Returns
   40|       |    ///
   41|       |    /// An ID borrowing the validated static string without allocation.
   42|       |    ///
   43|       |    /// # Panics
   44|       |    ///
   45|       |    /// Panics when `value` violates the dot-separated ASCII protocol.
   46|       |    #[must_use]
   47|     37|    pub const fn new(value: &'static str) -> Self {
   48|     37|        match Self::try_new(value) {
   49|     36|            Ok(id) => id,
   50|      1|            Err(_) => panic!("invalid value codec ID"),
   51|       |        }
   52|     36|    }
   53|       |
   54|       |    /// Validates and creates an ID.
   55|       |    ///
   56|       |    /// # Parameters
   57|       |    ///
   58|       |    /// - `value`: Non-empty dot-separated ASCII segments; each starts with a
   59|       |    ///   letter and continues with letters, digits, or underscores.
   60|       |    ///
   61|       |    /// # Returns
   62|       |    ///
   63|       |    /// An ID borrowing the validated static string without allocation.
   64|       |    ///
   65|       |    /// # Errors
   66|       |    ///
   67|       |    /// Returns Empty for an empty ID, EmptySegment for leading/trailing or
   68|       |    /// consecutive dots, or InvalidSegment for an invalid ASCII segment.
   69|     58|    pub const fn try_new(value: &'static str) -> Result<Self, ValueCodecIdError> {
   70|     58|        match validate(value) {
   71|     39|            Ok(()) => Ok(Self(value)),
   72|     19|            Err(error) => Err(error),
   73|       |        }
   74|     58|    }
   75|       |
   76|       |    /// Returns the complete stable ID.
   77|       |    ///
   78|       |    /// # Returns
   79|       |    ///
   80|       |    /// The original static string; consuming this Copy ID does not invalidate
   81|       |    /// it.
   82|       |    #[inline]
   83|       |    #[must_use]
   84|     32|    pub const fn as_str(self) -> &'static str {
   85|     32|        self.0
   86|     32|    }
   87|       |}
   88|       |
   89|       |impl Borrow<str> for ValueCodecId {
   90|       |    /// Borrows the stable identifier for lookup without allocating.
   91|       |    ///
   92|       |    /// # Returns
   93|       |    ///
   94|       |    /// The complete ID string borrowed through this lookup key.
   95|       |    #[inline]
   96|     42|    fn borrow(&self) -> &str {
   97|     42|        self.0
   98|     42|    }
   99|       |}
  100|       |
  101|       |/// Validates one dot-separated ASCII identifier in linear time.
  102|       |///
  103|       |/// # Parameters
  104|       |///
  105|       |/// - `value`: Candidate identifier, borrowed without allocating.
  106|       |///
  107|       |/// # Returns
  108|       |///
  109|       |/// Unit when every segment follows the ASCII protocol.
  110|       |///
  111|       |/// # Errors
  112|       |///
  113|       |/// Returns Empty for no bytes, EmptySegment for a dot-delimited empty part,
  114|       |/// or InvalidSegment for a non-letter start or unsupported continuation.
  115|     58|const fn validate(value: &str) -> Result<(), ValueCodecIdError> {
  116|     58|    let bytes = value.as_bytes();
  117|     58|    if bytes.is_empty() {
  118|      2|        return Err(ValueCodecIdError::Empty);
  119|     56|    }
  120|     56|    let mut start = 0;
  121|     56|    let mut index = 0;
  122|    962|    while index < bytes.len() {
  123|    914|        if bytes[index] == b'.' {
  124|     96|            if start == index || index + 1 == bytes.len() {
                                               ^92
  125|      6|                return Err(ValueCodecIdError::EmptySegment);
  126|     90|            }
  127|     90|            if let Err(error) = validate_segment(bytes, start, index) {
                                     ^2
  128|      2|                return Err(error);
  129|     88|            }
  130|     88|            start = index + 1;
  131|    818|        }
  132|    906|        index += 1;
  133|       |    }
  134|     48|    validate_segment(bytes, start, bytes.len())
  135|     58|}
  136|       |
  137|       |/// Validates one non-empty ID segment in linear time.
  138|       |///
  139|       |/// # Parameters
  140|       |///
  141|       |/// - `bytes`: Complete identifier storage.
  142|       |/// - `start`: Inclusive segment boundary, no greater than `end`.
  143|       |/// - `end`: Exclusive boundary, no greater than `bytes.len()`.
  144|       |///
  145|       |/// # Returns
  146|       |///
  147|       |/// Unit for an ASCII letter followed by ASCII letters, digits, or underscores.
  148|       |///
  149|       |/// # Errors
  150|       |///
  151|       |/// Returns InvalidSegment for an empty segment or a disallowed byte.
  152|       |///
  153|       |/// # Panics
  154|       |///
  155|       |/// May panic if the caller violates the documented index bounds.
  156|    138|const fn validate_segment(bytes: &[u8], start: usize, end: usize) -> Result<(), ValueCodecIdError> {
  157|    138|    if start == end || !bytes[start].is_ascii_alphabetic() {
  158|      5|        return Err(ValueCodecIdError::InvalidSegment);
  159|    133|    }
  160|    133|    let mut index = start + 1;
  161|    748|    while index < end {
  162|    621|        let byte = bytes[index];
  163|    621|        if !(byte.is_ascii_alphanumeric() || byte == b'_') {
                                                           ^8
  164|      6|            return Err(ValueCodecIdError::InvalidSegment);
  165|    615|        }
  166|    615|        index += 1;
  167|       |    }
  168|    127|    Ok(())
  169|    138|}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/value_codec_registration.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |
    9|       |//! Registration type definitions are colocated with their public module.
   10|       |
   11|       |use crate::ValueCodecId;
   12|       |use crate::ValueCodecRegistrationSource;
   13|       |
   14|       |/// One statically linked value-codec implementation.
   15|       |///
   16|       |/// A registration bundles the stable identifier used for lookup and duplicate
   17|       |/// detection, a borrowed executable descriptor, and the link-time location the
   18|       |/// registration was declared at. It is produced by
   19|       |/// [`register_value_string_codec!`](crate::register_value_string_codec) or
   20|       |/// [`register_value_bytes_codec!`](crate::register_value_bytes_codec)
   21|       |/// rather than built by hand. Applications may also construct a registration
   22|       |/// manually from static data and pass it to a local
   23|       |/// [`ValueCodecRegistry`](crate::ValueCodecRegistry).
   24|       |///
   25|       |/// The ID has already been validated by [`ValueCodecId`]. The descriptor must
   26|       |/// live in static storage because registrations borrow it with a `'static`
   27|       |/// lifetime. The source records where the registration came from for duplicate
   28|       |/// ID diagnostics; it does not affect those static requirements.
   29|       |///
   30|       |/// The registration macros validate codecs through the matching descriptor.
   31|       |/// Encoder and decoder error types must implement `Error + Send + Sync +
   32|       |/// 'static`; codec and value types do not need those bounds.
   33|       |///
   34|       |/// Registration macros create these values from codec descriptors and submit
   35|       |/// them to the process-wide inventory. See
   36|       |/// [`register_value_string_codec!`](crate::register_value_string_codec) and
   37|       |/// [`register_value_bytes_codec!`](crate::register_value_bytes_codec) for
   38|       |/// executable registration examples. For a local registry, this type can be
   39|       |/// constructed directly from a validated ID, a static descriptor, and source
   40|       |/// information as shown below.
   41|       |/// # Type Parameters
   42|       |///
   43|       |/// - `D`: Static descriptor type that executes the registered codec.
   44|       |///
   45|       |/// # Examples
   46|       |///
   47|       |/// ```
   48|       |/// use qubit_codec::ValueCodecId;
   49|       |/// use qubit_codec::ValueCodecRegistration;
   50|       |/// use qubit_codec::ValueCodecRegistrationSource;
   51|       |///
   52|       |/// static DESCRIPTOR: u8 = 1;
   53|       |/// let registration = ValueCodecRegistration::new(
   54|       |///     ValueCodecId::new("example.u8"),
   55|       |///     &DESCRIPTOR,
   56|       |///     ValueCodecRegistrationSource::new("qubit-codec", "example", "src/example.rs", 1),
   57|       |/// );
   58|       |/// assert_eq!(registration.id().as_str(), "example.u8");
   59|       |/// assert_eq!(*registration.descriptor(), 1);
   60|       |/// ```
   61|       |#[derive(Clone, Copy, Debug)]
   62|       |pub struct ValueCodecRegistration<D: 'static> {
   63|       |    /// Stable identifier used by registry lookups and duplicate detection.
   64|       |    id: ValueCodecId,
   65|       |    /// Executable descriptor borrowed from static storage for the registration
   66|       |    /// lifetime.
   67|       |    descriptor: &'static D,
   68|       |    /// Link-time location identifying where the registration macro was invoked.
   69|       |    source: ValueCodecRegistrationSource,
   70|       |}
   71|       |impl<D: 'static> ValueCodecRegistration<D> {
   72|       |    /// Creates a registration from the validated facts captured by a
   73|       |    /// registration macro.
   74|       |    ///
   75|       |    /// Use the registration macros to submit a codec to the process-wide
   76|       |    /// inventory. For a local registry, construct this value from static data
   77|       |    /// and pass a reference to
   78|       |    /// [`ValueCodecRegistry::from_registrations`](crate::ValueCodecRegistry::from_registrations).
   79|       |    /// The ID must already be validated, and the descriptor must have static
   80|       |    /// storage. The source provides location details for duplicate-ID
   81|       |    /// diagnostics.
   82|       |    ///
   83|       |    /// # Parameters
   84|       |    ///
   85|       |    /// - `id`: Stable identifier used for registry lookup and duplicate
   86|       |    ///   detection.
   87|       |    /// - `descriptor`: Descriptor in static storage that performs codec
   88|       |    ///   operations.
   89|       |    /// - `source`: Declaration location captured by the registration macro.
   90|       |    ///
   91|       |    /// # Returns
   92|       |    ///
   93|       |    /// A copyable registration borrowing the static descriptor.
   94|       |    #[inline]
   95|       |    #[must_use]
   96|     35|    pub const fn new(id: ValueCodecId, descriptor: &'static D, source: ValueCodecRegistrationSource) -> Self {
   97|     35|        Self { id, descriptor, source }
   98|     35|    }
   99|       |
  100|       |    /// Returns the stable value-codec ID used by registry lookups.
  101|       |    ///
  102|       |    /// # Returns
  103|       |    ///
  104|       |    /// The registration's validated, copyable identifier.
  105|       |    #[inline]
  106|       |    #[must_use]
  107|    280|    pub const fn id(&self) -> ValueCodecId {
  108|    280|        self.id
  109|    280|    }
  110|       |
  111|       |    /// Returns the descriptor used to execute this codec.
  112|       |    ///
  113|       |    /// # Returns
  114|       |    ///
  115|       |    /// A static borrow of the descriptor associated with this registration.
  116|       |    #[inline]
  117|       |    #[must_use = "the codec descriptor is needed to perform codec operations"]
  118|     37|    pub const fn descriptor(&self) -> &'static D {
  119|     37|        self.descriptor
  120|     37|    }
  121|       |
  122|       |    /// Returns the source location captured at the registration declaration.
  123|       |    ///
  124|       |    /// # Returns
  125|       |    ///
  126|       |    /// The copyable package, module, file, and line information for this
  127|       |    /// registration.
  128|       |    #[inline]
  129|       |    #[must_use]
  130|     75|    pub const fn source(&self) -> ValueCodecRegistrationSource {
  131|     75|        self.source
  132|     75|    }
  133|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/value_codec_registration_macros.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |
    9|       |//! Distributed value-codec registrations.
   10|       |
   11|       |/// Registers a default-constructible bidirectional string codec.
   12|       |///
   13|       |/// The macro must be invoked at module scope; it defines the descriptor, a
   14|       |/// registration function, and an inventory submission in an anonymous const.
   15|       |/// The `id` must be a constant expression. Use a literal, constant, or
   16|       |/// `const fn` result; a runtime function call cannot define a static ID.
   17|       |/// The encoder and decoder error types must implement `Error + Send + Sync +
   18|       |/// 'static`, as required by [`crate::ValueStringCodecDescriptor::of`].
   19|       |/// Without `input`, the encoder input type is `value`. The optional `input`
   20|       |/// form selects a borrowed view of the owned `value` type for encoding; the
   21|       |/// descriptor still accepts and decodes the owned value type.
   22|       |///
   23|       |/// # Examples
   24|       |///
   25|       |/// A registry-only example with a local codec:
   26|       |///
   27|       |/// ```
   28|       |/// use core::convert::Infallible;
   29|       |/// use qubit_codec::{ValueDecoder, ValueEncoder, ValueStringCodecRegistry};
   30|       |/// use qubit_codec::register_value_string_codec;
   31|       |///
   32|       |/// #[derive(Default)]
   33|       |/// struct TextCodec;
   34|       |/// impl ValueEncoder<String> for TextCodec {
   35|       |///     type Output = String;
   36|       |///     type Error = Infallible;
   37|       |///     fn encode(&mut self, value: &String) -> Result<String, Infallible> { Ok(value.clone()) }
   38|       |/// }
   39|       |/// impl ValueDecoder<str> for TextCodec {
   40|       |///     type Output = String;
   41|       |///     type Error = Infallible;
   42|       |///     fn decode(&mut self, input: &str) -> Result<String, Infallible> { Ok(input.to_owned()) }
   43|       |/// }
   44|       |/// register_value_string_codec!(id = "example.text", codec = TextCodec, value = String);
   45|       |/// let registry = ValueStringCodecRegistry::try_global().unwrap();
   46|       |/// let descriptor = registry.get("example.text").unwrap().descriptor();
   47|       |/// let encoded = descriptor.encode(&String::from("hello")).unwrap();
   48|       |/// assert_eq!("hello", encoded);
   49|       |/// let decoded = descriptor.decode(&encoded).unwrap();
   50|       |/// assert_eq!("hello", *decoded.downcast::<String>().unwrap());
   51|       |/// ```
   52|       |///
   53|       |/// An invalid ID fails during macro expansion:
   54|       |///
   55|       |/// ```compile_fail
   56|       |/// # use core::convert::Infallible;
   57|       |/// # use qubit_codec::{ValueDecoder, ValueEncoder};
   58|       |/// # use qubit_codec::register_value_string_codec;
   59|       |/// # #[derive(Default)] struct TextCodec;
   60|       |/// # impl ValueEncoder<String> for TextCodec { type Output = String; type Error = Infallible; fn encode(&mut self, value: &String) -> Result<String, Infallible> { Ok(value.clone()) } }
   61|       |/// # impl ValueDecoder<str> for TextCodec { type Output = String; type Error = Infallible; fn decode(&mut self, input: &str) -> Result<String, Infallible> { Ok(input.to_owned()) } }
   62|       |/// register_value_string_codec!(id = "invalid-id", codec = TextCodec, value = String);
   63|       |/// ```
   64|       |///
   65|       |/// A runtime ID cannot initialize the static registration:
   66|       |///
   67|       |/// ```compile_fail
   68|       |/// # use core::convert::Infallible;
   69|       |/// # use qubit_codec::{ValueDecoder, ValueEncoder};
   70|       |/// # use qubit_codec::register_value_string_codec;
   71|       |/// # #[derive(Default)] struct TextCodec;
   72|       |/// # impl ValueEncoder<String> for TextCodec { type Output = String; type Error = Infallible; fn encode(&mut self, value: &String) -> Result<String, Infallible> { Ok(value.clone()) } }
   73|       |/// # impl ValueDecoder<str> for TextCodec { type Output = String; type Error = Infallible; fn decode(&mut self, input: &str) -> Result<String, Infallible> { Ok(input.to_owned()) } }
   74|       |/// fn runtime_id() -> &'static str { "example.text" }
   75|       |/// register_value_string_codec!(id = runtime_id(), codec = TextCodec, value = String);
   76|       |/// ```
   77|       |#[macro_export]
   78|       |macro_rules! register_value_string_codec {
   79|       |    (id = $id:expr, codec = $codec:ty, value = $value:ty, input = $input:ty $(,)?) => {
   80|       |        const _: () = {
   81|       |            const ID: $crate::ValueCodecId = $crate::ValueCodecId::new($id);
   82|       |            static DESCRIPTOR: $crate::ValueStringCodecDescriptor =
   83|       |                $crate::ValueStringCodecDescriptor::of_borrowed::<$codec, $value, $input>();
   84|      1|            fn registration() -> $crate::ValueStringCodecRegistration {
   85|      1|                $crate::ValueCodecRegistration::new(
   86|       |                    ID,
   87|      1|                    &DESCRIPTOR,
   88|      1|                    $crate::ValueCodecRegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
   89|       |                )
   90|      1|            }
   91|       |            $crate::__private::inventory::submit! {
   92|       |                $crate::ValueStringCodecRegistrationFactory(registration)
   93|       |            }
   94|       |        };
   95|       |    };
   96|       |    (id = $id:expr, codec = $codec:ty, value = $value:ty $(,)?) => {
   97|       |        const _: () = {
   98|       |            const ID: $crate::ValueCodecId = $crate::ValueCodecId::new($id);
   99|       |            static DESCRIPTOR: $crate::ValueStringCodecDescriptor =
  100|       |                $crate::ValueStringCodecDescriptor::of::<$codec, $value>();
  101|       |
  102|      6|            fn registration() -> $crate::ValueStringCodecRegistration {
  103|      6|                $crate::ValueCodecRegistration::new(
  104|       |                    ID,
  105|      6|                    &DESCRIPTOR,
  106|      6|                    $crate::ValueCodecRegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
  107|       |                )
  108|      6|            }
  109|       |
  110|       |            $crate::__private::inventory::submit! {
  111|       |                $crate::ValueStringCodecRegistrationFactory(registration)
  112|       |            }
  113|       |        };
  114|       |    };
  115|       |}
  116|       |
  117|       |/// Registers a default-constructible bidirectional bytes codec.
  118|       |///
  119|       |/// The macro must be invoked at module scope; it defines the descriptor, a
  120|       |/// registration function, and an inventory submission in an anonymous const.
  121|       |/// The `id` must be a constant expression. Use a literal, constant, or
  122|       |/// `const fn` result; a runtime function call cannot define a static ID.
  123|       |/// The encoder and decoder error types must implement `Error + Send + Sync +
  124|       |/// 'static`, as required by [`crate::ValueBytesCodecDescriptor::of`].
  125|       |///
  126|       |/// # Examples
  127|       |///
  128|       |/// A registry-only example with a local codec:
  129|       |///
  130|       |/// ```
  131|       |/// use core::convert::Infallible;
  132|       |/// use qubit_codec::{ValueBytesCodecRegistry, ValueDecoder, ValueEncoder};
  133|       |/// use qubit_codec::register_value_bytes_codec;
  134|       |///
  135|       |/// #[derive(Default)]
  136|       |/// struct BytesCodec;
  137|       |/// impl ValueEncoder<Vec<u8>> for BytesCodec {
  138|       |///     type Output = Vec<u8>;
  139|       |///     type Error = Infallible;
  140|       |///     fn encode(&mut self, value: &Vec<u8>) -> Result<Vec<u8>, Infallible> { Ok(value.clone()) }
  141|       |/// }
  142|       |/// impl ValueDecoder<[u8]> for BytesCodec {
  143|       |///     type Output = Vec<u8>;
  144|       |///     type Error = Infallible;
  145|       |///     fn decode(&mut self, input: &[u8]) -> Result<Vec<u8>, Infallible> { Ok(input.to_vec()) }
  146|       |/// }
  147|       |/// register_value_bytes_codec!(id = "example.bytes", codec = BytesCodec, value = Vec<u8>);
  148|       |/// let registry = ValueBytesCodecRegistry::try_global().unwrap();
  149|       |/// let descriptor = registry.get("example.bytes").unwrap().descriptor();
  150|       |/// let encoded = descriptor.encode(&vec![1_u8, 2]).unwrap();
  151|       |/// assert_eq!(vec![1_u8, 2], encoded);
  152|       |/// let decoded = descriptor.decode(&encoded).unwrap();
  153|       |/// assert_eq!(vec![1_u8, 2], *decoded.downcast::<Vec<u8>>().unwrap());
  154|       |/// ```
  155|       |///
  156|       |/// An invalid ID fails during macro expansion:
  157|       |///
  158|       |/// ```compile_fail
  159|       |/// # use core::convert::Infallible;
  160|       |/// # use qubit_codec::{ValueDecoder, ValueEncoder};
  161|       |/// # use qubit_codec::register_value_bytes_codec;
  162|       |/// # #[derive(Default)] struct BytesCodec;
  163|       |/// # impl ValueEncoder<Vec<u8>> for BytesCodec { type Output = Vec<u8>; type Error = Infallible; fn encode(&mut self, value: &Vec<u8>) -> Result<Vec<u8>, Infallible> { Ok(value.clone()) } }
  164|       |/// # impl ValueDecoder<[u8]> for BytesCodec { type Output = Vec<u8>; type Error = Infallible; fn decode(&mut self, input: &[u8]) -> Result<Vec<u8>, Infallible> { Ok(input.to_vec()) } }
  165|       |/// register_value_bytes_codec!(id = "invalid-id", codec = BytesCodec, value = Vec<u8>);
  166|       |/// ```
  167|       |#[macro_export]
  168|       |macro_rules! register_value_bytes_codec {
  169|       |    (id = $id:expr, codec = $codec:ty, value = $value:ty $(,)?) => {
  170|       |        const _: () = {
  171|       |            const ID: $crate::ValueCodecId = $crate::ValueCodecId::new($id);
  172|       |            static DESCRIPTOR: $crate::ValueBytesCodecDescriptor =
  173|       |                $crate::ValueBytesCodecDescriptor::of::<$codec, $value>();
  174|       |
  175|      6|            fn registration() -> $crate::ValueBytesCodecRegistration {
  176|      6|                $crate::ValueCodecRegistration::new(
  177|       |                    ID,
  178|      6|                    &DESCRIPTOR,
  179|      6|                    $crate::ValueCodecRegistrationSource::new(env!("CARGO_PKG_NAME"), module_path!(), file!(), line!()),
  180|       |                )
  181|      6|            }
  182|       |
  183|       |            $crate::__private::inventory::submit! {
  184|       |                $crate::ValueBytesCodecRegistrationFactory(registration)
  185|       |            }
  186|       |        };
  187|       |    };
  188|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/value_codec_registration_source.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |
    9|       |//! Value-codec registration source locations.
   10|       |
   11|       |/// Source location for one linked value-codec registration.
   12|       |///
   13|       |/// The four parts are captured at compile time by the registration macros from
   14|       |/// `CARGO_PKG_NAME`, `module_path!`, `file!`, and `line!`, so they point at the
   15|       |/// declaration site rather than at the call site that later looks the
   16|       |/// registration up. Ordering compares the source fields in derived order:
   17|       |/// crate name, module path, file, then line. Duplicate reports therefore have
   18|       |/// stable source-field ordering independent of registration iteration order.
   19|       |///
   20|       |/// # Examples
   21|       |///
   22|       |/// ```
   23|       |/// use qubit_codec::ValueCodecRegistrationSource;
   24|       |///
   25|       |/// let source = ValueCodecRegistrationSource::new(
   26|       |///     "qubit-codec",
   27|       |///     "my_crate::codecs",
   28|       |///     "src/codec.rs",
   29|       |///     42,
   30|       |/// );
   31|       |/// assert_eq!(source.crate_name(), "qubit-codec");
   32|       |/// assert_eq!(source.line(), 42);
   33|       |/// ```
   34|       |#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
   35|       |pub struct ValueCodecRegistrationSource {
   36|       |    /// Package name captured from `CARGO_PKG_NAME` at the declaration site.
   37|       |    crate_name: &'static str,
   38|       |    /// Module path captured from `module_path!` at the declaration site.
   39|       |    module_path: &'static str,
   40|       |    /// Source file captured from `file!` at the declaration site.
   41|       |    file: &'static str,
   42|       |    /// One-based line captured from `line!` at the declaration site.
   43|       |    line: u32,
   44|       |}
   45|       |
   46|       |impl ValueCodecRegistrationSource {
   47|       |    /// Creates a source location captured by a registration macro.
   48|       |    ///
   49|       |    /// # Parameters
   50|       |    ///
   51|       |    /// - `crate_name`: Package name captured from `CARGO_PKG_NAME`.
   52|       |    /// - `module_path`: Module path captured from `module_path!`.
   53|       |    /// - `file`: Source file captured from `file!`.
   54|       |    /// - `line`: One-based line captured from `line!`.
   55|       |    ///
   56|       |    /// # Returns
   57|       |    ///
   58|       |    /// Returns a location that borrows all four captured parts statically.
   59|       |    ///
   60|       |    /// Prefer the registration macros over calling this directly.
   61|       |    #[doc(hidden)]
   62|       |    #[inline]
   63|       |    #[must_use]
   64|     57|    pub const fn new(crate_name: &'static str, module_path: &'static str, file: &'static str, line: u32) -> Self {
   65|     57|        Self {
   66|     57|            crate_name,
   67|     57|            module_path,
   68|     57|            file,
   69|     57|            line,
   70|     57|        }
   71|     57|    }
   72|       |
   73|       |    /// Returns the declaring crate name.
   74|       |    ///
   75|       |    /// # Returns
   76|       |    ///
   77|       |    /// Returns the `CARGO_PKG_NAME` of the crate that declared the codec.
   78|       |    #[must_use]
   79|       |    #[inline]
   80|     12|    pub const fn crate_name(&self) -> &'static str {
   81|     12|        self.crate_name
   82|     12|    }
   83|       |
   84|       |    /// Returns the declaring module path.
   85|       |    ///
   86|       |    /// # Returns
   87|       |    ///
   88|       |    /// Returns the `module_path!` value of the declaring module.
   89|       |    #[must_use]
   90|       |    #[inline]
   91|      9|    pub const fn module_path(&self) -> &'static str {
   92|      9|        self.module_path
   93|      9|    }
   94|       |
   95|       |    /// Returns the source file path.
   96|       |    ///
   97|       |    /// # Returns
   98|       |    ///
   99|       |    /// Returns the `file!` value of the declaring file.
  100|       |    #[must_use]
  101|       |    #[inline]
  102|      7|    pub const fn file(&self) -> &'static str {
  103|      7|        self.file
  104|      7|    }
  105|       |
  106|       |    /// Returns the one-based source line.
  107|       |    ///
  108|       |    /// # Returns
  109|       |    ///
  110|       |    /// Returns the `line!` value of the declaring macro invocation.
  111|       |    #[must_use]
  112|       |    #[inline]
  113|      9|    pub const fn line(&self) -> u32 {
  114|      9|        self.line
  115|      9|    }
  116|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/value_codec_registry.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |
    9|       |//! Local immutable value-codec registry.
   10|       |
   11|       |use std::collections::BTreeMap;
   12|       |
   13|       |use crate::ValueCodecId;
   14|       |use crate::ValueCodecRegistration;
   15|       |use crate::ValueCodecRegistryError;
   16|       |
   17|       |// Implements process-wide lookup for the bytes-wire specialization.
   18|       |#[path = "value_codec_registry/value_bytes_codec_registry.rs"]
   19|       |mod value_bytes_codec_registry;
   20|       |// Implements process-wide lookup for the string-wire specialization.
   21|       |#[path = "value_codec_registry/value_string_codec_registry.rs"]
   22|       |mod value_string_codec_registry;
   23|       |
   24|       |pub use value_bytes_codec_registry::ValueBytesCodecRegistry;
   25|       |pub use value_string_codec_registry::ValueStringCodecRegistry;
   26|       |
   27|       |/// An immutable value-codec registry sorted by stable ID.
   28|       |///
   29|       |/// # Type Parameters
   30|       |///
   31|       |/// - `D`: Static descriptor type identifying one wire representation. The
   32|       |///   registry API is implemented for descriptors that also satisfy `Copy`.
   33|       |///
   34|       |/// # Examples
   35|       |///
   36|       |/// ```
   37|       |/// use qubit_codec::ValueCodecRegistry;
   38|       |/// use qubit_codec::ValueStringCodecDescriptor;
   39|       |/// use qubit_codec::ValueStringCodecRegistry;
   40|       |///
   41|       |/// let registry: ValueCodecRegistry<ValueStringCodecDescriptor> =
   42|       |///     ValueStringCodecRegistry::empty();
   43|       |/// assert!(registry.get("missing.example").is_none());
   44|       |/// ```
   45|       |#[derive(Debug)]
   46|       |pub struct ValueCodecRegistry<D: 'static> {
   47|       |    /// Owned registrations sorted once by stable ID during construction.
   48|       |    registrations: Box<[ValueCodecRegistration<D>]>,
   49|       |    /// Stable ID to position mapping into the immutable registration slice.
   50|       |    indices: BTreeMap<ValueCodecId, usize>,
   51|       |}
   52|       |
   53|       |impl<D: Copy + 'static> ValueCodecRegistry<D> {
   54|       |    /// Builds a local registry from static registrations without consulting
   55|       |    /// or modifying the process-wide registry.
   56|       |    ///
   57|       |    /// # Parameters
   58|       |    ///
   59|       |    /// - `registrations`: Static entries copied into this registry; input order
   60|       |    ///   is ignored.
   61|       |    ///
   62|       |    /// # Returns
   63|       |    ///
   64|       |    /// An owned, immutable registry sorted by ID, independent of the iterator.
   65|       |    ///
   66|       |    /// # Errors
   67|       |    ///
   68|       |    /// Returns a duplicate-ID error when multiple entries claim one ID.
   69|       |    ///
   70|       |    /// # Example
   71|       |    ///
   72|       |    /// This example uses a local static registration; it does not submit an
   73|       |    /// entry to the process-wide inventory.
   74|       |    ///
   75|       |    /// ```
   76|       |    /// use std::convert::Infallible;
   77|       |    /// use qubit_codec::ValueCodecId;
   78|       |    /// use qubit_codec::ValueCodecRegistration;
   79|       |    /// use qubit_codec::ValueCodecRegistrationSource;
   80|       |    /// use qubit_codec::ValueDecoder;
   81|       |    /// use qubit_codec::ValueEncoder;
   82|       |    /// use qubit_codec::ValueStringCodecDescriptor;
   83|       |    /// use qubit_codec::ValueStringCodecRegistration;
   84|       |    /// use qubit_codec::ValueStringCodecRegistry;
   85|       |    ///
   86|       |    /// #[derive(Default)]
   87|       |    /// struct LocalText;
   88|       |    ///
   89|       |    /// impl ValueEncoder<String> for LocalText {
   90|       |    ///     type Output = String;
   91|       |    ///     type Error = Infallible;
   92|       |    ///
   93|       |    ///     fn encode(&mut self, input: &String) -> Result<String, Infallible> {
   94|       |    ///         Ok(input.clone())
   95|       |    ///     }
   96|       |    /// }
   97|       |    ///
   98|       |    /// impl ValueDecoder<str> for LocalText {
   99|       |    ///     type Output = String;
  100|       |    ///     type Error = Infallible;
  101|       |    ///
  102|       |    ///     fn decode(&mut self, input: &str) -> Result<String, Infallible> {
  103|       |    ///         Ok(input.to_owned())
  104|       |    ///     }
  105|       |    /// }
  106|       |    ///
  107|       |    /// static DESCRIPTOR: ValueStringCodecDescriptor =
  108|       |    ///     ValueStringCodecDescriptor::of::<LocalText, String>();
  109|       |    /// static REGISTRATION: ValueStringCodecRegistration = ValueCodecRegistration::new(
  110|       |    ///     ValueCodecId::new("example.local.text"),
  111|       |    ///     &DESCRIPTOR,
  112|       |    ///     ValueCodecRegistrationSource::new("local-example", "example", file!(), line!()),
  113|       |    /// );
  114|       |    ///
  115|       |    /// let registry = ValueStringCodecRegistry::from_registrations([&REGISTRATION])
  116|       |    ///     .expect("unique local registration");
  117|       |    /// let descriptor = registry.get("example.local.text")
  118|       |    ///     .expect("registered locally").descriptor();
  119|       |    /// let encoded = descriptor.encode(&String::from("hello")).expect("encode");
  120|       |    /// assert_eq!("hello", encoded);
  121|       |    /// let decoded = descriptor.decode(&encoded).expect("decode");
  122|       |    /// assert_eq!("hello", *decoded.downcast::<String>().expect("String output"));
  123|       |    /// assert!(registry.get("example.missing").is_none());
  124|       |    /// ```
  125|     17|    pub fn from_registrations(
  126|     17|        registrations: impl IntoIterator<Item = &'static ValueCodecRegistration<D>>,
  127|     17|    ) -> Result<Self, ValueCodecRegistryError> {
  128|     17|        Self::build(registrations.into_iter().copied().collect())
  129|     17|    }
  130|       |
  131|       |    /// Returns an empty local registry.
  132|       |    ///
  133|       |    /// # Returns
  134|       |    ///
  135|       |    /// A local registry with no registrations or indexed IDs.
  136|       |    #[must_use]
  137|       |    #[inline]
  138|      1|    pub fn empty() -> Self {
  139|      1|        Self {
  140|      1|            registrations: Box::new([]),
  141|      1|            indices: BTreeMap::new(),
  142|      1|        }
  143|      1|    }
  144|       |
  145|       |    /// Freezes registrations and rejects duplicate IDs.
  146|       |    ///
  147|       |    /// # Parameters
  148|       |    ///
  149|       |    /// - `registrations`: Owned entries sorted in place before indexing.
  150|       |    ///
  151|       |    /// # Returns
  152|       |    ///
  153|       |    /// An immutable registry owning the sorted entries and their lookup index.
  154|       |    ///
  155|       |    /// # Errors
  156|       |    ///
  157|       |    /// Returns `DuplicateId` with the lexically first conflicting ID and all
  158|       |    /// of its sources sorted by source fields. Repeated identical sources are
  159|       |    /// retained because each registration is reported.
  160|     21|    pub(super) fn build(mut registrations: Vec<ValueCodecRegistration<D>>) -> Result<Self, ValueCodecRegistryError> {
  161|     21|        registrations.sort_by_key(ValueCodecRegistration::id);
  162|     26|        for (index, pair) in registrations.windows(2).enumerate() {
                                           ^21                      ^21
  163|     26|            if pair[0].id() == pair[1].id() {
  164|     17|                let id = pair[0].id();
  165|     17|                let mut sources = registrations[index..]
  166|     17|                    .iter()
  167|     51|                    .take_while(|registration| registration.id() == id)
                                   ^17
  168|     17|                    .map(ValueCodecRegistration::source)
  169|     17|                    .collect::<Vec<_>>();
  170|     17|                sources.sort_unstable();
  171|     17|                return Err(ValueCodecRegistryError::DuplicateId {
  172|     17|                    id: id.as_str(),
  173|     17|                    sources,
  174|     17|                });
  175|      9|            }
  176|       |        }
  177|      4|        let indices = registrations
  178|      4|            .iter()
  179|      4|            .enumerate()
  180|     13|            .map(|(index, registration)| (registration.id(), index))
                           ^4
  181|      4|            .collect();
  182|      4|        Ok(Self {
  183|      4|            registrations: registrations.into_boxed_slice(),
  184|      4|            indices,
  185|      4|        })
  186|     21|    }
  187|       |
  188|       |    /// Finds a registration by stable ID.
  189|       |    ///
  190|       |    /// # Parameters
  191|       |    ///
  192|       |    /// - `id`: Exact stable ID to look up, without normalization.
  193|       |    ///
  194|       |    /// # Returns
  195|       |    ///
  196|       |    /// `Some` borrows the matching registration from this registry; `None`
  197|       |    /// means the ID is absent. Lookup uses a logarithmic tree search.
  198|       |    #[must_use]
  199|       |    #[inline]
  200|     17|    pub fn get(&self, id: &str) -> Option<&ValueCodecRegistration<D>> {
  201|     17|        let index = *self.indices.get(id)?;
                          ^16                          ^1
  202|     16|        self.registrations.get(index)
  203|     17|    }
  204|       |
  205|       |    /// Returns registrations in deterministic ID order.
  206|       |    ///
  207|       |    /// # Returns
  208|       |    ///
  209|       |    /// The immutable registration slice borrowed from this registry without
  210|       |    /// allocation.
  211|       |    #[must_use]
  212|       |    #[inline]
  213|      2|    pub fn registrations(&self) -> &[ValueCodecRegistration<D>] {
  214|      2|        &self.registrations
  215|      2|    }
  216|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/value_codec_registry/value_bytes_codec_registry.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2025 - 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |
    9|       |//! Process-wide registry for bytes-wire value codecs.
   10|       |
   11|       |use std::sync::OnceLock;
   12|       |
   13|       |use inventory::iter;
   14|       |
   15|       |use crate::ValueBytesCodecDescriptor;
   16|       |use crate::ValueBytesCodecRegistrationFactory;
   17|       |use crate::ValueCodecRegistry;
   18|       |use crate::ValueCodecRegistryError;
   19|       |
   20|       |/// An immutable bytes-wire value-codec registry sorted by stable ID.
   21|       |///
   22|       |/// # Examples
   23|       |///
   24|       |/// ```
   25|       |/// use qubit_codec::ValueBytesCodecRegistry;
   26|       |///
   27|       |/// let registry = ValueBytesCodecRegistry::empty();
   28|       |/// assert!(registry.get("missing.example").is_none());
   29|       |/// ```
   30|       |pub type ValueBytesCodecRegistry = ValueCodecRegistry<ValueBytesCodecDescriptor>;
   31|       |
   32|       |impl ValueBytesCodecRegistry {
   33|       |    /// Initializes and returns the process-wide linked bytes registry.
   34|       |    ///
   35|       |    /// Initialization is synchronized and cached, including construction
   36|       |    /// failures.
   37|       |    ///
   38|       |    /// # Returns
   39|       |    ///
   40|       |    /// A shared static registry containing all linked bytes registrations.
   41|       |    ///
   42|       |    /// # Errors
   43|       |    ///
   44|       |    /// Returns the cached construction error when linked registrations
   45|       |    /// conflict. The error identifies the lexically first duplicate ID and
   46|       |    /// all declarations for that ID in source-field order.
   47|     10|    pub fn try_global() -> Result<&'static Self, ValueCodecRegistryError> {
   48|       |        // Caches both successful construction and duplicate-ID failures.
   49|       |        static REGISTRY: OnceLock<Result<ValueBytesCodecRegistry, ValueCodecRegistryError>> = OnceLock::new();
   50|     10|        match REGISTRY.get_or_init(|| {
                                                    ^2
   51|      2|            let registrations = iter::<ValueBytesCodecRegistrationFactory>
   52|      2|                .into_iter()
   53|      9|                .map(|factory| (factory.0)())
                               ^2
   54|      2|                .collect();
   55|      2|            ValueCodecRegistry::build(registrations)
   56|      2|        }) {
   57|      7|            Ok(registry) => Ok(registry),
   58|      3|            Err(error) => Err(error.clone()),
   59|       |        }
   60|     10|    }
   61|       |
   62|       |    /// Returns the process-wide bytes registry or panics with a stable
   63|       |    /// diagnostic.
   64|       |    ///
   65|       |    /// # Returns
   66|       |    ///
   67|       |    /// A shared static registry, initialized at most once by `try_global`.
   68|       |    ///
   69|       |    /// # Panics
   70|       |    ///
   71|       |    /// Panics when linked registrations conflict.
   72|       |    #[must_use]
   73|      4|    pub fn global() -> &'static Self {
   74|      4|        Self::try_global().unwrap_or_else(|error| panic!("invalid global value bytes codec registry: {error}"))
                                                                ^1
   75|      4|    }
   76|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/value_codec_registry/value_string_codec_registry.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |
    9|       |//! Process-wide registry for string-wire value codecs.
   10|       |
   11|       |use std::sync::OnceLock;
   12|       |
   13|       |use inventory::iter;
   14|       |
   15|       |use crate::ValueCodecRegistry;
   16|       |use crate::ValueCodecRegistryError;
   17|       |use crate::ValueStringCodecDescriptor;
   18|       |use crate::ValueStringCodecRegistrationFactory;
   19|       |
   20|       |/// An immutable string-wire value-codec registry sorted by stable ID.
   21|       |///
   22|       |/// # Examples
   23|       |///
   24|       |/// ```
   25|       |/// use qubit_codec::ValueStringCodecRegistry;
   26|       |///
   27|       |/// let registry = ValueStringCodecRegistry::empty();
   28|       |/// assert!(registry.get("missing.example").is_none());
   29|       |/// ```
   30|       |pub type ValueStringCodecRegistry = ValueCodecRegistry<ValueStringCodecDescriptor>;
   31|       |
   32|       |impl ValueStringCodecRegistry {
   33|       |    /// Initializes and returns the process-wide linked string registry.
   34|       |    ///
   35|       |    /// Initialization is synchronized and cached, including construction
   36|       |    /// failures.
   37|       |    ///
   38|       |    /// # Returns
   39|       |    ///
   40|       |    /// A shared static registry containing all linked string registrations.
   41|       |    ///
   42|       |    /// # Errors
   43|       |    ///
   44|       |    /// Returns the cached construction error when linked registrations
   45|       |    /// conflict. The error identifies the lexically first duplicate ID and
   46|       |    /// all declarations for that ID in source-field order.
   47|     20|    pub fn try_global() -> Result<&'static Self, ValueCodecRegistryError> {
   48|       |        // Caches both successful construction and duplicate-ID failures.
   49|       |        static REGISTRY: OnceLock<Result<ValueStringCodecRegistry, ValueCodecRegistryError>> = OnceLock::new();
   50|     20|        match REGISTRY.get_or_init(|| {
                                                    ^2
   51|      2|            let registrations = iter::<ValueStringCodecRegistrationFactory>
   52|      2|                .into_iter()
   53|     10|                .map(|factory| (factory.0)())
                               ^2
   54|      2|                .collect();
   55|      2|            ValueCodecRegistry::build(registrations)
   56|      2|        }) {
   57|     17|            Ok(registry) => Ok(registry),
   58|      3|            Err(error) => Err(error.clone()),
   59|       |        }
   60|     20|    }
   61|       |
   62|       |    /// Returns the process-wide string registry or panics with a stable
   63|       |    /// diagnostic.
   64|       |    ///
   65|       |    /// # Returns
   66|       |    ///
   67|       |    /// A shared static registry, initialized at most once by `try_global`.
   68|       |    ///
   69|       |    /// # Panics
   70|       |    ///
   71|       |    /// Panics when linked registrations conflict.
   72|       |    #[must_use]
   73|       |    #[inline]
   74|      5|    pub fn global() -> &'static Self {
   75|      5|        Self::try_global().unwrap_or_else(|error| panic!("invalid global value string codec registry: {error}"))
                                                                ^1
   76|      5|    }
   77|       |}

/data/working/qubit/rust-common/.worktrees/rs-codec/rc-r2-7a98bc17/src/value/value_string_codec_descriptor.rs:
    1|       |// =============================================================================
    2|       |//    Copyright (c) 2026 Haixing Hu.
    3|       |//
    4|       |//    SPDX-License-Identifier: Apache-2.0
    5|       |//
    6|       |//    Licensed under the Apache License, Version 2.0.
    7|       |// =============================================================================
    8|       |
    9|       |//! Safe type-erased bidirectional string value-codec descriptors.
   10|       |
   11|       |use core::any::type_name;
   12|       |use core::fmt::Debug;
   13|       |use core::fmt::Formatter;
   14|       |use core::fmt::Result as FmtResult;
   15|       |use std::any::Any;
   16|       |use std::any::TypeId;
   17|       |use std::borrow::Borrow;
   18|       |use std::error::Error;
   19|       |
   20|       |use crate::ValueCodecExecutionError;
   21|       |use crate::ValueDecoder;
   22|       |use crate::ValueEncoder;
   23|       |
   24|       |/// Checked type-erased encoding entry point returning an owned string.
   25|       |type EncodeFn = fn(&dyn Any) -> Result<String, ValueCodecExecutionError>;
   26|       |/// Type-erased decoding entry point returning a boxed owned value.
   27|       |type DecodeFn = fn(&str) -> Result<Box<dyn Any>, ValueCodecExecutionError>;
   28|       |
   29|       |/// An immutable, safely erased bidirectional string codec for one value type.
   30|       |///
   31|       |/// Each operation creates a fresh default codec; no codec state is retained
   32|       |/// between calls. Type identities are process-local and names are diagnostic,
   33|       |/// not stable IDs.
   34|       |///
   35|       |/// # Examples
   36|       |///
   37|       |/// ```
   38|       |/// use core::convert::Infallible;
   39|       |///
   40|       |/// use qubit_codec::ValueDecoder;
   41|       |/// use qubit_codec::ValueEncoder;
   42|       |/// use qubit_codec::ValueStringCodecDescriptor;
   43|       |///
   44|       |/// #[derive(Default)]
   45|       |/// struct Text;
   46|       |/// impl ValueEncoder<u32> for Text {
   47|       |///     type Output = String;
   48|       |///     type Error = Infallible;
   49|       |///
   50|       |///     fn encode(&mut self, input: &u32) -> Result<String, Infallible> {
   51|       |///         Ok(input.to_string())
   52|       |///     }
   53|       |/// }
   54|       |/// impl ValueDecoder<str> for Text {
   55|       |///     type Output = u32;
   56|       |///     type Error = Infallible;
   57|       |///
   58|       |///     fn decode(&mut self, input: &str) -> Result<u32, Infallible> {
   59|       |///         Ok(input.len() as u32)
   60|       |///     }
   61|       |/// }
   62|       |///
   63|       |/// let descriptor = ValueStringCodecDescriptor::of::<Text, u32>();
   64|       |///
   65|       |/// // `encode` takes an erased `&dyn Any`, so the value type must be named.
   66|       |/// let value: u32 = 7;
   67|       |/// assert_eq!(descriptor.encode(&value).unwrap(), "7");
   68|       |///
   69|       |/// let decoded = descriptor.decode("hello").unwrap();
   70|       |/// assert_eq!(*decoded.downcast::<u32>().unwrap(), 5);
   71|       |/// ```
   72|       |#[derive(Clone, Copy)]
   73|       |pub struct ValueStringCodecDescriptor {
   74|       |    /// Returns the process-local identity of the concrete codec.
   75|       |    codec_type_id: fn() -> TypeId,
   76|       |    /// Returns the diagnostic name of the concrete codec.
   77|       |    codec_type_name: fn() -> &'static str,
   78|       |    /// Returns the process-local identity of the decoded value.
   79|       |    value_type_id: fn() -> TypeId,
   80|       |    /// Returns the diagnostic name of the decoded value.
   81|       |    value_type_name: fn() -> &'static str,
   82|       |    /// Checks the erased input type and invokes a fresh default encoder.
   83|       |    encode: EncodeFn,
   84|       |    /// Invokes a fresh default decoder and boxes its owned output.
   85|       |    decode: DecodeFn,
   86|       |}
   87|       |
   88|       |impl ValueStringCodecDescriptor {
   89|       |    /// Creates a descriptor for codec `C` and value type `V`.
   90|       |    ///
   91|       |    /// The codec error types must implement `Send + Sync` so execution errors
   92|       |    /// can retain their source across thread boundaries. The codec and value
   93|       |    /// types themselves need not implement these traits; `V` only needs to be
   94|       |    /// `'static` because decoded values are returned through `Any`.
   95|       |    /// Use [`Self::of_borrowed`] when the encoder accepts a borrowed view of
   96|       |    /// `V` instead of `V` itself.
   97|       |    ///
   98|       |    /// # Type Parameters
   99|       |    ///
  100|       |    /// - `C`: Default-constructible string encoder/decoder with owned errors.
  101|       |    /// - `V`: Owned value type accepted and produced by `C`.
  102|       |    ///
  103|       |    /// # Returns
  104|       |    ///
  105|       |    /// Returns immutable function metadata without constructing a codec.
  106|       |    #[must_use]
  107|       |    #[inline]
  108|      1|    pub const fn of<C, V>() -> Self
  109|      1|    where
  110|      1|        C: Default + ValueEncoder<V, Output = String> + ValueDecoder<str, Output = V> + 'static,
  111|      1|        <C as ValueEncoder<V>>::Error: Error + Send + Sync + 'static,
  112|      1|        <C as ValueDecoder<str>>::Error: Error + Send + Sync + 'static,
  113|      1|        V: 'static,
  114|       |    {
  115|      1|        Self::of_borrowed::<C, V, V>()
  116|      1|    }
  117|       |
  118|       |    /// Creates a descriptor that encodes an owned value through a borrowed
  119|       |    /// view.
  120|       |    ///
  121|       |    /// The descriptor continues to accept and return the owned type `V`.
  122|       |    /// During each encode call it checks the erased value is exactly `V`, then
  123|       |    /// borrows an `I` view with [`Borrow`] for that call only. The view is
  124|       |    /// never stored. Each operation constructs a fresh `C::default()`
  125|       |    /// codec. Encoder and decoder errors must implement `Error + Send +
  126|       |    /// Sync + 'static`; `C` and `V` themselves need not be `Send` or
  127|       |    /// `Sync`.
  128|       |    ///
  129|       |    /// # Type Parameters
  130|       |    ///
  131|       |    /// - `C`: Default-constructible codec encoding `I` and decoding into `V`.
  132|       |    /// - `V`: Owned value type erased by this descriptor and produced by
  133|       |    ///   decode.
  134|       |    /// - `I`: Borrowed input view accepted by the encoder; it may be unsized.
  135|       |    ///
  136|       |    /// # Returns
  137|       |    ///
  138|       |    /// Returns immutable function metadata without constructing a codec.
  139|       |    ///
  140|       |    /// # Errors
  141|       |    ///
  142|       |    /// Encoding returns a type mismatch if the erased value is not `V`, or
  143|       |    /// retains the codec's source error. Decoding retains the decoder's source
  144|       |    /// error.
  145|       |    ///
  146|       |    /// # Examples
  147|       |    ///
  148|       |    /// ```
  149|       |    /// use std::io::Error;
  150|       |    /// use qubit_codec::{ValueDecoder, ValueEncoder, ValueStringCodecDescriptor};
  151|       |    ///
  152|       |    /// #[derive(Default)]
  153|       |    /// struct Utf8;
  154|       |    /// impl ValueEncoder<[u8]> for Utf8 {
  155|       |    ///     type Output = String;
  156|       |    ///     type Error = Error;
  157|       |    ///     fn encode(&mut self, input: &[u8]) -> Result<String, Error> {
  158|       |    ///         String::from_utf8(input.to_vec()).map_err(Error::other)
  159|       |    ///     }
  160|       |    /// }
  161|       |    /// impl ValueDecoder<str> for Utf8 {
  162|       |    ///     type Output = Vec<u8>;
  163|       |    ///     type Error = Error;
  164|       |    ///     fn decode(&mut self, input: &str) -> Result<Vec<u8>, Error> {
  165|       |    ///         Ok(input.as_bytes().to_vec())
  166|       |    ///     }
  167|       |    /// }
  168|       |    ///
  169|       |    /// let descriptor = ValueStringCodecDescriptor::of_borrowed::<Utf8, Vec<u8>, [u8]>();
  170|       |    /// let value = b"hello".to_vec();
  171|       |    /// assert_eq!(descriptor.encode(&value).unwrap(), "hello");
  172|       |    /// let decoded = descriptor.decode("hello").unwrap().downcast::<Vec<u8>>().unwrap();
  173|       |    /// assert_eq!(*decoded, value);
  174|       |    /// ```
  175|       |    ///
  176|       |    /// The owned type must implement [`Borrow`] for the selected input view,
  177|       |    /// and the decoder output must be exactly `V`:
  178|       |    ///
  179|       |    /// ```compile_fail
  180|       |    /// use std::io::Error;
  181|       |    /// use qubit_codec::{ValueDecoder, ValueEncoder, ValueStringCodecDescriptor};
  182|       |    ///
  183|       |    /// #[derive(Default)] struct Codec;
  184|       |    /// impl ValueEncoder<[u8]> for Codec {
  185|       |    ///     type Output = String; type Error = Error;
  186|       |    ///     fn encode(&mut self, input: &[u8]) -> Result<String, Error> {
  187|       |    ///         String::from_utf8(input.to_vec()).map_err(Error::other)
  188|       |    ///     }
  189|       |    /// }
  190|       |    /// impl ValueDecoder<str> for Codec {
  191|       |    ///     type Output = String; type Error = Error;
  192|       |    ///     fn decode(&mut self, input: &str) -> Result<String, Error> { Ok(input.to_owned()) }
  193|       |    /// }
  194|       |    /// let _ = ValueStringCodecDescriptor::of_borrowed::<Codec, String, [u8]>();
  195|       |    /// ```
  196|       |    ///
  197|       |    /// ```compile_fail
  198|       |    /// use std::io::Error;
  199|       |    /// use qubit_codec::{ValueDecoder, ValueEncoder, ValueStringCodecDescriptor};
  200|       |    ///
  201|       |    /// #[derive(Default)] struct Codec;
  202|       |    /// impl ValueEncoder<[u8]> for Codec {
  203|       |    ///     type Output = String; type Error = Error;
  204|       |    ///     fn encode(&mut self, input: &[u8]) -> Result<String, Error> {
  205|       |    ///         String::from_utf8(input.to_vec()).map_err(Error::other)
  206|       |    ///     }
  207|       |    /// }
  208|       |    /// impl ValueDecoder<str> for Codec {
  209|       |    ///     type Output = String; type Error = Error;
  210|       |    ///     fn decode(&mut self, input: &str) -> Result<String, Error> { Ok(input.to_owned()) }
  211|       |    /// }
  212|       |    /// let _ = ValueStringCodecDescriptor::of_borrowed::<Codec, Vec<u8>, [u8]>();
  213|       |    /// ```
  214|       |    #[must_use]
  215|       |    #[inline]
  216|      2|    pub const fn of_borrowed<C, V, I>() -> Self
  217|      2|    where
  218|      2|        C: Default + ValueEncoder<I, Output = String> + ValueDecoder<str, Output = V> + 'static,
  219|      2|        <C as ValueEncoder<I>>::Error: Error + Send + Sync + 'static,
  220|      2|        <C as ValueDecoder<str>>::Error: Error + Send + Sync + 'static,
  221|      2|        V: Borrow<I> + 'static,
  222|      2|        I: ?Sized,
  223|       |    {
  224|      2|        Self {
  225|      2|            codec_type_id: TypeId::of::<C>,
  226|      2|            codec_type_name: type_name::<C>,
  227|      2|            value_type_id: TypeId::of::<V>,
  228|      2|            value_type_name: type_name::<V>,
  229|      2|            encode: encode::<C, V, I>,
  230|      2|            decode: decode::<C, V>,
  231|      2|        }
  232|      2|    }
  233|       |
  234|       |    /// Returns the process-local identity of the concrete codec.
  235|       |    ///
  236|       |    /// # Returns
  237|       |    ///
  238|       |    /// Returns the `TypeId` of the codec this descriptor was built for.
  239|       |    #[must_use]
  240|       |    #[inline]
  241|     14|    pub fn codec_type_id(&self) -> TypeId {
  242|     14|        (self.codec_type_id)()
  243|     14|    }
  244|       |
  245|       |    /// Returns the diagnostic name of the concrete codec.
  246|       |    ///
  247|       |    /// # Returns
  248|       |    ///
  249|       |    /// Returns the `type_name` of the codec, for diagnostics only.
  250|       |    #[must_use]
  251|       |    #[inline]
  252|      6|    pub fn codec_type_name(&self) -> &'static str {
  253|      6|        (self.codec_type_name)()
  254|      6|    }
  255|       |
  256|       |    /// Returns the process-local identity of the decoded value.
  257|       |    ///
  258|       |    /// # Returns
  259|       |    ///
  260|       |    /// Returns the `TypeId` of the value this descriptor accepts and produces.
  261|       |    #[must_use]
  262|       |    #[inline]
  263|      8|    pub fn value_type_id(&self) -> TypeId {
  264|      8|        (self.value_type_id)()
  265|      8|    }
  266|       |
  267|       |    /// Returns the diagnostic name of the decoded value.
  268|       |    ///
  269|       |    /// # Returns
  270|       |    ///
  271|       |    /// Returns the `type_name` of the value, for diagnostics only.
  272|       |    #[must_use]
  273|       |    #[inline]
  274|      6|    pub fn value_type_name(&self) -> &'static str {
  275|      6|        (self.value_type_name)()
  276|      6|    }
  277|       |
  278|       |    /// Encodes one erased value after checking its concrete type.
  279|       |    ///
  280|       |    /// # Parameters
  281|       |    ///
  282|       |    /// - `value`: Borrowed value whose concrete type must match the descriptor.
  283|       |    ///
  284|       |    /// # Returns
  285|       |    ///
  286|       |    /// Returns an owned string produced by a fresh default codec.
  287|       |    ///
  288|       |    /// # Errors
  289|       |    ///
  290|       |    /// Returns a type mismatch or the typed encoder source error.
  291|       |    #[inline]
  292|     28|    pub fn encode(&self, value: &dyn Any) -> Result<String, ValueCodecExecutionError> {
  293|     28|        (self.encode)(value)
  294|     28|    }
  295|       |
  296|       |    /// Decodes one string into a safely erased value of the declared type.
  297|       |    ///
  298|       |    /// # Parameters
  299|       |    ///
  300|       |    /// - `input`: String borrowed for one decoding operation.
  301|       |    ///
  302|       |    /// # Returns
  303|       |    ///
  304|       |    /// Returns a boxed owned value that can be downcast to the declared type.
  305|       |    ///
  306|       |    /// # Errors
  307|       |    ///
  308|       |    /// Returns the typed decoder source error.
  309|       |    #[inline]
  310|     20|    pub fn decode(&self, input: &str) -> Result<Box<dyn Any>, ValueCodecExecutionError> {
  311|     20|        (self.decode)(input)
  312|     20|    }
  313|       |}
  314|       |
  315|       |impl Debug for ValueStringCodecDescriptor {
  316|       |    /// Formats descriptor metadata without invoking either codec function.
  317|       |    ///
  318|       |    /// # Parameters
  319|       |    ///
  320|       |    /// - `formatter`: Destination receiving the diagnostic type names.
  321|       |    ///
  322|       |    /// # Returns
  323|       |    ///
  324|       |    /// Returns success after both names have been formatted.
  325|       |    ///
  326|       |    /// # Errors
  327|       |    ///
  328|       |    /// Returns the underlying formatting failure.
  329|      3|    fn fmt(&self, formatter: &mut Formatter<'_>) -> FmtResult {
  330|      3|        formatter
  331|      3|            .debug_struct("ValueStringCodecDescriptor")
  332|      3|            .field("codec_type_name", &self.codec_type_name())
  333|      3|            .field("value_type_name", &self.value_type_name())
  334|      3|            .finish_non_exhaustive()
  335|      3|    }
  336|       |}
  337|       |
  338|       |/// Downcasts an erased value and encodes it with a fresh default codec.
  339|       |///
  340|       |/// # Type Parameters
  341|       |///
  342|       |/// - `C`: Default codec encoding the selected borrowed view.
  343|       |/// - `V`: Concrete owned value type required by the descriptor.
  344|       |/// - `I`: Borrowed input view accepted by the encoder.
  345|       |///
  346|       |/// # Parameters
  347|       |///
  348|       |/// - `value`: Erased input borrowed until serialization completes.
  349|       |///
  350|       |/// # Returns
  351|       |///
  352|       |/// Returns the owned string produced by the codec.
  353|       |///
  354|       |/// # Errors
  355|       |///
  356|       |/// Returns `TypeMismatch` when the input has a different concrete type, or
  357|       |/// `EncodeFailed` with the codec source error when encoding fails.
  358|     28|fn encode<C, V, I>(value: &dyn Any) -> Result<String, ValueCodecExecutionError>
  359|     28|where
  360|     28|    C: Default + ValueEncoder<I, Output = String>,
  361|     28|    <C as ValueEncoder<I>>::Error: Error + Send + Sync + 'static,
  362|     28|    V: Borrow<I> + 'static,
  363|     28|    I: ?Sized,
  364|       |{
  365|     28|    let value = value
                      ^24
  366|     28|        .downcast_ref::<V>()
  367|     28|        .ok_or_else(|| ValueCodecExecutionError::TypeMismatch {
  368|      4|            expected_type: type_name::<V>(),
  369|      4|            actual_type: value.type_id(),
  370|      4|        })?;
  371|     24|    <C as ValueEncoder<I>>::encode(&mut C::default(), <V as Borrow<I>>::borrow(value)).map_err(|source| {
                                                                                                                      ^3
  372|      3|        ValueCodecExecutionError::EncodeFailed {
  373|      3|            codec_type: type_name::<C>(),
  374|      3|            source: Box::new(source),
  375|      3|        }
  376|      3|    })
  377|     28|}
  378|       |
  379|       |/// Decodes one string with a fresh default codec and erases its typed output.
  380|       |///
  381|       |/// # Type Parameters
  382|       |///
  383|       |/// - `C`: Default codec with matching string encode/decode contracts.
  384|       |/// - `V`: Concrete owned value type produced by the descriptor.
  385|       |///
  386|       |/// # Parameters
  387|       |///
  388|       |/// - `input`: String slice borrowed until decoding completes.
  389|       |///
  390|       |/// # Returns
  391|       |///
  392|       |/// Returns the decoded value in a box that can be downcast to `V`.
  393|       |///
  394|       |/// # Errors
  395|       |///
  396|       |/// Returns `DecodeFailed` with the codec source error when decoding fails.
  397|     20|fn decode<C, V>(input: &str) -> Result<Box<dyn Any>, ValueCodecExecutionError>
  398|     20|where
  399|     20|    C: Default + ValueDecoder<str, Output = V>,
  400|     20|    <C as ValueDecoder<str>>::Error: Error + Send + Sync + 'static,
  401|     20|    V: 'static,
  402|       |{
  403|     20|    C::default()
  404|     20|        .decode(input)
  405|     20|        .map(|value| Box::new(value) as Box<dyn Any>)
                                   ^15      ^15       ^15
  406|     20|        .map_err(|source| ValueCodecExecutionError::DecodeFailed {
  407|      5|            codec_type: type_name::<C>(),
  408|      5|            source: Box::new(source),
  409|      5|        })
  410|     20|}
```

</details>

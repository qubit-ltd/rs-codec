// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Low-level value codec trait.

use core::num::NonZeroUsize;

use super::decode_failure::DecodeFailure;

/// Encodes and decodes one value or codec quantum against a unit buffer.
///
/// `Codec` is the lowest-level abstraction in the codec stack. It is intended
/// for hot paths that have already validated buffer capacity and want to avoid
/// constructing subslices for every value. Higher-level transcoders and
/// convenience APIs are responsible for checked buffer management and owned
/// output allocation.
///
/// `MIN_UNITS_PER_VALUE` and `MAX_DECODE_UNITS_PER_VALUE` describe the readable
/// representation-width bounds for one decoded value. If fewer than the
/// minimum units are available, no complete value can exist, so a streaming
/// caller can request more input or report an incomplete EOF tail. The minimum
/// is also the smallest safety precondition checked callers must satisfy before
/// entering [`decode`](Self::decode). The decode maximum bounds successful
/// consumption and incomplete-input requirements for one value.
///
/// `MAX_ENCODE_UNITS_PER_VALUE` is the independent value-agnostic upper bound
/// for main-phase encode output. Reserving that many units makes encoding safe
/// for every encodable value. Once a value and encode state are known, callers
/// may instead reserve the exact [`encode_len`](Self::encode_len), which must
/// not exceed the encode maximum. Complete-lifecycle adapters that still need
/// to run [`encode_reset`](Self::encode_reset) must reserve the maximum first
/// because reset may change the exact width.
///
/// A codec may keep decode-side and encode-side stream state. That state is an
/// implementation detail owned by the codec. Callers do not snapshot or restore
/// it; implementations must keep their own state internally consistent across
/// every public operation, including operations that return `Err`.
///
/// Decode operations see only the currently supplied input slice and codec
/// state. They do not receive an explicit EOF marker and they cannot look past
/// the visible input. Returning [`DecodeFailure::Incomplete`] requests
/// more input for the current value; it is not itself an EOF error. The default
/// codec-backed streaming adapters therefore fit formats whose value boundary
/// is locally decidable from the visible prefix plus codec state. Formats that
/// require EOF-aware maximal-munch parsing, delayed boundary decisions, or
/// reinterpretation of an incomplete tail at EOF should put that policy in a
/// custom [`crate::Transcoder`] or value-level facade instead of relying on the
/// default `Codec` bridge.
///
/// # Associated Types
///
/// - `Value`: Logical value decoded from or encoded into the buffer. This may
///   be a scalar such as `u8`, `u16`, `u64`, a `char`, a fixed quantum such as
///   `[u8; 3]`, or an owned value such as `String`/`Vec<u8>`. Adapters that
///   need scratch initialization add their own bounds at the use site.
/// - `Unit`: Buffer unit used by the encoded representation. Implementations
///   are typically scalar storage units such as `u8`, `u16`, or `char`.
///   Adapters that allocate owned output add their own initialization bounds at
///   the use site.
///
/// Implementors must uphold the safety contract documented by
/// [`decode`](Self::decode), [`encode`](Self::encode),
/// [`encode_reset`](Self::encode_reset), and
/// [`decode_finish`](Self::decode_finish). Unchecked implementations must not
/// read or write outside the caller-provided ranges. Implementations should use
/// `debug_assert!` to state the expected buffer bounds at the unchecked entry
/// point.
///
/// Implementations must also guarantee that
/// [`MIN_UNITS_PER_VALUE`](Self::MIN_UNITS_PER_VALUE) is non-zero and no larger
/// than [`MAX_DECODE_UNITS_PER_VALUE`](Self::MAX_DECODE_UNITS_PER_VALUE), which
/// must also be non-zero. `MAX_ENCODE_UNITS_PER_VALUE` may be zero for a codec
/// that always buffers each accepted value instead of immediately emitting
/// units. Checked adapters enforce the decode-bound invariants with
/// compile-time assertions before using codec-provided bounds.
pub trait Codec {
    /// The type of logical values decoded from or encoded into the buffer.
    type Value;

    /// The type of buffer units used by the encoded representation.
    type Unit;

    /// The type of errors reported when decoding malformed units.
    type DecodeError;

    /// The type of errors reported when encoding an unsupported value.
    type EncodeError;

    /// The minimum possible non-zero readable unit count for one decoded value.
    ///
    /// This is a lower bound used by checked callers for planning and fast
    /// impossibility checks. If a streaming decoder has fewer than this many
    /// readable units, no complete value can be present at the current
    /// position. It is also the denominator used by default decode hooks when
    /// estimating the maximum number of values that can be produced from an
    /// input unit count.
    const MIN_UNITS_PER_VALUE: usize;

    /// The maximum unit count emitted when encoding one value.
    ///
    /// This is a value-independent upper bound for the main encode phase. When
    /// no concrete value is available, reserving this many writable units is
    /// sufficient for any encodable value, without first calling
    /// [`encode_len`](Self::encode_len). For a known value, callers may reserve
    /// only the exact length returned by `encode_len` instead. This bound does
    /// not include output produced by encode reset or finish phases.
    ///
    /// This is also the multiplier used by default encode hooks when estimating
    /// the maximum number of output units needed for an input value count.
    ///
    /// The bound may be zero for a codec that buffers every accepted value and
    /// emits retained output only during a later lifecycle phase.
    const MAX_ENCODE_UNITS_PER_VALUE: usize;

    /// The maximum non-zero unit count consumed to decode one value.
    ///
    /// This is a value-independent upper bound for one complete encoded value
    /// or codec quantum on the decode side. Successful decode consumption and
    /// the `required_total` carried by [`DecodeFailure::Incomplete`] must not
    /// exceed this bound.
    const MAX_DECODE_UNITS_PER_VALUE: usize;

    /// The maximum unit count emitted when resetting encode state.
    ///
    /// This bound covers only output written by
    /// [`encode_reset`](Self::encode_reset). It does not include hook-owned
    /// reset output or any ordinary encoded input values. It must cover every
    /// reachable pre-reset encode state. Stateless codecs should use the
    /// default `0`.
    const MAX_ENCODE_RESET_UNITS: usize = 0;

    /// The maximum unit count emitted when finishing encode state at EOF.
    ///
    /// This bound covers only output written by
    /// [`encode_finish`](Self::encode_finish). It does not include hook-owned
    /// finish output and must cover every reachable encode state, even when the
    /// current stream has no pending finish output. Stateless codecs should use
    /// the default `0`. Codecs that emit a stream trailer (padding, checksum,
    /// or end-of-stream marker) should override this with the exact maximum
    /// unit count.
    const MAX_ENCODE_FINISH_UNITS: usize = 0;

    /// The maximum value count emitted when resetting decode state.
    ///
    /// This bound covers only values written by
    /// [`decode_reset`](Self::decode_reset). It does not include hook-owned
    /// reset output or values decoded from source units. It must cover every
    /// reachable pre-reset decode state. Stateless codecs should use the
    /// default `0`. Codecs that emit a stream-start sentinel or BOM on reset
    /// should override this.
    const MAX_DECODE_RESET_VALUES: usize = 0;

    /// The maximum value count emitted when finishing decode state.
    ///
    /// This bound covers only values written by
    /// [`decode_finish`](Self::decode_finish). It does not include hook-owned
    /// finish output and must cover every reachable decode state, even when the
    /// current stream has no pending finish output. Stateless codecs should use
    /// the default `0`.
    const MAX_DECODE_FINISH_VALUES: usize = 0;

    /// The aggregate maximum output count of either decode lifecycle phase.
    ///
    /// This derived bound is the larger of
    /// [`MAX_DECODE_RESET_VALUES`](Self::MAX_DECODE_RESET_VALUES) and
    /// [`MAX_DECODE_FINISH_VALUES`](Self::MAX_DECODE_FINISH_VALUES). It does
    /// not describe storage that preserves both phases simultaneously;
    /// lifecycle-aware APIs use separate reset and finish buffers.
    /// Implementations should not override this constant.
    const MAX_DECODE_LIFECYCLE_VALUES: usize =
        if Self::MAX_DECODE_RESET_VALUES > Self::MAX_DECODE_FINISH_VALUES {
            Self::MAX_DECODE_RESET_VALUES
        } else {
            Self::MAX_DECODE_FINISH_VALUES
        };

    /// Returns whether `value` is in this codec's encodable value domain.
    ///
    /// The default implementation returns `true`, which is correct for codecs
    /// whose [`Value`](Self::Value) type contains only values they can encode.
    /// Codecs whose logical value type is broader than their representation
    /// domain, such as an ASCII codec with `Value = char`, must override this
    /// method.
    ///
    /// Checked encoder adapters call this method in the same codec state used
    /// to query [`encode_len`](Self::encode_len) and enter the unsafe
    /// [`encode`](Self::encode) method. Complete-lifecycle adapters do so after
    /// [`encode_reset`](Self::encode_reset). Direct unsafe callers must uphold
    /// the same ordering.
    ///
    /// # Parameters
    ///
    /// - `value`: Value whose encodability is queried.
    ///
    /// # Returns
    ///
    /// Returns `true` when `value` may be passed to
    /// [`encode_len`](Self::encode_len) and [`encode`](Self::encode).
    #[inline(always)]
    #[must_use]
    fn can_encode_value(&self, _value: &Self::Value) -> bool {
        true
    }

    /// Returns the exact unit count this codec will write when encoding
    /// `value`.
    ///
    /// The default implementation returns
    /// [`MAX_ENCODE_UNITS_PER_VALUE`](Self::MAX_ENCODE_UNITS_PER_VALUE). This
    /// default is correct only when every successful main encode writes exactly
    /// that many units in the relevant codec state, as fixed-width codecs
    /// normally do.
    ///
    /// Variable-width, buffering, or otherwise state-dependent codecs must
    /// override this method to report the true encoded length for encodable
    /// `value`s. The return value is never an estimate or an "unknown" marker:
    /// it must equal the unit count [`encode`](Self::encode) subsequently
    /// writes for the same `value` under the same codec state. It must also
    /// never exceed
    /// [`MAX_ENCODE_UNITS_PER_VALUE`](Self::MAX_ENCODE_UNITS_PER_VALUE).
    /// Callers that do not yet have a concrete value should use
    /// `MAX_ENCODE_UNITS_PER_VALUE` directly instead of calling this method.
    ///
    /// Default codec-backed encoders use this exact value for per-value output
    /// capacity. The contract requires callers to use this method only when
    /// [`can_encode_value`](Self::can_encode_value) returned `true` for the
    /// same `value`.
    ///
    /// # Parameters
    ///
    /// - `value`: Value whose encoded length is queried.
    ///
    /// # Returns
    ///
    /// Returns the unit count [`encode`](Self::encode) will write for an
    /// encodable `value`.
    ///
    /// A return value of `0` is valid for stateful encoders that accept a
    /// value into internal state without immediately emitting units. Examples
    /// include codecs that aggregate values into fixed-size quanta, shift-state
    /// encodings, or framing layers that defer output until enough values have
    /// been seen or EOF is finalized. Stateless and directly value-to-unit
    /// codecs should continue returning a positive length.
    #[inline(always)]
    #[must_use]
    fn encode_len(&self, _value: &Self::Value) -> usize {
        Self::MAX_ENCODE_UNITS_PER_VALUE
    }

    /// Emits stream-start output and resets encode-side state.
    ///
    /// # Parameters
    ///
    /// - `output`: Destination unit buffer.
    /// - `output_index`: Start index in `output`.
    ///
    /// # Returns
    ///
    /// Returns the number of reset units written.
    ///
    /// # Errors
    ///
    /// Returns `Self::EncodeError` when reset output cannot be emitted.
    /// Implementations must leave their internal state consistent when
    /// returning an error.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the implementation can write up to
    /// [`MAX_ENCODE_RESET_UNITS`](Self::MAX_ENCODE_RESET_UNITS) units starting
    /// at `output_index`.
    #[inline(always)]
    #[must_use = "reset output and reset errors must be handled"]
    unsafe fn encode_reset(
        &mut self,
        _output: &mut [Self::Unit],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Ok(0)
    }

    /// Encodes one borrowed value into `output` starting at `output_index`.
    ///
    /// # Parameters
    ///
    /// - `value`: Value to encode.
    /// - `output`: Destination unit buffer.
    /// - `output_index`: Start index in `output`.
    ///
    /// # Returns
    ///
    /// Returns the number of written units.
    ///
    /// A successful encode may return `0` when the codec accepted `value` into
    /// internal encode-side state but intentionally deferred output. This is
    /// the normal shape for accumulative encoders such as base64-byte
    /// aggregators, shift-state encodings, or frame builders. In that case
    /// [`encode_len`](Self::encode_len) for the same `value` and state must
    /// also return `0`, and the retained output must be emitted by a later
    /// successful [`encode`](Self::encode),
    /// [`encode_finish`](Self::encode_finish), or codec-specific facade.
    ///
    /// # Errors
    ///
    /// Returns `Self::EncodeError` for encode-side state or representation
    /// failures other than a value being outside the codec's encodable domain.
    /// Checked callers reject values for which
    /// [`can_encode_value`](Self::can_encode_value) returns `false` before
    /// entering this unsafe method. Implementations must leave their internal
    /// state consistent when returning an error.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that
    /// [`can_encode_value`](Self::can_encode_value) returned `true` for
    /// `value`. Starting at `output_index`, the caller must also provide either
    /// at least
    /// [`MAX_ENCODE_UNITS_PER_VALUE`](Self::MAX_ENCODE_UNITS_PER_VALUE)
    /// writable
    /// units, which is sufficient for every encodable value, or at least the
    /// exact [`encode_len`](Self::encode_len) for the same `value` and codec
    /// state. On success, implementations must return that exact written unit
    /// count, including `0` for deliberate buffering, and the count must be no
    /// larger than `MAX_ENCODE_UNITS_PER_VALUE`. Implementations must not write
    /// beyond the exact `encode_len` capacity on either the success or error
    /// path.
    #[must_use = "encoded length and encode errors must be handled"]
    unsafe fn encode(
        &mut self,
        value: &Self::Value,
        output: &mut [Self::Unit],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError>;

    /// Emits EOF trailer output and finishes encode-side state.
    ///
    /// This is the encode-side counterpart of
    /// [`decode_finish`](Self::decode_finish). Codecs that append stream
    /// trailers (padding, checksums, end-of-stream markers) emit them here.
    /// Stateless codecs use the default no-op.
    ///
    /// # Parameters
    ///
    /// - `output`: Destination unit buffer.
    /// - `output_index`: Start index in `output`.
    ///
    /// # Returns
    ///
    /// Returns the number of finish units written.
    ///
    /// # Errors
    ///
    /// Returns `Self::EncodeError` when finish output cannot be emitted.
    /// Implementations must leave their internal state consistent when
    /// returning an error.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the implementation can write up to
    /// [`MAX_ENCODE_FINISH_UNITS`](Self::MAX_ENCODE_FINISH_UNITS) units
    /// starting at `output_index`.
    #[inline(always)]
    #[must_use = "finish output and finish errors must be handled"]
    unsafe fn encode_finish(
        &mut self,
        _output: &mut [Self::Unit],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Ok(0)
    }

    /// Emits stream-start values and resets decode-side state.
    ///
    /// This is the decode-side counterpart of
    /// [`encode_reset`](Self::encode_reset). Codecs that emit a
    /// stream-start marker or BOM before decoding a new stream emit them
    /// here. Stateless codecs use the default no-op.
    ///
    /// # Parameters
    ///
    /// - `output`: Destination value buffer.
    /// - `output_index`: Start index in `output`.
    ///
    /// # Returns
    ///
    /// Returns the number of reset values written.
    ///
    /// # Errors
    ///
    /// Returns `Self::DecodeError` when reset output cannot be emitted.
    /// Implementations must leave their internal state consistent when
    /// returning an error.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the implementation can write up to
    /// [`MAX_DECODE_RESET_VALUES`](Self::MAX_DECODE_RESET_VALUES) values
    /// starting at `output_index`.
    #[inline(always)]
    #[must_use = "reset output and reset errors must be handled"]
    unsafe fn decode_reset(
        &mut self,
        _output: &mut [Self::Value],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Ok(0)
    }

    /// Decodes one value from `input` starting at `input_index`.
    ///
    /// # Parameters
    ///
    /// - `input`: Source unit buffer.
    /// - `input_index`: Start index in `input`.
    ///
    /// # Returns
    ///
    /// Returns the decoded value and the non-zero number of consumed units.
    ///
    /// # Errors
    ///
    /// Returns [`DecodeFailure::Incomplete`] when the visible input is a
    /// valid prefix but more units are needed to decide or complete a value.
    /// This reports a streaming boundary, not a final EOF condition; the
    /// caller or higher-level adapter decides what an incomplete tail means
    /// when the upstream source is closed.
    /// Returns [`DecodeFailure::Invalid`] when the units are malformed,
    /// non-canonical, unmappable, or otherwise invalid for this codec and the
    /// span is known or unknown. Unknown span is represented as
    /// [`DecodeFailure::Invalid`] with an unknown consumed-unit hint.
    /// Implementations must leave their internal state consistent when
    /// returning an error.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `input_index` is a valid boundary in
    /// `input` and that at least
    /// [`MIN_UNITS_PER_VALUE`](Self::MIN_UNITS_PER_VALUE)
    /// units are readable from `input_index`. Implementations must not read
    /// beyond the currently available units under that precondition. They
    /// may return [`DecodeFailure::Incomplete`] when those units are a
    /// valid but incomplete prefix.
    ///
    /// On success, implementations must return a consumed unit count no larger
    /// than either the available input or
    /// [`MAX_DECODE_UNITS_PER_VALUE`](Self::MAX_DECODE_UNITS_PER_VALUE). An
    /// incomplete result's `required_total` must also not exceed that maximum.
    /// The return type guarantees that successful decoding always consumes at
    /// least one unit. Implementations should use `debug_assert!` to state
    /// these unchecked entry-point assumptions.
    #[must_use = "decoded value, consumed length, and decode errors must be handled"]
    unsafe fn decode(
        &mut self,
        input: &[Self::Unit],
        input_index: usize,
    ) -> Result<(Self::Value, NonZeroUsize), DecodeFailure<Self::DecodeError>>;

    /// Finishes decode-side EOF state into `output`.
    ///
    /// `decode_finish` receives no source input. Callers must have already
    /// handled any tail reported by [`DecodeFailure::Incomplete`] before
    /// finishing decode state. Implementations may emit retained values or
    /// validate internal EOF state, but they must not depend on re-reading the
    /// incomplete source tail.
    ///
    /// # Parameters
    ///
    /// - `output`: Destination value buffer.
    /// - `output_index`: Start index in `output`.
    ///
    /// # Returns
    ///
    /// Returns the number of finished values written.
    ///
    /// # Errors
    ///
    /// Returns `Self::DecodeError` when retained decode state is invalid at
    /// EOF. Implementations must leave their internal state consistent when
    /// returning an error.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the implementation can write up to
    /// [`MAX_DECODE_FINISH_VALUES`](Self::MAX_DECODE_FINISH_VALUES) values
    /// starting at `output_index`.
    #[inline(always)]
    #[must_use = "finish output length and finish errors must be handled"]
    unsafe fn decode_finish(
        &mut self,
        _output: &mut [Self::Value],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Ok(0)
    }
}

/// Compile-time asserts the public unit-bound invariants required by [`Codec`].
///
/// # Type Parameters
///
/// - `C`: Codec implementation to validate.
///
/// # Returns
///
/// Returns unit `()`.
///
/// # Panics
///
/// Panics at compile time when either decode width bound is zero or when
/// [`Codec::MIN_UNITS_PER_VALUE`] is greater than
/// [`Codec::MAX_DECODE_UNITS_PER_VALUE`], because these invariants must hold
/// for any well-formed [`Codec`] implementation and violating them is always a
/// bug.
#[inline(always)]
pub(crate) fn assert_unit_bounds<C>()
where
    C: Codec,
{
    const {
        assert!(
            C::MIN_UNITS_PER_VALUE > 0,
            "Codec::MIN_UNITS_PER_VALUE must be non-zero",
        );
        assert!(
            C::MAX_DECODE_UNITS_PER_VALUE > 0,
            "Codec::MAX_DECODE_UNITS_PER_VALUE must be non-zero",
        );
        assert!(
            C::MIN_UNITS_PER_VALUE <= C::MAX_DECODE_UNITS_PER_VALUE,
            "Codec::MIN_UNITS_PER_VALUE must not exceed Codec::MAX_DECODE_UNITS_PER_VALUE",
        );
    }
}

/// Validates the declared bounds for a complete decode lifecycle.
///
/// # Type Parameters
///
/// - `C`: Codec whose decode lifecycle bounds are validated.
///
/// # Returns
///
/// Returns unit after verifying that [`Codec::MAX_DECODE_LIFECYCLE_VALUES`] is
/// the larger of the codec's reset and finish output bounds.
///
/// # Panics
///
/// Panics when the codec overrides the derived lifecycle bound with a value
/// that does not match its reset and finish bounds.
#[inline(always)]
pub(crate) fn assert_decode_lifecycle_bounds<C>()
where
    C: Codec,
{
    assert_unit_bounds::<C>();
    let expected = C::MAX_DECODE_RESET_VALUES.max(C::MAX_DECODE_FINISH_VALUES);
    assert_eq!(
        C::MAX_DECODE_LIFECYCLE_VALUES,
        expected,
        "Codec::MAX_DECODE_LIFECYCLE_VALUES must match its lifecycle bounds",
    );
}

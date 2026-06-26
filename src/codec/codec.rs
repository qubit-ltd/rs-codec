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
/// `MIN_UNITS_PER_VALUE` and `MAX_UNITS_PER_VALUE` describe the representation
/// width bounds for one value. The minimum is a lower-bound hint for checked
/// layers: if fewer than this many units are available, no complete value can
/// exist, so a streaming caller can request more input or report an incomplete
/// EOF tail. For decoding, this minimum is the smallest safety precondition
/// checked callers must satisfy before entering
/// [`decode`](Self::decode). The maximum is a value-independent upper bound
/// callers can use for coarse capacity planning. For encoding a known value,
/// checked callers should reserve the exact [`encode_len`](Self::encode_len)
/// instead of pessimistically reserving the maximum width.
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
/// [`decode_flush`](Self::decode_flush). Unchecked implementations must not
/// read or write outside the caller-provided ranges. Implementations should use
/// `debug_assert!` to state the expected buffer bounds at the unchecked entry
/// point.
///
/// Implementations must also guarantee that
/// [`MIN_UNITS_PER_VALUE`](Self::MIN_UNITS_PER_VALUE) is less than or equal to
/// [`MAX_UNITS_PER_VALUE`](Self::MAX_UNITS_PER_VALUE). Both bounds are non-zero
/// by type, and `MAX_UNITS_PER_VALUE` must be a valid upper bound for one
/// complete encoded value or codec quantum. Checked adapters assert this
/// invariant before using codec-provided bounds.
pub trait Codec {
    /// The type of logical values decoded from or encoded into the buffer.
    type Value;

    /// The type of buffer units used by the encoded representation.
    type Unit;

    /// The type of errors reported when decoding malformed units.
    type DecodeError;

    /// The type of errors reported when encoding an unsupported value.
    type EncodeError;

    /// The minimum possible unit count for one encoded value.
    ///
    /// This is a lower bound used by checked callers for planning and fast
    /// impossibility checks. If a streaming decoder has fewer than this many
    /// readable units, no complete value can be present at the current
    /// position. It is also the denominator used by default decode hooks when
    /// estimating the maximum number of values that can be produced from an
    /// input unit count.
    const MIN_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;

    /// The maximum non-zero unit count needed to encode or decode one value.
    ///
    /// This is a value-independent upper bound for one complete encoded value
    /// or codec quantum. It is the multiplier used by default encode hooks
    /// when estimating the maximum number of output units needed for an input
    /// value count.
    const MAX_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;

    /// The maximum unit count emitted when resetting encode state.
    ///
    /// This bound covers only output written by
    /// [`encode_reset`](Self::encode_reset). It does not include hook-owned
    /// reset output or any ordinary encoded input values. Stateless codecs
    /// should use the default `0`.
    const MAX_ENCODE_RESET_UNITS: usize = 0;

    /// The maximum unit count emitted when flushing encode state at EOF.
    ///
    /// This bound covers only output written by
    /// [`encode_flush`](Self::encode_flush). It does not include hook-owned
    /// finish output. Stateless codecs should use the default `0`. Codecs
    /// that emit a stream trailer (padding, checksum, or end-of-stream marker)
    /// should override this with the exact maximum unit count.
    const MAX_ENCODE_FLUSH_UNITS: usize = 0;

    /// The maximum value count emitted when resetting decode state.
    ///
    /// This bound covers only values written by
    /// [`decode_reset`](Self::decode_reset). It does not include hook-owned
    /// reset output or values decoded from source units. Stateless codecs
    /// should use the default `0`. Codecs that emit a stream-start sentinel or
    /// BOM on reset should override this.
    const MAX_DECODE_RESET_VALUES: usize = 0;

    /// The maximum value count emitted when flushing decode state.
    ///
    /// This bound covers only values written by
    /// [`decode_flush`](Self::decode_flush). It does not include hook-owned
    /// finish output. Stateless codecs should use the default `0`.
    const MAX_DECODE_FLUSH_VALUES: usize = 0;

    /// Returns whether `value` is in this codec's encodable value domain.
    ///
    /// The default implementation returns `true`, which is correct for codecs
    /// whose [`Value`](Self::Value) type contains only values they can encode.
    /// Codecs whose logical value type is broader than their representation
    /// domain, such as an ASCII codec with `Value = char`, must override this
    /// method.
    ///
    /// Checked encoder adapters call this method before querying
    /// [`encode_len`](Self::encode_len) or entering the unsafe
    /// [`encode`](Self::encode) method. Direct unsafe callers must do the same.
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

    /// Returns the exact non-zero unit count this codec will write when
    /// encoding `value`.
    ///
    /// The default implementation returns
    /// [`MAX_UNITS_PER_VALUE`](Self::MAX_UNITS_PER_VALUE), which is the
    /// conservative bound callers can use when no specific value is available.
    /// Fixed-width codecs do not need to override this method.
    ///
    /// Variable-width codecs (LEB128, UTF-8, GB18030, …) should override this
    /// to report the true encoded length for encodable `value`s. Doing so lets
    /// buffered adapters and stream writers reserve only what is actually
    /// needed and enables capacity probing without performing the encode.
    /// Default codec-backed encoders use this exact value for per-value output
    /// capacity. The contract requires callers to use this method only when
    /// [`can_encode_value`](Self::can_encode_value) returned `true` for the
    /// same `value`. Under that precondition, the returned length must equal
    /// the unit count [`encode`](Self::encode) writes for the same `value`
    /// under the same codec state, and must never exceed
    /// [`MAX_UNITS_PER_VALUE`](Self::MAX_UNITS_PER_VALUE).
    ///
    /// # Parameters
    ///
    /// - `value`: Value whose encoded length is queried.
    ///
    /// # Returns
    ///
    /// Returns the non-zero unit count [`encode`](Self::encode) will write for
    /// an encodable `value`.
    #[inline(always)]
    #[must_use]
    fn encode_len(&self, _value: &Self::Value) -> NonZeroUsize {
        Self::MAX_UNITS_PER_VALUE
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
    /// Returns the non-zero number of written units. A successful encode
    /// always emits at least one unit; stateful encoders that need to defer
    /// output should report that intent through a custom encode error
    /// instead of returning a zero count.
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
    /// `value`, and that the implementation can write at least
    /// [`encode_len`](Self::encode_len) units for the same `value` and codec
    /// state starting at `output_index`. On success, implementations must
    /// return that exact written unit count, and the count must be no
    /// larger than [`MAX_UNITS_PER_VALUE`](Self::MAX_UNITS_PER_VALUE).
    #[must_use = "encoded length and encode errors must be handled"]
    unsafe fn encode(
        &mut self,
        value: &Self::Value,
        output: &mut [Self::Unit],
        output_index: usize,
    ) -> Result<NonZeroUsize, Self::EncodeError>;

    /// Emits EOF trailer output and flushes encode-side state.
    ///
    /// This is the encode-side counterpart of
    /// [`decode_flush`](Self::decode_flush). Codecs that append stream
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
    /// Returns the number of flush units written.
    ///
    /// # Errors
    ///
    /// Returns `Self::EncodeError` when flush output cannot be emitted.
    /// Implementations must leave their internal state consistent when
    /// returning an error.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the implementation can write up to
    /// [`MAX_ENCODE_FLUSH_UNITS`](Self::MAX_ENCODE_FLUSH_UNITS) units starting
    /// at `output_index`.
    #[inline(always)]
    #[must_use = "flush output and flush errors must be handled"]
    unsafe fn encode_flush(
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
    /// non-canonical, unmappable, or otherwise invalid for this codec. The
    /// concrete error type carries only codec-domain invalidity.
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
    /// than the available input. The return type guarantees that successful
    /// decoding always consumes at least one unit. Implementations should use
    /// `debug_assert!` to state these unchecked entry-point assumptions.
    #[must_use = "decoded value, consumed length, and decode errors must be handled"]
    unsafe fn decode(
        &mut self,
        input: &[Self::Unit],
        input_index: usize,
    ) -> Result<(Self::Value, NonZeroUsize), DecodeFailure<Self::DecodeError>>;

    /// Flushes decode-side EOF state into `output`.
    ///
    /// `decode_flush` receives no source input. Callers must have already
    /// handled any tail reported by [`DecodeFailure::Incomplete`] before
    /// flushing decode state. Implementations may emit retained values or
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
    /// Returns the number of flushed values written.
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
    /// [`MAX_DECODE_FLUSH_VALUES`](Self::MAX_DECODE_FLUSH_VALUES) values
    /// starting at `output_index`.
    #[inline(always)]
    #[must_use = "flush output length and flush errors must be handled"]
    unsafe fn decode_flush(
        &mut self,
        _output: &mut [Self::Value],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Ok(0)
    }
}

/// Compile-time asserts the public unit-bound invariant required by [`Codec`].
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
/// Panics at compile time when [`Codec::MIN_UNITS_PER_VALUE`] is greater than
/// [`Codec::MAX_UNITS_PER_VALUE`], because the invariant must hold for any
/// well-formed [`Codec`] implementation and violating it is always a bug.
#[inline(always)]
pub(crate) fn assert_unit_bounds<C>()
where
    C: Codec,
{
    const {
        assert!(
            C::MIN_UNITS_PER_VALUE.get() <= C::MAX_UNITS_PER_VALUE.get(),
            "Codec::MIN_UNITS_PER_VALUE must not exceed Codec::MAX_UNITS_PER_VALUE",
        );
    }
}

// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Low-level value codec trait.

use core::num::NonZeroUsize;

/// Encodes and decodes one value or codec quantum against a unit buffer.
///
/// `Codec` is the lowest-level abstraction in the codec stack. It is intended
/// for hot paths that have already validated buffer capacity and want to avoid
/// constructing subslices for every value. Higher-level transcoders and
/// convenience APIs are responsible for checked buffer management and owned
/// output allocation.
///
/// `min_units_per_value` and `max_units_per_value` describe the representation
/// width bounds for one value. The minimum is a lower-bound hint for checked
/// layers: if fewer than this many units are available, no complete value can
/// exist, so a streaming caller can request more input, report an incomplete
/// EOF tail. For decoding, this minimum is the smallest safety precondition
/// checked callers must satisfy before entering
/// [`decode`](Self::decode). The maximum is the
/// conservative bound callers normally use to prove that unchecked writes stay
/// inside the provided output buffer.
///
/// A codec may keep decode-side and encode-side stream state. That state is an
/// implementation detail owned by the codec. Callers do not snapshot or restore
/// it; implementations must keep their own state internally consistent across
/// every public operation, including operations that return `Err`.
///
/// # Associated Types
///
/// - `Value`: Logical value decoded from or encoded into the buffer. This may
///   be a scalar such as `u64`, a `char`, or a fixed quantum such as `[u8; 3]`.
///   Implementations must provide [`Copy`] and [`Default`] so convenience
///   adapters can allocate flush scratch buffers.
/// - `Unit`: Buffer unit used by the encoded representation. Implementations
///   must provide [`Copy`] and [`Default`] so convenience adapters can allocate
///   output unit buffers.
///
/// # Safety
///
/// Implementors must uphold the safety contract documented by
/// [`decode`](Self::decode), [`encode`](Self::encode),
/// [`encode_reset`](Self::encode_reset), and
/// [`decode_flush`](Self::decode_flush). In particular, unchecked
/// implementations must not read or write outside the caller-provided ranges.
/// Implementations should use `debug_assert!` to state the expected buffer
/// bounds at the unchecked entry point.
///
/// Implementations must also guarantee that
/// [`min_units_per_value`](Self::min_units_per_value) is less than or equal to
/// [`max_units_per_value`](Self::max_units_per_value). Both bounds are non-zero
/// by type, and `max_units_per_value` must be a valid upper bound for one
/// complete encoded value or codec quantum. Checked adapters assert this
/// invariant before using codec-provided bounds.
pub unsafe trait Codec {
    /// The type of logical values decoded from or encoded into the buffer.
    type Value: Copy + Default;

    /// The type of buffer units used by the encoded representation.
    type Unit: Copy + Default;

    /// The type of errors reported when decoding malformed units.
    type DecodeError;

    /// The type of errors reported when encoding an unsupported value.
    type EncodeError;

    /// Returns the minimum possible unit count for one encoded value.
    ///
    /// This is a lower bound used by checked callers for planning and fast
    /// impossibility checks. If a streaming decoder has fewer than this many
    /// readable units, no complete value can be present at the current
    /// position. If the stream has reached EOF, such a tail is necessarily
    /// incomplete; otherwise the caller should read more input. Similarly,
    /// an encoder or transcoder can avoid calling into the codec when the
    /// remaining output capacity is smaller than this lower bound.
    ///
    /// This value does not prove that encoding will fit. For variable-width
    /// representations, a value may require more units, up to
    /// [`max_units_per_value`](Self::max_units_per_value). For decoding, this
    /// is the minimum safety precondition required by
    /// [`decode`](Self::decode); if fewer units are
    /// available, a checked caller must request more input or report a closed
    /// incomplete tail without calling into the unchecked method.
    ///
    /// # Returns
    ///
    /// Returns a non-zero lower bound for one complete value. Variable-width
    /// codecs such as LEB128 should return the shortest valid representation
    /// length. For example, a UTF-16 byte codec can return `2`, while its
    /// maximum is `4` because a surrogate pair needs four bytes.
    #[must_use]
    fn min_units_per_value(&self) -> NonZeroUsize;

    /// Returns the maximum non-zero unit count needed to encode or decode one
    /// value.
    ///
    /// # Returns
    ///
    /// Returns an upper bound for one complete value or codec quantum.
    #[must_use]
    fn max_units_per_value(&self) -> NonZeroUsize;

    /// Returns the exact non-zero unit count this codec will write when
    /// encoding `value`.
    ///
    /// The default implementation returns
    /// [`max_units_per_value`](Self::max_units_per_value), which is the
    /// conservative bound checked callers already use to prove output
    /// capacity. Fixed-width codecs do not need to override this method.
    ///
    /// Variable-width codecs (LEB128, UTF-8, GB18030, …) should override this
    /// to report the true encoded length for `value`. Doing so lets buffered
    /// adapters and stream writers reserve only what is actually needed and
    /// enables capacity probing without performing the encode. The contract
    /// requires the returned length to equal the unit count
    /// [`encode`](Self::encode) writes for the same `value` under the same
    /// codec state, and to never exceed
    /// [`max_units_per_value`](Self::max_units_per_value).
    ///
    /// # Parameters
    ///
    /// - `value`: Value whose encoded length is queried.
    ///
    /// # Returns
    ///
    /// Returns the non-zero unit count [`encode`](Self::encode) will write
    /// for `value`.
    #[must_use]
    #[inline(always)]
    fn encode_len(&self, _value: &Self::Value) -> NonZeroUsize {
        self.max_units_per_value()
    }

    /// Returns the maximum unit count emitted when resetting encode state.
    ///
    /// Stateful encoders may need a stream-start sequence, such as a byte order
    /// mark, before the first encoded value. Buffered encoders use this bound
    /// to reserve output capacity before calling
    /// [`encode_reset`](Self::encode_reset).
    ///
    /// # Returns
    ///
    /// Returns the finite reset-output upper bound. Stateless codecs should
    /// use the default `0`.
    #[must_use]
    #[inline(always)]
    fn max_encode_reset_units(&self) -> usize {
        0
    }

    /// Returns the maximum value count emitted when flushing decode state.
    ///
    /// Stateful decoders may need to produce final values at EOF from retained
    /// state. Buffered decoders use this bound to reserve output capacity
    /// before calling [`decode_flush`](Self::decode_flush).
    ///
    /// # Returns
    ///
    /// Returns the finite flush-output upper bound. Stateless codecs should
    /// use the default `0`.
    #[must_use]
    #[inline(always)]
    fn max_decode_flush_values(&self) -> usize {
        0
    }

    /// Emits stream-start output and resets encode-side state.
    ///
    /// # Parameters
    ///
    /// - `output`: Destination unit buffer.
    /// - `index`: Start index in `output`.
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
    /// [`max_encode_reset_units`](Self::max_encode_reset_units) units starting
    /// at `index`.
    #[inline(always)]
    unsafe fn encode_reset(
        &mut self,
        _output: &mut [Self::Unit],
        _index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Ok(0)
    }

    /// Encodes one borrowed value into `output` starting at `index`.
    ///
    /// # Parameters
    ///
    /// - `value`: Value to encode.
    /// - `output`: Destination unit buffer.
    /// - `index`: Start index in `output`.
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
    /// Returns `Self::EncodeError` when `value` cannot be represented by this
    /// codec. Implementations must leave their internal state consistent when
    /// returning an error.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the implementation can write up to
    /// [`max_units_per_value`](Self::max_units_per_value) units starting at
    /// `index`. On success, implementations must return a written unit count
    /// no larger than [`max_units_per_value`](Self::max_units_per_value) and
    /// must match the value returned by
    /// [`encode_len`](Self::encode_len) for the same `value` and codec state.
    unsafe fn encode(
        &mut self,
        value: &Self::Value,
        output: &mut [Self::Unit],
        index: usize,
    ) -> Result<NonZeroUsize, Self::EncodeError>;

    /// Decodes one value from `input` starting at `index`.
    ///
    /// # Parameters
    ///
    /// - `input`: Source unit buffer.
    /// - `index`: Start index in `input`.
    ///
    /// # Returns
    ///
    /// Returns the decoded value and the non-zero number of consumed units.
    ///
    /// # Errors
    ///
    /// Returns `Self::DecodeError` when the units are malformed, non-canonical,
    /// incomplete, or otherwise invalid for this codec. The concrete error type
    /// carries the codec-specific reason and context. Implementations must
    /// leave their internal state consistent when returning an error.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that `index` is a valid boundary in `input`
    /// and that at least [`min_units_per_value`](Self::min_units_per_value)
    /// units are readable from `index`. Implementations must not read beyond
    /// the currently available units under that precondition. They may
    /// return `Self::DecodeError` when those units are a valid but
    /// incomplete prefix.
    ///
    /// On success, implementations must return a consumed unit count no larger
    /// than the available input. The return type guarantees that successful
    /// decoding always consumes at least one unit. Implementations should use
    /// `debug_assert!` to state these unchecked entry-point assumptions.
    unsafe fn decode(
        &mut self,
        input: &[Self::Unit],
        index: usize,
    ) -> Result<(Self::Value, NonZeroUsize), Self::DecodeError>;

    /// Flushes decode-side EOF state into `output`.
    ///
    /// # Parameters
    ///
    /// - `output`: Destination value buffer.
    /// - `index`: Start index in `output`.
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
    /// [`max_decode_flush_values`](Self::max_decode_flush_values) values
    /// starting at `index`.
    #[inline(always)]
    unsafe fn decode_flush(
        &mut self,
        _output: &mut [Self::Value],
        _index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Ok(0)
    }
}

/// Asserts the public unit-bound invariant required by [`Codec`].
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
/// Panics when [`Codec::min_units_per_value`] is greater than
/// [`Codec::max_units_per_value`].
pub(crate) fn assert_unit_bounds<C>(codec: &C)
where
    C: Codec,
{
    assert!(
        codec.min_units_per_value() <= codec.max_units_per_value(),
        "Codec::min_units_per_value() must not exceed Codec::max_units_per_value()",
    );
}

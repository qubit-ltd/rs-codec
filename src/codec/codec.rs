// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Low-level value codec trait.

use core::num::NonZeroUsize;

use super::{
    CodecDecodeError,
    CodecEncodeError,
};
use crate::CapacityError;

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
    #[must_use]
    #[inline(always)]
    fn can_encode_value(&self, _value: &Self::Value) -> bool {
        true
    }

    /// Returns the exact non-zero unit count this codec will write when
    /// encoding `value`.
    ///
    /// The default implementation returns
    /// [`max_units_per_value`](Self::max_units_per_value), which is the
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
    /// [`max_units_per_value`](Self::max_units_per_value).
    ///
    /// # Parameters
    ///
    /// - `value`: Value whose encoded length is queried.
    ///
    /// # Returns
    ///
    /// Returns the non-zero unit count [`encode`](Self::encode) will write for
    /// an encodable `value`.
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

    /// Returns the maximum unit count emitted by one reset-prefixed value
    /// encode.
    ///
    /// This is the checked sum of
    /// [`max_encode_reset_units`](Self::max_encode_reset_units) and
    /// [`max_units_per_value`](Self::max_units_per_value). It is useful for
    /// callers that want to reuse a scratch buffer for repeated one-value
    /// encodes without manually duplicating capacity arithmetic.
    ///
    /// # Returns
    ///
    /// Returns the maximum reset-plus-value output length.
    ///
    /// # Errors
    ///
    /// Returns [`CapacityError::OutputLengthOverflow`] when the sum cannot be
    /// represented as `usize`.
    #[must_use = "capacity planning can fail on overflow"]
    #[inline(always)]
    fn max_encode_value_units(&self) -> Result<usize, CapacityError> {
        self.max_encode_reset_units()
            .checked_add(self.max_units_per_value().get())
            .ok_or(CapacityError::OutputLengthOverflow)
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
    /// state starting at `index`. On success, implementations must return that
    /// exact written unit count, and the count must be no larger than
    /// [`max_units_per_value`](Self::max_units_per_value).
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

    /// Encodes one value after emitting reset output into a caller buffer.
    ///
    /// The method validates the output index and the combined reset-plus-value
    /// capacity before calling the unchecked codec hooks. It is a convenience
    /// wrapper for code paths that need one complete value and want to reuse
    /// caller-owned storage.
    ///
    /// # Parameters
    ///
    /// - `value`: Value to encode.
    /// - `output`: Destination unit buffer.
    /// - `output_index`: Start index in `output`.
    ///
    /// # Returns
    ///
    /// Returns the total number of reset and value units written.
    ///
    /// # Errors
    ///
    /// Returns [`CodecEncodeError::UnencodableValue`] when `value` is outside
    /// this codec's encodable domain,
    /// [`CodecEncodeError::InvalidOutputIndex`] when `output_index` is outside
    /// `output`, [`CodecEncodeError::InsufficientOutput`] when the writable
    /// suffix cannot hold the reset output plus exact encoded value width,
    /// [`CodecEncodeError::OutputLengthOverflow`] when the bound overflows, or
    /// [`CodecEncodeError::Encode`] when reset or value encoding fails.
    ///
    /// # Panics
    ///
    /// Panics when the codec writes or reports more units than its declared
    /// reset or value bound.
    fn encode_value_with_reset(
        &mut self,
        value: &Self::Value,
        output: &mut [Self::Unit],
        output_index: usize,
    ) -> Result<usize, CodecEncodeError<Self::EncodeError>> {
        if !self.can_encode_value(value) {
            return Err(CodecEncodeError::unencodable_value(0));
        }
        let reset_units = self.max_encode_reset_units();
        let value_units = self.encode_len(value).get();
        let required = reset_units
            .checked_add(value_units)
            .ok_or_else(CodecEncodeError::output_length_overflow)?;
        CodecEncodeError::ensure_output_capacity(
            output.len(),
            output_index,
            required,
        )?;

        let reset_written = unsafe {
            // SAFETY: The capacity check above reserves the combined
            // reset-plus-value output bound at `output_index`.
            self.encode_reset(output, output_index)
        }
        .map_err(|error| CodecEncodeError::encode(error, 0))?;
        assert!(
            reset_written <= reset_units,
            "Codec::encode_reset wrote beyond its reset bound",
        );

        let value_written = unsafe {
            // SAFETY: `reset_written <= reset_units` and the earlier combined
            // capacity check leave the exact value width writable.
            self.encode(value, output, output_index + reset_written)
        }
        .map_err(|error| CodecEncodeError::encode(error, 0))?
        .get();
        assert!(
            value_written == value_units,
            "Codec::encode wrote a different length than Codec::encode_len",
        );
        Ok(reset_written + value_written)
    }

    /// Decodes one value and flushes decode-side state into caller storage.
    ///
    /// The method validates input and flush-output bounds before entering the
    /// unchecked codec hooks. It returns the decoded value, the consumed input
    /// count, and the number of flushed values written to `flush_output`.
    ///
    /// # Parameters
    ///
    /// - `input`: Source unit buffer.
    /// - `input_index`: Start index in `input`.
    /// - `flush_output`: Destination value buffer for decode-flush output.
    /// - `flush_output_index`: Start index in `flush_output`.
    ///
    /// # Returns
    ///
    /// Returns `(value, consumed, flushed)`.
    ///
    /// # Errors
    ///
    /// Returns [`CodecDecodeError::InvalidInputIndex`] when `input_index` is
    /// outside `input`, [`CodecDecodeError::Incomplete`] when fewer than
    /// [`min_units_per_value`](Self::min_units_per_value) units are readable,
    /// [`CodecDecodeError::InvalidOutputIndex`] or
    /// [`CodecDecodeError::InsufficientOutput`] when flush output cannot hold
    /// [`max_decode_flush_values`](Self::max_decode_flush_values), or
    /// [`CodecDecodeError::Decode`] when decoding or flushing fails.
    ///
    /// # Panics
    ///
    /// Panics when the codec consumes beyond available input or flushes more
    /// values than its declared bound.
    #[inline]
    fn decode_value_with_flush(
        &mut self,
        input: &[Self::Unit],
        input_index: usize,
        flush_output: &mut [Self::Value],
        flush_output_index: usize,
    ) -> Result<
        (Self::Value, NonZeroUsize, usize),
        CodecDecodeError<Self::DecodeError>,
    > {
        CodecDecodeError::ensure_input_index(input.len(), input_index)?;
        let min_units = self.min_units_per_value().get();
        CodecDecodeError::ensure_min_input(
            input.len(),
            input_index,
            min_units,
        )?;

        let flush_cap = self.max_decode_flush_values();
        CodecDecodeError::ensure_output_capacity(
            flush_output.len(),
            flush_output_index,
            flush_cap,
        )?;

        let (value, consumed) = unsafe {
            // SAFETY: The input checks above guarantee the minimum readable
            // units required by `Codec::decode`.
            self.decode(input, input_index)
        }
        .map_err(|error| CodecDecodeError::decode(error, input_index))?;
        let available = input.len() - input_index;
        assert!(
            consumed.get() <= available,
            "Codec::decode consumed beyond available input",
        );

        let flushed = unsafe {
            // SAFETY: The flush-output checks above reserve the declared flush
            // output bound at `flush_output_index`.
            self.decode_flush(flush_output, flush_output_index)
        }
        .map_err(|error| {
            CodecDecodeError::decode(error, input_index + consumed.get())
        })?;
        assert!(
            flushed <= flush_cap,
            "Codec::decode_flush wrote beyond its flush bound",
        );
        Ok((value, consumed, flushed))
    }

    /// Decodes exactly one value and then flushes decode-side state.
    ///
    /// Unlike [`decode_value_with_flush`](Self::decode_value_with_flush), this
    /// helper requires the supplied input slice to contain exactly one encoded
    /// value. It validates trailing input before calling
    /// [`decode_flush`](Self::decode_flush), preserving whole-value decoder
    /// semantics while still centralizing flush scratch-buffer handling.
    ///
    /// # Parameters
    ///
    /// - `input`: Source units for exactly one encoded value.
    /// - `flush_output`: Destination value buffer for decode-flush output.
    /// - `flush_output_index`: Start index in `flush_output`.
    ///
    /// # Returns
    ///
    /// Returns `(value, flushed)`.
    ///
    /// # Errors
    ///
    /// Returns [`CodecDecodeError::Incomplete`] when fewer than
    /// [`min_units_per_value`](Self::min_units_per_value) units are available,
    /// [`CodecDecodeError::TrailingInput`] when decode succeeds but leaves
    /// extra units, output-capacity errors for invalid flush storage, or
    /// [`CodecDecodeError::Decode`] when decoding or flushing fails.
    ///
    /// # Panics
    ///
    /// Panics when the codec consumes beyond available input or flushes more
    /// values than its declared bound.
    #[inline]
    fn decode_exact_value_with_flush(
        &mut self,
        input: &[Self::Unit],
        flush_output: &mut [Self::Value],
        flush_output_index: usize,
    ) -> Result<(Self::Value, usize), CodecDecodeError<Self::DecodeError>> {
        let min_units = self.min_units_per_value().get();
        CodecDecodeError::ensure_min_input(input.len(), 0, min_units)?;

        let flush_cap = self.max_decode_flush_values();
        CodecDecodeError::ensure_output_capacity(
            flush_output.len(),
            flush_output_index,
            flush_cap,
        )?;

        let (value, consumed) = unsafe {
            // SAFETY: The input check above guarantees the minimum readable
            // units required by `Codec::decode` at index 0.
            self.decode(input, 0)
        }
        .map_err(|error| CodecDecodeError::decode(error, 0))?;
        assert!(
            consumed.get() <= input.len(),
            "Codec::decode consumed beyond available input",
        );
        CodecDecodeError::ensure_no_trailing_input(
            consumed.get(),
            input.len(),
        )?;

        let flushed = unsafe {
            // SAFETY: The flush-output checks above reserve the declared flush
            // output bound at `flush_output_index`.
            self.decode_flush(flush_output, flush_output_index)
        }
        .map_err(|error| CodecDecodeError::decode(error, consumed.get()))?;
        assert!(
            flushed <= flush_cap,
            "Codec::decode_flush wrote beyond its flush bound",
        );
        Ok((value, flushed))
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

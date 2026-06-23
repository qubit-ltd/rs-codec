// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy hooks used by buffered encoder engines.

use super::super::encode_context::EncodeContext;
use crate::{CapacityError, Codec, EncodeOutcome};

/// Policy hooks for [`crate::TranscodeEncodeEngine`].
///
/// Hooks own policy state, such as replacement or ignore behavior, but not the
/// codec or engine cursor state. The engine passes the codec into hook methods
/// when policy code needs codec metadata or one-value encode operations.
///
/// Implement this trait when a buffered encoder needs policy decisions around
/// individual values while reusing the common engine loop. Examples include
/// rejecting unsupported values with adapter-level context, consuming values
/// without writing output, writing replacement units, resetting hook-owned
/// state in [`before_reset`](Self::before_reset), or emitting final state in
/// [`finish`](Self::finish).
///
/// The engine calls [`encode_value`](Self::encode_value) for the current input
/// value. The hook either consumes that value and reports written output units,
/// or returns [`EncodeOutcome::NeedOutput`] without consuming it. This lets
/// the engine stop with [`crate::TranscodeStatus::NeedOutput`] and resume later
/// at the same input value.
///
/// # Example
///
/// This hook writes each value with the wrapped codec and reports
/// `NeedOutput` when the current output slice cannot fit the value.
///
/// ```rust
/// use core::{
///     convert::Infallible,
///     num::NonZeroUsize,
/// };
/// use qubit_codec::{
///     TranscodeEncodeHooks,
///     Codec,
///     CodecDecodeFailure,
///     CodecEncodeError,
///     EncodeContext,
///     EncodeOutcome,
/// };
///
/// #[derive(Clone, Copy)]
/// struct ByteCodec;
///
/// impl Codec for ByteCodec {
///     type Value = u8;
///     type Unit = u8;
///     type DecodeError = Infallible;
///     type EncodeError = Infallible;
///
///     const MIN_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;
///     const MAX_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;
///
///     unsafe fn decode(
///         &mut self,
///         input: &[u8],
///         index: usize,
///     ) -> Result<(u8, NonZeroUsize), CodecDecodeFailure<Self::DecodeError>> {
///         Ok((input[index], NonZeroUsize::MIN))
///     }
///
///     unsafe fn encode(
///         &mut self,
///         value: &u8,
///         output: &mut [u8],
///         index: usize,
///     ) -> Result<NonZeroUsize, Self::EncodeError> {
///         output[index] = *value;
///         Ok(NonZeroUsize::MIN)
///     }
/// }
///
/// struct StrictHooks;
///
/// impl<C> TranscodeEncodeHooks<C> for StrictHooks
/// where
///     C: Codec,
/// {
///     type Error = CodecEncodeError<C::EncodeError>;
///
///     fn encode_value(
///         &mut self,
///         codec: &mut C,
///         context: EncodeContext<'_, C::Value, C::Unit>,
///     ) -> Result<EncodeOutcome, Self::Error> {
///         let required = C::MAX_UNITS_PER_VALUE;
///         if context.available_output() < required.get() {
///             return Ok(EncodeOutcome::need_output(required));
///         }
///         let written = unsafe {
///             codec.encode(context.input_value, context.output, context.output_index)
///         }
///         .map(NonZeroUsize::get)
///         .map_err(|error| CodecEncodeError::encode(error, context.input_index))?;
///         Ok(EncodeOutcome::consumed(written))
///     }
/// }
/// ```
///
/// # Type Parameters
///
/// - `C`: Low-level codec owned by the engine.
pub trait TranscodeEncodeHooks<C>
where
    C: Codec,
{
    /// Domain error type returned by the buffered encoder policy.
    ///
    /// The error type must also accept codec encode errors so the engine can
    /// report lifecycle failures such as [`Codec::encode_reset`] without a
    /// hook-specific mapping method.
    type Error: From<C::EncodeError>;

    /// Returns the maximum output units needed for `input_len` values.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `input_len`: Number of input values the caller plans to encode.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound derived from the codec's
    /// [`Codec::MAX_UNITS_PER_VALUE`].
    #[inline]
    #[must_use = "capacity planning can fail on overflow"]
    fn max_output_len(&self, _codec: &C, input_len: usize) -> Result<usize, CapacityError> {
        input_len
            .checked_mul(C::MAX_UNITS_PER_VALUE.get())
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    /// Returns an upper bound for units emitted by finishing hook-owned state.
    ///
    /// `finish` never receives more input values. Implementations must only
    /// report output derived from hook-owned state that remains after the
    /// caller has supplied all input.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    ///
    /// # Returns
    ///
    /// Returns the finite final-output upper bound.
    #[inline(always)]
    #[must_use]
    fn max_finish_output_len(&self, _codec: &C) -> usize {
        0
    }

    /// Processes one input value at the current output cursor.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `context`: Encode context containing the input value, input index,
    ///   output slice, and output cursor.
    ///
    /// # Returns
    ///
    /// Returns whether the current input value was consumed or needs more
    /// output capacity.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when the policy rejects the value or the wrapped
    /// codec fails while writing it.
    fn encode_value(
        &mut self,
        codec: &mut C,
        context: EncodeContext<'_, C::Value, C::Unit>,
    ) -> Result<EncodeOutcome, Self::Error>;

    /// Runs hook-owned cleanup before stream reset.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    #[inline(always)]
    fn before_reset(&mut self, _codec: &mut C) {}

    /// Finishes hook-owned state and writes any retained output units.
    ///
    /// The default implementation is a no-op for stateless encode hooks.
    /// Stateful hooks may emit final units such as reset sequences, checksums,
    /// or trailers. The caller must provide at least
    /// [`TranscodeEncodeHooks::max_finish_output_len`] writable units from
    /// `output_index`. Implementations must not write beyond that declared
    /// final-output bound.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `output`: Output unit slice visible to the hook.
    /// - `output_index`: Absolute output unit index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of units written by finalization. This count must not
    /// exceed [`TranscodeEncodeHooks::max_finish_output_len`].
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when hook-owned state cannot be finalized.
    #[inline(always)]
    fn finish(
        &mut self,
        _codec: &mut C,
        _output: &mut [C::Unit],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

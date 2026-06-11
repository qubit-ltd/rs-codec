// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Policy hooks used by buffered encoder engines.

use super::super::{
    encode_context::EncodeContext,
    encode_plan::EncodePlan,
};
use crate::{
    CapacityError,
    Codec,
    TranscodeError,
};

/// Policy hooks for [`crate::TranscodeEncodeEngine`].
///
/// Hooks own policy state, such as replacement or ignore behavior, but not the
/// codec or engine cursor state. The engine passes the codec into hook methods
/// when policy code needs codec metadata or one-value encode operations.
///
/// Implement this trait when a buffered encoder needs policy decisions around
/// individual values while reusing the common engine loop. Examples include
/// rejecting unsupported values with adapter-level context, consuming values
/// without writing output, writing replacement units, or emitting final state
/// in [`finish`](Self::finish).
///
/// The engine calls [`prepare_encode`](Self::prepare_encode) before each value
/// is consumed. The returned [`EncodePlan`] states the required output capacity
/// and may carry an action computed by the hook. Only after that capacity is
/// available does the engine call [`write_encode`](Self::write_encode) with the
/// same cursor context and the prepared plan. This split lets the engine stop
/// with [`crate::TranscodeStatus::NeedOutput`]
/// without consuming the next input value.
///
/// # Example
///
/// This hook writes each value with the wrapped codec and uses the codec's
/// maximum width as the capacity plan.
///
/// ```rust
/// use core::{
///     convert::Infallible,
///     num::NonZeroUsize,
/// };
/// use qubit_codec::{
///     TranscodeEncodeHooks,
///     Codec,
///     CodecEncodeError,
///     EncodeContext,
///     EncodePlan,
/// };
///
/// #[derive(Clone, Copy)]
/// struct ByteCodec;
///
/// unsafe impl Codec for ByteCodec {
///     type Value = u8;
///     type Unit = u8;
///     type DecodeError = Infallible;
///     type EncodeError = Infallible;
///     type DecodeState = ();
///     type EncodeState = ();
///
///     fn min_units_per_value(&self) -> NonZeroUsize {
///         NonZeroUsize::MIN
///     }
///
///     fn max_units_per_value(&self) -> NonZeroUsize {
///         NonZeroUsize::MIN
///     }
///
///     unsafe fn decode(
///         &mut self,
///         input: &[u8],
///         index: usize,
///     ) -> Result<(u8, NonZeroUsize), Self::DecodeError> {
///         Ok((input[index], NonZeroUsize::MIN))
///     }
///
///     unsafe fn encode(
///         &mut self,
///         value: &u8,
///         output: &mut [u8],
///         index: usize,
///     ) -> Result<usize, Self::EncodeError> {
///         output[index] = *value;
///         Ok(1)
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
///     type ErrorContext = ();
///     type PlanAction = ();
///
///     fn prepare_encode(
///         &mut self,
///         codec: &mut C,
///         _value: &C::Value,
///         _input_index: usize,
///     ) -> Result<EncodePlan<()>, Self::Error> {
///         Ok(EncodePlan::new(codec.max_units_per_value().get(), ()))
///     }
///
///     unsafe fn write_encode(
///         &mut self,
///         codec: &mut C,
///         context: EncodeContext<'_, C::Value, C::Unit>,
///         _plan: EncodePlan<()>,
///     ) -> Result<usize, Self::Error> {
///         unsafe {
///             codec.encode(context.input_value, context.output, context.output_index)
///         }
///         .map_err(|error| CodecEncodeError::encode(error, context.input_index))
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
    /// Error type returned by the buffered encoder.
    type Error: TranscodeError<Self::ErrorContext>;

    /// Context passed to [`TranscodeError`] factories for contract failures.
    type ErrorContext: Copy + Send + Sync + Default + 'static;

    /// Returns context used to build contract errors for this hook policy.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    ///
    /// # Returns
    ///
    /// Returns the error context associated with `codec`.
    #[inline(always)]
    fn error_context(_codec: &C) -> Self::ErrorContext {
        Self::ErrorContext::default()
    }

    /// Concrete action stored in [`EncodePlan::action`].
    type PlanAction;

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
    /// [`Codec::max_units_per_value`].
    #[must_use = "capacity planning can fail on overflow"]
    #[inline]
    fn max_output_len(
        &self,
        codec: &C,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        input_len
            .checked_mul(codec.max_units_per_value().get())
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
    #[must_use]
    #[inline(always)]
    fn max_finish_output_len(&self, _codec: &C) -> usize {
        0
    }

    /// Prepares an encoding plan for one input value.
    ///
    /// This method must not write output. It decides the output capacity bound
    /// needed before [`write_encode`](Self::write_encode) may be called and
    /// returns an implementation-specific plan action.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `input_value`: Input value being encoded.
    /// - `input_index`: Absolute input index of `value`.
    ///
    /// # Returns
    ///
    /// Returns the write plan for `value`.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when this value cannot be encoded under the hook
    /// policy.
    fn prepare_encode(
        &mut self,
        codec: &mut C,
        input_value: &C::Value,
        input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error>;

    /// Writes one input value according to a previously prepared plan.
    ///
    /// This method is called only after the engine has verified that
    /// [`EncodePlan::max_output_units`] units from `plan` are writable from
    /// [`EncodeContext::output_index`]. Implementations may rely on that
    /// capacity guarantee and do not need to report output starvation here. If
    /// a value needs more output than the plan declared, fix
    /// [`prepare_encode`](Self::prepare_encode) to return a larger bound.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `context`: Encode-write context containing the input value, input
    ///   index, output slice, and output cursor.
    /// - `plan`: Prepared plan returned by
    ///   [`prepare_encode`](Self::prepare_encode).
    ///
    /// # Returns
    ///
    /// Returns the number of output units written.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when writing fails under the hook policy. Output
    /// capacity exhaustion is handled before this method is called and should
    /// not be reported as a write error.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that at least the corresponding
    /// [`EncodePlan::max_output_units`] units are writable from
    /// [`EncodeContext::output_index`] in [`EncodeContext::output`].
    unsafe fn write_encode(
        &mut self,
        codec: &mut C,
        context: EncodeContext<'_, C::Value, C::Unit>,
        plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error>;

    /// Maps a codec-level reset error into this hook's public error type.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `error`: Error returned by [`Codec::encode_reset`].
    ///
    /// # Returns
    ///
    /// Returns the hook-specific error.
    #[inline]
    fn map_encode_reset_error(
        &mut self,
        _codec: &mut C,
        _error: C::EncodeError,
    ) -> Self::Error {
        panic!(
            "TranscodeEncodeHooks::map_encode_reset_error must be implemented for fallible reset codecs"
        )
    }

    /// Writes encoder reset output through the wrapped codec.
    ///
    /// The default implementation delegates to [`Codec::encode_reset`] and
    /// maps errors through
    /// [`map_encode_reset_error`](Self::map_encode_reset_error).
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `output`: Destination unit buffer.
    /// - `output_index`: Absolute output index where reset output starts.
    ///
    /// # Returns
    ///
    /// Returns the number of reset units written.
    ///
    /// # Errors
    ///
    /// Returns hook-specific reset errors.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that at least
    /// [`Codec::max_encode_reset_units`] units are writable from
    /// `output_index`.
    #[inline]
    unsafe fn write_encode_reset(
        &mut self,
        codec: &mut C,
        output: &mut [C::Unit],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        // SAFETY: Forwarded from this method's safety contract.
        unsafe { codec.encode_reset(output, output_index) }
            .map_err(|error| self.map_encode_reset_error(codec, error))
    }

    /// Finishes hook-owned state and writes any retained output units.
    ///
    /// The default implementation is a no-op for stateless encode hooks.
    /// Stateful hooks may emit final units such as reset sequences, checksums,
    /// or trailers. The caller must provide at least
    /// [`TranscodeEncodeHooks::max_finish_output_len`] writable units from
    /// `output_index`. Engines may pass an output slice whose upper bound is
    /// capped at `output_index + max_finish_output_len`, so implementations
    /// must not write beyond that declared final-output bound.
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
    #[inline]
    fn finish(
        &mut self,
        _codec: &mut C,
        _output: &mut [C::Unit],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }

    /// Resets hook-owned policy state.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    #[inline(always)]
    fn reset(&mut self, _codec: &mut C) {}
}

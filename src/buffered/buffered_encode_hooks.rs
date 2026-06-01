/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Policy hooks used by buffered encoder engines.

use core::num::NonZeroUsize;

use super::{
    encode_context::EncodeContext,
    encode_plan::EncodePlan,
    transcode_progress::TranscodeProgress,
};
use crate::{
    CapacityError,
    Codec,
};

/// Policy hooks for [`crate::BufferedEncodeEngine`].
///
/// Hooks own policy state, such as replacement or ignore behavior, but not the
/// codec or engine cursor state. The engine passes the codec into hook methods
/// when policy code needs codec metadata or one-value encode operations.
///
/// Implement this trait when a buffered encoder needs policy decisions around
/// individual values while reusing the common engine loop. Examples include
/// rejecting unsupported values with adapter-level context, consuming values
/// without writing output, writing replacement units, or emitting final state in
/// [`finish`](Self::finish).
///
/// The engine calls [`prepare_encode`](Self::prepare_encode) before each value
/// is consumed. The returned [`EncodePlan`] states the required output capacity
/// and may carry an action computed by the hook. Only after that capacity is
/// available does the engine call [`write_encode`](Self::write_encode). This
/// split lets the engine stop with [`crate::TranscodeStatus::NeedOutput`]
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
///     BufferedEncodeHooks,
///     Codec,
///     CodecEncodeError,
///     EncodeContext,
///     EncodePlan,
/// };
///
/// #[derive(Clone, Copy)]
/// struct ByteCodec;
///
/// unsafe impl Codec<u8, u8> for ByteCodec {
///     type DecodeError = Infallible;
///     type EncodeError = Infallible;
///
///     fn min_units_per_value(&self) -> NonZeroUsize {
///         NonZeroUsize::MIN
///     }
///
///     fn max_units_per_value(&self) -> NonZeroUsize {
///         NonZeroUsize::MIN
///     }
///
///     unsafe fn decode_unchecked(
///         &self,
///         input: &[u8],
///         index: usize,
///     ) -> Result<(u8, NonZeroUsize), Self::DecodeError> {
///         Ok((input[index], NonZeroUsize::MIN))
///     }
///
///     unsafe fn encode_unchecked(
///         &self,
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
/// impl<C, Value, Unit> BufferedEncodeHooks<C, Value, Unit> for StrictHooks
/// where
///     C: Codec<Value, Unit>,
///     Unit: Copy,
/// {
///     type Error = CodecEncodeError<C::EncodeError>;
///     type PlanAction = ();
///
///     fn prepare_encode(
///         &mut self,
///         codec: &C,
///         _value: &Value,
///         _input_index: usize,
///     ) -> Result<EncodePlan<()>, Self::Error> {
///         Ok(EncodePlan::new(codec.max_units_per_value().get(), ()))
///     }
///
///     unsafe fn write_encode(
///         &mut self,
///         codec: &C,
///         context: EncodeContext<'_, Value, Unit, ()>,
///     ) -> Result<usize, Self::Error> {
///         unsafe {
///             codec.encode_unchecked(context.input_value, context.output, context.output_index)
///         }
///         .map_err(|error| CodecEncodeError::encode(error, context.input_index))
///     }
///
///     fn invalid_input_index(
///         &mut self,
///         _codec: &C,
///         index: usize,
///         input_len: usize,
///     ) -> Self::Error {
///         CodecEncodeError::invalid_input_index(index, input_len)
///     }
/// }
/// ```
///
/// # Type Parameters
///
/// - `C`: Low-level codec owned by the engine.
/// - `Value`: Logical input value type.
/// - `Unit`: Encoded output unit type.
pub trait BufferedEncodeHooks<C, Value, Unit>
where
    C: Codec<Value, Unit>,
    Unit: Copy,
{
    /// Error type returned by the buffered encoder.
    type Error;

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
    fn max_output_len(&self, codec: &C, input_len: usize) -> Result<usize, CapacityError> {
        input_len
            .checked_mul(codec.max_units_per_value().get())
            .ok_or(CapacityError::OutputLengthOverflow)
    }

    /// Returns an upper bound for units emitted by finishing hook-owned state.
    ///
    /// `finish` never receives more input values. Implementations must only
    /// report output derived from hook-owned state that remains after the caller
    /// has supplied all input.
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
        codec: &C,
        input_value: &Value,
        input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error>;

    /// Writes one input value according to a previously prepared plan.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `context`: Prepared encode-write context containing the input value,
    ///   input index, plan action, output slice, and output cursor.
    ///
    /// # Returns
    ///
    /// Returns the number of output units written.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when writing fails under the hook policy.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that at least the corresponding
    /// [`EncodePlan::max_output_units`] units are writable from
    /// [`EncodeContext::output_index`].
    unsafe fn write_encode(
        &mut self,
        codec: &C,
        context: EncodeContext<'_, Value, Unit, Self::PlanAction>,
    ) -> Result<usize, Self::Error>;

    /// Builds an error for a caller-supplied input index outside the input slice.
    ///
    /// The engine calls this hook before it reads input. Keeping this
    /// construction in the hook lets codec-backed adapters preserve their own
    /// concrete error type without a separate public factory trait.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `index`: Invalid absolute input index supplied by the caller.
    /// - `input_len`: Length of the input slice.
    ///
    /// # Returns
    ///
    /// Returns the hook-specific invalid-input-index error.
    fn invalid_input_index(&mut self, codec: &C, index: usize, input_len: usize) -> Self::Error;

    /// Finishes hook-owned state and writes any retained output units.
    ///
    /// The default implementation is a no-op for stateless encode hooks.
    /// Stateful hooks may emit final units such as reset sequences, checksums, or
    /// trailers. If `output` does not provide enough capacity, return
    /// [`crate::TranscodeStatus::NeedOutput`] and keep the unwritten state for a
    /// later `finish` call.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    /// - `output`: Complete output unit slice visible to the hook.
    /// - `output_index`: Absolute output unit index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns progress for units written by finalization.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` when hook-owned state cannot be finalized.
    #[inline(always)]
    fn finish(
        &mut self,
        _codec: &C,
        output: &mut [Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if output_index > output.len() {
            return Ok(TranscodeProgress::need_output(output_index, NonZeroUsize::MIN, 0, 0, 0));
        }
        Ok(TranscodeProgress::complete(0, 0))
    }

    /// Resets hook-owned policy state.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec owned by the engine.
    #[inline(always)]
    fn reset(&mut self, _codec: &C) {}
}

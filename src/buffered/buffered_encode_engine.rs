/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Reusable buffered encoder engine.

use super::{
    buffered_encode_hooks::BufferedEncodeHooks,
    encode_context::EncodeContext,
    encode_plan::EncodePlan,
    encode_state::EncodeState,
    encode_step::EncodeStep,
    finish_error::FinishError,
    transcode_progress::TranscodeProgress,
};
use crate::{
    CapacityError,
    Codec,
    codec::debug_assert_unit_bounds,
};

/// Reusable buffered encoding engine for codec-backed encoders.
///
/// The engine owns the low-level codec and hook object. It keeps the common
/// buffered encoding loop private: input-index validation, output-capacity
/// checks, input consumption, output progress, and [`crate::TranscodeStatus`]
/// reporting.
///
/// Use this type to build a streaming encoder over a one-value [`Codec`]. The
/// engine does not allocate output. It repeatedly asks hooks to plan one input
/// value, verifies that the caller-provided output slice can hold that plan, and
/// then lets the hooks write the value. If the next value would not fit, the
/// engine returns [`crate::TranscodeStatus::NeedOutput`] without consuming that
/// value; the caller can provide a larger or fresh output buffer and resume
/// with the returned input index.
///
/// For the common strict policy that simply wraps codec errors, use
/// [`crate::CodecBufferedEncoder`]. Use `BufferedEncodeEngine` directly when
/// the encode policy needs custom planning, replacement, skipped values, or
/// finish-time output.
///
/// # Example
///
/// ```rust
/// use core::{
///     convert::Infallible,
///     num::NonZeroUsize,
/// };
/// use qubit_codec::{
///     BufferedEncodeEngine,
///     BufferedEncodeHooks,
///     Codec,
///     CodecEncodeError,
///     EncodeContext,
///     EncodePlan,
///     TranscodeStatus,
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
/// impl BufferedEncodeHooks<ByteCodec> for StrictHooks {
///     type Error = CodecEncodeError<Infallible>;
///     type PlanAction = ();
///
///     fn prepare_encode(
///         &mut self,
///         codec: &ByteCodec,
///         _value: &u8,
///         _input_index: usize,
///     ) -> Result<EncodePlan<()>, Self::Error> {
///         Ok(EncodePlan::new(codec.max_units_per_value().get(), ()))
///     }
///
///     unsafe fn write_encode(
///         &mut self,
///         codec: &ByteCodec,
///         context: EncodeContext<'_, u8, u8>,
///         _plan: EncodePlan<()>,
///     ) -> Result<usize, Self::Error> {
///         unsafe {
///             codec.encode_unchecked(context.input_value, context.output, context.output_index)
///         }
///         .map_err(|error| CodecEncodeError::encode(error, context.input_index))
///     }
///
///     fn invalid_input_index(
///         &mut self,
///         _codec: &ByteCodec,
///         index: usize,
///         input_len: usize,
///     ) -> Self::Error {
///         CodecEncodeError::invalid_input_index(index, input_len)
///     }
/// }
///
/// let mut engine = BufferedEncodeEngine::new(ByteCodec, StrictHooks);
/// let input = [1_u8, 2, 3];
/// let mut output = [0_u8; 2];
///
/// let progress = engine.transcode(&input, 0, &mut output, 0)?;
/// match progress.status() {
///     TranscodeStatus::Complete => unreachable!("output is intentionally short"),
///     TranscodeStatus::NeedOutput { output_index, .. } => {
///         assert_eq!(2, output_index);
///         assert_eq!([1, 2], output);
///         // Write out `output[..output_index]`, then resume at
///         // `progress.read()` with fresh output capacity.
///     }
///     TranscodeStatus::NeedInput { .. } => unreachable!("encoders do not read encoded input"),
/// }
/// # Ok::<(), CodecEncodeError<Infallible>>(())
/// ```
///
/// # Type Parameters
///
/// - `C`: Low-level codec used by the engine.
/// - `H`: Policy hook object used by the engine.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct BufferedEncodeEngine<C, H> {
    /// Low-level codec used for one-value encoding.
    pub(super) codec: C,
    /// Policy hooks used for planning and writing values.
    pub(super) hooks: H,
}

impl<C, H> BufferedEncodeEngine<C, H>
where
    C: Codec,
    H: BufferedEncodeHooks<C>,
{
    /// Creates a buffered encoder engine.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec used for one-value encoding.
    /// - `hooks`: Policy hooks used for planning and writing values.
    ///
    /// # Returns
    ///
    /// Returns a buffered encoder engine.
    #[must_use]
    #[inline(always)]
    pub const fn new(codec: C, hooks: H) -> Self {
        Self { codec, hooks }
    }

    /// Prepares one value for writing through the configured encode hooks.
    ///
    /// # Parameters
    ///
    /// - `input_value`: Input value to be planned.
    /// - `input_index`: Absolute input value index at which this value is located.
    ///
    /// # Returns
    ///
    /// Returns an encode plan containing the maximum planned output width and action.
    #[inline(always)]
    pub(crate) fn prepare_value(
        &mut self,
        input_value: &C::Value,
        input_index: usize,
    ) -> Result<EncodePlan<H::PlanAction>, H::Error> {
        self.hooks.prepare_encode(&self.codec, input_value, input_index)
    }

    /// Writes one value using a previously prepared encode context.
    ///
    /// # Safety
    ///
    /// The caller must guarantee that the context was built from an encode plan
    /// whose output-capacity bound is writable from `context.output_index`.
    ///
    /// # Parameters
    ///
    /// - `context`: Context containing prepared plan, input index, and output.
    /// - `plan`: Prepared encode plan that selected the write action.
    ///
    /// # Returns
    ///
    /// Returns the number of output units written, which must not exceed the
    /// prepared capacity.
    ///
    /// # Errors
    ///
    /// Returns `H::Error` when hook-specific writing fails.
    #[inline(always)]
    pub(crate) unsafe fn write_prepared_value(
        &mut self,
        context: EncodeContext<'_, C::Value, C::Unit>,
        plan: EncodePlan<H::PlanAction>,
    ) -> Result<usize, H::Error> {
        // SAFETY: Forwarded from this method's safety contract.
        unsafe { self.hooks.write_encode(&self.codec, context, plan) }
    }

    /// Returns a conservative upper bound for output units needed for `input_len` values.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Number of input values the caller plans to encode.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound, or a capacity error on arithmetic
    /// overflow.
    #[must_use = "capacity planning can fail on overflow"]
    #[inline(always)]
    pub fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        debug_assert_unit_bounds::<C>(&self.codec);
        self.hooks.max_output_len(&self.codec, input_len)
    }

    /// Returns the maximum output units emitted by finishing hook-owned state.
    ///
    /// # Returns
    ///
    /// Returns the hook-provided final output bound.
    #[must_use]
    #[inline(always)]
    pub fn max_finish_output_len(&self) -> usize {
        self.hooks.max_finish_output_len(&self.codec)
    }

    /// Resets hook-owned state.
    ///
    /// # Parameters
    ///
    /// - `self`: Encoder instance whose hook state is reset.
    ///
    /// # Returns
    ///
    /// Returns unit `()`.
    #[inline(always)]
    pub fn reset(&mut self) {
        self.hooks.reset(&self.codec);
    }

    /// Encodes values into a caller-provided output buffer.
    ///
    /// The engine stops before consuming the next input value when the current
    /// output buffer does not satisfy that value's planned capacity bound.
    ///
    /// # Parameters
    ///
    /// - `input`: Complete input value slice visible to the encoder.
    /// - `input_index`: Absolute input value index where encoding starts.
    /// - `output`: Complete output unit slice visible to the encoder.
    /// - `output_index`: Absolute output unit index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns progress describing input values consumed, output units written,
    /// and why encoding stopped.
    ///
    /// # Errors
    ///
    /// Returns hook errors when `input_index` is outside `input`, or when hook
    /// planning or writing rejects a value.
    pub fn transcode(
        &mut self,
        input: &[C::Value],
        input_index: usize,
        output: &mut [C::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, H::Error> {
        if input_index > input.len() {
            return Err(self.hooks.invalid_input_index(&self.codec, input_index, input.len()));
        }
        debug_assert_unit_bounds::<C>(&self.codec);
        let mut state = EncodeState::new(input, input_index, output, output_index);
        if !state.output_cursor_in_bounds() {
            return Ok(state.need_output_progress(self.codec.max_units_per_value().get()));
        }

        while state.has_input() {
            // SAFETY: The loop condition proves that the current input cursor
            // points at an available value.
            let context = unsafe { state.context_unchecked() };
            let step = self.encode_step(context)?;
            if let Some(progress) = step.apply_to_state(&mut state) {
                return Ok(progress);
            }
        }

        Ok(state.complete_progress())
    }

    /// Finishes hook-owned output after EOF.
    ///
    /// The engine owns no final output state itself. Hook implementations may
    /// finish their own retained state and emit final output after the caller has
    /// supplied all input values. The caller must provide enough output capacity
    /// for [`BufferedEncodeEngine::max_finish_output_len`].
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output unit slice visible to the encoder.
    /// - `output_index`: Absolute output unit index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of units written by finalization.
    ///
    /// # Errors
    ///
    /// Returns [`FinishError`] when the caller provides invalid or insufficient
    /// output capacity, or when hook finalization fails.
    ///
    /// # Panics
    ///
    /// Panics when the hook writes or reports more final output units than
    /// [`BufferedEncodeEngine::max_finish_output_len`] declared.
    pub fn finish(&mut self, output: &mut [C::Unit], output_index: usize) -> Result<usize, FinishError<H::Error>> {
        let required = self.max_finish_output_len();
        FinishError::ensure_output_capacity(output.len(), output_index, required)?;
        let output_end = output_index + required;
        let output = &mut output[..output_end];
        let written = self
            .hooks
            .finish(&self.codec, output, output_index)
            .map_err(FinishError::source)?;
        assert!(
            written <= required,
            "BufferedEncodeEngine hook wrote beyond its finish bound",
        );
        Ok(written)
    }

    /// Encodes one value attempt into a normalized encode step.
    ///
    /// # Parameters
    ///
    /// - `context`: Current value, absolute input index, and target output cursor.
    ///
    /// # Returns
    ///
    /// Returns an encode step describing either written output or missing output
    /// capacity.
    ///
    /// # Errors
    ///
    /// Returns hook errors when planning or writing rejects the value.
    #[inline]
    pub(super) fn encode_step(
        &mut self,
        context: EncodeContext<'_, C::Value, C::Unit>,
    ) -> Result<EncodeStep, H::Error> {
        let plan = self.prepare_value(context.input_value, context.input_index)?;
        let max_output_units = plan.max_output_units;
        let available = context.available_output();
        if available < max_output_units {
            return Ok(EncodeStep::need_output(max_output_units, available));
        }

        // SAFETY: The capacity check above guarantees the bound requested by
        // the prepared plan.
        let written = unsafe { self.write_prepared_value(context, plan) }?;
        debug_assert!(
            written <= max_output_units,
            "BufferedEncodeEngine hook wrote beyond its prepared capacity bound",
        );
        Ok(EncodeStep::written(written))
    }
}

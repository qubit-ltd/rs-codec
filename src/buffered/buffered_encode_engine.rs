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

use core::num::NonZeroUsize;

use super::{
    buffered_encode_hooks::BufferedEncodeHooks,
    encode_context::EncodeContext,
    encode_plan::EncodePlan,
    encode_state::EncodeState,
    encode_step::EncodeStep,
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
///         context: EncodeContext<'_, u8, u8, ()>,
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
        context: EncodeContext<'_, C::Value, C::Unit, H::PlanAction>,
    ) -> Result<usize, H::Error> {
        // SAFETY: Forwarded from this method's safety contract.
        unsafe { self.hooks.write_encode(&self.codec, context) }
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
            let (input_value, input_cursor, output, output_cursor) = unsafe { state.current_encode_parts_unchecked() };
            let step = self.encode_value_step(input_value, input_cursor, output, output_cursor)?;
            if let Some(progress) = step.apply_to_encode_state(&mut state) {
                return Ok(progress);
            }
        }

        Ok(state.complete_progress())
    }

    /// Finishes hook-owned output after EOF.
    ///
    /// The engine owns no final output state itself. Hook implementations may
    /// finish their own retained state and emit final output after the caller has
    /// supplied all input values.
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output unit slice visible to the encoder.
    /// - `output_index`: Absolute output unit index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns hook-provided finalization progress.
    ///
    /// # Errors
    ///
    /// Returns hook errors when finalization fails.
    pub fn finish(&mut self, output: &mut [C::Unit], output_index: usize) -> Result<TranscodeProgress, H::Error> {
        if output_index > output.len() {
            return Ok(TranscodeProgress::need_output(output_index, NonZeroUsize::MIN, 0, 0, 0));
        }
        self.hooks.finish(&self.codec, output, output_index)
    }

    /// Encodes one value into the provided output slice.
    ///
    /// # Parameters
    ///
    /// - `input_value`: Logical value to encode.
    /// - `input_index`: Absolute input value index used for hook context.
    /// - `output`: Complete output unit slice visible to the caller.
    /// - `output_index`: Absolute output unit index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns an encode step describing either written output or missing output
    /// capacity.
    ///
    /// # Errors
    ///
    /// Returns hook errors when planning or writing rejects the value.
    #[inline(always)]
    pub(super) fn encode_value_step(
        &mut self,
        input_value: &C::Value,
        input_index: usize,
        output: &mut [C::Unit],
        output_index: usize,
    ) -> Result<EncodeStep, H::Error> {
        debug_assert!(
            output_index <= output.len(),
            "output index must be within the output slice"
        );
        let plan = self.prepare_value(input_value, input_index)?;
        let max_output_units = plan.max_output_units;
        let available = output.len().saturating_sub(output_index);
        if available < max_output_units {
            return Ok(EncodeStep::need_output(max_output_units, available));
        }

        let context = EncodeContext {
            input_value,
            input_index,
            plan_action: plan.action,
            output,
            output_index,
        };
        // SAFETY: The capacity check above guarantees the bound requested by
        // the prepared plan.
        let written = unsafe { self.write_prepared_value(context) }?;
        debug_assert!(
            written <= max_output_units,
            "BufferedEncodeEngine hook wrote beyond its prepared capacity bound",
        );
        Ok(EncodeStep::written(written))
    }
}

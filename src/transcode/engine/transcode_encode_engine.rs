// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Reusable buffered encoder engine.

use super::super::internal::{encode_state::EncodeState, encode_step::EncodeStep};
use crate::codec::assert_unit_bounds;
use crate::{
    CapacityError, Codec, EncodeContext, TranscodeEncodeHooks, TranscodeError, TranscodeProgress,
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
/// value, verifies that the caller-provided output slice can hold that plan,
/// and then lets the hooks write the value. If the next value would not fit,
/// the engine returns [`crate::TranscodeStatus::NeedOutput`] without consuming
/// that value; the caller can provide a larger or fresh output buffer and
/// resume with the returned input index.
///
/// For the common strict policy that simply wraps codec errors, use
/// [`crate::CodecTranscodeEncoder`]. Use `TranscodeEncodeEngine` directly when
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
///     TranscodeEncodeEngine,
///     TranscodeEncodeHooks,
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
///     ) -> Result<NonZeroUsize, Self::EncodeError> {
///         output[index] = *value;
///         Ok(NonZeroUsize::MIN)
///     }
/// }
///
/// struct StrictHooks;
///
/// impl TranscodeEncodeHooks<ByteCodec> for StrictHooks {
///     type Error = CodecEncodeError<Infallible>;
///     type PlanAction = ();
///
///     fn prepare_encode(
///         &mut self,
///         codec: &mut ByteCodec,
///         _value: &u8,
///         _input_index: usize,
///     ) -> Result<EncodePlan<()>, Self::Error> {
///         Ok(EncodePlan::new(codec.max_units_per_value().get(), ()))
///     }
///
///     unsafe fn write_encode(
///         &mut self,
///         codec: &mut ByteCodec,
///         context: EncodeContext<'_, u8, u8>,
///         _plan: EncodePlan<()>,
///     ) -> Result<usize, Self::Error> {
///         unsafe {
///             codec.encode(context.input_value, context.output, context.output_index)
///         }
///         .map(NonZeroUsize::get)
///         .map_err(|error| CodecEncodeError::encode(error, context.input_index))
///     }
/// }
///
/// let mut engine = TranscodeEncodeEngine::new(ByteCodec, StrictHooks);
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
/// # Ok::<(), qubit_codec::TranscodeError<CodecEncodeError<Infallible>>>(())
/// ```
///
/// # Type Parameters
///
/// - `C`: Low-level codec used by the engine.
/// - `H`: Policy hook object used by the engine.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TranscodeEncodeEngine<C, H> {
    /// Low-level codec used for one-value encoding.
    pub(super) codec: C,
    /// Policy hooks used for planning and writing values.
    pub(super) hooks: H,
}

impl<C, H> TranscodeEncodeEngine<C, H>
where
    C: Codec,
    H: TranscodeEncodeHooks<C>,
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
    ///
    /// # Panics
    ///
    /// Panics when the supplied codec violates the
    /// [`Codec::min_units_per_value`] / [`Codec::max_units_per_value`] ordering
    /// invariant.
    #[must_use]
    #[inline]
    pub fn new(codec: C, hooks: H) -> Self {
        assert_unit_bounds::<C>(&codec);
        Self { codec, hooks }
    }

    /// Gets a conservative upper bound for output units needed for
    /// `input_len` values.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Number of input values the caller plans to encode.
    ///
    /// # Returns
    ///
    /// a conservative upper bound for output units, or a capacity error on
    /// arithmetic overflow.
    #[must_use = "capacity planning can fail on overflow"]
    #[inline(always)]
    pub fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        self.hooks.max_output_len(&self.codec, input_len)
    }

    /// Gets the maximum output units emitted by stream reset.
    ///
    /// # Returns
    ///
    /// the codec's reset-output upper bound.
    #[must_use]
    #[inline(always)]
    pub fn max_reset_output_len(&self) -> usize {
        self.codec.max_encode_reset_units()
    }

    /// Gets the maximum output units emitted by finishing hook-owned state.
    ///
    /// # Returns
    ///
    /// the hook-provided final output bound.
    #[must_use]
    #[inline(always)]
    pub fn max_finish_output_len(&self) -> usize {
        self.hooks.max_finish_output_len(&self.codec)
    }

    /// Resets codec encode state, hook-owned state, and stream-start output.
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output unit slice visible to the encoder.
    /// - `output_index`: Absolute output unit index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns the number of reset units written.
    ///
    /// # Errors
    ///
    /// Returns hook errors when the caller provides invalid or insufficient
    /// output capacity, or when reset output cannot be emitted.
    pub fn reset(
        &mut self,
        output: &mut [C::Unit],
        output_index: usize,
    ) -> Result<usize, TranscodeError<H::Error>> {
        let required = self.max_reset_output_len();
        TranscodeError::ensure_output_capacity(output.len(), output_index, required)?;
        self.hooks.reset(&mut self.codec);
        let written = unsafe {
            // SAFETY: The capacity check above reserves the codec's declared
            // reset-output bound at `output_index`.
            self.hooks
                .write_encode_reset(&mut self.codec, output, output_index)
        }
        .map_err(TranscodeError::domain)?;
        assert!(
            written <= required,
            "Codec::encode_reset wrote beyond its reset bound",
        );
        Ok(written)
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
    /// Returns hook errors when `input_index` is outside `input`, when
    /// `output_index` is outside `output`, or when hook planning or writing
    /// rejects a value.
    pub fn transcode(
        &mut self,
        input: &[C::Value],
        input_index: usize,
        output: &mut [C::Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeError<H::Error>> {
        TranscodeError::ensure_transcode_indices(
            input.len(),
            input_index,
            output.len(),
            output_index,
        )?;
        let mut state = EncodeState::new(input, input_index, output, output_index);

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
    /// finish their own retained state and emit final output after the caller
    /// has supplied all input values. The caller must provide enough output
    /// capacity for [`TranscodeEncodeEngine::max_finish_output_len`].
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
    /// Returns hook errors when the caller provides invalid or insufficient
    /// output capacity, or when hook finalization fails.
    ///
    /// # Panics
    ///
    /// Panics when the hook writes or reports more final output units than
    /// [`TranscodeEncodeEngine::max_finish_output_len`] declared.
    pub fn finish(
        &mut self,
        output: &mut [C::Unit],
        output_index: usize,
    ) -> Result<usize, TranscodeError<H::Error>> {
        let required = self.max_finish_output_len();
        TranscodeError::ensure_output_capacity(output.len(), output_index, required)?;
        let written = self
            .hooks
            .finish(&mut self.codec, output, output_index)
            .map_err(TranscodeError::domain)?;
        assert!(
            written <= required,
            "TranscodeEncodeEngine hook wrote beyond its finish bound",
        );
        Ok(written)
    }

    /// Encodes one value attempt into a normalized encode step.
    ///
    /// # Parameters
    ///
    /// - `context`: Current value, absolute input index, and target output
    ///   cursor.
    ///
    /// # Returns
    ///
    /// Returns an encode step describing either written output or missing
    /// output capacity.
    ///
    /// # Errors
    ///
    /// Returns hook errors when planning or writing rejects the value.
    pub(super) fn encode_step(
        &mut self,
        context: EncodeContext<'_, C::Value, C::Unit>,
    ) -> Result<EncodeStep, TranscodeError<H::Error>> {
        let plan = self
            .hooks
            .prepare_encode(&mut self.codec, context.input_value, context.input_index)
            .map_err(TranscodeError::domain)?;
        let max_output_units = plan.max_output_units;
        let available = context.available_output();
        if available < max_output_units {
            return Ok(EncodeStep::need_output(max_output_units, available));
        }

        // SAFETY: The capacity check above guarantees the bound requested by
        // the prepared plan.
        let written = match unsafe { self.hooks.write_encode(&mut self.codec, context, plan) } {
            Ok(written) => written,
            Err(error) => return Err(TranscodeError::domain(error)),
        };
        assert!(
            written <= max_output_units,
            "TranscodeEncodeEngine hook wrote beyond its prepared capacity bound",
        );
        Ok(EncodeStep::written(written))
    }
}

impl<C, H> Default for TranscodeEncodeEngine<C, H>
where
    C: Codec + Default,
    H: TranscodeEncodeHooks<C> + Default,
{
    /// Creates a default buffered encoder engine.
    ///
    /// # Returns
    ///
    /// Returns an engine with default codec and hooks.
    #[inline(always)]
    fn default() -> Self {
        Self::new(C::default(), H::default())
    }
}

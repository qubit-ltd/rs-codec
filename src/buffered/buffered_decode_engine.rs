/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Reusable buffered decoder engine.

use core::marker::PhantomData;

use super::{
    buffered_decode_hooks::BufferedDecodeHooks,
    decode_action::DecodeAction,
    decode_context::DecodeContext,
    decode_state::DecodeState,
    transcode_progress::TranscodeProgress,
    transcode_status::TranscodeStatus,
};
use crate::{
    Codec,
    DecodeErrorFactory,
    codec::debug_assert_unit_bounds,
};

/// Reusable buffered decoding engine for codec-backed decoders.
///
/// The engine owns the low-level codec and hook object. It keeps the common
/// buffered decoding loop private: input-index validation, output-capacity
/// checks, calls to [`Codec::decode_unchecked`], hook dispatch, and
/// [`TranscodeStatus`] reporting. Incomplete input tails are left in the
/// caller-provided input slice; callers own input-buffer refill.
///
/// # Type Parameters
///
/// - `C`: Low-level codec used by the engine.
/// - `H`: Policy hook object used by the engine.
/// - `Unit`: Encoded input unit type accepted by the engine.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct BufferedDecodeEngine<C, H, Unit> {
    /// Low-level codec used for one-value decoding.
    codec: C,
    /// Policy hooks used for decode failures.
    hooks: H,
    /// Binds the engine to the encoded input unit type.
    marker: PhantomData<fn(Unit)>,
}

impl<C, H, Unit> BufferedDecodeEngine<C, H, Unit> {
    /// Creates a buffered decoder engine.
    ///
    /// # Parameters
    ///
    /// - `codec`: Low-level codec used for one-value decoding.
    /// - `hooks`: Policy hooks used for decode failures.
    ///
    /// # Returns
    ///
    /// Returns a buffered decoder engine.
    #[must_use]
    #[inline(always)]
    pub const fn new(codec: C, hooks: H) -> Self {
        Self {
            codec,
            hooks,
            marker: PhantomData,
        }
    }

    /// Returns the wrapped low-level codec.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the codec.
    #[must_use]
    #[inline(always)]
    pub const fn codec(&self) -> &C {
        &self.codec
    }

    /// Returns the wrapped low-level codec mutably.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the codec.
    #[must_use]
    #[inline(always)]
    pub fn codec_mut(&mut self) -> &mut C {
        &mut self.codec
    }

    /// Returns the policy hooks.
    ///
    /// # Returns
    ///
    /// Returns a shared reference to the hooks.
    #[must_use]
    #[inline(always)]
    pub const fn hooks(&self) -> &H {
        &self.hooks
    }

    /// Returns the policy hooks mutably.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to the hooks.
    #[must_use]
    #[inline(always)]
    pub fn hooks_mut(&mut self) -> &mut H {
        &mut self.hooks
    }

    /// Consumes the engine and returns the wrapped codec.
    ///
    /// # Returns
    ///
    /// Returns the codec supplied at construction time.
    #[must_use]
    #[inline(always)]
    pub fn into_codec(self) -> C {
        self.codec
    }

    /// Returns an upper bound for decoded values produced from `input_len` units.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Number of source units the caller plans to decode.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound, or `None` on overflow.
    #[must_use]
    #[inline(always)]
    pub fn max_output_len<Value>(&self, input_len: usize) -> Option<usize>
    where
        C: Codec<Value, Unit>,
        H: BufferedDecodeHooks<C, Unit, Value>,
        Unit: Copy,
    {
        debug_assert_unit_bounds::<C, Value, Unit>(&self.codec);
        self.hooks.max_output_len(&self.codec, input_len)
    }

    /// Returns the maximum values emitted by finishing hook-owned state.
    ///
    /// # Returns
    ///
    /// Returns the hook-provided final output bound.
    #[must_use]
    #[inline(always)]
    pub fn max_finish_output_len<Value>(&self) -> Option<usize>
    where
        C: Codec<Value, Unit>,
        H: BufferedDecodeHooks<C, Unit, Value>,
        Unit: Copy,
    {
        self.hooks.max_finish_output_len(&self.codec)
    }

    /// Resets hook-owned state.
    #[inline(always)]
    pub fn reset<Value>(&mut self)
    where
        C: Codec<Value, Unit>,
        H: BufferedDecodeHooks<C, Unit, Value>,
        Unit: Copy,
    {
        self.hooks.reset(&self.codec);
    }

    /// Decodes source units into caller-provided output values.
    ///
    /// # Parameters
    ///
    /// - `input`: Complete input unit slice visible to the decoder.
    /// - `input_index`: Absolute input unit index where decoding starts.
    /// - `output`: Complete output value slice visible to the decoder.
    /// - `output_index`: Absolute output value index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns progress describing input units consumed, output values written,
    /// and why decoding stopped.
    ///
    /// # Errors
    ///
    /// Returns hook errors when `input_index` is outside `input`, or when a
    /// concrete policy hook rejects a value.
    #[inline]
    pub fn transcode<Value>(
        &mut self,
        input: &[Unit],
        input_index: usize,
        output: &mut [Value],
        output_index: usize,
    ) -> Result<TranscodeProgress, <H as BufferedDecodeHooks<C, Unit, Value>>::Error>
    where
        C: Codec<Value, Unit>,
        H: BufferedDecodeHooks<C, Unit, Value>,
        Unit: Copy,
    {
        if input_index > input.len() {
            return Err(<H::Error as DecodeErrorFactory<C>>::invalid_input_index(
                &self.codec,
                input_index,
                input.len(),
            ));
        }
        debug_assert_unit_bounds::<C, Value, Unit>(&self.codec);
        let min_units = self.codec.min_units_per_value();
        let mut state = DecodeState::new(input, input_index, output, output_index, min_units);
        if !state.output_cursor_in_bounds() {
            return Ok(state.need_output_progress());
        }

        while state.has_input() {
            if state.needs_input() {
                return Ok(state.need_input_progress());
            }

            // SAFETY: `needs_input()` returned false, so the state has at
            // least `min_units_per_value()` units available from the current
            // cursor.
            let result = unsafe { self.codec.decode_unchecked(state.input(), state.input_cursor()) };
            if let Some(progress) = self.handle_decode_result(&mut state, result)? {
                return Ok(progress);
            }
        }

        Ok(state.complete_progress())
    }

    /// Finishes hook-owned output after EOF.
    ///
    /// The engine owns no final output state itself. Hook implementations may
    /// finish their own retained state and emit final output after the caller
    /// has handled any incomplete input tail.
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output value slice visible to the decoder.
    /// - `output_index`: Absolute output value index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns hook-provided finalization progress.
    ///
    /// # Errors
    ///
    /// Returns hook errors when finalization fails.
    #[inline(always)]
    pub fn finish<Value>(
        &mut self,
        output: &mut [Value],
        output_index: usize,
    ) -> Result<TranscodeProgress, <H as BufferedDecodeHooks<C, Unit, Value>>::Error>
    where
        C: Codec<Value, Unit>,
        H: BufferedDecodeHooks<C, Unit, Value>,
        Unit: Copy,
    {
        if output_index > output.len() {
            let additional = self.hooks.max_finish_output_len(&self.codec).unwrap_or(1).max(1);
            return Ok(TranscodeProgress::need_output(output_index, additional, 0, 0, 0));
        }
        self.hooks.finish(&self.codec, output, output_index)
    }

    /// Handles one low-level decode result and updates the decode state.
    ///
    /// # Parameters
    ///
    /// - `state`: Mutable decode call state.
    /// - `result`: Low-level codec decode result.
    ///
    /// # Returns
    ///
    /// Returns `Some(progress)` when the caller must stop transcoding, or `None`
    /// when the main loop should continue.
    ///
    /// # Errors
    ///
    /// Returns hook errors when the policy rejects the input.
    #[inline]
    fn handle_decode_result<Value>(
        &mut self,
        state: &mut DecodeState<'_, Unit, Value>,
        result: Result<(Value, core::num::NonZeroUsize), C::DecodeError>,
    ) -> Result<Option<TranscodeProgress>, <H as BufferedDecodeHooks<C, Unit, Value>>::Error>
    where
        C: Codec<Value, Unit>,
        H: BufferedDecodeHooks<C, Unit, Value>,
        Unit: Copy,
    {
        match result {
            Ok((value, consumed)) => {
                if state.needs_output() {
                    return Ok(Some(state.need_output_progress()));
                }
                state.emit(value, consumed);
                Ok(None)
            }
            Err(error) => {
                let context = state.context();
                let action = self.hooks.handle_decode_error(&self.codec, error, context)?;
                let status = self.apply_decode_action(state, action, context);
                Ok(status.map(|status| state.status_progress(status)))
            }
        }
    }

    /// Applies a decode action selected by the hook policy.
    ///
    /// # Parameters
    ///
    /// - `state`: Mutable decode call state.
    /// - `action`: Action selected by the hook policy.
    /// - `context`: Decode attempt context visible to the hook.
    ///
    /// # Returns
    ///
    /// Returns `Some(status)` when conversion must stop, or `None` when the main
    /// loop should continue.
    #[inline]
    fn apply_decode_action<Value>(
        &self,
        state: &mut DecodeState<'_, Unit, Value>,
        action: DecodeAction<Value>,
        context: DecodeContext,
    ) -> Option<TranscodeStatus> {
        match action {
            DecodeAction::NeedInput { required_total } => {
                let additional = required_total.saturating_sub(context.available).max(1);
                Some(TranscodeStatus::need_input(
                    context.input_index,
                    additional,
                    context.available,
                ))
            }
            DecodeAction::Skip { consumed } => {
                let consumed = state.normalize_consumed(consumed);
                state.skip(consumed);
                None
            }
            DecodeAction::Emit { value, consumed } => {
                if state.needs_output() {
                    return Some(TranscodeStatus::need_output(context.output_index, 1, 0));
                }
                let consumed = state.normalize_consumed(consumed);
                state.emit(value, consumed);
                None
            }
        }
    }
}

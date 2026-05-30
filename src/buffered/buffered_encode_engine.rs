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
    encode_state::EncodeState,
    transcode_progress::TranscodeProgress,
};
use crate::{
    Codec,
    EncodeErrorFactory,
    codec::debug_assert_unit_bounds,
};

/// Reusable buffered encoding engine for codec-backed encoders.
///
/// The engine owns the low-level codec and hook object. It keeps the common
/// buffered encoding loop private: input-index validation, output-capacity
/// checks, input consumption, output progress, and [`TranscodeStatus`]
/// reporting.
///
/// # Type Parameters
///
/// - `C`: Low-level codec used by the engine.
/// - `H`: Policy hook object used by the engine.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct BufferedEncodeEngine<C, H> {
    /// Low-level codec used for one-value encoding.
    codec: C,
    /// Policy hooks used for planning and writing values.
    hooks: H,
}

impl<C, H> BufferedEncodeEngine<C, H> {
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

    /// Returns the maximum output units needed for `input_len` values.
    ///
    /// # Parameters
    ///
    /// - `input_len`: Number of input values the caller plans to encode.
    ///
    /// # Returns
    ///
    /// Returns a conservative upper bound, or `None` on overflow.
    #[must_use]
    #[inline(always)]
    pub fn max_output_len<Value, Unit>(&self, input_len: usize) -> Option<usize>
    where
        C: Codec<Value, Unit>,
        H: BufferedEncodeHooks<C, Value, Unit>,
        Unit: Copy,
    {
        debug_assert_unit_bounds::<C, Value, Unit>(&self.codec);
        self.hooks.max_output_len(&self.codec, input_len)
    }

    /// Returns the maximum output units emitted by finishing hook-owned state.
    ///
    /// # Returns
    ///
    /// Returns the hook-provided final output bound.
    #[must_use]
    #[inline(always)]
    pub fn max_finish_output_len<Value, Unit>(&self) -> Option<usize>
    where
        C: Codec<Value, Unit>,
        H: BufferedEncodeHooks<C, Value, Unit>,
        Unit: Copy,
    {
        self.hooks.max_finish_output_len(&self.codec)
    }

    /// Resets hook-owned state.
    #[inline(always)]
    pub fn reset<Value, Unit>(&mut self)
    where
        C: Codec<Value, Unit>,
        H: BufferedEncodeHooks<C, Value, Unit>,
        Unit: Copy,
    {
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
    #[inline]
    pub fn transcode<Value, Unit>(
        &mut self,
        input: &[Value],
        input_index: usize,
        output: &mut [Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, <H as BufferedEncodeHooks<C, Value, Unit>>::Error>
    where
        C: Codec<Value, Unit>,
        H: BufferedEncodeHooks<C, Value, Unit>,
        Unit: Copy,
    {
        if input_index > input.len() {
            return Err(<H::Error as EncodeErrorFactory<C>>::invalid_input_index(
                &self.codec,
                input_index,
                input.len(),
            ));
        }
        debug_assert_unit_bounds::<C, Value, Unit>(&self.codec);
        let mut state = EncodeState::new(input, input_index, output, output_index);
        if !state.output_cursor_in_bounds() {
            return Ok(state.need_output_progress(self.codec.max_units_per_value().get()));
        }

        while state.has_input() {
            let (input_value, input_cursor) = state.current_input();
            let plan = self.hooks.prepare_encode(&self.codec, input_value, input_cursor)?;
            let max_output_units = plan.max_output_units;
            if !state.has_output_for(max_output_units) {
                return Ok(state.need_output_progress(max_output_units));
            }

            let (input_value, input_cursor, output, output_cursor) = state.write_parts();
            // SAFETY: The capacity check above guarantees the bound requested
            // by the prepared plan.
            let written = unsafe {
                self.hooks.write_encode(
                    &self.codec,
                    input_value,
                    input_cursor,
                    plan.payload,
                    output,
                    output_cursor,
                )
            }?;
            debug_assert!(
                written <= max_output_units,
                "BufferedEncodeEngine hook wrote beyond its prepared capacity bound",
            );
            state.accept_written_value(written);
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
    #[inline(always)]
    pub fn finish<Value, Unit>(
        &mut self,
        output: &mut [Unit],
        output_index: usize,
    ) -> Result<TranscodeProgress, <H as BufferedEncodeHooks<C, Value, Unit>>::Error>
    where
        C: Codec<Value, Unit>,
        H: BufferedEncodeHooks<C, Value, Unit>,
        Unit: Copy,
    {
        if output_index > output.len() {
            let additional = self.hooks.max_finish_output_len(&self.codec).unwrap_or(1).max(1);
            return Ok(TranscodeProgress::need_output(output_index, additional, 0, 0, 0));
        }
        self.hooks.finish(&self.codec, output, output_index)
    }
}

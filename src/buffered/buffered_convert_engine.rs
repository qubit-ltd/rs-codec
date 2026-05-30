/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Reusable buffered converter engine.

use core::{
    marker::PhantomData,
    num::NonZeroUsize,
};

use super::{
    buffered_convert_hooks::BufferedConvertHooks,
    convert_state::ConvertState,
    transcode_progress::TranscodeProgress,
};
use crate::ConvertErrorFactory;

/// Reusable buffered conversion engine.
///
/// The engine owns source and target conversion components plus a hook object.
/// It keeps common buffered converter control flow private: index validation,
/// pending-output draining, repeated one-step conversion, finalization dispatch,
/// and [`crate::TranscodeStatus`] progress reporting.
///
/// # Type Parameters
///
/// - `D`: Source-side decoder or input component.
/// - `E`: Target-side encoder or output component.
/// - `H`: Policy hook object used by the engine.
/// - `Input`: Source unit type.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct BufferedConvertEngine<D, E, H, Input> {
    /// Source-side conversion component.
    decoder: D,
    /// Target-side conversion component.
    encoder: E,
    /// Policy hooks used by the converter.
    hooks: H,
    /// Binds the engine to the source input unit type.
    marker: PhantomData<fn(Input)>,
}

impl<D, E, H, Input> BufferedConvertEngine<D, E, H, Input> {
    /// Creates a buffered converter engine.
    ///
    /// # Parameters
    ///
    /// - `decoder`: Source-side conversion component.
    /// - `encoder`: Target-side conversion component.
    /// - `hooks`: Policy hooks used by the converter.
    ///
    /// # Returns
    ///
    /// Returns a buffered converter engine.
    #[must_use]
    #[inline(always)]
    pub const fn new(decoder: D, encoder: E, hooks: H) -> Self {
        Self {
            decoder,
            encoder,
            hooks,
            marker: PhantomData,
        }
    }

    /// Returns the source-side component.
    #[must_use]
    #[inline(always)]
    pub const fn decoder(&self) -> &D {
        &self.decoder
    }

    /// Returns the target-side component.
    #[must_use]
    #[inline(always)]
    pub const fn encoder(&self) -> &E {
        &self.encoder
    }

    /// Returns the hook object.
    #[must_use]
    #[inline(always)]
    pub const fn hooks(&self) -> &H {
        &self.hooks
    }

    /// Returns the source-side component mutably.
    #[must_use]
    #[inline(always)]
    pub fn decoder_mut(&mut self) -> &mut D {
        &mut self.decoder
    }

    /// Returns the target-side component mutably.
    #[must_use]
    #[inline(always)]
    pub fn encoder_mut(&mut self) -> &mut E {
        &mut self.encoder
    }

    /// Returns the hook object mutably.
    #[must_use]
    #[inline(always)]
    pub fn hooks_mut(&mut self) -> &mut H {
        &mut self.hooks
    }

    /// Consumes the engine and returns its parts.
    ///
    /// # Returns
    ///
    /// Returns `(decoder, encoder, hooks)` supplied at construction time.
    #[must_use]
    #[inline(always)]
    pub fn into_parts(self) -> (D, E, H) {
        (self.decoder, self.encoder, self.hooks)
    }

    /// Returns an upper bound for target units produced from `input_len` units.
    #[must_use]
    #[inline(always)]
    pub fn max_output_len<Value, Output>(&self, input_len: usize) -> Option<usize>
    where
        H: BufferedConvertHooks<D, E, Input, Value, Output>,
    {
        self.hooks.max_output_len(&self.decoder, &self.encoder, input_len)
    }

    /// Returns the maximum target units emitted by finishing hook-owned state.
    #[must_use]
    #[inline(always)]
    pub fn max_finish_output_len<Value, Output>(&self) -> Option<usize>
    where
        H: BufferedConvertHooks<D, E, Input, Value, Output>,
    {
        self.hooks.max_finish_output_len(&self.decoder, &self.encoder)
    }

    /// Resets hook-owned and component-owned state.
    #[inline(always)]
    pub fn reset<Value, Output>(&mut self)
    where
        H: BufferedConvertHooks<D, E, Input, Value, Output>,
    {
        self.hooks.reset(&mut self.decoder, &mut self.encoder);
    }

    /// Converts source units into target units.
    ///
    /// # Parameters
    ///
    /// - `input`: Complete input unit slice visible to the converter.
    /// - `input_index`: Absolute input index where conversion starts.
    /// - `output`: Complete output unit slice visible to the converter.
    /// - `output_index`: Absolute output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns conversion progress.
    ///
    /// # Errors
    ///
    /// Returns hook errors when indices are invalid or concrete conversion fails.
    #[inline]
    pub fn transcode<Value, Output>(
        &mut self,
        input: &[Input],
        input_index: usize,
        output: &mut [Output],
        output_index: usize,
    ) -> Result<TranscodeProgress, <H as BufferedConvertHooks<D, E, Input, Value, Output>>::Error>
    where
        H: BufferedConvertHooks<D, E, Input, Value, Output>,
    {
        if input_index > input.len() {
            return Err(<H::Error as ConvertErrorFactory<D>>::invalid_input_index(
                &self.decoder,
                input_index,
                input.len(),
            ));
        }
        let mut state = ConvertState::new(input, input_index, output, output_index);
        if !state.output_cursor_in_bounds() {
            let additional = self.hooks.invalid_output_additional(&self.decoder, &self.encoder);
            return Ok(state.need_output_progress(additional, 0));
        }

        if let Some(progress) = self
            .hooks
            .drain_pending(&mut self.decoder, &mut self.encoder, &mut state)?
        {
            return Ok(progress);
        }

        while state.has_input() {
            let previous_read = state.read();
            let previous_written = state.written();
            if let Some(progress) = self
                .hooks
                .convert_next(&mut self.decoder, &mut self.encoder, &mut state)?
            {
                return Ok(progress);
            }
            debug_assert!(
                state.read() > previous_read || state.written() > previous_written,
                "BufferedConvertHooks::convert_next must make progress or stop",
            );
        }

        Ok(state.complete_progress())
    }

    /// Finishes hook-owned and component-owned output after EOF.
    ///
    /// # Parameters
    ///
    /// - `output`: Complete output unit slice visible to the converter.
    /// - `output_index`: Absolute output index where writing starts.
    ///
    /// # Returns
    ///
    /// Returns hook-provided finalization progress.
    ///
    /// # Errors
    ///
    /// Returns hook errors when finalization fails.
    #[inline]
    pub fn finish<Value, Output>(
        &mut self,
        output: &mut [Output],
        output_index: usize,
    ) -> Result<TranscodeProgress, <H as BufferedConvertHooks<D, E, Input, Value, Output>>::Error>
    where
        H: BufferedConvertHooks<D, E, Input, Value, Output>,
    {
        if output_index > output.len() {
            let additional = self
                .hooks
                .max_finish_output_len(&self.decoder, &self.encoder)
                .and_then(NonZeroUsize::new)
                .unwrap_or(NonZeroUsize::MIN);
            return Ok(TranscodeProgress::need_output(output_index, additional.get(), 0, 0, 0));
        }
        self.hooks
            .finish(&mut self.decoder, &mut self.encoder, output, output_index)
    }
}

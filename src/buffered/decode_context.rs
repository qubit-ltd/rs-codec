/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Decode context passed to buffered decoder policy hooks.

/// Context for one codec decode attempt inside a buffered decoder engine.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DecodeContext {
    /// Absolute source index where this `transcode` call starts.
    pub input_start: usize,
    /// Absolute source index where the attempted value starts.
    pub input_index: usize,
    /// Absolute output index where this `transcode` call starts.
    pub output_start: usize,
    /// Absolute output index where the next decoded value would be written.
    pub output_index: usize,
    /// Units visible to the codec from `input_index`.
    pub available: usize,
}

impl DecodeContext {
    /// Creates a decode context.
    ///
    /// # Parameters
    ///
    /// - `input_start`: Absolute source index where this `transcode` call starts.
    /// - `input_index`: Absolute source index where the attempted value starts.
    /// - `output_start`: Absolute output index where this `transcode` call starts.
    /// - `output_index`: Absolute output index where the next value would be written.
    /// - `available`: Units visible to the codec from `input_index`.
    ///
    /// # Returns
    ///
    /// Returns a decode context.
    #[must_use]
    #[inline(always)]
    pub const fn new(
        input_start: usize,
        input_index: usize,
        output_start: usize,
        output_index: usize,
        available: usize,
    ) -> Self {
        Self {
            input_start,
            input_index,
            output_start,
            output_index,
            available,
        }
    }

    /// Returns input units consumed since this `transcode` call started.
    ///
    /// # Returns
    ///
    /// Returns `input_index - input_start`.
    #[must_use]
    #[inline(always)]
    pub const fn input_used(self) -> usize {
        self.input_index - self.input_start
    }

    /// Returns output values written since this `transcode` call started.
    ///
    /// # Returns
    ///
    /// Returns `output_index - output_start`.
    #[must_use]
    #[inline(always)]
    pub const fn output_written(self) -> usize {
        self.output_index - self.output_start
    }
}

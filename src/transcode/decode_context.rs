// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Decode context passed to buffered decoder policy hooks.

/// Context for one codec decode attempt inside a buffered decoder engine.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DecodeContext {
    /// Absolute source index where this `transcode` call starts.
    input_start: usize,
    /// Absolute source index where the attempted value starts.
    input_index: usize,
    /// Absolute output index where this `transcode` call starts.
    output_start: usize,
    /// Absolute output index where the next decoded value would be written.
    output_index: usize,
    /// Units visible to the codec from `input_index`.
    available: usize,
}

impl DecodeContext {
    /// Creates a decode context.
    ///
    /// # Parameters
    ///
    /// - `input_start`: Absolute source index where this `transcode` call
    ///   starts.
    /// - `input_index`: Absolute source index where the attempted value starts.
    /// - `output_start`: Absolute output index where this `transcode` call
    ///   starts.
    /// - `output_index`: Absolute output index where the next value would be
    ///   written.
    /// - `available`: Units visible to the codec from `input_index`.
    ///
    /// # Returns
    ///
    /// Returns a decode context.
    ///
    /// # Panics
    ///
    /// Panics when `input_index < input_start` or
    /// `output_index < output_start`.
    #[inline(always)]
    #[must_use]
    pub const fn new(
        input_start: usize,
        input_index: usize,
        output_start: usize,
        output_index: usize,
        available: usize,
    ) -> Self {
        assert!(
            input_start <= input_index,
            "decode context input index must not precede input start",
        );
        assert!(
            output_start <= output_index,
            "decode context output index must not precede output start",
        );
        Self {
            input_start,
            input_index,
            output_start,
            output_index,
            available,
        }
    }

    /// Returns the absolute source index where this `transcode` call starts.
    ///
    /// # Returns
    ///
    /// Returns the input start index.
    #[inline(always)]
    #[must_use]
    pub const fn input_start(self) -> usize {
        self.input_start
    }

    /// Returns the absolute source index where the attempted value starts.
    ///
    /// # Returns
    ///
    /// Returns the current input index.
    #[inline(always)]
    #[must_use]
    pub const fn input_index(self) -> usize {
        self.input_index
    }

    /// Returns the absolute output index where this `transcode` call starts.
    ///
    /// # Returns
    ///
    /// Returns the output start index.
    #[inline(always)]
    #[must_use]
    pub const fn output_start(self) -> usize {
        self.output_start
    }

    /// Returns the absolute output index where the next decoded value would be
    /// written.
    ///
    /// # Returns
    ///
    /// Returns the current output index.
    #[inline(always)]
    #[must_use]
    pub const fn output_index(self) -> usize {
        self.output_index
    }

    /// Returns units visible to the codec from [`Self::input_index`].
    ///
    /// # Returns
    ///
    /// Returns the available input-unit count.
    #[inline(always)]
    #[must_use]
    pub const fn available(self) -> usize {
        self.available
    }

    /// Returns input units consumed since this `transcode` call started.
    ///
    /// # Returns
    ///
    /// Returns `input_index - input_start`.
    #[inline(always)]
    #[must_use]
    pub const fn input_used(self) -> usize {
        self.input_index - self.input_start
    }

    /// Returns output values written since this `transcode` call started.
    ///
    /// # Returns
    ///
    /// Returns `output_index - output_start`.
    #[inline(always)]
    #[must_use]
    pub const fn output_written(self) -> usize {
        self.output_index - self.output_start
    }
}

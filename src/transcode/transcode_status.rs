// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use core::num::NonZeroUsize;

/// Reports why a [`crate::Transcoder`] stopped converting input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TranscodeStatus {
    /// All currently supplied input was consumed.
    Complete,

    /// More input is needed to complete the next output value.
    ///
    /// The transcoder does not consume incomplete input tails. The caller
    /// should preserve `input[input_index..]`, refill the input buffer when
    /// more data is available, or apply its EOF policy when the upstream
    /// source is closed. Calling [`crate::Transcoder::finish`] does not pass
    /// this tail back to the transcoder.
    ///
    /// - `input_index`: Absolute input index where input ended while decoding.
    /// - `required`: Total input units required from the current input
    ///   position.
    /// - `available`: Number of input units currently available from the
    ///   current input position.
    NeedInput {
        /// Absolute input index where input ended.
        input_index: usize,
        /// Total input units required from the current input position.
        required: NonZeroUsize,
        /// Number of input units currently available.
        available: usize,
    },

    /// More output capacity is needed before conversion can continue.
    ///
    /// - `output_index`: Absolute output index where output ended while
    ///   decoding.
    /// - `required`: Total output units required from the current output
    ///   position.
    /// - `available`: Number of output units currently available from the
    ///   current output position.
    NeedOutput {
        /// Absolute output index where output ended.
        output_index: usize,
        /// Total output units required from the current output position.
        required: NonZeroUsize,
        /// Number of output units currently available.
        available: usize,
    },
}

impl TranscodeStatus {
    /// Creates a status that requests more input.
    ///
    /// # Parameters
    ///
    /// - `input_index`: Absolute input boundary where conversion stopped.
    /// - `required`: Total input units required from the current input
    ///   position.
    /// - `available`: Input units currently available at the boundary.
    ///
    /// # Returns
    ///
    /// Returns a [`TranscodeStatus::NeedInput`] value.
    #[inline(always)]
    #[must_use]
    pub const fn need_input(
        input_index: usize,
        required: NonZeroUsize,
        available: usize,
    ) -> Self {
        Self::NeedInput {
            input_index,
            required,
            available,
        }
    }

    /// Creates a status that requests more output capacity.
    ///
    /// # Parameters
    ///
    /// - `output_index`: Absolute output boundary where conversion stopped.
    /// - `required`: Total output units required from the current output
    ///   position.
    /// - `available`: Output units currently available at the boundary.
    ///
    /// # Returns
    ///
    /// Returns a [`TranscodeStatus::NeedOutput`] value.
    #[inline(always)]
    #[must_use]
    pub const fn need_output(
        output_index: usize,
        required: NonZeroUsize,
        available: usize,
    ) -> Self {
        Self::NeedOutput {
            output_index,
            required,
            available,
        }
    }
}

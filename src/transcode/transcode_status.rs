// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use core::num::NonZeroUsize;

/// Reports why a [`crate::Transcoder`] stopped converting input.
///
/// # Examples
///
/// ```
/// use core::num::NonZeroUsize;
/// use qubit_codec::TranscodeStatus;
///
/// let status = TranscodeStatus::need_output(NonZeroUsize::new(2).unwrap());
/// assert!(matches!(status, TranscodeStatus::NeedOutput { required } if required.get() == 2));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TranscodeStatus {
    /// All currently supplied input was consumed.
    ///
    /// A `Complete` status means the current [`crate::Transcoder::transcode`]
    /// call consumed every input unit visible from the supplied `input_index`.
    /// Implementations must report [`NeedInput`](Self::NeedInput) for an
    /// incomplete input tail and [`NeedOutput`](Self::NeedOutput) when output
    /// capacity prevents further input consumption.
    Complete,

    /// More input is needed to complete the next output value.
    ///
    /// The transcoder does not consume incomplete input tails. The caller
    /// should preserve `input[input_index..]`, refill the input buffer when
    /// more data is available, or apply its EOF policy when the upstream
    /// source is closed. Calling [`crate::Transcoder::finish`] does not pass
    /// this tail back to the transcoder.
    ///
    /// - `required`: Current minimum total input units required before retrying
    ///   from the current input position. A later retry may raise this lower
    ///   bound.
    NeedInput {
        /// Current minimum total input units required before retrying from the
        /// current input position. A later retry may raise this lower bound.
        required: NonZeroUsize,
    },

    /// More output capacity is needed before conversion can continue.
    ///
    /// - `required`: Total output units required from the current output
    ///   position.
    NeedOutput {
        /// Total output units required from the current output position.
        required: NonZeroUsize,
    },
}

impl TranscodeStatus {
    /// Creates a status that requests more input.
    ///
    /// # Parameters
    ///
    /// - `required`: Current minimum total input units required before retrying
    ///   from the current input position. A later retry may raise this lower
    ///   bound.
    ///
    /// # Returns
    ///
    /// Returns a [`TranscodeStatus::NeedInput`] value.
    #[inline(always)]
    #[must_use]
    pub const fn need_input(required: NonZeroUsize) -> Self {
        Self::NeedInput { required }
    }

    /// Creates a status that requests more output capacity.
    ///
    /// # Parameters
    ///
    /// - `required`: Total output units required from the current output
    ///   position.
    ///
    /// # Returns
    ///
    /// Returns a [`TranscodeStatus::NeedOutput`] value.
    #[inline(always)]
    #[must_use]
    pub const fn need_output(required: NonZeroUsize) -> Self {
        Self::NeedOutput { required }
    }
}

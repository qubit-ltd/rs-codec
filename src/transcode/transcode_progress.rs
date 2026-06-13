// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use core::num::NonZeroUsize;

use super::TranscodeStatus;

/// Counts how much work a [`crate::Transcoder`] completed before
/// returning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TranscodeProgress {
    /// Stop reason reported by the transcoder.
    status: TranscodeStatus,
    /// Number of input units consumed from the requested input index.
    read: usize,
    /// Number of output units written from the requested output index.
    written: usize,
}

impl TranscodeProgress {
    /// Creates a progress value.
    ///
    /// # Parameters
    ///
    /// - `status`: The reason conversion stopped.
    /// - `read`: Number of input units consumed from the call's input index.
    /// - `written`: Number of output units written from the call's output
    ///   index.
    ///
    /// # Returns
    ///
    /// Returns a progress value carrying the supplied counters.
    #[must_use]
    #[inline(always)]
    pub const fn new(status: TranscodeStatus, read: usize, written: usize) -> Self {
        Self {
            status,
            read,
            written,
        }
    }

    /// Creates a completed progress value.
    ///
    /// # Parameters
    ///
    /// - `read`: Number of consumed input units.
    /// - `written`: Number of produced output units.
    ///
    /// # Returns
    ///
    /// Returns a progress value whose status is [`TranscodeStatus::Complete`].
    #[must_use]
    #[inline(always)]
    pub const fn complete(read: usize, written: usize) -> Self {
        Self::new(TranscodeStatus::Complete, read, written)
    }

    /// Creates progress that stopped because more input is needed.
    ///
    /// # Parameters
    ///
    /// - `input_index`: Absolute input boundary where conversion stopped.
    /// - `additional`: Additional input units required to continue.
    /// - `available`: Input units currently available at the boundary.
    /// - `read`: Number of consumed input units.
    /// - `written`: Number of produced output units.
    ///
    /// # Returns
    ///
    /// Returns a progress value with [`TranscodeStatus::NeedInput`].
    #[must_use]
    #[inline(always)]
    pub const fn need_input(
        input_index: usize,
        additional: NonZeroUsize,
        available: usize,
        read: usize,
        written: usize,
    ) -> Self {
        Self::new(
            TranscodeStatus::need_input(input_index, additional, available),
            read,
            written,
        )
    }

    /// Creates progress that stopped because more output capacity is needed.
    ///
    /// # Parameters
    ///
    /// - `output_index`: Absolute output boundary where conversion stopped.
    /// - `additional`: Additional output units required to continue.
    /// - `available`: Output units currently available at the boundary.
    /// - `read`: Number of consumed input units.
    /// - `written`: Number of produced output units.
    ///
    /// # Returns
    ///
    /// Returns a progress value with [`TranscodeStatus::NeedOutput`].
    #[must_use]
    #[inline(always)]
    pub const fn need_output(
        output_index: usize,
        additional: NonZeroUsize,
        available: usize,
        read: usize,
        written: usize,
    ) -> Self {
        Self::new(
            TranscodeStatus::need_output(output_index, additional, available),
            read,
            written,
        )
    }

    /// Returns the status that stopped conversion.
    ///
    /// # Returns
    ///
    /// Returns the stored [`TranscodeStatus`].
    #[must_use]
    #[inline(always)]
    pub const fn status(self) -> TranscodeStatus {
        self.status
    }

    /// Returns the number of input units consumed by the call.
    ///
    /// # Returns
    ///
    /// Returns a count relative to the input index passed to the conversion
    /// call.
    #[must_use]
    #[inline(always)]
    pub const fn read(self) -> usize {
        self.read
    }

    /// Returns the number of output units written by the call.
    ///
    /// # Returns
    ///
    /// Returns a count relative to the output index passed to the conversion
    /// call.
    #[must_use]
    #[inline(always)]
    pub const fn written(self) -> usize {
        self.written
    }
}

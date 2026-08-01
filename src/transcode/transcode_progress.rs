// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use core::num::NonZeroUsize;

use super::{TranscodeContractError, TranscodeStatus};

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
    #[inline(always)]
    #[must_use]
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
    #[inline(always)]
    #[must_use]
    pub const fn complete(read: usize, written: usize) -> Self {
        Self::new(TranscodeStatus::Complete, read, written)
    }

    /// Creates progress that stopped because more input is needed.
    ///
    /// # Parameters
    ///
    /// - `required`: Current minimum total input units required before retrying
    ///   from the current input position. A later retry may raise this lower
    ///   bound.
    /// - `read`: Number of consumed input units.
    /// - `written`: Number of produced output units.
    ///
    /// # Returns
    ///
    /// Returns a progress value with [`TranscodeStatus::NeedInput`].
    #[inline(always)]
    #[must_use]
    pub const fn need_input(required: NonZeroUsize, read: usize, written: usize) -> Self {
        Self::new(TranscodeStatus::need_input(required), read, written)
    }

    /// Creates progress that stopped because more output capacity is needed.
    ///
    /// # Parameters
    ///
    /// - `required`: Total output units required from the current output
    ///   position.
    /// - `read`: Number of consumed input units.
    /// - `written`: Number of produced output units.
    ///
    /// # Returns
    ///
    /// Returns a progress value with [`TranscodeStatus::NeedOutput`].
    #[inline(always)]
    #[must_use]
    pub const fn need_output(required: NonZeroUsize, read: usize, written: usize) -> Self {
        Self::new(TranscodeStatus::need_output(required), read, written)
    }

    /// Returns the status that stopped conversion.
    ///
    /// # Returns
    ///
    /// Returns the stored [`TranscodeStatus`].
    #[inline(always)]
    #[must_use]
    pub const fn status(self) -> TranscodeStatus {
        self.status
    }

    /// Returns whether conversion consumed all currently supplied input.
    ///
    /// # Returns
    ///
    /// Returns `true` when the stored status is
    /// [`TranscodeStatus::Complete`].
    #[inline(always)]
    #[must_use]
    pub const fn is_complete(self) -> bool {
        matches!(self.status, TranscodeStatus::Complete)
    }

    /// Returns whether conversion stopped because more input is needed.
    ///
    /// # Returns
    ///
    /// Returns `true` when the stored status is
    /// [`TranscodeStatus::NeedInput`].
    #[inline(always)]
    #[must_use]
    pub const fn is_need_input(self) -> bool {
        matches!(self.status, TranscodeStatus::NeedInput { .. })
    }

    /// Returns whether conversion stopped because more output capacity is
    /// needed.
    ///
    /// # Returns
    ///
    /// Returns `true` when the stored status is
    /// [`TranscodeStatus::NeedOutput`].
    #[inline(always)]
    #[must_use]
    pub const fn is_need_output(self) -> bool {
        matches!(self.status, TranscodeStatus::NeedOutput { .. })
    }

    /// Returns the number of input units consumed by the call.
    ///
    /// # Returns
    ///
    /// Returns a count relative to the input index passed to the conversion
    /// call.
    #[inline(always)]
    #[must_use]
    pub const fn read(self) -> usize {
        self.read
    }

    /// Returns the number of output units written by the call.
    ///
    /// # Returns
    ///
    /// Returns a count relative to the output index passed to the conversion
    /// call.
    #[inline(always)]
    #[must_use]
    pub const fn written(self) -> usize {
        self.written
    }

    /// Validates this progress against the call bounds supplied to a
    /// transcoder.
    ///
    /// Buffered drivers should call this before using [`Self::read`] or
    /// [`Self::written`] to advance unchecked input or output cursors. The
    /// method checks relative counters, unsatisfied `NeedInput` / `NeedOutput`
    /// requirements, and the `Complete` invariant that all visible input was
    /// consumed.
    ///
    /// This is a contract checker, not a semantic recovery policy. It performs
    /// the same validation in every build profile. Drivers that advance unsafe
    /// cursors may convert failures into their own error type, while
    /// convenience helpers may assert that validation succeeds. Custom
    /// [`crate::Transcoder`] implementations must therefore return progress
    /// that satisfies the documented contract in all builds.
    ///
    /// # Parameters
    ///
    /// - `input_index`: Input index originally passed to the transcoder.
    /// - `available_input`: Number of input units visible from `input_index`.
    /// - `output_index`: Output index originally passed to the transcoder.
    /// - `available_output`: Number of output slots visible from
    ///   `output_index`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when progress is internally consistent with the
    /// supplied call bounds.
    ///
    /// # Errors
    ///
    /// Returns [`TranscodeContractError`] when a custom transcoder reports
    /// counters or missing-capacity requirements that do not match the buffers
    /// supplied by the caller.
    pub fn validate(
        &self,
        input_index: usize,
        available_input: usize,
        output_index: usize,
        available_output: usize,
    ) -> Result<(), TranscodeContractError> {
        if self.read > available_input {
            return Err(TranscodeContractError::OverRead {
                read: self.read,
                available: available_input,
            });
        }
        if self.written > available_output {
            return Err(TranscodeContractError::OverWritten {
                written: self.written,
                available: available_output,
            });
        }

        input_index.checked_add(self.read).ok_or(
            TranscodeContractError::ProgressIndexOverflow {
                index: input_index,
                advanced: self.read,
            },
        )?;
        output_index.checked_add(self.written).ok_or(
            TranscodeContractError::ProgressIndexOverflow {
                index: output_index,
                advanced: self.written,
            },
        )?;
        match self.status {
            TranscodeStatus::Complete => {
                if self.read != available_input {
                    return Err(TranscodeContractError::CompleteWithRemainingInput {
                        read: self.read,
                        available: available_input,
                    });
                }
                Ok(())
            }
            TranscodeStatus::NeedInput { required } => {
                let available = available_input - self.read;
                if required.get() <= available {
                    return Err(TranscodeContractError::SatisfiedNeed {
                        required: required.get(),
                        available,
                    });
                }
                Ok(())
            }
            TranscodeStatus::NeedOutput { required } => {
                let available = available_output - self.written;
                if required.get() <= available {
                    return Err(TranscodeContractError::SatisfiedNeed {
                        required: required.get(),
                        available,
                    });
                }
                Ok(())
            }
        }
    }
}

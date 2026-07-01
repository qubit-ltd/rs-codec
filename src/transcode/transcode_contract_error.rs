// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
use thiserror::Error;

/// Error reported when a transcoder returns inconsistent progress.
///
/// This error represents a broken [`crate::Transcoder`] implementation rather
/// than malformed input data. Buffered drivers call
/// [`crate::TranscodeProgress::validate`] before trusting progress counters for
/// unchecked buffer cursor movement.
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
#[non_exhaustive]
pub enum TranscodeContractError {
    /// The transcoder consumed more input units than the caller supplied.
    #[error("transcoder consumed {read} units but only {available} were available")]
    OverRead {
        /// Input units reported as consumed.
        read: usize,
        /// Input units available to the transcode call.
        available: usize,
    },

    /// The transcoder wrote more output units than the caller supplied.
    #[error("transcoder wrote {written} units but only {available} output slots were available")]
    OverWritten {
        /// Output units reported as written.
        written: usize,
        /// Output slots available to the transcode call.
        available: usize,
    },

    /// Progress could not be represented as an absolute index.
    #[error("transcoder progress overflow: index {index} plus advanced {advanced}")]
    ProgressIndexOverflow {
        /// Absolute index supplied to the transcode call.
        index: usize,
        /// Relative progress reported by the transcoder.
        advanced: usize,
    },

    /// A status reported an index that does not match relative progress.
    #[error("transcoder reported status index {reported}, expected {expected}")]
    StatusIndexMismatch {
        /// Index reported by the status.
        reported: usize,
        /// Index implied by the progress counter.
        expected: usize,
    },

    /// A status reported an available count that does not match progress.
    #[error("transcoder reported status available {reported}, expected {expected}")]
    StatusAvailableMismatch {
        /// Available count reported by the status.
        reported: usize,
        /// Available count implied by the progress counter and call bounds.
        expected: usize,
    },

    /// A status requested input or output that is already available.
    #[error("transcoder reported required {required} with available {available}")]
    SatisfiedNeed {
        /// Required units reported by the status.
        required: usize,
        /// Available units reported by the status.
        available: usize,
    },
}

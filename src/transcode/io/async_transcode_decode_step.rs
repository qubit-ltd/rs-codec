// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Result type for one asynchronous decode operation.

use crate::TranscodeProgress;

/// Result of one cancellation-safe asynchronous decode operation.
///
/// # Examples
///
/// ```
/// use qubit_codec::AsyncTranscodeDecodeStep;
///
/// let step = AsyncTranscodeDecodeStep::EndOfInput;
/// assert!(matches!(step, AsyncTranscodeDecodeStep::EndOfInput));
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[must_use]
pub enum AsyncTranscodeDecodeStep {
    /// The wrapped input reached EOF with no unread units. Call
    /// [`crate::AsyncTranscodeDecodeInput::transcode_eof_step`] to dispatch
    /// that EOF event to the decoder.
    EndOfInput,
    /// The result of one decode step, returned without another await.
    /// Both `read()` and `written()` may be zero. Inspect the stop status to
    /// decide whether to refill input, provide output capacity, or handle EOF.
    Progress(
        /// Consumed and written counts and the decoder stop status.
        TranscodeProgress,
    ),
}

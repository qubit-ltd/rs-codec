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
pub enum AsyncTranscodeDecodeStep {
    /// The wrapped input reached EOF with no unread units.
    EndOfInput,
    /// The decoder consumed input or initialized output without another await.
    Progress(TranscodeProgress),
}

// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Stable value-codec ID errors.

use thiserror::Error;

/// A stable value-codec ID protocol violation.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ValueCodecIdError {
    /// The complete ID is empty.
    #[error("value codec ID cannot be empty")]
    Empty,
    /// One dot-separated segment is empty.
    #[error("value codec ID contains an empty segment")]
    EmptySegment,
    /// One segment has an invalid initial byte or character.
    #[error("value codec ID contains an invalid segment")]
    InvalidSegment,
}

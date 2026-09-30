// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Stable value-codec ID errors.

use thiserror::Error;

/// A stable value-codec ID protocol violation.
///
/// # Examples
///
/// ```
/// use qubit_codec::ValueCodecId;
/// use qubit_codec::ValueCodecIdError;
///
/// let error = ValueCodecId::try_new("").unwrap_err();
/// assert_eq!(error, ValueCodecIdError::Empty);
/// assert_eq!(error.to_string(), "value codec ID cannot be empty");
/// ```
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[must_use]
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

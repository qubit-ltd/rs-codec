// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Stable value-codec identifiers.

use core::borrow::Borrow;

use crate::ValueCodecIdError;

/// A validated, process-independent value-codec identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ValueCodecId(&'static str);

impl ValueCodecId {
    /// Creates an ID from a static string.
    ///
    /// # Panics
    ///
    /// Panics when `value` violates the point-separated ASCII protocol.
    #[must_use]
    pub const fn new(value: &'static str) -> Self {
        match Self::try_new(value) {
            Ok(id) => id,
            Err(_) => panic!("invalid value codec ID"),
        }
    }

    /// Validates and creates an ID.
    ///
    /// # Errors
    ///
    /// Returns the exact stable-ID protocol violation.
    pub const fn try_new(value: &'static str) -> Result<Self, ValueCodecIdError> {
        match validate(value) {
            Ok(()) => Ok(Self(value)),
            Err(error) => Err(error),
        }
    }

    /// Returns the complete stable ID.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl Borrow<str> for ValueCodecId {
    fn borrow(&self) -> &str {
        self.0
    }
}

/// Validates one point-separated ASCII identifier.
const fn validate(value: &str) -> Result<(), ValueCodecIdError> {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return Err(ValueCodecIdError::Empty);
    }
    let mut start = 0;
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'.' {
            if start == index || index + 1 == bytes.len() {
                return Err(ValueCodecIdError::EmptySegment);
            }
            if let Err(error) = validate_segment(bytes, start, index) {
                return Err(error);
            }
            start = index + 1;
        }
        index += 1;
    }
    validate_segment(bytes, start, bytes.len())
}

/// Validates one non-empty ID segment.
const fn validate_segment(bytes: &[u8], start: usize, end: usize) -> Result<(), ValueCodecIdError> {
    if start == end || !bytes[start].is_ascii_alphabetic() {
        return Err(ValueCodecIdError::InvalidSegment);
    }
    let mut index = start + 1;
    while index < end {
        let byte = bytes[index];
        if !(byte.is_ascii_alphanumeric() || byte == b'_') {
            return Err(ValueCodecIdError::InvalidSegment);
        }
        index += 1;
    }
    Ok(())
}

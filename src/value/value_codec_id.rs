// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Stable value-codec identifiers.

use core::borrow::Borrow;

use crate::ValueCodecIdError;

/// A validated, process-independent value-codec identifier.
///
/// # Examples
///
/// ```
/// use qubit_codec::ValueCodecId;
///
/// let id = ValueCodecId::new("example.u16");
/// assert_eq!(id.as_str(), "example.u16");
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ValueCodecId(
    /// Validated static identifier, borrowed without allocation.
    &'static str,
);

impl ValueCodecId {
    /// Creates an ID from a static string.
    ///
    /// # Parameters
    ///
    /// - `value`: Non-empty dot-separated ASCII segments; each starts with a
    ///   letter and continues with letters, digits, or underscores.
    ///
    /// # Returns
    ///
    /// An ID borrowing the validated static string without allocation.
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
    /// # Parameters
    ///
    /// - `value`: Non-empty dot-separated ASCII segments; each starts with a
    ///   letter and continues with letters, digits, or underscores.
    ///
    /// # Returns
    ///
    /// An ID borrowing the validated static string without allocation.
    ///
    /// # Errors
    ///
    /// Returns Empty for an empty ID, EmptySegment for leading/trailing or
    /// consecutive dots, or InvalidSegment for an invalid ASCII segment.
    pub const fn try_new(value: &'static str) -> Result<Self, ValueCodecIdError> {
        match validate(value) {
            Ok(()) => Ok(Self(value)),
            Err(error) => Err(error),
        }
    }

    /// Returns the complete stable ID.
    ///
    /// # Returns
    ///
    /// The original static string; consuming this Copy ID does not invalidate
    /// it.
    #[inline]
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl Borrow<str> for ValueCodecId {
    /// Borrows the stable identifier for lookup without allocating.
    ///
    /// # Returns
    ///
    /// The complete ID string borrowed through this lookup key.
    #[inline]
    fn borrow(&self) -> &str {
        self.0
    }
}

/// Validates one point-separated ASCII identifier in linear time.
///
/// # Parameters
///
/// - `value`: Candidate identifier, borrowed without allocating.
///
/// # Returns
///
/// Unit when every segment follows the ASCII protocol.
///
/// # Errors
///
/// Returns Empty for no bytes, EmptySegment for a dot-delimited empty part,
/// or InvalidSegment for a non-letter start or unsupported continuation.
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

/// Validates one non-empty ID segment in linear time.
///
/// # Parameters
///
/// - `bytes`: Complete identifier storage.
/// - `start`: Inclusive segment boundary, no greater than `end`.
/// - `end`: Exclusive boundary, no greater than `bytes.len()`.
///
/// # Returns
///
/// Unit for an ASCII letter followed by ASCII letters, digits, or underscores.
///
/// # Errors
///
/// Returns InvalidSegment for an empty segment or a disallowed byte.
///
/// # Panics
///
/// May panic if the caller violates the documented index bounds.
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

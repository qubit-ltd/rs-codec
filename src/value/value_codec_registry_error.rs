// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Value-codec registry errors.

use thiserror::Error;

use crate::ValueCodecRegistrationSource;

/// Failure while freezing a value-codec registry.
///
/// Freezing rejects a registry that files two registrations under one stable
/// ID, and reports the first conflicting ID together with every declaration
/// site that claimed it.
///
/// # Examples
///
/// ```
/// use qubit_codec::ValueCodecRegistrationSource;
/// use qubit_codec::ValueCodecRegistryError;
///
/// let error = ValueCodecRegistryError::DuplicateId {
///     id: "example.u32",
///     sources: vec![ValueCodecRegistrationSource::new(
///         "qubit-codec",
///         "my_crate::codecs",
///         "src/codec.rs",
///         42,
///     )],
/// };
/// assert!(error.to_string().starts_with("duplicate value codec ID example.u32"));
/// ```
#[derive(Clone, Debug, Error)]
#[must_use]
pub enum ValueCodecRegistryError {
    /// Multiple registrations claim one stable ID.
    #[error("duplicate value codec ID {id} from {sources:?}")]
    DuplicateId {
        /// Conflicting stable ID.
        id: &'static str,
        /// Registration sources in deterministic order.
        sources: Vec<ValueCodecRegistrationSource>,
    },
}

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
/// Freezing rejects duplicate stable IDs and reports the lexically first
/// conflicting ID together with all of its declaration sites. Sources are
/// sorted by their fields; repeated identical sources are retained because
/// each registration is reported.
///
/// The registry returns this error when `ValueStringCodecRegistry::try_global`
/// or `ValueBytesCodecRegistry::try_global` encounters conflicting linked
/// registrations.
///
/// # Examples
///
/// ```
/// use qubit_codec::ValueBytesCodecRegistry;
/// use qubit_codec::ValueCodecRegistryError;
///
/// fn describe_registry_error(error: ValueCodecRegistryError) -> String {
///     match error {
///         ValueCodecRegistryError::DuplicateId { id, sources } => {
///             format!("duplicate value codec ID {id} from {} source(s)", sources.len())
///         }
///     }
/// }
///
/// if let Err(error) = ValueBytesCodecRegistry::try_global() {
///     let description = describe_registry_error(error);
///     assert!(description.starts_with("duplicate value codec ID "));
/// }
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

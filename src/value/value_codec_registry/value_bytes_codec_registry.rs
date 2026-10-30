// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Process-wide registry for bytes-wire value codecs.

use std::sync::OnceLock;

use inventory::iter;

use crate::ValueBytesCodecDescriptor;
use crate::ValueBytesCodecRegistrationFactory;
use crate::ValueCodecRegistry;
use crate::ValueCodecRegistryError;

/// An immutable bytes-wire value-codec registry sorted by stable ID.
///
/// # Examples
///
/// ```
/// use qubit_codec::ValueBytesCodecRegistry;
///
/// let registry = ValueBytesCodecRegistry::empty();
/// assert!(registry.get("missing.example").is_none());
/// ```
pub type ValueBytesCodecRegistry = ValueCodecRegistry<ValueBytesCodecDescriptor>;

impl ValueBytesCodecRegistry {
    /// Initializes and returns the process-wide linked bytes registry.
    ///
    /// Initialization is synchronized and cached, including construction
    /// failures.
    ///
    /// # Returns
    ///
    /// A shared static registry containing all linked bytes registrations.
    ///
    /// # Errors
    ///
    /// Returns the cached construction error when linked registrations
    /// conflict. The error identifies the lexically first duplicate ID and
    /// all declarations for that ID in source-field order.
    pub fn try_global() -> Result<&'static Self, ValueCodecRegistryError> {
        // Caches both successful construction and duplicate-ID failures.
        static REGISTRY: OnceLock<Result<ValueBytesCodecRegistry, ValueCodecRegistryError>> = OnceLock::new();
        match REGISTRY.get_or_init(|| {
            let registrations = iter::<ValueBytesCodecRegistrationFactory>
                .into_iter()
                .map(|factory| (factory.0)())
                .collect();
            ValueCodecRegistry::build(registrations)
        }) {
            Ok(registry) => Ok(registry),
            Err(error) => Err(error.clone()),
        }
    }

    /// Returns the process-wide bytes registry or panics with a stable
    /// diagnostic.
    ///
    /// # Returns
    ///
    /// A shared static registry, initialized at most once by `try_global`.
    ///
    /// # Panics
    ///
    /// Panics when linked registrations conflict.
    #[must_use]
    pub fn global() -> &'static Self {
        Self::try_global().unwrap_or_else(|error| panic!("invalid global value bytes codec registry: {error}"))
    }
}

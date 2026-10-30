// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Process-wide registry for string-wire value codecs.

use std::sync::OnceLock;

use inventory::iter;

use crate::ValueCodecRegistry;
use crate::ValueCodecRegistryError;
use crate::ValueStringCodecDescriptor;
use crate::ValueStringCodecRegistrationFactory;

/// An immutable string-wire value-codec registry sorted by stable ID.
///
/// # Examples
///
/// ```
/// use qubit_codec::ValueStringCodecRegistry;
///
/// let registry = ValueStringCodecRegistry::empty();
/// assert!(registry.get("missing.example").is_none());
/// ```
pub type ValueStringCodecRegistry = ValueCodecRegistry<ValueStringCodecDescriptor>;

impl ValueStringCodecRegistry {
    /// Initializes and returns the process-wide linked string registry.
    ///
    /// Initialization is synchronized and cached, including construction
    /// failures.
    ///
    /// # Returns
    ///
    /// A shared static registry containing all linked string registrations.
    ///
    /// # Errors
    ///
    /// Returns the cached construction error when linked registrations
    /// conflict. The error identifies the lexically first duplicate ID and
    /// all declarations for that ID in source-field order.
    pub fn try_global() -> Result<&'static Self, ValueCodecRegistryError> {
        // Caches both successful construction and duplicate-ID failures.
        static REGISTRY: OnceLock<Result<ValueStringCodecRegistry, ValueCodecRegistryError>> = OnceLock::new();
        match REGISTRY.get_or_init(|| {
            let registrations = iter::<ValueStringCodecRegistrationFactory>
                .into_iter()
                .map(|factory| (factory.0)())
                .collect();
            ValueCodecRegistry::build(registrations)
        }) {
            Ok(registry) => Ok(registry),
            Err(error) => Err(error.clone()),
        }
    }

    /// Returns the process-wide string registry or panics with a stable
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
    #[inline]
    pub fn global() -> &'static Self {
        Self::try_global().unwrap_or_else(|error| panic!("invalid global value string codec registry: {error}"))
    }
}

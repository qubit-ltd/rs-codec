// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Immutable local and process-wide value-codec registries.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::ValueCodecId;
use crate::ValueCodecRegistration;
use crate::ValueCodecRegistrationFactory;
use crate::ValueCodecRegistryError;

/// An immutable value-codec registry sorted by stable ID.
///
/// # Examples
///
/// ```
/// use qubit_codec::ValueCodecRegistry;
///
/// let registry = ValueCodecRegistry::empty();
/// assert!(registry.get("missing.example").is_none());
/// ```
#[derive(Debug)]
pub struct ValueCodecRegistry {
    registrations: Box<[ValueCodecRegistration]>,
    indices: BTreeMap<ValueCodecId, usize>,
}

impl ValueCodecRegistry {
    /// Builds a local registry from static registrations.
    ///
    /// # Errors
    ///
    /// Returns a duplicate-ID error when multiple entries claim one ID.
    pub fn from_registrations(
        registrations: impl IntoIterator<Item = &'static ValueCodecRegistration>,
    ) -> Result<Self, ValueCodecRegistryError> {
        Self::build(registrations.into_iter().copied().collect())
    }

    /// Returns an empty local registry.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            registrations: Box::new([]),
            indices: BTreeMap::new(),
        }
    }

    /// Initializes and returns the process-wide linked registry.
    ///
    /// # Errors
    ///
    /// Returns the cached construction error when linked registrations
    /// conflict.
    pub fn try_global() -> Result<&'static Self, ValueCodecRegistryError> {
        static REGISTRY: OnceLock<Result<ValueCodecRegistry, ValueCodecRegistryError>> = OnceLock::new();
        match REGISTRY.get_or_init(|| {
            let registrations = inventory::iter::<ValueCodecRegistrationFactory>
                .into_iter()
                .map(|factory| (factory.0)())
                .collect();
            Self::build(registrations)
        }) {
            Ok(registry) => Ok(registry),
            Err(error) => Err(error.clone()),
        }
    }

    /// Returns the process-wide registry or panics with a stable diagnostic.
    ///
    /// # Panics
    ///
    /// Panics when linked registrations conflict.
    #[must_use]
    pub fn global() -> &'static Self {
        Self::try_global().unwrap_or_else(|error| panic!("invalid global value codec registry: {error}"))
    }

    /// Finds a registration by stable ID.
    ///
    /// Returns `None` when `id` is absent.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&ValueCodecRegistration> {
        let index = *self.indices.get(id)?;
        self.registrations.get(index)
    }

    /// Returns registrations in deterministic ID order.
    #[must_use]
    pub fn registrations(&self) -> &[ValueCodecRegistration] {
        &self.registrations
    }

    /// Freezes registrations and rejects duplicate IDs.
    fn build(mut registrations: Vec<ValueCodecRegistration>) -> Result<Self, ValueCodecRegistryError> {
        registrations.sort_by_key(ValueCodecRegistration::id);
        for pair in registrations.windows(2) {
            if pair[0].id() == pair[1].id() {
                let mut sources = pair.iter().map(ValueCodecRegistration::source).collect::<Vec<_>>();
                sources.sort_unstable();
                return Err(ValueCodecRegistryError::DuplicateId {
                    id: pair[0].id().as_str(),
                    sources,
                });
            }
        }
        let indices = registrations
            .iter()
            .enumerate()
            .map(|(index, registration)| (registration.id(), index))
            .collect();
        Ok(Self {
            registrations: registrations.into_boxed_slice(),
            indices,
        })
    }
}

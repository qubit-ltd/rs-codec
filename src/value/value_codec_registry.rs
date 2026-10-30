// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Immutable local and process-wide value-codec registries.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use inventory::iter;

use crate::ValueBytesCodecDescriptor;
use crate::ValueBytesCodecRegistrationFactory;
use crate::ValueCodecId;
use crate::ValueCodecRegistration;
use crate::ValueCodecRegistryError;
use crate::ValueStringCodecDescriptor;
use crate::ValueStringCodecRegistrationFactory;

/// An immutable string-wire value-codec registry sorted by stable ID.
pub type ValueStringCodecRegistry = ValueCodecRegistry<ValueStringCodecDescriptor>;

/// An immutable bytes-wire value-codec registry sorted by stable ID.
pub type ValueBytesCodecRegistry = ValueCodecRegistry<ValueBytesCodecDescriptor>;

/// An immutable value-codec registry sorted by stable ID.
///
/// # Type Parameters
///
/// - `D`: Copyable, static descriptor identifying one wire representation.
///
/// # Examples
///
/// ```
/// use qubit_codec::ValueCodecRegistry;
/// use qubit_codec::ValueStringCodecDescriptor;
/// use qubit_codec::ValueStringCodecRegistry;
///
/// let registry: ValueCodecRegistry<ValueStringCodecDescriptor> =
///     ValueStringCodecRegistry::empty();
/// assert!(registry.get("missing.example").is_none());
/// ```
#[derive(Debug)]
pub struct ValueCodecRegistry<D: 'static> {
    /// Owned registrations sorted once by stable ID during construction.
    registrations: Box<[ValueCodecRegistration<D>]>,
    /// Stable ID to position mapping into the immutable registration slice.
    indices: BTreeMap<ValueCodecId, usize>,
}

impl<D: Copy + 'static> ValueCodecRegistry<D> {
    /// Builds a local registry from static registrations without consulting
    /// or modifying the process-wide registry.
    ///
    /// # Parameters
    ///
    /// - `registrations`: Static entries copied into this registry; input order
    ///   is ignored.
    ///
    /// # Returns
    ///
    /// An owned, immutable registry sorted by ID, independent of the iterator.
    ///
    /// # Errors
    ///
    /// Returns a duplicate-ID error when multiple entries claim one ID.
    ///
    /// # Example
    ///
    /// This example uses a local static registration; it does not submit an
    /// entry to the process-wide inventory.
    ///
    /// ```
    /// use std::convert::Infallible;
    /// use qubit_codec::ValueCodecId;
    /// use qubit_codec::ValueCodecRegistration;
    /// use qubit_codec::ValueCodecRegistrationSource;
    /// use qubit_codec::ValueDecoder;
    /// use qubit_codec::ValueEncoder;
    /// use qubit_codec::ValueStringCodecDescriptor;
    /// use qubit_codec::ValueStringCodecRegistration;
    /// use qubit_codec::ValueStringCodecRegistry;
    ///
    /// #[derive(Default)]
    /// struct LocalText;
    ///
    /// impl ValueEncoder<String> for LocalText {
    ///     type Output = String;
    ///     type Error = Infallible;
    ///
    ///     fn encode(&mut self, input: &String) -> Result<String, Infallible> {
    ///         Ok(input.clone())
    ///     }
    /// }
    ///
    /// impl ValueDecoder<str> for LocalText {
    ///     type Output = String;
    ///     type Error = Infallible;
    ///
    ///     fn decode(&mut self, input: &str) -> Result<String, Infallible> {
    ///         Ok(input.to_owned())
    ///     }
    /// }
    ///
    /// static DESCRIPTOR: ValueStringCodecDescriptor =
    ///     ValueStringCodecDescriptor::of::<LocalText, String>();
    /// static REGISTRATION: ValueStringCodecRegistration = ValueCodecRegistration::new(
    ///     ValueCodecId::new("example.local.text"),
    ///     &DESCRIPTOR,
    ///     ValueCodecRegistrationSource::new("local-example", "example", file!(), line!()),
    /// );
    ///
    /// let registry = ValueStringCodecRegistry::from_registrations([&REGISTRATION])
    ///     .expect("unique local registration");
    /// let descriptor = registry.get("example.local.text")
    ///     .expect("registered locally").descriptor();
    /// let encoded = descriptor.encode(&String::from("hello")).expect("encode");
    /// assert_eq!("hello", encoded);
    /// let decoded = descriptor.decode(&encoded).expect("decode");
    /// assert_eq!("hello", *decoded.downcast::<String>().expect("String output"));
    /// assert!(registry.get("example.missing").is_none());
    /// ```
    pub fn from_registrations(
        registrations: impl IntoIterator<Item = &'static ValueCodecRegistration<D>>,
    ) -> Result<Self, ValueCodecRegistryError> {
        Self::build(registrations.into_iter().copied().collect())
    }

    /// Returns an empty local registry.
    ///
    /// # Returns
    ///
    /// A local registry with no registrations or indexed IDs.
    #[must_use]
    #[inline]
    pub fn empty() -> Self {
        Self {
            registrations: Box::new([]),
            indices: BTreeMap::new(),
        }
    }

    /// Freezes registrations and rejects duplicate IDs.
    ///
    /// # Parameters
    ///
    /// - `registrations`: Owned entries sorted in place before indexing.
    ///
    /// # Returns
    ///
    /// An immutable registry owning the sorted entries and their lookup index.
    ///
    /// # Errors
    ///
    /// Returns `DuplicateId` with the lexically first conflicting ID and all
    /// of its sources sorted by source fields. Repeated identical sources are
    /// retained because each registration is reported.
    fn build(mut registrations: Vec<ValueCodecRegistration<D>>) -> Result<Self, ValueCodecRegistryError> {
        registrations.sort_by_key(ValueCodecRegistration::id);
        for (index, pair) in registrations.windows(2).enumerate() {
            if pair[0].id() == pair[1].id() {
                let id = pair[0].id();
                let mut sources = registrations[index..]
                    .iter()
                    .take_while(|registration| registration.id() == id)
                    .map(ValueCodecRegistration::source)
                    .collect::<Vec<_>>();
                sources.sort_unstable();
                return Err(ValueCodecRegistryError::DuplicateId {
                    id: id.as_str(),
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

    /// Finds a registration by stable ID.
    ///
    /// # Parameters
    ///
    /// - `id`: Exact stable ID to look up, without normalization.
    ///
    /// # Returns
    ///
    /// `Some` borrows the matching registration from this registry; `None`
    /// means the ID is absent. Lookup uses a logarithmic tree search.
    #[must_use]
    #[inline]
    pub fn get(&self, id: &str) -> Option<&ValueCodecRegistration<D>> {
        let index = *self.indices.get(id)?;
        self.registrations.get(index)
    }

    /// Returns registrations in deterministic ID order.
    ///
    /// # Returns
    ///
    /// The immutable registration slice borrowed from this registry without
    /// allocation.
    #[must_use]
    #[inline]
    pub fn registrations(&self) -> &[ValueCodecRegistration<D>] {
        &self.registrations
    }
}

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
            Self::build(registrations)
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
    pub fn global() -> &'static Self {
        Self::try_global().unwrap_or_else(|error| panic!("invalid global value string codec registry: {error}"))
    }
}

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
            Self::build(registrations)
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

// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Inventory factory for string-codec registrations.

use inventory::collect;

use crate::ValueStringCodecRegistration;

/// Callback wrapper submitted by
/// [`register_value_string_codec!`](crate::register_value_string_codec) for
/// link-time collection. The global registry invokes each callback to create
/// its owned registration; callers normally use the registration macro rather
/// than constructing this hidden support type directly.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "json")]
/// # {
/// use qubit_codec::JsonStringValueCodec;
/// use qubit_codec::ValueStringCodecRegistry;
/// use qubit_codec::register_value_string_codec;
///
/// register_value_string_codec!(
///     id = "example.factory.u32",
///     codec = JsonStringValueCodec<u32>,
///     value = u32,
/// );
///
/// let registry = ValueStringCodecRegistry::global();
/// assert!(registry.get("example.factory.u32").is_some());
/// # }
/// ```
#[doc(hidden)]
pub struct ValueStringCodecRegistrationFactory(
    /// Builds the owned registration when the global registry collects this
    /// inventory entry. The callback takes no arguments; the registration
    /// macro captures its codec identifier, descriptor, and declaration site.
    pub fn() -> ValueStringCodecRegistration,
);

collect!(ValueStringCodecRegistrationFactory);

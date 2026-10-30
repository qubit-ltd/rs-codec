// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Inventory factory for byte-codec registrations.

use inventory::collect;

/// Factory submitted by
/// [`register_value_bytes_codec!`](crate::register_value_bytes_codec).
/// The inventory collector supplies this factory to the registry; callers do
/// not construct it directly.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "json")]
/// # {
/// use qubit_codec::JsonBytesValueCodec;
/// use qubit_codec::ValueBytesCodecRegistry;
/// use qubit_codec::register_value_bytes_codec;
///
/// register_value_bytes_codec!(
///     id = "example.factory.u32",
///     codec = JsonBytesValueCodec<u32>,
///     value = u32,
/// );
///
/// let registry = ValueBytesCodecRegistry::global();
/// assert!(registry.get("example.factory.u32").is_some());
/// # }
/// ```
pub use super::value_bytes_codec_registration_factory::ValueBytesCodecRegistrationFactory;

collect!(ValueBytesCodecRegistrationFactory);

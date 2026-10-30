// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Defines the factory type stored in the byte-codec registration inventory.

use crate::ValueBytesCodecRegistration;

/// Factory submitted by
/// [`register_value_bytes_codec!`](crate::register_value_bytes_codec).
/// The inventory collector supplies this factory to the registry; callers do
/// not construct it directly.
#[doc(hidden)]
pub struct ValueBytesCodecRegistrationFactory(
    /// Builds an owned byte-codec registration when inventory entries are
    /// collected into a registry.
    pub fn() -> ValueBytesCodecRegistration,
);

// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Inventory factory for byte-codec registrations.

use inventory::collect;

use crate::ValueBytesCodecRegistration;

/// Factory submitted by
/// [`register_value_bytes_codec!`](crate::register_value_bytes_codec).
#[doc(hidden)]
pub struct ValueBytesCodecRegistrationFactory(
    /// Builds an owned byte-codec registration when inventory entries are
    /// collected into a registry. The callback takes no arguments and
    /// supplies its own codec identifier, metadata, and construction logic.
    pub fn() -> ValueBytesCodecRegistration,
);

collect!(ValueBytesCodecRegistrationFactory);

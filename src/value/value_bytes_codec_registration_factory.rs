// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Inventory factory for byte-codec registrations.

use crate::ValueBytesCodecRegistration;

/// Factory submitted by
/// [`register_value_bytes_codec!`](crate::register_value_bytes_codec).
#[doc(hidden)]
pub struct ValueBytesCodecRegistrationFactory(pub fn() -> ValueBytesCodecRegistration);

inventory::collect!(ValueBytesCodecRegistrationFactory);

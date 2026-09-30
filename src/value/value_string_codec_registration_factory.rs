// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Inventory factory for string-codec registrations.

use crate::ValueStringCodecRegistration;

/// Factory submitted by
/// [`register_value_string_codec!`](crate::register_value_string_codec).
#[doc(hidden)]
pub struct ValueStringCodecRegistrationFactory(pub fn() -> ValueStringCodecRegistration);

inventory::collect!(ValueStringCodecRegistrationFactory);

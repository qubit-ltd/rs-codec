// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Inventory value-codec registration factories.

use crate::ValueBytesCodecRegistration;
use crate::ValueStringCodecRegistration;

/// Factory submitted by
/// [`register_value_string_codec!`](crate::register_value_string_codec).
#[doc(hidden)]
pub struct ValueStringCodecRegistrationFactory(pub fn() -> ValueStringCodecRegistration);

inventory::collect!(ValueStringCodecRegistrationFactory);

/// Factory submitted by
/// [`register_value_bytes_codec!`](crate::register_value_bytes_codec).
#[doc(hidden)]
pub struct ValueBytesCodecRegistrationFactory(pub fn() -> ValueBytesCodecRegistration);

inventory::collect!(ValueBytesCodecRegistrationFactory);

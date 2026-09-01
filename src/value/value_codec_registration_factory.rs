// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Inventory value-codec registration factories.

use crate::ValueCodecRegistration;

/// Factory submitted by [`register_value_codec!`](crate::register_value_codec).
#[doc(hidden)]
pub struct ValueCodecRegistrationFactory(pub fn() -> ValueCodecRegistration);

inventory::collect!(ValueCodecRegistrationFactory);

// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::ByteOrder;
use super::ByteOrderSpec;

/// Type-level marker for native-endian byte order.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NativeEndian;

impl ByteOrderSpec for NativeEndian {
    /// The native-endian byte order.
    const ORDER: ByteOrder = ByteOrder::NativeEndian;
}

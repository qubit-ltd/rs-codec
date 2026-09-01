// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Runtime byte order selector.
///
/// # Examples
///
/// ```
/// use qubit_codec::ByteOrder;
///
/// let order = ByteOrder::BigEndian;
/// assert_eq!(order, ByteOrder::BigEndian);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ByteOrder {
    /// Big-endian byte order.
    BigEndian,

    /// Little-endian byte order.
    LittleEndian,

    /// Native-endian byte order.
    NativeEndian,
}

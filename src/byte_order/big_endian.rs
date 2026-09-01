// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use super::ByteOrder;
use super::ByteOrderSpec;

/// Type-level marker for big-endian byte order.
///
/// # Examples
///
/// ```
/// use qubit_codec::{BigEndian, ByteOrder, ByteOrderSpec};
///
/// let _: BigEndian = Default::default();
/// assert_eq!(BigEndian::ORDER, ByteOrder::BigEndian);
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BigEndian;

impl ByteOrderSpec for BigEndian {
    /// The big-endian byte order.
    const ORDER: ByteOrder = ByteOrder::BigEndian;
}

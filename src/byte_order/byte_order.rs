/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

/// Runtime byte order selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ByteOrder {
    /// Big-endian byte order.
    BigEndian,

    /// Little-endian byte order.
    LittleEndian,
}

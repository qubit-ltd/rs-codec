// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::ByteOrder;

#[test]
fn test_byte_order_variants_are_distinct_and_copyable() {
    let big = ByteOrder::BigEndian;
    let little = ByteOrder::LittleEndian;

    assert_eq!(ByteOrder::BigEndian, big);
    assert_eq!(ByteOrder::LittleEndian, little);
    assert_ne!(big, little);
}

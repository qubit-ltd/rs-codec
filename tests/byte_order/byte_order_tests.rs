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
    let mut order = ByteOrder::BigEndian;
    let copied_order = order;
    order = ByteOrder::NativeEndian;

    assert_eq!(
        copied_order,
        ByteOrder::BigEndian,
        "the copied selector keeps the value it was copied from"
    );
    assert_eq!(
        order,
        ByteOrder::NativeEndian,
        "rebinding the original selector does not change the copy"
    );

    let rendered = [
        format!("{:?}", ByteOrder::BigEndian),
        format!("{:?}", ByteOrder::LittleEndian),
        format!("{:?}", ByteOrder::NativeEndian),
    ];

    assert_eq!(
        rendered[0], "BigEndian",
        "the big-endian selector renders its own variant name"
    );
    assert_eq!(
        rendered[1], "LittleEndian",
        "the little-endian selector renders its own variant name"
    );
    assert_eq!(
        rendered[2], "NativeEndian",
        "the native-endian selector renders its own variant name"
    );
    assert_ne!(
        rendered[0], rendered[1],
        "big-endian and little-endian render distinguishable values"
    );
    assert_ne!(
        rendered[0], rendered[2],
        "big-endian and native-endian render distinguishable values"
    );
    assert_ne!(
        rendered[1], rendered[2],
        "little-endian and native-endian render distinguishable values"
    );
}

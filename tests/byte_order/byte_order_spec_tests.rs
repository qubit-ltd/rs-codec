// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::BigEndian;
use qubit_codec::ByteOrder;
use qubit_codec::ByteOrderSpec;
use qubit_codec::LittleEndian;
use qubit_codec::NativeEndian;

#[test]
fn test_byte_order_spec_exposes_runtime_order() {
    assert_eq!(ByteOrder::BigEndian, BigEndian::ORDER);
    assert_eq!(ByteOrder::LittleEndian, LittleEndian::ORDER);
    assert_eq!(ByteOrder::NativeEndian, NativeEndian::ORDER);
}

// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{BigEndian, ByteOrder, ByteOrderSpec, LittleEndian};

#[test]
fn test_byte_order_spec_exposes_runtime_order() {
    assert_eq!(ByteOrder::BigEndian, BigEndian::ORDER);
    assert_eq!(ByteOrder::LittleEndian, LittleEndian::ORDER);
}

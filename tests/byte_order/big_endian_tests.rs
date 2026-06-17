// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::BigEndian;

#[test]
fn test_big_endian_is_copyable_default_marker() {
    let marker = BigEndian;

    assert_eq!(marker, BigEndian);
}

// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::LittleEndian;

#[test]
fn test_little_endian_is_copyable_default_marker() {
    let marker = LittleEndian;

    assert_eq!(marker, LittleEndian);
}

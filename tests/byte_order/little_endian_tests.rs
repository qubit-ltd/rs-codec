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
    let marker: LittleEndian = Default::default();
    let copied = marker;

    assert_eq!(marker, copied, "copying preserves the little-endian marker");
    assert_eq!(marker, LittleEndian, "the default is the little-endian marker");
}

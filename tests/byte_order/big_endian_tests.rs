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
    let marker: BigEndian = Default::default();
    let copied = marker;

    assert_eq!(marker, copied, "copying preserves the big-endian marker");
    assert_eq!(marker, BigEndian, "the default is the big-endian marker");
}

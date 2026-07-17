// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::NativeEndian;

#[test]
fn test_native_endian_is_copyable_default_marker() {
    let marker = NativeEndian;

    assert_eq!(marker, NativeEndian);
}

// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::num::NonZeroUsize;

const THREE: NonZeroUsize = qubit_codec::nz!(3);

/// Tests that the non-zero helpers work in const and runtime expressions.
#[test]
fn test_nz_supports_const_and_runtime_calls() {
    assert_eq!(THREE.get(), 3);
    assert_eq!(qubit_codec::nz(7).get(), 7);
    assert_eq!(qubit_codec::nz_const(9).get(), 9);
}

/// Tests that the non-zero macro rejects zero.
#[test]
#[should_panic(expected = "qubit_codec::nz!(): value must be non-zero")]
fn test_nz_rejects_zero() {
    let _ = qubit_codec::nz!(0);
}

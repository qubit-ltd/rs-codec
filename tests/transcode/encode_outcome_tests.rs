// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::EncodeOutcome;

#[test]
fn test_encode_outcome_constructors_match_variants() {
    assert_eq!(
        EncodeOutcome::Consumed { written: 2 },
        EncodeOutcome::consumed(2),
    );
    assert_eq!(
        EncodeOutcome::NeedOutput {
            required: qubit_io::nz!(3),
        },
        EncodeOutcome::need_output(qubit_io::nz!(3)),
    );
}

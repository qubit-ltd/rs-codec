// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::EncodeValueResult;

#[test]
fn test_encode_value_result_constructors_match_variants() {
    assert_eq!(
        EncodeValueResult::Consumed { written: 2 },
        EncodeValueResult::consumed(2),
    );
    assert_eq!(
        EncodeValueResult::NeedOutput {
            required: qubit_io::nz!(3),
        },
        EncodeValueResult::need_output(qubit_io::nz!(3)),
    );
}

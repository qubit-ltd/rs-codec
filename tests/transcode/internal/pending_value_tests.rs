// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::CodecTranscodeConverter;
use qubit_codec::TranscodeStatus;

use crate::common::IdentityCodec;

/// Pins the zero-capacity boundary: a converter offered no output room at all
/// must still consume exactly one source unit, must not report a wider
/// requirement, and must not advance the target cursor.
///
/// The two-unit input makes that bound observable: with a single-unit input
/// `read() == 1` would be vacuous, because consuming everything available
/// yields the same value.
#[test]
fn test_pending_value_zero_capacity_output_consumes_exactly_one_unit() {
    let mut converter = CodecTranscodeConverter::new(IdentityCodec, IdentityCodec);
    let mut reset_output = [];
    converter.reset(&mut reset_output, 0).expect("initialize stream");

    let progress = converter
        .transcode(&[11, 12], 0, &mut [], 0)
        .expect("zero-capacity output must not be a domain error");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            required: crate::nonzero(1),
        },
        progress.status(),
        "an empty output slice must still report the single unit actually required",
    );
    assert_eq!(
        (1, 0),
        (progress.read(), progress.written()),
        "one decoded unit is retained and nothing is written when no output room exists",
    );
}

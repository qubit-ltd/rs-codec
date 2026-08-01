// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests scratch-backed single-value decode lifecycle progress.

use qubit_codec::{CodecValueDecoder, DecodeLifecycleProgress};

use super::codec_value_decoder_tests::ResetSensitiveLifecycleCodec;

#[test]
fn test_decode_lifecycle_progress_uses_separate_phase_storage() {
    let mut decoder = CodecValueDecoder::new(ResetSensitiveLifecycleCodec::default());
    let mut reset = [0_u8; 1];
    let mut finish = [0_u8; 1];

    let progress: DecodeLifecycleProgress<u8> = decoder
        .decode_lifecycle_with_scratch(&[43], &mut reset, &mut finish)
        .expect("complete lifecycle output should use separate scratch buffers");

    assert_eq!(&42, progress.value());
    assert_eq!(1, progress.reset_written());
    assert_eq!(1, progress.finish_written());
    assert_eq!([0xfe], reset);
    assert_eq!([2], finish);
    assert_eq!((42, 1, 1), progress.into_parts());
}

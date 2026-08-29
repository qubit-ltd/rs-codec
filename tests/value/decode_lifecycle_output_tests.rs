// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests owned single-value decode lifecycle output.

use qubit_codec::CodecValueDecoder;
use qubit_codec::DecodeLifecycleOutput;

use super::codec_value_decoder_tests::ResetSensitiveLifecycleCodec;

#[test]
fn test_decode_lifecycle_output_preserves_every_phase() {
    let mut decoder = CodecValueDecoder::new(ResetSensitiveLifecycleCodec::default());

    let output: DecodeLifecycleOutput<u8> = decoder
        .decode_lifecycle(&[43])
        .expect("complete lifecycle output should be preserved");

    assert_eq!(&[0xfe], output.reset());
    assert_eq!(&42, output.value());
    assert_eq!(&[2], output.finish());
    assert_eq!((vec![0xfe], 42, vec![2]), output.into_parts());
}

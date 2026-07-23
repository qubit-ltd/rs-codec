// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{
    CodecValueDecoder,
    CodecValueEncoder,
    ValueEncoder,
};

use crate::common::IdentityCodec;

#[test]
fn test_codec_value_lifecycle_round_trips_one_complete_value() {
    let mut encoder = CodecValueEncoder::new(IdentityCodec);
    let encoded = encoder.encode(&31).expect("value should encode");

    let mut decoder = CodecValueDecoder::new(IdentityCodec);
    let decoded = decoder.decode(&encoded).expect("value should decode");

    assert_eq!(vec![31], encoded);
    assert_eq!(31, decoded);
}

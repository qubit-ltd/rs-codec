// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::{io::Cursor, io::Error};

use qubit_codec::TranscodeDecodeInput;

use crate::common::IdentityCodec;

#[test]
fn test_codec_decode_driver_consumes_one_buffered_value_per_call() {
    let mut input = TranscodeDecodeInput::new(Cursor::new(vec![21, 22]));
    let mut codec = IdentityCodec;

    let first = input
        .read_decoded_with(&mut codec, Error::other)
        .expect("first value should decode");
    let second = input
        .read_decoded_with(&mut codec, Error::other)
        .expect("second value should decode");

    assert_eq!((21, 22), (first, second));
}

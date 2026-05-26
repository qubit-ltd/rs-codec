/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for lightweight encoder and decoder traits.

use qubit_codec::{
    Decoder,
    Encoder,
};

#[derive(Default)]
struct UppercaseCodec;

impl Encoder<str> for UppercaseCodec {
    type Output = String;
    type Error = core::convert::Infallible;

    fn encode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_ascii_uppercase())
    }
}

impl Decoder<str> for UppercaseCodec {
    type Output = String;
    type Error = core::convert::Infallible;

    fn decode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_ascii_lowercase())
    }
}

#[test]
fn test_codec_types_can_be_used_through_traits() {
    let codec = UppercaseCodec;
    let encoded = Encoder::<str>::encode(&codec, "abc").expect("uppercase encoding should be infallible");
    let decoded = Decoder::<str>::decode(&codec, &encoded).expect("lowercase decoding should be infallible");

    assert_eq!("ABC", encoded);
    assert_eq!("abc", decoded);
}

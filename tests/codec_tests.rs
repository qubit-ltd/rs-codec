/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for the bidirectional codec trait.

use qubit_codec::{
    Codec,
    Decoder,
    Encoder,
};

#[derive(Default)]
struct EchoCodec;

impl Encoder<str> for EchoCodec {
    type Output = String;
    type Error = core::convert::Infallible;

    fn encode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

impl Decoder<str> for EchoCodec {
    type Output = String;
    type Error = core::convert::Infallible;

    fn decode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

fn roundtrip<C>(codec: &C, text: &str) -> String
where
    C: Codec<str, str>
        + Encoder<str, Output = String, Error = core::convert::Infallible>
        + Decoder<str, Output = String, Error = core::convert::Infallible>,
{
    let encoded = Encoder::<str>::encode(codec, text).expect("echo encoding should be infallible");
    Decoder::<str>::decode(codec, &encoded).expect("echo decoding should be infallible")
}

#[test]
fn test_codec_trait_combines_encoder_and_decoder() {
    assert_eq!("codec", roundtrip(&EchoCodec, "codec"));
}

/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for the encoder trait contract.

use qubit_codec::ValueEncoder;

#[derive(Default)]
struct StringEncoder;

impl ValueEncoder<str> for StringEncoder {
    type Output = String;
    type Error = core::convert::Infallible;

    fn encode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

#[test]
fn test_encoder_trait_dispatches_to_implementor() {
    let encoded = ValueEncoder::<str>::encode(&StringEncoder, "text").expect("encoding should be infallible");

    assert_eq!("text", encoded);
}

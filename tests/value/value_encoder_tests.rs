// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the encoder trait contract.

use qubit_codec::ValueEncoder;

#[derive(Default)]
struct StringEncoder;

impl ValueEncoder<str> for StringEncoder {
    type Output = String;
    type Error = core::convert::Infallible;
    type DomainError = core::convert::Infallible;

    fn map_error(&self, error: Self::DomainError) -> Self::Error {
        match error {}
    }

    fn encode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

#[test]
fn test_encoder_trait_dispatches_to_implementor() {
    let encoded = ValueEncoder::<str>::encode(&mut StringEncoder, "text")
        .expect("encoding should be infallible");

    assert_eq!("text", encoded);
}

#[derive(Default)]
struct UppercaseCodec;

impl ValueEncoder<str> for UppercaseCodec {
    type Output = String;
    type Error = core::convert::Infallible;
    type DomainError = core::convert::Infallible;

    fn map_error(&self, error: Self::DomainError) -> Self::Error {
        match error {}
    }

    fn encode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_ascii_uppercase())
    }
}

#[test]
fn test_codec_types_can_be_used_through_encoder_trait() {
    let encoded = ValueEncoder::<str>::encode(&mut UppercaseCodec, "abc")
        .expect("uppercase encoding should be infallible");

    assert_eq!("ABC", encoded);
}

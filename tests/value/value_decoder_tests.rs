// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the decoder trait contract.

use qubit_codec::ValueDecoder;

#[derive(Default)]
struct StringDecoder;

impl ValueDecoder<str> for StringDecoder {
    type Output = String;
    type Error = core::convert::Infallible;

    fn decode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

#[test]
fn test_decoder_trait_dispatches_to_implementor() {
    let decoded =
        ValueDecoder::<str>::decode(&StringDecoder, "text").expect("decoding should be infallible");

    assert_eq!("text", decoded);
}

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

    fn encode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

#[test]
fn test_encoder_trait_dispatches_to_implementor() {
    let encoded = ValueEncoder::<str>::encode(&mut StringEncoder, "text").expect("encoding should be infallible");

    assert_eq!("text", encoded);
}

#[derive(Default)]
struct UppercaseEncoder;

impl ValueEncoder<str> for UppercaseEncoder {
    type Output = String;
    type Error = core::convert::Infallible;

    fn encode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_ascii_uppercase())
    }
}

#[test]
fn test_encoder_trait_dispatches_to_each_implementor() {
    let encoded =
        ValueEncoder::<str>::encode(&mut UppercaseEncoder, "abc").expect("uppercase encoding should be infallible");

    assert_eq!("ABC", encoded, "each implementor supplies its own output");
}

#[test]
fn test_encoder_output_owns_its_data() {
    let owned = String::from("borrowed then dropped");
    let encoded = {
        let borrowed: &str = owned.as_str();
        ValueEncoder::<str>::encode(&mut StringEncoder, borrowed).expect("encoding should be infallible")
    };

    drop(owned);

    assert_eq!(
        "borrowed then dropped", encoded,
        "Output owns its data and outlives the input borrow"
    );
}

#[derive(Default)]
struct RejectingEncoder;

#[derive(Debug, Eq, PartialEq)]
struct UnrepresentableInput;

impl ValueEncoder<str> for RejectingEncoder {
    type Output = String;
    type Error = UnrepresentableInput;

    fn encode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        if input.contains('\u{0}') {
            return Err(UnrepresentableInput);
        }
        Ok(input.to_owned())
    }
}

#[test]
fn test_encoder_reports_error_for_unrepresentable_input() {
    let rejected = ValueEncoder::<str>::encode(&mut RejectingEncoder, "bad\0value")
        .expect_err("an embedded NUL cannot be represented");

    assert_eq!(
        UnrepresentableInput, rejected,
        "the implementor's own error type reaches the caller"
    );
    assert!(
        format!("{rejected:?}").contains("UnrepresentableInput"),
        "the error type is observable through Debug"
    );
}

#[test]
fn test_encoder_accepts_representable_input_from_fallible_implementor() {
    let accepted =
        ValueEncoder::<str>::encode(&mut RejectingEncoder, "fine").expect("an encodable value must still succeed");

    assert_eq!("fine", accepted, "the Ok path of a fallible encoder still works");
}

/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use qubit_codec::prelude::{
    BigEndian,
    ByteOrder,
    ByteOrderSpec,
    Decoder,
    Encoder,
    TranscodeProgress,
    TranscodeStatus,
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

#[test]
fn test_prelude_imports_core_codec_traits_and_markers() {
    assert_eq!(ByteOrder::BigEndian, BigEndian::ORDER);

    let codec = EchoCodec;

    let encoded = Encoder::<str>::encode(&codec, "abc").expect("echo encode should be infallible");
    let decoded = Decoder::<str>::decode(&codec, &encoded).expect("echo decode should be infallible");
    assert_eq!("abc", decoded);

    let progress = TranscodeProgress::complete(1, 2);
    assert_eq!(TranscodeStatus::Complete, progress.status());
}

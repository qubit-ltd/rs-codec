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
    BufferedConverter,
    BufferedDecoder,
    BufferedEncoder,
    ByteOrder,
    ByteOrderSpec,
    Codec,
    CodecBufferedEncoder,
    CodecValueEncoder,
    DecodeErrorInfo,
    DecodeFailure,
    TranscodeProgress,
    TranscodeStatus,
    ValueDecoder,
    ValueEncoder,
};

#[derive(Default)]
struct EchoCodec;

impl ValueEncoder<str> for EchoCodec {
    type Output = String;
    type Error = core::convert::Infallible;

    fn encode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

impl ValueDecoder<str> for EchoCodec {
    type Output = String;
    type Error = core::convert::Infallible;

    fn decode(&self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

unsafe impl Codec<u8, u8> for EchoCodec {
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> usize {
        1
    }

    fn max_units_per_value(&self) -> usize {
        1
    }

    unsafe fn decode_unchecked(&self, input: &[u8], index: usize) -> Result<(u8, usize), Self::DecodeError> {
        debug_assert!(index < input.len());

        // SAFETY: The caller guarantees that `index` is readable.
        let value = unsafe { *input.as_ptr().add(index) };
        Ok((value, 1))
    }

    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        debug_assert!(index < output.len());

        // SAFETY: The caller guarantees that `index` is writable.
        unsafe {
            *output.as_mut_ptr().add(index) = *value;
        }
        Ok(1)
    }
}

#[test]
fn test_prelude_imports_core_codec_traits_and_markers() {
    fn _accept_buffered_encoder<T: BufferedEncoder<char, u8>>() {}
    fn _accept_buffered_decoder<T: BufferedDecoder<u8, char>>() {}
    fn _accept_buffered_converter<T: BufferedConverter<u8, u16>>() {}
    fn _accept_codec_value_encoder<T: ValueEncoder<u8, Output = Vec<u8>>>() {}
    fn _accept_codec_buffered_encoder<T: BufferedEncoder<u8, u8>>() {}

    assert_eq!(ByteOrder::BigEndian, BigEndian::ORDER);
    _accept_codec_value_encoder::<CodecValueEncoder<EchoCodec, u8, u8>>();
    _accept_codec_buffered_encoder::<CodecBufferedEncoder<EchoCodec>>();

    let codec = EchoCodec;

    let encoded = ValueEncoder::<str>::encode(&codec, "abc").expect("echo encode should be infallible");
    let decoded = ValueDecoder::<str>::decode(&codec, &encoded).expect("echo decode should be infallible");
    assert_eq!("abc", decoded);

    let progress = TranscodeProgress::complete(1, 2);
    assert_eq!(TranscodeStatus::Complete, progress.status());

    let failure = DecodeFailure::Invalid { consumed: 1 };
    assert_eq!(Some(1), failure.invalid_consumed());

    fn _accept_decode_error_info<T: DecodeErrorInfo>() {}
    _accept_decode_error_info::<core::convert::Infallible>();

    let (decoded, consumed) = unsafe { codec.decode_unchecked(&[1], 0) }.expect("decode should be infallible");
    assert_eq!((1, 1), (decoded, consumed));
}

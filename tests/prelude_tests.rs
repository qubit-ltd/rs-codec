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
    BufferedConvertHooks,
    BufferedConverter,
    BufferedDecoder,
    BufferedEncoder,
    ByteOrder,
    ByteOrderSpec,
    Codec,
    CodecBufferedConverter,
    CodecBufferedDecoder,
    CodecBufferedEncoder,
    CodecConvertError,
    CodecDecodeError,
    CodecEncodeError,
    CodecValueDecoder,
    CodecValueEncoder,
    ConvertErrorFactory,
    EncodePlan,
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

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    unsafe fn decode_unchecked(
        &self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        // SAFETY: The caller guarantees that `index` is readable.
        let value = unsafe { *input.as_ptr().add(index) };
        Ok((value, core::num::NonZeroUsize::MIN))
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
    fn _accept_codec_value_decoder<T: ValueDecoder<[u8], Output = u8>>() {}
    fn _accept_codec_buffered_encoder<T: BufferedEncoder<u8, u8>>() {}
    fn _accept_codec_buffered_decoder<T: BufferedDecoder<u8, u8>>() {}
    fn _accept_codec_buffered_converter<T: BufferedConverter<u8, u8>>() {}
    fn _accept_buffered_decode_engine<T>() {}
    fn _accept_buffered_encode_engine<T>() {}
    fn _accept_buffered_convert_engine<T>() {}
    fn _accept_buffered_decode_hooks<T: qubit_codec::BufferedDecodeHooks<EchoCodec, u8, u8>>() {}
    fn _accept_buffered_encode_hooks<T: qubit_codec::BufferedEncodeHooks<EchoCodec, u8, u8>>() {}
    fn _accept_buffered_convert_hooks<T: BufferedConvertHooks<EchoCodec, EchoCodec, u8, u8>>() {}

    assert_eq!(ByteOrder::BigEndian, BigEndian::ORDER);
    _accept_codec_value_encoder::<CodecValueEncoder<EchoCodec, u8, u8>>();
    _accept_codec_value_decoder::<CodecValueDecoder<EchoCodec, u8, u8>>();
    _accept_codec_buffered_encoder::<CodecBufferedEncoder<EchoCodec>>();
    _accept_codec_buffered_decoder::<CodecBufferedDecoder<EchoCodec, u8>>();
    _accept_codec_buffered_converter::<CodecBufferedConverter<EchoCodec, EchoCodec, u8, u8>>();
    _accept_buffered_decode_engine::<qubit_codec::BufferedDecodeEngine<EchoCodec, (), u8>>();
    _accept_buffered_encode_engine::<qubit_codec::BufferedEncodeEngine<EchoCodec, ()>>();
    let codec = EchoCodec;

    let encoded = ValueEncoder::<str>::encode(&codec, "abc").expect("echo encode should be infallible");
    let decoded = ValueDecoder::<str>::decode(&codec, &encoded).expect("echo decode should be infallible");
    assert_eq!("abc", decoded);

    let progress = TranscodeProgress::complete(1, 2);
    assert_eq!(TranscodeStatus::Complete, progress.status());

    let decode_error = CodecDecodeError::<core::convert::Infallible>::incomplete(0, 2, 1);
    assert!(matches!(
        decode_error,
        CodecDecodeError::Incomplete {
            input_index: 0,
            required_total: 2,
            available: 1,
        }
    ));

    let convert_error = CodecConvertError::<core::convert::Infallible, core::convert::Infallible>::decode(decode_error);
    assert!(matches!(convert_error, CodecConvertError::Decode { .. }));

    let encode_error = CodecEncodeError::<core::convert::Infallible>::invalid_input_index(2, 1);
    assert!(matches!(encode_error, CodecEncodeError::InvalidInputIndex { .. }));

    let convert_factory_error = <CodecConvertError<
        core::convert::Infallible,
        core::convert::Infallible,
    > as ConvertErrorFactory<EchoCodec>>::invalid_input_index(&codec, 2, 1);
    assert!(matches!(
        convert_factory_error,
        CodecConvertError::Decode {
            source: CodecDecodeError::InvalidInputIndex { .. }
        }
    ));

    let encode_plan = EncodePlan::new(3, "payload");
    assert_eq!(3, encode_plan.max_output_units);
    assert_eq!("payload", encode_plan.payload);

    let (decoded, consumed) = unsafe { codec.decode_unchecked(&[1], 0) }.expect("decode should be infallible");
    assert_eq!((1, 1), (decoded, consumed.get()));
}

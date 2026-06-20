// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{
    BigEndian, ByteOrder, ByteOrderSpec, Codec, CodecConvertError, CodecDecodeError,
    CodecDecodeErrorSignal, CodecEncodeError, CodecTranscodeConverter, CodecTranscodeDecoder,
    CodecTranscodeEncoder, CodecValueDecoder, CodecValueEncoder, CodecValueExt, EncodeContext,
    EncodePlan, TranscodeConvertHooks, TranscodeConverter, TranscodeDecoder, TranscodeEncoder,
    TranscodeProgress, TranscodeStatus, ValueDecoder, ValueEncoder,
};

#[derive(Default)]
struct EchoCodec;

impl ValueEncoder<str> for EchoCodec {
    type Output = String;
    type Error = core::convert::Infallible;

    fn encode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

impl ValueDecoder<str> for EchoCodec {
    type Output = String;
    type Error = core::convert::Infallible;

    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

unsafe impl Codec for EchoCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        // SAFETY: The caller guarantees that `index` is readable.
        let value = unsafe { *input.as_ptr().add(index) };
        Ok((value, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(index < output.len());

        // SAFETY: The caller guarantees that `index` is writable.
        unsafe {
            *output.as_mut_ptr().add(index) = *value;
        }
        Ok(qubit_io::nz!(1))
    }
}

#[test]
fn test_prelude_imports_core_codec_traits_and_markers() {
    fn _accept_transcode_encoder<T: TranscodeEncoder<char, u8>>() {}
    fn _accept_transcode_decoder<T: TranscodeDecoder<u8, char>>() {}
    fn _accept_transcode_converter<T: TranscodeConverter<u8, u16>>() {}
    fn _accept_codec_value_encoder<T: ValueEncoder<u8, Output = Vec<u8>>>() {}
    fn _accept_codec_value_decoder<T: ValueDecoder<[u8], Output = u8>>() {}
    fn _accept_codec_value_ext<T: CodecValueExt>() {}
    fn _accept_codec_transcode_encoder<T: TranscodeEncoder<u8, u8>>() {}
    fn _accept_codec_transcode_decoder<T: TranscodeDecoder<u8, u8>>() {}
    fn _accept_codec_transcode_converter<T: TranscodeConverter<u8, u8>>() {}
    fn _accept_codec_decode_error_signal<T: CodecDecodeErrorSignal>() {}
    fn _accept_transcode_decode_engine<T>() {}
    fn _accept_transcode_encode_engine<T>() {}
    fn _accept_transcode_convert_engine<T>() {}
    fn _accept_transcode_decode_hooks<T: qubit_codec::TranscodeDecodeHooks<EchoCodec>>() {}
    fn _accept_transcode_encode_hooks<T: qubit_codec::TranscodeEncodeHooks<EchoCodec>>() {}
    fn _accept_transcode_convert_hooks<T: TranscodeConvertHooks<EchoCodec, EchoCodec>>() {}

    assert_eq!(ByteOrder::BigEndian, BigEndian::ORDER);
    _accept_codec_value_encoder::<CodecValueEncoder<EchoCodec>>();
    _accept_codec_value_decoder::<CodecValueDecoder<EchoCodec>>();
    _accept_codec_value_ext::<EchoCodec>();
    _accept_codec_transcode_encoder::<CodecTranscodeEncoder<EchoCodec>>();
    _accept_codec_transcode_decoder::<CodecTranscodeDecoder<EchoCodec>>();
    _accept_codec_transcode_converter::<CodecTranscodeConverter<EchoCodec, EchoCodec>>();
    _accept_codec_decode_error_signal::<core::convert::Infallible>();
    _accept_transcode_decode_engine::<qubit_codec::TranscodeDecodeEngine<EchoCodec, ()>>();
    _accept_transcode_encode_engine::<qubit_codec::TranscodeEncodeEngine<EchoCodec, ()>>();
    let mut codec = EchoCodec;

    let encoded =
        ValueEncoder::<str>::encode(&mut codec, "abc").expect("echo encode should be infallible");
    let decoded = ValueDecoder::<str>::decode(&mut codec, &encoded)
        .expect("echo decode should be infallible");
    assert_eq!("abc", decoded);

    let progress = TranscodeProgress::complete(1, 2);
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(
        qubit_codec::TranscodeError::<
            CodecConvertError<core::convert::Infallible, core::convert::Infallible>,
        >::InvalidOutputIndex {
            index: 1,
            len: 0
        },
        qubit_codec::TranscodeError::invalid_output_index(1, 0),
    );

    let decode_error = CodecDecodeError::<core::convert::Infallible>::incomplete(0, 2, 1);
    assert!(matches!(
        decode_error,
        CodecDecodeError::Incomplete {
            input_index: 0,
            required_total: 2,
            available: 1,
        }
    ));

    let convert_error =
        CodecConvertError::<core::convert::Infallible, core::convert::Infallible>::decode(
            decode_error,
        );
    assert!(matches!(convert_error, CodecConvertError::Decode { .. }));

    let encode_error = CodecEncodeError::<core::convert::Infallible>::invalid_output_index(2, 1);
    assert!(matches!(
        encode_error,
        CodecEncodeError::InvalidOutputIndex { .. }
    ));
    let convert_error =
        CodecConvertError::<core::convert::Infallible, core::convert::Infallible>::encode(
            encode_error,
        );
    assert!(matches!(
        convert_error,
        CodecConvertError::Encode {
            source: CodecEncodeError::InvalidOutputIndex { .. },
        },
    ));

    let mut output = [0_u8; 1];
    let context = EncodeContext {
        input_value: &1_u8,
        input_index: 0,
        output: &mut output,
        output_index: 0,
    };
    assert_eq!(0, context.input_index);
    assert_eq!(1, context.available_output());

    let encode_plan = EncodePlan::new(3, "action");
    assert_eq!(3, encode_plan.max_output_units);
    assert_eq!("action", encode_plan.action);

    let (decoded, consumed) =
        unsafe { Codec::decode(&mut codec, &[1], 0) }.expect("decode should be infallible");
    assert_eq!((1, 1), (decoded, consumed.get()));
}

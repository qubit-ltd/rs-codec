// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use core::convert::Infallible;
use core::num::NonZeroUsize;

use qubit_codec as codec;
use qubit_codec::BigEndian;
use qubit_codec::ByteOrder;
use qubit_codec::ByteOrderSpec;
use qubit_codec::Codec;
use qubit_codec::CodecTranscodeConverter;
use qubit_codec::CodecTranscodeDecoder;
use qubit_codec::CodecTranscodeEncoder;
use qubit_codec::CodecValueDecoder;
use qubit_codec::CodecValueEncoder;
use qubit_codec::NativeEndian;
use qubit_codec::TranscodeConvertError;
use qubit_codec::TranscodeConvertErrorOf;
use qubit_codec::TranscodeConverter;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeDecodeErrorOf;
use qubit_codec::TranscodeDecoder;
use qubit_codec::TranscodeEncodeError;
use qubit_codec::TranscodeEncodeErrorOf;
use qubit_codec::TranscodeEncoder;
use qubit_codec::TranscodeFailure;
use qubit_codec::TranscodeProgress;
use qubit_codec::TranscodeStatus;
use qubit_codec::ValueDecoder;
use qubit_codec::ValueEncoder;
use qubit_codec::engine::DecodeContext;
use qubit_codec::engine::DecodeInvalidAction;
use qubit_codec::engine::DecodeInvalidActionOf;
use qubit_codec::engine::EncodeContext;
use qubit_codec::engine::EncodeUnencodableAction;
use qubit_codec::engine::EncodeUnencodableActionOf;
use qubit_codec::engine::TranscodeConvertEngine;
use qubit_codec::engine::TranscodeDecodeEngine;
use qubit_codec::engine::TranscodeDecodeHooks;
use qubit_codec::engine::TranscodeEncodeEngine;
use qubit_codec::engine::TranscodeEncodeHooks;

use crate::nonzero;

#[derive(Default)]
struct EchoCodec;

impl ValueEncoder<str> for EchoCodec {
    type Output = String;
    type Error = Infallible;

    fn encode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

impl ValueDecoder<str> for EchoCodec {
    type Output = String;
    type Error = Infallible;

    fn decode(&mut self, input: &str) -> Result<Self::Output, Self::Error> {
        Ok(input.to_owned())
    }
}

impl Codec for EchoCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index < input.len());

        // SAFETY: The caller guarantees that `input_index` is readable.
        let value = unsafe { *input.as_ptr().add(input_index) };
        Ok((value, NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        debug_assert!(output_index < output.len());

        // SAFETY: The caller guarantees that `output_index` is writable.
        unsafe {
            *output.as_mut_ptr().add(output_index) = *value;
        }
        Ok(1)
    }
}

struct EchoDecodeHooks;

impl TranscodeDecodeHooks<EchoCodec> for EchoDecodeHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut EchoCodec,
        error: &Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeError<Infallible>> {
        match *error {}
    }
}

struct EchoEncodeHooks;

impl TranscodeEncodeHooks<EchoCodec> for EchoEncodeHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut EchoCodec,
        _context: &EncodeContext<'_, u8>,
    ) -> Result<EncodeUnencodableAction<u8>, codec::TranscodeEncodeError<Infallible, u8>> {
        Ok(EncodeUnencodableAction::Reject)
    }
}

#[test]
fn test_prelude_imports_core_codec_traits_and_markers() {
    fn _accept_transcode_encoder<T: TranscodeEncoder<Input = char, Output = u8>>() {}
    fn _accept_transcode_decoder<T: TranscodeDecoder<Input = u8, Output = char>>() {}
    fn _accept_transcode_converter<T: TranscodeConverter<Input = u8, Output = u16>>() {}
    fn _accept_codec_value_encoder<T: ValueEncoder<u8, Output = Vec<u8>>>() {}
    fn _accept_codec_value_decoder<T: ValueDecoder<[u8], Output = u8>>() {}
    fn _accept_codec_transcode_encoder<T: TranscodeEncoder<Input = u8, Output = u8>>() {}
    fn _accept_codec_transcode_decoder<T: TranscodeDecoder<Input = u8, Output = u8>>() {}
    fn _accept_codec_transcode_converter<T: TranscodeConverter<Input = u8, Output = u8>>() {}
    fn _accept_transcode_decode_engine<T>() {}
    fn _accept_transcode_encode_engine<T>() {}
    fn _accept_transcode_convert_engine<T>() {}
    fn _accept_transcode_decode_hooks<T: TranscodeDecodeHooks<EchoCodec>>() {}
    fn _accept_transcode_encode_hooks<T: TranscodeEncodeHooks<EchoCodec>>() {}
    assert_eq!(
        ByteOrder::BigEndian,
        BigEndian::ORDER,
        "BigEndian exposes the big-endian byte order"
    );
    assert_eq!(
        ByteOrder::NativeEndian,
        NativeEndian::ORDER,
        "NativeEndian exposes the native byte order"
    );
    _accept_codec_value_encoder::<CodecValueEncoder<EchoCodec>>();
    _accept_codec_value_decoder::<CodecValueDecoder<EchoCodec>>();
    _accept_codec_transcode_encoder::<CodecTranscodeEncoder<EchoCodec>>();
    _accept_codec_transcode_decoder::<CodecTranscodeDecoder<EchoCodec>>();
    _accept_codec_transcode_converter::<CodecTranscodeConverter<EchoCodec, EchoCodec>>();
    _accept_transcode_decode_engine::<TranscodeDecodeEngine<EchoCodec, ()>>();
    _accept_transcode_encode_engine::<TranscodeEncodeEngine<EchoCodec, ()>>();
    _accept_transcode_convert_engine::<TranscodeConvertEngine<EchoCodec, EchoCodec, EchoDecodeHooks, EchoEncodeHooks>>(
    );
    let mut codec = EchoCodec;

    let encoded = ValueEncoder::<str>::encode(&mut codec, "abc").expect("echo encode should be infallible");
    let decoded = ValueDecoder::<str>::decode(&mut codec, &encoded).expect("echo decode should be infallible");
    assert_eq!("abc", decoded, "the value encoder and decoder round-trip text");

    let progress = TranscodeProgress::complete(1, 2);
    assert_eq!(
        TranscodeStatus::Complete,
        progress.status(),
        "a completed progress value reports the complete status"
    );
    let _: DecodeInvalidActionOf<EchoCodec> = DecodeInvalidAction::Emit {
        value: 1,
        consumed: nonzero(1),
    };
    let _: EncodeUnencodableActionOf<EchoCodec> = EncodeUnencodableAction::Replace { value: 1 };
    let _: TranscodeDecodeErrorOf<EchoCodec> =
        TranscodeDecodeError::Failure(codec::TranscodeFailure::incomplete_input(0, 1, 0));
    let _: TranscodeEncodeErrorOf<EchoCodec> = TranscodeEncodeError::unencodable_without_context(0);
    let _: TranscodeConvertErrorOf<EchoCodec, EchoCodec> =
        TranscodeConvertError::Failure(codec::TranscodeFailure::invalid_input_index(1, 0));
    assert_eq!(
        TranscodeConvertError::<Infallible, Infallible, u8>::Failure(codec::TranscodeFailure::invalid_output_index(
            1, 0
        )),
        TranscodeConvertError::Failure(codec::TranscodeFailure::invalid_output_index(1, 0)),
        "the inferred conversion error preserves its failure payload"
    );

    let decode_error = TranscodeDecodeError::<Infallible>::Failure(codec::TranscodeFailure::incomplete_input(0, 2, 1));
    assert!(
        matches!(
            decode_error,
            TranscodeDecodeError::Failure(TranscodeFailure::IncompleteInput {
                input_index: 0,
                required: 2,
                available: 1,
            })
        ),
        "the decode error retains incomplete-input details"
    );

    type ConvertPreludeError = TranscodeConvertError<&'static str, &'static str, u8>;

    let convert_error = ConvertPreludeError::decode_domain_main("decode failed", 0);
    assert!(
        matches!(convert_error, TranscodeConvertError::DecodeDomain(_)),
        "decode-domain errors retain their domain variant"
    );

    let encode_error = TranscodeEncodeError::<Infallible, u8>::unencodable_without_context(2);
    assert_eq!(
        TranscodeEncodeError::Unencodable {
            input_index: 2,
            value: None,
        },
        encode_error,
        "an unencodable error without context omits the value"
    );
    let convert_error = ConvertPreludeError::encode_domain_main("encode failed", 0);
    assert!(
        matches!(convert_error, TranscodeConvertError::EncodeDomain(_)),
        "encode-domain errors retain their domain variant"
    );

    let context = EncodeContext::new(&1_u8, 0, 0, 1);
    assert_eq!(0, context.input_index(), "the encode context records its input index");
    assert_eq!(
        1,
        context.available_output(),
        "the encode context records available output capacity"
    );

    let (decoded, consumed) = unsafe { Codec::decode(&mut codec, &[1], 0) }.expect("decode should be infallible");
    assert_eq!(
        (1, 1),
        (decoded, consumed.get()),
        "the codec decodes the value and reports the consumed unit count"
    );
}

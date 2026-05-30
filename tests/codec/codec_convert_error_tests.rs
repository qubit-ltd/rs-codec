use qubit_codec::{
    CodecConvertError,
    CodecDecodeError,
};

#[test]
fn test_codec_convert_error_wraps_decode_error_explicitly() {
    let decode = CodecDecodeError::<&'static str>::invalid_input_index(4, 1);
    let error = CodecConvertError::<&'static str, &'static str>::decode(decode);

    assert!(matches!(
        error,
        CodecConvertError::Decode {
            source: CodecDecodeError::InvalidInputIndex { index: 4, len: 1 },
        },
    ));
}

#[test]
fn test_codec_convert_error_wraps_encode_error_explicitly() {
    let error = CodecConvertError::<&'static str, &'static str>::encode("encode failed");

    assert_eq!(
        CodecConvertError::Encode {
            source: "encode failed",
        },
        error,
    );
}

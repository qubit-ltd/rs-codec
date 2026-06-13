use qubit_codec::{
    CodecConvertError,
    CodecDecodeError,
    CodecEncodeError,
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
    let encode = CodecEncodeError::encode("encode failed", 7);
    let error = CodecConvertError::<&'static str, &'static str>::encode(encode);

    assert_eq!(
        CodecConvertError::Encode {
            source: CodecEncodeError::Encode {
                source: "encode failed",
                input_index: 7,
            },
        },
        error,
    );
}

#[test]
fn test_codec_convert_error_wraps_invalid_output_index() {
    let encode = CodecEncodeError::<&'static str>::invalid_output_index(5, 2);
    let error = CodecConvertError::<&'static str, &'static str>::encode(encode);

    assert_eq!(
        CodecConvertError::Encode {
            source: CodecEncodeError::InvalidOutputIndex { index: 5, len: 2 },
        },
        error,
    );
}

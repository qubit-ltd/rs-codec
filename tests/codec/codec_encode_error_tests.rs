use qubit_codec::CodecEncodeError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TestEncodeError;

#[test]
fn test_codec_encode_error_wraps_codec_error() {
    let error = CodecEncodeError::encode(TestEncodeError, 7);

    assert_eq!(
        CodecEncodeError::Encode {
            source: TestEncodeError,
            input_index: 7,
        },
        error,
    );
}

#[test]
fn test_codec_encode_error_reports_invalid_input_index() {
    let error = CodecEncodeError::<TestEncodeError>::invalid_input_index(5, 2);

    assert_eq!(CodecEncodeError::InvalidInputIndex { index: 5, len: 2 }, error);
}

#[test]
fn test_codec_encode_error_reports_invalid_output_index() {
    let error = CodecEncodeError::<TestEncodeError>::invalid_output_index(5, 2);

    assert_eq!(CodecEncodeError::InvalidOutputIndex { index: 5, len: 2 }, error);
}

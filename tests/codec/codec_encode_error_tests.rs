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

    assert_eq!(
        CodecEncodeError::InvalidInputIndex { index: 5, len: 2 },
        error
    );
}

#[test]
fn test_codec_encode_error_reports_invalid_output_index() {
    let error = CodecEncodeError::<TestEncodeError>::invalid_output_index(5, 2);

    assert_eq!(
        CodecEncodeError::InvalidOutputIndex { index: 5, len: 2 },
        error
    );
}

#[test]
fn test_codec_encode_error_reports_insufficient_output() {
    let error =
        CodecEncodeError::<TestEncodeError>::insufficient_output(2, 4, 1);

    assert_eq!(
        CodecEncodeError::InsufficientOutput {
            output_index: 2,
            required: 4,
            available: 1,
        },
        error,
    );
    assert!(
        CodecEncodeError::<&'static str>::insufficient_output(2, 4, 1)
            .to_string()
            .contains("insufficient finish output")
    );
}

#[test]
fn test_codec_encode_error_display_formats_framework_variants() {
    assert!(
        CodecEncodeError::<&'static str>::invalid_input_index(1, 0)
            .to_string()
            .contains("invalid input index")
    );
    assert!(
        CodecEncodeError::<&'static str>::invalid_output_index(1, 0)
            .to_string()
            .contains("invalid output index")
    );
    assert!(
        CodecEncodeError::encode("codec failure", 2)
            .to_string()
            .contains("codec encode error")
    );
}

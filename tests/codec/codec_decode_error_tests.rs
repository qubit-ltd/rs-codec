use qubit_codec::CodecDecodeError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TestDecodeError {
    Invalid { consumed: usize },
}

#[test]
fn test_codec_decode_error_wraps_codec_error() {
    let error = CodecDecodeError::decode(TestDecodeError::Invalid { consumed: 2 }, 7);

    assert_eq!(
        CodecDecodeError::Decode {
            source: TestDecodeError::Invalid { consumed: 2 },
            input_index: 7,
        },
        error,
    );
}

#[test]
fn test_codec_decode_error_reports_adapter_incomplete_input() {
    let error = CodecDecodeError::<TestDecodeError>::incomplete(3, 4, 2);

    assert_eq!(
        CodecDecodeError::Incomplete {
            input_index: 3,
            required_total: 4,
            available: 2,
        },
        error,
    );
}

#[test]
fn test_codec_decode_error_reports_trailing_input() {
    let error = CodecDecodeError::<TestDecodeError>::trailing_input(1, 3);

    assert_eq!(
        CodecDecodeError::TrailingInput {
            consumed: 1,
            remaining: 3,
        },
        error,
    );
}

#[test]
fn test_codec_decode_error_reports_invalid_input_index() {
    let error = CodecDecodeError::<TestDecodeError>::invalid_input_index(5, 2);

    assert_eq!(CodecDecodeError::InvalidInputIndex { index: 5, len: 2 }, error);
}

#[test]
fn test_codec_decode_error_reports_invalid_output_index() {
    let error = CodecDecodeError::<TestDecodeError>::invalid_output_index(5, 2);

    assert_eq!(CodecDecodeError::InvalidOutputIndex { index: 5, len: 2 }, error);
}

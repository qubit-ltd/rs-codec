use qubit_codec::{
    CodecDecodeError,
    DecodeErrorInfo,
    DecodeFailure,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TestDecodeError {
    Invalid { consumed: usize },
}

impl DecodeErrorInfo for TestDecodeError {
    fn failure(&self) -> DecodeFailure {
        match self {
            Self::Invalid { consumed } => DecodeFailure::Invalid { consumed: *consumed },
        }
    }
}

#[test]
fn test_codec_decode_error_delegates_wrapped_decode_failure() {
    let error = CodecDecodeError::decode(TestDecodeError::Invalid { consumed: 2 }, 7);

    assert_eq!(DecodeFailure::Invalid { consumed: 2 }, error.failure());
}

#[test]
fn test_codec_decode_error_reports_adapter_incomplete_failure() {
    let error = CodecDecodeError::<TestDecodeError>::incomplete(3, 4, 2);

    assert_eq!(
        DecodeFailure::Incomplete {
            required_total: 4,
            available: 2,
        },
        error.failure(),
    );
}

#[test]
fn test_codec_decode_error_reports_trailing_input_as_invalid_failure() {
    let error = CodecDecodeError::<TestDecodeError>::trailing_input(1, 3);

    assert_eq!(DecodeFailure::Invalid { consumed: 4 }, error.failure());
}

#[test]
fn test_codec_decode_error_reports_invalid_input_index_without_consumption() {
    let error = CodecDecodeError::<TestDecodeError>::invalid_input_index(5, 2);

    assert_eq!(DecodeFailure::Invalid { consumed: 0 }, error.failure());
}

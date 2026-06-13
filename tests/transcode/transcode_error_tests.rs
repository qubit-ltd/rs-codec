use core::error::Error;

use qubit_codec::TranscodeError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("domain failure")]
struct DomainError;

#[test]
fn test_transcode_error_domain_helpers() {
    let domain = TranscodeError::domain("failure");
    assert!(domain.is_domain());
    assert_eq!(Some(&"failure"), domain.domain_ref());

    let framework = TranscodeError::<&'static str>::invalid_input_index(1, 0);
    assert!(!framework.is_domain());
    assert_eq!(None, framework.domain_ref());
}

#[test]
fn test_transcode_error_from_wraps_domain() {
    let error: TranscodeError<&'static str> = "wrapped".into();
    assert_eq!(TranscodeError::Domain("wrapped"), error);
}

#[test]
fn test_transcode_error_map_domain_preserves_framework_errors() {
    let mapped = TranscodeError::invalid_input_index(3, 1)
        .map_domain(|error: &'static str| format!("mapped {error}"));
    assert_eq!(
        TranscodeError::InvalidInputIndex { index: 3, len: 1 },
        mapped
    );

    let mapped = TranscodeError::invalid_output_index(4, 2)
        .map_domain(|error: &'static str| format!("mapped {error}"));
    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 4, len: 2 },
        mapped
    );

    let mapped = TranscodeError::insufficient_output(1, 3, 2)
        .map_domain(|error: &'static str| format!("mapped {error}"));
    assert_eq!(
        TranscodeError::InsufficientOutput {
            output_index: 1,
            required: 3,
            available: 2,
        },
        mapped,
    );

    let mapped = TranscodeError::<&'static str>::OutputLengthOverflow
        .map_domain(|error: &'static str| format!("mapped {error}"));
    assert_eq!(TranscodeError::OutputLengthOverflow, mapped);

    let mapped = TranscodeError::<String>::domain("inner".to_string())
        .map_domain(|error| format!("mapped {error}"));
    assert_eq!(TranscodeError::Domain("mapped inner".to_string()), mapped,);
}

#[test]
fn test_transcode_error_display_formats_all_variants() {
    assert_eq!(
        "invalid input index 3; input length is 1",
        TranscodeError::<DomainError>::invalid_input_index(3, 1).to_string(),
    );
    assert_eq!(
        "invalid output index 4; output length is 2",
        TranscodeError::<DomainError>::invalid_output_index(4, 2).to_string(),
    );
    assert_eq!(
        "insufficient output at index 1; required 3, available 2",
        TranscodeError::<DomainError>::insufficient_output(1, 3, 2).to_string(),
    );
    assert_eq!(
        "output length arithmetic overflow",
        TranscodeError::<DomainError>::OutputLengthOverflow.to_string(),
    );
    assert_eq!(
        "domain failure",
        TranscodeError::Domain(DomainError).to_string(),
    );
}

#[test]
fn test_transcode_error_source_returns_domain_error() {
    let error = TranscodeError::Domain(DomainError);
    assert!(error.source().is_some());
    assert!(
        TranscodeError::<DomainError>::invalid_input_index(0, 0)
            .source()
            .is_none()
    );
}

#[test]
fn test_transcode_error_ensure_input_index_accepts_valid_index() {
    TranscodeError::<&'static str>::ensure_input_index(4, 2)
        .expect("valid index");
}

#[test]
fn test_transcode_error_ensure_input_index_rejects_out_of_range() {
    let error = TranscodeError::<&'static str>::ensure_input_index(2, 5)
        .expect_err("out-of-range index");

    assert_eq!(TranscodeError::invalid_input_index(5, 2), error,);
}

#[test]
fn test_transcode_error_ensure_output_index_accepts_valid_index() {
    TranscodeError::<&'static str>::ensure_output_index(4, 4)
        .expect("valid index");
}

#[test]
fn test_transcode_error_ensure_output_index_rejects_out_of_range() {
    let error = TranscodeError::<&'static str>::ensure_output_index(1, 2)
        .expect_err("out-of-range index");

    assert_eq!(TranscodeError::invalid_output_index(2, 1), error);
}

#[test]
fn test_transcode_error_ensure_transcode_indices_accepts_valid_indices() {
    TranscodeError::<&'static str>::ensure_transcode_indices(3, 1, 5, 2)
        .expect("valid indices");
}

#[test]
fn test_transcode_error_ensure_transcode_indices_rejects_invalid_output_index()
{
    let error =
        TranscodeError::<&'static str>::ensure_transcode_indices(3, 0, 1, 2)
            .expect_err("invalid output index");

    assert_eq!(TranscodeError::invalid_output_index(2, 1), error);
}

#[test]
fn test_transcode_error_ensure_output_capacity_accepts_sufficient_capacity() {
    TranscodeError::<&'static str>::ensure_output_capacity(4, 1, 2)
        .expect("sufficient capacity");
}

#[test]
fn test_transcode_error_ensure_output_capacity_delegates_to_output_index() {
    let error = TranscodeError::<&'static str>::ensure_output_capacity(2, 5, 0)
        .expect_err("invalid output index");

    assert_eq!(TranscodeError::invalid_output_index(5, 2), error,);
}

#[test]
fn test_transcode_error_ensure_output_capacity_rejects_insufficient_capacity() {
    let error = TranscodeError::<&'static str>::ensure_output_capacity(4, 2, 3)
        .expect_err("insufficient capacity");

    assert_eq!(TranscodeError::insufficient_output(2, 3, 2), error);
}

#[test]
fn test_transcode_error_ensure_output_range_accepts_valid_range() {
    TranscodeError::<&'static str>::ensure_output_range(4, 1, 2, 2)
        .expect("valid range");
}

#[test]
fn test_transcode_error_ensure_output_range_rejects_insufficient_range() {
    let error = TranscodeError::<&'static str>::ensure_output_range(4, 1, 1, 2)
        .expect_err("insufficient range");

    assert_eq!(TranscodeError::insufficient_output(1, 2, 1), error,);
}

#[test]
fn test_transcode_error_ensure_output_range_rejects_overflowing_range() {
    let error = TranscodeError::<&'static str>::ensure_output_range(4, 3, 2, 0)
        .expect_err("overflowing range");

    assert_eq!(TranscodeError::invalid_output_index(3, 4), error,);
}

#[test]
fn test_transcode_error_ensure_output_range_rejects_invalid_output_index() {
    let error = TranscodeError::<&'static str>::ensure_output_range(4, 5, 0, 0)
        .expect_err("invalid output index");

    assert_eq!(TranscodeError::invalid_output_index(5, 4), error);
}

#[test]
fn test_transcode_error_ensure_output_range_rejects_range_length_overflow() {
    let error = TranscodeError::<&'static str>::ensure_output_range(
        usize::MAX,
        usize::MAX,
        1,
        0,
    )
    .expect_err("range length overflow");

    assert_eq!(
        TranscodeError::invalid_output_index(usize::MAX, usize::MAX),
        error,
    );
}

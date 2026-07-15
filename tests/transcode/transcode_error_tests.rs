// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{
    CapacityError, DecodeFailure, TranscodeConvertError, TranscodeDecodeError,
    TranscodeDomainError, TranscodeEncodeError, TranscodeFailure,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("domain error")]
struct DomainError;

#[test]
fn test_decode_error_wraps_framework_and_domain_errors() {
    let failure = TranscodeDecodeError::<DomainError>::invalid_input_index(3, 1);
    assert_eq!(
        TranscodeDecodeError::Failure(TranscodeFailure::InvalidInputIndex {
            index: 3,
            input_len: 1,
        }),
        failure,
    );

    let domain = TranscodeDecodeError::<DomainError>::domain_finish(DomainError);
    assert_eq!(Some(&DomainError), domain.domain_ref());
    assert_eq!(
        Some(&TranscodeDomainError::Finish {
            source: DomainError,
        }),
        domain.domain_error_ref(),
    );
}

#[test]
fn test_decode_error_maps_decode_failure() {
    let incomplete = DecodeFailure::<DomainError>::Incomplete {
        required_total: crate::nz(4),
    };
    assert_eq!(
        TranscodeDecodeError::incomplete_input(2, 4, 1),
        TranscodeDecodeError::from_decode_failure(incomplete, 2, 1),
    );

    let invalid = DecodeFailure::invalid(DomainError, crate::nz(1));
    assert_eq!(
        TranscodeDecodeError::domain_main_with_consumed(DomainError, 5, Some(crate::nz(1)),),
        TranscodeDecodeError::from_decode_failure(invalid, 5, 3),
    );
}

#[test]
fn test_encode_error_carries_unencodable_value_context() {
    let error = TranscodeEncodeError::<DomainError, char>::unencodable(4, 'x');
    assert_eq!(Some((4, Some(&'x'))), error.unencodable_ref());

    let mapped = error.map_value(|value| value as u32);
    assert_eq!(
        TranscodeEncodeError::<DomainError, u32>::unencodable(4, 'x' as u32),
        mapped,
    );
}

#[test]
fn test_convert_error_from_decode_and_encode_errors() {
    type Convert = TranscodeConvertError<&'static str, &'static str, char>;

    let decode: TranscodeDecodeError<&'static str> = TranscodeDecodeError::domain_main("decode", 7);
    assert_eq!(Convert::decode_domain_main("decode", 7), decode.into());

    let encode = TranscodeEncodeError::unencodable_without_context(3);
    assert_eq!(
        Convert::unencodable_without_context(3),
        Convert::from(encode),
    );
}

#[test]
fn test_convert_error_adds_fallback_unencodable_context() {
    type Convert = TranscodeConvertError<&'static str, &'static str, char>;

    let encode = TranscodeEncodeError::unencodable_without_context(9);
    assert_eq!(
        Convert::unencodable(9, 'z'),
        Convert::from_encode_error_with_value(encode, 'z'),
    );
}

#[test]
fn test_capacity_errors_map_to_framework_failures() {
    let decode: TranscodeDecodeError<DomainError> = CapacityError::OutputLengthOverflow.into();
    let encode: TranscodeEncodeError<DomainError, char> =
        CapacityError::OutputLengthOverflow.into();
    let convert: TranscodeConvertError<DomainError, DomainError, char> =
        CapacityError::OutputLengthOverflow.into();

    assert_eq!(TranscodeDecodeError::output_length_overflow(), decode);
    assert_eq!(TranscodeEncodeError::output_length_overflow(), encode);
    assert_eq!(TranscodeConvertError::output_length_overflow(), convert);
}

#[test]
fn test_transcode_failure_output_range_validation() {
    assert_eq!(Ok(()), TranscodeFailure::ensure_output_range(4, 1, 2, 2));
    assert_eq!(
        Err(TranscodeFailure::InvalidOutputIndex {
            index: 5,
            output_len: 4,
        }),
        TranscodeFailure::ensure_output_range(4, 5, 0, 0),
    );
    assert_eq!(
        Err(TranscodeFailure::InvalidOutputIndex {
            index: 3,
            output_len: 4,
        }),
        TranscodeFailure::ensure_output_range(4, 3, 2, 1),
    );
    assert_eq!(
        Err(TranscodeFailure::InsufficientOutput {
            output_index: 1,
            required: 3,
            available: 2,
        }),
        TranscodeFailure::ensure_output_range(4, 1, 2, 3),
    );
}

#[test]
fn test_domain_error_accessors_and_mapping_cover_all_phases() {
    let reset = TranscodeDomainError::reset("reset");
    let main = TranscodeDomainError::main_with_consumed("main", 4, Some(crate::nz(2)));
    let finish = TranscodeDomainError::finish("finish");

    assert_eq!("reset", *reset.source());
    assert_eq!("main", *main.source());
    assert_eq!("reset", reset.into_source());
    assert_eq!(Some(4), main.input_index());
    assert_eq!(Some(crate::nz(2)), main.input_consumed());
    assert_eq!(None, finish.input_index());
    assert_eq!(None, finish.input_consumed());
    assert_eq!("finish", finish.into_source());

    assert_eq!(
        TranscodeDomainError::Reset { source: 5 },
        TranscodeDomainError::reset("reset").map_source(str::len),
    );
    assert_eq!(
        TranscodeDomainError::Main {
            source: 4,
            input_index: 4,
            input_consumed: Some(crate::nz(2)),
        },
        main.map_source(str::len),
    );
    assert_eq!(
        TranscodeDomainError::Finish { source: 6 },
        TranscodeDomainError::finish("finish").map_source(str::len),
    );
}

#[test]
fn test_decode_error_accessors_mapping_and_validation() {
    let failure = TranscodeDecodeError::<&str>::invalid_output_index(3, 1);
    let domain = TranscodeDecodeError::<&str>::domain_main("decode", 2);

    assert!(!failure.is_domain());
    assert!(domain.is_domain());
    assert_eq!(
        Some(&TranscodeFailure::InvalidOutputIndex {
            index: 3,
            output_len: 1,
        }),
        failure.failure_ref(),
    );
    assert_eq!(None, domain.failure_ref());
    assert_eq!(None, failure.domain_error_ref());
    assert_eq!(
        Some(&TranscodeDomainError::Main {
            source: "decode",
            input_index: 2,
            input_consumed: None,
        }),
        domain.domain_error_ref(),
    );
    assert_eq!(None, failure.domain_ref());

    assert_eq!(
        TranscodeDecodeError::<usize>::invalid_output_index(3, 1),
        failure.map_domain(str::len),
    );
    assert_eq!(
        TranscodeDecodeError::<usize>::domain_main(6, 2),
        domain.map_domain(str::len),
    );

    assert_eq!(
        Ok(()),
        TranscodeDecodeError::<&str>::ensure_input_index(2, 2)
    );
    assert!(matches!(
        TranscodeDecodeError::<&str>::ensure_input_index(2, 3),
        Err(TranscodeDecodeError::Failure(
            TranscodeFailure::InvalidInputIndex { .. }
        ))
    ));
    assert_eq!(
        Ok(()),
        TranscodeDecodeError::<&str>::ensure_min_input(3, 1, 2)
    );
    assert!(matches!(
        TranscodeDecodeError::<&str>::ensure_min_input(3, 2, 2),
        Err(TranscodeDecodeError::Failure(
            TranscodeFailure::IncompleteInput { .. }
        ))
    ));
    assert_eq!(
        Ok(()),
        TranscodeDecodeError::<&str>::ensure_no_trailing_input(2, 2),
    );
    assert!(matches!(
        TranscodeDecodeError::<&str>::ensure_no_trailing_input(1, 2),
        Err(TranscodeDecodeError::Failure(
            TranscodeFailure::TrailingInput { .. }
        ))
    ));
    assert_eq!(
        Ok(()),
        TranscodeDecodeError::<&str>::ensure_output_range(4, 1, 2, 2)
    );
    assert!(matches!(
        TranscodeDecodeError::<&str>::ensure_output_range(4, 1, 1, 2),
        Err(TranscodeDecodeError::Failure(
            TranscodeFailure::InsufficientOutput { .. }
        ))
    ));
}

#[test]
fn test_encode_error_accessors_mapping_and_validation() {
    let failure = TranscodeEncodeError::<&str, char>::incomplete_input(1, 2, 0);
    let trailing = TranscodeEncodeError::<&str, char>::trailing_input(1, 1);
    let domain = TranscodeEncodeError::<&str, char>::domain_reset("reset");
    let main = TranscodeEncodeError::<&str, char>::domain_main("encode", 3);
    let finish = TranscodeEncodeError::<&str, char>::domain_finish("finish");
    let no_context = TranscodeEncodeError::<&str, char>::unencodable_without_context(8);

    assert!(!failure.is_domain());
    assert!(domain.is_domain());
    assert!(matches!(
        trailing.failure_ref(),
        Some(TranscodeFailure::TrailingInput { .. })
    ));
    assert_eq!(None, domain.failure_ref());
    assert_eq!(None, failure.domain_error_ref());
    assert_eq!(
        Some(&TranscodeDomainError::Reset { source: "reset" }),
        domain.domain_error_ref(),
    );
    assert_eq!(Some(&"reset"), domain.domain_ref());
    assert_eq!(None, failure.domain_ref());
    assert_eq!(None, failure.unencodable_ref());
    assert_eq!(Some((8, None)), no_context.unencodable_ref());

    assert_eq!(
        TranscodeEncodeError::<usize, char>::incomplete_input(1, 2, 0),
        failure.map_domain(str::len),
    );
    assert_eq!(
        TranscodeEncodeError::<usize, char>::unencodable_without_context(8),
        no_context.map_domain(str::len),
    );
    assert_eq!(
        TranscodeEncodeError::<usize, char>::domain_main(6, 3),
        main.map_domain(str::len),
    );
    assert_eq!(
        TranscodeEncodeError::<&str, u32>::domain_finish("finish"),
        finish.map_value(|value: char| value as u32),
    );
    assert_eq!(
        TranscodeEncodeError::<&str, u32>::trailing_input(1, 1),
        TranscodeEncodeError::<&str, char>::trailing_input(1, 1)
            .map_value(|value: char| value as u32),
    );

    assert_eq!(
        Ok(()),
        TranscodeEncodeError::<&str, char>::ensure_output_index(2, 2)
    );
    assert!(matches!(
        TranscodeEncodeError::<&str, char>::ensure_output_index(2, 3),
        Err(TranscodeEncodeError::Failure(
            TranscodeFailure::InvalidOutputIndex { .. }
        ))
    ));
    assert_eq!(
        Ok(()),
        TranscodeEncodeError::<&str, char>::ensure_transcode_indices(2, 1, 2, 1),
    );
    assert!(matches!(
        TranscodeEncodeError::<&str, char>::ensure_transcode_indices(2, 3, 2, 1),
        Err(TranscodeEncodeError::Failure(
            TranscodeFailure::InvalidInputIndex { .. }
        ))
    ));
    assert_eq!(
        Ok(()),
        TranscodeEncodeError::<&str, char>::ensure_output_capacity(3, 1, 2),
    );
    assert!(matches!(
        TranscodeEncodeError::<&str, char>::ensure_output_capacity(3, 1, 3),
        Err(TranscodeEncodeError::Failure(
            TranscodeFailure::InsufficientOutput { .. }
        ))
    ));
}

#[test]
fn test_convert_error_accessors_mapping_and_validation() {
    type Convert<V = char> = TranscodeConvertError<&'static str, &'static str, V>;

    let failure: Convert = Convert::incomplete_input(1, 2, 0);
    let trailing: Convert = Convert::trailing_input(1, 1);
    let decode_reset: Convert = Convert::decode_domain_reset("decode reset");
    let decode_main: Convert = Convert::decode_domain_main("decode", 3);
    let decode_consumed: Convert =
        Convert::decode_domain_main_with_consumed("decode", 3, Some(crate::nz(1)));
    let decode_finish: Convert = Convert::decode_domain_finish("decode finish");
    let encode_reset: Convert = Convert::encode_domain_reset("encode reset");
    let encode_main: Convert = Convert::encode_domain_main("encode", 4);
    let encode_finish: Convert = Convert::encode_domain_finish("encode finish");
    let unencodable = Convert::unencodable(7, 'q');
    let no_context: Convert = Convert::unencodable_without_context(8);

    assert!(matches!(
        failure.failure_ref(),
        Some(TranscodeFailure::IncompleteInput { .. })
    ));
    assert!(matches!(
        trailing.failure_ref(),
        Some(TranscodeFailure::TrailingInput { .. })
    ));
    assert_eq!(None, decode_reset.failure_ref());
    assert_eq!(None, encode_reset.failure_ref());
    assert_eq!(None, unencodable.failure_ref());
    assert_eq!(Some((7, Some(&'q'))), unencodable.unencodable_ref());
    assert_eq!(Some((8, None)), no_context.unencodable_ref());
    assert_eq!(None, failure.unencodable_ref());
    assert_eq!(None, decode_reset.unencodable_ref());
    assert_eq!(None, encode_reset.unencodable_ref());

    assert_eq!(
        TranscodeConvertError::<usize, &str, char>::incomplete_input(1, 2, 0),
        failure.map_decode_domain(str::len),
    );
    assert_eq!(
        TranscodeConvertError::<usize, &str, char>::decode_domain_main(6, 3),
        decode_main.map_decode_domain(str::len),
    );
    assert_eq!(
        TranscodeConvertError::<usize, &str, char>::encode_domain_main("encode", 4),
        encode_main.map_decode_domain(str::len),
    );
    assert_eq!(
        TranscodeConvertError::<usize, &str, char>::unencodable(7, 'q'),
        unencodable.map_decode_domain(str::len),
    );
    assert_eq!(
        TranscodeConvertError::<&str, usize, char>::decode_domain_main("decode", 3),
        TranscodeConvertError::<&str, &str, char>::decode_domain_main("decode", 3)
            .map_encode_domain(str::len),
    );
    assert_eq!(
        TranscodeConvertError::<&str, usize, char>::incomplete_input(1, 2, 0),
        Convert::<char>::incomplete_input(1, 2, 0).map_encode_domain(str::len),
    );
    assert_eq!(
        TranscodeConvertError::<&str, usize, char>::encode_domain_finish(13),
        encode_finish.map_encode_domain(str::len),
    );
    assert_eq!(
        TranscodeConvertError::<&str, usize, char>::unencodable(7, 'q'),
        TranscodeConvertError::<&str, &str, char>::unencodable(7, 'q').map_encode_domain(str::len),
    );
    assert_eq!(
        TranscodeConvertError::<&str, &str, u32>::decode_domain_finish("decode finish"),
        decode_finish.map_value(|value: char| value as u32),
    );
    assert_eq!(
        TranscodeConvertError::<&str, &str, u32>::incomplete_input(1, 2, 0),
        Convert::<char>::incomplete_input(1, 2, 0).map_value(|value: char| value as u32),
    );
    assert_eq!(
        TranscodeConvertError::<&str, &str, u32>::encode_domain_reset("encode reset"),
        encode_reset.map_value(|value: char| value as u32),
    );
    assert_eq!(
        TranscodeConvertError::<&str, &str, u32>::unencodable(7, 'q' as u32),
        TranscodeConvertError::<&str, &str, char>::unencodable(7, 'q')
            .map_value(|value| value as u32),
    );

    let encode_failure = TranscodeEncodeError::<&str, char>::invalid_output_index(3, 1);
    assert_eq!(
        Convert::<char>::invalid_output_index(3, 1),
        Convert::<char>::from(encode_failure)
    );
    let encode_domain = TranscodeEncodeError::<&str, char>::domain_reset("encode reset");
    assert_eq!(
        Convert::<char>::encode_domain_reset("encode reset"),
        Convert::<char>::from(encode_domain)
    );
    let encode_unencodable = TranscodeEncodeError::<&str, char>::unencodable(9, 'x');
    assert_eq!(
        Convert::<char>::unencodable(9, 'x'),
        Convert::<char>::from(encode_unencodable)
    );
    assert_eq!(
        Convert::<char>::encode_domain_finish("encode finish"),
        Convert::<char>::from_encode_error_with_value(
            TranscodeEncodeError::<&str, char>::domain_finish("encode finish"),
            'z',
        ),
    );
    let decode_failure = TranscodeDecodeError::<&str>::invalid_input_index(4, 1);
    assert_eq!(
        Convert::<char>::invalid_input_index(4, 1),
        Convert::<char>::from(decode_failure)
    );
    let decode_domain = TranscodeDecodeError::<&str>::domain_reset("decode reset");
    assert_eq!(
        Convert::<char>::decode_domain_reset("decode reset"),
        Convert::<char>::from(decode_domain)
    );
    assert_eq!(
        decode_consumed,
        Convert::<char>::decode_domain_main_with_consumed("decode", 3, Some(crate::nz(1)))
    );

    assert_eq!(Ok(()), Convert::<char>::ensure_output_index(2, 2));
    assert!(matches!(
        Convert::<char>::ensure_output_index(2, 3),
        Err(TranscodeConvertError::Failure(
            TranscodeFailure::InvalidOutputIndex { .. }
        ))
    ));
    assert_eq!(
        Ok(()),
        Convert::<char>::ensure_transcode_indices(2, 1, 2, 1)
    );
    assert!(matches!(
        Convert::<char>::ensure_transcode_indices(2, 3, 2, 1),
        Err(TranscodeConvertError::Failure(
            TranscodeFailure::InvalidInputIndex { .. }
        ))
    ));
    assert_eq!(Ok(()), Convert::<char>::ensure_output_capacity(3, 1, 2));
    assert!(matches!(
        Convert::<char>::ensure_output_capacity(3, 1, 3),
        Err(TranscodeConvertError::Failure(
            TranscodeFailure::InsufficientOutput { .. }
        ))
    ));
}

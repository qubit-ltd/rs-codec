// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::TranscodeConvertError;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeEncodeError;
use qubit_codec::TranscodeFailure;

#[test]
fn test_convert_error_from_decode_and_encode_errors() {
    type Convert = TranscodeConvertError<&'static str, &'static str, char>;

    let decode: TranscodeDecodeError<&'static str> =
        TranscodeDecodeError::domain_main("decode", 7);
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
fn test_convert_error_accessors_mapping_and_validation() {
    type Convert<V = char> =
        TranscodeConvertError<&'static str, &'static str, V>;

    let failure: Convert =
        Convert::Failure(TranscodeFailure::incomplete_input(1, 2, 0));
    let trailing: Convert =
        Convert::Failure(TranscodeFailure::trailing_input(1, 1));
    let decode_reset: Convert = Convert::decode_domain_reset("decode reset");
    let decode_main: Convert = Convert::decode_domain_main("decode", 3);
    let decode_consumed: Convert = Convert::decode_domain_main_with_consumed(
        "decode",
        3,
        Some(crate::nonzero(1)),
    );
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
        TranscodeConvertError::Failure(TranscodeFailure::incomplete_input(
            1, 2, 0
        )),
        failure.map_decode_domain(str::len),
    );
    assert_eq!(
        TranscodeConvertError::<usize, &str, char>::decode_domain_main(6, 3),
        decode_main.map_decode_domain(str::len),
    );
    assert_eq!(
        TranscodeConvertError::<usize, &str, char>::encode_domain_main(
            "encode", 4
        ),
        encode_main.map_decode_domain(str::len),
    );
    assert_eq!(
        TranscodeConvertError::<usize, &str, char>::unencodable(7, 'q'),
        unencodable.map_decode_domain(str::len),
    );
    assert_eq!(
        TranscodeConvertError::<&str, usize, char>::decode_domain_main(
            "decode", 3
        ),
        TranscodeConvertError::<&str, &str, char>::decode_domain_main(
            "decode", 3
        )
        .map_encode_domain(str::len),
    );
    assert_eq!(
        TranscodeConvertError::Failure(TranscodeFailure::incomplete_input(
            1, 2, 0
        )),
        Convert::<char>::Failure(TranscodeFailure::incomplete_input(1, 2, 0))
            .map_encode_domain(str::len),
    );
    assert_eq!(
        TranscodeConvertError::<&str, usize, char>::encode_domain_finish(13),
        encode_finish.map_encode_domain(str::len),
    );
    assert_eq!(
        TranscodeConvertError::<&str, usize, char>::unencodable(7, 'q'),
        TranscodeConvertError::<&str, &str, char>::unencodable(7, 'q')
            .map_encode_domain(str::len),
    );
    assert_eq!(
        TranscodeConvertError::<&str, &str, u32>::decode_domain_finish(
            "decode finish"
        ),
        decode_finish.map_value(|value: char| value as u32),
    );
    assert_eq!(
        TranscodeConvertError::Failure(TranscodeFailure::incomplete_input(
            1, 2, 0
        )),
        Convert::<char>::Failure(TranscodeFailure::incomplete_input(1, 2, 0))
            .map_value(|value: char| value as u32),
    );
    assert_eq!(
        TranscodeConvertError::<&str, &str, u32>::encode_domain_reset(
            "encode reset"
        ),
        encode_reset.map_value(|value: char| value as u32),
    );
    assert_eq!(
        TranscodeConvertError::<&str, &str, u32>::unencodable(7, 'q' as u32),
        TranscodeConvertError::<&str, &str, char>::unencodable(7, 'q')
            .map_value(|value| value as u32),
    );
    assert_eq!(
        TranscodeConvertError::<&str, &str, u32>::unencodable_without_context(
            8
        ),
        no_context.map_value(|value: char| value as u32),
    );

    let encode_failure = TranscodeEncodeError::Failure(
        TranscodeFailure::invalid_output_index(3, 1),
    );
    assert_eq!(
        Convert::<char>::Failure(TranscodeFailure::invalid_output_index(3, 1)),
        Convert::<char>::from(encode_failure)
    );
    let encode_domain =
        TranscodeEncodeError::<&str, char>::domain_reset("encode reset");
    assert_eq!(
        Convert::<char>::encode_domain_reset("encode reset"),
        Convert::<char>::from(encode_domain)
    );
    let encode_unencodable =
        TranscodeEncodeError::<&str, char>::unencodable(9, 'x');
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
    assert_eq!(
        Convert::<char>::unencodable(9, 'x'),
        Convert::<char>::from_encode_error_with_value(
            TranscodeEncodeError::<&str, char>::unencodable(9, 'x'),
            'z',
        ),
    );
    let decode_failure = TranscodeDecodeError::Failure(
        TranscodeFailure::invalid_input_index(4, 1),
    );
    assert_eq!(
        Convert::<char>::Failure(TranscodeFailure::invalid_input_index(4, 1)),
        Convert::<char>::from(decode_failure)
    );
    let decode_domain =
        TranscodeDecodeError::<&str>::domain_reset("decode reset");
    assert_eq!(
        Convert::<char>::decode_domain_reset("decode reset"),
        Convert::<char>::from(decode_domain)
    );
    assert_eq!(
        decode_consumed,
        Convert::<char>::decode_domain_main_with_consumed(
            "decode",
            3,
            Some(crate::nonzero(1))
        )
    );

    assert_eq!(
        Ok::<(), Convert>(()),
        TranscodeFailure::ensure_output_index(2, 2)
            .map_err(Convert::<char>::from),
    );
    assert!(matches!(
        TranscodeFailure::ensure_output_index(2, 3)
            .map_err(Convert::<char>::from),
        Err(TranscodeConvertError::Failure(
            TranscodeFailure::InvalidOutputIndex { .. }
        ))
    ));
    assert_eq!(
        Ok::<(), Convert>(()),
        TranscodeFailure::ensure_transcode_indices(2, 1, 2, 1)
            .map_err(Convert::<char>::from)
    );
    assert!(matches!(
        TranscodeFailure::ensure_transcode_indices(2, 3, 2, 1)
            .map_err(Convert::<char>::from),
        Err(TranscodeConvertError::Failure(
            TranscodeFailure::InvalidInputIndex { .. }
        ))
    ));
    assert_eq!(
        Ok::<(), Convert>(()),
        TranscodeFailure::ensure_output_capacity(3, 1, 2)
            .map_err(Convert::<char>::from),
    );
    assert!(matches!(
        TranscodeFailure::ensure_output_capacity(3, 1, 3)
            .map_err(Convert::<char>::from),
        Err(TranscodeConvertError::Failure(
            TranscodeFailure::InsufficientOutput { .. }
        ))
    ));
}

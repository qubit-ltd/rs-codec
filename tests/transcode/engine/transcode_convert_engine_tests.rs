// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the reusable buffered converter engine.

use core::cell::Cell;
use std::hint::black_box;
use std::rc::Rc;

use qubit_codec as codec;
use qubit_codec::CapacityError;
use qubit_codec::TranscodeConvertError;
use qubit_codec::TranscodeConverter;
use qubit_codec::TranscodeFailure;
use qubit_codec::TranscodeProgress;
use qubit_codec::TranscodeStatus;
use qubit_codec::Transcoder;
use qubit_codec::engine::TranscodeConvertEngine;

use super::internal::ChangingFinishBoundDecodeHooks;
use super::internal::CopyConvertEngine;
use super::internal::CountedTargetReplacementHooks;
use super::internal::EngineError;
use super::internal::EofIncompleteDecodeHooks;
use super::internal::EofIncompleteMode;
use super::internal::ErrorPathDecodeFinish;
use super::internal::ErrorPathEncodeMode;
use super::internal::ErrorPathHooks;
use super::internal::ErrorSourceCodec;
use super::internal::FactoryDecodeHooks;
use super::internal::FactoryEncodeHooks;
use super::internal::FinishDecodeHooks;
use super::internal::FinishFailSourceDecodeHooks;
use super::internal::FinishFailingSourceCodec;
use super::internal::FinishHooks;
use super::internal::FinishOverflowEncodeHooks;
use super::internal::FinishOverflowTargetCodec;
use super::internal::FinishValueSourceCodec;
use super::internal::FixedFinishBoundDecodeHooks;
use super::internal::ImpossibleFailureDecodeHooks;
use super::internal::MismatchCapacityEncodeHooks;
use super::internal::MismatchCapacityResetEmittingDecodeHooks;
use super::internal::MismatchCapacityTargetCodec;
use super::internal::OverflowResetSourceCodec;
use super::internal::OverflowResetSourceDecodeHooks;
use super::internal::OverflowResetTargetCodec;
use super::internal::OverflowResetTargetEncodeHooks;
use super::internal::PerBatchOverheadEncodeHooks;
use super::internal::RepairAction;
use super::internal::ResetDependentTargetCodec;
use super::internal::ResetEmittingSourceCodec;
use super::internal::ResetEmittingTargetCodec;
use super::internal::ResetEncodingFailSourceCodec;
use super::internal::ResetEncodingFailSourceDecodeHooks;
use super::internal::ResetFailSourceDecodeHooks;
use super::internal::ResetFailTargetCodec;
use super::internal::ResetFailTargetHooks;
use super::internal::ResetFailingSourceCodec;
use super::internal::ResetObservingDecodeHooks;
use super::internal::ResetSourceDecodeHooks;
use super::internal::ResetTargetHooks;
use super::internal::SOURCE_RESET_SENTINEL;
use super::internal::ShortSourceCodec;
use super::internal::SourceCodec;
use super::internal::StatelessResetFailingSourceCodec;
use super::internal::StatelessResetFailingSourceDecodeHooks;
use super::internal::StatelessResetSourceCodec;
use super::internal::StatelessResetSourceDecodeHooks;
use super::internal::StrictDecodeHooks;
use super::internal::StrictEncodeHooks;
use super::internal::TargetCodec;
use super::internal::TargetResetFailError;
use super::internal::new_batch_finish_engine;
use super::internal::new_copy_engine;
use super::internal::new_error_path_engine;
use super::internal::new_finish_encode_engine;
use super::internal::new_finish_engine;
use super::internal::new_repair_engine;

#[test]
fn test_transcode_convert_engine_exposes_codecs_hooks_and_parts() {
    let mut engine = TranscodeConvertEngine::new(SourceCodec, TargetCodec, StrictDecodeHooks, StrictEncodeHooks);

    assert_eq!(&SourceCodec, engine.source_codec());
    assert_eq!(&TargetCodec, engine.target_codec());
    assert_eq!(&StrictDecodeHooks, engine.decode_hooks());
    assert_eq!(&StrictEncodeHooks, engine.encode_hooks());
    *engine.source_codec_mut() = SourceCodec;
    *engine.target_codec_mut() = TargetCodec;
    *engine.decode_hooks_mut() = StrictDecodeHooks;
    *engine.encode_hooks_mut() = StrictEncodeHooks;

    let (source, target, decode_hooks, encode_hooks) = engine.into_parts();
    assert_eq!(SourceCodec, source);
    assert_eq!(TargetCodec, target);
    assert_eq!(StrictDecodeHooks, decode_hooks);
    assert_eq!(StrictEncodeHooks, encode_hooks);
}

#[test]
fn test_transcode_convert_engine_applies_incomplete_eof_replacement() {
    let mut engine = TranscodeConvertEngine::new(
        ShortSourceCodec,
        TargetCodec,
        EofIncompleteDecodeHooks {
            mode: EofIncompleteMode::Emit,
        },
        StrictEncodeHooks,
    );
    engine.reset(&mut [], 0).expect("initialize stream");
    let mut output = [0_u8; 1];

    let progress = engine
        .transcode_eof(&[0x5a], 0, &mut output, 0)
        .expect("EOF replacement should pass through the target encoder");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(1, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([77], output);
}

#[test]
fn test_transcode_convert_engine_skips_incomplete_eof_tail() {
    let mut engine = TranscodeConvertEngine::new(
        ShortSourceCodec,
        TargetCodec,
        EofIncompleteDecodeHooks {
            mode: EofIncompleteMode::Skip,
        },
        StrictEncodeHooks,
    );
    engine.reset(&mut [], 0).expect("initialize stream");
    let mut output = [0_u8; 1];

    let progress = engine
        .transcode_eof(&[0x5a], 0, &mut output, 0)
        .expect("EOF skip should consume the tail");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(1, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_transcode_convert_engine_rejects_incomplete_eof_tail() {
    let mut engine = TranscodeConvertEngine::new(
        ShortSourceCodec,
        TargetCodec,
        EofIncompleteDecodeHooks {
            mode: EofIncompleteMode::Reject,
        },
        StrictEncodeHooks,
    );
    engine.reset(&mut [], 0).expect("initialize stream");
    let mut output = [0_u8; 1];

    let error = engine
        .transcode_eof(&[0x5a], 0, &mut output, 0)
        .expect_err("default EOF policy should reject the tail");

    assert_eq!(
        TranscodeConvertError::Failure(TranscodeFailure::incomplete_input(0, 2, 1)),
        error,
    );
}

#[test]
fn test_buffered_convert_engine_reports_bounds_and_resets() {
    type ConvertErrorType = TranscodeConvertError<EngineError, EngineError, u8>;
    type TranscodeCompleteIntoFn = fn(&mut CopyConvertEngine, &[u8], &mut [u8]) -> Result<usize, ConvertErrorType>;

    let mut engine = new_copy_engine();
    let max_total_output_len: fn(&CopyConvertEngine, usize) -> Result<usize, CapacityError> =
        CopyConvertEngine::max_total_output_len;
    let transcode_complete_into: TranscodeCompleteIntoFn = CopyConvertEngine::transcode_complete_into;

    assert_eq!(Ok(4), engine.max_transcode_output_len(3));
    assert_eq!(Ok(5), max_total_output_len(&engine, 3));
    assert_eq!(Ok(1), engine.max_finish_output_len());
    assert_eq!(Ok(0), engine.max_reset_output_len());

    let mut output = [0_u8; 5];
    let written = transcode_complete_into(&mut engine, &[1, 2, 3], &mut output)
        .expect("complete conversion should fit the planned output");
    assert_eq!(3, written);
    assert_eq!(&[2, 3, 4], &output[..written]);

    engine.reset(&mut [], 0).expect("reset");
    assert_eq!(Ok(1), engine.max_finish_output_len());
}

/// Verifies that converter bounds include every reachable pending state.
#[test]
fn test_buffered_convert_engine_bounds_include_possible_pending_value() {
    let engine = new_copy_engine();

    assert_eq!(Ok(1), engine.max_transcode_output_len(0));
    assert_eq!(Ok(1), engine.max_finish_output_len());
    assert_eq!(Ok(2), engine.max_total_output_len(0));
}

#[test]
fn test_buffered_convert_engine_finish_bound_sums_separate_encode_batches() {
    let engine = TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        FinishDecodeHooks::default(),
        PerBatchOverheadEncodeHooks,
    );

    assert_eq!(Ok(4), engine.max_finish_output_len());
}

#[test]
fn test_buffered_convert_engine_implements_transcoder() {
    fn assert_converter<T: TranscodeConverter<Input = u8, Output = u8>>() {}
    assert_converter::<CopyConvertEngine>();

    type EngineResult<T> = Result<T, TranscodeConvertError<EngineError, EngineError, u8>>;
    type TranscodeFn = fn(&mut CopyConvertEngine, &[u8], usize, &mut [u8], usize) -> EngineResult<TranscodeProgress>;
    type OutputFn = fn(&mut CopyConvertEngine, &mut [u8], usize) -> EngineResult<usize>;

    let mut engine = new_copy_engine();
    let mut output = [0_u8; 2];
    let max_transcode_output_len: fn(&CopyConvertEngine, usize) -> Result<usize, CapacityError> =
        black_box(<CopyConvertEngine as Transcoder>::max_transcode_output_len);
    let max_finish_output_len: fn(&CopyConvertEngine) -> Result<usize, CapacityError> =
        black_box(<CopyConvertEngine as Transcoder>::max_finish_output_len);
    let max_reset_output_len: fn(&CopyConvertEngine) -> Result<usize, CapacityError> =
        black_box(<CopyConvertEngine as Transcoder>::max_reset_output_len);
    let transcode: TranscodeFn = black_box(<CopyConvertEngine as Transcoder>::transcode);
    let reset: OutputFn = black_box(<CopyConvertEngine as Transcoder>::reset);
    let finish: OutputFn = black_box(<CopyConvertEngine as Transcoder>::finish);

    assert_eq!(Ok(3), max_transcode_output_len(&engine, 2));
    assert_eq!(Ok(1), max_finish_output_len(&engine));
    assert_eq!(Ok(0), max_reset_output_len(&engine));
    let progress = transcode(&mut engine, &[3, 4], 0, &mut output, 0).expect("engine should convert through the trait");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((2, 2), (progress.read(), progress.written()));
    assert_eq!([4, 5], output);

    let mut empty_output = [0_u8; 0];
    let reset = reset(&mut engine, &mut empty_output, 0).expect("engine should reset through the trait");
    let finished = finish(&mut engine, &mut empty_output, 0).expect("engine should finish through the trait");

    assert_eq!(0, reset);
    assert_eq!(0, finished);
}

#[test]
fn test_buffered_convert_engine_default_builds_engine() {
    let mut engine =
        TranscodeConvertEngine::<SourceCodec, TargetCodec, StrictDecodeHooks, StrictEncodeHooks>::default();
    engine
        .reset(&mut [], 0)
        .expect("reset before default converter transcode");
    let mut output = [0_u8; 1];

    let progress = engine
        .transcode(&[8], 0, &mut output, 0)
        .expect("default components should convert one value");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((1, 1), (progress.read(), progress.written()));
    assert_eq!([9], output);

    engine.reset(&mut [], 0).expect("reset");
}

#[test]
fn test_buffered_convert_engine_new_uses_supplied_components() {
    let mut engine = TranscodeConvertEngine::new(SourceCodec, TargetCodec, StrictDecodeHooks, StrictEncodeHooks);
    let mut reset_output = [];
    engine.reset(&mut reset_output, 0).expect("initialize stream");

    let mut output = [0_u8; 1];

    let progress = engine
        .transcode(&[6], 0, &mut output, 0)
        .expect("supplied components should convert one value");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((1, 1), (progress.read(), progress.written()));
    assert_eq!([7], output);
}

#[test]
fn test_buffered_convert_engine_new_uses_supplied_policy_hooks() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        FactoryDecodeHooks { marker: 11 },
        FactoryEncodeHooks { offset: 7 },
    );
    let mut reset_output = [];
    engine.reset(&mut reset_output, 0).expect("initialize stream");

    assert_eq!(Ok(12), engine.max_transcode_output_len(1));

    let mut output = [0_u8; 1];
    let progress = engine
        .transcode(&[1], 0, &mut output, 0)
        .expect("supplied encode hooks should convert the value");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((1, 1), (progress.read(), progress.written()));
    assert_eq!([2], output);

    engine.reset(&mut [], 0).expect("reset");
}

#[test]
fn test_buffered_convert_engine_owns_pending_value_between_calls() {
    let mut engine = new_copy_engine();
    let mut empty_output = [0_u8; 0];

    let progress = engine
        .transcode(&[1], 0, &mut empty_output, 0)
        .expect("conversion should retain decoded value when output is empty");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            required: crate::nonzero(1),
        },
        progress.status(),
    );
    assert_eq!((1, 0), (progress.read(), progress.written()));
    assert_eq!(Ok(2), engine.max_transcode_output_len(1));
    assert_eq!(Ok(1), engine.max_finish_output_len());

    let mut output = [0_u8; 2];
    let progress = engine
        .transcode(&[9], 0, &mut output, 0)
        .expect("conversion should drain pending before reading new input");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((1, 2), (progress.read(), progress.written()));
    assert_eq!([2, 10], output);
    assert_eq!(Ok(1), engine.max_finish_output_len());
}

#[test]
fn test_buffered_convert_engine_preserves_pending_value_before_encode_hook_capacity() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        StrictDecodeHooks,
        CountedTargetReplacementHooks::default(),
    );
    engine.reset(&mut [], 0).expect("initialize stream");

    let progress = engine
        .transcode(&[98], 0, &mut [], 0)
        .expect("decoded value should remain pending under output pressure");
    assert_eq!(
        TranscodeStatus::NeedOutput {
            required: crate::nonzero(1)
        },
        progress.status()
    );
    assert_eq!(1, progress.read());
    assert_eq!(0, progress.written());
    assert_eq!(0, engine.encode_hooks().calls);

    let mut output = [0_u8; 1];
    let progress = engine
        .transcode(&[98], progress.read(), &mut output, 0)
        .expect("pending value should encode before new source input");
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(0, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([41], output);
    assert_eq!(1, engine.encode_hooks().calls);
}

#[test]
fn test_buffered_convert_engine_reports_pending_need_output_before_new_input() {
    let mut engine = new_copy_engine();
    let mut empty_output = [0_u8; 0];

    let progress = engine
        .transcode(&[1], 0, &mut empty_output, 0)
        .expect("conversion should retain decoded value when output is empty");
    assert_eq!(
        TranscodeStatus::NeedOutput {
            required: crate::nonzero(1),
        },
        progress.status(),
    );
    assert_eq!((1, 0), (progress.read(), progress.written()));

    let progress = engine
        .transcode(&[9], 0, &mut empty_output, 0)
        .expect("conversion should report pending output before reading new input");
    assert_eq!(
        TranscodeStatus::NeedOutput {
            required: crate::nonzero(1),
        },
        progress.status(),
    );
    assert_eq!((0, 0), (progress.read(), progress.written()));
    assert_eq!(Ok(2), engine.max_transcode_output_len(1));

    let mut output = [0_u8; 2];
    let progress = engine
        .transcode(&[9], 0, &mut output, 0)
        .expect("conversion should keep pending value after repeated output starvation");
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((1, 2), (progress.read(), progress.written()));
    assert_eq!([2, 10], output);
}

#[test]
fn test_buffered_convert_engine_maps_pending_encode_error_before_new_input() {
    let mut engine = new_copy_engine();
    let progress = engine
        .transcode(&[0, 0, 12], 2, &mut [], 0)
        .expect("retain the value from original source index 2");
    assert!(progress.is_need_output());
    assert_eq!((1, 0), (progress.read(), progress.written()));

    let mut output = [0xa5_u8; 1];
    let error = engine
        .transcode(&[0, 1], 1, &mut output, 0)
        .expect_err("pending failure precedes the new input");

    assert!(matches!(
        error,
        TranscodeConvertError::EncodeDomain(codec::TranscodeDomainError::Main {
            source: EngineError::Encode,
            input_index: 2,
            input_consumed: None
        }),
    ));
    assert_eq!([0xa5], output, "failed pending encoding writes no output");
}

#[test]
fn test_buffered_convert_engine_reports_invalid_indices() {
    let mut engine = new_copy_engine();
    let mut output = [0_u8; 1];

    let error = engine
        .transcode(&[1], 2, &mut output, 0)
        .expect_err("invalid input index should fail");
    assert_eq!(
        codec::TranscodeConvertError::Failure(codec::TranscodeFailure::invalid_input_index(2, 1)),
        error,
    );

    let error = engine
        .transcode(&[1], 0, &mut output, 2)
        .expect_err("invalid output index should fail");
    assert_eq!(
        codec::TranscodeConvertError::Failure(codec::TranscodeFailure::invalid_output_index(2, 1)),
        error,
    );
}

#[test]
fn test_buffered_convert_engine_error_path_fixture_supports_successful_lifecycle() {
    let mut engine = new_error_path_engine(ErrorPathHooks::default());

    assert_eq!(Ok(2), engine.max_transcode_output_len(1));
    assert_eq!(Ok(1), engine.max_finish_output_len());
    assert_eq!(Ok(3), engine.max_total_output_len(1));
    assert_eq!(0, engine.reset(&mut [], 0).expect("reset should succeed"));

    let mut output = [0_u8; 1];
    let progress = engine
        .transcode(&[1], 0, &mut output, 0)
        .expect("conversion should succeed");
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((1, 1), (progress.read(), progress.written()));
    assert_eq!([2], output);

    assert_eq!(0, engine.finish(&mut [], 0).expect("finish should succeed"));
}

#[test]
fn test_buffered_convert_engine_reports_capacity_errors() {
    let engine = new_error_path_engine(ErrorPathHooks {
        decode_max_output_error: true,
        ..ErrorPathHooks::default()
    });
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_transcode_output_len(1)
    );
    assert_eq!(Err(CapacityError::OutputLengthOverflow), engine.max_total_output_len(1));

    let engine = new_error_path_engine(ErrorPathHooks {
        encode_max_output_error_for_len: Some(2),
        ..ErrorPathHooks::default()
    });
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_transcode_output_len(2)
    );

    let engine = new_error_path_engine(ErrorPathHooks {
        decode_finish_len: 2,
        encode_max_output_error_for_len: Some(2),
        ..ErrorPathHooks::default()
    });
    assert_eq!(Err(CapacityError::OutputLengthOverflow), engine.max_finish_output_len());

    let mut engine = new_error_path_engine(ErrorPathHooks {
        decode_finish_len: 2,
        encode_max_output_error_for_len: Some(2),
        ..ErrorPathHooks::default()
    });
    let mut reset_output = [];
    engine.reset(&mut reset_output, 0).expect("initialize stream");
    let error = engine
        .finish(&mut [], 0)
        .expect_err("encoding decoder finish values should report bound overflow");
    assert_eq!(
        codec::TranscodeConvertError::Failure(codec::TranscodeFailure::output_length_overflow()),
        error
    );

    let engine = new_error_path_engine(ErrorPathHooks {
        decode_finish_len: 1,
        encode_max_output_error: true,
        ..ErrorPathHooks::default()
    });
    assert_eq!(Err(CapacityError::OutputLengthOverflow), engine.max_finish_output_len());
    assert_eq!(Err(CapacityError::OutputLengthOverflow), engine.max_total_output_len(1));

    let engine = new_error_path_engine(ErrorPathHooks {
        encode_finish_len: usize::MAX,
        ..ErrorPathHooks::default()
    });
    assert_eq!(Err(CapacityError::OutputLengthOverflow), engine.max_total_output_len(1));

    let mut engine = new_error_path_engine(ErrorPathHooks {
        encode_max_output_error: true,
        ..ErrorPathHooks::default()
    });
    assert_eq!(Err(CapacityError::OutputLengthOverflow), engine.max_reset_output_len());
    let error = engine
        .reset(&mut [], 0)
        .expect_err("reset bound overflow should be mapped");
    assert_eq!(
        codec::TranscodeConvertError::Failure(codec::TranscodeFailure::output_length_overflow()),
        error
    );

    let mut engine = new_error_path_engine(ErrorPathHooks {
        encode_max_output_error_for_len: Some(1),
        ..ErrorPathHooks::default()
    });
    let mut reset_output = [];
    engine.reset(&mut reset_output, 0).expect("initialize stream");
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_transcode_output_len(1)
    );
    let progress = engine
        .transcode(&[1], 0, &mut [], 0)
        .expect("output pressure should be reported before encoding");
    assert!(matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));

    let mut empty_output = [0_u8; 0];
    let mut engine = new_error_path_engine(ErrorPathHooks {
        decode_finish_len: usize::MAX,
        ..ErrorPathHooks::default()
    });
    let mut reset_output = [];
    engine.reset(&mut reset_output, 0).expect("initialize stream");
    let progress = engine
        .transcode(&[1], 0, &mut empty_output, 0)
        .expect("conversion should retain pending value");
    assert!(matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));
    assert_eq!(Err(CapacityError::OutputLengthOverflow), engine.max_finish_output_len());
    assert_eq!(Err(CapacityError::OutputLengthOverflow), engine.max_total_output_len(0));
}

#[test]
fn test_buffered_convert_engine_finish_maps_encoder_finish_bound_overflow() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        FinishOverflowTargetCodec,
        StrictDecodeHooks,
        FinishOverflowEncodeHooks,
    );
    let mut reset_output = [];
    engine.reset(&mut reset_output, 0).expect("initialize stream");

    assert_eq!(Err(CapacityError::OutputLengthOverflow), engine.max_finish_output_len());

    let error = engine
        .finish(&mut [], 0)
        .expect_err("encoder finish bound overflow should be mapped");

    assert_eq!(
        codec::TranscodeConvertError::Failure(codec::TranscodeFailure::output_length_overflow()),
        error
    );
}

#[test]
fn test_buffered_convert_engine_finish_maps_initial_decode_finish_bound_overflow() {
    let mut engine = TranscodeConvertEngine::new(
        FinishValueSourceCodec::<{ usize::MAX }>,
        TargetCodec,
        FixedFinishBoundDecodeHooks { finish_len: 1 },
        StrictEncodeHooks,
    );
    let mut reset_output = [];
    engine.reset(&mut reset_output, 0).expect("initialize stream");

    let mut output = [];

    assert_eq!(Err(CapacityError::OutputLengthOverflow), engine.max_finish_output_len(),);

    let error = engine
        .finish(&mut output, 0)
        .expect_err("overflowing decode finish bound should be mapped");

    assert_eq!(
        codec::TranscodeConvertError::Failure(codec::TranscodeFailure::output_length_overflow()),
        error
    );
}

#[test]
fn test_buffered_convert_engine_finish_maps_late_decode_finish_bound_overflow() {
    let mut engine = TranscodeConvertEngine::new(
        FinishValueSourceCodec::<1>,
        TargetCodec,
        ChangingFinishBoundDecodeHooks { calls: Cell::new(0) },
        StrictEncodeHooks,
    );
    let mut reset_output = [];
    engine.reset(&mut reset_output, 0).expect("initialize stream");

    let mut output = [0_u8; 1];

    let error = engine
        .finish(&mut output, 0)
        .expect_err("late decode finish bound overflow should be mapped");

    assert_eq!(
        codec::TranscodeConvertError::Failure(codec::TranscodeFailure::output_length_overflow()),
        error
    );
}

#[test]
fn test_buffered_convert_engine_maps_encode_value_error() {
    let mut engine = new_error_path_engine(ErrorPathHooks {
        decode_finish: ErrorPathDecodeFinish::Normal,
        encode_mode: ErrorPathEncodeMode::PrepareError,
        ..ErrorPathHooks::default()
    });
    let mut reset_output = [];
    engine.reset(&mut reset_output, 0).expect("initialize stream");
    let mut output = [0_u8; 1];

    let error = engine
        .transcode(&[98], 0, &mut output, 0)
        .expect_err("encode value error should be mapped through convert hooks");

    assert!(matches!(
        error,
        TranscodeConvertError::EncodeDomain(codec::TranscodeDomainError::Main {
            source: EngineError::Encode,
            input_index: _,
            input_consumed: None
        }),
    ));
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_finish_reports_output_index_beyond_buffer() {
    let mut engine = new_copy_engine();
    let mut output = [];

    let error = engine
        .finish(&mut output, 1)
        .expect_err("out-of-range finish output index should be rejected");

    assert_eq!(
        codec::TranscodeConvertError::Failure(codec::TranscodeFailure::invalid_output_index(1, 0)),
        error,
    );
}

#[test]
fn test_buffered_convert_engine_finish_maps_decode_error() {
    let mut engine = new_error_path_engine(ErrorPathHooks {
        decode_finish: ErrorPathDecodeFinish::Error,
        encode_mode: ErrorPathEncodeMode::Normal,
        ..ErrorPathHooks::default()
    });
    let mut reset_output = [];
    engine.reset(&mut reset_output, 0).expect("initialize stream");
    let mut output = [0_u8; 1];

    let error = engine
        .finish(&mut output, 0)
        .expect_err("decode finish error should be mapped through convert hooks");

    assert!(matches!(
        error,
        TranscodeConvertError::DecodeDomain(codec::TranscodeDomainError::Finish {
            source: EngineError::Decode,
        }),
    ));
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_finish_runs_zero_bound_decoder_teardown() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        FinishDecodeHooks {
            value: None,
            finish_bound: 0,
            fail: false,
        },
        StrictEncodeHooks,
    );
    let mut reset_output = [];
    engine.reset(&mut reset_output, 0).expect("initialize stream");

    assert_eq!(0, engine.finish(&mut [], 0).expect("finish should succeed"));
}

#[test]
fn test_buffered_convert_engine_finish_maps_configured_decoder_teardown_error() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        FinishDecodeHooks {
            value: None,
            finish_bound: 0,
            fail: true,
        },
        StrictEncodeHooks,
    );
    let mut reset_output = [];
    engine.reset(&mut reset_output, 0).expect("initialize stream");

    let error = engine
        .finish(&mut [], 0)
        .expect_err("decoder teardown errors should be mapped");

    assert!(matches!(
        error,
        TranscodeConvertError::DecodeDomain(codec::TranscodeDomainError::Finish {
            source: EngineError::Decode,
        }),
    ));
}

#[test]
fn test_buffered_convert_engine_finish_maps_encode_error() {
    let mut engine = new_error_path_engine(ErrorPathHooks {
        decode_finish: ErrorPathDecodeFinish::Normal,
        encode_mode: ErrorPathEncodeMode::FinishError,
        ..ErrorPathHooks::default()
    });
    let mut reset_output = [];
    engine.reset(&mut reset_output, 0).expect("initialize stream");
    let mut output = [0_u8; 1];

    let error = engine
        .finish(&mut output, 0)
        .expect_err("encode finish error should be mapped through convert hooks");

    assert!(matches!(
        error,
        TranscodeConvertError::EncodeDomain(codec::TranscodeDomainError::Finish {
            source: EngineError::Encode,
        }),
    ));
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_finish_maps_pending_encode_error() {
    let mut engine = new_copy_engine();
    let progress = engine
        .transcode(&[0, 0, 12], 2, &mut [], 0)
        .expect("retain the value from original source index 2");
    assert!(progress.is_need_output());
    assert_eq!((1, 0), (progress.read(), progress.written()));

    let mut output = [0xa5_u8; 1];
    let error = engine
        .finish(&mut output, 0)
        .expect_err("finish should map pending encode error");

    assert!(matches!(
        error,
        TranscodeConvertError::EncodeDomain(codec::TranscodeDomainError::Main {
            source: EngineError::Encode,
            input_index: 2,
            input_consumed: None,
        }),
    ));
    assert_eq!([0xa5], output, "failed pending encoding writes no output");
}

#[test]
fn test_buffered_convert_engine_finish_maps_decoder_output_encode_error() {
    let mut engine = new_finish_engine(FinishHooks { value: 13 });
    let mut output = [0_u8; 1];

    let error = engine
        .finish(&mut output, 0)
        .expect_err("finish should map encode errors for decoder-emitted values");

    assert!(matches!(
        error,
        TranscodeConvertError::EncodeDomain(codec::TranscodeDomainError::Main {
            source: EngineError::Encode,
            input_index: 0,
            input_consumed: None
        }),
    ));
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_applies_decode_policy_skip() {
    let mut engine = new_repair_engine(RepairAction::Skip);
    let mut output = [0_u8; 1];

    let progress = engine
        .transcode(&[1, 2], 0, &mut output, 0)
        .expect("skip policy should consume invalid source units");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((2, 0), (progress.read(), progress.written()));
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_applies_decode_policy_emit() {
    let mut engine = new_repair_engine(RepairAction::Emit);
    let mut output = [0_u8; 2];

    let progress = engine
        .transcode(&[1, 2], 0, &mut output, 0)
        .expect("emit policy should replace invalid source units");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((2, 2), (progress.read(), progress.written()));
    assert_eq!([42, 42], output);
}

#[test]
fn test_buffered_convert_engine_finish_drains_pending_value() {
    let mut engine = new_copy_engine();
    let mut empty_output = [0_u8; 0];
    let progress = engine
        .transcode(&[4], 0, &mut empty_output, 0)
        .expect("conversion should retain decoded value");
    assert!(matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));

    let error = engine
        .finish(&mut empty_output, 0)
        .expect_err("finish should reject insufficient output before draining pending value");
    assert_eq!(
        codec::TranscodeConvertError::Failure(codec::TranscodeFailure::insufficient_output(0, 1, 0,)),
        error,
    );
    assert_eq!(Ok(1), engine.max_finish_output_len());

    let mut output = [0_u8; 1];
    let written = engine
        .finish(&mut output, 0)
        .expect("finish should write pending value");
    assert_eq!(1, written);
    assert_eq!([5], output);
}

#[test]
fn test_buffered_convert_engine_finish_encodes_decoder_finish_output() {
    let mut engine = new_finish_engine(FinishHooks::default());
    assert_eq!(Ok(2), engine.max_finish_output_len());

    let mut empty_output = [0_u8; 0];
    let error = engine
        .finish(&mut empty_output, 0)
        .expect_err("finish should reject insufficient output before decoder finish");
    assert_eq!(
        codec::TranscodeConvertError::Failure(codec::TranscodeFailure::insufficient_output(0, 1, 0,)),
        error,
    );
    assert_eq!(Ok(2), engine.max_finish_output_len());

    let mut output = [0_u8; 1];
    let written = engine
        .finish(&mut output, 0)
        .expect("finish should encode decoder finish value");
    assert_eq!(1, written);
    assert_eq!([40], output);
    assert_eq!(Ok(2), engine.max_finish_output_len());
}

#[test]
fn test_buffered_convert_engine_finish_drains_decoder_finish_batches() {
    let mut engine = new_batch_finish_engine();
    assert_eq!(Ok(3), engine.max_finish_output_len());

    let mut output = [0_u8; 2];
    let written = engine
        .finish(&mut output, 0)
        .expect("finish should keep draining decoder finish batches");

    assert_eq!(2, written);
    assert_eq!([50, 51], output);
    assert_eq!(Ok(3), engine.max_finish_output_len());
}

#[test]
fn test_buffered_convert_engine_finish_drains_pending_before_decoder_finish_output() {
    let mut engine = new_finish_engine(FinishHooks::default());
    let mut empty_output = [0_u8; 0];
    let progress = engine
        .transcode(&[4], 0, &mut empty_output, 0)
        .expect("conversion should retain decoded input value");
    assert!(matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));
    assert_eq!(Ok(2), engine.max_finish_output_len());

    let mut output = [0_u8; 1];
    let error = engine
        .finish(&mut output, 0)
        .expect_err("finish should reject partial one-shot output");
    assert_eq!(
        codec::TranscodeConvertError::Failure(codec::TranscodeFailure::insufficient_output(0, 2, 1,)),
        error,
    );
    assert_eq!([0], output);
    assert_eq!(Ok(2), engine.max_finish_output_len());

    let mut output = [0_u8; 2];
    let written = engine
        .finish(&mut output, 0)
        .expect("finish should write pending input value before decoder finish value");
    assert_eq!(2, written);
    assert_eq!([5, 40], output);
}

#[test]
fn test_buffered_convert_engine_finish_delegates_to_encoder_finish() {
    let mut engine = new_finish_encode_engine();
    assert_eq!(Ok(2), engine.max_finish_output_len());

    let mut empty_output = [0_u8; 0];
    let error = engine
        .finish(&mut empty_output, 0)
        .expect_err("target finish hook should require one-shot output capacity");
    assert_eq!(
        codec::TranscodeConvertError::Failure(codec::TranscodeFailure::insufficient_output(0, 1, 0,)),
        error,
    );

    let mut output = [0_u8; 1];
    let written = engine
        .finish(&mut output, 0)
        .expect("target finish hook should write final output");
    assert_eq!(1, written);
    assert_eq!([0xee], output);
    assert_eq!(Ok(2), engine.max_finish_output_len());
}

#[test]
fn test_buffered_convert_engine_reset_emits_target_reset_output() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        ResetEmittingTargetCodec,
        StrictDecodeHooks,
        ResetTargetHooks,
    );
    let mut output = [0_u8; 1];

    let written = engine
        .reset(&mut output, 0)
        .expect("reset should emit target reset units");

    assert_eq!(1, written);
    assert_eq!([0xaa], output);
    assert_eq!(Ok(1), engine.max_reset_output_len());
}

#[test]
fn test_buffered_convert_engine_reset_calls_decode_reset_hooks() {
    let called = Rc::new(Cell::new(false));
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        ResetObservingDecodeHooks { called: called.clone() },
        StrictEncodeHooks,
    );

    engine.reset(&mut [], 0).expect("reset should succeed");

    assert!(called.get(), "decode before_reset should be called");
}

#[test]
fn test_buffered_convert_engine_reset_maps_target_reset_errors() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        ResetFailTargetCodec,
        StrictDecodeHooks,
        ResetFailTargetHooks,
    );
    let mut output = [0_u8; 1];

    let error = engine
        .reset(&mut output, 0)
        .expect_err("target reset errors should be mapped through convert hooks");

    assert!(matches!(
        error,
        TranscodeConvertError::EncodeDomain(codec::TranscodeDomainError::Reset {
            source: TargetResetFailError,
            ..
        }),
    ));
}

#[test]
fn test_buffered_convert_engine_reset_rejects_invalid_output_index() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        ResetEmittingTargetCodec,
        StrictDecodeHooks,
        ResetTargetHooks,
    );
    let mut output = [0_u8; 1];

    let error = engine
        .reset(&mut output, 2)
        .expect_err("invalid reset output index should be rejected");

    assert_eq!(
        codec::TranscodeConvertError::Failure(codec::TranscodeFailure::invalid_output_index(2, 1)),
        error,
    );
}

#[test]
fn test_buffered_convert_engine_reset_with_stateless_decode_reset_values() {
    let called = Rc::new(Cell::new(false));
    let mut engine = TranscodeConvertEngine::new(
        StatelessResetSourceCodec,
        TargetCodec,
        StatelessResetSourceDecodeHooks { called: called.clone() },
        StrictEncodeHooks,
    );

    assert_eq!(Ok(0), engine.max_reset_output_len());

    let written = engine
        .reset(&mut [], 0)
        .expect("stateless decode reset should clear state without output");

    assert_eq!(0, written);
    assert!(
        called.get(),
        "stateless decode reset should still invoke decode-side reset hooks",
    );
}

#[test]
fn test_buffered_convert_engine_reset_maps_stateless_decoder_reset_error() {
    let mut engine = TranscodeConvertEngine::new(
        StatelessResetFailingSourceCodec,
        TargetCodec,
        StatelessResetFailingSourceDecodeHooks,
        StrictEncodeHooks,
    );

    let error = engine
        .reset(&mut [], 0)
        .expect_err("stateless decoder reset errors should be mapped");

    assert!(matches!(
        error,
        TranscodeConvertError::DecodeDomain(codec::TranscodeDomainError::Reset {
            source: EngineError::Decode
        }),
    ));
}

#[test]
fn test_buffered_convert_engine_max_reset_output_len_overflow() {
    let engine = TranscodeConvertEngine::new(
        OverflowResetSourceCodec,
        OverflowResetTargetCodec,
        OverflowResetSourceDecodeHooks,
        OverflowResetTargetEncodeHooks,
    );

    let error = engine
        .max_reset_output_len()
        .expect_err("max reset bound should report overflow");

    assert_eq!(CapacityError::OutputLengthOverflow, error);
    assert_eq!(Err(CapacityError::OutputLengthOverflow), engine.max_total_output_len(0));
}

#[test]
#[should_panic(expected = "converter reset bound must reserve space for decode reset values")]
fn test_buffered_convert_engine_reset_reaches_unreachable_decode_reset_path() {
    let mut engine = TranscodeConvertEngine::new(
        ResetEmittingSourceCodec,
        MismatchCapacityTargetCodec,
        MismatchCapacityResetEmittingDecodeHooks,
        MismatchCapacityEncodeHooks,
    );
    let mut output = [0_u8; 1];

    let _ = engine.reset(&mut output, 0);
}

#[test]
fn test_buffered_convert_engine_invalid_reset_preserves_pending_value() {
    let mut engine = new_copy_engine();
    let mut empty_output = [0_u8; 0];
    let progress = engine
        .transcode(&[4], 0, &mut empty_output, 0)
        .expect("conversion should retain decoded value");
    assert!(matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));
    assert_eq!(Ok(1), engine.max_finish_output_len());

    let mut output = [0_u8; 1];
    let error = engine
        .reset(&mut output, 2)
        .expect_err("invalid reset output index should be rejected");

    assert_eq!(
        codec::TranscodeConvertError::Failure(codec::TranscodeFailure::invalid_output_index(2, 1)),
        error,
    );
    assert_eq!(Ok(1), engine.max_finish_output_len());

    let written = engine
        .finish(&mut output, 0)
        .expect("invalid reset must not discard pending value");
    assert_eq!(1, written);
    assert_eq!([5], output);
}

#[test]
#[should_panic(expected = "converter finish bound must reserve space for pending values")]
fn test_buffered_convert_engine_finish_hits_pending_unreachable() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        MismatchCapacityTargetCodec,
        StrictDecodeHooks,
        MismatchCapacityEncodeHooks,
    );
    let mut reset_output = [];
    engine.reset(&mut reset_output, 0).expect("initialize stream");

    let mut empty_output = [0_u8; 0];

    let _ = engine.transcode(&[1], 0, &mut empty_output, 0);
    let required = engine.max_finish_output_len().expect("finish bound");
    let mut output = vec![0_u8; required];
    let _ = engine.finish(&mut output, 0);
}

#[test]
#[should_panic(expected = "converter finish bound must reserve space for decode finish values")]
fn test_buffered_convert_engine_finish_hits_decoder_finish_unreachable() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        MismatchCapacityTargetCodec,
        FinishDecodeHooks::default(),
        MismatchCapacityEncodeHooks,
    );
    let mut reset_output = [];
    engine.reset(&mut reset_output, 0).expect("initialize stream");

    let required = engine.max_finish_output_len().expect("finish bound");
    let mut output = vec![0_u8; required];
    let _ = engine.finish(&mut output, 0);
}

#[test]
fn test_buffered_convert_engine_maps_decode_policy_domain_error() {
    let mut engine = TranscodeConvertEngine::new(
        ErrorSourceCodec,
        TargetCodec,
        ImpossibleFailureDecodeHooks,
        StrictEncodeHooks,
    );
    let mut reset_output = [];
    engine.reset(&mut reset_output, 0).expect("initialize stream");

    let mut output = [0_u8; 1];

    let error = engine
        .transcode(&[1], 0, &mut output, 0)
        .expect_err("decode policy domain errors should map to decode side");

    assert_eq!(TranscodeConvertError::decode_domain_main(EngineError::Decode, 0), error);
}

#[test]
fn test_buffered_convert_engine_reset_includes_decode_reset_in_bound() {
    let engine = TranscodeConvertEngine::new(
        ResetEmittingSourceCodec,
        TargetCodec,
        ResetSourceDecodeHooks,
        StrictEncodeHooks,
    );

    let bound = engine
        .max_reset_output_len()
        .expect("reset bound should be representable");
    assert!(
        bound >= 1,
        "decode-reset value must be reserved in reset bound, got {bound}",
    );
}

#[test]
fn test_buffered_convert_engine_reset_pipes_decode_reset_values_into_encoder() {
    let mut engine = TranscodeConvertEngine::new(
        ResetEmittingSourceCodec,
        TargetCodec,
        ResetSourceDecodeHooks,
        StrictEncodeHooks,
    );

    let required = engine
        .max_reset_output_len()
        .expect("reset bound should be representable");
    let mut output = vec![0_u8; required];

    let written = engine
        .reset(&mut output, 0)
        .expect("reset should succeed when the source emits a reset value");

    assert!(
        written >= 1,
        "reset must report the encoded decode-reset value, got {written}",
    );
    assert_eq!(
        SOURCE_RESET_SENTINEL, output[0],
        "decode-reset value must be encoded into the target output",
    );
}

#[test]
fn test_buffered_convert_engine_resets_target_before_encoding_source_reset_values() {
    let mut engine = TranscodeConvertEngine::new(
        ResetEmittingSourceCodec,
        ResetDependentTargetCodec::default(),
        ResetSourceDecodeHooks,
        StrictEncodeHooks,
    );
    let mut output = [0_u8; 2];

    let written = engine
        .reset(&mut output, 0)
        .expect("target reset must run before source-reset values are encoded");

    assert_eq!(2, written);
    assert_eq!([0xaa, SOURCE_RESET_SENTINEL], output);
}

#[test]
fn test_buffered_convert_engine_reset_maps_decode_reset_value_encode_error() {
    let mut engine = TranscodeConvertEngine::new(
        ResetEncodingFailSourceCodec,
        TargetCodec,
        ResetEncodingFailSourceDecodeHooks,
        StrictEncodeHooks,
    );
    let mut output = [0_u8; 1];

    let error = engine
        .reset(&mut output, 0)
        .expect_err("errors while encoding decode-reset values should be mapped");

    assert!(matches!(
        error,
        TranscodeConvertError::EncodeDomain(codec::TranscodeDomainError::Main {
            source: EngineError::Encode,
            input_index: 0,
            input_consumed: None,
        }),
    ));
}

#[test]
fn test_buffered_convert_engine_reset_maps_decoder_reset_error() {
    let mut engine = TranscodeConvertEngine::new(
        ResetFailingSourceCodec,
        TargetCodec,
        ResetFailSourceDecodeHooks,
        StrictEncodeHooks,
    );
    let mut output = [0_u8; 1];

    let error = engine
        .reset(&mut output, 0)
        .expect_err("decoder reset errors should be mapped through convert hooks");

    assert!(matches!(
        error,
        TranscodeConvertError::DecodeDomain(codec::TranscodeDomainError::Reset {
            source: EngineError::Decode,
        }),
    ));
}

#[test]
fn test_buffered_convert_engine_finish_maps_decoder_finish_error() {
    let mut engine = TranscodeConvertEngine::new(
        FinishFailingSourceCodec,
        TargetCodec,
        FinishFailSourceDecodeHooks,
        StrictEncodeHooks,
    );
    let mut reset_output = [];
    engine.reset(&mut reset_output, 0).expect("initialize stream");

    let mut output = [0_u8; 1];

    let error = engine
        .finish(&mut output, 0)
        .expect_err("decoder finish error should be mapped through convert hooks");

    assert!(matches!(
        error,
        TranscodeConvertError::DecodeDomain(codec::TranscodeDomainError::Finish {
            source: EngineError::Decode,
        }),
    ));
}

// ============================================================================
// Lifecycle guard wiring
// ============================================================================

#[test]
fn test_buffered_convert_engine_lifecycle_rejects_double_finish() {
    let mut engine = new_copy_engine();
    let mut output = [0_u8; 0];
    engine
        .finish(&mut output, 0)
        .expect("first finish should succeed for a stateless converter");
    assert_eq!(
        Err(TranscodeConvertError::Failure(TranscodeFailure::FinishAfterFinish,)),
        engine.finish(&mut output, 0),
    );
}

#[test]
fn test_buffered_convert_engine_lifecycle_rejects_transcode_after_finish() {
    let mut engine = new_copy_engine();
    let mut output = [0_u8; 1];
    engine.finish(&mut output, 0).expect("finish closes the logical stream");
    assert_eq!(
        Err(TranscodeConvertError::Failure(TranscodeFailure::TranscodeAfterFinish,)),
        engine.transcode(&[1_u8], 0, &mut output, 0),
    );
}

#[test]
fn test_buffered_convert_engine_lifecycle_allows_reuse_after_reset() {
    let mut engine = new_copy_engine();
    let mut output = [0_u8; 2];
    engine.finish(&mut output, 0).expect("first logical stream finalizes");
    engine.reset(&mut output, 0).expect("reset reopens the engine");
    let progress = engine
        .transcode(&[1_u8], 0, &mut output, 0)
        .expect("transcode after reset");
    assert_eq!(1, progress.read());
    engine.finish(&mut output, 1).expect("second logical stream finalizes");
}

#[test]
fn test_buffered_convert_engine_failed_reset_preserves_finished_state() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        ResetEmittingTargetCodec,
        StrictDecodeHooks,
        ResetTargetHooks,
    );
    let mut reset_output = [0_u8; 1];
    engine.reset(&mut reset_output, 0).expect("initialize stream");

    let mut output = [0_u8; 1];
    engine.finish(&mut output, 0).expect("finish closes the logical stream");

    let error = engine
        .reset(&mut [], 0)
        .expect_err("reset should reject insufficient output");
    assert_eq!(
        codec::TranscodeConvertError::Failure(codec::TranscodeFailure::insufficient_output(0, 1, 0,)),
        error
    );
    assert_eq!(
        Err(TranscodeConvertError::Failure(TranscodeFailure::TranscodeAfterFinish,)),
        engine.transcode(&[1_u8], 0, &mut output, 0),
    );
}

#[test]
fn test_buffered_convert_engine_partial_reset_failure_poisoned() {
    let mut engine = TranscodeConvertEngine::new(
        ResetFailingSourceCodec,
        ResetEmittingTargetCodec,
        ResetFailSourceDecodeHooks,
        ResetTargetHooks,
    );
    let mut output = [0_u8; 2];
    let _ = engine
        .reset(&mut output, 0)
        .expect_err("source reset should fail after target reset succeeds");
    assert_eq!(0xaa, output[0]);
    assert_eq!(
        Err(TranscodeConvertError::Failure(TranscodeFailure::LifecyclePoisoned,)),
        engine.transcode(&[1_u8], 0, &mut output, 0),
    );
}

#[test]
fn test_buffered_convert_engine_failed_finish_poisoned_until_reset() {
    let mut engine = TranscodeConvertEngine::new(
        FinishFailingSourceCodec,
        TargetCodec,
        FinishFailSourceDecodeHooks,
        StrictEncodeHooks,
    );
    let mut reset_output = [];
    engine.reset(&mut reset_output, 0).expect("initialize stream");

    let mut output = [0_u8; 1];
    let _ = engine.finish(&mut output, 0).expect_err("source finish should fail");
    assert_eq!(
        Err(TranscodeConvertError::Failure(TranscodeFailure::LifecyclePoisoned,)),
        engine.finish(&mut output, 0),
    );
    assert_eq!(
        Err(TranscodeConvertError::Failure(TranscodeFailure::LifecyclePoisoned,)),
        engine.transcode(&[1_u8], 0, &mut output, 0),
    );

    engine
        .reset(&mut output, 0)
        .expect("successful reset should recover a poisoned converter");
    let progress = engine
        .transcode(&[1_u8], 0, &mut output, 0)
        .expect("transcode should resume after successful reset");
    assert_eq!(1, progress.read());
}

#[test]
fn test_buffered_convert_engine_reports_complete_stream_bound() {
    let engine = new_copy_engine();

    assert_eq!(Ok(4), engine.max_total_output_len(2));
}

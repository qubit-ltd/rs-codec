// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec as codec;
use qubit_codec::CapacityError;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeProgress;
use qubit_codec::TranscodeStatus;
use qubit_codec::Transcoder;
use qubit_utils as utils_crate;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("domain error")]
struct DomainFailure;

macro_rules! infallible_transcoder_error {
    () => {
        type Error = TranscodeDecodeError<core::convert::Infallible>;
    };
}

#[derive(Default)]
struct MappingTranscoder;

impl Transcoder for MappingTranscoder {
    type Input = u8;
    type Output = u8;
    type Error = TranscodeDecodeError<DomainFailure>;

    fn max_transcode_output_len(&self, _input_len: usize) -> Result<usize, CapacityError> {
        Ok(usize::MAX)
    }

    fn reset(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn transcode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        Err(TranscodeDecodeError::domain_main(DomainFailure, 0))
    }

    fn finish(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

#[test]
fn test_transcoder_default_method_returns_fixed_transcode_error() {
    let mut transcoder = MappingTranscoder;
    let error = transcoder
        .transcode_complete_into(&[1], &mut [])
        .expect_err("overflow should be returned as transcode error");

    assert_eq!(
        codec::TranscodeDecodeError::Failure(codec::TranscodeFailure::insufficient_output(0, usize::MAX, 0)),
        error
    );
}

#[derive(Default)]
struct CopyTranscoder;

impl Transcoder for CopyTranscoder {
    type Input = u8;
    type Output = u8;
    infallible_transcoder_error!();

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let mut read = 0;
        let mut written = 0;
        while input_index + read < input.len() && output_index + written < output.len() {
            output[output_index + written] = input[input_index + read];
            read += 1;
            written += 1;
        }
        if input_index + read == input.len() {
            Ok(TranscodeProgress::complete(read, written))
        } else {
            let status = TranscodeStatus::NeedOutput {
                required: crate::nonzero(1),
            };
            Ok(TranscodeProgress::new(status, read, written))
        }
    }

    fn finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }
}

#[derive(Default)]
struct FinishingTranscoder {
    suffix_index: usize,
}

impl Transcoder for FinishingTranscoder {
    type Input = u8;
    type Output = u8;
    infallible_transcoder_error!();

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(2)
    }

    fn reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        self.suffix_index = 0;
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        CopyTranscoder.transcode(input, input_index, output, output_index)
    }

    fn finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        let suffix = *b"!\n";
        let required = suffix.len() - self.suffix_index;
        codec::TranscodeFailure::ensure_output_capacity(output.len(), output_index, required)?;
        let mut written = 0;
        while self.suffix_index < suffix.len() {
            output[output_index + written] = suffix[self.suffix_index];
            self.suffix_index += 1;
            written += 1;
        }
        Ok(written)
    }
}

#[derive(Default)]
struct PreflightTranscoder {
    reset_calls: usize,
    reset_bound_fails: bool,
}

impl Transcoder for PreflightTranscoder {
    type Input = u8;
    type Output = u8;
    infallible_transcoder_error!();

    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        if self.reset_bound_fails {
            Err(CapacityError::OutputLengthOverflow)
        } else {
            Ok(1)
        }
    }

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_capacity(output.len(), output_index, 1)?;
        self.reset_calls += 1;
        output[output_index] = b'^';
        Ok(1)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        CopyTranscoder.transcode(input, input_index, output, output_index)
    }

    fn finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }
}

#[derive(Default)]
struct PairTranscoder;

impl Transcoder for PairTranscoder {
    type Input = u8;
    type Output = u8;
    infallible_transcoder_error!();

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len / 2)
    }

    fn reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let available = input.len() - input_index;
        if !available.is_multiple_of(2) {
            let complete_len = available - 1;
            for i in 0..complete_len / 2 {
                output[output_index + i] = input[input_index + i * 2] ^ input[input_index + i * 2 + 1];
            }
            return Ok(TranscodeProgress::new(
                TranscodeStatus::NeedInput {
                    required: crate::nonzero(2),
                },
                complete_len,
                complete_len / 2,
            ));
        }
        for i in 0..available / 2 {
            output[output_index + i] = input[input_index + i * 2] ^ input[input_index + i * 2 + 1];
        }
        Ok(TranscodeProgress::complete(available, available / 2))
    }

    fn finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }
}

#[derive(Default)]
struct NeedInputEofTranscoder;

impl Transcoder for NeedInputEofTranscoder {
    type Input = u8;
    type Output = u8;
    infallible_transcoder_error!();

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        let available = input.len() - input_index;
        if available == 0 {
            return Ok(TranscodeProgress::complete(0, 0));
        }
        output[output_index] = input[input_index];
        Ok(TranscodeProgress::new(
            TranscodeStatus::NeedInput {
                required: utils_crate::nonzero(2),
            },
            1,
            1,
        ))
    }

    fn transcode_eof(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        self.transcode(input, input_index, output, output_index)
    }

    fn finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }
}

#[derive(Default)]
struct OverreadingEofTranscoder;

impl Transcoder for OverreadingEofTranscoder {
    type Input = u8;
    type Output = u8;
    infallible_transcoder_error!();

    fn max_transcode_output_len(&self, _input_len: usize) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }

    fn transcode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        Ok(TranscodeProgress::need_input(utils_crate::nonzero(2), 2, 0))
    }

    fn finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }
}

#[derive(Default)]
struct UnderestimatingTranscoder;

impl Transcoder for UnderestimatingTranscoder {
    type Input = u8;
    type Output = u8;
    infallible_transcoder_error!();

    fn max_transcode_output_len(&self, _input_len: usize) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        CopyTranscoder.transcode(input, input_index, output, output_index)
    }

    fn finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }
}

#[derive(Default)]
struct PartialCompleteTranscoder;

impl Transcoder for PartialCompleteTranscoder {
    type Input = u8;
    type Output = u8;
    infallible_transcoder_error!();

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        codec::TranscodeFailure::ensure_transcode_indices(input.len(), input_index, output.len(), output_index)?;
        output[output_index] = input[input_index];
        Ok(TranscodeProgress::complete(1, 1))
    }

    fn finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }
}

#[derive(Default)]
struct OverreportingCompleteTranscoder;

impl Transcoder for OverreportingCompleteTranscoder {
    type Input = u8;
    type Output = u8;
    infallible_transcoder_error!();

    fn max_transcode_output_len(&self, _input_len: usize) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }

    fn transcode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        Ok(TranscodeProgress::complete(0, 1))
    }

    fn finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }
}

#[derive(Default)]
struct OverreportingResetTranscoder;

impl Transcoder for OverreportingResetTranscoder {
    type Input = u8;
    type Output = u8;
    infallible_transcoder_error!();

    fn max_transcode_output_len(&self, _input_len: usize) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn reset(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(1)
    }

    fn transcode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }
}

#[derive(Default)]
struct OverreportingFinishTranscoder;

impl Transcoder for OverreportingFinishTranscoder {
    type Input = u8;
    type Output = u8;
    infallible_transcoder_error!();

    fn max_transcode_output_len(&self, _input_len: usize) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn reset(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn transcode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        Ok(TranscodeProgress::complete(0, 0))
    }

    fn finish(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(1)
    }
}

#[derive(Default)]
struct FinishBeyondAvailableTranscoder;

impl Transcoder for FinishBeyondAvailableTranscoder {
    type Input = u8;
    type Output = u8;
    infallible_transcoder_error!();

    fn max_transcode_output_len(&self, _input_len: usize) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }

    fn reset(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn transcode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        output[output_index] = 1;
        Ok(TranscodeProgress::complete(0, 1))
    }

    fn finish(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        Ok(1)
    }
}

#[derive(Default)]
struct OverflowBoundTranscoder;

impl Transcoder for OverflowBoundTranscoder {
    type Input = u8;
    type Output = u8;
    infallible_transcoder_error!();

    fn max_transcode_output_len(&self, _input_len: usize) -> Result<usize, CapacityError> {
        Ok(usize::MAX)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }

    fn reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        Ok(0)
    }

    fn transcode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        unreachable!("capacity overflow happens before transcode")
    }

    fn finish(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::Error> {
        unreachable!("capacity overflow happens before finish")
    }
}

#[derive(Clone, Copy)]
enum FailurePoint {
    ResetBound,
    TranscodeBound,
    FinishBound,
    SumBound,
    Reset,
    Transcode,
    Finish,
}

struct FailingTranscoder {
    failure: FailurePoint,
}

impl Transcoder for FailingTranscoder {
    type Input = u8;
    type Output = u8;
    type Error = TranscodeDecodeError<&'static str>;

    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        if matches!(self.failure, FailurePoint::ResetBound) {
            Err(CapacityError::OutputLengthOverflow)
        } else if matches!(self.failure, FailurePoint::SumBound) {
            Ok(usize::MAX)
        } else {
            Ok(0)
        }
    }

    fn max_transcode_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        if matches!(self.failure, FailurePoint::TranscodeBound) {
            Err(CapacityError::OutputLengthOverflow)
        } else {
            Ok(input_len)
        }
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        if matches!(self.failure, FailurePoint::FinishBound) {
            Err(CapacityError::OutputLengthOverflow)
        } else {
            Ok(0)
        }
    }

    fn reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        if matches!(self.failure, FailurePoint::Reset) {
            Err(TranscodeDecodeError::domain_reset("reset"))
        } else {
            Ok(0)
        }
    }

    fn transcode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if matches!(self.failure, FailurePoint::Transcode) {
            Err(TranscodeDecodeError::domain_main("transcode", 0))
        } else {
            Ok(TranscodeProgress::complete(0, 0))
        }
    }

    fn finish(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::Error> {
        codec::TranscodeFailure::ensure_output_index(output.len(), output_index)?;
        if matches!(self.failure, FailurePoint::Finish) {
            Err(TranscodeDecodeError::domain_finish("finish"))
        } else {
            Ok(0)
        }
    }
}

#[test]
fn test_transcoder_error_is_domain_error_type() {
    fn assert_domain_error<T, Input, Output>()
    where
        T: Transcoder<Input = Input, Output = Output>,
    {
    }

    assert_domain_error::<CopyTranscoder, u8, u8>();
}

#[test]
fn test_transcoder_error_associated_type_matches_framework_error() {
    type ResetFn<T> = fn(&mut T, &mut [<T as Transcoder>::Output], usize) -> Result<usize, <T as Transcoder>::Error>;

    let reset: ResetFn<CopyTranscoder> = <CopyTranscoder as Transcoder>::reset;

    let mut transcoder = CopyTranscoder;
    let mut output = [];
    assert_eq!(Ok(0), reset(&mut transcoder, &mut output, 0));
}

#[test]
fn test_transcoder_contract_uses_absolute_indices_and_relative_progress() {
    let mut transcoder = CopyTranscoder;
    let mut output = [0_u8; 4];

    let progress = transcoder
        .transcode(b"abc", 1, &mut output, 2)
        .expect("infallible copy");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(2, progress.read());
    assert_eq!(2, progress.written());
    assert_eq!([0, 0, b'b', b'c'], output);
}

#[test]
fn test_transcoder_stateless_reset_and_finish_are_explicit_noops() {
    let mut transcoder = CopyTranscoder;
    let mut output = [0_u8; 1];

    assert_eq!(Ok(3), transcoder.max_transcode_output_len(3));
    assert_eq!(Ok(3), transcoder.max_total_output_len(3));
    assert_eq!(Ok(0), transcoder.max_finish_output_len());
    assert_eq!(Ok(0), transcoder.max_reset_output_len());

    Transcoder::reset(&mut transcoder, &mut output, 0).expect("reset is noop");
    let written = transcoder.finish(&mut output, 0).expect("finish is noop");

    assert_eq!(0, written);
    assert_eq!([0], output);
}

#[test]
fn test_transcoder_total_output_len_sums_reset_transcode_and_finish() {
    let transcoder = FinishingTranscoder::default();

    assert_eq!(Ok(5), transcoder.max_total_output_len(3));
}

/// Verifies that capacity bounds do not shrink with transient stream state.
#[test]
fn test_transcoder_capacity_bounds_are_global() {
    let mut transcoder = FinishingTranscoder::default();
    let mut output = [0_u8; 2];

    assert_eq!(Ok(2), transcoder.max_finish_output_len());
    assert_eq!(2, transcoder.finish(&mut output, 0).expect("finish should fit"));
    assert_eq!(Ok(2), transcoder.max_finish_output_len());
    assert_eq!(Ok(5), transcoder.max_total_output_len(3));
}

#[test]
fn test_transcoder_total_output_len_reports_component_errors() {
    for failure in [
        FailurePoint::ResetBound,
        FailurePoint::TranscodeBound,
        FailurePoint::FinishBound,
        FailurePoint::SumBound,
    ] {
        let transcoder = FailingTranscoder { failure };

        assert_eq!(
            Err(CapacityError::OutputLengthOverflow),
            transcoder.max_total_output_len(1),
        );
    }
}

/// Verifies that full-stream capacity is rejected before reset mutates state
/// or output.
#[test]
fn test_transcoder_transcode_complete_into_preflights_total_capacity_before_reset() {
    let mut transcoder = PreflightTranscoder::default();
    let original = [0xaa_u8; 3];
    let mut output = original;

    let error = transcoder
        .transcode_complete_into(b"abc", &mut output)
        .expect_err("complete output bound is four units");

    assert_eq!(0, transcoder.reset_calls);
    assert_eq!(original, output);
    assert_eq!(
        codec::TranscodeDecodeError::Failure(codec::TranscodeFailure::insufficient_output(0, 4, 3)),
        error,
    );
}

/// Verifies that reset-bound overflow is reported before reset is called.
#[test]
fn test_transcoder_transcode_complete_into_checks_reset_bound_before_reset() {
    let mut transcoder = PreflightTranscoder {
        reset_bound_fails: true,
        ..PreflightTranscoder::default()
    };
    let original = [0xaa_u8; 1];
    let mut output = original;

    let error = transcoder
        .transcode_complete_into(b"", &mut output)
        .expect_err("reset bound overflow should fail preflight");

    assert_eq!(0, transcoder.reset_calls);
    assert_eq!(original, output);
    assert_eq!(
        codec::TranscodeDecodeError::Failure(codec::TranscodeFailure::output_length_overflow()),
        error
    );
}

/// Verifies that the exact complete-stream bound admits the full lifecycle.
#[test]
fn test_transcoder_transcode_complete_into_accepts_exact_total_capacity() {
    let mut transcoder = PreflightTranscoder::default();
    let mut output = [0_u8; 4];

    let written = transcoder
        .transcode_complete_into(b"abc", &mut output)
        .expect("exact complete output bound should fit");

    assert_eq!(1, transcoder.reset_calls);
    assert_eq!(4, written);
    assert_eq!(b"^abc", &output);
}

#[test]
fn test_transcoder_transcode_complete_into_runs_reset_transcode_and_finish() {
    let mut transcoder = FinishingTranscoder::default();
    let mut output = [0_u8; 5];

    let written = transcoder
        .transcode_complete_into(b"abc", &mut output)
        .expect("complete transcode should fit");

    assert_eq!(5, written);
    assert_eq!(b"abc!\n", &output);
    assert_eq!(Ok(2), transcoder.max_finish_output_len());
}

#[test]
fn test_transcoder_transcode_complete_into_reports_stage_errors() {
    for (failure, expected) in [
        (FailurePoint::Reset, TranscodeDecodeError::domain_reset("reset")),
        (
            FailurePoint::TranscodeBound,
            codec::TranscodeDecodeError::Failure(codec::TranscodeFailure::output_length_overflow()),
        ),
        (
            FailurePoint::FinishBound,
            codec::TranscodeDecodeError::Failure(codec::TranscodeFailure::output_length_overflow()),
        ),
        (
            FailurePoint::Transcode,
            TranscodeDecodeError::domain_main("transcode", 0),
        ),
        (FailurePoint::Finish, TranscodeDecodeError::domain_finish("finish")),
    ] {
        let mut transcoder = FailingTranscoder { failure };
        let mut output = [0_u8; 1];

        let error = transcoder
            .transcode_complete_into(b"", &mut output)
            .expect_err("configured stage should fail");

        assert_eq!(expected, error);
    }
}

#[test]
fn test_transcoder_transcode_complete_into_reports_insufficient_output() {
    let mut transcoder = FinishingTranscoder::default();
    let mut output = [0_u8; 4];

    let error = transcoder
        .transcode_complete_into(b"abc", &mut output)
        .expect_err("complete transcode requires five output units");

    assert_eq!(
        codec::TranscodeDecodeError::Failure(codec::TranscodeFailure::insufficient_output(0, 5, 4)),
        error,
    );
}

#[test]
fn test_transcoder_transcode_complete_into_maps_runtime_need_output() {
    let mut transcoder = UnderestimatingTranscoder;
    let mut output = [];

    let error = transcoder
        .transcode_complete_into(b"a", &mut output)
        .expect_err("runtime need-output status should be an output error");

    assert_eq!(
        codec::TranscodeDecodeError::Failure(codec::TranscodeFailure::insufficient_output(0, 1, 0)),
        error,
    );
}

#[test]
fn test_transcoder_transcode_complete_into_rejects_trailing_input_progress() {
    let mut transcoder = PartialCompleteTranscoder;
    let mut output = [0_u8; 2];

    let error = transcoder
        .transcode_complete_into(b"ab", &mut output)
        .expect_err("default EOF handling must reject invalid complete progress");

    assert_eq!(
        codec::TranscodeDecodeError::Failure(codec::TranscodeFailure::invalid_progress(
            codec::TranscodeContractError::CompleteWithRemainingInput { read: 1, available: 2 },
        )),
        error,
    );
}

#[test]
fn test_transcoder_transcode_complete_into_validates_progress() {
    let mut transcoder = OverreportingCompleteTranscoder;
    let mut output = [];

    let error = transcoder
        .transcode_complete_into(b"", &mut output)
        .expect_err("default EOF handling must reject invalid output progress");

    assert_eq!(
        codec::TranscodeDecodeError::Failure(codec::TranscodeFailure::invalid_progress(
            codec::TranscodeContractError::OverWritten {
                written: 1,
                available: 0,
            },
        )),
        error,
    );
}

#[test]
#[should_panic(expected = "Transcoder::reset wrote beyond its bound")]
fn test_transcoder_transcode_complete_into_panics_when_reset_overreports() {
    let mut transcoder = OverreportingResetTranscoder;
    let mut output = [];

    let _ = transcoder.transcode_complete_into(b"", &mut output);
}

#[test]
#[should_panic(expected = "Transcoder::finish wrote beyond its bound")]
fn test_transcoder_transcode_complete_into_panics_when_finish_overreports() {
    let mut transcoder = OverreportingFinishTranscoder;
    let mut output = [];

    let _ = transcoder.transcode_complete_into(b"", &mut output);
}

#[test]
#[should_panic(expected = "Transcoder::finish wrote beyond available output")]
fn test_transcoder_transcode_complete_into_panics_when_finish_exceeds_available_output() {
    let mut transcoder = FinishBeyondAvailableTranscoder;
    let mut output = [0_u8; 1];

    let _ = transcoder.transcode_complete_into(b"", &mut output);
}

#[test]
fn test_transcoder_transcode_complete_into_reports_remaining_bound_overflow() {
    let mut transcoder = OverflowBoundTranscoder;
    let mut output = [];

    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        transcoder.max_total_output_len(0),
    );

    let error = transcoder
        .transcode_complete_into(b"", &mut output)
        .expect_err("transcode plus finish bound overflows");

    assert_eq!(
        codec::TranscodeDecodeError::Failure(codec::TranscodeFailure::output_length_overflow()),
        error
    );
}

#[test]
fn test_transcoder_transcode_complete_into_reports_incomplete_input() {
    let mut transcoder = PairTranscoder;
    let mut output = [0_u8; 1];

    let error = transcoder
        .transcode_complete_into(b"abc", &mut output)
        .expect_err("odd-length complete input is incomplete");

    assert_eq!(
        codec::TranscodeDecodeError::Failure(codec::TranscodeFailure::incomplete_input(2, 2, 1)),
        error,
    );

    let error = PairTranscoder
        .transcode_eof(b"a", 0, &mut [], 0)
        .expect_err("the default EOF policy should reject incomplete input");
    assert_eq!(
        codec::TranscodeDecodeError::Failure(codec::TranscodeFailure::incomplete_input(0, 2, 1)),
        error,
    );

    let mut transcoder = NeedInputEofTranscoder;
    let error = transcoder
        .transcode_complete_into(b"a", &mut [0])
        .expect_err("EOF progress that remains incomplete must be rejected");
    assert_eq!(
        codec::TranscodeDecodeError::Failure(codec::TranscodeFailure::incomplete_input(1, 2, 0)),
        error,
    );
}

#[test]
fn test_transcode_eof_rejects_progress_that_reads_beyond_available_input() {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut transcoder = OverreadingEofTranscoder;
        transcoder.transcode_eof(&[0], 0, &mut [], 0)
    }));

    assert!(result.is_ok(), "invalid EOF progress must not panic");
    let error = result
        .expect("invalid EOF progress must not panic")
        .expect_err("invalid EOF progress must return a framework failure");
    assert_eq!(
        codec::TranscodeDecodeError::Failure(codec::TranscodeFailure::invalid_progress(
            codec::TranscodeContractError::OverRead { read: 2, available: 1 },
        )),
        error,
    );
}

#[test]
fn test_transcoder_explicit_finish_reports_output_index_beyond_buffer() {
    let mut transcoder = CopyTranscoder;
    let mut output = [];

    let error = transcoder
        .finish(&mut output, 1)
        .expect_err("out-of-range finish output index should be rejected");

    assert_eq!(
        codec::TranscodeDecodeError::Failure(codec::TranscodeFailure::invalid_output_index(1, 0)),
        error
    );
}

#[test]
fn test_transcoder_finish_requires_one_shot_output_capacity() {
    let mut transcoder = FinishingTranscoder::default();
    let mut output = [0_u8; 1];

    assert_eq!(Ok(2), transcoder.max_finish_output_len());

    let error = transcoder
        .finish(&mut output, 0)
        .expect_err("finish should reject partial output capacity");

    assert_eq!(
        codec::TranscodeDecodeError::Failure(codec::TranscodeFailure::insufficient_output(0, 2, 1)),
        error,
    );
    assert_eq!([0], output);
    assert_eq!(Ok(2), transcoder.max_finish_output_len());

    let mut output = [0_u8; 2];
    let written = transcoder
        .finish(&mut output, 0)
        .expect("finish should write the whole suffix once capacity is available");

    assert_eq!(2, written);
    assert_eq!(*b"!\n", output);
    assert_eq!(Ok(2), transcoder.max_finish_output_len());

    transcoder
        .reset(&mut output, 0)
        .expect("reset clears finish suffix state");
    assert_eq!(Ok(2), transcoder.max_finish_output_len());
}

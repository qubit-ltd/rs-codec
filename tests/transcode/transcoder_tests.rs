// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_codec::{
    CapacityError,
    CodecConvertError,
    TranscodeError,
    TranscodeProgress,
    TranscodeStatus,
    Transcoder,
};

#[derive(Default)]
struct CopyTranscoder;

impl Transcoder<u8, u8> for CopyTranscoder {
    type Error =
        CodecConvertError<core::convert::Infallible, core::convert::Infallible>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn reset(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        TranscodeError::<Self::Error>::ensure_output_index(
            output.len(),
            output_index,
        )?;
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeError<Self::Error>> {
        let mut read = 0;
        let mut written = 0;
        while input_index + read < input.len()
            && output_index + written < output.len()
        {
            output[output_index + written] = input[input_index + read];
            read += 1;
            written += 1;
        }
        if input_index + read == input.len() {
            Ok(TranscodeProgress::complete(read, written))
        } else {
            let status = TranscodeStatus::NeedOutput {
                output_index: output_index + written,
                required: crate::nz(1),
                available: output.len().saturating_sub(output_index + written),
            };
            Ok(TranscodeProgress::new(status, read, written))
        }
    }

    fn finish(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        TranscodeError::<Self::Error>::ensure_output_index(
            output.len(),
            output_index,
        )?;
        Ok(0)
    }
}

#[derive(Default)]
struct FinishingTranscoder {
    suffix_index: usize,
}

impl Transcoder<u8, u8> for FinishingTranscoder {
    type Error =
        CodecConvertError<core::convert::Infallible, core::convert::Infallible>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(2 - self.suffix_index)
    }

    fn reset(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        TranscodeError::<Self::Error>::ensure_output_index(
            output.len(),
            output_index,
        )?;
        self.suffix_index = 0;
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeError<Self::Error>> {
        CopyTranscoder.transcode(input, input_index, output, output_index)
    }

    fn finish(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        let suffix = *b"!\n";
        let required = suffix.len() - self.suffix_index;
        TranscodeError::<Self::Error>::ensure_output_capacity(
            output.len(),
            output_index,
            required,
        )?;
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
struct PairTranscoder;

impl Transcoder<u8, u8> for PairTranscoder {
    type Error =
        CodecConvertError<core::convert::Infallible, core::convert::Infallible>;

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len / 2)
    }

    fn reset(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        TranscodeError::<Self::Error>::ensure_output_index(
            output.len(),
            output_index,
        )?;
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeError<Self::Error>> {
        let available = input.len() - input_index;
        if !available.is_multiple_of(2) {
            let complete_len = available - 1;
            for i in 0..complete_len / 2 {
                output[output_index + i] =
                    input[input_index + i * 2] ^ input[input_index + i * 2 + 1];
            }
            return Ok(TranscodeProgress::new(
                TranscodeStatus::NeedInput {
                    input_index: input_index + complete_len,
                    required: crate::nz(2),
                    available: 1,
                },
                complete_len,
                complete_len / 2,
            ));
        }
        for i in 0..available / 2 {
            output[output_index + i] =
                input[input_index + i * 2] ^ input[input_index + i * 2 + 1];
        }
        Ok(TranscodeProgress::complete(available, available / 2))
    }

    fn finish(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        TranscodeError::<Self::Error>::ensure_output_index(
            output.len(),
            output_index,
        )?;
        Ok(0)
    }
}

#[derive(Default)]
struct UnderestimatingTranscoder;

impl Transcoder<u8, u8> for UnderestimatingTranscoder {
    type Error =
        CodecConvertError<core::convert::Infallible, core::convert::Infallible>;

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(0)
    }

    fn reset(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        TranscodeError::<Self::Error>::ensure_output_index(
            output.len(),
            output_index,
        )?;
        Ok(0)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeError<Self::Error>> {
        CopyTranscoder.transcode(input, input_index, output, output_index)
    }

    fn finish(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        TranscodeError::<Self::Error>::ensure_output_index(
            output.len(),
            output_index,
        )?;
        Ok(0)
    }
}

#[derive(Default)]
struct OverflowBoundTranscoder;

impl Transcoder<u8, u8> for OverflowBoundTranscoder {
    type Error =
        CodecConvertError<core::convert::Infallible, core::convert::Infallible>;

    fn max_transcode_output_len(
        &self,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(usize::MAX)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(1)
    }

    fn reset(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        TranscodeError::<Self::Error>::ensure_output_index(
            output.len(),
            output_index,
        )?;
        Ok(0)
    }

    fn transcode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<TranscodeProgress, TranscodeError<Self::Error>> {
        unreachable!("capacity overflow happens before transcode")
    }

    fn finish(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        unreachable!("capacity overflow happens before finish")
    }
}

#[derive(Clone, Copy)]
enum FailurePoint {
    ResetBound,
    TranscodeBound,
    FinishBound,
    Reset,
    Transcode,
    Finish,
}

struct FailingTranscoder {
    failure: FailurePoint,
}

impl Transcoder<u8, u8> for FailingTranscoder {
    type Error = &'static str;

    fn max_reset_output_len(&self) -> Result<usize, CapacityError> {
        if matches!(self.failure, FailurePoint::ResetBound) {
            Err(CapacityError::OutputLengthOverflow)
        } else {
            Ok(0)
        }
    }

    fn max_transcode_output_len(
        &self,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
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

    fn reset(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        TranscodeError::<Self::Error>::ensure_output_index(
            output.len(),
            output_index,
        )?;
        if matches!(self.failure, FailurePoint::Reset) {
            Err(TranscodeError::Domain("reset"))
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
    ) -> Result<TranscodeProgress, TranscodeError<Self::Error>> {
        if matches!(self.failure, FailurePoint::Transcode) {
            Err(TranscodeError::Domain("transcode"))
        } else {
            Ok(TranscodeProgress::complete(0, 0))
        }
    }

    fn finish(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, TranscodeError<Self::Error>> {
        TranscodeError::<Self::Error>::ensure_output_index(
            output.len(),
            output_index,
        )?;
        if matches!(self.failure, FailurePoint::Finish) {
            Err(TranscodeError::Domain("finish"))
        } else {
            Ok(0)
        }
    }
}

#[test]
fn test_transcoder_error_is_domain_error_type() {
    fn assert_domain_error<T, Input, Output>()
    where
        T: Transcoder<Input, Output>,
    {
    }

    assert_domain_error::<CopyTranscoder, u8, u8>();
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

    Transcoder::<u8, u8>::reset(&mut transcoder, &mut output, 0)
        .expect("reset is noop");
    let written = transcoder.finish(&mut output, 0).expect("finish is noop");

    assert_eq!(0, written);
    assert_eq!([0], output);
}

#[test]
fn test_transcoder_total_output_len_sums_reset_transcode_and_finish() {
    let transcoder = FinishingTranscoder::default();

    assert_eq!(Ok(5), transcoder.max_total_output_len(3));
}

#[test]
fn test_transcoder_total_output_len_reports_component_errors() {
    for failure in [
        FailurePoint::ResetBound,
        FailurePoint::TranscodeBound,
        FailurePoint::FinishBound,
    ] {
        let transcoder = FailingTranscoder { failure };

        assert_eq!(
            Err(CapacityError::OutputLengthOverflow),
            transcoder.max_total_output_len(1),
        );
    }
}

#[test]
fn test_transcoder_transcode_all_into_runs_reset_transcode_and_finish() {
    let mut transcoder = FinishingTranscoder::default();
    let mut output = [0_u8; 5];

    let written = transcoder
        .transcode_all_into(b"abc", &mut output)
        .expect("complete transcode should fit");

    assert_eq!(5, written);
    assert_eq!(b"abc!\n", &output);
    assert_eq!(Ok(0), transcoder.max_finish_output_len());
}

#[test]
fn test_transcoder_transcode_all_into_reports_stage_errors() {
    for (failure, expected) in [
        (FailurePoint::Reset, TranscodeError::Domain("reset")),
        (
            FailurePoint::TranscodeBound,
            TranscodeError::OutputLengthOverflow,
        ),
        (
            FailurePoint::FinishBound,
            TranscodeError::OutputLengthOverflow,
        ),
        (FailurePoint::Transcode, TranscodeError::Domain("transcode")),
        (FailurePoint::Finish, TranscodeError::Domain("finish")),
    ] {
        let mut transcoder = FailingTranscoder { failure };
        let mut output = [0_u8; 1];

        let error = transcoder
            .transcode_all_into(b"", &mut output)
            .expect_err("configured stage should fail");

        assert_eq!(expected, error);
    }
}

#[test]
fn test_transcoder_transcode_all_into_reports_insufficient_output() {
    let mut transcoder = FinishingTranscoder::default();
    let mut output = [0_u8; 4];

    let error = transcoder
        .transcode_all_into(b"abc", &mut output)
        .expect_err("complete transcode requires five output units");

    assert_eq!(
        TranscodeError::InsufficientOutput {
            output_index: 0,
            required: 5,
            available: 4,
        },
        error,
    );
}

#[test]
fn test_transcoder_transcode_all_into_maps_runtime_need_output() {
    let mut transcoder = UnderestimatingTranscoder;
    let mut output = [];

    let error = transcoder
        .transcode_all_into(b"a", &mut output)
        .expect_err("runtime need-output status should be an output error");

    assert_eq!(
        TranscodeError::InsufficientOutput {
            output_index: 0,
            required: 1,
            available: 0,
        },
        error,
    );
}

#[test]
fn test_transcoder_transcode_all_into_reports_remaining_bound_overflow() {
    let mut transcoder = OverflowBoundTranscoder;
    let mut output = [];

    let error = transcoder
        .transcode_all_into(b"", &mut output)
        .expect_err("transcode plus finish bound overflows");

    assert_eq!(TranscodeError::OutputLengthOverflow, error);
}

#[test]
fn test_transcoder_transcode_all_into_reports_incomplete_input() {
    let mut transcoder = PairTranscoder;
    let mut output = [0_u8; 1];

    let error = transcoder
        .transcode_all_into(b"abc", &mut output)
        .expect_err("odd-length complete input is incomplete");

    assert_eq!(
        TranscodeError::IncompleteInput {
            input_index: 2,
            required: 2,
            available: 1,
        },
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
        TranscodeError::InvalidOutputIndex { index: 1, len: 0 },
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
        TranscodeError::InsufficientOutput {
            output_index: 0,
            required: 2,
            available: 1
        },
        error,
    );
    assert_eq!([0], output);
    assert_eq!(Ok(2), transcoder.max_finish_output_len());

    let mut output = [0_u8; 2];
    let written = transcoder.finish(&mut output, 0).expect(
        "finish should write the whole suffix once capacity is available",
    );

    assert_eq!(2, written);
    assert_eq!(*b"!\n", output);
    assert_eq!(Ok(0), transcoder.max_finish_output_len());

    transcoder
        .reset(&mut output, 0)
        .expect("reset clears finish suffix state");
    assert_eq!(Ok(2), transcoder.max_finish_output_len());
}

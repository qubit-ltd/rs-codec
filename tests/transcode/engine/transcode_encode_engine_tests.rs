// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the reusable buffered encoder engine.

use qubit_codec::{
    CapacityError,
    Codec,
    CodecEncodeFlushError,
    CodecEncodeResetError,
    EncodeContext,
    EncodeOutcome,
    TranscodeEncodeEngine,
    TranscodeEncodeHooks,
    TranscodeError,
    TranscodeProgress,
    TranscodeStatus,
    Transcoder,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct WideCodec;

impl Codec for WideCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(4);

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
        debug_assert!(input_index < input.len());

        // SAFETY: The caller guarantees that `input_index` is readable.
        let value = unsafe { *input.as_ptr().add(input_index) };
        Ok((value, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(output_index < output.len());

        // SAFETY: The caller guarantees that `output_index` is writable.
        unsafe {
            *output.as_mut_ptr().add(output_index) = *value;
        }
        Ok(qubit_io::nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
enum EngineError {
    #[error("rejected input at index {input_index}")]
    Rejected { input_index: usize },
}

impl From<core::convert::Infallible> for EngineError {
    fn from(error: core::convert::Infallible) -> Self {
        match error {}
    }
}

impl From<CodecEncodeResetError<core::convert::Infallible>> for EngineError {
    #[allow(unreachable_code)]
    fn from(error: CodecEncodeResetError<core::convert::Infallible>) -> Self {
        match error.into_source() {}
    }
}

impl From<CodecEncodeFlushError<EngineError>> for EngineError {
    fn from(error: CodecEncodeFlushError<EngineError>) -> Self {
        error.into_source()
    }
}

impl From<CodecEncodeFlushError<core::convert::Infallible>> for EngineError {
    #[allow(unreachable_code)]
    fn from(error: CodecEncodeFlushError<core::convert::Infallible>) -> Self {
        match error.into_source() {}
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ExactWidthHooks;

impl TranscodeEncodeHooks<WideCodec> for ExactWidthHooks {
    type Error = EngineError;

    fn encode_value(
        &mut self,
        _codec: &mut WideCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeOutcome, Self::Error> {
        if context.available_output() < 1 {
            return Ok(EncodeOutcome::need_output(crate::nz(1)));
        }
        let (input_value, _, output, output_index) = context.into_parts();
        debug_assert!(output_index < output.len());

        // SAFETY: The capacity check above reserves one output unit.
        unsafe {
            *output.as_mut_ptr().add(output_index) =
                input_value.wrapping_add(10);
        }
        Ok(EncodeOutcome::consumed(1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct SkippingHooks;

impl TranscodeEncodeHooks<WideCodec> for SkippingHooks {
    type Error = EngineError;

    fn encode_value(
        &mut self,
        _codec: &mut WideCodec,
        _context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeOutcome, Self::Error> {
        Ok(EncodeOutcome::consumed(0))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RejectingHooks;

impl TranscodeEncodeHooks<WideCodec> for RejectingHooks {
    type Error = EngineError;

    fn encode_value(
        &mut self,
        _codec: &mut WideCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeOutcome, Self::Error> {
        Err(EngineError::Rejected {
            input_index: context.input_index(),
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FailingFinishHooks;

impl TranscodeEncodeHooks<WideCodec> for FailingFinishHooks {
    type Error = EngineError;

    fn encode_value(
        &mut self,
        _codec: &mut WideCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeOutcome, Self::Error> {
        if context.available_output() < 1 {
            return Ok(EncodeOutcome::need_output(crate::nz(1)));
        }
        let (v, _, out, oi) = context.into_parts();
        out[oi] = *v;
        Ok(EncodeOutcome::consumed(1))
    }

    fn max_finish_output_len(&self, _codec: &WideCodec) -> usize {
        1
    }

    fn finish_hooks(
        &mut self,
        _codec: &mut WideCodec,
        _output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        Err(EngineError::Rejected {
            input_index: output_index,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FlushFailingCodec;

impl Codec for FlushFailingCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = EngineError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize = qubit_io::nz!(1);

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
        Ok((input[input_index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(qubit_io::nz!(1))
    }

    unsafe fn encode_flush(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Err(EngineError::Rejected { input_index: 0 })
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FlushFailingHooks;

impl TranscodeEncodeHooks<FlushFailingCodec> for FlushFailingHooks {
    type Error = EngineError;

    fn encode_value(
        &mut self,
        _codec: &mut FlushFailingCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeOutcome, Self::Error> {
        if context.available_output() < 1 {
            return Ok(EncodeOutcome::need_output(crate::nz(1)));
        }
        let (v, _, out, oi) = context.into_parts();
        out[oi] = *v;
        Ok(EncodeOutcome::consumed(1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverreportingWriteHooks;

impl TranscodeEncodeHooks<WideCodec> for OverreportingWriteHooks {
    type Error = EngineError;

    fn encode_value(
        &mut self,
        _codec: &mut WideCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeOutcome, Self::Error> {
        if context.available_output() < 1 {
            return Ok(EncodeOutcome::need_output(crate::nz(1)));
        }
        let (v, _, out, oi) = context.into_parts();
        out[oi] = *v;
        Ok(EncodeOutcome::consumed(2))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FinishHooks {
    pending_suffix: bool,
}

impl Default for FinishHooks {
    fn default() -> Self {
        Self {
            pending_suffix: true,
        }
    }
}

impl TranscodeEncodeHooks<WideCodec> for FinishHooks {
    type Error = EngineError;

    fn encode_value(
        &mut self,
        _codec: &mut WideCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeOutcome, Self::Error> {
        if context.available_output() < 1 {
            return Ok(EncodeOutcome::need_output(crate::nz(1)));
        }
        let (input_value, _, output, output_index) = context.into_parts();
        output[output_index] = *input_value;
        Ok(EncodeOutcome::consumed(1))
    }

    fn max_finish_output_len(&self, _codec: &WideCodec) -> usize {
        usize::from(self.pending_suffix)
    }

    fn finish_hooks(
        &mut self,
        _codec: &mut WideCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        if !self.pending_suffix {
            return Ok(0);
        }

        output[output_index] = 0xee;
        self.pending_suffix = false;
        Ok(1)
    }

    fn reset_hooks(&mut self, _codec: &mut WideCodec) {
        self.pending_suffix = false;
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverwritingFinishHooks;

impl TranscodeEncodeHooks<WideCodec> for OverwritingFinishHooks {
    type Error = EngineError;

    fn encode_value(
        &mut self,
        _codec: &mut WideCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeOutcome, Self::Error> {
        if context.available_output() < 1 {
            return Ok(EncodeOutcome::need_output(crate::nz(1)));
        }
        let (v, _, out, oi) = context.into_parts();
        out[oi] = *v;
        Ok(EncodeOutcome::consumed(1))
    }

    fn max_finish_output_len(&self, _codec: &WideCodec) -> usize {
        1
    }

    fn finish_hooks(
        &mut self,
        _codec: &mut WideCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        output[output_index] = 0xee;
        output[output_index + 1] = 0xdd;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverreportingFinishHooks;

impl TranscodeEncodeHooks<WideCodec> for OverreportingFinishHooks {
    type Error = EngineError;

    fn encode_value(
        &mut self,
        _codec: &mut WideCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeOutcome, Self::Error> {
        if context.available_output() < 1 {
            return Ok(EncodeOutcome::need_output(crate::nz(1)));
        }
        let (v, _, out, oi) = context.into_parts();
        out[oi] = *v;
        Ok(EncodeOutcome::consumed(1))
    }

    fn max_finish_output_len(&self, _codec: &WideCodec) -> usize {
        1
    }

    fn finish_hooks(
        &mut self,
        _codec: &mut WideCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        output[output_index] = 0xee;
        Ok(2)
    }
}

#[test]
fn test_buffered_encode_engine_reports_bounds_and_resets() {
    let mut encoder =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, ExactWidthHooks);

    assert_eq!(Ok(8), encoder.max_output_len(2));
    assert_eq!(Ok(0), encoder.max_finish_output_len());
    assert_eq!(Ok(0), encoder.max_reset_output_len());
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        encoder.max_output_len(usize::MAX),
    );
    encoder.reset(&mut [], 0).expect("reset");
}

#[test]
fn test_buffered_encode_engine_delegates_finish_to_hooks() {
    let mut encoder =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, FinishHooks::default());
    let mut output = [0_u8; 1];

    assert_eq!(Ok(1), encoder.max_finish_output_len());

    let error = encoder.finish(&mut [], 0).expect_err(
        "finish should reject insufficient output before calling hooks",
    );
    assert_eq!(
        TranscodeError::InsufficientOutput {
            output_index: 0,
            required: 1,
            available: 0
        },
        error,
    );
    assert_eq!(Ok(1), encoder.max_finish_output_len());

    let written = encoder
        .finish(&mut output, 0)
        .expect("hook should write final output");
    assert_eq!(1, written);
    assert_eq!([0xee], output);
    assert_eq!(Ok(0), encoder.max_finish_output_len());

    let mut encoder =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, FinishHooks::default());
    encoder.reset(&mut [], 0).expect("reset");
    assert_eq!(Ok(0), encoder.max_finish_output_len());
}

#[test]
fn test_buffered_encode_engine_implements_transcoder() {
    type Engine = TranscodeEncodeEngine<WideCodec, ExactWidthHooks>;
    type EngineResult<T> = Result<T, TranscodeError<EngineError>>;
    type TranscodeFn = fn(
        &mut Engine,
        &[u8],
        usize,
        &mut [u8],
        usize,
    ) -> EngineResult<TranscodeProgress>;
    type OutputFn = fn(&mut Engine, &mut [u8], usize) -> EngineResult<usize>;

    let mut encoder =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, ExactWidthHooks);
    let mut output = [0_u8; 2];
    let max_output_len: fn(&Engine, usize) -> Result<usize, CapacityError> =
        std::hint::black_box(<Engine as Transcoder<u8, u8>>::max_output_len);
    let max_finish_output_len: fn(&Engine) -> Result<usize, CapacityError> =
        std::hint::black_box(
            <Engine as Transcoder<u8, u8>>::max_finish_output_len,
        );
    let max_reset_output_len: fn(&Engine) -> Result<usize, CapacityError> =
        std::hint::black_box(
            <Engine as Transcoder<u8, u8>>::max_reset_output_len,
        );
    let transcode: TranscodeFn =
        std::hint::black_box(<Engine as Transcoder<u8, u8>>::transcode);
    let reset: OutputFn =
        std::hint::black_box(<Engine as Transcoder<u8, u8>>::reset);
    let finish: OutputFn =
        std::hint::black_box(<Engine as Transcoder<u8, u8>>::finish);

    assert_eq!(Ok(8), max_output_len(&encoder, 2));
    assert_eq!(Ok(0), max_finish_output_len(&encoder));
    assert_eq!(Ok(0), max_reset_output_len(&encoder));
    let progress = transcode(&mut encoder, &[1, 2], 0, &mut output, 0)
        .expect("engine should transcode through the trait");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((2, 2), (progress.read(), progress.written()));
    assert_eq!([11, 12], output);

    let mut empty_output = [0_u8; 0];
    let reset = reset(&mut encoder, &mut empty_output, 0)
        .expect("engine should reset through the trait");
    let finished = finish(&mut encoder, &mut empty_output, 0)
        .expect("engine should finish through the trait");

    assert_eq!(0, reset);
    assert_eq!(0, finished);
}

#[test]
fn test_buffered_encode_engine_finish_maps_hook_errors() {
    let mut encoder =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, FailingFinishHooks);
    let mut output = [0_u8; 1];

    let error = encoder
        .finish(&mut output, 0)
        .expect_err("finish hook error should be propagated");

    assert_eq!(
        TranscodeError::Domain(EngineError::Rejected { input_index: 0 }),
        error,
    );
}

#[test]
fn test_buffered_encode_engine_finish_converts_codec_flush_errors() {
    let mut encoder = TranscodeEncodeEngine::<_, _>::new(
        FlushFailingCodec,
        FlushFailingHooks,
    );
    let mut output = [0_u8; 1];

    let error = encoder
        .finish(&mut output, 0)
        .expect_err("finish should convert codec flush errors");

    assert_eq!(
        TranscodeError::Domain(EngineError::Rejected { input_index: 0 }),
        error,
    );
}

#[test]
fn test_buffered_encode_engine_finish_passes_full_output_to_hooks() {
    let mut encoder =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, OverwritingFinishHooks);
    let mut output = [0_u8; 2];

    let written = encoder
        .finish(&mut output, 0)
        .expect("hook should receive the caller-provided output slice");

    assert_eq!(1, written);
    assert_eq!([0xee, 0xdd], output);
}

#[test]
#[should_panic(
    expected = "TranscodeEncodeEngine hook wrote beyond its finish bound"
)]
fn test_buffered_encode_engine_finish_panics_when_hook_overreports_bound() {
    let mut encoder =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, OverreportingFinishHooks);
    let mut output = [0_u8; 2];

    let _ = encoder.finish(&mut output, 0);
}

#[test]
fn test_buffered_encode_engine_finish_reports_output_index_beyond_buffer() {
    let mut encoder =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, FinishHooks::default());
    let mut output = [];

    let error = encoder
        .finish(&mut output, 1)
        .expect_err("out-of-range finish output index should be rejected");

    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 1, len: 0 },
        error,
    );
}

#[test]
fn test_buffered_encode_engine_default_finish_reports_output_index_beyond_buffer()
 {
    let mut encoder =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, ExactWidthHooks);
    let mut output = [];

    let error = encoder
        .finish(&mut output, 1)
        .expect_err("default finish should reject out-of-range output index");

    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 1, len: 0 },
        error,
    );
}

#[test]
fn test_buffered_encode_hooks_default_finish_is_noop() {
    let mut hooks = ExactWidthHooks;
    let mut output = [];

    let written = TranscodeEncodeHooks::<WideCodec>::finish_hooks(
        &mut hooks,
        &mut WideCodec,
        &mut output,
        1,
    )
    .expect("default hook finish should be a no-op");

    assert_eq!(0, written);
}

#[test]
fn test_buffered_encode_engine_uses_hook_capacity_instead_of_codec_max_width() {
    let mut encoder = TranscodeEncodeEngine::new(WideCodec, ExactWidthHooks);
    let mut output = [0_u8; 1];

    let progress = encoder
        .transcode(&[1, 2], 0, &mut output, 0)
        .expect("engine encoding should succeed");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 1,
            required: crate::nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(1, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([11], output);
    assert_eq!(Ok(8), encoder.max_output_len(2));
}

#[test]
fn test_buffered_encode_engine_allows_zero_width_value_to_consume_input() {
    let mut encoder = TranscodeEncodeEngine::new(WideCodec, SkippingHooks);
    let mut output = [];

    let progress = encoder
        .transcode(&[1, 2, 3], 0, &mut output, 0)
        .expect("zero-width value should not need output");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(3, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_buffered_encode_engine_reports_output_index_beyond_buffer() {
    let mut encoder = TranscodeEncodeEngine::new(WideCodec, ExactWidthHooks);
    let mut output = [];

    let error = encoder
        .transcode(&[1], 0, &mut output, 1)
        .expect_err("out-of-range output index should fail");

    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 1, len: 0 },
        error,
    );
}

#[test]
#[should_panic(
    expected = "EncodeOutcome::Consumed wrote beyond available output"
)]
fn test_buffered_encode_engine_panics_when_hook_reports_too_many_written_units()
{
    let mut encoder =
        TranscodeEncodeEngine::new(WideCodec, OverreportingWriteHooks);
    let mut output = [0_u8; 1];

    let _ = encoder.transcode(&[1], 0, &mut output, 0);
}

#[test]
fn test_buffered_encode_engine_propagates_encode_value_error_without_consuming_input()
 {
    let mut encoder = TranscodeEncodeEngine::new(WideCodec, RejectingHooks);
    let mut output = [0_u8; 4];

    let error = encoder
        .transcode(&[1], 0, &mut output, 0)
        .expect_err("encode hook error should be propagated");

    assert_eq!(
        TranscodeError::Domain(EngineError::Rejected { input_index: 0 }),
        error
    );
    assert_eq!([0, 0, 0, 0], output);
}

#[test]
fn test_buffered_encode_engine_uses_hooks_for_invalid_input_index() {
    let mut encoder = TranscodeEncodeEngine::new(WideCodec, ExactWidthHooks);
    let mut output = [];

    let error = encoder
        .transcode(&[1], 2, &mut output, 0)
        .expect_err("invalid input index should be rejected");

    assert_eq!(
        TranscodeError::InvalidInputIndex { index: 2, len: 1 },
        error,
    );
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetEmittingCodec;

impl Codec for ResetEmittingCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_ENCODE_RESET_UNITS: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
        Ok((input[input_index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(qubit_io::nz!(1))
    }

    unsafe fn encode_reset(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = 0xaa;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetFailCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("reset failed")]
struct ResetFailError;

impl From<CodecEncodeResetError<ResetFailError>> for ResetFailError {
    fn from(error: CodecEncodeResetError<ResetFailError>) -> Self {
        error.into_source()
    }
}

impl Codec for ResetFailCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = ResetFailError;

    const MIN_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: core::num::NonZeroUsize =
        core::num::NonZeroUsize::MIN;

    const MAX_ENCODE_RESET_UNITS: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
        Ok((input[input_index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(qubit_io::nz!(1))
    }

    unsafe fn encode_reset(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Err(ResetFailError)
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct ResetErrorMappingHooks;

impl TranscodeEncodeHooks<ResetFailCodec> for ResetErrorMappingHooks {
    type Error = ResetFailError;

    fn encode_value(
        &mut self,
        _codec: &mut ResetFailCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeOutcome, Self::Error> {
        if context.available_output() < 1 {
            return Ok(EncodeOutcome::need_output(crate::nz(1)));
        }
        let (v, _, out, oi) = context.into_parts();
        out[oi] = *v;
        Ok(EncodeOutcome::consumed(1))
    }
}

#[test]
fn test_buffered_encode_engine_default_builds_engine() {
    let mut encoder =
        TranscodeEncodeEngine::<WideCodec, ExactWidthHooks>::default();
    let mut output = [0_u8; 1];

    let progress = encoder
        .transcode(&[7], 0, &mut output, 0)
        .expect("default engine should encode one value");

    assert_eq!(1, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([17], output);
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetPassthroughHooks;

impl TranscodeEncodeHooks<ResetEmittingCodec> for ResetPassthroughHooks {
    type Error = EngineError;

    fn encode_value(
        &mut self,
        _codec: &mut ResetEmittingCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeOutcome, Self::Error> {
        if context.available_output() < 1 {
            return Ok(EncodeOutcome::need_output(crate::nz(1)));
        }
        let (v, _, out, oi) = context.into_parts();
        out[oi] = *v;
        Ok(EncodeOutcome::consumed(1))
    }
}

#[test]
fn test_buffered_encode_engine_reset_emits_codec_reset_output() {
    let mut encoder = TranscodeEncodeEngine::<_, _>::new(
        ResetEmittingCodec,
        ResetPassthroughHooks,
    );
    let mut output = [0_u8; 1];

    let written = encoder
        .reset(&mut output, 0)
        .expect("reset should emit codec reset output");

    assert_eq!(1, written);
    assert_eq!([0xaa], output);
}

#[test]
fn test_buffered_encode_engine_reset_rejects_insufficient_output() {
    let mut encoder = TranscodeEncodeEngine::<_, _>::new(
        ResetEmittingCodec,
        ResetPassthroughHooks,
    );
    let mut output = [];

    let error = encoder
        .reset(&mut output, 0)
        .expect_err("reset should reject insufficient output capacity");

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
fn test_buffered_encode_engine_reset_converts_codec_reset_errors() {
    let mut encoder = TranscodeEncodeEngine::<_, _>::new(
        ResetFailCodec,
        ResetErrorMappingHooks,
    );
    let mut output = [0_u8; 1];

    let error = encoder
        .reset(&mut output, 0)
        .expect_err("reset should convert codec reset errors");

    assert_eq!(TranscodeError::Domain(ResetFailError), error);
}

// ============================================================================
// Lifecycle guard wiring
// ============================================================================

#[cfg(debug_assertions)]
#[test]
#[should_panic(
    expected = "Transcoder::finish called twice without an intervening reset"
)]
fn test_buffered_encode_engine_lifecycle_rejects_double_finish() {
    let mut engine =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, ExactWidthHooks);
    let mut output = [0_u8; 0];
    engine
        .finish(&mut output, 0)
        .expect("first finish should succeed for a stateless encoder");
    let _ = engine.finish(&mut output, 0);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(
    expected = "Transcoder::transcode called after finish without an \
                intervening reset"
)]
fn test_buffered_encode_engine_lifecycle_rejects_transcode_after_finish() {
    let mut engine =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, ExactWidthHooks);
    let mut output = [0_u8; 1];
    engine
        .finish(&mut output, 0)
        .expect("finish closes the logical stream");
    let _ = engine.transcode(&[1_u8], 0, &mut output, 0);
}

#[test]
fn test_buffered_encode_engine_lifecycle_allows_reuse_after_reset() {
    let mut engine =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, ExactWidthHooks);
    let mut output = [0_u8; 2];
    engine
        .finish(&mut output, 0)
        .expect("first logical stream finalizes");
    engine
        .reset(&mut output, 0)
        .expect("reset reopens the engine");
    let progress = engine
        .transcode(&[1_u8], 0, &mut output, 0)
        .expect("transcode after reset");
    assert_eq!(1, progress.read());
    engine
        .finish(&mut output, 1)
        .expect("second logical stream finalizes");
}

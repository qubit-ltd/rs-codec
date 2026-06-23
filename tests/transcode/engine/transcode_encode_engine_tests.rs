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
    EncodeContext,
    EncodeOutcome,
    TranscodeEncodeEngine,
    TranscodeEncodeHooks,
    TranscodeError,
    TranscodeStatus,
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
        index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
        debug_assert!(index < input.len());

        // SAFETY: The caller guarantees that `index` is readable.
        let value = unsafe { *input.as_ptr().add(index) };
        Ok((value, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(index < output.len());

        // SAFETY: The caller guarantees that `index` is writable.
        unsafe {
            *output.as_mut_ptr().add(index) = *value;
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
        let EncodeContext {
            input_value,
            output,
            output_index,
            ..
        } = context;
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
            input_index: context.input_index,
        })
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
        context.output[context.output_index] = *context.input_value;
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
        let EncodeContext {
            input_value,
            output,
            output_index,
            ..
        } = context;
        output[output_index] = *input_value;
        Ok(EncodeOutcome::consumed(1))
    }

    fn max_finish_output_len(&self, _codec: &WideCodec) -> usize {
        usize::from(self.pending_suffix)
    }

    fn finish(
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

    fn before_reset(&mut self, _codec: &mut WideCodec) {
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
        context.output[context.output_index] = *context.input_value;
        Ok(EncodeOutcome::consumed(1))
    }

    fn max_finish_output_len(&self, _codec: &WideCodec) -> usize {
        1
    }

    fn finish(
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
        context.output[context.output_index] = *context.input_value;
        Ok(EncodeOutcome::consumed(1))
    }

    fn max_finish_output_len(&self, _codec: &WideCodec) -> usize {
        1
    }

    fn finish(
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
    assert_eq!(0, encoder.max_finish_output_len());
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

    assert_eq!(1, encoder.max_finish_output_len());

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
    assert_eq!(1, encoder.max_finish_output_len());

    let written = encoder
        .finish(&mut output, 0)
        .expect("hook should write final output");
    assert_eq!(1, written);
    assert_eq!([0xee], output);
    assert_eq!(0, encoder.max_finish_output_len());

    let mut encoder =
        TranscodeEncodeEngine::<_, _>::new(WideCodec, FinishHooks::default());
    encoder.reset(&mut [], 0).expect("reset");
    assert_eq!(0, encoder.max_finish_output_len());
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

    let written = TranscodeEncodeHooks::<WideCodec>::finish(
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
        index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
        Ok((input[index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[index] = *value;
        Ok(qubit_io::nz!(1))
    }

    unsafe fn encode_reset(
        &mut self,
        output: &mut [u8],
        index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[index] = 0xaa;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetFailCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("reset failed")]
struct ResetFailError;

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
        index: usize,
    ) -> Result<
        (u8, core::num::NonZeroUsize),
        qubit_codec::CodecDecodeFailure<Self::DecodeError>,
    > {
        Ok((input[index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[index] = *value;
        Ok(qubit_io::nz!(1))
    }

    unsafe fn encode_reset(
        &mut self,
        _output: &mut [u8],
        _index: usize,
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
        context.output[context.output_index] = *context.input_value;
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
    type Error = core::convert::Infallible;

    fn encode_value(
        &mut self,
        _codec: &mut ResetEmittingCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeOutcome, Self::Error> {
        if context.available_output() < 1 {
            return Ok(EncodeOutcome::need_output(crate::nz(1)));
        }
        context.output[context.output_index] = *context.input_value;
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

// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the reusable buffered encoder engine.

use qubit_codec::{
    BufferedEncodeEngine, BufferedEncodeHooks, CapacityError, Codec, EncodeContext, EncodePlan,
    FinishError, TranscodeStatus,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct WideCodec;

unsafe impl Codec for WideCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        unsafe { core::num::NonZeroUsize::new_unchecked(4) }
    }

    unsafe fn decode_unchecked(
        &self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        debug_assert!(index < input.len());

        // SAFETY: The caller guarantees that `index` is readable.
        let value = unsafe { *input.as_ptr().add(index) };
        Ok((value, core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode_unchecked(
        &self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<usize, Self::EncodeError> {
        debug_assert!(index < output.len());

        // SAFETY: The caller guarantees that `index` is writable.
        unsafe {
            *output.as_mut_ptr().add(index) = *value;
        }
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EngineError {
    InvalidInputIndex { index: usize, input_len: usize },
    InvalidOutputIndex { index: usize, output_len: usize },
    Rejected { input_index: usize },
}

impl EngineError {
    const fn invalid_input_index(index: usize, input_len: usize) -> Self {
        Self::InvalidInputIndex { index, input_len }
    }

    const fn invalid_output_index(index: usize, output_len: usize) -> Self {
        Self::InvalidOutputIndex { index, output_len }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ExactWidthHooks;

impl BufferedEncodeHooks<WideCodec> for ExactWidthHooks {
    type Error = EngineError;
    type PlanAction = ();

    fn prepare_encode(
        &mut self,
        _codec: &WideCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        Ok(EncodePlan::new(1, ()))
    }

    unsafe fn write_encode(
        &mut self,
        _codec: &WideCodec,
        context: EncodeContext<'_, u8, u8>,
        _plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        let EncodeContext {
            input_value,
            output,
            output_index,
            ..
        } = context;
        debug_assert!(output_index < output.len());

        // SAFETY: The engine checked the one-unit capacity requested by
        // `prepare_encode`.
        unsafe {
            *output.as_mut_ptr().add(output_index) = input_value.wrapping_add(10);
        }
        Ok(1)
    }

    fn invalid_input_index(
        &mut self,
        _codec: &WideCodec,
        index: usize,
        input_len: usize,
    ) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
    }

    fn invalid_output_index(
        &mut self,
        _codec: &WideCodec,
        index: usize,
        output_len: usize,
    ) -> Self::Error {
        EngineError::invalid_output_index(index, output_len)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct SkippingHooks;

impl BufferedEncodeHooks<WideCodec> for SkippingHooks {
    type Error = EngineError;
    type PlanAction = ();

    fn prepare_encode(
        &mut self,
        _codec: &WideCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        Ok(EncodePlan::new(0, ()))
    }

    unsafe fn write_encode(
        &mut self,
        _codec: &WideCodec,
        _context: EncodeContext<'_, u8, u8>,
        _plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn invalid_input_index(
        &mut self,
        _codec: &WideCodec,
        index: usize,
        input_len: usize,
    ) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
    }

    fn invalid_output_index(
        &mut self,
        _codec: &WideCodec,
        index: usize,
        output_len: usize,
    ) -> Self::Error {
        EngineError::invalid_output_index(index, output_len)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RejectingHooks;

impl BufferedEncodeHooks<WideCodec> for RejectingHooks {
    type Error = EngineError;
    type PlanAction = ();

    fn prepare_encode(
        &mut self,
        _codec: &WideCodec,
        _value: &u8,
        input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        Err(EngineError::Rejected { input_index })
    }

    unsafe fn write_encode(
        &mut self,
        _codec: &WideCodec,
        _context: EncodeContext<'_, u8, u8>,
        _plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        unreachable!("prepare_encode rejects every value")
    }

    fn invalid_input_index(
        &mut self,
        _codec: &WideCodec,
        index: usize,
        input_len: usize,
    ) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
    }

    fn invalid_output_index(
        &mut self,
        _codec: &WideCodec,
        index: usize,
        output_len: usize,
    ) -> Self::Error {
        EngineError::invalid_output_index(index, output_len)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverreportingWriteHooks;

impl BufferedEncodeHooks<WideCodec> for OverreportingWriteHooks {
    type Error = EngineError;
    type PlanAction = ();

    fn prepare_encode(
        &mut self,
        _codec: &WideCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        Ok(EncodePlan::new(1, ()))
    }

    unsafe fn write_encode(
        &mut self,
        _codec: &WideCodec,
        context: EncodeContext<'_, u8, u8>,
        _plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        context.output[context.output_index] = *context.input_value;
        Ok(2)
    }

    fn invalid_input_index(
        &mut self,
        _codec: &WideCodec,
        index: usize,
        input_len: usize,
    ) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
    }

    fn invalid_output_index(
        &mut self,
        _codec: &WideCodec,
        index: usize,
        output_len: usize,
    ) -> Self::Error {
        EngineError::invalid_output_index(index, output_len)
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

impl BufferedEncodeHooks<WideCodec> for FinishHooks {
    type Error = EngineError;
    type PlanAction = ();

    fn prepare_encode(
        &mut self,
        _codec: &WideCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        Ok(EncodePlan::new(1, ()))
    }

    unsafe fn write_encode(
        &mut self,
        _codec: &WideCodec,
        context: EncodeContext<'_, u8, u8>,
        _plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        let EncodeContext {
            input_value,
            output,
            output_index,
            ..
        } = context;
        output[output_index] = *input_value;
        Ok(1)
    }

    fn invalid_input_index(
        &mut self,
        _codec: &WideCodec,
        index: usize,
        input_len: usize,
    ) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
    }

    fn invalid_output_index(
        &mut self,
        _codec: &WideCodec,
        index: usize,
        output_len: usize,
    ) -> Self::Error {
        EngineError::invalid_output_index(index, output_len)
    }

    fn max_finish_output_len(&self, _codec: &WideCodec) -> usize {
        usize::from(self.pending_suffix)
    }

    fn finish(
        &mut self,
        _codec: &WideCodec,
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

    fn reset(&mut self, _codec: &WideCodec) {
        self.pending_suffix = false;
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverwritingFinishHooks;

impl BufferedEncodeHooks<WideCodec> for OverwritingFinishHooks {
    type Error = EngineError;
    type PlanAction = ();

    fn prepare_encode(
        &mut self,
        _codec: &WideCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        Ok(EncodePlan::new(1, ()))
    }

    unsafe fn write_encode(
        &mut self,
        _codec: &WideCodec,
        context: EncodeContext<'_, u8, u8>,
        _plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        context.output[context.output_index] = *context.input_value;
        Ok(1)
    }

    fn invalid_input_index(
        &mut self,
        _codec: &WideCodec,
        index: usize,
        input_len: usize,
    ) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
    }

    fn invalid_output_index(
        &mut self,
        _codec: &WideCodec,
        index: usize,
        output_len: usize,
    ) -> Self::Error {
        EngineError::invalid_output_index(index, output_len)
    }

    fn max_finish_output_len(&self, _codec: &WideCodec) -> usize {
        1
    }

    fn finish(
        &mut self,
        _codec: &WideCodec,
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

impl BufferedEncodeHooks<WideCodec> for OverreportingFinishHooks {
    type Error = EngineError;
    type PlanAction = ();

    fn prepare_encode(
        &mut self,
        _codec: &WideCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        Ok(EncodePlan::new(1, ()))
    }

    unsafe fn write_encode(
        &mut self,
        _codec: &WideCodec,
        context: EncodeContext<'_, u8, u8>,
        _plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        context.output[context.output_index] = *context.input_value;
        Ok(1)
    }

    fn invalid_input_index(
        &mut self,
        _codec: &WideCodec,
        index: usize,
        input_len: usize,
    ) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
    }

    fn invalid_output_index(
        &mut self,
        _codec: &WideCodec,
        index: usize,
        output_len: usize,
    ) -> Self::Error {
        EngineError::invalid_output_index(index, output_len)
    }

    fn max_finish_output_len(&self, _codec: &WideCodec) -> usize {
        1
    }

    fn finish(
        &mut self,
        _codec: &WideCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        output[output_index] = 0xee;
        Ok(2)
    }
}

#[test]
fn test_buffered_encode_engine_reports_bounds_and_resets() {
    let mut encoder = BufferedEncodeEngine::<_, _>::new(WideCodec, ExactWidthHooks);

    assert_eq!(Ok(8), encoder.max_output_len(2));
    assert_eq!(0, encoder.max_finish_output_len());
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        encoder.max_output_len(usize::MAX),
    );
    encoder.reset();
}

#[test]
fn test_buffered_encode_engine_delegates_finish_to_hooks() {
    let mut encoder = BufferedEncodeEngine::<_, _>::new(WideCodec, FinishHooks::default());
    let mut output = [0_u8; 1];

    assert_eq!(1, encoder.max_finish_output_len());

    let error = encoder
        .finish(&mut [], 0)
        .expect_err("finish should reject insufficient output before calling hooks");
    assert_eq!(
        FinishError::InsufficientOutput {
            output_index: 0,
            required: 1,
            available: 0,
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

    let mut encoder = BufferedEncodeEngine::<_, _>::new(WideCodec, FinishHooks::default());
    encoder.reset();
    assert_eq!(0, encoder.max_finish_output_len());
}

#[test]
#[should_panic]
fn test_buffered_encode_engine_finish_passes_bounded_output_to_hooks() {
    let mut encoder = BufferedEncodeEngine::<_, _>::new(WideCodec, OverwritingFinishHooks);
    let mut output = [0_u8; 2];

    let _ = encoder.finish(&mut output, 0);
}

#[test]
#[should_panic(expected = "BufferedEncodeEngine hook wrote beyond its finish bound")]
fn test_buffered_encode_engine_finish_panics_when_hook_overreports_bound() {
    let mut encoder = BufferedEncodeEngine::<_, _>::new(WideCodec, OverreportingFinishHooks);
    let mut output = [0_u8; 2];

    let _ = encoder.finish(&mut output, 0);
}

#[test]
fn test_buffered_encode_engine_finish_reports_output_index_beyond_buffer() {
    let mut encoder = BufferedEncodeEngine::<_, _>::new(WideCodec, FinishHooks::default());
    let mut output = [];

    let error = encoder
        .finish(&mut output, 1)
        .expect_err("out-of-range finish output index should be rejected");

    assert_eq!(FinishError::InvalidOutputIndex { index: 1, len: 0 }, error);
}

#[test]
fn test_buffered_encode_engine_default_finish_reports_output_index_beyond_buffer() {
    let mut encoder = BufferedEncodeEngine::<_, _>::new(WideCodec, ExactWidthHooks);
    let mut output = [];

    let error = encoder
        .finish(&mut output, 1)
        .expect_err("default finish should reject out-of-range output index");

    assert_eq!(FinishError::InvalidOutputIndex { index: 1, len: 0 }, error);
}

#[test]
fn test_buffered_encode_hooks_default_finish_is_noop() {
    let mut hooks = ExactWidthHooks;
    let mut output = [];

    let written = BufferedEncodeHooks::<WideCodec>::finish(&mut hooks, &WideCodec, &mut output, 1)
        .expect("default hook finish should be a no-op");

    assert_eq!(0, written);
}

#[test]
fn test_buffered_encode_engine_uses_plan_capacity_instead_of_codec_max_width() {
    let mut encoder = BufferedEncodeEngine::new(WideCodec, ExactWidthHooks);
    let mut output = [0_u8; 1];

    let progress = encoder
        .transcode(&[1, 2], 0, &mut output, 0)
        .expect("engine encoding should succeed");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 1,
            additional: super::nz(1),
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
fn test_buffered_encode_engine_allows_zero_width_plan_to_consume_input() {
    let mut encoder = BufferedEncodeEngine::new(WideCodec, SkippingHooks);
    let mut output = [];

    let progress = encoder
        .transcode(&[1, 2, 3], 0, &mut output, 0)
        .expect("zero-width plan should not need output");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(3, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_buffered_encode_engine_reports_output_index_beyond_buffer() {
    let mut encoder = BufferedEncodeEngine::new(WideCodec, ExactWidthHooks);
    let mut output = [];

    let error = encoder
        .transcode(&[1], 0, &mut output, 1)
        .expect_err("out-of-range output index should fail");

    assert_eq!(
        EngineError::InvalidOutputIndex {
            index: 1,
            output_len: 0,
        },
        error,
    );
}

#[test]
#[should_panic(expected = "BufferedEncodeEngine hook wrote beyond its prepared capacity bound")]
fn test_buffered_encode_engine_panics_when_hook_reports_too_many_written_units() {
    let mut encoder = BufferedEncodeEngine::new(WideCodec, OverreportingWriteHooks);
    let mut output = [0_u8; 1];

    let _ = encoder.transcode(&[1], 0, &mut output, 0);
}

#[test]
fn test_buffered_encode_engine_propagates_prepare_error_without_consuming_input() {
    let mut encoder = BufferedEncodeEngine::new(WideCodec, RejectingHooks);
    let mut output = [0_u8; 4];

    let error = encoder
        .transcode(&[1], 0, &mut output, 0)
        .expect_err("prepare hook error should be propagated");

    assert_eq!(EngineError::Rejected { input_index: 0 }, error);
    assert_eq!([0, 0, 0, 0], output);
}

#[test]
fn test_buffered_encode_engine_uses_hooks_for_invalid_input_index() {
    let mut encoder = BufferedEncodeEngine::new(WideCodec, ExactWidthHooks);
    let mut output = [];

    let error = encoder
        .transcode(&[1], 2, &mut output, 0)
        .expect_err("invalid input index should be rejected");

    assert_eq!(
        EngineError::InvalidInputIndex {
            index: 2,
            input_len: 1
        },
        error,
    );
}

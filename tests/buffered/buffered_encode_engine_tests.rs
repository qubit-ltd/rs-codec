/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for the reusable buffered encoder engine.

use qubit_codec::{
    BufferedEncodeEngine,
    BufferedEncodeHooks,
    CapacityError,
    Codec,
    EncodePlan,
    TranscodeStatus,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct WideCodec;

unsafe impl Codec<u8, u8> for WideCodec {
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

    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
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
    Rejected { input_index: usize },
}

impl EngineError {
    const fn invalid_input_index(index: usize, input_len: usize) -> Self {
        Self::InvalidInputIndex { index, input_len }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ExactWidthHooks;

impl BufferedEncodeHooks<WideCodec, u8, u8> for ExactWidthHooks {
    type Error = EngineError;
    type PlanPayload = ();

    fn prepare_encode(
        &mut self,
        _codec: &WideCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanPayload>, Self::Error> {
        Ok(EncodePlan::new(1, ()))
    }

    unsafe fn write_encode(
        &mut self,
        _codec: &WideCodec,
        value: &u8,
        _input_index: usize,
        _plan_payload: Self::PlanPayload,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        debug_assert!(output_index < output.len());

        // SAFETY: The engine checked the one-unit capacity requested by
        // `prepare_encode`.
        unsafe {
            *output.as_mut_ptr().add(output_index) = value.wrapping_add(10);
        }
        Ok(1)
    }

    fn invalid_input_index(&mut self, _codec: &WideCodec, index: usize, input_len: usize) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct SkippingHooks;

impl BufferedEncodeHooks<WideCodec, u8, u8> for SkippingHooks {
    type Error = EngineError;
    type PlanPayload = ();

    fn prepare_encode(
        &mut self,
        _codec: &WideCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanPayload>, Self::Error> {
        Ok(EncodePlan::new(0, ()))
    }

    unsafe fn write_encode(
        &mut self,
        _codec: &WideCodec,
        _value: &u8,
        _input_index: usize,
        _plan_payload: Self::PlanPayload,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn invalid_input_index(&mut self, _codec: &WideCodec, index: usize, input_len: usize) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RejectingHooks;

impl BufferedEncodeHooks<WideCodec, u8, u8> for RejectingHooks {
    type Error = EngineError;
    type PlanPayload = ();

    fn prepare_encode(
        &mut self,
        _codec: &WideCodec,
        _value: &u8,
        input_index: usize,
    ) -> Result<EncodePlan<Self::PlanPayload>, Self::Error> {
        Err(EngineError::Rejected { input_index })
    }

    unsafe fn write_encode(
        &mut self,
        _codec: &WideCodec,
        _value: &u8,
        _input_index: usize,
        _plan_payload: Self::PlanPayload,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        unreachable!("prepare_encode rejects every value")
    }

    fn invalid_input_index(&mut self, _codec: &WideCodec, index: usize, input_len: usize) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FinishHooks {
    pending_suffix: bool,
}

impl Default for FinishHooks {
    fn default() -> Self {
        Self { pending_suffix: true }
    }
}

impl BufferedEncodeHooks<WideCodec, u8, u8> for FinishHooks {
    type Error = EngineError;
    type PlanPayload = ();

    fn prepare_encode(
        &mut self,
        _codec: &WideCodec,
        _value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanPayload>, Self::Error> {
        Ok(EncodePlan::new(1, ()))
    }

    unsafe fn write_encode(
        &mut self,
        _codec: &WideCodec,
        value: &u8,
        _input_index: usize,
        _plan_payload: Self::PlanPayload,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        output[output_index] = *value;
        Ok(1)
    }

    fn invalid_input_index(&mut self, _codec: &WideCodec, index: usize, input_len: usize) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
    }

    fn max_finish_output_len(&self, _codec: &WideCodec) -> usize {
        usize::from(self.pending_suffix)
    }

    fn finish(
        &mut self,
        _codec: &WideCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<qubit_codec::TranscodeProgress, Self::Error> {
        if !self.pending_suffix {
            return Ok(qubit_codec::TranscodeProgress::complete(0, 0));
        }

        let available = output.len().saturating_sub(output_index);
        if available == 0 {
            let status = TranscodeStatus::NeedOutput {
                output_index,
                additional: 1,
                available,
            };
            return Ok(qubit_codec::TranscodeProgress::new(status, 0, 0));
        }

        output[output_index] = 0xee;
        self.pending_suffix = false;
        Ok(qubit_codec::TranscodeProgress::complete(0, 1))
    }

    fn reset(&mut self, _codec: &WideCodec) {
        self.pending_suffix = false;
    }
}

#[test]
fn test_buffered_encode_engine_reports_bounds_and_resets() {
    let mut encoder = BufferedEncodeEngine::new(WideCodec, ExactWidthHooks);

    assert_eq!(Ok(8), encoder.max_output_len::<u8, u8>(2));
    assert_eq!(0, encoder.max_finish_output_len::<u8, u8>());
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        encoder.max_output_len::<u8, u8>(usize::MAX),
    );
    encoder.reset::<u8, u8>();
}

#[test]
fn test_buffered_encode_engine_delegates_finish_to_hooks() {
    let mut encoder = BufferedEncodeEngine::new(WideCodec, FinishHooks::default());
    let mut output = [0_u8; 1];

    assert_eq!(1, encoder.max_finish_output_len::<u8, u8>());

    let progress = encoder
        .finish::<u8, u8>(&mut [], 0)
        .expect("hook should request output for pending finish output");
    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: 1,
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(1, encoder.max_finish_output_len::<u8, u8>());

    let progress = encoder
        .finish::<u8, u8>(&mut output, 0)
        .expect("hook should write final output");
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(0, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([0xee], output);
    assert_eq!(0, encoder.max_finish_output_len::<u8, u8>());

    let mut encoder = BufferedEncodeEngine::new(WideCodec, FinishHooks::default());
    encoder.reset::<u8, u8>();
    assert_eq!(0, encoder.max_finish_output_len::<u8, u8>());
}

#[test]
fn test_buffered_encode_engine_finish_reports_output_index_beyond_buffer() {
    let mut encoder = BufferedEncodeEngine::new(WideCodec, FinishHooks::default());
    let mut output = [];

    let progress = encoder
        .finish::<u8, u8>(&mut output, 1)
        .expect("out-of-range finish output index should request capacity");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 1,
            additional: 1,
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_buffered_encode_engine_default_finish_reports_output_index_beyond_buffer() {
    let mut encoder = BufferedEncodeEngine::new(WideCodec, ExactWidthHooks);
    let mut output = [];

    let progress = encoder
        .finish::<u8, u8>(&mut output, 1)
        .expect("default finish should report out-of-range output index");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 1,
            additional: 1,
            available: 0,
        },
        progress.status(),
    );
}

#[test]
fn test_buffered_encode_hooks_default_finish_reports_output_index_beyond_buffer() {
    let mut hooks = ExactWidthHooks;
    let mut output = [];

    let progress = BufferedEncodeHooks::<WideCodec, u8, u8>::finish(&mut hooks, &WideCodec, &mut output, 1)
        .expect("default hook finish should report out-of-range output index");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 1,
            additional: 1,
            available: 0,
        },
        progress.status(),
    );
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
            additional: 1,
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(1, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([11], output);
    assert_eq!(Ok(8), encoder.max_output_len::<u8, u8>(2));
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

    let progress = encoder
        .transcode(&[1], 0, &mut output, 1)
        .expect("out-of-range output index should request capacity");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 1,
            additional: 4,
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
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

    assert_eq!(EngineError::InvalidInputIndex { index: 2, input_len: 1 }, error,);
}

/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Tests for the reusable buffered converter engine.

use core::num::NonZeroUsize;

use qubit_codec::{
    BufferedConvertEngine,
    BufferedConvertHooks,
    ConvertDecodeResult,
    ConvertErrorFactory,
    ConvertState,
    ConvertWriteResult,
    TranscodeProgress,
    TranscodeStatus,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Source {
    reset: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Target {
    reset: bool,
    finish_suffix: Option<u8>,
}

impl Default for Target {
    fn default() -> Self {
        Self {
            reset: false,
            finish_suffix: Some(0xee),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EngineError {
    InvalidInputIndex { index: usize, input_len: usize },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct CopyHooks {
    pending: Option<u8>,
}

impl ConvertErrorFactory<Source> for EngineError {
    fn invalid_input_index(_decoder: &Source, index: usize, input_len: usize) -> Self {
        Self::InvalidInputIndex { index, input_len }
    }
}

impl CopyHooks {
    fn write_pending(
        &mut self,
        state: &mut ConvertState<'_, u8, u8>,
    ) -> Result<Option<TranscodeProgress>, EngineError> {
        let Some(value) = self.pending else {
            return Ok(None);
        };
        if state.available_output() == 0 {
            return Ok(Some(state.need_output_progress(NonZeroUsize::MIN, 0)));
        }

        let output_cursor = state.output_cursor();
        state.output_mut()[output_cursor] = value;
        state.advance_output(1);
        self.pending = None;
        Ok(None)
    }
}

impl BufferedConvertHooks<Source, Target, u8, u8, u8> for CopyHooks {
    type Error = EngineError;

    fn max_output_len(&self, _decoder: &Source, _encoder: &Target, input_len: usize) -> Option<usize> {
        input_len.checked_add(usize::from(self.pending.is_some()))
    }

    fn max_finish_output_len(&self, _decoder: &Source, encoder: &Target) -> Option<usize> {
        Some(usize::from(encoder.finish_suffix.is_some()))
    }

    fn reset(&mut self, decoder: &mut Source, encoder: &mut Target) {
        self.pending = None;
        decoder.reset = true;
        encoder.reset = true;
    }

    fn drain_pending(
        &mut self,
        _decoder: &mut Source,
        _encoder: &mut Target,
        state: &mut ConvertState<'_, u8, u8>,
    ) -> Result<Option<TranscodeProgress>, Self::Error> {
        self.write_pending(state)
    }

    fn decode_next(
        &mut self,
        _decoder: &mut Source,
        state: &mut ConvertState<'_, u8, u8>,
    ) -> Result<ConvertDecodeResult<u8>, Self::Error> {
        let input_cursor = state.input_cursor();
        Ok(ConvertDecodeResult::Decoded {
            value: state.input()[input_cursor].wrapping_add(1),
            consumed: NonZeroUsize::MIN,
        })
    }

    fn write_value(
        &mut self,
        _encoder: &mut Target,
        value: u8,
        state: &mut ConvertState<'_, u8, u8>,
    ) -> Result<ConvertWriteResult, Self::Error> {
        if state.available_output() == 0 {
            self.pending = Some(value);
            return Ok(ConvertWriteResult::NeedOutput {
                additional: NonZeroUsize::MIN,
                available: 0,
                written: 0,
            });
        }

        let output_cursor = state.output_cursor();
        state.output_mut()[output_cursor] = value;
        Ok(ConvertWriteResult::Written { written: 1 })
    }

    fn finish(
        &mut self,
        _decoder: &mut Source,
        encoder: &mut Target,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let Some(value) = encoder.finish_suffix else {
            return Ok(TranscodeProgress::complete(0, 0));
        };
        if output_index == output.len() {
            return Ok(TranscodeProgress::need_output(output_index, 1, 0, 0, 0));
        }

        output[output_index] = value;
        encoder.finish_suffix = None;
        Ok(TranscodeProgress::complete(0, 1))
    }
}

#[test]
fn test_buffered_convert_engine_exposes_accessors_and_reset() {
    let mut engine =
        BufferedConvertEngine::<_, _, _, u8>::new(Source::default(), Target::default(), CopyHooks::default());

    assert_eq!(&Source::default(), engine.decoder());
    assert_eq!(&Target::default(), engine.encoder());
    assert_eq!(&CopyHooks::default(), engine.hooks());
    assert_eq!(&mut Source::default(), engine.decoder_mut());
    assert_eq!(&mut Target::default(), engine.encoder_mut());
    assert_eq!(&mut CopyHooks::default(), engine.hooks_mut());
    assert_eq!(Some(3), engine.max_output_len::<u8, u8>(3));
    assert_eq!(Some(1), engine.max_finish_output_len::<u8, u8>());

    engine.reset::<u8, u8>();
    assert!(engine.decoder().reset);
    assert!(engine.encoder().reset);

    let (decoder, encoder, hooks) = engine.into_parts();
    assert!(decoder.reset);
    assert!(encoder.reset);
    assert_eq!(CopyHooks::default(), hooks);
}

#[test]
fn test_buffered_convert_engine_drains_pending_before_input() {
    let hooks = CopyHooks { pending: Some(100) };
    let mut engine = BufferedConvertEngine::<_, _, _, u8>::new(Source::default(), Target::default(), hooks);
    let mut output = [0_u8; 2];

    let progress = engine
        .transcode::<u8, u8>(&[1, 2], 0, &mut output, 0)
        .expect("copy hooks should not fail");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 2,
            additional: 1,
            available: 0,
        },
        progress.status()
    );
    assert_eq!((2, 2), (progress.read(), progress.written()));
    assert_eq!([100, 2], output);
}

#[test]
fn test_buffered_convert_engine_reports_invalid_indices() {
    let mut engine =
        BufferedConvertEngine::<_, _, _, u8>::new(Source::default(), Target::default(), CopyHooks::default());
    let mut output = [0_u8; 1];

    let error = engine
        .transcode::<u8, u8>(&[1], 2, &mut output, 0)
        .expect_err("invalid input index should fail");
    assert_eq!(EngineError::InvalidInputIndex { index: 2, input_len: 1 }, error);

    let progress = engine
        .transcode::<u8, u8>(&[1], 0, &mut output, 2)
        .expect("invalid output index is reported as NeedOutput");
    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 2,
            additional: 1,
            available: 0,
        },
        progress.status()
    );
}

#[test]
fn test_buffered_convert_engine_delegates_finish_to_hooks() {
    let mut engine =
        BufferedConvertEngine::<_, _, _, u8>::new(Source::default(), Target::default(), CopyHooks::default());

    let progress = engine
        .finish::<u8, u8>(&mut [], 0)
        .expect("finish should request output for pending suffix");
    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: 1,
            available: 0,
        },
        progress.status()
    );

    let mut output = [0_u8; 1];
    let progress = engine
        .finish::<u8, u8>(&mut output, 0)
        .expect("finish should write pending suffix");
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((0, 1), (progress.read(), progress.written()));
    assert_eq!([0xee], output);
}

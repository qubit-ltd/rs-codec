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
    BufferedDecodeHooks,
    BufferedEncodeHooks,
    CapacityError,
    Codec,
    ConvertErrorFactory,
    DecodeAction,
    DecodeContext,
    DecodeErrorFactory,
    EncodeErrorFactory,
    EncodePlan,
    TranscodeProgress,
    TranscodeStatus,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct SourceCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct TargetCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EngineError {
    InvalidInputIndex { index: usize, input_len: usize },
    Encode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConvertEngineError<E> {
    Decode(EngineError),
    Encode(E),
}

unsafe impl Codec<u8, u8> for SourceCodec {
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    unsafe fn decode_unchecked(&self, input: &[u8], index: usize) -> Result<(u8, NonZeroUsize), Self::DecodeError> {
        // SAFETY: The caller proved that at least one input unit is readable.
        let value = unsafe { *input.get_unchecked(index) };
        Ok((value.wrapping_add(1), NonZeroUsize::MIN))
    }

    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        // SAFETY: The caller proved that one output unit is writable.
        unsafe {
            *output.get_unchecked_mut(index) = *value;
        }
        Ok(1)
    }
}

unsafe impl Codec<u8, u8> for TargetCodec {
    type DecodeError = core::convert::Infallible;
    type EncodeError = EngineError;

    fn min_units_per_value(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    unsafe fn decode_unchecked(&self, input: &[u8], index: usize) -> Result<(u8, NonZeroUsize), Self::DecodeError> {
        // SAFETY: The caller proved that at least one input unit is readable.
        let value = unsafe { *input.get_unchecked(index) };
        Ok((value, NonZeroUsize::MIN))
    }

    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        if *value == 13 {
            return Err(EngineError::Encode);
        }
        // SAFETY: The caller proved that one output unit is writable.
        unsafe {
            *output.get_unchecked_mut(index) = *value;
        }
        Ok(1)
    }
}

impl ConvertErrorFactory<SourceCodec> for EngineError {
    fn invalid_input_index(_decoder: &SourceCodec, index: usize, input_len: usize) -> Self {
        Self::InvalidInputIndex { index, input_len }
    }
}

impl<E> ConvertErrorFactory<SourceCodec> for ConvertEngineError<E> {
    fn invalid_input_index(_decoder: &SourceCodec, index: usize, input_len: usize) -> Self {
        Self::Decode(EngineError::InvalidInputIndex { index, input_len })
    }
}

impl DecodeErrorFactory<SourceCodec> for EngineError {
    fn invalid_input_index(_decoder: &SourceCodec, index: usize, input_len: usize) -> Self {
        Self::InvalidInputIndex { index, input_len }
    }
}

impl EncodeErrorFactory<TargetCodec> for EngineError {
    fn invalid_input_index(_codec: &TargetCodec, index: usize, input_len: usize) -> Self {
        Self::InvalidInputIndex { index, input_len }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct StrictDecodeHooks;

impl BufferedDecodeHooks<SourceCodec, u8, u8> for StrictDecodeHooks {
    type Error = EngineError;

    fn handle_decode_error(
        &mut self,
        _codec: &SourceCodec,
        error: core::convert::Infallible,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        match error {}
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct StrictEncodeHooks;

impl BufferedEncodeHooks<TargetCodec, u8, u8> for StrictEncodeHooks {
    type Error = EngineError;
    type PlanPayload = ();

    fn prepare_encode(
        &mut self,
        codec: &TargetCodec,
        _input_value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanPayload>, Self::Error> {
        Ok(EncodePlan::new(codec.max_units_per_value().get(), ()))
    }

    unsafe fn write_encode(
        &mut self,
        codec: &TargetCodec,
        input_value: &u8,
        _input_index: usize,
        _plan_payload: Self::PlanPayload,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        // SAFETY: The engine checked the prepared output capacity.
        unsafe { codec.encode_unchecked(input_value, output, output_index) }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct CopyHooks {
    reset_called: bool,
}

impl BufferedConvertHooks<SourceCodec, TargetCodec, u8, u8> for CopyHooks {
    type DecodeHooks = StrictDecodeHooks;
    type EncodeHooks = StrictEncodeHooks;
    type EncodeError<Output>
        = EngineError
    where
        TargetCodec: Codec<u8, Output>,
        Output: Copy;
    type Error<Output>
        = ConvertEngineError<EngineError>
    where
        TargetCodec: Codec<u8, Output>,
        Output: Copy,
        StrictEncodeHooks: BufferedEncodeHooks<TargetCodec, u8, Output, Error = Self::EncodeError<Output>>;

    fn create_decode_hooks(&self, _decoder: &SourceCodec, _encoder: &TargetCodec) -> Self::DecodeHooks {
        StrictDecodeHooks
    }

    fn create_encode_hooks(&self, _decoder: &SourceCodec, _encoder: &TargetCodec) -> Self::EncodeHooks {
        StrictEncodeHooks
    }

    fn map_decode_error<Output>(&self, error: EngineError) -> Self::Error<Output>
    where
        TargetCodec: Codec<u8, Output>,
        Output: Copy,
        StrictEncodeHooks: BufferedEncodeHooks<TargetCodec, u8, Output, Error = Self::EncodeError<Output>>,
    {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error<Output>(&self, error: Self::EncodeError<Output>) -> Self::Error<Output>
    where
        TargetCodec: Codec<u8, Output>,
        Output: Copy,
        StrictEncodeHooks: BufferedEncodeHooks<TargetCodec, u8, Output, Error = Self::EncodeError<Output>>,
    {
        ConvertEngineError::Encode(error)
    }

    fn reset(&mut self) {
        self.reset_called = true;
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FinishDecodeHooks {
    value: Option<u8>,
}

impl Default for FinishDecodeHooks {
    fn default() -> Self {
        Self { value: Some(40) }
    }
}

impl BufferedDecodeHooks<SourceCodec, u8, u8> for FinishDecodeHooks {
    type Error = EngineError;

    fn max_finish_output_len(&self, _codec: &SourceCodec) -> usize {
        usize::from(self.value.is_some())
    }

    fn handle_decode_error(
        &mut self,
        _codec: &SourceCodec,
        error: core::convert::Infallible,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        match error {}
    }

    fn finish(
        &mut self,
        _codec: &SourceCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if output_index > output.len() {
            return Ok(TranscodeProgress::need_output(output_index, 1, 0, 0, 0));
        }
        let Some(value) = self.value else {
            return Ok(TranscodeProgress::complete(0, 0));
        };
        if output_index == output.len() {
            return Ok(TranscodeProgress::need_output(output_index, 1, 0, 0, 0));
        }
        output[output_index] = value;
        self.value = None;
        Ok(TranscodeProgress::complete(0, 1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FinishHooks;

impl BufferedConvertHooks<SourceCodec, TargetCodec, u8, u8> for FinishHooks {
    type DecodeHooks = FinishDecodeHooks;
    type EncodeHooks = StrictEncodeHooks;
    type EncodeError<Output>
        = EngineError
    where
        TargetCodec: Codec<u8, Output>,
        Output: Copy;
    type Error<Output>
        = ConvertEngineError<EngineError>
    where
        TargetCodec: Codec<u8, Output>,
        Output: Copy,
        StrictEncodeHooks: BufferedEncodeHooks<TargetCodec, u8, Output, Error = Self::EncodeError<Output>>;

    fn create_decode_hooks(&self, _decoder: &SourceCodec, _encoder: &TargetCodec) -> Self::DecodeHooks {
        FinishDecodeHooks::default()
    }

    fn create_encode_hooks(&self, _decoder: &SourceCodec, _encoder: &TargetCodec) -> Self::EncodeHooks {
        StrictEncodeHooks
    }

    fn map_decode_error<Output>(&self, error: EngineError) -> Self::Error<Output>
    where
        TargetCodec: Codec<u8, Output>,
        Output: Copy,
        StrictEncodeHooks: BufferedEncodeHooks<TargetCodec, u8, Output, Error = Self::EncodeError<Output>>,
    {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error<Output>(&self, error: Self::EncodeError<Output>) -> Self::Error<Output>
    where
        TargetCodec: Codec<u8, Output>,
        Output: Copy,
        StrictEncodeHooks: BufferedEncodeHooks<TargetCodec, u8, Output, Error = Self::EncodeError<Output>>,
    {
        ConvertEngineError::Encode(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FactoryDecodeHooks {
    marker: u8,
}

impl BufferedDecodeHooks<SourceCodec, u8, u8> for FactoryDecodeHooks {
    type Error = EngineError;

    fn max_output_len(&self, _codec: &SourceCodec, _input_len: usize) -> Result<usize, CapacityError> {
        Ok(self.marker as usize)
    }

    fn handle_decode_error(
        &mut self,
        _codec: &SourceCodec,
        error: core::convert::Infallible,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        match error {}
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FactoryEncodeHooks {
    offset: u8,
}

impl BufferedEncodeHooks<TargetCodec, u8, u8> for FactoryEncodeHooks {
    type Error = EngineError;
    type PlanPayload = ();

    fn prepare_encode(
        &mut self,
        codec: &TargetCodec,
        _input_value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanPayload>, Self::Error> {
        Ok(EncodePlan::new(codec.max_units_per_value().get(), ()))
    }

    unsafe fn write_encode(
        &mut self,
        _codec: &TargetCodec,
        input_value: &u8,
        _input_index: usize,
        _plan_payload: Self::PlanPayload,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        output[output_index] = input_value.wrapping_add(self.offset);
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FactoryHooks {
    decode_marker: u8,
    encode_offset: u8,
}

impl BufferedConvertHooks<SourceCodec, TargetCodec, u8, u8> for FactoryHooks {
    type DecodeHooks = FactoryDecodeHooks;
    type EncodeHooks = FactoryEncodeHooks;
    type EncodeError<Output>
        = EngineError
    where
        TargetCodec: Codec<u8, Output>,
        Output: Copy;
    type Error<Output>
        = ConvertEngineError<EngineError>
    where
        TargetCodec: Codec<u8, Output>,
        Output: Copy,
        FactoryEncodeHooks: BufferedEncodeHooks<TargetCodec, u8, Output, Error = Self::EncodeError<Output>>;

    fn create_decode_hooks(&self, _decoder: &SourceCodec, _encoder: &TargetCodec) -> Self::DecodeHooks {
        FactoryDecodeHooks {
            marker: self.decode_marker,
        }
    }

    fn create_encode_hooks(&self, _decoder: &SourceCodec, _encoder: &TargetCodec) -> Self::EncodeHooks {
        FactoryEncodeHooks {
            offset: self.encode_offset,
        }
    }

    fn map_decode_error<Output>(&self, error: EngineError) -> Self::Error<Output>
    where
        TargetCodec: Codec<u8, Output>,
        Output: Copy,
        FactoryEncodeHooks: BufferedEncodeHooks<TargetCodec, u8, Output, Error = Self::EncodeError<Output>>,
    {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error<Output>(&self, error: Self::EncodeError<Output>) -> Self::Error<Output>
    where
        TargetCodec: Codec<u8, Output>,
        Output: Copy,
        FactoryEncodeHooks: BufferedEncodeHooks<TargetCodec, u8, Output, Error = Self::EncodeError<Output>>,
    {
        ConvertEngineError::Encode(error)
    }
}

#[test]
fn test_buffered_convert_engine_reports_bounds_and_resets() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(SourceCodec, TargetCodec, CopyHooks::default());

    assert_eq!(Ok(3), engine.max_output_len::<u8>(3));
    assert_eq!(Ok(0), engine.max_finish_output_len::<u8>());

    engine.reset::<u8>();
    assert_eq!(Ok(0), engine.max_finish_output_len::<u8>());
}

#[test]
fn test_buffered_convert_engine_new_uses_convert_hook_factories() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(
        SourceCodec,
        TargetCodec,
        FactoryHooks {
            decode_marker: 11,
            encode_offset: 7,
        },
    );

    assert_eq!(Ok(11), engine.max_output_len::<u8>(1));

    let mut output = [0_u8; 1];
    let progress = engine
        .transcode::<u8>(&[1], 0, &mut output, 0)
        .expect("factory-created encode hooks should convert the value");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((1, 1), (progress.read(), progress.written()));
    assert_eq!([9], output);
}

#[test]
fn test_buffered_convert_engine_owns_pending_value_between_calls() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(SourceCodec, TargetCodec, CopyHooks::default());
    let mut empty_output = [0_u8; 0];

    let progress = engine
        .transcode::<u8>(&[1], 0, &mut empty_output, 0)
        .expect("conversion should retain decoded value when output is empty");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: 1,
            available: 0,
        },
        progress.status(),
    );
    assert_eq!((1, 0), (progress.read(), progress.written()));
    assert_eq!(Ok(2), engine.max_output_len::<u8>(1));
    assert_eq!(Ok(1), engine.max_finish_output_len::<u8>());

    let mut output = [0_u8; 2];
    let progress = engine
        .transcode::<u8>(&[9], 0, &mut output, 0)
        .expect("conversion should drain pending before reading new input");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((1, 2), (progress.read(), progress.written()));
    assert_eq!([2, 10], output);
    assert_eq!(Ok(0), engine.max_finish_output_len::<u8>());
}

#[test]
fn test_buffered_convert_engine_reports_invalid_indices() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(SourceCodec, TargetCodec, CopyHooks::default());
    let mut output = [0_u8; 1];

    let error = engine
        .transcode::<u8>(&[1], 2, &mut output, 0)
        .expect_err("invalid input index should fail");
    assert_eq!(
        ConvertEngineError::Decode(EngineError::InvalidInputIndex { index: 2, input_len: 1 }),
        error,
    );

    let progress = engine
        .transcode::<u8>(&[1], 0, &mut output, 2)
        .expect("invalid output index is reported as NeedOutput");
    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 2,
            additional: 1,
            available: 0,
        },
        progress.status(),
    );
}

#[test]
fn test_buffered_convert_engine_finish_drains_pending_value() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(SourceCodec, TargetCodec, CopyHooks::default());
    let mut empty_output = [0_u8; 0];
    let progress = engine
        .transcode::<u8>(&[4], 0, &mut empty_output, 0)
        .expect("conversion should retain decoded value");
    assert!(matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));

    let progress = engine
        .finish::<u8>(&mut empty_output, 0)
        .expect("finish should keep pending value when output is empty");
    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: 1,
            available: 0,
        },
        progress.status(),
    );
    assert_eq!((0, 0), (progress.read(), progress.written()));

    let mut output = [0_u8; 1];
    let progress = engine
        .finish::<u8>(&mut output, 0)
        .expect("finish should write pending value");
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((0, 1), (progress.read(), progress.written()));
    assert_eq!([5], output);
}

#[test]
fn test_buffered_convert_engine_finish_encodes_decoder_finish_output() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(SourceCodec, TargetCodec, FinishHooks);
    assert_eq!(Ok(1), engine.max_finish_output_len::<u8>());

    let mut empty_output = [0_u8; 0];
    let progress = engine
        .finish::<u8>(&mut empty_output, 0)
        .expect("finish should retain decoder finish value when target output is empty");
    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: 1,
            available: 0,
        },
        progress.status(),
    );
    assert_eq!((0, 0), (progress.read(), progress.written()));
    assert_eq!(Ok(1), engine.max_finish_output_len::<u8>());

    let mut output = [0_u8; 1];
    let progress = engine
        .finish::<u8>(&mut output, 0)
        .expect("finish should encode decoder finish value");
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((0, 1), (progress.read(), progress.written()));
    assert_eq!([40], output);
    assert_eq!(Ok(0), engine.max_finish_output_len::<u8>());
}

#[test]
fn test_buffered_convert_engine_finish_drains_pending_before_decoder_finish_output() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(SourceCodec, TargetCodec, FinishHooks);
    let mut empty_output = [0_u8; 0];
    let progress = engine
        .transcode::<u8>(&[4], 0, &mut empty_output, 0)
        .expect("conversion should retain decoded input value");
    assert!(matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));
    assert_eq!(Ok(2), engine.max_finish_output_len::<u8>());

    let mut output = [0_u8; 1];
    let progress = engine
        .finish::<u8>(&mut output, 0)
        .expect("finish should write pending input value first");
    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 1,
            additional: 1,
            available: 0,
        },
        progress.status(),
    );
    assert_eq!((0, 1), (progress.read(), progress.written()));
    assert_eq!([5], output);
    assert_eq!(Ok(1), engine.max_finish_output_len::<u8>());

    let mut output = [0_u8; 1];
    let progress = engine
        .finish::<u8>(&mut output, 0)
        .expect("finish should then write decoder finish value");
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((0, 1), (progress.read(), progress.written()));
    assert_eq!([40], output);
}

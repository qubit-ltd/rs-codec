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
    EncodeErrorFactory,
    EncodePlan,
    TranscodeProgress,
    TranscodeStatus,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct SourceCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct TargetCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ErrorSourceCodec;

fn one_consumed() -> NonZeroUsize {
    NonZeroUsize::MIN
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EngineError {
    InvalidInputIndex { index: usize, input_len: usize },
    Decode,
    Encode,
}

impl EngineError {
    fn invalid_input_index(index: usize, input_len: usize) -> Self {
        Self::InvalidInputIndex { index, input_len }
    }
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

unsafe impl Codec<u8, u8> for ErrorSourceCodec {
    type DecodeError = EngineError;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    unsafe fn decode_unchecked(&self, _input: &[u8], _index: usize) -> Result<(u8, NonZeroUsize), Self::DecodeError> {
        Err(EngineError::Decode)
    }

    unsafe fn encode_unchecked(&self, value: &u8, output: &mut [u8], index: usize) -> Result<usize, Self::EncodeError> {
        // SAFETY: The caller proved that one output unit is writable.
        unsafe {
            *output.get_unchecked_mut(index) = *value;
        }
        Ok(1)
    }
}

impl ConvertErrorFactory<SourceCodec> for EngineError {
    fn invalid_input_index(_decoder: &SourceCodec, index: usize, input_len: usize) -> Self {
        Self::invalid_input_index(index, input_len)
    }
}

impl<E> ConvertErrorFactory<SourceCodec> for ConvertEngineError<E> {
    fn invalid_input_index(_decoder: &SourceCodec, index: usize, input_len: usize) -> Self {
        Self::Decode(EngineError::invalid_input_index(index, input_len))
    }
}

impl<E> ConvertErrorFactory<ErrorSourceCodec> for ConvertEngineError<E> {
    fn invalid_input_index(_decoder: &ErrorSourceCodec, index: usize, input_len: usize) -> Self {
        Self::Decode(EngineError::invalid_input_index(index, input_len))
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

    fn invalid_input_index(&mut self, _codec: &SourceCodec, index: usize, input_len: usize) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RepairAction {
    Emit,
    NeedInput,
    Skip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RepairDecodeHooks {
    action: RepairAction,
}

impl BufferedDecodeHooks<ErrorSourceCodec, u8, u8> for RepairDecodeHooks {
    type Error = EngineError;

    fn handle_decode_error(
        &mut self,
        _codec: &ErrorSourceCodec,
        _error: EngineError,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        match self.action {
            RepairAction::Emit => Ok(DecodeAction::Emit {
                value: 42,
                consumed: one_consumed(),
            }),
            RepairAction::NeedInput => Ok(DecodeAction::NeedInput { required_total: 3 }),
            RepairAction::Skip => Ok(DecodeAction::Skip {
                consumed: one_consumed(),
            }),
        }
    }

    fn invalid_input_index(&mut self, _codec: &ErrorSourceCodec, index: usize, input_len: usize) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RepairHooks {
    action: RepairAction,
}

impl BufferedConvertHooks<ErrorSourceCodec, TargetCodec, u8, u8> for RepairHooks {
    type DecodeHooks = RepairDecodeHooks;
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

    fn create_decode_hooks(&self, _decoder: &ErrorSourceCodec, _encoder: &TargetCodec) -> Self::DecodeHooks {
        RepairDecodeHooks { action: self.action }
    }

    fn create_encode_hooks(&self, _decoder: &ErrorSourceCodec, _encoder: &TargetCodec) -> Self::EncodeHooks {
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

    fn invalid_input_index(&mut self, _codec: &SourceCodec, index: usize, input_len: usize) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FinishHooks {
    value: u8,
}

impl Default for FinishHooks {
    fn default() -> Self {
        Self { value: 40 }
    }
}

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
        FinishDecodeHooks {
            value: Some(self.value),
        }
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
struct BatchFinishDecodeHooks {
    next: u8,
    remaining: u8,
}

impl Default for BatchFinishDecodeHooks {
    fn default() -> Self {
        Self { next: 50, remaining: 2 }
    }
}

impl BufferedDecodeHooks<SourceCodec, u8, u8> for BatchFinishDecodeHooks {
    type Error = EngineError;

    fn max_finish_output_len(&self, _codec: &SourceCodec) -> usize {
        self.remaining as usize
    }

    fn handle_decode_error(
        &mut self,
        _codec: &SourceCodec,
        error: core::convert::Infallible,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        match error {}
    }

    fn invalid_input_index(&mut self, _codec: &SourceCodec, index: usize, input_len: usize) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
    }

    fn finish(
        &mut self,
        _codec: &SourceCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if self.remaining == 0 {
            return Ok(TranscodeProgress::complete(0, 0));
        }
        if output_index == output.len() {
            return Ok(TranscodeProgress::need_output(output_index, 1, 0, 0, 0));
        }

        output[output_index] = self.next;
        self.next = self.next.wrapping_add(1);
        self.remaining -= 1;
        if self.remaining == 0 {
            Ok(TranscodeProgress::complete(0, 1))
        } else {
            Ok(TranscodeProgress::new(
                TranscodeStatus::NeedOutput {
                    output_index: output_index + 1,
                    additional: 1,
                    available: 0,
                },
                0,
                1,
            ))
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct BatchFinishHooks;

impl BufferedConvertHooks<SourceCodec, TargetCodec, u8, u8> for BatchFinishHooks {
    type DecodeHooks = BatchFinishDecodeHooks;
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
        BatchFinishDecodeHooks::default()
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
struct FinishEncodeHooks {
    pending: bool,
}

impl Default for FinishEncodeHooks {
    fn default() -> Self {
        Self { pending: true }
    }
}

impl BufferedEncodeHooks<TargetCodec, u8, u8> for FinishEncodeHooks {
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

    fn max_finish_output_len(&self, _codec: &TargetCodec) -> usize {
        usize::from(self.pending)
    }

    fn finish(
        &mut self,
        _codec: &TargetCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        if !self.pending {
            return Ok(TranscodeProgress::complete(0, 0));
        }
        let available = output.len().saturating_sub(output_index);
        if available == 0 {
            return Ok(TranscodeProgress::need_output(output_index, 1, available, 0, 0));
        }
        output[output_index] = 0xee;
        self.pending = false;
        Ok(TranscodeProgress::complete(0, 1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FinishEncodeHooksOnly;

impl BufferedConvertHooks<SourceCodec, TargetCodec, u8, u8> for FinishEncodeHooksOnly {
    type DecodeHooks = StrictDecodeHooks;
    type EncodeHooks = FinishEncodeHooks;
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
        FinishEncodeHooks: BufferedEncodeHooks<TargetCodec, u8, Output, Error = Self::EncodeError<Output>>;

    fn create_decode_hooks(&self, _decoder: &SourceCodec, _encoder: &TargetCodec) -> Self::DecodeHooks {
        StrictDecodeHooks
    }

    fn create_encode_hooks(&self, _decoder: &SourceCodec, _encoder: &TargetCodec) -> Self::EncodeHooks {
        FinishEncodeHooks::default()
    }

    fn map_decode_error<Output>(&self, error: EngineError) -> Self::Error<Output>
    where
        TargetCodec: Codec<u8, Output>,
        Output: Copy,
        FinishEncodeHooks: BufferedEncodeHooks<TargetCodec, u8, Output, Error = Self::EncodeError<Output>>,
    {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error<Output>(&self, error: Self::EncodeError<Output>) -> Self::Error<Output>
    where
        TargetCodec: Codec<u8, Output>,
        Output: Copy,
        FinishEncodeHooks: BufferedEncodeHooks<TargetCodec, u8, Output, Error = Self::EncodeError<Output>>,
    {
        ConvertEngineError::Encode(error)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ErrorPathDecodeHooks {
    finish: ErrorPathDecodeFinish,
    finish_len: usize,
    max_output_error: bool,
}

impl BufferedDecodeHooks<SourceCodec, u8, u8> for ErrorPathDecodeHooks {
    type Error = EngineError;

    fn max_output_len(&self, codec: &SourceCodec, input_len: usize) -> Result<usize, CapacityError> {
        if self.max_output_error {
            Err(CapacityError::OutputLengthOverflow)
        } else {
            Ok(input_len / codec.min_units_per_value().get())
        }
    }

    fn max_finish_output_len(&self, _codec: &SourceCodec) -> usize {
        self.finish_len
    }

    fn handle_decode_error(
        &mut self,
        _codec: &SourceCodec,
        error: core::convert::Infallible,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        match error {}
    }

    fn invalid_input_index(&mut self, _codec: &SourceCodec, index: usize, input_len: usize) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
    }

    fn finish(
        &mut self,
        _codec: &SourceCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        match self.finish {
            ErrorPathDecodeFinish::Normal => Ok(TranscodeProgress::complete(0, 0)),
            ErrorPathDecodeFinish::Error => Err(EngineError::Decode),
            ErrorPathDecodeFinish::NeedInput => Ok(TranscodeProgress::need_input(0, 1, 0, 0, 0)),
            ErrorPathDecodeFinish::NeedOutputWithoutValue => {
                Ok(TranscodeProgress::need_output(output_index, 1, 0, 0, 0))
            }
            ErrorPathDecodeFinish::EmitNeedInput => {
                output[output_index] = 0xab;
                Ok(TranscodeProgress::need_input(0, 1, 0, 0, 1))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
enum ErrorPathDecodeFinish {
    #[default]
    Normal,
    Error,
    NeedInput,
    NeedOutputWithoutValue,
    EmitNeedInput,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
enum ErrorPathEncodeMode {
    #[default]
    Normal,
    PrepareError,
    FinishError,
    FinishNeedInput,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ErrorPathEncodeHooks {
    finish_len: usize,
    max_output_error: bool,
    mode: ErrorPathEncodeMode,
}

impl<Output> BufferedEncodeHooks<TargetCodec, u8, Output> for ErrorPathEncodeHooks
where
    TargetCodec: Codec<u8, Output>,
    Output: Copy,
{
    type Error = EngineError;
    type PlanPayload = ();

    fn max_output_len(&self, codec: &TargetCodec, input_len: usize) -> Result<usize, CapacityError> {
        if self.max_output_error {
            Err(CapacityError::OutputLengthOverflow)
        } else {
            input_len
                .checked_mul(codec.max_units_per_value().get())
                .ok_or(CapacityError::OutputLengthOverflow)
        }
    }

    fn max_finish_output_len(&self, _codec: &TargetCodec) -> usize {
        self.finish_len
    }

    fn prepare_encode(
        &mut self,
        codec: &TargetCodec,
        _input_value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanPayload>, Self::Error> {
        match self.mode {
            ErrorPathEncodeMode::PrepareError => Err(EngineError::Encode),
            ErrorPathEncodeMode::Normal | ErrorPathEncodeMode::FinishError | ErrorPathEncodeMode::FinishNeedInput => {
                Ok(EncodePlan::new(codec.max_units_per_value().get(), ()))
            }
        }
    }

    unsafe fn write_encode(
        &mut self,
        _codec: &TargetCodec,
        _input_value: &u8,
        _input_index: usize,
        _plan_payload: Self::PlanPayload,
        _output: &mut [Output],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn finish(
        &mut self,
        _codec: &TargetCodec,
        _output: &mut [Output],
        _output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        match self.mode {
            ErrorPathEncodeMode::FinishError => Err(EngineError::Encode),
            ErrorPathEncodeMode::FinishNeedInput => Ok(TranscodeProgress::need_input(0, 1, 0, 0, 0)),
            ErrorPathEncodeMode::Normal | ErrorPathEncodeMode::PrepareError => Ok(TranscodeProgress::complete(0, 0)),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ErrorPathHooks {
    decode_finish: ErrorPathDecodeFinish,
    decode_finish_len: usize,
    decode_max_output_error: bool,
    encode_finish_len: usize,
    encode_max_output_error: bool,
    encode_mode: ErrorPathEncodeMode,
}

impl BufferedConvertHooks<SourceCodec, TargetCodec, u8, u8> for ErrorPathHooks {
    type DecodeHooks = ErrorPathDecodeHooks;
    type EncodeHooks = ErrorPathEncodeHooks;
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
        ErrorPathEncodeHooks: BufferedEncodeHooks<TargetCodec, u8, Output, Error = Self::EncodeError<Output>>;

    fn create_decode_hooks(&self, _decoder: &SourceCodec, _encoder: &TargetCodec) -> Self::DecodeHooks {
        ErrorPathDecodeHooks {
            finish: self.decode_finish,
            finish_len: self.decode_finish_len,
            max_output_error: self.decode_max_output_error,
        }
    }

    fn create_encode_hooks(&self, _decoder: &SourceCodec, _encoder: &TargetCodec) -> Self::EncodeHooks {
        ErrorPathEncodeHooks {
            finish_len: self.encode_finish_len,
            max_output_error: self.encode_max_output_error,
            mode: self.encode_mode,
        }
    }

    fn map_decode_error<Output>(&self, error: EngineError) -> Self::Error<Output>
    where
        TargetCodec: Codec<u8, Output>,
        Output: Copy,
        ErrorPathEncodeHooks: BufferedEncodeHooks<TargetCodec, u8, Output, Error = Self::EncodeError<Output>>,
    {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error<Output>(&self, error: Self::EncodeError<Output>) -> Self::Error<Output>
    where
        TargetCodec: Codec<u8, Output>,
        Output: Copy,
        ErrorPathEncodeHooks: BufferedEncodeHooks<TargetCodec, u8, Output, Error = Self::EncodeError<Output>>,
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

    fn invalid_input_index(&mut self, _codec: &SourceCodec, index: usize, input_len: usize) -> Self::Error {
        EngineError::invalid_input_index(index, input_len)
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
fn test_buffered_convert_engine_default_builds_engine() {
    let mut engine = BufferedConvertEngine::<SourceCodec, TargetCodec, CopyHooks, u8, u8>::default();
    let mut output = [0_u8; 1];

    let progress = engine
        .transcode::<u8>(&[8], 0, &mut output, 0)
        .expect("default components should convert one value");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((1, 1), (progress.read(), progress.written()));
    assert_eq!([9], output);

    engine.reset::<u8>();
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

    engine.reset::<u8>();
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
fn test_buffered_convert_engine_reports_pending_need_output_before_new_input() {
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

    let progress = engine
        .transcode::<u8>(&[9], 0, &mut empty_output, 0)
        .expect("conversion should report pending output before reading new input");
    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: 1,
            available: 0,
        },
        progress.status(),
    );
    assert_eq!((0, 0), (progress.read(), progress.written()));
    assert_eq!(Ok(2), engine.max_output_len::<u8>(1));

    let mut output = [0_u8; 2];
    let progress = engine
        .transcode::<u8>(&[9], 0, &mut output, 0)
        .expect("conversion should keep pending value after repeated output starvation");
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((1, 2), (progress.read(), progress.written()));
    assert_eq!([2, 10], output);
}

#[test]
fn test_buffered_convert_engine_maps_pending_encode_error_before_new_input() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(SourceCodec, TargetCodec, CopyHooks::default());
    let mut empty_output = [0_u8; 0];
    let progress = engine
        .transcode::<u8>(&[12], 0, &mut empty_output, 0)
        .expect("conversion should retain decoded value before encoding");
    assert!(matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));

    let mut output = [0_u8; 1];
    let error = engine
        .transcode::<u8>(&[1], 0, &mut output, 0)
        .expect_err("pending encode error should be mapped before new input is consumed");

    assert_eq!(ConvertEngineError::Encode(EngineError::Encode), error);
    assert_eq!([0], output);
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
fn test_buffered_convert_engine_reports_capacity_errors() {
    let engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(
        SourceCodec,
        TargetCodec,
        ErrorPathHooks {
            decode_max_output_error: true,
            ..ErrorPathHooks::default()
        },
    );
    assert_eq!(Err(CapacityError::OutputLengthOverflow), engine.max_output_len::<u8>(1));

    let engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(
        SourceCodec,
        TargetCodec,
        ErrorPathHooks {
            decode_finish_len: 1,
            encode_max_output_error: true,
            ..ErrorPathHooks::default()
        },
    );
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_finish_output_len::<u8>()
    );

    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(
        SourceCodec,
        TargetCodec,
        ErrorPathHooks {
            encode_max_output_error: true,
            ..ErrorPathHooks::default()
        },
    );
    let mut empty_output = [0_u8; 0];
    let progress = engine
        .transcode::<u8>(&[1], 0, &mut empty_output, 0)
        .expect("conversion should retain pending value");
    assert!(matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));
    assert_eq!(Err(CapacityError::OutputLengthOverflow), engine.max_output_len::<u8>(0));
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_finish_output_len::<u8>()
    );

    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(
        SourceCodec,
        TargetCodec,
        ErrorPathHooks {
            decode_finish_len: usize::MAX,
            ..ErrorPathHooks::default()
        },
    );
    let progress = engine
        .transcode::<u8>(&[1], 0, &mut empty_output, 0)
        .expect("conversion should retain pending value");
    assert!(matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_finish_output_len::<u8>()
    );
}

#[test]
fn test_buffered_convert_engine_maps_prepare_encode_error() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(
        SourceCodec,
        TargetCodec,
        ErrorPathHooks {
            decode_finish: ErrorPathDecodeFinish::Normal,
            encode_mode: ErrorPathEncodeMode::PrepareError,
            ..ErrorPathHooks::default()
        },
    );
    let mut output = [0_u8; 1];

    let error = engine
        .transcode::<u8>(&[1], 0, &mut output, 0)
        .expect_err("prepare encode error should be mapped through convert hooks");

    assert_eq!(ConvertEngineError::Encode(EngineError::Encode), error);
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_finish_reports_output_index_beyond_buffer() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(SourceCodec, TargetCodec, CopyHooks::default());
    let mut output = [];

    let progress = engine
        .finish::<u8>(&mut output, 1)
        .expect("out-of-range finish output index should request capacity");

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
fn test_buffered_convert_engine_finish_maps_decode_error() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(
        SourceCodec,
        TargetCodec,
        ErrorPathHooks {
            decode_finish: ErrorPathDecodeFinish::Error,
            encode_mode: ErrorPathEncodeMode::Normal,
            ..ErrorPathHooks::default()
        },
    );
    let mut output = [0_u8; 1];

    let error = engine
        .finish::<u8>(&mut output, 0)
        .expect_err("decode finish error should be mapped through convert hooks");

    assert_eq!(ConvertEngineError::Decode(EngineError::Decode), error);
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_finish_maps_encode_error() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(
        SourceCodec,
        TargetCodec,
        ErrorPathHooks {
            decode_finish: ErrorPathDecodeFinish::Normal,
            encode_mode: ErrorPathEncodeMode::FinishError,
            ..ErrorPathHooks::default()
        },
    );
    let mut output = [0_u8; 1];

    let error = engine
        .finish::<u8>(&mut output, 0)
        .expect_err("encode finish error should be mapped through convert hooks");

    assert_eq!(ConvertEngineError::Encode(EngineError::Encode), error);
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_finish_maps_pending_encode_error() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(SourceCodec, TargetCodec, CopyHooks::default());
    let mut empty_output = [0_u8; 0];
    let progress = engine
        .transcode::<u8>(&[12], 0, &mut empty_output, 0)
        .expect("conversion should retain decoded value before encoding");
    assert!(matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));

    let mut output = [0_u8; 1];
    let error = engine
        .finish::<u8>(&mut output, 0)
        .expect_err("finish should map pending encode error");

    assert_eq!(ConvertEngineError::Encode(EngineError::Encode), error);
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_finish_maps_decoder_output_encode_error() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(SourceCodec, TargetCodec, FinishHooks { value: 13 });
    let mut output = [0_u8; 1];

    let error = engine
        .finish::<u8>(&mut output, 0)
        .expect_err("finish should map encode errors for decoder-emitted values");

    assert_eq!(ConvertEngineError::Encode(EngineError::Encode), error);
    assert_eq!([0], output);
}

#[test]
#[should_panic(expected = "buffered decode engine finish cannot request source input")]
fn test_buffered_convert_engine_rejects_decode_finish_need_input() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(
        SourceCodec,
        TargetCodec,
        ErrorPathHooks {
            decode_finish: ErrorPathDecodeFinish::NeedInput,
            encode_mode: ErrorPathEncodeMode::Normal,
            ..ErrorPathHooks::default()
        },
    );
    let mut output = [0_u8; 1];

    let _ = engine.finish::<u8>(&mut output, 0);
}

#[test]
#[should_panic(expected = "decode finish hook must emit progress before requesting more decoded output")]
fn test_buffered_convert_engine_rejects_decode_finish_need_output_without_value() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(
        SourceCodec,
        TargetCodec,
        ErrorPathHooks {
            decode_finish: ErrorPathDecodeFinish::NeedOutputWithoutValue,
            encode_mode: ErrorPathEncodeMode::Normal,
            ..ErrorPathHooks::default()
        },
    );
    let mut output = [0_u8; 1];

    let _ = engine.finish::<u8>(&mut output, 0);
}

#[test]
#[should_panic(expected = "buffered decode engine finish cannot request source input")]
fn test_buffered_convert_engine_rejects_decode_finish_emit_need_input() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(
        SourceCodec,
        TargetCodec,
        ErrorPathHooks {
            decode_finish: ErrorPathDecodeFinish::EmitNeedInput,
            encode_mode: ErrorPathEncodeMode::Normal,
            ..ErrorPathHooks::default()
        },
    );
    let mut output = [0_u8; 1];

    let _ = engine.finish::<u8>(&mut output, 0);
}

#[test]
#[should_panic(expected = "buffered encode engine cannot request source input")]
fn test_buffered_convert_engine_rejects_encode_finish_need_input() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(
        SourceCodec,
        TargetCodec,
        ErrorPathHooks {
            decode_finish: ErrorPathDecodeFinish::Normal,
            encode_mode: ErrorPathEncodeMode::FinishNeedInput,
            ..ErrorPathHooks::default()
        },
    );
    let mut output = [0_u8; 1];

    let _ = engine.finish::<u8>(&mut output, 0);
}

#[test]
fn test_buffered_convert_engine_applies_decode_policy_skip() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(
        ErrorSourceCodec,
        TargetCodec,
        RepairHooks {
            action: RepairAction::Skip,
        },
    );
    let mut output = [0_u8; 1];

    let progress = engine
        .transcode::<u8>(&[1, 2], 0, &mut output, 0)
        .expect("skip policy should consume invalid source units");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((2, 0), (progress.read(), progress.written()));
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_applies_decode_policy_emit() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(
        ErrorSourceCodec,
        TargetCodec,
        RepairHooks {
            action: RepairAction::Emit,
        },
    );
    let mut output = [0_u8; 2];

    let progress = engine
        .transcode::<u8>(&[1, 2], 0, &mut output, 0)
        .expect("emit policy should replace invalid source units");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((2, 2), (progress.read(), progress.written()));
    assert_eq!([42, 42], output);
}

#[test]
fn test_buffered_convert_engine_applies_decode_policy_need_input() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(
        ErrorSourceCodec,
        TargetCodec,
        RepairHooks {
            action: RepairAction::NeedInput,
        },
    );
    let mut output = [0_u8; 1];

    let progress = engine
        .transcode::<u8>(&[1], 0, &mut output, 0)
        .expect("need-input policy should stop without consuming input");

    assert_eq!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            additional: 2,
            available: 1,
        },
        progress.status(),
    );
    assert_eq!((0, 0), (progress.read(), progress.written()));
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
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(SourceCodec, TargetCodec, FinishHooks::default());
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
fn test_buffered_convert_engine_finish_drains_decoder_finish_batches() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(SourceCodec, TargetCodec, BatchFinishHooks);
    assert_eq!(Ok(2), engine.max_finish_output_len::<u8>());

    let mut output = [0_u8; 2];
    let progress = engine
        .finish::<u8>(&mut output, 0)
        .expect("finish should keep draining decoder finish batches");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((0, 2), (progress.read(), progress.written()));
    assert_eq!([50, 51], output);
    assert_eq!(Ok(0), engine.max_finish_output_len::<u8>());
}

#[test]
fn test_buffered_convert_engine_finish_drains_pending_before_decoder_finish_output() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(SourceCodec, TargetCodec, FinishHooks::default());
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

#[test]
fn test_buffered_convert_engine_finish_delegates_to_encoder_finish() {
    let mut engine = BufferedConvertEngine::<_, _, _, u8, u8>::new(SourceCodec, TargetCodec, FinishEncodeHooksOnly);
    assert_eq!(Ok(1), engine.max_finish_output_len::<u8>());

    let mut empty_output = [0_u8; 0];
    let progress = engine
        .finish::<u8>(&mut empty_output, 0)
        .expect("target finish hook should request output capacity");
    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: 1,
            available: 0,
        },
        progress.status(),
    );

    let mut output = [0_u8; 1];
    let progress = engine
        .finish::<u8>(&mut output, 0)
        .expect("target finish hook should write final output");
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((0, 1), (progress.read(), progress.written()));
    assert_eq!([0xee], output);
    assert_eq!(Ok(0), engine.max_finish_output_len::<u8>());
}

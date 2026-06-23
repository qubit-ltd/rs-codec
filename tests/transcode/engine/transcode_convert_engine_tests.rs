// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the reusable buffered converter engine.

use core::{cell::Cell, num::NonZeroUsize};

use qubit_codec::{
    CapacityError, Codec, DecodeContext, DecodeInvalidAction, EncodeContext, EncodeValueResult,
    TranscodeConvertEngine, TranscodeConvertHooks, TranscodeDecodeHooks, TranscodeEncodeHooks,
    TranscodeError, TranscodeStatus,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct SourceCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct TargetCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ErrorSourceCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FlushValueSourceCodec<const FLUSH_BOUND: usize>;

fn one_consumed() -> NonZeroUsize {
    NonZeroUsize::MIN
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
enum EngineError {
    #[error("decode error")]
    Decode,
    #[error("encode error")]
    Encode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
enum ConvertEngineError<E> {
    #[error("decode: {0}")]
    Decode(EngineError),
    #[error("encode: {0}")]
    Encode(E),
}

impl Codec for SourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::CodecDecodeFailure<Self::DecodeError>> {
        // SAFETY: The caller proved that at least one input unit is readable.
        let value = unsafe { *input.get_unchecked(index) };
        Ok((value.wrapping_add(1), NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<NonZeroUsize, Self::EncodeError> {
        // SAFETY: The caller proved that one output unit is writable.
        unsafe {
            *output.get_unchecked_mut(index) = *value;
        }
        Ok(qubit_io::nz!(1))
    }
}

impl Codec for TargetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = EngineError;

    const MIN_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::CodecDecodeFailure<Self::DecodeError>> {
        // SAFETY: The caller proved that at least one input unit is readable.
        let value = unsafe { *input.get_unchecked(index) };
        Ok((value, NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<NonZeroUsize, Self::EncodeError> {
        if *value == 13 {
            return Err(EngineError::Encode);
        }
        // SAFETY: The caller proved that one output unit is writable.
        unsafe {
            *output.get_unchecked_mut(index) = *value;
        }
        Ok(qubit_io::nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetEmittingTargetCodec;

impl Codec for ResetEmittingTargetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;

    const MAX_ENCODE_RESET_UNITS: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::CodecDecodeFailure<Self::DecodeError>> {
        unsafe { Ok((*input.get_unchecked(index), NonZeroUsize::MIN)) }
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<NonZeroUsize, Self::EncodeError> {
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
struct ResetFailTargetCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("target reset failed")]
struct TargetResetFailError;

impl Codec for ResetFailTargetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = TargetResetFailError;

    const MIN_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;

    const MAX_ENCODE_RESET_UNITS: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::CodecDecodeFailure<Self::DecodeError>> {
        unsafe { Ok((*input.get_unchecked(index), NonZeroUsize::MIN)) }
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<NonZeroUsize, Self::EncodeError> {
        output[index] = *value;
        Ok(qubit_io::nz!(1))
    }

    unsafe fn encode_reset(
        &mut self,
        _output: &mut [u8],
        _index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Err(TargetResetFailError)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetTargetHooks;

impl TranscodeEncodeHooks<ResetEmittingTargetCodec> for ResetTargetHooks {
    type Error = EngineError;

    fn encode_value(
        &mut self,
        codec: &mut ResetEmittingTargetCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeValueResult, Self::Error> {
        let required = <ResetEmittingTargetCodec as Codec>::MAX_UNITS_PER_VALUE;
        if context.available_output() < required.get() {
            return Ok(EncodeValueResult::need_output(required));
        }
        let written = unsafe {
            // SAFETY: The hook checked that the codec output range is writable.
            codec
                .encode(context.input_value, context.output, context.output_index)
                .expect("infallible target encode")
                .get()
        };
        Ok(EncodeValueResult::consumed(written))
    }
}

impl TranscodeConvertHooks<SourceCodec, ResetEmittingTargetCodec> for ResetTargetHooks {
    type DecodeError = EngineError;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
        ConvertEngineError::Encode(error)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetFailTargetHooks;

impl TranscodeEncodeHooks<ResetFailTargetCodec> for ResetFailTargetHooks {
    type Error = TargetResetFailError;

    fn encode_value(
        &mut self,
        codec: &mut ResetFailTargetCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeValueResult, Self::Error> {
        let required = <ResetFailTargetCodec as Codec>::MAX_UNITS_PER_VALUE;
        if context.available_output() < required.get() {
            return Ok(EncodeValueResult::need_output(required));
        }
        let written = unsafe {
            // SAFETY: The hook checked that the codec output range is writable.
            codec.encode(context.input_value, context.output, context.output_index)
        }?
        .get();
        Ok(EncodeValueResult::consumed(written))
    }

    fn map_encode_reset_error(
        &mut self,
        _codec: &mut ResetFailTargetCodec,
        error: TargetResetFailError,
    ) -> Self::Error {
        error
    }
}

impl TranscodeConvertHooks<SourceCodec, ResetFailTargetCodec> for ResetFailTargetHooks {
    type DecodeError = EngineError;
    type EncodeError = TargetResetFailError;
    type Error = ConvertEngineError<TargetResetFailError>;

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
        ConvertEngineError::Encode(error)
    }
}

impl Codec for ErrorSourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = EngineError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;

    unsafe fn decode(
        &mut self,
        _input: &[u8],
        _index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::CodecDecodeFailure<Self::DecodeError>> {
        Err(qubit_codec::CodecDecodeFailure::invalid_without_consumed(
            EngineError::Decode,
        ))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<NonZeroUsize, Self::EncodeError> {
        // SAFETY: The caller proved that one output unit is writable.
        unsafe {
            *output.get_unchecked_mut(index) = *value;
        }
        Ok(qubit_io::nz!(1))
    }
}

impl<const FLUSH_BOUND: usize> Codec for FlushValueSourceCodec<FLUSH_BOUND> {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;

    const MAX_UNITS_PER_VALUE: NonZeroUsize = NonZeroUsize::MIN;

    const MAX_DECODE_FLUSH_VALUES: usize = FLUSH_BOUND;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::CodecDecodeFailure<Self::DecodeError>> {
        // SAFETY: The caller proved that at least one input unit is readable.
        let value = unsafe { *input.get_unchecked(index) };
        Ok((value, NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<NonZeroUsize, Self::EncodeError> {
        // SAFETY: The caller proved that one output unit is writable.
        unsafe {
            *output.get_unchecked_mut(index) = *value;
        }
        Ok(NonZeroUsize::MIN)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct StrictDecodeHooks;

impl TranscodeDecodeHooks<SourceCodec> for StrictDecodeHooks {
    type Error = EngineError;
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut SourceCodec,
        error: core::convert::Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match error {}
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct StrictEncodeHooks;

impl TranscodeEncodeHooks<TargetCodec> for StrictEncodeHooks {
    type Error = EngineError;

    fn encode_value(
        &mut self,
        codec: &mut TargetCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeValueResult, Self::Error> {
        let required = <TargetCodec as Codec>::MAX_UNITS_PER_VALUE;
        if context.available_output() < required.get() {
            return Ok(EncodeValueResult::need_output(required));
        }
        let EncodeContext {
            input_value,
            output,
            output_index,
            ..
        } = context;
        let written = unsafe {
            // SAFETY: The hook checked that the codec output range is writable.
            codec.encode(input_value, output, output_index)
        }?
        .get();
        Ok(EncodeValueResult::consumed(written))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct MismatchCapacityEncodeHooks;

impl TranscodeEncodeHooks<TargetCodec> for MismatchCapacityEncodeHooks {
    type Error = EngineError;

    fn max_output_len(
        &self,
        _codec: &TargetCodec,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn encode_value(
        &mut self,
        _codec: &mut TargetCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeValueResult, Self::Error> {
        let required = qubit_io::nz!(2);
        if context.available_output() < required.get() {
            return Ok(EncodeValueResult::need_output(required));
        }
        unsafe {
            // SAFETY: The hook checked that two output units are writable.
            *context.output.get_unchecked_mut(context.output_index) = *context.input_value;
            *context.output.get_unchecked_mut(context.output_index + 1) =
                context.input_value.wrapping_add(1);
        }
        Ok(EncodeValueResult::consumed(2))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct MismatchPendingFinishHooks;

impl TranscodeConvertHooks<SourceCodec, TargetCodec> for MismatchPendingFinishHooks {
    type DecodeError = EngineError;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
        ConvertEngineError::Encode(error)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct MismatchDecoderFinishHooks;

impl TranscodeConvertHooks<SourceCodec, TargetCodec> for MismatchDecoderFinishHooks {
    type DecodeError = EngineError;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
        ConvertEngineError::Encode(error)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct CopyHooks {
    reset_called: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RepairAction {
    Emit,
    Skip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RepairDecodeHooks {
    action: RepairAction,
}

impl TranscodeDecodeHooks<ErrorSourceCodec> for RepairDecodeHooks {
    type Error = EngineError;

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut ErrorSourceCodec,
        _error: EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match self.action {
            RepairAction::Emit => Ok(DecodeInvalidAction::Emit {
                value: 42,
                consumed: one_consumed(),
            }),
            RepairAction::Skip => Ok(DecodeInvalidAction::Skip {
                consumed: one_consumed(),
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RepairHooks {
    action: RepairAction,
}

impl TranscodeConvertHooks<ErrorSourceCodec, TargetCodec> for RepairHooks {
    type DecodeError = EngineError;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
        ConvertEngineError::Encode(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FixedFinishBoundDecodeHooks {
    finish_len: usize,
}

impl<const FLUSH_BOUND: usize> TranscodeDecodeHooks<FlushValueSourceCodec<FLUSH_BOUND>>
    for FixedFinishBoundDecodeHooks
{
    type Error = EngineError;

    fn max_finish_output_len(&self, _codec: &FlushValueSourceCodec<FLUSH_BOUND>) -> usize {
        self.finish_len
    }

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut FlushValueSourceCodec<FLUSH_BOUND>,
        error: core::convert::Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match error {}
    }
}

#[derive(Debug)]
struct ChangingFinishBoundDecodeHooks {
    calls: Cell<usize>,
}

impl<const FLUSH_BOUND: usize> TranscodeDecodeHooks<FlushValueSourceCodec<FLUSH_BOUND>>
    for ChangingFinishBoundDecodeHooks
{
    type Error = EngineError;

    fn max_finish_output_len(&self, _codec: &FlushValueSourceCodec<FLUSH_BOUND>) -> usize {
        let calls = self.calls.get();
        self.calls.set(calls + 1);
        if calls == 0 { 0 } else { usize::MAX }
    }

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut FlushValueSourceCodec<FLUSH_BOUND>,
        error: core::convert::Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match error {}
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FixedFinishBoundHooks {
    finish_len: usize,
}

impl<const FLUSH_BOUND: usize>
    TranscodeConvertHooks<FlushValueSourceCodec<FLUSH_BOUND>, TargetCodec>
    for FixedFinishBoundHooks
{
    type DecodeError = EngineError;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
        ConvertEngineError::Encode(error)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ChangingFinishBoundHooks;

impl<const FLUSH_BOUND: usize>
    TranscodeConvertHooks<FlushValueSourceCodec<FLUSH_BOUND>, TargetCodec>
    for ChangingFinishBoundHooks
{
    type DecodeError = EngineError;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
        ConvertEngineError::Encode(error)
    }
}

impl TranscodeConvertHooks<SourceCodec, TargetCodec> for CopyHooks {
    type DecodeError = EngineError;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
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

impl TranscodeDecodeHooks<SourceCodec> for FinishDecodeHooks {
    type Error = EngineError;

    fn max_finish_output_len(&self, _codec: &SourceCodec) -> usize {
        usize::from(self.value.is_some())
    }

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut SourceCodec,
        error: core::convert::Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match error {}
    }

    fn finish(
        &mut self,
        _codec: &mut SourceCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        let Some(value) = self.value else {
            return Ok(0);
        };
        output[output_index] = value;
        self.value = None;
        Ok(1)
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

impl TranscodeConvertHooks<SourceCodec, TargetCodec> for FinishHooks {
    type DecodeError = EngineError;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
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
        Self {
            next: 50,
            remaining: 2,
        }
    }
}

impl TranscodeDecodeHooks<SourceCodec> for BatchFinishDecodeHooks {
    type Error = EngineError;

    fn max_finish_output_len(&self, _codec: &SourceCodec) -> usize {
        self.remaining as usize
    }

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut SourceCodec,
        error: core::convert::Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match error {}
    }

    fn finish(
        &mut self,
        _codec: &mut SourceCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        if self.remaining == 0 {
            return Ok(0);
        }

        let mut written = 0;
        while self.remaining > 0 {
            output[output_index + written] = self.next;
            self.next = self.next.wrapping_add(1);
            self.remaining -= 1;
            written += 1;
        }
        Ok(written)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct BatchFinishHooks;

impl TranscodeConvertHooks<SourceCodec, TargetCodec> for BatchFinishHooks {
    type DecodeError = EngineError;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
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

impl TranscodeEncodeHooks<TargetCodec> for FinishEncodeHooks {
    type Error = EngineError;

    fn encode_value(
        &mut self,
        codec: &mut TargetCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeValueResult, Self::Error> {
        let required = <TargetCodec as Codec>::MAX_UNITS_PER_VALUE;
        if context.available_output() < required.get() {
            return Ok(EncodeValueResult::need_output(required));
        }
        let EncodeContext {
            input_value,
            output,
            output_index,
            ..
        } = context;
        let written = unsafe {
            // SAFETY: The hook checked that the codec output range is writable.
            codec.encode(input_value, output, output_index)
        }?
        .get();
        Ok(EncodeValueResult::consumed(written))
    }

    fn max_finish_output_len(&self, _codec: &TargetCodec) -> usize {
        usize::from(self.pending)
    }

    fn finish(
        &mut self,
        _codec: &mut TargetCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::Error> {
        if !self.pending {
            return Ok(0);
        }
        output[output_index] = 0xee;
        self.pending = false;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FinishEncodeHooksOnly;

impl TranscodeConvertHooks<SourceCodec, TargetCodec> for FinishEncodeHooksOnly {
    type DecodeError = EngineError;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
        ConvertEngineError::Encode(error)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ErrorPathDecodeHooks {
    finish: ErrorPathDecodeFinish,
    finish_len: usize,
    max_output_error: bool,
}

impl TranscodeDecodeHooks<SourceCodec> for ErrorPathDecodeHooks {
    type Error = EngineError;

    fn max_output_len(
        &self,
        _codec: &SourceCodec,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        if self.max_output_error {
            Err(CapacityError::OutputLengthOverflow)
        } else {
            Ok(input_len / <SourceCodec as Codec>::MIN_UNITS_PER_VALUE.get())
        }
    }

    fn max_finish_output_len(&self, _codec: &SourceCodec) -> usize {
        self.finish_len
    }

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut SourceCodec,
        error: core::convert::Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match error {}
    }

    fn finish(
        &mut self,
        _codec: &mut SourceCodec,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        match self.finish {
            ErrorPathDecodeFinish::Normal => Ok(0),
            ErrorPathDecodeFinish::Error => Err(EngineError::Decode),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
enum ErrorPathDecodeFinish {
    #[default]
    Normal,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
enum ErrorPathEncodeMode {
    #[default]
    Normal,
    PrepareError,
    FinishError,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ErrorPathEncodeHooks {
    finish_len: usize,
    max_output_error: bool,
    mode: ErrorPathEncodeMode,
}

impl TranscodeEncodeHooks<TargetCodec> for ErrorPathEncodeHooks {
    type Error = EngineError;

    fn max_output_len(
        &self,
        _codec: &TargetCodec,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        if self.max_output_error {
            Err(CapacityError::OutputLengthOverflow)
        } else {
            input_len
                .checked_mul(<TargetCodec as Codec>::MAX_UNITS_PER_VALUE.get())
                .ok_or(CapacityError::OutputLengthOverflow)
        }
    }

    fn max_finish_output_len(&self, _codec: &TargetCodec) -> usize {
        self.finish_len
    }

    fn encode_value(
        &mut self,
        _codec: &mut TargetCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeValueResult, Self::Error> {
        match self.mode {
            ErrorPathEncodeMode::PrepareError => Err(EngineError::Encode),
            ErrorPathEncodeMode::Normal | ErrorPathEncodeMode::FinishError => {
                let required = <TargetCodec as Codec>::MAX_UNITS_PER_VALUE;
                if context.available_output() < required.get() {
                    Ok(EncodeValueResult::need_output(required))
                } else {
                    Ok(EncodeValueResult::consumed(0))
                }
            }
        }
    }

    fn finish(
        &mut self,
        _codec: &mut TargetCodec,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        match self.mode {
            ErrorPathEncodeMode::FinishError => Err(EngineError::Encode),
            ErrorPathEncodeMode::Normal | ErrorPathEncodeMode::PrepareError => Ok(0),
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

impl TranscodeConvertHooks<SourceCodec, TargetCodec> for ErrorPathHooks {
    type DecodeError = EngineError;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
        ConvertEngineError::Encode(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FactoryDecodeHooks {
    marker: u8,
}

impl TranscodeDecodeHooks<SourceCodec> for FactoryDecodeHooks {
    type Error = EngineError;

    fn max_output_len(
        &self,
        _codec: &SourceCodec,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(self.marker as usize)
    }

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut SourceCodec,
        error: core::convert::Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match error {}
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FactoryEncodeHooks {
    offset: u8,
}

impl TranscodeEncodeHooks<TargetCodec> for FactoryEncodeHooks {
    type Error = EngineError;

    fn encode_value(
        &mut self,
        _codec: &mut TargetCodec,
        context: EncodeContext<'_, u8, u8>,
    ) -> Result<EncodeValueResult, Self::Error> {
        let required = <TargetCodec as Codec>::MAX_UNITS_PER_VALUE;
        if context.available_output() < required.get() {
            return Ok(EncodeValueResult::need_output(required));
        }
        let EncodeContext {
            input_value,
            output,
            output_index,
            ..
        } = context;
        output[output_index] = input_value.wrapping_add(self.offset);
        Ok(EncodeValueResult::consumed(1))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DirectConvertHooks {
    decode_marker: u8,
    encode_offset: u8,
}

impl TranscodeConvertHooks<SourceCodec, TargetCodec> for DirectConvertHooks {
    type DecodeError = EngineError;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
        ConvertEngineError::Encode(error)
    }
}

type CopyConvertEngine = TranscodeConvertEngine<
    SourceCodec,
    TargetCodec,
    StrictDecodeHooks,
    StrictEncodeHooks,
    CopyHooks,
>;

fn new_copy_engine() -> CopyConvertEngine {
    TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        StrictDecodeHooks,
        StrictEncodeHooks,
        CopyHooks::default(),
    )
}

fn new_error_path_engine(
    hooks: ErrorPathHooks,
) -> TranscodeConvertEngine<
    SourceCodec,
    TargetCodec,
    ErrorPathDecodeHooks,
    ErrorPathEncodeHooks,
    ErrorPathHooks,
> {
    let decode_hooks = ErrorPathDecodeHooks {
        finish: hooks.decode_finish,
        finish_len: hooks.decode_finish_len,
        max_output_error: hooks.decode_max_output_error,
    };
    let encode_hooks = ErrorPathEncodeHooks {
        finish_len: hooks.encode_finish_len,
        max_output_error: hooks.encode_max_output_error,
        mode: hooks.encode_mode,
    };
    TranscodeConvertEngine::new(SourceCodec, TargetCodec, decode_hooks, encode_hooks, hooks)
}

fn new_finish_engine(
    hooks: FinishHooks,
) -> TranscodeConvertEngine<
    SourceCodec,
    TargetCodec,
    FinishDecodeHooks,
    StrictEncodeHooks,
    FinishHooks,
> {
    TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        FinishDecodeHooks {
            value: Some(hooks.value),
        },
        StrictEncodeHooks,
        hooks,
    )
}

fn new_batch_finish_engine() -> TranscodeConvertEngine<
    SourceCodec,
    TargetCodec,
    BatchFinishDecodeHooks,
    StrictEncodeHooks,
    BatchFinishHooks,
> {
    TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        BatchFinishDecodeHooks::default(),
        StrictEncodeHooks,
        BatchFinishHooks,
    )
}

fn new_finish_encode_engine() -> TranscodeConvertEngine<
    SourceCodec,
    TargetCodec,
    StrictDecodeHooks,
    FinishEncodeHooks,
    FinishEncodeHooksOnly,
> {
    TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        StrictDecodeHooks,
        FinishEncodeHooks::default(),
        FinishEncodeHooksOnly,
    )
}

fn new_repair_engine(
    action: RepairAction,
) -> TranscodeConvertEngine<
    ErrorSourceCodec,
    TargetCodec,
    RepairDecodeHooks,
    StrictEncodeHooks,
    RepairHooks,
> {
    TranscodeConvertEngine::new(
        ErrorSourceCodec,
        TargetCodec,
        RepairDecodeHooks { action },
        StrictEncodeHooks,
        RepairHooks { action },
    )
}

#[test]
fn test_buffered_convert_engine_reports_bounds_and_resets() {
    let mut engine = new_copy_engine();

    assert_eq!(Ok(3), engine.max_output_len(3));
    assert_eq!(Ok(0), engine.max_finish_output_len());
    assert_eq!(Ok(0), engine.max_reset_output_len());

    engine.reset(&mut [], 0).expect("reset");
    assert_eq!(Ok(0), engine.max_finish_output_len());
}

#[test]
fn test_buffered_convert_engine_default_builds_engine() {
    let mut engine = TranscodeConvertEngine::<
        SourceCodec,
        TargetCodec,
        StrictDecodeHooks,
        StrictEncodeHooks,
        CopyHooks,
    >::default();
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
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        StrictDecodeHooks,
        StrictEncodeHooks,
        CopyHooks::default(),
    );
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
        DirectConvertHooks {
            decode_marker: 11,
            encode_offset: 7,
        },
    );

    assert_eq!(Ok(11), engine.max_output_len(1));

    let mut output = [0_u8; 1];
    let progress = engine
        .transcode(&[1], 0, &mut output, 0)
        .expect("supplied encode hooks should convert the value");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((1, 1), (progress.read(), progress.written()));
    assert_eq!([9], output);

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
            output_index: 0,
            required: crate::nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!((1, 0), (progress.read(), progress.written()));
    assert_eq!(Ok(2), engine.max_output_len(1));
    assert_eq!(Ok(1), engine.max_finish_output_len());

    let mut output = [0_u8; 2];
    let progress = engine
        .transcode(&[9], 0, &mut output, 0)
        .expect("conversion should drain pending before reading new input");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((1, 2), (progress.read(), progress.written()));
    assert_eq!([2, 10], output);
    assert_eq!(Ok(0), engine.max_finish_output_len());
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
            output_index: 0,
            required: crate::nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!((1, 0), (progress.read(), progress.written()));

    let progress = engine
        .transcode(&[9], 0, &mut empty_output, 0)
        .expect("conversion should report pending output before reading new input");
    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            required: crate::nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!((0, 0), (progress.read(), progress.written()));
    assert_eq!(Ok(2), engine.max_output_len(1));

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
    let mut empty_output = [0_u8; 0];
    let progress = engine
        .transcode(&[12], 0, &mut empty_output, 0)
        .expect("conversion should retain decoded value before encoding");
    assert!(matches!(
        progress.status(),
        TranscodeStatus::NeedOutput { .. }
    ));

    let mut output = [0_u8; 1];
    let error = engine
        .transcode(&[1], 0, &mut output, 0)
        .expect_err("pending encode error should be mapped before new input is consumed");

    assert_eq!(
        TranscodeError::Domain(ConvertEngineError::Encode(EngineError::Encode)),
        error
    );
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_reports_invalid_indices() {
    let mut engine = new_copy_engine();
    let mut output = [0_u8; 1];

    let error = engine
        .transcode(&[1], 2, &mut output, 0)
        .expect_err("invalid input index should fail");
    assert_eq!(
        TranscodeError::InvalidInputIndex { index: 2, len: 1 },
        error,
    );

    let error = engine
        .transcode(&[1], 0, &mut output, 2)
        .expect_err("invalid output index should fail");
    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 2, len: 1 },
        error,
    );
}

#[test]
fn test_buffered_convert_engine_reports_capacity_errors() {
    let engine = new_error_path_engine(ErrorPathHooks {
        decode_max_output_error: true,
        ..ErrorPathHooks::default()
    });
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_output_len(1)
    );

    let engine = new_error_path_engine(ErrorPathHooks {
        decode_finish_len: 1,
        encode_max_output_error: true,
        ..ErrorPathHooks::default()
    });
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_finish_output_len()
    );

    let mut engine = new_error_path_engine(ErrorPathHooks {
        encode_max_output_error: true,
        ..ErrorPathHooks::default()
    });
    let mut empty_output = [0_u8; 0];
    let progress = engine
        .transcode(&[1], 0, &mut empty_output, 0)
        .expect("conversion should retain pending value");
    assert!(matches!(
        progress.status(),
        TranscodeStatus::NeedOutput { .. }
    ));
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_output_len(0)
    );
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_finish_output_len()
    );

    let mut engine = new_error_path_engine(ErrorPathHooks {
        decode_finish_len: usize::MAX,
        ..ErrorPathHooks::default()
    });
    let progress = engine
        .transcode(&[1], 0, &mut empty_output, 0)
        .expect("conversion should retain pending value");
    assert!(matches!(
        progress.status(),
        TranscodeStatus::NeedOutput { .. }
    ));
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_finish_output_len()
    );
}

#[test]
fn test_buffered_convert_engine_finish_maps_initial_decode_finish_bound_overflow() {
    let mut engine = TranscodeConvertEngine::new(
        FlushValueSourceCodec::<{ usize::MAX }>,
        TargetCodec,
        FixedFinishBoundDecodeHooks { finish_len: 1 },
        StrictEncodeHooks,
        FixedFinishBoundHooks { finish_len: 1 },
    );
    let mut output = [];

    let error = engine
        .finish(&mut output, 0)
        .expect_err("overflowing decode finish bound should be mapped");

    assert_eq!(TranscodeError::output_length_overflow(), error);
}

#[test]
fn test_buffered_convert_engine_finish_maps_late_decode_finish_bound_overflow() {
    let mut engine = TranscodeConvertEngine::new(
        FlushValueSourceCodec::<1>,
        TargetCodec,
        ChangingFinishBoundDecodeHooks {
            calls: Cell::new(0),
        },
        StrictEncodeHooks,
        ChangingFinishBoundHooks,
    );
    let mut output = [0_u8; 1];

    let error = engine
        .finish(&mut output, 0)
        .expect_err("late decode finish bound overflow should be mapped");

    assert_eq!(TranscodeError::output_length_overflow(), error);
}

#[test]
fn test_buffered_convert_engine_maps_encode_value_error() {
    let mut engine = new_error_path_engine(ErrorPathHooks {
        decode_finish: ErrorPathDecodeFinish::Normal,
        encode_mode: ErrorPathEncodeMode::PrepareError,
        ..ErrorPathHooks::default()
    });
    let mut output = [0_u8; 1];

    let error = engine
        .transcode(&[1], 0, &mut output, 0)
        .expect_err("encode value error should be mapped through convert hooks");

    assert_eq!(
        TranscodeError::Domain(ConvertEngineError::Encode(EngineError::Encode)),
        error
    );
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
        TranscodeError::InvalidOutputIndex { index: 1, len: 0 },
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
    let mut output = [0_u8; 1];

    let error = engine
        .finish(&mut output, 0)
        .expect_err("decode finish error should be mapped through convert hooks");

    assert_eq!(
        TranscodeError::Domain(ConvertEngineError::Decode(EngineError::Decode)),
        error
    );
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_finish_maps_encode_error() {
    let mut engine = new_error_path_engine(ErrorPathHooks {
        decode_finish: ErrorPathDecodeFinish::Normal,
        encode_mode: ErrorPathEncodeMode::FinishError,
        ..ErrorPathHooks::default()
    });
    let mut output = [0_u8; 1];

    let error = engine
        .finish(&mut output, 0)
        .expect_err("encode finish error should be mapped through convert hooks");

    assert_eq!(
        TranscodeError::Domain(ConvertEngineError::Encode(EngineError::Encode)),
        error
    );
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_finish_maps_pending_encode_error() {
    let mut engine = new_copy_engine();
    let mut empty_output = [0_u8; 0];
    let progress = engine
        .transcode(&[12], 0, &mut empty_output, 0)
        .expect("conversion should retain decoded value before encoding");
    assert!(matches!(
        progress.status(),
        TranscodeStatus::NeedOutput { .. }
    ));

    let mut output = [0_u8; 1];
    let error = engine
        .finish(&mut output, 0)
        .expect_err("finish should map pending encode error");

    assert_eq!(
        TranscodeError::Domain(ConvertEngineError::Encode(EngineError::Encode)),
        error
    );
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_finish_maps_decoder_output_encode_error() {
    let mut engine = new_finish_engine(FinishHooks { value: 13 });
    let mut output = [0_u8; 1];

    let error = engine
        .finish(&mut output, 0)
        .expect_err("finish should map encode errors for decoder-emitted values");

    assert_eq!(
        TranscodeError::Domain(ConvertEngineError::Encode(EngineError::Encode)),
        error
    );
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
    assert!(matches!(
        progress.status(),
        TranscodeStatus::NeedOutput { .. }
    ));

    let error = engine
        .finish(&mut empty_output, 0)
        .expect_err("finish should reject insufficient output before draining pending value");
    assert_eq!(
        TranscodeError::InsufficientOutput {
            output_index: 0,
            required: 1,
            available: 0
        },
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
    assert_eq!(Ok(1), engine.max_finish_output_len());

    let mut empty_output = [0_u8; 0];
    let error = engine
        .finish(&mut empty_output, 0)
        .expect_err("finish should reject insufficient output before decoder finish");
    assert_eq!(
        TranscodeError::InsufficientOutput {
            output_index: 0,
            required: 1,
            available: 0
        },
        error,
    );
    assert_eq!(Ok(1), engine.max_finish_output_len());

    let mut output = [0_u8; 1];
    let written = engine
        .finish(&mut output, 0)
        .expect("finish should encode decoder finish value");
    assert_eq!(1, written);
    assert_eq!([40], output);
    assert_eq!(Ok(0), engine.max_finish_output_len());
}

#[test]
fn test_buffered_convert_engine_finish_drains_decoder_finish_batches() {
    let mut engine = new_batch_finish_engine();
    assert_eq!(Ok(2), engine.max_finish_output_len());

    let mut output = [0_u8; 2];
    let written = engine
        .finish(&mut output, 0)
        .expect("finish should keep draining decoder finish batches");

    assert_eq!(2, written);
    assert_eq!([50, 51], output);
    assert_eq!(Ok(0), engine.max_finish_output_len());
}

#[test]
fn test_buffered_convert_engine_finish_drains_pending_before_decoder_finish_output() {
    let mut engine = new_finish_engine(FinishHooks::default());
    let mut empty_output = [0_u8; 0];
    let progress = engine
        .transcode(&[4], 0, &mut empty_output, 0)
        .expect("conversion should retain decoded input value");
    assert!(matches!(
        progress.status(),
        TranscodeStatus::NeedOutput { .. }
    ));
    assert_eq!(Ok(2), engine.max_finish_output_len());

    let mut output = [0_u8; 1];
    let error = engine
        .finish(&mut output, 0)
        .expect_err("finish should reject partial one-shot output");
    assert_eq!(
        TranscodeError::InsufficientOutput {
            output_index: 0,
            required: 2,
            available: 1
        },
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
    assert_eq!(Ok(1), engine.max_finish_output_len());

    let mut empty_output = [0_u8; 0];
    let error = engine
        .finish(&mut empty_output, 0)
        .expect_err("target finish hook should require one-shot output capacity");
    assert_eq!(
        TranscodeError::InsufficientOutput {
            output_index: 0,
            required: 1,
            available: 0
        },
        error,
    );

    let mut output = [0_u8; 1];
    let written = engine
        .finish(&mut output, 0)
        .expect("target finish hook should write final output");
    assert_eq!(1, written);
    assert_eq!([0xee], output);
    assert_eq!(Ok(0), engine.max_finish_output_len());
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetFailDecodeHooks;

impl TranscodeDecodeHooks<SourceCodec> for ResetFailDecodeHooks {
    type Error = EngineError;

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut SourceCodec,
        error: core::convert::Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, Self::Error> {
        match error {}
    }

    fn reset(&mut self, _codec: &mut SourceCodec) -> Result<(), Self::Error> {
        Err(EngineError::Decode)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetFailDecodeConvertHooks;

impl TranscodeConvertHooks<SourceCodec, TargetCodec> for ResetFailDecodeConvertHooks {
    type DecodeError = EngineError;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
        ConvertEngineError::Encode(error)
    }
}

#[test]
fn test_buffered_convert_engine_reset_emits_target_reset_output() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        ResetEmittingTargetCodec,
        StrictDecodeHooks,
        ResetTargetHooks,
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
fn test_buffered_convert_engine_reset_maps_decode_reset_errors() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        ResetFailDecodeHooks,
        StrictEncodeHooks,
        ResetFailDecodeConvertHooks,
    );

    let error = engine
        .reset(&mut [], 0)
        .expect_err("decode reset errors should be mapped through convert hooks");

    assert_eq!(
        TranscodeError::Domain(ConvertEngineError::Decode(EngineError::Decode)),
        error,
    );
}

#[test]
fn test_buffered_convert_engine_reset_maps_target_reset_errors() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        ResetFailTargetCodec,
        StrictDecodeHooks,
        ResetFailTargetHooks,
        ResetFailTargetHooks,
    );
    let mut output = [0_u8; 1];

    let error = engine
        .reset(&mut output, 0)
        .expect_err("target reset errors should be mapped through convert hooks");

    assert_eq!(
        TranscodeError::Domain(ConvertEngineError::Encode(TargetResetFailError)),
        error,
    );
}

#[test]
fn test_buffered_convert_engine_reset_rejects_invalid_output_index() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        ResetEmittingTargetCodec,
        StrictDecodeHooks,
        ResetTargetHooks,
        ResetTargetHooks,
    );
    let mut output = [0_u8; 1];

    let error = engine
        .reset(&mut output, 2)
        .expect_err("invalid reset output index should be rejected");

    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 2, len: 1 },
        error,
    );
}

#[test]
fn test_buffered_convert_engine_invalid_reset_preserves_pending_value() {
    let mut engine = new_copy_engine();
    let mut empty_output = [0_u8; 0];
    let progress = engine
        .transcode(&[4], 0, &mut empty_output, 0)
        .expect("conversion should retain decoded value");
    assert!(matches!(
        progress.status(),
        TranscodeStatus::NeedOutput { .. }
    ));
    assert_eq!(Ok(1), engine.max_finish_output_len());

    let mut output = [0_u8; 1];
    let error = engine
        .reset(&mut output, 2)
        .expect_err("invalid reset output index should be rejected");

    assert_eq!(
        TranscodeError::InvalidOutputIndex { index: 2, len: 1 },
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
        TargetCodec,
        StrictDecodeHooks,
        MismatchCapacityEncodeHooks,
        MismatchPendingFinishHooks,
    );
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
        TargetCodec,
        FinishDecodeHooks::default(),
        MismatchCapacityEncodeHooks,
        MismatchDecoderFinishHooks,
    );
    let required = engine.max_finish_output_len().expect("finish bound");
    let mut output = vec![0_u8; required];
    let _ = engine.finish(&mut output, 0);
}

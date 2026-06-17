// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the reusable buffered converter engine.

use core::num::NonZeroUsize;

use qubit_codec::{
    CapacityError,
    Codec,
    DecodeAction,
    DecodeContext,
    EncodeContext,
    EncodePlan,
    TranscodeConvertEngine,
    TranscodeConvertHooks,
    TranscodeDecodeHooks,
    TranscodeEncodeHooks,
    TranscodeError,
    TranscodeStatus,
    nz,
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

unsafe impl Codec for SourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, NonZeroUsize), Self::DecodeError> {
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
        Ok(nz!(1))
    }
}

unsafe impl Codec for TargetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = EngineError;

    fn min_units_per_value(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, NonZeroUsize), Self::DecodeError> {
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
        Ok(nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetEmittingTargetCodec;

unsafe impl Codec for ResetEmittingTargetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    fn max_encode_reset_units(&self) -> usize {
        1
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, NonZeroUsize), Self::DecodeError> {
        unsafe { Ok((*input.get_unchecked(index), NonZeroUsize::MIN)) }
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<NonZeroUsize, Self::EncodeError> {
        output[index] = *value;
        Ok(nz!(1))
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

unsafe impl Codec for ResetFailTargetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = TargetResetFailError;

    fn min_units_per_value(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    fn max_encode_reset_units(&self) -> usize {
        1
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, NonZeroUsize), Self::DecodeError> {
        unsafe { Ok((*input.get_unchecked(index), NonZeroUsize::MIN)) }
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<NonZeroUsize, Self::EncodeError> {
        output[index] = *value;
        Ok(nz!(1))
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
    type PlanAction = ();

    fn prepare_encode(
        &mut self,
        codec: &mut ResetEmittingTargetCodec,
        _input_value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        Ok(EncodePlan::new(codec.max_units_per_value().get(), ()))
    }

    unsafe fn write_encode(
        &mut self,
        codec: &mut ResetEmittingTargetCodec,
        context: EncodeContext<'_, u8, u8>,
        _plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        Ok(unsafe {
            codec
                .encode(
                    context.input_value,
                    context.output,
                    context.output_index,
                )
                .expect("infallible target encode")
                .get()
        })
    }
}

impl TranscodeConvertHooks<SourceCodec, ResetEmittingTargetCodec>
    for ResetTargetHooks
{
    type DecodeError = EngineError;
    type DecodeHooks = StrictDecodeHooks;
    type EncodeHooks = ResetTargetHooks;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn create_decode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &ResetEmittingTargetCodec,
    ) -> Self::DecodeHooks {
        StrictDecodeHooks
    }

    fn create_encode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &ResetEmittingTargetCodec,
    ) -> Self::EncodeHooks {
        ResetTargetHooks
    }

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
    type PlanAction = ();

    fn prepare_encode(
        &mut self,
        codec: &mut ResetFailTargetCodec,
        _input_value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        Ok(EncodePlan::new(codec.max_units_per_value().get(), ()))
    }

    unsafe fn write_encode(
        &mut self,
        codec: &mut ResetFailTargetCodec,
        context: EncodeContext<'_, u8, u8>,
        _plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        unsafe {
            codec.encode(
                context.input_value,
                context.output,
                context.output_index,
            )
        }
        .map(NonZeroUsize::get)
    }

    fn map_encode_reset_error(
        &mut self,
        _codec: &mut ResetFailTargetCodec,
        error: TargetResetFailError,
    ) -> Self::Error {
        error
    }
}

impl TranscodeConvertHooks<SourceCodec, ResetFailTargetCodec>
    for ResetFailTargetHooks
{
    type DecodeError = EngineError;
    type DecodeHooks = StrictDecodeHooks;
    type EncodeHooks = ResetFailTargetHooks;
    type EncodeError = TargetResetFailError;
    type Error = ConvertEngineError<TargetResetFailError>;

    fn create_decode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &ResetFailTargetCodec,
    ) -> Self::DecodeHooks {
        StrictDecodeHooks
    }

    fn create_encode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &ResetFailTargetCodec,
    ) -> Self::EncodeHooks {
        ResetFailTargetHooks
    }

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
        ConvertEngineError::Encode(error)
    }
}

unsafe impl Codec for ErrorSourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = EngineError;
    type EncodeError = core::convert::Infallible;

    fn min_units_per_value(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> NonZeroUsize {
        NonZeroUsize::MIN
    }

    unsafe fn decode(
        &mut self,
        _input: &[u8],
        _index: usize,
    ) -> Result<(u8, NonZeroUsize), Self::DecodeError> {
        Err(EngineError::Decode)
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
        Ok(nz!(1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct StrictDecodeHooks;

impl TranscodeDecodeHooks<SourceCodec> for StrictDecodeHooks {
    type Error = EngineError;
    fn handle_decode_error(
        &mut self,
        _codec: &mut SourceCodec,
        error: core::convert::Infallible,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        match error {}
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct StrictEncodeHooks;

impl TranscodeEncodeHooks<TargetCodec> for StrictEncodeHooks {
    type Error = EngineError;
    type PlanAction = ();

    fn prepare_encode(
        &mut self,
        codec: &mut TargetCodec,
        _input_value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        Ok(EncodePlan::new(codec.max_units_per_value().get(), ()))
    }

    unsafe fn write_encode(
        &mut self,
        codec: &mut TargetCodec,
        context: EncodeContext<'_, u8, u8>,
        _plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        let EncodeContext {
            input_value,
            output,
            output_index,
            ..
        } = context;
        // SAFETY: The engine checked the prepared output capacity.
        unsafe { codec.encode(input_value, output, output_index) }
            .map(NonZeroUsize::get)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct MismatchCapacityEncodeHooks;

impl TranscodeEncodeHooks<TargetCodec> for MismatchCapacityEncodeHooks {
    type Error = EngineError;
    type PlanAction = ();

    fn max_output_len(
        &self,
        _codec: &TargetCodec,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn prepare_encode(
        &mut self,
        _codec: &mut TargetCodec,
        _input_value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        Ok(EncodePlan::new(2, ()))
    }

    unsafe fn write_encode(
        &mut self,
        _codec: &mut TargetCodec,
        context: EncodeContext<'_, u8, u8>,
        _plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        if context.output_index + 1 >= context.output.len() {
            return Err(EngineError::Encode);
        }
        unsafe {
            *context.output.get_unchecked_mut(context.output_index) =
                *context.input_value;
            *context.output.get_unchecked_mut(context.output_index + 1) =
                context.input_value.wrapping_add(1);
        }
        Ok(2)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct MismatchPendingFinishHooks;

impl TranscodeConvertHooks<SourceCodec, TargetCodec>
    for MismatchPendingFinishHooks
{
    type DecodeError = EngineError;
    type DecodeHooks = StrictDecodeHooks;
    type EncodeHooks = MismatchCapacityEncodeHooks;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn create_decode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::DecodeHooks {
        StrictDecodeHooks
    }

    fn create_encode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::EncodeHooks {
        MismatchCapacityEncodeHooks
    }

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
        ConvertEngineError::Encode(error)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct MismatchDecoderFinishHooks;

impl TranscodeConvertHooks<SourceCodec, TargetCodec>
    for MismatchDecoderFinishHooks
{
    type DecodeError = EngineError;
    type DecodeHooks = FinishDecodeHooks;
    type EncodeHooks = MismatchCapacityEncodeHooks;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn create_decode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::DecodeHooks {
        FinishDecodeHooks::default()
    }

    fn create_encode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::EncodeHooks {
        MismatchCapacityEncodeHooks
    }

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
    NeedInput,
    Skip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RepairDecodeHooks {
    action: RepairAction,
}

impl TranscodeDecodeHooks<ErrorSourceCodec> for RepairDecodeHooks {
    type Error = EngineError;

    fn handle_decode_error(
        &mut self,
        _codec: &mut ErrorSourceCodec,
        _error: EngineError,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        match self.action {
            RepairAction::Emit => Ok(DecodeAction::Emit {
                value: 42,
                consumed: one_consumed(),
            }),
            RepairAction::NeedInput => {
                Ok(DecodeAction::NeedInput { required_total: 3 })
            }
            RepairAction::Skip => Ok(DecodeAction::Skip {
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
    type DecodeHooks = RepairDecodeHooks;
    type EncodeHooks = StrictEncodeHooks;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn create_decode_hooks(
        &self,
        _decode_codec: &ErrorSourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::DecodeHooks {
        RepairDecodeHooks {
            action: self.action,
        }
    }

    fn create_encode_hooks(
        &self,
        _decode_codec: &ErrorSourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::EncodeHooks {
        StrictEncodeHooks
    }

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
        ConvertEngineError::Encode(error)
    }
}

impl TranscodeConvertHooks<SourceCodec, TargetCodec> for CopyHooks {
    type DecodeError = EngineError;
    type DecodeHooks = StrictDecodeHooks;
    type EncodeHooks = StrictEncodeHooks;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn create_decode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::DecodeHooks {
        StrictDecodeHooks
    }

    fn create_encode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::EncodeHooks {
        StrictEncodeHooks
    }

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

    fn handle_decode_error(
        &mut self,
        _codec: &mut SourceCodec,
        error: core::convert::Infallible,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
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
    type DecodeHooks = FinishDecodeHooks;
    type EncodeHooks = StrictEncodeHooks;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn create_decode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::DecodeHooks {
        FinishDecodeHooks {
            value: Some(self.value),
        }
    }

    fn create_encode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::EncodeHooks {
        StrictEncodeHooks
    }

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

    fn handle_decode_error(
        &mut self,
        _codec: &mut SourceCodec,
        error: core::convert::Infallible,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
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
    type DecodeHooks = BatchFinishDecodeHooks;
    type EncodeHooks = StrictEncodeHooks;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn create_decode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::DecodeHooks {
        BatchFinishDecodeHooks::default()
    }

    fn create_encode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::EncodeHooks {
        StrictEncodeHooks
    }

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
    type PlanAction = ();

    fn prepare_encode(
        &mut self,
        codec: &mut TargetCodec,
        _input_value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        Ok(EncodePlan::new(codec.max_units_per_value().get(), ()))
    }

    unsafe fn write_encode(
        &mut self,
        codec: &mut TargetCodec,
        context: EncodeContext<'_, u8, u8>,
        _plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        let EncodeContext {
            input_value,
            output,
            output_index,
            ..
        } = context;
        // SAFETY: The engine checked the prepared output capacity.
        unsafe { codec.encode(input_value, output, output_index) }
            .map(NonZeroUsize::get)
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
    type DecodeHooks = StrictDecodeHooks;
    type EncodeHooks = FinishEncodeHooks;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn create_decode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::DecodeHooks {
        StrictDecodeHooks
    }

    fn create_encode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::EncodeHooks {
        FinishEncodeHooks::default()
    }

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
        codec: &SourceCodec,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
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
        _codec: &mut SourceCodec,
        error: core::convert::Infallible,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
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
    type PlanAction = ();

    fn max_output_len(
        &self,
        codec: &TargetCodec,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
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
        codec: &mut TargetCodec,
        _input_value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        match self.mode {
            ErrorPathEncodeMode::PrepareError => Err(EngineError::Encode),
            ErrorPathEncodeMode::Normal | ErrorPathEncodeMode::FinishError => {
                Ok(EncodePlan::new(codec.max_units_per_value().get(), ()))
            }
        }
    }

    unsafe fn write_encode(
        &mut self,
        _codec: &mut TargetCodec,
        _context: EncodeContext<'_, u8, u8>,
        _plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }

    fn finish(
        &mut self,
        _codec: &mut TargetCodec,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::Error> {
        match self.mode {
            ErrorPathEncodeMode::FinishError => Err(EngineError::Encode),
            ErrorPathEncodeMode::Normal | ErrorPathEncodeMode::PrepareError => {
                Ok(0)
            }
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
    type DecodeHooks = ErrorPathDecodeHooks;
    type EncodeHooks = ErrorPathEncodeHooks;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn create_decode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::DecodeHooks {
        ErrorPathDecodeHooks {
            finish: self.decode_finish,
            finish_len: self.decode_finish_len,
            max_output_error: self.decode_max_output_error,
        }
    }

    fn create_encode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::EncodeHooks {
        ErrorPathEncodeHooks {
            finish_len: self.encode_finish_len,
            max_output_error: self.encode_max_output_error,
            mode: self.encode_mode,
        }
    }

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

    fn handle_decode_error(
        &mut self,
        _codec: &mut SourceCodec,
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

impl TranscodeEncodeHooks<TargetCodec> for FactoryEncodeHooks {
    type Error = EngineError;
    type PlanAction = ();

    fn prepare_encode(
        &mut self,
        codec: &mut TargetCodec,
        _input_value: &u8,
        _input_index: usize,
    ) -> Result<EncodePlan<Self::PlanAction>, Self::Error> {
        Ok(EncodePlan::new(codec.max_units_per_value().get(), ()))
    }

    unsafe fn write_encode(
        &mut self,
        _codec: &mut TargetCodec,
        context: EncodeContext<'_, u8, u8>,
        _plan: EncodePlan<Self::PlanAction>,
    ) -> Result<usize, Self::Error> {
        let EncodeContext {
            input_value,
            output,
            output_index,
            ..
        } = context;
        output[output_index] = input_value.wrapping_add(self.offset);
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FactoryHooks {
    decode_marker: u8,
    encode_offset: u8,
}

impl TranscodeConvertHooks<SourceCodec, TargetCodec> for FactoryHooks {
    type DecodeError = EngineError;
    type DecodeHooks = FactoryDecodeHooks;
    type EncodeHooks = FactoryEncodeHooks;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn create_decode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::DecodeHooks {
        FactoryDecodeHooks {
            marker: self.decode_marker,
        }
    }

    fn create_encode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::EncodeHooks {
        FactoryEncodeHooks {
            offset: self.encode_offset,
        }
    }

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
        ConvertEngineError::Encode(error)
    }
}

#[test]
fn test_buffered_convert_engine_reports_bounds_and_resets() {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        CopyHooks::default(),
    );

    assert_eq!(Ok(3), engine.max_output_len(3));
    assert_eq!(Ok(0), engine.max_finish_output_len());
    assert_eq!(Ok(0), engine.max_reset_output_len());

    engine.reset(&mut [], 0).expect("reset");
    assert_eq!(Ok(0), engine.max_finish_output_len());
}

#[test]
fn test_buffered_convert_engine_default_builds_engine() {
    let mut engine =
        TranscodeConvertEngine::<SourceCodec, TargetCodec, CopyHooks>::default(
        );
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
fn test_buffered_convert_engine_new_uses_convert_hook_factories() {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        FactoryHooks {
            decode_marker: 11,
            encode_offset: 7,
        },
    );

    assert_eq!(Ok(11), engine.max_output_len(1));

    let mut output = [0_u8; 1];
    let progress = engine
        .transcode(&[1], 0, &mut output, 0)
        .expect("factory-created encode hooks should convert the value");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((1, 1), (progress.read(), progress.written()));
    assert_eq!([9], output);

    engine.reset(&mut [], 0).expect("reset");
}

#[test]
fn test_buffered_convert_engine_owns_pending_value_between_calls() {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        CopyHooks::default(),
    );
    let mut empty_output = [0_u8; 0];

    let progress = engine
        .transcode(&[1], 0, &mut empty_output, 0)
        .expect("conversion should retain decoded value when output is empty");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: nz(1),
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
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        CopyHooks::default(),
    );
    let mut empty_output = [0_u8; 0];

    let progress = engine
        .transcode(&[1], 0, &mut empty_output, 0)
        .expect("conversion should retain decoded value when output is empty");
    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!((1, 0), (progress.read(), progress.written()));

    let progress = engine.transcode(&[9], 0, &mut empty_output, 0).expect(
        "conversion should report pending output before reading new input",
    );
    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            additional: nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!((0, 0), (progress.read(), progress.written()));
    assert_eq!(Ok(2), engine.max_output_len(1));

    let mut output = [0_u8; 2];
    let progress = engine.transcode(&[9], 0, &mut output, 0).expect(
        "conversion should keep pending value after repeated output starvation",
    );
    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((1, 2), (progress.read(), progress.written()));
    assert_eq!([2, 10], output);
}

#[test]
fn test_buffered_convert_engine_maps_pending_encode_error_before_new_input() {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        CopyHooks::default(),
    );
    let mut empty_output = [0_u8; 0];
    let progress = engine
        .transcode(&[12], 0, &mut empty_output, 0)
        .expect("conversion should retain decoded value before encoding");
    assert!(matches!(
        progress.status(),
        TranscodeStatus::NeedOutput { .. }
    ));

    let mut output = [0_u8; 1];
    let error = engine.transcode(&[1], 0, &mut output, 0).expect_err(
        "pending encode error should be mapped before new input is consumed",
    );

    assert_eq!(
        TranscodeError::Domain(ConvertEngineError::Encode(EngineError::Encode)),
        error
    );
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_reports_invalid_indices() {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        CopyHooks::default(),
    );
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
    let engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        ErrorPathHooks {
            decode_max_output_error: true,
            ..ErrorPathHooks::default()
        },
    );
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_output_len(1)
    );

    let engine = TranscodeConvertEngine::<_, _, _>::new(
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
        engine.max_finish_output_len()
    );

    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        ErrorPathHooks {
            encode_max_output_error: true,
            ..ErrorPathHooks::default()
        },
    );
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

    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        ErrorPathHooks {
            decode_finish_len: usize::MAX,
            ..ErrorPathHooks::default()
        },
    );
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
fn test_buffered_convert_engine_maps_prepare_encode_error() {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        ErrorPathHooks {
            decode_finish: ErrorPathDecodeFinish::Normal,
            encode_mode: ErrorPathEncodeMode::PrepareError,
            ..ErrorPathHooks::default()
        },
    );
    let mut output = [0_u8; 1];

    let error = engine.transcode(&[1], 0, &mut output, 0).expect_err(
        "prepare encode error should be mapped through convert hooks",
    );

    assert_eq!(
        TranscodeError::Domain(ConvertEngineError::Encode(EngineError::Encode)),
        error
    );
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_finish_reports_output_index_beyond_buffer() {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        CopyHooks::default(),
    );
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
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        ErrorPathHooks {
            decode_finish: ErrorPathDecodeFinish::Error,
            encode_mode: ErrorPathEncodeMode::Normal,
            ..ErrorPathHooks::default()
        },
    );
    let mut output = [0_u8; 1];

    let error = engine.finish(&mut output, 0).expect_err(
        "decode finish error should be mapped through convert hooks",
    );

    assert_eq!(
        TranscodeError::Domain(ConvertEngineError::Decode(EngineError::Decode)),
        error
    );
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_finish_maps_encode_error() {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        ErrorPathHooks {
            decode_finish: ErrorPathDecodeFinish::Normal,
            encode_mode: ErrorPathEncodeMode::FinishError,
            ..ErrorPathHooks::default()
        },
    );
    let mut output = [0_u8; 1];

    let error = engine.finish(&mut output, 0).expect_err(
        "encode finish error should be mapped through convert hooks",
    );

    assert_eq!(
        TranscodeError::Domain(ConvertEngineError::Encode(EngineError::Encode)),
        error
    );
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_finish_maps_pending_encode_error() {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        CopyHooks::default(),
    );
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
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        FinishHooks { value: 13 },
    );
    let mut output = [0_u8; 1];

    let error = engine.finish(&mut output, 0).expect_err(
        "finish should map encode errors for decoder-emitted values",
    );

    assert_eq!(
        TranscodeError::Domain(ConvertEngineError::Encode(EngineError::Encode)),
        error
    );
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_applies_decode_policy_skip() {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        ErrorSourceCodec,
        TargetCodec,
        RepairHooks {
            action: RepairAction::Skip,
        },
    );
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
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        ErrorSourceCodec,
        TargetCodec,
        RepairHooks {
            action: RepairAction::Emit,
        },
    );
    let mut output = [0_u8; 2];

    let progress = engine
        .transcode(&[1, 2], 0, &mut output, 0)
        .expect("emit policy should replace invalid source units");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((2, 2), (progress.read(), progress.written()));
    assert_eq!([42, 42], output);
}

#[test]
fn test_buffered_convert_engine_applies_decode_policy_need_input() {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        ErrorSourceCodec,
        TargetCodec,
        RepairHooks {
            action: RepairAction::NeedInput,
        },
    );
    let mut output = [0_u8; 1];

    let progress = engine
        .transcode(&[1], 0, &mut output, 0)
        .expect("need-input policy should stop without consuming input");

    assert_eq!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            additional: nz(2),
            available: 1,
        },
        progress.status(),
    );
    assert_eq!((0, 0), (progress.read(), progress.written()));
}

#[test]
fn test_buffered_convert_engine_finish_drains_pending_value() {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        CopyHooks::default(),
    );
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
            available: 0,
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
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        FinishHooks::default(),
    );
    assert_eq!(Ok(1), engine.max_finish_output_len());

    let mut empty_output = [0_u8; 0];
    let error = engine.finish(&mut empty_output, 0).expect_err(
        "finish should reject insufficient output before decoder finish",
    );
    assert_eq!(
        TranscodeError::InsufficientOutput {
            output_index: 0,
            required: 1,
            available: 0,
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
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        BatchFinishHooks,
    );
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
fn test_buffered_convert_engine_finish_drains_pending_before_decoder_finish_output()
 {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        FinishHooks::default(),
    );
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
            available: 1,
        },
        error,
    );
    assert_eq!([0], output);
    assert_eq!(Ok(2), engine.max_finish_output_len());

    let mut output = [0_u8; 2];
    let written = engine.finish(&mut output, 0).expect(
        "finish should write pending input value before decoder finish value",
    );
    assert_eq!(2, written);
    assert_eq!([5, 40], output);
}

#[test]
fn test_buffered_convert_engine_finish_delegates_to_encoder_finish() {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        FinishEncodeHooksOnly,
    );
    assert_eq!(Ok(1), engine.max_finish_output_len());

    let mut empty_output = [0_u8; 0];
    let error = engine.finish(&mut empty_output, 0).expect_err(
        "target finish hook should require one-shot output capacity",
    );
    assert_eq!(
        TranscodeError::InsufficientOutput {
            output_index: 0,
            required: 1,
            available: 0,
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

    fn handle_decode_error(
        &mut self,
        _codec: &mut SourceCodec,
        error: core::convert::Infallible,
        _context: DecodeContext,
    ) -> Result<DecodeAction<u8>, Self::Error> {
        match error {}
    }

    fn reset(&mut self, _codec: &mut SourceCodec) -> Result<(), Self::Error> {
        Err(EngineError::Decode)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetFailDecodeConvertHooks;

impl TranscodeConvertHooks<SourceCodec, TargetCodec>
    for ResetFailDecodeConvertHooks
{
    type DecodeError = EngineError;
    type DecodeHooks = ResetFailDecodeHooks;
    type EncodeHooks = StrictEncodeHooks;
    type EncodeError = EngineError;
    type Error = ConvertEngineError<EngineError>;

    fn create_decode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::DecodeHooks {
        ResetFailDecodeHooks
    }

    fn create_encode_hooks(
        &self,
        _decode_codec: &SourceCodec,
        _encode_codec: &TargetCodec,
    ) -> Self::EncodeHooks {
        StrictEncodeHooks
    }

    fn map_decode_error(&self, error: Self::DecodeError) -> Self::Error {
        ConvertEngineError::Decode(error)
    }

    fn map_encode_error(&self, error: Self::EncodeError) -> Self::Error {
        ConvertEngineError::Encode(error)
    }
}

#[test]
fn test_buffered_convert_engine_reset_emits_target_reset_output() {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        ResetEmittingTargetCodec,
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
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        ResetFailDecodeConvertHooks,
    );

    let error = engine.reset(&mut [], 0).expect_err(
        "decode reset errors should be mapped through convert hooks",
    );

    assert_eq!(
        TranscodeError::Domain(ConvertEngineError::Decode(EngineError::Decode)),
        error,
    );
}

#[test]
fn test_buffered_convert_engine_reset_maps_target_reset_errors() {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        ResetFailTargetCodec,
        ResetFailTargetHooks,
    );
    let mut output = [0_u8; 1];

    let error = engine.reset(&mut output, 0).expect_err(
        "target reset errors should be mapped through convert hooks",
    );

    assert_eq!(
        TranscodeError::Domain(ConvertEngineError::Encode(
            TargetResetFailError
        )),
        error,
    );
}

#[test]
fn test_buffered_convert_engine_reset_rejects_invalid_output_index() {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        ResetEmittingTargetCodec,
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
#[should_panic(
    expected = "converter finish bound must reserve space for pending values"
)]
fn test_buffered_convert_engine_finish_hits_pending_unreachable() {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        MismatchPendingFinishHooks,
    );
    let mut empty_output = [0_u8; 0];

    let _ = engine.transcode(&[1], 0, &mut empty_output, 0);
    let required = engine.max_finish_output_len().expect("finish bound");
    let mut output = vec![0_u8; required];
    let _ = engine.finish(&mut output, 0);
}

#[test]
#[should_panic(
    expected = "converter finish bound must reserve space for decode finish values"
)]
fn test_buffered_convert_engine_finish_hits_decoder_finish_unreachable() {
    let mut engine = TranscodeConvertEngine::<_, _, _>::new(
        SourceCodec,
        TargetCodec,
        MismatchDecoderFinishHooks,
    );
    let required = engine.max_finish_output_len().expect("finish bound");
    let mut output = vec![0_u8; required];
    let _ = engine.finish(&mut output, 0);
}

// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Tests for the reusable buffered converter engine.

use core::{
    cell::Cell,
    num::NonZeroUsize,
};
use std::rc::Rc;

use qubit_codec::engine::{
    DecodeContext,
    DecodeInvalidAction,
    EncodeContext,
    EncodeUnencodableAction,
    TranscodeConvertEngine,
    TranscodeDecodeHooks,
    TranscodeEncodeHooks,
};
use qubit_codec::{
    CapacityError,
    Codec,
    TranscodeConvertError,
    TranscodeDecodeError,
    TranscodeEncodeError,
    TranscodeProgress,
    TranscodeStatus,
    Transcoder,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct SourceCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct TargetCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ErrorSourceCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FinishValueSourceCodec<const FINISH_BOUND: usize>;

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

impl From<core::convert::Infallible> for EngineError {
    fn from(error: core::convert::Infallible) -> Self {
        match error {}
    }
}

impl Codec for SourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = EngineError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_UNITS_PER_VALUE: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
    {
        // SAFETY: The caller proved that at least one input unit is readable.
        let value = unsafe { *input.get_unchecked(input_index) };
        Ok((value.wrapping_add(1), NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        // SAFETY: The caller proved that one output unit is writable.
        unsafe {
            *output.get_unchecked_mut(output_index) = *value;
        }
        Ok(1)
    }
}

impl Codec for TargetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = EngineError;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_UNITS_PER_VALUE: usize = 1;

    fn can_encode_value(&self, value: &u8) -> bool {
        *value != 99
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
    {
        // SAFETY: The caller proved that at least one input unit is readable.
        let value = unsafe { *input.get_unchecked(input_index) };
        Ok((value, NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        if *value == 13 {
            return Err(EngineError::Encode);
        }
        // SAFETY: The caller proved that one output unit is writable.
        unsafe {
            *output.get_unchecked_mut(output_index) = *value;
        }
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetEmittingTargetCodec;

impl Codec for ResetEmittingTargetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_RESET_UNITS: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
    {
        unsafe { Ok((*input.get_unchecked(input_index), NonZeroUsize::MIN)) }
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(1)
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
struct ResetFailTargetCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FinishOverflowTargetCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct MismatchCapacityTargetCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("target reset failed")]
struct TargetResetFailError;

impl Codec for ResetFailTargetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = TargetResetFailError;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_RESET_UNITS: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
    {
        unsafe { Ok((*input.get_unchecked(input_index), NonZeroUsize::MIN)) }
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(1)
    }

    unsafe fn encode_reset(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Err(TargetResetFailError)
    }
}

impl Codec for FinishOverflowTargetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_FINISH_UNITS: usize = usize::MAX;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
    {
        unsafe { Ok((*input.get_unchecked(input_index), NonZeroUsize::MIN)) }
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = *value;
        Ok(1)
    }
}

impl Codec for MismatchCapacityTargetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = EngineError;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
    {
        unsafe { Ok((*input.get_unchecked(input_index), NonZeroUsize::MIN)) }
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        output[output_index] = *value;
        output[output_index + 1] = value.wrapping_add(1);
        Ok(2)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetTargetHooks;

impl TranscodeEncodeHooks<ResetEmittingTargetCodec> for ResetTargetHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut ResetEmittingTargetCodec,
        _context: &EncodeContext<'_, u8, u8>,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeErrorOf<ResetEmittingTargetCodec>,
    > {
        Ok(EncodeUnencodableAction::Reject)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FinishOverflowEncodeHooks;

impl TranscodeEncodeHooks<FinishOverflowTargetCodec>
    for FinishOverflowEncodeHooks
{
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut FinishOverflowTargetCodec,
        _context: &EncodeContext<'_, u8, u8>,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeErrorOf<FinishOverflowTargetCodec>,
    > {
        Ok(EncodeUnencodableAction::Reject)
    }

    fn max_finish_output_len(
        &self,
        _codec: &FinishOverflowTargetCodec,
    ) -> usize {
        1
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetFailTargetHooks;

impl TranscodeEncodeHooks<ResetFailTargetCodec> for ResetFailTargetHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut ResetFailTargetCodec,
        _context: &EncodeContext<'_, u8, u8>,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeErrorOf<ResetFailTargetCodec>,
    > {
        Err(TranscodeEncodeError::domain_main(
            TargetResetFailError,
            _context.input_index(),
        ))
    }
}

impl Codec for ErrorSourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = EngineError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_UNITS_PER_VALUE: usize = 1;

    unsafe fn decode(
        &mut self,
        _input: &[u8],
        _input_index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
    {
        Err(qubit_codec::DecodeFailure::invalid_unknown(
            EngineError::Decode,
        ))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        // SAFETY: The caller proved that one output unit is writable.
        unsafe {
            *output.get_unchecked_mut(output_index) = *value;
        }
        Ok(1)
    }
}

impl<const FINISH_BOUND: usize> Codec for FinishValueSourceCodec<FINISH_BOUND> {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_FINISH_VALUES: usize = FINISH_BOUND;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
    {
        // SAFETY: The caller proved that at least one input unit is readable.
        let value = unsafe { *input.get_unchecked(input_index) };
        Ok((value, NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        // SAFETY: The caller proved that one output unit is writable.
        unsafe {
            *output.get_unchecked_mut(output_index) = *value;
        }
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct StrictDecodeHooks;

impl TranscodeDecodeHooks<SourceCodec> for StrictDecodeHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut SourceCodec,
        error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<
        DecodeInvalidAction<u8>,
        qubit_codec::TranscodeDecodeErrorOf<SourceCodec>,
    > {
        match error {
            EngineError::Decode | EngineError::Encode => {
                unreachable!("SourceCodec should not produce decode errors")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ImpossibleFailureDecodeHooks;

impl TranscodeDecodeHooks<ErrorSourceCodec> for ImpossibleFailureDecodeHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut ErrorSourceCodec,
        _error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<
        DecodeInvalidAction<u8>,
        qubit_codec::TranscodeDecodeErrorOf<ErrorSourceCodec>,
    > {
        Err(qubit_codec::TranscodeDecodeError::domain_main(
            EngineError::Decode,
            0,
        ))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct StrictEncodeHooks;

impl<C> TranscodeEncodeHooks<C> for StrictEncodeHooks
where
    C: Codec<Value = u8, Unit = u8, EncodeError = EngineError>,
{
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut C,
        _context: &EncodeContext<'_, u8, u8>,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeErrorOf<C>,
    > {
        Err(TranscodeEncodeError::domain_main(
            EngineError::Encode,
            _context.input_index(),
        ))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct MismatchCapacityEncodeHooks;

impl TranscodeEncodeHooks<MismatchCapacityTargetCodec>
    for MismatchCapacityEncodeHooks
{
    fn max_transcode_output_len(
        &self,
        _codec: &MismatchCapacityTargetCodec,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut MismatchCapacityTargetCodec,
        _context: &EncodeContext<'_, u8, u8>,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeErrorOf<MismatchCapacityTargetCodec>,
    > {
        Err(TranscodeEncodeError::domain_main(
            EngineError::Encode,
            _context.input_index(),
        ))
    }
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
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut ErrorSourceCodec,
        _error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<
        DecodeInvalidAction<u8>,
        qubit_codec::TranscodeDecodeErrorOf<ErrorSourceCodec>,
    > {
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
struct FixedFinishBoundDecodeHooks {
    finish_len: usize,
}

impl<const FINISH_BOUND: usize>
    TranscodeDecodeHooks<FinishValueSourceCodec<FINISH_BOUND>>
    for FixedFinishBoundDecodeHooks
{
    fn max_finish_output_len(
        &self,
        _codec: &FinishValueSourceCodec<FINISH_BOUND>,
    ) -> usize {
        self.finish_len
    }

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut FinishValueSourceCodec<FINISH_BOUND>,
        error: &core::convert::Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<
        DecodeInvalidAction<u8>,
        qubit_codec::TranscodeDecodeErrorOf<
            FinishValueSourceCodec<FINISH_BOUND>,
        >,
    > {
        match *error {}
    }
}

#[derive(Debug)]
struct ChangingFinishBoundDecodeHooks {
    calls: Cell<usize>,
}

impl<const FINISH_BOUND: usize>
    TranscodeDecodeHooks<FinishValueSourceCodec<FINISH_BOUND>>
    for ChangingFinishBoundDecodeHooks
{
    fn max_finish_output_len(
        &self,
        _codec: &FinishValueSourceCodec<FINISH_BOUND>,
    ) -> usize {
        let calls = self.calls.get();
        self.calls.set(calls + 1);
        if calls == 0 { 0 } else { usize::MAX }
    }

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut FinishValueSourceCodec<FINISH_BOUND>,
        error: &core::convert::Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<
        DecodeInvalidAction<u8>,
        qubit_codec::TranscodeDecodeErrorOf<
            FinishValueSourceCodec<FINISH_BOUND>,
        >,
    > {
        match *error {}
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
    fn max_finish_output_len(&self, _codec: &SourceCodec) -> usize {
        usize::from(self.value.is_some())
    }

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut SourceCodec,
        error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<
        DecodeInvalidAction<u8>,
        qubit_codec::TranscodeDecodeErrorOf<SourceCodec>,
    > {
        match error {
            EngineError::Decode | EngineError::Encode => {
                unreachable!("SourceCodec should not produce decode errors")
            }
        }
    }

    fn finish_hooks(
        &mut self,
        _codec: &mut SourceCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, qubit_codec::TranscodeDecodeErrorOf<SourceCodec>> {
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
    fn max_finish_output_len(&self, _codec: &SourceCodec) -> usize {
        self.remaining as usize
    }

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut SourceCodec,
        error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<
        DecodeInvalidAction<u8>,
        qubit_codec::TranscodeDecodeErrorOf<SourceCodec>,
    > {
        match error {
            EngineError::Decode | EngineError::Encode => {
                unreachable!("SourceCodec should not produce decode errors")
            }
        }
    }

    fn finish_hooks(
        &mut self,
        _codec: &mut SourceCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, qubit_codec::TranscodeDecodeErrorOf<SourceCodec>> {
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
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut TargetCodec,
        context: &EncodeContext<'_, u8, u8>,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeErrorOf<TargetCodec>,
    > {
        Err(TranscodeEncodeError::domain_main(
            EngineError::Encode,
            context.input_index(),
        ))
    }

    fn max_finish_output_len(&self, _codec: &TargetCodec) -> usize {
        usize::from(self.pending)
    }

    fn finish_hooks(
        &mut self,
        _codec: &mut TargetCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, qubit_codec::TranscodeEncodeErrorOf<TargetCodec>> {
        if !self.pending {
            return Ok(0);
        }
        output[output_index] = 0xee;
        self.pending = false;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ErrorPathDecodeHooks {
    finish: ErrorPathDecodeFinish,
    finish_len: usize,
    max_output_error: bool,
}

impl TranscodeDecodeHooks<SourceCodec> for ErrorPathDecodeHooks {
    fn max_transcode_output_len(
        &self,
        _codec: &SourceCodec,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        if self.max_output_error {
            Err(CapacityError::OutputLengthOverflow)
        } else {
            Ok(input_len / <SourceCodec as Codec>::MIN_UNITS_PER_VALUE)
        }
    }

    fn max_finish_output_len(&self, _codec: &SourceCodec) -> usize {
        self.finish_len
    }

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut SourceCodec,
        error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<
        DecodeInvalidAction<u8>,
        qubit_codec::TranscodeDecodeErrorOf<SourceCodec>,
    > {
        match error {
            EngineError::Decode | EngineError::Encode => {
                unreachable!("SourceCodec should not produce decode errors")
            }
        }
    }

    fn finish_hooks(
        &mut self,
        _codec: &mut SourceCodec,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, qubit_codec::TranscodeDecodeErrorOf<SourceCodec>> {
        match self.finish {
            ErrorPathDecodeFinish::Normal => Ok(0),
            ErrorPathDecodeFinish::Error => {
                Err(TranscodeDecodeError::domain_finish(EngineError::Decode))
            }
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
    fn max_transcode_output_len(
        &self,
        _codec: &TargetCodec,
        input_len: usize,
    ) -> Result<usize, CapacityError> {
        if self.max_output_error {
            Err(CapacityError::OutputLengthOverflow)
        } else {
            input_len
                .checked_mul(<TargetCodec as Codec>::MAX_UNITS_PER_VALUE)
                .ok_or(CapacityError::OutputLengthOverflow)
        }
    }

    fn max_finish_output_len(&self, _codec: &TargetCodec) -> usize {
        self.finish_len
    }

    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut TargetCodec,
        _context: &EncodeContext<'_, u8, u8>,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeErrorOf<TargetCodec>,
    > {
        match self.mode {
            ErrorPathEncodeMode::PrepareError => {
                Err(TranscodeEncodeError::domain_main(
                    EngineError::Encode,
                    _context.input_index(),
                ))
            }
            ErrorPathEncodeMode::Normal | ErrorPathEncodeMode::FinishError => {
                Ok(EncodeUnencodableAction::Skip)
            }
        }
    }

    fn finish_hooks(
        &mut self,
        _codec: &mut TargetCodec,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, qubit_codec::TranscodeEncodeErrorOf<TargetCodec>> {
        match self.mode {
            ErrorPathEncodeMode::FinishError => {
                Err(TranscodeEncodeError::domain_finish(EngineError::Encode))
            }
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FactoryDecodeHooks {
    marker: u8,
}

impl TranscodeDecodeHooks<SourceCodec> for FactoryDecodeHooks {
    fn max_transcode_output_len(
        &self,
        _codec: &SourceCodec,
        _input_len: usize,
    ) -> Result<usize, CapacityError> {
        Ok(self.marker as usize)
    }

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut SourceCodec,
        error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<
        DecodeInvalidAction<u8>,
        qubit_codec::TranscodeDecodeErrorOf<SourceCodec>,
    > {
        match error {
            EngineError::Decode | EngineError::Encode => {
                unreachable!("SourceCodec should not produce decode errors")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FactoryEncodeHooks {
    offset: u8,
}

impl TranscodeEncodeHooks<TargetCodec> for FactoryEncodeHooks {
    fn max_finish_output_len(&self, _codec: &TargetCodec) -> usize {
        self.offset as usize
    }

    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut TargetCodec,
        _context: &EncodeContext<'_, u8, u8>,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeErrorOf<TargetCodec>,
    > {
        Err(TranscodeEncodeError::domain_main(
            EngineError::Encode,
            _context.input_index(),
        ))
    }
}

type CopyConvertEngine = TranscodeConvertEngine<
    SourceCodec,
    TargetCodec,
    StrictDecodeHooks,
    StrictEncodeHooks,
>;

fn new_copy_engine() -> CopyConvertEngine {
    TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        StrictDecodeHooks,
        StrictEncodeHooks,
    )
}

#[test]
fn test_transcode_convert_engine_exposes_codecs_hooks_and_parts() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        StrictDecodeHooks,
        StrictEncodeHooks,
    );

    assert_eq!(&SourceCodec, engine.source_codec());
    assert_eq!(&TargetCodec, engine.target_codec());
    *engine.source_codec_mut() = SourceCodec;
    *engine.target_codec_mut() = TargetCodec;

    let (source, target, decode_hooks, encode_hooks) = engine.into_parts();
    assert_eq!(SourceCodec, source);
    assert_eq!(TargetCodec, target);
    assert_eq!(StrictDecodeHooks, decode_hooks);
    assert_eq!(StrictEncodeHooks, encode_hooks);
}

fn new_error_path_engine(
    hooks: ErrorPathHooks,
) -> TranscodeConvertEngine<
    SourceCodec,
    TargetCodec,
    ErrorPathDecodeHooks,
    ErrorPathEncodeHooks,
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
    TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        decode_hooks,
        encode_hooks,
    )
}

fn new_finish_engine(
    hooks: FinishHooks,
) -> TranscodeConvertEngine<
    SourceCodec,
    TargetCodec,
    FinishDecodeHooks,
    StrictEncodeHooks,
> {
    TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        FinishDecodeHooks {
            value: Some(hooks.value),
        },
        StrictEncodeHooks,
    )
}

fn new_batch_finish_engine() -> TranscodeConvertEngine<
    SourceCodec,
    TargetCodec,
    BatchFinishDecodeHooks,
    StrictEncodeHooks,
> {
    TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        BatchFinishDecodeHooks::default(),
        StrictEncodeHooks,
    )
}

fn new_finish_encode_engine() -> TranscodeConvertEngine<
    SourceCodec,
    TargetCodec,
    StrictDecodeHooks,
    FinishEncodeHooks,
> {
    TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        StrictDecodeHooks,
        FinishEncodeHooks::default(),
    )
}

fn new_repair_engine(
    action: RepairAction,
) -> TranscodeConvertEngine<
    ErrorSourceCodec,
    TargetCodec,
    RepairDecodeHooks,
    StrictEncodeHooks,
> {
    TranscodeConvertEngine::new(
        ErrorSourceCodec,
        TargetCodec,
        RepairDecodeHooks { action },
        StrictEncodeHooks,
    )
}

#[test]
fn test_buffered_convert_engine_reports_bounds_and_resets() {
    type ConvertErrorType = TranscodeConvertError<EngineError, EngineError, u8>;
    type TranscodeCompleteIntoFn = fn(
        &mut CopyConvertEngine,
        &[u8],
        &mut [u8],
    ) -> Result<usize, ConvertErrorType>;

    let mut engine = new_copy_engine();
    let max_total_output_len: fn(
        &CopyConvertEngine,
        usize,
    ) -> Result<usize, CapacityError> = CopyConvertEngine::max_total_output_len;
    let transcode_complete_into: TranscodeCompleteIntoFn =
        CopyConvertEngine::transcode_complete_into;

    assert_eq!(Ok(3), engine.max_transcode_output_len(3));
    assert_eq!(Ok(3), max_total_output_len(&engine, 3));
    assert_eq!(Ok(0), engine.max_finish_output_len());
    assert_eq!(Ok(0), engine.max_reset_output_len());

    let mut output = [0_u8; 3];
    let written = transcode_complete_into(&mut engine, &[1, 2, 3], &mut output)
        .expect("complete conversion should fit the planned output");
    assert_eq!(3, written);
    assert_eq!(&[2, 3, 4], &output[..written]);

    engine.reset(&mut [], 0).expect("reset");
    assert_eq!(Ok(0), engine.max_finish_output_len());
}

#[test]
fn test_buffered_convert_engine_implements_transcoder() {
    type EngineResult<T> =
        Result<T, TranscodeConvertError<EngineError, EngineError, u8>>;
    type TranscodeFn = fn(
        &mut CopyConvertEngine,
        &[u8],
        usize,
        &mut [u8],
        usize,
    ) -> EngineResult<TranscodeProgress>;
    type OutputFn =
        fn(&mut CopyConvertEngine, &mut [u8], usize) -> EngineResult<usize>;

    let mut engine = new_copy_engine();
    let mut output = [0_u8; 2];
    let max_transcode_output_len: fn(
        &CopyConvertEngine,
        usize,
    ) -> Result<usize, CapacityError> = std::hint::black_box(
        <CopyConvertEngine as Transcoder>::max_transcode_output_len,
    );
    let max_finish_output_len: fn(
        &CopyConvertEngine,
    ) -> Result<usize, CapacityError> = std::hint::black_box(
        <CopyConvertEngine as Transcoder>::max_finish_output_len,
    );
    let max_reset_output_len: fn(
        &CopyConvertEngine,
    ) -> Result<usize, CapacityError> = std::hint::black_box(
        <CopyConvertEngine as Transcoder>::max_reset_output_len,
    );
    let transcode: TranscodeFn =
        std::hint::black_box(<CopyConvertEngine as Transcoder>::transcode);
    let reset: OutputFn =
        std::hint::black_box(<CopyConvertEngine as Transcoder>::reset);
    let finish: OutputFn =
        std::hint::black_box(<CopyConvertEngine as Transcoder>::finish);

    assert_eq!(Ok(2), max_transcode_output_len(&engine, 2));
    assert_eq!(Ok(0), max_finish_output_len(&engine));
    assert_eq!(Ok(0), max_reset_output_len(&engine));
    let progress = transcode(&mut engine, &[3, 4], 0, &mut output, 0)
        .expect("engine should convert through the trait");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((2, 2), (progress.read(), progress.written()));
    assert_eq!([4, 5], output);

    let mut empty_output = [0_u8; 0];
    let reset = reset(&mut engine, &mut empty_output, 0)
        .expect("engine should reset through the trait");
    let finished = finish(&mut engine, &mut empty_output, 0)
        .expect("engine should finish through the trait");

    assert_eq!(0, reset);
    assert_eq!(0, finished);
}

#[test]
fn test_buffered_convert_engine_default_builds_engine() {
    let mut engine = TranscodeConvertEngine::<
        SourceCodec,
        TargetCodec,
        StrictDecodeHooks,
        StrictEncodeHooks,
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
    );

    assert_eq!(Ok(11), engine.max_transcode_output_len(1));

    let mut output = [0_u8; 1];
    let progress = engine
        .transcode(&[1], 0, &mut output, 0)
        .expect("supplied encode hooks should convert the value");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!((1, 1), (progress.read(), progress.written()));
    assert_eq!([2], output);

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
    assert_eq!(Ok(2), engine.max_transcode_output_len(1));
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

    let progress = engine.transcode(&[9], 0, &mut empty_output, 0).expect(
        "conversion should report pending output before reading new input",
    );
    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 0,
            required: crate::nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!((0, 0), (progress.read(), progress.written()));
    assert_eq!(Ok(2), engine.max_transcode_output_len(1));

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
    let error = engine.transcode(&[1], 0, &mut output, 0).expect_err(
        "pending encode error should be mapped before new input is consumed",
    );

    assert!(matches!(
        error,
        TranscodeConvertError::EncodeDomain(
            qubit_codec::TranscodeDomainError::Main {
                source: EngineError::Encode,
                input_index: _,
                input_consumed: None
            }
        ),
    ));
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_reports_invalid_indices() {
    let mut engine = new_copy_engine();
    let mut output = [0_u8; 1];

    let error = engine
        .transcode(&[1], 2, &mut output, 0)
        .expect_err("invalid input index should fail");
    assert_eq!(TranscodeConvertError::invalid_input_index(2, 1), error,);

    let error = engine
        .transcode(&[1], 0, &mut output, 2)
        .expect_err("invalid output index should fail");
    assert_eq!(TranscodeConvertError::invalid_output_index(2, 1), error,);
}

#[test]
fn test_buffered_convert_engine_reports_capacity_errors() {
    let engine = new_error_path_engine(ErrorPathHooks {
        decode_max_output_error: true,
        ..ErrorPathHooks::default()
    });
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_transcode_output_len(1)
    );
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_total_output_len(1)
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
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_total_output_len(1)
    );

    let engine = new_error_path_engine(ErrorPathHooks {
        encode_finish_len: usize::MAX,
        ..ErrorPathHooks::default()
    });
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_total_output_len(1)
    );

    let mut engine = new_error_path_engine(ErrorPathHooks {
        encode_max_output_error: true,
        ..ErrorPathHooks::default()
    });
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_reset_output_len()
    );
    let error = engine
        .reset(&mut [], 0)
        .expect_err("reset bound overflow should be mapped");
    assert_eq!(TranscodeConvertError::output_length_overflow(), error);

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
        engine.max_transcode_output_len(0)
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
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_total_output_len(0)
    );
}

#[test]
fn test_buffered_convert_engine_finish_maps_encoder_finish_bound_overflow() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        FinishOverflowTargetCodec,
        StrictDecodeHooks,
        FinishOverflowEncodeHooks,
    );
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_finish_output_len()
    );

    let error = engine
        .finish(&mut [], 0)
        .expect_err("encoder finish bound overflow should be mapped");

    assert_eq!(TranscodeConvertError::output_length_overflow(), error);
}

#[test]
fn test_buffered_convert_engine_finish_maps_initial_decode_finish_bound_overflow()
 {
    let mut engine = TranscodeConvertEngine::new(
        FinishValueSourceCodec::<{ usize::MAX }>,
        TargetCodec,
        FixedFinishBoundDecodeHooks { finish_len: 1 },
        StrictEncodeHooks,
    );
    let mut output = [];

    let error = engine
        .finish(&mut output, 0)
        .expect_err("overflowing decode finish bound should be mapped");

    assert_eq!(TranscodeConvertError::output_length_overflow(), error);
}

#[test]
fn test_buffered_convert_engine_finish_maps_late_decode_finish_bound_overflow()
{
    let mut engine = TranscodeConvertEngine::new(
        FinishValueSourceCodec::<1>,
        TargetCodec,
        ChangingFinishBoundDecodeHooks {
            calls: Cell::new(0),
        },
        StrictEncodeHooks,
    );
    let mut output = [0_u8; 1];

    let error = engine
        .finish(&mut output, 0)
        .expect_err("late decode finish bound overflow should be mapped");

    assert_eq!(TranscodeConvertError::output_length_overflow(), error);
}

#[test]
fn test_buffered_convert_engine_maps_encode_value_error() {
    let mut engine = new_error_path_engine(ErrorPathHooks {
        decode_finish: ErrorPathDecodeFinish::Normal,
        encode_mode: ErrorPathEncodeMode::PrepareError,
        ..ErrorPathHooks::default()
    });
    let mut output = [0_u8; 1];

    let error = engine.transcode(&[98], 0, &mut output, 0).expect_err(
        "encode value error should be mapped through convert hooks",
    );

    assert!(matches!(
        error,
        TranscodeConvertError::EncodeDomain(
            qubit_codec::TranscodeDomainError::Main {
                source: EngineError::Encode,
                input_index: _,
                input_consumed: None
            }
        ),
    ));
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_finish_reports_output_index_beyond_buffer() {
    let mut engine = new_copy_engine();
    let mut output = [];

    let error = engine
        .finish(&mut output, 1)
        .expect_err("out-of-range finish output index should be rejected");

    assert_eq!(TranscodeConvertError::invalid_output_index(1, 0), error,);
}

#[test]
fn test_buffered_convert_engine_finish_maps_decode_error() {
    let mut engine = new_error_path_engine(ErrorPathHooks {
        decode_finish: ErrorPathDecodeFinish::Error,
        encode_mode: ErrorPathEncodeMode::Normal,
        ..ErrorPathHooks::default()
    });
    let mut output = [0_u8; 1];

    let error = engine.finish(&mut output, 0).expect_err(
        "decode finish error should be mapped through convert hooks",
    );

    assert!(matches!(
        error,
        TranscodeConvertError::DecodeDomain(
            qubit_codec::TranscodeDomainError::Finish {
                source: EngineError::Decode,
            }
        ),
    ));
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

    let error = engine.finish(&mut output, 0).expect_err(
        "encode finish error should be mapped through convert hooks",
    );

    assert!(matches!(
        error,
        TranscodeConvertError::EncodeDomain(
            qubit_codec::TranscodeDomainError::Finish {
                source: EngineError::Encode,
            }
        ),
    ));
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

    assert!(matches!(
        error,
        TranscodeConvertError::EncodeDomain(
            qubit_codec::TranscodeDomainError::Main {
                source: EngineError::Encode,
                input_index: _,
                input_consumed: None,
            }
        ),
    ));
    assert_eq!([0], output);
}

#[test]
fn test_buffered_convert_engine_finish_maps_decoder_output_encode_error() {
    let mut engine = new_finish_engine(FinishHooks { value: 13 });
    let mut output = [0_u8; 1];

    let error = engine.finish(&mut output, 0).expect_err(
        "finish should map encode errors for decoder-emitted values",
    );

    assert!(matches!(
        error,
        TranscodeConvertError::EncodeDomain(
            qubit_codec::TranscodeDomainError::Main {
                source: EngineError::Encode,
                input_index: _,
                input_consumed: None
            }
        ),
    ));
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
    assert_eq!(TranscodeConvertError::insufficient_output(0, 1, 0), error,);
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
    let error = engine.finish(&mut empty_output, 0).expect_err(
        "finish should reject insufficient output before decoder finish",
    );
    assert_eq!(TranscodeConvertError::insufficient_output(0, 1, 0), error,);
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
fn test_buffered_convert_engine_finish_drains_pending_before_decoder_finish_output()
 {
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
    assert_eq!(TranscodeConvertError::insufficient_output(0, 2, 1), error,);
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
    let mut engine = new_finish_encode_engine();
    assert_eq!(Ok(1), engine.max_finish_output_len());

    let mut empty_output = [0_u8; 0];
    let error = engine.finish(&mut empty_output, 0).expect_err(
        "target finish hook should require one-shot output capacity",
    );
    assert_eq!(TranscodeConvertError::insufficient_output(0, 1, 0), error,);

    let mut output = [0_u8; 1];
    let written = engine
        .finish(&mut output, 0)
        .expect("target finish hook should write final output");
    assert_eq!(1, written);
    assert_eq!([0xee], output);
    assert_eq!(Ok(0), engine.max_finish_output_len());
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ResetObservingDecodeHooks {
    called: std::rc::Rc<Cell<bool>>,
}

impl TranscodeDecodeHooks<SourceCodec> for ResetObservingDecodeHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut SourceCodec,
        error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<
        DecodeInvalidAction<u8>,
        qubit_codec::TranscodeDecodeErrorOf<SourceCodec>,
    > {
        match error {
            EngineError::Decode | EngineError::Encode => {
                unreachable!("SourceCodec should not produce decode errors")
            }
        }
    }

    fn reset_hooks(&mut self, _codec: &mut SourceCodec) {
        self.called.set(true);
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct StatelessResetSourceCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct StatelessResetFailingSourceCodec;

impl Codec for StatelessResetSourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_RESET_VALUES: usize = 0;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
    {
        let value = unsafe { *input.get_unchecked(input_index) };
        Ok((value, NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        unsafe {
            *output.get_unchecked_mut(output_index) = *value;
        }
        Ok(1)
    }

    unsafe fn decode_reset(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Ok(0)
    }
}

impl Codec for StatelessResetFailingSourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = EngineError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_RESET_VALUES: usize = 0;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
    {
        let value = unsafe { *input.get_unchecked(input_index) };
        Ok((value, NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        unsafe {
            *output.get_unchecked_mut(output_index) = *value;
        }
        Ok(1)
    }

    unsafe fn decode_reset(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Err(EngineError::Decode)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StatelessResetSourceDecodeHooks {
    called: Rc<Cell<bool>>,
}

impl TranscodeDecodeHooks<StatelessResetSourceCodec>
    for StatelessResetSourceDecodeHooks
{
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut StatelessResetSourceCodec,
        error: &core::convert::Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<
        DecodeInvalidAction<u8>,
        qubit_codec::TranscodeDecodeErrorOf<StatelessResetSourceCodec>,
    > {
        match *error {}
    }

    fn reset_hooks(&mut self, _codec: &mut StatelessResetSourceCodec) {
        self.called.set(true);
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct StatelessResetFailingSourceDecodeHooks;

impl TranscodeDecodeHooks<StatelessResetFailingSourceCodec>
    for StatelessResetFailingSourceDecodeHooks
{
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut StatelessResetFailingSourceCodec,
        error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<
        DecodeInvalidAction<u8>,
        qubit_codec::TranscodeDecodeErrorOf<StatelessResetFailingSourceCodec>,
    > {
        match error {
            EngineError::Decode | EngineError::Encode => {
                unreachable!("reset path should not produce decode errors")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverflowResetSourceCodec;

impl Codec for OverflowResetSourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_RESET_VALUES: usize = usize::MAX;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
    {
        let value = unsafe { *input.get_unchecked(input_index) };
        Ok((value, NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        unsafe {
            *output.get_unchecked_mut(output_index) = *value;
        }
        Ok(1)
    }

    unsafe fn decode_reset(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Ok(0)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverflowResetTargetCodec;

impl Codec for OverflowResetTargetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_RESET_UNITS: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
    {
        let value = unsafe { *input.get_unchecked(input_index) };
        Ok((value, NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        unsafe {
            *output.get_unchecked_mut(output_index) = *value;
        }
        Ok(1)
    }

    unsafe fn encode_reset(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverflowResetSourceDecodeHooks;

impl TranscodeDecodeHooks<OverflowResetSourceCodec>
    for OverflowResetSourceDecodeHooks
{
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut OverflowResetSourceCodec,
        error: &core::convert::Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<
        DecodeInvalidAction<u8>,
        qubit_codec::TranscodeDecodeErrorOf<OverflowResetSourceCodec>,
    > {
        match *error {}
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct OverflowResetTargetEncodeHooks;

impl TranscodeEncodeHooks<OverflowResetTargetCodec>
    for OverflowResetTargetEncodeHooks
{
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut OverflowResetTargetCodec,
        _context: &EncodeContext<'_, u8, u8>,
    ) -> Result<
        EncodeUnencodableAction<u8>,
        qubit_codec::TranscodeEncodeErrorOf<OverflowResetTargetCodec>,
    > {
        unreachable!("overflow reset target codec accepts all u8 values")
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct MismatchCapacityResetEmittingDecodeHooks;

impl TranscodeDecodeHooks<ResetEmittingSourceCodec>
    for MismatchCapacityResetEmittingDecodeHooks
{
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut ResetEmittingSourceCodec,
        error: &core::convert::Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<
        DecodeInvalidAction<u8>,
        qubit_codec::TranscodeDecodeErrorOf<ResetEmittingSourceCodec>,
    > {
        match *error {}
    }
}

#[test]
fn test_buffered_convert_engine_reset_emits_target_reset_output() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        ResetEmittingTargetCodec,
        StrictDecodeHooks,
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
fn test_buffered_convert_engine_reset_calls_decode_before_reset() {
    let called = std::rc::Rc::new(Cell::new(false));
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        ResetObservingDecodeHooks {
            called: called.clone(),
        },
        StrictEncodeHooks,
    );

    engine.reset(&mut [], 0).expect("reset should succeed");

    assert!(called.get(), "decode before_reset should be called");
}

#[test]
fn test_buffered_convert_engine_reset_maps_target_reset_errors() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        ResetFailTargetCodec,
        StrictDecodeHooks,
        ResetFailTargetHooks,
    );
    let mut output = [0_u8; 1];

    let error = engine.reset(&mut output, 0).expect_err(
        "target reset errors should be mapped through convert hooks",
    );

    assert!(matches!(
        error,
        TranscodeConvertError::EncodeDomain(
            qubit_codec::TranscodeDomainError::Reset {
                source: TargetResetFailError,
                ..
            }
        ),
    ));
}

#[test]
fn test_buffered_convert_engine_reset_rejects_invalid_output_index() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        ResetEmittingTargetCodec,
        StrictDecodeHooks,
        ResetTargetHooks,
    );
    let mut output = [0_u8; 1];

    let error = engine
        .reset(&mut output, 2)
        .expect_err("invalid reset output index should be rejected");

    assert_eq!(TranscodeConvertError::invalid_output_index(2, 1), error,);
}

#[test]
fn test_buffered_convert_engine_reset_with_stateless_decode_reset_values() {
    let called = Rc::new(Cell::new(false));
    let mut engine = TranscodeConvertEngine::new(
        StatelessResetSourceCodec,
        TargetCodec,
        StatelessResetSourceDecodeHooks {
            called: called.clone(),
        },
        StrictEncodeHooks,
    );

    assert_eq!(Ok(0), engine.max_reset_output_len());

    let written = engine
        .reset(&mut [], 0)
        .expect("stateless decode reset should clear state without output");

    assert_eq!(0, written);
    assert!(
        called.get(),
        "stateless decode reset should still invoke decode-side reset hooks",
    );
}

#[test]
fn test_buffered_convert_engine_reset_maps_stateless_decoder_reset_error() {
    let mut engine = TranscodeConvertEngine::new(
        StatelessResetFailingSourceCodec,
        TargetCodec,
        StatelessResetFailingSourceDecodeHooks,
        StrictEncodeHooks,
    );

    let error = engine
        .reset(&mut [], 0)
        .expect_err("stateless decoder reset errors should be mapped");

    assert!(matches!(
        error,
        TranscodeConvertError::DecodeDomain(
            qubit_codec::TranscodeDomainError::Reset {
                source: EngineError::Decode
            }
        ),
    ));
}

#[test]
fn test_buffered_convert_engine_max_reset_output_len_overflow() {
    let engine = TranscodeConvertEngine::new(
        OverflowResetSourceCodec,
        OverflowResetTargetCodec,
        OverflowResetSourceDecodeHooks,
        OverflowResetTargetEncodeHooks,
    );

    let error = engine
        .max_reset_output_len()
        .expect_err("max reset bound should report overflow");

    assert_eq!(CapacityError::OutputLengthOverflow, error);
    assert_eq!(
        Err(CapacityError::OutputLengthOverflow),
        engine.max_total_output_len(0)
    );
}

#[test]
#[should_panic(
    expected = "converter reset bound must reserve space for decode reset values"
)]
fn test_buffered_convert_engine_reset_reaches_unreachable_decode_reset_path() {
    let mut engine = TranscodeConvertEngine::new(
        ResetEmittingSourceCodec,
        MismatchCapacityTargetCodec,
        MismatchCapacityResetEmittingDecodeHooks,
        MismatchCapacityEncodeHooks,
    );
    let mut output = [0_u8; 1];

    let _ = engine.reset(&mut output, 0);
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

    assert_eq!(TranscodeConvertError::invalid_output_index(2, 1), error,);
    assert_eq!(Ok(1), engine.max_finish_output_len());

    let written = engine
        .finish(&mut output, 0)
        .expect("invalid reset must not discard pending value");
    assert_eq!(1, written);
    assert_eq!([5], output);
}

#[test]
#[should_panic(
    expected = "converter finish bound must reserve space for pending values"
)]
fn test_buffered_convert_engine_finish_hits_pending_unreachable() {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        MismatchCapacityTargetCodec,
        StrictDecodeHooks,
        MismatchCapacityEncodeHooks,
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
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        MismatchCapacityTargetCodec,
        FinishDecodeHooks::default(),
        MismatchCapacityEncodeHooks,
    );
    let required = engine.max_finish_output_len().expect("finish bound");
    let mut output = vec![0_u8; required];
    let _ = engine.finish(&mut output, 0);
}

#[test]
fn test_buffered_convert_engine_maps_decode_policy_domain_error() {
    let mut engine = TranscodeConvertEngine::new(
        ErrorSourceCodec,
        TargetCodec,
        ImpossibleFailureDecodeHooks,
        StrictEncodeHooks,
    );
    let mut output = [0_u8; 1];

    let error = engine
        .transcode(&[1], 0, &mut output, 0)
        .expect_err("decode policy domain errors should map to decode side");

    assert_eq!(
        TranscodeConvertError::decode_domain_main(EngineError::Decode, 0),
        error
    );
}

// Decode-reset emit fixtures for regression testing
// `TranscodeConvertEngine::reset` against codecs whose
// `MAX_DECODE_RESET_VALUES > 0`. The previous implementation hard-asserted that
// decode-side resets are absent, which silently dropped any stream-start values
// the source decoder wanted to emit (such as a BOM) before they reached the
// target encoder.

/// Decode value sentinel written by [`ResetEmittingSourceCodec::decode_reset`].
const SOURCE_RESET_SENTINEL: u8 = 0xbb;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetEmittingSourceCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetEncodingFailSourceCodec;

impl Codec for ResetEmittingSourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_RESET_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
    {
        // SAFETY: The caller proved that at least one input unit is readable.
        let value = unsafe { *input.get_unchecked(input_index) };
        Ok((value, NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        // SAFETY: The caller proved that one output unit is writable.
        unsafe {
            *output.get_unchecked_mut(output_index) = *value;
        }
        Ok(1)
    }

    unsafe fn decode_reset(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        // SAFETY: The caller guarantees room for one reset value at
        // `output_index`.
        unsafe {
            *output.get_unchecked_mut(output_index) = SOURCE_RESET_SENTINEL;
        }
        Ok(1)
    }
}

impl Codec for ResetEncodingFailSourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_RESET_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
    {
        // SAFETY: The caller proved that at least one input unit is readable.
        let value = unsafe { *input.get_unchecked(input_index) };
        Ok((value, NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        // SAFETY: The caller proved that one output unit is writable.
        unsafe {
            *output.get_unchecked_mut(output_index) = *value;
        }
        Ok(1)
    }

    unsafe fn decode_reset(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        // SAFETY: The caller guarantees room for one reset value at
        // `output_index`.
        unsafe {
            *output.get_unchecked_mut(output_index) = 13;
        }
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetSourceDecodeHooks;

impl TranscodeDecodeHooks<ResetEmittingSourceCodec> for ResetSourceDecodeHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut ResetEmittingSourceCodec,
        error: &core::convert::Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<
        DecodeInvalidAction<u8>,
        qubit_codec::TranscodeDecodeErrorOf<ResetEmittingSourceCodec>,
    > {
        match *error {}
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetEncodingFailSourceDecodeHooks;

impl TranscodeDecodeHooks<ResetEncodingFailSourceCodec>
    for ResetEncodingFailSourceDecodeHooks
{
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut ResetEncodingFailSourceCodec,
        error: &core::convert::Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<
        DecodeInvalidAction<u8>,
        qubit_codec::TranscodeDecodeErrorOf<ResetEncodingFailSourceCodec>,
    > {
        match *error {}
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetFailingSourceCodec;

impl Codec for ResetFailingSourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = EngineError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_RESET_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
    {
        let value = unsafe { *input.get_unchecked(input_index) };
        Ok((value, NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        // SAFETY: The caller proved that one output unit is writable.
        unsafe {
            *output.get_unchecked_mut(output_index) = *value;
        }
        Ok(1)
    }

    unsafe fn decode_reset(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Err(EngineError::Decode)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetFailSourceDecodeHooks;

impl TranscodeDecodeHooks<ResetFailingSourceCodec>
    for ResetFailSourceDecodeHooks
{
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut ResetFailingSourceCodec,
        error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<
        DecodeInvalidAction<u8>,
        qubit_codec::TranscodeDecodeErrorOf<ResetFailingSourceCodec>,
    > {
        match error {
            EngineError::Decode | EngineError::Encode => {
                unreachable!("reset path should not produce decode errors")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FinishFailingSourceCodec;

impl Codec for FinishFailingSourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = EngineError;
    type EncodeError = core::convert::Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_FINISH_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), qubit_codec::DecodeFailure<Self::DecodeError>>
    {
        let value = unsafe { *input.get_unchecked(input_index) };
        Ok((value, NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, Self::EncodeError> {
        // SAFETY: The caller proved that one output unit is writable.
        unsafe {
            *output.get_unchecked_mut(output_index) = *value;
        }
        Ok(1)
    }

    unsafe fn decode_finish(
        &mut self,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, Self::DecodeError> {
        Err(EngineError::Decode)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct FinishFailSourceDecodeHooks;

impl TranscodeDecodeHooks<FinishFailingSourceCodec>
    for FinishFailSourceDecodeHooks
{
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut FinishFailingSourceCodec,
        error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<
        DecodeInvalidAction<u8>,
        qubit_codec::TranscodeDecodeErrorOf<FinishFailingSourceCodec>,
    > {
        match error {
            EngineError::Decode | EngineError::Encode => {
                unreachable!("finish path should not produce decode errors")
            }
        }
    }
}

#[test]
fn test_buffered_convert_engine_reset_includes_decode_reset_in_bound() {
    let engine = TranscodeConvertEngine::new(
        ResetEmittingSourceCodec,
        TargetCodec,
        ResetSourceDecodeHooks,
        StrictEncodeHooks,
    );

    let bound = engine
        .max_reset_output_len()
        .expect("reset bound should be representable");
    assert!(
        bound >= 1,
        "decode-reset value must be reserved in reset bound, got {bound}",
    );
}

#[test]
fn test_buffered_convert_engine_reset_pipes_decode_reset_values_into_encoder() {
    let mut engine = TranscodeConvertEngine::new(
        ResetEmittingSourceCodec,
        TargetCodec,
        ResetSourceDecodeHooks,
        StrictEncodeHooks,
    );

    let required = engine
        .max_reset_output_len()
        .expect("reset bound should be representable");
    let mut output = vec![0_u8; required];

    let written = engine
        .reset(&mut output, 0)
        .expect("reset should succeed when the source emits a reset value");

    assert!(
        written >= 1,
        "reset must report the encoded decode-reset value, got {written}",
    );
    assert_eq!(
        SOURCE_RESET_SENTINEL, output[0],
        "decode-reset value must be encoded into the target output",
    );
}

#[test]
fn test_buffered_convert_engine_reset_maps_decode_reset_value_encode_error() {
    let mut engine = TranscodeConvertEngine::new(
        ResetEncodingFailSourceCodec,
        TargetCodec,
        ResetEncodingFailSourceDecodeHooks,
        StrictEncodeHooks,
    );
    let mut output = [0_u8; 1];

    let error = engine.reset(&mut output, 0).expect_err(
        "errors while encoding decode-reset values should be mapped",
    );

    assert!(matches!(
        error,
        TranscodeConvertError::EncodeDomain(
            qubit_codec::TranscodeDomainError::Main {
                source: EngineError::Encode,
                input_index: _,
                input_consumed: None,
            }
        ),
    ));
}

#[test]
fn test_buffered_convert_engine_reset_maps_decoder_reset_error() {
    let mut engine = TranscodeConvertEngine::new(
        ResetFailingSourceCodec,
        TargetCodec,
        ResetFailSourceDecodeHooks,
        StrictEncodeHooks,
    );
    let mut output = [0_u8; 1];

    let error = engine.reset(&mut output, 0).expect_err(
        "decoder reset errors should be mapped through convert hooks",
    );

    assert!(matches!(
        error,
        TranscodeConvertError::DecodeDomain(
            qubit_codec::TranscodeDomainError::Reset {
                source: EngineError::Decode,
            }
        ),
    ));
}

#[test]
fn test_buffered_convert_engine_finish_maps_decoder_finish_error() {
    let mut engine = TranscodeConvertEngine::new(
        FinishFailingSourceCodec,
        TargetCodec,
        FinishFailSourceDecodeHooks,
        StrictEncodeHooks,
    );
    let mut output = [0_u8; 1];

    let error = engine.finish(&mut output, 0).expect_err(
        "decoder finish error should be mapped through convert hooks",
    );

    assert!(matches!(
        error,
        TranscodeConvertError::DecodeDomain(
            qubit_codec::TranscodeDomainError::Finish {
                source: EngineError::Decode,
            }
        ),
    ));
}

// ============================================================================
// Lifecycle guard wiring
// ============================================================================

#[cfg(debug_assertions)]
#[test]
#[should_panic(
    expected = "Transcoder::finish called twice without an intervening reset"
)]
fn test_buffered_convert_engine_lifecycle_rejects_double_finish() {
    let mut engine = new_copy_engine();
    let mut output = [0_u8; 0];
    engine
        .finish(&mut output, 0)
        .expect("first finish should succeed for a stateless converter");
    let _ = engine.finish(&mut output, 0);
}

#[cfg(debug_assertions)]
#[test]
#[should_panic(
    expected = "Transcoder::transcode called after finish without an \
                intervening reset"
)]
fn test_buffered_convert_engine_lifecycle_rejects_transcode_after_finish() {
    let mut engine = new_copy_engine();
    let mut output = [0_u8; 1];
    engine
        .finish(&mut output, 0)
        .expect("finish closes the logical stream");
    let _ = engine.transcode(&[1_u8], 0, &mut output, 0);
}

#[test]
fn test_buffered_convert_engine_lifecycle_allows_reuse_after_reset() {
    let mut engine = new_copy_engine();
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

#[test]
fn test_buffered_convert_engine_reports_complete_stream_bound() {
    let engine = new_copy_engine();

    assert_eq!(Ok(2), engine.max_total_output_len(2));
}

// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared codec, hook, and engine fixtures for conversion-engine tests.

use core::cell::Cell;
use core::convert::Infallible;
use core::num::NonZeroUsize;
use std::rc::Rc;

use qubit_codec as codec;
use qubit_codec::CapacityError;
use qubit_codec::Codec;
use qubit_codec::TranscodeDecodeError;
use qubit_codec::TranscodeEncodeError;
use qubit_codec::engine::DecodeContext;
use qubit_codec::engine::DecodeIncompleteAction;
use qubit_codec::engine::DecodeInvalidAction;
use qubit_codec::engine::EncodeContext;
use qubit_codec::engine::EncodeUnencodableAction;
use qubit_codec::engine::TranscodeConvertEngine;
use qubit_codec::engine::TranscodeDecodeHooks;
use qubit_codec::engine::TranscodeEncodeHooks;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct SourceCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct ShortSourceCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct TargetCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct ErrorSourceCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::transcode::engine) struct FinishValueSourceCodec<const FINISH_BOUND: usize>;

#[must_use]
#[inline]
pub(in crate::transcode::engine) fn one_consumed() -> NonZeroUsize {
    NonZeroUsize::MIN
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub(in crate::transcode::engine) enum EngineError {
    #[error("decode error")]
    Decode,
    #[error("encode error")]
    Encode,
}

impl From<Infallible> for EngineError {
    fn from(error: Infallible) -> Self {
        match error {}
    }
}

impl Codec for SourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = EngineError;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index <= input.len());
        debug_assert!(input.len() - input_index >= Self::MIN_UNITS_PER_VALUE);
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

impl Codec for ShortSourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = EngineError;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index <= input.len());
        debug_assert!(input.len() - input_index >= Self::MIN_UNITS_PER_VALUE);
        Ok((
            input[input_index].wrapping_add(input[input_index + 1]),
            crate::nonzero(2),
        ))
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

impl Codec for TargetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = EngineError;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    fn can_encode_value(&self, value: &u8) -> bool {
        *value != 99
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index <= input.len());
        debug_assert!(input.len() - input_index >= Self::MIN_UNITS_PER_VALUE);
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
pub(in crate::transcode::engine) struct ResetEmittingTargetCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct ResetDependentTargetCodec {
    reset: bool,
}

impl Codec for ResetEmittingTargetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_RESET_UNITS: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index <= input.len());
        debug_assert!(input.len() - input_index >= Self::MIN_UNITS_PER_VALUE);
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

    unsafe fn encode_reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::EncodeError> {
        output[output_index] = 0xaa;
        Ok(1)
    }
}

impl Codec for ResetDependentTargetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = EngineError;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_RESET_UNITS: usize = 1;

    fn can_encode_value(&self, _value: &u8) -> bool {
        self.reset
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index <= input.len());
        debug_assert!(input.len() - input_index >= Self::MIN_UNITS_PER_VALUE);
        // SAFETY: The caller proved that one input unit is readable.
        unsafe { Ok((*input.get_unchecked(input_index), NonZeroUsize::MIN)) }
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

    unsafe fn encode_reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::EncodeError> {
        self.reset = true;
        // SAFETY: The caller proved that one output unit is writable.
        unsafe {
            *output.get_unchecked_mut(output_index) = 0xaa;
        }
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct ResetFailTargetCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct FinishOverflowTargetCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct MismatchCapacityTargetCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
#[error("target reset failed")]
pub(in crate::transcode::engine) struct TargetResetFailError;

impl Codec for ResetFailTargetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = TargetResetFailError;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_RESET_UNITS: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index <= input.len());
        debug_assert!(input.len() - input_index >= Self::MIN_UNITS_PER_VALUE);
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

    unsafe fn encode_reset(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::EncodeError> {
        Err(TargetResetFailError)
    }
}

impl Codec for FinishOverflowTargetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_FINISH_UNITS: usize = usize::MAX;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index <= input.len());
        debug_assert!(input.len() - input_index >= Self::MIN_UNITS_PER_VALUE);
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
    type DecodeError = Infallible;
    type EncodeError = EngineError;

    const MIN_UNITS_PER_VALUE: usize = 3;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 3;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 3;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index <= input.len());
        debug_assert!(input.len() - input_index >= Self::MIN_UNITS_PER_VALUE);
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
        output[output_index + 2] = value.wrapping_add(2);
        Ok(3)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct ResetTargetHooks;

impl TranscodeEncodeHooks<ResetEmittingTargetCodec> for ResetTargetHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut ResetEmittingTargetCodec,
        _context: &EncodeContext<'_, u8>,
    ) -> Result<EncodeUnencodableAction<u8>, codec::TranscodeEncodeErrorOf<ResetEmittingTargetCodec>> {
        Ok(EncodeUnencodableAction::Reject)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct FinishOverflowEncodeHooks;

impl TranscodeEncodeHooks<FinishOverflowTargetCodec> for FinishOverflowEncodeHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut FinishOverflowTargetCodec,
        _context: &EncodeContext<'_, u8>,
    ) -> Result<EncodeUnencodableAction<u8>, codec::TranscodeEncodeErrorOf<FinishOverflowTargetCodec>> {
        Ok(EncodeUnencodableAction::Reject)
    }

    fn max_finish_output_len(&self, _codec: &FinishOverflowTargetCodec) -> usize {
        1
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct ResetFailTargetHooks;

impl TranscodeEncodeHooks<ResetFailTargetCodec> for ResetFailTargetHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut ResetFailTargetCodec,
        _context: &EncodeContext<'_, u8>,
    ) -> Result<EncodeUnencodableAction<u8>, codec::TranscodeEncodeErrorOf<ResetFailTargetCodec>> {
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
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index <= input.len());
        debug_assert!(input.len() - input_index >= Self::MIN_UNITS_PER_VALUE);
        Err(codec::DecodeFailure::invalid_unknown(EngineError::Decode))
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
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;

    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_FINISH_VALUES: usize = FINISH_BOUND;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index <= input.len());
        debug_assert!(input.len() - input_index >= Self::MIN_UNITS_PER_VALUE);
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
pub(in crate::transcode::engine) struct StrictDecodeHooks;

impl TranscodeDecodeHooks<SourceCodec> for StrictDecodeHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut SourceCodec,
        error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<SourceCodec>> {
        match error {
            EngineError::Decode | EngineError::Encode => {
                unreachable!("SourceCodec should not produce decode errors")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::transcode::engine) enum EofIncompleteMode {
    Emit,
    Skip,
    Reject,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::transcode::engine) struct EofIncompleteDecodeHooks {
    pub(in crate::transcode::engine) mode: EofIncompleteMode,
}

impl TranscodeDecodeHooks<ShortSourceCodec> for EofIncompleteDecodeHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut ShortSourceCodec,
        _error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<ShortSourceCodec>> {
        Ok(DecodeInvalidAction::Reject)
    }

    fn handle_incomplete_decode(
        &mut self,
        _codec: &mut ShortSourceCodec,
        source: Option<&EngineError>,
        required_total: NonZeroUsize,
        context: DecodeContext,
    ) -> Result<DecodeIncompleteAction<u8>, codec::TranscodeDecodeErrorOf<ShortSourceCodec>> {
        assert_eq!(None, source);
        assert_eq!(crate::nonzero(2), required_total);
        assert_eq!(1, context.available());
        match self.mode {
            EofIncompleteMode::Emit => Ok(DecodeIncompleteAction::Emit { value: 77 }),
            EofIncompleteMode::Skip => Ok(DecodeIncompleteAction::Skip),
            EofIncompleteMode::Reject => Ok(DecodeIncompleteAction::Reject),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct ImpossibleFailureDecodeHooks;

impl TranscodeDecodeHooks<ErrorSourceCodec> for ImpossibleFailureDecodeHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut ErrorSourceCodec,
        _error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<ErrorSourceCodec>> {
        Err(codec::TranscodeDecodeError::domain_main(EngineError::Decode, 0))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct StrictEncodeHooks;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct CountedTargetReplacementHooks {
    pub(in crate::transcode::engine) calls: usize,
}

impl TranscodeEncodeHooks<TargetCodec> for CountedTargetReplacementHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut TargetCodec,
        _context: &EncodeContext<'_, u8>,
    ) -> Result<EncodeUnencodableAction<u8>, codec::TranscodeEncodeErrorOf<TargetCodec>> {
        self.calls += 1;
        Ok(EncodeUnencodableAction::Replace {
            value: 40 + self.calls as u8,
        })
    }
}

impl<C> TranscodeEncodeHooks<C> for StrictEncodeHooks
where
    C: Codec<Value = u8, Unit = u8, EncodeError = EngineError>,
{
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut C,
        _context: &EncodeContext<'_, u8>,
    ) -> Result<EncodeUnencodableAction<u8>, codec::TranscodeEncodeErrorOf<C>> {
        Err(TranscodeEncodeError::domain_main(
            EngineError::Encode,
            _context.input_index(),
        ))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct PerBatchOverheadEncodeHooks;

impl TranscodeEncodeHooks<TargetCodec> for PerBatchOverheadEncodeHooks {
    fn max_transcode_output_len(&self, _codec: &TargetCodec, input_len: usize) -> Result<usize, CapacityError> {
        if input_len == 0 {
            Ok(0)
        } else {
            input_len.checked_add(1).ok_or(CapacityError::OutputLengthOverflow)
        }
    }

    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut TargetCodec,
        context: &EncodeContext<'_, u8>,
    ) -> Result<EncodeUnencodableAction<u8>, codec::TranscodeEncodeErrorOf<TargetCodec>> {
        Err(TranscodeEncodeError::domain_main(
            EngineError::Encode,
            context.input_index(),
        ))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct MismatchCapacityEncodeHooks;

impl TranscodeEncodeHooks<MismatchCapacityTargetCodec> for MismatchCapacityEncodeHooks {
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
        _context: &EncodeContext<'_, u8>,
    ) -> Result<EncodeUnencodableAction<u8>, codec::TranscodeEncodeErrorOf<MismatchCapacityTargetCodec>> {
        Err(TranscodeEncodeError::domain_main(
            EngineError::Encode,
            _context.input_index(),
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::transcode::engine) enum RepairAction {
    Emit,
    Skip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::transcode::engine) struct RepairDecodeHooks {
    pub(super) action: RepairAction,
}

impl TranscodeDecodeHooks<ErrorSourceCodec> for RepairDecodeHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut ErrorSourceCodec,
        _error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<ErrorSourceCodec>> {
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
pub(in crate::transcode::engine) struct FixedFinishBoundDecodeHooks {
    pub(in crate::transcode::engine) finish_len: usize,
}

impl<const FINISH_BOUND: usize> TranscodeDecodeHooks<FinishValueSourceCodec<FINISH_BOUND>>
    for FixedFinishBoundDecodeHooks
{
    fn max_finish_output_len(&self, _codec: &FinishValueSourceCodec<FINISH_BOUND>) -> usize {
        self.finish_len
    }

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut FinishValueSourceCodec<FINISH_BOUND>,
        error: &Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<FinishValueSourceCodec<FINISH_BOUND>>> {
        match *error {}
    }
}

#[derive(Debug)]
pub(in crate::transcode::engine) struct ChangingFinishBoundDecodeHooks {
    pub(in crate::transcode::engine) calls: Cell<usize>,
}

impl<const FINISH_BOUND: usize> TranscodeDecodeHooks<FinishValueSourceCodec<FINISH_BOUND>>
    for ChangingFinishBoundDecodeHooks
{
    fn max_finish_output_len(&self, _codec: &FinishValueSourceCodec<FINISH_BOUND>) -> usize {
        let calls = self.calls.get();
        self.calls.set(calls + 1);
        if calls == 0 { 0 } else { usize::MAX }
    }

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut FinishValueSourceCodec<FINISH_BOUND>,
        error: &Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<FinishValueSourceCodec<FINISH_BOUND>>> {
        match *error {}
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::transcode::engine) struct FinishDecodeHooks {
    pub(in crate::transcode::engine) value: Option<u8>,
    pub(in crate::transcode::engine) finish_bound: usize,
    pub(in crate::transcode::engine) fail: bool,
}

impl Default for FinishDecodeHooks {
    fn default() -> Self {
        Self {
            value: Some(40),
            finish_bound: 1,
            fail: false,
        }
    }
}

impl TranscodeDecodeHooks<SourceCodec> for FinishDecodeHooks {
    fn max_finish_output_len(&self, _codec: &SourceCodec) -> usize {
        self.finish_bound
    }

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut SourceCodec,
        error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<SourceCodec>> {
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
    ) -> Result<usize, codec::TranscodeDecodeErrorOf<SourceCodec>> {
        if self.fail {
            return Err(TranscodeDecodeError::domain_finish(EngineError::Decode));
        }
        let Some(value) = self.value else {
            return Ok(0);
        };
        output[output_index] = value;
        self.value = None;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::transcode::engine) struct FinishHooks {
    pub(in crate::transcode::engine) value: u8,
}

impl Default for FinishHooks {
    fn default() -> Self {
        Self { value: 40 }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::transcode::engine) struct BatchFinishDecodeHooks {
    next: u8,
    remaining: u8,
}

impl Default for BatchFinishDecodeHooks {
    fn default() -> Self {
        Self { next: 50, remaining: 2 }
    }
}

impl TranscodeDecodeHooks<SourceCodec> for BatchFinishDecodeHooks {
    fn max_finish_output_len(&self, _codec: &SourceCodec) -> usize {
        2
    }

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut SourceCodec,
        error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<SourceCodec>> {
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
    ) -> Result<usize, codec::TranscodeDecodeErrorOf<SourceCodec>> {
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
pub(in crate::transcode::engine) struct FinishEncodeHooks {
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
        context: &EncodeContext<'_, u8>,
    ) -> Result<EncodeUnencodableAction<u8>, codec::TranscodeEncodeErrorOf<TargetCodec>> {
        Err(TranscodeEncodeError::domain_main(
            EngineError::Encode,
            context.input_index(),
        ))
    }

    fn max_finish_output_len(&self, _codec: &TargetCodec) -> usize {
        1
    }

    fn finish_hooks(
        &mut self,
        _codec: &mut TargetCodec,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, codec::TranscodeEncodeErrorOf<TargetCodec>> {
        if !self.pending {
            return Ok(0);
        }
        output[output_index] = 0xee;
        self.pending = false;
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct ErrorPathDecodeHooks {
    finish: ErrorPathDecodeFinish,
    finish_len: usize,
    max_output_error: bool,
}

impl TranscodeDecodeHooks<SourceCodec> for ErrorPathDecodeHooks {
    fn max_transcode_output_len(&self, _codec: &SourceCodec, input_len: usize) -> Result<usize, CapacityError> {
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
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<SourceCodec>> {
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
    ) -> Result<usize, codec::TranscodeDecodeErrorOf<SourceCodec>> {
        match self.finish {
            ErrorPathDecodeFinish::Normal => Ok(0),
            ErrorPathDecodeFinish::Error => Err(TranscodeDecodeError::domain_finish(EngineError::Decode)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub(in crate::transcode::engine) enum ErrorPathDecodeFinish {
    #[default]
    Normal,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub(in crate::transcode::engine) enum ErrorPathEncodeMode {
    #[default]
    Normal,
    PrepareError,
    FinishError,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct ErrorPathEncodeHooks {
    finish_len: usize,
    max_output_error: bool,
    max_output_error_for_len: Option<usize>,
    mode: ErrorPathEncodeMode,
}

impl TranscodeEncodeHooks<TargetCodec> for ErrorPathEncodeHooks {
    fn max_transcode_output_len(&self, _codec: &TargetCodec, input_len: usize) -> Result<usize, CapacityError> {
        if self.max_output_error || self.max_output_error_for_len == Some(input_len) {
            Err(CapacityError::OutputLengthOverflow)
        } else {
            input_len
                .checked_mul(<TargetCodec as Codec>::MAX_ENCODE_UNITS_PER_VALUE)
                .ok_or(CapacityError::OutputLengthOverflow)
        }
    }

    fn max_finish_output_len(&self, _codec: &TargetCodec) -> usize {
        self.finish_len
    }

    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut TargetCodec,
        _context: &EncodeContext<'_, u8>,
    ) -> Result<EncodeUnencodableAction<u8>, codec::TranscodeEncodeErrorOf<TargetCodec>> {
        match self.mode {
            ErrorPathEncodeMode::PrepareError => Err(TranscodeEncodeError::domain_main(
                EngineError::Encode,
                _context.input_index(),
            )),
            ErrorPathEncodeMode::Normal | ErrorPathEncodeMode::FinishError => Ok(EncodeUnencodableAction::Skip),
        }
    }

    fn finish_hooks(
        &mut self,
        _codec: &mut TargetCodec,
        _output: &mut [u8],
        _output_index: usize,
    ) -> Result<usize, codec::TranscodeEncodeErrorOf<TargetCodec>> {
        match self.mode {
            ErrorPathEncodeMode::FinishError => Err(TranscodeEncodeError::domain_finish(EngineError::Encode)),
            ErrorPathEncodeMode::Normal | ErrorPathEncodeMode::PrepareError => Ok(0),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct ErrorPathHooks {
    pub(in crate::transcode::engine) decode_finish: ErrorPathDecodeFinish,
    pub(in crate::transcode::engine) decode_finish_len: usize,
    pub(in crate::transcode::engine) decode_max_output_error: bool,
    pub(in crate::transcode::engine) encode_finish_len: usize,
    pub(in crate::transcode::engine) encode_max_output_error: bool,
    pub(in crate::transcode::engine) encode_max_output_error_for_len: Option<usize>,
    pub(in crate::transcode::engine) encode_mode: ErrorPathEncodeMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::transcode::engine) struct FactoryDecodeHooks {
    pub(in crate::transcode::engine) marker: u8,
}

impl TranscodeDecodeHooks<SourceCodec> for FactoryDecodeHooks {
    fn max_transcode_output_len(&self, _codec: &SourceCodec, _input_len: usize) -> Result<usize, CapacityError> {
        Ok(self.marker as usize)
    }

    fn handle_invalid_decode(
        &mut self,
        _codec: &mut SourceCodec,
        error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<SourceCodec>> {
        match error {
            EngineError::Decode | EngineError::Encode => {
                unreachable!("SourceCodec should not produce decode errors")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::transcode::engine) struct FactoryEncodeHooks {
    pub(in crate::transcode::engine) offset: u8,
}

impl TranscodeEncodeHooks<TargetCodec> for FactoryEncodeHooks {
    fn max_finish_output_len(&self, _codec: &TargetCodec) -> usize {
        self.offset as usize
    }

    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut TargetCodec,
        _context: &EncodeContext<'_, u8>,
    ) -> Result<EncodeUnencodableAction<u8>, codec::TranscodeEncodeErrorOf<TargetCodec>> {
        Err(TranscodeEncodeError::domain_main(
            EngineError::Encode,
            _context.input_index(),
        ))
    }
}

pub(in crate::transcode::engine) type CopyConvertEngine =
    TranscodeConvertEngine<SourceCodec, TargetCodec, StrictDecodeHooks, StrictEncodeHooks>;

#[must_use]
pub(in crate::transcode::engine) fn new_copy_engine() -> CopyConvertEngine {
    let mut engine = TranscodeConvertEngine::new(SourceCodec, TargetCodec, StrictDecodeHooks, StrictEncodeHooks);
    engine.reset(&mut [], 0).expect("reset before copy conversion");
    engine
}

#[must_use]
pub(in crate::transcode::engine) fn new_error_path_engine(
    hooks: ErrorPathHooks,
) -> TranscodeConvertEngine<SourceCodec, TargetCodec, ErrorPathDecodeHooks, ErrorPathEncodeHooks> {
    let decode_hooks = ErrorPathDecodeHooks {
        finish: hooks.decode_finish,
        finish_len: hooks.decode_finish_len,
        max_output_error: hooks.decode_max_output_error,
    };
    let encode_hooks = ErrorPathEncodeHooks {
        finish_len: hooks.encode_finish_len,
        max_output_error: hooks.encode_max_output_error,
        max_output_error_for_len: hooks.encode_max_output_error_for_len,
        mode: hooks.encode_mode,
    };
    TranscodeConvertEngine::new(SourceCodec, TargetCodec, decode_hooks, encode_hooks)
}

#[must_use]
pub(in crate::transcode::engine) fn new_finish_engine(
    hooks: FinishHooks,
) -> TranscodeConvertEngine<SourceCodec, TargetCodec, FinishDecodeHooks, StrictEncodeHooks> {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        FinishDecodeHooks {
            value: Some(hooks.value),
            finish_bound: 1,
            fail: false,
        },
        StrictEncodeHooks,
    );
    engine.reset(&mut [], 0).expect("reset before finish conversion");
    engine
}

#[must_use]
pub(in crate::transcode::engine) fn new_batch_finish_engine()
-> TranscodeConvertEngine<SourceCodec, TargetCodec, BatchFinishDecodeHooks, StrictEncodeHooks> {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        BatchFinishDecodeHooks::default(),
        StrictEncodeHooks,
    );
    engine.reset(&mut [], 0).expect("reset before batch finish conversion");
    engine
}

#[must_use]
pub(in crate::transcode::engine) fn new_finish_encode_engine()
-> TranscodeConvertEngine<SourceCodec, TargetCodec, StrictDecodeHooks, FinishEncodeHooks> {
    let mut engine = TranscodeConvertEngine::new(
        SourceCodec,
        TargetCodec,
        StrictDecodeHooks,
        FinishEncodeHooks::default(),
    );
    engine.reset(&mut [], 0).expect("reset before finish encode conversion");
    engine
}

#[must_use]
pub(in crate::transcode::engine) fn new_repair_engine(
    action: RepairAction,
) -> TranscodeConvertEngine<ErrorSourceCodec, TargetCodec, RepairDecodeHooks, StrictEncodeHooks> {
    let mut engine = TranscodeConvertEngine::new(
        ErrorSourceCodec,
        TargetCodec,
        RepairDecodeHooks { action },
        StrictEncodeHooks,
    );
    engine.reset(&mut [], 0).expect("reset before repair conversion");
    engine
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::transcode::engine) struct ResetObservingDecodeHooks {
    pub(in crate::transcode::engine) called: Rc<Cell<bool>>,
}

impl TranscodeDecodeHooks<SourceCodec> for ResetObservingDecodeHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut SourceCodec,
        error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<SourceCodec>> {
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
pub(in crate::transcode::engine) struct StatelessResetSourceCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct StatelessResetFailingSourceCodec;

impl Codec for StatelessResetSourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_RESET_VALUES: usize = 0;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index <= input.len());
        debug_assert!(input.len() - input_index >= Self::MIN_UNITS_PER_VALUE);
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

    unsafe fn decode_reset(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::DecodeError> {
        Ok(0)
    }
}

impl Codec for StatelessResetFailingSourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = EngineError;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_RESET_VALUES: usize = 0;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index <= input.len());
        debug_assert!(input.len() - input_index >= Self::MIN_UNITS_PER_VALUE);
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

    unsafe fn decode_reset(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::DecodeError> {
        Err(EngineError::Decode)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::transcode::engine) struct StatelessResetSourceDecodeHooks {
    pub(in crate::transcode::engine) called: Rc<Cell<bool>>,
}

impl TranscodeDecodeHooks<StatelessResetSourceCodec> for StatelessResetSourceDecodeHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut StatelessResetSourceCodec,
        error: &Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<StatelessResetSourceCodec>> {
        match *error {}
    }

    fn reset_hooks(&mut self, _codec: &mut StatelessResetSourceCodec) {
        self.called.set(true);
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct StatelessResetFailingSourceDecodeHooks;

impl TranscodeDecodeHooks<StatelessResetFailingSourceCodec> for StatelessResetFailingSourceDecodeHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut StatelessResetFailingSourceCodec,
        error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<StatelessResetFailingSourceCodec>> {
        match error {
            EngineError::Decode | EngineError::Encode => {
                unreachable!("reset path should not produce decode errors")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct OverflowResetSourceCodec;

impl Codec for OverflowResetSourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_RESET_VALUES: usize = usize::MAX;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index <= input.len());
        debug_assert!(input.len() - input_index >= Self::MIN_UNITS_PER_VALUE);
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

    unsafe fn decode_reset(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::DecodeError> {
        Ok(0)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct OverflowResetTargetCodec;

impl Codec for OverflowResetTargetCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_RESET_UNITS: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index <= input.len());
        debug_assert!(input.len() - input_index >= Self::MIN_UNITS_PER_VALUE);
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

    unsafe fn encode_reset(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::EncodeError> {
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct OverflowResetSourceDecodeHooks;

impl TranscodeDecodeHooks<OverflowResetSourceCodec> for OverflowResetSourceDecodeHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut OverflowResetSourceCodec,
        error: &Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<OverflowResetSourceCodec>> {
        match *error {}
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct OverflowResetTargetEncodeHooks;

impl TranscodeEncodeHooks<OverflowResetTargetCodec> for OverflowResetTargetEncodeHooks {
    fn handle_unencodable_encode(
        &mut self,
        _codec: &mut OverflowResetTargetCodec,
        _context: &EncodeContext<'_, u8>,
    ) -> Result<EncodeUnencodableAction<u8>, codec::TranscodeEncodeErrorOf<OverflowResetTargetCodec>> {
        unreachable!("overflow reset target codec accepts all u8 values")
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct MismatchCapacityResetEmittingDecodeHooks;

impl TranscodeDecodeHooks<ResetEmittingSourceCodec> for MismatchCapacityResetEmittingDecodeHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut ResetEmittingSourceCodec,
        error: &Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<ResetEmittingSourceCodec>> {
        match *error {}
    }
}

// Decode-reset emit fixtures for regression testing
// `TranscodeConvertEngine::reset` against codecs whose
// `MAX_DECODE_RESET_VALUES > 0`. The previous implementation hard-asserted that
// decode-side resets are absent, which silently dropped any stream-start values
// the source decoder wanted to emit (such as a BOM) before they reached the
// target encoder.

/// Decode value sentinel written by [`ResetEmittingSourceCodec::decode_reset`].
pub(in crate::transcode::engine) const SOURCE_RESET_SENTINEL: u8 = 0xbb;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct ResetEmittingSourceCodec;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct ResetEncodingFailSourceCodec;

impl Codec for ResetEmittingSourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_RESET_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index <= input.len());
        debug_assert!(input.len() - input_index >= Self::MIN_UNITS_PER_VALUE);
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

    unsafe fn decode_reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::DecodeError> {
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
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_RESET_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index <= input.len());
        debug_assert!(input.len() - input_index >= Self::MIN_UNITS_PER_VALUE);
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

    unsafe fn decode_reset(&mut self, output: &mut [u8], output_index: usize) -> Result<usize, Self::DecodeError> {
        // SAFETY: The caller guarantees room for one reset value at
        // `output_index`.
        unsafe {
            *output.get_unchecked_mut(output_index) = 13;
        }
        Ok(1)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct ResetSourceDecodeHooks;

impl TranscodeDecodeHooks<ResetEmittingSourceCodec> for ResetSourceDecodeHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut ResetEmittingSourceCodec,
        error: &Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<ResetEmittingSourceCodec>> {
        match *error {}
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct ResetEncodingFailSourceDecodeHooks;

impl TranscodeDecodeHooks<ResetEncodingFailSourceCodec> for ResetEncodingFailSourceDecodeHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut ResetEncodingFailSourceCodec,
        error: &Infallible,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<ResetEncodingFailSourceCodec>> {
        match *error {}
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct ResetFailingSourceCodec;

impl Codec for ResetFailingSourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = EngineError;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_RESET_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index <= input.len());
        debug_assert!(input.len() - input_index >= Self::MIN_UNITS_PER_VALUE);
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

    unsafe fn decode_reset(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::DecodeError> {
        Err(EngineError::Decode)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct ResetFailSourceDecodeHooks;

impl TranscodeDecodeHooks<ResetFailingSourceCodec> for ResetFailSourceDecodeHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut ResetFailingSourceCodec,
        error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<ResetFailingSourceCodec>> {
        match error {
            EngineError::Decode | EngineError::Encode => {
                unreachable!("reset path should not produce decode errors")
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct FinishFailingSourceCodec;

impl Codec for FinishFailingSourceCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = EngineError;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 1;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 1;

    const MAX_DECODE_UNITS_PER_VALUE: usize = 1;
    const MAX_DECODE_FINISH_VALUES: usize = 1;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u8, NonZeroUsize), codec::DecodeFailure<Self::DecodeError>> {
        debug_assert!(input_index <= input.len());
        debug_assert!(input.len() - input_index >= Self::MIN_UNITS_PER_VALUE);
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

    unsafe fn decode_finish(&mut self, _output: &mut [u8], _output_index: usize) -> Result<usize, Self::DecodeError> {
        Err(EngineError::Decode)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::transcode::engine) struct FinishFailSourceDecodeHooks;

impl TranscodeDecodeHooks<FinishFailingSourceCodec> for FinishFailSourceDecodeHooks {
    fn handle_invalid_decode(
        &mut self,
        _codec: &mut FinishFailingSourceCodec,
        error: &EngineError,
        _consumed: Option<NonZeroUsize>,
        _context: DecodeContext,
    ) -> Result<DecodeInvalidAction<u8>, codec::TranscodeDecodeErrorOf<FinishFailingSourceCodec>> {
        match error {
            EngineError::Decode | EngineError::Encode => {
                unreachable!("finish path should not produce decode errors")
            }
        }
    }
}

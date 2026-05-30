/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Policy hooks used by the default codec-backed buffered converter.

use core::{
    marker::PhantomData,
    num::NonZeroUsize,
};

use super::{
    ConvertDecodeResult,
    ConvertWriteResult,
    buffered_convert_hooks::BufferedConvertHooks,
    convert_state::ConvertState,
    transcode_progress::TranscodeProgress,
};
use crate::{
    Codec,
    CodecConvertError,
    CodecDecodeError,
    DecodeErrorInfo,
    DecodeFailure,
    codec::debug_assert_unit_bounds,
};

/// Policy hooks for [`super::CodecBufferedConverter`].
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub(super) struct CodecBufferedConvertHooks<Value> {
    /// Decoded value waiting for target output capacity.
    pending: Option<Value>,
    /// Binds the hooks to one decoded logical value type.
    marker: PhantomData<fn(Value)>,
}

/// Result of attempting to encode one decoded converter value.
enum WriteValueResult {
    /// The value was written and this many output units were produced.
    Written(usize),
    /// The value was retained and this many extra output units are needed.
    Pending { additional: NonZeroUsize },
}

impl<Value> CodecBufferedConvertHooks<Value> {
    /// Creates codec-backed converter hooks.
    ///
    /// # Returns
    ///
    /// Returns hooks with no pending decoded value.
    #[must_use]
    #[inline(always)]
    pub(super) const fn new() -> Self {
        Self {
            pending: None,
            marker: PhantomData,
        }
    }

    /// Returns whether a decoded value is waiting for output capacity.
    #[must_use]
    #[inline(always)]
    fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// Writes a decoded value or retains it until more output is available.
    ///
    /// # Parameters
    ///
    /// - `encoder`: Target codec used to encode the value.
    /// - `value`: Decoded value to encode.
    /// - `output`: Complete target output slice visible to this call.
    /// - `output_cursor`: Absolute target output cursor.
    ///
    /// # Returns
    ///
    /// Returns whether the value was written or retained.
    ///
    /// # Errors
    ///
    /// Returns the wrapped encoder error when the value cannot be encoded.
    #[inline]
    fn write_or_keep_pending<E, OutputUnit>(
        &mut self,
        encoder: &E,
        value: Value,
        output: &mut [OutputUnit],
        output_cursor: usize,
    ) -> Result<WriteValueResult, E::EncodeError>
    where
        E: Codec<Value, OutputUnit>,
        OutputUnit: Copy,
    {
        let output_available = output.len().saturating_sub(output_cursor);
        let max_output_units = encoder.max_units_per_value();
        let required = max_output_units.get();
        if output_available < required {
            self.pending = Some(value);
            return Ok(WriteValueResult::Pending {
                additional: NonZeroUsize::new(required.saturating_sub(output_available)).unwrap_or(NonZeroUsize::MIN),
            });
        }

        // SAFETY: The remaining output capacity is at least the encoder codec's
        // declared maximum width for one encoded value.
        let produced = unsafe { encoder.encode_unchecked(&value, output, output_cursor) }?;
        debug_assert!(
            produced <= output_available,
            "Codec::encode_unchecked wrote beyond available output",
        );
        Ok(WriteValueResult::Written(produced))
    }

    /// Writes a previously retained decoded value.
    ///
    /// # Parameters
    ///
    /// - `encoder`: Target codec used to encode the value.
    /// - `output`: Complete target output slice visible to this call.
    /// - `output_cursor`: Absolute target output cursor.
    ///
    /// # Returns
    ///
    /// Returns whether the retained value was written or kept pending. If no
    /// pending value exists, returns [`WriteValueResult::Written`] with zero
    /// produced units.
    ///
    /// # Errors
    ///
    /// Returns the wrapped encoder error when the retained value cannot be
    /// encoded.
    #[inline]
    fn write_pending<E, OutputUnit>(
        &mut self,
        encoder: &E,
        output: &mut [OutputUnit],
        output_cursor: usize,
    ) -> Result<WriteValueResult, E::EncodeError>
    where
        E: Codec<Value, OutputUnit>,
        OutputUnit: Copy,
    {
        let Some(value) = self.pending.take() else {
            return Ok(WriteValueResult::Written(0));
        };
        self.write_or_keep_pending(encoder, value, output, output_cursor)
    }
}

impl<D, E, Value, InputUnit, OutputUnit> BufferedConvertHooks<D, E, InputUnit, Value, OutputUnit>
    for CodecBufferedConvertHooks<Value>
where
    D: Codec<Value, InputUnit>,
    D::DecodeError: DecodeErrorInfo,
    E: Codec<Value, OutputUnit>,
    InputUnit: Copy,
    OutputUnit: Copy,
{
    type Error = CodecConvertError<D::DecodeError, E::EncodeError>;

    /// Returns the target codec maximum width for invalid output indices.
    #[inline(always)]
    fn invalid_output_additional(&self, _decoder: &D, encoder: &E) -> NonZeroUsize {
        encoder.max_units_per_value()
    }

    /// Returns an upper bound for target units produced from `input_len` units.
    #[inline(always)]
    fn max_output_len(&self, decoder: &D, encoder: &E, input_len: usize) -> Option<usize> {
        debug_assert_unit_bounds::<D, Value, InputUnit>(decoder);
        debug_assert_unit_bounds::<E, Value, OutputUnit>(encoder);
        let units_per_value = encoder.max_units_per_value().get();
        let pending_units = usize::from(self.has_pending()).checked_mul(units_per_value)?;
        let decoded_values = input_len / decoder.min_units_per_value().get();
        pending_units.checked_add(decoded_values.checked_mul(units_per_value)?)
    }

    /// Returns the maximum target units emitted by finishing internal state.
    #[inline(always)]
    fn max_finish_output_len(&self, _decoder: &D, encoder: &E) -> Option<usize> {
        debug_assert_unit_bounds::<E, Value, OutputUnit>(encoder);
        Some(usize::from(self.has_pending()) * encoder.max_units_per_value().get())
    }

    /// Clears retained pending output.
    #[inline(always)]
    fn reset(&mut self, _decoder: &mut D, _encoder: &mut E) {
        self.pending = None;
    }

    /// Writes retained pending output before reading new input.
    #[inline]
    fn drain_pending(
        &mut self,
        _decoder: &mut D,
        encoder: &mut E,
        state: &mut ConvertState<'_, InputUnit, OutputUnit>,
    ) -> Result<Option<TranscodeProgress>, Self::Error> {
        let output_cursor = state.output_cursor();
        match self
            .write_pending(encoder, state.output_mut(), output_cursor)
            .map_err(CodecConvertError::encode)?
        {
            WriteValueResult::Written(produced) => {
                state.advance_output(produced);
                Ok(None)
            }
            WriteValueResult::Pending { additional } => {
                Ok(Some(state.need_output_progress(additional, state.available_output())))
            }
        }
    }

    /// Decodes one codec-backed value from the current input cursor.
    #[inline]
    fn decode_next(
        &mut self,
        decoder: &mut D,
        state: &mut ConvertState<'_, InputUnit, OutputUnit>,
    ) -> Result<ConvertDecodeResult<Value>, Self::Error> {
        let min_units = decoder.min_units_per_value().get();
        let available = state.available_input();
        if available < min_units {
            return Ok(ConvertDecodeResult::NeedInput {
                additional: NonZeroUsize::new(min_units - available).expect("missing input is non-zero"),
                available,
            });
        }

        let input_cursor = state.input_cursor();
        let (value, consumed) = match unsafe { decoder.decode_unchecked(state.input(), input_cursor) } {
            Ok(result) => result,
            Err(error) => match error.failure() {
                DecodeFailure::Incomplete {
                    required_total,
                    available,
                } => {
                    return Ok(ConvertDecodeResult::NeedInput {
                        additional: NonZeroUsize::new(required_total.saturating_sub(available))
                            .unwrap_or(NonZeroUsize::MIN),
                        available,
                    });
                }
                DecodeFailure::Invalid { .. } => {
                    let error = CodecDecodeError::decode(error, input_cursor);
                    return Err(CodecConvertError::decode(error));
                }
            },
        };

        let consumed = consumed.get();
        debug_assert!(
            consumed <= state.available_input(),
            "Codec::decode_unchecked consumed beyond available input",
        );
        let consumed = NonZeroUsize::new(consumed).expect("codec decode consumption is non-zero");
        Ok(ConvertDecodeResult::Decoded { value, consumed })
    }

    /// Writes one decoded codec-backed value.
    #[inline]
    fn write_value(
        &mut self,
        encoder: &mut E,
        value: Value,
        state: &mut ConvertState<'_, InputUnit, OutputUnit>,
    ) -> Result<ConvertWriteResult, Self::Error> {
        let output_cursor = state.output_cursor();
        match self
            .write_or_keep_pending(encoder, value, state.output_mut(), output_cursor)
            .map_err(CodecConvertError::encode)?
        {
            WriteValueResult::Pending { additional } => Ok(ConvertWriteResult::NeedOutput {
                additional,
                available: state.available_output(),
                written: 0,
            }),
            WriteValueResult::Written(written) => Ok(ConvertWriteResult::Written { written }),
        }
    }

    /// Finishes retained pending output after EOF.
    #[inline]
    fn finish(
        &mut self,
        _decoder: &mut D,
        encoder: &mut E,
        output: &mut [OutputUnit],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        debug_assert_unit_bounds::<E, Value, OutputUnit>(encoder);
        match self
            .write_pending(encoder, output, output_index)
            .map_err(CodecConvertError::encode)?
        {
            WriteValueResult::Written(produced) => Ok(TranscodeProgress::complete(0, produced)),
            WriteValueResult::Pending { additional } => {
                let available = output.len().saturating_sub(output_index);
                Ok(TranscodeProgress::need_output(
                    output_index,
                    additional.get(),
                    available,
                    0,
                    0,
                ))
            }
        }
    }
}

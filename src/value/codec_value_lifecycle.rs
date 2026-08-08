// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal helpers for complete single-value codec lifecycles.

use crate::CapacityError;
use crate::Codec;
use crate::TranscodeDecodeError;
use crate::TranscodeDecodeErrorOf;
use crate::TranscodeEncodeError;
use crate::TranscodeEncodeErrorOf;
use crate::TranscodeFailure;
use crate::codec::assert_unit_bounds;

/// Returns the conservative maximum unit count for a complete encode lifecycle.
///
/// # Type Parameters
///
/// - `C`: Codec whose encode bounds are queried.
///
/// # Returns
///
/// Returns the checked sum of encode reset, value, and finish unit bounds.
///
/// # Errors
///
/// Returns [`CapacityError::OutputLengthOverflow`] when the sum cannot be
/// represented as `usize`.
#[inline(always)]
pub(crate) fn max_complete_encode_units<C>() -> Result<usize, CapacityError>
where
    C: Codec,
{
    assert_unit_bounds::<C>();
    C::MAX_ENCODE_RESET_UNITS
        .checked_add(C::MAX_ENCODE_UNITS_PER_VALUE)
        .and_then(|units| units.checked_add(C::MAX_ENCODE_FINISH_UNITS))
        .ok_or(CapacityError::OutputLengthOverflow)
}

/// Encodes one value through reset, encode, and finish into reserved output.
///
/// # Parameters
///
/// - `codec`: Codec used for the complete encode lifecycle.
/// - `value`: Value to encode.
/// - `output`: Destination unit buffer.
/// - `output_index`: Start index in `output`.
/// - `reserved`: Conservative complete output length returned by
///   [`max_complete_encode_units`].
///
/// # Returns
///
/// Returns the total number of units written.
///
/// # Errors
///
/// Returns [`TranscodeEncodeError`] when the reset-state codec domain rejects
/// `value`, output length arithmetic overflows, reserved output is
/// insufficient for a codec-reported exact width, or a codec reset, encode, or
/// finish hook reports a domain error.
///
/// # Panics
///
/// Panics when the caller did not reserve `reserved` units at `output_index`,
/// or when the codec violates its reset, encoded-value, or finish write-count
/// contract.
pub(crate) fn encode_complete_value_into_reserved<C>(
    codec: &mut C,
    value: &C::Value,
    output: &mut [C::Unit],
    output_index: usize,
    reserved: usize,
) -> Result<usize, TranscodeEncodeErrorOf<C>>
where
    C: Codec,
{
    assert!(
        output
            .len()
            .checked_sub(output_index)
            .is_some_and(|available| available >= reserved),
        "complete encode output was not reserved",
    );

    let reset_written = unsafe {
        // SAFETY: The caller reserved `reserved` units, which includes the
        // codec-declared reset bound at `output_index`.
        codec.encode_reset(output, output_index)
    }
    .map_err(TranscodeEncodeError::domain_reset)?;
    assert!(
        reset_written <= C::MAX_ENCODE_RESET_UNITS,
        "Codec::encode_reset wrote beyond its reset bound",
    );

    if !codec.can_encode_value(value) {
        return Err(TranscodeEncodeError::unencodable_without_context(0));
    }
    let value_units = codec.encode_len(value);
    assert!(
        value_units <= C::MAX_ENCODE_UNITS_PER_VALUE,
        "Codec::encode_len exceeded Codec::MAX_ENCODE_UNITS_PER_VALUE",
    );
    let value_and_finish = value_units
        .checked_add(C::MAX_ENCODE_FINISH_UNITS)
        .ok_or(CapacityError::OutputLengthOverflow)?;
    let value_index = output_index + reset_written;
    TranscodeFailure::ensure_output_capacity(
        output.len(),
        value_index,
        value_and_finish,
    )?;
    let value_written = unsafe {
        // SAFETY: The capacity check above leaves the reset-state exact value
        // width writable after reset output, and the reset-state domain check
        // establishes the codec's value precondition.
        codec.encode(value, output, value_index)
    }
    .map_err(|error| TranscodeEncodeError::domain_main(error, 0))?;
    assert!(
        value_written == value_units,
        "Codec::encode wrote a different length than Codec::encode_len",
    );

    let finish_index = value_index + value_written;
    let finish_written = unsafe {
        // SAFETY: The reserved complete lifecycle output includes the
        // codec-declared finish bound after reset and value output.
        codec.encode_finish(output, finish_index)
    }
    .map_err(TranscodeEncodeError::domain_finish)?;
    assert!(
        finish_written <= C::MAX_ENCODE_FINISH_UNITS,
        "Codec::encode_finish wrote beyond its finish bound",
    );
    Ok(reset_written + value_written + finish_written)
}

/// Decodes exactly one value through reset, decode, and finish.
///
/// Reset and finish output values are written into separate caller-provided
/// buffers so both phases remain observable.
///
/// # Parameters
///
/// - `codec`: Codec used for the complete decode lifecycle.
/// - `input`: Source units for exactly one encoded value.
/// - `reset_output`: Destination for reset output values.
/// - `finish_output`: Destination for finish output values.
///
/// # Returns
///
/// Returns the main decoded value and the actual reset and finish output
/// lengths.
///
/// # Errors
///
/// Returns [`TranscodeDecodeError`] when either lifecycle output buffer is
/// shorter than its declared bound, decoding fails, trailing input remains, or
/// reset or finish fails.
///
/// # Panics
///
/// Panics when the codec consumes beyond available input or its declared decode
/// maximum, or writes more reset or finish values than its declared bounds.
pub(crate) fn decode_exact_complete_value<C>(
    codec: &mut C,
    input: &[C::Unit],
    reset_output: &mut [C::Value],
    finish_output: &mut [C::Value],
) -> Result<(C::Value, usize, usize), TranscodeDecodeErrorOf<C>>
where
    C: Codec,
{
    assert_unit_bounds::<C>();
    TranscodeFailure::ensure_output_capacity(
        reset_output.len(),
        0,
        C::MAX_DECODE_RESET_VALUES,
    )?;
    TranscodeFailure::ensure_output_capacity(
        finish_output.len(),
        0,
        C::MAX_DECODE_FINISH_VALUES,
    )?;
    let reset_written = unsafe {
        // SAFETY: The capacity check above reserves the codec's declared
        // decode-reset output bound.
        codec.decode_reset(reset_output, 0)
    }
    .map_err(TranscodeDecodeError::domain_reset)?;
    assert!(
        reset_written <= C::MAX_DECODE_RESET_VALUES,
        "Codec::decode_reset wrote beyond its reset bound",
    );

    TranscodeFailure::ensure_min_input(input.len(), 0, C::MIN_UNITS_PER_VALUE)?;
    let (value, consumed) = unsafe {
        // SAFETY: The input check above guarantees the minimum readable units
        // required by `Codec::decode_eof` at index 0.
        codec.decode_eof(input, 0)
    }
    .map_err(|failure| {
        TranscodeDecodeError::from_decode_failure(failure, 0, input.len())
    })?;
    assert!(
        consumed.get() <= input.len(),
        "Codec::decode consumed beyond available input",
    );
    assert!(
        consumed.get() <= C::MAX_DECODE_UNITS_PER_VALUE,
        "Codec::decode consumed beyond Codec::MAX_DECODE_UNITS_PER_VALUE",
    );
    TranscodeFailure::ensure_no_trailing_input(consumed.get(), input.len())?;

    let finish_written = unsafe {
        // SAFETY: The capacity check above reserves the codec's declared
        // decode-finish output bound.
        codec.decode_finish(finish_output, 0)
    }
    .map_err(TranscodeDecodeError::domain_finish)?;
    assert!(
        finish_written <= C::MAX_DECODE_FINISH_VALUES,
        "Codec::decode_finish wrote beyond its finish bound",
    );
    Ok((value, reset_written, finish_written))
}

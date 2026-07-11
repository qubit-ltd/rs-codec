// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Internal helpers for complete single-value codec lifecycles.

use crate::{
    CapacityError,
    Codec,
    TranscodeDecodeError,
    TranscodeDecodeErrorOf,
    TranscodeEncodeError,
    TranscodeEncodeErrorOf,
    TranscodeFailure,
    codec::decode_lifecycle_scratch_len,
};

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
    C::MAX_ENCODE_RESET_UNITS
        .checked_add(C::MAX_UNITS_PER_VALUE)
        .and_then(|units| units.checked_add(C::MAX_ENCODE_FINISH_UNITS))
        .ok_or(CapacityError::OutputLengthOverflow)
}

/// Returns the exact unit count needed for a complete encode lifecycle.
///
/// # Parameters
///
/// - `codec`: Codec used to validate and size the value.
/// - `value`: Value that will be encoded.
///
/// # Returns
///
/// Returns the checked reset, exact value, and finish output length.
///
/// # Errors
///
/// Returns [`TranscodeEncodeError`] when `value` is outside the codec domain or
/// when output length arithmetic overflows.
pub(crate) fn complete_encode_len<C>(
    codec: &C,
    value: &C::Value,
) -> Result<usize, TranscodeEncodeErrorOf<C>>
where
    C: Codec,
{
    if !codec.can_encode_value(value) {
        return Err(TranscodeEncodeError::unencodable_without_context(0));
    }
    let units = C::MAX_ENCODE_RESET_UNITS
        .checked_add(codec.encode_len(value))
        .and_then(|units| units.checked_add(C::MAX_ENCODE_FINISH_UNITS))
        .ok_or(CapacityError::OutputLengthOverflow)?;
    Ok(units)
}

/// Encodes one value through reset, encode, and finish into reserved output.
///
/// # Parameters
///
/// - `codec`: Codec used for the complete encode lifecycle.
/// - `value`: Value to encode.
/// - `output`: Destination unit buffer.
/// - `output_index`: Start index in `output`.
/// - `required`: Exact complete output length previously returned by
///   [`complete_encode_len`].
///
/// # Returns
///
/// Returns the total number of units written.
///
/// # Errors
///
/// Returns [`TranscodeEncodeError`] when a codec reset, encode, or finish hook
/// reports a domain error.
///
/// # Panics
///
/// Panics when the caller did not reserve `required` units at `output_index`,
/// or when the codec violates its reset, value, or finish length contract.
pub(crate) fn encode_complete_value_into_reserved<C>(
    codec: &mut C,
    value: &C::Value,
    output: &mut [C::Unit],
    output_index: usize,
    required: usize,
) -> Result<usize, TranscodeEncodeErrorOf<C>>
where
    C: Codec,
{
    assert!(
        output
            .len()
            .checked_sub(output_index)
            .is_some_and(|available| available >= required),
        "complete encode output was not reserved",
    );

    let reset_written = unsafe {
        // SAFETY: The caller reserved `required` units, which includes the
        // codec-declared reset bound at `output_index`.
        codec.encode_reset(output, output_index)
    }
    .map_err(TranscodeEncodeError::domain_reset)?;
    assert!(
        reset_written <= C::MAX_ENCODE_RESET_UNITS,
        "Codec::encode_reset wrote beyond its reset bound",
    );

    let value_units = codec.encode_len(value);
    let value_written = unsafe {
        // SAFETY: The reserved complete lifecycle output leaves the exact
        // value width writable after reset output.
        codec.encode(value, output, output_index + reset_written)
    }
    .map_err(|error| TranscodeEncodeError::domain_main(error, 0))?;
    assert!(
        value_written == value_units,
        "Codec::encode wrote a different length than Codec::encode_len",
    );

    let finish_index = output_index + reset_written + value_written;
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
/// Reset and finish output values are written into `scratch` and discarded by
/// value-level adapters; the returned value is the main decoded value.
///
/// # Parameters
///
/// - `codec`: Codec used for the complete decode lifecycle.
/// - `input`: Source units for exactly one encoded value.
/// - `scratch`: Scratch destination for reset and finish output.
///
/// # Returns
///
/// Returns the main decoded value.
///
/// # Errors
///
/// Returns [`TranscodeDecodeError`] when decoding fails, when trailing input
/// remains, or when reset or finish fails.
///
/// # Panics
///
/// Panics when [`Codec::MAX_DECODE_LIFECYCLE_VALUES`] does not match the reset
/// and finish bounds, when the caller did not reserve enough lifecycle
/// scratch, when the codec consumes beyond available input, or when the codec
/// writes more reset or finish values than its declared bounds.
pub(crate) fn decode_exact_complete_value<C>(
    codec: &mut C,
    input: &[C::Unit],
    scratch: &mut [C::Value],
) -> Result<C::Value, TranscodeDecodeErrorOf<C>>
where
    C: Codec,
{
    TranscodeFailure::ensure_min_input(input.len(), 0, C::MIN_UNITS_PER_VALUE)?;

    let scratch_cap = decode_lifecycle_scratch_len::<C>();
    assert!(
        scratch.len() >= scratch_cap,
        "complete decode scratch output was not reserved",
    );
    let reset_written = unsafe {
        // SAFETY: The scratch capacity check above reserves the codec's
        // declared decode-reset output bound.
        codec.decode_reset(scratch, 0)
    }
    .map_err(TranscodeDecodeError::domain_reset)?;
    assert!(
        reset_written <= C::MAX_DECODE_RESET_VALUES,
        "Codec::decode_reset wrote beyond its reset bound",
    );

    let (value, consumed) = unsafe {
        // SAFETY: The input check above guarantees the minimum readable units
        // required by `Codec::decode` at index 0.
        codec.decode(input, 0)
    }
    .map_err(|failure| {
        TranscodeDecodeError::from_decode_failure(failure, 0, input.len())
    })?;
    assert!(
        consumed.get() <= input.len(),
        "Codec::decode consumed beyond available input",
    );
    TranscodeFailure::ensure_no_trailing_input(consumed.get(), input.len())?;

    let finish_written = unsafe {
        // SAFETY: The scratch capacity check above reserves the codec's
        // declared decode-finish output bound.
        codec.decode_finish(scratch, 0)
    }
    .map_err(TranscodeDecodeError::domain_finish)?;
    assert!(
        finish_written <= C::MAX_DECODE_FINISH_VALUES,
        "Codec::decode_finish wrote beyond its finish bound",
    );
    Ok(value)
}

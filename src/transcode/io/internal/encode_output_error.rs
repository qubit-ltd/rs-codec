// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Error translation helpers for buffered encoder outputs.

use core::result::Result as CoreResult;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result;

use crate::CapacityError;
use crate::TranscodeEncodeError;

/// Wraps a streaming capacity failure as `InvalidData`, retaining its source.
///
/// # Parameters
///
/// * `error` - Capacity planning failure from a streaming transcoder.
///
/// # Returns
///
/// An owned I/O error carrying the original capacity error.
#[must_use]
pub(in crate::transcode::io) fn capacity_error_to_invalid_data(error: CapacityError) -> Error {
    Error::new(ErrorKind::InvalidData, error)
}

/// Preserves successful values and maps one-value failures into I/O errors.
///
/// # Type Parameters
///
/// * `T` - Successful result value.
/// * `E` - Codec-domain error passed to the mapper.
/// * `Value` - Source value type carried by an unencodable error.
///
/// # Parameters
///
/// * `result` - Codec lifecycle result to translate.
/// * `map_error` - Called only for a codec-domain failure.
///
/// # Returns
///
/// The original successful value without transformation.
///
/// # Errors
///
/// Returns mapped domain errors or `InvalidInput` for bounds and domain
/// rejection.
pub(in crate::transcode::io) fn map_encode_value_result<T, E, Value>(
    result: CoreResult<T, TranscodeEncodeError<E, Value>>,
    map_error: &mut dyn FnMut(E) -> Error,
) -> Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => Err(map_encode_value_error(error, map_error)),
    }
}

/// Maps domain failures through the caller and other failures to
/// `InvalidInput`.
///
/// # Type Parameters
///
/// * `E` - Codec-domain error passed to the mapper.
/// * `Value` - Rejected source value type, discarded during I/O translation.
///
/// # Parameters
///
/// * `error` - Encoding failure consumed by this translation.
/// * `map_error` - Invoked only for the `Domain` variant.
///
/// # Returns
///
/// An I/O error retaining mapped domain detail or a bound/rejection message.
#[must_use]
#[inline(never)]
fn map_encode_value_error<E, Value>(
    error: TranscodeEncodeError<E, Value>,
    map_error: &mut dyn FnMut(E) -> Error,
) -> Error {
    match error {
        TranscodeEncodeError::Domain(error) => map_error(error.into_source()),
        TranscodeEncodeError::Unencodable { .. } => Error::new(ErrorKind::InvalidInput, "codec cannot encode value"),
        TranscodeEncodeError::Failure(_) => Error::new(ErrorKind::InvalidInput, "codec output bound overflow"),
    }
}

/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Decode error metadata trait.

use core::convert::Infallible;

use super::decode_failure::DecodeFailure;

/// Exposes buffered-decode control-flow metadata from a codec-specific error.
///
/// Codec errors remain responsible for carrying the full domain-specific reason
/// and indexes. This trait is only the generic view needed by adapters that
/// cannot know each codec's private error taxonomy.
pub trait DecodeErrorInfo {
    /// Returns the generic buffered-decode view of this error.
    ///
    /// # Returns
    ///
    /// Returns whether the caller should wait for more input or consume invalid
    /// units to make progress.
    fn failure(&self) -> DecodeFailure;
}

impl DecodeErrorInfo for Infallible {
    /// Converts an impossible decode error into failure metadata.
    fn failure(&self) -> DecodeFailure {
        match *self {}
    }
}

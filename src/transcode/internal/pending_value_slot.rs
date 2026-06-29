// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Slot that owns the converter's retained decoded value.

use super::pending_value::PendingValue;
use crate::{CapacityError, Codec, TranscodeEncodeEngine, TranscodeEncodeHooks};

/// Slot that owns the converter's retained decoded value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(in crate::transcode) struct PendingValueSlot<Value> {
    /// Retained decoded value waiting for output capacity.
    value: Option<PendingValue<Value>>,
}

impl<Value> PendingValueSlot<Value> {
    /// Creates an empty pending-value slot.
    #[inline(always)]
    #[must_use]
    pub(in crate::transcode) const fn empty() -> Self {
        Self { value: None }
    }

    /// Returns the target-output bound for the retained value.
    ///
    /// # Type Parameters
    ///
    /// - `E`: Encoder codec type used to query output bounds.
    /// - `H`: Encoder hook type used by the encoder engine.
    ///
    /// # Parameters
    ///
    /// - `engine`: Target encode engine for one-value output bound query.
    ///
    /// # Returns
    ///
    /// Returns the output unit bound contributed by the retained value.
    #[must_use = "capacity planning can fail on overflow"]
    #[inline]
    pub(in crate::transcode) fn max_transcode_output_len<E, H>(
        &self,
        engine: &TranscodeEncodeEngine<E, H>,
    ) -> Result<usize, CapacityError>
    where
        E: Codec<Value = Value>,
        H: TranscodeEncodeHooks<E>,
    {
        if self.value.is_some() {
            engine
                .max_transcode_output_len(1)
                .map_err(|_| CapacityError::OutputLengthOverflow)
        } else {
            Ok(0)
        }
    }

    /// Removes any retained decoded value.
    ///
    /// # Returns
    ///
    /// Returns unit `()`.
    #[inline(always)]
    pub(in crate::transcode) fn clear(&mut self) {
        self.value = None;
    }

    /// Takes the retained decoded value, if any.
    ///
    /// # Returns
    ///
    /// Returns the retained value when present, otherwise `None`.
    #[inline(always)]
    pub(in crate::transcode) fn take(&mut self) -> Option<PendingValue<Value>> {
        self.value.take()
    }

    /// Stores a decoded value that could not be encoded yet.
    ///
    /// # Parameters
    ///
    /// - `pending`: Decoded value and its source input position.
    #[inline(always)]
    pub(in crate::transcode) fn put(&mut self, pending: PendingValue<Value>) {
        self.value = Some(pending);
    }
}

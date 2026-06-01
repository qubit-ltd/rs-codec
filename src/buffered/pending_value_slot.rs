/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Slot that owns the converter's retained decoded value.

use super::{
    buffered_encode_engine::BufferedEncodeEngine,
    buffered_encode_hooks::BufferedEncodeHooks,
    convert_state::ConvertState,
    pending_encode_step::PendingEncodeStep,
    pending_value::PendingValue,
    transcode_progress::TranscodeProgress,
};
use crate::{
    CapacityError,
    Codec,
};

/// Slot that owns the converter's retained decoded value.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct PendingValueSlot<Value> {
    /// Retained decoded value waiting for output capacity.
    value: Option<PendingValue<Value>>,
}

impl<Value> PendingValueSlot<Value> {
    /// Creates an empty pending-value slot.
    #[must_use]
    #[inline(always)]
    pub(super) const fn empty() -> Self {
        Self { value: None }
    }

    /// Returns the target-output bound for the retained value.
    #[must_use = "capacity planning can fail on overflow"]
    #[inline(always)]
    pub(super) fn max_output_len<E, H, Output>(
        &self,
        engine: &BufferedEncodeEngine<E, H>,
    ) -> Result<usize, CapacityError>
    where
        E: Codec<Value, Output>,
        H: BufferedEncodeHooks<E, Value, Output>,
        Output: Copy,
    {
        if self.value.is_some() {
            engine.max_output_len::<Value, Output>(1)
        } else {
            Ok(0)
        }
    }

    /// Removes any retained decoded value.
    #[inline(always)]
    pub(super) fn clear(&mut self) {
        self.value = None;
    }

    /// Takes the retained decoded value, if any.
    #[inline(always)]
    pub(super) fn take(&mut self) -> Option<PendingValue<Value>> {
        self.value.take()
    }

    /// Applies a pending-value encode step to this slot and the current conversion state.
    #[inline(always)]
    pub(super) fn apply_pending_encode_step<Input, Output>(
        &mut self,
        step: PendingEncodeStep<Value>,
        state: &mut ConvertState<'_, Input, Output>,
    ) -> Option<TranscodeProgress> {
        match step {
            PendingEncodeStep::Written { written } => {
                state.advance_output(written);
                None
            }
            PendingEncodeStep::NeedOutput {
                pending,
                additional,
                available,
            } => {
                self.value = Some(pending);
                Some(state.need_output_progress(additional, available))
            }
        }
    }
}

/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Converter action produced by one source-side finish hook call.

use super::{
    decode_finish_after_emit::DecodeFinishAfterEmit,
    pending_value::PendingValue,
    transcode_progress::TranscodeProgress,
    transcode_status::TranscodeStatus,
};

/// Converter action produced by one source-side finish hook call.
pub(super) enum DecodeFinishStep<Value> {
    /// Source-side finish hooks are complete.
    Complete,
    /// A final decoded value must be encoded.
    Emit {
        /// Pending final value produced by source-side finish hooks.
        pending: PendingValue<Value>,
        /// Source-side finish state after the value is encoded.
        after_emit: DecodeFinishAfterEmit,
    },
    /// Source-side finish requested more decoded output without emitting.
    #[cfg(not(debug_assertions))]
    NeedOutputWithoutValue,
}

impl<Value> DecodeFinishStep<Value> {
    /// Builds a finish step from source-side finish progress.
    ///
    /// # Parameters
    ///
    /// - `finish`: Progress returned by the source-side finish hook.
    /// - `decoded`: One-value scratch buffer passed to the finish hook.
    ///
    /// # Returns
    ///
    /// Returns the converter-level finish step represented by `finish`.
    #[must_use]
    pub(super) fn from_progress(finish: TranscodeProgress, decoded: [Value; 1]) -> Self {
        debug_assert!(
            finish.written() <= 1,
            "BufferedDecodeEngine finish wrote beyond the converter scratch buffer",
        );

        if finish.written() != 0 {
            let [value] = decoded;
            return Self::Emit {
                pending: PendingValue::new(value, 0),
                after_emit: DecodeFinishAfterEmit::from_status(finish.status()),
            };
        }

        match finish.status() {
            TranscodeStatus::Complete => Self::Complete,
            TranscodeStatus::NeedOutput { .. } => {
                #[cfg(debug_assertions)]
                {
                    unreachable!("decode finish hook must emit progress before requesting more decoded output")
                }
                #[cfg(not(debug_assertions))]
                {
                    Self::NeedOutputWithoutValue
                }
            }
            TranscodeStatus::NeedInput { .. } => {
                unreachable!("buffered decode engine finish cannot request source input")
            }
        }
    }
}

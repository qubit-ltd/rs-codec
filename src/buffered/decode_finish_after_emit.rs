/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Source-side finish state after an emitted final value is encoded.

use super::transcode_status::TranscodeStatus;

/// Source-side finish state after an emitted final value is encoded.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum DecodeFinishAfterEmit {
    /// Source-side finish hooks are complete after this value.
    Complete,
    /// Source-side finish hooks may emit more values.
    Continue,
}

impl DecodeFinishAfterEmit {
    /// Converts source-side finish status into post-emit control flow.
    ///
    /// # Parameters
    ///
    /// - `status`: Status returned by source-side finish hooks.
    ///
    /// # Returns
    ///
    /// Returns whether finalization is complete after the emitted value.
    #[must_use]
    pub(super) fn from_status(status: TranscodeStatus) -> Self {
        match status {
            TranscodeStatus::Complete => Self::Complete,
            TranscodeStatus::NeedOutput { .. } => Self::Continue,
            TranscodeStatus::NeedInput { .. } => {
                unreachable!("buffered decode engine finish cannot request source input")
            }
        }
    }

    /// Returns whether no more source-side finish values are expected.
    #[must_use]
    #[inline(always)]
    pub(super) const fn is_complete(self) -> bool {
        matches!(self, Self::Complete)
    }
}

// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Lifecycle guard shared by transcode engines.
//!
//! [`Transcoder`](crate::Transcoder) documents a lifecycle of
//! `reset → transcode* → finish` and then `reset` again before reusing the
//! instance for another logical stream. The trait itself does not enforce
//! this; engines historically rely on caller discipline. `LifecycleGuard`
//! rejects common misuse (calling `transcode` after `finish` without an
//! intervening `reset`, or calling `finish` twice in a row) in every build
//! profile.

use super::lifecycle_phase::LifecyclePhase;
use crate::TranscodeFailure;

/// Lifecycle guard for transcode engines.
///
/// Lifecycle rules:
///
/// - `transcode` is rejected when the engine is `Finished`. Callers must
///   `reset` before starting another logical stream.
/// - `finish` is rejected when the engine is already `Finished`. Repeating
///   `finish` is almost always a bug.
/// - `reset` is always legal and returns the engine to `Fresh`.
///
/// `Fresh → finish` is intentionally allowed: stateless transcoders may
/// finalize an empty stream, and forcing a synthetic `transcode(&[])` call
/// just to satisfy the guard would be noise.
#[derive(Debug, Default)]
pub(crate) struct LifecycleGuard {
    /// Current lifecycle phase.
    phase: LifecyclePhase,
}

impl LifecycleGuard {
    /// Creates a guard in the [`LifecyclePhase::Fresh`] phase.
    ///
    /// # Returns
    ///
    /// Returns a guard ready to observe the first lifecycle event.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn new() -> Self {
        Self {
            phase: LifecyclePhase::Fresh,
        }
    }

    /// Records a `reset` event. Always legal; returns the guard to
    /// [`LifecyclePhase::Fresh`].
    #[inline(always)]
    pub(crate) fn on_reset(&mut self) {
        self.phase = LifecyclePhase::Fresh;
    }

    /// Records a `transcode` entry.
    ///
    /// # Errors
    ///
    /// Returns [`TranscodeFailure::TranscodeAfterFinish`] when `transcode` is
    /// called after `finish` without an intervening `reset`.
    #[inline(always)]
    pub(crate) fn on_transcode(&mut self) -> Result<(), TranscodeFailure> {
        if self.phase == LifecyclePhase::Finished {
            return Err(TranscodeFailure::TranscodeAfterFinish);
        }
        if self.phase == LifecyclePhase::Fresh {
            self.phase = LifecyclePhase::Streaming;
        }
        Ok(())
    }

    /// Validates that the guard may enter `finish`. Does not change
    /// state, so callers that fail before completing finish (for example,
    /// capacity checks rejecting the supplied output) can retry without
    /// being marked closed.
    ///
    /// # Errors
    ///
    /// Returns [`TranscodeFailure::FinishAfterFinish`] when `finish` is called
    /// twice without an intervening `reset`.
    #[inline(always)]
    pub(crate) fn on_finish_attempt(&self) -> Result<(), TranscodeFailure> {
        if self.phase == LifecyclePhase::Finished {
            return Err(TranscodeFailure::FinishAfterFinish);
        }
        Ok(())
    }

    /// Commits the `Finished` state after `finish` actually completed. Call
    /// only on the success path.
    #[inline(always)]
    pub(crate) fn on_finish_success(&mut self) {
        self.phase = LifecyclePhase::Finished;
    }
}

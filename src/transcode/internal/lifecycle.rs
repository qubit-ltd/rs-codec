// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Debug-only lifecycle guard for transcode engines.
//!
//! [`Transcoder`](crate::Transcoder) documents a lifecycle of
//! `reset → transcode* → finish` and then `reset` again before reusing the
//! instance for another logical stream. The trait itself does not enforce
//! this; engines historically rely on caller discipline. `LifecycleGuard`
//! catches common misuse (calling `transcode` after `finish` without an
//! intervening `reset`, or calling `finish` twice in a row) in debug builds
//! while collapsing to a zero-sized type in release builds so hot paths pay
//! no extra cost.

/// Internal lifecycle phases tracked by [`LifecycleGuard`] in debug builds.
///
/// The variant ordering mirrors the documented call sequence:
///
/// 1. `Fresh` — newly constructed, or just reset; first input may be
///    supplied.
/// 2. `Streaming` — at least one `transcode` call has been observed.
/// 3. `Finished` — `finish` has been called; the only legal next step is
///    `reset` (which returns to `Fresh`).
#[cfg(debug_assertions)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(crate) enum LifecyclePhase {
    /// Fresh or just-reset engine ready to accept the next logical stream.
    #[default]
    Fresh,
    /// At least one `transcode` call has been observed since the last reset.
    Streaming,
    /// `finish` has been called and the logical stream is closed.
    Finished,
}

/// Debug-only lifecycle guard for transcode engines.
///
/// The guard tracks the current [`LifecyclePhase`] in debug builds and runs
/// `debug_assert!` checks on every public entry point. In release builds it
/// is an empty type, so engines pay no runtime cost.
///
/// Lifecycle rules enforced in debug builds:
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
#[cfg(debug_assertions)]
#[derive(Debug, Default)]
pub(crate) struct LifecycleGuard {
    phase: LifecyclePhase,
}

#[cfg(not(debug_assertions))]
#[derive(Debug, Default)]
pub(crate) struct LifecycleGuard;

impl LifecycleGuard {
    /// Creates a guard in the [`LifecyclePhase::Fresh`] phase.
    ///
    /// # Returns
    ///
    /// Returns a guard ready to observe the first lifecycle event.
    #[inline(always)]
    #[must_use]
    pub(crate) const fn new() -> Self {
        #[cfg(debug_assertions)]
        {
            Self {
                phase: LifecyclePhase::Fresh,
            }
        }
        #[cfg(not(debug_assertions))]
        {
            Self
        }
    }

    /// Records a `reset` event. Always legal; returns the guard to
    /// [`LifecyclePhase::Fresh`].
    #[inline(always)]
    pub(crate) fn on_reset(&mut self) {
        #[cfg(debug_assertions)]
        {
            self.phase = LifecyclePhase::Fresh;
        }
    }

    /// Records a `transcode` entry. In debug builds, asserts the guard is
    /// not in [`LifecyclePhase::Finished`].
    ///
    /// # Panics
    ///
    /// In debug builds, panics when `transcode` is called after `finish`
    /// without an intervening `reset`.
    #[inline(always)]
    pub(crate) fn on_transcode(&mut self) {
        #[cfg(debug_assertions)]
        {
            debug_assert_ne!(
                LifecyclePhase::Finished,
                self.phase,
                "Transcoder::transcode called after finish without an \
                 intervening reset; call reset() to start a new logical \
                 stream",
            );
            if self.phase == LifecyclePhase::Fresh {
                self.phase = LifecyclePhase::Streaming;
            }
        }
    }

    /// Asserts the guard is allowed to enter `finish`. Does not change
    /// state, so callers that fail before completing finish (for example,
    /// capacity checks rejecting the supplied output) can retry without
    /// being marked closed.
    ///
    /// # Panics
    ///
    /// In debug builds, panics when `finish` is called twice without an
    /// intervening `reset`.
    #[inline(always)]
    pub(crate) fn on_finish_attempt(&self) {
        #[cfg(debug_assertions)]
        {
            debug_assert_ne!(
                LifecyclePhase::Finished,
                self.phase,
                "Transcoder::finish called twice without an intervening \
                 reset; the logical stream is already closed",
            );
        }
    }

    /// Commits the `Finished` state after `finish` actually completed. Call
    /// only on the success path.
    #[inline(always)]
    pub(crate) fn on_finish_success(&mut self) {
        #[cfg(debug_assertions)]
        {
            self.phase = LifecyclePhase::Finished;
        }
    }
}

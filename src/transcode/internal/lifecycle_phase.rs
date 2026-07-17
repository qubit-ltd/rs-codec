// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Debug-only lifecycle phase for transcode engines.

/// Internal lifecycle phase tracked by the debug-only lifecycle guard.
///
/// The variant ordering mirrors the documented call sequence:
///
/// 1. `Fresh` — newly constructed, or just reset; first input may be supplied.
/// 2. `Streaming` — at least one `transcode` call has been observed.
/// 3. `Finished` — `finish` has been called; the only legal next step is
///    `reset`, which returns to `Fresh`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(in crate::transcode) enum LifecyclePhase {
    /// Fresh or just-reset engine ready to accept the next logical stream.
    #[default]
    Fresh,
    /// At least one `transcode` call has been observed since the last reset.
    Streaming,
    /// `finish` has been called and the logical stream is closed.
    Finished,
}

// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Lifecycle phase for transcode engines.

/// Internal lifecycle phase tracked by the lifecycle guard.
///
/// The variant ordering mirrors the documented call sequence:
///
/// 1. `Uninitialized` — newly constructed; only `reset` is legal.
/// 2. `Fresh` — just reset; first input may be supplied.
/// 3. `Streaming` — at least one `transcode` call has been observed.
/// 4. `Finished` — `finish` completed; the only legal next step is `reset`.
/// 5. `Poisoned` — reset or finish execution failed after state may have
///    changed; only a successful `reset` can return to `Fresh`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub(in crate::transcode) enum LifecyclePhase {
    /// Newly constructed engine that has not been reset.
    #[default]
    Uninitialized,
    /// Fresh or just-reset engine ready to accept the next logical stream.
    Fresh,
    /// At least one `transcode` call has been observed since the last reset.
    Streaming,
    /// `finish` completed and the logical stream is closed.
    Finished,
    /// A state-mutating lifecycle operation failed partway through.
    Poisoned,
}

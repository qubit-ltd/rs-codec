// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Codec lifecycle phase attached to transcode domain errors.

/// Phase of a codec lifecycle operation.
///
/// The phase is carried by [`crate::TranscodeError::Domain`] so callers can
/// distinguish ordinary value conversion from reset and flush failures without
/// introducing separate engine error enums.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CodecPhase {
    /// Codec reset operation.
    Reset,

    /// Main encode or decode operation.
    Main,

    /// Codec flush operation.
    Flush,
}

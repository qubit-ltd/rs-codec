// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Domain-specific transcode errors with codec phase context.

use thiserror::Error;

use super::codec_phase::CodecPhase;

/// Domain-specific codec, charset, or policy error with transcode context.
///
/// # Type Parameters
///
/// - `E`: Domain error reported by the concrete codec, hook, or facade.
#[derive(Clone, Copy, Debug, Eq, Error, Hash, PartialEq)]
#[error("codec {phase:?} error at input index {input_index:?}: {source}")]
pub struct TranscodeDomainError<E> {
    /// Domain error returned by the codec or policy facade.
    #[source]
    pub source: E,
    /// Codec lifecycle phase where the error occurred.
    pub phase: CodecPhase,
    /// Absolute input index when the phase is associated with an input value.
    pub input_index: Option<usize>,
}

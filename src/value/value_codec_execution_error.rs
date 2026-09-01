// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Type-erased value-codec execution errors.

use std::any::TypeId;

use thiserror::Error;

/// Failure produced by a type-erased value-codec invocation.
#[derive(Debug, Error)]
pub enum ValueCodecExecutionError {
    /// The supplied value has the wrong Rust type.
    #[error("value codec for {expected_type} received incompatible type {actual_type:?}")]
    TypeMismatch {
        /// Expected diagnostic Rust type name.
        expected_type: &'static str,
        /// Actual process-local Rust type identity.
        actual_type: TypeId,
    },
    /// The typed encoder failed.
    #[error("value codec {codec_type} failed to encode: {source}")]
    EncodeFailed {
        /// Diagnostic Rust codec type name.
        codec_type: &'static str,
        /// Typed encoder source error.
        #[source]
        source: Box<dyn std::error::Error>,
    },
    /// The typed decoder failed.
    #[error("value codec {codec_type} failed to decode: {source}")]
    DecodeFailed {
        /// Diagnostic Rust codec type name.
        codec_type: &'static str,
        /// Typed decoder source error.
        #[source]
        source: Box<dyn std::error::Error>,
    },
}

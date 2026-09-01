// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Value-codec registry errors.

use thiserror::Error;

use crate::ValueCodecRegistrationSource;

/// Failure while freezing a value-codec registry.
#[derive(Clone, Debug, Error)]
pub enum ValueCodecRegistryError {
    /// Multiple registrations claim one stable ID.
    #[error("duplicate value codec ID {id} from {sources:?}")]
    DuplicateId {
        /// Conflicting stable ID.
        id: &'static str,
        /// Registration sources in deterministic order.
        sources: Vec<ValueCodecRegistrationSource>,
    },
}

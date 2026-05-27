/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Progress-oriented conversion traits and status types.

#![allow(clippy::module_inception)]

mod transcode_progress;
mod transcode_status;
mod transcoder;

pub use transcode_progress::TranscodeProgress;
pub use transcode_status::TranscodeStatus;
pub use transcoder::Transcoder;

// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

mod async_sequence_model;
mod codec_decode_driver_tests;
mod encode_output_support;
mod wake_driver;

pub(super) use async_sequence_model::InputAction;
pub(super) use async_sequence_model::IoEvent;
pub(super) use async_sequence_model::OutputAction;
pub(super) use async_sequence_model::input_actions;
pub(super) use async_sequence_model::io_events;
pub(super) use async_sequence_model::next;
pub(super) use async_sequence_model::output_actions;
pub(super) use async_sequence_model::payload;
pub(super) use encode_output_support::UnitOutput;
pub(super) use encode_output_support::encode_with;
pub(super) use encode_output_support::finish_with;
pub(super) use encode_output_support::map_codec_error;
pub(super) use encode_output_support::map_error;
pub(super) use encode_output_support::panic_codec_error;
pub(super) use wake_driver::WakeCount;
pub(super) use wake_driver::complete_when_woken;

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub(super) enum PairEncodeError {
    #[error("bad input index")]
    BadInputIndex,
    #[error("bad output index")]
    BadOutputIndex,
    #[error("invalid output index {index} for output length {len}")]
    InvalidOutputIndex { index: usize, len: usize },
    #[error("insufficient output at index {output_index}: required {required}, available {available}")]
    InsufficientOutput {
        output_index: usize,
        required: usize,
        available: usize,
    },
    #[error("capacity overflow")]
    CapacityOverflow,
}

// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared test helpers for transcode engine tests.

mod conversion_fixtures;
mod decode_outcome_tests;
mod encode_outcome_tests;

pub(super) use conversion_fixtures::ChangingFinishBoundDecodeHooks;
pub(super) use conversion_fixtures::CopyConvertEngine;
pub(super) use conversion_fixtures::CountedTargetReplacementHooks;
pub(super) use conversion_fixtures::EngineError;
pub(super) use conversion_fixtures::EofIncompleteDecodeHooks;
pub(super) use conversion_fixtures::EofIncompleteMode;
pub(super) use conversion_fixtures::ErrorPathDecodeFinish;
pub(super) use conversion_fixtures::ErrorPathEncodeMode;
pub(super) use conversion_fixtures::ErrorPathHooks;
pub(super) use conversion_fixtures::ErrorSourceCodec;
pub(super) use conversion_fixtures::FactoryDecodeHooks;
pub(super) use conversion_fixtures::FactoryEncodeHooks;
pub(super) use conversion_fixtures::FinishDecodeHooks;
pub(super) use conversion_fixtures::FinishFailSourceDecodeHooks;
pub(super) use conversion_fixtures::FinishFailingSourceCodec;
pub(super) use conversion_fixtures::FinishHooks;
pub(super) use conversion_fixtures::FinishOverflowEncodeHooks;
pub(super) use conversion_fixtures::FinishOverflowTargetCodec;
pub(super) use conversion_fixtures::FinishValueSourceCodec;
pub(super) use conversion_fixtures::FixedFinishBoundDecodeHooks;
pub(super) use conversion_fixtures::ImpossibleFailureDecodeHooks;
pub(super) use conversion_fixtures::MismatchCapacityEncodeHooks;
pub(super) use conversion_fixtures::MismatchCapacityResetEmittingDecodeHooks;
pub(super) use conversion_fixtures::MismatchCapacityTargetCodec;
pub(super) use conversion_fixtures::OverflowResetSourceCodec;
pub(super) use conversion_fixtures::OverflowResetSourceDecodeHooks;
pub(super) use conversion_fixtures::OverflowResetTargetCodec;
pub(super) use conversion_fixtures::OverflowResetTargetEncodeHooks;
pub(super) use conversion_fixtures::PerBatchOverheadEncodeHooks;
pub(super) use conversion_fixtures::RepairAction;
pub(super) use conversion_fixtures::ResetDependentTargetCodec;
pub(super) use conversion_fixtures::ResetEmittingSourceCodec;
pub(super) use conversion_fixtures::ResetEmittingTargetCodec;
pub(super) use conversion_fixtures::ResetEncodingFailSourceCodec;
pub(super) use conversion_fixtures::ResetEncodingFailSourceDecodeHooks;
pub(super) use conversion_fixtures::ResetFailSourceDecodeHooks;
pub(super) use conversion_fixtures::ResetFailTargetCodec;
pub(super) use conversion_fixtures::ResetFailTargetHooks;
pub(super) use conversion_fixtures::ResetFailingSourceCodec;
pub(super) use conversion_fixtures::ResetObservingDecodeHooks;
pub(super) use conversion_fixtures::ResetSourceDecodeHooks;
pub(super) use conversion_fixtures::ResetTargetHooks;
pub(super) use conversion_fixtures::SOURCE_RESET_SENTINEL;
pub(super) use conversion_fixtures::ShortSourceCodec;
pub(super) use conversion_fixtures::SourceCodec;
pub(super) use conversion_fixtures::StatelessResetFailingSourceCodec;
pub(super) use conversion_fixtures::StatelessResetFailingSourceDecodeHooks;
pub(super) use conversion_fixtures::StatelessResetSourceCodec;
pub(super) use conversion_fixtures::StatelessResetSourceDecodeHooks;
pub(super) use conversion_fixtures::StrictDecodeHooks;
pub(super) use conversion_fixtures::StrictEncodeHooks;
pub(super) use conversion_fixtures::TargetCodec;
pub(super) use conversion_fixtures::TargetResetFailError;
pub(super) use conversion_fixtures::new_batch_finish_engine;
pub(super) use conversion_fixtures::new_copy_engine;
pub(super) use conversion_fixtures::new_error_path_engine;
pub(super) use conversion_fixtures::new_finish_encode_engine;
pub(super) use conversion_fixtures::new_finish_engine;
pub(super) use conversion_fixtures::new_repair_engine;

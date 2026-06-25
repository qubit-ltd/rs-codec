// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Converter error type alias selected by decode and encode hooks.

use crate::{
    Codec,
    TranscodeConvertEngineError,
    TranscodeDecodeEngineError,
    TranscodeDecodeHooks,
    TranscodeEncodeEngineError,
    TranscodeEncodeHooks,
    TranscodeError,
};

/// Converter error type selected by hooks for one target output unit type.
///
/// # Type Parameters
///
/// - `D`: Source codec type.
/// - `E`: Target codec type.
/// - `DH`: Decode hook type.
/// - `EH`: Encode hook type.
pub(in crate::transcode) type ConvertErrorOf<D, E, DH, EH> = TranscodeError<
    TranscodeConvertEngineError<
        TranscodeDecodeEngineError<
            <D as Codec>::DecodeError,
            <DH as TranscodeDecodeHooks<D>>::Error,
        >,
        TranscodeEncodeEngineError<
            <E as Codec>::EncodeError,
            <EH as TranscodeEncodeHooks<E>>::Error,
        >,
    >,
>;

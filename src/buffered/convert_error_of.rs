/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Converter error type selected by hooks for one target output unit type.

use super::buffered_convert_hooks::BufferedConvertHooks;

/// Converter error type selected by hooks for one target output unit type.
pub(super) type ConvertErrorOf<D, E, H, Input, Value, Output> =
    <H as BufferedConvertHooks<D, E, Input, Value>>::Error<Output>;

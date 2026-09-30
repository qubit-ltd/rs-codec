// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Built-in registry entries for Serde JSON value codecs.

use serde_json::Value;

use crate::JsonBytesValueCodec;
use crate::JsonStringValueCodec;
use crate::register_value_bytes_codec;
use crate::register_value_string_codec;

/// Stable registry ID for JSON string value codecs.
pub const JSON_STRING_VALUE_CODEC_ID: &str = "qubit.codec.json.string";

/// Stable registry ID for JSON bytes value codecs.
pub const JSON_BYTES_VALUE_CODEC_ID: &str = "qubit.codec.json.bytes";

register_value_string_codec!(
    id = JSON_STRING_VALUE_CODEC_ID,
    codec = JsonStringValueCodec<Value>,
    value = Value,
);

register_value_bytes_codec!(
    id = JSON_BYTES_VALUE_CODEC_ID,
    codec = JsonBytesValueCodec<Value>,
    value = Value,
);

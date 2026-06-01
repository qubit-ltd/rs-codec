/*******************************************************************************
 *
 *    Copyright (c) 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Source-side finish reader used by the converter finalization path.

use super::{
    buffered_convert_hooks::BufferedConvertHooks,
    buffered_decode_engine::BufferedDecodeEngine,
    buffered_encode_hooks::BufferedEncodeHooks,
    convert_decode_finish_result::ConvertDecodeFinishResult,
    decode_finish_step::DecodeFinishStep,
    source_value_reader::SourceValueReader,
};
use crate::Codec;

/// Source-side finish reader used by the converter finalization path.
pub(super) struct SourceFinishReader<'a, D, E, H, Input, Value>
where
    D: Codec<Value, Input>,
    H: BufferedConvertHooks<D, E, Input, Value>,
    Input: Copy,
{
    /// Source-side reader used for finish hook dispatch.
    source: SourceValueReader<'a, D, E, H, Input, Value>,
}

impl<'a, D, E, H, Input, Value> SourceFinishReader<'a, D, E, H, Input, Value>
where
    D: Codec<Value, Input>,
    H: BufferedConvertHooks<D, E, Input, Value>,
    Input: Copy,
{
    /// Creates a source-side finish reader.
    #[inline(always)]
    pub(super) const fn new(engine: &'a mut BufferedDecodeEngine<D, H::DecodeHooks, Input>, hooks: &'a H) -> Self {
        Self {
            source: SourceValueReader::new(engine, hooks),
        }
    }

    /// Reads the next source-side finish step.
    ///
    /// # Returns
    ///
    /// Returns the decoded final value, completion, or a source finish stop
    /// condition.
    ///
    /// # Errors
    ///
    /// Returns mapped decode errors produced by source-side finish hooks.
    #[inline]
    pub(super) fn read_next<Output>(&mut self) -> ConvertDecodeFinishResult<D, E, H, Input, Value, Output>
    where
        E: Codec<Value, Output>,
        H::EncodeHooks: BufferedEncodeHooks<E, Value, Output, Error = H::EncodeError<Output>>,
        Value: Default,
        Output: Copy,
    {
        let mut decoded: [Value; 1] = core::array::from_fn(|_| Value::default());
        let finish = self.source.finish_one::<Output>(&mut decoded)?;
        Ok(DecodeFinishStep::from_progress(finish, decoded))
    }
}

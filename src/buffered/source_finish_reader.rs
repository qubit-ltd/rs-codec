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
    convert_decode_finish_result::ConvertDecodeFinishResult,
    decode_finish_step::DecodeFinishStep,
    source_value_reader::SourceValueReader,
};
use crate::Codec;

/// Source-side finish reader used by the converter finalization path.
pub(super) struct SourceFinishReader<'a, D, E, H>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    H: BufferedConvertHooks<D, E>,
{
    /// Source-side reader used for finish hook dispatch.
    source: SourceValueReader<'a, D, E, H>,
}

impl<'a, D, E, H> SourceFinishReader<'a, D, E, H>
where
    D: Codec,
    E: Codec<Value = D::Value>,
    H: BufferedConvertHooks<D, E>,
{
    /// Creates a source-side finish reader.
    ///
    /// # Type Parameters
    ///
    /// - `D`: Source codec used by the buffered decode engine.
    /// - `E`: Target codec used by the converter; `E::Value` must equal
    ///   `D::Value`.
    /// - `H`: Converter-level hook aggregator.
    ///
    /// # Parameters
    ///
    /// - `engine`: Mutable reference to the shared source decode engine.
    /// - `hooks`: Converter hooks used to map decode errors.
    ///
    /// # Returns
    ///
    /// Returns a source-side finish reader bound to the provided engine.
    #[inline(always)]
    pub(super) const fn new(engine: &'a mut BufferedDecodeEngine<D, H::DecodeHooks>, hooks: &'a H) -> Self {
        Self {
            source: SourceValueReader::new(engine, hooks),
        }
    }

    /// Reads the next source-side finish step.
    ///
    /// This call always enters finish-mode (`D::Value: Default`) and asks source
    /// decoding to emit at most one trailing value.
    ///
    /// # Parameters
    ///
    /// - This method has no explicit parameters.
    ///
    /// # Returns
    ///
    /// Returns the decoded final value, completion, or a source finish stop
    /// condition.
    ///
    /// # Errors
    ///
    /// Returns mapped decode errors produced by source-side finish hooks.
    ///
    /// # Notes
    ///
    /// This method requires `D::Value: Default` so it can allocate a one-value
    /// scratch slot for the trailing decoded result.
    #[inline]
    pub(super) fn read_next(&mut self) -> ConvertDecodeFinishResult<D, E, H>
    where
        D::Value: Default,
    {
        let mut decoded: [D::Value; 1] = core::array::from_fn(|_| D::Value::default());
        let finish = self.source.finish_one(&mut decoded)?;
        Ok(DecodeFinishStep::from_progress(finish, decoded))
    }
}

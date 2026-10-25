// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Demonstrates stepwise encoding through the buffered I/O bridge.

// io-guide-start
use core::convert::Infallible;
use core::num::NonZeroUsize;
use std::io::Cursor;
use std::io::Error;

use qubit_codec::Codec;
use qubit_codec::CodecTranscodeEncoder;
use qubit_codec::DecodeFailure;
use qubit_codec::TranscodeEncodeOutput;
use qubit_codec::TranscodeStatus;

/// Encodes one fixed-width big-endian value.
struct U16BeCodec;

impl Codec for U16BeCodec {
    type Value = u16;
    type Unit = u8;
    type DecodeError = Infallible;
    type EncodeError = Infallible;

    const MIN_UNITS_PER_VALUE: usize = 2;
    const MAX_ENCODE_UNITS_PER_VALUE: usize = 2;
    const MAX_DECODE_UNITS_PER_VALUE: usize = 2;

    unsafe fn decode(
        &mut self,
        input: &[u8],
        input_index: usize,
    ) -> Result<(u16, NonZeroUsize), DecodeFailure<Infallible>> {
        debug_assert!(input_index + 2 <= input.len());
        Ok((
            u16::from_be_bytes([input[input_index], input[input_index + 1]]),
            NonZeroUsize::new(2).unwrap(),
        ))
    }

    unsafe fn encode(&mut self, value: &u16, out: &mut [u8], index: usize) -> Result<usize, Infallible> {
        debug_assert!(index + 2 <= out.len());
        out[index..index + 2].copy_from_slice(&value.to_be_bytes());
        Ok(2)
    }
}

/// Runs a complete stepwise encode lifecycle and checks the delivered bytes.
fn main() -> Result<(), Error> {
    let values = [0x1234_u16, 0xabcd];
    let mut encoder = CodecTranscodeEncoder::new(U16BeCodec);
    let mut output = TranscodeEncodeOutput::with_capacity(Cursor::new(Vec::<u8>::new()), 2);
    let mut map_error = |_| Error::other("encode error");
    output.reset(&mut encoder, &mut map_error)?;
    let mut offset = 0;
    while offset < values.len() {
        let progress = output.transcode_step(&mut encoder, &mut map_error, &values, offset, values.len() - offset)?;
        offset += progress.read();
        if let TranscodeStatus::NeedOutput { required, .. } = progress.status() {
            output.ensure_spare_capacity(required.get())?;
        }
    }
    output.finish(&mut encoder, &mut map_error)?;
    assert_eq!(output.inner().get_ref(), &[0x12, 0x34, 0xab, 0xcd]);
    Ok(())
}
// io-guide-end

#[cfg(test)]
mod tests {
    /// Reads exactly the marked part; missing markers are test failures.
    fn marked<'a>(text: &'a str, start: &str, end: &str) -> &'a str {
        text.split_once(start)
            .expect("start marker must exist")
            .1
            .split_once(end)
            .expect("end marker must exist")
            .0
            .trim()
    }

    #[test]
    fn test_io_guide_matches_the_compiled_example_in_both_languages() {
        let example = marked(
            include_str!("io_transcode_step.rs"),
            "// io-guide-start\n",
            "\n// io-guide-end",
        );
        for guide in [
            include_str!("../doc/user_guide.md"),
            include_str!("../doc/user_guide.zh_CN.md"),
        ] {
            let snippet = marked(
                guide,
                "<!-- io-guide-start -->\n```rust\n",
                "\n```\n<!-- io-guide-end -->",
            );
            assert_eq!(snippet, example, "the displayed program must be the compiled program");
        }
    }

    #[test]
    fn test_io_guide_program_delivers_the_expected_bytes() {
        super::main().expect("the documented lifecycle must succeed");
    }
}

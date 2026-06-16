use qubit_codec::{Codec, CodecEncodeError, CodecTranscodeEncoder, TranscodeError, Transcoder};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct ResetFailCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("reset failed")]
struct ResetFailError;

unsafe impl Codec for ResetFailCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = ResetFailError;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn can_encode_value(&self, value: &u8) -> bool {
        value.is_multiple_of(2)
    }

    fn max_encode_reset_units(&self) -> usize {
        1
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        Ok((input[index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        output[index] = *value;
        Ok(qubit_codec::nz!(1))
    }

    unsafe fn encode_reset(
        &mut self,
        _output: &mut [u8],
        _index: usize,
    ) -> Result<usize, Self::EncodeError> {
        Err(ResetFailError)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RejectOddCodec;

unsafe impl Codec for RejectOddCodec {
    type Value = u8;
    type Unit = u8;
    type DecodeError = core::convert::Infallible;
    type EncodeError = &'static str;

    fn min_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn max_units_per_value(&self) -> core::num::NonZeroUsize {
        core::num::NonZeroUsize::MIN
    }

    fn can_encode_value(&self, value: &u8) -> bool {
        value.is_multiple_of(2)
    }

    unsafe fn decode(
        &mut self,
        input: &[u8],
        index: usize,
    ) -> Result<(u8, core::num::NonZeroUsize), Self::DecodeError> {
        Ok((input[index], core::num::NonZeroUsize::MIN))
    }

    unsafe fn encode(
        &mut self,
        value: &u8,
        output: &mut [u8],
        index: usize,
    ) -> Result<core::num::NonZeroUsize, Self::EncodeError> {
        debug_assert!(self.can_encode_value(value));
        output[index] = *value;
        Ok(qubit_codec::nz!(1))
    }
}

#[test]
fn test_codec_transcode_encode_hooks_wraps_encode_errors() {
    let mut encoder = CodecTranscodeEncoder::new(RejectOddCodec);
    let mut output = [0_u8; 1];

    let error = encoder
        .transcode(&[7], 0, &mut output, 0)
        .expect_err("strict encode hooks should reject unencodable values");

    assert_eq!(
        TranscodeError::Domain(CodecEncodeError::UnencodableValue { input_index: 0 }),
        error,
    );
}

#[test]
fn test_codec_transcode_encode_hooks_wraps_encode_reset_errors() {
    let mut encoder = CodecTranscodeEncoder::new(ResetFailCodec);
    let mut output = [0_u8; 1];

    let error = encoder
        .reset(&mut output, 0)
        .expect_err("reset errors should be wrapped");

    assert_eq!(
        TranscodeError::Domain(CodecEncodeError::Encode {
            source: ResetFailError,
            input_index: 0,
        }),
        error,
    );
}

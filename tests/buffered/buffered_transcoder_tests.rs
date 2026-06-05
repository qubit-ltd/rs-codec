use qubit_codec::{
    BufferedTranscoder,
    CapacityError,
    FinishError,
    TranscodeProgress,
    TranscodeStatus,
};

#[derive(Default)]
struct CopyBufferedTranscoder;

impl BufferedTranscoder<u8, u8> for CopyBufferedTranscoder {
    type Error = core::convert::Infallible;

    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        let mut read = 0;
        let mut written = 0;
        while input_index + read < input.len()
            && output_index + written < output.len()
        {
            output[output_index + written] = input[input_index + read];
            read += 1;
            written += 1;
        }
        if input_index + read == input.len() {
            Ok(TranscodeProgress::complete(read, written))
        } else {
            let status = TranscodeStatus::NeedOutput {
                output_index: output_index + written,
                additional: super::nz(1),
                available: output.len().saturating_sub(output_index + written),
            };
            Ok(TranscodeProgress::new(status, read, written))
        }
    }
}

#[derive(Default)]
struct FinishingBufferedTranscoder {
    suffix_index: usize,
}

impl BufferedTranscoder<u8, u8> for FinishingBufferedTranscoder {
    type Error = core::convert::Infallible;

    fn max_output_len(&self, input_len: usize) -> Result<usize, CapacityError> {
        Ok(input_len)
    }

    fn max_finish_output_len(&self) -> Result<usize, CapacityError> {
        Ok(2 - self.suffix_index)
    }

    fn reset(&mut self) {
        self.suffix_index = 0;
    }

    fn transcode(
        &mut self,
        input: &[u8],
        input_index: usize,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<TranscodeProgress, Self::Error> {
        CopyBufferedTranscoder.transcode(
            input,
            input_index,
            output,
            output_index,
        )
    }

    fn finish(
        &mut self,
        output: &mut [u8],
        output_index: usize,
    ) -> Result<usize, FinishError<Self::Error>> {
        let suffix = *b"!\n";
        let required = suffix.len() - self.suffix_index;
        FinishError::ensure_output_capacity(
            output.len(),
            output_index,
            required,
        )?;
        let mut written = 0;
        while self.suffix_index < suffix.len() {
            output[output_index + written] = suffix[self.suffix_index];
            self.suffix_index += 1;
            written += 1;
        }
        Ok(written)
    }
}

#[test]
fn test_buffered_transcoder_contract_uses_absolute_indices_and_relative_progress()
 {
    let mut transcoder = CopyBufferedTranscoder;
    let mut output = [0_u8; 4];

    let progress = transcoder
        .transcode(b"abc", 1, &mut output, 2)
        .expect("infallible copy");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(2, progress.read());
    assert_eq!(2, progress.written());
    assert_eq!([0, 0, b'b', b'c'], output);
}

#[test]
fn test_buffered_transcoder_default_reset_and_finish_are_noops() {
    let mut transcoder = CopyBufferedTranscoder;
    let mut output = [0_u8; 1];

    assert_eq!(Ok(3), transcoder.max_output_len(3));
    assert_eq!(Ok(0), transcoder.max_finish_output_len());

    BufferedTranscoder::<u8, u8>::reset(&mut transcoder);
    let written = transcoder.finish(&mut output, 0).expect("finish is noop");

    assert_eq!(0, written);
    assert_eq!([0], output);
}

#[test]
fn test_buffered_transcoder_default_finish_reports_output_index_beyond_buffer()
{
    let mut transcoder = CopyBufferedTranscoder;
    let mut output = [];

    let error = transcoder
        .finish(&mut output, 1)
        .expect_err("out-of-range finish output index should be rejected");

    assert_eq!(FinishError::InvalidOutputIndex { index: 1, len: 0 }, error);
}

#[test]
fn test_buffered_transcoder_finish_requires_one_shot_output_capacity() {
    let mut transcoder = FinishingBufferedTranscoder::default();
    let mut output = [0_u8; 1];

    assert_eq!(Ok(2), transcoder.max_finish_output_len());

    let error = transcoder
        .finish(&mut output, 0)
        .expect_err("finish should reject partial output capacity");

    assert_eq!(
        FinishError::InsufficientOutput {
            output_index: 0,
            required: 2,
            available: 1,
        },
        error,
    );
    assert_eq!([0], output);
    assert_eq!(Ok(2), transcoder.max_finish_output_len());

    let mut output = [0_u8; 2];
    let written = transcoder.finish(&mut output, 0).expect(
        "finish should write the whole suffix once capacity is available",
    );

    assert_eq!(2, written);
    assert_eq!(*b"!\n", output);
    assert_eq!(Ok(0), transcoder.max_finish_output_len());

    transcoder.reset();
    assert_eq!(Ok(2), transcoder.max_finish_output_len());
}

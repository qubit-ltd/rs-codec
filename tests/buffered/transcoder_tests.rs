use qubit_codec::{
    CapacityError,
    TranscodeProgress,
    TranscodeStatus,
    Transcoder,
};

#[derive(Default)]
struct CopyTranscoder;

impl Transcoder<u8, u8> for CopyTranscoder {
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
        while input_index + read < input.len() && output_index + written < output.len() {
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
struct FinishingTranscoder {
    suffix_index: usize,
}

impl Transcoder<u8, u8> for FinishingTranscoder {
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
        CopyTranscoder.transcode(input, input_index, output, output_index)
    }

    fn finish(&mut self, output: &mut [u8], output_index: usize) -> Result<TranscodeProgress, Self::Error> {
        let suffix = *b"!\n";
        let mut written = 0;
        while self.suffix_index < suffix.len() {
            if output_index + written == output.len() {
                let status = TranscodeStatus::NeedOutput {
                    output_index: output_index + written,
                    additional: super::nz(suffix.len() - self.suffix_index),
                    available: 0,
                };
                return Ok(TranscodeProgress::new(status, 0, written));
            }
            output[output_index + written] = suffix[self.suffix_index];
            self.suffix_index += 1;
            written += 1;
        }
        Ok(TranscodeProgress::complete(0, written))
    }
}

#[test]
fn test_transcoder_contract_uses_absolute_indices_and_relative_progress() {
    let mut transcoder = CopyTranscoder;
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
fn test_transcoder_default_reset_and_finish_are_noops() {
    let mut transcoder = CopyTranscoder;
    let mut output = [0_u8; 1];

    assert_eq!(Ok(3), transcoder.max_output_len(3));
    assert_eq!(Ok(0), transcoder.max_finish_output_len());

    Transcoder::<u8, u8>::reset(&mut transcoder);
    let progress = transcoder.finish(&mut output, 0).expect("finish is noop");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
    assert_eq!([0], output);
}

#[test]
fn test_transcoder_default_finish_reports_output_index_beyond_buffer() {
    let mut transcoder = CopyTranscoder;
    let mut output = [];

    let progress = transcoder
        .finish(&mut output, 1)
        .expect("out-of-range finish output index should request capacity");

    assert_eq!(
        TranscodeStatus::NeedOutput {
            output_index: 1,
            additional: super::nz(1),
            available: 0,
        },
        progress.status(),
    );
    assert_eq!(0, progress.read());
    assert_eq!(0, progress.written());
}

#[test]
fn test_transcoder_finish_can_report_bounded_pending_output() {
    let mut transcoder = FinishingTranscoder::default();
    let mut output = [0_u8; 1];

    assert_eq!(Ok(2), transcoder.max_finish_output_len());

    let progress = transcoder
        .finish(&mut output, 0)
        .expect("finish writes suffix until output fills");

    assert!(matches!(progress.status(), TranscodeStatus::NeedOutput { .. }));
    assert_eq!(0, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([b'!'], output);
    assert_eq!(Ok(1), transcoder.max_finish_output_len());

    let progress = transcoder
        .finish(&mut output, 0)
        .expect("second finish call completes suffix");

    assert_eq!(TranscodeStatus::Complete, progress.status());
    assert_eq!(0, progress.read());
    assert_eq!(1, progress.written());
    assert_eq!([b'\n'], output);
    assert_eq!(Ok(0), transcoder.max_finish_output_len());

    transcoder.reset();
    assert_eq!(Ok(2), transcoder.max_finish_output_len());
}

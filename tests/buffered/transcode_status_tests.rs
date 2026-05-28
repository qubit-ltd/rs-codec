use qubit_codec::TranscodeStatus;

#[test]
fn test_transcoder_status_variants_are_distinct() {
    assert_ne!(
        TranscodeStatus::Complete,
        TranscodeStatus::NeedInput {
            input_index: 0,
            required: 0,
            available: 0
        }
    );
    assert_ne!(
        TranscodeStatus::NeedInput {
            input_index: 0,
            required: 0,
            available: 0
        },
        TranscodeStatus::NeedOutput {
            output_index: 0,
            required: 0,
            available: 0,
        }
    );
}

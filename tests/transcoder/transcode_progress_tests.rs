use qubit_codec::{
    TranscodeProgress,
    TranscodeStatus,
};

#[test]
fn test_transcoder_progress_exposes_status_and_counts() {
    let complete = TranscodeProgress::complete(2, 3);
    assert_eq!(TranscodeStatus::Complete, complete.status());
    assert_eq!(2, complete.read());
    assert_eq!(3, complete.written());
    assert_eq!(0, complete.required());
    assert_eq!(None, complete.index());
    assert_eq!(0, complete.available());

    let status = TranscodeStatus::NeedInput {
        input_index: 0,
        required: 0,
        available: 0,
    };
    assert!(matches!(
        TranscodeProgress::new(status, 1, 1).status(),
        TranscodeStatus::NeedInput { .. },
    ));
    let status = TranscodeStatus::NeedInput {
        input_index: 4,
        required: 3,
        available: 1,
    };
    let need_input = TranscodeProgress::new(status, 1, 2);
    assert_eq!(3, need_input.required());
    assert_eq!(Some(4), need_input.index());
    assert_eq!(1, need_input.available());

    let status = TranscodeStatus::NeedOutput {
        output_index: 0,
        required: 0,
        available: 0,
    };
    assert!(matches!(
        TranscodeProgress::new(status, 1, 0).status(),
        TranscodeStatus::NeedOutput { .. },
    ));
    let status = TranscodeStatus::NeedOutput {
        output_index: 7,
        required: 8,
        available: 9,
    };
    let need_output = TranscodeProgress::new(status, 5, 6);
    assert_eq!(8, need_output.required());
    assert_eq!(Some(7), need_output.index());
    assert_eq!(9, need_output.available());
}

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
    assert_eq!(0, complete.additional());
    assert_eq!(None, complete.index());
    assert_eq!(0, complete.available());

    let status = TranscodeStatus::NeedInput {
        input_index: 0,
        additional: super::nz(1),
        available: 0,
    };
    assert!(matches!(
        TranscodeProgress::new(status, 1, 1).status(),
        TranscodeStatus::NeedInput { .. },
    ));
    let status = TranscodeStatus::NeedInput {
        input_index: 4,
        additional: super::nz(3),
        available: 1,
    };
    let need_input = TranscodeProgress::new(status, 1, 2);
    assert_eq!(3, need_input.additional());
    assert_eq!(Some(4), need_input.index());
    assert_eq!(1, need_input.available());

    let status = TranscodeStatus::NeedOutput {
        output_index: 0,
        additional: super::nz(1),
        available: 0,
    };
    assert!(matches!(
        TranscodeProgress::new(status, 1, 0).status(),
        TranscodeStatus::NeedOutput { .. },
    ));
    let status = TranscodeStatus::NeedOutput {
        output_index: 7,
        additional: super::nz(8),
        available: 9,
    };
    let need_output = TranscodeProgress::new(status, 5, 6);
    assert_eq!(8, need_output.additional());
    assert_eq!(Some(7), need_output.index());
    assert_eq!(9, need_output.available());
}

#[test]
fn test_transcoder_progress_constructors_create_expected_progress() {
    let need_input = TranscodeProgress::need_input(4, super::nz(2), 1, 5, 6);
    assert_eq!(TranscodeStatus::need_input(4, super::nz(2), 1), need_input.status());
    assert_eq!(5, need_input.read());
    assert_eq!(6, need_input.written());

    let need_output = TranscodeProgress::need_output(7, super::nz(3), 0, 8, 9);
    assert_eq!(TranscodeStatus::need_output(7, super::nz(3), 0), need_output.status());
    assert_eq!(8, need_output.read());
    assert_eq!(9, need_output.written());
}

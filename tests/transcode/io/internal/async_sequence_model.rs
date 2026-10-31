// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

/// Advances the deterministic xorshift generator used by sequence tests.
pub fn next(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

/// Generates a bounded payload reproducibly from a seed.
pub fn payload(seed: u64) -> Vec<u8> {
    let mut state = seed + 1;
    let len = (next(&mut state) % 49) as usize;
    (0..len).map(|_| next(&mut state) as u8).collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IoEvent {
    ReadyShort(usize),
    PendingThenWake,
    FailOnce,
    Ready,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputAction {
    Reset,
    Transcode(usize),
    Drain,
    Flush,
    CancelDrain,
    RecoverWrite,
    Finish,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputAction {
    Reset,
    Refill,
    Transcode,
    CancelRead,
    RecoverRead,
    Eof,
    EofStep,
    Finish,
}

/// Generates a bounded legal output action schedule reproducibly from a seed.
pub fn output_actions(seed: u64, payload_len: usize) -> Vec<OutputAction> {
    let mut state = seed + 1;
    let mut actions = vec![OutputAction::Reset];
    if matches!(io_events(seed).1, IoEvent::PendingThenWake) {
        actions.extend([OutputAction::CancelDrain, OutputAction::Drain]);
    }
    if seed.is_multiple_of(13) {
        actions.extend([OutputAction::Drain, OutputAction::RecoverWrite]);
    }
    let mut planned = 0;
    while planned < payload_len {
        match next(&mut state) % 12 {
            0 => actions.push(OutputAction::Reset),
            1 => actions.push(OutputAction::Drain),
            2 => actions.push(OutputAction::Flush),
            3 if planned > 0 => actions.push(OutputAction::Drain),
            _ => {}
        }
        let amount = (1 + (next(&mut state) % 4) as usize).min(payload_len - planned);
        actions.push(OutputAction::Transcode(amount));
        planned += amount;
        assert!(
            actions.len() <= 128,
            "seed={seed}, action={}, poll=0: output action budget exceeded",
            actions.len()
        );
    }
    actions.push(OutputAction::Finish);
    actions.push(OutputAction::Flush);
    assert!(
        actions.len() <= 128,
        "seed={seed}, action={}, poll=0: output action budget exceeded",
        actions.len()
    );
    actions
}

/// Generates a bounded legal input action schedule for a capacity and read
/// chunk.
pub fn input_actions(seed: u64, payload_len: usize, capacity: usize, chunk: usize) -> Vec<InputAction> {
    let mut state = seed + 1;
    let mut actions = vec![InputAction::Reset];
    if payload_len == 0 {
        if seed.is_multiple_of(13) {
            actions.extend([InputAction::Refill, InputAction::RecoverRead]);
        }
        actions.extend([InputAction::Eof, InputAction::EofStep, InputAction::Finish]);
        return actions;
    }
    let step = capacity.min(chunk).max(1);
    let groups = payload_len.div_ceil(step).max(1);
    if matches!(io_events(seed).0, IoEvent::PendingThenWake) && next(&mut state).is_multiple_of(2) {
        actions.push(InputAction::CancelRead);
    }
    actions.push(InputAction::Refill);
    if seed.is_multiple_of(13) {
        actions.push(InputAction::RecoverRead);
    }
    for group in 0..groups {
        actions.push(InputAction::Transcode);
        if group + 1 < groups {
            actions.push(InputAction::Refill);
        }
    }
    actions.extend([InputAction::Eof, InputAction::EofStep, InputAction::Finish]);
    assert!(
        actions.len() <= 128,
        "seed={seed}, action={}, poll=0: input action budget exceeded",
        actions.len()
    );
    actions
}

/// Chooses deterministic I/O events without using process randomness.
pub fn io_events(seed: u64) -> (IoEvent, IoEvent) {
    let mut state = seed + 1;
    let write = if seed.is_multiple_of(13) {
        IoEvent::FailOnce
    } else {
        match next(&mut state) % 4 {
            0 => IoEvent::ReadyShort(1 + (next(&mut state) % 4) as usize),
            1 => IoEvent::PendingThenWake,
            _ => IoEvent::Ready,
        }
    };
    let read = if seed.is_multiple_of(13) {
        IoEvent::FailOnce
    } else {
        match next(&mut state) % 4 {
            0 => IoEvent::ReadyShort(1 + (next(&mut state) % 4) as usize),
            1 => IoEvent::PendingThenWake,
            _ => IoEvent::Ready,
        }
    };
    (read, write)
}

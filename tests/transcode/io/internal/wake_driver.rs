// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::task::Context;
use std::task::Poll;
use std::task::Wake;
use std::task::Waker;

/// Counts notifications sent through the current task's waker.
#[derive(Default)]
pub(crate) struct WakeCount(pub(crate) AtomicUsize);

impl Wake for WakeCount {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

/// Polls a fixture future until it completes, enforcing wake-before-repoll.
///
/// Panics if a pending poll does not notify the waker or if the future exceeds
/// the bounded number of polls.
pub(crate) fn complete_when_woken<F: Future>(future: F) -> F::Output {
    let notifications = Arc::new(WakeCount::default());
    let waker = Waker::from(Arc::clone(&notifications));
    let mut cx = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    for _ in 0..128 {
        let before = notifications.0.load(Ordering::SeqCst);
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(value) => return value,
            Poll::Pending => assert!(
                notifications.0.load(Ordering::SeqCst) > before,
                "pending fixture must notify the current waker before repoll"
            ),
        }
    }
    panic!("test future did not complete within the bounded driver");
}

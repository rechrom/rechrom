// Translation status: complete
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/base/unique_notifier.h:1-57
// Source: cc/base/unique_notifier.cc:1-58

use crate::adapter::{
    task_runner::{RepeatingClosure, SequencedTaskRunner, TaskLocation},
    time::TimeDelta,
};
use std::sync::{Arc, Mutex};

// Source: cc/base/unique_notifier.h:18-53. Arc/Weak callback ownership is the
// Rust equivalent of the source WeakPtrFactory lifetime contract.
pub struct UniqueNotifier {
    task_runner_: Arc<dyn SequencedTaskRunner>,
    closure_: RepeatingClosure,
    notification_pending_: Mutex<bool>,
}

impl UniqueNotifier {
    // Source: cc/base/unique_notifier.h:21-29
    // Source: cc/base/unique_notifier.cc:16-22
    pub fn new(task_runner: Arc<dyn SequencedTaskRunner>, closure: RepeatingClosure) -> Arc<Self> {
        Arc::new(Self {
            task_runner_: task_runner,
            closure_: closure,
            notification_pending_: Mutex::new(false),
        })
    }

    // Source: cc/base/unique_notifier.h:37-38
    // Source: cc/base/unique_notifier.cc:24-28
    pub fn Cancel(&self) {
        debug_assert!(self.task_runner_.CalledOnValidThread());
        *self.notification_pending_.lock().unwrap() = false;
    }

    // Source: cc/base/unique_notifier.h:33-35
    // Source: cc/base/unique_notifier.cc:30-42
    pub fn Schedule(self: &Arc<Self>) {
        debug_assert!(self.task_runner_.CalledOnValidThread());
        let mut notification_pending = self.notification_pending_.lock().unwrap();
        if *notification_pending {
            return;
        }
        let notifier = Arc::downgrade(self);
        self.task_runner_.PostDelayedTask(
            TaskLocation {
                function: "UniqueNotifier::Schedule",
                file: "cc/base/unique_notifier.cc",
                line: 36,
            },
            Box::new(move || {
                if let Some(notifier) = notifier.upgrade() {
                    notifier.Notify();
                }
            }),
            TimeDelta::default(),
        );
        *notification_pending = true;
    }

    // Source: cc/base/unique_notifier.h:41
    // Source: cc/base/unique_notifier.cc:44-56
    fn Notify(&self) {
        debug_assert!(self.task_runner_.CalledOnValidThread());
        {
            let mut notification_pending = self.notification_pending_.lock().unwrap();
            if !*notification_pending {
                return;
            }
            *notification_pending = false;
        }
        (self.closure_)();
    }
}

// Source: cc/base/unique_notifier.h:5-17,31,43-55,57;
// Source: cc/base/unique_notifier.cc:5-15,58.
// Includes, namespaces, deleted copy operations, default destruction, and the
// weak callback factory map to Rust imports and Arc/Weak ownership.

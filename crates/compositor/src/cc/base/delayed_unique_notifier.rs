// Translation status: complete
// Copyright 2014 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/base/delayed_unique_notifier.h:1-75
// Source: cc/base/delayed_unique_notifier.cc:1-97
use crate::adapter::{
    task_runner::{RepeatingClosure, SequencedTaskRunner, TaskLocation},
    time::{TimeDelta, TimeTicks},
};
use std::sync::{Arc, Mutex, Weak};

// Source: cc/base/delayed_unique_notifier.h:21-71
// The Rust owner is pinned logically by Arc. Weak callbacks hold no ownership
// of the notifier, preserving WeakPtrFactory's destruction/cancellation lifetime.
pub struct DelayedUniqueNotifier {
    // Source: cc/base/delayed_unique_notifier.h:63-65
    task_runner_: Arc<dyn SequencedTaskRunner>,
    closure_: RepeatingClosure,
    delay_: TimeDelta,
    state_: Mutex<NotifierState>,
}
struct NotifierState {
    // Source: cc/base/delayed_unique_notifier.h:67-68
    next_notification_time_: TimeTicks,
    notification_pending_: bool,
    // Source: cc/base/delayed_unique_notifier.h:70
    // A weak lifetime token implements the dependency's InvalidateWeakPtrs
    // operation without retaining callbacks or relying on wrapping generations.
    weak_ptr_factory_: Option<Arc<()>>,
}
impl DelayedUniqueNotifier {
    // Source: cc/base/delayed_unique_notifier.h:23-34
    // Source: cc/base/delayed_unique_notifier.cc:16-27
    pub fn new(
        task_runner: Arc<dyn SequencedTaskRunner>,
        closure: RepeatingClosure,
        delay: TimeDelta,
    ) -> Arc<Self> {
        debug_assert!(!delay.is_zero());
        Arc::new(Self {
            task_runner_: task_runner,
            closure_: closure,
            delay_: delay,
            state_: Mutex::new(NotifierState {
                next_notification_time_: TimeTicks::default(),
                notification_pending_: false,
                weak_ptr_factory_: Some(Arc::new(())),
            }),
        })
    }

    // Source: cc/base/delayed_unique_notifier.h:36-41
    // Source: cc/base/delayed_unique_notifier.cc:29-43
    pub fn Schedule(self: &Arc<Self>) {
        debug_assert!(self.task_runner_.CalledOnValidThread());
        let mut state = self.state_.lock().unwrap();
        if state.notification_pending_ {
            state.next_notification_time_ = self.Now() + self.delay_;
            return;
        }
        state.next_notification_time_ = self.Now() + self.delay_;
        self.PostNotifyTask(
            &state,
            self.delay_,
            TaskLocation {
                function: "DelayedUniqueNotifier::Schedule",
                file: "cc/base/delayed_unique_notifier.cc",
                line: 37,
            },
        );
        state.notification_pending_ = true;
    }

    // Source: cc/base/delayed_unique_notifier.h:43-44
    // Source: cc/base/delayed_unique_notifier.cc:45-48
    pub fn Cancel(&self) {
        debug_assert!(self.task_runner_.CalledOnValidThread());
        self.state_.lock().unwrap().next_notification_time_ = TimeTicks::default();
    }

    // Source: cc/base/delayed_unique_notifier.h:46-49
    // Source: cc/base/delayed_unique_notifier.cc:50-59
    pub fn Shutdown(&self) {
        debug_assert!(self.task_runner_.CalledOnValidThread());
        let mut state = self.state_.lock().unwrap();
        state.weak_ptr_factory_ = None;
        state.notification_pending_ = true;
    }

    // Source: cc/base/delayed_unique_notifier.h:51-52
    // Source: cc/base/delayed_unique_notifier.cc:61-64
    pub fn HasPendingNotification(&self) -> bool {
        debug_assert!(self.task_runner_.CalledOnValidThread());
        let state = self.state_.lock().unwrap();
        state.notification_pending_ && !state.next_notification_time_.is_null()
    }

    // Source: cc/base/delayed_unique_notifier.h:54-56
    // Source: cc/base/delayed_unique_notifier.cc:66-68
    pub fn Now(&self) -> TimeTicks {
        self.task_runner_.Now()
    }

    // Source: cc/base/delayed_unique_notifier.h:59
    // Source: cc/base/delayed_unique_notifier.cc:70-95
    fn NotifyIfTime(self: &Arc<Self>) {
        debug_assert!(self.task_runner_.CalledOnValidThread());
        let mut state = self.state_.lock().unwrap();
        if state.next_notification_time_.is_null() {
            state.notification_pending_ = false;
            return;
        }
        let now = self.Now();
        if state.next_notification_time_ > now {
            self.PostNotifyTask(
                &state,
                state.next_notification_time_ - now,
                TaskLocation {
                    function: "DelayedUniqueNotifier::NotifyIfTime",
                    file: "cc/base/delayed_unique_notifier.cc",
                    line: 82,
                },
            );
            return;
        }
        state.notification_pending_ = false;
        // The source clears pending before invoking the closure. Releasing the
        // Rust state borrow here permits the same synchronous rescheduling.
        drop(state);
        (self.closure_)();
    }

    // Rust implementation of the source's base::BindOnce + WeakPtrFactory
    // dependency at cc/base/delayed_unique_notifier.cc:37-41,82-86.
    fn PostNotifyTask(
        self: &Arc<Self>,
        state: &NotifierState,
        delay: TimeDelta,
        location: TaskLocation,
    ) {
        let weak_notifier = Arc::downgrade(self);
        let weak_lifetime = state
            .weak_ptr_factory_
            .as_ref()
            .map(Arc::downgrade)
            .unwrap_or_else(Weak::new);
        self.task_runner_.PostDelayedTask(
            location,
            Box::new(move || {
                if weak_lifetime.upgrade().is_none() {
                    return;
                }
                if let Some(notifier) = weak_notifier.upgrade() {
                    notifier.NotifyIfTime();
                }
            }),
            delay,
        );
    }
}

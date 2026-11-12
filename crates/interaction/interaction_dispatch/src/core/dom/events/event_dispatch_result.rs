// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: third_party/blink/renderer/core/dom/events/event_dispatch_result.h:1-31

// Source: third_party/blink/renderer/core/dom/events/event_dispatch_result.h:10-27
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DispatchEventResult {
    kNotCanceled,
    kCanceledByEventHandler,
    kCanceledByDefaultEventHandler,
    kCanceledBeforeDispatch,
}

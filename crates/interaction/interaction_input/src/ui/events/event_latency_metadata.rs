// Copyright 2023 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the Chromium LICENSE file.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
use crate::base::time::time::TimeTicks;

// Source: ui/events/event_latency_metadata.h:12-26
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EventLatencyMetadata {
    // Source: ui/events/event_latency_metadata.h:16-17
    pub arrived_in_browser_main_timestamp: TimeTicks,
    // Source: ui/events/event_latency_metadata.h:19-22
    pub scrolls_blocking_touch_dispatched_to_renderer: TimeTicks,
    // Source: ui/events/event_latency_metadata.h:24-25
    pub dispatched_to_renderer: TimeTicks,
}

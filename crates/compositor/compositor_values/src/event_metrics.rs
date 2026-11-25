// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/metrics/event_metrics.h (class EventMetrics). The main
// LayerTreeHost queues them (CommitState::event_metrics, commit_state.h) and
// the impl frame reporter consumes them; events_metrics_manager.h:38-39
// moves the owner (nullptr is None).

/// The native EventMetrics owner. Its contents belong to the metrics owner;
/// cc only moves it through the commit.
pub trait EventMetricsBackend: Send {}
pub type EventMetrics = Box<dyn EventMetricsBackend>;

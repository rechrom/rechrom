// Translation status: complete
// Copyright 2016 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/raster/task_category.h:1-36

// Source: cc/raster/task_category.h:12-31
// Chromium stores these values in TaskGraph::Node::category as a uint16_t.
#[repr(u16)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaskCategory {
    TASK_CATEGORY_NONCONCURRENT_FOREGROUND = 0,
    TASK_CATEGORY_FOREGROUND = 1,
    TASK_CATEGORY_BACKGROUND = 2,
    TASK_CATEGORY_BACKGROUND_WITH_NORMAL_THREAD_PRIORITY = 3,
}

// Source: cc/raster/task_category.h:32-32
pub const LAST_TASK_CATEGORY: TaskCategory =
    TaskCategory::TASK_CATEGORY_BACKGROUND_WITH_NORMAL_THREAD_PRIORITY;

// Source: cc/raster/task_category.h:34-36. Namespace and include-guard closure
// are represented by this Rust module and one-time module inclusion.

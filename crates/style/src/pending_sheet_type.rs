// Copyright 2022 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/pending_sheet_type.h:14-28

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PendingSheetType {
    // Not a pending sheet, hasn't started or already finished.
    kNone,
    // Pending but does not block anything.
    kNonBlocking,
    // Dynamically inserted render-blocking but not script-blocking sheet.
    kDynamicRenderBlocking,
    // Parser-inserted sheet that blocks scripts, and may block rendering or
    // the parser depending on where it appears.
    kBlocking,
}

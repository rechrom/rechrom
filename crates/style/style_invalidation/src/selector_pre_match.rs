// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/invalidation/selector_pre_match.h:10-12

// Describes the result of a pre-match operation in which the components of a
// selector are scanned to determine whether it can ever match an element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum SelectorPreMatch {
    kNeverMatches,
    kMayMatch,
}

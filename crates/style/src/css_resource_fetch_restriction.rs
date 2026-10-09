// Copyright 2019 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/css_resource_fetch_restriction.h:9-12

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceFetchRestriction {
    kNone,
    kOnlyDataUrls,
}

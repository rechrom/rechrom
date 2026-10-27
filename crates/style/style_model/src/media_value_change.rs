// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/media_value_change.h:9-17

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaValueChange {
    // Viewport or device size changed. width/height/device-width/device-height.
    kSize,
    // dv* unit evaluation changed.
    kDynamicViewport,
    // Any other value which affect media query evaluations changed.
    kOther,
}

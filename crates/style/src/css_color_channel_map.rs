// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/css_color_channel_map.h:13-17

use foundation::CSSValueID;
use std::collections::HashMap;

// Used in channel keyword substitutions for relative color syntax. A missing
// channel value means the base color was unknown at parse time.
pub type CSSColorChannelMap = HashMap<CSSValueID, Option<f64>>;

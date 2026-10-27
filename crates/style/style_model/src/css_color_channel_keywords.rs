// Copyright 2024 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/css_color_channel_keywords.h:11-16
// cpp: third_party/blink/renderer/core/css/css_color_channel_keywords.cc:12-76

use foundation::{CSSValueID, ColorChannelKeyword};

pub fn CSSValueIDToColorChannelKeyword(value: CSSValueID) -> ColorChannelKeyword {
    match value {
        CSSValueID::kA => ColorChannelKeyword::kA,
        CSSValueID::kB => ColorChannelKeyword::kB,
        CSSValueID::kC => ColorChannelKeyword::kC,
        CSSValueID::kG => ColorChannelKeyword::kG,
        CSSValueID::kH => ColorChannelKeyword::kH,
        CSSValueID::kL => ColorChannelKeyword::kL,
        CSSValueID::kR => ColorChannelKeyword::kR,
        CSSValueID::kS => ColorChannelKeyword::kS,
        CSSValueID::kW => ColorChannelKeyword::kW,
        CSSValueID::kX => ColorChannelKeyword::kX,
        CSSValueID::kY => ColorChannelKeyword::kY,
        CSSValueID::kZ => ColorChannelKeyword::kZ,
        CSSValueID::kAlpha => ColorChannelKeyword::kAlpha,
        _ => unreachable!("CSS value is not a color channel keyword"),
    }
}

pub fn ColorChannelKeywordToCSSValueID(keyword: ColorChannelKeyword) -> CSSValueID {
    match keyword {
        ColorChannelKeyword::kA => CSSValueID::kA,
        ColorChannelKeyword::kB => CSSValueID::kB,
        ColorChannelKeyword::kC => CSSValueID::kC,
        ColorChannelKeyword::kG => CSSValueID::kG,
        ColorChannelKeyword::kH => CSSValueID::kH,
        ColorChannelKeyword::kL => CSSValueID::kL,
        ColorChannelKeyword::kR => CSSValueID::kR,
        ColorChannelKeyword::kS => CSSValueID::kS,
        ColorChannelKeyword::kW => CSSValueID::kW,
        ColorChannelKeyword::kX => CSSValueID::kX,
        ColorChannelKeyword::kY => CSSValueID::kY,
        ColorChannelKeyword::kZ => CSSValueID::kZ,
        ColorChannelKeyword::kAlpha => CSSValueID::kAlpha,
    }
}

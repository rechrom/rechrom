// Copyright 2022 The Chromium Authors. BSD-style license; see Chromium LICENSE.
// cpp: third_party/blink/renderer/core/css/css_anchor_query_enums.h:12-45
// These exact value types already live at the foundation dependency boundary.
// Re-exporting preserves their identity for AnchorQuery and CSS length users.
pub use foundation::style_values::css::css_anchor_query_enums::{
    kCSSAnchorQueryTypesAll, kCSSAnchorQueryTypesNone, CSSAnchorQueryType, CSSAnchorQueryTypes,
    CSSAnchorSizeValue, CSSAnchorValue,
};

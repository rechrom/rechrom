// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// cpp: third_party/blink/renderer/core/css/parser/at_rule_names.json5:5-139

// This file specifies names and metadata for the left-hand-sides of
// declarations that can appear inside an @rule (e.g. @font-face).

// cpp: third_party/blink/renderer/core/css/parser/at_rule_names.json5:9-14
pub const ALIAS_VALID_TYPE: &str = "str";
pub const ALIAS_DEFAULT: &str = "";

// cpp: third_party/blink/renderer/core/css/parser/at_rule_names.json5:16-138
// Each tuple is the descriptor name and its alias (including the default).
pub const AT_RULE_NAMES: [(&str, &str); 40] = [
    ("additive-symbols", ""),
    ("ascent-override", ""),
    ("base-palette", ""),
    ("base-url", ""),
    ("descent-override", ""),
    ("fallback", ""),
    ("font-display", ""),
    ("font-family", ""),
    ("font-feature-settings", "-webkit-font-feature-settings"),
    ("font-stretch", ""),
    ("font-style", ""),
    ("font-variant", ""),
    ("font-variation-settings", ""),
    ("font-weight", ""),
    ("hash", ""),
    ("hostname", ""),
    ("inherits", ""),
    ("initial-value", ""),
    ("line-gap-override", ""),
    ("navigation", ""),
    ("negative", ""),
    ("override-colors", ""),
    ("pad", ""),
    ("pathname", ""),
    ("pattern", ""),
    ("port", ""),
    ("prefix", ""),
    ("protocol", ""),
    ("range", ""),
    ("result", ""),
    ("search", ""),
    ("size-adjust", ""),
    ("speak-as", ""),
    ("src", ""),
    ("suffix", ""),
    ("symbols", ""),
    ("syntax", ""),
    ("system", ""),
    ("types", ""),
    ("unicode-range", ""),
];

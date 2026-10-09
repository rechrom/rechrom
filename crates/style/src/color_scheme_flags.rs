// Copyright 2022 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/color_scheme_flags.h:9-20

// A set of flags extracted from a computed list of color-schemes. Contains one
// flag for each known color-scheme, and a flag for the 'only' keyword.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ColorSchemeFlag {
    kNormal = 0,
    kDark = 1,
    kLight = 2,
    kOnly = 4,
}

// Bitset for ColorSchemeFlag.
pub type ColorSchemeFlags = u8;

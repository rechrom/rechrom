// Copyright 2022 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/resolver/match_flags.h:11-34

// During rule-matching, we collect information about what the match result
// depended on. This supports targeted invalidation when pseudo-state changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MatchFlag {
    // :-webkit-drag
    kAffectedByDrag = 1 << 0,
    // :focus-within
    kAffectedByFocusWithin = 1 << 1,
    // :hover
    kAffectedByHover = 1 << 2,
    // :active
    kAffectedByActive = 1 << 3,
    // @starting-style
    kAffectedByStartingStyle = 1 << 4,
}

pub type MatchFlags = u8;

// Copyright 2020 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
//
// cpp: third_party/blink/renderer/core/css/resolver/cascade_origin.h:11-36

// Represents the origin criteria described by css-cascade.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CascadeOrigin {
    kNone = 0,
    kUserAgent = 0b0001,
    kUser = 0b0010,
    // https://drafts.csswg.org/css-cascade-5/#preshint
    kAuthorPresentationalHint = 0b0011,
    kAuthor = 0b0100,
    kAnimation = 0b0101,
    // The lower four bits of kAuthor, kUser and kUserAgent can be inverted to
    // efficiently produce a cascade-correct value for important declarations.
    // kTransition remains higher priority than every important origin.
    kTransition = 0b10000,
}

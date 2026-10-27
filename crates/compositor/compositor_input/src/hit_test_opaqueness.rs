// Translation status: complete
// Copyright 2023 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/input/hit_test_opaqueness.h:1-14,26-38;
//         cc/input/hit_test_opaqueness.cc:1-14,19-20,59.
use compositor_tree::geometry::{MaximumCoveredRect, Rect};
// Source: cc/input/hit_test_opaqueness.h:15-24.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum HitTestOpaqueness {
    kTransparent,
    kMixed,
    kOpaque,
}
// Source: cc/input/hit_test_opaqueness.cc:15-17.
fn UnionIsTight(rect1: Rect, rect2: Rect) -> bool {
    let mut union = rect1;
    union.Union(rect2);
    union == MaximumCoveredRect(rect1, rect2)
}
// Source: cc/input/hit_test_opaqueness.cc:21-25.
pub fn HitTestOpaquenessToString(opaqueness: HitTestOpaqueness) -> &'static str {
    if opaqueness == HitTestOpaqueness::kTransparent {
        "transparent"
    } else if opaqueness == HitTestOpaqueness::kOpaque {
        "opaque"
    } else {
        "mixed"
    }
}
// Source: cc/input/hit_test_opaqueness.cc:27-57.
pub fn UnionHitTestOpaqueness(
    rect1: Rect,
    opaqueness1: HitTestOpaqueness,
    rect2: Rect,
    opaqueness2: HitTestOpaqueness,
) -> HitTestOpaqueness {
    if rect1.IsEmpty() {
        return opaqueness2;
    }
    if rect2.IsEmpty() {
        return opaqueness1;
    }
    if opaqueness1 < opaqueness2 {
        return UnionHitTestOpaqueness(rect2, opaqueness2, rect1, opaqueness1);
    }
    match opaqueness1 {
        HitTestOpaqueness::kTransparent => {
            debug_assert_eq!(opaqueness2, HitTestOpaqueness::kTransparent);
            HitTestOpaqueness::kTransparent
        }
        HitTestOpaqueness::kMixed => {
            debug_assert_ne!(opaqueness2, HitTestOpaqueness::kOpaque);
            HitTestOpaqueness::kMixed
        }
        HitTestOpaqueness::kOpaque => {
            if opaqueness2 == HitTestOpaqueness::kOpaque {
                if UnionIsTight(rect1, rect2) {
                    HitTestOpaqueness::kOpaque
                } else {
                    HitTestOpaqueness::kMixed
                }
            } else if rect1.Contains(rect2) {
                HitTestOpaqueness::kOpaque
            } else {
                HitTestOpaqueness::kMixed
            }
        }
    }
}

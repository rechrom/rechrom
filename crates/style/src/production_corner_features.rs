// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! CSSCornersShorthand runtime JSON:1589-1590 experimental stable disabled.
//! Shape longhands/shape-only shorthands have no runtime flag; CSSBorderShape
//! runtime JSON:1520-1521 is stable. No fabricated host override is installed.
#![allow(non_snake_case)]
pub(crate) fn IsExposed(id: foundation::CSSPropertyID) -> bool {
    use foundation::CSSPropertyID::*;
    !matches!(
        id,
        kCorner
            | kCornerTopLeft
            | kCornerTopRight
            | kCornerBottomLeft
            | kCornerBottomRight
            | kCornerStartStart
            | kCornerStartEnd
            | kCornerEndStart
            | kCornerEndEnd
            | kCornerTop
            | kCornerRight
            | kCornerBottom
            | kCornerLeft
            | kCornerBlockStart
            | kCornerBlockEnd
            | kCornerInlineStart
            | kCornerInlineEnd
    )
}

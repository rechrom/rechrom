// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! PositionArea keyword classification and span conversion from Chromium source.
#![allow(non_snake_case)]
use foundation::CSSValueID;
use layoutng_style::style::position_area::PositionAreaRegion as R;
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    General,
    Horizontal,
    Vertical,
    Block,
    Inline,
    SelfBlock,
    SelfInline,
    StartEnd,
    SelfStartEnd,
}
// css_parsing_utils.cc:10192-10273.
pub(crate) fn KeywordKind(id: CSSValueID, allow_any: bool) -> Option<Kind> {
    use CSSValueID::*;
    if id == kAny && !allow_any {
        return None;
    }
    Some(match id {
        kAny | kSpanAll | kCenter => Kind::General,
        kLeft | kRight | kSpanLeft | kSpanRight | kXStart | kXEnd | kSpanXStart | kSpanXEnd
        | kSelfXStart | kSelfXEnd | kSpanSelfXStart | kSpanSelfXEnd => Kind::Horizontal,
        kTop | kBottom | kSpanTop | kSpanBottom | kYStart | kYEnd | kSpanYStart | kSpanYEnd
        | kSelfYStart | kSelfYEnd | kSpanSelfYStart | kSpanSelfYEnd => Kind::Vertical,
        kBlockStart | kBlockEnd | kSpanBlockStart | kSpanBlockEnd => Kind::Block,
        kInlineStart | kInlineEnd | kSpanInlineStart | kSpanInlineEnd => Kind::Inline,
        kSelfBlockStart | kSelfBlockEnd | kSpanSelfBlockStart | kSpanSelfBlockEnd => {
            Kind::SelfBlock
        }
        kSelfInlineStart | kSelfInlineEnd | kSpanSelfInlineStart | kSpanSelfInlineEnd => {
            Kind::SelfInline
        }
        kStart | kEnd | kSpanStart | kSpanEnd => Kind::StartEnd,
        kSelfStart | kSelfEnd | kSpanSelfStart | kSpanSelfEnd => Kind::SelfStartEnd,
        _ => return None,
    })
}
// css_parsing_utils.cc:10165-10182.
pub(crate) fn IsCompatible(a: Kind, b: Kind) -> bool {
    use Kind::*;
    a == General
        || b == General
        || matches!(
            (a, b),
            (Horizontal, Vertical) | (Block, Inline) | (SelfBlock, SelfInline)
        )
        || a == b && matches!(a, StartEnd | SelfStartEnd)
}
// css_parsing_utils.cc:10351-10375.
pub(crate) fn IsRepeated(id: CSSValueID) -> bool {
    use CSSValueID::*;
    matches!(
        id,
        kSpanAll
            | kCenter
            | kStart
            | kEnd
            | kSpanStart
            | kSpanEnd
            | kSelfStart
            | kSelfEnd
            | kSpanSelfStart
            | kSpanSelfEnd
            | kAny
    )
}
// style_builder_converter.cc:3999-4190, literal switch arms retained.
pub(crate) fn ConvertSpan(id: CSSValueID, allow_any: bool) -> Option<(R, R)> {
    use CSSValueID::*;
    if id == kAny && !allow_any {
        return None;
    }
    Some(match id {
        kSpanAll => (R::kAll, R::kAll),
        kCenter => (R::kCenter, R::kCenter),
        kLeft => (R::kLeft, R::kLeft),
        kRight => (R::kRight, R::kRight),
        kSpanLeft => (R::kLeft, R::kCenter),
        kSpanRight => (R::kCenter, R::kRight),
        kXStart => (R::kXStart, R::kXStart),
        kXEnd => (R::kXEnd, R::kXEnd),
        kSpanXStart => (R::kXStart, R::kCenter),
        kSpanXEnd => (R::kCenter, R::kXEnd),
        kSelfXStart => (R::kSelfXStart, R::kSelfXStart),
        kSelfXEnd => (R::kSelfXEnd, R::kSelfXEnd),
        kSpanSelfXStart => (R::kSelfXStart, R::kCenter),
        kSpanSelfXEnd => (R::kCenter, R::kSelfXEnd),
        kTop => (R::kTop, R::kTop),
        kBottom => (R::kBottom, R::kBottom),
        kSpanTop => (R::kTop, R::kCenter),
        kSpanBottom => (R::kCenter, R::kBottom),
        kYStart => (R::kYStart, R::kYStart),
        kYEnd => (R::kYEnd, R::kYEnd),
        kSpanYStart => (R::kYStart, R::kCenter),
        kSpanYEnd => (R::kCenter, R::kYEnd),
        kSelfYStart => (R::kSelfYStart, R::kSelfYStart),
        kSelfYEnd => (R::kSelfYEnd, R::kSelfYEnd),
        kSpanSelfYStart => (R::kSelfYStart, R::kCenter),
        kSpanSelfYEnd => (R::kCenter, R::kSelfYEnd),
        kBlockStart => (R::kBlockStart, R::kBlockStart),
        kBlockEnd => (R::kBlockEnd, R::kBlockEnd),
        kSpanBlockStart => (R::kBlockStart, R::kCenter),
        kSpanBlockEnd => (R::kCenter, R::kBlockEnd),
        kSelfBlockStart => (R::kSelfBlockStart, R::kSelfBlockStart),
        kSelfBlockEnd => (R::kSelfBlockEnd, R::kSelfBlockEnd),
        kSpanSelfBlockStart => (R::kSelfBlockStart, R::kCenter),
        kSpanSelfBlockEnd => (R::kCenter, R::kSelfBlockEnd),
        kInlineStart => (R::kInlineStart, R::kInlineStart),
        kInlineEnd => (R::kInlineEnd, R::kInlineEnd),
        kSpanInlineStart => (R::kInlineStart, R::kCenter),
        kSpanInlineEnd => (R::kCenter, R::kInlineEnd),
        kSelfInlineStart => (R::kSelfInlineStart, R::kSelfInlineStart),
        kSelfInlineEnd => (R::kSelfInlineEnd, R::kSelfInlineEnd),
        kSpanSelfInlineStart => (R::kSelfInlineStart, R::kCenter),
        kSpanSelfInlineEnd => (R::kCenter, R::kSelfInlineEnd),
        kStart => (R::kStart, R::kStart),
        kEnd => (R::kEnd, R::kEnd),
        kSpanStart => (R::kStart, R::kCenter),
        kSpanEnd => (R::kCenter, R::kEnd),
        kSelfStart => (R::kSelfStart, R::kSelfStart),
        kSelfEnd => (R::kSelfEnd, R::kSelfEnd),
        kSpanSelfStart => (R::kSelfStart, R::kCenter),
        kSpanSelfEnd => (R::kCenter, R::kSelfEnd),
        kAny => (R::kAny, R::kAny),
        _ => return None,
    })
}

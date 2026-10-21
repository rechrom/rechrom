#![allow(non_camel_case_types, non_upper_case_globals)]

// cpp: foundation/style_values/css/css_anchor_query_enums.h:12-15
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum CSSAnchorQueryType {
    kAnchor = 1 << 0,
    kAnchorSize = 1 << 1,
}

// cpp: foundation/style_values/css/css_anchor_query_enums.h:17-20
pub type CSSAnchorQueryTypes = u8;
pub const kCSSAnchorQueryTypesNone: CSSAnchorQueryTypes = 0;
pub const kCSSAnchorQueryTypesAll: CSSAnchorQueryTypes = !kCSSAnchorQueryTypesNone;

// cpp: foundation/style_values/css/css_anchor_query_enums.h:22-35
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum CSSAnchorValue {
    kInside,
    kOutside,
    kTop,
    kLeft,
    kRight,
    kBottom,
    kStart,
    kEnd,
    kSelfStart,
    kSelfEnd,
    kCenter,
    kPercentage,
}

// cpp: foundation/style_values/css/css_anchor_query_enums.h:37-45
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum CSSAnchorSizeValue {
    kImplicit,
    kWidth,
    kHeight,
    kBlock,
    kInline,
    kSelfBlock,
    kSelfInline,
}

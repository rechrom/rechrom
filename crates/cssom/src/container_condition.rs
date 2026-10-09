// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Neutral data projection of ContainerQuerySet/ConditionalExpNode/media bounds.
//! No condition text parser or document-owned registry lives in CSSOM.
#![allow(non_camel_case_types)]
use foundation::{AtomicString, CSSValueID};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CSSContainerNumber(u64);
impl CSSContainerNumber {
    pub fn new(value: f64) -> Self {
        Self(value.to_bits())
    }
    pub fn value(self) -> f64 {
        f64::from_bits(self.0)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSContainerUnit {
    kUnknown,
    kNumber,
    kPercentage,
    kEms,
    kExs,
    kPixels,
    kCentimeters,
    kMillimeters,
    kInches,
    kPoints,
    kPicas,
    kQuarterMillimeters,
    kViewportWidth,
    kViewportHeight,
    kViewportInlineSize,
    kViewportBlockSize,
    kViewportMin,
    kViewportMax,
    kSmallViewportWidth,
    kSmallViewportHeight,
    kSmallViewportInlineSize,
    kSmallViewportBlockSize,
    kSmallViewportMin,
    kSmallViewportMax,
    kLargeViewportWidth,
    kLargeViewportHeight,
    kLargeViewportInlineSize,
    kLargeViewportBlockSize,
    kLargeViewportMin,
    kLargeViewportMax,
    kDynamicViewportWidth,
    kDynamicViewportHeight,
    kDynamicViewportInlineSize,
    kDynamicViewportBlockSize,
    kDynamicViewportMin,
    kDynamicViewportMax,
    kContainerWidth,
    kContainerHeight,
    kContainerInlineSize,
    kContainerBlockSize,
    kContainerMin,
    kContainerMax,
    kRems,
    kRexs,
    kRchs,
    kRics,
    kChs,
    kIcs,
    kLhs,
    kRlhs,
    kCaps,
    kRcaps,
    kUserUnits,
    kDegrees,
    kRadians,
    kGradians,
    kTurns,
    kMilliseconds,
    kSeconds,
    kHertz,
    kKilohertz,
    kDotsPerPixel,
    kX,
    kDotsPerInch,
    kDotsPerCentimeter,
    kFlex,
    kInteger,
    kIdent,
    kQuirkyEms,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSContainerOperator {
    None,
    Equal,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSContainerFunction {
    Style,
    ScrollState,
    Anchored,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSContainerSyntaxToken {
    BadUrl,
    IncludeMatch,
    DashMatch,
    PrefixMatch,
    SuffixMatch,
    SubstringMatch,
    Column,
    Whitespace,
    CDO,
    CDC,
    Colon,
    Semicolon,
    Comma,
    LeftParenthesis,
    RightParenthesis,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    BadString,
    EOF,
    Comment,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSContainerNumericSign {
    None,
    Plus,
    Minus,
}
/// Typed custom-property tokens retain token boundaries through CSSOM ownership.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CSSContainerToken {
    Ident(AtomicString),
    Function(AtomicString),
    AtKeyword(AtomicString),
    Hash(AtomicString, bool),
    Url(AtomicString),
    String(AtomicString),
    Delimiter(u16),
    Number {
        value: CSSContainerNumber,
        integer: bool,
        sign: CSSContainerNumericSign,
    },
    Percentage {
        value: CSSContainerNumber,
        integer: bool,
    },
    Dimension {
        value: CSSContainerNumber,
        integer: bool,
        unit: CSSContainerUnit,
        unit_name: AtomicString,
    },
    UnicodeRange(i32, i32),
    Syntax(CSSContainerSyntaxToken),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSSContainerVariableToken {
    pub token: CSSContainerToken,
    pub text: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSSContainerVariableData {
    pub tokens: Vec<CSSContainerVariableToken>,
    pub original_text: String,
    pub features: u8,
    pub is_animation_tainted: bool,
    pub is_attr_tainted: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CSSContainerOperand {
    Invalid,
    Identifier(CSSValueID),
    Numeric(CSSContainerNumber, CSSContainerUnit),
    Ratio(Box<Self>, Box<Self>),
    Unparsed(CSSContainerVariableData),
    List(Vec<Self>, CSSContainerListSeparator),
    CustomIdent(AtomicString),
    String(AtomicString),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSSContainerComparison {
    pub operator: CSSContainerOperator,
    pub value: CSSContainerOperand,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSSContainerFeature {
    pub name: AtomicString,
    pub left: CSSContainerComparison,
    pub right: CSSContainerComparison,
    pub reference: Option<CSSContainerVariableData>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSContainerUnsupportedReason {
    MathExpression,
    ValueClass,
    Function,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CSSContainerCondition {
    Feature(CSSContainerFeature),
    Unknown(String),
    Not(Box<Self>),
    Nested(Box<Self>),
    Function(CSSContainerFunction, Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Unsupported(CSSContainerUnsupportedReason),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CSSContainerQuery {
    pub name: AtomicString,
    pub condition: Option<CSSContainerCondition>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CSSContainerQuerySet {
    pub queries: Vec<CSSContainerQuery>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSContainerListSeparator {
    Space,
    Comma,
    Slash,
}

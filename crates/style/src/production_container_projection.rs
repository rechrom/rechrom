// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Ownership projection only: preserves parsed conditional nodes and variable token identity.
#![allow(non_snake_case)]
use crate::{
    css_primitive_value::UnitType,
    css_value::CSSValuePayload,
    media_queries::{conditional_exp_node::ConditionalExpNode, media_query_exp::*},
    parser::{css_parser_mode::CSSParserMode, css_parser_token::*},
    production_container_parser::{ContainerCondition, ContainerSet},
    production_css_value as values,
};
use ::cssom::*;
use foundation::{AtomicString, CSSPropertyID, String, StringView};
use std::rc::Rc;
pub fn ProjectContainerUnit(u: UnitType) -> CSSContainerUnit {
    match u {
        UnitType::kUnknown => CSSContainerUnit::kUnknown,
        UnitType::kNumber => CSSContainerUnit::kNumber,
        UnitType::kPercentage => CSSContainerUnit::kPercentage,
        UnitType::kEms => CSSContainerUnit::kEms,
        UnitType::kExs => CSSContainerUnit::kExs,
        UnitType::kPixels => CSSContainerUnit::kPixels,
        UnitType::kCentimeters => CSSContainerUnit::kCentimeters,
        UnitType::kMillimeters => CSSContainerUnit::kMillimeters,
        UnitType::kInches => CSSContainerUnit::kInches,
        UnitType::kPoints => CSSContainerUnit::kPoints,
        UnitType::kPicas => CSSContainerUnit::kPicas,
        UnitType::kQuarterMillimeters => CSSContainerUnit::kQuarterMillimeters,
        UnitType::kViewportWidth => CSSContainerUnit::kViewportWidth,
        UnitType::kViewportHeight => CSSContainerUnit::kViewportHeight,
        UnitType::kViewportInlineSize => CSSContainerUnit::kViewportInlineSize,
        UnitType::kViewportBlockSize => CSSContainerUnit::kViewportBlockSize,
        UnitType::kViewportMin => CSSContainerUnit::kViewportMin,
        UnitType::kViewportMax => CSSContainerUnit::kViewportMax,
        UnitType::kSmallViewportWidth => CSSContainerUnit::kSmallViewportWidth,
        UnitType::kSmallViewportHeight => CSSContainerUnit::kSmallViewportHeight,
        UnitType::kSmallViewportInlineSize => CSSContainerUnit::kSmallViewportInlineSize,
        UnitType::kSmallViewportBlockSize => CSSContainerUnit::kSmallViewportBlockSize,
        UnitType::kSmallViewportMin => CSSContainerUnit::kSmallViewportMin,
        UnitType::kSmallViewportMax => CSSContainerUnit::kSmallViewportMax,
        UnitType::kLargeViewportWidth => CSSContainerUnit::kLargeViewportWidth,
        UnitType::kLargeViewportHeight => CSSContainerUnit::kLargeViewportHeight,
        UnitType::kLargeViewportInlineSize => CSSContainerUnit::kLargeViewportInlineSize,
        UnitType::kLargeViewportBlockSize => CSSContainerUnit::kLargeViewportBlockSize,
        UnitType::kLargeViewportMin => CSSContainerUnit::kLargeViewportMin,
        UnitType::kLargeViewportMax => CSSContainerUnit::kLargeViewportMax,
        UnitType::kDynamicViewportWidth => CSSContainerUnit::kDynamicViewportWidth,
        UnitType::kDynamicViewportHeight => CSSContainerUnit::kDynamicViewportHeight,
        UnitType::kDynamicViewportInlineSize => CSSContainerUnit::kDynamicViewportInlineSize,
        UnitType::kDynamicViewportBlockSize => CSSContainerUnit::kDynamicViewportBlockSize,
        UnitType::kDynamicViewportMin => CSSContainerUnit::kDynamicViewportMin,
        UnitType::kDynamicViewportMax => CSSContainerUnit::kDynamicViewportMax,
        UnitType::kContainerWidth => CSSContainerUnit::kContainerWidth,
        UnitType::kContainerHeight => CSSContainerUnit::kContainerHeight,
        UnitType::kContainerInlineSize => CSSContainerUnit::kContainerInlineSize,
        UnitType::kContainerBlockSize => CSSContainerUnit::kContainerBlockSize,
        UnitType::kContainerMin => CSSContainerUnit::kContainerMin,
        UnitType::kContainerMax => CSSContainerUnit::kContainerMax,
        UnitType::kRems => CSSContainerUnit::kRems,
        UnitType::kRexs => CSSContainerUnit::kRexs,
        UnitType::kRchs => CSSContainerUnit::kRchs,
        UnitType::kRics => CSSContainerUnit::kRics,
        UnitType::kChs => CSSContainerUnit::kChs,
        UnitType::kIcs => CSSContainerUnit::kIcs,
        UnitType::kLhs => CSSContainerUnit::kLhs,
        UnitType::kRlhs => CSSContainerUnit::kRlhs,
        UnitType::kCaps => CSSContainerUnit::kCaps,
        UnitType::kRcaps => CSSContainerUnit::kRcaps,
        UnitType::kUserUnits => CSSContainerUnit::kUserUnits,
        UnitType::kDegrees => CSSContainerUnit::kDegrees,
        UnitType::kRadians => CSSContainerUnit::kRadians,
        UnitType::kGradians => CSSContainerUnit::kGradians,
        UnitType::kTurns => CSSContainerUnit::kTurns,
        UnitType::kMilliseconds => CSSContainerUnit::kMilliseconds,
        UnitType::kSeconds => CSSContainerUnit::kSeconds,
        UnitType::kHertz => CSSContainerUnit::kHertz,
        UnitType::kKilohertz => CSSContainerUnit::kKilohertz,
        UnitType::kDotsPerPixel => CSSContainerUnit::kDotsPerPixel,
        UnitType::kX => CSSContainerUnit::kX,
        UnitType::kDotsPerInch => CSSContainerUnit::kDotsPerInch,
        UnitType::kDotsPerCentimeter => CSSContainerUnit::kDotsPerCentimeter,
        UnitType::kFlex => CSSContainerUnit::kFlex,
        UnitType::kInteger => CSSContainerUnit::kInteger,
        UnitType::kIdent => CSSContainerUnit::kIdent,
        UnitType::kQuirkyEms => CSSContainerUnit::kQuirkyEms,
    }
}
pub(crate) fn RestoreContainerUnit(u: CSSContainerUnit) -> UnitType {
    match u {
        CSSContainerUnit::kUnknown => UnitType::kUnknown,
        CSSContainerUnit::kNumber => UnitType::kNumber,
        CSSContainerUnit::kPercentage => UnitType::kPercentage,
        CSSContainerUnit::kEms => UnitType::kEms,
        CSSContainerUnit::kExs => UnitType::kExs,
        CSSContainerUnit::kPixels => UnitType::kPixels,
        CSSContainerUnit::kCentimeters => UnitType::kCentimeters,
        CSSContainerUnit::kMillimeters => UnitType::kMillimeters,
        CSSContainerUnit::kInches => UnitType::kInches,
        CSSContainerUnit::kPoints => UnitType::kPoints,
        CSSContainerUnit::kPicas => UnitType::kPicas,
        CSSContainerUnit::kQuarterMillimeters => UnitType::kQuarterMillimeters,
        CSSContainerUnit::kViewportWidth => UnitType::kViewportWidth,
        CSSContainerUnit::kViewportHeight => UnitType::kViewportHeight,
        CSSContainerUnit::kViewportInlineSize => UnitType::kViewportInlineSize,
        CSSContainerUnit::kViewportBlockSize => UnitType::kViewportBlockSize,
        CSSContainerUnit::kViewportMin => UnitType::kViewportMin,
        CSSContainerUnit::kViewportMax => UnitType::kViewportMax,
        CSSContainerUnit::kSmallViewportWidth => UnitType::kSmallViewportWidth,
        CSSContainerUnit::kSmallViewportHeight => UnitType::kSmallViewportHeight,
        CSSContainerUnit::kSmallViewportInlineSize => UnitType::kSmallViewportInlineSize,
        CSSContainerUnit::kSmallViewportBlockSize => UnitType::kSmallViewportBlockSize,
        CSSContainerUnit::kSmallViewportMin => UnitType::kSmallViewportMin,
        CSSContainerUnit::kSmallViewportMax => UnitType::kSmallViewportMax,
        CSSContainerUnit::kLargeViewportWidth => UnitType::kLargeViewportWidth,
        CSSContainerUnit::kLargeViewportHeight => UnitType::kLargeViewportHeight,
        CSSContainerUnit::kLargeViewportInlineSize => UnitType::kLargeViewportInlineSize,
        CSSContainerUnit::kLargeViewportBlockSize => UnitType::kLargeViewportBlockSize,
        CSSContainerUnit::kLargeViewportMin => UnitType::kLargeViewportMin,
        CSSContainerUnit::kLargeViewportMax => UnitType::kLargeViewportMax,
        CSSContainerUnit::kDynamicViewportWidth => UnitType::kDynamicViewportWidth,
        CSSContainerUnit::kDynamicViewportHeight => UnitType::kDynamicViewportHeight,
        CSSContainerUnit::kDynamicViewportInlineSize => UnitType::kDynamicViewportInlineSize,
        CSSContainerUnit::kDynamicViewportBlockSize => UnitType::kDynamicViewportBlockSize,
        CSSContainerUnit::kDynamicViewportMin => UnitType::kDynamicViewportMin,
        CSSContainerUnit::kDynamicViewportMax => UnitType::kDynamicViewportMax,
        CSSContainerUnit::kContainerWidth => UnitType::kContainerWidth,
        CSSContainerUnit::kContainerHeight => UnitType::kContainerHeight,
        CSSContainerUnit::kContainerInlineSize => UnitType::kContainerInlineSize,
        CSSContainerUnit::kContainerBlockSize => UnitType::kContainerBlockSize,
        CSSContainerUnit::kContainerMin => UnitType::kContainerMin,
        CSSContainerUnit::kContainerMax => UnitType::kContainerMax,
        CSSContainerUnit::kRems => UnitType::kRems,
        CSSContainerUnit::kRexs => UnitType::kRexs,
        CSSContainerUnit::kRchs => UnitType::kRchs,
        CSSContainerUnit::kRics => UnitType::kRics,
        CSSContainerUnit::kChs => UnitType::kChs,
        CSSContainerUnit::kIcs => UnitType::kIcs,
        CSSContainerUnit::kLhs => UnitType::kLhs,
        CSSContainerUnit::kRlhs => UnitType::kRlhs,
        CSSContainerUnit::kCaps => UnitType::kCaps,
        CSSContainerUnit::kRcaps => UnitType::kRcaps,
        CSSContainerUnit::kUserUnits => UnitType::kUserUnits,
        CSSContainerUnit::kDegrees => UnitType::kDegrees,
        CSSContainerUnit::kRadians => UnitType::kRadians,
        CSSContainerUnit::kGradians => UnitType::kGradians,
        CSSContainerUnit::kTurns => UnitType::kTurns,
        CSSContainerUnit::kMilliseconds => UnitType::kMilliseconds,
        CSSContainerUnit::kSeconds => UnitType::kSeconds,
        CSSContainerUnit::kHertz => UnitType::kHertz,
        CSSContainerUnit::kKilohertz => UnitType::kKilohertz,
        CSSContainerUnit::kDotsPerPixel => UnitType::kDotsPerPixel,
        CSSContainerUnit::kX => UnitType::kX,
        CSSContainerUnit::kDotsPerInch => UnitType::kDotsPerInch,
        CSSContainerUnit::kDotsPerCentimeter => UnitType::kDotsPerCentimeter,
        CSSContainerUnit::kFlex => UnitType::kFlex,
        CSSContainerUnit::kInteger => UnitType::kInteger,
        CSSContainerUnit::kIdent => UnitType::kIdent,
        CSSContainerUnit::kQuirkyEms => UnitType::kQuirkyEms,
    }
}
fn atom(t: &CSSParserToken) -> AtomicString {
    AtomicString::from_utf16(t.Value().ToString().Span16().unwrap_or_default())
}
fn view(a: &AtomicString) -> StringView {
    StringView::from(&String::from_utf16(a.utf16_units().unwrap_or_default()))
}
fn ProjectToken(t: &CSSParserToken) -> CSSContainerToken {
    use CSSContainerToken::*;
    use CSSParserTokenType::*;
    let integer = || t.GetNumericValueType() == NumericValueType::kIntegerValueType;
    let number = || CSSContainerNumber::new(t.NumericValue());
    match t.GetType() {
        kIdentToken => Ident(atom(t)),
        kFunctionToken => Function(atom(t)),
        kAtKeywordToken => AtKeyword(atom(t)),
        kHashToken => Hash(atom(t), t.GetHashTokenType() == HashTokenType::kHashTokenId),
        kUrlToken => Url(atom(t)),
        kStringToken => String(atom(t)),
        kDelimiterToken => Delimiter(t.Delimiter()),
        kNumberToken => Number {
            value: number(),
            integer: integer(),
            sign: match t.GetNumericSign() {
                NumericSign::kNoSign => CSSContainerNumericSign::None,
                NumericSign::kPlusSign => CSSContainerNumericSign::Plus,
                NumericSign::kMinusSign => CSSContainerNumericSign::Minus,
            },
        },
        kPercentageToken => Percentage {
            value: number(),
            integer: integer(),
        },
        kDimensionToken => Dimension {
            value: number(),
            integer: integer(),
            unit: ProjectContainerUnit(t.GetUnitType()),
            unit_name: atom(t),
        },
        kUnicodeRangeToken => UnicodeRange(t.UnicodeRangeStart(), t.UnicodeRangeEnd()),
        kBadUrlToken => Syntax(CSSContainerSyntaxToken::BadUrl),
        kIncludeMatchToken => Syntax(CSSContainerSyntaxToken::IncludeMatch),
        kDashMatchToken => Syntax(CSSContainerSyntaxToken::DashMatch),
        kPrefixMatchToken => Syntax(CSSContainerSyntaxToken::PrefixMatch),
        kSuffixMatchToken => Syntax(CSSContainerSyntaxToken::SuffixMatch),
        kSubstringMatchToken => Syntax(CSSContainerSyntaxToken::SubstringMatch),
        kColumnToken => Syntax(CSSContainerSyntaxToken::Column),
        kWhitespaceToken => Syntax(CSSContainerSyntaxToken::Whitespace),
        kCDOToken => Syntax(CSSContainerSyntaxToken::CDO),
        kCDCToken => Syntax(CSSContainerSyntaxToken::CDC),
        kColonToken => Syntax(CSSContainerSyntaxToken::Colon),
        kSemicolonToken => Syntax(CSSContainerSyntaxToken::Semicolon),
        kCommaToken => Syntax(CSSContainerSyntaxToken::Comma),
        kLeftParenthesisToken => Syntax(CSSContainerSyntaxToken::LeftParenthesis),
        kRightParenthesisToken => Syntax(CSSContainerSyntaxToken::RightParenthesis),
        kLeftBracketToken => Syntax(CSSContainerSyntaxToken::LeftBracket),
        kRightBracketToken => Syntax(CSSContainerSyntaxToken::RightBracket),
        kLeftBraceToken => Syntax(CSSContainerSyntaxToken::LeftBrace),
        kRightBraceToken => Syntax(CSSContainerSyntaxToken::RightBrace),
        kBadStringToken => Syntax(CSSContainerSyntaxToken::BadString),
        kEOFToken => Syntax(CSSContainerSyntaxToken::EOF),
        kCommentToken => Syntax(CSSContainerSyntaxToken::Comment),
    }
}
fn RestoreToken(t: &CSSContainerToken) -> CSSParserToken {
    use CSSContainerToken::*;
    use CSSParserTokenType::*;
    let number = |v: CSSContainerNumber, i: bool, sign: NumericSign| {
        CSSParserToken::WithNumber(
            kNumberToken,
            v.value(),
            if i {
                NumericValueType::kIntegerValueType
            } else {
                NumericValueType::kNumberValueType
            },
            sign,
        )
    };
    let value = |ty, a: &AtomicString, b| CSSParserToken::WithValue(ty, view(a), b, None);
    match t {
        Ident(a) => value(kIdentToken, a, BlockType::kNotBlock),
        Function(a) => CSSParserToken::WithValue(
            kFunctionToken,
            view(a),
            BlockType::kBlockStart,
            Some(crate::parser::css_property_parser::CssValueKeywordID(
                &view(a),
            )),
        ),
        AtKeyword(a) => value(kAtKeywordToken, a, BlockType::kNotBlock),
        Url(a) => value(kUrlToken, a, BlockType::kNotBlock),
        String(a) => value(kStringToken, a, BlockType::kNotBlock),
        Hash(a, id) => CSSParserToken::WithHash(
            if *id {
                HashTokenType::kHashTokenId
            } else {
                HashTokenType::kHashTokenUnrestricted
            },
            view(a),
        ),
        Delimiter(d) => CSSParserToken::WithDelimiter(kDelimiterToken, *d),
        Number {
            value,
            integer,
            sign,
        } => number(
            *value,
            *integer,
            match sign {
                CSSContainerNumericSign::None => NumericSign::kNoSign,
                CSSContainerNumericSign::Plus => NumericSign::kPlusSign,
                CSSContainerNumericSign::Minus => NumericSign::kMinusSign,
            },
        ),
        Percentage { value, integer } => {
            let mut t = number(*value, *integer, NumericSign::kNoSign);
            t.ConvertToPercentage();
            t
        }
        Dimension {
            value,
            integer,
            unit_name,
            ..
        } => {
            let mut t = number(*value, *integer, NumericSign::kNoSign);
            t.ConvertToDimensionWithUnit(view(unit_name));
            t
        }
        UnicodeRange(a, b) => CSSParserToken::WithUnicodeRange(kUnicodeRangeToken, *a, *b),
        Syntax(s) => {
            let (ty, block) = match s {
                CSSContainerSyntaxToken::BadUrl => (kBadUrlToken, BlockType::kNotBlock),
                CSSContainerSyntaxToken::IncludeMatch => (kIncludeMatchToken, BlockType::kNotBlock),
                CSSContainerSyntaxToken::DashMatch => (kDashMatchToken, BlockType::kNotBlock),
                CSSContainerSyntaxToken::PrefixMatch => (kPrefixMatchToken, BlockType::kNotBlock),
                CSSContainerSyntaxToken::SuffixMatch => (kSuffixMatchToken, BlockType::kNotBlock),
                CSSContainerSyntaxToken::SubstringMatch => {
                    (kSubstringMatchToken, BlockType::kNotBlock)
                }
                CSSContainerSyntaxToken::Column => (kColumnToken, BlockType::kNotBlock),
                CSSContainerSyntaxToken::Whitespace => (kWhitespaceToken, BlockType::kNotBlock),
                CSSContainerSyntaxToken::CDO => (kCDOToken, BlockType::kNotBlock),
                CSSContainerSyntaxToken::CDC => (kCDCToken, BlockType::kNotBlock),
                CSSContainerSyntaxToken::Colon => (kColonToken, BlockType::kNotBlock),
                CSSContainerSyntaxToken::Semicolon => (kSemicolonToken, BlockType::kNotBlock),
                CSSContainerSyntaxToken::Comma => (kCommaToken, BlockType::kNotBlock),
                CSSContainerSyntaxToken::LeftParenthesis => {
                    (kLeftParenthesisToken, BlockType::kBlockStart)
                }
                CSSContainerSyntaxToken::RightParenthesis => {
                    (kRightParenthesisToken, BlockType::kBlockEnd)
                }
                CSSContainerSyntaxToken::LeftBracket => (kLeftBracketToken, BlockType::kBlockStart),
                CSSContainerSyntaxToken::RightBracket => (kRightBracketToken, BlockType::kBlockEnd),
                CSSContainerSyntaxToken::LeftBrace => (kLeftBraceToken, BlockType::kBlockStart),
                CSSContainerSyntaxToken::RightBrace => (kRightBraceToken, BlockType::kBlockEnd),
                CSSContainerSyntaxToken::BadString => (kBadStringToken, BlockType::kNotBlock),
                CSSContainerSyntaxToken::EOF => (kEOFToken, BlockType::kNotBlock),
                CSSContainerSyntaxToken::Comment => (kCommentToken, BlockType::kNotBlock),
            };
            CSSParserToken::new(ty, block)
        }
    }
}
pub fn ProjectContainerVariableData(d: &values::CSSVariableData) -> CSSContainerVariableData {
    CSSContainerVariableData {
        tokens: d
            .tokens
            .iter()
            .map(|t| CSSContainerVariableToken {
                token: ProjectToken(&t.token),
                text: t.text.Utf8(),
            })
            .collect(),
        original_text: d.original_text.Utf8(),
        features: d.features,
        is_animation_tainted: d.is_animation_tainted,
        is_attr_tainted: d.is_attr_tainted,
    }
}
pub(crate) fn RestoreContainerVariableData(
    d: &CSSContainerVariableData,
) -> values::CSSVariableData {
    values::CSSVariableData {
        tokens: d
            .tokens
            .iter()
            .map(|t| values::VariableToken {
                token: RestoreToken(&t.token),
                text: String::FromUtf8(t.text.as_bytes()),
            })
            .collect(),
        original_text: String::FromUtf8(d.original_text.as_bytes()),
        features: d.features,
        is_animation_tainted: d.is_animation_tainted,
        is_attr_tainted: d.is_attr_tainted,
    }
}
fn ProjectValue(v: &values::Value) -> Result<CSSContainerOperand, CSSContainerUnsupportedReason> {
    use CSSContainerOperand::*;
    Ok(match v.Payload() {
        CSSValuePayload::kNumericLiteralClass(v) => Numeric(
            CSSContainerNumber::new(v.DoubleValue()),
            ProjectContainerUnit(v.GetType()),
        ),
        CSSValuePayload::kIdentifierClass(v) => Identifier(v.0),
        CSSValuePayload::kUnparsedDeclarationClass(v) => {
            Unparsed(ProjectContainerVariableData(&v.data))
        }
        CSSValuePayload::kCustomIdentClass(v) => CustomIdent(v.name.clone()),
        CSSValuePayload::kStringClass(v) => {
            String(AtomicString::from_utf16(v.0.Span16().unwrap_or_default()))
        }
        CSSValuePayload::kValueListClass(v) => List(
            v.values
                .iter()
                .map(|v| ProjectValue(v))
                .collect::<Result<Vec<_>, _>>()?,
            match v.separator {
                values::ListSeparator::Space => CSSContainerListSeparator::Space,
                values::ListSeparator::Comma => CSSContainerListSeparator::Comma,
                values::ListSeparator::Slash => CSSContainerListSeparator::Slash,
            },
        ),
        CSSValuePayload::kInitialClass(_) => Identifier(foundation::CSSValueID::kInitial),
        CSSValuePayload::kInheritedClass(_) => Identifier(foundation::CSSValueID::kInherit),
        CSSValuePayload::kUnsetClass(_) => Identifier(foundation::CSSValueID::kUnset),
        CSSValuePayload::kRevertClass(_) => Identifier(foundation::CSSValueID::kRevert),
        CSSValuePayload::kRevertLayerClass(_) => Identifier(foundation::CSSValueID::kRevertLayer),
        CSSValuePayload::kRevertRuleClass(_) => Identifier(foundation::CSSValueID::kRevertRule),
        _ => return Err(CSSContainerUnsupportedReason::ValueClass),
    })
}
pub(crate) fn RestoreContainerOperand(o: &CSSContainerOperand) -> Option<Rc<values::Value>> {
    use CSSContainerOperand::*;
    Some(match o {
        Invalid | Ratio(..) => return None,
        Identifier(id) => values::wide(*id).unwrap_or_else(|| values::identifier(*id)),
        Numeric(v, u) => values::numeric(v.value(), RestoreContainerUnit(*u)),
        Unparsed(d) => values::unparsed(
            RestoreContainerVariableData(d),
            CSSParserMode::kHTMLStandardMode,
        ),
        CustomIdent(n) => values::custom_ident(
            &foundation::String::from_utf16(n.utf16_units().unwrap_or_default()),
            CSSPropertyID::kInvalid,
        ),
        String(s) => values::string(foundation::String::from_utf16(
            s.utf16_units().unwrap_or_default(),
        )),
        List(items, separator) => values::list(
            items
                .iter()
                .map(RestoreContainerOperand)
                .collect::<Option<Vec<_>>>()?,
            match separator {
                CSSContainerListSeparator::Space => values::ListSeparator::Space,
                CSSContainerListSeparator::Comma => values::ListSeparator::Comma,
                CSSContainerListSeparator::Slash => values::ListSeparator::Slash,
            },
        ),
    })
}
fn Operand(
    v: &MediaQueryExpValue<values::Value>,
) -> Result<CSSContainerOperand, CSSContainerUnsupportedReason> {
    Ok(match v {
        MediaQueryExpValue::Invalid => CSSContainerOperand::Invalid,
        MediaQueryExpValue::Id(id) => CSSContainerOperand::Identifier(*id),
        MediaQueryExpValue::Value(v) => ProjectValue(v)?,
        MediaQueryExpValue::Ratio(v) => {
            CSSContainerOperand::Ratio(Box::new(ProjectValue(&v.0)?), Box::new(ProjectValue(&v.1)?))
        }
    })
}
fn Comparison(
    c: &MediaQueryExpComparison<values::Value>,
) -> Result<CSSContainerComparison, CSSContainerUnsupportedReason> {
    Ok(CSSContainerComparison {
        operator: match c.op {
            MediaQueryOperator::kNone => CSSContainerOperator::None,
            MediaQueryOperator::kEq => CSSContainerOperator::Equal,
            MediaQueryOperator::kLt => CSSContainerOperator::Less,
            MediaQueryOperator::kLe => CSSContainerOperator::LessEqual,
            MediaQueryOperator::kGt => CSSContainerOperator::Greater,
            MediaQueryOperator::kGe => CSSContainerOperator::GreaterEqual,
        },
        value: Operand(&c.value)?,
    })
}
pub fn ProjectContainerCondition(n: &ContainerCondition) -> CSSContainerCondition {
    use CSSContainerCondition as C;
    use ConditionalExpNode::*;
    match n {
        Feature(f) => {
            let result = (|| {
                Some(CSSContainerFeature {
                    name: if f.HasMediaFeature() {
                        f.MediaFeature().clone()
                    } else {
                        AtomicString::default()
                    },
                    left: Comparison(&f.Bounds().left).ok()?,
                    right: Comparison(&f.Bounds().right).ok()?,
                    reference: if f.HasStyleRange() {
                        Some(ProjectContainerVariableData(&f.ReferenceValue().data))
                    } else {
                        None
                    },
                })
            })();
            result
                .map(C::Feature)
                .unwrap_or(C::Unsupported(CSSContainerUnsupportedReason::ValueClass))
        }
        Unknown(t) => C::Unknown(t.Utf8()),
        NotNode(n) => C::Not(Box::new(ProjectContainerCondition(n))),
        NestedNode(n) => C::Nested(Box::new(ProjectContainerCondition(n))),
        FunctionNode(n, name) => match name.Utf8().as_str() {
            "style" => C::Function(
                CSSContainerFunction::Style,
                Box::new(ProjectContainerCondition(n)),
            ),
            "scroll-state" => C::Function(
                CSSContainerFunction::ScrollState,
                Box::new(ProjectContainerCondition(n)),
            ),
            "anchored" => C::Function(
                CSSContainerFunction::Anchored,
                Box::new(ProjectContainerCondition(n)),
            ),
            _ => C::Unsupported(CSSContainerUnsupportedReason::Function),
        },
        AndNode(a, b) => C::And(
            Box::new(ProjectContainerCondition(a)),
            Box::new(ProjectContainerCondition(b)),
        ),
        OrNode(a, b) => C::Or(
            Box::new(ProjectContainerCondition(a)),
            Box::new(ProjectContainerCondition(b)),
        ),
    }
}
pub fn ProjectContainerQuerySet(set: &ContainerSet) -> CSSContainerQuerySet {
    CSSContainerQuerySet {
        queries: set
            .Queries()
            .iter()
            .map(|q| CSSContainerQuery {
                name: q.SelectorName(),
                condition: q.Query().map(ProjectContainerCondition),
            })
            .collect(),
    }
}

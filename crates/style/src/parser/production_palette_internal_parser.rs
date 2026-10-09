// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! ConsumeFontPalette and UA-only internal keyword consumers.
#![allow(non_snake_case)]
use super::*;
pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kFontPalette
            | CSSPropertyID::kInternalAlignContentBlock
            | CSSPropertyID::kInternalEmptyLineHeight
    )
}
pub(super) fn IsUAOnly(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kInternalAlignContentBlock | CSSPropertyID::kInternalEmptyLineHeight
    )
}
// cpp: css_parsing_utils.cc:6352-6367; longhands_custom.cc:12775-12789.
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSPropertyID::*;
    if id == kInternalAlignContentBlock {
        return ConsumeLiteral(id, s, mode, Grammar::Keywords(&["center", "normal"]));
    }
    if id == kInternalEmptyLineHeight {
        return ConsumeLiteral(id, s, mode, Grammar::Keywords(&["fabricated", "none"]));
    }
    if matches!(
        s.Peek().Id(),
        CSSValueID::kNormal | CSSValueID::kLight | CSSValueID::kDark
    ) {
        return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
    }
    if s.Peek().FunctionId() == Some(CSSValueID::kPaletteMix) {
        return Err(unsupported(
            id,
            "ConsumePaletteMixFunction / CSSPaletteMixValue owner",
        ));
    }
    anchor_parser::DashedIdent(id, s)?.ok_or_else(|| invalid(id))
}

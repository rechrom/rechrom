// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium longhands_custom.cc ConsumeReflect typed consumer.
#![allow(non_snake_case)]
use super::*;
pub(super) fn Consume<T: TokenStreamTokenizer>(
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    let id = CSSPropertyID::kWebkitBoxReflect;
    if !matches!(
        s.Peek().Id(),
        CSSValueID::kAbove | CSSValueID::kBelow | CSSValueID::kLeft | CSSValueID::kRight
    ) {
        return Err(invalid(id));
    }
    let direction = values::identifier(s.ConsumeIncludingWhitespace().Id());
    s.EnsureLookAhead();
    let save = s.Save();
    let offset = match ConsumeLiteral(
        id,
        s,
        mode,
        Grammar::Length {
            percent: true,
            nonnegative: false,
            quirks: false,
            keywords: &[],
        },
    ) {
        Ok(v) => v,
        Err(e) if e.kind == PropertyParseErrorKind::Unsupported => return Err(e),
        Err(_) => {
            s.EnsureLookAhead();
            s.Restore(save);
            return Ok(crate::production_reflect_value::reflect(
                direction,
                values::numeric(0.0, UnitType::kPixels),
                None,
            ));
        }
    };
    let mask = if at_value_end(s) {
        None
    } else {
        Some(border_image_parser::ConsumeWebkitBorderImage(id, s, mode)?)
    };
    Ok(crate::production_reflect_value::reflect(
        direction, offset, mask,
    ))
}

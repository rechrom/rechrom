// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! longhands_custom.cc:8061-8077,8100-8112,8131-8141. The experimental
//! CSSViewTransitionAutoName flag (runtime_enabled_features.json5:2118-2120)
//! is disabled at stable defaults. ident() remains a typed collaborator.
#![allow(non_snake_case)]
use super::*;
pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kViewTransitionName
            | CSSPropertyID::kViewTransitionClass
            | CSSPropertyID::kViewTransitionGroup
    )
}
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    let keyword = s.Peek().Id();
    match id {
        CSSPropertyID::kViewTransitionName => {
            if keyword == CSSValueID::kAuto {
                return Err(invalid(id));
            }
            if matches!(keyword, CSSValueID::kNone | CSSValueID::kMatchElement) {
                return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
            }
            list_counter_parser::CustomIdentWithNone(id, s, true)
        }
        CSSPropertyID::kViewTransitionClass => {
            if keyword == CSSValueID::kNone {
                return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
            }
            let mut names = Vec::new();
            loop {
                names.push(list_counter_parser::CustomIdentWithNone(id, s, false)?);
                if at_value_end(s) {
                    break;
                }
            }
            Ok(values::list(names, values::ListSeparator::Space))
        }
        CSSPropertyID::kViewTransitionGroup => {
            if matches!(
                keyword,
                CSSValueID::kNormal | CSSValueID::kNearest | CSSValueID::kContain
            ) {
                return Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()));
            }
            list_counter_parser::CustomIdentWithNone(id, s, true)
        }
        _ => Err(invalid(id)),
    }
}

// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Logical border shorthands reuse the production color/style/width consumers.
// Selected Chromium source audit: effective 469, mapped 457, omitted 10,
// remaining 2. Parser/expand 292/282/8/2; generated metadata 96/96/0/0;
// existing physical mapping 81/79/2/0. Effective lines exclude comments,
// blank/preprocessor/namespace/using/visibility/brace-only lines. Shared
// leaf color/style/width consumers, CSSMath, native Apply/length/color service
// bodies and WritingDirectionMode implementation are outside this selected
// expansion audit and remain partial collaborators in both ledgers. All six
// properties remain partial overall; complete expansion is not complete CSS.
//
// Source ranges under third_party/blink/renderer/core:
// css/properties/shorthands/shorthands_custom.cc:
//   574-597,616-634,848-871,890-908 (six new shorthands);
//   554-562,636-644,656-664,828-836,910-918,930-938
//   (existing one/two-value component shorthand consumers reused unchanged).
// css/properties/css_parsing_utils.cc:
//   1030-1037,2779-2824,3966-3992,4188-4225,4288-4356;
// css/parser/css_property_parser.cc:419-467 (existing CSS-wide expansion).
// Remaining shared helper: utils 4335-4336, alternate use_initial_value_function
// branch. This cluster uses the source default false and native CSSInitialValue.
// Omitted DCHECK/unselected overflow counter: utils 3972-3974,4195,
//   4212-4214,4296,4351.
//
// Generated out/Min/gen/.../core/style_property_shorthand.cc:
//   176-189,191-200,202-212,214-224,226-235,237-246,
//   287-300,302-311,313-323,325-335,337-346,348-357.
// Existing generated .../core/css/properties/longhands.cc ToPhysicalInternal:
//   3909-3913,3950-3954,3985-3989,4020-4024,4061-4065,4096-4100,
//   4677-4681,4718-4722,4753-4757,4788-4792,4829-4833,4864-4868;
// css/properties/css_direction_aware_resolver.cc:
//   76-81,88-93,131-133,144-146,194-196,336-359;
// omitted DCHECK 80,91. No second native border application is introduced.
#![allow(non_snake_case)]
use super::*;

pub(super) fn IsShorthand(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kBorderBlock
            | kBorderInline
            | kBorderBlockStart
            | kBorderBlockEnd
            | kBorderInlineStart
            | kBorderInlineEnd
    )
}

// shorthands_custom.cc:574-597,616-634,848-871,890-908;
// css_parsing_utils.cc:2779-2824,4288-4356.
pub(super) fn ParseShorthand<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    use CSSPropertyID::*;
    if !matches!(id, kBorderBlock | kBorderInline) {
        return ConsumeShorthandGreedilyViaLonghands(id, s, mode, out);
    }
    // ConsumeBorderShorthand parses one width/style/color triple. Logical
    // start longhands use exactly its no-quirks width/color paths in this
    // shorthand context. They share native values and the existing consumers.
    let start = if id == kBorderBlock {
        kBorderBlockStart
    } else {
        kBorderInlineStart
    };
    let components = ShorthandFor(start);
    let mut parsed: [Option<Rc<Value>>; 3] = [None, None, None];
    while parsed.iter().any(Option::is_none) {
        let mut found = false;
        let mut pending = None;
        for (index, &property) in components.iter().enumerate() {
            if parsed[index].is_some() {
                continue;
            }
            s.EnsureLookAhead();
            let saved = s.Save();
            match ConsumeLonghand(property, s, mode) {
                Ok(value) => {
                    parsed[index] = Some(value);
                    // Chromium ConsumeBorderShorthand consumes one optional
                    // comma after every component, including the final one.
                    if s.Peek().GetType() == kCommaToken {
                        s.ConsumeIncludingWhitespace();
                    }
                    found = true;
                    break;
                }
                Err(error) => {
                    s.Restore(saved);
                    if error.kind == PropertyParseErrorKind::Unsupported && pending.is_none() {
                        pending = Some(error);
                    }
                }
            }
        }
        if !found {
            if let Some(error) = pending {
                return Err(error);
            }
            break;
        }
    }
    if parsed.iter().all(Option::is_none) {
        return Err(invalid(id));
    }
    // The two-axis source uses concrete defaults, and each expanded longhand
    // records the intermediate width/style/color shorthand as its origin.
    let shorthands = if id == kBorderBlock {
        [kBorderBlockWidth, kBorderBlockStyle, kBorderBlockColor]
    } else {
        [kBorderInlineWidth, kBorderInlineStyle, kBorderInlineColor]
    };
    let defaults = [
        CSSValueID::kMedium,
        CSSValueID::kNone,
        CSSValueID::kCurrentcolor,
    ];
    for ((shorthand, value), default) in shorthands.into_iter().zip(parsed).zip(defaults) {
        let value = value.unwrap_or_else(|| values::identifier(default));
        for &longhand in ShorthandFor(shorthand) {
            out.push(make_expanded(longhand, shorthand, value.clone(), false));
        }
    }
    Ok(())
}

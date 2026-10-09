// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Typed Chromium interaction/render-hint longhand consumers.
#![allow(non_snake_case)]
use super::*;

pub(super) fn IsInteractionProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kTouchAction
            | kWillChange
            | kAppearance
            | kImageOrientation
            | kHangingPunctuation
            | kMarginTrim
    )
}

pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSPropertyID::*;
    use CSSValueID::*;
    match id {
        // longhands_custom.cc:10398-10452. One alternative from each axis;
        // canonical order is x, y, pinch-zoom, regardless of input order.
        kTouchAction => {
            let first = s.Peek().Id();
            if matches!(first, kAuto | kNone | kManipulation) {
                s.ConsumeIncludingWhitespace();
                return Ok(values::list(
                    vec![values::identifier(first)],
                    values::ListSeparator::Space,
                ));
            }
            let mut axes = [None; 3];
            loop {
                let k = s.Peek().Id();
                let axis = match k {
                    kPanX | kPanLeft | kPanRight => 0,
                    kPanY | kPanUp | kPanDown => 1,
                    kPinchZoom => 2,
                    _ => break,
                };
                if axes[axis].is_some() {
                    break;
                }
                axes[axis] = Some(k);
                s.ConsumeIncludingWhitespace();
            }
            let items = axes
                .into_iter()
                .flatten()
                .map(values::identifier)
                .collect::<Vec<_>>();
            if items.is_empty() {
                return Err(invalid(id));
            }
            Ok(values::list(items, values::ListSeparator::Space))
        }
        // longhands_custom.cc:12432-12480. Tokens must be identifiers; the
        // shared custom-ident consumer enforces default and CSS-wide exclusions.
        kWillChange => {
            if s.Peek().Id() == kAuto {
                s.ConsumeIncludingWhitespace();
                return Ok(values::identifier(kAuto));
            }
            let mut items = Vec::new();
            loop {
                let token = s.Peek().clone();
                if token.GetType() != kIdentToken {
                    return Err(invalid(id));
                }
                if matches!(token.Id(), kContents | kScrollPosition) {
                    items.push(values::identifier(s.ConsumeIncludingWhitespace().Id()));
                } else {
                    if matches!(token.Id(), kNone | CSSValueID::kAll | kAuto) {
                        return Err(invalid(id));
                    }
                    let name = token.Value().ToString().Utf8();
                    if FindProperty(name.to_ascii_lowercase().as_bytes())
                        .is_some_and(|p| p.id_and_exposed_bit == kWillChange as i32)
                    {
                        return Err(invalid(id));
                    }
                    items.push(ConsumeAnimationName(id, s, false)?.ok_or_else(|| invalid(id))?);
                }
                if s.Peek().GetType() != kCommaToken {
                    break;
                }
                s.ConsumeIncludingWhitespace();
            }
            Ok(values::list(items, values::ListSeparator::Comma))
        }
        // longhands_custom.cc:11007-11023; css_parser_fast_paths.cc:1142-1150,
        //1467-1488; css_parser.cc:197-204,256-260 normalize aliases before
        // the keyword fast path. The production entry already passes resolved
        // IDs, preserving the actual webkit-appearance declaration behavior
        // covered by WPT css/css-ui/webkit-appearance-parsing.html:12-41.
        kAppearance => {
            let k = s.Peek().Id();
            if !crate::css_value_keywords::IsValueAllowedInMode(k, mode) {
                return Err(invalid(id));
            }
            let valid = matches!(
                k,
                kNone
                    | kAuto
                    | kCheckbox
                    | kRadio
                    | kButton
                    | kListbox
                    | kInternalMediaControl
                    | kMenulist
                    | kMenulistButton
                    | kMeter
                    | kProgressBar
                    | kSearchfield
                    | kTextfield
                    | kTextarea
                    | kBaseSelect
                    | kSliderVertical
            ) || k == kBase
                && foundation::RuntimeEnabledFeatures::AppearanceBaseEnabled();
            if !valid {
                return Err(invalid(id));
            }
            Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()))
        }
        // longhands_custom.cc:5330-5336. Legacy angles/flip are not accepted.
        kImageOrientation => {
            if !matches!(s.Peek().Id(), kFromImage | kNone) {
                return Err(invalid(id));
            }
            Ok(values::identifier(s.ConsumeIncludingWhitespace().Id()))
        }
        // longhands_custom.cc:5211-5216; css_parsing_utils.cc:7854-7886.
        kHangingPunctuation => {
            if s.Peek().Id() == kNone {
                s.ConsumeIncludingWhitespace();
                return Ok(values::identifier(kNone));
            }
            let mut seen = [false; 3];
            let mut items = Vec::new();
            loop {
                let k = s.Peek().Id();
                let index = match k {
                    kFirst => 0,
                    kAllowEnd => 1,
                    kLast => 2,
                    _ => break,
                };
                if seen[index] {
                    break;
                }
                seen[index] = true;
                items.push(values::identifier(s.ConsumeIncludingWhitespace().Id()));
            }
            if items.is_empty() {
                return Err(invalid(id));
            }
            Ok(values::list(items, values::ListSeparator::Space))
        }
        // longhands_custom.cc:6871-6929. Collapse both block sides to block.
        kMarginTrim => {
            if s.Peek().Id() == kNone {
                s.ConsumeIncludingWhitespace();
                return Ok(values::identifier(kNone));
            }
            let mut mask = 0;
            loop {
                let k = s.Peek().Id();
                let bits = match k {
                    kBlock if mask == 0 => 3,
                    kBlockStart => 1,
                    kBlockEnd => 2,
                    _ => break,
                };
                if mask & bits != 0 {
                    break;
                }
                mask |= bits;
                s.ConsumeIncludingWhitespace();
                if k == kBlock {
                    break;
                }
            }
            let k = match mask {
                1 => kBlockStart,
                2 => kBlockEnd,
                3 => kBlock,
                _ => return Err(invalid(id)),
            };
            Ok(values::list(
                vec![values::identifier(k)],
                values::ListSeparator::Space,
            ))
        }
        _ => Err(invalid(id)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parse(id: CSSPropertyID, text: &str) -> Result<Rc<Value>, PropertyParseError> {
        let mut s: Stream = Stream::new(StringView::from(text), 0);
        s.ConsumeWhitespace();
        let v = Consume(id, &mut s, CSSParserMode::kHTMLStandardMode)?;
        if !s.AtEnd() {
            return Err(invalid(id));
        }
        Ok(v)
    }
    #[test]
    fn production_interaction_disabled_consumers_match_source_combinations_and_canonicalization() {
        for text in ["last first allow-end", "allow-end last", "none"] {
            assert_eq!(
                parse(CSSPropertyID::kHangingPunctuation, text)
                    .unwrap()
                    .CssText()
                    .Utf8(),
                text
            );
        }
        for text in [
            "first first",
            "force-end",
            "none first",
            "last allow-end last",
            "",
        ] {
            assert!(parse(CSSPropertyID::kHangingPunctuation, text).is_err());
        }
        assert_eq!(
            parse(CSSPropertyID::kMarginTrim, "block-end block-start")
                .unwrap()
                .CssText()
                .Utf8(),
            "block"
        );
        for text in ["none", "block", "block-start", "block-end"] {
            assert_eq!(
                parse(CSSPropertyID::kMarginTrim, text)
                    .unwrap()
                    .CssText()
                    .Utf8(),
                text
            );
        }
        for text in [
            "block block-start",
            "block-start block",
            "block-end block-end",
            "inline",
            "none block",
            "",
        ] {
            assert!(parse(CSSPropertyID::kMarginTrim, text).is_err());
        }
    }
}

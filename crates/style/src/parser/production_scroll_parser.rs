// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium scrollbar, overscroll, snap and logical scroll-side consumers.
#![allow(non_snake_case)]
use super::*;

pub(super) fn IsScrollProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kScrollbarColor
            | kScrollbarGutter
            | kOverscrollBehaviorX
            | kOverscrollBehaviorY
            | kScrollSnapAlign
            | kScrollSnapType
            | kScrollMarginTop
            | kScrollMarginRight
            | kScrollMarginBottom
            | kScrollMarginLeft
            | kScrollMarginBlockStart
            | kScrollMarginBlockEnd
            | kScrollMarginInlineStart
            | kScrollMarginInlineEnd
            | kScrollPaddingTop
            | kScrollPaddingRight
            | kScrollPaddingBottom
            | kScrollPaddingLeft
            | kScrollPaddingBlockStart
            | kScrollPaddingBlockEnd
            | kScrollPaddingInlineStart
            | kScrollPaddingInlineEnd
    )
}
pub(super) fn IsScrollShorthand(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kOverscrollBehavior
            | kScrollMarginBlock
            | kScrollMarginInline
            | kScrollPaddingBlock
            | kScrollPaddingInline
    )
}
fn IsPadding(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kScrollPaddingTop
            | kScrollPaddingRight
            | kScrollPaddingBottom
            | kScrollPaddingLeft
            | kScrollPaddingBlockStart
            | kScrollPaddingBlockEnd
            | kScrollPaddingInlineStart
            | kScrollPaddingInlineEnd
    )
}
fn Ident<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    words: &[&str],
) -> Result<Rc<Value>, PropertyParseError> {
    if !matches(stream.Peek().Id(), words) {
        return Err(invalid(id));
    }
    Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()))
}
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSPropertyID::*;
    use CSSValueID::*;
    match id {
        // longhands_custom.cc:8804-8830.
        kScrollbarColor => {
            if stream.Peek().Id() == kAuto {
                return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
            }
            let thumb = ConsumeColor(id, stream)?;
            let track = ConsumeColor(id, stream)?;
            Ok(values::list(
                vec![thumb, track],
                values::ListSeparator::Space,
            ))
        }
        // longhands_custom.cc:8852-8891; canonical stable/both-edges ordering.
        kScrollbarGutter => {
            if stream.Peek().Id() == kAuto {
                return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
            }
            let mut stable = false;
            let mut both_edges = false;
            loop {
                match stream.Peek().Id() {
                    kStable if !stable => stable = true,
                    kBothEdges if !both_edges => both_edges = true,
                    _ => break,
                }
                stream.ConsumeIncludingWhitespace();
            }
            if !stable {
                return Err(invalid(id));
            }
            if both_edges {
                Ok(values::list(
                    vec![values::identifier(kStable), values::identifier(kBothEdges)],
                    values::ListSeparator::Space,
                ))
            } else {
                Ok(values::identifier(kStable))
            }
        }
        kOverscrollBehaviorX | kOverscrollBehaviorY => {
            Ident(id, stream, &["auto", "chain", "contain", "none"])
        }
        // longhands_custom.cc:9086-9108.
        kScrollSnapAlign => {
            let words = &["none", "start", "end", "center"];
            let block = Ident(id, stream, words)?;
            if !matches(stream.Peek().Id(), words) {
                return Ok(block);
            }
            let inline = Ident(id, stream, words)?;
            Ok(Rc::new(Value::new(CSSValuePayload::kValuePairClass(
                values::CSSValuePair {
                    first: block,
                    second: inline,
                    drop_identical: true,
                },
            ))))
        }
        // longhands_custom.cc:9127-9159. pair axis is an experimental runtime branch.
        kScrollSnapType => {
            if stream.Peek().Id() == kPair {
                return Err(unsupported(
                    id,
                    "ScrollSnapType CSSScrollSnapTypePair runtime feature",
                ));
            }
            let axis = Ident(id, stream, &["none", "x", "y", "block", "inline", "both"])?;
            if matches!(axis.Payload(), CSSValuePayload::kIdentifierClass(k) if k.0 == kNone) {
                return Ok(axis);
            }
            if stream.Peek().Id() == kProximity {
                stream.ConsumeIncludingWhitespace();
                return Ok(axis);
            }
            if stream.Peek().Id() != kMandatory {
                return Ok(axis);
            }
            let strictness = values::identifier(stream.ConsumeIncludingWhitespace().Id());
            Ok(Rc::new(Value::new(CSSValuePayload::kValuePairClass(
                values::CSSValuePair {
                    first: axis,
                    second: strictness,
                    drop_identical: true,
                },
            ))))
        }
        // longhands_custom.cc:8946-9041; css_parsing_utils.cc:8972-8982.
        // ScrollPadding overrides to standard mode; ScrollMargin preserves the
        // source context, whose SVG mode accepts unitless user units.
        _ if IsScrollProperty(id) => {
            if !IsPadding(id) && mode == CSSParserMode::kSVGAttributeMode && IsMathFunction(stream)
            {
                use crate::css_math_expression_node::CalculationResultCategory as C;
                return ConsumeMath(
                    id,
                    stream,
                    &[C::Length, C::Number],
                    crate::css_math_function_value::ValueRange::All,
                );
            }
            ConsumeLiteral(
                id,
                stream,
                if IsPadding(id) {
                    CSSParserMode::kHTMLStandardMode
                } else {
                    mode
                },
                Grammar::Length {
                    percent: IsPadding(id),
                    nonnegative: IsPadding(id),
                    quirks: false,
                    keywords: if IsPadding(id) { &["auto"] } else { &[] },
                },
            )
        }
        _ => Err(unsupported(id, "scroll property consumer")),
    }
}
// css_parsing_utils.cc:4188-4225; shorthands_custom.cc:4529-4537,
// 4943-4951,4983-4991,5003-5011,5043-5051.
pub(super) fn ParseShorthand<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    let longhands = ShorthandFor(id);
    let first = ConsumeLonghand(longhands[0], stream, mode)?;
    stream.EnsureLookAhead();
    let save = stream.Save();
    let second = match ConsumeLonghand(longhands[1], stream, mode) {
        Ok(value) => value,
        Err(failure) if failure.kind == PropertyParseErrorKind::Unsupported => return Err(failure),
        Err(_) => {
            stream.Restore(save);
            first.clone()
        }
    };
    out.push(make_expanded(longhands[0], id, first, false));
    out.push(make_expanded(longhands[1], id, second, false));
    Ok(())
}

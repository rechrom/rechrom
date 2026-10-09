// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Typed legacy clip, zoom and page/locale consumers from Blink longhands.
// Selected Chromium source audit: effective 561, mapped 454, omitted 22,
// remaining 85. Parser 275/256/8/11; Apply/converter 188/170/11/7;
// native/assembly 37/28/2/7; missing page owner 61/0/1/60.
// Includes genuine shared consumers/native methods reused by this cluster.
// Effective lines exclude blank/comments/preprocessor/namespace/using/
// visibility/brace-only lines. Nested CSSMath and conversion services,
// generated native fields, CSSQuadValue implementation, font update backend,
// CSSParserImpl @page entry and page-margin/pagination owners are outside
// this selected scope; their dependencies remain explicit in both ledgers.
// Partial branches do not count as complete properties.
//
// Source ranges under third_party/blink/renderer (core unless platform):
// Parser css/properties/longhands/longhands_custom.cc:
//   2201-2210,2214-2254,7274-7293,8040-8048,9327-9333,9368-9411,
//   11344-11352,12722-12750; css/properties/css_parsing_utils.cc:
//   1262-1284,1286-1292,1294-1392,1394-1418,1757-1772,1803-1809.
// Parser remaining: custom 7281-7289 (shape owner); utils 1760-1763
// (CSSIdentFunction test runtime flag, stable disabled). Omitted custom
// 12740-12748 (usage counter) and utils 1399 (DCHECK).
//
// Apply out/Min/gen/.../core/css/properties/longhands.cc:
//   1118-1123,5895-5915,11738-11751,12985-12993;
// custom longhands.cc:9334-9366,9413-9484,11365-11375,
//   12761-12763,12765-12767,12769-12773;
// css/resolver/style_builder_converter.cc:
//   342-350,2063-2067,2087-2109,2111-2120,3501-3509,3761-3769.
// Apply remaining: generated 5899-5903,11742-11746 (stable enabled
// StandardizedBrowserZoom parent reconversion); converter 3508 (shape).
// Omitted: custom 9364,9442-9445,9453,11369; converter 2089,2108,
// 3766-3767 (DCHECK/NOTREACHED).
//
// Native/assembly css/resolver/style_resolver_state.cc:325-339;
// css/resolver/font_builder.cc:43-45,149-153;
// style/computed_style.cc:3102-3111; style/computed_style.h:3104-3111;
// platform/text/layout_locale.cc:277-291.
// Production remaining: state 336-338, font_builder 43-44,150 (the generic
// translation exists, but production has no dirty/update-font owner);
// locale 280,287 (existing native ICU script/per-thread cache owner partial).
// Omitted state 332-334 usage counter. Page owner:
// css/resolver/style_resolver.cc:2182-2270 entirely remaining except
// omitted DCHECK 2191. @page must remain typed Unsupported at document entry;
// direct Size native application is tested without constructing fake page rules.
// Page's source/native field is AtomicString, not ScopedCSSName.
#![allow(non_snake_case)]
use super::*;

pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kZoom | kClip | kSize | kPage | kWebkitLocale | kObjectViewBox
    )
}

fn Length<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    nonnegative: bool,
    quirks: bool,
    keywords: &'static [&'static str],
) -> Result<Rc<Value>, PropertyParseError> {
    // ConsumeLength's SVG mode forbids calc(), unlike ConsumeLengthOrPercent.
    if mode == CSSParserMode::kSVGAttributeMode && stream.Peek().GetType() == kFunctionToken {
        return Err(invalid(id));
    }
    ConsumeLiteral(
        id,
        stream,
        mode,
        Grammar::Length {
            percent: false,
            nonnegative,
            quirks,
            keywords,
        },
    )
}

pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSPropertyID::*;
    match id {
        // longhands_custom.cc:12722-12750. The use counter is omitted.
        kZoom => {
            if stream.Peek().GetType() == kIdentToken {
                return ConsumeLiteral(id, stream, mode, Grammar::Keywords(&["normal"]));
            }
            if IsMathFunction(stream) {
                use crate::css_math_expression_node::CalculationResultCategory as C;
                return ConsumeMath(
                    id,
                    stream,
                    &[C::Percent, C::Number],
                    crate::css_math_function_value::ValueRange::NonNegative,
                );
            }
            if stream.Peek().GetType() == kPercentageToken {
                let percent = stream.Peek().NumericValue();
                if percent < 0. {
                    return Err(invalid(id));
                }
                stream.ConsumeIncludingWhitespace();
                return Ok(values::numeric(percent, UnitType::kPercentage));
            }
            ConsumeLiteral(id, stream, mode, Grammar::Number { nonnegative: true })
        }
        // longhands_custom.cc:2201-2210,2214-2254.
        kClip => {
            if stream.Peek().Id() == CSSValueID::kAuto {
                return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
            }
            if stream.Peek().FunctionId() != Some(CSSValueID::kRect) {
                return Err(invalid(id));
            }
            let value;
            {
                let mut guard = RestoringBlockGuard::new(stream);
                guard.ConsumeWhitespace();
                let top = Length(id, &mut guard, mode, false, true, &["auto"])?;
                let comma = guard.Peek().GetType() == kCommaToken;
                if comma {
                    guard.ConsumeIncludingWhitespace();
                }
                let right = Length(id, &mut guard, mode, false, true, &["auto"])?;
                if comma {
                    if guard.Peek().GetType() != kCommaToken {
                        return Err(invalid(id));
                    }
                    guard.ConsumeIncludingWhitespace();
                }
                let bottom = Length(id, &mut guard, mode, false, true, &["auto"])?;
                if comma {
                    if guard.Peek().GetType() != kCommaToken {
                        return Err(invalid(id));
                    }
                    guard.ConsumeIncludingWhitespace();
                }
                let left = Length(id, &mut guard, mode, false, true, &["auto"])?;
                if !guard.AtEnd() || !guard.Release() {
                    return Err(invalid(id));
                }
                value=Rc::new(Value::new(CSSValuePayload::kQuadClass(values::CSSQuadValue {
                    sides:[top,right,bottom,left], serialization_type:crate::production_border_image_values::TypeForSerialization::kSerializeAsRect,
                })));
            }
            stream.ConsumeWhitespace();
            Ok(value)
        }
        kSize => ConsumeSize(stream, mode),
        // longhands_custom.cc:8040-8048: this property converts to AtomicString,
        // not ScopedCSSName. Generic ident() remains a distinct runtime owner.
        kPage => {
            if stream.Peek().Id() == CSSValueID::kAuto {
                return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
            }
            if stream.Peek().FunctionId() == Some(CSSValueID::kIdent) {
                // CSSIdentFunction is test-only and disabled at stable defaults.
                return Err(invalid(id));
            }
            if stream.Peek().GetType() != kIdentToken
                || stream.Peek().Id() == CSSValueID::kDefault
                || values::wide(stream.Peek().Id()).is_some()
            {
                return Err(invalid(id));
            }
            Ok(values::custom_ident(
                &stream.ConsumeIncludingWhitespace().Value().ToString(),
                CSSPropertyID::kInvalid,
            ))
        }
        // longhands_custom.cc:11344-11352.
        kWebkitLocale => {
            if stream.Peek().Id() == CSSValueID::kAuto {
                return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
            }
            if stream.Peek().GetType() != kStringToken {
                return Err(invalid(id));
            }
            Ok(values::string(
                stream.ConsumeIncludingWhitespace().Value().ToString(),
            ))
        }
        // longhands_custom.cc:7274-7293. Only native none can be assembled:
        // the actual inset/rect/xywh BasicShape/Path owners remain absent.
        kObjectViewBox => {
            if stream.Peek().Id() == CSSValueID::kNone {
                return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
            }
            if matches!(
                stream.Peek().FunctionId(),
                Some(CSSValueID::kInset | CSSValueID::kRect | CSSValueID::kXywh)
            ) {
                return Err(unsupported(
                    id,
                    "ObjectViewBox BasicShapeInset/Rect/XYWH Path owner",
                ));
            }
            Err(invalid(id))
        }
        _ => Err(invalid(id)),
    }
}

fn PageSize<T: TokenStreamTokenizer>(stream: &mut Stream<T>) -> Option<Rc<Value>> {
    if matches!(
        stream.Peek().Id(),
        CSSValueID::kA3
            | CSSValueID::kA4
            | CSSValueID::kA5
            | CSSValueID::kB4
            | CSSValueID::kB5
            | CSSValueID::kJisB4
            | CSSValueID::kJisB5
            | CSSValueID::kLedger
            | CSSValueID::kLegal
            | CSSValueID::kLetter
    ) {
        Some(values::identifier(stream.ConsumeIncludingWhitespace().Id()))
    } else {
        None
    }
}
fn ConsumeSize<T: TokenStreamTokenizer>(
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    // longhands_custom.cc:9327-9333,9368-9411; canonical page-size before
    // orientation and redundant portrait omitted exactly as the source.
    let id = CSSPropertyID::kSize;
    if stream.Peek().Id() == CSSValueID::kAuto {
        return Ok(values::list(
            vec![values::identifier(stream.ConsumeIncludingWhitespace().Id())],
            values::ListSeparator::Space,
        ));
    }
    if stream.Peek().GetType() != kIdentToken {
        let width = Length(id, stream, mode, true, false, &[])?;
        stream.EnsureLookAhead();
        let save = stream.Save();
        let height = Length(id, stream, mode, true, false, &[]);
        let mut items = vec![width];
        if let Ok(height) = height {
            items.push(height);
        } else {
            stream.Restore(save);
        }
        return Ok(values::list(items, values::ListSeparator::Space));
    }
    let mut page = PageSize(stream);
    let orientation = if matches!(
        stream.Peek().Id(),
        CSSValueID::kPortrait | CSSValueID::kLandscape
    ) {
        Some(stream.ConsumeIncludingWhitespace().Id())
    } else {
        None
    };
    if page.is_none() {
        page = PageSize(stream);
    }
    if page.is_none() && orientation.is_none() {
        return Err(invalid(id));
    }
    let has_page = page.is_some();
    let mut items = page.into_iter().collect::<Vec<_>>();
    if let Some(orientation) = orientation {
        if !has_page || orientation != CSSValueID::kPortrait {
            items.push(values::identifier(orientation));
        }
    }
    Ok(values::list(items, values::ListSeparator::Space))
}

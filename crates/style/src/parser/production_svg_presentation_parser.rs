// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Remaining SVG presentation longhands and their genuine source consumers.
// Selected Chromium source audit: effective 732, mapped 546, omitted 15,
// remaining 171. Parser: 454/365/9/80; Apply/converter: 278/181/6/91.
// Includes existing typed helpers reused by this cluster. Effective lines
// exclude comments/blank/preprocessor/namespace/using/visibility/brace-only
// lines. Nested CSSMath, ColorFunctionParser, ResolveColorValue wrapper,
// keyword tables and native generated field bodies are outside this scope;
// their dependencies remain explicit in both ledgers. Partial is not complete.
//
// Parser ranges (under third_party/blink/renderer/core unless generated):
// css/properties/longhands/longhands_custom.cc:
//   1242-1254,3585-3590,6950-6958,6976-6984,7002-7010,9640-9647,12194-12202;
// out/Min/gen/.../core/css/properties/longhands.cc:19251-19256;
// css/css_properties.json5:6103,6117 (the actual generated keyword lists);
// css/properties/shorthands/shorthands_custom.cc:4353-4375,6458-6466;
// css/properties/css_parsing_utils.cc:
//   1262-1281,1283-1292,1294-1392,1394-1418,1453-1500,1834-1842,
//   1981-2038,2355-2366,2400-2417,2421-2485,2516-2528,4288-4345,
//   8361-8375,8403-8439,9030-9037.
// Parser remaining in utils:
//   1478-1491,1837-1841,2008-2009,2024-2026,2031,2037,2400-2417,
//   2425-2440,2444-2446,2452-2456,2474-2482,8368-8373,8410,8415-8416,
//   8424-8438. Omitted utils DCHECK/unselected InitialValue branch:
//   1399,1998-2000,2015,4296,4335-4337.
//
// Apply ranges in generated longhands.cc:
//   3777-3779,7621-7629,10613-10618,10639-10644,10665-10670,
//   15824-15832,15853-15861,15882-15890,19258-19260,19262-19264,
//   19266-19268,19289-19291,19292-19299,19300-19302;
// custom longhands.cc:1273-1283,1285-1311,6968-6973,6994-6999,7020-7025;
// css/resolver/style_builder_converter.cc:
//   270-283,2063-2067,2898-3010,3054-3070,3224-3235,3450-3458;
// css/resolver/style_builder_converter.h:504-539;
// css/css_identifier_value_mappings.h:1094-1108,1126-1140.
// Apply remaining: generated 19293-19297; custom 1274-1278;
// converter.cc:279-282,2902,2911-2914,2917-2918,2921-2922,2927-3009,
//   3058-3068,3452-3454. Omitted DCHECK/NOTREACHED: custom1302;
//   converter.cc275,3456; converter.h519; identifier mappings1107,1139.
//
// Stable defaults: StandardizedBrowserZoom, CSSURLRequestModifiers,
// CSSAlphaColorFunction and CSSContrastColor are enabled in Chromium.
// Their missing owners stay pending, rather than pretending the flags are off.
#![allow(non_snake_case)]
use super::*;

pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kD | kMarkerStart
            | kMarkerMid
            | kMarkerEnd
            | kBaselineShift
            | kStrokeLinecap
            | kStrokeLinejoin
            | kStrokeMiterlimit
            | kWebkitTextStrokeWidth
            | kWebkitTextStrokeColor
    )
}

pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    use CSSPropertyID::*;
    match id {
        // longhands_custom.cc:3585-3590,6950-6958,6976-6984,7002-7010.
        kD | kMarkerStart | kMarkerMid | kMarkerEnd => {
            if stream.Peek().Id() == CSSValueID::kNone {
                return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
            }
            if id == kD {
                // css_parsing_utils.cc:8361-8375,8403-8437,9030-9037:
                // production CSSPathValue/SVGPathStringSource/byte-stream builder
                // owner is absent; foundation::Path is uninhabited. Preserve the
                // typed boundary rather than substituting DOM paint commands.
                if stream.Peek().FunctionId() != Some(CSSValueID::kPath) {
                    return Err(invalid(id));
                }
                let mut guard = RestoringBlockGuard::new(stream);
                guard.ConsumeWhitespace();
                if guard.Peek().GetType() != kStringToken {
                    return Err(invalid(id));
                }
                guard.ConsumeIncludingWhitespace();
                if !guard.AtEnd() {
                    return Err(invalid(id));
                }
                return Err(unsupported(
                    id,
                    "ConsumePathStringArg/SVGPathByteStream owner",
                ));
            }
            ConsumeUrl(id, stream)?
                .map(values::uri)
                .ok_or_else(|| invalid(id))
        }
        // longhands_custom.cc:1242-1254, SVG user units even in CSS mode.
        kBaselineShift => {
            if matches!(
                stream.Peek().Id(),
                CSSValueID::kBaseline | CSSValueID::kSub | CSSValueID::kSuper
            ) {
                return Ok(values::identifier(stream.ConsumeIncludingWhitespace().Id()));
            }
            svg_parser::ConsumeLength(
                id,
                stream,
                CSSParserMode::kSVGAttributeMode,
                true,
                false,
                &[],
            )
        }
        kStrokeLinecap => ConsumeLiteral(
            id,
            stream,
            mode,
            Grammar::Keywords(&["butt", "round", "square"]),
        ),
        kStrokeLinejoin => ConsumeLiteral(
            id,
            stream,
            mode,
            Grammar::Keywords(&["miter", "round", "bevel"]),
        ),
        // longhands_custom.cc:9640-9647,12194-12202; generated:19251-19256.
        kStrokeMiterlimit => {
            ConsumeLiteral(id, stream, mode, Grammar::Number { nonnegative: true })
        }
        kWebkitTextStrokeWidth => ConsumeLiteral(
            id,
            stream,
            mode,
            Grammar::Length {
                percent: false,
                nonnegative: true,
                quirks: false,
                keywords: &["thin", "medium", "thick"],
            },
        ),
        kWebkitTextStrokeColor => {
            // css_parsing_utils.cc:2448-2451, actual UA-only color keyword gate.
            if !crate::css_value_keywords::IsValueAllowedInMode(stream.Peek().Id(), mode) {
                return Err(invalid(id));
            }
            ConsumeLiteral(id, stream, mode, Grammar::Color)
        }
        _ => Err(unsupported(
            id,
            "ConsumePathFunction/SVGPathByteStream owner",
        )),
    }
}

// shorthands_custom.cc:4353-4375. All three slots share one parsed CSSValue.
pub(super) fn ParseMarker<T: TokenStreamTokenizer>(
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    let value = Consume(CSSPropertyID::kMarkerStart, stream, mode)?;
    for &id in ShorthandFor(CSSPropertyID::kMarker) {
        out.push(make_expanded(
            id,
            CSSPropertyID::kMarker,
            value.clone(),
            false,
        ));
    }
    Ok(())
}

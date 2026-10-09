// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium stable auto-color and color-scheme consumers.
// Selected-source audit (includes reused entrypoint collaborators; nested
// functional-color/math/named-table/mode-policy/provider/native-field bodies
// are outside these counts, with their missing branches retained in ledgers).
// Effective lines exclude comments/blanks/preprocessing/namespaces/using,
// access labels and punctuation-only lines; count multiline statements by line.
// Scope                         effective mapped omitted remaining
// Parser entries/shared consumers     242    190       0        52
// Native Apply/converters             222    136       7        79
// Scheme native algorithm/assembly     53     53       0         0
// Total selected scope                517    379       7       131
// Parser: longhands_custom.cc:2090-2123,2456-2503,5563-5569,5870-5876,
// 7464-7483; generated longhands.cc:1316-1321,1360-1365,1398-1403,
// 1453-1458,1497-1502,1535-1540,1584-1589,1633-1638;
// css_parsing_utils.cc:1757-1773,2355-2366,2368-2398,2400-2417,
// 2421-2499,9653-9662. Parser remaining: custom:2109-2122; utilities:
// 1759-1763,2400-2417,2425-2440,2444-2446,2452-2456,2474-2482.
// Application: generated:1405-1415,1542-1552,1591-1601,1640-1650,
// 1674-1684,1786-1796,2017-2025,5802-5810; custom:2526-2553,
// 2557-2559,2561-2566,2568-2575; style_builder_converter.cc:2898-3010,
// 3054-3070,3072-3082,3084-3096,3856-3889. Converter remaining:
// 2902,2911-2914,2917-2918,2921-2922,2927-3009,3058-3068.
// Omitted: custom:2545-2552 (root use counter),2572-2573 (DCHECK);
// converter:3089 (DCHECK),3885 (NOTREACHED).
// Native/assembly: computed_style.cc:3147-3190; style_resolver.cc:2398-2400;
// blink/public/mojom/css/preferred_color_scheme.mojom:7-10;
// generated longhands.cc:1306-1310,1350-1354,1443-1447,1487-1491;
// css_direction_aware_resolver.cc:326-334. Page settings are real typed host
// inputs; page/meta publication and visited-history ownership remain adapters,
// outside this direct scope. No full property completion is inferred from it.
// Stable defaults: CSSCaretColorWithOptionalSecondValue=false (experimental),
// CSSIdentFunction=false (test); CSSAlphaColorFunction/CSSContrastColor=true;
// CSSAccentColorKeyword=true on Mac/Win/Linux/ChromeOS (default experimental).
// Enabled advanced/system branches remain typed Unsupported until collaborators
// exist; alternate flags stay in the ledger. Ordinary border/outline reuse the
// same native color converter rather than adding another application path.
#![allow(non_snake_case)]
use super::*;
use CSSPropertyID::*;
use CSSValueID::*;
pub(super) fn IsColorUIProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        kAccentColor
            | kCaretColor
            | kColorScheme
            | kInternalVisitedCaretColor
            | kInternalVisitedOutlineColor
            | kInternalVisitedBorderTopColor
            | kInternalVisitedBorderRightColor
            | kInternalVisitedBorderBottomColor
            | kInternalVisitedBorderLeftColor
            | kInternalVisitedBorderBlockStartColor
            | kInternalVisitedBorderBlockEndColor
            | kInternalVisitedBorderInlineStartColor
            | kInternalVisitedBorderInlineEndColor
    )
}
// cpp: longhands_custom.cc:2456-2503. Keep author custom-ident case/order and
// duplicates. 'only' is accepted at one end and is serialized last.
fn ColorScheme<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
) -> Result<Rc<Value>, PropertyParseError> {
    if stream.Peek().Id() == kNormal {
        stream.ConsumeIncludingWhitespace();
        return Ok(values::identifier(kNormal));
    }
    let mut only = None;
    let mut items = Vec::new();
    loop {
        let token = stream.Peek();
        let keyword = token.Id();
        if keyword == kNormal {
            return Err(invalid(id));
        }
        if keyword == kOnly {
            if only.is_some() {
                return Err(invalid(id));
            }
            let value = values::identifier(stream.ConsumeIncludingWhitespace().Id());
            if !items.is_empty() {
                items.push(value);
                return Ok(values::list(items, values::ListSeparator::Space));
            }
            only = Some(value);
            continue;
        }
        let value = if matches!(keyword, kDark | kLight) {
            values::identifier(stream.ConsumeIncludingWhitespace().Id())
        } else {
            // CSSIdentFunction has status=test, so its alternate branch is
            // outside stable grammar and remains a documented collaborator.
            if token.GetType() != kIdentToken
                || values::wide(keyword).is_some()
                || keyword == kDefault
            {
                break;
            }
            let name = stream.ConsumeIncludingWhitespace().Value().ToString();
            values::custom_ident(&name, CSSPropertyID::kInvalid)
        };
        items.push(value);
        if stream.AtEnd() {
            break;
        }
    }
    if items.is_empty() {
        return Err(invalid(id));
    }
    if let Some(only) = only {
        items.push(only);
    }
    Ok(values::list(items, values::ListSeparator::Space))
}
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    // cpp: css_parsing_utils.cc:2448-2451. Outline's explicit focus-ring
    // consumer precedes ConsumeColor and intentionally bypasses this gate.
    let keyword = stream.Peek().Id();
    if id != kColorScheme
        && !(id == kInternalVisitedOutlineColor && keyword == kWebkitFocusRingColor)
        && !crate::css_value_keywords::IsValueAllowedInMode(keyword, mode)
    {
        return Err(invalid(id));
    }
    match id {
        // cpp: longhands_custom.cc:7475-7483,2090-2123,5563-5569.
        // CSSCaretColorWithOptionalSecondValue is experimental (stable=false).
        kAccentColor | kCaretColor | kInternalVisitedCaretColor => {
            if stream.Peek().Id() == kAuto {
                stream.ConsumeIncludingWhitespace();
                Ok(values::identifier(kAuto))
            } else {
                ConsumeColor(id, stream)
            }
        }
        // cpp: longhands_custom.cc:5870-5876 delegates OutlineColor:7464-7473.
        kInternalVisitedOutlineColor => {
            if stream.Peek().Id() == kWebkitFocusRingColor {
                stream.ConsumeIncludingWhitespace();
                Ok(values::identifier(kWebkitFocusRingColor))
            } else {
                ConsumeColor(id, stream)
            }
        }
        kColorScheme => ColorScheme(id, stream),
        // cpp: generated longhands.cc:1398-1403,1535-1540,1584-1589,
        // 1633-1638; logical surrogates use the same border-side consumer.
        _ => ConsumeBorderColorSide(id, stream, mode),
    }
}

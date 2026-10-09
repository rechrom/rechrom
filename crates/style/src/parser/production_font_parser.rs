// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Stable font consumers; every result is a native CSSValue subtype.
// Source audit of direct entrypoints and converters (nested collaborator bodies
// are outside these counts; their pending branches remain in parser ledger).
// Effective lines exclude comments, blanks, preprocessing/includes/namespaces,
// using declarations and lines containing only braces/parentheses/semicolons.
// Scope                                      effective mapped omitted remaining
// longhands_custom.cc parser entries                 90     88       2         0
// shorthands_custom.cc font entrypoints             293    289       0         4
// css_parsing_utils.cc language/size-adjust           47     47       0         0
// style_builder_converter.cc font + line-height     272    234      16        22
// generated longhands.cc Apply*                      63     60       0         3
// Total direct-entrypoint scope                     765    718      18        29
// Parser ranges: longhands_custom.cc:4464-4471,4543-4551,4561-4585,
// 4595-4620,4630-4654,4664-4688,4762-4768; shorthands_custom.cc:3087-3276,
// 3280-3290,3300-3462; css_parsing_utils.cc:6735-6773,6775-6789,9926-9952.
// Converter ranges: 745-757,981-1029,1185-1216,1218-1268,1270-1317,
// 1319-1383,1385-1432,2200-2235. Omitted: DCHECK/NOTREACHED diagnostics
// (multiline statements count every effective line). Remaining: system-font
// dispatch/values, explicit oblique angle consumption, full line-height adjusted
// zoom/metric collaborators and generated standardized-zoom inheritance branch.
// Stable defaults: CSSFontSizeAdjust=true; FontLanguageOverride=true;
// CSSIdentFunction=false (status=test). Alternate feature branches stay in ledger.
#![allow(non_snake_case)]
use super::*;
use CSSPropertyID::*;
use CSSValueID::*;

pub(super) fn IsFontProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        kFontVariantCaps
            | kFontVariantLigatures
            | kFontVariantNumeric
            | kFontVariantEastAsian
            | kFontVariantAlternates
            | kFontVariantPosition
            | kFontVariantEmoji
            | kFontLanguageOverride
            | kFontSizeAdjust
    )
}
// cpp: font_variant_ligatures_parser.h:28-77; numeric_parser.h:24-74;
// east_asian_parser.h:29-62. The group is the source's mutually exclusive slot.
fn Group(id: CSSPropertyID, value: CSSValueID) -> Option<usize> {
    Some(match id {
        kFontVariantLigatures => match value {
            kCommonLigatures | kNoCommonLigatures => 0,
            kDiscretionaryLigatures | kNoDiscretionaryLigatures => 1,
            kHistoricalLigatures | kNoHistoricalLigatures => 2,
            kContextual | kNoContextual => 3,
            _ => return None,
        },
        kFontVariantNumeric => match value {
            kLiningNums | kOldstyleNums => 0,
            kProportionalNums | kTabularNums => 1,
            kDiagonalFractions | kStackedFractions => 2,
            kOrdinal => 3,
            kSlashedZero => 4,
            _ => return None,
        },
        kFontVariantEastAsian => match value {
            kJis78 | kJis83 | kJis90 | kJis04 | kSimplified | kTraditional => 0,
            kFullWidth | kProportionalWidth => 1,
            kRuby => 2,
            _ => return None,
        },
        kFontVariantCaps => match value {
            kSmallCaps | kAllSmallCaps | kPetiteCaps | kAllPetiteCaps | kUnicase | kTitlingCaps => {
                0
            }
            _ => return None,
        },
        kFontVariantPosition => match value {
            kSub | kSuper => 0,
            _ => return None,
        },
        kFontVariantEmoji => match value {
            kText | kEmoji | kUnicode => 0,
            _ => return None,
        },
        _ => return None,
    })
}
fn Finalize(id: CSSPropertyID, mut items: Vec<(usize, Rc<Value>)>) -> Rc<Value> {
    if items.is_empty() {
        return values::identifier(kNormal);
    }
    // East Asian and alternates are canonicalized by Chromium's finalizers.
    if matches!(id, kFontVariantEastAsian | kFontVariantAlternates) {
        items.sort_by_key(|v| v.0);
    }
    if matches!(
        id,
        kFontVariantCaps | kFontVariantPosition | kFontVariantEmoji
    ) {
        return items.pop().unwrap().1;
    }
    values::list(
        items.into_iter().map(|v| v.1).collect(),
        values::ListSeparator::Space,
    )
}
// cpp: font_variant_alternates_parser.cc:33-108,110-155. Historical forms may
// repeat; functional alternates have one occurrence and custom-ident arguments.
fn ConsumeAlternate<T: TokenStreamTokenizer>(
    stream: &mut Stream<T>,
) -> Result<Option<(usize, Rc<Value>)>, PropertyParseError> {
    let id = kFontVariantAlternates;
    if stream.Peek().Id() == kHistoricalForms {
        stream.ConsumeIncludingWhitespace();
        return Ok(Some((1, values::identifier(kHistoricalForms))));
    }
    let Some(function) = stream.Peek().FunctionId() else {
        return Ok(None);
    };
    let index = match function {
        kStylistic => 0,
        kStyleset => 2,
        kCharacterVariant => 3,
        kSwash => 4,
        kOrnaments => 5,
        kAnnotation => 6,
        _ => return Ok(None),
    };
    let mut aliases = Vec::new();
    {
        let mut guard = RestoringBlockGuard::new(stream);
        guard.ConsumeWhitespace();
        loop {
            let token = guard.Peek();
            if token.GetType() == kFunctionToken {
                return Err(invalid(id));
            }
            if token.GetType() != kIdentToken
                || token.Id() == kDefault
                || values::wide(token.Id()).is_some()
            {
                return Err(invalid(id));
            }
            let name = token.Value().ToString();
            guard.ConsumeIncludingWhitespace();
            aliases.push(values::custom_ident(&name, CSSPropertyID::kInvalid));
            if guard.Peek().GetType() != kCommaToken {
                break;
            }
            guard.ConsumeIncludingWhitespace();
        }
        if !guard.AtEnd()
            || (aliases.len() > 1 && !matches!(function, kStyleset | kCharacterVariant))
        {
            return Err(invalid(id));
        }
        guard.Release();
    }
    stream.ConsumeWhitespace();
    Ok(Some((index, values::alternate(function, aliases))))
}
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    // cpp: css_parsing_utils.cc:6735-6789. Validate original UTF-16 length before
    // trimming trailing CSS whitespace; preserve case and do not pad tags.
    if id == kFontLanguageOverride {
        if stream.Peek().Id() == kNormal {
            stream.ConsumeIncludingWhitespace();
            return Ok(values::identifier(kNormal));
        }
        if stream.Peek().GetType() != kStringToken {
            return Err(invalid(id));
        }
        let text = stream
            .ConsumeIncludingWhitespace()
            .Value()
            .ToString()
            .Utf8();
        if text.encode_utf16().count() > 4 || !text.is_ascii() {
            return Err(invalid(id));
        }
        let text = text.trim_end_matches([' ', '\t', '\n', '\r', '\x0c']);
        if text.is_empty() {
            return Err(invalid(id));
        }
        return Ok(values::string(String::from(text)));
    }
    // cpp: css_parsing_utils.cc:9926-9953. ex-height is the implicit metric.
    if id == kFontSizeAdjust {
        if stream.Peek().Id() == kNone {
            stream.ConsumeIncludingWhitespace();
            return Ok(values::identifier(kNone));
        }
        let metric = stream.Peek().Id();
        let has_metric = matches!(
            metric,
            kExHeight | kCapHeight | kChWidth | kIcWidth | kIcHeight
        );
        if has_metric {
            stream.ConsumeIncludingWhitespace();
        }
        let value = if stream.Peek().Id() == kFromFont {
            stream.ConsumeIncludingWhitespace();
            values::identifier(kFromFont)
        } else if is_function(stream) {
            ConsumeMath(
                id,
                stream,
                &[crate::css_math_expression_node::CalculationResultCategory::Number],
                crate::css_math_function_value::ValueRange::NonNegative,
            )?
        } else {
            ConsumeLiteral(id, stream, mode, Grammar::Number { nonnegative: true })?
        };
        return Ok(if !has_metric || metric == kExHeight {
            value
        } else {
            Rc::new(Value::new(CSSValuePayload::kValuePairClass(
                values::CSSValuePair {
                    first: values::identifier(metric),
                    second: value,
                    drop_identical: false,
                },
            )))
        });
    }
    // cpp: longhands_custom.cc:4543-4688; position/emoji keyword consumers.
    let keyword = stream.Peek().Id();
    if keyword == kNormal || id == kFontVariantLigatures && keyword == kNone {
        stream.ConsumeIncludingWhitespace();
        return Ok(values::identifier(keyword));
    }
    let mut items = Vec::new();
    let mut seen = [false; 7];
    loop {
        let item = if id == kFontVariantAlternates {
            ConsumeAlternate(stream)?
        } else {
            Group(id, stream.Peek().Id()).map(|group| {
                (
                    group,
                    values::identifier(stream.ConsumeIncludingWhitespace().Id()),
                )
            })
        };
        let Some((group, value)) = item else {
            break;
        };
        if seen[group] && !(id == kFontVariantAlternates && group == 1) {
            return Err(invalid(id));
        }
        if !seen[group] {
            items.push((group, value));
        }
        seen[group] = true;
        if matches!(
            id,
            kFontVariantCaps | kFontVariantPosition | kFontVariantEmoji
        ) {
            break;
        }
    }
    if items.is_empty() {
        return Err(invalid(id));
    }
    Ok(Finalize(id, items))
}
// cpp: shorthands_custom.cc:3087-3276,3280-3290,3300-3458.
pub(super) fn ParseShorthand<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    stream: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    if id == kFontVariant {
        let longhands = ShorthandFor(id);
        if matches!(stream.Peek().Id(), kNormal | kNone) {
            let ligatures = stream.ConsumeIncludingWhitespace().Id();
            for &property in longhands {
                out.push(make_expanded(
                    property,
                    id,
                    values::identifier(if property == kFontVariantLigatures {
                        ligatures
                    } else {
                        kNormal
                    }),
                    false,
                ));
            }
            return Ok(());
        }
        let mut groups: Vec<Vec<(usize, Rc<Value>)>> = vec![Vec::new(); longhands.len()];
        let mut found = false;
        loop {
            let mut consumed = false;
            for (index, &property) in longhands.iter().enumerate() {
                let item = if property == kFontVariantAlternates {
                    ConsumeAlternate(stream)?
                } else {
                    Group(property, stream.Peek().Id()).map(|group| {
                        (
                            group,
                            values::identifier(stream.ConsumeIncludingWhitespace().Id()),
                        )
                    })
                };
                if let Some((group, value)) = item {
                    let duplicate = groups[index].iter().any(|item| item.0 == group);
                    if duplicate && !(property == kFontVariantAlternates && group == 1) {
                        return Err(invalid(id));
                    }
                    if !duplicate {
                        groups[index].push((group, value));
                    }
                    consumed = true;
                    found = true;
                    break;
                }
            }
            if !consumed {
                break;
            }
        }
        if !found {
            return Err(invalid(id));
        }
        for (&property, items) in longhands.iter().zip(groups) {
            out.push(make_expanded(
                property,
                id,
                Finalize(property, items),
                false,
            ));
        }
        return Ok(());
    }
    if matches!(
        stream.Peek().Id(),
        kCaption | kIcon | kMenu | kMessageBox | kSmallCaption | kStatusBar
    ) {
        return Err(unsupported(
            id,
            "ConsumeSystemFont CSSPendingSystemFontValue and platform settings",
        ));
    }
    let mut optional = [None, None, None, None];
    let optional_ids = [kFontStyle, kFontVariantCaps, kFontWeight, kFontStretch];
    for _ in 0..4 {
        let value = stream.Peek().Id();
        if value == kNormal {
            stream.ConsumeIncludingWhitespace();
            continue;
        }
        let index = if optional[0].is_none() && matches!(value, kItalic | kOblique) {
            Some(0)
        } else if optional[1].is_none() && value == kSmallCaps {
            Some(1)
        } else if optional[2].is_none()
            && (matches!(value, kBold | kBolder | kLighter)
                || stream.Peek().GetType() == kNumberToken
                || is_function(stream))
        {
            Some(2)
        } else if optional[3].is_none()
            && matches!(
                value,
                kUltraCondensed
                    | kExtraCondensed
                    | kCondensed
                    | kSemiCondensed
                    | kSemiExpanded
                    | kExpanded
                    | kExtraExpanded
                    | kUltraExpanded
            )
        {
            Some(3)
        } else {
            None
        };
        let Some(index) = index else {
            break;
        };
        optional[index] = Some(if index == 2 {
            stream.EnsureLookAhead();
            let saved = stream.Save();
            match ConsumeLonghand(kFontWeight, stream, mode) {
                Ok(value) => value,
                Err(_) => {
                    stream.Restore(saved);
                    break;
                }
            }
        } else {
            stream.ConsumeIncludingWhitespace();
            values::identifier(value)
        });
    }
    let size = ConsumeLonghand(kFontSize, stream, mode)?;
    if stream.AtEnd() {
        return Err(invalid(id));
    }
    let height =
        if stream.Peek().GetType() == kDelimiterToken && stream.Peek().Delimiter() == b'/' as u16 {
            stream.ConsumeIncludingWhitespace();
            ConsumeLonghand(kLineHeight, stream, mode)?
        } else {
            values::identifier(kNormal)
        };
    let family = ConsumeLonghand(kFontFamily, stream, mode)?;
    for &property in ShorthandFor(id) {
        let value = if let Some(index) = optional_ids.iter().position(|&v| v == property) {
            optional[index]
                .take()
                .unwrap_or_else(|| values::identifier(kNormal))
        } else if property == kFontSize {
            size.clone()
        } else if property == kLineHeight {
            height.clone()
        } else if property == kFontFamily {
            family.clone()
        } else {
            values::identifier(match property {
                kFontSizeAdjust => kNone,
                kFontKerning | kFontOpticalSizing => kAuto,
                _ => kNormal,
            })
        };
        out.push(make_expanded(property, id, value, false));
    }
    Ok(())
}

// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Stable TimelineTrigger: longhands_custom.cc:714-861; css_parsing_utils.cc:5010-5104.
//! Selected source audit: effective 625, mapped 560, omitted 24, remaining 41.
//! Parser: 267/248/16/3; Apply/map: 222/177/7/38; native: 136/135/1/0.
//! Effective excludes comments, blank/preprocessor/namespace/using/visibility/
//! brace-only lines. Omitted lines are DCHECK and continuation only. Remaining
//! selected lines: utils 4719-4723 (view/scroll CSSValue owners); map 359,
//! 361-386,416,419,750-751 (typed timelines, length contexts and dynamic/scope
//! identity); generated range Inherit changed-zoom reconversion branches.
//! Nested shared math/length/ident and runtime execution are outside this
//! selected-line audit and remain pending in the property/dispatch ledgers.
//! Source ranges (all under Blink core except generated):
//! longhands_custom: 714-722,744-746,748-755,767-769,771-778,790-792,
//! 794-802,814-816,818-825,837-839,841-848,859-861.
//! shorthands_custom:308-344,443-476,5423-5448,5471-5489,5510-5529.
//! css_parsing_utils:1048-1061,4709-4723,4834-4862,4948-4955,4963-4968,5010-5104.
//! css_to_style_map:350-387,400-423,740-754,762-774,776-784,786-800.
//! generated longhands:16994-17024,17051-17081,17108-17138,17165-17195,
//! 17222-17247,17274-17299 (ApplyInitial/Inherit/Value only).
//! native css_animation_data.h:29,33,38-44,76-99,130-149,186-201,220-227;
//! css_animation_data.cc:13-30,42-51,56,73-89,106-110,122-151;
//! timeline_offset.h:67-82,87-88. Trigger attachments are excluded from this batch.
//! Runtime defaults: runtime_enabled_features.json5:6347-6348 stable;
//! generated style_property_shorthand.cc:1786-1859 stable *1 lists are reused.
#![allow(non_snake_case)]
use super::*;
pub(super) const FIELDS: [CSSPropertyID; 6] = [
    CSSPropertyID::kTimelineTriggerName,
    CSSPropertyID::kTimelineTriggerSource,
    CSSPropertyID::kTimelineTriggerActivationRangeStart,
    CSSPropertyID::kTimelineTriggerActivationRangeEnd,
    CSSPropertyID::kTimelineTriggerActiveRangeStart,
    CSSPropertyID::kTimelineTriggerActiveRangeEnd,
];
pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    FIELDS.contains(&id)
}
pub(super) fn IsShorthand(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kTimelineTrigger
            | CSSPropertyID::kTimelineTriggerActivationRange
            | CSSPropertyID::kTimelineTriggerActiveRange
    )
}
fn Name<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    if s.Peek().Id() == CSSValueID::kNone {
        return Ok(Some(values::identifier(
            s.ConsumeIncludingWhitespace().Id(),
        )));
    }
    if s.Peek().FunctionId() == Some(CSSValueID::kIdent) {
        return Err(unsupported(
            id,
            "ConsumeDashedIdent ident() runtime collaborator",
        ));
    }
    if s.Peek().GetType() == kIdentToken && s.Peek().Value().ToString().Utf8().starts_with("--") {
        return ConsumeAnimationName(id, s, false);
    }
    Ok(None)
}
fn Item<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Option<Rc<Value>>, PropertyParseError> {
    let i = FIELDS.iter().position(|f| *f == id).unwrap();
    match i {
        0 => Name(id, s),
        1 => ConsumeAnimationTimeline(id, s),
        _ => {
            ConsumeAnimationRangeWithAuto(id, s, mode, if i % 2 == 0 { 0.0 } else { 100.0 }, i >= 4)
        }
    }
}
pub(super) fn Consume<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
) -> Result<Rc<Value>, PropertyParseError> {
    let mut items = Vec::new();
    loop {
        items.push(Item(id, s, mode)?.ok_or_else(|| invalid(id))?);
        if s.Peek().GetType() != kCommaToken {
            break;
        }
        s.ConsumeIncludingWhitespace();
    }
    Ok(values::list(items, values::ListSeparator::Comma))
}
// cpp: css_parsing_utils.cc:1048-1060. The implied end retains only the named range.
pub(super) fn Implied(v: &Option<Rc<Value>>) -> Option<Rc<Value>> {
    if let Some(v) = v {
        if let CSSValuePayload::kValueListClass(l) = v.Payload() {
            if l.values[0].IsIdentifierValue() {
                return Some(values::list(
                    vec![l.values[0].clone()],
                    values::ListSeparator::Space,
                ));
            }
        }
    }
    None
}
pub(super) fn Expand<T: TokenStreamTokenizer>(
    id: CSSPropertyID,
    s: &mut Stream<T>,
    mode: CSSParserMode,
    out: &mut Vec<PropertyValue>,
) -> Result<(), PropertyParseError> {
    let full = id == CSSPropertyID::kTimelineTrigger;
    let active = id == CSSPropertyID::kTimelineTriggerActiveRange;
    let indices: Vec<usize> = if full {
        (0..6).collect()
    } else if active {
        vec![4, 5]
    } else {
        vec![2, 3]
    };
    let mut lists: Vec<Vec<Rc<Value>>> = (0..6).map(|_| Vec::new()).collect();
    loop {
        let mut vals: Vec<Option<Rc<Value>>> = (0..6).map(|_| None).collect();
        if full {
            for i in 0..4 {
                vals[i] = Item(FIELDS[i], s, mode)?;
            }
            if s.Peek().GetType() == kDelimiterToken && s.Peek().Delimiter() == b'/' as u16 {
                s.ConsumeIncludingWhitespace();
                vals[4] = Item(FIELDS[4], s, mode)?;
                if vals[4].is_none() {
                    return Err(invalid(id));
                }
                vals[5] = Item(FIELDS[5], s, mode)?;
            }
            for i in [3, 5] {
                if vals[i].is_none() {
                    vals[i] = Implied(&vals[i - 1]);
                }
            }
        } else {
            let i = indices[0];
            vals[i] = Item(FIELDS[i], s, mode)?;
            if vals[i].is_none() {
                return Err(invalid(id));
            }
            vals[i + 1] = Item(FIELDS[i + 1], s, mode)?;
            if vals[i + 1].is_none() {
                vals[i + 1] = Implied(&vals[i]);
            }
            if vals[i + 1].is_none() {
                vals[i + 1] = Some(values::identifier(if active {
                    CSSValueID::kAuto
                } else {
                    CSSValueID::kNormal
                }));
            }
        }
        for &i in &indices {
            lists[i].push(vals[i].take().unwrap_or_else(|| {
                values::identifier(match i {
                    0 => CSSValueID::kNone,
                    2 | 3 => CSSValueID::kNormal,
                    _ => CSSValueID::kAuto,
                })
            }));
        }
        if s.Peek().GetType() != kCommaToken {
            break;
        }
        s.ConsumeIncludingWhitespace();
    }
    for i in indices {
        out.push(make_expanded(
            FIELDS[i],
            id,
            values::list(std::mem::take(&mut lists[i]), values::ListSeparator::Comma),
            false,
        ));
    }
    Ok(())
}

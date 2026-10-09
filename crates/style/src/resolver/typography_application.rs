// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium typography native fields and converter branches.
#![allow(non_snake_case)]
use super::*;
use foundation::{
    AtomicString, ERubyOverhang, RubyPosition, TextDecorationSkipSpaces, TextEmphasisFill,
    TextEmphasisMark,
};
use layoutng_style::style::{
    computed_style_constants::TextEmphasisPosition,
    style_hyphenate_limit_chars::StyleHyphenateLimitChars, text_size_adjust::TextSizeAdjust,
};
pub(super) fn IsTypographyProperty(id: CSSPropertyID) -> bool {
    use CSSPropertyID::*;
    matches!(
        id,
        kTextEmphasisStyle
            | kTextEmphasisPosition
            | kHyphenateCharacter
            | kHyphenateLimitChars
            | kRubyPosition
            | kRubyOverhang
            | kTextSizeAdjust
            | kTextDecorationSkipSpaces
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::production_css_value as values;
    #[test]
    fn hidden_typography_flags_and_percentage_converter_use_native_storage() {
        let _heap = foundation::LayoutHeapScope::new();
        let initial = unsafe { &*ComputedStyle::GetInitialStyleSingleton() };
        let mut b = ComputedStyleBuilder::from_style(initial);
        let media = MediaValuesCachedData::default();
        let list = values::list(
            vec![
                values::identifier(CSSValueID::kStart),
                values::identifier(CSSValueID::kEnd),
            ],
            values::ListSeparator::Space,
        );
        ApplyInternal(
            CSSPropertyID::kTextDecorationSkipSpaces,
            &mut b,
            None,
            &list,
            16.0,
            &media,
        )
        .unwrap();
        assert_eq!(b.GetTextDecorationSkipSpaces().bits(), 3);
        let parent = unsafe { &*b.TakeStyle() };
        let mut child = ComputedStyleBuilder::from_style(initial);
        ApplyInternal(
            CSSPropertyID::kTextDecorationSkipSpaces,
            &mut child,
            Some(parent),
            &values::wide(CSSValueID::kInherit).unwrap(),
            16.0,
            &media,
        )
        .unwrap();
        assert_eq!(child.GetTextDecorationSkipSpaces().bits(), 3);
        let all = values::list(
            vec![values::identifier(CSSValueID::kAll)],
            values::ListSeparator::Space,
        );
        ApplyInternal(
            CSSPropertyID::kTextDecorationSkipSpaces,
            &mut child,
            None,
            &all,
            16.0,
            &media,
        )
        .unwrap();
        assert_eq!(
            child.GetTextDecorationSkipSpaces(),
            TextDecorationSkipSpaces::kAll
        );
        ApplyInternal(
            CSSPropertyID::kTextDecorationSkipSpaces,
            &mut child,
            None,
            &values::wide(CSSValueID::kInitial).unwrap(),
            16.0,
            &media,
        )
        .unwrap();
        assert_eq!(
            child.GetTextDecorationSkipSpaces(),
            TextDecorationSkipSpaces::kNone
        );
        let n = 16777217.0;
        let percent = values::numeric(n, UnitType::kPercentage);
        ApplyInternal(
            CSSPropertyID::kTextSizeAdjust,
            &mut child,
            None,
            &percent,
            16.0,
            &media,
        )
        .unwrap();
        assert_eq!(
            child.GetTextSizeAdjust().Multiplier().to_bits(),
            ((n as f32) / 100.0).to_bits()
        );
    }
}
fn List<'a>(
    id: CSSPropertyID,
    v: &'a Value,
) -> std::result::Result<&'a [std::rc::Rc<Value>], LonghandApplicationError> {
    match v.Payload() {
        CSSValuePayload::kValueListClass(l)
            if l.separator == crate::production_css_value::ListSeparator::Space =>
        {
            Ok(&l.values)
        }
        _ => Err(LonghandApplicationError::InvalidValue(id)),
    }
}
pub(super) fn Scalar(
    id: CSSPropertyID,
    b: &ComputedStyleBuilder,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
    percent: bool,
) -> std::result::Result<f64, LonghandApplicationError> {
    use crate::css_math_expression_node::CalculationResultCategory as C;
    match v.Payload() {
        CSSValuePayload::kNumericLiteralClass(n)
            if if percent {
                n.GetType() == UnitType::kPercentage
            } else {
                matches!(n.GetType(), UnitType::kNumber | UnitType::kInteger)
            } =>
        {
            Ok(n.DoubleValue())
        }
        CSSValuePayload::kMathFunctionClass(m)
            if m.Category() == if percent { C::Percent } else { C::Number } =>
        {
            m.ComputeValue(
                &mut MathLengthResolver(
                    id,
                    b.GetFontDescription().ComputedSize(),
                    root,
                    b.EffectiveZoom(),
                    media,
                ),
                None,
            )
            .map_err(|_| LonghandApplicationError::Unsupported(id))
        }
        _ => Err(LonghandApplicationError::InvalidValue(id)),
    }
}
fn Fill(
    id: CSSPropertyID,
    k: CSSValueID,
) -> std::result::Result<TextEmphasisFill, LonghandApplicationError> {
    match k {
        CSSValueID::kFilled => Ok(TextEmphasisFill::kFilled),
        CSSValueID::kOpen => Ok(TextEmphasisFill::kOpen),
        _ => Err(LonghandApplicationError::InvalidValue(id)),
    }
}
fn Mark(
    id: CSSPropertyID,
    k: CSSValueID,
) -> std::result::Result<TextEmphasisMark, LonghandApplicationError> {
    use CSSValueID::*;
    Ok(match k {
        kNone => TextEmphasisMark::kNone,
        kDot => TextEmphasisMark::kDot,
        kCircle => TextEmphasisMark::kCircle,
        kDoubleCircle => TextEmphasisMark::kDoubleCircle,
        kTriangle => TextEmphasisMark::kTriangle,
        kSesame => TextEmphasisMark::kSesame,
        _ => return Err(LonghandApplicationError::InvalidValue(id)),
    })
}
pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result {
    if !crate::production_typography_features::IsExposed(id) {
        return Err(LonghandApplicationError::Unsupported(id));
    }
    ApplyInternal(id, b, parent, v, root, media)
}
pub(super) fn ApplyInternal(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result {
    use CSSPropertyID::*;
    use CSSValueID::*;
    use CSSValueID::{kAll, kLeft, kRight};
    let inherit = v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial = v.IsInitialValue() || v.IsUnsetValue() && !inherit || inherit && parent.is_none();
    let inherited = parent.filter(|_| inherit && !initial);
    match id {
        kTextEmphasisColor => {
            let color = if let Some(p) = inherited {
                p.TextEmphasisColor().clone()
            } else if initial {
                StyleColor::CurrentColor()
            } else {
                return Err(LonghandApplicationError::InvalidValue(id));
            };
            ApplyColor(id, b, parent, &color)?;
        }
        // longhands_custom.cc:12059-12123, including resolved auto on inherit.
        kTextEmphasisStyle => {
            let (fill, mark, custom) = if initial {
                (
                    TextEmphasisFill::kFilled,
                    TextEmphasisMark::kNone,
                    AtomicString::default(),
                )
            } else if let Some(p) = inherited {
                (
                    p.GetTextEmphasisFill(),
                    p.GetTextEmphasisMark(),
                    p.TextEmphasisCustomMark().clone(),
                )
            } else {
                let mut value = v;
                if let CSSValuePayload::kValueListClass(l) = v.Payload() {
                    if l.values.len() == 1 {
                        value = &l.values[0];
                    }
                }
                match value.Payload() {
                    CSSValuePayload::kStringClass(s) => (
                        TextEmphasisFill::kFilled,
                        TextEmphasisMark::kCustom,
                        AtomicString::from_utf16(s.0.Span16().unwrap_or_default()),
                    ),
                    CSSValuePayload::kValueListClass(_) => {
                        let list = List(id, value)?;
                        if list.len() != 2 {
                            return Err(LonghandApplicationError::InvalidValue(id));
                        }
                        let (mut fill, mut mark) = (None, None);
                        for item in list {
                            let k = Identifier(id, item)?;
                            if matches!(k, kFilled | kOpen) {
                                if fill.is_some() {
                                    return Err(LonghandApplicationError::InvalidValue(id));
                                }
                                fill = Some(Fill(id, k)?);
                            } else {
                                if mark.is_some() {
                                    return Err(LonghandApplicationError::InvalidValue(id));
                                }
                                mark = Some(Mark(id, k)?);
                            }
                        }
                        (
                            fill.ok_or(LonghandApplicationError::InvalidValue(id))?,
                            mark.ok_or(LonghandApplicationError::InvalidValue(id))?,
                            AtomicString::default(),
                        )
                    }
                    _ => {
                        let k = Identifier(id, value)?;
                        if matches!(k, kFilled | kOpen) {
                            (
                                Fill(id, k)?,
                                TextEmphasisMark::kAuto,
                                AtomicString::default(),
                            )
                        } else {
                            (
                                TextEmphasisFill::kFilled,
                                Mark(id, k)?,
                                AtomicString::default(),
                            )
                        }
                    }
                }
            };
            b.SetTextEmphasisFill(fill);
            b.SetTextEmphasisMark(mark);
            b.SetTextEmphasisCustomMark(&custom);
        }
        // style_builder_converter.cc:3190-3222.
        kTextEmphasisPosition => {
            let position = if initial {
                ComputedStyleInitialValues::InitialTextEmphasisPosition()
            } else if let Some(p) = inherited {
                p.GetTextEmphasisPosition()
            } else {
                let list = List(id, v)?;
                if list.is_empty() || list.len() > 2 {
                    return Err(LonghandApplicationError::InvalidValue(id));
                }
                let first = Identifier(id, &list[0])?;
                let second = if list.len() == 2 {
                    Identifier(id, &list[1])?
                } else {
                    kRight
                };
                match(first,second){(kAuto,_) if list.len()==1&&foundation::RuntimeEnabledFeatures::TextEmphasisPositionAutoEnabled()=>TextEmphasisPosition::kAuto,(kOver,kRight)=>TextEmphasisPosition::kOverRight,(kOver,kLeft)=>TextEmphasisPosition::kOverLeft,(kUnder,kRight)=>TextEmphasisPosition::kUnderRight,(kUnder,kLeft)=>TextEmphasisPosition::kUnderLeft,_=>return Err(LonghandApplicationError::InvalidValue(id))}
            };
            b.SetTextEmphasisPosition(position);
        }
        kHyphenateCharacter => {
            let value = if initial {
                AtomicString::default()
            } else if let Some(p) = inherited {
                p.HyphenationString().clone()
            } else {
                match v.Payload() {
                    CSSValuePayload::kStringClass(s) => {
                        AtomicString::from_utf16(s.0.Span16().unwrap_or_default())
                    }
                    CSSValuePayload::kIdentifierClass(k) if k.0 == kAuto => AtomicString::default(),
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            b.SetHyphenationString(&value);
        }
        // style_builder_converter.cc:1911-1940; native saturates each u8.
        kHyphenateLimitChars => {
            let value = if initial {
                StyleHyphenateLimitChars::default()
            } else if let Some(p) = inherited {
                *p.HyphenateLimitChars()
            } else if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==kAuto) {
                StyleHyphenateLimitChars::default()
            } else {
                let list = List(id, v)?;
                if !(1..=3).contains(&list.len()) {
                    return Err(LonghandApplicationError::InvalidValue(id));
                }
                let mut parts = [0; 3];
                for (i, item) in list.iter().enumerate() {
                    if matches!(item.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==kAuto) {
                        continue;
                    }
                    let number = Scalar(id, b, item, root, media, false)?;
                    if number.is_nan()
                        || number < 1.0
                        || number.is_finite() && number.fract() != 0.0
                    {
                        return Err(LonghandApplicationError::InvalidValue(id));
                    }
                    parts[i] = (number as i32) as u32;
                }
                StyleHyphenateLimitChars::new(parts[0], parts[1], parts[2])
            };
            b.SetHyphenateLimitChars(&value);
        }
        kRubyPosition => {
            let value = if initial {
                RubyPosition::kOver
            } else if let Some(p) = inherited {
                p.GetRubyPosition()
            } else {
                match Identifier(id, v)? {
                    kOver | kBefore => RubyPosition::kOver,
                    kUnder | kAfter => RubyPosition::kUnder,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            b.SetRubyPosition(value);
        }
        kRubyOverhang => {
            let value = if initial {
                ERubyOverhang::kAuto
            } else if let Some(p) = inherited {
                p.RubyOverhang()
            } else {
                match Identifier(id, v)? {
                    kAuto => ERubyOverhang::kAuto,
                    kSpaces => ERubyOverhang::kSpaces,
                    _ => return Err(LonghandApplicationError::InvalidValue(id)),
                }
            };
            b.SetRubyOverhang(value);
        }
        // style_builder_converter.cc:3237-3253. Preserve float cast before /100.
        // StyleResolverState's font invalidation and FontBuilder owner remain
        // explicitly partial; no host autosizing setting is synthesized here.
        kTextSizeAdjust => {
            let value = if initial {
                TextSizeAdjust::AdjustAuto()
            } else if let Some(p) = inherited {
                *p.GetTextSizeAdjust()
            } else {
                match v.Payload() {
                    CSSValuePayload::kIdentifierClass(k) if k.0 == kAuto => {
                        TextSizeAdjust::AdjustAuto()
                    }
                    CSSValuePayload::kIdentifierClass(k) if k.0 == kNone => {
                        TextSizeAdjust::AdjustNone()
                    }
                    _ => {
                        let n = Scalar(id, b, v, root, media, true)?;
                        if n.is_nan() || n < 0.0 {
                            return Err(LonghandApplicationError::InvalidValue(id));
                        }
                        TextSizeAdjust::new((n.min(f32::MAX as f64) as f32) / 100.0)
                    }
                }
            };
            b.SetTextSizeAdjust(&value);
        }
        kTextDecorationSkipSpaces => {
            let value = if initial {
                TextDecorationSkipSpaces::kNone
            } else if let Some(p) = inherited {
                p.GetTextDecorationSkipSpaces()
            } else if matches!(v.Payload(),CSSValuePayload::kIdentifierClass(k) if k.0==kNone) {
                TextDecorationSkipSpaces::kNone
            } else {
                let list = List(id, v)?;
                if list.is_empty() || list.len() > 2 {
                    return Err(LonghandApplicationError::InvalidValue(id));
                }
                let mut flags = TextDecorationSkipSpaces::kNone;
                for item in list {
                    let flag = match Identifier(id, item)? {
                        kStart => TextDecorationSkipSpaces::kStart,
                        kEnd => TextDecorationSkipSpaces::kEnd,
                        kAll if list.len() == 1 => TextDecorationSkipSpaces::kAll,
                        _ => return Err(LonghandApplicationError::InvalidValue(id)),
                    };
                    if flags.bits() & flag.bits() != 0 {
                        return Err(LonghandApplicationError::InvalidValue(id));
                    }
                    flags |= flag;
                }
                flags
            };
            b.SetTextDecorationSkipSpaces(value);
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    if inherit && !initial {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}

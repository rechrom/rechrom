// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Chromium column-rule converters into native GapDataList storage.
#![allow(non_snake_case)]
use super::*;
use foundation::{EBorderStyle, EInsideLink};
use layoutng_style::style::{
    gap_data::{GapData, GapValue},
    gap_data_list::GapDataList,
};
pub(super) fn IsProperty(id: CSSPropertyID) -> bool {
    matches!(
        id,
        CSSPropertyID::kColumnRuleColor
            | CSSPropertyID::kColumnRuleStyle
            | CSSPropertyID::kColumnRuleWidth
            | CSSPropertyID::kRowRuleColor
            | CSSPropertyID::kRowRuleStyle
            | CSSPropertyID::kRowRuleWidth
    )
}
fn Entries<'a>(
    id: CSSPropertyID,
    v: &'a Value,
) -> std::result::Result<Vec<&'a Value>, LonghandApplicationError> {
    match v.Payload() {
        CSSValuePayload::kValueListClass(l)
            if l.separator == crate::production_css_value::ListSeparator::Comma
                && !l.values.is_empty() =>
        {
            Ok(l.values.iter().map(|v| v.as_ref()).collect())
        }
        CSSValuePayload::kValueListClass(_) => Err(LonghandApplicationError::InvalidValue(id)),
        _ => Ok(vec![v]),
    }
}
fn List<T: GapValue>(values: Vec<T>) -> GapDataList<T> {
    let mut list = GapDataList::default();
    for v in values {
        list.AddGapData(&GapData::from_value(v));
    }
    list
}
pub(super) fn Apply(
    id: CSSPropertyID,
    b: &mut ComputedStyleBuilder,
    parent: Option<&ComputedStyle>,
    v: &Value,
    root: f32,
    media: &MediaValuesCachedData,
) -> Result {
    use CSSPropertyID::*;
    let inherit = v.IsInheritedValue() || v.IsUnsetValue() && CSSProperty::Get(id).IsInherited();
    let initial = v.IsInitialValue() || v.IsUnsetValue() && !inherit || inherit && parent.is_none();
    let inherited = parent.filter(|_| inherit && !initial);
    match id {
        kColumnRuleColor | kRowRuleColor => {
            let row = id == kRowRuleColor;
            let colors = if initial {
                GapDataList::DefaultGapColorDataList()
            } else if let Some(p) = inherited {
                if row {
                    p.RowRuleColor().clone()
                } else {
                    p.ColumnRuleColor().clone()
                }
            } else {
                let entries = Entries(id, v)?;
                // converter.cc:2634-2641: use the real native InsideLink state
                // when supplied. Document link-state/visited ownership remains
                // explicit partial; no link state is inferred from CSS strings.
                if entries.len() > 1 && b.InsideLink() != EInsideLink::kNotInsideLink {
                    GapDataList::DefaultGapColorDataList()
                } else {
                    List(
                        entries
                            .iter()
                            .map(|v| color_ui_application::ConvertStyleColorValue(id, v))
                            .collect::<std::result::Result<Vec<StyleColor>, _>>()?,
                    )
                }
            };
            if row {
                b.SetRowRuleColor(&colors);
            } else {
                b.SetColumnRuleColor(&colors);
            }
        }
        kColumnRuleStyle | kRowRuleStyle => {
            let row = id == kRowRuleStyle;
            let styles = if initial {
                GapDataList::DefaultGapStyleDataList()
            } else if let Some(p) = inherited {
                if row {
                    p.RowRuleStyle().clone()
                } else {
                    p.ColumnRuleStyle().clone()
                }
            } else {
                List(
                    Entries(id, v)?
                        .iter()
                        .map(|v| ConvertBorderStyle(id, Identifier(id, v)?))
                        .collect::<std::result::Result<Vec<EBorderStyle>, _>>()?,
                )
            };
            if row {
                b.SetRowRuleStyle(&styles);
            } else {
                b.SetColumnRuleStyle(&styles);
            }
        }
        kColumnRuleWidth | kRowRuleWidth => {
            let row = id == kRowRuleWidth;
            let widths = if initial {
                // custom longhands.cc:2732-2737, initial int is zoomed pixels.
                // RowRuleWidth custom:2767-2772 uses its native legacy value.
                let width = if row {
                    ComputedStyleInitialValues::InitialRowRuleWidth().GetLegacyValue()
                } else {
                    ComputedStyleInitialValues::InitialColumnRuleWidth().GetSingleValue()
                };
                GapDataList::from_value(&((width as f64 * b.EffectiveZoom() as f64) as i32))
            } else if let Some(p) = inherited {
                if p.EffectiveZoom() != b.EffectiveZoom() {
                    return Err(LonghandApplicationError::Unsupported(id));
                }
                if row {
                    p.RowRuleWidth().clone()
                } else {
                    p.ColumnRuleWidth().clone()
                }
            } else {
                List(
                    Entries(id, v)?
                        .iter()
                        .map(|v| {
                            border_application::ConvertBorderWidthValue(id, b, v, root, media)
                                .map(|width| width.min(u16::MAX as i32))
                        })
                        .collect::<std::result::Result<Vec<i32>, _>>()?,
                )
            };
            if row {
                b.SetRowRuleWidth(&widths);
            } else {
                b.SetColumnRuleWidth(&widths);
            }
        }
        _ => return Err(LonghandApplicationError::Unsupported(id)),
    }
    if inherit && !initial {
        b.SetHasExplicitInheritance();
        parent.unwrap().SetChildHasExplicitInheritance();
    }
    Ok(())
}

//! style_property_serializer.cc:1066-1347,1843-2203.
use super::basic::{AppendValue, Identifier, IsIdentifier};
use super::{CSSValueID, CSSValuePayload, Id, Shorthand, String, StylePropertySerializer, Value};
use crate::{css_primitive_value::UnitType, production_css_value::CSSValueList};
pub(super) fn List(value: &Value) -> &CSSValueList {
    assert!(value.IsValueList());
    value.ValueListView()
}
fn BaseList(value: &Value) -> Option<&CSSValueList> {
    match value.Payload() {
        CSSValuePayload::kValueListClass(list) => Some(list),
        _ => None,
    }
}
fn IdentifierPair(value: &Value, id: CSSValueID) -> bool {
    matches!(value.Payload(),CSSValuePayload::kValuePairClass(pair) if IsIdentifier(&pair.first,id)&&IsIdentifier(&pair.second,id))
}
fn RangePercent(value: &Value, default: f64) -> (CSSValueID, f64) {
    if !value.IsValueList() {
        return (CSSValueID::kNormal, default);
    }
    let list = List(value);
    assert!((1..=2).contains(&list.values.len()));
    let mut name = CSSValueID::kNormal;
    let mut percent = default;
    let offset = if let Some(id) = Identifier(&list.values[0]) {
        name = id;
        if list.values.len() == 2 {
            Some(list.values[1].as_ref())
        } else {
            None
        }
    } else {
        Some(list.values[0].as_ref())
    };
    if let Some(value) = offset {
        percent=match value.Payload(){
        CSSValuePayload::kNumericLiteralClass(value) if value.GetType()==UnitType::kPercentage=>value.GetValueIfKnown().unwrap_or(-1.0),
        CSSValuePayload::kMathFunctionClass(value) if value.Category()==layoutng_style::css::css_math_expression_node::CalculationResultCategory::Percent=>value.GetValueIfKnown().unwrap_or(-1.0),_=>-1.0,
    };
    }
    (name, percent)
}
fn DropAnimationRangeEndValue(start: &Value, end: &Value) -> bool {
    let (start_name, _) = RangePercent(start, 0.0);
    RangePercent(end, 100.0) == (start_name, 100.0)
}
fn IsZeroPercent(value: &Value) -> bool {
    matches!(value.Payload(),CSSValuePayload::kNumericLiteralClass(value) if value.GetType()==UnitType::kPercentage && value.GetValueIfKnown()==Some(0.0))
}
fn SerializeMaskOriginAndClip(out: &mut String, origin: CSSValueID, clip: CSSValueID) {
    if origin == CSSValueID::kBorderBox && clip == CSSValueID::kBorderBox {
        return;
    }
    if !out.empty() {
        out.push(' ');
    }
    if origin == clip {
        out.push_string(&crate::production_css_value::identifier(origin).CssText());
    } else if origin == CSSValueID::kBorderBox && clip == CSSValueID::kNoClip {
        out.push_string(&crate::production_css_value::identifier(clip).CssText());
    } else {
        out.push_string(&crate::production_css_value::identifier(origin).CssText());
        out.push(' ');
        out.push_string(&crate::production_css_value::identifier(clip).CssText());
    }
}
impl StylePropertySerializer<'_> {
    fn TimelineValue(&self, shorthand: Shorthand) -> String {
        assert!((2..=3).contains(&shorthand.properties.len()));
        let names = List(self.Required(shorthand.properties[0]));
        let axes = List(self.Required(shorthand.properties[1]));
        let insets = if shorthand.properties.len() == 3 {
            Some(List(self.Required(shorthand.properties[2])))
        } else {
            None
        };
        assert!(!axes.values.is_empty());
        if let Some(insets) = insets {
            assert!(!insets.values.is_empty());
        }
        let axis_initial =
            axes.values.len() == 1 && IsIdentifier(&axes.values[0], CSSValueID::kBlock);
        let inset_initial = insets.is_some_and(|list| {
            list.values.len() == 1 && IdentifierPair(&list.values[0], CSSValueID::kAuto)
        });
        if !axis_initial && axes.values.len() != names.values.len()
            || insets.is_some_and(|list| !inset_initial && list.values.len() != names.values.len())
        {
            return String::new();
        }
        let mut out = String::from("");
        for (i, name) in names.values.iter().enumerate() {
            if i != 0 {
                out.push_str(", ");
            }
            out.push_string(&name.CssText());
            let axis = &axes.values[if axis_initial { 0 } else { i }];
            if !IsIdentifier(axis, CSSValueID::kBlock) {
                out.push(' ');
                out.push_string(&axis.CssText());
            }
            if let Some(insets) = insets {
                let inset = &insets.values[if inset_initial { 0 } else { i }];
                if !IdentifierPair(inset, CSSValueID::kAuto) {
                    out.push(' ');
                    out.push_string(&inset.CssText());
                }
            }
        }
        out
    }
    pub(super) fn ScrollTimelineValue(&self) -> String {
        self.TimelineValue(Shorthand::For(Id::kScrollTimeline))
    }
    pub(super) fn ViewTimelineValue(&self) -> String {
        self.TimelineValue(Shorthand::For(Id::kViewTimeline))
    }
    fn RangeValue(&self, start: Id, end: Id, exit: bool) -> String {
        let start = List(self.Required(start));
        let end = List(self.Required(end));
        if start.values.len() != end.values.len() {
            return String::from("");
        }
        let mut out = String::from("");
        for (i, (start, end)) in start.values.iter().zip(&end.values).enumerate() {
            if i != 0 {
                out.push_str(", ");
            }
            out.push_string(&start.CssText());
            if exit {
                if let Some(id) = Identifier(end) {
                    debug_assert!(matches!(id, CSSValueID::kAuto | CSSValueID::kNormal));
                    if id == CSSValueID::kNormal {
                        out.push(' ');
                        out.push_string(&end.CssText());
                    }
                    continue;
                }
            }
            if !DropAnimationRangeEndValue(start, end) {
                out.push(' ');
                out.push_string(&end.CssText());
            }
        }
        out
    }
    pub(super) fn AnimationRangeShorthandValue(&self) -> String {
        self.RangeValue(Id::kAnimationRangeStart, Id::kAnimationRangeEnd, false)
    }
    pub(super) fn TimelineTriggerActivationRangeShorthandValue(&self) -> String {
        self.RangeValue(
            Id::kTimelineTriggerActivationRangeStart,
            Id::kTimelineTriggerActivationRangeEnd,
            false,
        )
    }
    pub(super) fn TimelineTriggerExitRangeShorthandValue(&self) -> String {
        self.RangeValue(
            Id::kTimelineTriggerActiveRangeStart,
            Id::kTimelineTriggerActiveRangeEnd,
            true,
        )
    }
    pub(super) fn GetLayeredShorthandValue(&self, shorthand: Shorthand) -> String {
        let values: Vec<_> = shorthand
            .properties
            .iter()
            .map(|&id| self.Required(id))
            .collect();
        let layers = values
            .iter()
            .filter_map(|value| BaseList(value).map(|list| list.values.len()))
            .max()
            .unwrap_or(1)
            .max(1);
        let mut result = String::from("");
        for layer in 0..layers {
            let mut out = String::from("");
            let mut x_serialized = false;
            let mut y_serialized = false;
            let mut mask_x = None;
            let mut mask_origin = CSSValueID::kBorderBox;
            for (property_index, (&id, &all_value)) in
                shorthand.properties.iter().zip(&values).enumerate()
            {
                let value = if let Some(list) = BaseList(all_value) {
                    list.values.get(layer).map(|value| value.as_ref())
                } else if layer == 0 && id != Id::kBackgroundColor
                    || layer == layers - 1 && id == Id::kBackgroundColor
                {
                    Some(all_value)
                } else {
                    None
                };
                let Some(value) = value else {
                    continue;
                };
                let mut omit = value.IsInitialValue();
                // animation shorthand resets timeline/range; an additional
                // list layer cannot be represented even if each item is initial.
                if id == Id::kAnimationTimeline {
                    if !IsIdentifier(value, CSSValueID::kAuto) || layer > 0 {
                        return String::from("");
                    }
                    omit = true;
                }
                if matches!(id, Id::kAnimationRangeStart | Id::kAnimationRangeEnd) {
                    if !IsIdentifier(value, CSSValueID::kNormal) || layer > 0 {
                        return String::from("");
                    }
                    omit = true;
                }
                if id == Id::kTransitionBehavior {
                    assert_eq!(shorthand.id, Id::kTransition);
                    if IsIdentifier(value, CSSValueID::kNormal) {
                        omit = true;
                    }
                }
                if matches!(id, Id::kTransitionDelay | Id::kTransitionDuration) {
                    if matches!(value.Payload(),CSSValuePayload::kNumericLiteralClass(value) if value.GetValueIfKnown()==Some(0.0))
                    {
                        omit = true;
                    }
                } else if id == Id::kTransitionTimingFunction {
                    if IsIdentifier(value, CSSValueID::kEase) {
                        omit = true;
                    }
                } else if id == Id::kTransitionProperty {
                    if matches!(value.Payload(),CSSValuePayload::kCustomIdentClass(value) if value.property==Id::kAll)
                        || IsIdentifier(value, CSSValueID::kAll)
                    {
                        omit = true;
                    }
                }
                if shorthand.id == Id::kMask {
                    match id {
                        Id::kMaskImage => {
                            if IsIdentifier(value, CSSValueID::kNone) {
                                omit = true;
                            }
                        }
                        Id::kMaskOrigin => {
                            if let Some(id) = Identifier(value) {
                                mask_origin = id;
                            }
                            omit = true;
                        }
                        Id::kMaskClip => {
                            SerializeMaskOriginAndClip(
                                &mut out,
                                mask_origin,
                                Identifier(value).unwrap_or(CSSValueID::kBorderBox),
                            );
                            omit = true;
                        }
                        Id::kMaskComposite => {
                            if IsIdentifier(value, CSSValueID::kAdd) {
                                omit = true;
                            }
                        }
                        Id::kMaskMode => {
                            if IsIdentifier(value, CSSValueID::kMatchSource) {
                                omit = true;
                            }
                        }
                        Id::kMaskRepeat => {
                            if matches!(value.Payload(),CSSValuePayload::kRepeatStyleClass(value) if value.IsRepeat())
                            {
                                omit = true;
                            }
                        }
                        Id::kMaskSize => {
                            if IsIdentifier(value, CSSValueID::kAuto) {
                                omit = true;
                            }
                        }
                        Id::kWebkitMaskPositionX => {
                            omit = true;
                            mask_x = Some(value);
                        }
                        Id::kWebkitMaskPositionY => {
                            omit = true;
                            if let Some(x) = mask_x {
                                if !IsZeroPercent(x) || !IsZeroPercent(value) {
                                    x_serialized = true;
                                    y_serialized = true;
                                    AppendValue(&mut out, x, " ");
                                    out.push(' ');
                                    out.push_string(&value.CssText());
                                }
                            }
                        }
                        _ => {}
                    }
                }
                if shorthand.id == Id::kTimelineTrigger {
                    match id {
                        Id::kTimelineTriggerName => {
                            if let Some(id) = Identifier(value) {
                                debug_assert_eq!(id, CSSValueID::kNone);
                                omit = true;
                            }
                        }
                        Id::kTimelineTriggerSource => {
                            if let Some(id) = Identifier(value) {
                                debug_assert!(matches!(id, CSSValueID::kAuto | CSSValueID::kNone));
                                omit = id == CSSValueID::kAuto;
                            }
                        }
                        Id::kTimelineTriggerActivationRangeStart => {
                            if let Some(id) = Identifier(value) {
                                debug_assert_eq!(id, CSSValueID::kNormal);
                                omit = true;
                            }
                        }
                        Id::kTimelineTriggerActivationRangeEnd
                        | Id::kTimelineTriggerActiveRangeEnd => {
                            if let Some(keyword) = Identifier(value) {
                                debug_assert!(if id == Id::kTimelineTriggerActivationRangeEnd {
                                    keyword == CSSValueID::kNormal
                                } else {
                                    matches!(keyword, CSSValueID::kAuto | CSSValueID::kNormal)
                                });
                                omit = id == Id::kTimelineTriggerActivationRangeEnd
                                    || keyword == CSSValueID::kAuto;
                            } else {
                                let previous = List(values[property_index - 1]);
                                omit = DropAnimationRangeEndValue(&previous.values[layer], value);
                            }
                        }
                        Id::kTimelineTriggerActiveRangeStart => {
                            if let Some(id) = Identifier(value) {
                                debug_assert!(matches!(
                                    id,
                                    CSSValueID::kAuto | CSSValueID::kNormal
                                ));
                                omit = id == CSSValueID::kAuto;
                            }
                        }
                        _ => {}
                    }
                }
                if !omit {
                    if matches!(id, Id::kBackgroundSize | Id::kMaskSize) {
                        out.push_str(if x_serialized || y_serialized {
                            " / "
                        } else {
                            " 0% 0% / "
                        });
                    } else if id == Id::kTimelineTriggerActiveRangeStart {
                        out.push_str(" / ");
                    } else if !out.empty() {
                        out.push(' ');
                    }
                    out.push_string(&value.CssText());
                    if id == Id::kBackgroundPositionX {
                        x_serialized = true;
                    }
                    if id == Id::kBackgroundPositionY {
                        y_serialized = true;
                    }
                }
            }
            if out.empty() {
                match shorthand.id {
                    Id::kMask | Id::kTimelineTrigger => out.push_str("none"),
                    Id::kTransition => out.push_str("all"),
                    _ => {}
                }
            }
            if !out.empty() {
                if !result.empty() {
                    result.push_str(", ");
                }
                result.push_string(&out);
            }
        }
        result
    }
}

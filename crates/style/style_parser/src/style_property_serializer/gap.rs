//! style_property_serializer.cc:2236-2700, Chromium 6c1d401fcca5.
use super::basic::{AppendValue, IsIdentifier};
use super::{
    CSSValueID, CSSValuePayload, GapDirection, Id, Shorthand, String, StylePropertySerializer,
    Value,
};
use crate::production_repeat_value::CSSRepeatValue;
use std::rc::Rc;
fn AllEqual(values: &[&Value]) -> bool {
    values[1..].iter().all(|value| *value == values[0])
}
fn AppendFour(values: [&Value; 4]) -> String {
    let mut out = values[0].CssText();
    AppendValue(&mut out, values[1], " ");
    out.push_str(" / ");
    out.push_string(&values[2].CssText());
    AppendValue(&mut out, values[3], " ");
    out
}
fn InsetProperties(direction: GapDirection) -> [Id; 4] {
    match direction {
        GapDirection::Column => [
            Id::kColumnRuleInsetCapStart,
            Id::kColumnRuleInsetCapEnd,
            Id::kColumnRuleInsetJunctionStart,
            Id::kColumnRuleInsetJunctionEnd,
        ],
        GapDirection::Row => [
            Id::kRowRuleInsetCapStart,
            Id::kRowRuleInsetCapEnd,
            Id::kRowRuleInsetJunctionStart,
            Id::kRowRuleInsetJunctionEnd,
        ],
    }
}
fn Repeat(value: &Value) -> Option<&CSSRepeatValue> {
    match value.Payload() {
        CSSValuePayload::kRepeatClass(value) => Some(value),
        _ => None,
    }
}
// Source's temporary single-item CSSValueList wrapper is a borrowed sequence;
// the original values and their pointer identity remain unchanged.
enum Sequence<'a> {
    List(&'a [Rc<Value>]),
    Single(&'a Value),
}
impl<'a> Sequence<'a> {
    fn new(value: &'a Value) -> Self {
        if value.IsValueList() {
            Self::List(&value.ValueListView().values)
        } else {
            Self::Single(value)
        }
    }
    fn len(&self) -> usize {
        match self {
            Self::List(values) => values.len(),
            Self::Single(_) => 1,
        }
    }
    fn get(&self, i: usize) -> &'a Value {
        match self {
            Self::List(values) => values[i].as_ref(),
            Self::Single(value) => {
                assert_eq!(i, 0);
                value
            }
        }
    }
}
fn Segment(width: &Value, style: &Value, color: &Value) -> String {
    let mut out = String::from("");
    for (value, initial) in [
        (width, CSSValueID::kMedium),
        (style, CSSValueID::kNone),
        (color, CSSValueID::kCurrentcolor),
    ] {
        if !IsIdentifier(value, initial) && !value.IsInitialValue() {
            AppendValue(&mut out, value, " ");
        }
    }
    if out.empty() {
        out.push_str("medium");
    }
    out
}
impl StylePropertySerializer<'_> {
    fn GapValues<'a>(&'a self, shorthand: Shorthand, count: usize) -> Vec<&'a Value> {
        assert_eq!(shorthand.properties.len(), count);
        shorthand
            .properties
            .iter()
            .map(|&id| self.Required(id))
            .collect()
    }
    pub(super) fn GetShorthandValueForRule(&self, row: Shorthand, column: Shorthand) -> String {
        assert_eq!(row.properties.len(), column.properties.len());
        if row
            .properties
            .iter()
            .zip(column.properties)
            .any(|(&row, &column)| self.Get(row) != self.Get(column))
        {
            return String::new();
        }
        self.GetShorthandValueForGapDecorationsRule(column, GapDirection::Column)
    }
    pub(super) fn GetShorthandValueForBidirectionalGapRuleInset(
        &self,
        shorthand: Shorthand,
    ) -> String {
        let v = self.GapValues(shorthand, 8);
        if (0..4).any(|i| v[i] != v[i + 4]) {
            return String::new();
        }
        if AllEqual(&v[..4]) {
            return v[0].CssText();
        }
        if !v[0].IsInitialValue() {
            AppendFour([v[0], v[1], v[2], v[3]])
        } else {
            String::new()
        }
    }
    pub(super) fn GetShorthandValueForBidirectionalGapRuleInsetCapJunction(
        &self,
        shorthand: Shorthand,
    ) -> String {
        let v = self.GapValues(shorthand, 4);
        // Source repeats these two structural checks. They intentionally don't
        // substitute for its later pointer comparison between start and end.
        if v[0] != v[2] || v[1] != v[3] || v[0] != v[2] || v[1] != v[3] {
            return String::new();
        }
        let mut out = String::from("");
        if !v[0].IsInitialValue() {
            out.push_string(&v[0].CssText());
            if !std::ptr::eq(v[0], v[1]) {
                AppendValue(&mut out, v[1], " ");
            }
        }
        out
    }
    pub(super) fn GetShorthandValueForBidirectionalGapRuleInsetStartEnd(
        &self,
        shorthand: Shorthand,
    ) -> String {
        let v = self.GapValues(shorthand, 4);
        if !AllEqual(&v) {
            return String::new();
        }
        if !v[0].IsInitialValue() {
            v[0].CssText()
        } else {
            String::new()
        }
    }
    pub(super) fn GetShorthandValueForBidirectionalGapRules(&self, shorthand: Shorthand) -> String {
        let v = self.GapValues(shorthand, 2);
        if v[0] != v[1] || v[0].IsInitialValue() {
            String::new()
        } else {
            v[0].CssText()
        }
    }
    pub(super) fn GetShorthandValueForGapDecorationsRule(
        &self,
        shorthand: Shorthand,
        direction: GapDirection,
    ) -> String {
        let expected = match direction {
            GapDirection::Column => [
                Id::kColumnRuleWidth,
                Id::kColumnRuleStyle,
                Id::kColumnRuleColor,
            ],
            GapDirection::Row => [Id::kRowRuleWidth, Id::kRowRuleStyle, Id::kRowRuleColor],
        };
        assert_eq!(shorthand.properties, &expected);
        let v = self.GapValues(shorthand, 3);
        let width = Sequence::new(v[0]);
        let style = Sequence::new(v[1]);
        let color = Sequence::new(v[2]);
        if width.len() != style.len() || width.len() != color.len() {
            return String::new();
        }
        let mut result = String::from("");
        for i in 0..width.len() {
            if i > 0 {
                result.push_str(", ");
            }
            let width = width.get(i);
            let style = style.get(i);
            let color = color.get(i);
            let style_repeat = Repeat(style);
            let color_repeat = Repeat(color);
            if let Some(width) = Repeat(width) {
                let (Some(style), Some(color)) = (style_repeat, color_repeat) else {
                    return String::new();
                };
                let auto = width.IsAutoRepeatValue();
                if auto != style.IsAutoRepeatValue() || auto != color.IsAutoRepeatValue() {
                    return String::new();
                }
                let repetitions = if auto {
                    None
                } else {
                    let repetitions = width.Repetitions();
                    if repetitions != style.Repetitions() || repetitions != color.Repetitions() {
                        return String::new();
                    }
                    Some(repetitions)
                };
                let width = &width.Values().values;
                let style = &style.Values().values;
                let color = &color.Values().values;
                if width.len() != style.len() || width.len() != color.len() {
                    return String::new();
                }
                let mut segment = String::from("");
                for j in 0..width.len() {
                    if j > 0 {
                        segment.push_str(", ");
                    }
                    segment.push_string(&Segment(&width[j], &style[j], &color[j]));
                }
                result.push_str("repeat(");
                if let Some(repetitions) = repetitions {
                    result.push_string(&repetitions.CssText());
                } else {
                    result.push_str("auto");
                }
                result.push_str(", ");
                result.push_string(&segment);
                result.push(')');
            } else {
                if style_repeat.is_some() || color_repeat.is_some() {
                    return String::new();
                }
                result.push_string(&Segment(width, style, color));
            }
        }
        result
    }
    pub(super) fn GetShorthandValueForGapDecorationsRuleInsetCapJunction(
        &self,
        shorthand: Shorthand,
        direction: GapDirection,
        is_cap: bool,
    ) -> String {
        let expected = InsetProperties(direction);
        assert_eq!(
            shorthand.properties,
            if is_cap {
                &expected[..2]
            } else {
                &expected[2..]
            }
        );
        let v = self.GapValues(shorthand, 2);
        let mut out = v[0].CssText();
        if !std::ptr::eq(v[0], v[1]) {
            AppendValue(&mut out, v[1], " ");
        }
        out
    }
    pub(super) fn GetShorthandValueForGapDecorationsRuleInsetStartEnd(
        &self,
        shorthand: Shorthand,
    ) -> String {
        let v = self.GapValues(shorthand, 2);
        if !std::ptr::eq(v[0], v[1]) {
            String::new()
        } else {
            v[0].CssText()
        }
    }
    pub(super) fn GetShorthandValueForGapDecorationsRuleInset(
        &self,
        shorthand: Shorthand,
        direction: GapDirection,
    ) -> String {
        assert_eq!(shorthand.properties, &InsetProperties(direction));
        let v = self.GapValues(shorthand, 4);
        if AllEqual(&v) {
            v[0].CssText()
        } else {
            AppendFour([v[0], v[1], v[2], v[3]])
        }
    }
}

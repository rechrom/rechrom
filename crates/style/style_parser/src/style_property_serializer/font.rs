//! style_property_serializer.cc:941-1038,1348-1734.
use super::basic::{AppendValue, Identifier, IsIdentifier};
use super::{CSSValueID, CSSValuePayload, Id, Shorthand, String, StylePropertySerializer, Value};
use crate::production_css_value as values;
use std::rc::Rc;
fn StretchKeyword(value: &Value) -> Option<Rc<Value>> {
    if Identifier(value).is_some() {
        return None;
    } // caller keeps the original identifier allocation
    let number = match value.Payload() {
        CSSValuePayload::kNumericLiteralClass(v) => v.GetValueIfKnown(),
        CSSValuePayload::kMathFunctionClass(v) => v.GetValueIfKnown(),
        _ => None,
    }?;
    let id = match number {
        50.0 => CSSValueID::kUltraCondensed,
        62.5 => CSSValueID::kExtraCondensed,
        75.0 => CSSValueID::kCondensed,
        87.5 => CSSValueID::kSemiCondensed,
        100.0 => CSSValueID::kNormal,
        112.5 => CSSValueID::kSemiExpanded,
        125.0 => CSSValueID::kExpanded,
        150.0 => CSSValueID::kExtraExpanded,
        200.0 => CSSValueID::kUltraExpanded,
        _ => return None,
    };
    Some(values::identifier(id))
}
fn NonInitial(value: &Value, initial: CSSValueID) -> bool {
    Identifier(value).is_some_and(|id| id != initial)
}
impl StylePropertySerializer<'_> {
    fn AppendFontLonghandValueIfNotNormal(&self, id: Id, out: &mut String) -> bool {
        let original = self.Required(id);
        let keyword;
        let value = if id == Id::kFontStretch && Identifier(original).is_none() {
            let Some(value) = StretchKeyword(original) else {
                return false;
            };
            keyword = value;
            keyword.as_ref()
        } else {
            original
        };
        if IsIdentifier(value, CSSValueID::kNormal) {
            return true;
        }
        let text = if id == Id::kFontVariantLigatures && IsIdentifier(value, CSSValueID::kNone) {
            String::from("none")
        } else {
            value.CssText()
        };
        if text.empty() {
            return true;
        }
        if !out.empty() {
            match id {
                Id::kFontStyle => {}
                Id::kLineHeight => out.push_str(" / "),
                Id::kFontFamily
                | Id::kFontStretch
                | Id::kFontVariantCaps
                | Id::kFontVariantLigatures
                | Id::kFontVariantNumeric
                | Id::kFontVariantEastAsian
                | Id::kFontVariantAlternates
                | Id::kFontVariantPosition
                | Id::kFontVariantEmoji
                | Id::kFontWeight => out.push(' '),
                _ => unreachable!("source font prefix property"),
            }
        }
        out.push_string(&text);
        true
    }
    pub(super) fn FontValue(&self) -> String {
        // CommonShorthandChecks already established the entire original font
        // longhand set. Keep the exact source noninitial/list exclusion order.
        for id in [
            Id::kFontVariantLigatures,
            Id::kFontVariantNumeric,
            Id::kFontVariantEastAsian,
        ] {
            let value = self.Required(id);
            if NonInitial(value, CSSValueID::kNormal) || value.IsValueList() {
                return String::from("");
            }
        }
        if NonInitial(self.Required(Id::kFontKerning), CSSValueID::kAuto)
            || NonInitial(self.Required(Id::kFontOpticalSizing), CSSValueID::kAuto)
        {
            return String::from("");
        }
        for id in [
            Id::kFontVariationSettings,
            Id::kFontFeatureSettings,
            Id::kFontVariantAlternates,
        ] {
            let value = self.Required(id);
            if NonInitial(value, CSSValueID::kNormal) || value.IsValueList() {
                return String::from("");
            }
        }
        for id in [Id::kFontVariantPosition, Id::kFontVariantEmoji] {
            if NonInitial(self.Required(id), CSSValueID::kNormal) {
                return String::from("");
            }
        }
        if style_model::production_typography_features::CSSFontSizeAdjustEnabled() {
            let value = self.Required(Id::kFontSizeAdjust);
            if NonInitial(value, CSSValueID::kNone) || value.IsNumericLiteralValue() {
                return String::from("");
            }
        }
        if style_model::production_typography_features::FontLanguageOverrideEnabled() {
            let value = self.Required(Id::kFontLanguageOverride);
            if NonInitial(value, CSSValueID::kNormal) || value.IsStringValue() {
                return String::from("");
            }
        }
        let shorthand = Shorthand::For(Id::kFont);
        let first = self.Required(shorthand.properties[0]);
        if let CSSValuePayload::kPendingSystemFontValueClass(font) = first.Payload() {
            if shorthand.properties[1..]
                .iter()
                .any(|&id| self.Get(id).is_none_or(|value| value != first))
            {
                return String::from("");
            }
            return values::identifier(font.system_font_id).CssText();
        }
        if shorthand.properties[1..]
            .iter()
            .any(|&id| self.Required(id).IsPendingSystemFontValue())
        {
            return String::from("");
        }
        let mut result = String::from("");
        self.AppendFontLonghandValueIfNotNormal(Id::kFontStyle, &mut result);
        let caps = self.Required(Id::kFontVariantCaps);
        if Identifier(caps)
            .is_some_and(|id| id != CSSValueID::kSmallCaps && id != CSSValueID::kNormal)
        {
            return String::from("");
        }
        self.AppendFontLonghandValueIfNotNormal(Id::kFontVariantCaps, &mut result);
        self.AppendFontLonghandValueIfNotNormal(Id::kFontWeight, &mut result);
        if !self.AppendFontLonghandValueIfNotNormal(Id::kFontStretch, &mut result) {
            return String::new();
        }
        AppendValue(&mut result, self.Required(Id::kFontSize), " ");
        self.AppendFontLonghandValueIfNotNormal(Id::kLineHeight, &mut result);
        AppendValue(&mut result, self.Required(Id::kFontFamily), " ");
        result
    }
    pub(super) fn FontVariantValue(&self) -> String {
        let mut result = String::from("");
        self.AppendFontLonghandValueIfNotNormal(Id::kFontVariantLigatures, &mut result);
        let none = result == String::from("none");
        let length = result.length();
        for id in [
            Id::kFontVariantCaps,
            Id::kFontVariantAlternates,
            Id::kFontVariantNumeric,
            Id::kFontVariantEastAsian,
            Id::kFontVariantPosition,
            Id::kFontVariantEmoji,
        ] {
            self.AppendFontLonghandValueIfNotNormal(id, &mut result);
        }
        if none && result.length() != length {
            return String::from("");
        }
        if result.empty() {
            String::from("normal")
        } else {
            result
        }
    }
    pub(super) fn FontSynthesisValue(&self) -> String {
        let mut out = String::from("");
        for (id, text) in [
            (Id::kFontSynthesisWeight, "weight"),
            (Id::kFontSynthesisStyle, "style"),
            (Id::kFontSynthesisSmallCaps, "small-caps"),
        ] {
            if IsIdentifier(self.Required(id), CSSValueID::kAuto) {
                if !out.empty() {
                    out.push(' ');
                }
                out.push_str(text);
            }
        }
        if out.empty() {
            String::from("none")
        } else {
            out
        }
    }
    pub(super) fn OffsetValue(&self) -> String {
        let position = self.Get(Id::kOffsetPosition);
        let path = self.Get(Id::kOffsetPath);
        let distance = self.Get(Id::kOffsetDistance);
        let rotate = self.Get(Id::kOffsetRotate);
        let anchor = self.Get(Id::kOffsetAnchor);
        let zero = |value: &Value| matches!(value.Payload(),CSSValuePayload::kNumericLiteralClass(v) if v.DoubleValue()==0.0);
        let use_distance = distance.is_some_and(|v| !zero(v));
        let list = rotate.and_then(|v| match v.Payload() {
            CSSValuePayload::kValueListClass(list) => Some(list),
            _ => None,
        });
        let auto = list
            .is_some_and(|v| v.values.len() == 1 && IsIdentifier(&v.values[0], CSSValueID::kAuto));
        let zero_rotation = list.is_some_and(|v| v.values.len() == 1 && zero(&v.values[0]));
        let auto_zero = list.is_some_and(|v| {
            v.values.len() == 2
                && zero(&v.values[1])
                && IsIdentifier(&v.values[0], CSSValueID::kAuto)
        });
        let use_rotate = rotate.is_some_and(|v| {
            use_distance && zero_rotation
                || !IsIdentifier(v, CSSValueID::kAuto) && !auto && !auto_zero
        });
        let use_path =
            path.is_some_and(|v| use_rotate || use_distance || !IsIdentifier(v, CSSValueID::kNone));
        let use_position =
            position.is_some_and(|v| !use_path || !IsIdentifier(v, CSSValueID::kNormal));
        let use_anchor = anchor.is_some_and(|v| !IsIdentifier(v, CSSValueID::kAuto));
        let mut out = String::from("");
        if use_position {
            out.push_string(&position.unwrap().CssText());
        }
        if use_path {
            AppendValue(&mut out, path.unwrap(), " ");
        }
        if use_distance {
            out.push(' ');
            out.push_string(&distance.unwrap().CssText());
        }
        if use_rotate {
            out.push(' ');
            out.push_string(&rotate.unwrap().CssText());
        }
        if use_anchor {
            out.push_str(" / ");
            out.push_string(&anchor.unwrap().CssText());
        }
        out
    }
}

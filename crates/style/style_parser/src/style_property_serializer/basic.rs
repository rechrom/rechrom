//! style_property_serializer.cc:1039-1065,1736-1842,2204-2234,
//! 2701-2768,3028-3042,3285-3522. Source builders only; original values remain
//! borrowed from the compiled declaration throughout serialization.
use super::{CSSValueID, CSSValuePayload, Id, Shorthand, String, StylePropertySerializer, Value};
pub(super) fn Identifier(value: &Value) -> Option<CSSValueID> {
    match value.Payload() {
        CSSValuePayload::kIdentifierClass(value) => Some(value.0),
        _ => None,
    }
}
pub(super) fn IsIdentifier(value: &Value, id: CSSValueID) -> bool {
    Identifier(value) == Some(id)
}
pub(super) fn AppendValue(out: &mut String, value: &Value, separator: &str) {
    if !out.empty() {
        out.push_str(separator);
    }
    out.push_string(&value.CssText());
}
impl StylePropertySerializer<'_> {
    pub(super) fn Get2Values(&self, shorthand: Shorthand) -> String {
        let first = self.Required(shorthand.properties[0]);
        let last = self.Required(shorthand.properties[1]);
        let mut result = first.CssText();
        if first != last {
            result.push(' ');
            result.push_string(&last.CssText());
        }
        result
    }
    pub(super) fn Get4Values(&self, shorthand: Shorthand) -> String {
        let v = shorthand.properties.map_values(self);
        let show_left = v[1] != v[3];
        let show_bottom = v[0] != v[2] || show_left;
        let show_right = v[0] != v[1] || show_bottom;
        let mut result = v[0].CssText();
        for (value, show) in [(v[1], show_right), (v[2], show_bottom), (v[3], show_left)] {
            if show {
                AppendValue(&mut result, value, " ");
            }
        }
        result
    }
    pub(super) fn GetShorthandValue(&self, shorthand: Shorthand) -> String {
        self.GetShorthandValueWithSeparator(shorthand, " ")
    }
    pub(super) fn GetShorthandValueWithSeparator(
        &self,
        shorthand: Shorthand,
        separator: &str,
    ) -> String {
        let mut result = String::from("");
        for &id in shorthand.properties {
            let value = self.Required(id);
            let text = value.CssText();
            if value.IsInitialValue() {
                continue;
            }
            if !result.empty() {
                result.push_str(separator);
            }
            result.push_string(&text);
        }
        result
    }
    pub(super) fn GetShorthandValueForBorder(&self, shorthand: Shorthand) -> String {
        assert_eq!(shorthand.properties.len(), 3);
        let texts: Vec<_> = shorthand
            .properties
            .iter()
            .map(|&id| self.Required(id).CssText())
            .collect();
        SerializeBorderValues(&texts)
    }
    pub(super) fn GetCommonValue(&self, shorthand: Shorthand) -> String {
        let mut result = None;
        for &id in shorthand.properties {
            let text = self.Required(id).CssText();
            if result.as_ref().is_some_and(|result| *result != text) {
                return String::new();
            }
            result = Some(text);
        }
        result.unwrap_or_default()
    }
    pub(super) fn ContainerValue(&self) -> String {
        let name = self.Required(Id::kContainerName);
        let kind = self.Required(Id::kContainerType);
        let mut out = name.CssText();
        if !IsIdentifier(kind, CSSValueID::kNormal) {
            AppendValue(&mut out, kind, " / ");
        }
        out
    }
    pub(super) fn GetShorthandValueForColumns(&self, _shorthand: Shorthand) -> String {
        let width = self.Required(Id::kColumnWidth);
        let count = self.Required(Id::kColumnCount);
        let height = self.Required(Id::kColumnHeight);
        let width_auto = IsIdentifier(width, CSSValueID::kAuto);
        let count_auto = IsIdentifier(count, CSSValueID::kAuto);
        let mut result = String::from("");
        if width_auto && count_auto {
            result.push_str("auto");
        } else {
            if !width_auto {
                result.push_string(&width.CssText());
            }
            if !count_auto {
                AppendValue(&mut result, count, " ");
            }
        }
        if !IsIdentifier(height, CSSValueID::kAuto) {
            result.push_str(" / ");
            result.push_string(&height.CssText());
        }
        result
    }
    pub(super) fn TextDecorationValue(&self) -> String {
        let mut out = String::from("");
        for &id in Shorthand::For(Id::kTextDecoration).properties {
            let value = self.Required(id);
            let text = value.CssText();
            if value.IsInitialValue() {
                continue;
            }
            let omit = match id {
                Id::kTextDecorationThickness => Some(CSSValueID::kAuto),
                Id::kTextDecorationLine => Some(CSSValueID::kNone),
                Id::kTextDecorationStyle => Some(CSSValueID::kSolid),
                Id::kTextDecorationColor => Some(CSSValueID::kCurrentcolor),
                _ => None,
            };
            if omit.is_some_and(|id| IsIdentifier(value, id)) {
                continue;
            }
            if !out.empty() {
                out.push(' ');
            }
            out.push_string(&text);
        }
        if out.empty() {
            String::from("none")
        } else {
            out
        }
    }
    pub(super) fn PageBreakPropertyValue(&self, shorthand: Shorthand) -> String {
        let value = self.Required(shorthand.properties[0]);
        match Identifier(value).expect("page-break parse stores identifier") {
            CSSValueID::kPage => String::from("always"),
            CSSValueID::kAuto | CSSValueID::kLeft | CSSValueID::kRight | CSSValueID::kAvoid => {
                value.CssText()
            }
            _ => String::new(),
        }
    }
    pub(super) fn ContainIntrinsicSizeValue(&self) -> String {
        let shorthand = Shorthand::For(Id::kContainIntrinsicSize);
        let common = self.GetCommonValue(shorthand);
        if !common.IsNull() {
            common
        } else {
            self.GetShorthandValue(shorthand)
        }
    }
    pub(super) fn TextBoxValue(&self) -> String {
        let trim = self.Required(Id::kTextBoxTrim);
        let trim_id = Identifier(trim).expect("text-box-trim identifier");
        let edge = self.Required(Id::kTextBoxEdge);
        if IsIdentifier(edge, CSSValueID::kAuto) {
            return if trim_id == CSSValueID::kNone {
                String::from("normal")
            } else {
                trim.CssText()
            };
        }
        if trim_id == CSSValueID::kTrimBoth {
            return edge.CssText();
        }
        let mut out = trim.CssText();
        AppendValue(&mut out, edge, " ");
        out
    }
    pub(super) fn TextSpacingValue(&self) -> String {
        let autospace = self.Required(Id::kTextAutospace);
        let trim = self.Required(Id::kTextSpacingTrim);
        let a = Identifier(autospace).expect("text-autospace identifier");
        let t = Identifier(trim).expect("text-spacing-trim identifier");
        if a == CSSValueID::kNormal && t == CSSValueID::kNormal {
            return String::from("normal");
        }
        if a == CSSValueID::kNoAutospace && t == CSSValueID::kSpaceAll {
            return String::from("none");
        }
        let mut out = String::from("");
        if t != CSSValueID::kNormal {
            out.push_string(&trim.CssText());
        }
        if a != CSSValueID::kNormal {
            AppendValue(&mut out, autospace, " ");
        }
        debug_assert!(!out.empty());
        out
    }
    pub(super) fn TextWrapValue(&self) -> String {
        let (Some(mode), Some(style)) = (self.Get(Id::kTextWrapMode), self.Get(Id::kTextWrapStyle))
        else {
            return String::from("");
        };
        // CSSIdentifierValue::ConvertTo + ComputedStyleInitialValues defaults.
        let mode = Identifier(mode).unwrap_or_else(|| {
            assert!(mode.IsInitialValue());
            CSSValueID::kWrap
        });
        let style = Identifier(style).unwrap_or_else(|| {
            assert!(style.IsInitialValue());
            CSSValueID::kAuto
        });
        if style == CSSValueID::kAuto {
            return crate::production_css_value::identifier(mode).CssText();
        }
        if mode == CSSValueID::kWrap {
            return crate::production_css_value::identifier(style).CssText();
        }
        let mut out = crate::production_css_value::identifier(mode).CssText();
        out.push(' ');
        out.push_string(&crate::production_css_value::identifier(style).CssText());
        out
    }
    pub(super) fn WhiteSpaceValue(&self) -> String {
        let (Some(collapse), Some(wrap)) = (
            self.Get(Id::kWhiteSpaceCollapse),
            self.Get(Id::kTextWrapMode),
        ) else {
            return String::from("");
        };
        let collapse = Identifier(collapse).unwrap_or_else(|| {
            assert!(collapse.IsInitialValue());
            CSSValueID::kCollapse
        });
        let wrap = Identifier(wrap).unwrap_or_else(|| {
            assert!(wrap.IsInitialValue());
            CSSValueID::kWrap
        });
        // Exact style white-space enum combinations (ToWhiteSpace/IsValidWhiteSpace).
        let combined = match (collapse, wrap) {
            (CSSValueID::kCollapse, CSSValueID::kWrap) => Some(CSSValueID::kNormal),
            (CSSValueID::kCollapse, CSSValueID::kNowrap) => Some(CSSValueID::kNowrap),
            (CSSValueID::kPreserve, CSSValueID::kNowrap) => Some(CSSValueID::kPre),
            (CSSValueID::kPreserve, CSSValueID::kWrap) => Some(CSSValueID::kPreWrap),
            (CSSValueID::kPreserveBreaks, CSSValueID::kWrap) => Some(CSSValueID::kPreLine),
            (CSSValueID::kBreakSpaces, CSSValueID::kWrap) => Some(CSSValueID::kBreakSpaces),
            _ => None,
        };
        if let Some(value) = combined {
            return crate::production_css_value::identifier(value).CssText();
        }
        let mut out = String::from("");
        if collapse != CSSValueID::kCollapse {
            out.push_string(&crate::production_css_value::identifier(collapse).CssText());
        }
        if wrap != CSSValueID::kWrap {
            if !out.empty() {
                out.push(' ');
            }
            out.push_string(&crate::production_css_value::identifier(wrap).CssText());
        }
        debug_assert!(!out.empty());
        out
    }
    pub(super) fn LineClampValue(&self, webkit: bool) -> String {
        let lines = self.Required(Id::kMaxLines);
        let ellipsis = self.Required(Id::kBlockEllipsis);
        let continuation = self.Required(Id::kContinue);
        let e = Identifier(ellipsis).expect("block-ellipsis identifier");
        let c = Identifier(continuation).expect("continue identifier");
        if c == CSSValueID::kNormal {
            return if lines.IsIdentifierValue() && e == CSSValueID::kNoEllipsis {
                debug_assert!(IsIdentifier(lines, CSSValueID::kAuto));
                String::from("none")
            } else {
                String::from("")
            };
        }
        let mut out = String::from("");
        if !lines.IsIdentifierValue() || e == CSSValueID::kEllipsis {
            if !lines.IsNumericLiteralValue() && webkit {
                return String::from("");
            }
            out.push_string(&lines.CssText());
        } else if webkit {
            return String::from("");
        }
        if e != CSSValueID::kEllipsis {
            if webkit {
                return String::from("");
            }
            AppendValue(&mut out, ellipsis, " ");
        }
        if c == CSSValueID::kWebkitLegacy {
            if !webkit {
                AppendValue(&mut out, continuation, " ");
            }
        } else {
            if webkit {
                return String::from("");
            }
            debug_assert_eq!(c, CSSValueID::kCollapse);
        }
        debug_assert!(!out.empty());
        out
    }
    pub(super) fn PositionTryValue(&self, shorthand: Shorthand) -> String {
        assert_eq!(shorthand.properties.len(), 2);
        assert_eq!(shorthand.properties[0], Id::kPositionTryOrder);
        let order = self.Required(Id::kPositionTryOrder);
        let fallbacks = self.Required(shorthand.properties[1]);
        let mut out = String::from("");
        if !IsIdentifier(order, CSSValueID::kNormal) {
            out.push_string(&order.CssText());
        }
        AppendValue(&mut out, fallbacks, " ");
        out
    }
}
trait FourValues {
    fn map_values<'a>(&self, serializer: &StylePropertySerializer<'a>) -> [&'a Value; 4];
}
impl FourValues for [Id] {
    fn map_values<'a>(&self, serializer: &StylePropertySerializer<'a>) -> [&'a Value; 4] {
        assert_eq!(self.len(), 4);
        [
            serializer.Required(self[0]),
            serializer.Required(self[1]),
            serializer.Required(self[2]),
            serializer.Required(self[3]),
        ]
    }
}
pub(super) fn SerializeBorderValues(values: &[String]) -> String {
    const INITIAL: [&str; 3] = ["medium", "none", "currentcolor"];
    let mut out = String::from("");
    for (value, initial) in values.iter().zip(INITIAL) {
        if value.IsNull() || *value == String::from(initial) {
            continue;
        }
        if !out.empty() {
            out.push(' ');
        }
        out.push_string(value);
    }
    out
}

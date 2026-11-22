//! style_property_serializer.cc:2771-3027.
use super::basic::{AppendValue, IsIdentifier};
use super::border::InitialSpecified;
use super::{CSSValueID, Id, Shorthand, String, StylePropertySerializer, Value};
use crate::{cssom_utils, production_css_value::CSSValueList};
fn List(value: &Value) -> Option<&CSSValueList> {
    value.IsValueList().then(|| value.ValueListView())
}
fn InitialList(value: Option<&CSSValueList>, id: Id) -> bool {
    value.is_some_and(|list| {
        list.values.len() == 1 && list.values[0].as_ref() == InitialSpecified(id).as_ref()
    })
}
impl StylePropertySerializer<'_> {
    pub(super) fn GetShorthandValueForGrid(&self, shorthand: Shorthand) -> String {
        assert_eq!(shorthand.properties.len(), 6);
        let ids = shorthand.properties;
        let rows = self.Required(ids[0]);
        let columns = self.Required(ids[1]);
        let areas = self.Required(ids[2]);
        let flow = self.Required(ids[3]);
        let auto_rows = self.Required(ids[4]);
        let auto_columns = self.Required(ids[5]);
        let row_list = List(auto_rows);
        let column_list = List(auto_columns);
        let flow_list = List(flow);
        let row_initial =
            InitialList(row_list, Id::kGridAutoRows) || IsIdentifier(auto_rows, CSSValueID::kAuto);
        let column_initial = InitialList(column_list, Id::kGridAutoColumns)
            || IsIdentifier(auto_columns, CSSValueID::kAuto);
        let flow_initial =
            InitialList(flow_list, Id::kGridAutoFlow) || IsIdentifier(flow, CSSValueID::kRow);
        let specified_rows = row_list.is_some() && !row_initial;
        let specified_columns = column_list.is_some() && !column_initial;
        if (flow_list.is_some() || row_list.is_some() || column_list.is_some())
            && areas != InitialSpecified(Id::kGridTemplateAreas).as_ref()
        {
            return String::new();
        }
        let template_rows = !IsIdentifier(rows, CSSValueID::kNone);
        let template_columns = !IsIdentifier(columns, CSSValueID::kNone);
        if template_rows && specified_rows
            || template_columns && specified_columns
            || specified_rows && specified_columns
        {
            return String::new();
        }
        if flow_list.is_none() || flow_initial && !(specified_columns || specified_rows) {
            return self.GetShorthandValueForGridTemplate(shorthand);
        }
        if template_rows && template_columns {
            return String::new();
        }
        let mut flow_text = String::from("auto-flow");
        if flow_list.is_some_and(|list| {
            list.values
                .iter()
                .any(|value| IsIdentifier(value, CSSValueID::kDense))
        }) {
            flow_text.push_str(" dense");
        }
        let mut out = String::from("");
        if flow_list.is_some_and(|list| {
            list.values
                .iter()
                .any(|value| IsIdentifier(value, CSSValueID::kColumn))
        }) {
            out.push_string(&rows.CssText());
            out.push_str(" / ");
            out.push_string(&flow_text);
            if specified_columns {
                AppendValue(&mut out, auto_columns, " ");
            }
        } else {
            out.push_string(&flow_text);
            if specified_rows {
                AppendValue(&mut out, auto_rows, " ");
            }
            out.push_str(" / ");
            out.push_string(&columns.CssText());
        }
        out
    }
    pub(super) fn GetShorthandValueForGridArea(&self, shorthand: Shorthand) -> String {
        assert_eq!(shorthand.properties.len(), 4);
        let v = shorthand
            .properties
            .iter()
            .map(|&id| self.Required(id))
            .collect::<Vec<_>>();
        let column_start = cssom_utils::IncludeDependentGridLineEndValue(v[0], v[1]);
        let row_end = cssom_utils::IncludeDependentGridLineEndValue(v[0], v[2]);
        let column_end = cssom_utils::IncludeDependentGridLineEndValue(v[1], v[3]);
        let mut out = v[0].CssText();
        for (value, include) in [
            (v[1], column_start || row_end || column_end),
            (v[2], row_end || column_end),
            (v[3], column_end),
        ] {
            if include {
                out.push_str(" / ");
                out.push_string(&value.CssText());
            }
        }
        out
    }
    pub(super) fn GetShorthandValueForGridLine(&self, shorthand: Shorthand) -> String {
        assert_eq!(shorthand.properties.len(), 2);
        let start = self.Required(shorthand.properties[0]);
        let end = self.Required(shorthand.properties[1]);
        let mut out = start.CssText();
        if cssom_utils::IncludeDependentGridLineEndValue(start, end) {
            out.push_str(" / ");
            out.push_string(&end.CssText());
        }
        out
    }
    pub(super) fn GetShorthandValueForGridLanes(&self, shorthand: Shorthand) -> String {
        let direction = self.RequiredOwner(shorthand.properties[3]);
        let index = if cssom_utils::IsGridLanesNormalDirectionValue(&direction)
            || cssom_utils::IsGridLanesColumnDirectionValue(&direction)
        {
            1
        } else {
            2
        };
        cssom_utils::ComputedValueForGridLanesShorthand(
            Some(self.RequiredOwner(shorthand.properties[index])),
            Some(self.RequiredOwner(shorthand.properties[0])),
            direction,
        )
        .CssText()
    }
    pub(super) fn GetShorthandValueForGridTemplate(&self, shorthand: Shorthand) -> String {
        cssom_utils::ComputedValueForGridTemplateShorthand(
            self.RequiredOwner(shorthand.properties[0]),
            self.RequiredOwner(shorthand.properties[1]),
            Some(self.RequiredOwner(shorthand.properties[2])),
        )
        .CssText()
    }
}

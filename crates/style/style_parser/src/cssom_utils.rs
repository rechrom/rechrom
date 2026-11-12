//! CSSOMUtils, cssom_utils.cc:16-275, Chromium 6c1d401fcca5.
//! Results contain original CSSValue Rc aliases; grid-area strings are the
//! exact source serialization of the existing named-area owner.
use crate::css_value::CSSValuePayload;
use crate::production_css_value::{
    self as values, CSSGridTemplateAreasValue, CSSValueList, ListSeparator, Value,
};
use foundation::{CSSValueID, String};
use std::rc::Rc;
fn Identifier(value: &Value) -> Option<CSSValueID> {
    match value.Payload() {
        CSSValuePayload::kIdentifierClass(value) => Some(value.0),
        _ => None,
    }
}
fn List(value: &Value) -> Option<&CSSValueList> {
    value.IsValueList().then(|| value.ValueListView())
}
pub fn IncludeDependentGridLineEndValue(start: &Value, end: &Value) -> bool {
    let initial = Identifier(end) == Some(CSSValueID::kAuto);
    end.IsValueList() || end != start && (start.IsCustomIdentValue() || !initial)
}
pub fn IsAutoValue(value: &Value) -> bool {
    Identifier(value) == Some(CSSValueID::kAuto)
}
pub fn IsNoneValue(value: &Value) -> bool {
    Identifier(value) == Some(CSSValueID::kNone)
}
pub fn IsEmptyValueList(value: &Value) -> bool {
    List(value).is_some_and(|list| list.values.is_empty())
}
pub fn HasGridRepeatValue(value: Option<&CSSValueList>) -> bool {
    value.is_some_and(|list| list.values.iter().any(|value| value.IsGridRepeatValue()))
}
pub fn IsGridLanesNormalDirectionValue(value: &Value) -> bool {
    if let Some(id) = Identifier(value) {
        assert_eq!(id, CSSValueID::kNormal);
        true
    } else {
        false
    }
}
pub fn IsGridLanesColumnDirectionValue(value: &Value) -> bool {
    if let Some(list) = List(value) {
        assert!(!list.values.is_empty());
        Identifier(&list.values[0]) == Some(CSSValueID::kColumn)
    } else {
        false
    }
}
fn NamedGridAreaTextForPosition(
    areas: &CSSGridTemplateAreasValue,
    row: u32,
    column: u32,
) -> String {
    for (name, area) in areas.grid_area_map.iter() {
        if row >= area.rows.StartLine()
            && row < area.rows.EndLine()
            && column >= area.columns.StartLine()
            && column < area.columns.EndLine()
        {
            return name.clone();
        }
    }
    String::from(".")
}
pub fn SerializeGridAreaText(
    areas: &CSSGridTemplateAreasValue,
    fixed: u32,
    is_row: bool,
) -> String {
    let count = if is_row {
        areas.column_count
    } else {
        areas.row_count
    };
    let mut out = String::new();
    for i in 0..count {
        out.push_string(&if is_row {
            NamedGridAreaTextForPosition(areas, fixed, i)
        } else {
            NamedGridAreaTextForPosition(areas, i, fixed)
        });
        if i != count - 1 {
            out.push(' ');
        }
    }
    out
}
pub fn ComputedValueForGridTemplateShorthand(
    rows: Rc<Value>,
    columns: Rc<Value>,
    areas: Option<Rc<Value>>,
) -> Rc<Value> {
    let initial_rows = IsNoneValue(&rows);
    let initial_columns = IsNoneValue(&columns);
    let initial_areas = areas.as_ref().is_none_or(|value| IsNoneValue(value));
    let mut result = Vec::new();
    if initial_areas && initial_rows && initial_columns {
        result.push(rows);
        return values::list(result, ListSeparator::Slash);
    }
    if !initial_areas && initial_rows {
        return values::list(result, ListSeparator::Slash);
    }
    if !rows.IsValueList() || initial_areas {
        result.push(rows);
        result.push(columns);
        return values::list(result, ListSeparator::Slash);
    }
    let row_list = List(&rows).expect("source grid row list");
    if HasGridRepeatValue(Some(row_list)) || HasGridRepeatValue(List(&columns)) {
        return values::list(result, ListSeparator::Slash);
    }
    let areas = areas.as_ref().expect("noninitial grid areas");
    let CSSValuePayload::kGridTemplateAreasClass(areas) = areas.Payload() else {
        unreachable!("source typed grid template areas");
    };
    let mut serialized_rows = Vec::new();
    let mut row = 0;
    for value in &row_list.values {
        if value.IsGridLineNamesValue() {
            serialized_rows.push(value.clone());
            continue;
        }
        let text = SerializeGridAreaText(areas, row, true);
        debug_assert!(!text.empty());
        serialized_rows.push(values::string(text));
        row += 1;
        if !IsAutoValue(value) {
            serialized_rows.push(value.clone());
        }
    }
    if row != areas.row_count {
        return values::list(result, ListSeparator::Slash);
    }
    result.push(values::list(serialized_rows, ListSeparator::Space));
    if !initial_columns {
        result.push(columns);
    }
    values::list(result, ListSeparator::Slash)
}
pub fn ComputedValueForGridLanesShorthand(
    tracks: Option<Rc<Value>>,
    areas: Option<Rc<Value>>,
    direction: Rc<Value>,
) -> Rc<Value> {
    let initial_tracks = tracks.as_ref().is_none_or(|value| IsNoneValue(value));
    let initial_areas = areas.as_ref().is_none_or(|value| IsNoneValue(value));
    let mut result = Vec::new();
    if !initial_areas {
        let areas = areas.as_ref().expect("noninitial grid areas");
        if IsGridLanesColumnDirectionValue(&direction)
            || IsGridLanesNormalDirectionValue(&direction)
        {
            result.push(areas.clone());
        } else {
            let CSSValuePayload::kGridTemplateAreasClass(areas) = areas.Payload() else {
                unreachable!("source typed grid template areas");
            };
            result.push(values::string(SerializeGridAreaText(areas, 0, false)));
        }
    }
    if !initial_tracks {
        result.push(tracks.expect("noninitial grid tracks"));
    }
    result.push(direction);
    values::list(result, ListSeparator::Space)
}

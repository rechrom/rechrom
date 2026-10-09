// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Typed CSSGridTemplateAreasValue, retaining the source native area map.
#![allow(non_snake_case)]
use crate::css_value::CSSValueSubclass;
use foundation::String;
use layoutng_style::style::grid_area::NamedGridAreaMap;
// cpp: core/css/css_grid_template_areas_value.h:37-65; .cc:41-101.
pub struct CSSGridTemplateAreasValue {
    pub grid_area_map: NamedGridAreaMap,
    pub row_count: u32,
    pub column_count: u32,
}
impl CSSValueSubclass for CSSGridTemplateAreasValue {
    fn CustomCSSText(&self) -> String {
        let mut text = String::new();
        for row in 0..self.row_count {
            if row != 0 {
                text.push_str(" ");
            }
            text.push_str("\"");
            for column in 0..self.column_count {
                if column != 0 {
                    text.push_str(" ");
                }
                let name = self.grid_area_map.iter().find_map(|(name, area)| {
                    (row >= area.rows.StartLine()
                        && row < area.rows.EndLine()
                        && column >= area.columns.StartLine()
                        && column < area.columns.EndLine())
                    .then_some(name)
                });
                if let Some(name) = name {
                    text.push_string(name);
                } else {
                    text.push_str(".");
                }
            }
            text.push_str("\"");
        }
        text
    }
    fn Equals(&self, other: &Self) -> bool {
        self.grid_area_map == other.grid_area_map
            && self.row_count == other.row_count
            && self.column_count == other.column_count
    }
}

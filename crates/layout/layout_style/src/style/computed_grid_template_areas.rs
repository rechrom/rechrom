use foundation::{StrCat, String, Visitor};

use super::grid_area::{GridSpan, NamedGridAreaMap};
use super::grid_enums::GridTrackSizingDirection;
use super::named_grid_lines_map::NamedGridLinesMap;

// cpp: layoutng_style/style/computed_grid_template_areas.h:16-44
pub struct ComputedGridTemplateAreas {
    pub named_areas: NamedGridAreaMap,
    pub implicit_named_grid_row_lines: NamedGridLinesMap,
    pub implicit_named_grid_column_lines: NamedGridLinesMap,
    pub row_count: u32,
    pub column_count: u32,
}

#[allow(non_snake_case)]
impl ComputedGridTemplateAreas {
    // cpp: layoutng_style/style/computed_grid_template_areas.h:24
    pub fn Trace(&self, _visitor: Option<&mut Visitor>) {}

    // cpp: layoutng_style/style/computed_grid_template_areas.h:33-35
    // cpp: layoutng_style/style/computed_grid_template_areas.cc:8-27
    pub fn CreateImplicitNamedGridLinesFromGridArea(
        named_areas: &NamedGridAreaMap,
        direction: GridTrackSizingDirection,
    ) -> NamedGridLinesMap {
        let mut named_grid_lines = NamedGridLinesMap::default();
        for (name, area) in named_areas.iter() {
            let area_span: GridSpan = if direction == GridTrackSizingDirection::kForRows {
                area.rows
            } else {
                area.columns
            };
            let start = named_grid_lines
                .entry(StrCat(&[name.clone(), String::from("-start")]))
                .or_default();
            start.push(area_span.StartLine());
            start.sort();
            let end = named_grid_lines
                .entry(StrCat(&[name.clone(), String::from("-end")]))
                .or_default();
            end.push(area_span.EndLine());
            end.sort();
        }
        named_grid_lines
    }

    // cpp: layoutng_style/style/computed_grid_template_areas.h:20-22
    // cpp: layoutng_style/style/computed_grid_template_areas.cc:29-39
    pub fn new(named_areas: &NamedGridAreaMap, row_count: u32, column_count: u32) -> Self {
        Self {
            named_areas: named_areas.clone(),
            implicit_named_grid_row_lines: Self::CreateImplicitNamedGridLinesFromGridArea(
                named_areas,
                GridTrackSizingDirection::kForRows,
            ),
            implicit_named_grid_column_lines: Self::CreateImplicitNamedGridLinesFromGridArea(
                named_areas,
                GridTrackSizingDirection::kForColumns,
            ),
            row_count,
            column_count,
        }
    }
}

// cpp: layoutng_style/style/computed_grid_template_areas.h:19
impl Default for ComputedGridTemplateAreas {
    fn default() -> Self {
        Self {
            named_areas: NamedGridAreaMap::default(),
            implicit_named_grid_row_lines: NamedGridLinesMap::default(),
            implicit_named_grid_column_lines: NamedGridLinesMap::default(),
            row_count: 0,
            column_count: 0,
        }
    }
}

// cpp: layoutng_style/style/computed_grid_template_areas.h:26-31
impl PartialEq for ComputedGridTemplateAreas {
    fn eq(&self, other: &Self) -> bool {
        self.named_areas == other.named_areas
            && self.row_count == other.row_count
            && self.column_count == other.column_count
    }
}
impl Eq for ComputedGridTemplateAreas {}

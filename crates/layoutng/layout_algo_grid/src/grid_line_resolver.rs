use crate::grid_named_line_collection::GridNamedLineCollection;
use foundation::{AtomicString, IsParallelWritingMode, Persistent, StrCat, String};
use layoutng_style::style::{
    computed_grid_template_areas::ComputedGridTemplateAreas,
    computed_grid_track_list::ComputedGridTrackList,
    computed_style::ComputedStyle,
    grid_area::{GridArea, GridSpan, NamedGridAreaMap, K_GRID_MAX_TRACKS},
    grid_enums::{
        GridPositionSide::{self, *},
        GridTrackSizingDirection::{self, *},
    },
    grid_position::{GridPosition, GridPositionType},
    named_grid_lines_map::NamedGridLinesMap,
};

// cpp: layoutng_grid/grid_line_resolver.cc:19-23
fn DirectionFromSide(side: GridPositionSide) -> GridTrackSizingDirection {
    if matches!(side, kColumnStartSide | kColumnEndSide) {
        kForColumns
    } else {
        kForRows
    }
}
// The String conversion preserves UTF-16 units (including unpaired surrogates).
fn PositionName(position: &GridPosition) -> String {
    String::from_utf16(position.NamedGridLine().utf16_units().unwrap_or_default())
}
// cpp: layoutng_grid/grid_line_resolver.cc:25-30
fn ImplicitNamedGridLineForSide(name: &String, side: GridPositionSide) -> String {
    StrCat(&[
        name.clone(),
        String::from(if matches!(side, kColumnStartSide | kRowStartSide) {
            "-start"
        } else {
            "-end"
        }),
    ])
}

// cpp: layoutng_grid/grid_line_resolver.h:27-158
#[derive(Clone)]
pub struct GridLineResolver {
    style_: Persistent<ComputedStyle>,
    column_auto_repetitions_: u32,
    row_auto_repetitions_: u32,
    subgridded_columns_span_size_: u32,
    subgridded_rows_span_size_: u32,
    subgridded_columns_merged_explicit_grid_line_names_: Option<NamedGridLinesMap>,
    subgridded_rows_merged_explicit_grid_line_names_: Option<NamedGridLinesMap>,
    subgridded_columns_merged_implicit_grid_line_names_: Option<NamedGridLinesMap>,
    subgridded_rows_merged_implicit_grid_line_names_: Option<NamedGridLinesMap>,
    subgrid_merged_named_areas_: Option<NamedGridAreaMap>,
}
impl GridLineResolver {
    // cpp: layoutng_grid/grid_line_resolver.h:31-37,138-158
    pub fn new(style: &ComputedStyle, columns: u32, rows: u32) -> Self {
        Self {
            style_: Persistent::from_ptr((style as *const ComputedStyle).cast_mut()),
            column_auto_repetitions_: columns,
            row_auto_repetitions_: rows,
            subgridded_columns_span_size_: u32::MAX,
            subgridded_rows_span_size_: u32::MAX,
            subgridded_columns_merged_explicit_grid_line_names_: None,
            subgridded_rows_merged_explicit_grid_line_names_: None,
            subgridded_columns_merged_implicit_grid_line_names_: None,
            subgridded_rows_merged_implicit_grid_line_names_: None,
            subgrid_merged_named_areas_: None,
        }
    }
    fn Style(&self) -> &ComputedStyle {
        unsafe { &*self.style_.Get() }
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:32-40
    pub fn for_grid_lanes(style: &ComputedStyle, auto_repetitions: u32) -> Self {
        let mut result = Self::new(style, 1, 1);
        debug_assert!(style.IsDisplayGridLanes());
        if style.GridLanesTrackSizingDirection() == kForColumns {
            result.column_auto_repetitions_ = auto_repetitions;
        } else {
            result.row_auto_repetitions_ = auto_repetitions;
        }
        result
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:42-412
    pub fn for_subgrid(
        style: &ComputedStyle,
        parent: &GridLineResolver,
        area: GridArea,
        columns: u32,
        rows: u32,
        can_inherit_line_names_from_parent: bool,
    ) -> Self {
        let mut result = Self::new(style, columns, rows);
        result.subgridded_columns_merged_explicit_grid_line_names_ = Some(
            style
                .TemplateTracks(kForColumns)
                .GetNamedGridLines()
                .clone(),
        );
        result.subgridded_rows_merged_explicit_grid_line_names_ =
            Some(style.TemplateTracks(kForRows).GetNamedGridLines().clone());
        let has_columns = area.columns.IsTranslatedDefinite();
        let has_rows = area.rows.IsTranslatedDefinite();
        if has_columns {
            result.subgridded_columns_span_size_ = area.SpanSize(kForColumns);
        }
        if has_rows {
            result.subgridded_rows_span_size_ = area.SpanSize(kForRows);
        }
        let opposite = style.Direction() != parent.Style().Direction();
        let parallel =
            IsParallelWritingMode(style.GetWritingMode(), parent.Style().GetWritingMode());
        if has_columns && can_inherit_line_names_from_parent {
            let direction = if parallel { kForColumns } else { kForRows };
            let map = result
                .subgridded_columns_merged_explicit_grid_line_names_
                .as_mut()
                .unwrap();
            MergeNamedGridLinesWithParent(
                map,
                parent.ExplicitNamedLinesMap(direction),
                area.columns,
                opposite,
            );
            ExpandAutoRepeatTracksFromParent(
                map,
                parent.AutoRepeatLineNamesMap(direction),
                parent.ComputedGridTrackList(direction),
                area.columns,
                parent.AutoRepetitions(direction),
                opposite,
                parent.IsSubgridded(direction),
            );
        }
        if has_rows && can_inherit_line_names_from_parent {
            let direction = if parallel { kForRows } else { kForColumns };
            let map = result
                .subgridded_rows_merged_explicit_grid_line_names_
                .as_mut()
                .unwrap();
            MergeNamedGridLinesWithParent(
                map,
                parent.ExplicitNamedLinesMap(direction),
                area.rows,
                opposite,
            );
            ExpandAutoRepeatTracksFromParent(
                map,
                parent.AutoRepeatLineNamesMap(direction),
                parent.ComputedGridTrackList(direction),
                area.rows,
                parent.AutoRepetitions(direction),
                opposite,
                parent.IsSubgridded(direction),
            );
        }
        let areas = style.GridTemplateAreas().Get();
        if !areas.is_null() {
            let mut map = NamedGridAreaMap::default();
            ClampSubgridAreas(&mut map, &unsafe { &*areas }.named_areas, &area);
            result.subgrid_merged_named_areas_ = Some(map);
        }
        if can_inherit_line_names_from_parent {
            if let Some(parent_areas) = parent.NamedAreasMap() {
                let map = result
                    .subgrid_merged_named_areas_
                    .get_or_insert_with(NamedGridAreaMap::default);
                MergeAndClampGridAreasWithParent(map, parent_areas, area, parallel);
            }
        }
        if let Some(map) = &result.subgrid_merged_named_areas_ {
            if has_columns {
                result.subgridded_columns_merged_implicit_grid_line_names_ = Some(
                    ComputedGridTemplateAreas::CreateImplicitNamedGridLinesFromGridArea(
                        map,
                        kForColumns,
                    ),
                );
            }
            if has_rows {
                result.subgridded_rows_merged_implicit_grid_line_names_ = Some(
                    ComputedGridTemplateAreas::CreateImplicitNamedGridLinesFromGridArea(
                        map, kForRows,
                    ),
                );
            }
        }
        result
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:425-449
    fn InitialAndFinalPositionsFromStyle(
        &self,
        item_style: &ComputedStyle,
        direction: GridTrackSizingDirection,
    ) -> (GridPosition, GridPosition) {
        let mut initial = item_style.TrackStart(direction).clone();
        let mut final_position = item_style.TrackEnd(direction).clone();
        if initial.IsSpan() && final_position.IsSpan() {
            final_position.SetAutoPosition();
        }
        if initial.IsAuto() && final_position.IsSpan() && !final_position.NamedGridLine().IsNull() {
            final_position.SetSpanPosition(1, &AtomicString::default());
        }
        if final_position.IsAuto() && initial.IsSpan() && !initial.NamedGridLine().IsNull() {
            initial.SetSpanPosition(1, &AtomicString::default());
        }
        (initial, final_position)
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:451-475
    fn LookAheadForNamedGridLine(
        &self,
        start: i32,
        mut number_of_lines: u32,
        last: u32,
        lines: &GridNamedLineCollection<'_>,
    ) -> u32 {
        debug_assert!(number_of_lines != 0);
        let mut end = start.max(0) as u32;
        if !lines.HasNamedLines() {
            end = end.max(last.wrapping_add(1));
            return end.wrapping_add(number_of_lines).wrapping_sub(1);
        }
        while number_of_lines != 0 {
            if end > last || lines.Contains(end) {
                number_of_lines -= 1;
            }
            end = end.wrapping_add(1);
        }
        debug_assert!(end != 0);
        end.wrapping_sub(1)
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:477-498
    fn LookBackForNamedGridLine(
        &self,
        end: i32,
        mut number_of_lines: u32,
        last: i32,
        lines: &GridNamedLineCollection<'_>,
    ) -> i32 {
        debug_assert!(number_of_lines != 0);
        let mut start = end.min(last);
        if !lines.HasNamedLines() {
            start = start.min(-1);
            return (start as u32).wrapping_sub(number_of_lines).wrapping_add(1) as i32;
        }
        while number_of_lines != 0 {
            if start < 0 || lines.Contains(start as u32) {
                number_of_lines -= 1;
            }
            start = start.wrapping_sub(1);
        }
        start.wrapping_add(1)
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:500-521
    fn DefiniteGridSpanWithNamedSpanAgainstOpposite(
        &self,
        opposite: i32,
        position: &GridPosition,
        side: GridPositionSide,
        last: i32,
        lines: &GridNamedLineCollection<'_>,
    ) -> GridSpan {
        let span = position.SpanPosition() as u32;
        let (start, end) = if matches!(side, kRowStartSide | kColumnStartSide) {
            (
                self.LookBackForNamedGridLine(opposite - 1, span, last, lines),
                opposite,
            )
        } else {
            (
                opposite,
                self.LookAheadForNamedGridLine(opposite + 1, span, last as u32, lines) as i32,
            )
        };
        GridSpan::UntranslatedDefiniteGridSpan(start, end)
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:523-534
    fn IsSubgridded(&self, direction: GridTrackSizingDirection) -> bool {
        if direction == kForColumns {
            self.subgridded_columns_merged_explicit_grid_line_names_
                .is_some()
        } else {
            self.subgridded_rows_merged_explicit_grid_line_names_
                .is_some()
        }
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:536-551
    pub fn ExplicitGridColumnCount(&self) -> u32 {
        if self.subgridded_columns_span_size_ != u32::MAX {
            return self.subgridded_columns_span_size_;
        }
        let mut count = self
            .Style()
            .TemplateTracks(kForColumns)
            .GetTrackList()
            .TrackCountWithoutAutoRepeat()
            .wrapping_add(self.AutoRepeatTrackCount(kForColumns));
        let areas = self.Style().GridTemplateAreas().Get();
        if !areas.is_null() {
            count = count.max(unsafe { &*areas }.column_count);
        }
        count.min(K_GRID_MAX_TRACKS as u32)
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:553-568
    pub fn ExplicitGridRowCount(&self) -> u32 {
        if self.subgridded_rows_span_size_ != u32::MAX {
            return self.subgridded_rows_span_size_;
        }
        let mut count = self
            .Style()
            .TemplateTracks(kForRows)
            .GetTrackList()
            .TrackCountWithoutAutoRepeat()
            .wrapping_add(self.AutoRepeatTrackCount(kForRows));
        let areas = self.Style().GridTemplateAreas().Get();
        if !areas.is_null() {
            count = count.max(unsafe { &*areas }.row_count);
        }
        count.min(K_GRID_MAX_TRACKS as u32)
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:570-574
    pub fn ExplicitGridTrackCount(&self, direction: GridTrackSizingDirection) -> u32 {
        if direction == kForColumns {
            self.ExplicitGridColumnCount()
        } else {
            self.ExplicitGridRowCount()
        }
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:576-580
    pub fn AutoRepetitions(&self, direction: GridTrackSizingDirection) -> u32 {
        if direction == kForColumns {
            self.column_auto_repetitions_
        } else {
            self.row_auto_repetitions_
        }
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:582-589
    pub fn AutoRepeatTrackCount(&self, direction: GridTrackSizingDirection) -> u32 {
        self.AutoRepetitions(direction).wrapping_mul(
            self.ComputedGridTrackList(direction)
                .GetTrackList()
                .AutoRepeatTrackCount(),
        )
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:591-595
    pub fn SubgridSpanSize(&self, direction: GridTrackSizingDirection) -> u32 {
        if direction == kForColumns {
            self.subgridded_columns_span_size_
        } else {
            self.subgridded_rows_span_size_
        }
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:597-603
    pub fn HasStandaloneAxis(&self, direction: GridTrackSizingDirection) -> bool {
        self.SubgridSpanSize(direction) == u32::MAX
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:605-610
    fn ExplicitGridSizeForSide(&self, side: GridPositionSide) -> u32 {
        self.ExplicitGridTrackCount(DirectionFromSide(side))
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:612-637
    fn ResolveNamedGridLinePositionAgainstOppositePosition(
        &self,
        opposite: i32,
        position: &GridPosition,
        side: GridPositionSide,
    ) -> GridSpan {
        debug_assert!(position.IsSpan());
        debug_assert!(!position.NamedGridLine().IsNull());
        debug_assert!(position.SpanPosition() > 0);
        let direction = DirectionFromSide(side);
        let last = self.ExplicitGridSizeForSide(side);
        let lines = self.Lines(&PositionName(position), direction, last);
        self.DefiniteGridSpanWithNamedSpanAgainstOpposite(
            opposite,
            position,
            side,
            last as i32,
            &lines,
        )
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:653-677
    pub fn ImplicitNamedLinesMap(&self, direction: GridTrackSizingDirection) -> &NamedGridLinesMap {
        let merged = if direction == kForColumns {
            &self.subgridded_columns_merged_implicit_grid_line_names_
        } else {
            &self.subgridded_rows_merged_implicit_grid_line_names_
        };
        if let Some(map) = merged {
            return map;
        }
        let areas = self.Style().GridTemplateAreas().Get();
        if !areas.is_null() {
            let areas = unsafe { &*areas };
            return if direction == kForColumns {
                &areas.implicit_named_grid_column_lines
            } else {
                &areas.implicit_named_grid_row_lines
            };
        }
        static EMPTY: std::sync::LazyLock<NamedGridLinesMap> =
            std::sync::LazyLock::new(NamedGridLinesMap::default);
        &EMPTY
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:660-668
    pub fn ExplicitNamedLinesMap(&self, direction: GridTrackSizingDirection) -> &NamedGridLinesMap {
        let merged = if direction == kForColumns {
            &self.subgridded_columns_merged_explicit_grid_line_names_
        } else {
            &self.subgridded_rows_merged_explicit_grid_line_names_
        };
        merged
            .as_ref()
            .unwrap_or_else(|| self.ComputedGridTrackList(direction).GetNamedGridLines())
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:670-678
    pub fn NamedAreasMap(&self) -> Option<&NamedGridAreaMap> {
        if let Some(map) = &self.subgrid_merged_named_areas_ {
            return Some(map);
        }
        let areas = self.Style().GridTemplateAreas().Get();
        if areas.is_null() {
            None
        } else {
            Some(&unsafe { &*areas }.named_areas)
        }
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:680-685
    fn AutoRepeatLineNamesMap(&self, direction: GridTrackSizingDirection) -> &NamedGridLinesMap {
        self.ComputedGridTrackList(direction)
            .GetAutoRepeatNamedGridLines()
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:687-690
    fn ComputedGridTrackList(&self, direction: GridTrackSizingDirection) -> &ComputedGridTrackList {
        self.Style().TemplateTracks(direction)
    }
    fn Lines(
        &self,
        name: &String,
        direction: GridTrackSizingDirection,
        last: u32,
    ) -> GridNamedLineCollection<'_> {
        GridNamedLineCollection::new(
            name,
            direction,
            self.ImplicitNamedLinesMap(direction),
            self.ExplicitNamedLinesMap(direction),
            self.ComputedGridTrackList(direction),
            last,
            self.AutoRepeatTrackCount(direction),
            self.IsSubgridded(direction),
        )
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:692-716
    fn ResolveGridPositionAgainstOppositePosition(
        &self,
        opposite: i32,
        position: &GridPosition,
        side: GridPositionSide,
    ) -> GridSpan {
        if position.IsAuto() {
            return if matches!(side, kColumnStartSide | kRowStartSide) {
                GridSpan::UntranslatedDefiniteGridSpan(opposite - 1, opposite)
            } else {
                GridSpan::UntranslatedDefiniteGridSpan(opposite, opposite + 1)
            };
        }
        debug_assert!(position.IsSpan());
        debug_assert!(position.SpanPosition() > 0);
        if !position.NamedGridLine().IsNull() {
            return self
                .ResolveNamedGridLinePositionAgainstOppositePosition(opposite, position, side);
        }
        DefiniteGridSpanWithSpanAgainstOpposite(opposite, position, side)
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:718-733
    fn SpanSizeFromPositions(&self, initial: &GridPosition, final_position: &GridPosition) -> u32 {
        debug_assert!(
            initial.ShouldBeResolvedAgainstOppositePosition()
                && final_position.ShouldBeResolvedAgainstOppositePosition()
        );
        if initial.IsAuto() && final_position.IsAuto() {
            return 1;
        }
        let span = if initial.IsSpan() {
            initial
        } else {
            final_position
        };
        debug_assert!(span.IsSpan() && span.SpanPosition() != 0);
        span.SpanPosition() as u32
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:735-760
    fn ResolveNamedGridLinePosition(&self, position: &GridPosition, side: GridPositionSide) -> i32 {
        debug_assert!(!position.NamedGridLine().IsNull());
        let last = self.ExplicitGridSizeForSide(side);
        let direction = DirectionFromSide(side);
        let lines = self.Lines(&PositionName(position), direction, last);
        if position.IsPositive() {
            self.LookAheadForNamedGridLine(
                0,
                position.IntegerPosition().unsigned_abs(),
                last,
                &lines,
            ) as i32
        } else {
            self.LookBackForNamedGridLine(
                last as i32,
                position.IntegerPosition().unsigned_abs(),
                last as i32,
                &lines,
            )
        }
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:762-820
    fn ResolveGridPosition(&self, position: &GridPosition, side: GridPositionSide) -> i32 {
        let direction = DirectionFromSide(side);
        let _auto_repeat_tracks_count = self.AutoRepeatTrackCount(direction);
        match position.GetType() {
            GridPositionType::kExplicitPosition => {
                debug_assert!(position.IntegerPosition() != 0);
                if !position.NamedGridLine().IsNull() {
                    return self.ResolveNamedGridLinePosition(position, side);
                }
                if position.IsPositive() {
                    return position.IntegerPosition() - 1;
                }
                let resolved = position.IntegerPosition().unsigned_abs().wrapping_sub(1);
                self.ExplicitGridSizeForSide(side).wrapping_sub(resolved) as i32
            }
            GridPositionType::kNamedGridAreaPosition => {
                let name = PositionName(position);
                debug_assert!(!position.NamedGridLine().IsNull());
                let last = self.ExplicitGridSizeForSide(side);
                let implicit =
                    self.Lines(&ImplicitNamedGridLineForSide(&name, side), direction, last);
                if implicit.HasNamedLines() {
                    return implicit.FirstPosition() as i32;
                }
                let explicit = self.Lines(&name, direction, last);
                if explicit.HasNamedLines() {
                    return explicit.FirstPosition() as i32;
                }
                last.wrapping_add(1) as i32
            }
            GridPositionType::kAutoPosition | GridPositionType::kSpanPosition => {
                unreachable!("auto/span must resolve against opposite position")
            }
        }
    }
    // cpp: layoutng_grid/grid_line_resolver.cc:822-878
    pub fn ResolveGridPositionsFromStyle(
        &self,
        style: &ComputedStyle,
        direction: GridTrackSizingDirection,
    ) -> GridSpan {
        let (initial, final_position) = self.InitialAndFinalPositionsFromStyle(style, direction);
        let initial_opposite = initial.ShouldBeResolvedAgainstOppositePosition();
        let final_opposite = final_position.ShouldBeResolvedAgainstOppositePosition();
        if initial_opposite && final_opposite {
            return GridSpan::IndefiniteGridSpan(
                self.SpanSizeFromPositions(&initial, &final_position),
            );
        }
        let initial_side = if direction == kForColumns {
            kColumnStartSide
        } else {
            kRowStartSide
        };
        let final_side = if direction == kForColumns {
            kColumnEndSide
        } else {
            kRowEndSide
        };
        if initial_opposite {
            let end = self.ResolveGridPosition(&final_position, final_side);
            return self.ResolveGridPositionAgainstOppositePosition(end, &initial, initial_side);
        }
        if final_opposite {
            let start = self.ResolveGridPosition(&initial, initial_side);
            return self.ResolveGridPositionAgainstOppositePosition(
                start,
                &final_position,
                final_side,
            );
        }
        let mut start = self.ResolveGridPosition(&initial, initial_side);
        let mut end = self.ResolveGridPosition(&final_position, final_side);
        if end < start {
            std::mem::swap(&mut end, &mut start);
        } else if end == start {
            end = start + 1;
        }
        GridSpan::UntranslatedDefiniteGridSpan(start, end)
    }
}
// cpp: layoutng_grid/grid_line_resolver.cc:414-423
impl PartialEq for GridLineResolver {
    fn eq(&self, other: &Self) -> bool {
        self.column_auto_repetitions_ == other.column_auto_repetitions_
            && self.row_auto_repetitions_ == other.row_auto_repetitions_
            && self.subgridded_columns_span_size_ == other.subgridded_columns_span_size_
            && self.subgridded_rows_span_size_ == other.subgridded_rows_span_size_
    }
}
// cpp: layoutng_grid/grid_line_resolver.cc:639-651
fn DefiniteGridSpanWithSpanAgainstOpposite(
    opposite: i32,
    position: &GridPosition,
    side: GridPositionSide,
) -> GridSpan {
    let offset = position.SpanPosition() as u32;
    if matches!(side, kColumnStartSide | kRowStartSide) {
        GridSpan::UntranslatedDefiniteGridSpan(
            (opposite as u32).wrapping_sub(offset) as i32,
            opposite,
        )
    } else {
        GridSpan::UntranslatedDefiniteGridSpan(
            opposite,
            (opposite as u32).wrapping_add(offset) as i32,
        )
    }
}
// cpp: layoutng_grid/grid_line_resolver.cc:76-133
fn MergeNamedGridLinesWithParent(
    subgrid: &mut NamedGridLinesMap,
    parent: &NamedGridLinesMap,
    span: GridSpan,
    opposite: bool,
) {
    for (name, positions) in parent.iter() {
        let mut merged = Vec::new();
        for &position in positions {
            if span.Contains(position) {
                merged.push(if opposite {
                    span.EndLine() - position
                } else {
                    position - span.StartLine()
                });
            }
        }
        if let Some(existing) = subgrid.get(name) {
            merged.extend(existing.iter().copied());
            merged.sort_unstable();
            merged.dedup();
        }
        if !merged.is_empty() {
            subgrid.Set(name.clone(), merged);
        }
    }
}
// cpp: layoutng_grid/grid_line_resolver.cc:134-219
fn ExpandAutoRepeatTracksFromParent(
    subgrid: &mut NamedGridLinesMap,
    parent: &NamedGridLinesMap,
    tracks: &ComputedGridTrackList,
    span: GridSpan,
    repetitions: u32,
    opposite: bool,
    nested: bool,
) {
    let count = tracks.GetTrackList().AutoRepeatTrackCount();
    let total = count.wrapping_mul(repetitions);
    if total == 0 {
        return;
    }
    let insertion = tracks.GetAutoRepeatInsertionPoint();
    if !nested {
        // Source Set replaces values while iterating the same keys. Retain key
        // iteration order with a key snapshot, avoiding aliased Rust borrows.
        let names: Vec<_> = subgrid.keys().cloned().collect();
        for name in names {
            let mut shifted = Vec::new();
            for &position in subgrid.get(&name).unwrap() {
                if position >= insertion {
                    let expanded = position.wrapping_add(total);
                    if span.Contains(expanded) {
                        shifted.push(expanded);
                    }
                }
            }
            subgrid.Set(name, shifted);
        }
    }
    for (name, positions) in parent.iter() {
        let mut merged = Vec::new();
        for &position in positions {
            for i in 0..repetitions {
                for j in 0..count {
                    let expanded = insertion
                        .wrapping_add(position)
                        .wrapping_add(i)
                        .wrapping_add(j);
                    if span.Contains(expanded) {
                        merged.push(if opposite {
                            span.EndLine() - expanded
                        } else {
                            expanded - span.StartLine()
                        });
                    }
                }
            }
            if let Some(existing) = subgrid.get(name) {
                merged.extend(existing.iter().copied());
                merged.sort_unstable();
            }
            if !merged.is_empty() {
                subgrid.Set(name.clone(), merged.clone());
            }
        }
    }
}
// cpp: layoutng_grid/grid_line_resolver.cc:224-235
fn ClampSubgridAreas(subgrid: &mut NamedGridAreaMap, style: &NamedGridAreaMap, span: &GridArea) {
    for (name, area) in style.iter() {
        let mut clamped = *area;
        if span.columns.IsTranslatedDefinite() {
            clamped
                .columns
                .Intersect(0, span.columns.IntegerSpan() as i32);
        }
        if span.rows.IsTranslatedDefinite() {
            clamped.rows.Intersect(0, span.rows.IntegerSpan() as i32);
        }
        subgrid.Set(name.clone(), clamped);
    }
}
// cpp: layoutng_grid/grid_line_resolver.cc:246-327
fn MergeAndClampGridAreasWithParent(
    subgrid: &mut NamedGridAreaMap,
    parent: &NamedGridAreaMap,
    span: GridArea,
    parallel: bool,
) {
    let columns = span.columns.IsTranslatedDefinite();
    let rows = span.rows.IsTranslatedDefinite();
    for (name, area) in parent.iter() {
        let mut position = *area;
        debug_assert!(position.columns.IsTranslatedDefinite());
        debug_assert!(position.rows.IsTranslatedDefinite());
        if !parallel {
            position.Transpose();
        }
        let rows_intersect = rows && span.rows.Intersects(position.rows);
        let columns_intersect = columns && span.columns.Intersects(position.columns);
        if !rows_intersect && !columns_intersect {
            continue;
        }
        if rows {
            position
                .rows
                .Intersect(span.rows.StartLine() as i32, span.rows.EndLine() as i32);
        }
        if columns {
            position.columns.Intersect(
                span.columns.StartLine() as i32,
                span.columns.EndLine() as i32,
            );
        }
        if rows {
            position
                .rows
                .Translate(span.rows.StartLine().wrapping_neg());
        }
        if columns {
            position
                .columns
                .Translate(span.columns.StartLine().wrapping_neg());
        }
        if let Some(existing) = subgrid.get(name) {
            if rows {
                position
                    .rows
                    .SetStart(position.rows.StartLine().min(existing.rows.StartLine()) as i32);
                position
                    .rows
                    .SetEnd(position.rows.EndLine().min(existing.rows.EndLine()) as i32);
            }
            if columns {
                position.columns.SetStart(
                    position
                        .columns
                        .StartLine()
                        .min(existing.columns.StartLine()) as i32,
                );
                position
                    .columns
                    .SetEnd(position.columns.EndLine().min(existing.columns.EndLine()) as i32);
            }
        }
        subgrid.Set(
            name.clone(),
            GridArea::new(&position.rows, &position.columns),
        );
    }
}

//! Native Grid style projection.
//!
//! Included by `boundary::native_input` at the reciprocal assembly boundary.
//! Track-list expansion and template-area construction can scale with author
//! input, so their heavy entries retain Grid-qualified sampling frames. Small
//! value conversions remain inlineable and do not emit per-style trace events.

use super::*;

// cpp: layoutng/internal/boundary/native_input.cc:723-748
fn TrackBreadth(breadth: &GridTrackBreadth, allow_flex: bool, field: &str) -> Length {
    match breadth.kind {
        GridTrackBreadthKind::kAuto => Length::Auto().clone(),
        GridTrackBreadthKind::kFixed => Pixels(breadth.pixels, field, true),
        GridTrackBreadthKind::kPercentage => {
            Length::Percent(Number(breadth.percentage, field, true))
        }
        GridTrackBreadthKind::kCalculated => CalculatedLength(
            Some(breadth.pixels),
            Some(breadth.percentage),
            field,
            NativeLengthValueRange::NonNegative,
        ),
        GridTrackBreadthKind::kMinContent => Length::MinContent().clone(),
        GridTrackBreadthKind::kMaxContent => Length::MaxContent().clone(),
        GridTrackBreadthKind::kFlex => {
            if !allow_flex || !breadth.pixels.is_finite() || breadth.pixels <= 0.0 {
                Invalid(field);
            }
            Length::Flex(breadth.pixels as f32)
        }
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:749-763
fn TrackSize(track: &GridTrack) -> GridTrackSize {
    let minimum = TrackBreadth(&track.minimum, false, "grid minimum");
    let maximum = TrackBreadth(&track.maximum, true, "grid maximum");
    if track.fit_content {
        if !minimum.IsAuto() || maximum.IsFlex() || maximum.HasAutoOrContentOrIntrinsic() {
            Invalid("grid fit-content");
        }
        return GridTrackSize::new(&maximum, GridTrackSizeType::kFitContentTrackSizing);
    }
    if minimum == maximum {
        return GridTrackSize::from_length(&minimum);
    }
    GridTrackSize::new_minmax(&minimum, &maximum)
}

// cpp: layoutng/internal/boundary/native_input.cc:764-777
fn RepeatType(kind: GridRepeatKind) -> GridTrackRepeatType {
    match kind {
        GridRepeatKind::kNone => GridTrackRepeatType::kNoRepeat,
        GridRepeatKind::kInteger => GridTrackRepeatType::kInteger,
        GridRepeatKind::kAutoFill => GridTrackRepeatType::kAutoFill,
        GridRepeatKind::kAutoFit => GridTrackRepeatType::kAutoFit,
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:778-786
fn AddNamedLine(map: &mut NamedGridLinesMap, input: &str, position: u32) {
    if input.is_empty() {
        Invalid("grid line name");
    }
    let name = BlinkString::FromUtf8(input.as_bytes());
    map.entry(name).or_default().push(position);
}

// cpp: layoutng/internal/boundary/native_input.cc:787-791
fn SortNamedLines(map: &mut NamedGridLinesMap) {
    for value in map.values_mut() {
        value.sort();
    }
}

// cpp: layoutng/internal/boundary/native_input.cc:792-867
#[inline(never)]
pub(super) fn Tracks(
    tracks: &GridTrackListInput,
    fixed: &[f64],
    subgrid: bool,
) -> *mut ComputedGridTrackList {
    if (!tracks.empty() && !fixed.is_empty()) || (subgrid && (!tracks.empty() || !fixed.is_empty()))
    {
        Invalid("conflicting grid track definitions");
    }
    let result_ptr = MakeGarbageCollected(ComputedGridTrackList::default());
    let result = unsafe { &mut *result_ptr };
    if subgrid {
        result.SetGridAxisType(GridAxisType::kSubgriddedAxis);
        result
            .GetMutableTrackList()
            .SetAxisType(GridAxisType::kSubgriddedAxis);
        return result_ptr;
    }
    for line in &tracks.line_names {
        AddNamedLine(result.GetMutableNamedGridLines(), &line.name, line.position);
    }
    let mut conceptual_line = 0u32;
    for repeater in &tracks.repeaters {
        if repeater.tracks.is_empty()
            || repeater.count == 0
            || (repeater.kind != GridRepeatKind::kInteger && repeater.count != 1)
        {
            Invalid("grid repeater");
        }
        let mut sizes = Vector::<GridTrackSize>::default();
        for track in &repeater.tracks {
            sizes.push(TrackSize(track));
        }
        if !repeater.line_names.is_empty() && repeater.line_names.len() != repeater.tracks.len() + 1
        {
            Invalid("grid repeater line names");
        }
        if !repeater.line_names.is_empty() {
            if repeater.kind == GridRepeatKind::kAutoFill
                || repeater.kind == GridRepeatKind::kAutoFit
            {
                for (line, names) in repeater.line_names.iter().enumerate() {
                    for name in names {
                        AddNamedLine(
                            result.GetMutableAutoRepeatNamedGridLines(),
                            name,
                            line as u32,
                        );
                    }
                }
            } else {
                let repetitions = if repeater.kind == GridRepeatKind::kInteger {
                    repeater.count
                } else {
                    1
                };
                for repetition in 0..repetitions {
                    for (line, names) in repeater.line_names.iter().enumerate() {
                        for name in names {
                            let position = conceptual_line
                                .wrapping_add(repetition.wrapping_mul(repeater.tracks.len() as u32))
                                .wrapping_add(line as u32);
                            AddNamedLine(result.GetMutableNamedGridLines(), name, position);
                        }
                    }
                }
            }
        }
        if !result.GetMutableTrackList().AddRepeaterWithCount(
            &sizes,
            RepeatType(repeater.kind),
            repeater.count,
        ) {
            Invalid("grid track count");
        }
        if repeater.kind == GridRepeatKind::kAutoFill {
            result.SetAutoRepeatType(AutoRepeatType::kAutoFill);
        } else if repeater.kind == GridRepeatKind::kAutoFit {
            result.SetAutoRepeatType(AutoRepeatType::kAutoFit);
        }
        conceptual_line = conceptual_line.wrapping_add(
            if repeater.kind == GridRepeatKind::kAutoFill
                || repeater.kind == GridRepeatKind::kAutoFit
            {
                1
            } else {
                (repeater.tracks.len() as u32).wrapping_mul(repeater.count)
            },
        );
    }
    for line in &tracks.line_names {
        if line.position > conceptual_line {
            Invalid("grid line position");
        }
    }
    SortNamedLines(result.GetMutableNamedGridLines());
    SortNamedLines(result.GetMutableAutoRepeatNamedGridLines());
    if !fixed.is_empty() {
        let mut sizes = Vector::<GridTrackSize>::default();
        for value in fixed {
            sizes.push(GridTrackSize::from_length(&Pixels(
                *value,
                "grid track",
                true,
            )));
        }
        if !result.GetMutableTrackList().AddRepeaterDefault(&sizes) {
            Invalid("grid track count");
        }
    }
    if result.GetTrackList().HasAutoRepeater() {
        result.SetAutoRepeatInsertionPoint(result.GetTrackList().TrackCountBeforeAutoRepeat());
    }
    result_ptr
}

// cpp: layoutng/internal/boundary/native_input.cc:868-881
#[inline(never)]
pub(super) fn AutoTracks(tracks: &GridTrackListInput) -> GridTrackList {
    let mut result = GridTrackList::default();
    if tracks.repeaters.len() != 1
        || tracks.repeaters[0].kind != GridRepeatKind::kNone
        || tracks.repeaters[0].count != 1
    {
        Invalid("grid auto tracks cannot contain repeaters");
    }
    let mut sizes = Vector::<GridTrackSize>::default();
    for track in &tracks.repeaters[0].tracks {
        sizes.push(TrackSize(track));
    }
    if !sizes.is_empty() && !result.AddRepeaterDefault(&sizes) {
        Invalid("grid auto track count");
    }
    result
}

// cpp: layoutng/internal/boundary/native_input.cc:882-908
pub(super) fn GridPosition(input: &GridLine, field: &str) -> NativeGridPosition {
    let mut result = NativeGridPosition::default();
    // Rust's public String input is already valid UTF-8. Preserve the C++
    // empty-name and number guards before creating the atomic name.
    let name = if input.name.is_empty() {
        AtomicString::default()
    } else {
        AtomicString::from_utf16(&input.name.encode_utf16().collect::<Vec<_>>())
    };
    match input.kind {
        GridLineKind::kAuto => {
            if input.number != 0 || !input.name.is_empty() {
                Invalid(field);
            }
        }
        GridLineKind::kExplicit => {
            if input.number == 0 {
                Invalid(field);
            }
            result.SetExplicitPosition(input.number, &name);
        }
        GridLineKind::kSpan => {
            if input.number < 1 {
                Invalid(field);
            }
            result.SetSpanPosition(input.number, &name);
        }
        GridLineKind::kNamedArea => {
            if input.number != 0 || input.name.is_empty() {
                Invalid(field);
            }
            result.SetNamedGridArea(&name);
        }
    }
    result
}

// cpp: layoutng/internal/boundary/native_input.cc:909-936
#[inline(never)]
pub(super) fn GridTemplateAreas(input: &GridTemplateAreasInput) -> *mut ComputedGridTemplateAreas {
    if input.row_count == 0
        || input.column_count == 0
        || input.row_count > i32::MAX as u32
        || input.column_count > i32::MAX as u32
    {
        Invalid("grid template area dimensions");
    }
    let mut areas = NamedGridAreaMap::default();
    for area in &input.areas {
        if area.name.is_empty()
            || area.row_start >= area.row_end
            || area.column_start >= area.column_end
            || area.row_end > input.row_count
            || area.column_end > input.column_count
        {
            Invalid("grid template area");
        }
        let name = BlinkString::FromUtf8(area.name.as_bytes());
        if areas.contains_key(&name) {
            Invalid("duplicate grid template area");
        }
        let rows =
            GridSpan::UntranslatedDefiniteGridSpan(area.row_start as i32, area.row_end as i32);
        let columns = GridSpan::UntranslatedDefiniteGridSpan(
            area.column_start as i32,
            area.column_end as i32,
        );
        areas.insert(name, GridArea::new(&rows, &columns));
    }
    MakeGarbageCollected(ComputedGridTemplateAreas::new(
        &areas,
        input.row_count,
        input.column_count,
    ))
}

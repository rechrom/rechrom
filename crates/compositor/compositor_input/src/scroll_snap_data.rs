// Translation status: complete
// Source: cc/input/scroll_snap_data.h:1-569
// Source: cc/input/scroll_snap_data.cc:1-1220
// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/input/scroll_snap_data.h:1-23,559-569;
//         cc/input/scroll_snap_data.cc:1-24,182-183,1220.
// Rust modules/imports replace guards, includes, namespaces and test-friend
// declarations. Stack-only borrowed search results retain canonical area identity.
use crate::{
    input_types::ElementId,
    snap_geometry::RangeF,
    snap_selection_strategy::{SnapSelectionStrategy, SnapStopAlwaysFilter},
};
use compositor_tree::input_geometry::{PointF, RectF, Vector2dF};
use std::cell::Cell;
fn min(a: f32, b: f32) -> f32 {
    if b < a {
        b
    } else {
        a
    }
}
fn max(a: f32, b: f32) -> f32 {
    if a < b {
        b
    } else {
        a
    }
}
fn clamp(value: f32, lo: f32, hi: f32) -> f32 {
    debug_assert!(!(hi < lo));
    if value < lo {
        lo
    } else if hi < value {
        hi
    } else {
        value
    }
}
// Source: cc/input/scroll_snap_data.h:24-33.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum SnapAxis {
    kBoth,
    kX,
    kY,
    kBlock,
    kInline,
    kPair,
}
// Source: cc/input/scroll_snap_data.h:34-36.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum SearchAxis {
    kX,
    kY,
}
// Source: cc/input/scroll_snap_data.h:37-39.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum SnapStrictness {
    kProximity,
    kMandatory,
}
// Source: cc/input/scroll_snap_data.h:40-42.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum SnapAlignment {
    kNone,
    kStart,
    kEnd,
    kCenter,
}
// Source: cc/input/scroll_snap_data.h:43-66.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollSnapType {
    pub is_none: bool,
    pub axis: SnapAxis,
    pub strictness: SnapStrictness,
}
impl Default for ScrollSnapType {
    fn default() -> Self {
        Self {
            is_none: true,
            axis: SnapAxis::kBoth,
            strictness: SnapStrictness::kProximity,
        }
    }
}
impl ScrollSnapType {
    pub fn new(snap_type_none: bool, axis: SnapAxis, strictness: SnapStrictness) -> Self {
        Self {
            is_none: snap_type_none,
            axis,
            strictness,
        }
    }
}
// Source: cc/input/scroll_snap_data.h:68-90.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScrollSnapAlign {
    pub alignment_block: SnapAlignment,
    pub alignment_inline: SnapAlignment,
}
impl Default for ScrollSnapAlign {
    fn default() -> Self {
        Self {
            alignment_block: SnapAlignment::kNone,
            alignment_inline: SnapAlignment::kNone,
        }
    }
}
impl ScrollSnapAlign {
    pub fn FromAlignment(alignment: SnapAlignment) -> Self {
        Self {
            alignment_block: alignment,
            alignment_inline: alignment,
        }
    }
    pub fn new(b: SnapAlignment, i: SnapAlignment) -> Self {
        Self {
            alignment_block: b,
            alignment_inline: i,
        }
    }
}
// Source: cc/input/scroll_snap_data.h:92-141.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SnapAreaData {
    pub scroll_snap_align: ScrollSnapAlign,
    pub rect: RectF,
    pub must_snap: bool,
    pub has_focus_within: bool,
    pub element_id: ElementId,
}
impl SnapAreaData {
    pub const kInvalidScrollPosition: i32 = -1;
    pub fn new(
        align: &ScrollSnapAlign,
        rec: &RectF,
        msnap: bool,
        has_focus_within: bool,
        id: ElementId,
    ) -> Self {
        Self {
            scroll_snap_align: *align,
            rect: *rec,
            must_snap: msnap,
            has_focus_within,
            element_id: id,
        }
    }
}
// Source: cc/input/scroll_snap_data.h:143-176.
// A both-axis aligned alternative may beat the main result in its cross axis.
#[derive(Clone, Copy, Debug)]
pub struct SnapSearchResultAlternative<'a> {
    pub area: Option<&'a SnapAreaData>,
    pub area_rect: RectF,
    pub cross_axis_snap_offset: f32,
}
impl<'a> SnapSearchResultAlternative<'a> {
    pub fn new(
        area: Option<&'a SnapAreaData>,
        rect: RectF,
        area_cross_axis_snap_offset: f32,
    ) -> Self {
        Self {
            area,
            area_rect: rect,
            cross_axis_snap_offset: area_cross_axis_snap_offset,
        }
    }
}
// Source: cc/input/scroll_snap_data.h:178-189,252-293.
// Option fields represent C++ default-constructor uninitialized scalars; reads
// require their source initialization path rather than silently using a zero.
#[derive(Clone, Copy, Debug, Default)]
pub struct SnapSearchResult<'a> {
    snap_offset_: Option<f32>,
    area_: Option<&'a SnapAreaData>,
    covered_range_: Option<RangeF>,
    axis_: Option<SearchAxis>,
    snapport_visible_range_: RangeF,
    snapport_max_visible_: Option<f32>,
    rect_: Option<RectF>,
    alternative_: Option<SnapSearchResultAlternative<'a>>,
}
impl<'a> SnapSearchResult<'a> {
    // Source: cc/input/scroll_snap_data.h:186-189; cc/input/scroll_snap_data.cc:184-192.
    pub fn new(
        offset: f32,
        axis: SearchAxis,
        snapport_visible_range: RangeF,
        snapport_max_visible: f32,
    ) -> Self {
        Self {
            snap_offset_: Some(offset),
            axis_: Some(axis),
            snapport_visible_range_: snapport_visible_range,
            snapport_max_visible_: Some(snapport_max_visible),
            ..Self::default()
        }
    }
    // Source: cc/input/scroll_snap_data.h:190-191; cc/input/scroll_snap_data.cc:193-196.
    pub fn Clip(&mut self, max_snap: f32) {
        self.snap_offset_ = Some(clamp(self.snap_offset(), 0.0, max_snap));
    }
    // Source: cc/input/scroll_snap_data.h:193-197; cc/input/scroll_snap_data.cc:197-204.
    pub fn Union(&mut self, other: &Self) {
        debug_assert!(self.snap_offset() == other.snap_offset());
        debug_assert!(self.rect_.is_some() && other.rect().is_some());
        if let (Some(rect), Some(other_rect)) = (&mut self.rect_, other.rect()) {
            rect.Union(other_rect);
        }
    }
    // Source: cc/input/scroll_snap_data.h:199-200.
    pub fn snap_offset(&self) -> f32 {
        self.snap_offset_.expect("initialized snap offset")
    }
    pub fn set_snap_offset(&mut self, offset: f32) {
        self.snap_offset_ = Some(offset);
    }
    // Source: cc/input/scroll_snap_data.h:202-217.
    pub fn visible_range(&self) -> RangeF {
        let max_visible = self.snapport_max_visible_.expect("initialized max visible");
        let Some(rect) = self.rect_ else {
            return RangeF::new(0.0, max_visible);
        };
        let rect_start = if self.axis() == SearchAxis::kX {
            rect.y()
        } else {
            rect.x()
        };
        let rect_end = if self.axis() == SearchAxis::kX {
            rect.bottom()
        } else {
            rect.right()
        };
        RangeF::new(
            clamp(
                rect_start - self.snapport_visible_range_.end(),
                0.0,
                max_visible,
            ),
            clamp(
                rect_end - self.snapport_visible_range_.start(),
                0.0,
                max_visible,
            ),
        )
    }
    // Source: cc/input/scroll_snap_data.h:219-220.
    pub fn area(&self) -> Option<&'a SnapAreaData> {
        self.area_
    }
    pub fn set_area(&mut self, area: Option<&'a SnapAreaData>) {
        self.area_ = area;
    }
    // Source: cc/input/scroll_snap_data.h:222-224.
    pub fn element_id(&self) -> ElementId {
        self.area_
            .map_or(ElementId::default(), |area| area.element_id)
    }
    // Source: cc/input/scroll_snap_data.h:226-228.
    pub fn covered_range(&self) -> Option<RangeF> {
        self.covered_range_
    }
    pub fn set_covered_range(&mut self, range: &RangeF) {
        self.covered_range_ = Some(*range);
    }
    pub fn has_focus_within(&self) -> bool {
        self.area_.is_some_and(|area| area.has_focus_within)
    }
    // Source: cc/input/scroll_snap_data.h:230-237.
    pub fn axis(&self) -> SearchAxis {
        self.axis_.expect("initialized search axis")
    }
    pub fn set_axis(&mut self, axis: &SearchAxis) {
        self.axis_ = Some(*axis);
    }
    pub fn set_snapport_visible_range(&mut self, range: &RangeF) {
        self.snapport_visible_range_ = *range;
    }
    pub fn set_snapport_max_visible(&mut self, position: f32) {
        self.snapport_max_visible_ = Some(position);
    }
    // Source: cc/input/scroll_snap_data.h:239-240.
    pub fn rect(&self) -> Option<RectF> {
        self.rect_
    }
    pub fn set_rect(&mut self, rect: &RectF) {
        self.rect_ = Some(*rect);
    }
    // Source: cc/input/scroll_snap_data.h:242-250.
    pub fn alternative(&self) -> Option<SnapSearchResultAlternative<'a>> {
        self.alternative_
    }
    pub fn set_alternative(
        &mut self,
        area: Option<&'a SnapAreaData>,
        rect: &RectF,
        alt_cross_snap_offset: f32,
    ) {
        self.alternative_ = Some(SnapSearchResultAlternative::new(
            area,
            *rect,
            alt_cross_snap_offset,
        ));
    }
}
// Source: cc/input/scroll_snap_data.h:295-311.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TargetSnapAreaElementIds {
    pub x: ElementId,
    pub y: ElementId,
}
impl TargetSnapAreaElementIds {
    pub fn new(x_id: ElementId, y_id: ElementId) -> Self {
        Self { x: x_id, y: y_id }
    }
    // Source: cc/input/scroll_snap_data.cc:1215-1219.
    pub fn ToString(&self) -> String {
        format!(
            "TargetSnapAreaElementIds{{x={}, y={}}}",
            self.x.ToString(),
            self.y.ToString()
        )
    }
}
// Source: cc/input/scroll_snap_data.h:313-314.
pub type SnapAreaList = Vec<SnapAreaData>;
// Source: cc/input/scroll_snap_data.h:315-334.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SnapPositionType {
    #[default]
    kNone,
    kAligned,
    kCovered,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct SnapPositionData {
    pub r#type: SnapPositionType,
    pub position: PointF,
    pub target_element_ids: TargetSnapAreaElementIds,
    pub covered_range_x: Option<RangeF>,
    pub covered_range_y: Option<RangeF>,
    pub is_extremity: bool,
}

// Source: cc/input/scroll_snap_data.cc:25-46.
fn DistanceFromCorridor(dx: f64, dy: f64, area: &RectF) -> Vector2dF {
    let mut distance = Vector2dF::default();
    if dx < 0.0 {
        distance.set_x(-dx as f32);
    } else if dx > area.width() as f64 {
        distance.set_x((dx - area.width() as f64) as f32);
    } else {
        distance.set_x(0.0);
    }
    if dy < 0.0 {
        distance.set_y(-dy as f32);
    } else if dy > area.height() as f64 {
        distance.set_y((dy - area.height() as f64) as f32);
    } else {
        distance.set_y(0.0);
    }
    distance
}
// Source: cc/input/scroll_snap_data.cc:47-51.
fn IsMutualVisible(a: &SnapSearchResult<'_>, b: &SnapSearchResult<'_>) -> bool {
    RangeF::At(b.snap_offset()).IsBoundedBy(a.visible_range())
        && RangeF::At(a.snap_offset()).IsBoundedBy(b.visible_range())
}
// Source: cc/input/scroll_snap_data.cc:52-63.
fn SetOrUpdateResult<'a>(
    candidate: &SnapSearchResult<'a>,
    result: &mut Option<SnapSearchResult<'a>>,
) {
    if let Some(value) = result {
        value.Union(candidate);
        if candidate.has_focus_within() {
            value.set_area(candidate.area());
        }
    } else {
        *result = Some(*candidate);
    }
}
// Source: cc/input/scroll_snap_data.cc:64-87.
fn ClosestSearchResult<'r, 'a>(
    reference_point: PointF,
    axis: SearchAxis,
    a: &'r Option<SnapSearchResult<'a>>,
    b: &'r Option<SnapSearchResult<'a>>,
) -> &'r Option<SnapSearchResult<'a>> {
    if a.is_none() {
        return b;
    }
    if b.is_none() {
        return a;
    }
    let reference_position = if axis == SearchAxis::kX {
        reference_point.x()
    } else {
        reference_point.y()
    };
    let position_a = a.as_ref().unwrap().snap_offset();
    let position_b = b.as_ref().unwrap().snap_offset();
    debug_assert!(
        (reference_position <= position_a && reference_position <= position_b)
            || (reference_position >= position_a && reference_position >= position_b)
    );
    let distance_a = (position_a - reference_position).abs();
    let distance_b = (position_b - reference_position).abs();
    if distance_a < distance_b {
        a
    } else {
        b
    }
}
// Source: cc/input/scroll_snap_data.cc:88-142.
fn SearchResultForDodgingRange<'a>(
    area_range: &RangeF,
    dodging_range: &RangeF,
    aligned_candidate: &SnapSearchResult<'a>,
    preferred_offset: f32,
    scroll_padding: f32,
    snapport_size: f32,
    alignment: SnapAlignment,
) -> Option<SnapSearchResult<'a>> {
    if dodging_range.is_empty() || dodging_range.is_reversed() {
        return None;
    }
    let mut result = *aligned_candidate;
    let mut min_offset = dodging_range.start() - scroll_padding;
    let mut max_offset = dodging_range.end() - scroll_padding - snapport_size;
    if max_offset > min_offset {
        result.set_snap_offset(clamp(preferred_offset, min_offset, max_offset));
        result.set_covered_range(&RangeF::new(min_offset, max_offset));
        return Some(result);
    }
    // Even a too-small dodging range must provide a reachable aligned position.
    let offset = match alignment {
        SnapAlignment::kStart => min_offset,
        SnapAlignment::kCenter => (min_offset + max_offset) / 2.0,
        SnapAlignment::kEnd => max_offset,
        _ => unreachable!("NOTREACHED: snap alignment"),
    };
    min_offset = area_range.start() - scroll_padding;
    max_offset = area_range.end() - scroll_padding - snapport_size;
    if max_offset < min_offset {
        return None;
    }
    result.set_snap_offset(clamp(offset, min_offset, max_offset));
    Some(result)
}
// Source: cc/input/scroll_snap_data.cc:143-151.
fn CanCoverSnapportOnAxis(axis: SearchAxis, container_rect: &RectF, area_rect: &RectF) -> bool {
    (axis == SearchAxis::kY && area_rect.height() >= container_rect.height())
        || (axis == SearchAxis::kX && area_rect.width() >= container_rect.width())
}
// Source: cc/input/scroll_snap_data.cc:152-170.
fn CalculateCoveredRange(axis: SearchAxis, snapport: &RectF, area_rect: &RectF) -> Option<RangeF> {
    if axis == SearchAxis::kX {
        if area_rect.width() > snapport.width() {
            return Some(RangeF::new(
                area_rect.x() - snapport.x(),
                area_rect.right() - snapport.right(),
            ));
        }
    } else if area_rect.height() > snapport.height() {
        return Some(RangeF::new(
            area_rect.y() - snapport.y(),
            area_rect.bottom() - snapport.bottom(),
        ));
    }
    None
}
// Source: cc/input/scroll_snap_data.cc:171-181.
fn UpdateMinMax(offset: f32, min_offset: &mut Option<f32>, max_offset: &mut Option<f32>) {
    if min_offset.is_none_or(|min| offset < min) {
        *min_offset = Some(offset);
    }
    if max_offset.is_none_or(|max| offset > max) {
        *max_offset = Some(offset);
    }
}
// Source: cc/input/scroll_snap_data.h:336-358,511-558.
#[derive(Clone, Debug)]
pub struct SnapContainerData {
    scroll_snap_type_: ScrollSnapType,
    rect_: RectF,
    max_position_: PointF,
    proximity_range_: PointF,
    snap_area_list_: Vec<SnapAreaData>,
    target_snap_area_element_ids_: TargetSnapAreaElementIds,
    snapport_height_adjustment_: Cell<f64>,
    has_horizontal_writing_mode_: bool,
    targeted_area_id_: Option<ElementId>,
    min_snap_offset_x_: Option<f32>,
    max_snap_offset_x_: Option<f32>,
    min_snap_offset_y_: Option<f32>,
    max_snap_offset_y_: Option<f32>,
}
// Source: cc/input/scroll_snap_data.h:359-372.
impl PartialEq for SnapContainerData {
    fn eq(&self, other: &Self) -> bool {
        other.scroll_snap_type_ == self.scroll_snap_type_
            && other.rect_ == self.rect_
            && other.max_position_ == self.max_position_
            && other.proximity_range_ == self.proximity_range_
            && other.snap_area_list_ == self.snap_area_list_
            && other.target_snap_area_element_ids_ == self.target_snap_area_element_ids_
            && other.targeted_area_id_ == self.targeted_area_id_
            && other.has_horizontal_writing_mode_ == self.has_horizontal_writing_mode_
    }
}
// Source: cc/input/scroll_snap_data.h:347; cc/input/scroll_snap_data.cc:205-208.
impl Default for SnapContainerData {
    fn default() -> Self {
        Self {
            scroll_snap_type_: ScrollSnapType::default(),
            rect_: RectF::default(),
            max_position_: PointF::default(),
            proximity_range_: PointF::new(f32::MAX, f32::MAX),
            snap_area_list_: Vec::new(),
            target_snap_area_element_ids_: TargetSnapAreaElementIds::default(),
            snapport_height_adjustment_: Cell::new(0.0),
            has_horizontal_writing_mode_: true,
            targeted_area_id_: None,
            min_snap_offset_x_: None,
            max_snap_offset_x_: None,
            min_snap_offset_y_: None,
            max_snap_offset_y_: None,
        }
    }
}
impl SnapContainerData {
    // Source: cc/input/scroll_snap_data.h:348; cc/input/scroll_snap_data.cc:209-213.
    pub fn FromType(r#type: ScrollSnapType) -> Self {
        Self {
            scroll_snap_type_: r#type,
            ..Self::default()
        }
    }
    // Source: cc/input/scroll_snap_data.h:349-351; cc/input/scroll_snap_data.cc:214-222.
    pub fn new(r#type: ScrollSnapType, rect: &RectF, max: &PointF) -> Self {
        Self {
            scroll_snap_type_: r#type,
            rect_: *rect,
            max_position_: *max,
            ..Self::default()
        }
    }
    // Source: cc/input/scroll_snap_data.h:352-358; cc/input/scroll_snap_data.cc:223-234.
    // Clone/clone_from, moves and automatic field drop implement C++ defaults.

    // Source: cc/input/scroll_snap_data.h:385; cc/input/scroll_snap_data.cc:235-278.
    pub fn AddSnapAreaData(&mut self, snap_area_data: SnapAreaData) {
        self.snap_area_list_.push(snap_area_data);
        let rect = self.snapport();
        if snap_area_data.scroll_snap_align.alignment_inline != SnapAlignment::kNone {
            let result = self.GetSnapSearchResult(SearchAxis::kX, &snap_area_data);
            let covered_range = CalculateCoveredRange(SearchAxis::kX, &rect, &snap_area_data.rect);
            if let Some(range) = covered_range {
                UpdateMinMax(
                    range.start(),
                    &mut self.min_snap_offset_x_,
                    &mut self.max_snap_offset_x_,
                );
                UpdateMinMax(
                    range.end(),
                    &mut self.min_snap_offset_x_,
                    &mut self.max_snap_offset_x_,
                );
            } else {
                UpdateMinMax(
                    result.snap_offset(),
                    &mut self.min_snap_offset_x_,
                    &mut self.max_snap_offset_x_,
                );
            }
        }
        if snap_area_data.scroll_snap_align.alignment_block != SnapAlignment::kNone {
            let result = self.GetSnapSearchResult(SearchAxis::kY, &snap_area_data);
            let covered_range = CalculateCoveredRange(SearchAxis::kY, &rect, &snap_area_data.rect);
            if let Some(range) = covered_range {
                UpdateMinMax(
                    range.start(),
                    &mut self.min_snap_offset_y_,
                    &mut self.max_snap_offset_y_,
                );
                UpdateMinMax(
                    range.end(),
                    &mut self.min_snap_offset_y_,
                    &mut self.max_snap_offset_y_,
                );
            } else {
                UpdateMinMax(
                    result.snap_offset(),
                    &mut self.min_snap_offset_y_,
                    &mut self.max_snap_offset_y_,
                );
            }
        }
    }
    // Source: cc/input/scroll_snap_data.h:396; cc/input/scroll_snap_data.cc:279-283.
    pub fn set_rect(&mut self, rect: &RectF) {
        self.rect_ = *rect;
        self.UpdateExtremes();
    }
    // Source: cc/input/scroll_snap_data.h:418; cc/input/scroll_snap_data.cc:284-322.
    fn UpdateExtremes(&mut self) {
        self.min_snap_offset_x_ = None;
        self.max_snap_offset_x_ = None;
        self.min_snap_offset_y_ = None;
        self.max_snap_offset_y_ = None;
        let rect = self.snapport();
        for area in &self.snap_area_list_ {
            if area.scroll_snap_align.alignment_inline != SnapAlignment::kNone {
                let sr = self.GetSnapSearchResult(SearchAxis::kX, area);
                let covered_range = CalculateCoveredRange(SearchAxis::kX, &rect, &area.rect);
                if let Some(range) = covered_range {
                    UpdateMinMax(
                        range.start(),
                        &mut self.min_snap_offset_x_,
                        &mut self.max_snap_offset_x_,
                    );
                    UpdateMinMax(
                        range.end(),
                        &mut self.min_snap_offset_x_,
                        &mut self.max_snap_offset_x_,
                    );
                } else {
                    UpdateMinMax(
                        sr.snap_offset(),
                        &mut self.min_snap_offset_x_,
                        &mut self.max_snap_offset_x_,
                    );
                }
            }
            if area.scroll_snap_align.alignment_block != SnapAlignment::kNone {
                let sr = self.GetSnapSearchResult(SearchAxis::kY, area);
                let covered_range = CalculateCoveredRange(SearchAxis::kY, &rect, &area.rect);
                if let Some(range) = covered_range {
                    UpdateMinMax(
                        range.start(),
                        &mut self.min_snap_offset_y_,
                        &mut self.max_snap_offset_y_,
                    );
                    UpdateMinMax(
                        range.end(),
                        &mut self.min_snap_offset_y_,
                        &mut self.max_snap_offset_y_,
                    );
                } else {
                    UpdateMinMax(
                        sr.snap_offset(),
                        &mut self.min_snap_offset_y_,
                        &mut self.max_snap_offset_y_,
                    );
                }
            }
        }
    }
    // Source: cc/input/scroll_snap_data.h:407; cc/input/scroll_snap_data.cc:323-334.
    pub fn GetSnapRange(&self, axis: SearchAxis) -> RangeF {
        if axis == SearchAxis::kX {
            RangeF::new(
                self.min_snap_offset_x_.unwrap_or(0.0),
                self.max_snap_offset_x_.unwrap_or(self.max_position_.x()),
            )
        } else {
            RangeF::new(
                self.min_snap_offset_y_.unwrap_or(0.0),
                self.max_snap_offset_y_.unwrap_or(self.max_position_.y()),
            )
        }
    }

    // Source: cc/input/scroll_snap_data.h:374-376; cc/input/scroll_snap_data.cc:335-342.
    pub fn FindSnapPositionWithViewportAdjustment(
        &mut self,
        strategy: &dyn SnapSelectionStrategy,
        snapport_height_adjustment: f64,
    ) -> SnapPositionData {
        // base::AutoReset expressed as a scoped Cell restore guard. The guard
        // also restores on unwind, while search can safely borrow self.
        struct Reset<'a> {
            target: &'a Cell<f64>,
            old: f64,
        }
        impl Drop for Reset<'_> {
            fn drop(&mut self) {
                self.target.set(self.old);
            }
        }
        let resetter = Reset {
            target: &self.snapport_height_adjustment_,
            old: self
                .snapport_height_adjustment_
                .replace(snapport_height_adjustment),
        };
        let result = self.FindSnapPosition(strategy);
        drop(resetter);
        result
    }
    // Source: cc/input/scroll_snap_data.h:378-379; cc/input/scroll_snap_data.cc:343-520.
    pub fn FindSnapPosition(&self, strategy: &dyn SnapSelectionStrategy) -> SnapPositionData {
        let mut result = SnapPositionData::default();
        result.target_element_ids = TargetSnapAreaElementIds::default();
        if self.scroll_snap_type_.is_none {
            return result;
        }
        let base_position = strategy.base_position();
        let axis = self.scroll_snap_type_.axis;
        let should_snap_on_x =
            strategy.ShouldSnapOnX() && (axis == SnapAxis::kX || axis == SnapAxis::kBoth);
        let should_snap_on_y =
            strategy.ShouldSnapOnY() && (axis == SnapAxis::kY || axis == SnapAxis::kBoth);
        if !should_snap_on_x && !should_snap_on_y {
            if axis == SnapAxis::kY {
                result.target_element_ids.y = self.target_snap_area_element_ids_.y;
            } else {
                result.target_element_ids.x = self.target_snap_area_element_ids_.x;
            }
            return result;
        }
        let should_prioritize_x_target = strategy.ShouldPrioritizeSnapTargets()
            && self.target_snap_area_element_ids_.x != ElementId::default();
        let should_prioritize_y_target = strategy.ShouldPrioritizeSnapTargets()
            && self.target_snap_area_element_ids_.y != ElementId::default();
        let mut selected_x = None;
        let mut selected_y = None;
        // Source: cc/input/scroll_snap_data.cc:376-406.
        if should_snap_on_x {
            let initial_snap_position_y = SnapSearchResult::new(
                clamp(base_position.y(), 0.0, self.max_position_.y()),
                SearchAxis::kY,
                RangeF::new(self.rect_.x(), self.rect_.right()),
                self.max_position_.x(),
            );
            if should_prioritize_x_target {
                selected_x = self.GetTargetSnapAreaSearchResult(
                    strategy,
                    SearchAxis::kX,
                    initial_snap_position_y,
                );
            }
            if selected_x.is_none() {
                selected_x =
                    self.FindClosestValidArea(SearchAxis::kX, strategy, &initial_snap_position_y);
            }
        }
        if should_snap_on_y {
            let initial_snap_position_x = SnapSearchResult::new(
                clamp(base_position.x(), 0.0, self.max_position_.x()),
                SearchAxis::kX,
                RangeF::new(self.rect_.y(), self.rect_.bottom()),
                self.max_position_.y(),
            );
            if should_prioritize_y_target {
                selected_y = self.GetTargetSnapAreaSearchResult(
                    strategy,
                    SearchAxis::kY,
                    initial_snap_position_x,
                );
            }
            if selected_y.is_none() {
                selected_y =
                    self.FindClosestValidArea(SearchAxis::kY, strategy, &initial_snap_position_x);
            }
        }
        // Source: cc/input/scroll_snap_data.cc:408-418.
        if selected_x.is_none() && selected_y.is_none() {
            if should_snap_on_x
                && should_snap_on_y
                && !strategy.ShouldRespectSnapStop()
                && self.FindSnapPositionForMutualSnap(strategy, &mut result)
            {
                result.r#type = SnapPositionType::kAligned;
            }
            return result;
        }
        // Source: cc/input/scroll_snap_data.cc:420-442.
        if should_snap_on_x
            && should_snap_on_y
            && (selected_x.is_none()
                || selected_y.is_none()
                || !IsMutualVisible(selected_x.as_ref().unwrap(), selected_y.as_ref().unwrap()))
        {
            let mut axis_to_follow = SnapAxis::kX;
            if let (Some(x), Some(y)) = (&selected_x, &selected_y) {
                axis_to_follow = self.SelectAxisToFollowForMutualVisibility(strategy, x, y);
            } else if selected_y.is_some() {
                axis_to_follow = SnapAxis::kY;
            }
            if axis_to_follow == SnapAxis::kX {
                selected_y = self.FindClosestValidArea(
                    SearchAxis::kY,
                    strategy,
                    selected_x.as_ref().unwrap(),
                );
            } else {
                selected_x = self.FindClosestValidArea(
                    SearchAxis::kX,
                    strategy,
                    selected_y.as_ref().unwrap(),
                );
            }
        }
        // Source: cc/input/scroll_snap_data.cc:444-455.
        if let Some(y) = selected_y.as_mut() {
            if y.alternative().is_some() {
                self.SelectAlternativeIdForSearchResult(
                    y,
                    &selected_x,
                    strategy.current_position().x(),
                    self.max_position_.x(),
                );
            }
        }
        if let Some(x) = selected_x.as_mut() {
            if x.alternative().is_some() {
                self.SelectAlternativeIdForSearchResult(
                    x,
                    &selected_y,
                    strategy.current_position().y(),
                    self.max_position_.y(),
                );
            }
        }
        // Source: cc/input/scroll_snap_data.cc:457-475.
        result.r#type = SnapPositionType::kAligned;
        result.position = *strategy.current_position();
        for area in &self.snap_area_list_ {
            if area.element_id == self.target_snap_area_element_ids_.x {
                result.target_element_ids.x = self.target_snap_area_element_ids_.x;
            }
            if area.element_id == self.target_snap_area_element_ids_.y {
                result.target_element_ids.y = self.target_snap_area_element_ids_.y;
            }
        }
        let direction = if strategy.HasIntendedDirection() {
            strategy.intended_position() - *strategy.current_position()
        } else {
            Vector2dF::default()
        };
        // Source: cc/input/scroll_snap_data.cc:477-495.
        if let Some(x) = selected_x {
            result.position.set_x(x.snap_offset());
            result.target_element_ids.x = x.element_id();
            result.covered_range_x = x.covered_range();
            if !x.area().unwrap().must_snap
                && ((direction.x() > 0.0
                    && self.max_snap_offset_x_.is_some_and(|max| {
                        x.snap_offset() == max
                            || x.covered_range().is_some_and(|range| range.end() == max)
                    }))
                    || (direction.x() < 0.0
                        && self.min_snap_offset_x_.is_some_and(|min| {
                            x.snap_offset() == min
                                || x.covered_range().is_some_and(|range| range.start() == min)
                        })))
            {
                result.is_extremity = true;
            }
        }
        // Source: cc/input/scroll_snap_data.cc:496-514.
        if let Some(y) = selected_y {
            result.position.set_y(y.snap_offset());
            result.target_element_ids.y = y.element_id();
            result.covered_range_y = y.covered_range();
            if !y.area().unwrap().must_snap
                && ((direction.y() > 0.0
                    && self.max_snap_offset_y_.is_some_and(|max| {
                        y.snap_offset() == max
                            || y.covered_range().is_some_and(|range| range.end() == max)
                    }))
                    || (direction.y() < 0.0
                        && self.min_snap_offset_y_.is_some_and(|min| {
                            y.snap_offset() == min
                                || y.covered_range().is_some_and(|range| range.start() == min)
                        })))
            {
                result.is_extremity = true;
            }
        }
        // Source: cc/input/scroll_snap_data.cc:515-520.
        if (selected_x.is_none() || result.covered_range_x.is_some())
            && (selected_y.is_none() || result.covered_range_y.is_some())
        {
            result.r#type = SnapPositionType::kCovered;
        }
        result
    }
    // Source: cc/input/scroll_snap_data.h:450-451; cc/input/scroll_snap_data.cc:522-578.
    fn FindSnapPositionForMutualSnap(
        &self,
        strategy: &dyn SnapSelectionStrategy,
        result: &mut SnapPositionData,
    ) -> bool {
        debug_assert!(strategy.ShouldSnapOnX() && strategy.ShouldSnapOnY());
        let mut found = false;
        let mut smallest_distance = Vector2dF::new(f32::MAX, f32::MAX);
        for area in &self.snap_area_list_ {
            if !strategy.IsValidSnapArea(SearchAxis::kX, area) {
                continue;
            }
            if !strategy.IsValidSnapArea(SearchAxis::kY, area) {
                continue;
            }
            let x_candidate = self.GetSnapSearchResult(SearchAxis::kX, area);
            let dx = x_candidate.snap_offset() - strategy.current_position().x();
            if dx.abs() > self.proximity_range_.x() {
                continue;
            }
            let y_candidate = self.GetSnapSearchResult(SearchAxis::kY, area);
            let dy = y_candidate.snap_offset() - strategy.current_position().y();
            if dy.abs() > self.proximity_range_.y() {
                continue;
            }
            let distance = DistanceFromCorridor(dx as f64, dy as f64, &self.snapport());
            if distance.y() < smallest_distance.y()
                || (distance.y() == smallest_distance.y() && distance.x() < smallest_distance.x())
            {
                smallest_distance = distance;
                result.position.set_x(x_candidate.snap_offset());
                result.position.set_y(y_candidate.snap_offset());
                result.target_element_ids.x = x_candidate.element_id();
                result.target_element_ids.y = y_candidate.element_id();
                found = true;
            }
        }
        found
    }
    // Source: cc/input/scroll_snap_data.h:453-458; cc/input/scroll_snap_data.cc:579-611.
    fn GetTargetSnapAreaSearchResult<'a>(
        &'a self,
        strategy: &dyn SnapSelectionStrategy,
        axis: SearchAxis,
        cross_axis_snap_result: SnapSearchResult<'_>,
    ) -> Option<SnapSearchResult<'a>> {
        let target_id = if axis == SearchAxis::kX {
            self.target_snap_area_element_ids_.x
        } else {
            self.target_snap_area_element_ids_.y
        };
        if target_id == ElementId::default() {
            return None;
        }
        for area in &self.snap_area_list_ {
            if area.element_id == target_id && strategy.IsValidSnapArea(axis, area) {
                let aligned_result = self.GetSnapSearchResult(axis, area);
                if CanCoverSnapportOnAxis(axis, &self.snapport(), &area.rect) {
                    let area_range = if axis == SearchAxis::kX {
                        RangeF::new(area.rect.x(), area.rect.right())
                    } else {
                        RangeF::new(area.rect.y(), area.rect.bottom())
                    };
                    let covering_result = self.FindClosestValidAreaInternal(
                        axis,
                        strategy,
                        &cross_axis_snap_result,
                        true,
                        Some(area_range),
                    );
                    return Some(covering_result.unwrap_or(aligned_result));
                }
                return Some(aligned_result);
            }
        }
        None
    }
    // Source: cc/input/scroll_snap_data.h:473-474; cc/input/scroll_snap_data.cc:612-620.
    fn UpdateSnapAreaForTesting(&mut self, element_id: ElementId, snap_area_data: SnapAreaData) {
        for area in &mut self.snap_area_list_ {
            if area.element_id == element_id {
                *area = snap_area_data;
            }
        }
    }
    // Source: cc/input/scroll_snap_data.h:381; cc/input/scroll_snap_data.cc:621-625.
    pub fn GetTargetSnapAreaElementIds(&self) -> &TargetSnapAreaElementIds {
        &self.target_snap_area_element_ids_
    }
    // Source: cc/input/scroll_snap_data.h:382-383; cc/input/scroll_snap_data.cc:626-634.
    pub fn SetTargetSnapAreaElementIds(&mut self, ids: TargetSnapAreaElementIds) -> bool {
        if self.target_snap_area_element_ids_ == ids {
            return false;
        }
        self.target_snap_area_element_ids_ = ids;
        true
    }
    // Source: cc/input/scroll_snap_data.h:441-448; cc/input/scroll_snap_data.cc:635-679.
    fn FindClosestValidArea<'a>(
        &'a self,
        axis: SearchAxis,
        strategy: &dyn SnapSelectionStrategy,
        cross_axis_snap_result: &SnapSearchResult<'_>,
    ) -> Option<SnapSearchResult<'a>> {
        let mut result =
            self.FindClosestValidAreaInternal(axis, strategy, cross_axis_snap_result, true, None);
        if result.is_some() && strategy.ShouldRespectSnapStop() {
            let must_only_strategy = <dyn SnapSelectionStrategy>::CreateForDirection(
                *strategy.current_position(),
                strategy.intended_position() - *strategy.current_position(),
                strategy.UsingFractionalOffsets(),
                SnapStopAlwaysFilter::kRequire,
            );
            let must_only_result = self.FindClosestValidAreaInternal(
                axis,
                must_only_strategy.as_ref(),
                cross_axis_snap_result,
                false,
                None,
            );
            result = *ClosestSearchResult(
                *strategy.current_position(),
                axis,
                &result,
                &must_only_result,
            );
        }
        if result.is_some()
            || self.scroll_snap_type_.strictness == SnapStrictness::kProximity
            || !strategy.HasIntendedDirection()
        {
            return result;
        }
        let relaxed_strategy = <dyn SnapSelectionStrategy>::CreateForEndPosition(
            strategy.current_position(),
            strategy.ShouldSnapOnX(),
            strategy.ShouldSnapOnY(),
        );
        self.FindClosestValidAreaInternal(
            axis,
            relaxed_strategy.as_ref(),
            cross_axis_snap_result,
            true,
            None,
        )
    }

    // Source: cc/input/scroll_snap_data.h:420-439; cc/input/scroll_snap_data.cc:680-828.
    fn FindClosestValidAreaInternal<'a>(
        &'a self,
        axis: SearchAxis,
        strategy: &dyn SnapSelectionStrategy,
        cross_axis_snap_result: &SnapSearchResult<'_>,
        should_consider_covering: bool,
        active_element_range: Option<RangeF>,
    ) -> Option<SnapSearchResult<'a>> {
        let horiz = axis == SearchAxis::kX;
        debug_assert!(
            cross_axis_snap_result.snap_offset() >= 0.0
                && cross_axis_snap_result.snap_offset()
                    <= if horiz {
                        self.max_position_.y()
                    } else {
                        self.max_position_.x()
                    }
        );
        let mut closest: Option<SnapSearchResult<'a>> = None;
        let mut covering_intended = None;
        let intended_position = if horiz {
            strategy.intended_position().x()
        } else {
            strategy.intended_position().y()
        };
        let base_position = if horiz {
            strategy.base_position().x()
        } else {
            strategy.base_position().y()
        };
        let mut preferred_candidate = false;
        let mut smallest_distance = if horiz {
            self.proximity_range_.x()
        } else {
            self.proximity_range_.y()
        };
        let proximity_distance = if horiz {
            self.proximity_range_.x()
        } else {
            self.proximity_range_.y()
        };
        // Source: cc/input/scroll_snap_data.cc:714-782.
        let mut evaluate = |candidate: &SnapSearchResult<'a>, area: &'a SnapAreaData| {
            if !IsMutualVisible(candidate, cross_axis_snap_result) {
                return;
            }
            if !strategy.IsValidSnapPosition(axis, candidate.snap_offset()) {
                return;
            }
            let distance = (candidate.snap_offset() - base_position).abs();
            if distance > proximity_distance {
                return;
            }
            let is_preferred_candidate =
                strategy.IsPreferredSnapPosition(axis, candidate.snap_offset());
            if preferred_candidate && !is_preferred_candidate {
                return;
            }
            if distance > smallest_distance && (preferred_candidate || !is_preferred_candidate) {
                return;
            }
            if distance < smallest_distance
                || (is_preferred_candidate
                    && (!preferred_candidate || candidate.has_focus_within()))
            {
                smallest_distance = distance;
                closest = Some(*candidate);
                preferred_candidate = is_preferred_candidate;
            } else if let Some(current_closest) =
                closest.as_mut().filter(|result| !result.has_focus_within())
            {
                if Some(current_closest.element_id()) == self.targeted_area_id_ {
                    return;
                }
                if Some(candidate.element_id()) == self.targeted_area_id_ {
                    *current_closest = *candidate;
                    preferred_candidate = is_preferred_candidate;
                    return;
                }
                let candidate_rect = candidate.rect();
                let closest_rect = current_closest.rect();
                debug_assert!(closest_rect.is_some() && candidate_rect.is_some());
                if closest_rect
                    .zip(candidate_rect)
                    .is_some_and(|(c, r)| c.Contains(r))
                    && closest_rect != candidate_rect
                {
                    smallest_distance = distance;
                    *current_closest = *candidate;
                    preferred_candidate = is_preferred_candidate;
                } else if self.scroll_snap_type_.axis == SnapAxis::kBoth
                    && area.scroll_snap_align.alignment_block != SnapAlignment::kNone
                    && area.scroll_snap_align.alignment_inline != SnapAlignment::kNone
                    && is_preferred_candidate == preferred_candidate
                {
                    self.UpdateSearchAlternative(current_closest, candidate, area, strategy);
                }
            }
        };
        // Source: cc/input/scroll_snap_data.cc:784-822.
        for area in &self.snap_area_list_ {
            if !strategy.IsValidSnapArea(axis, area) {
                continue;
            }
            if let Some(active_range) = active_element_range {
                let area_range = if horiz {
                    RangeF::new(area.rect.x(), area.rect.right())
                } else {
                    RangeF::new(area.rect.y(), area.rect.bottom())
                };
                if !active_range.Intersects(area_range) {
                    continue;
                }
            }
            let candidate = self.GetSnapSearchResult(axis, area);
            evaluate(&candidate, area);
            if should_consider_covering
                && CanCoverSnapportOnAxis(axis, &self.snapport(), &area.rect)
            {
                if let Some(mut covering) =
                    self.FindCoveringCandidate(area, axis, &candidate, intended_position)
                {
                    covering.set_rect(&area.rect);
                    if covering.snap_offset() == intended_position {
                        SetOrUpdateResult(&covering, &mut covering_intended);
                    } else {
                        evaluate(&covering, area);
                    }
                }
            }
            // Continue through all areas even if one covers the intended
            // position: neighboring aligned targets can still reject it.
        }
        // Source: cc/input/scroll_snap_data.cc:824-827.
        *strategy.PickBestResult(&closest, &covering_intended)
    }
    // Source: cc/input/scroll_snap_data.h:460-467; cc/input/scroll_snap_data.cc:829-879.
    fn GetSnapSearchResult<'a>(
        &self,
        axis: SearchAxis,
        area: &'a SnapAreaData,
    ) -> SnapSearchResult<'a> {
        let mut result = SnapSearchResult::default();
        let rect = self.snapport();
        if axis == SearchAxis::kX {
            let offset = match area.scroll_snap_align.alignment_inline {
                SnapAlignment::kStart => area.rect.x() - rect.x(),
                SnapAlignment::kCenter => area.rect.CenterPoint().x() - rect.CenterPoint().x(),
                SnapAlignment::kEnd => area.rect.right() - rect.right(),
                _ => unreachable!("NOTREACHED: inline snap alignment"),
            };
            result.set_snap_offset(offset);
            result.Clip(self.max_position_.x());
            result.set_snapport_max_visible(self.max_position_.y());
            result.set_snapport_visible_range(&RangeF::new(rect.y(), rect.bottom()));
        } else {
            let offset = match area.scroll_snap_align.alignment_block {
                SnapAlignment::kStart => area.rect.y() - rect.y(),
                SnapAlignment::kCenter => area.rect.CenterPoint().y() - rect.CenterPoint().y(),
                SnapAlignment::kEnd => area.rect.bottom() - rect.bottom(),
                _ => unreachable!("NOTREACHED: block snap alignment"),
            };
            result.set_snap_offset(offset);
            result.Clip(self.max_position_.y());
            result.set_snapport_max_visible(self.max_position_.x());
            result.set_snapport_visible_range(&RangeF::new(rect.x(), rect.right()));
        }
        result.set_axis(&axis);
        result.set_rect(&area.rect);
        result.set_area(Some(area));
        result
    }
    // Source: cc/input/scroll_snap_data.h:476-480; cc/input/scroll_snap_data.cc:880-995.
    fn FindCoveringCandidate<'a>(
        &self,
        area: &'a SnapAreaData,
        axis: SearchAxis,
        aligned_candidate: &SnapSearchResult<'a>,
        intended_position: f32,
    ) -> Option<SnapSearchResult<'a>> {
        let horiz = axis == SearchAxis::kX;
        let rect = self.snapport();
        let scroll_padding = if horiz { rect.x() } else { rect.y() };
        let snapport_size = if horiz { rect.width() } else { rect.height() };
        let alignment = if horiz {
            area.scroll_snap_align.alignment_inline
        } else {
            area.scroll_snap_align.alignment_block
        };
        let area_range = if horiz {
            RangeF::new(area.rect.x(), area.rect.right())
        } else {
            RangeF::new(area.rect.y(), area.rect.bottom())
        };
        let preferred_snapport = RangeF::new(
            intended_position + scroll_padding,
            intended_position + scroll_padding + snapport_size,
        );
        let mut backward_dodging_range = area_range;
        let mut middle_dodging_range = area_range;
        let mut forward_dodging_range = area_range;
        // Source: cc/input/scroll_snap_data.cc:902-960.
        for intruder in &self.snap_area_list_ {
            let intruder_range = if horiz {
                RangeF::new(intruder.rect.x(), intruder.rect.right())
            } else {
                RangeF::new(intruder.rect.y(), intruder.rect.bottom())
            };
            if intruder_range.start() > area_range.end()
                || intruder_range.end() < area_range.start()
            {
                continue;
            }
            if intruder_range.start() <= area_range.start()
                && intruder_range.end() >= area_range.end()
            {
                continue;
            }
            // Three dodge ranges, only retaining potentially better positions
            // than the intruders' own aligned snap positions.
            if intruder_range.end() < preferred_snapport.start() {
                backward_dodging_range
                    .set_start(max(backward_dodging_range.start(), intruder_range.end()));
            } else {
                backward_dodging_range
                    .set_end(min(backward_dodging_range.end(), intruder_range.start()));
            }
            if intruder_range.start() > preferred_snapport.end() {
                forward_dodging_range
                    .set_end(min(forward_dodging_range.end(), intruder_range.start()));
            } else {
                forward_dodging_range
                    .set_start(max(forward_dodging_range.start(), intruder_range.end()));
            }
            if intruder_range.Contains(preferred_snapport)
                || preferred_snapport.Contains(intruder_range)
            {
                middle_dodging_range = RangeF::default();
            } else if intruder_range.start() <= preferred_snapport.start() {
                middle_dodging_range
                    .set_start(max(middle_dodging_range.start(), intruder_range.end()));
            } else {
                debug_assert!(intruder_range.end() >= preferred_snapport.end());
                middle_dodging_range
                    .set_end(min(middle_dodging_range.end(), intruder_range.start()));
            }
        }
        // Source: cc/input/scroll_snap_data.cc:962-994.
        let middle_candidate = SearchResultForDodgingRange(
            &area_range,
            &middle_dodging_range,
            aligned_candidate,
            intended_position,
            scroll_padding,
            snapport_size,
            alignment,
        );
        if middle_candidate.is_some() {
            return middle_candidate;
        }
        let backward_candidate = SearchResultForDodgingRange(
            &area_range,
            &backward_dodging_range,
            aligned_candidate,
            intended_position,
            scroll_padding,
            snapport_size,
            alignment,
        );
        let forward_candidate = SearchResultForDodgingRange(
            &area_range,
            &forward_dodging_range,
            aligned_candidate,
            intended_position,
            scroll_padding,
            snapport_size,
            alignment,
        );
        if backward_candidate.is_none() {
            return forward_candidate;
        }
        if forward_candidate.is_none() {
            return backward_candidate;
        }
        let backward_distance =
            (backward_candidate.as_ref().unwrap().snap_offset() - intended_position).abs();
        let forward_distance =
            (forward_candidate.as_ref().unwrap().snap_offset() - intended_position).abs();
        if backward_distance < forward_distance {
            backward_candidate
        } else {
            forward_candidate
        }
    }

    // Source: cc/input/scroll_snap_data.h:469-471; cc/input/scroll_snap_data.cc:996-1026.
    fn IsSnapportCoveredOnAxis(
        &self,
        axis: SearchAxis,
        current_offset: f32,
        area_rect: &RectF,
    ) -> bool {
        const kSnapportCoveredTolerance: f32 = 0.5;
        let rect = self.snapport();
        if axis == SearchAxis::kX {
            if area_rect.width() < rect.width() {
                return false;
            }
            let left = area_rect.x() - rect.x();
            let right = area_rect.right() - rect.right();
            current_offset >= left - kSnapportCoveredTolerance
                && current_offset <= right + kSnapportCoveredTolerance
        } else {
            if area_rect.height() < rect.height() {
                return false;
            }
            let top = area_rect.y() - rect.y();
            let bottom = area_rect.bottom() - rect.bottom();
            current_offset >= top - kSnapportCoveredTolerance
                && current_offset <= bottom + kSnapportCoveredTolerance
        }
    }
    // Source: cc/input/scroll_snap_data.h:482-483; cc/input/scroll_snap_data.cc:1027-1071.
    fn IsSnappedToArea(&self, area: &SnapAreaData, scroll_offset: &PointF) -> bool {
        let covered_on_y =
            self.IsSnapportCoveredOnAxis(SearchAxis::kY, scroll_offset.y(), &area.rect);
        let covered_on_x =
            self.IsSnapportCoveredOnAxis(SearchAxis::kX, scroll_offset.x(), &area.rect);
        let snaps_on_x = self.scroll_snap_type_.axis == SnapAxis::kX
            || self.scroll_snap_type_.axis == SnapAxis::kBoth;
        let snaps_on_y = self.scroll_snap_type_.axis == SnapAxis::kY
            || self.scroll_snap_type_.axis == SnapAxis::kBoth;
        if (snaps_on_x && covered_on_x) && (snaps_on_y && covered_on_y) {
            return true;
        }
        if snaps_on_y && area.scroll_snap_align.alignment_block != SnapAlignment::kNone {
            let snap_result_y = self.GetSnapSearchResult(SearchAxis::kY, area);
            if ((snap_result_y.snap_offset() - scroll_offset.y()).abs() <= kSnappedToTolerance
                || covered_on_y)
                && RangeF::At(scroll_offset.x()).IsBoundedBy(snap_result_y.visible_range())
            {
                return true;
            }
        }
        if snaps_on_x && area.scroll_snap_align.alignment_inline != SnapAlignment::kNone {
            let snap_result_x = self.GetSnapSearchResult(SearchAxis::kX, area);
            if ((snap_result_x.snap_offset() - scroll_offset.x()).abs() <= kSnappedToTolerance
                || covered_on_x)
                && RangeF::At(scroll_offset.y()).IsBoundedBy(snap_result_x.visible_range())
            {
                return true;
            }
        }
        false
    }
    // Source: cc/input/scroll_snap_data.h:485; cc/input/scroll_snap_data.cc:1072-1083.
    fn snapport(&self) -> RectF {
        if self.snapport_height_adjustment_.get() == 0.0 {
            return self.rect_;
        }
        let mut adjusted = self.rect_;
        adjusted
            .set_height((adjusted.height() as f64 + self.snapport_height_adjustment_.get()) as f32);
        adjusted
    }
    // Source: cc/input/scroll_snap_data.h:487-495; cc/input/scroll_snap_data.cc:1084-1129.
    fn UpdateSearchAlternative<'a>(
        &self,
        current_result: &mut SnapSearchResult<'a>,
        candidate_result: &SnapSearchResult<'a>,
        candidate_area: &'a SnapAreaData,
        strategy: &dyn SnapSelectionStrategy,
    ) {
        let horiz = current_result.axis() == SearchAxis::kX;
        let candidate_cross_axis_aligned_result = self.GetSnapSearchResult(
            if horiz {
                SearchAxis::kY
            } else {
                SearchAxis::kX
            },
            candidate_area,
        );
        let candidate_rect = candidate_result.rect();
        let current_result_rect = current_result.rect();
        debug_assert!(candidate_rect.is_some() && current_result_rect.is_some());
        if candidate_rect.is_none()
            || current_result_rect.is_none()
            || candidate_rect
                .unwrap()
                .Contains(current_result_rect.unwrap())
        {
            return;
        }
        if let Some(alt) = current_result.alternative() {
            let cross_axis_base_position = if horiz {
                strategy.base_position().y()
            } else {
                strategy.base_position().x()
            };
            let candidate_cross_axis_distance = (cross_axis_base_position
                - candidate_cross_axis_aligned_result.snap_offset())
            .abs();
            let alt_cross_axis_distance =
                (cross_axis_base_position - alt.cross_axis_snap_offset).abs();
            if candidate_cross_axis_distance > alt_cross_axis_distance {
                return;
            }
            let alt_rect = alt.area_rect;
            if candidate_cross_axis_distance < alt_cross_axis_distance
                || (alt_rect != candidate_rect.unwrap()
                    && alt_rect.Contains(candidate_rect.unwrap()))
            {
                current_result.set_alternative(
                    Some(candidate_area),
                    &candidate_rect.unwrap(),
                    candidate_cross_axis_aligned_result.snap_offset(),
                );
            }
        } else {
            current_result.set_alternative(
                Some(candidate_area),
                &candidate_rect.unwrap(),
                candidate_cross_axis_aligned_result.snap_offset(),
            );
        }
    }
    // Source: cc/input/scroll_snap_data.h:497-504; cc/input/scroll_snap_data.cc:1130-1152.
    fn SelectAlternativeIdForSearchResult<'a>(
        &self,
        selection: &mut SnapSearchResult<'a>,
        cross_selection: &Option<SnapSearchResult<'_>>,
        cross_current_position: f32,
        cross_max_position: f32,
    ) {
        let within_snapped_tolerance = |v1: f32, v2: f32| (v1 - v2).abs() <= kSnappedToTolerance;
        if let Some(cross) = cross_selection {
            if within_snapped_tolerance(
                cross.snap_offset(),
                selection.alternative().unwrap().cross_axis_snap_offset,
            ) {
                selection.set_area(selection.alternative().unwrap().area);
            }
        } else if within_snapped_tolerance(
            clamp(cross_current_position, 0.0, cross_max_position),
            selection.alternative().unwrap().cross_axis_snap_offset,
        ) {
            selection.set_area(selection.alternative().unwrap().area);
        }
    }
    // Source: cc/input/scroll_snap_data.h:506-509; cc/input/scroll_snap_data.cc:1153-1200.
    fn SelectAxisToFollowForMutualVisibility(
        &self,
        strategy: &dyn SnapSelectionStrategy,
        selected_x: &SnapSearchResult<'_>,
        selected_y: &SnapSearchResult<'_>,
    ) -> SnapAxis {
        if strategy.ShouldPrioritizeSnapTargets() {
            if self.target_snap_area_element_ids_.x == ElementId::default() {
                return SnapAxis::kY;
            } else if self.target_snap_area_element_ids_.y == ElementId::default() {
                return SnapAxis::kX;
            }
            if selected_x.has_focus_within() {
                return SnapAxis::kX;
            } else if selected_y.has_focus_within() {
                return SnapAxis::kY;
            } else if Some(selected_x.element_id()) == self.targeted_area_id_ {
                return SnapAxis::kX;
            } else if Some(selected_y.element_id()) == self.targeted_area_id_ {
                return SnapAxis::kY;
            }
            return if self.has_horizontal_writing_mode_ {
                SnapAxis::kY
            } else {
                SnapAxis::kX
            };
        }
        if (selected_x.snap_offset() - strategy.base_position().x()).abs()
            <= (selected_y.snap_offset() - strategy.base_position().y()).abs()
        {
            SnapAxis::kX
        } else {
            SnapAxis::kY
        }
    }
    // Source: cc/input/scroll_snap_data.h:386-389.
    pub fn UpdateSnapAreaFocus(&mut self, index: usize, has_focus_within: bool) {
        debug_assert!(index < self.snap_area_list_.len());
        self.snap_area_list_[index].has_focus_within = has_focus_within;
    }
    // Source: cc/input/scroll_snap_data.h:390.
    pub fn size(&self) -> usize {
        self.snap_area_list_.len()
    }
    // Source: cc/input/scroll_snap_data.h:391.
    pub fn at(&self, index: usize) -> &SnapAreaData {
        &self.snap_area_list_[index]
    }
    // Source: cc/input/scroll_snap_data.h:393-394.
    pub fn set_scroll_snap_type(&mut self, r#type: ScrollSnapType) {
        self.scroll_snap_type_ = r#type;
    }
    pub fn scroll_snap_type(&self) -> ScrollSnapType {
        self.scroll_snap_type_
    }
    // Source: cc/input/scroll_snap_data.h:397.
    pub fn rect(&self) -> RectF {
        self.rect_
    }
    // Source: cc/input/scroll_snap_data.h:399-400.
    pub fn set_max_position(&mut self, position: PointF) {
        self.max_position_ = position;
    }
    pub fn max_position(&self) -> PointF {
        self.max_position_
    }
    // Source: cc/input/scroll_snap_data.h:402-405.
    pub fn set_proximity_range(&mut self, range: &PointF) {
        self.proximity_range_ = *range;
    }
    pub fn proximity_range(&self) -> PointF {
        self.proximity_range_
    }
    // Source: cc/input/scroll_snap_data.h:409-411.
    pub fn set_targeted_area_id(&mut self, id: &Option<ElementId>) {
        self.targeted_area_id_ = *id;
    }
    // Source: cc/input/scroll_snap_data.h:413-415.
    pub fn set_has_horizontal_writing_mode(&mut self, value: bool) {
        self.has_horizontal_writing_mode_ = value;
    }
}
// Source: cc/input/scroll_snap_data.cc:1027-1030.
const kSnappedToTolerance: f32 = 1.0;
// Source: cc/input/scroll_snap_data.h:564; cc/input/scroll_snap_data.cc:1201-1204.
impl std::fmt::Display for SnapAreaData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.rect.ToString())
    }
}
// Source: cc/input/scroll_snap_data.h:565; cc/input/scroll_snap_data.cc:1205-1214.
impl std::fmt::Display for SnapContainerData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "container_rect: {}", self.rect().ToString())?;
        write!(f, "area_rects: ")?;
        for i in 0..self.size() {
            writeln!(f, "{}", self.at(i))?;
        }
        Ok(())
    }
}

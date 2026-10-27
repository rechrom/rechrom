// Translation status: complete
// Source: cc/input/snap_selection_strategy.h:1-253
// Source: cc/input/snap_selection_strategy.cc:1-262
// Copyright 2018 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.
// Source: cc/input/snap_selection_strategy.h:1-13,251-253;
//         cc/input/snap_selection_strategy.cc:1-14,262.
// Rust trait dispatch replaces C++ virtual inheritance; each concrete strategy
// retains its base class's immutable current position. Default destruction,
// cloning and Box ownership replace C++ default members and unique_ptr.
use crate::scroll_snap_data::{SearchAxis, SnapAlignment, SnapAreaData, SnapSearchResult};
use crate::scroll_utils::ScrollUtils;
use compositor_tree::input_geometry::{PointF, Size, Vector2dF};
// Source: cc/input/snap_selection_strategy.h:14.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapStopAlwaysFilter {
    kIgnore,
    kRequire,
}
// Source: cc/input/snap_selection_strategy.h:15-16.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapTargetsPrioritization {
    kIgnore,
    kRequire,
}
// Source: cc/input/snap_selection_strategy.h:17-24,71-122.
// Full strategy interface for end-position, direction and displacement scrolls.
pub trait SnapSelectionStrategy {
    // Source: cc/input/snap_selection_strategy.h:71-73.
    fn ShouldSnapOnX(&self) -> bool;
    fn ShouldSnapOnY(&self) -> bool;
    // Source: cc/input/snap_selection_strategy.h:75-77;
    //         cc/input/snap_selection_strategy.cc:101-104.
    fn ShouldPrioritizeSnapTargets(&self) -> bool {
        false
    }
    // Source: cc/input/snap_selection_strategy.h:79-80.
    fn intended_position(&self) -> PointF;
    // Source: cc/input/snap_selection_strategy.h:81-83.
    fn base_position(&self) -> PointF;
    // Source: cc/input/snap_selection_strategy.h:84-85,118-121.
    fn current_position(&self) -> &PointF;
    // Source: cc/input/snap_selection_strategy.h:87-89.
    fn IsPreferredSnapPosition(&self, axis: SearchAxis, position: f32) -> bool;
    // Source: cc/input/snap_selection_strategy.h:91-93.
    fn IsValidSnapPosition(&self, axis: SearchAxis, position: f32) -> bool;
    // Source: cc/input/snap_selection_strategy.h:94;
    //         cc/input/snap_selection_strategy.cc:94-100.
    fn IsValidSnapArea(&self, axis: SearchAxis, area: &SnapAreaData) -> bool {
        IsValidSnapAreaBase(axis, area)
    }
    // Source: cc/input/snap_selection_strategy.h:96;
    //         cc/input/snap_selection_strategy.cc:86-89.
    fn HasIntendedDirection(&self) -> bool {
        true
    }
    // Source: cc/input/snap_selection_strategy.h:98-100;
    //         cc/input/snap_selection_strategy.cc:90-93.
    fn ShouldRespectSnapStop(&self) -> bool {
        false
    }
    // Source: cc/input/snap_selection_strategy.h:102-110.
    fn PickBestResult<'r, 'a>(
        &self,
        closest: &'r Option<SnapSearchResult<'a>>,
        covering: &'r Option<SnapSearchResult<'a>>,
    ) -> &'r Option<SnapSearchResult<'a>>;
    // Source: cc/input/snap_selection_strategy.h:112-114;
    //         cc/input/snap_selection_strategy.cc:105-108.
    fn UsingFractionalOffsets(&self) -> bool {
        false
    }
    // Source: cc/input/snap_selection_strategy.h:116-117.
    fn Clone(&self) -> Box<dyn SnapSelectionStrategy>;
}
// Source: cc/input/snap_selection_strategy.cc:94-100.
// Shared base implementation remains callable by the overriding direction class.
fn IsValidSnapAreaBase(axis: SearchAxis, area: &SnapAreaData) -> bool {
    if axis == SearchAxis::kX {
        area.scroll_snap_align.alignment_inline != SnapAlignment::kNone
    } else {
        area.scroll_snap_align.alignment_block != SnapAlignment::kNone
    }
}
impl dyn SnapSelectionStrategy {
    // Source: cc/input/snap_selection_strategy.h:25-31;
    //         cc/input/snap_selection_strategy.cc:15-22.
    pub fn CreateForEndPosition(
        current_position: &PointF,
        scrolled_x: bool,
        scrolled_y: bool,
    ) -> Box<dyn SnapSelectionStrategy> {
        Box::new(EndPositionStrategy::new(
            current_position,
            scrolled_x,
            scrolled_y,
            SnapTargetsPrioritization::kIgnore,
        ))
    }
    // Source: cc/input/snap_selection_strategy.h:32-41;
    //         cc/input/snap_selection_strategy.cc:23-32.
    pub fn CreateForDirection(
        current_position: PointF,
        step: Vector2dF,
        use_fractional_offsets: bool,
        filter: SnapStopAlwaysFilter,
    ) -> Box<dyn SnapSelectionStrategy> {
        Box::new(DirectionStrategy::new(
            &current_position,
            &step,
            StepPreference::kDirection,
            Vector2dF::default(),
            Vector2dF::default(),
            filter,
            use_fractional_offsets,
        ))
    }
    // Source: cc/input/snap_selection_strategy.h:42-52;
    //         cc/input/snap_selection_strategy.cc:33-45.
    pub fn CreateForDisplacement(
        current_position: PointF,
        displacement: Vector2dF,
        use_fractional_offsets: bool,
        filter: SnapStopAlwaysFilter,
    ) -> Box<dyn SnapSelectionStrategy> {
        Box::new(DirectionStrategy::new(
            &current_position,
            &displacement,
            StepPreference::kDistance,
            Vector2dF::default(),
            Vector2dF::new(f32::MAX, f32::MAX),
            filter,
            use_fractional_offsets,
        ))
    }
    // Source: cc/input/snap_selection_strategy.h:53-64;
    //         cc/input/snap_selection_strategy.cc:46-78.
    pub fn CreateForPageScroll(
        current_position: PointF,
        direction: Vector2dF,
        page_size: Size,
        use_fractional_offsets: bool,
        filter: SnapStopAlwaysFilter,
    ) -> Box<dyn SnapSelectionStrategy> {
        let displacement = Vector2dF::new(
            direction.x() * ScrollUtils::CalculatePageStep(page_size.width()) as f32,
            direction.y() * ScrollUtils::CalculatePageStep(page_size.height()) as f32,
        );
        let min_displacement = Vector2dF::new(
            direction.x() * ScrollUtils::CalculateMinPageSnap(page_size.width()) as f32,
            direction.y() * ScrollUtils::CalculateMinPageSnap(page_size.height()) as f32,
        );
        let mut max_displacement = Vector2dF::new(
            direction.x() * ScrollUtils::CalculateMaxPageSnap(page_size.width()) as f32,
            direction.y() * ScrollUtils::CalculateMaxPageSnap(page_size.height()) as f32,
        );
        if direction.x() == 0.0 {
            max_displacement.set_x(f32::MAX);
        }
        if direction.y() == 0.0 {
            max_displacement.set_y(f32::MAX);
        }
        Box::new(DirectionStrategy::new(
            &current_position,
            &displacement,
            StepPreference::kDistance,
            min_displacement,
            max_displacement,
            filter,
            use_fractional_offsets,
        ))
    }
    // Source: cc/input/snap_selection_strategy.h:65-70;
    //         cc/input/snap_selection_strategy.cc:79-85.
    pub fn CreateForTargetElement(current_position: PointF) -> Box<dyn SnapSelectionStrategy> {
        Box::new(EndPositionStrategy::new(
            &current_position,
            true,
            true,
            SnapTargetsPrioritization::kRequire,
        ))
    }
}
// Source: cc/input/snap_selection_strategy.h:124-169.
#[derive(Clone)]
pub struct EndPositionStrategy {
    current_position_: PointF,
    scrolled_x_: bool,
    scrolled_y_: bool,
    snap_targets_prioritization_: SnapTargetsPrioritization,
}
impl EndPositionStrategy {
    // Source: cc/input/snap_selection_strategy.h:136-146.
    pub fn new(
        current_position: &PointF,
        scrolled_x: bool,
        scrolled_y: bool,
        snap_targets_prioritization: SnapTargetsPrioritization,
    ) -> Self {
        Self {
            current_position_: *current_position,
            scrolled_x_: scrolled_x,
            scrolled_y_: scrolled_y,
            snap_targets_prioritization_: snap_targets_prioritization,
        }
    }
}
impl SnapSelectionStrategy for EndPositionStrategy {
    // Source: cc/input/snap_selection_strategy.h:148; cc/input/snap_selection_strategy.cc:109-112.
    fn ShouldSnapOnX(&self) -> bool {
        self.scrolled_x_
    }
    // Source: cc/input/snap_selection_strategy.h:149; cc/input/snap_selection_strategy.cc:113-116.
    fn ShouldSnapOnY(&self) -> bool {
        self.scrolled_y_
    }
    // Source: cc/input/snap_selection_strategy.h:151; cc/input/snap_selection_strategy.cc:117-120.
    fn intended_position(&self) -> PointF {
        self.current_position_
    }
    // Source: cc/input/snap_selection_strategy.h:152; cc/input/snap_selection_strategy.cc:121-124.
    fn base_position(&self) -> PointF {
        self.current_position_
    }
    // Source: cc/input/snap_selection_strategy.h:85,119-121.
    fn current_position(&self) -> &PointF {
        &self.current_position_
    }
    // Source: cc/input/snap_selection_strategy.h:154; cc/input/snap_selection_strategy.cc:125-130.
    fn IsPreferredSnapPosition(&self, _axis: SearchAxis, _position: f32) -> bool {
        true
    }
    // Source: cc/input/snap_selection_strategy.h:155; cc/input/snap_selection_strategy.cc:131-136.
    fn IsValidSnapPosition(&self, axis: SearchAxis, _position: f32) -> bool {
        (self.scrolled_x_ && axis == SearchAxis::kX) || (self.scrolled_y_ && axis == SearchAxis::kY)
    }
    // Source: cc/input/snap_selection_strategy.h:156; cc/input/snap_selection_strategy.cc:137-140.
    fn HasIntendedDirection(&self) -> bool {
        false
    }
    // Source: cc/input/snap_selection_strategy.h:157; cc/input/snap_selection_strategy.cc:141-144.
    fn ShouldPrioritizeSnapTargets(&self) -> bool {
        self.snap_targets_prioritization_ == SnapTargetsPrioritization::kRequire
    }
    // Source: cc/input/snap_selection_strategy.h:159-161; cc/input/snap_selection_strategy.cc:145-150.
    fn PickBestResult<'r, 'a>(
        &self,
        closest: &'r Option<SnapSearchResult<'a>>,
        covering: &'r Option<SnapSearchResult<'a>>,
    ) -> &'r Option<SnapSearchResult<'a>> {
        if covering.is_some() {
            covering
        } else {
            closest
        }
    }
    // Source: cc/input/snap_selection_strategy.h:162; cc/input/snap_selection_strategy.cc:151-154.
    fn Clone(&self) -> Box<dyn SnapSelectionStrategy> {
        Box::new(self.clone())
    }
}
// Source: cc/input/snap_selection_strategy.h:188-193.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepPreference {
    kDirection,
    kDistance,
}
// Source: cc/input/snap_selection_strategy.h:171-187,211-212,235-249.
#[derive(Clone)]
pub struct DirectionStrategy {
    current_position_: PointF,
    step_: Vector2dF,
    preferred_step_: StepPreference,
    preferred_min_displacement_: Vector2dF,
    preferred_max_displacement_: Vector2dF,
    snap_stop_always_filter_: SnapStopAlwaysFilter,
    use_fractional_offsets_: bool,
}
impl DirectionStrategy {
    // Source: cc/input/snap_selection_strategy.h:195-210.
    pub fn new(
        current_position: &PointF,
        step: &Vector2dF,
        preferred_step: StepPreference,
        preferred_min_displacement: Vector2dF,
        preferred_max_displacement: Vector2dF,
        filter: SnapStopAlwaysFilter,
        use_fractional_offsets: bool,
    ) -> Self {
        Self {
            current_position_: *current_position,
            step_: *step,
            preferred_step_: preferred_step,
            preferred_min_displacement_: preferred_min_displacement,
            preferred_max_displacement_: preferred_max_displacement,
            snap_stop_always_filter_: filter,
            use_fractional_offsets_: use_fractional_offsets,
        }
    }
}
impl SnapSelectionStrategy for DirectionStrategy {
    // Source: cc/input/snap_selection_strategy.h:214; cc/input/snap_selection_strategy.cc:155-158.
    fn ShouldSnapOnX(&self) -> bool {
        self.step_.x() != 0.0
    }
    // Source: cc/input/snap_selection_strategy.h:215; cc/input/snap_selection_strategy.cc:159-162.
    fn ShouldSnapOnY(&self) -> bool {
        self.step_.y() != 0.0
    }
    // Source: cc/input/snap_selection_strategy.h:217; cc/input/snap_selection_strategy.cc:163-166.
    fn intended_position(&self) -> PointF {
        self.current_position_ + self.step_
    }
    // Source: cc/input/snap_selection_strategy.h:218; cc/input/snap_selection_strategy.cc:167-172.
    fn base_position(&self) -> PointF {
        if self.preferred_step_ == StepPreference::kDirection {
            self.current_position_
        } else {
            self.current_position_ + self.step_
        }
    }
    // Source: cc/input/snap_selection_strategy.h:85,119-121.
    fn current_position(&self) -> &PointF {
        &self.current_position_
    }
    // Source: cc/input/snap_selection_strategy.h:220; cc/input/snap_selection_strategy.cc:173-185.
    fn IsPreferredSnapPosition(&self, axis: SearchAxis, position: f32) -> bool {
        if axis == SearchAxis::kX {
            let delta = position - self.current_position_.x();
            delta.abs() >= self.preferred_min_displacement_.x().abs()
                && delta.abs() <= self.preferred_max_displacement_.x().abs()
        } else {
            let delta = position - self.current_position_.y();
            delta.abs() >= self.preferred_min_displacement_.y().abs()
                && delta.abs() <= self.preferred_max_displacement_.y().abs()
        }
    }
    // Source: cc/input/snap_selection_strategy.h:221; cc/input/snap_selection_strategy.cc:186-206.
    fn IsValidSnapPosition(&self, axis: SearchAxis, position: f32) -> bool {
        if axis == SearchAxis::kX {
            let mut delta = position - self.current_position_.x();
            if !self.use_fractional_offsets_ {
                delta = if delta > 0.0 {
                    delta.floor()
                } else {
                    delta.ceil()
                };
            }
            (self.step_.x() > 0.0 && delta > 0.0) || (self.step_.x() < 0.0 && delta < 0.0)
        } else {
            let mut delta = position - self.current_position_.y();
            if !self.use_fractional_offsets_ {
                delta = if delta > 0.0 {
                    delta.floor()
                } else {
                    delta.ceil()
                };
            }
            (self.step_.y() > 0.0 && delta > 0.0) || (self.step_.y() < 0.0 && delta < 0.0)
        }
    }
    // Source: cc/input/snap_selection_strategy.h:222-223; cc/input/snap_selection_strategy.cc:207-213.
    fn IsValidSnapArea(&self, axis: SearchAxis, area: &SnapAreaData) -> bool {
        IsValidSnapAreaBase(axis, area)
            && (self.snap_stop_always_filter_ == SnapStopAlwaysFilter::kIgnore || area.must_snap)
    }
    // Source: cc/input/snap_selection_strategy.h:225; cc/input/snap_selection_strategy.cc:214-217.
    fn ShouldRespectSnapStop(&self) -> bool {
        true
    }
    // Source: cc/input/snap_selection_strategy.h:227-229; cc/input/snap_selection_strategy.cc:218-253.
    fn PickBestResult<'r, 'a>(
        &self,
        closest: &'r Option<SnapSearchResult<'a>>,
        covering: &'r Option<SnapSearchResult<'a>>,
    ) -> &'r Option<SnapSearchResult<'a>> {
        if closest.is_none() {
            return covering;
        }
        if covering.is_none() {
            return closest;
        }
        if covering.as_ref().unwrap().element_id() == closest.as_ref().unwrap().element_id() {
            return covering;
        }
        if self.preferred_step_ == StepPreference::kDirection {
            if (self.step_.x() > 0.0 || self.step_.y() > 0.0)
                && closest.as_ref().unwrap().snap_offset()
                    < covering.as_ref().unwrap().snap_offset()
            {
                return closest;
            }
            if (self.step_.x() < 0.0 || self.step_.y() < 0.0)
                && closest.as_ref().unwrap().snap_offset()
                    > covering.as_ref().unwrap().snap_offset()
            {
                return closest;
            }
        }
        covering
    }
    // Source: cc/input/snap_selection_strategy.h:231; cc/input/snap_selection_strategy.cc:254-257.
    fn UsingFractionalOffsets(&self) -> bool {
        self.use_fractional_offsets_
    }
    // Source: cc/input/snap_selection_strategy.h:233; cc/input/snap_selection_strategy.cc:258-261.
    fn Clone(&self) -> Box<dyn SnapSelectionStrategy> {
        Box::new(self.clone())
    }
}

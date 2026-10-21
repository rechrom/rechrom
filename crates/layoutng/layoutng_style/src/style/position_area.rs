use foundation::{PhysicalToLogical, WritingDirectionMode};
use layoutng_geometry::geometry::axis::{
    kLogicalAxesBlock, kLogicalAxesInline, kPhysicalAxesBoth, kPhysicalAxesHorizontal,
    kPhysicalAxesNone, kPhysicalAxesVertical, PhysicalAxes, ToPhysicalAxes,
};
use layoutng_geometry::geometry::box_sides::PhysicalBoxSides;
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;

use super::computed_style_constants::{ItemPosition, OverflowAlignment};
use super::style_self_alignment_data::StyleSelfAlignmentData;

// cpp: layoutng_style/style/position_area.h:24-54
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[allow(non_camel_case_types)]
pub enum PositionAreaRegion {
    kNone,
    kAll,
    kCenter,
    kStart,
    kEnd,
    kSelfStart,
    kSelfEnd,
    kInlineStart,
    kInlineEnd,
    kSelfInlineStart,
    kSelfInlineEnd,
    kBlockStart,
    kBlockEnd,
    kSelfBlockStart,
    kSelfBlockEnd,
    kTop,
    kBottom,
    kLeft,
    kRight,
    kXStart,
    kXEnd,
    kYStart,
    kYEnd,
    kSelfXStart,
    kSelfXEnd,
    kSelfYStart,
    kSelfYEnd,
    kAny,
}

// cpp: layoutng_style/style/position_area.h:56-132
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PositionArea {
    span1_start_: PositionAreaRegion,
    span1_end_: PositionAreaRegion,
    span2_start_: PositionAreaRegion,
    span2_end_: PositionAreaRegion,
}

#[allow(non_snake_case)]
impl PositionArea {
    // cpp: layoutng_style/style/position_area.h:73-80
    pub const fn new(
        span1_start: PositionAreaRegion,
        span1_end: PositionAreaRegion,
        span2_start: PositionAreaRegion,
        span2_end: PositionAreaRegion,
    ) -> Self {
        Self {
            span1_start_: span1_start,
            span1_end_: span1_end,
            span2_start_: span2_start,
            span2_end_: span2_end,
        }
    }

    // cpp: layoutng_style/style/position_area.h:82-85
    pub const fn FirstStart(&self) -> PositionAreaRegion {
        self.span1_start_
    }
    pub const fn FirstEnd(&self) -> PositionAreaRegion {
        self.span1_end_
    }
    pub const fn SecondStart(&self) -> PositionAreaRegion {
        self.span2_start_
    }
    pub const fn SecondEnd(&self) -> PositionAreaRegion {
        self.span2_end_
    }

    // cpp: layoutng_style/style/position_area.h:92
    pub fn IsNone(&self) -> bool {
        self.span1_start_ == PositionAreaRegion::kNone
    }

    // cpp: layoutng_style/style/position_area.h:93-98
    pub fn ContainsAny(&self) -> bool {
        self.span1_start_ == PositionAreaRegion::kAny
            || self.span1_end_ == PositionAreaRegion::kAny
            || self.span2_start_ == PositionAreaRegion::kAny
            || self.span2_end_ == PositionAreaRegion::kAny
    }

    // cpp: layoutng_style/style/position_area.h:99-108
    pub fn Matches(&self, other: &Self) -> bool {
        (self.span1_start_ == PositionAreaRegion::kAny
            || other.span1_start_ == PositionAreaRegion::kAny
            || (self.span1_start_ == other.span1_start_ && self.span1_end_ == other.span1_end_))
            && (self.span2_start_ == PositionAreaRegion::kAny
                || other.span2_start_ == PositionAreaRegion::kAny
                || (self.span2_start_ == other.span2_start_ && self.span2_end_ == other.span2_end_))
    }
}

// cpp: layoutng_style/style/position_area.h:72
impl Default for PositionArea {
    fn default() -> Self {
        Self::new(
            PositionAreaRegion::kNone,
            PositionAreaRegion::kNone,
            PositionAreaRegion::kNone,
            PositionAreaRegion::kNone,
        )
    }
}

// cpp: layoutng_style/style/position_area.h:134-146
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PositionAreaOffsets {
    pub insets: PhysicalBoxStrut,
    pub behaves_as_auto: PhysicalBoxSides,
}
impl PositionAreaOffsets {
    // cpp: layoutng_style/style/position_area.h:140-141
    pub const fn new(insets: PhysicalBoxStrut, behaves_as_auto: PhysicalBoxSides) -> Self {
        Self {
            insets,
            behaves_as_auto,
        }
    }
}

// cpp: layoutng_style/style/position_area.cc:14-52
#[allow(non_snake_case)]
fn PhysicalAxisFromRegion(
    region: PositionAreaRegion,
    container: &WritingDirectionMode,
    self_direction: &WritingDirectionMode,
) -> PhysicalAxes {
    use PositionAreaRegion::*;
    match region {
        kTop | kBottom | kYStart | kYEnd | kSelfYStart | kSelfYEnd => kPhysicalAxesVertical,
        kLeft | kRight | kXStart | kXEnd | kSelfXStart | kSelfXEnd => kPhysicalAxesHorizontal,
        kInlineStart | kInlineEnd => {
            if container.IsHorizontal() {
                kPhysicalAxesHorizontal
            } else {
                kPhysicalAxesVertical
            }
        }
        kSelfInlineStart | kSelfInlineEnd => {
            if self_direction.IsHorizontal() {
                kPhysicalAxesHorizontal
            } else {
                kPhysicalAxesVertical
            }
        }
        kBlockStart | kBlockEnd => {
            if container.IsHorizontal() {
                kPhysicalAxesVertical
            } else {
                kPhysicalAxesHorizontal
            }
        }
        kSelfBlockStart | kSelfBlockEnd => {
            if self_direction.IsHorizontal() {
                kPhysicalAxesVertical
            } else {
                kPhysicalAxesHorizontal
            }
        }
        _ => kPhysicalAxesNone,
    }
}

// cpp: layoutng_style/style/position_area.cc:54-62
#[allow(non_snake_case)]
fn PhysicalAxisFromSpan(
    start: PositionAreaRegion,
    end: PositionAreaRegion,
    container: &WritingDirectionMode,
    self_direction: &WritingDirectionMode,
) -> PhysicalAxes {
    if start == PositionAreaRegion::kAll {
        return kPhysicalAxesNone;
    }
    PhysicalAxisFromRegion(
        if start == PositionAreaRegion::kCenter {
            end
        } else {
            start
        },
        container,
        self_direction,
    )
}

// cpp: layoutng_style/style/position_area.cc:64-124
#[allow(non_snake_case)]
fn ToPhysicalRegion(
    region: PositionAreaRegion,
    axis: PhysicalAxes,
    container: &WritingDirectionMode,
    self_direction: &WritingDirectionMode,
) -> PositionAreaRegion {
    use PositionAreaRegion::*;
    let horizontal = axis == kPhysicalAxesHorizontal;
    let axis_region = match region {
        kNone | kAll => unreachable!("nonphysical empty region"),
        kCenter | kTop | kBottom | kLeft | kRight | kAny => return region,
        kStart | kInlineStart | kBlockStart => {
            if horizontal {
                kXStart
            } else {
                kYStart
            }
        }
        kEnd | kInlineEnd | kBlockEnd => {
            if horizontal {
                kXEnd
            } else {
                kYEnd
            }
        }
        kSelfStart | kSelfInlineStart | kSelfBlockStart => {
            if horizontal {
                kSelfXStart
            } else {
                kSelfYStart
            }
        }
        kSelfEnd | kSelfInlineEnd | kSelfBlockEnd => {
            if horizontal {
                kSelfXEnd
            } else {
                kSelfYEnd
            }
        }
        _ => region,
    };
    if horizontal {
        if (axis_region == kXStart && container.IsFlippedX())
            || (axis_region == kXEnd && !container.IsFlippedX())
            || (axis_region == kSelfXStart && self_direction.IsFlippedX())
            || (axis_region == kSelfXEnd && !self_direction.IsFlippedX())
        {
            return kRight;
        }
        return kLeft;
    }
    if (axis_region == kYStart && container.IsFlippedY())
        || (axis_region == kYEnd && !container.IsFlippedY())
        || (axis_region == kSelfYStart && self_direction.IsFlippedY())
        || (axis_region == kSelfYEnd && !self_direction.IsFlippedY())
    {
        return kBottom;
    }
    kTop
}

// cpp: layoutng_style/style/position_area.cc:126-129
#[allow(non_snake_case)]
fn IsAmbiguousSelfAreaRegion(region: PositionAreaRegion) -> bool {
    region == PositionAreaRegion::kSelfStart || region == PositionAreaRegion::kSelfEnd
}

// cpp: layoutng_style/style/position_area.cc:133-138
#[allow(non_snake_case)]
impl PositionArea {
    fn IsAmbiguousSelfReferenceBox(&self) -> bool {
        IsAmbiguousSelfAreaRegion(self.FirstStart())
            || IsAmbiguousSelfAreaRegion(self.FirstEnd())
            || IsAmbiguousSelfAreaRegion(self.SecondStart())
            || IsAmbiguousSelfAreaRegion(self.SecondEnd())
    }
}

#[allow(non_snake_case)]
impl PositionArea {
    // cpp: layoutng_style/style/position_area.cc:140-186
    pub fn ToPhysical(
        &self,
        container: &WritingDirectionMode,
        self_direction: &WritingDirectionMode,
    ) -> Self {
        if self.IsNone() {
            return *self;
        }
        let mut first = PhysicalAxisFromSpan(
            self.FirstStart(),
            self.FirstEnd(),
            container,
            self_direction,
        );
        let mut second = PhysicalAxisFromSpan(
            self.SecondStart(),
            self.SecondEnd(),
            container,
            self_direction,
        );
        if first == second {
            assert_eq!(first, kPhysicalAxesNone);
            let mode = if self.IsAmbiguousSelfReferenceBox() {
                self_direction.GetWritingMode()
            } else {
                container.GetWritingMode()
            };
            first = ToPhysicalAxes(kLogicalAxesBlock, mode);
            second = ToPhysicalAxes(kLogicalAxesInline, mode);
        } else if first == kPhysicalAxesNone {
            first = second ^ kPhysicalAxesBoth;
        } else if second == kPhysicalAxesNone {
            second = first ^ kPhysicalAxesBoth;
        }
        debug_assert_eq!(first ^ second, kPhysicalAxesBoth);

        let mut regions = [
            PositionAreaRegion::kTop,
            PositionAreaRegion::kBottom,
            PositionAreaRegion::kLeft,
            PositionAreaRegion::kRight,
        ];
        let mut index = if first == kPhysicalAxesHorizontal {
            2
        } else {
            0
        };
        if self.FirstStart() != PositionAreaRegion::kAll {
            regions[index] = ToPhysicalRegion(self.FirstStart(), first, container, self_direction);
            regions[index + 1] =
                ToPhysicalRegion(self.FirstEnd(), first, container, self_direction);
        }
        index = (index + 2) % 4;
        if self.SecondStart() != PositionAreaRegion::kAll {
            regions[index] =
                ToPhysicalRegion(self.SecondStart(), second, container, self_direction);
            regions[index + 1] =
                ToPhysicalRegion(self.SecondEnd(), second, container, self_direction);
        }
        if regions[0] == PositionAreaRegion::kBottom || regions[1] == PositionAreaRegion::kTop {
            regions.swap(0, 1);
        }
        if regions[2] == PositionAreaRegion::kRight || regions[3] == PositionAreaRegion::kLeft {
            regions.swap(2, 3);
        }
        Self::new(regions[0], regions[1], regions[2], regions[3])
    }

    // cpp: layoutng_style/style/position_area.cc:188-218
    pub fn AlignJustifySelfFromPhysical(
        &self,
        container: WritingDirectionMode,
    ) -> (StyleSelfAlignmentData, StyleSelfAlignmentData) {
        let mut align = ItemPosition::kStart;
        let mut align_reverse = ItemPosition::kEnd;
        let mut justify = ItemPosition::kStart;
        let mut justify_reverse = ItemPosition::kEnd;
        assert!(!self.ContainsAny());
        if self.FirstStart() == PositionAreaRegion::kTop
            && self.FirstEnd() == PositionAreaRegion::kBottom
        {
            align_reverse = ItemPosition::kAnchorCenter;
            align = align_reverse;
        } else if self.FirstStart() == PositionAreaRegion::kCenter
            && self.FirstEnd() == PositionAreaRegion::kCenter
        {
            align_reverse = ItemPosition::kCenter;
            align = align_reverse;
        } else if self.FirstStart() == PositionAreaRegion::kTop {
            std::mem::swap(&mut align, &mut align_reverse);
        }
        if self.SecondStart() == PositionAreaRegion::kLeft
            && self.SecondEnd() == PositionAreaRegion::kRight
        {
            justify_reverse = ItemPosition::kAnchorCenter;
            justify = justify_reverse;
        } else if self.SecondStart() == PositionAreaRegion::kCenter
            && self.SecondEnd() == PositionAreaRegion::kCenter
        {
            justify_reverse = ItemPosition::kCenter;
            justify = justify_reverse;
        } else if self.SecondStart() == PositionAreaRegion::kLeft {
            std::mem::swap(&mut justify, &mut justify_reverse);
        }
        let converter =
            PhysicalToLogical::new(container, align, justify_reverse, align_reverse, justify);
        (
            StyleSelfAlignmentData::new_nonlegacy(
                converter.BlockStart(),
                OverflowAlignment::kDefault,
            ),
            StyleSelfAlignmentData::new_nonlegacy(
                converter.InlineStart(),
                OverflowAlignment::kDefault,
            ),
        )
    }
}

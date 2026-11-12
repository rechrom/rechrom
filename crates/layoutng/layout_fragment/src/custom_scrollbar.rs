use foundation::{
    gfx, EDisplay, EScrollbarWidth, HeapHashMap, MakeGarbageCollected, Member, Visitor,
};
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_style::style::computed_style::ComputedStyle;

use super::super::layout_custom_scrollbar_part::LayoutCustomScrollbarPart;
use super::super::layout_input_types::ScrollbarThemeMetrics;
use super::super::scrollbar_orientation::ScrollbarOrientation;
use super::super::scrollbar_part::ScrollbarPart;
use super::super::scrollbar_theme_metrics::{
    ScrollbarThemeThickness, ValidateScrollbarThemeMetrics,
};

#[derive(Debug)]
pub struct InvalidScrollbarOrientation;
impl std::fmt::Display for InvalidScrollbarOrientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid scrollbar orientation")
    }
}
impl std::error::Error for InvalidScrollbarOrientation {}

#[derive(Debug)]
pub struct InvalidScrollbarPart;
impl std::fmt::Display for InvalidScrollbarPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Expected a single scrollbar part")
    }
}
impl std::error::Error for InvalidScrollbarPart {}

pub struct CustomScrollbar {
    orientation_: ScrollbarOrientation,
    frame_rect_: gfx::Rect,
    metrics_: ScrollbarThemeMetrics,
    parts_: HeapHashMap<ScrollbarPart, Member<LayoutCustomScrollbarPart>>,
}

#[allow(non_snake_case)]
impl CustomScrollbar {
    pub fn new(
        orientation: ScrollbarOrientation,
        frame: &gfx::Rect,
        metrics: ScrollbarThemeMetrics,
    ) -> Self {
        if orientation != ScrollbarOrientation::kHorizontalScrollbar
            && orientation != ScrollbarOrientation::kVerticalScrollbar
        {
            std::panic::panic_any(InvalidScrollbarOrientation);
        }
        ValidateScrollbarThemeMetrics(&metrics);
        Self {
            orientation_: orientation,
            frame_rect_: *frame,
            metrics_: metrics,
            parts_: HeapHashMap::default(),
        }
    }

    pub fn HypotheticalScrollbarThickness(
        orientation: ScrollbarOrientation,
        background_style: *const ComputedStyle,
        metrics: ScrollbarThemeMetrics,
    ) -> i32 {
        let scrollbar =
            MakeGarbageCollected(Self::new(orientation, &gfx::Rect::default(), metrics));
        unsafe { &mut *scrollbar }.SetPartStyle(ScrollbarPart::kScrollbarBGPart, background_style);
        unsafe { &*scrollbar }.ComputeThickness()
    }

    pub fn Orientation(&self) -> ScrollbarOrientation {
        self.orientation_
    }
    pub fn FrameRect(&self) -> &gfx::Rect {
        &self.frame_rect_
    }
    pub fn SetFrameRect(&mut self, rect: &gfx::Rect) {
        self.frame_rect_ = *rect;
    }
    pub fn X(&self) -> i32 {
        self.frame_rect_.x()
    }
    pub fn Y(&self) -> i32 {
        self.frame_rect_.y()
    }
    pub fn Width(&self) -> i32 {
        self.frame_rect_.width()
    }
    pub fn Height(&self) -> i32 {
        self.frame_rect_.height()
    }
    pub fn Location(&self) -> gfx::Point {
        self.frame_rect_.origin()
    }

    pub fn ThemeThickness(&self, width: EScrollbarWidth) -> i32 {
        ScrollbarThemeThickness(&self.metrics_, width)
    }

    pub fn ThemeMinimumThumbLength(&self) -> i32 {
        self.metrics_.minimum_thumb_length
    }

    pub fn ComputeThickness(&self) -> i32 {
        let part = self.GetPart(ScrollbarPart::kScrollbarBGPart);
        if part.is_null() {
            0
        } else {
            unsafe { &*part }.ComputeThickness()
        }
    }

    pub fn IsOverlayScrollbar(&self) -> bool {
        false
    }

    pub fn GetPart(&self, part: ScrollbarPart) -> *const LayoutCustomScrollbarPart {
        self.parts_
            .get(&part)
            .map_or(std::ptr::null(), |value| value.Get())
    }

    #[inline(never)]
    pub fn SetPartStyle(&mut self, part_type: ScrollbarPart, part_style: *const ComputedStyle) {
        if part_type == ScrollbarPart::kNoPart {
            return;
        }
        match part_type {
            ScrollbarPart::kBackButtonStartPart
            | ScrollbarPart::kForwardButtonStartPart
            | ScrollbarPart::kBackTrackPart
            | ScrollbarPart::kThumbPart
            | ScrollbarPart::kForwardTrackPart
            | ScrollbarPart::kBackButtonEndPart
            | ScrollbarPart::kForwardButtonEndPart
            | ScrollbarPart::kScrollbarBGPart
            | ScrollbarPart::kTrackBGPart => {}
            _ => std::panic::panic_any(InvalidScrollbarPart),
        }
        let mut need_layout_object =
            !part_style.is_null() && unsafe { &*part_style }.Display() != EDisplay::kNone;
        if need_layout_object && unsafe { &*part_style }.Display() != EDisplay::kBlock {
            match part_type {
                ScrollbarPart::kBackButtonStartPart | ScrollbarPart::kForwardButtonEndPart => {
                    need_layout_object = self.metrics_.has_buttons;
                }
                ScrollbarPart::kBackButtonEndPart | ScrollbarPart::kForwardButtonStartPart => {
                    need_layout_object = false;
                }
                _ => {}
            }
        }
        let mut part_layout_object = self.GetPart(part_type) as *mut LayoutCustomScrollbarPart;
        if part_layout_object.is_null() && need_layout_object {
            part_layout_object =
                MakeGarbageCollected(LayoutCustomScrollbarPart::new(self, part_type, part_style));
            self.parts_
                .insert(part_type, Member::from_ptr(part_layout_object));
        } else if !part_layout_object.is_null() && !need_layout_object {
            self.parts_.erase(&part_type);
            part_layout_object = std::ptr::null_mut();
        }
        if !part_layout_object.is_null() {
            unsafe { &mut *part_layout_object }.SetStyle(part_style);
        }
    }

    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.parts_);
    }

    pub fn ButtonRect(&self, part_type: ScrollbarPart) -> gfx::Rect {
        let part = self.GetPart(part_type);
        if part.is_null() {
            return gfx::Rect::default();
        }
        let is_horizontal = self.Orientation() == ScrollbarOrientation::kHorizontalScrollbar;
        let button_length = unsafe { &*part }.ComputeLength();
        let mut rect = gfx::Rect::new(
            gfx::Point::new(self.X(), self.Y()),
            gfx::Size::new(
                if is_horizontal {
                    button_length
                } else {
                    self.Width()
                },
                if is_horizontal {
                    self.Height()
                } else {
                    button_length
                },
            ),
        );
        match part_type {
            ScrollbarPart::kBackButtonStartPart => {}
            ScrollbarPart::kForwardButtonEndPart => {
                rect.Offset(
                    if is_horizontal {
                        self.Width() - button_length
                    } else {
                        0
                    },
                    if is_horizontal {
                        0
                    } else {
                        self.Height() - button_length
                    },
                );
            }
            ScrollbarPart::kForwardButtonStartPart => {
                let previous = self.ButtonRect(ScrollbarPart::kBackButtonStartPart);
                rect.Offset(
                    if is_horizontal { previous.width() } else { 0 },
                    if is_horizontal { 0 } else { previous.height() },
                );
            }
            ScrollbarPart::kBackButtonEndPart => {
                let next = self.ButtonRect(ScrollbarPart::kForwardButtonEndPart);
                rect.Offset(
                    if is_horizontal {
                        self.Width() - next.width() - button_length
                    } else {
                        0
                    },
                    if is_horizontal {
                        0
                    } else {
                        self.Height() - next.height() - button_length
                    },
                );
            }
            _ => std::process::abort(),
        }
        rect
    }

    pub fn TrackRect(&self, mut start_length: i32, mut end_length: i32) -> gfx::Rect {
        let part = self.GetPart(ScrollbarPart::kTrackBGPart);
        let margins = if part.is_null() {
            PhysicalBoxStrut::default()
        } else {
            unsafe { &*part }.MarginOutsets()
        };
        if self.Orientation() == ScrollbarOrientation::kHorizontalScrollbar {
            start_length += margins.left.ToInt();
            end_length += margins.right.ToInt();
            let total_length = start_length + end_length;
            return gfx::Rect::new(
                gfx::Point::new(self.X() + start_length, self.Y()),
                gfx::Size::new(self.Width() - total_length, self.Height()),
            );
        }
        start_length += margins.top.ToInt();
        end_length += margins.bottom.ToInt();
        let total_length = start_length + end_length;
        gfx::Rect::new(
            gfx::Point::new(self.X(), self.Y() + start_length),
            gfx::Size::new(self.Width(), self.Height() - total_length),
        )
    }

    pub fn TrackPieceRectWithMargins(
        &self,
        part_type: ScrollbarPart,
        old_rect: &gfx::Rect,
    ) -> gfx::Rect {
        let part = self.GetPart(part_type);
        if part.is_null() {
            return *old_rect;
        }
        let margins = unsafe { &*part }.MarginOutsets();
        let mut rect = *old_rect;
        if self.Orientation() == ScrollbarOrientation::kHorizontalScrollbar {
            rect.set_x((foundation::LayoutUnit::from_signed(rect.x()) + margins.left).ToInt());
            rect.set_width(
                (foundation::LayoutUnit::from_signed(rect.width()) - margins.HorizontalSum())
                    .ToInt(),
            );
        } else {
            rect.set_y((foundation::LayoutUnit::from_signed(rect.y()) + margins.top).ToInt());
            rect.set_height(
                (foundation::LayoutUnit::from_signed(rect.height()) - margins.VerticalSum())
                    .ToInt(),
            );
        }
        rect
    }

    // cpp: CustomScrollbarTheme::ButtonSizesAlongTrackAxis/TrackRect and
    // ConstrainTrackRectToTrackPieces, custom_scrollbar_theme.cc:60-139.
    pub(crate) fn ThemeTrackRect(&self) -> gfx::Rect {
        let horizontal = self.Orientation() == ScrollbarOrientation::kHorizontalScrollbar;
        let length = |part| {
            let rect = self.ButtonRect(part);
            if horizontal {
                rect.width()
            } else {
                rect.height()
            }
        };
        let before = length(ScrollbarPart::kBackButtonStartPart)
            + length(ScrollbarPart::kForwardButtonStartPart);
        let after = length(ScrollbarPart::kBackButtonEndPart)
            + length(ScrollbarPart::kForwardButtonEndPart);
        if before + after
            > if horizontal {
                self.Width()
            } else {
                self.Height()
            }
        {
            return *self.FrameRect();
        }
        self.TrackRect(before, after)
    }
    pub(crate) fn ConstrainTrackRect(&self, rect: &gfx::Rect) -> gfx::Rect {
        let back = self.TrackPieceRectWithMargins(ScrollbarPart::kBackTrackPart, rect);
        let forward = self.TrackPieceRectWithMargins(ScrollbarPart::kForwardTrackPart, rect);
        let mut result = *rect;
        if self.Orientation() == ScrollbarOrientation::kHorizontalScrollbar {
            result.set_x(back.x());
            result.set_width(forward.right() - back.x());
        } else {
            result.set_y(back.y());
            result.set_height(forward.bottom() - back.y());
        }
        result
    }
    // cpp: scrollbar_theme.cc:195-217,248-278; cc/input/scroll_utils.cc:59-73.
    pub(crate) fn SplitTrack(
        &self,
        enabled: bool,
        visible: i32,
        total: i32,
        current: f32,
    ) -> (gfx::Rect, gfx::Rect, gfx::Rect, gfx::Rect) {
        let unconstrained = self.ThemeTrackRect();
        let track = self.ConstrainTrackRect(&unconstrained);
        let horizontal = self.Orientation() == ScrollbarOrientation::kHorizontalScrollbar;
        let track_length = if horizontal {
            track.width()
        } else {
            track.height()
        };
        let thumb_length = if enabled {
            let proportion = if total > 0 {
                visible as f32 / total as f32
            } else {
                0.0
            };
            ((proportion * track_length as f32).round() as i32)
                .max(self.MinimumThumbLength())
                .min(track_length)
        } else {
            0
        };
        let size = (total - visible) as f32;
        let position = if enabled && size != 0.0 {
            let pos = current.max(0.0) * (track_length - thumb_length) as f32 / size;
            if pos > 0.0 && pos < 1.0 {
                1
            } else {
                pos as i32
            }
        } else {
            0
        };
        if horizontal {
            let thumb = gfx::Rect::new(
                gfx::Point::new(track.x() + position, track.y()),
                gfx::Size::new(thumb_length, self.Height()),
            );
            let before = gfx::Rect::new(
                track.origin(),
                gfx::Size::new(position + thumb.width() / 2, track.height()),
            );
            let after = gfx::Rect::new(
                gfx::Point::new(track.x() + before.width(), track.y()),
                gfx::Size::new(track.right() - before.right(), track.height()),
            );
            (unconstrained, before, thumb, after)
        } else {
            let thumb = gfx::Rect::new(
                gfx::Point::new(track.x(), track.y() + position),
                gfx::Size::new(self.Width(), thumb_length),
            );
            let before = gfx::Rect::new(
                track.origin(),
                gfx::Size::new(track.width(), position + thumb.height() / 2),
            );
            let after = gfx::Rect::new(
                gfx::Point::new(track.x(), track.y() + before.height()),
                gfx::Size::new(track.width(), track.bottom() - before.bottom()),
            );
            (unconstrained, before, thumb, after)
        }
    }
    pub fn PartRect(
        &self,
        part: ScrollbarPart,
        enabled: bool,
        visible: i32,
        total: i32,
        current: f32,
    ) -> gfx::Rect {
        let (track, before, thumb, after) = self.SplitTrack(enabled, visible, total, current);
        match part {
            ScrollbarPart::kBackButtonStartPart
            | ScrollbarPart::kForwardButtonStartPart
            | ScrollbarPart::kBackButtonEndPart
            | ScrollbarPart::kForwardButtonEndPart => self.ButtonRect(part),
            ScrollbarPart::kBackTrackPart => before,
            ScrollbarPart::kForwardTrackPart => after,
            ScrollbarPart::kThumbPart => thumb,
            ScrollbarPart::kTrackBGPart => track,
            ScrollbarPart::kScrollbarBGPart => *self.FrameRect(),
            _ => gfx::Rect::default(),
        }
    }
    // cpp: custom_scrollbar.cc:377-424. Part paint offsets and overridden sizes
    // stay on the actual anonymous part/FragmentData owners.
    pub fn PositionScrollbarParts(
        &mut self,
        enabled: bool,
        visible: i32,
        total: i32,
        current: f32,
    ) {
        let (track, before, thumb, after) = self.SplitTrack(enabled, visible, total, current);
        for (part, owner) in self.parts_.iter() {
            let rect = match *part {
                ScrollbarPart::kBackButtonStartPart
                | ScrollbarPart::kForwardButtonStartPart
                | ScrollbarPart::kBackButtonEndPart
                | ScrollbarPart::kForwardButtonEndPart => self.ButtonRect(*part),
                ScrollbarPart::kBackTrackPart => before,
                ScrollbarPart::kForwardTrackPart => after,
                ScrollbarPart::kThumbPart => thumb,
                ScrollbarPart::kTrackBGPart => track,
                ScrollbarPart::kScrollbarBGPart => *self.FrameRect(),
                _ => unreachable!("custom part is individual"),
            };
            unsafe { &mut *owner.Get() }.SetPartFrameRect(&rect);
        }
    }

    pub fn MinimumThumbLength(&self) -> i32 {
        let part = self.GetPart(ScrollbarPart::kThumbPart);
        if part.is_null() {
            0
        } else {
            unsafe { &*part }.ComputeLength()
        }
    }
}

#[allow(non_snake_case)]
impl CustomScrollbar {
    // Source: core/layout/custom_scrollbar.cc:426-447.
    pub fn GetScrollbarPartStyleForCursor(
        &self,
        part_type: ScrollbarPart,
    ) -> Option<&ComputedStyle> {
        if let Some(part) = unsafe { self.GetPart(part_type).as_ref() } {
            return Some(part.StyleRef());
        }
        match part_type {
            ScrollbarPart::kBackButtonStartPart
            | ScrollbarPart::kForwardButtonStartPart
            | ScrollbarPart::kBackButtonEndPart
            | ScrollbarPart::kForwardButtonEndPart
            | ScrollbarPart::kTrackBGPart
            | ScrollbarPart::kThumbPart => {
                self.GetScrollbarPartStyleForCursor(ScrollbarPart::kScrollbarBGPart)
            }
            ScrollbarPart::kBackTrackPart | ScrollbarPart::kForwardTrackPart => {
                self.GetScrollbarPartStyleForCursor(ScrollbarPart::kTrackBGPart)
            }
            _ => None,
        }
    }
}

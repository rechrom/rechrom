use foundation::{
    gfx, EDisplay, EScrollbarWidth, HeapHashMap, MakeGarbageCollected, Member, Visitor,
};
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_style::style::computed_style::ComputedStyle;

use super::layout_custom_scrollbar_part::LayoutCustomScrollbarPart;
use super::layout_input_types::ScrollbarThemeMetrics;
use super::scrollbar_orientation::ScrollbarOrientation;
use super::scrollbar_part::ScrollbarPart;
use super::scrollbar_theme_metrics::{ScrollbarThemeThickness, ValidateScrollbarThemeMetrics};

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

// cpp: layoutng/internal/custom_scrollbar.h:34-76
pub struct CustomScrollbar {
    orientation_: ScrollbarOrientation,
    frame_rect_: gfx::Rect,
    metrics_: ScrollbarThemeMetrics,
    parts_: HeapHashMap<ScrollbarPart, Member<LayoutCustomScrollbarPart>>,
}

#[allow(non_snake_case)]
impl CustomScrollbar {
    // cpp: layoutng/internal/custom_scrollbar.h:36-36
    // cpp: layoutng/internal/custom_scrollbar.cc:31-37
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

    // cpp: layoutng/internal/custom_scrollbar.h:37-38
    // cpp: layoutng/internal/custom_scrollbar.cc:41-46
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

    // cpp: layoutng/internal/custom_scrollbar.h:41-48
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

    // cpp: layoutng/internal/custom_scrollbar.h:49-50
    // cpp: layoutng/internal/custom_scrollbar.cc:38-40
    pub fn ThemeThickness(&self, width: EScrollbarWidth) -> i32 {
        ScrollbarThemeThickness(&self.metrics_, width)
    }

    pub fn NativeThemeMinimumThumbLength(&self) -> i32 {
        self.metrics_.minimum_thumb_length
    }

    // cpp: layoutng/internal/custom_scrollbar.h:61-61
    // cpp: layoutng/internal/custom_scrollbar.cc:47-50
    pub fn ComputeThickness(&self) -> i32 {
        let part = self.GetPart(ScrollbarPart::kScrollbarBGPart);
        if part.is_null() {
            0
        } else {
            unsafe { &*part }.ComputeThickness()
        }
    }

    // cpp: layoutng/internal/custom_scrollbar.h:64-68
    pub fn IsOverlayScrollbar(&self) -> bool {
        false
    }

    pub fn GetPart(&self, part: ScrollbarPart) -> *const LayoutCustomScrollbarPart {
        self.parts_
            .get(&part)
            .map_or(std::ptr::null(), |value| value.Get())
    }

    // cpp: layoutng/internal/custom_scrollbar.h:39-39
    // cpp: layoutng/internal/custom_scrollbar.cc:51-90
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

    // cpp: layoutng/internal/custom_scrollbar.h:69-69
    // cpp: layoutng/internal/custom_scrollbar.cc:91-91
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.parts_);
    }

    // cpp: layoutng/internal/custom_scrollbar.h:51-51
    // cpp: layoutng/internal/custom_scrollbar.cc:93-128
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

    // cpp: layoutng/internal/custom_scrollbar.h:52-52
    // cpp: layoutng/internal/custom_scrollbar.cc:130-147
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

    // cpp: layoutng/internal/custom_scrollbar.h:53-53
    // cpp: layoutng/internal/custom_scrollbar.cc:149-167
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

    // cpp: layoutng/internal/custom_scrollbar.h:54-54
    // cpp: layoutng/internal/custom_scrollbar.cc:169-173
    pub fn MinimumThumbLength(&self) -> i32 {
        let part = self.GetPart(ScrollbarPart::kThumbPart);
        if part.is_null() {
            0
        } else {
            unsafe { &*part }.ComputeLength()
        }
    }
}

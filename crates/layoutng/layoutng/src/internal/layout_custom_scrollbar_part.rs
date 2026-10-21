use foundation::{
    EDisplay, LayoutUnit, Length, Member, MinimumValueForLength, PhysicalSize,
    RuntimeEnabledFeatures, Visitor,
};
use layoutng_geometry::geometry::box_strut::PhysicalBoxStrut;
use layoutng_style::style::computed_style::ComputedStyle;

use super::custom_scrollbar::CustomScrollbar;
use super::scrollbar_orientation::ScrollbarOrientation;
use super::scrollbar_part::ScrollbarPart;

// cpp: layoutng/internal/layout_custom_scrollbar_part.h:44-44
#[derive(Clone, Copy, PartialEq, Eq)]
enum ScrollbarSizeComputeMode {
    Thickness,
    Length,
}

// cpp: layoutng/internal/layout_custom_scrollbar_part.h:31-51
pub struct LayoutCustomScrollbarPart {
    style_: Member<ComputedStyle>,
    scrollbar_: Member<CustomScrollbar>,
    overridden_size_: PhysicalSize,
    part_: ScrollbarPart,
}

#[allow(non_snake_case)]
impl LayoutCustomScrollbarPart {
    // cpp: layoutng/internal/layout_custom_scrollbar_part.h:34-34
    // cpp: layoutng/internal/layout_custom_scrollbar_part.cc:32-34
    pub fn new(
        scrollbar: *mut CustomScrollbar,
        part: ScrollbarPart,
        style: *const ComputedStyle,
    ) -> Self {
        assert!(!style.is_null());
        Self {
            style_: Member::from_ptr(style as *mut ComputedStyle),
            scrollbar_: Member::from_ptr(scrollbar),
            overridden_size_: PhysicalSize::default(),
            part_: part,
        }
    }

    // cpp: layoutng/internal/layout_custom_scrollbar_part.h:35-35
    // cpp: layoutng/internal/layout_custom_scrollbar_part.cc:35-38
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.style_);
        visitor.Trace(&self.scrollbar_);
    }

    // cpp: layoutng/internal/layout_custom_scrollbar_part.h:36-37
    pub fn StyleRef(&self) -> &ComputedStyle {
        unsafe { &*self.style_.Get() }
    }

    // cpp: layoutng/internal/layout_custom_scrollbar_part.h:45
    pub fn SetStyle(&mut self, style: *const ComputedStyle) {
        assert!(!style.is_null());
        self.style_ = Member::from_ptr(style as *mut ComputedStyle);
    }

    // cpp: layoutng/internal/layout_custom_scrollbar_part.cc:44-60
    fn compute_size(
        &self,
        length: &Length,
        container_size: i32,
        compute_mode: ScrollbarSizeComputeMode,
    ) -> i32 {
        if !length.HasAutoOrContentOrIntrinsic() && !length.HasStretch() {
            assert!(length.HasOnlyFixedAndPercent());
            return MinimumValueForLength(length, LayoutUnit::from_signed(container_size)).ToInt();
        }
        if RuntimeEnabledFeatures::CustomScrollbarApplyMinimumThumbLengthEnabled()
            && compute_mode == ScrollbarSizeComputeMode::Length
            && self.part_ == ScrollbarPart::kThumbPart
        {
            return unsafe { &*self.scrollbar_.Get() }.NativeThemeMinimumThumbLength();
        }
        unsafe { &*self.scrollbar_.Get() }.ThemeThickness(self.StyleRef().UsedScrollbarWidth())
    }

    // cpp: layoutng/internal/layout_custom_scrollbar_part.cc:62-80
    fn compute_width(&self, container_width: i32, compute_mode: ScrollbarSizeComputeMode) -> i32 {
        let style = self.StyleRef();
        if style.Display() == EDisplay::kNone {
            return 0;
        }
        let width = self.compute_size(style.Width(), container_width, compute_mode);
        let min_width = if style.MinWidth().IsAuto() {
            0
        } else {
            self.compute_size(style.MinWidth(), container_width, compute_mode)
        };
        let max_width = if style.MaxWidth().IsNone() {
            width
        } else {
            self.compute_size(style.MaxWidth(), container_width, compute_mode)
        };
        min_width.max(max_width.min(width))
    }

    // cpp: layoutng/internal/layout_custom_scrollbar_part.cc:82-100
    fn compute_height(&self, container_height: i32, compute_mode: ScrollbarSizeComputeMode) -> i32 {
        let style = self.StyleRef();
        if style.Display() == EDisplay::kNone {
            return 0;
        }
        let height = self.compute_size(style.Height(), container_height, compute_mode);
        let min_height = if style.MinHeight().IsAuto() {
            0
        } else {
            self.compute_size(style.MinHeight(), container_height, compute_mode)
        };
        let max_height = if style.MaxHeight().IsNone() {
            height
        } else {
            self.compute_size(style.MaxHeight(), container_height, compute_mode)
        };
        min_height.max(max_height.min(height))
    }

    // cpp: layoutng/internal/layout_custom_scrollbar_part.h:38-38
    // cpp: layoutng/internal/layout_custom_scrollbar_part.cc:102-111
    pub fn ComputeThickness(&self) -> i32 {
        debug_assert_eq!(self.part_, ScrollbarPart::kScrollbarBGPart);
        if unsafe { &*self.scrollbar_.Get() }.Orientation()
            == ScrollbarOrientation::kHorizontalScrollbar
        {
            self.compute_height(0, ScrollbarSizeComputeMode::Thickness)
        } else {
            self.compute_width(0, ScrollbarSizeComputeMode::Thickness)
        }
    }

    // cpp: layoutng/internal/layout_custom_scrollbar_part.h:39-39
    // cpp: layoutng/internal/layout_custom_scrollbar_part.cc:113-122
    pub fn ComputeLength(&self) -> i32 {
        debug_assert_ne!(self.part_, ScrollbarPart::kScrollbarBGPart);
        let scrollbar = unsafe { &*self.scrollbar_.Get() };
        if scrollbar.Orientation() == ScrollbarOrientation::kHorizontalScrollbar {
            self.compute_width(
                scrollbar.FrameRect().width(),
                ScrollbarSizeComputeMode::Length,
            )
        } else {
            self.compute_height(
                scrollbar.FrameRect().height(),
                ScrollbarSizeComputeMode::Length,
            )
        }
    }

    // cpp: layoutng/internal/layout_custom_scrollbar_part.h:41-42
    // cpp: layoutng/internal/layout_custom_scrollbar_part.cc:124-130
    pub fn SetOverriddenSize(&mut self, size: &PhysicalSize) {
        self.overridden_size_ = *size;
    }

    pub fn StitchedSize(&self) -> PhysicalSize {
        self.overridden_size_
    }

    // cpp: layoutng/internal/layout_custom_scrollbar_part.cc:138-149
    pub fn MarginOutsets(&self) -> PhysicalBoxStrut {
        let scrollbar_ptr = self.scrollbar_.Get();
        let is_horizontal = !scrollbar_ptr.is_null()
            && unsafe { &*scrollbar_ptr }.Orientation()
                == ScrollbarOrientation::kHorizontalScrollbar;
        let is_vertical = !scrollbar_ptr.is_null()
            && unsafe { &*scrollbar_ptr }.Orientation() == ScrollbarOrientation::kVerticalScrollbar;
        PhysicalBoxStrut::new(
            if is_horizontal {
                LayoutUnit::default()
            } else {
                compute_margin(self.StyleRef().MarginTop())
            },
            if is_vertical {
                LayoutUnit::default()
            } else {
                compute_margin(self.StyleRef().MarginRight())
            },
            if is_horizontal {
                LayoutUnit::default()
            } else {
                compute_margin(self.StyleRef().MarginBottom())
            },
            if is_vertical {
                LayoutUnit::default()
            } else {
                compute_margin(self.StyleRef().MarginLeft())
            },
        )
    }
}

// cpp: layoutng/internal/layout_custom_scrollbar_part.cc:132-136
fn compute_margin(style_margin: &Length) -> LayoutUnit {
    LayoutUnit::from_signed(MinimumValueForLength(style_margin, LayoutUnit::default()).Round())
}

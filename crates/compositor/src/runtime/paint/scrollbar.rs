use crate::{
    adapter::{
        geometry::{Rect, Size},
        layer_tree_impl::Color4f,
        scrollbar::Insets,
    },
    cc::input::scrollbar::{Scrollbar, ScrollbarOrientation},
};

/// The retained Blink ScrollbarDisplayItem inputs consumed by cc's native
/// solid-color scrollbar layer. Geometry remains relative to the layer, as in
/// Chromium's WebScrollbar adapter.
pub struct NativeScrollbar {
    pub element_id: crate::adapter::input_types::ElementId,
    orientation: ScrollbarOrientation,
    left_side_vertical: bool,
    overlay: bool,
    track: Rect,
    thumb: Rect,
    thumb_color: Color4f,
    minimum_thumb_length: i32,
    needs_update_display: bool,
}

impl NativeScrollbar {
    pub fn from_paint(value: &paint::paint_engine::ScrollbarDisplayItemData) -> Self {
        let rect = |r: paint::paint_engine::PaintRect| {
            Rect::new(
                r.x.round() as i32,
                r.y.round() as i32,
                r.width.round() as i32,
                r.height.round() as i32,
            )
        };
        Self {
            element_id: crate::adapter::input_types::ElementId::new(
                value.element_id.GetInternalValue(),
            ),
            orientation: if value.horizontal {
                ScrollbarOrientation::kHorizontal
            } else {
                ScrollbarOrientation::kVertical
            },
            left_side_vertical: value.left_side_vertical,
            overlay: value.overlay,
            track: rect(value.track_rect),
            thumb: rect(value.thumb_rect),
            thumb_color: Color4f {
                red: value.thumb_color.red,
                green: value.thumb_color.green,
                blue: value.thumb_color.blue,
                alpha: value.thumb_color.alpha,
            },
            minimum_thumb_length: value.minimum_thumb_length,
            needs_update_display: value.needs_update_display,
        }
    }
}

impl Scrollbar<()> for NativeScrollbar {
    fn IsSame(&self, other: &dyn Scrollbar<()>) -> bool {
        std::ptr::eq(self as &dyn Scrollbar<()>, other)
    }
    fn Orientation(&self) -> ScrollbarOrientation {
        self.orientation
    }
    fn IsLeftSideVerticalScrollbar(&self) -> bool {
        self.left_side_vertical
    }
    fn IsSolidColor(&self) -> bool {
        true
    }
    fn IsOverlay(&self) -> bool {
        self.overlay
    }
    fn IsRunningWebTest(&self) -> bool {
        false
    }
    fn IsFluentOverlayScrollbarMinimalMode(&self) -> bool {
        false
    }
    fn HasThumb(&self) -> bool {
        self.thumb.width > 0 && self.thumb.height > 0
    }
    fn SupportsDragSnapBack(&self) -> bool {
        true
    }
    fn JumpOnTrackClick(&self) -> bool {
        false
    }
    fn IsOpaque(&self) -> bool {
        !self.overlay && self.thumb_color.alpha >= 1.0
    }
    fn MinimumThumbLength(&self) -> i32 {
        self.minimum_thumb_length
    }
    fn ThumbRect(&self) -> Rect {
        self.thumb
    }
    fn TrackRect(&self) -> Rect {
        self.track
    }
    fn BackButtonRect(&self) -> Rect {
        Rect::default()
    }
    fn ForwardButtonRect(&self) -> Rect {
        Rect::default()
    }
    fn Opacity(&self) -> f32 {
        1.0
    }
    fn HasTickmarks(&self) -> bool {
        false
    }
    fn ThumbNeedsRepaint(&self) -> bool {
        self.needs_update_display
    }
    fn ClearThumbNeedsRepaint(&mut self) {
        self.needs_update_display = false;
    }
    fn TrackAndButtonsNeedRepaint(&self) -> bool {
        false
    }
    fn PaintThumb(&mut self, _canvas: &mut (), _rect: &Rect) {}
    fn PaintTrackAndButtons(&mut self, _canvas: &mut (), _rect: &Rect) {}
    fn ThumbColor(&self) -> Color4f {
        self.thumb_color
    }
    fn NeedsUpdateDisplay(&self) -> bool {
        self.needs_update_display
    }
    fn ClearNeedsUpdateDisplay(&mut self) {
        self.needs_update_display = false;
    }
    fn UsesNinePatchThumbResource(&self) -> bool {
        false
    }
    fn NinePatchThumbCanvasSize(&self) -> Size {
        Size::default()
    }
    fn NinePatchThumbAperture(&self) -> Rect {
        Rect::default()
    }
    fn UsesSolidColorThumb(&self) -> bool {
        true
    }
    fn SolidColorThumbInsets(&self) -> Insets {
        Insets::default()
    }
    fn UsesNinePatchTrackAndButtonsResource(&self) -> bool {
        false
    }
    fn NinePatchTrackAndButtonsCanvasSize(&self, _scale: f32) -> Size {
        Size::default()
    }
    fn NinePatchTrackAndButtonsAperture(&self, _scale: f32) -> Rect {
        Rect::default()
    }
    fn ShrinkMainThreadedMinimalModeThumbRect(&self, rect: &mut Rect) -> Rect {
        *rect
    }
}

use crate::{
    adapter::{
        geometry::{Rect, Size},
        layer_tree_impl::Color4f,
        scrollbar::Insets,
    },
    cc::input::scrollbar::{Scrollbar as ScrollbarContract, ScrollbarOrientation},
};

/// The retained Blink ScrollbarDisplayItem inputs consumed by cc's native
/// solid-color scrollbar layer. Geometry remains relative to the layer, as in
/// Chromium's WebScrollbar adapter.
#[derive(Clone)]
pub struct Scrollbar {
    pub element_id: crate::adapter::input_types::ElementId,
    orientation: ScrollbarOrientation,
    left_side_vertical: bool,
    overlay: bool,
    track: Rect,
    thumb: Rect,
    thumb_color: Color4f,
    painted_opacity: f32,
    painted_thumb_width: i32,
    is_solid_color: bool,
    uses_macos_painted_scrollbars: bool,
    minimum_thumb_length: i32,
    needs_update_display: bool,
}

impl Scrollbar {
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
            painted_opacity: value.painted_opacity,
            painted_thumb_width: value.painted_thumb_width,
            is_solid_color: value.is_solid_color,
            uses_macos_painted_scrollbars: value.uses_macos_painted_scrollbars,
            minimum_thumb_length: value.minimum_thumb_length,
            needs_update_display: value.needs_update_display,
        }
    }
}

/// Transparent N32-premultiplied bitmap used by PaintedScrollbarLayer's
/// source-shaped RasterizeScrollbarPart operation. The byte order is native
/// little-endian BGRA, matching Skia's N32 upload on the supported mac host.
pub struct ScrollbarCanvas {
    logical_size: Size,
    pixel_size: Size,
    pixels: Vec<u8>,
}

impl ScrollbarCanvas {
    pub fn new(logical_size: Size, pixel_size: Size) -> Self {
        let len = pixel_size.width().max(0) as usize * pixel_size.height().max(0) as usize * 4;
        Self {
            logical_size,
            pixel_size,
            pixels: vec![0; len],
        }
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn pixel_size(&self) -> Size {
        self.pixel_size
    }

    pub fn row_bytes(&self) -> usize {
        self.pixel_size.width().max(0) as usize * 4
    }

    // Source: ui/native_theme/native_theme_mac.mm:142-181. Coverage sampling
    // stands in for Skia's antialiasing while retaining the same rounded bounds.
    fn DrawMacScrollbarThumb(
        &mut self,
        orientation: ScrollbarOrientation,
        left_side_vertical: bool,
        track_box_width: i32,
    ) {
        let logical_w = self.logical_size.width().max(0) as f32;
        let logical_h = self.logical_size.height().max(0) as f32;
        if logical_w == 0.0 || logical_h == 0.0 || track_box_width <= 0 {
            return;
        }
        let track = track_box_width as f32;
        let (mut x, mut y, mut w, mut h) = match orientation {
            ScrollbarOrientation::kHorizontal => (0.0, logical_h - track, logical_w, track),
            ScrollbarOrientation::kVertical if left_side_vertical => (0.0, 0.0, track, logical_h),
            ScrollbarOrientation::kVertical => (logical_w - track, 0.0, track, logical_h),
        };
        let base = 2.0;
        let (top, left, bottom, right) = match orientation {
            ScrollbarOrientation::kHorizontal => (base + 1.0, base, base, base),
            ScrollbarOrientation::kVertical => (base, base + 1.0, base, base),
        };
        x += left;
        y += top;
        w = (w - left - right).max(0.0);
        h = (h - top - bottom).max(0.0);
        if w == 0.0 || h == 0.0 {
            return;
        }
        let radius = w.min(h) * 0.5;
        let sx = self.pixel_size.width() as f32 / logical_w;
        let sy = self.pixel_size.height() as f32 / logical_h;
        let samples = [
            (0.125f32, 0.125f32),
            (0.375, 0.125),
            (0.625, 0.125),
            (0.875, 0.125),
            (0.125, 0.375),
            (0.375, 0.375),
            (0.625, 0.375),
            (0.875, 0.375),
            (0.125, 0.625),
            (0.375, 0.625),
            (0.625, 0.625),
            (0.875, 0.625),
            (0.125, 0.875),
            (0.375, 0.875),
            (0.625, 0.875),
            (0.875, 0.875),
        ];
        let cx0 = x + radius;
        let cx1 = x + w - radius;
        let cy0 = y + radius;
        let cy1 = y + h - radius;
        for py in 0..self.pixel_size.height().max(0) {
            for px in 0..self.pixel_size.width().max(0) {
                let mut covered = 0u8;
                for (ox, oy) in samples {
                    let lx = (px as f32 + ox) / sx;
                    let ly = (py as f32 + oy) / sy;
                    let qx = lx.clamp(cx0, cx1);
                    let qy = ly.clamp(cy0, cy1);
                    if (lx - qx) * (lx - qx) + (ly - qy) * (ly - qy) <= radius * radius {
                        covered += 1;
                    }
                }
                if covered != 0 {
                    let alpha = ((128u16 * covered as u16 + 8) / 16) as u8;
                    let offset = (py as usize * self.pixel_size.width() as usize + px as usize) * 4;
                    self.pixels[offset + 3] = alpha;
                }
            }
        }
    }
}

impl ScrollbarContract<ScrollbarCanvas> for Scrollbar {
    fn IsSame(&self, other: &dyn ScrollbarContract<ScrollbarCanvas>) -> bool {
        std::ptr::eq(self as &dyn ScrollbarContract<ScrollbarCanvas>, other)
    }
    fn Orientation(&self) -> ScrollbarOrientation {
        self.orientation
    }
    fn IsLeftSideVerticalScrollbar(&self) -> bool {
        self.left_side_vertical
    }
    fn IsSolidColor(&self) -> bool {
        self.is_solid_color
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
        self.painted_opacity
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
    fn PaintThumb(&mut self, canvas: &mut ScrollbarCanvas, _rect: &Rect) {
        if self.uses_macos_painted_scrollbars {
            canvas.DrawMacScrollbarThumb(
                self.orientation,
                self.left_side_vertical,
                self.painted_thumb_width,
            );
        }
        self.needs_update_display = false;
    }
    fn PaintTrackAndButtons(&mut self, _canvas: &mut ScrollbarCanvas, _rect: &Rect) {}
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
        self.is_solid_color
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mac_vertical_thumb_matches_native_theme_insets_at_two_x() {
        let mut canvas = ScrollbarCanvas::new(Size::new(16, 40), Size::new(32, 80));
        canvas.DrawMacScrollbarThumb(ScrollbarOrientation::kVertical, false, 12);

        let row = 40usize;
        let visible_columns = (0..32usize)
            .filter(|&x| canvas.pixels[(row * 32 + x) * 4 + 3] != 0)
            .collect::<Vec<_>>();
        assert_eq!(visible_columns, (14..28).collect::<Vec<_>>());
        assert_eq!(canvas.pixels[(row * 32 + 21) * 4 + 3], 128);
        assert_eq!(canvas.pixels[3], 0);
    }
}

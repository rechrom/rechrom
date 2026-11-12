//! VisualViewport/ChromeClient coordinate leaves used by nonlocated menus.
//! Chromium visual_viewport.cc:1062-1087 and chrome_client_impl.cc:609-636;
//! widget_base.cc:2059-2062. These small values are supplied by the original
//! FrameView/VisualViewport/widget owners for the duration of one query.
use foundation::gfx;

/// Native widget visual-properties input, independent of DOM or selection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeContextMenuWidgetGeometry {
    pub screen_origin_dip: gfx::Point,
    pub blink_to_dip_scale: f32,
}

#[derive(Clone, Copy)]
pub struct NativeContextMenuFrameCoordinates {
    pub visual_viewport_offset: gfx::PointF,
    pub visual_viewport_scale: f32,
    pub visual_viewport_size: gfx::Size,
    /// Widget::ViewRect origin in screen DIPs, supplied by the native host.
    pub widget_screen_origin_dip: gfx::Point,
    /// WidgetBase's reverse original device scale factor. The embedding must
    /// express the actual coordinate contract, not infer this from CSS zoom.
    pub blink_to_dip_scale: f32,
    pub outermost_main_frame: bool,
}
impl NativeContextMenuFrameCoordinates {
    pub fn IsValid(&self) -> bool {
        self.visual_viewport_scale.is_finite()
            && self.visual_viewport_scale > 0.0
            && self.blink_to_dip_scale.is_finite()
            && self.blink_to_dip_scale > 0.0
            && self.visual_viewport_offset.x().is_finite()
            && self.visual_viewport_offset.y().is_finite()
    }
    pub fn VisualViewportSize(&self) -> gfx::Size {
        self.visual_viewport_size
    }
    pub fn VisualViewportScrollOffset(&self) -> gfx::PointF {
        self.visual_viewport_offset
    }
    pub fn RootFrameToViewport(&self, rect: gfx::Rect) -> gfx::Rect {
        let scale = self.visual_viewport_scale;
        gfx::ToEnclosingRect(gfx::RectF::new(
            gfx::PointF::new(
                (rect.x() as f32 - self.visual_viewport_offset.x()) * scale,
                (rect.y() as f32 - self.visual_viewport_offset.y()) * scale,
            ),
            gfx::SizeF::new(rect.width() as f32 * scale, rect.height() as f32 * scale),
        ))
    }
    pub fn ViewportToRootFrame(&self, rect: gfx::Rect) -> gfx::Rect {
        let scale = self.visual_viewport_scale;
        gfx::ToEnclosingRect(gfx::RectF::new(
            gfx::PointF::new(
                rect.x() as f32 / scale + self.visual_viewport_offset.x(),
                rect.y() as f32 / scale + self.visual_viewport_offset.y(),
            ),
            gfx::SizeF::new(rect.width() as f32 / scale, rect.height() as f32 / scale),
        ))
    }
    /// LocalFrameView::FrameToViewport, local_frame_view.cc:4349-4353. The
    /// caller first uses that frame's real ConvertToRootFrame rect leaf.
    pub fn FrameRectToWidget(&self, root_rect: gfx::Rect) -> gfx::Rect {
        if self.outermost_main_frame {
            self.RootFrameToViewport(root_rect)
        } else {
            root_rect
        }
    }
    pub fn RootFramePointToScreenDIPs(&self, point: gfx::Point) -> gfx::Point {
        self.RootFrameRectToScreenDIPs(gfx::Rect::new(point, gfx::Size::default()))
            .origin()
    }
    /// LocalFrameView::FrameToScreen preserves the source rectangle through
    /// enclosing/enclosed viewport/widget rounding before reading its origin.
    pub fn RootFrameRectToScreenDIPs(&self, root_rect: gfx::Rect) -> gfx::Rect {
        let widget_rect = self.FrameRectToWidget(root_rect);
        let screen_rect = gfx::ScaleToEnclosedRect(widget_rect, self.blink_to_dip_scale);
        gfx::Rect::new(
            gfx::Point::new(
                screen_rect
                    .x()
                    .saturating_add(self.widget_screen_origin_dip.x()),
                screen_rect
                    .y()
                    .saturating_add(self.widget_screen_origin_dip.y()),
            ),
            screen_rect.size(),
        )
    }
}

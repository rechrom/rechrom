//! Source: third_party/blink/renderer/core/exported/web_view_impl.h:894-901;
//! web_view_impl.cc:219-228,671-718,725-904,982-1025.
//! WebView's retained double-tap state and scale/scroll calculations. The
//! embedding supplies native viewport facts and block bounds, not decisions.
use foundation::gfx::{Point, Rect, Size, Vector2dF};

#[derive(Clone, Copy, Debug)]
pub struct PageScaleSettings {
    pub double_tap_to_zoom_enabled: bool,
    pub minimum_scale: f32,
    pub maximum_scale: f32,
    pub maximum_legible_scale: f32,
    pub accessibility_font_scale: f32,
    pub wide_viewport_quirk_enabled: bool,
    pub text_scale_meta_tag_present: bool,
    pub align_zoom_to_center: bool,
}
impl Default for PageScaleSettings {
    fn default() -> Self {
        Self {
            double_tap_to_zoom_enabled: false,
            minimum_scale: 1.,
            maximum_scale: 1.,
            maximum_legible_scale: 1.,
            accessibility_font_scale: 1.,
            wide_viewport_quirk_enabled: false,
            text_scale_meta_tag_present: false,
            align_zoom_to_center: false,
        }
    }
}
impl PageScaleSettings {
    pub fn IsValid(self) -> bool {
        self.minimum_scale.is_finite()
            && self.minimum_scale > 0.
            && self.maximum_scale.is_finite()
            && self.maximum_scale >= self.minimum_scale
            && self.maximum_legible_scale.is_finite()
            && self.maximum_legible_scale > 0.
            && self.accessibility_font_scale.is_finite()
            && self.accessibility_font_scale > 0.
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ViewportFacts {
    pub size: Size,
    pub document_size: Size,
    pub layout_scroll_offset: Vector2dF,
    pub visual_scroll_offset: Vector2dF,
    pub layout_viewport_size: Size,
    pub minimum_layout_scroll_offset: foundation::gfx::Vector2d,
    pub maximum_layout_scroll_offset: foundation::gfx::Vector2d,
}
#[derive(Clone, Copy, Debug)]
pub struct PageScaleAnimation {
    pub target_position: Point,
    pub use_anchor: bool,
    pub scale: f32,
    pub duration_milliseconds: i64,
}
#[derive(Debug)]
pub struct WebViewZoom {
    settings: PageScaleSettings,
    scale: f32,
    visual_offset: Vector2dF,
    double_tap_zoom_page_scale_factor: f32,
    double_tap_zoom_pending: bool,
}
impl Default for WebViewZoom {
    fn default() -> Self {
        Self {
            settings: PageScaleSettings::default(),
            scale: 1.,
            visual_offset: Vector2dF::default(),
            double_tap_zoom_page_scale_factor: 0.,
            double_tap_zoom_pending: false,
        }
    }
}
impl WebViewZoom {
    pub fn Settings(&self) -> PageScaleSettings {
        self.settings
    }
    pub fn Scale(&self) -> f32 {
        self.scale
    }
    pub fn VisualOffset(&self) -> Vector2dF {
        self.visual_offset
    }
    /// VisualViewport::MaximumScrollOffset, source visual_viewport.cc.
    pub fn MaximumScrollOffset(&self, facts: &ViewportFacts) -> Vector2dF {
        Vector2dF::new(
            (facts.layout_viewport_size.width() as f32 - facts.size.width() as f32 / self.scale)
                .max(0.0),
            (facts.layout_viewport_size.height() as f32 - facts.size.height() as f32 / self.scale)
                .max(0.0),
        )
    }
    /// VisualViewport::VisibleRect, visual_viewport.cc:440-453.
    pub fn VisibleRect(&self, size_excluding_scrollbars: Size) -> foundation::gfx::RectF {
        foundation::gfx::RectF::new(
            foundation::gfx::PointF::new(self.visual_offset.x(), self.visual_offset.y()),
            foundation::gfx::SizeF::new(
                size_excluding_scrollbars.width() as f32 / self.scale,
                size_excluding_scrollbars.height() as f32 / self.scale,
            ),
        )
    }

    /// VisualViewport::SetScaleAndLocation, visual_viewport.cc:~1070. The
    /// caller supplies this same owner with cc's acknowledged viewport delta.
    pub fn ApplyVisualOffsetDelta(&mut self, delta: Vector2dF) {
        self.visual_offset += delta;
    }
    pub fn ViewportToRootFrame(&self, point: foundation::gfx::PointF) -> foundation::gfx::PointF {
        foundation::gfx::PointF::new(
            point.x() / self.scale + self.visual_offset.x(),
            point.y() / self.scale + self.visual_offset.y(),
        )
    }
    /// VisualViewport::RootFrameToViewport, visual_viewport.cc:1098-1104.
    pub fn RootFrameToViewport(&self, point: foundation::gfx::PointF) -> foundation::gfx::PointF {
        foundation::gfx::PointF::new(
            (point.x() - self.visual_offset.x()) * self.scale,
            (point.y() - self.visual_offset.y()) * self.scale,
        )
    }

    pub fn SetScaleAndSettings(&mut self, scale: f32, settings: PageScaleSettings) {
        assert!(
            settings.IsValid()
                && scale.is_finite()
                && scale >= settings.minimum_scale
                && scale <= settings.maximum_scale
        );
        self.settings = settings;
        self.scale = scale;
    }
    pub fn ApplyScaleDelta(&mut self, delta: f32) {
        assert!(delta.is_finite() && delta > 0.);
        self.scale =
            (self.scale * delta).clamp(self.settings.minimum_scale, self.settings.maximum_scale);
    }
    pub fn DidCompletePageScaleAnimation(&mut self) {
        self.double_tap_zoom_pending = false;
    }
    pub fn DoubleTapEnabled(&self) -> bool {
        self.settings.double_tap_to_zoom_enabled
            && self.settings.minimum_scale != self.settings.maximum_scale
    }
    fn MaximumLegiblePageScale(&self) -> f32 {
        let settings = self.settings;
        if settings.accessibility_font_scale == 1.
            || (settings.wide_viewport_quirk_enabled && !settings.text_scale_meta_tag_present)
        {
            settings.maximum_legible_scale
        } else {
            settings.maximum_legible_scale * settings.accessibility_font_scale
        }
    }
    fn ClampDocumentOffsetAtScale(facts: ViewportFacts, point: Point, scale: f32) -> Point {
        // VisualViewport.cc:892-915. Layout and visual maxima retain their
        // separate source owners and integer rounding.
        let max_x = facts.maximum_layout_scroll_offset.x()
            + (facts.layout_viewport_size.width() as f32 - facts.size.width() as f32 / scale)
                .floor() as i32;
        let max_y = facts.maximum_layout_scroll_offset.y()
            + (facts.layout_viewport_size.height() as f32 - facts.size.height() as f32 / scale)
                .floor() as i32;
        Point::new(
            point
                .x()
                .min(max_x)
                .max(facts.minimum_layout_scroll_offset.x()),
            point
                .y()
                .min(max_y)
                .max(facts.minimum_layout_scroll_offset.y()),
        )
    }
    fn RootFrameToDocument(facts: ViewportFacts, point: Point) -> Point {
        Point::new(
            (point.x() as f32 + facts.layout_scroll_offset.x()) as i32,
            (point.y() as f32 + facts.layout_scroll_offset.y()) as i32,
        )
    }
    fn WidenRectWithinPageBounds(
        facts: ViewportFacts,
        source: Rect,
        target_margin: i32,
        minimum_margin: i32,
    ) -> Rect {
        let mut left = target_margin;
        let mut right = target_margin;
        let absolute_x = (source.x() as f32 + facts.layout_scroll_offset.x()) as i32;
        if left > absolute_x {
            left = absolute_x;
            right = left.max(minimum_margin);
        }
        let maximum_right = facts.document_size.width() - (source.width() + absolute_x);
        if right > maximum_right {
            right = maximum_right;
            left = left.min(right.max(minimum_margin));
        }
        Rect::new(
            Point::new(source.x() - left, source.y()),
            Size::new((source.width() + left + right).max(0), source.height()),
        )
    }
    pub fn ComputeScaleAndScrollForBlockRect(
        &self,
        facts: ViewportFacts,
        hit: Rect,
        mut block: Rect,
        padding: f32,
        default_scale: f32,
    ) -> (f32, Point) {
        let mut scale = self.scale;
        if !block.IsEmpty() {
            block = Self::WidenRectWithinPageBounds(
                facts,
                block,
                (5. * block.width() as f32 / facts.size.width() as f32).floor() as i32,
                (2. * block.width() as f32 / facts.size.width() as f32).floor() as i32,
            );
            scale = (facts.size.width() as f32 / block.width() as f32)
                .min(self.MaximumLegiblePageScale());
            if self.scale < default_scale {
                scale = scale.max(default_scale);
            }
            scale = scale.clamp(self.settings.minimum_scale, self.settings.maximum_scale);
        }
        let width = facts.size.width() as f32 / scale;
        let height = facts.size.height() as f32 / scale;
        let offset = if self.settings.align_zoom_to_center {
            let margin = 20. / scale;
            let hit_width = hit.width().min((width - margin) as i32).max(0);
            let hit_height = hit.height().min((height - margin) as i32).max(0);
            let center_x = if block.width() as f32 <= width {
                block.x() + block.width() / 2
            } else {
                hit.x() + hit_width / 2
            };
            let center_y = if block.height() as f32 <= height {
                block.y() + block.height() / 2
            } else {
                hit.y() + hit_height / 2
            };
            Point::new(
                center_x - (width * 0.5) as i32,
                center_y - (height * 0.5) as i32,
            )
        } else {
            let y = if (block.height() as f32) < height {
                block.y() + (-0.5 * (height - block.height() as f32)) as i32
            } else {
                (block.y() as f32).max(hit.y() as f32 + padding - height) as i32
            };
            let x = if (block.width() as f32) < width {
                block.x() + (-0.5 * (width - block.width() as f32)) as i32
            } else {
                (block.x() as f32).max(hit.x() as f32 + padding - width) as i32
            };
            Point::new(x, y)
        };
        let offset = Self::RootFrameToDocument(facts, offset);
        (
            scale,
            Self::ClampDocumentOffsetAtScale(facts, offset, scale),
        )
    }
    pub fn AnimateDoubleTapZoom(
        &mut self,
        point: Point,
        block: Rect,
        facts: ViewportFacts,
    ) -> Option<PageScaleAnimation> {
        if !self.DoubleTapEnabled() || facts.size.IsEmpty() {
            return None;
        }
        let (mut scale, scroll) = self.ComputeScaleAndScrollForBlockRect(
            facts,
            Rect::new(point, Size::new(1, 1)),
            block,
            32.,
            self.settings.minimum_scale * 1.2,
        );
        let previous = (self.scale == self.double_tap_zoom_page_scale_factor
            && self.double_tap_zoom_page_scale_factor != self.settings.minimum_scale)
            || self.double_tap_zoom_pending;
        let zoom_out = block.IsEmpty() || (self.scale - scale).abs() < 0.01 || previous;
        let target = if zoom_out {
            scale = self.settings.minimum_scale;
            Self::RootFrameToDocument(facts, point)
        } else {
            scroll
        };
        if zoom_out && scale == self.scale {
            return None;
        }
        self.double_tap_zoom_page_scale_factor = scale;
        self.double_tap_zoom_pending = true;
        Some(PageScaleAnimation {
            target_position: target,
            use_anchor: zoom_out,
            scale,
            duration_milliseconds: 250,
        })
    }
}

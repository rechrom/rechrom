//! Identity and ownership portion of Blink core/scroll/scrollbar.{h,cc}.
//! Theme and per-box geometry stay in the neighboring fragment owners.

#![allow(non_snake_case)]

use super::super::layout_input_types::ScrollbarThemeMetrics;
use super::super::layout_object::LayoutObject;
use super::super::layout_scrollable_area::PaintLayerScrollableArea;
use super::super::scrollbar_orientation::ScrollbarOrientation;
use foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient;
use foundation::{gfx, EScrollbarWidth, Member, PaintInvalidationReason, Traceable, Visitor};

#[repr(C)]
pub struct Scrollbar {
    client: DisplayItemClient,
    scrollable_area: Member<PaintLayerScrollableArea>,
    orientation: ScrollbarOrientation,
    style_source: Member<LayoutObject>,
    is_custom_scrollbar: bool,
    is_overlay_scrollbar: bool,
    theme_metrics: ScrollbarThemeMetrics,
    css_scrollbar_width: EScrollbarWidth,
    enabled: bool,
    thickness: i32,
    visible_size: i32,
    total_size: i32,
    current_pos: f32,
    frame_rect: gfx::Rect,
}

impl Scrollbar {
    // Blink Scrollbar::Scrollbar, core/scroll/scrollbar.cc:74-102; its
    // DisplayItemClient base owns the identity, never the containing LayoutBox
    // or an exported index.
    pub fn new(
        area: *mut PaintLayerScrollableArea,
        orientation: ScrollbarOrientation,
        style_source: *mut LayoutObject,
        is_custom_scrollbar: bool,
        is_overlay_scrollbar: bool,
        theme_metrics: ScrollbarThemeMetrics,
        css_scrollbar_width: EScrollbarWidth,
        thickness: i32,
        current_pos: f32,
    ) -> Self {
        Self {
            client: Default::default(),
            scrollable_area: Member::from_ptr(area),
            orientation,
            style_source: Member::from_ptr(style_source),
            is_custom_scrollbar,
            is_overlay_scrollbar,
            theme_metrics,
            css_scrollbar_width,
            // Source: core/scroll/scrollbar.cc:91.
            enabled: true,
            thickness,
            visible_size: 0,
            total_size: 0,
            current_pos,
            frame_rect: gfx::Rect::default(),
        }
    }

    pub fn DisplayItemClient(&self) -> &DisplayItemClient {
        &self.client
    }

    // Source: core/scroll/scrollbar.h:90.
    pub fn IsCustomScrollbar(&self) -> bool {
        self.is_custom_scrollbar
    }

    // Source: core/scroll/scrollbar.h:103.
    pub fn StyleSource(&self) -> *mut LayoutObject {
        self.style_source.Get()
    }

    // The source package carries the immutable theme input snapshot in place
    // of Blink's Page-owned ScrollbarTheme identity.
    pub fn ThemeMetrics(&self) -> &ScrollbarThemeMetrics {
        &self.theme_metrics
    }

    // Source: core/scroll/scrollbar.h:96.
    pub fn CSSScrollbarWidth(&self) -> EScrollbarWidth {
        self.css_scrollbar_width
    }

    // Source: core/scroll/scrollbar.cc:768-770.
    pub fn IsOverlayScrollbar(&self) -> bool {
        self.is_overlay_scrollbar
    }

    // Source: core/scroll/scrollbar.cc:948-959 and
    // core/layout/custom_scrollbar.h:74. Custom scrollbars conservatively
    // override the base result to false; represented non-custom platform
    // scrollbars have no CSS track color and their non-overlay theme is opaque.
    pub fn IsOpaque(&self) -> bool {
        !self.IsOverlayScrollbar() && !self.IsCustomScrollbar()
    }

    // Source: core/scroll/scrollbar.h:108 and scrollbar.cc:744-755. Theme
    // callbacks and per-part invalidation stay on the Paint side of this
    // extracted package; the retained client receives the equivalent full
    // scroll-control invalidation when the source state changes.
    pub fn Enabled(&self) -> bool {
        self.enabled
    }

    pub fn SetEnabled(&mut self, enabled: bool) {
        if self.enabled == enabled {
            return;
        }
        self.enabled = enabled;
        self.client
            .Invalidate(PaintInvalidationReason::kScrollControl);
    }

    // Source: core/scroll/scrollbar.h:107 and
    // core/layout/custom_scrollbar.cc:93-95. The base implementation is a
    // no-op; the represented custom scrollbar conservatively invalidates its
    // retained client because its shadow parts are rebuilt by Chromium.
    pub fn StyleChanged(&mut self) {
        if self.IsCustomScrollbar() {
            self.client
                .Invalidate(PaintInvalidationReason::kScrollControl);
        }
    }

    // Source: core/scroll/scrollbar.cc:796-802.  Overlay visibility belongs to
    // the retained ScrollableArea owner, not to Paint's clipping policy.
    pub fn ShouldParticipateInHitTesting(&self) -> bool {
        let area = self.scrollable_area.Get();
        assert!(!area.is_null());
        if !self.IsOverlayScrollbar() {
            return true;
        }
        !unsafe { &*area }.ScrollbarsHiddenIfOverlay()
    }

    // Source: core/scroll/scrollbar.cc:757-762.
    pub fn ScrollbarThickness(&self) -> i32 {
        self.thickness
    }

    // Source: core/scroll/scrollbar.h:83-85. Geometry is positioned by the
    // owning PaintLayerScrollableArea and consumed unchanged by Paint.
    pub fn FrameRect(&self) -> &gfx::Rect {
        &self.frame_rect
    }

    pub fn SetFrameRect(&mut self, rect: &gfx::Rect) {
        if self.frame_rect == *rect {
            return;
        }
        // Source: core/scroll/scrollbar.cc:114-123. Nine-patch repaint
        // elision is unavailable, so retain the source's conservative path.
        self.client
            .Invalidate(PaintInvalidationReason::kScrollControl);
        self.frame_rect = *rect;
        let area = self.scrollable_area.Get();
        if !area.is_null() {
            unsafe { &mut *area }.ScrollbarFrameRectChanged();
        }
    }

    // Source: core/scroll/scrollbar.h:97-100.
    pub fn CurrentPos(&self) -> f32 {
        self.current_pos
    }

    pub fn VisibleSize(&self) -> i32 {
        self.visible_size
    }

    pub fn TotalSize(&self) -> i32 {
        self.total_size
    }

    // Source: core/scroll/scrollbar.cc:177-201. Pressed-part correction and
    // theme invalidation remain in Paint/Interaction; this owner retains the
    // source position that drives immutable thumb geometry.
    pub fn OffsetDidChange(&mut self, position: f32) {
        if position != self.current_pos {
            self.current_pos = position;
        }
    }

    // Source: core/scroll/scrollbar.cc:207-220. Paint invalidation is derived
    // from the immutable geometry revision at the package boundary.
    pub fn SetProportion(&mut self, visible_size: i32, total_size: i32) {
        if visible_size == self.visible_size && total_size == self.total_size {
            return;
        }
        self.visible_size = visible_size;
        self.total_size = total_size;
    }

    pub fn DisconnectFromScrollableArea(&mut self) {
        self.scrollable_area = Member::default();
    }
}

impl Traceable for Scrollbar {
    fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.scrollable_area);
        visitor.Trace(&self.style_source);
        self.client.Trace(visitor);
    }
}

// Blink PaintLayerScrollableArea::ScrollCornerDisplayItemClient, .h:947.
#[repr(C)]
pub struct ScrollCornerDisplayItemClient {
    client: DisplayItemClient,
    area: Member<PaintLayerScrollableArea>,
}

impl ScrollCornerDisplayItemClient {
    pub fn new(area: *mut PaintLayerScrollableArea) -> Self {
        Self {
            client: Default::default(),
            area: Member::from_ptr(area),
        }
    }

    pub fn DisplayItemClient(&self) -> &DisplayItemClient {
        &self.client
    }
}

impl Traceable for ScrollCornerDisplayItemClient {
    fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.area);
        self.client.Trace(visitor);
    }
}

// Blink PaintLayerScrollableArea::ScrollingBackgroundDisplayItemClient,
// paint_layer_scrollable_area.h:930-946.
#[repr(C)]
pub struct ScrollingBackgroundDisplayItemClient {
    client: DisplayItemClient,
    area: Member<PaintLayerScrollableArea>,
}

impl ScrollingBackgroundDisplayItemClient {
    pub fn new(area: *mut PaintLayerScrollableArea) -> Self {
        Self {
            client: Default::default(),
            area: Member::from_ptr(area),
        }
    }

    pub fn DisplayItemClient(&self) -> &DisplayItemClient {
        &self.client
    }
}

impl Traceable for ScrollingBackgroundDisplayItemClient {
    fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.area);
        self.client.Trace(visitor);
    }
}

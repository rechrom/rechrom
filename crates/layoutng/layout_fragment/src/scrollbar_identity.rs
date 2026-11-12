//! Identity and ownership portion of Blink core/scroll/scrollbar.{h,cc}.
//! Theme and per-box geometry stay in the neighboring fragment owners.

#![allow(non_snake_case)]

use super::super::layout_input_types::ScrollbarThemeMetrics;
use super::super::layout_object::LayoutObject;
use super::super::layout_scrollable_area::PaintLayerScrollableArea;
use super::super::scrollbar_orientation::ScrollbarOrientation;
use foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient;
use foundation::{gfx, EScrollbarWidth, Member, PaintInvalidationReason, Traceable, Visitor};
use std::cell::Cell;

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
    enabled: Cell<bool>,
    thickness: Cell<i32>,
    visible_size: i32,
    total_size: i32,
    current_pos: f32,
    hovered_part: Cell<super::super::scrollbar_part::ScrollbarPart>,
    pressed_part: Cell<super::super::scrollbar_part::ScrollbarPart>,
    input_state: foundation::scrollbar_input_state::ScrollbarInputState,
    custom_style_update_pending: Cell<bool>,
    custom: Cell<Member<super::super::custom_scrollbar::CustomScrollbar>>,
    frame_rect: Cell<gfx::Rect>,
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
            enabled: Cell::new(true),
            thickness: Cell::new(thickness),
            visible_size: 0,
            total_size: 0,
            current_pos,
            // Source: Scrollbar::Scrollbar initializes both input owners to kNoPart.
            hovered_part: Cell::new(super::super::scrollbar_part::ScrollbarPart::kNoPart),
            pressed_part: Cell::new(super::super::scrollbar_part::ScrollbarPart::kNoPart),
            input_state: Default::default(),
            custom_style_update_pending: Cell::new(is_custom_scrollbar),
            custom: Cell::new(Member::default()),
            frame_rect: Cell::new(gfx::Rect::default()),
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
        self.enabled.get()
    }

    pub fn SetEnabled(&self, enabled: bool) {
        if self.enabled.get() == enabled {
            return;
        }
        self.enabled.set(enabled);
        self.StyleChanged();
        self.client
            .Invalidate(PaintInvalidationReason::kScrollControl);
    }

    // Source: core/scroll/scrollbar.h:107 and
    // core/layout/custom_scrollbar.cc:93-95. The base implementation is a
    // no-op; the represented custom scrollbar conservatively invalidates its
    // retained client because its shadow parts are rebuilt by Chromium.
    pub fn StyleChanged(&self) {
        if self.IsCustomScrollbar() {
            self.custom_style_update_pending.set(true);
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
        self.thickness.get()
    }

    // Source: core/scroll/scrollbar.h:83-85. Geometry is positioned by the
    // owning PaintLayerScrollableArea and consumed unchanged by Paint.
    pub fn FrameRect(&self) -> gfx::Rect {
        self.frame_rect.get()
    }

    pub fn SetFrameRect(&self, rect: &gfx::Rect) {
        if self.frame_rect.get() == *rect {
            return;
        }
        // Source: core/scroll/scrollbar.cc:114-123. Nine-patch repaint
        // elision is unavailable, so retain the source's conservative path.
        self.client
            .Invalidate(PaintInvalidationReason::kScrollControl);
        self.frame_rect.set(*rect);
        // CustomScrollbar's extracted base geometry observes this same frame
        // mutation before part used-value/position queries.
        if let Some(custom) = unsafe { self.custom.get().Get().as_mut() } {
            custom.SetFrameRect(rect);
        }
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
        self.OffsetDidChangeWithType(
            position,
            layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kProgrammatic,
        );
    }
    // Source scrollbar.cc:177-201: offset acknowledgement clears speculative
    // injection and keeps the pressed point aligned with the moving thumb.
    pub fn OffsetDidChangeWithType(
        &mut self,
        position: f32,
        scroll_type: layoutng_style::style::scroll_enums::mojom::blink::ScrollType,
    ) {
        self.input_state
            .pending_injected_delta
            .set(gfx::Vector2dF::default());
        if position == self.current_pos {
            return;
        }
        let old_thumb = self.ThemeGeometryAt(self.current_pos).thumb_position;
        self.current_pos = position;
        self.client
            .Invalidate(PaintInvalidationReason::kScrollControl);
        if self.PressedPart() == super::super::scrollbar_part::ScrollbarPart::kThumbPart
            && scroll_type
                != layoutng_style::style::scroll_enums::mojom::blink::ScrollType::kAnchoring
        {
            let new_thumb = self.ThemeGeometryAt(position).thumb_position;
            self.input_state
                .pressed_pos
                .set(self.input_state.pressed_pos.get() + new_thumb - old_thumb);
        }
        // cpp: CustomScrollbar::OffsetDidChange, custom_scrollbar.cc:372-375.
        self.PositionCustomScrollbarParts();
    }
    pub fn InputState(&self) -> &foundation::scrollbar_input_state::ScrollbarInputState {
        &self.input_state
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
        visitor.Trace(&self.custom.get());
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

impl Scrollbar {
    // Source: core/scroll/scrollbar.h:104,122; retained on this native owner.
    pub fn HoveredPart(&self) -> super::super::scrollbar_part::ScrollbarPart {
        self.hovered_part.get()
    }
    pub fn SetHoveredPart(&self, part: super::super::scrollbar_part::ScrollbarPart) -> bool {
        // CustomScrollbar::SetHoveredPart, custom_scrollbar.cc:97-116.
        if self.IsCustomScrollbar() && self.scrollable_area.Get().is_null() {
            return false;
        }
        if self.hovered_part.get() == part {
            return false;
        }
        // Scrollbar::SetHoveredPart, scrollbar.cc:358-370. A pressed part
        // suppresses the native hover state; custom pseudo styles still update.
        let invalidated =
            self.pressed_part.get() == super::super::scrollbar_part::ScrollbarPart::kNoPart;
        if invalidated {
            self.InvalidatePaintForInput();
        }
        self.hovered_part.set(part);
        self.StyleChanged();
        invalidated || self.IsCustomScrollbar()
    }
    pub fn PressedPart(&self) -> super::super::scrollbar_part::ScrollbarPart {
        self.pressed_part.get()
    }
    /// The input owner supplies the actual pressed-part transition. No cursor
    /// or Style query infers pressed state from hover or DOM :active state.
    // cpp: scrollbar.cc:371-385; custom_scrollbar.cc:118-134.
    pub fn SetPressedPart(&self, part: super::super::scrollbar_part::ScrollbarPart) -> bool {
        if self.IsCustomScrollbar() && self.scrollable_area.Get().is_null() {
            return false;
        }
        let previous = self.pressed_part.get();
        let invalidated = previous != super::super::scrollbar_part::ScrollbarPart::kNoPart
            || self.HoveredPart() != super::super::scrollbar_part::ScrollbarPart::kNoPart;
        if invalidated {
            self.InvalidatePaintForInput();
        }
        if self.pressed_part.replace(part) != part {
            self.StyleChanged();
        }
        invalidated || (self.IsCustomScrollbar() && previous != part)
    }
    pub fn Orientation(&self) -> ScrollbarOrientation {
        self.orientation
    }
    // cpp: scrollbar.cc:974-976. Read the same ScrollableArea owner.
    pub fn IsScrollCornerVisible(&self) -> bool {
        unsafe { self.scrollable_area.Get().as_ref() }
            .is_some_and(|area| area.IsScrollCornerVisible())
    }
    pub fn IsConnectedToScrollableArea(&self) -> bool {
        !self.scrollable_area.Get().is_null()
    }
    pub fn ScrollableArea(&self) -> *mut PaintLayerScrollableArea {
        self.scrollable_area.Get()
    }
    pub fn CustomPartStyleUpdatePending(&self) -> bool {
        self.custom_style_update_pending.get()
    }
    pub fn DidUpdateCustomPartStyles(&self) {
        self.custom_style_update_pending.set(false);
    }

    /// Completes the nine same-owner style publications. Layout owns all
    /// thickness/fragment decisions; Page only observes the retry effect.
    // cpp: custom_scrollbar.cc:181-215.
    pub fn FinishCustomPartStyles(&self) -> bool {
        let Some(custom) = (unsafe { self.custom.get().Get().as_mut() }) else {
            return false;
        };
        let horizontal = self.orientation == ScrollbarOrientation::kHorizontalScrollbar;
        let frame = self.frame_rect.get();
        let old_thickness = if horizontal {
            frame.height()
        } else {
            frame.width()
        };
        let new_thickness = custom.ComputeThickness();
        let published_thickness_changed = self.thickness.replace(new_thickness) != new_thickness;
        if new_thickness != old_thickness {
            self.SetFrameRect(&gfx::Rect::new(
                frame.origin(),
                gfx::Size::new(
                    if horizontal {
                        frame.width()
                    } else {
                        new_thickness
                    },
                    if horizontal {
                        new_thickness
                    } else {
                        frame.height()
                    },
                ),
            ));
            if let Some(area) = unsafe { self.scrollable_area.Get().as_mut() } {
                area.CustomScrollbarThicknessChanged();
            }
            return true;
        }
        // A staged hypothetical request can resolve display:none while its
        // empty frame is already zero-sized. Its provisional exposed thickness
        // still changed and must not escape the publication retry.
        if published_thickness_changed {
            if let Some(area) = unsafe { self.scrollable_area.Get().as_mut() } {
                area.CustomScrollbarThicknessChanged();
            }
            return true;
        }
        if let Some(area) = unsafe { self.scrollable_area.Get().as_ref() } {
            let box_ = unsafe { &*area.GetLayoutBox() };
            if box_.IsLayoutView() || !box_.Parent().is_null() {
                custom.PositionScrollbarParts(
                    self.enabled.get(),
                    self.visible_size,
                    self.total_size,
                    self.current_pos,
                );
            }
        }
        false
    }
    pub fn PositionCustomScrollbarParts(&self) {
        if let Some(custom) = unsafe { self.custom.get().Get().as_mut() } {
            custom.PositionScrollbarParts(
                self.enabled.get(),
                self.visible_size,
                self.total_size,
                self.current_pos,
            );
        }
    }
    pub fn CustomPartRect(
        &self,
        part: super::super::scrollbar_part::ScrollbarPart,
    ) -> Option<gfx::Rect> {
        let custom = unsafe { self.custom.get().Get().as_ref() }?;
        Some(custom.PartRect(
            part,
            self.enabled.get(),
            self.visible_size,
            self.total_size,
            self.current_pos,
        ))
    }

    pub fn GetScrollbarPartStyleForCursor(
        &self,
        part: u32,
    ) -> Option<&layoutng_style::style::computed_style::ComputedStyle> {
        use super::super::scrollbar_part::ScrollbarPart as P;
        let part = match part {
            0 => P::kNoPart,
            1 => P::kBackButtonStartPart,
            2 => P::kForwardButtonStartPart,
            4 => P::kBackTrackPart,
            8 => P::kThumbPart,
            16 => P::kForwardTrackPart,
            32 => P::kBackButtonEndPart,
            64 => P::kForwardButtonEndPart,
            128 => P::kScrollbarBGPart,
            256 => P::kTrackBGPart,
            _ => return None,
        };
        unsafe { self.custom.get().Get().as_ref() }?.GetScrollbarPartStyleForCursor(part)
    }
    /// Source: core/layout/custom_scrollbar.cc:92-133,155-159. Style resolves
    /// each native part request with its original scrollbar state, then sends
    /// that exact owner here. A general pseudo-element cache is insufficient:
    /// hover/pressed/orientation and individual button/track parts can differ.
    pub fn SetCustomPartStyle(
        &self,
        part: super::super::scrollbar_part::ScrollbarPart,
        style: *const layoutng_style::style::computed_style::ComputedStyle,
    ) {
        assert!(self.IsCustomScrollbar());
        if self.custom.get().Get().is_null() {
            self.custom
                .set(Member::from_ptr(foundation::MakeGarbageCollected(
                    super::super::custom_scrollbar::CustomScrollbar::new(
                        self.orientation,
                        &self.frame_rect.get(),
                        self.theme_metrics,
                    ),
                )));
        }
        unsafe { &mut *self.custom.get().Get() }.SetPartStyle(part, style);
    }
}

#[path = "scrollbar_input_geometry.rs"]
mod input_geometry;
pub use input_geometry::{NativeScrollbarGeometry, NativeScrollbarScrollState};

impl Scrollbar {
    /// Scrollbar::SetNeedsPaintInvalidation, scrollbar.cc:858-871. The retained
    /// Paint boundary caches the scrollbar's display item as one native client.
    pub fn InvalidatePaintForInput(&self) {
        self.client
            .Invalidate(PaintInvalidationReason::kScrollControl);
        if let Some(area) = unsafe { self.scrollable_area.Get().as_mut() } {
            area.SetScrollbarNeedsPaintInvalidation(self.Orientation());
        }
    }
}

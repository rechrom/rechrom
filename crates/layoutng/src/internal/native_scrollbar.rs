//! Identity/ownership portion of Blink core/scroll/scrollbar.{h,cc}.
//! Theme geometry still comes from the existing standalone theme adapter.
#![allow(non_snake_case)]
use super::layout_object::LayoutObject;
use super::layout_scrollable_area::PaintLayerScrollableArea;
use super::scrollbar_orientation::ScrollbarOrientation;
use foundation::graphics_types::graphics::paint::display_item_client::DisplayItemClient;
use foundation::{Member, Traceable, Visitor};

#[repr(C)]
pub struct Scrollbar {
    client: DisplayItemClient,
    scrollable_area: Member<PaintLayerScrollableArea>,
    orientation: ScrollbarOrientation,
    style_source: Member<LayoutObject>,
}
impl Scrollbar {
    // Blink Scrollbar::Scrollbar, scrollbar.cc:74; its DisplayItemClient base
    // owns the identity, never the containing LayoutBox or an exported index.
    pub fn new(
        area: *mut PaintLayerScrollableArea,
        orientation: ScrollbarOrientation,
        style_source: *mut LayoutObject,
    ) -> Self {
        Self {
            client: Default::default(),
            scrollable_area: Member::from_ptr(area),
            orientation,
            style_source: Member::from_ptr(style_source),
        }
    }
    pub fn DisplayItemClient(&self) -> &DisplayItemClient {
        &self.client
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

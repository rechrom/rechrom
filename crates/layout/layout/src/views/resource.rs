//! Resource-observer notifications on the resident LayoutObject tree.
//!
//! Each method delivers one resource notification to the core engine and
//! returns its outcome (whether the LayoutImage needs layout, or how many
//! resident clients were invalidated), so they form a borrowed view rather
//! than `LayoutMutation` commands. The view borrows `&mut` because delivery
//! marks the observing LayoutObjects dirty in place, as the source
//! ImageResourceObserver::ImageChanged notifications do.

use std::sync::Arc;

use layout_assembly::fragment_tree::PaintResources;
use layout_assembly::internal::layout_input::PaintImage;
use layout_assembly::layout_engine::HTMLImageResourceUpdate;

use crate::LayoutEngine;

/// Borrowed resource-notification view over one [`LayoutEngine`].
pub struct ResourceQueries<'a> {
    layout: &'a mut LayoutEngine,
}

impl LayoutEngine {
    /// Resource and environment notifications for resident LayoutObjects.
    #[inline]
    pub fn Resources(&mut self) -> ResourceQueries<'_> {
        ResourceQueries { layout: self }
    }
}

impl ResourceQueries<'_> {
    /// ImageLoader::UpdateLayoutObject on the resident native LayoutImage.
    #[inline]
    pub fn UpdateHTMLImageResource(
        self,
        node_id: u64,
        update: image_resource::ImageResourceContentUpdate,
    ) -> Option<HTMLImageResourceUpdate> {
        self.layout.engine.UpdateHTMLImageResource(node_id, update)
    }

    /// Resource-observer notification for document images whose embedding
    /// environment changed; returns the invalidated client count.
    #[inline]
    pub fn NotifyDocumentImageEnvironmentChanged(self, images: &[PaintImage]) -> usize {
        self.layout.engine.NotifyDocumentImageEnvironmentChanged(images)
    }

    /// Resource-observer notifications for a rebound document paint-resource
    /// catalog; returns the invalidated client count.
    #[inline]
    pub fn NotifyPaintResourcesChanged(
        self,
        previous: &[Arc<PaintResources>],
        current: &PaintResources,
    ) -> usize {
        self.layout.engine.NotifyPaintResourcesChanged(previous, current)
    }
}

//! Prepared LayoutView query over canonical same-local-root owners.
//! Source layout_view.cc:160-231 and layout_view.h:75-95,132-135,210,417-419.
use super::{
    child_frame_query::{LocalChildFrameIdentity, PreparedLocalChildLayoutView},
    query_services::ResidentQueryServices,
    ResidentHitTestServices, ResidentLayer, ResidentLayerServices,
};
use crate::core::layout::{
    hit_test_location::HitTestLocation,
    hit_test_result::{HitTestResult, HitTestResultRequest, NodeRef, ScrollbarRef},
    layout_embedded_content_hit_test::ChildFrameHitTestResult,
    layout_view::{self, LayoutViewHitTestSource, LayoutViewHitTestState},
    native_hit_test::NativeHitTestRequiredPort,
};
use foundation::{gfx, PhysicalOffset, PhysicalRect};
use layoutng_assembly::{internal::layout_view::LayoutView, layout_engine::PaintFragmentCursor};

/// Implemented by the same live LocalFrame/LocalFrameView owner. PreparedRevision
/// changes for layout, PrePaint, viewport or scrolling invalidation, even if
/// the DOM tree version did not change. No method prepares lifecycle.
pub trait PreparedLayoutViewLifecycle {
    fn Identity(&self) -> LocalChildFrameIdentity;
    fn IsPrePaintClean(&self) -> bool;
    fn PreparedRevision(&self) -> u64;
    fn DomTreeVersion(&self) -> u64;
    fn FrameSize(&self) -> Option<gfx::Size>;
    fn DocumentToFrameOffset(&self) -> PhysicalOffset;
    fn HasActiveUnboundedElements(&self) -> bool;
}

pub struct PreparedResidentLayoutView<'a> {
    view: &'a LayoutView,
    cursor: PaintFragmentCursor<'a>,
    services: &'a ResidentQueryServices<'a>,
    lifecycle: &'a dyn PreparedLayoutViewLifecycle,
    state: &'a LayoutViewHitTestState,
    identity: LocalChildFrameIdentity,
    revision: u64,
}
impl<'a> PreparedResidentLayoutView<'a> {
    pub fn new(
        view: &'a LayoutView,
        cursor: PaintFragmentCursor<'a>,
        services: &'a ResidentQueryServices<'a>,
        lifecycle: &'a dyn PreparedLayoutViewLifecycle,
        state: &'a LayoutViewHitTestState,
    ) -> Result<Self, NativeHitTestRequiredPort> {
        let query = Self {
            view,
            cursor,
            services,
            lifecycle,
            state,
            identity: lifecycle.Identity(),
            revision: lifecycle.PreparedRevision(),
        };
        query.Validate()?;
        if !state.BindPreparedOwner(view as *const LayoutView as usize, query.revision) {
            return Err(query.Required());
        }
        Ok(query)
    }
    fn Required(&self) -> NativeHitTestRequiredPort {
        NativeHitTestRequiredPort::LocalChildFrame {
            frame_identity: self.identity.frame,
        }
    }
    fn Validate(&self) -> Result<(), NativeHitTestRequiredPort> {
        if self.identity != self.lifecycle.Identity()
            || self.revision != self.lifecycle.PreparedRevision()
            || !self.lifecycle.IsPrePaintClean()
            || self.cursor.Cursor().GetLayoutObject() as usize
                != self.view as *const LayoutView as usize
            || self.view.NeedsLayout()
            || self.services.QueryError().is_some()
        {
            return Err(self.Required());
        }
        // ResidentHitTest checks exact fragment owner and its committed
        // PrePaint revisions; its LocalBorderBoxProperties are the canonical
        // source properties, not a bool inferred from another frame.
        let resident = self
            .services
            .paint
            .ResidentHitTest(self.cursor)
            .ok_or_else(|| self.Required())?;
        let identity = resident
            .LayoutObjectIdentity()
            .ok_or_else(|| self.Required())?;
        let layer = self
            .services
            .root
            .LayerForPaintObject(identity)
            .ok_or_else(|| self.Required())?;
        let native_layer = unsafe { self.view.Layer().as_ref() }.ok_or_else(|| self.Required())?;
        if !std::ptr::eq(layer, native_layer) {
            return Err(self.Required());
        }
        let _ = resident.LocalBorderBoxProperties();
        Ok(())
    }
}
impl PreparedLocalChildLayoutView for PreparedResidentLayoutView<'_> {
    fn HitTestNoLifecycleUpdate<'r>(
        &self,
        location: &HitTestLocation,
        request: &HitTestResultRequest<'r, ()>,
    ) -> Result<ChildFrameHitTestResult<'r>, NativeHitTestRequiredPort> {
        self.Validate()?;
        let mut result = HitTestResult::new(request, location);
        let inside = layout_view::HitTestNoLifecycleUpdate(self, location, &mut result)?;
        self.Validate()?;
        Ok(ChildFrameHitTestResult::new(inside, result))
    }
}
impl<'r> LayoutViewHitTestSource<'r> for PreparedResidentLayoutView<'_> {
    type Error = NativeHitTestRequiredPort;
    fn State(&self) -> &LayoutViewHitTestState {
        self.state
    }
    fn DomTreeVersion(&self) -> u64 {
        self.lifecycle.DomTreeVersion()
    }
    fn FrameSize(&self) -> Option<gfx::Size> {
        self.lifecycle.FrameSize()
    }
    fn DocumentRect(&self) -> PhysicalRect {
        self.view.DocumentRect()
    }
    fn DocumentToFrameOffset(&self) -> PhysicalOffset {
        self.lifecycle.DocumentToFrameOffset()
    }
    fn HasActiveUnboundedElements(&self) -> bool {
        self.lifecycle.HasActiveUnboundedElements()
    }
    fn LayerHitTest(
        &self,
        location: &HitTestLocation,
        result: &mut HitTestResult<'r, ()>,
        area: &PhysicalRect,
    ) -> Result<bool, Self::Error> {
        let layer = ResidentLayer::new(
            self.services.paint,
            self.services.root,
            self.cursor,
            self.services,
        )
        .ok_or_else(|| self.Required())?;
        let inside =
            crate::core::paint::paint_layer_hit_test::HitTest(&layer, location, result, area);
        if self.services.QueryError().is_some() {
            return Err(self.Required());
        }
        Ok(inside)
    }
    fn ScrollbarNode(&self, scrollbar: &ScrollbarRef) -> Result<Option<NodeRef>, Self::Error> {
        let owner = scrollbar
            .NativeScrollbarOwner()
            .ok_or_else(|| self.Required())?;
        let Some(scrollbar) = (unsafe { owner.Get().as_ref() }) else {
            return Ok(None);
        };
        let Some(area) = (unsafe { scrollbar.ScrollableArea().as_ref() }) else {
            return Ok(None);
        };
        let Some(box_) = (unsafe { area.GetLayoutBox().as_ref() }) else {
            return Ok(None);
        };
        let Some(node) = (unsafe { box_.GetNode().as_ref() }) else {
            return Ok(None);
        };
        Ok(if node.IsDocumentNode() {
            self.services.DocumentElement()
        } else {
            self.services.ResolveNativeNode(node)
        })
    }
}

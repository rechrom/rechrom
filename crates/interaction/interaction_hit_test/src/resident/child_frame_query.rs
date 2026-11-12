//! Prepared same-local-root leases for LayoutEmbeddedContent's child query.
//!
//! Chromium layout_embedded_content.cc:236-277 reads the live LocalFrameView
//! and enters its LayoutView::HitTestNoLifecycleUpdate (layout_view.cc:160-231).
//! Frame construction/lifecycle and LayoutView's cache remain with those owners;
//! this adapter never prepares lifecycle, calls Page, or copies a frame tree.
use crate::core::layout::{
    hit_test_location::HitTestLocation,
    hit_test_result::HitTestResultRequest,
    layout_embedded_content_hit_test::{
        ChildFrameHitTestResult, ChildFrameView, LocalChildFrameFacts, LocalChildFrameHitTest,
    },
    native_hit_test::NativeHitTestRequiredPort,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalChildFrameIdentity {
    pub frame: u64,
    pub local_root: u64,
    pub document_generation: u64,
}

/// Source LocalFrameView and Document admission facts, borrowed from the live
/// frame owner. No lifecycle state is inferred from the parent's clean frame.
#[derive(Clone, Copy)]
pub struct LocalChildFrameLifecycleFacts {
    pub should_throttle_rendering: bool,
    pub has_document: bool,
    pub is_prepaint_clean: bool,
    /// Actual LocalFrameView::GetLayoutView presence, independently of whether
    /// the embedding supplied a prepared query lease for that native owner.
    pub has_layout_view: bool,
}
pub trait LocalChildFrameLifecycle {
    fn Identity(&self) -> LocalChildFrameIdentity;
    fn Facts(&self) -> LocalChildFrameLifecycleFacts;
}

/// The actual prepared child LayoutView's no-update entry, including its own
/// cache, hit-test area, native PaintLayer walk, and scrollbar result handling.
/// Implementations borrow that same local root's resident owners. They must
/// propagate a missing query capability instead of publishing a partial miss.
pub trait PreparedLocalChildLayoutView {
    fn HitTestNoLifecycleUpdate<'r>(
        &self,
        location: &HitTestLocation,
        request: &HitTestResultRequest<'r, ()>,
    ) -> Result<ChildFrameHitTestResult<'r>, NativeHitTestRequiredPort>;
}

/// Call-scoped child owner. The lifecycle owner distinguishes a genuinely
/// absent LayoutView from a real LayoutView whose prepared lease is missing.
/// The latter remains a required error if the source enters its query branch.
pub struct PreparedLocalChildFrameQuery<'a> {
    lifecycle: &'a dyn LocalChildFrameLifecycle,
    layout_view: Option<&'a dyn PreparedLocalChildLayoutView>,
    identity: LocalChildFrameIdentity,
}
impl<'a> PreparedLocalChildFrameQuery<'a> {
    pub fn new(
        parent_local_root: u64,
        lifecycle: &'a dyn LocalChildFrameLifecycle,
        layout_view: Option<&'a dyn PreparedLocalChildLayoutView>,
    ) -> Result<Self, NativeHitTestRequiredPort> {
        let identity = lifecycle.Identity();
        if identity.local_root != parent_local_root {
            return Err(NativeHitTestRequiredPort::LocalChildFrame {
                frame_identity: identity.frame,
            });
        }
        Ok(Self {
            lifecycle,
            layout_view,
            identity,
        })
    }
    pub fn FrameView(
        &self,
        frame_identity: u64,
    ) -> Result<ChildFrameView<'_>, NativeHitTestRequiredPort> {
        if self.lifecycle.Identity().frame != frame_identity {
            return Err(NativeHitTestRequiredPort::LocalChildFrame { frame_identity });
        }
        Ok(ChildFrameView::Local(Box::new(self)))
    }
}
impl LocalChildFrameHitTest for PreparedLocalChildFrameQuery<'_> {
    fn Facts(&self) -> Result<LocalChildFrameFacts, NativeHitTestRequiredPort> {
        self.ValidateGeneration()?;
        let facts = self.lifecycle.Facts();
        Ok(LocalChildFrameFacts {
            should_throttle_rendering: facts.should_throttle_rendering,
            has_document: facts.has_document,
            is_prepaint_clean: facts.is_prepaint_clean,
            has_layout_view: facts.has_layout_view,
        })
    }
    fn HitTestNoLifecycleUpdate<'r>(
        &self,
        location: &HitTestLocation,
        request: &HitTestResultRequest<'r, ()>,
    ) -> Result<ChildFrameHitTestResult<'r>, NativeHitTestRequiredPort> {
        self.ValidateGeneration()?;
        self.layout_view
            .ok_or(NativeHitTestRequiredPort::LocalChildFrame {
                frame_identity: self.lifecycle.Identity().frame,
            })?
            .HitTestNoLifecycleUpdate(location, request)
    }
}
impl PreparedLocalChildFrameQuery<'_> {
    fn ValidateGeneration(&self) -> Result<(), NativeHitTestRequiredPort> {
        if self.lifecycle.Identity() != self.identity {
            return Err(NativeHitTestRequiredPort::LocalChildFrame {
                frame_identity: self.identity.frame,
            });
        }
        Ok(())
    }
}
impl<T: LocalChildFrameHitTest + ?Sized> LocalChildFrameHitTest for &T {
    fn Facts(&self) -> Result<LocalChildFrameFacts, NativeHitTestRequiredPort> {
        (**self).Facts()
    }
    fn HitTestNoLifecycleUpdate<'r>(
        &self,
        location: &HitTestLocation,
        request: &HitTestResultRequest<'r, ()>,
    ) -> Result<ChildFrameHitTestResult<'r>, NativeHitTestRequiredPort> {
        (**self).HitTestNoLifecycleUpdate(location, request)
    }
}

/// Lookup through the embedding's existing same-local-root Frame owner. This
/// port borrows prepared leases; it does not retain an alternate frame index.
pub trait PreparedLocalChildFrameQueries {
    fn ForFrame(
        &self,
        frame_identity: u64,
    ) -> Result<Box<dyn LocalChildFrameHitTest + '_>, NativeHitTestRequiredPort>;
}

/// Explicit assembly for a host that has not constructed local child Frames.
/// If DOM ever contains a real local Frame, this rejects its missing query
/// owner. Only DOM's actual absent/remote Frame classification is NoLocalFrame.
pub struct UnavailableLocalChildFrameQueries;
impl PreparedLocalChildFrameQueries for UnavailableLocalChildFrameQueries {
    fn ForFrame(
        &self,
        frame_identity: u64,
    ) -> Result<Box<dyn LocalChildFrameHitTest + '_>, NativeHitTestRequiredPort> {
        Err(NativeHitTestRequiredPort::LocalChildFrame { frame_identity })
    }
}

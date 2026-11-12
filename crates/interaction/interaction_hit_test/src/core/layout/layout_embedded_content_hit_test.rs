//! Source layout_embedded_content.cc:205-313, Chromium
//! 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
use super::{
    hit_test_location::HitTestLocation,
    hit_test_phase::HitTestPhase,
    hit_test_request::HitTestRequest,
    hit_test_result::{HitTestResult, HitTestResultRequest, NodeRef},
    native_hit_test::{
        NativeHitTestRequiredPort, NativeHitTestResult, ReplacedHitTestSource, ReplacedNodeAtPoint,
    },
};
use foundation::{PhysicalOffset, PhysicalRect};

/// These are the source LocalFrameView admission facts, read under its query
/// lease. A remote frame has no LocalFrameView and follows NoLocalFrame.
#[derive(Clone, Copy)]
pub struct LocalChildFrameFacts {
    pub should_throttle_rendering: bool,
    pub has_document: bool,
    pub is_prepaint_clean: bool,
    pub has_layout_view: bool,
}
pub enum ChildFrameView<'a> {
    NoLocalFrame,
    Local(Box<dyn LocalChildFrameHitTest + 'a>),
}
/// A completed child query. The caller receives its own result; the parent
/// result is never lent to the child owner or retained across frame domains.
pub struct ChildFrameHitTestResult<'r> {
    inside: bool,
    result: HitTestResult<'r, ()>,
}
impl<'r> ChildFrameHitTestResult<'r> {
    pub fn new(inside: bool, result: HitTestResult<'r, ()>) -> Self {
        Self { inside, result }
    }
    pub fn Result(&self) -> &HitTestResult<'r, ()> {
        &self.result
    }
}
/// A same-local-root, already prepared read lease. This interface must never
/// be implemented by a synchronous cross-Page or IPC call. Remote children
/// are admitted by the host's frame routing and expose NoLocalFrame here.
pub trait LocalChildFrameHitTest {
    fn Facts(&self) -> Result<LocalChildFrameFacts, NativeHitTestRequiredPort>;
    fn HitTestNoLifecycleUpdate<'r>(
        &self,
        location: &HitTestLocation,
        request: &HitTestResultRequest<'r, ()>,
    ) -> Result<ChildFrameHitTestResult<'r>, NativeHitTestRequiredPort>;
}
pub trait EmbeddedContentHitTestSource<'r> {
    fn Replaced(&self) -> &dyn ReplacedHitTestSource<'r>;
    fn ChildFrameView(&self) -> Result<ChildFrameView<'_>, NativeHitTestRequiredPort>;
    fn MatchesStopNode(&self, result: &HitTestResult<'r, ()>) -> bool;
    fn PointOverResizer(
        &self,
        touch: bool,
        location: &HitTestLocation,
        offset: PhysicalOffset,
    ) -> bool;
    fn PhysicalContentBoxRect(&self) -> PhysicalRect;
    fn OwnerNode(&self) -> Option<NodeRef>;
}
pub fn NodeAtPointOverEmbeddedContentView<'r>(
    source: &dyn EmbeddedContentHitTestSource<'r>,
    result: &mut HitTestResult<'r, ()>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
    phase: HitTestPhase,
) -> NativeHitTestResult {
    let had_result = result.InnerNode().is_some();
    let inside = ReplacedNodeAtPoint(source.Replaced(), result, location, offset, phase)?;
    let owner = source.OwnerNode();
    if (inside || location.IsRectBasedTest())
        && !had_result
        && result
            .InnerNode()
            .zip(owner.as_ref())
            .is_some_and(|(a, b)| a.DOMIdentity() == b.DOMIdentity())
    {
        result.SetIsOverEmbeddedContentView(
            source
                .PhysicalContentBoxRect()
                .ContainsPoint(result.LocalPoint())
                && !result.IsOverResizer(),
        );
    }
    Ok(inside)
}
pub fn NodeAtPoint<'r>(
    source: &dyn EmbeddedContentHitTestSource<'r>,
    result: &mut HitTestResult<'r, ()>,
    location: &HitTestLocation,
    offset: PhysicalOffset,
    phase: HitTestPhase,
) -> NativeHitTestResult {
    let child = source.ChildFrameView();
    let skip = source.MatchesStopNode(result)
        || !result.GetHitTestRequest().AllowsChildFrameContent()
        || source.PointOverResizer(result.GetHitTestRequest().TouchEvent(), location, offset);
    if skip {
        return NodeAtPointOverEmbeddedContentView(source, result, location, offset, phase);
    }
    let ChildFrameView::Local(child) = child? else {
        return NodeAtPointOverEmbeddedContentView(source, result, location, offset, phase);
    };
    let facts = child.Facts()?;
    if facts.should_throttle_rendering || !facts.has_document || !facts.is_prepaint_clean {
        return NodeAtPointOverEmbeddedContentView(source, result, location, offset, phase);
    }
    if phase == HitTestPhase::kForeground
        && source
            .Replaced()
            .VisibleToHitTestRequest(result.GetHitTestRequest().IgnorePointerEventsNone())
        && facts.has_layout_view
    {
        // Source cc:265-277. This is the content-box translation into the
        // child frame, not a DPR or root-view coordinate conversion.
        let child_location =
            HitTestLocation::WithOffset(location, -offset - source.PhysicalContentBoxRect().offset);
        let request = HitTestRequest::new(
            result.GetHitTestRequest().GetType()
                | HitTestRequest::<(), dyn super::hit_test_result::HitTestNode>::kChildFrameHitTest,
            result.GetHitTestRequest().GetStopNode(),
            None,
        );
        let ChildFrameHitTestResult {
            inside,
            result: mut child_result,
        } = child.HitTestNoLifecycleUpdate(&child_location, &request)?;
        if result.GetHitTestRequest().ListBased() {
            result.Append(&child_result);
        } else if inside {
            child_result.SetCacheable(false);
            result.Assign(&child_result);
        }
        if inside {
            if !location.IsRectBasedTest() {
                return Ok(true);
            }
            let mut over_view_result = result.clone();
            if NodeAtPointOverEmbeddedContentView(
                source,
                &mut over_view_result,
                location,
                offset,
                phase,
            )? {
                return Ok(true);
            }
            result.Assign(&over_view_result);
            return Ok(false);
        }
    }
    NodeAtPointOverEmbeddedContentView(source, result, location, offset, phase)
}

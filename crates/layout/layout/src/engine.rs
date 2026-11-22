//! `layout::LayoutEngine`: the public face of the core LayoutNG engine.
//!
//! The wrapper owns the core engine by value (no extra indirection) and
//! forwards a chosen set of methods with `#[inline]`, so other engines see
//! only these entry points instead of the core engine's full method set.

use std::rc::Rc;
use std::sync::Arc;

use foundation::{gfx, CompositorElementId, Persistent};
use layout_assembly::editing_state::LayoutEditingState;
use layout_assembly::engine_assembly::LayoutAssembly;
use layout_assembly::fragment_tree::{LayoutResultHandle, PaintResources};
use layout_assembly::internal::layout_input::{ConstraintSpace, ElementData, Offset, PaintImage};
use layout_assembly::internal::layout_scrollable_area::NativeScrollbarActivity;
use layout_assembly::layout_engine::{
    ContainerQuerySizeSnapshot, HTMLImageResourceUpdate, LayoutEngine as CoreLayoutEngine,
    LayoutMutation, LayoutNode, LayoutOutput, LayoutPaintOutput, LayoutPaintScope,
    NativeIntersectionRoot, PaintCommitOutput, ScrollLayoutUpdate, SuppressScrollAnchorScope,
};
use layout_assembly::layout_selection::{LayoutSelectionRange, SelectionStyleAndPaintInvalidation};
use layout_assembly::native_autoscroll_source::{
    NativeAutoscrollLayout, NativeAutoscrollSource, NativeLogicalScrollArea,
};
use layout_assembly::native_event_registry_source::NativeEventRegistryLayout;
use layout_assembly::native_page_scale_geometry::NativePageScaleGeometry;
use layout_assembly::native_scroll_source::{
    NativeCompositorScrollArea, NativeCompositorScrollUpdate, NativeScrollFrameGeometry,
    NativeScrollResult, NativeScrollSource,
};
use layout_assembly::native_user_scroll_source::NativeUserScrollSource;
use layout_assembly::native_wheel_scrollable_traversal::NativeWheelScrollableAreaTraversal;
use layout_assembly::password_echo::{PasswordEchoLayoutSettings, PasswordEchoTimerScheduler};
use layout_assembly::style_projection::StyleResourceBindings;
use layout_style::style::computed_style::ComputedStyle;

/// Owns the resident LayoutObject tree, its constraints and published result
/// through the core engine. DOM supplies mutations; Paint consumes the
/// fragment output.
pub struct LayoutEngine {
    pub(crate) engine: CoreLayoutEngine,
}

/// Construction.
impl LayoutEngine {
    /// An empty engine over an assembled algorithm set (see
    /// [`crate::CreateLayoutAssembly`]).
    #[inline]
    pub fn new(assembly: &LayoutAssembly) -> Self {
        Self {
            engine: CoreLayoutEngine::new(assembly),
        }
    }
}

/// Mutation: boundary 3 (Style/DOM -> Layout). `LayoutMutation::TreeUpdate`
/// lends Layout's tree builder for one synchronous attach transaction.
impl LayoutEngine {
    #[inline]
    pub fn ApplyMutation(&mut self, mutation: LayoutMutation<'_>) -> bool {
        self.engine.ApplyMutation(mutation)
    }

    /// Whether a `NodeInput` mutation would leave this node's geometry input
    /// unchanged.
    #[inline]
    pub fn GeometryInputMatches(
        &self,
        id: u64,
        style: &ComputedStyle,
        resources: &StyleResourceBindings<'_>,
        element: &Option<ElementData>,
    ) -> bool {
        self.engine
            .GeometryInputMatches(id, style, resources, element)
    }

    /// Elements whose layout subtree a document detach destroyed.
    #[inline]
    pub fn TakeDetachedElements(&mut self) -> Vec<u64> {
        self.engine.TakeDetachedElements()
    }
}

/// Lifecycle.
impl LayoutEngine {
    #[inline]
    pub fn Layout(&mut self) {
        self.engine.Layout()
    }

    #[inline]
    pub fn LayoutIfNeeded(&mut self) -> bool {
        self.engine.LayoutIfNeeded()
    }

    #[inline]
    pub fn UpdateLayout(&mut self) {
        self.engine.UpdateLayout()
    }

    #[inline]
    pub fn UpdateLayoutIfNeeded(&mut self) -> bool {
        self.engine.UpdateLayoutIfNeeded()
    }

    #[inline]
    pub fn RunProvisionalLayoutWithoutPublication(&mut self) {
        self.engine.RunProvisionalLayoutWithoutPublication()
    }

    #[inline]
    pub fn FinishLayoutWithoutPaintPublication(&mut self) -> bool {
        self.engine.FinishLayoutWithoutPaintPublication()
    }

    #[inline]
    pub fn UpdateScrollLayout(
        &mut self,
        fragments: &mut LayoutResultHandle,
        scroll_only: bool,
        refresh_cull_rect: bool,
    ) -> Option<ScrollLayoutUpdate> {
        self.engine
            .UpdateScrollLayout(fragments, scroll_only, refresh_cull_rect)
    }
}

/// Paint: boundary 4 (Layout -> Paint). Paint output, its call-scoped
/// virtual-dispatch scope, and the commit points Page drives.
impl LayoutEngine {
    #[inline]
    pub fn PaintScope(&self) -> LayoutPaintScope<'_> {
        self.engine.PaintScope()
    }

    #[inline]
    pub fn TakePaintOutput(&mut self) -> Option<LayoutPaintOutput> {
        self.engine.TakePaintOutput()
    }

    #[inline]
    pub fn ReturnPrePaintOutput(&mut self, layout: LayoutPaintOutput) {
        self.engine.ReturnPrePaintOutput(layout)
    }

    #[inline]
    pub fn TakePublishedPaintOutput(&mut self, fragments: LayoutResultHandle) -> LayoutPaintOutput {
        self.engine.TakePublishedPaintOutput(fragments)
    }

    #[inline]
    pub fn RefreshOutputPrePaintInputs(&mut self, output: &mut LayoutPaintOutput) {
        self.engine.RefreshOutputPrePaintInputs(output)
    }

    #[inline]
    pub fn CommitPaint<P: PaintCommitOutput>(
        &mut self,
        layout: LayoutPaintOutput,
        paint: P,
        editing: Option<&LayoutEditingState>,
    ) -> (LayoutResultHandle, P::Artifact) {
        self.engine.CommitPaint(layout, paint, editing)
    }

    #[inline]
    pub fn CommitRetainedPaint(&mut self, layout: LayoutPaintOutput) -> LayoutResultHandle {
        self.engine.CommitRetainedPaint(layout)
    }

    #[inline]
    pub fn PublishPaintData(&mut self) -> bool {
        self.engine.PublishPaintData()
    }

    #[inline]
    pub fn RefreshTargetedPaintData(&mut self) -> bool {
        self.engine.RefreshTargetedPaintData()
    }

    #[inline]
    pub fn RefreshTargetedPaintInputs(&mut self) -> bool {
        self.engine.RefreshTargetedPaintInputs()
    }

    #[inline]
    pub fn GetLayoutResult(&self) -> Option<&LayoutResultHandle> {
        self.engine.GetLayoutResult()
    }

    #[inline]
    pub fn TakeLayoutResult(&mut self) -> Option<LayoutResultHandle> {
        self.engine.TakeLayoutResult()
    }

    #[inline]
    pub fn AdoptPublishedLayoutResult(&mut self, fragments: LayoutResultHandle) {
        self.engine.AdoptPublishedLayoutResult(fragments)
    }

    #[inline]
    pub fn ReleaseLayoutResult(&mut self) {
        self.engine.ReleaseLayoutResult()
    }
}

/// Queries: synchronous geometry, scroll, selection, client-rect and hit-test
/// entry points over the resident fragments.
impl LayoutEngine {
    #[inline]
    pub fn Output(&self) -> Option<LayoutOutput<'_>> {
        self.engine.Output()
    }

    #[inline]
    pub fn GetConstraints(&self) -> &ConstraintSpace {
        self.engine.GetConstraints()
    }

    #[inline]
    pub fn LayoutZoomFactor(&self) -> f64 {
        self.engine.LayoutZoomFactor()
    }

    #[inline]
    pub fn HasCompletedLayoutForHitTest(&self) -> bool {
        self.engine.HasCompletedLayoutForHitTest()
    }

    #[inline]
    pub fn CurrentLayoutNode(&self, id: u64) -> Option<LayoutNode<'_>> {
        self.engine.CurrentLayoutNode(id)
    }

    #[inline]
    pub fn HasEffectiveAppearance(&self, id: u64) -> bool {
        self.engine.HasEffectiveAppearance(id)
    }

    #[inline]
    pub fn ContainerQuerySizes(&self) -> Vec<ContainerQuerySizeSnapshot> {
        self.engine.ContainerQuerySizes()
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn ComputeIntersectionGeometryForNode(
        &self,
        target_id: u64,
        root_node: Option<NativeIntersectionRoot>,
        standardized_browser_zoom: bool,
        frame_occlusion: intersection_observer::OcclusionState,
        params: &intersection_observer::IntersectionObserverParams,
        flags: u32,
        root_geometry: &mut Option<intersection_observer::RootGeometry>,
        cache: &mut intersection_observer::CachedRects,
    ) -> Result<
        intersection_observer::IntersectionGeometry,
        intersection_observer::RequiredIntersectionMechanism,
    > {
        self.engine.ComputeIntersectionGeometryForNode(
            target_id,
            root_node,
            standardized_browser_zoom,
            frame_occlusion,
            params,
            flags,
            root_geometry,
            cache,
        )
    }

    #[inline]
    pub fn AbsoluteLayoutSelectionBounds(&self) -> gfx::Rect {
        self.engine.AbsoluteLayoutSelectionBounds()
    }

    #[inline]
    pub fn PageScaleGeometry(&self) -> Option<NativePageScaleGeometry> {
        self.engine.PageScaleGeometry()
    }

    #[inline]
    pub fn TextControlValueWithHardLineBreaks(&self, inner_editor: u64) -> Option<Rc<[u16]>> {
        self.engine.TextControlValueWithHardLineBreaks(inner_editor)
    }

    #[inline]
    pub fn LayoutViewportLogicalScrollArea(&self) -> Option<NativeLogicalScrollArea> {
        self.engine.LayoutViewportLogicalScrollArea()
    }

    #[inline]
    pub fn WheelScrollableAreaTraversal(
        &self,
        target_node_id: u64,
    ) -> NativeWheelScrollableAreaTraversal {
        self.engine.WheelScrollableAreaTraversal(target_node_id)
    }

    #[inline]
    pub fn CompositorScrollArea(
        &self,
        expected: CompositorElementId,
    ) -> Option<NativeCompositorScrollArea> {
        self.engine.CompositorScrollArea(expected)
    }

    #[inline]
    pub fn CompositorScrollbarOwner(&self, expected: CompositorElementId) -> Option<u64> {
        self.engine.CompositorScrollbarOwner(expected)
    }
}

/// Legacy side channels that consumer crates call today.
// Wave E: fold into LayoutMutation / query views
impl LayoutEngine {
    // Resource and environment invalidations.

    #[inline]
    pub fn UpdateHTMLImageResource(
        &mut self,
        node_id: u64,
        update: image_resource::ImageResourceContentUpdate,
    ) -> Option<HTMLImageResourceUpdate> {
        self.engine.UpdateHTMLImageResource(node_id, update)
    }

    #[inline]
    pub fn NotifyDocumentImageEnvironmentChanged(&mut self, images: &[PaintImage]) -> usize {
        self.engine.NotifyDocumentImageEnvironmentChanged(images)
    }

    #[inline]
    pub fn NotifyPaintResourcesChanged(
        &mut self,
        previous: &[Arc<PaintResources>],
        current: &PaintResources,
    ) -> usize {
        self.engine.NotifyPaintResourcesChanged(previous, current)
    }

    // Selection, caret and editing invalidations.

    #[inline]
    pub fn CommitLayoutSelection<I: Iterator<Item = u64> + Clone>(
        &mut self,
        range: Option<LayoutSelectionRange<I>>,
        hidden: bool,
    ) -> bool {
        self.engine.CommitLayoutSelection(range, hidden)
    }

    #[inline]
    pub fn MarkSelectionEndpointPaintingLayerNeedsRepaint(&mut self, node_id: u64) {
        self.engine
            .MarkSelectionEndpointPaintingLayerNeedsRepaint(node_id)
    }

    #[inline]
    pub fn InvalidateStyleAndPaintForSelection(&mut self) -> SelectionStyleAndPaintInvalidation {
        self.engine.InvalidateStyleAndPaintForSelection()
    }

    #[inline]
    pub fn InvalidateBlockCaretText(&mut self, anchors: &[u64]) {
        self.engine.InvalidateBlockCaretText(anchors)
    }

    #[inline]
    pub fn SelectionTextOverflowOwnerIdentity(&self, node_id: u64) -> Option<usize> {
        self.engine.SelectionTextOverflowOwnerIdentity(node_id)
    }

    #[inline]
    pub fn SetContainsSelectionFocusByIdentity(
        &mut self,
        owner_identity: usize,
        contains_focus: bool,
    ) -> Option<u64> {
        self.engine
            .SetContainsSelectionFocusByIdentity(owner_identity, contains_focus)
    }

    #[inline]
    pub fn InvalidateTextOverflowLayoutForNode(&mut self, node_id: u64) -> bool {
        self.engine.InvalidateTextOverflowLayoutForNode(node_id)
    }

    #[inline]
    pub fn UpdateEditingNodeFactsForFocus(
        &mut self,
        node_id: u64,
        facts: layout_assembly::EditingNodeFacts,
    ) -> bool {
        self.engine.UpdateEditingNodeFactsForFocus(node_id, facts)
    }

    #[inline]
    pub fn InvalidateDocumentMarkerPaint(&mut self, id: u64) -> bool {
        self.engine.InvalidateDocumentMarkerPaint(id)
    }

    #[inline]
    pub fn MomentarilyRevealLastTypedCharacterIfSecure(
        &mut self,
        node_id: u64,
        offset: usize,
        settings: Option<PasswordEchoLayoutSettings>,
        scheduler: &dyn PasswordEchoTimerScheduler,
    ) -> bool {
        self.engine
            .MomentarilyRevealLastTypedCharacterIfSecure(node_id, offset, settings, scheduler)
    }

    // Form-control routes.

    #[inline]
    pub fn InvalidateInputCapsLockIndicatorPaint(&mut self, id: u64) -> bool {
        self.engine.InvalidateInputCapsLockIndicatorPaint(id)
    }

    #[inline]
    pub fn SetInputModifiedStyleOutsideStyleRecalc(
        &mut self,
        id: u64,
        style: Persistent<ComputedStyle>,
    ) -> bool {
        self.engine
            .SetInputModifiedStyleOutsideStyleRecalc(id, style)
    }

    #[inline]
    pub fn ScrollSelectOptionIntoView(
        &mut self,
        container: u64,
        option: u64,
    ) -> NativeScrollResult {
        self.engine.ScrollSelectOptionIntoView(container, option)
    }

    // Event-handler registry.

    #[inline]
    pub fn EventRegistryLayout(&self, node: Option<u64>) -> Option<NativeEventRegistryLayout> {
        self.engine.EventRegistryLayout(node)
    }

    #[inline]
    pub fn InvalidateEventRegistryLayout(
        &mut self,
        target: &NativeEventRegistryLayout,
        blocking_wheel: bool,
    ) {
        self.engine
            .InvalidateEventRegistryLayout(target, blocking_wheel)
    }

    // Native scroll sources and scroll side effects.

    #[inline]
    pub fn NativeScrollSource(&mut self, id: u64) -> Option<NativeScrollSource<'_>> {
        self.engine.NativeScrollSource(id)
    }

    #[inline]
    pub fn NativeUserScrollSource<'a>(
        &'a mut self,
        native: &'a NativeLogicalScrollArea,
    ) -> Option<NativeUserScrollSource<'a>> {
        self.engine.NativeUserScrollSource(native)
    }

    #[inline]
    pub fn NativeAutoscrollSource<'a>(
        &'a mut self,
        object: &'a NativeAutoscrollLayout,
        frame: NativeScrollFrameGeometry,
    ) -> Option<NativeAutoscrollSource<'a>> {
        self.engine.NativeAutoscrollSource(object, frame)
    }

    #[inline]
    pub fn AutoscrollLayoutFor(&self, node_id: u64) -> Option<NativeAutoscrollLayout> {
        self.engine.AutoscrollLayoutFor(node_id)
    }

    #[inline]
    pub fn LogicalScrollLayoutFor(&self, node_id: u64) -> Option<NativeAutoscrollLayout> {
        self.engine.LogicalScrollLayoutFor(node_id)
    }

    #[inline]
    pub fn LogicalScrollLayoutView(&self) -> Option<NativeAutoscrollLayout> {
        self.engine.LogicalScrollLayoutView()
    }

    #[inline]
    pub fn DidCompositorScroll(
        &mut self,
        expected: CompositorElementId,
        position: Offset,
        source_type: scroll_snap::scroll_source_type::ScrollSourceType,
    ) -> Option<NativeCompositorScrollUpdate> {
        self.engine
            .DidCompositorScroll(expected, position, source_type)
    }

    #[inline]
    pub fn SuppressScrollAnchoringForNode(
        &self,
        anchor_node: Option<u64>,
    ) -> SuppressScrollAnchorScope {
        self.engine.SuppressScrollAnchoringForNode(anchor_node)
    }

    #[inline]
    pub fn UpdateScrollbarsHiddenIfOverlay(&mut self, id: u64, hidden: bool) -> bool {
        self.engine.UpdateScrollbarsHiddenIfOverlay(id, hidden)
    }

    #[inline]
    pub fn TakeNativeScrollbarActivities(&mut self) -> Vec<NativeScrollbarActivity> {
        self.engine.TakeNativeScrollbarActivities()
    }
}

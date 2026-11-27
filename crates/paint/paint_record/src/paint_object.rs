//! The painted object as Chromium's painters receive it.
//!
//! Chromium painters take the LayoutObject / PhysicalBoxFragment being
//! painted, the FragmentData of the painted occurrence and a PaintInfo
//! (box_fragment_painter.h:43-60, scoped_paint_state.h:24-60,
//! object_painter.h:21-27). `PaintObject` is that borrowed pair for
//! paint_record: a small `Copy` view over the source fragment cursor, its
//! paint sidecar (`cursor.PaintData()`), the occurrence's
//! `PrePaintFragmentData`, the source `PaintLayer`, and the walk state
//! Chromium keeps as paint traversal state (ScopedDisplayItemFragment).
//!
//! It is still backed by the resident PrePaint node, which is the only place
//! this package names it. The accessors under "Node-backed values" read
//! PrePaint copies of state Chromium derives from Layout, FragmentData,
//! ObjectPaintProperties, PaintLayer or the paint walk itself; each is to be
//! re-sourced there, after which the backing node goes away and the view is
//! built from the walk's cursor and FragmentData instead.

#![allow(non_snake_case)]

use std::cell::Cell;
use std::sync::Arc;

use layout::paint::{FragmentPaintData, PaintFragmentCursor};
use layout::views::paint_layer::PaintLayer;
use paint_input_types::layout_input::{Offset, PaintClipPathChild, PaintPathCommand};
use paint_input_types::paint_input::PaintCornerRadii;
use paint_input_types::PaintResources;
use paint_model::paint_property_tree::{
    PaintPropertyRole, PropertyTreeState as PaintPropertyTreeState, TransformPaintPropertyNode,
};
use paint_model::{DisplayMaskLayer, PaintResult};
use prepaint::fragment_data::PrePaintFragmentData;
use prepaint::pre_paint_tree_walk::{
    FlatTransforms, PaintTreeNode, PropertyTreeState as PrePaintPropertyTreeState,
};

use crate::PaintRect;

/// One painted object occurrence: source fragment + FragmentData + walk state.
#[derive(Clone, Copy)]
pub struct PaintObject<'n, 'f> {
    node: &'n PaintTreeNode<'f>,
}

impl<'n, 'f> PaintObject<'n, 'f> {
    /// The resident PrePaint occurrence the paint walk is visiting.
    pub fn new(node: &'n PaintTreeNode<'f>) -> Self {
        Self { node }
    }

    /// The walk-side node, for paint_walk while its own traversal still runs
    /// over PrePaint nodes. Painters never use it.
    #[doc(hidden)]
    pub fn BackingNode(self) -> &'n PaintTreeNode<'f> {
        self.node
    }

    // -----------------------------------------------------------------
    // Source object and FragmentData.
    // -----------------------------------------------------------------

    /// The borrowed fragment cursor of the painted object (the walk handle).
    pub fn SourceCursor(self) -> Option<PaintFragmentCursor<'n>> {
        self.node.SourceCursor()
    }

    /// The fragment's paint sidecar (`cursor.PaintData()`): the paint view of
    /// the PhysicalBoxFragment / FragmentItem and its LayoutObject.
    pub fn Fragment(self) -> Option<&'n FragmentPaintData> {
        self.node.fragment.as_deref()
    }

    /// FragmentData of this occurrence (core/paint/fragment_data.h).
    pub fn FragmentData(self) -> Option<&'n PrePaintFragmentData> {
        self.node.FragmentData()
    }

    /// `To<LayoutBoxModelObject>(object).Layer()` of the source object.
    pub fn PaintLayer(self) -> Option<&'n PaintLayer> {
        self.SourceCursor()?.OwnPaintLayer()
    }

    /// Document / LocalFrameView paint resources of the source object.
    pub fn DocumentResources(self) -> Option<&'n Arc<PaintResources>> {
        self.node.DocumentResources()
    }

    /// The active ScopedDisplayItemFragment (paint traversal state).
    pub fn DisplayItemFragment(self) -> u32 {
        self.node.DisplayItemFragment()
    }

    /// The fragmentainer this occurrence paints inside (traversal state).
    pub fn FragmentainerInstanceId(self) -> u64 {
        self.node.FragmentainerInstanceId()
    }

    /// Whether ObjectPaintProperties holds the slot of `role`.
    pub fn HasOwnPropertyRole(self, role: PaintPropertyRole) -> bool {
        self.node.HasOwnPropertyRole(role)
    }

    // -----------------------------------------------------------------
    // Paint offset and snapping in the flat replay space.
    // -----------------------------------------------------------------

    /// FragmentData::PaintOffset() in the flat replay space.
    pub fn FlatPaintOffset(self) -> Offset {
        self.node.FlatPaintOffset()
    }

    /// FlatPaintOffset with this occurrence's FragmentData already resolved.
    pub fn FlatPaintOffsetOf(self, data: Option<&PrePaintFragmentData>) -> Offset {
        self.node.FlatPaintOffsetOf(data)
    }

    pub fn PixelSnappedRect(self, rect: &PaintRect) -> PaintRect {
        self.node.PixelSnappedRect(rect)
    }

    pub fn EnclosingRect(self, rect: PaintRect) -> PaintRect {
        self.node.EnclosingRect(rect)
    }

    pub fn PixelSnappedOffset(self, offset: Offset) -> Offset {
        self.node.PixelSnappedOffset(offset)
    }

    /// ObjectPaintProperties::MaskClip source rect in the flat replay space.
    pub fn ResolvedMaskClipRect(self) -> Option<PaintRect> {
        self.node.ResolvedMaskClipRect()
    }

    // -----------------------------------------------------------------
    // Tree access.
    // -----------------------------------------------------------------

    /// The painted children of this object in PrePaint traversal order.
    pub fn Children(self) -> PaintObjectChildren<'n, 'f> {
        PaintObjectChildren {
            inner: self.node.children.iter(),
        }
    }

    /// The painted SVG resource subtree root registered for `client`.
    pub fn SvgResourceNode(self, client: u64) -> Option<PaintObject<'n, 'static>> {
        self.node.SvgResourceNode(client).map(PaintObject::new)
    }

    // -----------------------------------------------------------------
    // Node-backed values (temporary; see the module comment).
    // -----------------------------------------------------------------

    /// Flat-to-property-space delta of the paint offset.
    pub fn paint_snap_offset(self) -> Offset {
        self.node.paint_snap_offset
    }
    /// Device scale used to snap paint geometry.
    pub fn pixel_snap_scale(self) -> f64 {
        self.node.pixel_snap_scale
    }
    pub fn root_clip(self) -> Option<PaintRect> {
        self.node.root_clip
    }
    pub fn local_inner_border_radius_clip(self) -> Option<PaintRect> {
        self.node.local_inner_border_radius_clip
    }
    pub fn local_inner_border_radius_radii(self) -> PaintCornerRadii {
        self.node.local_inner_border_radius_radii
    }
    pub fn local_clip(self) -> Option<PaintRect> {
        self.node.local_clip
    }
    pub fn local_css_clip(self) -> Option<PaintRect> {
        self.node.local_css_clip
    }
    pub fn local_clip_radii(self) -> PaintCornerRadii {
        self.node.local_clip_radii
    }
    pub fn local_clip_path(self) -> &'n Vec<PaintPathCommand> {
        &self.node.local_clip_path
    }
    pub fn local_clip_path_children(self) -> &'n Vec<PaintClipPathChild> {
        &self.node.local_clip_path_children
    }
    pub fn clip_path_even_odd(self) -> bool {
        self.node.clip_path_even_odd
    }
    /// Contents cull rect (a layer's CullRectUpdater contents cull rect).
    pub fn cull_rect(self) -> Option<PaintRect> {
        self.node.cull_rect
    }
    /// The PaintInfo cull rect of a non-layer object.
    pub fn self_cull_rect(self) -> Option<PaintRect> {
        self.node.self_cull_rect
    }
    pub fn transforms(self) -> &'n FlatTransforms {
        &self.node.transforms
    }
    pub fn local_transforms(self) -> &'n Vec<paint_input_types::layout_input::TransformMatrix> {
        &self.node.local_transforms
    }
    pub fn paint_offset_translation(self) -> &'n Option<Arc<TransformPaintPropertyNode>> {
        &self.node.paint_offset_translation
    }
    pub fn scroll_offset(self) -> Offset {
        self.node.scroll_offset
    }
    pub fn fixed_background_acts_as_scroll(self) -> bool {
        self.node.fixed_background_acts_as_scroll
    }
    pub fn is_paint_layer(self) -> bool {
        self.node.is_paint_layer
    }
    pub fn is_self_painting_layer(self) -> bool {
        self.node.is_self_painting_layer
    }
    pub fn paint_layer_is_fragmented(self) -> bool {
        self.node.paint_layer_is_fragmented
    }
    pub fn is_root(self) -> bool {
        self.node.is_root
    }
    pub fn needs_reorder_overlay_overflow_controls(self) -> &'n Cell<bool> {
        &self.node.needs_reorder_overlay_overflow_controls
    }
    pub fn previous_paint_result(self) -> &'n Cell<PaintResult> {
        &self.node.previous_paint_result
    }
    pub fn applies_transform(self) -> bool {
        self.node.applies_transform
    }
    pub fn applies_root_clip(self) -> bool {
        self.node.applies_root_clip
    }
    pub fn applies_overflow_clip(self) -> bool {
        self.node.applies_overflow_clip
    }
    pub fn applies_css_clip(self) -> bool {
        self.node.applies_css_clip
    }
    pub fn applies_clip_path(self) -> bool {
        self.node.applies_clip_path
    }
    pub fn applies_effect(self) -> bool {
        self.node.applies_effect
    }
    pub fn applies_opacity(self) -> bool {
        self.node.applies_opacity
    }
    pub fn applies_blend(self) -> bool {
        self.node.applies_blend
    }
    pub fn applies_filter(self) -> bool {
        self.node.applies_filter
    }
    pub fn applies_backdrop_filter(self) -> bool {
        self.node.applies_backdrop_filter
    }
    pub fn applies_mask(self) -> bool {
        self.node.applies_mask
    }
    pub fn paints_outline(self) -> bool {
        self.node.paints_outline
    }
    pub fn outline_rects(self) -> &'n Vec<PaintRect> {
        &self.node.outline_rects
    }
    pub fn mask_layers(self) -> &'n Vec<DisplayMaskLayer> {
        &self.node.mask_layers
    }
    /// Fixture copy of LocalBorderBoxProperties.
    pub fn properties(self) -> &'n PrePaintPropertyTreeState {
        &self.node.properties
    }
    /// Fixture copy of LocalBorderBoxProperties().PreTransform().
    pub fn pre_transform(self) -> &'n Arc<TransformPaintPropertyNode> {
        &self.node.pre_transform
    }
    /// Fixture copy of ContentsProperties.
    pub fn contents_properties(self) -> &'n PrePaintPropertyTreeState {
        &self.node.contents_properties
    }
    pub fn fixed_background_properties(self) -> &'n Option<PaintPropertyTreeState> {
        &self.node.fixed_background_properties
    }
    pub fn overflow_controls_properties(self) -> &'n Option<PaintPropertyTreeState> {
        &self.node.overflow_controls_properties
    }
    pub fn horizontal_scrollbar_properties(self) -> &'n Option<PaintPropertyTreeState> {
        &self.node.horizontal_scrollbar_properties
    }
    pub fn vertical_scrollbar_properties(self) -> &'n Option<PaintPropertyTreeState> {
        &self.node.vertical_scrollbar_properties
    }
    pub fn mask_properties(self) -> &'n Option<PaintPropertyTreeState> {
        &self.node.mask_properties
    }
}

impl<'n, 'f> From<&'n PaintTreeNode<'f>> for PaintObject<'n, 'f> {
    fn from(node: &'n PaintTreeNode<'f>) -> Self {
        Self::new(node)
    }
}

/// Iterator over a painted object's children.
#[derive(Clone)]
pub struct PaintObjectChildren<'n, 'f> {
    inner: std::slice::Iter<'n, Box<PaintTreeNode<'f>>>,
}

impl<'n, 'f> PaintObjectChildren<'n, 'f> {
    pub fn is_empty(&self) -> bool {
        self.inner.len() == 0
    }
}

impl<'n, 'f> Iterator for PaintObjectChildren<'n, 'f> {
    type Item = PaintObject<'n, 'f>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|child| PaintObject::new(child))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl DoubleEndedIterator for PaintObjectChildren<'_, '_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|child| PaintObject::new(child))
    }
}

impl ExactSizeIterator for PaintObjectChildren<'_, '_> {}

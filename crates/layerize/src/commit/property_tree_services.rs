//! Non-owning identity used in place of C++ PropertyTrees backpointers.
//! Data and algorithms remain in cc/trees/property_tree.rs. Real-time context
//! borrows resolve this identity; it never stores/copies tree or node snapshots.
pub use compositor_tree::property_tree::PropertyTreesOwnerId;

pub use super::geometry::{OutsetsF, RoundedVector2dF};
pub use compositor_tree::transform::PropertyTreeTransform;

/// External paint operations; implementation belongs to the paint backend.
pub use compositor_tree::effect_node::EffectTreeFilters as PropertyTreeFilters;
// Canonical native value owner; no duplicate flags representation.
pub use compositor_values::render_surface_flags::RenderSurfacePropertyChangedFlags;
pub trait PropertyTreeRenderSurface {
    fn id(&self) -> super::input_types::ElementId;
    fn set_effect_tree_index(&mut self, id: i32);
    fn GetPropertyChangeFlags(&self) -> RenderSurfacePropertyChangedFlags;
    fn ApplyPropertyChangeFlags(&mut self, flags: RenderSurfacePropertyChangedFlags);
    fn ResetPropertyChangedFlags(&mut self);
}
/// The render-surface slot type of the main LayerTreeHost's PropertyTrees.
/// cc::RenderSurfaceImpl exists only on the impl side: the main trees never
/// create one, and LayerTreeImpl::SetPropertyTrees copies only the effect
/// nodes (EffectTree::operator=, cc/trees/property_tree.cc:1716-1723 resizes
/// render_surfaces_ without copying them). Uninhabited, so every main
/// `render_surfaces_` slot is `None` by construction.
#[derive(Debug)]
pub enum NoRenderSurface {}
impl PropertyTreeRenderSurface for NoRenderSurface {
    fn id(&self) -> super::input_types::ElementId {
        match *self {}
    }
    fn set_effect_tree_index(&mut self, _id: i32) {
        match *self {}
    }
    fn GetPropertyChangeFlags(&self) -> RenderSurfacePropertyChangedFlags {
        match *self {}
    }
    fn ApplyPropertyChangeFlags(&mut self, _flags: RenderSurfacePropertyChangedFlags) {
        match *self {}
    }
    fn ResetPropertyChangedFlags(&mut self) {
        match *self {}
    }
}
pub trait PropertyTreeRenderSurfaceFactory<R: PropertyTreeRenderSurface> {
    fn CreateRenderSurface(&mut self, id: super::input_types::ElementId) -> Box<R>;
}
/// Viz requests retain owned lifetime and abort-on-drop behavior in the backend.
pub trait PropertyTreeCopyOutputRequest {
    fn has_area(&self) -> bool;
    fn area(&self) -> super::geometry::Rect;
    fn set_area(&mut self, area: super::geometry::Rect);
    fn is_scaled(&self) -> bool;
    fn has_result_selection(&self) -> bool;
    fn scale_from(&self) -> super::geometry::Vector2d;
    fn scale_to(&self) -> super::geometry::Vector2d;
    fn SetScaleRatio(&mut self, from: super::geometry::Vector2d, to: super::geometry::Vector2d);
}
// The property-tree facet of viz::CopyOutputRequest
// (cc/trees/property_tree.cc EffectTree copy requests).
impl PropertyTreeCopyOutputRequest for viz_common::copy_output_request::CopyOutputRequest {
    fn has_area(&self) -> bool {
        Self::has_area(self)
    }
    fn area(&self) -> super::geometry::Rect {
        *Self::area(self)
    }
    fn set_area(&mut self, area: super::geometry::Rect) {
        Self::set_area(self, area);
    }
    fn is_scaled(&self) -> bool {
        Self::is_scaled(self)
    }
    fn has_result_selection(&self) -> bool {
        Self::has_result_selection(self)
    }
    fn scale_from(&self) -> super::geometry::Vector2d {
        Self::scale_from(self)
    }
    fn scale_to(&self) -> super::geometry::Vector2d {
        Self::scale_to(self)
    }
    fn SetScaleRatio(&mut self, from: super::geometry::Vector2d, to: super::geometry::Vector2d) {
        Self::SetScaleRatio(self, from, to);
    }
}
pub trait PropertyTreeFeatures {
    fn SizeOopifEffectSurfacesAtExternalScale(&self) -> bool;
}

pub trait PropertyTreeScrollLayerTree<T: PropertyTreeTransform> {
    fn DidUpdateScrollOffset(
        &mut self,
        id: super::input_types::ElementId,
        pushed_from_main_or_pending_tree: bool,
        scroll_tree: &crate::commit::property_tree::ScrollTree,
        context: &mut crate::commit::property_tree::ScrollTreeMutationContext<'_, T>,
    );
    fn DidAnimateScrollOffset(&mut self);
}
pub trait PropertyTreeMutatorHost {
    fn ScrollOffsetAnimationWasInterrupted(&self, id: super::input_types::ElementId) -> bool;
}

pub use compositor_tree::target_property::TargetProperties;

//! Value facets that both halves name on the property-tree parameters
//! (`CommitTypes::{Filters, Path, CopyOutputRequest}`): main Layers read them
//! while building property trees and commit state, the impl draw/damage code
//! reads the same values after FinishCommit. Moved from
//! compositor/src/adapter/{draw_property,main_layer}.rs, which re-export them.
use super::{
    geometry::{RRectF, Rect, Vector2dF},
    property_tree_services::PropertyTreeFilters,
    property_types::PropertyPath,
    values::UnguessableToken,
};

/// cc::FilterOperations queries used by cc/trees/draw_property_utils.cc and
/// cc/layers/layer.cc (FilterIsAnimating / HasFilterThatMovesPixels).
pub trait DrawFilters: PropertyTreeFilters {
    fn HasReferenceFilter(&self) -> bool;
    fn HasFilterThatMovesPixels(&self) -> bool;
    fn ExpandRect(&self, rect: Rect, scale: Vector2dF) -> Rect;
    fn HasBlurFilter(&self) -> bool;
}

// External SkPath::RRect(SkRRect(gfx::RRectF)) value construction only.
pub trait MainLayerPath: PropertyPath {
    fn FromRRect(value: &RRectF) -> Self;
}

// Native Viz request identity; result ownership/destruction remains in Viz.
pub trait MainLayerCopyOutputRequest {
    type TaskRunner;
    fn has_result_task_runner(&self) -> bool;
    fn set_result_task_runner(&mut self, runner: Self::TaskRunner);
    fn has_area(&self) -> bool;
    fn area(&self) -> &Rect;
    fn set_area(&mut self, area: Rect);
    fn has_source(&self) -> bool;
    fn source(&self) -> UnguessableToken;
}

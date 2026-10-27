//! Facade compatibility boundary for property-tree values.
//!
//! Tree values and tracing live in `compositor_tree`. The facade combines the
//! tree trace vocabulary with tile-only trace fields for legacy call sites
//! that still serialize both kinds of state into one object.

pub use compositor_tile::tile_priority::TilePriorityTrace;
pub use compositor_tree::property_types::{
    AddCornerRadiiToTracedValue, AddElementIdToTracedValue, AddLinearGradientToTracedValue,
    AddPoint3FToTracedValue, AddPointFToTracedValue, AddRectFToTracedValue,
    AddTransformToTracedValue, AddVector2dFToTracedValue, BlendMode, FilterOperations,
    InlinedVector, MaskFilterInfo, PropertyTrace, PropertyTransform, RectFToString,
    SubtreeCaptureId, TracedValueJSON, ViewTransitionElementResourceId,
};

pub trait PropertyPath:
    compositor_tree::property_types::PropertyPath + super::geometry::Path
{
}

impl<T> PropertyPath for T where
    T: compositor_tree::property_types::PropertyPath + super::geometry::Path
{
}

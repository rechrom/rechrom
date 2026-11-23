//! Property-tree value vocabulary. Tree values and tracing live in
//! `compositor_tree`; moved from compositor/src/adapter/property_types.rs,
//! which re-exports it.

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

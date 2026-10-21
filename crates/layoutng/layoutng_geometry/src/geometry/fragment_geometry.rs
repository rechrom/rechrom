// cpp: layoutng_geometry/geometry/fragment_geometry.h:8-9
use super::box_strut::BoxStrut;
use super::logical_size::LogicalSize;

// cpp: layoutng_geometry/geometry/fragment_geometry.h:13-24
/// Geometry known before layout determines the fragment's final size.
#[derive(Clone, Debug, Default)]
pub struct FragmentGeometry {
    pub border_box_size: LogicalSize,
    pub border: BoxStrut,
    pub scrollbar: BoxStrut,
    pub padding: BoxStrut,
}

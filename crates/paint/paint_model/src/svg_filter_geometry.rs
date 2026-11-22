//! Source filter geometry is shared with native SVG Layout; do not maintain
//! a second bounds algorithm in Paint. Existing package API remains unchanged.
pub use layout::paint::svg_filter_geometry::*;

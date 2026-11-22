//! Hit-test family: the resident hit-test cursor (`hit_test.rs`) and its SVG
//! leaves (`svg_hit_test.rs`). Both are compiled as children of
//! `layout_output` because the cursor reads `LayoutNode`'s private fields, so
//! this module declares nothing. Exposed by `layout::views::hit_test`.

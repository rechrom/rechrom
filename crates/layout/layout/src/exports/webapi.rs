//! Layout items named by webapi, style_*, css_parser, raster and every other consumer.
//!
//! Each submodule feeds the public topic module of the same name in
//! `lib.rs` (`layout::<topic>::Item`). Seeded from the 2026-11-22 consumer
//! inventory; consumer migrations edit only this group's file.

pub mod registry {}

pub mod lifecycle {}

pub mod mutation {}

pub mod input {
    pub use layout_assembly::css_color_parser::ParseCSSColor;
    pub use layout_assembly::internal::css_zoom::ZoomedStyle;
    pub use layout_assembly::internal::layout_input::{
        Display, ExtendedStyle, Offset, PaintPathVerb, Position, TextAlign, TextAlignLast,
        TextDirection,
    };
    pub use layout_assembly::style_projection::{ProjectComputedStyle, ProjectListMarkerImage};
}

pub mod output {}

pub mod fragment {}

pub mod paint {}

pub mod paint_input {
    pub use layout_assembly::internal::boundary::paint_style_export::{
        Export, ExportGeneratedImageShader, SvgColor,
    };
}

pub mod editing {}

pub mod scroll {
    pub use layout_assembly::internal::layout_scrollable_area::{
        ComputeScrollbarWidthsForViewportUnitsFromInputs, StyleBasedScrollbarData,
    };
}

pub mod object {}

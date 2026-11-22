//! Layout items named by dom, document, html, document_loader, document_image and page_mutation.
//!
//! Each submodule feeds the public topic module of the same name in
//! `lib.rs` (`layout::<topic>::Item`). Seeded from the 2026-11-22 consumer
//! inventory; consumer migrations edit only this group's file.

pub mod registry {
    pub use layout_assembly::layout_registry::LayoutRegistry;
}

pub mod lifecycle {
    pub use layout_assembly::layout_engine::LayoutEngineError;
}

pub mod mutation {
    pub use layout_assembly::layout_engine::{LayoutMutation, LayoutTreeUpdate};
    // Chromium's LayoutTreeBuilder (dom_to_layout/layout_object_tree_builder.cc,
    // layout_tree_builder.h) holds the parent LayoutObject* it attaches under
    // and checks its class (LayoutSVGInline::IsChildAllowed). LayoutTreeUpdate
    // lends and returns those parents, so DOM's builder names them here, as
    // part of the boundary-3 attach transaction.
    pub use layout_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};
}

pub mod input {
    pub use layout_assembly::css_color_parser::ParseCSSColor;
    pub use layout_assembly::internal::form_control_types::FormControlType;
    pub use layout_assembly::internal::layout_input::{
        CompatibilityMode, ComputedStyle, ConstraintSpace, ContentLockState, Display,
        DocumentImageContainer, DocumentRole, ElementData, EmbeddedContentKind, ExtendedStyle,
        FocusedImageMapArea, FontFace, FrameDimension, FrameDimensionType, ImageMapAreaShape,
        ListStyleType, MathTokenKind, MediaChildKind, NodeKind, Offset, PaintImage,
        PaintPathCommand, PaintPathVerb, Size, SvgLengthAdjust, SvgMarkerData, SvgMarkerOrient,
        SvgShapeData, SvgShapeGeometry, SvgTextPathData, SvgUseData, SvgViewBoxData,
        SvgViewportData, SvgViewportLength, TextAreaSizing, TextFieldSizing, TrackedElementSubRect,
        VideoLayoutState, ViewportGeometry, WhiteSpace,
    };
    pub use layout_assembly::internal::layout_input_types::{Color, IntSize};
    pub use layout_assembly::style_projection::{ProjectListMarkerImage, StyleResourceBindings};
}

pub mod output {}

pub mod fragment {}

pub mod paint {}

pub mod paint_input {
    pub use layout_assembly::internal::boundary::paint_style_export::StyleColor;
    pub use layout_assembly::internal::paint_input::{
        PaintShader, PaintShaderKind, PaintSpreadMethod, SvgBlendMode, SvgClipGeometry,
        SvgClipLength, SvgClipPathChild, SvgClipPathGraph, SvgClipPathUnits,
        SvgClipResourceBinding, SvgClipResourceEntry, SvgClipResourceSnapshot,
        SvgClipResourceVersion, SvgClipText, SvgClipTextAnchor, SvgClipTextPath, SvgClipTransform,
        SvgClipTransformStep, SvgColorMatrixType, SvgCompositeOperator, SvgFilterChannel,
        SvgFilterEdgeMode, SvgFilterGraph, SvgFilterInput, SvgFilterLength, SvgFilterPrimitive,
        SvgFilterPrimitiveKind, SvgFilterRegion, SvgFilterUnits, SvgLightSource, SvgLightingType,
        SvgMarkerInstance, SvgMaskGraph, SvgMeetOrSlice, SvgMorphologyOperator, SvgPaintServer,
        SvgPreserveAspectRatio, SvgPreserveAspectRatioAlign, SvgTransferFunction,
        SvgTurbulenceType,
    };
}

pub mod editing {
    pub use layout_assembly::editing_state::LayoutEditingState;
    pub use layout_assembly::EditingNodeFacts;
}

pub mod scroll {}

// LayoutObject/LayoutObjectClass moved to `mutation` (tree-builder parents).
pub mod object {}

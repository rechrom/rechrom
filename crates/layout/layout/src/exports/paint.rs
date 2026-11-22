//! Layout items named by paint, prepaint, paint_* and compositor.
//!
//! Each submodule feeds the public topic module of the same name in
//! `lib.rs` (`layout::<topic>::Item`). Seeded from the 2026-11-22 consumer
//! inventory; consumer migrations edit only this group's file.

pub mod registry {}

pub mod lifecycle {}

pub mod mutation {}

pub mod input {
    pub use layout_assembly::internal::layout_input::{
        BorderLineStyle, BoxDecorationBreak, Display, Edges, FloatSide, FocusedImageMapArea,
        FontFace, FontSmoothing, FontVariation, ImageMapAreaShape, ListStyleType, NodeKind, Offset,
        Overflow, OverscrollBehavior, PaintClipPathChild, PaintImage, PaintPathCommand,
        PaintPathVerb, PaintShadow, PaintStyle, Position, Size, SvgShapeData, SvgShapeGeometry,
        SvgViewBoxData, TextDecorationPaint, TextDecorationStyle, TextDirection,
        TrackedElementFrameToken, TrackedElementSubRect, TrackedElementSubRectType,
        TransformMatrix, ViewportGeometry, WritingMode,
    };
    pub use layout_assembly::internal::layout_input_types::{Color, IntSize};
    pub use layout_assembly::internal::style_variant::{StyleVariant, ToParentStyleVariant};
    pub use layout_assembly::internal::used_font::UsedFont;
}

pub mod output {
    pub use layout_assembly::layout_engine::FragmentCursor;
}

pub mod fragment {
    pub use layout_assembly::fragment_item::{FragmentItem, ItemType};
    pub use layout_assembly::fragment_tree::{
        FragmentKind, PrePaintSourceObject, PreviousBoxGeometry, ResidentBoxGeometry,
    };
    pub use layout_assembly::inline_cursor::InlineCursor;
    pub use layout_assembly::layout_text_combine::DispatchIsLayoutTextCombine;
}

pub mod paint {
    pub use layout_assembly::fragment_tree::paint_resource_usage::{
        PaintResourceUsage, SamePaintResourcesFor,
    };
    pub use layout_assembly::fragment_tree::{
        BackgroundPaintLocation, CanvasBackgroundPaintData, CanvasBackgroundPaintView,
        CollapsedTablePaintData, CollapsedTableSection, FieldsetPaintData, FormControlPaintData,
        FragmentBoxSides, FragmentPaintData, FragmentPaintRect, GapRule, GapRulePaintData,
        GapRuleSide, MathMLPaintData, MathMLPaintKind, PageBackgroundPaintData,
        PageContainerBackgroundPaintData, PaintGlyph, PaintGlyphRun, PaintProperties,
        PaintResources, RasterEffectOutset, ReplacedContentPaintData, ScrollContainerPaintData,
        ScrollSnapAxisPaintData, ScrollSnapStrictnessPaintData, ScrollbarPaintAxis,
        ScrollbarPaintData, SelfOutlinePaintData, StickyAxisPaintData, StickyPositionPaintData,
        StitchedDecorationData, SvgViewportGeometry, TablePaintColumn,
        TextCombineEmphasisPaintData, TextCombinePaintData,
    };
    pub use layout_assembly::layout_engine::{
        LayoutPaintOutput, PaintClientCommit, PaintCommitOutput, PaintFragmentCursor,
        PaintFragmentLease, PaintLayoutObjectIdentity,
    };
    pub use layout_assembly::text_decoration_info::{
        CurrentSpellingGrammarLineStyle, CurrentSpellingGrammarThickness,
    };
    // Plain value enum: a LayoutObject's selection state as painters and
    // selection bounds recording read it (core/layout/selection_state.h:13-33).
    pub use layout_assembly::internal::selection_state::SelectionState;
    // Chromium's text decoration painting reads LayoutTheme's platform
    // spelling/grammar marker colours directly
    // (core/paint/text_decoration_info.cc:637-656). This stateless colour
    // theme is the only LayoutTheme surface Paint names.
    pub use layout_assembly::internal::layout_theme::LayoutTheme;
    pub use layout_assembly::{
        decorating_box, inline_paint_context, line_relative_rect, svg_filter_geometry,
        text_decoration_info,
    };
}

pub mod paint_input {
    pub use layout_assembly::internal::background_bleed_avoidance::{
        BackgroundBleedAvoidance, BleedAvoidanceIsClipping,
    };
    pub use layout_assembly::internal::paint_input::{
        BackgroundAttachment, BackgroundBox, BackgroundImageLayer, BackgroundRepeat,
        BackgroundRepeatRule, BackgroundSizeMode, ClipPathCircle, ClipPathLength, ClipPathPaint,
        ClipPathRadius, ClipPathRadiusKind, ClipPathReferenceBox, ClipPathShapeRadius,
        InterpolationQuality, NinePieceImagePaint, NinePieceImageWidth, NinePieceImageWidths,
        PaintBlendMode, PaintCanvasFilter, PaintColorInterpolationSpace, PaintColorStop,
        PaintComponentTransferFunction, PaintComponentTransferType, PaintConvolveEdgeMode,
        PaintCornerCurvatures, PaintCornerRadii, PaintCornerRadius, PaintFilterOperation,
        PaintFilterType, PaintHueInterpolationMethod, PaintImageTileRule, PaintMaskComposite,
        PaintMaskImageState, PaintMaskLayer, PaintMaskMode, PaintRadialExtent, PaintRadialShape,
        PaintReflectionDirection, PaintReflectionMaskLayer, PaintShader, PaintShaderKind,
        PaintSpreadMethod, PaintStyleData, PaintTransform, PaintTransformOperationKind,
        PaintTransformOrigin, PaintTransformReferenceBox, PaintTurbulenceType, SvgClipGeometry,
        SvgClipLength, SvgClipPathGraph, SvgClipPathUnits, SvgClipPreserveAspectRatio, SvgClipText,
        SvgClipTextAnchor, SvgClipTransform, SvgClipTransformStep, SvgClipViewport, SvgFilterGraph,
        SvgFilterInput, SvgFilterLength, SvgFilterPrimitive, SvgFilterPrimitiveKind,
        SvgFilterRegion, SvgFilterUnits, SvgMarkerInstance, SvgMarkerPrimitive, SvgMaskGraph,
        SvgMaskUnits, SvgPaintComponent, SvgPaintServer, SvgPaintServerReferenceBox,
        SvgPatternCommand, SvgPatternData, SvgPatternImage, SvgStrokeLineCap, SvgStrokeLineJoin,
        SvgTurbulenceType,
    };
    pub use layout_assembly::internal::text_decoration_offset::TextDecorationOffset;
}

pub mod editing {
    pub use layout_assembly::caret::geometry::LocalCaretRect;
    pub use layout_assembly::caret::CaretPaintState;
    pub use layout_assembly::editing_state::{
        DocumentCompositionMarker, DocumentSelection, EditingPaintState,
        TextControlSelectionPaintState,
    };
    pub use layout_assembly::internal::caret_rect::CaretShape;
}

pub mod scroll {}

pub mod object {}

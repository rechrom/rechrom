//! Layout items named by browser (Page assembly root) and apps/rechrom.
//!
//! Each submodule feeds the public topic module of the same name in
//! `lib.rs` (`layout::<topic>::Item`). Seeded from the 2026-11-22 consumer
//! inventory; consumer migrations edit only this group's file.

pub mod registry {
    pub use layout_base::layout_registry::LayoutRegistry;
}

pub mod lifecycle {
    pub use layout_base::layout_boundary_support::LayoutUpdateKind;
    pub use layout_base::layout_engine::ScrollLayoutUpdate;
}

pub mod mutation {
    pub use layout_base::layout_engine::{
        HTMLImageResourceUpdate, LayoutMutation, LayoutTreeUpdate,
    };
    // Wave E: scroll
    pub use layout_base::layout_engine::ScrollMutation;
    // Wave E: pointer
    pub use layout_base::engine::pointer::pointer_scroll_controls::PointerMutation;
}

pub mod input {
    pub use layout_base::internal::constraint_space::FragmentationType;
    pub use layout_base::internal::constraint_space_builder::ConstraintSpaceBuilder;
    pub use layout_base::internal::layout_input::{
        ComputedStyle, ConstraintSpace, Display, ElementData, FontFace, FontFallbackRole,
        FontUnicodeRange, FontVariation, NodeKind, Offset, OverscrollType, PaintImage, Size,
        TextOffsetMapping, TextTransform, TextTransformProvider, TextTransformResult,
        TrackedElementFrameToken, TrackedElementRelativeRect, TrackedElementSubRect,
        TrackedElementSubRectType, TransformMatrix, ViewportGeometry,
    };
    pub use layout_base::internal::layout_input_types::{
        Color, ControlThemeMetrics, IntSize, ScrollbarThemeMetrics,
    };
    pub use layout_base::internal::layout_node_metadata::{Element, Node, Text};
    pub use layout_base::internal::loader::fetch::resource_priority::VisibilityStatus;
}

pub mod output {
    pub use layout_base::layout_engine::{
        ContainerQuerySizeSnapshot, LayoutFocusNavigationFacts, LayoutMetric, LayoutRect,
        NativeIntersectionRoot,
    };
}

pub mod fragment {
    pub use layout_base::fragment_tree::{
        FragmentKind, LayoutResultHandle, ResidentBoxGeometry,
    };
    pub use layout_base::layout_result::LayoutResult;
    pub use layout_base::physical_box_fragment::PhysicalBoxFragment;
}

pub mod paint {
    pub use layout_base::fragment_tree::{FragmentPaintData, PaintResources, SvgTextPaintData};
    pub use layout_base::layout_engine::{LayoutPaintOutput, PaintFragmentCursor};
}

pub mod paint_input {
    pub use layout_base::internal::paint_input::{
        ClipPathPaint, SvgClipGeometry, SvgFilterGraph, SvgFilterInput, SvgFilterPrimitiveKind,
        SvgLightSource, SvgMaskGraph, SvgMaskReference, SvgPatternCommand, SvgStrokeLineCap,
        SvgTransferFunction,
    };
}

pub mod editing {
    pub use layout_base::editing_state::{
        DOMSelection, DocumentSelection, LayoutEditingState, Selection, SelectionState,
    };
    pub use layout_base::layout_selection::SelectionStyleAndPaintInvalidation;
    pub use layout_base::{
        EditingPosition, EditingPositionAnchorType, EditingPositionOptions,
        EditingPositionWithAffinity, EditingTextAffinity,
    };
}

pub mod scroll {
    pub use layout_base::internal::layout_scrollable_area::NativeUserSnapScrollKind;
    pub use layout_base::internal::scrollbar::Scrollbar;
    pub use layout_base::layout_engine::SuppressScrollAnchorScope;
    pub use layout_base::engine::scroll::autoscroll_source::{
        NativeAutoscrollLayout, NativeAutoscrollSearch, NativeAutoscrollSource,
        NativeLogicalScrollArea, NativeLogicalScrollBoxFacts, NativeLogicalScrollNodeFacts,
        NativeLogicalScrollingArea,
    };
    pub use layout_base::engine::event_registry::event_registry_source::NativeEventRegistryLayout;
    pub use layout_base::engine::pointer::mouse_coordinates::{
        NativeDragNodeImageGeometry, NativeDragResourceImageFacts, NativeMouseCoordinateLayout,
        NativeRegisteredScrollableArea,
    };
    pub use layout_base::engine::scroll::page_scale_geometry::NativePageScaleGeometry;
    pub use layout_base::engine::pointer::pointer_scroll_controls::{
        NativeMacScrollbarAnimationUpdate, NativeOverlayVisibilityUpdate, NativePointerResizeArea,
        NativeResizePlan, NativeScrollbarInteraction,
    };
    pub use layout_base::engine::scroll::scroll_source::{
        NativeCompositorScrollArea, NativeCompositorScrollUpdate, NativeLayoutSnapPlan,
        NativeScrollFrameGeometry, NativeScrollResult, NativeSnapAnimationState, ScrollAlignment,
        ScrollAlignmentBehavior, ScrollIntoViewParams,
    };
    pub use layout_base::engine::scroll::user_scroll_source::{
        NativeScrollFinishedPlan, NativeUserScrollAnimationRequest, NativeUserScrollAnimationState,
        NativeUserScrollAreaFacts, NativeUserScrollGranularity, NativeUserScrollPlan,
        NativeUserScrollSource,
    };
    pub use layout_base::engine::scroll::wheel_scrollable_traversal::{
        NativeWheelScrollableArea, NativeWheelScrollableAreaTraversal,
    };
}

// Browser production code names no LayoutObject internals; its tests use
// `layout::testing`.
pub mod object {}

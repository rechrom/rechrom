//! Layout items named by browser (Page assembly root) and apps/rechrom.
//!
//! Each submodule feeds the public topic module of the same name in
//! `lib.rs` (`layout::<topic>::Item`). Seeded from the 2026-11-22 consumer
//! inventory; consumer migrations edit only this group's file.

pub mod assembly {
    pub use layout_assembly::engine_assembly::LayoutAssembly;
}

pub mod lifecycle {
    pub use layout_assembly::layout_boundary_support::LayoutUpdateKind;
    pub use layout_assembly::layout_engine::ScrollLayoutUpdate;
}

pub mod mutation {
    pub use layout_assembly::layout_engine::{
        HTMLImageResourceUpdate, LayoutMutation, LayoutTreeUpdate,
    };
    // Wave E: pointer
    pub use layout_assembly::native_pointer_scroll_controls::PointerMutation;
}

pub mod input {
    pub use layout_assembly::internal::constraint_space::FragmentationType;
    pub use layout_assembly::internal::constraint_space_builder::ConstraintSpaceBuilder;
    pub use layout_assembly::internal::layout_input::{
        ComputedStyle, ConstraintSpace, Display, ElementData, FontFace, FontFallbackRole,
        FontUnicodeRange, FontVariation, NodeKind, Offset, OverscrollType, PaintImage, Size,
        TextOffsetMapping, TextTransform, TextTransformProvider, TextTransformResult,
        TrackedElementFrameToken, TrackedElementRelativeRect, TrackedElementSubRect,
        TrackedElementSubRectType, TransformMatrix, ViewportGeometry,
    };
    pub use layout_assembly::internal::layout_input_types::{
        Color, ControlThemeMetrics, IntSize, ScrollbarThemeMetrics,
    };
    pub use layout_assembly::internal::layout_node_metadata::{Element, Node, Text};
    pub use layout_assembly::internal::loader::fetch::resource_priority::VisibilityStatus;
}

pub mod output {
    pub use layout_assembly::layout_engine::{
        ContainerQuerySizeSnapshot, LayoutFocusNavigationFacts, LayoutMetric, LayoutRect,
        NativeIntersectionRoot,
    };
}

pub mod fragment {
    pub use layout_assembly::fragment_tree::{
        FragmentKind, LayoutResultHandle, ResidentBoxGeometry,
    };
    pub use layout_assembly::layout_result::LayoutResult;
    pub use layout_assembly::physical_box_fragment::PhysicalBoxFragment;
}

pub mod paint {
    pub use layout_assembly::fragment_tree::{FragmentPaintData, PaintResources, SvgTextPaintData};
    pub use layout_assembly::layout_engine::{LayoutPaintOutput, PaintFragmentCursor};
}

pub mod paint_input {
    pub use layout_assembly::internal::paint_input::{
        ClipPathPaint, SvgClipGeometry, SvgFilterGraph, SvgFilterInput, SvgFilterPrimitiveKind,
        SvgLightSource, SvgMaskGraph, SvgMaskReference, SvgPatternCommand, SvgStrokeLineCap,
        SvgTransferFunction,
    };
}

pub mod editing {
    pub use layout_assembly::editing_state::{
        DOMSelection, DocumentSelection, LayoutEditingState, Selection, SelectionState,
    };
    pub use layout_assembly::layout_selection::SelectionStyleAndPaintInvalidation;
    pub use layout_assembly::{
        EditingPosition, EditingPositionAnchorType, EditingPositionOptions,
        EditingPositionWithAffinity, EditingTextAffinity,
    };
}

pub mod scroll {
    pub use layout_assembly::internal::layout_scrollable_area::NativeUserSnapScrollKind;
    pub use layout_assembly::internal::scrollbar::Scrollbar;
    pub use layout_assembly::layout_engine::SuppressScrollAnchorScope;
    pub use layout_assembly::native_autoscroll_source::{
        NativeAutoscrollLayout, NativeAutoscrollSearch, NativeAutoscrollSource,
        NativeLogicalScrollArea, NativeLogicalScrollBoxFacts, NativeLogicalScrollNodeFacts,
        NativeLogicalScrollingArea,
    };
    pub use layout_assembly::native_event_registry_source::NativeEventRegistryLayout;
    pub use layout_assembly::native_mouse_coordinates::{
        NativeDragNodeImageGeometry, NativeDragResourceImageFacts, NativeMouseCoordinateLayout,
        NativeRegisteredScrollableArea,
    };
    pub use layout_assembly::native_page_scale_geometry::NativePageScaleGeometry;
    pub use layout_assembly::native_pointer_scroll_controls::{
        NativeMacScrollbarAnimationUpdate, NativeOverlayVisibilityUpdate, NativePointerResizeArea,
        NativeResizePlan, NativeScrollbarInteraction,
    };
    pub use layout_assembly::native_scroll_source::{
        NativeCompositorScrollArea, NativeCompositorScrollUpdate, NativeLayoutSnapPlan,
        NativeScrollFrameGeometry, NativeScrollResult, NativeSnapAnimationState, ScrollAlignment,
        ScrollAlignmentBehavior, ScrollIntoViewParams,
    };
    pub use layout_assembly::native_user_scroll_source::{
        NativeScrollFinishedPlan, NativeUserScrollAnimationRequest, NativeUserScrollAnimationState,
        NativeUserScrollAreaFacts, NativeUserScrollGranularity, NativeUserScrollPlan,
        NativeUserScrollSource,
    };
    pub use layout_assembly::native_wheel_scrollable_traversal::{
        NativeWheelScrollableArea, NativeWheelScrollableAreaTraversal,
    };
}

pub mod object {
    // Grid LayoutObject downcast target for the browser grid geometry tests.
    pub use layout_algo_grid::layout_grid::LayoutGrid;
    pub use layout_assembly::internal::block_node::BlockNode;
    pub use layout_assembly::internal::hit_test_phase::HitTestPhase;
    pub use layout_assembly::internal::layout_block_flow::LayoutBlockFlow;
    pub use layout_assembly::internal::layout_box::LayoutBox;
    pub use layout_assembly::internal::layout_box_model_object::LayoutBoxModelObject;
    pub use layout_assembly::internal::layout_object::{LayoutObject, StyleChangeContext};
    pub use layout_assembly::internal::layout_object_builder::LayoutObjectTree;
    pub use layout_assembly::internal::layout_pass_scope::{
        LayoutObjectFactoryScope, LayoutPassScope, SVGResourceQueryScope,
    };
    pub use layout_assembly::internal::layout_replaced::{LayoutInputReplaced, LayoutReplaced};
    pub use layout_assembly::internal::layout_view::LayoutView;
    pub use layout_assembly::internal::selection_state::SelectionState;
}

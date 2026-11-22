//! Layout items named by interaction, interaction_hit_test, interaction_default and interaction_dispatch.
//!
//! Each submodule feeds the public topic module of the same name in
//! `lib.rs` (`layout::<topic>::Item`). Seeded from the 2026-11-22 consumer
//! inventory; consumer migrations edit only this group's file.

pub mod assembly {}

pub mod lifecycle {}

pub mod mutation {}

pub mod input {
    pub use layout_assembly::internal::layout_input::{
        Display, FloatSide, NodeKind, Offset, Position, Size,
    };
    pub use layout_assembly::internal::layout_node_metadata::Node;
}

pub mod output {
    pub use layout_assembly::layout_engine::{
        FragmentCursor, LayoutFocusNavigationFacts, LayoutNode, LayoutOutput, NativeHitTestCursor,
        NativeHitTestInlineItem, NativeHitTestTarget, TableFocusgroupCursor,
    };
}

pub mod fragment {
    pub use layout_assembly::fragment_item::FragmentItem;
    pub use layout_assembly::fragment_tree::{FragmentKind, ResidentBoxGeometry};
    pub use layout_assembly::inline_cursor::{InlineBackwardCursor, InlineCursor};
    pub use layout_assembly::physical_box_fragment::PhysicalBoxFragment;
    pub use layout_assembly::physical_fragment::PhysicalFragment;
}

pub mod paint {
    pub use layout_assembly::fragment_tree::{FragmentPaintData, FragmentPaintRect};
    pub use layout_assembly::layout_engine::PaintFragmentCursor;
}

pub mod paint_input {
    pub use layout_assembly::internal::paint_input::ECursor;
}

pub mod editing {
    pub use layout_assembly::caret::TextCaret;
    pub use layout_assembly::editing_state::{
        DOMSelection, DocumentCompositionMarker, DocumentSelectionPosition, Selection,
        SelectionState,
    };
    pub use layout_assembly::layout_engine::{
        EditingBidiRangeAdjustment, EditingInlineContentsBlockIdentity, EditingLinePositionResult,
        EditingTextIteratorBehavior, EditingTextRunCursor, VisualCaretMoveDirection,
    };
    pub use layout_assembly::layout_selection::LayoutSelectionRange;
    pub use layout_assembly::password_echo::{
        PasswordEchoLayoutSettings, PasswordEchoTimerScheduler, PasswordEchoTimerToken,
    };
    pub use layout_assembly::{
        EditingPosition, EditingPositionAnchorType, EditingPositionOptions,
        EditingPositionWithAffinity, EditingTextAffinity,
    };
}

pub mod scroll {
    pub use layout_assembly::internal::layout_scrollable_area::{
        NativeScrollbarActivity, NativeScrollbarActivityKind, NativeUserSnapScrollKind,
        OverflowControlHit, PaintLayerScrollableArea, ResizerHitTestType,
    };
    pub use layout_assembly::internal::scrollbar::Scrollbar;
    pub use layout_assembly::internal::scrollbar_orientation::ScrollbarOrientation;
    pub use layout_assembly::internal::scrollbar_part::ScrollbarPart;
    pub use layout_assembly::layout_engine::SuppressScrollAnchorScope;
    pub use layout_assembly::native_autoscroll_source::{
        NativeAutoscrollLayout, NativeLogicalScrollArea, NativeLogicalScrollBoxFacts,
        NativeLogicalScrollNodeFacts, NativeLogicalScrollingArea,
    };
    pub use layout_assembly::native_mouse_coordinates::NativeMouseCoordinateLayout;
    pub use layout_assembly::native_pointer_scroll_controls::{
        NativePointerResizeArea, NativeScrollbarInteraction,
    };
    pub use layout_assembly::native_scroll_source::{
        NativeLayoutSnapPlan, NativeSnapAnimationState,
    };
    pub use layout_assembly::native_user_scroll_source::{
        NativeScrollCompletionMode, NativeScrollConsumption, NativeScrollFinishedPlan,
        NativeUserScrollAnimationRequest, NativeUserScrollAnimationState,
        NativeUserScrollAreaFacts, NativeUserScrollGranularity, NativeUserScrollPlan,
    };
    pub use layout_assembly::native_wheel_scrollable_traversal::NativeWheelScrollableArea;
}

// Hit-test owners and values moved to `layout::views::hit_test`.
pub mod object {}

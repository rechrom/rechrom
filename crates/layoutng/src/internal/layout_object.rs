#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use foundation::blink_geometry::transforms::affine_transform::AffineTransform;
use foundation::graphics_types;
use foundation::{
    gfx, Color, CompositingReasons, DynamicTo, EBackfaceVisibility, EColumnSpan, EDisplay,
    EOverflow, EOverscrollContainerType, EPosition, EVisibility, IsFullPaintInvalidationReason,
    IsLayoutHeapSweepingOnOwningThread, MakeGarbageCollected, Member, PaintInvalidationReason,
    PhysicalOffset, PhysicalRect, RespectImageOrientationEnum, RuntimeEnabledFeatures,
    String as BlinkString, StringBuilder, SubtreePaintPropertyUpdateReason, ThreadAffinity,
    ThreadingTrait, TouchAction, TransformState, UniqueObjectId, Vector, Visitor,
};
use graphics_types::graphics::paint::display_item_client::DisplayItemClient;
use graphics_types::graphics::visual_rect_flags::VisualRectFlags;
use layoutng_fragment_tree::fragment_data::{FragmentData, FragmentDataList};
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::axis::PhysicalAxis;
use layoutng_geometry::geometry::overflow_clip_axes::{
    kNoOverflowClip, kOverflowClipBothAxis, OverflowClipAxes,
};
use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};
use layoutng_style::style::computed_style_constants::PseudoId;
use layoutng_style::style::cursor_list::CursorList;
use layoutng_style::style::fill_layer::FillLayer;
use layoutng_style::style::forward::Longhand;
use layoutng_style::style::image_resource_observer::{
    ImageResourceObserver, ImageResourceObserverVTable,
};
use layoutng_style::style::outline_type::OutlineType;
use layoutng_style::style::shape_value::ShapeValue;
use layoutng_style::style::style_difference::StyleDifference;
use layoutng_style::style::style_image::StyleImage;
use std::cell::Cell;
use std::ffi::c_char;
use std::sync::atomic::{AtomicU32, Ordering};

static LAYOUT_OBJECT_IMAGE_OBSERVER_VTABLE: ImageResourceObserverVTable =
    ImageResourceObserverVTable {
        DebugName: |receiver| unsafe { &*receiver.cast::<LayoutObject>() }.DebugName(),
    };

use super::caret_rect::CaretShape;
use super::editing::forward::{PositionWithAffinity, TextAffinity};
use super::hit_test_phase::HitTestPhase;
use super::layout_block::LayoutBlock;
use super::layout_block_flow::LayoutBlockFlow;
use super::layout_box::LayoutBox;
use super::layout_box_model_object::{LayoutBoxModelObject, PaintLayer};
use super::layout_input::{CompatibilityMode, DocumentRole, ViewportGeometry};
use super::layout_input_types::ControlThemeMetrics;
use super::layout_node_metadata::{ContentLockBlocksChildLayout, Element, Node};
use super::layout_object_child_list::LayoutObjectChildList;
// cpp: layoutng/internal/layout_object.h:278-278
pub use super::loader::resource::image_resource_observer::CanDeferInvalidation;
use super::map_coordinates_flags::MapCoordinatesFlags;
use super::outline_info::LayoutOutlineInfo;
use super::outline_rect_collector::OutlineRectCollector;
use super::pre_paint_disable_side_effects_scope::PrePaintDisableSideEffectsScope;
use super::pre_paint_subtree_walk_reasons::PrePaintSubtreeWalkReasons;
use super::scrollbar_theme_metrics::{RequireScrollbarTheme, ScrollbarThemeMetrics};
use super::selection_state::SelectionState;
use super::style_variant::StyleVariant;

// These are forward declarations in the owning C++ header. Their concrete
// layouts belong to the hit-test and paint implementations.
// cpp: layoutng/internal/layout_object.h:89-91
pub enum HitTestLocation {}
pub enum HitTestRequest {}
pub enum HitTestResult {}

// cpp: layoutng/internal/layout_object.h:98-99
pub enum PaintInfo {}
pub enum PaintInvalidatorContext {}

// cpp: layoutng/internal/layout_object.h:97-97
pub enum StyleRequest {}

// cpp: layoutng/internal/layout_object.h:84-87
pub enum DisplayLockContext {}
pub enum AccompaniedFragmentIterator {}
// cpp: layoutng/internal/layout_object.h:83-83
pub enum PropertyTreeStateOrAlias {}

// cpp: layoutng/internal/layout_object.h:2700-2700
pub type OutlineInfo = LayoutOutlineInfo;

// cpp: layoutng/internal/layout_object.h:2461-2462
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IncludeDescendants(pub bool);

// This large owning header remains in progress. Independent value types and
// behaviors are mapped first; the bitfields, virtual interface, and member
// methods still require translation from layout_object.h.

// cpp: layoutng/internal/layout_object.h:105-110
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoxQuadType {
    kMargin,
    kBorder,
    kPadding,
    kContent,
}

// cpp: layoutng/internal/layout_object.h:112-113
pub fn LocalRectForBoxQuad(fragment: &PhysicalBoxFragment, box_type: BoxQuadType) -> PhysicalRect {
    unsafe { LocalRectForBoxQuadProvider(fragment, box_type) }
}

// cpp: layoutng/internal/layout_object.h:115-115
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorDirective {
    kSetCursorBasedOnStyle,
    kSetCursor,
    kDoNotSetCursor,
}

// cpp: layoutng/internal/layout_object.h:117-120
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarkingBehavior {
    kMarkOnlyThis,
    kMarkContainerChain,
}

// cpp: layoutng/internal/layout_object.h:122-122
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduleRelayoutBehavior {
    kScheduleRelayout,
    kDontScheduleRelayout,
}

// cpp: layoutng/internal/layout_object.h:124-133
pub type BackgroundPaintLocation = u32;
pub const kBackgroundPaintInBorderBoxSpace: BackgroundPaintLocation = 1 << 0;
pub const kBackgroundPaintInContentsSpace: BackgroundPaintLocation = 1 << 1;
pub const kBackgroundPaintInBothSpaces: BackgroundPaintLocation =
    kBackgroundPaintInBorderBoxSpace | kBackgroundPaintInContentsSpace;

// cpp: layoutng/internal/layout_object.h:418-421
pub const kLayoutObjectDepthBits: u32 = 10;
pub const kMaxLayoutObjectDepth: u32 = (1u32 << kLayoutObjectDepthBits) - 1;

// cpp: layoutng/internal/layout_object.h:135-143
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DraggableRegionValue {
    pub bounds: PhysicalRect,
    pub draggable: bool,
}

// cpp: layoutng/internal/layout_object.h:149-151
#[cfg(debug_assertions)]
pub const kShowTreeCharacterOffset: i32 = 39;

// cpp: layoutng/internal/layout_object.h:165-183
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RecalcScrollableOverflowResult {
    pub scrollable_overflow_changed: bool,
    pub rebuild_fragment_tree: bool,
}

impl RecalcScrollableOverflowResult {
    // cpp: layoutng/internal/layout_object.h:179-182
    pub fn Unite(&mut self, other: &Self) {
        self.scrollable_overflow_changed |= other.scrollable_overflow_changed;
        self.rebuild_fragment_tree |= other.rebuild_fragment_tree;
    }
}

// cpp: layoutng/internal/layout_object.h:1262-1265
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DescendantIsolationState {
    kDescendantIsolationRequired,
    kDescendantIsolationNeedsUpdate,
}

// cpp: layoutng/internal/layout_object.h:1923-1928
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionedState {
    kIsStaticallyPositioned = 0,
    kIsRelativelyPositioned = 1,
    kIsOutOfFlowPositioned = 2,
    kIsStickyPositioned = 3,
}

// cpp: layoutng/internal/layout_object.h:2203-2203
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyStyleChanges {
    kNo,
    kYes,
}

// cpp: layoutng/internal/layout_object.h:3375-3379
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StyleChangeContext {
    pub did_prevent_spanner_descendants: bool,
}

// cpp: layoutng/internal/layout_object.h:2875-2878
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OverflowRecalcType {
    kOnlyVisualOverflowRecalc,
    kLayoutAndVisualOverflowRecalc,
}

// Rust stores the source's bitfield values separately; the C++ bit packing is
// not an interop ABI in this standalone crate. Their accessors and mutation
// paths remain to be mapped before this owning header is complete.
// cpp: layoutng/internal/layout_object.h:3589-3909
pub(crate) struct LayoutObjectBitfields {
    pub(crate) paint_invalidation_reason_for_pre_paint_: u8,
    positioned_state_: PositionedState,
    selection_state_: u8,
    selection_state_for_paint_: u8,
    subtree_paint_property_update_reasons_: u8,
    background_paint_location_: BackgroundPaintLocation,
    overflow_clip_axes_: u8,
    #[cfg(debug_assertions)]
    has_ax_object_: bool,
    #[cfg(debug_assertions)]
    pub(crate) set_needs_layout_forbidden_: bool,
    #[cfg(debug_assertions)]
    is_in_detached_non_dom_tree_: bool,
    #[cfg(debug_assertions)]
    as_image_observer_count_: u32,
    self_needs_full_layout_: bool,
    child_needs_full_layout_: bool,
    needs_simplified_layout_: bool,
    self_needs_scrollable_overflow_recalc_: bool,
    child_needs_scrollable_overflow_recalc_: bool,
    pub(crate) intrinsic_logical_widths_dirty_: bool,
    pub(crate) intrinsic_logical_widths_depends_on_block_constraints_: bool,
    pub(crate) indefinite_intrinsic_logical_widths_dirty_: bool,
    pub(crate) definite_intrinsic_logical_widths_dirty_: bool,
    pub(crate) needs_collect_inlines_: bool,
    pub(crate) should_check_for_paint_invalidation_: bool,
    pub(crate) subtree_should_check_for_paint_invalidation_: bool,
    pub(crate) should_delay_full_paint_invalidation_: bool,
    pub(crate) subtree_should_do_full_paint_invalidation_: bool,
    may_need_paint_invalidation_animated_background_image_: bool,
    should_invalidate_selection_: bool,
    pub(crate) should_check_layout_for_paint_invalidation_: bool,
    pub(crate) descendant_should_check_layout_for_paint_invalidation_: bool,
    needs_paint_property_update_: bool,
    descendant_needs_paint_property_update_: bool,
    is_floating_: bool,
    is_anonymous_: bool,
    is_inline_: bool,
    pub(crate) is_in_layout_ng_inline_formatting_context_: bool,
    is_horizontal_writing_mode_: bool,
    has_layer_: bool,
    has_non_visible_overflow_: bool,
    has_transform_related_property_: bool,
    has_reflection_: bool,
    can_contain_absolute_position_objects_: bool,
    can_contain_fixed_position_objects_: bool,
    ever_had_layout_: bool,
    pub(crate) is_inside_multicol_: bool,
    inside_inactive_column_tab_: bool,
    pub(crate) subtree_change_listener_registered_: bool,
    pub(crate) notified_of_subtree_change_: bool,
    pub(crate) consumes_subtree_change_notification_: bool,
    children_inline_: bool,
    pub(crate) always_create_line_boxes_for_layout_inline_: bool,
    background_is_known_to_be_obscured_: bool,
    is_background_attachment_fixed_object_: bool,
    can_composite_background_attachment_fixed_: bool,
    is_scroll_anchor_object_: bool,
    scroll_anchor_disabling_style_changed_: bool,
    should_skip_layout_cache_: bool,
    pub(crate) has_box_decoration_background_: bool,
    background_needs_full_paint_invalidation_: bool,
    outline_may_be_affected_by_descendants_: bool,
    previous_outline_may_be_affected_by_descendants_: bool,
    previous_visibility_visible_: bool,
    is_truncated_: bool,
    pre_paint_subtree_walk_reasons_: u8,
    descendant_pre_paint_subtree_walk_reasons_: u8,
    inside_blocking_touch_event_handler_: bool,
    inside_blocking_wheel_event_handler_: bool,
    should_inherit_soft_navigation_context_: bool,
    should_inherit_container_timing_root_: bool,
    is_effective_root_scroller_: bool,
    is_global_root_scroller_: bool,
    registered_as_first_line_image_observer_: bool,
    being_destroyed_: bool,
    is_table_column_constraints_dirty_: bool,
    is_grid_placement_dirty_: bool,
    is_subgrid_min_max_sizes_cache_dirty_: bool,
    transform_affects_vector_effect_: bool,
    svg_descendant_may_have_transform_related_operations_: bool,
    has_viewport_dependence_: bool,
    should_skip_next_layout_shift_tracking_: bool,
    should_assume_paint_offset_translation_for_layout_shift_tracking_: bool,
    can_traverse_physical_fragments_: bool,
    whitespace_children_may_change_: bool,
    needs_devtools_info_: bool,
    may_contain_anchor_: bool,
    has_broken_spine_: bool,
    has_valid_cached_geometry_: Cell<bool>,
    scrollable_area_size_changed_: bool,
    may_be_non_contiguous_ifc_: bool,
    has_svg_text_descendants_: bool,
    pub(crate) is_multicol_container_: bool,
    contains_selection_focus_: bool,
    is_active_unbounded_element_or_descendant_: bool,
    depth_: u16,
}

impl Default for LayoutObjectBitfields {
    // cpp: layoutng/internal/layout_object.h:3589-3909
    fn default() -> Self {
        Self {
            paint_invalidation_reason_for_pre_paint_: 0,
            positioned_state_: PositionedState::kIsStaticallyPositioned,
            selection_state_: 0,
            selection_state_for_paint_: 0,
            subtree_paint_property_update_reasons_: 0,
            background_paint_location_: kBackgroundPaintInBorderBoxSpace,
            overflow_clip_axes_: 0,
            #[cfg(debug_assertions)]
            has_ax_object_: false,
            #[cfg(debug_assertions)]
            set_needs_layout_forbidden_: false,
            #[cfg(debug_assertions)]
            is_in_detached_non_dom_tree_: false,
            #[cfg(debug_assertions)]
            as_image_observer_count_: 0,
            self_needs_full_layout_: false,
            child_needs_full_layout_: false,
            needs_simplified_layout_: false,
            self_needs_scrollable_overflow_recalc_: false,
            child_needs_scrollable_overflow_recalc_: false,
            intrinsic_logical_widths_dirty_: false,
            intrinsic_logical_widths_depends_on_block_constraints_: true,
            indefinite_intrinsic_logical_widths_dirty_: true,
            definite_intrinsic_logical_widths_dirty_: true,
            needs_collect_inlines_: false,
            should_check_for_paint_invalidation_: true,
            subtree_should_check_for_paint_invalidation_: false,
            should_delay_full_paint_invalidation_: false,
            subtree_should_do_full_paint_invalidation_: false,
            may_need_paint_invalidation_animated_background_image_: false,
            should_invalidate_selection_: false,
            should_check_layout_for_paint_invalidation_: true,
            descendant_should_check_layout_for_paint_invalidation_: true,
            needs_paint_property_update_: true,
            descendant_needs_paint_property_update_: true,
            is_floating_: false,
            is_anonymous_: false,
            is_inline_: true,
            is_in_layout_ng_inline_formatting_context_: false,
            is_horizontal_writing_mode_: true,
            has_layer_: false,
            has_non_visible_overflow_: false,
            has_transform_related_property_: false,
            has_reflection_: false,
            can_contain_absolute_position_objects_: false,
            can_contain_fixed_position_objects_: false,
            ever_had_layout_: false,
            is_inside_multicol_: false,
            inside_inactive_column_tab_: false,
            subtree_change_listener_registered_: false,
            notified_of_subtree_change_: false,
            consumes_subtree_change_notification_: false,
            children_inline_: false,
            always_create_line_boxes_for_layout_inline_: false,
            background_is_known_to_be_obscured_: false,
            is_background_attachment_fixed_object_: false,
            can_composite_background_attachment_fixed_: false,
            is_scroll_anchor_object_: false,
            scroll_anchor_disabling_style_changed_: false,
            should_skip_layout_cache_: false,
            has_box_decoration_background_: false,
            background_needs_full_paint_invalidation_: true,
            outline_may_be_affected_by_descendants_: false,
            previous_outline_may_be_affected_by_descendants_: false,
            previous_visibility_visible_: false,
            is_truncated_: false,
            pre_paint_subtree_walk_reasons_: 0b1111,
            descendant_pre_paint_subtree_walk_reasons_: 0,
            inside_blocking_touch_event_handler_: false,
            inside_blocking_wheel_event_handler_: false,
            should_inherit_soft_navigation_context_: true,
            should_inherit_container_timing_root_: true,
            is_effective_root_scroller_: false,
            is_global_root_scroller_: false,
            registered_as_first_line_image_observer_: false,
            being_destroyed_: false,
            is_table_column_constraints_dirty_: false,
            is_grid_placement_dirty_: true,
            is_subgrid_min_max_sizes_cache_dirty_: true,
            transform_affects_vector_effect_: false,
            svg_descendant_may_have_transform_related_operations_: false,
            has_viewport_dependence_: false,
            should_skip_next_layout_shift_tracking_: true,
            should_assume_paint_offset_translation_for_layout_shift_tracking_: false,
            can_traverse_physical_fragments_: true,
            whitespace_children_may_change_: false,
            needs_devtools_info_: false,
            may_contain_anchor_: false,
            has_broken_spine_: false,
            has_valid_cached_geometry_: Cell::new(false),
            scrollable_area_size_changed_: false,
            may_be_non_contiguous_ifc_: false,
            has_svg_text_descendants_: false,
            is_multicol_container_: false,
            contains_selection_focus_: false,
            is_active_unbounded_element_or_descendant_: false,
            depth_: 0,
        }
    }
}

// LayoutObject is abstract in C++ (GetName is pure virtual). It is not given
// a standalone constructor here; concrete layout classes own its base value.
// DisplayItemClient precedes LayoutObject's own fields as the source base does.
// cpp: layoutng/internal/layout_object.h:280-280,3917-3927,3931
// C++ identifies the most-derived object through its vtable. Rust's base-first
// representation needs an explicit tag for equivalent virtual dispatch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutObjectClass {
    Base,
    BoxModelObject,
    Box,
    Block,
    FlexibleBox,
    Grid,
    Table,
    TableCaption,
    TableCell,
    TableColumn,
    TableRow,
    TableSection,
    BlockFlow,
    ListItem,
    InlineListItem,
    InsideListMarker,
    OutsideListMarker,
    TextCombine,
    // Most-derived vtable identities from //src/layoutng_forms. The concrete
    // objects stay in that crate; this base tag drives virtual dispatch.
    TextControlSingleLine,
    TextControlMultiLine,
    TextControlInnerEditor,
    Fieldset,
    SvgGroup,
    SvgShape,
    SvgRoot,
    SvgForeignObject,
    SvgInline,
    SvgTSpan,
    SvgText,
    SvgInlineText,
    SvgTextPath,
    Replaced,
    View,
    Inline,
    Text,
}

impl LayoutObjectClass {
    pub fn IsTableClass(self) -> bool {
        matches!(
            self,
            Self::Table
                | Self::TableCaption
                | Self::TableCell
                | Self::TableColumn
                | Self::TableRow
                | Self::TableSection
        )
    }
}

#[repr(C)]
pub struct LayoutObject {
    display_item_client_: DisplayItemClient,
    image_resource_observer_: ImageResourceObserver,
    runtime_class_: Cell<LayoutObjectClass>,
    pub(crate) bitfields_: LayoutObjectBitfields,
    style_: Member<ComputedStyle>,
    node_: Member<Node>,
    anonymous_input_owner_: Member<Node>,
    parent_: Member<LayoutObject>,
    previous_: Member<LayoutObject>,
    next_: Member<LayoutObject>,
    fragment_: Member<FragmentDataList>,
    #[cfg(debug_assertions)]
    is_destroyed_: bool,
}

// cpp: layoutng/internal/layout_object.h:3935-3939
#[allow(non_upper_case_globals)]
impl ThreadingTrait for LayoutObject {
    const kAffinity: ThreadAffinity = ThreadAffinity::kMainThreadOnly;
}

// cpp: layoutng/internal/layout_object.h:3941-3943
impl PartialEq for LayoutObject {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for LayoutObject {}

// C++ also defines reference/pointer and pointer/reference overloads. Rust
// names them explicitly while retaining address comparison.
// cpp: layoutng/internal/layout_object.h:3941-3943
pub fn LayoutObjectReferenceEqualsPointer(
    object: &LayoutObject,
    other: *const LayoutObject,
) -> bool {
    std::ptr::eq(object, other)
}

// cpp: layoutng/internal/layout_object.h:3941-3943
pub fn LayoutObjectPointerEqualsReference(
    object: *const LayoutObject,
    other: &LayoutObject,
) -> bool {
    std::ptr::eq(object, other)
}

// Destroy() is invoked through the base pointer but WillBeDestroyed() is a
// virtual call. The concrete layout-object family must provide this dispatch
// when its owners are connected; a base-only call would skip derived cleanup.
// cpp: layoutng/internal/layout_object_fragment_data.cc:89-102
unsafe extern "Rust" {
    fn LayoutObjectIsEligibleForSizeContainmentProvider(object: &LayoutObject) -> bool;
    fn DispatchLayoutObjectIsEligibleForPaintOrLayoutContainment(object: &LayoutObject) -> bool;
    fn DispatchLayoutObjectRespectsCSSOverflow(object: &LayoutObject) -> bool;
    fn DispatchLayoutObjectIsVideo(object: &LayoutObject) -> bool;
    fn DispatchLayoutObjectIsLayoutEmbeddedContent(object: &LayoutObject) -> bool;
    // LocalToAncestorQuad is declared but not defined in the supplied packages.
    fn DispatchLayoutObjectLocalToAncestorQuad(
        object: &LayoutObject,
        quad: &gfx::QuadF,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
    ) -> gfx::QuadF;
    fn DispatchLayoutObjectAncestorToLocalQuad(
        object: &LayoutObject,
        ancestor: *const LayoutBoxModelObject,
        quad: &gfx::QuadF,
        mode: MapCoordinatesFlags,
    ) -> gfx::QuadF;
    fn LayoutObjectAncestorToLocalPointFProvider(
        object: &LayoutObject,
        ancestor: *const LayoutBoxModelObject,
        point: &gfx::PointF,
        mode: MapCoordinatesFlags,
    ) -> gfx::PointF;
    fn LayoutObjectLocalToAncestorRectProvider(
        object: &LayoutObject,
        rect: &PhysicalRect,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
    ) -> PhysicalRect;
    fn LayoutObjectLocalToAncestorTransformProvider(
        object: &LayoutObject,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
    ) -> gfx::Transform;
    fn LayoutObjectContainingScrollContainerAnyAxisProvider(
        object: &LayoutObject,
        ignore_layout_view_for_fixed_pos: bool,
    ) -> *const LayoutBox;
    fn LayoutObjectContainingScrollContainerLayerAnyAxisProvider(
        object: &LayoutObject,
        ignore_layout_view_for_fixed_pos: bool,
    ) -> *const PaintLayer;
    fn LayoutObjectContainingScrollContainerLayerForAxisProvider(
        object: &LayoutObject,
        axis: PhysicalAxis,
        ignore_layout_view_for_fixed_pos: bool,
    ) -> *const PaintLayer;
    fn LayoutObjectOffsetFromAncestorProvider(
        object: &LayoutObject,
        ancestor: *const LayoutObject,
    ) -> PhysicalOffset;
    fn LayoutObjectAbsoluteBoundingBoxRectFProvider(
        object: &LayoutObject,
        mode: MapCoordinatesFlags,
    ) -> gfx::RectF;
    fn LayoutObjectAbsoluteBoundingBoxRectProvider(
        object: &LayoutObject,
        mode: MapCoordinatesFlags,
    ) -> gfx::Rect;
    fn LayoutObjectAbsoluteBoundingBoxRectForUnboundedElementProvider(
        object: &LayoutObject,
    ) -> gfx::Rect;
    fn LayoutObjectAbsoluteBoundingBoxRectHandlingEmptyInlineProvider(
        object: &LayoutObject,
        flags: MapCoordinatesFlags,
    ) -> PhysicalRect;
    fn LayoutObjectAbsoluteBoundingBoxRectForScrollIntoViewProvider(
        object: &LayoutObject,
    ) -> PhysicalRect;
    fn LayoutObjectAbsoluteBoundingBoxRectIncludingDescendantsProvider(
        object: &LayoutObject,
    ) -> gfx::Rect;
    fn LayoutObjectGetCachedPseudoElementStyleProvider(
        object: &LayoutObject,
        pseudo_id: PseudoId,
    ) -> *const ComputedStyle;
    fn LayoutObjectGetUncachedPseudoElementStyleProvider(
        object: &LayoutObject,
        request: &StyleRequest,
    ) -> *const ComputedStyle;
    fn DispatchLayoutObjectPositionForPoint(
        object: &LayoutObject,
        point: &PhysicalOffset,
    ) -> PositionWithAffinity;
    fn LayoutObjectCreatePositionWithAffinityProvider(
        object: &LayoutObject,
        offset: i32,
        affinity: TextAffinity,
    ) -> PositionWithAffinity;
    fn LayoutObjectCreatePositionWithAffinityDefaultProvider(
        object: &LayoutObject,
        offset: i32,
    ) -> PositionWithAffinity;
    fn LayoutObjectFindPositionProvider(object: &LayoutObject) -> PositionWithAffinity;
    fn LayoutObjectFirstPositionInOrBeforeThisProvider(
        object: &LayoutObject,
    ) -> PositionWithAffinity;
    fn LayoutObjectLastPositionInOrAfterThisProvider(object: &LayoutObject)
        -> PositionWithAffinity;
    fn LayoutObjectPositionAfterThisProvider(object: &LayoutObject) -> PositionWithAffinity;
    fn LayoutObjectPositionBeforeThisProvider(object: &LayoutObject) -> PositionWithAffinity;
    fn LayoutObjectSetModifiedStyleOutsideStyleRecalcProvider(
        object: &mut LayoutObject,
        style: *const ComputedStyle,
        apply_changes: ApplyStyleChanges,
    );
    fn LayoutObjectDumpLayoutObjectProvider(
        object: &LayoutObject,
        output: &mut StringBuilder,
        dump_address: bool,
        show_tree_character_offset: u32,
    );
    fn LayoutObjectShowTreeForThisProvider(object: &LayoutObject);
    fn LayoutObjectShowLayoutObjectProvider(object: &LayoutObject);
    fn LayoutObjectDumpLayoutTreeAndMarkProvider(
        object: &LayoutObject,
        output: &mut StringBuilder,
        marked_object1: *const LayoutObject,
        marked_label1: *const c_char,
        marked_object2: *const LayoutObject,
        marked_label2: *const c_char,
        depth: u32,
    );
    fn LayoutObjectAssertFragmentTreeProvider(object: &LayoutObject, display_locked: bool);
    fn LayoutObjectSetLayoutNeededForbiddenScopeConstructProvider(
        object: *mut LayoutObject,
    ) -> SetLayoutNeededForbiddenScope;
    fn LayoutObjectSetLayoutNeededForbiddenScopeDestructProvider(
        scope: &mut SetLayoutNeededForbiddenScope,
    );
    fn LocalRectForBoxQuadProvider(
        fragment: &PhysicalBoxFragment,
        box_type: BoxQuadType,
    ) -> PhysicalRect;
    fn LayoutObjectHitTestForOcclusionProvider(
        object: &LayoutObject,
        rect: &PhysicalRect,
    ) -> HitTestResult;
    fn LayoutObjectDestroyAndCleanupAnonymousWrappersProvider(
        object: &mut LayoutObject,
        performing_reattach: bool,
    );
    fn LayoutObjectCollectOutlineRectsAndAdvanceProvider(
        object: &LayoutObject,
        outline_type: OutlineType,
        iterator: &mut AccompaniedFragmentIterator,
    ) -> Vector<PhysicalRect>;
    fn LayoutObjectOutlineRectsProvider(
        object: &LayoutObject,
        info: *mut LayoutOutlineInfo,
        additional_offset: &PhysicalOffset,
        outline_type: OutlineType,
    ) -> Vector<PhysicalRect>;
    fn LayoutObjectGetDisplayLockContextProvider(object: &LayoutObject) -> *mut DisplayLockContext;
    fn LayoutObjectMapToVisualRectInAncestorSpaceInternalFastPathProvider(
        object: &LayoutObject,
        ancestor_or_null_for_viewport: *const LayoutBoxModelObject,
        rect: &mut gfx::RectF,
        flags: VisualRectFlags,
        intersects: &mut bool,
    ) -> bool;
    fn AssociatedLayoutObjectOfProvider(
        node: &Node,
        offset_in_node: i32,
        side: LayoutObjectSide,
    ) -> *const LayoutObject;
    fn ShowTreeProvider(object: *const LayoutObject);
    fn ShowLayoutTreeProvider(object: *const LayoutObject);
    fn ShowLayoutTreeWithSecondProvider(object1: *const LayoutObject, object2: *const LayoutObject);
    fn DispatchLayoutObjectIsLayoutTextCombine(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsMedia(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsTable(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsTextArea(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsTextField(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsImage(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsFragmentLessBox(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectHasInlineFragments(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectFirstInlineFragmentItemIndex(object: *const LayoutObject) -> usize;
    fn DispatchLayoutObjectClearFirstInlineFragmentItemIndex(object: *mut LayoutObject);
    fn DispatchLayoutObjectSetFirstInlineFragmentItemIndex(object: *mut LayoutObject, index: usize);
    fn DispatchLayoutObjectAnonymousHasStylePropagationOverride(object: *mut LayoutObject) -> bool;
    fn DispatchLayoutObjectInLayoutNGInlineFormattingContextWillChange(
        object: *mut LayoutObject,
        value: bool,
    );
    fn LayoutObjectPaintProvider(object: &LayoutObject, info: &PaintInfo);
    fn LayoutObjectInvalidateVisualOverflowProvider(object: &mut LayoutObject);
    fn LayoutObjectAddDraggableRegionsProvider(
        object: &mut LayoutObject,
        regions: &mut Vector<DraggableRegionValue>,
    );
    fn LayoutObjectCanHaveAdditionalCompositingReasonsProvider(object: &LayoutObject) -> bool;
    fn LayoutObjectAdditionalCompositingReasonsProvider(
        object: &LayoutObject,
    ) -> CompositingReasons;
    fn LayoutObjectHitTestAllPhasesProvider(
        object: &mut LayoutObject,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        accumulated_offset: &PhysicalOffset,
    ) -> bool;
    fn LayoutObjectNodeForHitTestProvider(object: &LayoutObject) -> *mut Node;
    fn LayoutObjectUpdateHitTestResultProvider(
        object: &LayoutObject,
        result: &mut HitTestResult,
        offset: &PhysicalOffset,
    );
    fn LayoutObjectNodeAtPointProvider(
        object: &mut LayoutObject,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        accumulated_offset: &PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool;
    fn LayoutObjectMapToVisualRectPhysicalProvider(
        object: &LayoutObject,
        ancestor: *const LayoutBoxModelObject,
        rect: &mut PhysicalRect,
        flags: VisualRectFlags,
    ) -> bool;
    fn LayoutObjectMapToVisualRectFloatProvider(
        object: &LayoutObject,
        ancestor: *const LayoutBoxModelObject,
        rect: &mut gfx::RectF,
        flags: VisualRectFlags,
    ) -> bool;
    fn DispatchLayoutObjectMapToVisualRectInternal(
        object: &LayoutObject,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: VisualRectFlags,
    ) -> bool;
    fn LayoutObjectGetPropertyContainerProvider(
        object: &LayoutObject,
        skip_info: *mut AncestorSkipInfo,
        state: *mut PropertyTreeStateOrAlias,
        flags: VisualRectFlags,
    ) -> *const LayoutObject;
    fn LayoutObjectLocalCaretRectProvider(
        object: &LayoutObject,
        offset: i32,
        shape: CaretShape,
    ) -> PhysicalRect;
    fn LayoutObjectMapLocalToAncestorProvider(
        object: &LayoutObject,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        mode: MapCoordinatesFlags,
    );
    fn LayoutObjectMapAncestorToLocalProvider(
        object: &LayoutObject,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        mode: MapCoordinatesFlags,
    );
    fn DispatchLayoutObjectAddOutlineRects(
        object: &LayoutObject,
        collector: &mut dyn OutlineRectCollector,
        info: *mut LayoutOutlineInfo,
        additional_offset: &PhysicalOffset,
        outline_type: OutlineType,
    );
    fn LayoutObjectInvalidatePaintProvider(
        object: &LayoutObject,
        context: &PaintInvalidatorContext,
    );
    fn LayoutObjectInvalidateDisplayItemClientsProvider(
        object: &LayoutObject,
        reason: PaintInvalidationReason,
    );
    fn LayoutObjectDebugRectProvider(object: &LayoutObject) -> PhysicalRect;
    fn DispatchLayoutObjectWillBeDestroyed(object: *mut LayoutObject);
    fn DispatchLayoutObjectVirtualChildren(
        object: *const LayoutObject,
    ) -> *mut LayoutObjectChildList;
    fn DispatchLayoutObjectIsLayoutInline(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsBoxModelObject(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsBox(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsText(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsLayoutBlockFlow(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsLayoutBlock(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsLayoutReplaced(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsLayoutView(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsLayoutIFrame(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsFlexibleBox(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsMathML(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsLayoutGrid(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsLayoutGridLanes(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsCustomLayoutLoaded(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsFieldset(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsFrameSet(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectOffsetFromContainerInternal(
        object: *const LayoutObject,
        container: *const LayoutObject,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset;
    fn DispatchLayoutObjectIsSVGInlineText(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsLayoutTableCol(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsTableCaption(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsTableCell(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsTableRow(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsTableSection(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsLayoutListItem(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsInlineListItem(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsLayoutInsideListMarker(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsLayoutOutsideListMarker(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsReplacedNormalFlowStackingContext(
        object: *const LayoutObject,
        style: &ComputedStyle,
    ) -> bool;
    fn DispatchLayoutObjectIsSVG(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsSVGRoot(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsSVGContainer(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsSVGHiddenContainer(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsSVGShape(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsSVGImage(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsValidColumnSpannerInTree(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectSetNeedsTransformUpdate(object: *mut LayoutObject);
    fn DispatchLayoutObjectSetNeedsBoundariesUpdate(object: *mut LayoutObject);
    fn DispatchLayoutObjectSetNeedsTextMetricsUpdate(object: *mut LayoutObject);
    fn DispatchLayoutObjectHasNonIsolatedBlendingDescendants(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectDescendantIsolationRequirementsChanged(
        object: *mut LayoutObject,
        state: DescendantIsolationState,
    );
    fn DispatchLayoutObjectLocalToSVGParentTransform(
        object: *const LayoutObject,
    ) -> AffineTransform;
    fn DispatchLayoutObjectIsInitialLetterBox(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsBR(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsCanvas(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsCounter(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsEmbeddedObject(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsFrame(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsListMarkerImage(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsMathMLRoot(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsProgress(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsQuote(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsLayoutCustom(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsLayoutImage(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsLayoutImageReplacement(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsLayoutCustomScrollbarPart(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsRuby(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsTextControlInnerEditor(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsViewTransitionContent(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsViewTransitionRoot(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsSVGTransformableContainer(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsSVGViewportContainer(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsSVGTextPath(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsSVGTSpan(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsSVGResourceContainer(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsSVGFilterPrimitive(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectVisualRectRespectsVisibility(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectUpdateFromElement(object: *mut LayoutObject);
    fn DispatchLayoutObjectCanBeSelectionLeafInternal(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectCanHaveChildren(object: *const LayoutObject) -> bool;
    fn DispatchLayoutObjectIsChildAllowed(
        object: *const LayoutObject,
        child: *mut LayoutObject,
        style: &ComputedStyle,
    ) -> bool;
    fn DispatchLayoutObjectStitchedRowGapIndex(
        object: *const LayoutObject,
        fragment: &PhysicalBoxFragment,
        gap_index: usize,
        line_index: Option<usize>,
    ) -> usize;
    fn LayoutObjectChildPrePaintBlockedByDisplayLockProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectAssertClearedPaintInvalidationFlagsProvider(object: *const LayoutObject);
    fn DispatchLayoutObjectUpdateAnonymousChildStyle(
        object: *const LayoutObject,
        child: *const LayoutObject,
        builder: &mut ComputedStyleBuilder,
    );
    fn LayoutObjectSetShouldDelayFullPaintInvalidationProvider(object: *mut LayoutObject);
    fn LayoutObjectSetShouldInheritContainerTimingRootProvider(
        object: *mut LayoutObject,
        inherit: bool,
    );
    fn LayoutObjectClearPaintFlagsProvider(object: *mut LayoutObject);
    fn LayoutObjectEnsureIsReadyForPaintInvalidationProvider(object: *mut LayoutObject);
    fn LayoutObjectDecoratedNameProvider(object: *const LayoutObject) -> BlinkString;
    fn LayoutObjectHasDistortingVisualEffectsProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectHasNonZeroEffectiveOpacityProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectEnclosingLayerProvider(object: *const LayoutObject) -> *mut PaintLayer;
    fn LayoutObjectPaintingLayerProvider(
        object: *const LayoutObject,
        max_depth: i32,
    ) -> *mut PaintLayer;
    fn LayoutObjectAddLayersProvider(object: *mut LayoutObject, parent_layer: *mut PaintLayer);
    fn LayoutObjectRemoveLayersProvider(object: *mut LayoutObject, parent_layer: *mut PaintLayer);
    fn LayoutObjectMoveLayersProvider(
        object: *mut LayoutObject,
        old_parent: *mut PaintLayer,
        new_parent: *mut PaintLayer,
    );
    fn LayoutObjectFindNextLayerProvider(
        object: *mut LayoutObject,
        parent_layer: *mut PaintLayer,
        start_point: *mut LayoutObject,
        check_parent: bool,
    ) -> *mut PaintLayer;
    fn LayoutObjectIsBackdropForOverscrollAreaParentProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectIsHRProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectIsInputButtonProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectIsMenuListProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectHasClipRelatedPropertyProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectIsScrollMarkerProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectIsScrollMarkerGroupBeforeProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectIsListMarkerForSummaryProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectIsInListMarkerProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectNotifyPriorityScrollAnchorStatusChangedProvider(object: *mut LayoutObject);
    fn LayoutObjectAddAbsoluteRectForLayerProvider(
        object: *mut LayoutObject,
        result: &mut gfx::Rect,
    );
    fn LayoutObjectCreateObjectProvider(
        element: *mut Element,
        style: &ComputedStyle,
    ) -> *mut LayoutObject;
    fn LayoutObjectCreateBlockFlowOrListItemProvider(
        element: *mut Element,
        style: &ComputedStyle,
    ) -> *mut LayoutBlockFlow;
    fn LayoutObjectGeneratingNodeProvider(object: *const LayoutObject) -> *mut Node;
    fn LayoutObjectEnclosingNodeProvider(object: *const LayoutObject) -> *mut Node;
    fn LayoutObjectScrollParentProvider(
        object: *const LayoutObject,
        base: *const Element,
    ) -> *mut Element;
    fn LayoutObjectOffsetParentProvider(
        object: *const LayoutObject,
        base: *const Element,
    ) -> *mut Element;
    fn LayoutObjectInvalidateSubtreePositionTryProvider(
        object: *mut LayoutObject,
        mark_style_dirty: bool,
    );
    fn LayoutObjectIsCanvasOrInCanvasSubtreeProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectIsInCanvasSubtreeProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectRecalcNormalFlowChildVisualOverflowIfNeededProvider(object: *mut LayoutObject);
    fn LayoutObjectCanUpdateSelectionOnRootLineBoxesProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectAbsoluteSelectionRectProvider(object: *const LayoutObject) -> PhysicalRect;
    fn LayoutObjectCanBeSelectionLeafProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectIsSelectedProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectIsSelectableProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectVisibleToHitTestRequestProvider(
        object: *const LayoutObject,
        request: &HitTestRequest,
    ) -> bool;
    fn LayoutObjectCanvasForDrawingLayoutObjectProvider(
        object: *const LayoutObject,
    ) -> *mut LayoutObject;
    fn LayoutObjectSetShouldInvalidatePaintForHitTestProvider(object: *mut LayoutObject);
    fn LayoutObjectClearPaintInvalidationFlagsProvider(object: *mut LayoutObject);
    fn LayoutObjectSetMayNeedPaintInvalidationAnimatedBackgroundImageProvider(
        object: *mut LayoutObject,
    );
    fn LayoutObjectClearShouldDelayFullPaintInvalidationProvider(object: *mut LayoutObject);
    fn LayoutObjectSetShouldInvalidateSelectionProvider(object: *mut LayoutObject);
    fn LayoutObjectInvalidateSelectionOnStyleChangeProvider(object: *mut LayoutObject);
    fn LayoutObjectSetNeedsPrePaintSubtreeWalkProvider(
        object: *mut LayoutObject,
        reasons: PrePaintSubtreeWalkReasons,
    );
    fn LayoutObjectSetDescendantNeedsPrePaintSubtreeWalkProvider(
        object: *mut LayoutObject,
        reasons: PrePaintSubtreeWalkReasons,
    );
    fn LayoutObjectMarkEffectiveAllowedTouchActionChangedProvider(object: *mut LayoutObject);
    fn LayoutObjectMarkBlockingWheelEventHandlerChangedProvider(object: *mut LayoutObject);
    fn LayoutObjectMarkSoftNavigationContextChangedProvider(object: *mut LayoutObject);
    fn LayoutObjectMarkContainerTimingChangedProvider(object: *mut LayoutObject);
    fn LayoutObjectChildPaintBlockedByDisplayLockProvider(object: *const LayoutObject) -> bool;
    fn MutableForPaintingSetShouldCheckForPaintInvalidationProvider(object: *mut LayoutObject);
    fn MutableForPaintingSetShouldDoFullPaintInvalidationProvider(
        object: *mut LayoutObject,
        reason: PaintInvalidationReason,
    );
    fn MutableForPaintingSetShouldDoFullPaintInvalidationWithoutLayoutChangeProvider(
        object: *mut LayoutObject,
        reason: PaintInvalidationReason,
    );
    fn MutableForPaintingSetOnlyThisNeedsPaintPropertyUpdateProvider(object: *mut LayoutObject);
    fn LayoutObjectMaybeClearIsScrollAnchorObjectProvider(object: *mut LayoutObject);
    fn LayoutObjectSetSVGDescendantMayHaveTransformRelatedOperationsProvider(
        object: *mut LayoutObject,
    );
    fn LayoutObjectUpdateAfterReinsertProvider(
        object: *mut LayoutObject,
        old_style: &ComputedStyle,
    );
    fn LayoutObjectSetIsBackgroundAttachmentFixedObjectProvider(
        object: *mut LayoutObject,
        value: bool,
    );
    fn LayoutObjectSetCanCompositeBackgroundAttachmentFixedProvider(
        object: *mut LayoutObject,
        value: bool,
    );
    fn LayoutObjectBackgroundIsKnownToBeObscuredProvider(object: *const LayoutObject) -> bool;
    fn LayoutObjectSetBackgroundIsKnownToBeObscuredProvider(object: *mut LayoutObject, value: bool);
    fn LayoutObjectFirstLineStyleWithoutFallbackProvider(
        object: *const LayoutObject,
    ) -> *const ComputedStyle;
    fn LayoutObjectMarkSelfPaintingLayerForVisualOverflowRecalcProvider(object: *mut LayoutObject);
    fn LayoutObjectGetImageOrientationProvider(
        object: *const LayoutObject,
    ) -> RespectImageOrientationEnum;
    fn LayoutObjectAddAsImageObserverProvider(object: *mut LayoutObject, image: *mut StyleImage);
    fn LayoutObjectRemoveAsImageObserverProvider(object: *mut LayoutObject, image: *mut StyleImage);
    fn LayoutObjectUpdateImageProvider(
        object: *mut LayoutObject,
        old_image: *mut StyleImage,
        new_image: *mut StyleImage,
    );
    fn LayoutObjectUpdateShapeImageProvider(
        object: *mut LayoutObject,
        old_shape: *const ShapeValue,
        new_shape: *const ShapeValue,
    );
    fn LayoutObjectUpdateFillImagesProvider(
        object: *mut LayoutObject,
        old_layers: *const FillLayer,
        new_layers: *const FillLayer,
    );
    fn LayoutObjectUpdateCursorImagesProvider(
        object: *mut LayoutObject,
        old_cursors: *const CursorList,
        new_cursors: *const CursorList,
    );
    fn LayoutObjectInvalidateSelectedChildrenOnStyleChangeProvider(object: *mut LayoutObject);
    fn LayoutObjectAdjustStyleDifferenceProvider(
        object: *const LayoutObject,
        difference: StyleDifference,
    ) -> StyleDifference;
    fn LayoutObjectUpdateImageObserversProvider(
        object: *mut LayoutObject,
        old_style: *const ComputedStyle,
        new_style: *const ComputedStyle,
    );
    fn LayoutObjectUpdateFirstLineImageObserversProvider(
        object: *mut LayoutObject,
        new_style: *const ComputedStyle,
    );
    fn LayoutObjectApplyPseudoElementStyleChangesProvider(
        object: *mut LayoutObject,
        old_style: *const ComputedStyle,
    );
    fn LayoutObjectApplyFirstLineChangesProvider(
        object: *mut LayoutObject,
        old_style: *const ComputedStyle,
    );
}

// cpp: layoutng/internal/layout_object_fragment_data.cc:76-76
static ALLOW_DESTROYING_IN_FINALIZER: AtomicU32 = AtomicU32::new(0);

// cpp: layoutng/internal/layout_object.h:157-163
pub struct AllowDestroyingLayoutObjectInFinalizerScope;

impl AllowDestroyingLayoutObjectInFinalizerScope {
    // cpp: layoutng/internal/layout_object_fragment_data.cc:78-81
    pub fn new() -> Self {
        ALLOW_DESTROYING_IN_FINALIZER.fetch_add(1, Ordering::Relaxed);
        Self
    }
}

impl Drop for AllowDestroyingLayoutObjectInFinalizerScope {
    // cpp: layoutng/internal/layout_object_fragment_data.cc:83-87
    fn drop(&mut self) {
        assert!(ALLOW_DESTROYING_IN_FINALIZER.load(Ordering::Relaxed) > 0);
        ALLOW_DESTROYING_IN_FINALIZER.fetch_sub(1, Ordering::Relaxed);
    }
}

// cpp: layoutng/internal/layout_object.h:489-499
pub struct SetLayoutNeededForbiddenScope {
    pub(crate) layout_object_: *mut LayoutObject,
    pub(crate) preexisting_forbidden_: bool,
}

impl SetLayoutNeededForbiddenScope {
    // cpp: layoutng/internal/layout_object.h:493-493
    pub fn new(object: &mut LayoutObject) -> Self {
        unsafe { LayoutObjectSetLayoutNeededForbiddenScopeConstructProvider(object) }
    }
}

impl Drop for SetLayoutNeededForbiddenScope {
    // cpp: layoutng/internal/layout_object.h:494-494
    fn drop(&mut self) {
        unsafe { LayoutObjectSetLayoutNeededForbiddenScopeDestructProvider(self) }
    }
}

// The source creates this short-lived mutator from a const LayoutObject and
// explicitly const_casts its owner; the raw pointer preserves that boundary.
// cpp: layoutng/internal/layout_object.h:2967-2970,3068-3072
pub struct MutableForPainting {
    layout_object_: *mut LayoutObject,
}

impl MutableForPainting {
    // cpp: layoutng/internal/layout_object.h:3071-3072
    fn new(layout_object: &LayoutObject) -> Self {
        Self {
            layout_object_: layout_object as *const LayoutObject as *mut LayoutObject,
        }
    }

    // cpp: layoutng/internal/layout_object.h:2973-2973
    pub fn ClearPaintFlags(&mut self) {
        unsafe { &mut *self.layout_object_ }.ClearPaintFlags();
    }

    // cpp: layoutng/internal/layout_object.h:2986-2988
    pub fn EnsureIsReadyForPaintInvalidation(&mut self) {
        unsafe { &mut *self.layout_object_ }.EnsureIsReadyForPaintInvalidation();
    }

    // cpp: layoutng/internal/layout_object.h:2978-2978
    pub fn SetShouldCheckForPaintInvalidation(&mut self) {
        unsafe { MutableForPaintingSetShouldCheckForPaintInvalidationProvider(self.layout_object_) }
    }

    // cpp: layoutng/internal/layout_object.h:2979-2979
    pub fn SetShouldDoFullPaintInvalidation(&mut self, reason: PaintInvalidationReason) {
        unsafe {
            MutableForPaintingSetShouldDoFullPaintInvalidationProvider(self.layout_object_, reason)
        }
    }

    // cpp: layoutng/internal/layout_object.h:2980-2981
    pub fn SetShouldDoFullPaintInvalidationWithoutLayoutChange(
        &mut self,
        reason: PaintInvalidationReason,
    ) {
        unsafe {
            MutableForPaintingSetShouldDoFullPaintInvalidationWithoutLayoutChangeProvider(
                self.layout_object_,
                reason,
            )
        }
    }

    // cpp: layoutng/internal/layout_object.h:3001-3001
    pub fn SetOnlyThisNeedsPaintPropertyUpdate(&mut self) {
        unsafe {
            MutableForPaintingSetOnlyThisNeedsPaintPropertyUpdateProvider(self.layout_object_)
        }
    }

    // cpp: layoutng/internal/layout_object.h:2990-2992
    pub fn SetBackgroundPaintLocation(&mut self, location: BackgroundPaintLocation) {
        unsafe { &mut *self.layout_object_ }.SetBackgroundPaintLocation(location);
    }

    // cpp: layoutng/internal/layout_object.h:2983-2985
    pub fn SetShouldDelayFullPaintInvalidation(&mut self) {
        unsafe { &mut *self.layout_object_ }.SetShouldDelayFullPaintInvalidation();
    }

    // cpp: layoutng/internal/layout_object.h:3003-3006
    pub fn AddSubtreePaintPropertyUpdateReason(
        &mut self,
        reason: SubtreePaintPropertyUpdateReason,
    ) {
        unsafe { &mut *self.layout_object_ }.AddSubtreePaintPropertyUpdateReason(reason);
    }

    // cpp: layoutng/internal/layout_object.h:2994-2997
    pub fn UpdatePreviousVisibilityVisible(&mut self) {
        let owner = unsafe { &mut *self.layout_object_ };
        let visible = owner.StyleRef().Visibility() == EVisibility::kVisible;
        owner.bitfields_.previous_visibility_visible_ = visible;
    }

    // cpp: layoutng/internal/layout_object.h:3008-3010
    pub fn UpdateInsideBlockingTouchEventHandler(&mut self, inside: bool) {
        unsafe { &mut *self.layout_object_ }.UpdateInsideBlockingTouchEventHandler(inside);
    }

    // cpp: layoutng/internal/layout_object.h:3012-3014
    pub fn UpdateInsideBlockingWheelEventHandler(&mut self, inside: bool) {
        unsafe { &mut *self.layout_object_ }.UpdateInsideBlockingWheelEventHandler(inside);
    }

    // cpp: layoutng/internal/layout_object.h:3016-3018
    pub fn UpdateIsActiveUnboundedElementOrDescendant(&mut self, inside: bool) {
        unsafe { &mut *self.layout_object_ }.UpdateIsActiveUnboundedElementOrDescendant(inside);
    }

    #[cfg(debug_assertions)]
    // cpp: layoutng/internal/layout_object.h:3021-3023
    pub fn ClearNeedsPaintPropertyUpdateForTesting(&mut self) {
        unsafe { &mut *self.layout_object_ }
            .bitfields_
            .needs_paint_property_update_ = false;
    }

    // cpp: layoutng/internal/layout_object.h:3026-3028
    pub fn SetShouldSkipNextLayoutShiftTracking(&mut self, skip: bool) {
        unsafe { &mut *self.layout_object_ }.SetShouldSkipNextLayoutShiftTracking(skip);
    }

    // cpp: layoutng/internal/layout_object.h:3030-3033
    pub fn SetShouldAssumePaintOffsetTranslationForLayoutShiftTracking(&mut self, assume: bool) {
        unsafe { &mut *self.layout_object_ }
            .SetShouldAssumePaintOffsetTranslationForLayoutShiftTracking(assume);
    }

    // cpp: layoutng/internal/layout_object.h:3035-3037
    pub fn SetShouldInheritSoftNavigationContext(&mut self, inherit: bool) {
        unsafe { &mut *self.layout_object_ }.SetShouldInheritSoftNavigationContext(inherit);
    }

    // cpp: layoutng/internal/layout_object.h:3039-3041
    pub fn SetShouldInheritContainerTimingRoot(&mut self, inherit: bool) {
        unsafe { &mut *self.layout_object_ }.SetShouldInheritContainerTimingRoot(inherit);
    }

    // cpp: layoutng/internal/layout_object.h:3043-3048
    pub fn FragmentCountChanged(&mut self) {
        unsafe { &mut *self.layout_object_ }
            .SetShouldDoFullPaintInvalidationWithReason(PaintInvalidationReason::kLayout);
    }

    // cpp: layoutng/internal/layout_object_fragment_data.cc:72-72
    pub fn FirstFragment(&mut self) -> &mut FragmentData {
        let owner = unsafe { &mut *self.layout_object_ };
        unsafe { &mut *owner.fragment_.Get() }
    }

    // cpp: layoutng/internal/layout_object_fragment_data.cc:74-74
    pub fn EnsureId(&mut self) {
        let owner = unsafe { &mut *self.layout_object_ };
        unsafe { &mut *owner.fragment_.Get() }.EnsureId();
    }

    // cpp: layoutng/internal/layout_object.h:3051-3051
    pub fn FragmentList(&mut self) -> &mut FragmentDataList {
        let owner = unsafe { &mut *self.layout_object_ };
        unsafe { &mut *owner.fragment_.Get() }
    }
}

impl Drop for LayoutObject {
    // cpp: layoutng/internal/layout_object_fragment_data.cc:45-50
    fn drop(&mut self) {
        debug_assert!(self.bitfields_.being_destroyed_);
        #[cfg(debug_assertions)]
        debug_assert!(self.is_destroyed_);
    }
}

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:3250-3253
    pub fn VisualRectRespectsVisibilityBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    pub fn VisualRectRespectsVisibility(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectVisualRectRespectsVisibility(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2155-2155
    pub fn UpdateFromElementBase(&mut self) {
        self.CheckIsNotDestroyed();
    }

    pub fn UpdateFromElement(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectUpdateFromElement(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1828-1835,1868-1870,2272-2280
    pub fn CanContainAbsolutePositionObjects(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.can_contain_absolute_position_objects_
    }

    pub fn CanContainFixedPositionObjects(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.can_contain_fixed_position_objects_
    }

    pub fn CanContainOutOfFlowPositionedElement(&self, position: EPosition) -> bool {
        self.CheckIsNotDestroyed();
        debug_assert!(position == EPosition::kAbsolute || position == EPosition::kFixed);
        (position == EPosition::kAbsolute && self.CanContainAbsolutePositionObjects())
            || (position == EPosition::kFixed && self.CanContainFixedPositionObjects())
    }

    pub fn ClearNeedsCollectInlines(&mut self) {
        self.CheckIsNotDestroyed();
        self.SetNeedsCollectInlinesFlag(false);
    }

    // cpp: layoutng/internal/layout_object.h:2438-2453,3470-3472
    pub fn QuadsInAncestorInternal(
        &self,
        _quads: &mut Vector<gfx::QuadF>,
        _ancestor: *const LayoutBoxModelObject,
        _mode: MapCoordinatesFlags,
        _box_type: BoxQuadType,
    ) {
        self.CheckIsNotDestroyed();
    }

    pub fn QuadsInAncestor(
        &self,
        quads: &mut Vector<gfx::QuadF>,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
        box_type: BoxQuadType,
    ) {
        self.CheckIsNotDestroyed();
        self.QuadsInAncestorInternal(quads, ancestor, mode, box_type);
    }

    pub fn AbsoluteQuads(
        &self,
        quads: &mut Vector<gfx::QuadF>,
        mode: MapCoordinatesFlags,
        box_type: BoxQuadType,
    ) {
        self.CheckIsNotDestroyed();
        self.QuadsInAncestor(quads, std::ptr::null(), mode, box_type);
    }

    // cpp: layoutng/internal/layout_object.h:3454-3457
    pub fn CanBeSelectionLeafInternalBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn CanBeSelectionLeafInternal(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectCanBeSelectionLeafInternal(self) }
    }

    // cpp: layoutng/internal/layout_object.h:542-550
    pub fn CanHaveChildrenBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        !self.VirtualChildren().is_null()
    }

    pub fn CanHaveChildren(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectCanHaveChildren(self) }
    }

    pub fn IsChildAllowedBase(&self, _child: *mut LayoutObject, _style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    pub fn IsChildAllowed(&self, child: *mut LayoutObject, style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsChildAllowed(self, child, style) }
    }

    // cpp: layoutng/internal/layout_object.h:2729-2745,2762-2766,2806-2811
    pub fn PaintInvalidationReasonForPrePaint(&self) -> PaintInvalidationReason {
        self.CheckIsNotDestroyed();
        PaintInvalidationReasonFromBits(self.bitfields_.paint_invalidation_reason_for_pre_paint_)
    }

    pub fn ShouldDoFullPaintInvalidation(&self) -> bool {
        self.CheckIsNotDestroyed();
        if self.ShouldDelayFullPaintInvalidation() {
            debug_assert!(!self.bitfields_.subtree_should_do_full_paint_invalidation_);
            return false;
        }
        if IsFullPaintInvalidationReason(self.PaintInvalidationReasonForPrePaint()) {
            debug_assert!(self.ShouldCheckForPaintInvalidation());
            return true;
        }
        false
    }

    pub fn ShouldInvalidatePaintForHitTestOnly(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.PaintInvalidationReasonForPrePaint() == PaintInvalidationReason::kHitTest
    }

    pub fn SubtreeShouldDoFullPaintInvalidation(&self) -> bool {
        self.CheckIsNotDestroyed();
        debug_assert!(
            !self.bitfields_.subtree_should_do_full_paint_invalidation_
                || self.ShouldDoFullPaintInvalidation()
        );
        self.bitfields_.subtree_should_do_full_paint_invalidation_
    }

    // cpp: layoutng/internal/layout_object.h:3097-3113
    pub fn AddSubtreePaintPropertyUpdateReason(
        &mut self,
        reason: SubtreePaintPropertyUpdateReason,
    ) {
        self.CheckIsNotDestroyed();
        let reason_bits = reason as u32;
        debug_assert!(reason_bits <= (1u32 << (4 - 1)));
        self.bitfields_.subtree_paint_property_update_reasons_ |= reason_bits as u8;
        self.SetNeedsPaintPropertyUpdate();
    }

    pub fn SubtreePaintPropertyUpdateReasons(&self) -> u32 {
        self.CheckIsNotDestroyed();
        self.bitfields_.subtree_paint_property_update_reasons_ as u32
    }

    pub fn DescendantNeedsPaintPropertyUpdate(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.descendant_needs_paint_property_update_
    }

    // cpp: layoutng/internal/layout_object.h:1166-1173,1178-1185,1194-1201,1214-1217
    pub fn IsSVGBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsSVG(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsSVG(self) }
    }

    pub fn IsSVGRootBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsSVGRoot(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsSVGRoot(self) }
    }

    pub fn IsSVGContainerBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsSVGContainer(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsSVGContainer(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1190-1193
    pub fn IsSVGHiddenContainerBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsSVGHiddenContainer(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsSVGHiddenContainer(self) }
    }

    pub fn IsSVGShapeBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsSVGShape(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsSVGShape(self) }
    }

    pub fn IsSVGImageBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsSVGImage(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsSVGImage(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1247-1254
    pub fn IsBlendingAllowed(&self) -> bool {
        self.CheckIsNotDestroyed();
        !self.IsSVG()
            || self.IsSVGShape()
            || self.IsSVGImage()
            || self.IsSVGInline()
            || self.IsSVGRoot()
            || self.IsSVGForeignObject()
            || self.IsSVGText()
            || (self.IsSVGContainer() && !self.IsSVGHiddenContainer())
    }

    // cpp: layoutng/internal/layout_object.h:1239-1241,1255-1272,1329-1332,1334-1337
    pub fn SetNeedsTransformUpdateBase(&mut self) {
        self.CheckIsNotDestroyed();
    }

    pub fn SetNeedsTransformUpdate(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectSetNeedsTransformUpdate(self) }
    }

    pub fn SetNeedsBoundariesUpdateBase(&mut self) {
        self.CheckIsNotDestroyed();
    }

    pub fn SetNeedsBoundariesUpdate(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectSetNeedsBoundariesUpdate(self) }
    }

    pub fn SetNeedsTextMetricsUpdateBase(&mut self) {
        self.CheckIsNotDestroyed();
        std::process::abort();
    }

    pub fn SetNeedsTextMetricsUpdate(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectSetNeedsTextMetricsUpdate(self) }
    }

    pub fn HasNonIsolatedBlendingDescendantsBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        debug_assert!(self.IsSVG());
        false
    }

    pub fn HasNonIsolatedBlendingDescendants(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectHasNonIsolatedBlendingDescendants(self) }
    }

    pub fn DescendantIsolationRequirementsChangedBase(&mut self, _state: DescendantIsolationState) {
        self.CheckIsNotDestroyed();
    }

    pub fn DescendantIsolationRequirementsChanged(&mut self, state: DescendantIsolationState) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectDescendantIsolationRequirementsChanged(self, state) }
    }

    // cpp: layoutng/internal/layout_object.h:1312-1315
    pub fn LocalToSVGParentTransformBase(&self) -> AffineTransform {
        self.CheckIsNotDestroyed();
        self.LocalSVGTransform()
    }

    pub fn LocalToSVGParentTransform(&self) -> AffineTransform {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectLocalToSVGParentTransform(self) }
    }

    pub fn IsInitialLetterBoxBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsInitialLetterBox(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsInitialLetterBox(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1682-1689,1726-1752
    pub fn NonPseudoNode(&self) -> *mut Node {
        self.CheckIsNotDestroyed();
        if self.IsPseudoElement() {
            std::ptr::null_mut()
        } else {
            self.GetNode()
        }
    }

    pub fn ClearNode(&mut self) {
        self.CheckIsNotDestroyed();
        self.node_ = Member::default();
    }

    pub fn IsValidColumnSpannerInTreeBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsValidColumnSpannerInTree(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsValidColumnSpannerInTree(self) }
    }

    pub fn IsColumnSpanAll(&self) -> bool {
        self.CheckIsNotDestroyed();
        let style = self.Style();
        !style.is_null()
            && unsafe { &*style }.GetColumnSpan() == EColumnSpan::kAll
            && self.IsValidColumnSpannerInTree()
    }

    pub fn BehavesLikeBlockContainer(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsLayoutBlockFlow() && self.StyleRef().IsDisplayBlockContainer()
    }

    // cpp: layoutng/internal/layout_object.h:1454-1477
    pub fn IntrinsicLogicalWidthsDependsOnBlockConstraints(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_
            .intrinsic_logical_widths_depends_on_block_constraints_
    }

    pub fn SetIntrinsicLogicalWidthsDependsOnBlockConstraints(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_
            .intrinsic_logical_widths_depends_on_block_constraints_ = value;
    }

    pub fn IndefiniteIntrinsicLogicalWidthsDirty(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.indefinite_intrinsic_logical_widths_dirty_
    }

    pub fn SetIndefiniteIntrinsicLogicalWidthsDirty(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.indefinite_intrinsic_logical_widths_dirty_ = value;
    }

    pub fn DefiniteIntrinsicLogicalWidthsDirty(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.definite_intrinsic_logical_widths_dirty_
    }

    pub fn SetDefiniteIntrinsicLogicalWidthsDirty(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.definite_intrinsic_logical_widths_dirty_ = value;
    }

    // cpp: layoutng/internal/layout_object.h:591-626
    pub fn RespectsCSSOverflowBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn RespectsCSSOverflow(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectRespectsCSSOverflow(self) }
    }

    pub fn ShouldApplyOverflowClipMargin(&self) -> bool {
        self.CheckIsNotDestroyed();
        if self.IsScrollContainer() {
            return false;
        }
        let style = self.StyleRef();
        if !style.OverflowClipMarginHasAnEffect() {
            return false;
        }
        let is_overflow_clip = if self.IsLayoutReplaced() {
            style.OverflowX() != EOverflow::kVisible && style.OverflowY() != EOverflow::kVisible
        } else {
            style.OverflowX() == EOverflow::kClip && style.OverflowY() == EOverflow::kClip
        };
        (is_overflow_clip || self.ShouldApplyPaintContainment()) && self.RespectsCSSOverflow()
    }

    // cpp: layoutng/internal/layout_object.h:628-647
    pub fn IsEligibleForPaintOrLayoutContainmentBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsEligibleForPaintOrLayoutContainment(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsEligibleForPaintOrLayoutContainment(self) }
    }

    pub fn IsEligibleForSizeContainmentBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsEligibleForSizeContainment(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectIsEligibleForSizeContainmentProvider(self) }
    }

    pub fn ShouldApplyPaintContainmentWithStyle(&self, style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        style.ContainsPaint() && self.IsEligibleForPaintOrLayoutContainment()
    }

    pub fn ShouldApplyPaintContainment(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.ShouldApplyPaintContainmentWithStyle(self.StyleRef())
    }

    // cpp: layoutng/internal/layout_object.h:649-691
    pub fn ShouldApplyLayoutContainmentWithStyle(&self, style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        style.ContainsLayout() && self.IsEligibleForPaintOrLayoutContainment()
    }

    pub fn ShouldApplyLayoutContainment(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.ShouldApplyLayoutContainmentWithStyle(self.StyleRef())
    }

    pub fn ShouldApplyInlineSizeContainment(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().ContainsInlineSize() && self.IsEligibleForSizeContainment()
    }

    pub fn ShouldApplyBlockSizeContainment(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().ContainsBlockSize() && self.IsEligibleForSizeContainment()
    }

    pub fn ShouldApplyAnySizeContainment(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().ContainsAnySize() && self.IsEligibleForSizeContainment()
    }

    pub fn ShouldApplyStyleContainment(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().ContainsStyle()
    }

    pub fn ShouldApplyContentContainment(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.ShouldApplyStyleContainment()
            && self.ShouldApplyPaintContainment()
            && self.ShouldApplyLayoutContainment()
    }

    pub fn ShouldApplyStrictContainment(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.ShouldApplyStyleContainment()
            && self.ShouldApplyPaintContainment()
            && self.ShouldApplyLayoutContainment()
            && self.ShouldApplySizeContainment()
    }

    pub fn ShouldApplyAnyContainment(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.ShouldApplyPaintContainment()
            || self.ShouldApplyLayoutContainment()
            || self.ShouldApplyStyleContainment()
            || self.ShouldApplyBlockSizeContainment()
            || self.ShouldApplyInlineSizeContainment()
    }

    // cpp: layoutng/internal/layout_object.h:698-739
    pub fn IsStackingContext(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsStackingContextWithStyle(self.StyleRef())
    }

    pub fn IsStackingContextWithStyle(&self, style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        style.IsStackingContextWithoutContainment()
            || ((style.ContainsLayout() || style.ContainsPaint())
                && self.IsEligibleForPaintOrLayoutContainment())
            || self.IsOverscrollAreaParent()
    }

    pub fn IsReplacedNormalFlowStackingContextBase(&self, _style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsReplacedNormalFlowStackingContext(&self, style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsReplacedNormalFlowStackingContext(self, style) }
    }

    pub fn IsStacked(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsStackedWithStyle(self.StyleRef())
    }

    pub fn IsStackedWithStyle(&self, style: &ComputedStyle) -> bool {
        self.CheckIsNotDestroyed();
        if style.IsUnboundedElementActive() {
            debug_assert!(RuntimeEnabledFeatures::UnboundedElementEnabled());
            return true;
        }
        style.GetPosition() != EPosition::kStatic
            || (self.IsStackingContextWithStyle(style)
                && (!RuntimeEnabledFeatures::StackingContextIsNotStackedEnabled()
                    || !self.IsReplacedNormalFlowStackingContext(style)))
    }

    // The C++ const StitchedSize path updates this cached-geometry bit.
    // Cell preserves the same interior mutability without aliasing a &mut.
    // cpp: layoutng/internal/layout_object.h:3510-3518
    pub fn SetHasValidCachedGeometry(&self, valid: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_valid_cached_geometry_.set(valid);
    }

    pub fn HasValidCachedGeometry(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_valid_cached_geometry_.get()
    }
    // cpp: layoutng/internal/layout_object.h:424-427
    pub fn Depth(&self) -> u32 {
        self.CheckIsNotDestroyed();
        if u32::from(self.bitfields_.depth_) < kMaxLayoutObjectDepth {
            u32::from(self.bitfields_.depth_)
        } else {
            self.DepthSlow()
        }
    }

    // cpp: layoutng/internal/layout_object_tree.cc:146-160
    pub fn DepthSlow(&self) -> u32 {
        self.CheckIsNotDestroyed();
        debug_assert_eq!(u32::from(self.bitfields_.depth_), kMaxLayoutObjectDepth);
        let mut extra = 0u32;
        let mut object: *const LayoutObject = self;
        while !object.is_null() {
            let current = unsafe { &*object };
            if u32::from(current.bitfields_.depth_) < kMaxLayoutObjectDepth {
                return u32::from(current.bitfields_.depth_).wrapping_add(extra);
            }
            extra = extra.wrapping_add(1);
            object = current.Parent();
        }
        std::process::abort();
    }

    // cpp: layoutng/internal/layout_object_tree.cc:162-180
    pub fn SetDepthIncludingDescendants(&mut self, depth: u32) {
        self.CheckIsNotDestroyed();
        let clamped_depth = depth.min(kMaxLayoutObjectDepth);
        if u32::from(self.bitfields_.depth_) == clamped_depth {
            return;
        }
        self.bitfields_.depth_ = clamped_depth as u16;
        let mut child = self.SlowFirstChild();
        while !child.is_null() {
            unsafe { &mut *child }.SetDepthIncludingDescendants(clamped_depth + 1);
            child = unsafe { &*child }.NextSibling();
        }
    }

    // cpp: layoutng/internal/layout_object.h:3122-3129
    pub fn ScrollAnchorDisablingStyleChanged(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.scroll_anchor_disabling_style_changed_
    }

    // cpp: layoutng/internal/layout_object.h:3131-3138
    pub fn ShouldSkipLayoutCache(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.should_skip_layout_cache_
    }

    pub fn SetShouldSkipLayoutCache(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.should_skip_layout_cache_ = value;
    }

    pub fn SetScrollAnchorDisablingStyleChanged(&mut self, changed: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.scroll_anchor_disabling_style_changed_ = changed;
    }

    // cpp: layoutng/internal/layout_node_data.cc:33-48
    pub fn SetScrollAnchorDisablingStyleChangedOnAncestor(&mut self) {
        self.CheckIsNotDestroyed();
        let mut object = self.Parent();
        let viewport_defining_element = self.ViewportDefiningElementForLayout();
        while !object.is_null() {
            let block = DynamicTo::<LayoutBlock>(object);
            if !block.is_null()
                && (unsafe { &*block }.IsScrollContainer()
                    || unsafe { &*block }.GetNode() == viewport_defining_element as *mut Node)
            {
                unsafe { &mut *block }.SetScrollAnchorDisablingStyleChanged(true);
                return;
            }
            object = unsafe { &*object }.Parent();
        }
    }

    // cpp: layoutng/internal/layout_node_data.cc:51-55
    pub fn ChildLayoutBlockedByDisplayLock(&self) -> bool {
        self.CheckIsNotDestroyed();
        ContentLockBlocksChildLayout(self.GetNode())
    }

    // cpp: layoutng/internal/layout_node_data.cc:70-76
    pub fn InputOwnerForLayout(&self) -> &Node {
        self.CheckIsNotDestroyed();
        if !self.node_.Get().is_null() {
            return unsafe { &*self.node_.Get() };
        }
        if !self.anonymous_input_owner_.Get().is_null() {
            return unsafe { &*self.anonymous_input_owner_.Get() };
        }
        let parent = self.Parent();
        assert!(!parent.is_null());
        unsafe { &*parent }.InputOwnerForLayout()
    }

    // cpp: layoutng/internal/layout_node_data.cc:78-83
    pub fn SetInputOwnerForAnonymous(&mut self, parent: &LayoutObject) {
        self.CheckIsNotDestroyed();
        assert!(self.IsAnonymous());
        assert!(self.node_.Get().is_null());
        self.anonymous_input_owner_ =
            Member::from_ptr(parent.InputOwnerForLayout() as *const Node as *mut Node);
    }

    // cpp: layoutng/internal/layout_node_data.cc:85-89
    pub fn HasPreparedLayoutInput(&self) -> bool {
        self.CheckIsNotDestroyed();
        if self.node_.Get().is_null()
            && self.anonymous_input_owner_.Get().is_null()
            && self.Parent().is_null()
        {
            return false;
        }
        self.InputOwnerForLayout().InputTreeIsSealed()
    }

    // cpp: layoutng/internal/layout_node_data.cc:91-94
    pub fn HasFirstLineStylesForLayout(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.InputOwnerForLayout().InputHasFirstLineStyles()
    }

    // cpp: layoutng/internal/layout_node_data.cc:96-99
    pub fn InQuirksModeForLayout(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.InputOwnerForLayout().InputCompatibilityMode() == CompatibilityMode::kQuirks
    }

    // cpp: layoutng/internal/layout_node_data.cc:101-104
    pub fn InLineHeightQuirksModeForLayout(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.InputOwnerForLayout().InputLineHeightQuirksMode()
    }

    // cpp: layoutng/internal/layout_node_data.cc:106-109
    pub fn MinimumFontPhysicalSizeForLayout(&self) -> Option<f32> {
        self.CheckIsNotDestroyed();
        self.InputOwnerForLayout().InputMinimumFontPhysicalSize()
    }

    // cpp: layoutng/internal/layout_node_data.cc:111-114
    pub fn DevicePixelRatioForLayout(&self) -> f64 {
        self.CheckIsNotDestroyed();
        self.InputOwnerForLayout().InputDevicePixelRatio()
    }

    // cpp: layoutng/internal/layout_node_data.cc:116-119
    pub fn CanvasDrawElementEnabledForLayout(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.InputOwnerForLayout().InputCanvasDrawElementEnabled()
    }

    // cpp: layoutng/internal/layout_node_data.cc:121-127
    pub fn ViewportGeometryForLayout(&self) -> &ViewportGeometry {
        self.CheckIsNotDestroyed();
        self.InputOwnerForLayout()
            .InputViewport()
            .as_ref()
            .expect("LayoutView scroll geometry requires viewport input")
    }

    // cpp: layoutng/internal/layout_node_data.cc:129-132
    pub fn ViewportDefiningElementForLayout(&self) -> *mut Element {
        self.CheckIsNotDestroyed();
        self.InputOwnerForLayout().InputViewportDefiningElement()
    }

    // cpp: layoutng/internal/layout_node_data.cc:134-137
    pub fn IsPrintingForLayout(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.InputOwnerForLayout().InputPrinting()
    }

    // cpp: layoutng/internal/layout_node_data.cc:139-142
    pub fn ScrollbarThemeForLayout(&self) -> &ScrollbarThemeMetrics {
        self.CheckIsNotDestroyed();
        RequireScrollbarTheme(self.InputOwnerForLayout().InputScrollbarTheme())
    }

    // cpp: layoutng/internal/layout_node_data.cc:144-147
    pub fn ControlThemeForLayout(&self) -> &ControlThemeMetrics {
        self.CheckIsNotDestroyed();
        self.InputOwnerForLayout().InputControlTheme()
    }

    // cpp: layoutng/internal/layout_node_data.cc:149-153
    pub fn IsDocumentElement(&self) -> bool {
        self.CheckIsNotDestroyed();
        let node = self.node_.Get();
        !node.is_null()
            && matches!(
                unsafe { &*node }.InputDocumentRole(),
                DocumentRole::kDocumentElement | DocumentRole::kDocumentElementAndBody
            )
    }

    // cpp: layoutng/internal/layout_node_data.cc:155-159
    pub fn IsBody(&self) -> bool {
        self.CheckIsNotDestroyed();
        let node = self.GetNode();
        !node.is_null()
            && matches!(
                unsafe { &*node }.InputDocumentRole(),
                DocumentRole::kBody | DocumentRole::kDocumentElementAndBody
            )
    }

    // cpp: layoutng/internal/layout_node_data.cc:166-175
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.style_);
        visitor.Trace(&self.node_);
        visitor.Trace(&self.anonymous_input_owner_);
        visitor.Trace(&self.parent_);
        visitor.Trace(&self.previous_);
        visitor.Trace(&self.next_);
        visitor.Trace(&self.fragment_);
        self.display_item_client_.Trace(visitor);
    }

    // cpp: layoutng/internal/layout_object.h:835-838
    pub fn IsBoxModelObjectBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsBoxModelObject(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsBoxModelObject(self) }
    }

    // cpp: layoutng/internal/layout_object.h:839-842
    pub fn IsBoxBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsBox(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsBox(self) }
    }

    // cpp: layoutng/internal/layout_object.h:875-878
    pub fn IsFlexibleBoxBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsFlexibleBox(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsFlexibleBox(self) }
    }

    // cpp: layoutng/internal/layout_object.h:907-910
    pub fn IsMathMLBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsMathML(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsMathML(self) }
    }

    // cpp: layoutng/internal/layout_object.h:935-947
    pub fn IsLayoutGridBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsLayoutGrid(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsLayoutGrid(self) }
    }

    pub fn IsLayoutGridLanesBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsLayoutGridLanes(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsLayoutGridLanes(self) }
    }

    pub fn IsLayoutGridOrGridLanes(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsLayoutGrid() || self.IsLayoutGridLanes()
    }

    // cpp: layoutng/internal/layout_object.h:843-846
    pub fn IsTextBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsText(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsText(self) }
    }

    // cpp: layoutng/internal/layout_object.h:899-902
    pub fn IsLayoutTableColBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsLayoutTableCol(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsLayoutTableCol(self) }
    }

    // cpp: layoutng/internal/layout_object.h:981-996
    pub fn IsTableCaptionBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsTableCaption(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsTableCaption(self) }
    }

    pub fn IsTableCellBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsTableCell(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsTableCell(self) }
    }

    pub fn IsTableRowBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsTableRow(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsTableRow(self) }
    }

    pub fn IsTableSectionBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsTableSection(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsTableSection(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1056-1060
    pub fn IsTablePart(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsTableCell()
            || self.IsLayoutTableCol()
            || self.IsTableCaption()
            || self.IsTableRow()
            || self.IsTableSection()
    }

    // cpp: layoutng/internal/layout_object.h:1210-1213
    pub fn IsSVGInlineTextBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsSVGInlineText(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsSVGInlineText(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1029-1032
    pub fn IsLayoutBlockFlowBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsLayoutBlockFlow(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsLayoutBlockFlow(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1025-1028
    pub fn IsLayoutBlockBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsLayoutBlock(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsLayoutBlock(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1323-1327
    pub fn IsAnonymousBlockFlow(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsAnonymous()
            && self.IsLayoutBlockFlow()
            && self.StyleRef().Display() == EDisplay::kBlock
    }

    // cpp: layoutng/internal/layout_object.h:959-962
    pub fn IsLayoutReplacedBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsLayoutReplaced(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsLayoutReplaced(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1009-1012,1037-1040
    pub fn IsVideoBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsVideo(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsVideo(self) }
    }

    pub fn IsLayoutEmbeddedContentBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsLayoutEmbeddedContent(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsLayoutEmbeddedContent(self) }
    }

    // cpp: layoutng/internal/layout_object.h:967-970
    pub fn IsLayoutViewBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsLayoutView(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsLayoutView(self) }
    }

    // cpp: layoutng/internal/layout_object.h:947-950
    pub fn IsLayoutIFrameBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsLayoutIFrame(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsLayoutIFrame(self) }
    }

    // cpp: layoutng/internal/layout_object.h:931-934
    pub fn IsCustomLayoutLoadedBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsCustomLayoutLoaded(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsCustomLayoutLoaded(self) }
    }

    // cpp: layoutng/internal/layout_object.h:863-866
    pub fn IsFieldsetBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsFieldset(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsFieldset(self) }
    }

    // cpp: layoutng/internal/layout_object.h:871-874
    pub fn IsFrameSetBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsFrameSet(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsFrameSet(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1374-1377
    pub fn IsAtomicInline(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsInline() && self.IsBox()
    }

    // cpp: layoutng/internal/layout_object.h:1352-1356
    pub fn IsFixedPositioned(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsOutOfFlowPositioned() && self.StyleRef().GetPosition() == EPosition::kFixed
    }

    // cpp: layoutng/internal/layout_object.h:1542-1553
    pub fn IsScrollContainer(&self) -> bool {
        self.CheckIsNotDestroyed();
        if self.IsLayoutReplaced() {
            return false;
        }
        self.HasNonVisibleOverflow() && self.StyleRef().IsScrollContainer()
    }

    // cpp: layoutng/internal/layout_object.h:1525-1535
    pub fn IsOverscrollContainer(&self) -> bool {
        self.CheckIsNotDestroyed();
        if !self.IsBox() {
            return false;
        }
        if self.IsOverscrollAreaParent() {
            return true;
        }
        self.StyleRef().EffectiveOverscrollContainerType() != EOverscrollContainerType::kNone
    }

    // cpp: layoutng/internal/layout_object.h:2401-2405
    pub fn OffsetFromContainer(
        &self,
        container: *const LayoutObject,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        self.OffsetFromContainerInternal(container, mode)
    }

    pub fn OffsetFromContainerDefault(&self, container: *const LayoutObject) -> PhysicalOffset {
        self.OffsetFromContainer(container, MapCoordinatesFlags::default())
    }

    // cpp: layoutng/internal/layout_object.h:3459-3461
    pub fn OffsetFromContainerInternal(
        &self,
        container: *const LayoutObject,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectOffsetFromContainerInternal(self, container, mode) }
    }

    // cpp: layoutng/internal/layout_object.h:2287-2294
    pub fn AncestorToLocalRect(
        &self,
        ancestor: *const LayoutBoxModelObject,
        rect: &PhysicalRect,
        mode: MapCoordinatesFlags,
    ) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        let quad = gfx::QuadF::from(gfx::RectF::from(*rect));
        PhysicalRect::EnclosingRect(
            &self
                .AncestorToLocalQuad(ancestor, &quad, mode)
                .BoundingBox(),
        )
    }

    // cpp: layoutng/internal/layout_object.h:2295-2297
    pub fn AncestorToLocalQuad(
        &self,
        ancestor: *const LayoutBoxModelObject,
        quad: &gfx::QuadF,
        mode: MapCoordinatesFlags,
    ) -> gfx::QuadF {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectAncestorToLocalQuad(self, ancestor, quad, mode) }
    }

    // cpp: layoutng/internal/layout_object.h:2298-2304
    pub fn AncestorToLocalPointPhysical(
        &self,
        ancestor: *const LayoutBoxModelObject,
        point: &PhysicalOffset,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        let float_point = gfx::PointF::new(point.left.ToFloat(), point.top.ToFloat());
        PhysicalOffset::FromPointFRound(&self.AncestorToLocalPointF(ancestor, &float_point, mode))
    }

    // cpp: layoutng/internal/layout_object.h:2305-2307
    pub fn AncestorToLocalPointF(
        &self,
        ancestor: *const LayoutBoxModelObject,
        point: &gfx::PointF,
        mode: MapCoordinatesFlags,
    ) -> gfx::PointF {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectAncestorToLocalPointFProvider(self, ancestor, point, mode) }
    }

    // cpp: layoutng/internal/layout_object.h:2315-2317
    pub fn LocalToAncestorRect(
        &self,
        rect: &PhysicalRect,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
    ) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectLocalToAncestorRectProvider(self, rect, ancestor, mode) }
    }

    // cpp: layoutng/internal/layout_object.h:2318-2323
    pub fn LocalRectToAncestorQuad(
        &self,
        rect: &PhysicalRect,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
    ) -> gfx::QuadF {
        self.CheckIsNotDestroyed();
        let quad = gfx::QuadF::from(gfx::RectF::from(*rect));
        self.LocalToAncestorQuad(&quad, ancestor, mode)
    }

    // cpp: layoutng/internal/layout_object.h:2324-2326
    pub fn LocalToAncestorQuad(
        &self,
        quad: &gfx::QuadF,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
    ) -> gfx::QuadF {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectLocalToAncestorQuad(self, quad, ancestor, mode) }
    }

    // cpp: layoutng/internal/layout_object.h:2327-2333
    pub fn LocalToAncestorPointPhysical(
        &self,
        point: &PhysicalOffset,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        let float_point = gfx::PointF::new(point.left.ToFloat(), point.top.ToFloat());
        PhysicalOffset::FromPointFRound(&self.LocalToAncestorPoint(&float_point, ancestor, mode))
    }

    // cpp: layoutng/internal/layout_object.h:2342-2343
    pub fn LocalToAncestorTransform(
        &self,
        ancestor: *const LayoutBoxModelObject,
        mode: MapCoordinatesFlags,
    ) -> gfx::Transform {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectLocalToAncestorTransformProvider(self, ancestor, mode) }
    }

    // cpp: layoutng/internal/layout_object.h:2344-2347
    pub fn LocalToAbsoluteTransform(&self, mode: MapCoordinatesFlags) -> gfx::Transform {
        self.CheckIsNotDestroyed();
        self.LocalToAncestorTransform(std::ptr::null(), mode)
    }

    // cpp: layoutng/internal/layout_object.h:2352-2356
    pub fn LocalToAbsoluteRect(
        &self,
        rect: &PhysicalRect,
        mode: MapCoordinatesFlags,
    ) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.LocalToAncestorRect(rect, std::ptr::null(), mode)
    }

    // cpp: layoutng/internal/layout_object.h:2357-2361
    pub fn LocalRectToAbsoluteQuad(
        &self,
        rect: &PhysicalRect,
        mode: MapCoordinatesFlags,
    ) -> gfx::QuadF {
        self.CheckIsNotDestroyed();
        self.LocalRectToAncestorQuad(rect, std::ptr::null(), mode)
    }

    // cpp: layoutng/internal/layout_object.h:2362-2366
    pub fn LocalToAbsoluteQuad(&self, quad: &gfx::QuadF, mode: MapCoordinatesFlags) -> gfx::QuadF {
        self.CheckIsNotDestroyed();
        self.LocalToAncestorQuad(quad, std::ptr::null(), mode)
    }

    // cpp: layoutng/internal/layout_object.h:2367-2371
    pub fn LocalToAbsolutePointPhysical(
        &self,
        point: &PhysicalOffset,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        self.LocalToAncestorPointPhysical(point, std::ptr::null(), mode)
    }

    // cpp: layoutng/internal/layout_object.h:2372-2376
    pub fn LocalToAbsolutePointF(
        &self,
        point: &gfx::PointF,
        mode: MapCoordinatesFlags,
    ) -> gfx::PointF {
        self.CheckIsNotDestroyed();
        self.LocalToAncestorPoint(point, std::ptr::null(), mode)
    }

    // cpp: layoutng/internal/layout_object.h:2377-2381
    pub fn AbsoluteToLocalRect(
        &self,
        rect: &PhysicalRect,
        mode: MapCoordinatesFlags,
    ) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        self.AncestorToLocalRect(std::ptr::null(), rect, mode)
    }

    // cpp: layoutng/internal/layout_object.h:2382-2386
    pub fn AbsoluteToLocalQuad(&self, quad: &gfx::QuadF, mode: MapCoordinatesFlags) -> gfx::QuadF {
        self.CheckIsNotDestroyed();
        self.AncestorToLocalQuad(std::ptr::null(), quad, mode)
    }

    // cpp: layoutng/internal/layout_object.h:2387-2391
    pub fn AbsoluteToLocalPointPhysical(
        &self,
        point: &PhysicalOffset,
        mode: MapCoordinatesFlags,
    ) -> PhysicalOffset {
        self.CheckIsNotDestroyed();
        self.AncestorToLocalPointPhysical(std::ptr::null(), point, mode)
    }

    // cpp: layoutng/internal/layout_object.h:2392-2396
    pub fn AbsoluteToLocalPointF(
        &self,
        point: &gfx::PointF,
        mode: MapCoordinatesFlags,
    ) -> gfx::PointF {
        self.CheckIsNotDestroyed();
        self.AncestorToLocalPointF(std::ptr::null(), point, mode)
    }

    // cpp: layoutng/internal/layout_object.h:1033-1036
    pub fn IsLayoutInlineBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsLayoutInline(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsLayoutInline(self) }
    }

    // The default C++ virtual body returns nullptr. Actual tree traversal
    // dispatches to the concrete subclass's child-list owner.
    // cpp: layoutng/internal/layout_object.h:390-398
    pub fn VirtualChildrenBase(&self) -> *mut LayoutObjectChildList {
        self.CheckIsNotDestroyed();
        std::ptr::null_mut()
    }

    pub fn VirtualChildren(&self) -> *mut LayoutObjectChildList {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectVirtualChildren(self) }
    }

    // cpp: layoutng/internal/layout_object.h:377-388
    pub fn SlowFirstChild(&self) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        let children = self.VirtualChildren();
        if children.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &*children }.FirstChild()
        }
    }

    pub fn SlowLastChild(&self) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        let children = self.VirtualChildren();
        if children.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &*children }.LastChild()
        }
    }

    // cpp: layoutng/internal/layout_object.h:1073-1081
    pub fn MayContainAnchor(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.may_contain_anchor_
    }

    pub fn SetSelfMayContainAnchor(&mut self) {
        self.CheckIsNotDestroyed();
        self.bitfields_.may_contain_anchor_ = true;
    }

    // cpp: layoutng/internal/layout_object.h:1110-1117
    pub fn ChildrenInline(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.children_inline_
    }

    pub fn SetChildrenInline(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.children_inline_ = value;
    }

    // cpp: layoutng/internal/layout_object.h:1420-1442
    pub fn NeedsSimplifiedLayoutOnly(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.needs_simplified_layout_
            && !self.bitfields_.self_needs_full_layout_
            && !self.bitfields_.child_needs_full_layout_
    }

    pub fn SelfNeedsFullLayout(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.self_needs_full_layout_
    }

    pub fn ChildNeedsFullLayout(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.child_needs_full_layout_
    }

    pub fn NeedsSimplifiedLayout(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.needs_simplified_layout_
    }

    // cpp: layoutng/internal/layout_object.h:1478-1507
    pub fn NeedsScrollableOverflowRecalc(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.self_needs_scrollable_overflow_recalc_
            || self.bitfields_.child_needs_scrollable_overflow_recalc_
    }

    pub fn SelfNeedsScrollableOverflowRecalc(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.self_needs_scrollable_overflow_recalc_
    }

    pub fn ChildNeedsScrollableOverflowRecalc(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.child_needs_scrollable_overflow_recalc_
    }

    pub fn SetSelfNeedsScrollableOverflowRecalc(&mut self) {
        self.CheckIsNotDestroyed();
        self.bitfields_.self_needs_scrollable_overflow_recalc_ = true;
    }

    pub fn SetChildNeedsScrollableOverflowRecalc(&mut self) {
        self.CheckIsNotDestroyed();
        self.bitfields_.child_needs_scrollable_overflow_recalc_ = true;
    }

    pub fn ClearSelfNeedsScrollableOverflowRecalc(&mut self) {
        self.CheckIsNotDestroyed();
        self.bitfields_.self_needs_scrollable_overflow_recalc_ = false;
    }

    pub fn ClearChildNeedsScrollableOverflowRecalc(&mut self) {
        self.CheckIsNotDestroyed();
        self.bitfields_.child_needs_scrollable_overflow_recalc_ = false;
    }

    // cpp: layoutng/internal/layout_object.h:3911-3914
    pub fn SetSelfNeedsFullLayout(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.self_needs_full_layout_ = value;
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:259-266
    pub fn SetChildNeedsFullLayout(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.child_needs_full_layout_ = value;
        if value {
            self.bitfields_.is_subgrid_min_max_sizes_cache_dirty_ = true;
            self.bitfields_.is_table_column_constraints_dirty_ = true;
        }
    }

    // cpp: layoutng/internal/layout_tree_invalidation.cc:254-257
    pub fn SetNeedsSimplifiedLayout(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.needs_simplified_layout_ = value;
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:262-269
    pub fn SetNeedsPaintPropertyUpdate(&mut self) {
        self.CheckIsNotDestroyed();
        if self.bitfields_.needs_paint_property_update_ {
            return;
        }
        self.bitfields_.needs_paint_property_update_ = true;
        let parent = self.Parent();
        if !parent.is_null() {
            unsafe { &mut *parent }.SetDescendantNeedsPaintPropertyUpdate();
        }
    }

    // cpp: layoutng/internal/layout_object_core_services.cc:271-279
    pub fn SetDescendantNeedsPaintPropertyUpdate(&mut self) {
        self.CheckIsNotDestroyed();
        let mut object: *mut LayoutObject = self;
        while !object.is_null()
            && !unsafe { &*object }
                .bitfields_
                .descendant_needs_paint_property_update_
        {
            let current = unsafe { &mut *object };
            current.bitfields_.descendant_needs_paint_property_update_ = true;
            object = current.Parent();
        }
    }

    // cpp: layoutng/internal/layout_object.h:2472-2477
    pub fn StylePtr(&self) -> *const ComputedStyle {
        self.CheckIsNotDestroyed();
        self.style_.Get()
    }

    // cpp: layoutng/internal/layout_object.h:3357-3361
    pub(crate) fn SetStyleInternal(&mut self, style: *const ComputedStyle) {
        self.CheckIsNotDestroyed();
        if style.is_null() {
            std::process::abort();
        }
        self.style_ = Member::from_ptr(style as *mut ComputedStyle);
    }

    // cpp: layoutng/internal/layout_object.h:3366-3371
    pub fn ResetStyle(&mut self) {
        self.CheckIsNotDestroyed();
        self.UpdateMaskImageObservers(self.StylePtr(), std::ptr::null());
        self.style_ = Member::default();
    }

    pub fn ImageResourceObserver(&self) -> &ImageResourceObserver {
        &self.image_resource_observer_
    }

    // Source UpdateImageObservers/UpdateFillImages subset for real native
    // mask StyleImages. Add new subscriptions before removing old ones.
    // cpp: core/layout/layout_object.cc:3109-3116,3200,3703-3722
    pub(crate) fn UpdateMaskImageObservers(
        &mut self,
        old: *const ComputedStyle,
        new: *const ComputedStyle,
    ) {
        if self.IsText() {
            return;
        }
        let old_layers = if old.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*old }.MaskLayers() as *const FillLayer
        };
        let new_layers = if new.is_null() {
            std::ptr::null()
        } else {
            unsafe { &*new }.MaskLayers() as *const FillLayer
        };
        let mut a = old_layers;
        let mut b = new_layers;
        let mut identical = true;
        while !a.is_null() || !b.is_null() {
            if a.is_null() || b.is_null() || unsafe { &*a }.GetImage() != unsafe { &*b }.GetImage()
            {
                identical = false;
                break;
            }
            a = unsafe { &*a }.Next();
            b = unsafe { &*b }.Next();
        }
        if identical {
            return;
        }
        let receiver = (self as *mut LayoutObject).cast();
        self.image_resource_observer_.BindReceiver(receiver);
        let observer = &mut self.image_resource_observer_ as *mut ImageResourceObserver;
        let mut layer = new_layers;
        while !layer.is_null() {
            let current = unsafe { &*layer };
            let image = current.GetImage();
            if !image.is_null() {
                unsafe { &mut *image }.AddClient(observer);
            }
            layer = current.Next();
        }
        layer = old_layers;
        while !layer.is_null() {
            let current = unsafe { &*layer };
            let image = current.GetImage();
            if !image.is_null() {
                unsafe { &mut *image }.RemoveClient(observer);
            }
            layer = current.Next();
        }
    }

    // cpp: layoutng/internal/layout_object.h:3073-3080
    pub fn GetMutableForPainting(&self) -> MutableForPainting {
        self.CheckIsNotDestroyed();
        debug_assert!(!PrePaintDisableSideEffectsScope::IsDisabled());
        MutableForPainting::new(self)
    }

    // This is the abstract base constructor used by concrete layout classes.
    // cpp: layoutng/internal/layout_object_fragment_data.cc:35-43
    pub(crate) fn new_base(node: *mut Node) -> Self {
        let fragment = MakeGarbageCollected(FragmentDataList::default());
        #[cfg(debug_assertions)]
        unsafe { &mut *fragment }.SetIsFirst();
        let mut bitfields = LayoutObjectBitfields::default();
        bitfields.is_anonymous_ = node.is_null();
        Self {
            display_item_client_: DisplayItemClient::default(),
            image_resource_observer_: ImageResourceObserver::new_for_derived(
                &LAYOUT_OBJECT_IMAGE_OBSERVER_VTABLE,
            ),
            runtime_class_: Cell::new(LayoutObjectClass::Base),
            bitfields_: bitfields,
            style_: Member::default(),
            node_: Member::from_ptr(node),
            anonymous_input_owner_: Member::default(),
            parent_: Member::default(),
            previous_: Member::default(),
            next_: Member::default(),
            fragment_: Member::from_ptr(fragment),
            #[cfg(debug_assertions)]
            is_destroyed_: false,
        }
    }

    pub fn SetRuntimeClass(&self, class: LayoutObjectClass) {
        self.runtime_class_.set(class);
    }

    pub fn RuntimeClass(&self) -> LayoutObjectClass {
        self.runtime_class_.get()
    }

    // cpp: layoutng/internal/layout_object_fragment_data.cc:52-55
    pub fn EnsureIdForTesting(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { &mut *self.fragment_.Get() }.EnsureId();
    }

    // cpp: layoutng/internal/layout_object_fragment_data.cc:57-60
    pub fn UniqueId(&self) -> UniqueObjectId {
        self.CheckIsNotDestroyed();
        unsafe { &*self.fragment_.Get() }.UniqueId()
    }

    // cpp: layoutng/internal/layout_object_fragment_data.cc:62-65
    pub fn FirstFragment(&self) -> &FragmentData {
        self.CheckIsNotDestroyed();
        unsafe { &*self.fragment_.Get() }
    }

    // cpp: layoutng/internal/layout_object.h:2868-2871
    pub fn FragmentList(&self) -> &FragmentDataList {
        self.CheckIsNotDestroyed();
        unsafe { &*self.fragment_.Get() }
    }

    // cpp: layoutng/internal/layout_object_fragment_data.cc:67-70
    pub fn IsFragmented(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.FragmentList().size() > 1
    }

    // cpp: layoutng/internal/layout_object_fragment_data.cc:89-102
    pub fn Destroy(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(
            ALLOW_DESTROYING_IN_FINALIZER.load(Ordering::Relaxed) != 0
                || !IsLayoutHeapSweepingOnOwningThread()
        );
        // cpp: core/layout/layout_object.cc:4249-4252
        self.UpdateMaskImageObservers(self.StylePtr(), std::ptr::null());
        self.bitfields_.being_destroyed_ = true;
        unsafe { DispatchLayoutObjectWillBeDestroyed(self) };
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.bitfields_.has_ax_object_);
            self.is_destroyed_ = true;
        }
    }

    // cpp: layoutng/internal/layout_object.h:312-317
    pub fn CheckIsNotDestroyed(&self) {
        #[cfg(debug_assertions)]
        debug_assert!(!self.is_destroyed_);
    }

    // cpp: layoutng/internal/layout_object.h:1083-1094
    pub fn SetHasBrokenSpine(&mut self) {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_broken_spine_ = true;
    }

    pub fn ClearHasBrokenSpine(&mut self) {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_broken_spine_ = false;
    }

    pub fn HasBrokenSpine(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_broken_spine_
    }

    // cpp: layoutng/internal/layout_object.h:3172-3175
    // Tree reconciliation holds Persistent handles until pruning. A parent
    // teardown may already have destroyed a retained child; inspect only the
    // lifetime bit, without invoking a live-object accessor on that child.
    pub(crate) fn WasDestroyedForTreeUpdate(&self) -> bool {
        self.bitfields_.being_destroyed_
    }

    pub fn BeingDestroyed(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.being_destroyed_
    }

    // cpp: layoutng/internal/layout_object.h:659-662
    pub fn ShouldApplySizeContainment(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().ContainsSize() && self.IsEligibleForSizeContainment()
    }

    // cpp: layoutng/internal/layout_object.h:1931-1950
    pub fn ToPositionedState(&self, position: EPosition) -> PositionedState {
        self.CheckIsNotDestroyed();
        debug_assert!(
            (position != EPosition::kAbsolute && position != EPosition::kFixed) || self.IsBox()
        );
        match position {
            EPosition::kStatic => PositionedState::kIsStaticallyPositioned,
            EPosition::kRelative => PositionedState::kIsRelativelyPositioned,
            EPosition::kAbsolute | EPosition::kFixed => PositionedState::kIsOutOfFlowPositioned,
            EPosition::kSticky => PositionedState::kIsStickyPositioned,
            _ => unreachable!("unsupported positioned state"),
        }
    }

    // cpp: layoutng/internal/layout_object.h:1952-1955
    pub fn ToPositionedStateCurrentStyle(&self) -> PositionedState {
        self.CheckIsNotDestroyed();
        self.ToPositionedState(self.StyleRef().GetPosition())
    }

    // cpp: layoutng/internal/layout_object.h:358-375
    pub fn Parent(&self) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        self.parent_.Get()
    }

    pub fn PreviousSibling(&self) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        self.previous_.Get()
    }

    pub fn NextSibling(&self) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        self.next_.Get()
    }

    // cpp: layoutng/internal/layout_object.h:1105-1108
    pub fn EverHadLayout(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.ever_had_layout_
    }

    // cpp: layoutng/internal/layout_object.h:1319-1322,1329-1332
    pub fn IsAnonymous(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_anonymous_
    }

    pub fn IsFloating(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_floating_
    }

    // cpp: layoutng/internal/layout_object.h:1340-1374
    pub fn IsOutOfFlowPositioned(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.positioned_state_ == PositionedState::kIsOutOfFlowPositioned
    }

    pub fn IsRelPositioned(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.positioned_state_ == PositionedState::kIsRelativelyPositioned
    }

    pub fn IsStickyPositioned(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.positioned_state_ == PositionedState::kIsStickyPositioned
    }

    pub fn IsPositioned(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.positioned_state_ != PositionedState::kIsStaticallyPositioned
    }

    pub fn IsInline(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_inline_
    }

    pub fn IsInLayoutNGInlineFormattingContext(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_in_layout_ng_inline_formatting_context_
    }

    // cpp: layoutng/internal/layout_object.h:1515-1518
    pub fn HasNonVisibleOverflow(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_non_visible_overflow_
    }

    // cpp: layoutng/internal/layout_object.h:1511-1514
    pub fn HasCSSClip(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsOutOfFlowPositioned() && !self.StyleRef().HasAutoClip()
    }

    // cpp: layoutng/internal/layout_object.h:1624-1627
    pub fn IsEffectiveRootScroller(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_effective_root_scroller_
    }

    // cpp: layoutng/internal/layout_object.h:2041-2058
    pub fn SetOverflowClipAxes(&mut self, axes: OverflowClipAxes) {
        self.CheckIsNotDestroyed();
        self.bitfields_.overflow_clip_axes_ = axes as u8;
    }

    pub fn GetOverflowClipAxes(&self) -> OverflowClipAxes {
        self.CheckIsNotDestroyed();
        self.bitfields_.overflow_clip_axes_ as OverflowClipAxes
    }

    pub fn ShouldClipOverflowAlongEitherAxis(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.GetOverflowClipAxes() != kNoOverflowClip
    }

    pub fn ShouldClipOverflowAlongBothAxis(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.GetOverflowClipAxes() == kOverflowClipBothAxis
    }

    // cpp: layoutng/internal/layout_object.h:1677-1680
    pub fn GetNode(&self) -> *mut Node {
        self.CheckIsNotDestroyed();
        if self.IsAnonymous() {
            std::ptr::null_mut()
        } else {
            self.node_.Get()
        }
    }

    // cpp: layoutng/internal/layout_object.h:2480-2484
    pub fn StyleRef(&self) -> &ComputedStyle {
        self.CheckIsNotDestroyed();
        let style = self.style_.Get();
        debug_assert!(!style.is_null());
        unsafe { &*style }
    }

    // cpp: layoutng/internal/layout_object.h:2577-2580
    pub fn HasReflection(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_reflection_
    }

    // cpp: layoutng/internal/layout_object.h:3092-3095
    pub fn NeedsPaintPropertyUpdate(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.needs_paint_property_update_
    }

    // cpp: layoutng/internal/layout_object.h:1605-1612
    pub fn HasFilterInducingProperty(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().HasNonInitialFilter() || self.HasReflection()
    }
}

// cpp: layoutng/internal/layout_object.h:1755-1796
pub struct AncestorSkipInfo {
    ancestor_: *const LayoutObject,
    check_for_filters_: bool,
    ancestor_skipped_: bool,
    filter_skipped_: bool,
}

impl AncestorSkipInfo {
    // cpp: layoutng/internal/layout_object.h:1759-1761
    pub fn new(ancestor: *const LayoutObject) -> Self {
        Self::new_with_filters(ancestor, false)
    }

    pub fn new_with_filters(ancestor: *const LayoutObject, check_for_filters: bool) -> Self {
        Self {
            ancestor_: ancestor,
            check_for_filters_: check_for_filters,
            ancestor_skipped_: false,
            filter_skipped_: false,
        }
    }

    // cpp: layoutng/internal/layout_object.h:1764-1768
    pub fn Update(&mut self, object: &LayoutObject) {
        if std::ptr::eq(object, self.ancestor_) {
            self.ancestor_skipped_ = true;
        }
        if self.check_for_filters_ && object.HasFilterInducingProperty() {
            self.filter_skipped_ = true;
        }
    }

    // cpp: layoutng/internal/layout_object.h:1771-1775
    #[cfg(debug_assertions)]
    pub fn AssertClean(&self) {
        debug_assert!(!self.ancestor_skipped_);
        debug_assert!(!self.filter_skipped_);
    }

    // cpp: layoutng/internal/layout_object.h:1778-1782
    pub fn AncestorSkipped(&self) -> bool {
        self.ancestor_skipped_
    }

    pub fn FilterSkipped(&self) -> bool {
        debug_assert!(self.check_for_filters_);
        self.filter_skipped_
    }
}

// cpp: layoutng/internal/layout_object.h:3945-3948
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutObjectSide {
    kRemainingTextIfOnBoundary,
    kFirstLetterIfOnBoundary,
}

// The source stores SelectionState in a three-bit field. Keep conversion
// defined even if an invalid bit pattern is observed, rather than constructing
// an invalid Rust enum value.
// cpp: layoutng/internal/layout_object.h:2585-2602
fn SelectionStateFromBits(bits: u8) -> SelectionState {
    match bits {
        0 => SelectionState::kNone,
        1 => SelectionState::kStart,
        2 => SelectionState::kInside,
        3 => SelectionState::kEnd,
        4 => SelectionState::kStartAndEnd,
        5 => SelectionState::kContain,
        _ => std::process::abort(),
    }
}

// The C++ five-bit field can represent more values than the source enum.
// Match every declared value without constructing an invalid Rust enum.
// cpp: layoutng/internal/layout_object.h:2729-2732
fn PaintInvalidationReasonFromBits(bits: u8) -> PaintInvalidationReason {
    match bits {
        0 => PaintInvalidationReason::kNone,
        1 => PaintInvalidationReason::kIncremental,
        2 => PaintInvalidationReason::kHitTest,
        3 => PaintInvalidationReason::kStyle,
        4 => PaintInvalidationReason::kOutline,
        5 => PaintInvalidationReason::kImage,
        6 => PaintInvalidationReason::kBackplate,
        7 => PaintInvalidationReason::kBackground,
        8 => PaintInvalidationReason::kSelection,
        9 => PaintInvalidationReason::kCaret,
        10 => PaintInvalidationReason::kLayout,
        11 => PaintInvalidationReason::kAppeared,
        12 => PaintInvalidationReason::kDisappeared,
        13 => PaintInvalidationReason::kScrollControl,
        14 => PaintInvalidationReason::kSubtree,
        15 => PaintInvalidationReason::kSVGResource,
        16 => PaintInvalidationReason::kDocumentMarker,
        17 => PaintInvalidationReason::kJustCreated,
        18 => PaintInvalidationReason::kReordered,
        19 => PaintInvalidationReason::kChunkAppeared,
        20 => PaintInvalidationReason::kChunkDisappeared,
        21 => PaintInvalidationReason::kChunkUncacheable,
        22 => PaintInvalidationReason::kChunkReordered,
        23 => PaintInvalidationReason::kPaintProperty,
        24 => PaintInvalidationReason::kFullLayer,
        25 => PaintInvalidationReason::kUncacheable,
        _ => std::process::abort(),
    }
}

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:879-898,2585-2602,2630-2660
    pub fn IsLayoutListItemBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsLayoutListItem(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsLayoutListItem(self) }
    }

    pub fn IsInlineListItemBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsInlineListItem(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsInlineListItem(self) }
    }

    pub fn IsLayoutInsideListMarkerBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsLayoutInsideListMarker(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsLayoutInsideListMarker(self) }
    }

    pub fn IsLayoutOutsideListMarkerBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsLayoutOutsideListMarker(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsLayoutOutsideListMarker(self) }
    }

    pub fn GetSelectionState(&self) -> SelectionState {
        self.CheckIsNotDestroyed();
        SelectionStateFromBits(self.bitfields_.selection_state_)
    }

    pub fn SetSelectionState(&mut self, state: SelectionState) {
        self.CheckIsNotDestroyed();
        self.bitfields_.selection_state_ = state as u8;
    }

    pub fn GetSelectionStateForPaint(&self) -> SelectionState {
        self.CheckIsNotDestroyed();
        SelectionStateFromBits(self.bitfields_.selection_state_for_paint_)
    }

    pub fn SetSelectionStateForPaint(&mut self, state: SelectionState) {
        self.CheckIsNotDestroyed();
        self.bitfields_.selection_state_for_paint_ = state as u8;
    }

    pub fn IsListItem(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsLayoutListItem() || self.IsInlineListItem()
    }

    pub fn IsListMarker(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsLayoutInsideListMarker() || self.IsLayoutOutsideListMarker()
    }

    pub fn Remove(&mut self) {
        self.CheckIsNotDestroyed();
        let parent = self.Parent();
        if !parent.is_null() {
            unsafe { &mut *parent }.RemoveChild(self);
        }
    }

    pub fn VisibleToHitTesting(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().VisibleToHitTesting()
    }

    pub fn AffectsWhitespaceSiblings(&self) -> bool {
        self.CheckIsNotDestroyed();
        !self.IsFloatingOrOutOfFlowPositioned() && !self.IsLayoutOutsideListMarker()
    }

    // cpp: layoutng/internal/layout_object.h:750-771,1140-1152,3320-3327
    pub(crate) fn SetPreviousSibling(&mut self, previous: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        self.previous_ = Member::from_ptr(previous);
    }

    pub(crate) fn SetNextSibling(&mut self, next: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        self.next_ = Member::from_ptr(next);
    }

    pub(crate) fn SetParent(&mut self, parent: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        self.parent_ = Member::from_ptr(parent);
        let inside_multicol = !parent.is_null()
            && (unsafe { &*parent }.IsInsideMulticol()
                || unsafe { &*parent }.IsMulticolContainer());
        if inside_multicol != self.IsInsideMulticol() {
            self.SetIsInsideMulticolIncludingDescendants(inside_multicol);
        }
        if !parent.is_null() {
            self.SetDepthIncludingDescendants(unsafe { &*parent }.Depth() + 1);
        }
    }

    // cpp: layoutng/internal/layout_object.h:479-486,776-783
    #[cfg(debug_assertions)]
    pub fn SetHasAXObject(&mut self, has_object: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_ax_object_ = has_object;
    }

    #[cfg(debug_assertions)]
    pub fn HasAXObject(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_ax_object_
    }

    #[cfg(debug_assertions)]
    pub fn IsSetNeedsLayoutForbidden(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.set_needs_layout_forbidden_
    }

    #[cfg(debug_assertions)]
    pub fn SetNeedsLayoutIsForbidden(&mut self, forbidden: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.set_needs_layout_forbidden_ = forbidden;
    }

    // cpp: layoutng/internal/layout_object.h:1957-1984,2033-2099
    pub fn SetPositionState(&mut self, position: PositionedState) {
        self.CheckIsNotDestroyed();
        self.bitfields_.positioned_state_ = position;
    }

    pub fn ClearPositionedState(&mut self) {
        self.CheckIsNotDestroyed();
        self.bitfields_.positioned_state_ = PositionedState::kIsStaticallyPositioned;
    }

    pub fn SetFloating(&mut self, floating: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_floating_ = floating;
    }

    pub fn SetInline(&mut self, inline: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_inline_ = inline;
    }

    pub fn CanTraversePhysicalFragments(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.can_traverse_physical_fragments_
    }

    pub fn SetIsHorizontalWritingMode(&mut self, horizontal: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_horizontal_writing_mode_ = horizontal;
    }

    pub fn SetHasNonVisibleOverflow(&mut self, non_visible: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_non_visible_overflow_ = non_visible;
    }

    pub fn SetHasLayer(&mut self, has_layer: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_layer_ = has_layer;
    }

    pub fn SetHasTransformRelatedProperty(&mut self, has_transform: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_transform_related_property_ = has_transform;
    }

    pub fn SetHasReflection(&mut self, has_reflection: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_reflection_ = has_reflection;
    }

    pub fn SetCanContainAbsolutePositionObjects(&mut self, can_contain: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.can_contain_absolute_position_objects_ = can_contain;
    }

    pub fn SetCanContainFixedPositionObjects(&mut self, can_contain: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.can_contain_fixed_position_objects_ = can_contain;
    }

    pub fn SetIsEffectiveRootScroller(&mut self, effective: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_effective_root_scroller_ = effective;
    }

    pub fn SetIsGlobalRootScroller(&mut self, global: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_global_root_scroller_ = global;
    }

    pub fn SetWhitespaceChildrenMayChange(&mut self, may_change: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.whitespace_children_may_change_ = may_change;
    }

    pub fn WhitespaceChildrenMayChange(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.whitespace_children_may_change_
    }

    pub fn SetNeedsDevtoolsInfo(&mut self, needs_info: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.needs_devtools_info_ = needs_info;
    }

    pub fn NeedsDevtoolsInfo(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.needs_devtools_info_
    }

    // cpp: layoutng/internal/layout_object.h:3114-3165,3177-3205
    pub fn SetIsScrollAnchorObject(&mut self) {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_scroll_anchor_object_ = true;
    }

    pub fn IsBackgroundAttachmentFixedObject(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_background_attachment_fixed_object_
    }

    pub fn CanCompositeBackgroundAttachmentFixed(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.can_composite_background_attachment_fixed_
    }

    pub fn SetOutlineMayBeAffectedByDescendants(&mut self, affected: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.outline_may_be_affected_by_descendants_ = affected;
    }

    pub fn IsTableColumnConstraintsDirty(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_table_column_constraints_dirty_
    }

    pub fn SetTableColumnConstraintsDirty(&mut self, dirty: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_table_column_constraints_dirty_ = dirty;
    }

    pub fn IsGridPlacementDirty(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_grid_placement_dirty_
    }

    pub fn SetGridPlacementDirty(&mut self, dirty: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_grid_placement_dirty_ = dirty;
    }

    pub fn IsSubgridMinMaxSizesCacheDirty(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_subgrid_min_max_sizes_cache_dirty_
    }

    pub fn SetSubgridMinMaxSizesCacheDirty(&mut self, dirty: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_subgrid_min_max_sizes_cache_dirty_ = dirty;
    }

    // cpp: layoutng/internal/layout_object.h:3215-3232
    #[cfg(debug_assertions)]
    pub fn IsInDetachedNonDomTree(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_in_detached_non_dom_tree_
    }

    pub fn SetIsDetachedNonDomRoot(&mut self, detached: bool) {
        self.CheckIsNotDestroyed();
        #[cfg(debug_assertions)]
        {
            debug_assert!(self.Parent().is_null());
            self.bitfields_.is_in_detached_non_dom_tree_ = detached;
        }
        #[cfg(not(debug_assertions))]
        let _ = detached;
    }

    pub fn InheritIsInDetachedNonDomTree(&mut self, parent: &LayoutObject) {
        self.CheckIsNotDestroyed();
        #[cfg(debug_assertions)]
        {
            self.bitfields_.is_in_detached_non_dom_tree_ = parent.IsInDetachedNonDomTree();
        }
        #[cfg(not(debug_assertions))]
        let _ = parent;
    }

    // cpp: layoutng/internal/layout_object.h:3235-3345
    pub fn PreviousVisibilityVisible(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.previous_visibility_visible_
    }

    pub fn IsInclusiveDescendantOfUnboundedElement(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_active_unbounded_element_or_descendant_
    }

    pub fn UpdateIsActiveUnboundedElementOrDescendant(&mut self, inside: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_active_unbounded_element_or_descendant_ = inside;
    }

    pub fn TransformAffectsVectorEffect(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.transform_affects_vector_effect_
    }

    pub fn SVGDescendantMayHaveTransformRelatedOperations(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_
            .svg_descendant_may_have_transform_related_operations_
    }

    pub fn HasViewportDependence(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_viewport_dependence_
    }

    pub fn SetHasViewportDependence(&mut self, dependent: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_viewport_dependence_ = dependent;
    }

    pub fn ShouldSkipNextLayoutShiftTracking(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.should_skip_next_layout_shift_tracking_
    }

    pub fn SetShouldSkipNextLayoutShiftTracking(&mut self, skip: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.should_skip_next_layout_shift_tracking_ = skip;
    }

    pub fn ShouldAssumePaintOffsetTranslationForLayoutShiftTracking(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_
            .should_assume_paint_offset_translation_for_layout_shift_tracking_
    }

    pub fn SetShouldAssumePaintOffsetTranslationForLayoutShiftTracking(&mut self, assume: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_
            .should_assume_paint_offset_translation_for_layout_shift_tracking_ = assume;
    }

    pub fn ScrollableAreaSizeChanged(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.scrollable_area_size_changed_
    }

    pub fn SetScrollableAreaSizeChanged(&mut self, changed: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.scrollable_area_size_changed_ = changed;
    }

    pub fn MayBeNonContiguousIfc(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.may_be_non_contiguous_ifc_
    }

    pub fn SetMayBeNonContiguousIfc(&mut self, may_be: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.may_be_non_contiguous_ifc_ = may_be;
    }

    pub fn HasSVGTextDescendants(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_svg_text_descendants_
    }

    pub fn SetHasSVGTextDescendants(&mut self, has_descendants: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_svg_text_descendants_ = has_descendants;
    }

    pub fn ContainsSelectionFocus(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.contains_selection_focus_
    }

    pub fn SetContainsSelectionFocus(&mut self, contains: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.contains_selection_focus_ = contains;
    }

    // cpp: layoutng/internal/layout_object.h:3449-3452,3493-3524
    pub fn SetEverHadLayout(&mut self) {
        self.CheckIsNotDestroyed();
        self.bitfields_.ever_had_layout_ = true;
    }

    pub fn SetTransformAffectsVectorEffect(&mut self, affects: bool) {
        self.CheckIsNotDestroyed();
        debug_assert!(self.IsSVGChild());
        self.bitfields_.transform_affects_vector_effect_ = affects;
    }

    pub fn ClearSVGDescendantMayHaveTransformRelatedOperations(&mut self) {
        self.CheckIsNotDestroyed();
        debug_assert!(self.IsSVGChild());
        self.bitfields_
            .svg_descendant_may_have_transform_related_operations_ = false;
    }

    pub fn SetCanTraversePhysicalFragments(&mut self, can_traverse: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.can_traverse_physical_fragments_ = can_traverse;
    }

    pub fn GetBackgroundPaintLocation(&self) -> BackgroundPaintLocation {
        self.CheckIsNotDestroyed();
        debug_assert!(self.IsBox());
        self.bitfields_.background_paint_location_
    }

    // cpp: layoutng/internal/layout_object.h:3149-3158,3525-3533
    pub fn BackgroundNeedsFullPaintInvalidation(&self) -> bool {
        self.CheckIsNotDestroyed();
        !self.bitfields_.should_delay_full_paint_invalidation_
            && self.bitfields_.background_needs_full_paint_invalidation_
    }

    pub fn SetBackgroundNeedsFullPaintInvalidation(&mut self) {
        self.CheckIsNotDestroyed();
        self.SetShouldDoFullPaintInvalidationWithoutLayoutChangeInternal(
            PaintInvalidationReason::kBackground,
        );
        self.bitfields_.background_needs_full_paint_invalidation_ = true;
    }

    pub fn SetBackgroundPaintLocation(&mut self, location: BackgroundPaintLocation) {
        self.CheckIsNotDestroyed();
        debug_assert!(self.IsBox());
        if self.GetBackgroundPaintLocation() != location {
            self.SetBackgroundNeedsFullPaintInvalidation();
            self.bitfields_.background_paint_location_ = location;
            debug_assert_eq!(location, self.GetBackgroundPaintLocation());
        }
    }

    // cpp: layoutng/internal/layout_object.h:2128-2151
    pub fn SetConsumesSubtreeChangeNotification(&mut self) {
        self.CheckIsNotDestroyed();
        self.bitfields_.consumes_subtree_change_notification_ = true;
    }

    pub fn WasNotifiedOfSubtreeChange(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.notified_of_subtree_change_
    }

    pub fn HasSubtreeChangeListenerRegistered(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.subtree_change_listener_registered_
    }

    // cpp: layoutng/internal/layout_object.h:2762-2765,2770-2829
    pub fn ShouldCheckForPaintInvalidation(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.should_check_for_paint_invalidation_
    }

    pub fn SubtreeShouldCheckForPaintInvalidation(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.subtree_should_check_for_paint_invalidation_
    }

    pub fn ShouldCheckLayoutForPaintInvalidation(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.should_check_layout_for_paint_invalidation_
    }

    pub fn DescendantShouldCheckLayoutForPaintInvalidation(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_
            .descendant_should_check_layout_for_paint_invalidation_
    }

    pub fn MayNeedPaintInvalidationAnimatedBackgroundImage(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_
            .may_need_paint_invalidation_animated_background_image_
    }

    pub fn ShouldDelayFullPaintInvalidation(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.should_delay_full_paint_invalidation_
    }

    pub fn ShouldInvalidateSelection(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.should_invalidate_selection_
    }

    // cpp: layoutng/internal/layout_object.h:2891-2901,2919-2966
    pub fn GetPrePaintSubtreeWalkReasons(&self) -> PrePaintSubtreeWalkReasons {
        self.CheckIsNotDestroyed();
        PrePaintSubtreeWalkReasons::from_bits(self.bitfields_.pre_paint_subtree_walk_reasons_)
    }

    pub fn GetDescendantPrePaintSubtreeWalkReasons(&self) -> PrePaintSubtreeWalkReasons {
        self.CheckIsNotDestroyed();
        PrePaintSubtreeWalkReasons::from_bits(
            self.bitfields_.descendant_pre_paint_subtree_walk_reasons_,
        )
    }

    pub fn InsideBlockingTouchEventHandler(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.inside_blocking_touch_event_handler_
    }

    pub fn UpdateInsideBlockingTouchEventHandler(&mut self, inside: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.inside_blocking_touch_event_handler_ = inside;
    }

    pub fn InsideBlockingWheelEventHandler(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.inside_blocking_wheel_event_handler_
    }

    pub fn UpdateInsideBlockingWheelEventHandler(&mut self, inside: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.inside_blocking_wheel_event_handler_ = inside;
    }

    pub fn ShouldInheritSoftNavigationContext(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.should_inherit_soft_navigation_context_
    }

    pub fn SetShouldInheritSoftNavigationContext(&mut self, inherit: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.should_inherit_soft_navigation_context_ = inherit;
    }

    pub fn ShouldInheritContainerTimingRoot(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.should_inherit_container_timing_root_
    }

    // cpp: layoutng/internal/layout_object.h:1096-1159,1378-1513
    pub fn IsTruncated(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_truncated_
    }

    pub fn SetIsTruncated(&mut self, truncated: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_truncated_ = truncated;
    }

    pub fn InsideInactiveColumnTab(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.inside_inactive_column_tab_
    }

    pub fn SetInsideInactiveColumnTab(&mut self, inside: bool) {
        self.CheckIsNotDestroyed();
        self.bitfields_.inside_inactive_column_tab_ = inside;
    }

    pub fn IsNonAtomicInline(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsInline() && !self.IsBox()
    }

    pub fn IsHorizontalWritingMode(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_horizontal_writing_mode_
    }

    pub fn IsHorizontalTypographicMode(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsHorizontalWritingMode() || self.StyleRef().IsHorizontalTypographicMode()
    }

    pub fn HasFlippedBlocksWritingMode(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().IsFlippedBlocksWritingMode()
    }

    pub fn HasLayer(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_layer_
    }

    pub fn HasBoxDecorationBackground(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_box_decoration_background_
    }

    pub fn NeedsLayout(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.self_needs_full_layout_
            || self.bitfields_.child_needs_full_layout_
            || self.bitfields_.needs_simplified_layout_
    }

    // cpp: layoutng/internal/layout_object.h:1563-1566
    pub fn HasTransformRelatedProperty(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.has_transform_related_property_
    }

    pub fn IsTransformApplicable(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsBox() || self.IsSVG()
    }

    // cpp: layoutng/internal/layout_object.h:1572-1620
    pub fn Preserves3D(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.HasTransformRelatedProperty() && self.StyleRef().Preserves3D() && !self.IsSVGChild()
    }

    pub fn HasPerspective(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().HasPerspective() && self.HasLayer() && self.IsTransformApplicable()
    }

    pub fn HasMask(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().HasMask()
    }

    pub fn HasClipPath(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().HasClipPath()
    }

    pub fn HasHiddenBackface(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().BackfaceVisibility() == EBackfaceVisibility::kHidden
    }

    pub fn HasNonInitialBackdropFilter(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().HasNonInitialBackdropFilter()
    }

    pub fn HasShapeOutside(&self) -> bool {
        self.CheckIsNotDestroyed();
        !self.StyleRef().ShapeOutside().is_null()
    }

    // cpp: layoutng/internal/layout_object.h:2468-2484,2561-2580,2607-2610
    pub fn HasStyle(&self) -> bool {
        self.CheckIsNotDestroyed();
        !self.style_.Get().is_null()
    }

    pub fn Style(&self) -> *const ComputedStyle {
        self.CheckIsNotDestroyed();
        self.style_.Get()
    }

    // cpp: layoutng/internal/layout_object.h:2847-2850,3353-3364
    pub fn GetSelectionDisplayItemClient(&self) -> *const DisplayItemClient {
        self.CheckIsNotDestroyed();
        std::ptr::null()
    }

    pub fn IsFloatingOrOutOfFlowPositioned(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsFloating() || self.IsOutOfFlowPositioned()
    }

    pub fn LocalSelectionVisualRect(&self) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        PhysicalRect::default()
    }

    // cpp: layoutng/internal/layout_object.h:3344-3349,3398-3405,3567-3570
    pub fn SetDestroyedForTesting(&mut self) {
        self.CheckIsNotDestroyed();
        self.bitfields_.being_destroyed_ = true;
        #[cfg(debug_assertions)]
        {
            self.is_destroyed_ = true;
        }
    }

    pub fn AnonymousHasStylePropagationOverrideBase(&mut self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn InLayoutNGInlineFormattingContextWillChangeBase(&self, _new_value: bool) {
        self.CheckIsNotDestroyed();
    }

    pub fn IsTextOrSVGChild(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsText() || self.IsSVGChild()
    }

    // cpp: layoutng/internal/layout_object.h:1631-1634
    pub fn IsGlobalRootScroller(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.bitfields_.is_global_root_scroller_
    }
}

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:847-850
    pub fn IsBRBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsBR(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsBR(self) }
    }

    // cpp: layoutng/internal/layout_object.h:851-854
    pub fn IsCanvasBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsCanvas(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsCanvas(self) }
    }

    // cpp: layoutng/internal/layout_object.h:855-858
    pub fn IsCounterBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsCounter(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsCounter(self) }
    }

    // cpp: layoutng/internal/layout_object.h:859-862
    pub fn IsEmbeddedObjectBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsEmbeddedObject(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsEmbeddedObject(self) }
    }

    // cpp: layoutng/internal/layout_object.h:867-870
    pub fn IsFrameBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsFrame(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsFrame(self) }
    }

    // cpp: layoutng/internal/layout_object.h:903-906
    pub fn IsListMarkerImageBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsListMarkerImage(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsListMarkerImage(self) }
    }

    // cpp: layoutng/internal/layout_object.h:911-914
    pub fn IsMathMLRootBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsMathMLRoot(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsMathMLRoot(self) }
    }

    // cpp: layoutng/internal/layout_object.h:919-922
    pub fn IsProgressBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsProgress(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsProgress(self) }
    }

    // cpp: layoutng/internal/layout_object.h:923-926
    pub fn IsQuoteBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsQuote(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsQuote(self) }
    }

    // cpp: layoutng/internal/layout_object.h:927-930
    pub fn IsLayoutCustomBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsLayoutCustom(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsLayoutCustom(self) }
    }

    // cpp: layoutng/internal/layout_object.h:951-954
    pub fn IsLayoutImageBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsLayoutImage(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsLayoutImage(self) }
    }

    // cpp: layoutng/internal/layout_object.h:955-958
    pub fn IsLayoutImageReplacementBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsLayoutImageReplacement(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsLayoutImageReplacement(self) }
    }

    // cpp: layoutng/internal/layout_object.h:963-966
    pub fn IsLayoutCustomScrollbarPartBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsLayoutCustomScrollbarPart(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsLayoutCustomScrollbarPart(self) }
    }

    // cpp: layoutng/internal/layout_object.h:971-974
    pub fn IsRubyBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsRuby(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsRuby(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1001-1004
    pub fn IsTextControlInnerEditorBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsTextControlInnerEditor(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsTextControlInnerEditor(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1017-1020
    pub fn IsViewTransitionContentBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsViewTransitionContent(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsViewTransitionContent(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1021-1024
    pub fn IsViewTransitionRootBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsViewTransitionRoot(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsViewTransitionRoot(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1182-1185
    pub fn IsSVGTransformableContainerBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsSVGTransformableContainer(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsSVGTransformableContainer(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1186-1189
    pub fn IsSVGViewportContainerBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsSVGViewportContainer(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsSVGViewportContainer(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1198-1201
    pub fn IsSVGTextPathBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsSVGTextPath(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsSVGTextPath(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1202-1205
    pub fn IsSVGTSpanBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsSVGTSpan(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsSVGTSpan(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1222-1225
    pub fn IsSVGResourceContainerBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsSVGResourceContainer(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsSVGResourceContainer(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1226-1229
    pub fn IsSVGFilterPrimitiveBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    pub fn IsSVGFilterPrimitive(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsSVGFilterPrimitive(self) }
    }
}

impl LayoutObject {
    #[cfg(debug_assertions)]
    // cpp: layoutng/internal/layout_object.h:501-506
    pub fn AssertLaidOut(&self) {
        self.CheckIsNotDestroyed();
        if self.NeedsLayout() && !self.ChildLayoutBlockedByDisplayLock() {
            self.ShowLayoutTreeForThis();
        }
        debug_assert!(!self.NeedsLayout() || self.ChildLayoutBlockedByDisplayLock());
    }

    #[cfg(debug_assertions)]
    // cpp: layoutng/internal/layout_object.h:508-516
    pub fn AssertSubtreeIsLaidOut(&self) {
        self.CheckIsNotDestroyed();
        let mut object: *const LayoutObject = self;
        while !object.is_null() {
            let current = unsafe { &*object };
            current.AssertLaidOut();
            object = if current.ChildLayoutBlockedByDisplayLock() {
                current.NextInPreOrderAfterChildren(self)
            } else {
                current.NextInPreOrder(self)
            };
        }
    }

    #[cfg(all(debug_assertions, feature = "expensive_dchecks"))]
    // cpp: layoutng/internal/layout_object.h:528-536
    pub fn AssertSubtreeClearedPaintInvalidationFlags(&self) {
        self.CheckIsNotDestroyed();
        let mut object: *const LayoutObject = self;
        while !object.is_null() {
            let current = unsafe { &*object };
            current.AssertClearedPaintInvalidationFlags();
            object = if current.ChildPrePaintBlockedByDisplayLock() {
                current.NextInPreOrderAfterChildren(self)
            } else {
                current.NextInPreOrder(self)
            };
        }
    }

    #[cfg(all(debug_assertions, feature = "expensive_dchecks"))]
    // cpp: layoutng/internal/layout_object.h:526-526
    pub fn AssertClearedPaintInvalidationFlags(&self) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectAssertClearedPaintInvalidationFlagsProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:3168-3168
    pub fn ChildPrePaintBlockedByDisplayLock(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectChildPrePaintBlockedByDisplayLockProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2820-2820
    pub fn SetShouldDelayFullPaintInvalidation(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectSetShouldDelayFullPaintInvalidationProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2963-2963
    pub fn SetShouldInheritContainerTimingRoot(&mut self, inherit: bool) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectSetShouldInheritContainerTimingRootProvider(self, inherit) }
    }

    // cpp: layoutng/internal/layout_object.h:3444-3444
    pub fn ClearPaintFlags(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectClearPaintFlagsProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:3443-3443
    pub fn EnsureIsReadyForPaintInvalidation(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectEnsureIsReadyForPaintInvalidationProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2854-2857
    pub fn UpdateAnonymousChildStyleBase(
        &self,
        _child: *const LayoutObject,
        _builder: &mut ComputedStyleBuilder,
    ) {
        self.CheckIsNotDestroyed();
    }

    // cpp: layoutng/internal/layout_object.h:2854-2857
    pub fn UpdateAnonymousChildStyle(
        &self,
        child: *const LayoutObject,
        builder: &mut ComputedStyleBuilder,
    ) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectUpdateAnonymousChildStyle(self, child, builder) }
    }

    // cpp: layoutng/internal/layout_object.h:1357-1361
    pub fn IsAbsolutePositioned(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsOutOfFlowPositioned() && self.StyleRef().GetPosition() == EPosition::kAbsolute
    }

    // cpp: layoutng/internal/layout_object.h:1537-1540
    pub fn IsContentMovingOverscrollContainer(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsBox() && self.StyleRef().IsContentMovingOverscrollContainer()
    }

    // cpp: layoutng/internal/layout_object.h:1555-1559
    pub fn IsScrollContainerWithScrollMarkerGroup(&self) -> bool {
        self.CheckIsNotDestroyed();
        (self.IsScrollContainer() || self.IsDocumentElement())
            && !self.StyleRef().ScrollMarkerGroupNone()
    }

    // cpp: layoutng/internal/layout_object.h:2023-2029
    pub fn StitchedRowGapIndexBase(
        &self,
        _fragment: &PhysicalBoxFragment,
        gap_index: usize,
        _line_index: Option<usize>,
    ) -> usize {
        self.CheckIsNotDestroyed();
        gap_index
    }

    // cpp: layoutng/internal/layout_object.h:2023-2029
    pub fn StitchedRowGapIndex(
        &self,
        fragment: &PhysicalBoxFragment,
        gap_index: usize,
        line_index: Option<usize>,
    ) -> usize {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectStitchedRowGapIndex(self, fragment, gap_index, line_index) }
    }

    // cpp: layoutng/internal/layout_object.h:2492-2497
    pub fn EffectiveStyle(&self, style_variant: StyleVariant) -> &ComputedStyle {
        self.CheckIsNotDestroyed();
        if style_variant == StyleVariant::kStandard {
            self.StyleRef()
        } else {
            self.SlowEffectiveStyle(style_variant)
        }
    }

    // cpp: layoutng/internal/layout_object.h:2499-2502
    pub fn ResolveColorWithStyle(style: &ComputedStyle, color_property: &Longhand) -> Color {
        style.VisitedDependentColor(color_property, None)
    }

    // cpp: layoutng/internal/layout_object.h:2504-2507
    pub fn ResolveColor(&self, color_property: &Longhand) -> Color {
        self.CheckIsNotDestroyed();
        self.StyleRef().VisitedDependentColor(color_property, None)
    }

    // cpp: layoutng/internal/layout_object.h:2687-2691
    pub fn CreatesGroup(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.StyleRef().HasGroupingProperty(self.HasReflection())
    }

    // cpp: layoutng/internal/layout_object.h:2906-2911
    pub fn EffectiveAllowedTouchAction(&self) -> TouchAction {
        self.CheckIsNotDestroyed();
        if self.InsideBlockingTouchEventHandler() {
            TouchAction::kNone
        } else {
            self.StyleRef().EffectiveTouchAction()
        }
    }

    // cpp: layoutng/internal/layout_object.h:2912-2915
    pub fn HasEffectiveAllowedTouchAction(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.EffectiveAllowedTouchAction() != TouchAction::kAuto
    }

    // cpp: layoutng/internal/layout_object.h:326-326
    pub fn DecoratedName(&self) -> BlinkString {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectDecoratedNameProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:334-334
    pub fn HasDistortingVisualEffects(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectHasDistortingVisualEffectsProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:337-337
    pub fn HasNonZeroEffectiveOpacity(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectHasNonZeroEffectiveOpacityProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:448-448
    pub fn EnclosingLayer(&self) -> *mut PaintLayer {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectEnclosingLayerProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:449-449
    pub fn AddLayers(&mut self, parent_layer: *mut PaintLayer) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectAddLayersProvider(self, parent_layer) }
    }

    // cpp: layoutng/internal/layout_object.h:450-450
    pub fn RemoveLayers(&mut self, parent_layer: *mut PaintLayer) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectRemoveLayersProvider(self, parent_layer) }
    }

    // cpp: layoutng/internal/layout_object.h:451-451
    pub fn MoveLayers(&mut self, old_parent: *mut PaintLayer, new_parent: *mut PaintLayer) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectMoveLayersProvider(self, old_parent, new_parent) }
    }

    // cpp: layoutng/internal/layout_object.h:452-454
    pub fn FindNextLayer(
        &mut self,
        parent_layer: *mut PaintLayer,
        start_point: *mut LayoutObject,
        check_parent: bool,
    ) -> *mut PaintLayer {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectFindNextLayerProvider(self, parent_layer, start_point, check_parent) }
    }

    // cpp: layoutng/internal/layout_object.h:452-454
    pub fn FindNextLayerDefault(
        &mut self,
        parent_layer: *mut PaintLayer,
        start_point: *mut LayoutObject,
    ) -> *mut PaintLayer {
        self.FindNextLayer(parent_layer, start_point, true)
    }

    // cpp: layoutng/internal/layout_object.h:458-458
    pub fn PaintingLayer(&self, max_depth: i32) -> *mut PaintLayer {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectPaintingLayerProvider(self, max_depth) }
    }

    // cpp: layoutng/internal/layout_object.h:458-458
    pub fn PaintingLayerDefault(&self) -> *mut PaintLayer {
        self.PaintingLayer(-1)
    }

    // cpp: layoutng/internal/layout_object.h:833-833
    pub fn IsBackdropForOverscrollAreaParent(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectIsBackdropForOverscrollAreaParentProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1051-1051
    pub fn IsHR(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectIsHRProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1053-1053
    pub fn IsInputButton(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectIsInputButtonProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1054-1054
    pub fn IsMenuList(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectIsMenuListProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1519-1519
    pub fn HasClipRelatedProperty(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectHasClipRelatedPropertyProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1655-1655
    pub fn IsScrollMarker(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectIsScrollMarkerProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1657-1657
    pub fn IsScrollMarkerGroupBefore(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectIsScrollMarkerGroupBeforeProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1661-1661
    pub fn IsListMarkerForSummary(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectIsListMarkerForSummaryProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1664-1664
    pub fn IsInListMarker(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutObjectIsInListMarkerProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:745-745
    pub fn NotifyPriorityScrollAnchorStatusChanged(&mut self) {
        unsafe { LayoutObjectNotifyPriorityScrollAnchorStatusChangedProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:786-786
    pub fn AddAbsoluteRectForLayer(&mut self, result: &mut gfx::Rect) {
        unsafe { LayoutObjectAddAbsoluteRectForLayerProvider(self, result) }
    }

    // cpp: layoutng/internal/layout_object.h:827-827
    pub fn CreateObject(element: *mut Element, style: &ComputedStyle) -> *mut LayoutObject {
        unsafe { LayoutObjectCreateObjectProvider(element, style) }
    }

    // cpp: layoutng/internal/layout_object.h:828-829
    pub fn CreateBlockFlowOrListItem(
        element: *mut Element,
        style: &ComputedStyle,
    ) -> *mut LayoutBlockFlow {
        unsafe { LayoutObjectCreateBlockFlowOrListItemProvider(element, style) }
    }

    // cpp: layoutng/internal/layout_object.h:1698-1698
    pub fn GeneratingNode(&self) -> *mut Node {
        unsafe { LayoutObjectGeneratingNodeProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1702-1702
    pub fn EnclosingNode(&self) -> *mut Node {
        unsafe { LayoutObjectEnclosingNodeProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1853-1853
    pub fn ScrollParent(&self, base: *const Element) -> *mut Element {
        unsafe { LayoutObjectScrollParentProvider(self, base) }
    }

    // cpp: layoutng/internal/layout_object.h:1853-1853
    pub fn ScrollParentDefault(&self) -> *mut Element {
        self.ScrollParent(std::ptr::null())
    }

    // cpp: layoutng/internal/layout_object.h:1857-1857
    pub fn OffsetParent(&self, base: *const Element) -> *mut Element {
        unsafe { LayoutObjectOffsetParentProvider(self, base) }
    }

    // cpp: layoutng/internal/layout_object.h:1857-1857
    pub fn OffsetParentDefault(&self) -> *mut Element {
        self.OffsetParent(std::ptr::null())
    }

    // cpp: layoutng/internal/layout_object.h:1914-1914
    pub fn InvalidateSubtreePositionTry(&mut self, mark_style_dirty: bool) {
        unsafe { LayoutObjectInvalidateSubtreePositionTryProvider(self, mark_style_dirty) }
    }

    // cpp: layoutng/internal/layout_object.h:1919-1919
    pub fn IsCanvasOrInCanvasSubtree(&self) -> bool {
        unsafe { LayoutObjectIsCanvasOrInCanvasSubtreeProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1920-1920
    pub fn IsInCanvasSubtree(&self) -> bool {
        unsafe { LayoutObjectIsInCanvasSubtreeProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2113-2113
    pub fn RecalcNormalFlowChildVisualOverflowIfNeeded(&mut self) {
        unsafe { LayoutObjectRecalcNormalFlowChildVisualOverflowIfNeededProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2593-2593
    pub fn CanUpdateSelectionOnRootLineBoxes(&self) -> bool {
        unsafe { LayoutObjectCanUpdateSelectionOnRootLineBoxesProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2612-2612
    pub fn AbsoluteSelectionRect(&self) -> PhysicalRect {
        unsafe { LayoutObjectAbsoluteSelectionRectProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2614-2614
    pub fn CanBeSelectionLeaf(&self) -> bool {
        unsafe { LayoutObjectCanBeSelectionLeafProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2615-2615
    pub fn IsSelected(&self) -> bool {
        unsafe { LayoutObjectIsSelectedProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2616-2616
    pub fn IsSelectable(&self) -> bool {
        unsafe { LayoutObjectIsSelectableProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2651-2651
    pub fn VisibleToHitTestRequest(&self, request: &HitTestRequest) -> bool {
        unsafe { LayoutObjectVisibleToHitTestRequestProvider(self, request) }
    }

    // cpp: layoutng/internal/layout_object.h:2674-2674
    pub fn CanvasForDrawingLayoutObject(&self) -> *mut LayoutObject {
        unsafe { LayoutObjectCanvasForDrawingLayoutObjectProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2761-2761
    pub fn SetShouldInvalidatePaintForHitTest(&mut self) {
        unsafe { LayoutObjectSetShouldInvalidatePaintForHitTestProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2768-2768
    pub fn ClearPaintInvalidationFlags(&mut self) {
        unsafe { LayoutObjectClearPaintInvalidationFlagsProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2802-2802
    pub fn SetMayNeedPaintInvalidationAnimatedBackgroundImage(&mut self) {
        unsafe { LayoutObjectSetMayNeedPaintInvalidationAnimatedBackgroundImageProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2821-2821
    pub fn ClearShouldDelayFullPaintInvalidation(&mut self) {
        unsafe { LayoutObjectClearShouldDelayFullPaintInvalidationProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2827-2827
    pub fn SetShouldInvalidateSelection(&mut self) {
        unsafe { LayoutObjectSetShouldInvalidateSelectionProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2882-2882
    pub fn InvalidateSelectionOnStyleChange(&mut self) {
        unsafe { LayoutObjectInvalidateSelectionOnStyleChangeProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2891-2891
    pub fn SetNeedsPrePaintSubtreeWalk(&mut self, reasons: PrePaintSubtreeWalkReasons) {
        unsafe { LayoutObjectSetNeedsPrePaintSubtreeWalkProvider(self, reasons) }
    }

    // cpp: layoutng/internal/layout_object.h:2897-2897
    pub fn SetDescendantNeedsPrePaintSubtreeWalk(&mut self, reasons: PrePaintSubtreeWalkReasons) {
        unsafe { LayoutObjectSetDescendantNeedsPrePaintSubtreeWalkProvider(self, reasons) }
    }

    // cpp: layoutng/internal/layout_object.h:2923-2923
    pub fn MarkEffectiveAllowedTouchActionChanged(&mut self) {
        unsafe { LayoutObjectMarkEffectiveAllowedTouchActionChangedProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2938-2938
    pub fn MarkBlockingWheelEventHandlerChanged(&mut self) {
        unsafe { LayoutObjectMarkBlockingWheelEventHandlerChangedProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2944-2944
    pub fn MarkSoftNavigationContextChanged(&mut self) {
        unsafe { LayoutObjectMarkSoftNavigationContextChangedProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2958-2958
    pub fn MarkContainerTimingChanged(&mut self) {
        unsafe { LayoutObjectMarkContainerTimingChangedProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:3170-3170
    pub fn ChildPaintBlockedByDisplayLock(&self) -> bool {
        unsafe { LayoutObjectChildPaintBlockedByDisplayLockProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:3120-3120
    pub fn MaybeClearIsScrollAnchorObject(&mut self) {
        unsafe { LayoutObjectMaybeClearIsScrollAnchorObjectProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:3264-3264
    pub fn SetSVGDescendantMayHaveTransformRelatedOperations(&mut self) {
        unsafe { LayoutObjectSetSVGDescendantMayHaveTransformRelatedOperationsProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:3341-3341
    pub fn UpdateAfterReinsert(&mut self, old_style: &ComputedStyle) {
        unsafe { LayoutObjectUpdateAfterReinsertProvider(self, old_style) }
    }

    // cpp: layoutng/internal/layout_object.h:3446-3446
    pub(crate) fn SetIsBackgroundAttachmentFixedObject(&mut self, value: bool) {
        unsafe { LayoutObjectSetIsBackgroundAttachmentFixedObjectProvider(self, value) }
    }

    // cpp: layoutng/internal/layout_object.h:3447-3447
    pub(crate) fn SetCanCompositeBackgroundAttachmentFixed(&mut self, value: bool) {
        unsafe { LayoutObjectSetCanCompositeBackgroundAttachmentFixedProvider(self, value) }
    }

    // cpp: layoutng/internal/layout_object.h:3474-3474
    pub(crate) fn BackgroundIsKnownToBeObscured(&self) -> bool {
        unsafe { LayoutObjectBackgroundIsKnownToBeObscuredProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:3475-3475
    pub(crate) fn SetBackgroundIsKnownToBeObscured(&mut self, value: bool) {
        unsafe { LayoutObjectSetBackgroundIsKnownToBeObscuredProvider(self, value) }
    }

    // cpp: layoutng/internal/layout_object.h:3491-3491
    pub(crate) fn FirstLineStyleWithoutFallback(&self) -> *const ComputedStyle {
        unsafe { LayoutObjectFirstLineStyleWithoutFallbackProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:3579-3579
    fn MarkSelfPaintingLayerForVisualOverflowRecalc(&mut self) {
        unsafe { LayoutObjectMarkSelfPaintingLayerForVisualOverflowRecalcProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2725-2725
    pub fn GetImageOrientation(object: *const LayoutObject) -> RespectImageOrientationEnum {
        unsafe { LayoutObjectGetImageOrientationProvider(object) }
    }

    // cpp: layoutng/internal/layout_object.h:3540-3540
    fn AddAsImageObserver(&mut self, image: *mut StyleImage) {
        unsafe { LayoutObjectAddAsImageObserverProvider(self, image) }
    }

    // cpp: layoutng/internal/layout_object.h:3541-3541
    fn RemoveAsImageObserver(&mut self, image: *mut StyleImage) {
        unsafe { LayoutObjectRemoveAsImageObserverProvider(self, image) }
    }

    // cpp: layoutng/internal/layout_object.h:3543-3543
    fn UpdateImage(&mut self, old_image: *mut StyleImage, new_image: *mut StyleImage) {
        unsafe { LayoutObjectUpdateImageProvider(self, old_image, new_image) }
    }

    // cpp: layoutng/internal/layout_object.h:3544-3544
    fn UpdateShapeImage(&mut self, old_shape: *const ShapeValue, new_shape: *const ShapeValue) {
        unsafe { LayoutObjectUpdateShapeImageProvider(self, old_shape, new_shape) }
    }

    // cpp: layoutng/internal/layout_object.h:3545-3546
    fn UpdateFillImages(&mut self, old_layers: *const FillLayer, new_layers: *const FillLayer) {
        unsafe { LayoutObjectUpdateFillImagesProvider(self, old_layers, new_layers) }
    }

    // cpp: layoutng/internal/layout_object.h:3547-3548
    fn UpdateCursorImages(
        &mut self,
        old_cursors: *const CursorList,
        new_cursors: *const CursorList,
    ) {
        unsafe { LayoutObjectUpdateCursorImagesProvider(self, old_cursors, new_cursors) }
    }

    // cpp: layoutng/internal/layout_object.h:3561-3561
    fn InvalidateSelectedChildrenOnStyleChange(&mut self) {
        unsafe { LayoutObjectInvalidateSelectedChildrenOnStyleChangeProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:3565-3565
    fn AdjustStyleDifference(&self, difference: StyleDifference) -> StyleDifference {
        unsafe { LayoutObjectAdjustStyleDifferenceProvider(self, difference) }
    }

    // cpp: layoutng/internal/layout_object.h:3572-3573
    fn UpdateImageObservers(
        &mut self,
        old_style: *const ComputedStyle,
        new_style: *const ComputedStyle,
    ) {
        unsafe { LayoutObjectUpdateImageObserversProvider(self, old_style, new_style) }
    }

    // cpp: layoutng/internal/layout_object.h:3574-3574
    fn UpdateFirstLineImageObservers(&mut self, new_style: *const ComputedStyle) {
        unsafe { LayoutObjectUpdateFirstLineImageObserversProvider(self, new_style) }
    }

    // cpp: layoutng/internal/layout_object.h:3576-3576
    fn ApplyPseudoElementStyleChanges(&mut self, old_style: *const ComputedStyle) {
        unsafe { LayoutObjectApplyPseudoElementStyleChangesProvider(self, old_style) }
    }

    // cpp: layoutng/internal/layout_object.h:3577-3577
    fn ApplyFirstLineChanges(&mut self, old_style: *const ComputedStyle) {
        unsafe { LayoutObjectApplyFirstLineChangesProvider(self, old_style) }
    }
}

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:2254-2255
    pub fn ContainingScrollContainerAnyAxis(
        &self,
        ignore_layout_view_for_fixed_pos: bool,
    ) -> *const LayoutBox {
        unsafe {
            LayoutObjectContainingScrollContainerAnyAxisProvider(
                self,
                ignore_layout_view_for_fixed_pos,
            )
        }
    }

    // cpp: layoutng/internal/layout_object.h:2254-2255
    pub fn ContainingScrollContainerAnyAxisDefault(&self) -> *const LayoutBox {
        self.ContainingScrollContainerAnyAxis(false)
    }

    // cpp: layoutng/internal/layout_object.h:2262-2263
    pub fn ContainingScrollContainerLayerAnyAxis(
        &self,
        ignore_layout_view_for_fixed_pos: bool,
    ) -> *const PaintLayer {
        unsafe {
            LayoutObjectContainingScrollContainerLayerAnyAxisProvider(
                self,
                ignore_layout_view_for_fixed_pos,
            )
        }
    }

    // cpp: layoutng/internal/layout_object.h:2262-2263
    pub fn ContainingScrollContainerLayerAnyAxisDefault(&self) -> *const PaintLayer {
        self.ContainingScrollContainerLayerAnyAxis(false)
    }

    // cpp: layoutng/internal/layout_object.h:2268-2270
    pub fn ContainingScrollContainerLayerForAxis(
        &self,
        axis: PhysicalAxis,
        ignore_layout_view_for_fixed_pos: bool,
    ) -> *const PaintLayer {
        unsafe {
            LayoutObjectContainingScrollContainerLayerForAxisProvider(
                self,
                axis,
                ignore_layout_view_for_fixed_pos,
            )
        }
    }

    // cpp: layoutng/internal/layout_object.h:2268-2270
    pub fn ContainingScrollContainerLayerForAxisDefault(
        &self,
        axis: PhysicalAxis,
    ) -> *const PaintLayer {
        self.ContainingScrollContainerLayerForAxis(axis, false)
    }

    // cpp: layoutng/internal/layout_object.h:2410-2410
    pub fn OffsetFromAncestor(&self, ancestor: *const LayoutObject) -> PhysicalOffset {
        unsafe { LayoutObjectOffsetFromAncestorProvider(self, ancestor) }
    }

    // cpp: layoutng/internal/layout_object.h:2412-2412
    pub fn AbsoluteBoundingBoxRectF(&self, mode: MapCoordinatesFlags) -> gfx::RectF {
        unsafe { LayoutObjectAbsoluteBoundingBoxRectFProvider(self, mode) }
    }

    // cpp: layoutng/internal/layout_object.h:2412-2412
    pub fn AbsoluteBoundingBoxRectFDefault(&self) -> gfx::RectF {
        self.AbsoluteBoundingBoxRectF(MapCoordinatesFlags::default())
    }

    // cpp: layoutng/internal/layout_object.h:2416-2416
    pub fn AbsoluteBoundingBoxRect(&self, mode: MapCoordinatesFlags) -> gfx::Rect {
        unsafe { LayoutObjectAbsoluteBoundingBoxRectProvider(self, mode) }
    }

    // cpp: layoutng/internal/layout_object.h:2416-2416
    pub fn AbsoluteBoundingBoxRectDefault(&self) -> gfx::Rect {
        self.AbsoluteBoundingBoxRect(MapCoordinatesFlags::default())
    }

    // cpp: layoutng/internal/layout_object.h:2422-2422
    pub fn AbsoluteBoundingBoxRectForUnboundedElement(&self) -> gfx::Rect {
        unsafe { LayoutObjectAbsoluteBoundingBoxRectForUnboundedElementProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2430-2431
    pub fn AbsoluteBoundingBoxRectHandlingEmptyInline(
        &self,
        flags: MapCoordinatesFlags,
    ) -> PhysicalRect {
        unsafe { LayoutObjectAbsoluteBoundingBoxRectHandlingEmptyInlineProvider(self, flags) }
    }

    // cpp: layoutng/internal/layout_object.h:2430-2431
    pub fn AbsoluteBoundingBoxRectHandlingEmptyInlineDefault(&self) -> PhysicalRect {
        self.AbsoluteBoundingBoxRectHandlingEmptyInline(MapCoordinatesFlags::default())
    }

    // cpp: layoutng/internal/layout_object.h:2434-2434
    pub fn AbsoluteBoundingBoxRectForScrollIntoView(&self) -> PhysicalRect {
        unsafe { LayoutObjectAbsoluteBoundingBoxRectForScrollIntoViewProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2456-2456
    pub fn AbsoluteBoundingBoxRectIncludingDescendants(&self) -> gfx::Rect {
        unsafe { LayoutObjectAbsoluteBoundingBoxRectIncludingDescendantsProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1671-1671
    pub fn GetCachedPseudoElementStyle(&self, pseudo_id: PseudoId) -> *const ComputedStyle {
        unsafe { LayoutObjectGetCachedPseudoElementStyleProvider(self, pseudo_id) }
    }

    // cpp: layoutng/internal/layout_object.h:1672-1672
    pub fn GetUncachedPseudoElementStyle(&self, request: &StyleRequest) -> *const ComputedStyle {
        unsafe { LayoutObjectGetUncachedPseudoElementStyleProvider(self, request) }
    }

    // cpp: layoutng/internal/layout_object.h:2187-2187
    pub fn PositionForPoint(&self, point: &PhysicalOffset) -> PositionWithAffinity {
        unsafe { DispatchLayoutObjectPositionForPoint(self, point) }
    }

    // cpp: layoutng/internal/layout_object.h:2188-2189
    pub fn CreatePositionWithAffinity(
        &self,
        offset: i32,
        affinity: TextAffinity,
    ) -> PositionWithAffinity {
        unsafe { LayoutObjectCreatePositionWithAffinityProvider(self, offset, affinity) }
    }

    // cpp: layoutng/internal/layout_object.h:2190-2190
    pub fn CreatePositionWithAffinityDefault(&self, offset: i32) -> PositionWithAffinity {
        unsafe { LayoutObjectCreatePositionWithAffinityDefaultProvider(self, offset) }
    }

    // cpp: layoutng/internal/layout_object.h:2191-2191
    pub fn FindPosition(&self) -> PositionWithAffinity {
        unsafe { LayoutObjectFindPositionProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2192-2192
    pub fn FirstPositionInOrBeforeThis(&self) -> PositionWithAffinity {
        unsafe { LayoutObjectFirstPositionInOrBeforeThisProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2193-2193
    pub fn LastPositionInOrAfterThis(&self) -> PositionWithAffinity {
        unsafe { LayoutObjectLastPositionInOrAfterThisProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2194-2194
    pub fn PositionAfterThis(&self) -> PositionWithAffinity {
        unsafe { LayoutObjectPositionAfterThisProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2195-2195
    pub fn PositionBeforeThis(&self) -> PositionWithAffinity {
        unsafe { LayoutObjectPositionBeforeThisProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2218-2219
    pub fn SetModifiedStyleOutsideStyleRecalc(
        &mut self,
        style: *const ComputedStyle,
        apply_changes: ApplyStyleChanges,
    ) {
        unsafe {
            LayoutObjectSetModifiedStyleOutsideStyleRecalcProvider(self, style, apply_changes)
        }
    }

    // cpp: layoutng/internal/layout_object.h:791-793
    #[cfg(debug_assertions)]
    pub fn DumpLayoutObject(
        &self,
        output: &mut StringBuilder,
        dump_address: bool,
        show_tree_character_offset: u32,
    ) {
        unsafe {
            LayoutObjectDumpLayoutObjectProvider(
                self,
                output,
                dump_address,
                show_tree_character_offset,
            )
        }
    }

    // cpp: layoutng/internal/layout_object.h:794-794
    #[cfg(debug_assertions)]
    pub fn ShowTreeForThis(&self) {
        unsafe { LayoutObjectShowTreeForThisProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:796-796
    #[cfg(debug_assertions)]
    pub fn ShowLayoutObject(&self) {
        unsafe { LayoutObjectShowLayoutObjectProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:803-808
    #[cfg(debug_assertions)]
    pub fn DumpLayoutTreeAndMark(
        &self,
        output: &mut StringBuilder,
        marked_object1: *const LayoutObject,
        marked_label1: *const c_char,
        marked_object2: *const LayoutObject,
        marked_label2: *const c_char,
        depth: u32,
    ) {
        unsafe {
            LayoutObjectDumpLayoutTreeAndMarkProvider(
                self,
                output,
                marked_object1,
                marked_label1,
                marked_object2,
                marked_label2,
                depth,
            )
        }
    }

    // cpp: layoutng/internal/layout_object.h:803-808
    #[cfg(debug_assertions)]
    pub fn DumpLayoutTreeAndMarkDefault(&self, output: &mut StringBuilder) {
        self.DumpLayoutTreeAndMark(
            output,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            0,
        )
    }

    // cpp: layoutng/internal/layout_object.h:524-524
    #[cfg(all(debug_assertions, feature = "expensive_dchecks"))]
    pub fn AssertFragmentTree(&self, display_locked: bool) {
        unsafe { LayoutObjectAssertFragmentTreeProvider(self, display_locked) }
    }

    // cpp: layoutng/internal/layout_object.h:524-524
    #[cfg(all(debug_assertions, feature = "expensive_dchecks"))]
    pub fn AssertFragmentTreeDefault(&self) {
        self.AssertFragmentTree(false)
    }

    // cpp: layoutng/internal/layout_object.h:2559-2559
    pub fn HitTestForOcclusion(&self, rect: &PhysicalRect) -> HitTestResult {
        unsafe { LayoutObjectHitTestForOcclusionProvider(self, rect) }
    }

    // cpp: layoutng/internal/layout_object.h:2626-2626
    pub fn DestroyAndCleanupAnonymousWrappers(&mut self, performing_reattach: bool) {
        unsafe { LayoutObjectDestroyAndCleanupAnonymousWrappersProvider(self, performing_reattach) }
    }

    // cpp: layoutng/internal/layout_object.h:2696-2698
    pub fn CollectOutlineRectsAndAdvance(
        &self,
        outline_type: OutlineType,
        iterator: &mut AccompaniedFragmentIterator,
    ) -> Vector<PhysicalRect> {
        unsafe { LayoutObjectCollectOutlineRectsAndAdvanceProvider(self, outline_type, iterator) }
    }

    // cpp: layoutng/internal/layout_object.h:2704-2706
    pub fn OutlineRects(
        &self,
        info: *mut LayoutOutlineInfo,
        additional_offset: &PhysicalOffset,
        outline_type: OutlineType,
    ) -> Vector<PhysicalRect> {
        unsafe { LayoutObjectOutlineRectsProvider(self, info, additional_offset, outline_type) }
    }

    // cpp: layoutng/internal/layout_object.h:3207-3207
    pub fn GetDisplayLockContext(&self) -> *mut DisplayLockContext {
        unsafe { LayoutObjectGetDisplayLockContextProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:3414-3418
    pub(crate) fn MapToVisualRectInAncestorSpaceInternalFastPath(
        &self,
        ancestor_or_null_for_viewport: *const LayoutBoxModelObject,
        rect: &mut gfx::RectF,
        flags: VisualRectFlags,
        intersects: &mut bool,
    ) -> bool {
        unsafe {
            LayoutObjectMapToVisualRectInAncestorSpaceInternalFastPathProvider(
                self,
                ancestor_or_null_for_viewport,
                rect,
                flags,
                intersects,
            )
        }
    }
}

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:895-898
    pub fn IsLayoutTextCombineBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_object.h:895-898
    pub fn IsLayoutTextCombine(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsLayoutTextCombine(self) }
    }

    // cpp: layoutng/internal/layout_object.h:915-918
    pub fn IsMediaBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_object.h:915-918
    pub fn IsMedia(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsMedia(self) }
    }

    // cpp: layoutng/internal/layout_object.h:977-980
    pub fn IsTableBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_object.h:977-980
    pub fn IsTable(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsTable(self) }
    }

    // cpp: layoutng/internal/layout_object.h:997-1000
    pub fn IsTextAreaBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_object.h:997-1000
    pub fn IsTextArea(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsTextArea(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1005-1008
    pub fn IsTextFieldBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_object.h:1005-1008
    pub fn IsTextField(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsTextField(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1013-1016
    pub fn IsImageBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_object.h:1013-1016
    pub fn IsImage(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsImage(self) }
    }

    // cpp: layoutng/internal/layout_object.h:1042-1045
    pub fn IsTextControl(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsTextArea() || self.IsTextField()
    }

    // cpp: layoutng/internal/layout_object.h:1382-1386
    pub fn IsBlockInInline(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsAnonymous() && !self.IsInline() && !self.IsFloatingOrOutOfFlowPositioned() && {
            let parent = self.Parent();
            !parent.is_null() && unsafe { &*parent }.IsLayoutInline()
        }
    }

    // cpp: layoutng/internal/layout_object.h:1644-1651
    pub fn IsRenderedLegend(&self) -> bool {
        self.CheckIsNotDestroyed();
        if !self.IsRenderedLegendCandidate() {
            return false;
        }
        self.IsRenderedLegendInternal()
    }

    // cpp: layoutng/internal/layout_object.h:1994-1997
    pub fn IsFragmentLessBoxBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_object.h:1994-1997
    pub fn IsFragmentLessBox(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectIsFragmentLessBox(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2001-2004
    pub fn HasInlineFragmentsBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng/internal/layout_object.h:2001-2004
    pub fn HasInlineFragments(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectHasInlineFragments(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2011-2014
    pub fn FirstInlineFragmentItemIndexBase(&self) -> usize {
        self.CheckIsNotDestroyed();
        0
    }

    // cpp: layoutng/internal/layout_object.h:2011-2014
    pub fn FirstInlineFragmentItemIndex(&self) -> usize {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectFirstInlineFragmentItemIndex(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2015-2015
    pub fn ClearFirstInlineFragmentItemIndexBase(&mut self) {
        self.CheckIsNotDestroyed();
    }

    // cpp: layoutng/internal/layout_object.h:2015-2015
    pub fn ClearFirstInlineFragmentItemIndex(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectClearFirstInlineFragmentItemIndex(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2016-2016
    pub fn SetFirstInlineFragmentItemIndexBase(&mut self, _index: usize) {
        self.CheckIsNotDestroyed();
    }

    // cpp: layoutng/internal/layout_object.h:2016-2016
    pub fn SetFirstInlineFragmentItemIndex(&mut self, index: usize) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectSetFirstInlineFragmentItemIndex(self, index) }
    }

    // cpp: layoutng/internal/layout_object.h:3398-3401
    pub fn AnonymousHasStylePropagationOverride(&mut self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectAnonymousHasStylePropagationOverride(self) }
    }

    // cpp: layoutng/internal/layout_object.h:3403-3405
    pub fn InLayoutNGInlineFormattingContextWillChange(&mut self, value: bool) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutObjectInLayoutNGInlineFormattingContextWillChange(self, value) }
    }
}

impl LayoutObject {
    // cpp: layoutng/internal/layout_object.h:2102-2102
    pub fn Paint(&self, info: &PaintInfo) {
        unsafe { LayoutObjectPaintProvider(self, info) }
    }

    // cpp: layoutng/internal/layout_object.h:2108-2108
    pub fn InvalidateVisualOverflow(&mut self) {
        unsafe { LayoutObjectInvalidateVisualOverflowProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2157-2157
    pub fn AddDraggableRegions(&mut self, regions: &mut Vector<DraggableRegionValue>) {
        unsafe { LayoutObjectAddDraggableRegionsProvider(self, regions) }
    }

    // cpp: layoutng/internal/layout_object.h:2160-2160
    pub fn CanHaveAdditionalCompositingReasons(&self) -> bool {
        unsafe { LayoutObjectCanHaveAdditionalCompositingReasonsProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2161-2161
    pub fn AdditionalCompositingReasons(&self) -> CompositingReasons {
        unsafe { LayoutObjectAdditionalCompositingReasonsProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2171-2173
    pub fn HitTestAllPhases(
        &mut self,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        accumulated_offset: &PhysicalOffset,
    ) -> bool {
        unsafe { LayoutObjectHitTestAllPhasesProvider(self, result, location, accumulated_offset) }
    }

    // cpp: layoutng/internal/layout_object.h:2178-2178
    pub fn NodeForHitTest(&self) -> *mut Node {
        unsafe { LayoutObjectNodeForHitTestProvider(self) }
    }

    // cpp: layoutng/internal/layout_object.h:2179-2179
    pub fn UpdateHitTestResult(&self, result: &mut HitTestResult, offset: &PhysicalOffset) {
        unsafe { LayoutObjectUpdateHitTestResultProvider(self, result, offset) }
    }

    // cpp: layoutng/internal/layout_object.h:2182-2185
    pub fn NodeAtPoint(
        &mut self,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        accumulated_offset: &PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool {
        unsafe {
            LayoutObjectNodeAtPointProvider(self, result, location, accumulated_offset, phase)
        }
    }

    // cpp: layoutng/internal/layout_object.h:2197-2197
    pub fn DirtyLinesFromChangedChild(&mut self, _child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
    }

    // cpp: layoutng/internal/layout_object.h:2534-2536
    pub fn MapToVisualRectInAncestorSpacePhysical(
        &self,
        ancestor: *const LayoutBoxModelObject,
        rect: &mut PhysicalRect,
        flags: VisualRectFlags,
    ) -> bool {
        unsafe { LayoutObjectMapToVisualRectPhysicalProvider(self, ancestor, rect, flags) }
    }

    // cpp: layoutng/internal/layout_object.h:2538-2540
    pub fn MapToVisualRectInAncestorSpaceFloat(
        &self,
        ancestor: *const LayoutBoxModelObject,
        rect: &mut gfx::RectF,
        flags: VisualRectFlags,
    ) -> bool {
        unsafe { LayoutObjectMapToVisualRectFloatProvider(self, ancestor, rect, flags) }
    }

    // cpp: layoutng/internal/layout_object.h:2544-2547
    pub fn MapToVisualRectInAncestorSpaceInternal(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        flags: VisualRectFlags,
    ) -> bool {
        unsafe { DispatchLayoutObjectMapToVisualRectInternal(self, ancestor, state, flags) }
    }

    // cpp: layoutng/internal/layout_object.h:2554-2556
    pub fn GetPropertyContainer(
        &self,
        skip_info: *mut AncestorSkipInfo,
        state: *mut PropertyTreeStateOrAlias,
        flags: VisualRectFlags,
    ) -> *const LayoutObject {
        unsafe { LayoutObjectGetPropertyContainerProvider(self, skip_info, state, flags) }
    }

    // cpp: layoutng/internal/layout_object.h:2623-2624
    pub fn LocalCaretRect(&self, offset: i32, shape: CaretShape) -> PhysicalRect {
        unsafe { LayoutObjectLocalCaretRectProvider(self, offset, shape) }
    }

    // cpp: layoutng/internal/layout_object.h:2666-2668
    pub fn MapLocalToAncestor(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        mode: MapCoordinatesFlags,
    ) {
        unsafe { LayoutObjectMapLocalToAncestorProvider(self, ancestor, state, mode) }
    }

    // cpp: layoutng/internal/layout_object.h:2669-2671
    pub fn MapAncestorToLocal(
        &self,
        ancestor: *const LayoutBoxModelObject,
        state: &mut TransformState,
        mode: MapCoordinatesFlags,
    ) {
        unsafe { LayoutObjectMapAncestorToLocalProvider(self, ancestor, state, mode) }
    }

    // cpp: layoutng/internal/layout_object.h:2712-2717
    pub fn AddOutlineRectsBase(
        &self,
        _collector: &mut dyn OutlineRectCollector,
        _info: *mut LayoutOutlineInfo,
        _additional_offset: &PhysicalOffset,
        _outline_type: OutlineType,
    ) {
        self.CheckIsNotDestroyed();
    }

    // cpp: layoutng/internal/layout_object.h:2712-2717
    pub fn AddOutlineRects(
        &self,
        collector: &mut dyn OutlineRectCollector,
        info: *mut LayoutOutlineInfo,
        additional_offset: &PhysicalOffset,
        outline_type: OutlineType,
    ) {
        self.CheckIsNotDestroyed();
        unsafe {
            DispatchLayoutObjectAddOutlineRects(
                self,
                collector,
                info,
                additional_offset,
                outline_type,
            )
        }
    }

    // cpp: layoutng/internal/layout_object.h:2834-2834
    pub fn InvalidatePaint(&self, context: &PaintInvalidatorContext) {
        unsafe { LayoutObjectInvalidatePaintProvider(self, context) }
    }

    // cpp: layoutng/internal/layout_object.h:2843-2843
    pub fn InvalidateDisplayItemClients(&self, reason: PaintInvalidationReason) {
        unsafe { LayoutObjectInvalidateDisplayItemClientsProvider(self, reason) }
    }

    // cpp: layoutng/internal/layout_object.h:2861-2861
    pub fn DebugRect(&self) -> PhysicalRect {
        unsafe { LayoutObjectDebugRectProvider(self) }
    }
}

// cpp: layoutng/internal/layout_object.h:3949-3952
pub fn AssociatedLayoutObjectOf(
    node: &Node,
    offset_in_node: i32,
    side: LayoutObjectSide,
) -> *const LayoutObject {
    unsafe { AssociatedLayoutObjectOfProvider(node, offset_in_node, side) }
}

// cpp: layoutng/internal/layout_object.h:3949-3952
pub fn AssociatedLayoutObjectOfDefault(node: &Node, offset_in_node: i32) -> *const LayoutObject {
    AssociatedLayoutObjectOf(
        node,
        offset_in_node,
        LayoutObjectSide::kRemainingTextIfOnBoundary,
    )
}

// cpp: layoutng/internal/layout_object.h:3961-3961
#[cfg(debug_assertions)]
pub fn ShowTree(object: *const LayoutObject) {
    unsafe { ShowTreeProvider(object) }
}

// cpp: layoutng/internal/layout_object.h:3962-3962
#[cfg(debug_assertions)]
pub fn ShowLayoutTree(object: *const LayoutObject) {
    unsafe { ShowLayoutTreeProvider(object) }
}

// cpp: layoutng/internal/layout_object.h:3965-3966
#[cfg(debug_assertions)]
pub fn ShowLayoutTreeWithSecond(object1: *const LayoutObject, object2: *const LayoutObject) {
    unsafe { ShowLayoutTreeWithSecondProvider(object1, object2) }
}

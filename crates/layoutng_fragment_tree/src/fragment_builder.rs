use foundation::{
    gfx, kIndefiniteSize, DynamicTo, EPosition, EScrollInitialTarget, GCedHeapVector, HeapHashMap,
    HeapVector, LayoutUnit, MakeGarbageCollected, MarginStrut, Member, PhysicalOffset,
    RuntimeEnabledFeatures, String, StringBuilder, TextDirection, To, TransformState,
    WritingDirectionMode, WritingMode,
};
use layoutng::internal::anchor_map::{AnchorMap, AnchorMapSetOptions};
use layoutng::internal::anchor_scope::ToAnchorScopedName;
use layoutng::internal::block_node::BlockNode;
use layoutng::internal::break_appeal::{kBreakAppealPerfect, BreakAppeal};
use layoutng::internal::constraint_space::ConstraintSpace;
use layoutng::internal::early_break::EarlyBreak;
use layoutng::internal::exclusions::exclusion_space::ExclusionSpace;
use layoutng::internal::fragmentation_utils::UpdateMinimalSpaceShortage;
use layoutng::internal::layout_box::LayoutBox;
use layoutng::internal::layout_box_model_object::LayoutBoxModelObject;
use layoutng::internal::layout_inline::LayoutInline;
use layoutng::internal::layout_input_node::LayoutInputNode;
use layoutng::internal::layout_node_metadata::Element;
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::layout_object_tree::IndexCache;
use layoutng::internal::oof_positioned_node::{
    LogicalOofNodeForFragmentation, LogicalOofPositionedNode, MulticolWithPendingOofs,
    OofContainingBlock, OofInlineContainer, PhysicalOofPositionedNodeToLogical,
    RelativeInsetToLogical,
};
use layoutng::internal::snap_area::SnapArea;
use layoutng::internal::split_axis_item::SplitAxisItem;
use layoutng::internal::style_variant::StyleVariant;
use layoutng::internal::transform_utils::UpdateTransformState;
use layoutng::internal::trigger_scoped_name::{
    ToTriggerScopedName, TriggerScopedName, TriggerScopedNameMap,
};
use layoutng::internal::unpositioned_list_marker::UnpositionedListMarker;
use layoutng_geometry::geometry::axis::{
    kPhysicalAxesBoth, kPhysicalAxesHorizontal, kPhysicalAxesNone, kPhysicalAxesVertical,
    PhysicalAxes,
};
use layoutng_geometry::geometry::logical_offset::LogicalOffset;
use layoutng_geometry::geometry::logical_rect::LogicalRect;
use layoutng_geometry::geometry::logical_size::{LogicalSize, ToLogicalSize, ToPhysicalSize};
use layoutng_geometry::geometry::static_position::LogicalStaticPosition;
use layoutng_geometry::geometry::writing_mode_converter::WritingModeConverter;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::scroll_snap_data::cc;

use crate::break_token::{BreakToken, BreakTokenVector};
use crate::fragment_items_builder::FragmentItemsBuilder;
use crate::inline_break_token::InlineBreakToken;
use crate::layout_result::{EStatus, LayoutResult};
use crate::logical_fragment_link::{LogicalFragmentLink, LogicalFragmentLinkVector};
use crate::physical_box_fragment::PhysicalBoxFragment;
use crate::physical_fragment::{BoxType, FragmentType, PhysicalFragment};
use crate::physical_line_box_fragment::PhysicalLineBoxFragment;

unsafe extern "Rust" {
    fn FragmentBuilderAddOutOfFlowInlineChildCandidate(
        builder: &mut FragmentBuilder,
        child: BlockNode,
        child_offset: &LogicalOffset,
        inline_container_writing_direction: WritingDirectionMode,
        line_box_block_size: LayoutUnit,
    );
}

// cpp: layoutng_fragment_tree/fragment_builder.cc:53-58
#[allow(non_snake_case)]
fn IsInlineContainerForNode(node: &BlockNode, inline_container: *const LayoutInline) -> bool {
    !inline_container.is_null()
        && unsafe { &*inline_container }
            .CanContainOutOfFlowPositionedElement(node.Style().GetPosition())
}

// cpp: layoutng_fragment_tree/fragment_builder.cc:60-65
#[allow(non_snake_case)]
fn PartitionAxes(
    container_axes: PhysicalAxes,
    target_axes: PhysicalAxes,
) -> (PhysicalAxes, PhysicalAxes) {
    let consumed = container_axes & target_axes;
    (consumed, target_axes ^ consumed)
}

// cpp: layoutng_fragment_tree/fragment_builder.cc:67-78
#[allow(non_snake_case)]
fn GetScrollSnapAlignAxes(
    align: &cc::ScrollSnapAlign,
    writing_direction: WritingDirectionMode,
) -> PhysicalAxes {
    let horizontal = writing_direction.IsHorizontal();
    let mut axes = kPhysicalAxesNone;
    if align.alignment_block != cc::SnapAlignment::kNone {
        axes |= if horizontal {
            kPhysicalAxesVertical
        } else {
            kPhysicalAxesHorizontal
        };
    }
    if align.alignment_inline != cc::SnapAlignment::kNone {
        axes |= if horizontal {
            kPhysicalAxesHorizontal
        } else {
            kPhysicalAxesVertical
        };
    }
    axes
}

// cpp: layoutng_fragment_tree/fragment_builder.h:37-39
// cpp: layoutng_fragment_tree/fragment_builder.h:51-54
// cpp: layoutng_fragment_tree/fragment_builder.h:549-648
pub struct FragmentBuilder {
    pub(crate) node_: LayoutInputNode,
    pub(crate) space_: ConstraintSpace,
    pub(crate) style_: *const ComputedStyle,
    pub(crate) writing_direction_: WritingDirectionMode,
    pub(crate) style_variant_: StyleVariant,
    pub(crate) box_type_: BoxType,
    pub(crate) size_: LogicalSize,
    pub(crate) layout_object_: *mut LayoutObject,
    pub(crate) previous_break_token_: *const BreakToken,
    pub(crate) break_token_: *const BreakToken,
    pub(crate) sticky_descendants_: *mut GCedHeapVector<SplitAxisItem<LayoutBoxModelObject>>,
    pub(crate) snap_areas_: *mut GCedHeapVector<SnapArea>,
    pub(crate) named_triggers_: *mut TriggerScopedNameMap,
    pub(crate) scroll_start_target_: *const LayoutObject,
    pub(crate) anchor_map_: *mut AnchorMap,
    pub(crate) bfc_line_offset_: LayoutUnit,
    pub(crate) bfc_block_offset_: Option<LayoutUnit>,
    pub(crate) end_margin_strut_: MarginStrut,
    pub(crate) exclusion_space_: ExclusionSpace,
    pub(crate) lines_until_clamp_: Option<i32>,
    pub(crate) line_clamp_after_layout_object_: *const LayoutObject,
    pub(crate) children_: LogicalFragmentLinkVector,
    pub(crate) children_with_size_dependent_propagation_: HeapVector<LogicalFragmentLink>,
    pub(crate) items_builder_: *mut FragmentItemsBuilder,
    pub(crate) child_break_tokens_: BreakTokenVector,
    pub(crate) last_inline_break_token_: *const InlineBreakToken,
    pub(crate) oof_positioned_candidates_: HeapVector<LogicalOofPositionedNode>,
    pub(crate) oof_positioned_fragmentainer_descendants_:
        HeapVector<LogicalOofNodeForFragmentation>,
    pub(crate) oof_positioned_descendants_: HeapVector<LogicalOofPositionedNode>,
    pub(crate) multicols_with_pending_oofs_:
        HeapHashMap<Member<LayoutBox>, Member<MulticolWithPendingOofs<LogicalOffset>>>,
    pub(crate) unpositioned_list_marker_: UnpositionedListMarker,
    pub(crate) early_break_: *const EarlyBreak,
    pub(crate) break_appeal_: BreakAppeal,
    pub(crate) annotation_overflow_: LayoutUnit,
    pub(crate) block_end_annotation_space_: LayoutUnit,
    pub(crate) minimal_space_shortage_: LayoutUnit,
    pub(crate) tallest_unbreakable_block_size_: LayoutUnit,
    pub(crate) line_count_: i32,
    pub(crate) adjoining_object_types_: i32,
    pub(crate) has_adjoining_object_descendants_: bool,
    pub(crate) is_self_collapsing_: bool,
    pub(crate) is_pushed_by_floats_: bool,
    pub(crate) subtree_modified_margin_strut_: bool,
    pub(crate) is_new_fc_: bool,
    pub(crate) is_block_in_inline_: bool,
    pub(crate) is_line_for_parallel_flow_: bool,
    pub(crate) has_floating_descendants_for_paint_: bool,
    pub(crate) has_descendant_that_depends_on_percentage_block_size_: bool,
    pub(crate) has_orthogonal_fallback_size_descendant_: bool,
    pub(crate) may_have_descendant_above_block_start_: bool,
    pub(crate) is_fragmentation_context_root_: bool,
    pub(crate) is_hidden_for_paint_: bool,
    pub(crate) is_opaque_: bool,
    pub(crate) has_collapsed_borders_: bool,
    pub(crate) requires_content_before_breaking_: bool,
    pub(crate) has_out_of_flow_fragment_child_: bool,
    pub(crate) has_out_of_flow_in_fragmentainer_subtree_: bool,
    pub(crate) is_block_end_trimmable_line_: bool,
    pub(crate) would_be_last_line_if_not_for_ellipsis_: bool,
    pub(crate) has_final_size_: bool,
    pub(crate) oof_candidates_may_have_anchors_: bool,
    pub(crate) oof_fragmentainer_descendants_may_have_anchors_: bool,
    pub(crate) has_running_anchor_transform_animation_: bool,
    pub(crate) is_may_have_descendant_above_block_start_explicitly_set_: bool,
    pub(crate) is_finalized_: bool,
}

#[allow(non_snake_case)]
impl FragmentBuilder {
    // cpp: layoutng_fragment_tree/fragment_builder.h:496-500
    // cpp: layoutng_fragment_tree/fragment_builder.cc:34-48
    pub fn new(
        node: LayoutInputNode,
        style: *const ComputedStyle,
        space: &ConstraintSpace,
        writing_direction: WritingDirectionMode,
        previous_break_token: *const BreakToken,
    ) -> Self {
        debug_assert!(!style.is_null());
        let layout_object = node.GetLayoutBox() as *mut LayoutObject;
        Self {
            node_: node,
            space_: space.clone(),
            style_: style,
            writing_direction_: writing_direction,
            style_variant_: StyleVariant::kStandard,
            box_type_: BoxType::kNormalBox,
            size_: LogicalSize::default(),
            layout_object_: layout_object,
            previous_break_token_: previous_break_token,
            break_token_: std::ptr::null(),
            sticky_descendants_: std::ptr::null_mut(),
            snap_areas_: std::ptr::null_mut(),
            named_triggers_: std::ptr::null_mut(),
            scroll_start_target_: std::ptr::null(),
            anchor_map_: std::ptr::null_mut(),
            bfc_line_offset_: LayoutUnit::default(),
            bfc_block_offset_: None,
            end_margin_strut_: MarginStrut::default(),
            exclusion_space_: ExclusionSpace::default(),
            lines_until_clamp_: None,
            line_clamp_after_layout_object_: std::ptr::null(),
            children_: LogicalFragmentLinkVector::default(),
            children_with_size_dependent_propagation_: HeapVector::default(),
            items_builder_: std::ptr::null_mut(),
            child_break_tokens_: BreakTokenVector::default(),
            last_inline_break_token_: std::ptr::null(),
            oof_positioned_candidates_: HeapVector::default(),
            oof_positioned_fragmentainer_descendants_: HeapVector::default(),
            oof_positioned_descendants_: HeapVector::default(),
            multicols_with_pending_oofs_: HeapHashMap::default(),
            unpositioned_list_marker_: UnpositionedListMarker::default(),
            early_break_: std::ptr::null(),
            break_appeal_: kBreakAppealPerfect,
            annotation_overflow_: LayoutUnit::default(),
            block_end_annotation_space_: LayoutUnit::default(),
            minimal_space_shortage_: kIndefiniteSize,
            tallest_unbreakable_block_size_: LayoutUnit::Min(),
            line_count_: 0,
            adjoining_object_types_: 0,
            has_adjoining_object_descendants_: false,
            is_self_collapsing_: false,
            is_pushed_by_floats_: false,
            subtree_modified_margin_strut_: false,
            is_new_fc_: false,
            is_block_in_inline_: false,
            is_line_for_parallel_flow_: false,
            has_floating_descendants_for_paint_: false,
            has_descendant_that_depends_on_percentage_block_size_: false,
            has_orthogonal_fallback_size_descendant_: false,
            may_have_descendant_above_block_start_: false,
            is_fragmentation_context_root_: false,
            is_hidden_for_paint_: space.IsHiddenForPaint(),
            is_opaque_: false,
            has_collapsed_borders_: false,
            requires_content_before_breaking_: false,
            has_out_of_flow_fragment_child_: false,
            has_out_of_flow_in_fragmentainer_subtree_: false,
            is_block_end_trimmable_line_: false,
            would_be_last_line_if_not_for_ellipsis_: false,
            has_final_size_: false,
            oof_candidates_may_have_anchors_: false,
            oof_fragmentainer_descendants_may_have_anchors_: false,
            has_running_anchor_transform_animation_: false,
            is_may_have_descendant_above_block_start_explicitly_set_: false,
            is_finalized_: false,
        }
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:56-72
    pub fn Style(&self) -> &ComputedStyle {
        debug_assert!(!self.style_.is_null());
        unsafe { &*self.style_ }
    }

    pub fn SetStyleVariant(&mut self, variant: StyleVariant) {
        self.style_variant_ = variant;
    }

    pub fn GetConstraintSpace(&self) -> &ConstraintSpace {
        &self.space_
    }

    pub fn GetWritingDirection(&self) -> WritingDirectionMode {
        self.writing_direction_
    }

    pub fn GetWritingMode(&self) -> WritingMode {
        self.writing_direction_.GetWritingMode()
    }

    pub fn Direction(&self) -> TextDirection {
        self.writing_direction_.Direction()
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:74
    // cpp: layoutng_fragment_tree/fragment_builder.cc:82-117
    pub fn AnchorOptionsForChild(&self, fragment: &PhysicalFragment) -> AnchorMapSetOptions {
        if !fragment.IsOutOfFlowPositioned() {
            return AnchorMapSetOptions::kInFlow;
        }
        let maybe_out_of_order_if_oof = self.IsBlockFragmentationContextRoot() || self.HasItems();
        if !maybe_out_of_order_if_oof {
            return AnchorMapSetOptions::kOutOfFlow;
        }
        let container = self.node_.GetLayoutBox();
        if container.is_null() {
            return AnchorMapSetOptions::kOutOfFlow;
        }
        let layout_object = fragment.GetLayoutObject();
        debug_assert!(!layout_object.is_null());
        let containing_block = unsafe { &*layout_object }.Container();
        debug_assert!(!containing_block.is_null());
        if containing_block == container as *mut LayoutObject {
            AnchorMapSetOptions::kOutOfFlow
        } else {
            AnchorMapSetOptions::kInFlow
        }
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:207-208
    // cpp: layoutng_fragment_tree/fragment_builder.cc:272-292
    pub fn PropagateChildAnchors(&mut self, child: &PhysicalFragment, offset: LogicalOffset) {
        if !child.HasAnchorsToPropagate() {
            return;
        }
        if !self.has_final_size_ {
            self.children_with_size_dependent_propagation_
                .push(LogicalFragmentLink::new(child, offset));
            return;
        }
        let container_object = self.GetLayoutObject();
        assert!(!container_object.is_null());
        let options = self.AnchorOptionsForChild(child);
        Self::PropagateChildAnchorsIntoMap(
            child,
            offset,
            unsafe { &*container_object },
            self.GetWritingDirection(),
            *self.Size(),
            options,
            &mut self.anchor_map_,
        );
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:209-215
    // cpp: layoutng_fragment_tree/fragment_builder.cc:294-364
    pub fn PropagateChildAnchorsIntoMap(
        child: &PhysicalFragment,
        offset: LogicalOffset,
        container_object: &LayoutObject,
        writing_direction: WritingDirectionMode,
        container_logical_size: LogicalSize,
        options: AnchorMapSetOptions,
        out_anchor_map: &mut *mut AnchorMap,
    ) {
        let mut context: *mut Element = std::ptr::null_mut();
        let node = child.GetNode();
        if !node.is_null() {
            let element = DynamicTo::<Element>(node);
            if !element.is_null() {
                if let Some(display_lock) = unsafe { &*element }.InputContentLock() {
                    if display_lock {
                        return;
                    }
                    context = element;
                }
            }
        }
        let physical_container_size =
            ToPhysicalSize(container_logical_size, writing_direction.GetWritingMode());
        if child.IsAnchor() {
            debug_assert!(!child.GetLayoutObject().is_null());
            let logical_rect = LogicalRect::new(
                offset,
                ToLogicalSize(child.Size(), writing_direction.GetWritingMode()),
            );
            let converter = WritingModeConverter::new(writing_direction, physical_container_size);
            let rect = converter.ToPhysicalRect(logical_rect);
            let mut transform_state = TransformState::new(
                TransformState::kApplyTransformDirection,
                gfx::QuadF::from(gfx::RectF::from(gfx::SizeF::from(rect.size))),
            );
            UpdateTransformState(
                child,
                rect.offset,
                container_object,
                physical_container_size,
                &mut transform_state,
            );
            if child.IsExplicitAnchor() {
                let anchor_names = child.Style().AnchorName().Get();
                for name in unsafe { &*anchor_names }.GetNames() {
                    let scoped_name = ToAnchorScopedName(unsafe { &*name.Get() }, unsafe {
                        &*child.GetLayoutObject()
                    });
                    Self::EnsureAnchorMap(out_anchor_map).Set(
                        scoped_name,
                        unsafe { &*child.GetLayoutObject() },
                        &transform_state,
                        options,
                        context,
                    );
                }
            }
            if child.IsImplicitAnchor() {
                Self::EnsureAnchorMap(out_anchor_map).Set(
                    To::<Element>(child.GetNode()),
                    unsafe { &*child.GetLayoutObject() },
                    &transform_state,
                    options,
                    context,
                );
            }
        }
        if !child.GetAnchorMap().is_null() {
            let converter = WritingModeConverter::new(writing_direction, physical_container_size);
            let additional_offset = converter.ToPhysicalOffset(offset, child.Size());
            Self::EnsureAnchorMap(out_anchor_map).SetFromChild(
                child,
                additional_offset,
                container_object,
                physical_container_size,
                options,
                context,
            );
        }
    }

    fn EnsureAnchorMap(out_anchor_map: &mut *mut AnchorMap) -> &mut AnchorMap {
        if (*out_anchor_map).is_null() {
            *out_anchor_map = MakeGarbageCollected(AnchorMap::default());
        }
        unsafe { &mut **out_anchor_map }
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:217
    pub fn GetAnchorMap(&self) -> *const AnchorMap {
        self.anchor_map_
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:506-510
    // cpp: layoutng_fragment_tree/fragment_builder.cc:366-374
    pub fn PropagateFromLayoutResultAndFragment(
        &mut self,
        child_result: &LayoutResult,
        child_offset: LogicalOffset,
        relative_offset: LogicalOffset,
        inline_container: *const OofInlineContainer<LogicalOffset>,
    ) {
        self.PropagateFromLayoutResult(child_result);
        self.PropagateFromFragment(
            child_result.GetPhysicalFragment(),
            child_offset,
            relative_offset,
            inline_container,
        );
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:512
    // cpp: layoutng_fragment_tree/fragment_builder.cc:376-381
    pub(crate) fn PropagateFromLayoutResult(&mut self, child_result: &LayoutResult) {
        self.has_orthogonal_fallback_size_descendant_ |= child_result
            .HasOrthogonalFallbackInlineSize()
            || child_result.HasOrthogonalFallbackSizeDescendant();
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:521-525
    // cpp: layoutng_fragment_tree/fragment_builder.cc:466-606
    pub(crate) fn PropagateFromFragment(
        &mut self,
        child: &PhysicalFragment,
        child_offset: LogicalOffset,
        relative_offset: LogicalOffset,
        inline_container: *const OofInlineContainer<LogicalOffset>,
    ) {
        if self.GetBoxType() == BoxType::kPageBorderBox {
            debug_assert_eq!(child.GetBoxType(), BoxType::kPageArea);
            return;
        }
        if child.HasAnchorsToPropagate() {
            self.PropagateChildAnchors(child, child_offset + relative_offset);
            self.has_running_anchor_transform_animation_ |=
                child.HasRunningAnchorTransformAnimation();
        }
        self.PropagateStickyDescendants(child);
        self.PropagateSnapAreas(child);
        self.PropagateScrollInitialTarget(child);
        self.PropagateNamedTriggers(child);

        if child.NeedsOOFPositionedInfoPropagation()
            && (RuntimeEnabledFeatures::FragmentedOofInCbEnabled()
                || !self.IsFragmentainerBoxType()
                || !child.IsOutOfFlowPositioned())
        {
            let adjustment = self.BlockOffsetAdjustmentForFragmentainer(LayoutUnit::default());
            self.PropagateOOFPositionedInfo(
                child,
                child_offset,
                relative_offset,
                LogicalOffset::default(),
                inline_container,
                adjustment,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                LogicalOffset::default(),
            );
        }

        if !self.has_descendant_that_depends_on_percentage_block_size_ {
            if child.DependsOnPercentageBlockSize() && !child.IsOutOfFlowPositioned() {
                self.has_descendant_that_depends_on_percentage_block_size_ = true;
            }
            let child_style = child.Style();
            if child.IsCSSBox() && child_style.GetPosition() == EPosition::kRelative {
                if self.Style().IsHorizontalWritingMode() {
                    if child_style.Top().HasPercent() || child_style.Bottom().HasPercent() {
                        self.has_descendant_that_depends_on_percentage_block_size_ = true;
                    }
                } else if child_style.Left().HasPercent() || child_style.Right().HasPercent() {
                    self.has_descendant_that_depends_on_percentage_block_size_ = true;
                }
            }
        }
        if !self.has_floating_descendants_for_paint_
            && (child.IsFloating()
                || (child.HasFloatingDescendantsForPaint() && !child.IsPaintedAtomically()))
        {
            self.has_floating_descendants_for_paint_ = true;
        }
        if !self.has_adjoining_object_descendants_
            && !child.IsFormattingContextRoot()
            && child.HasAdjoiningObjectDescendants()
        {
            self.has_adjoining_object_descendants_ = true;
        }

        if self.GetConstraintSpace().HasBlockFragmentation()
            && !child.IsFragmentainerBox()
            && self.break_token_.is_null()
        {
            let child_break_token = child.GetBreakToken();
            match child.Type() {
                FragmentType::kFragmentBox => {
                    if !child_break_token.is_null() {
                        self.child_break_tokens_
                            .push(Member::from_ptr(child_break_token as *mut BreakToken));
                    }
                }
                FragmentType::kFragmentLineBox => {
                    let line = unsafe { &*(child as *const _ as *const PhysicalLineBoxFragment) };
                    if line.IsLineForParallelFlow() {
                        return;
                    }
                    if !child_break_token.is_null() {
                        assert!(unsafe { &*child_break_token }.IsInlineType());
                    }
                    self.last_inline_break_token_ = child_break_token as *const InlineBreakToken;
                    if !line.IsEmptyLineBox() {
                        self.line_count_ += 1;
                    }
                }
            }
        }
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:77
    // cpp: layoutng_fragment_tree/fragment_builder.cc:119-121
    pub fn IsRoot(&self) -> bool {
        !self.node_.IsNull() && self.node_.IsView() && !self.GetConstraintSpace().IsAnonymous()
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:81
    // cpp: layoutng_fragment_tree/fragment_builder.cc:123-125
    pub fn IsPaginatedRoot(&self) -> bool {
        self.IsRoot() && self.node_.IsPaginatedRoot()
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:85-119
    pub fn PreviousBreakToken(&self) -> *const BreakToken {
        self.previous_break_token_
    }

    pub fn SetIsNewFormattingContext(&mut self, is_new_fc: bool) {
        self.is_new_fc_ = is_new_fc;
    }

    pub fn SetBoxType(&mut self, box_type: BoxType) {
        self.box_type_ = box_type;
    }

    pub fn IsFragmentainerBoxType(&self) -> bool {
        matches!(self.GetBoxType(), BoxType::kColumnBox | BoxType::kPageArea)
    }

    pub fn InlineSize(&self) -> LayoutUnit {
        self.size_.inline_size
    }

    pub fn BlockSize(&self) -> LayoutUnit {
        debug_assert!(self.size_.block_size != kIndefiniteSize);
        self.size_.block_size
    }

    pub fn Size(&self) -> &LogicalSize {
        debug_assert!(self.size_.block_size != kIndefiniteSize);
        &self.size_
    }

    pub fn SetBlockSize(&mut self, block_size: LayoutUnit) {
        self.size_.block_size = block_size;
    }

    pub fn HasBlockSize(&self) -> bool {
        self.size_.block_size != kIndefiniteSize
    }

    pub fn HasFinalSize(&self) -> bool {
        self.has_final_size_
    }

    pub fn SetIsHiddenForPaint(&mut self, hidden: bool) {
        self.is_hidden_for_paint_ = hidden;
    }

    pub fn SetIsOpaque(&mut self) {
        self.is_opaque_ = true;
    }

    pub fn SetHasCollapsedBorders(&mut self, collapsed: bool) {
        self.has_collapsed_borders_ = collapsed;
    }

    pub fn GetLayoutObject(&self) -> *const LayoutObject {
        self.layout_object_
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:90
    // cpp: layoutng_fragment_tree/fragment_builder.cc:127-166
    pub fn GetBoxType(&self) -> BoxType {
        if self.box_type_ != BoxType::kNormalBox {
            return self.box_type_;
        }
        let object = unsafe { &*self.layout_object_ };
        if object.IsFloating() {
            return BoxType::kFloating;
        }
        if object.IsOutOfFlowPositioned() {
            return BoxType::kOutOfFlowPositioned;
        }
        if object.IsRenderedLegend() {
            return BoxType::kRenderedLegend;
        }
        if object.StyleRef().IsPageMarginBox() {
            return BoxType::kPageMargin;
        }
        if object.IsAtomicInline() {
            return BoxType::kAtomicInline;
        }
        if object.IsInline() {
            return BoxType::kInlineBox;
        }
        debug_assert!(
            !self.node_.IsNull(),
            "Must call SetBoxType if there is no node"
        );
        if self.is_new_fc_ != self.node_.CreatesNewFormattingContext() {
            eprintln!(
                "formatting-context mismatch: {} builder={} node={}",
                object.GetName(),
                self.is_new_fc_,
                self.node_.CreatesNewFormattingContext()
            );
        }
        debug_assert_eq!(self.is_new_fc_, self.node_.CreatesNewFormattingContext());
        if self.is_new_fc_ {
            BoxType::kBlockFlowRoot
        } else {
            BoxType::kNormalBox
        }
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:121-124
    pub fn BfcLineOffset(&self) -> LayoutUnit {
        self.bfc_line_offset_
    }

    pub fn SetBfcLineOffset(&mut self, offset: LayoutUnit) {
        self.bfc_line_offset_ = offset;
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:128-134
    pub fn BfcBlockOffset(&self) -> &Option<LayoutUnit> {
        &self.bfc_block_offset_
    }

    pub fn SetBfcBlockOffset(&mut self, offset: LayoutUnit) {
        self.bfc_block_offset_ = Some(offset);
    }

    pub fn ResetBfcBlockOffset(&mut self) {
        self.bfc_block_offset_ = None;
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:136-138
    pub fn SetEndMarginStrut(&mut self, margin_strut: &MarginStrut) {
        self.end_margin_strut_ = margin_strut.clone();
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:140-145
    pub fn SetMayHaveDescendantAboveBlockStart(&mut self, value: bool) {
        #[cfg(debug_assertions)]
        {
            self.is_may_have_descendant_above_block_start_explicitly_set_ = true;
        }
        self.may_have_descendant_above_block_start_ = value;
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:147-150
    pub fn GetExclusionSpace(&mut self) -> &mut ExclusionSpace {
        &mut self.exclusion_space_
    }

    pub fn SetExclusionSpace(&mut self, exclusion_space: &ExclusionSpace) {
        self.exclusion_space_ = exclusion_space.clone();
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:152-154
    pub fn SetLinesUntilClamp(&mut self, value: Option<i32>) {
        self.lines_until_clamp_ = value;
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:156-161
    pub fn WouldBeLastLineIfNotForEllipsis(&self) -> bool {
        self.would_be_last_line_if_not_for_ellipsis_
    }

    pub fn SetWouldBeLastLineIfNotForEllipsis(&mut self) {
        self.would_be_last_line_if_not_for_ellipsis_ = true;
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:163-168
    pub fn SetLineClampAfterLayoutObject(&mut self, layout_object: *const LayoutObject) {
        self.line_clamp_after_layout_object_ = layout_object;
    }

    pub fn IsBlockEndTrimmableLine(&self) -> bool {
        self.is_block_end_trimmable_line_
    }

    pub fn SetIsBlockEndTrimmableLine(&mut self) {
        self.is_block_end_trimmable_line_ = true;
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:170-179
    pub fn GetUnpositionedListMarker(&self) -> &UnpositionedListMarker {
        &self.unpositioned_list_marker_
    }

    pub fn SetUnpositionedListMarker(&mut self, marker: &UnpositionedListMarker) {
        debug_assert!(!self.unpositioned_list_marker_.is_present() || !marker.is_present());
        self.unpositioned_list_marker_ = marker.clone();
    }

    pub fn ClearUnpositionedListMarker(&mut self) {
        self.unpositioned_list_marker_ = UnpositionedListMarker::default();
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:185-190
    pub fn SetChildOffset(&mut self, index: usize, offset: LogicalOffset) {
        debug_assert!(index < self.children_.len());
        self.children_[index].offset = offset;
    }

    pub fn Children(&self) -> &LogicalFragmentLinkVector {
        &self.children_
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:194-199
    pub fn HasItems(&self) -> bool {
        !self.items_builder_.is_null()
    }

    pub fn ItemsBuilder(&self) -> *mut FragmentItemsBuilder {
        self.items_builder_
    }

    pub fn SetItemsBuilder(&mut self, builder: *mut FragmentItemsBuilder) {
        self.items_builder_ = builder;
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:181-183
    // cpp: layoutng_fragment_tree/fragment_builder.cc:168-173
    pub fn ReplaceChild(&mut self, index: usize, child: &PhysicalFragment, offset: LogicalOffset) {
        debug_assert!(index < self.children_.len());
        self.children_[index] = LogicalFragmentLink::new(child, offset);
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:384-394
    pub fn SetIsSelfCollapsing(&mut self) {
        self.is_self_collapsing_ = true;
    }

    pub fn SetIsPushedByFloats(&mut self) {
        self.is_pushed_by_floats_ = true;
    }

    pub fn IsPushedByFloats(&self) -> bool {
        self.is_pushed_by_floats_
    }

    pub fn SetSubtreeModifiedMarginStrut(&mut self) {
        debug_assert!(self.BfcBlockOffset().is_none());
        self.subtree_modified_margin_strut_ = true;
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:396-412
    pub fn ResetAdjoiningObjectTypes(&mut self) {
        self.adjoining_object_types_ = 0;
        self.has_adjoining_object_descendants_ = false;
    }

    pub fn AddAdjoiningObjectTypes(&mut self, types: i32) {
        self.adjoining_object_types_ |= types;
        self.has_adjoining_object_descendants_ |= types != 0;
    }

    pub fn SetAdjoiningObjectTypes(&mut self, types: i32) {
        self.adjoining_object_types_ = types;
    }

    pub fn SetHasAdjoiningObjectDescendants(&mut self, value: bool) {
        self.has_adjoining_object_descendants_ = value;
    }

    pub fn GetAdjoiningObjectTypes(&self) -> i32 {
        self.adjoining_object_types_
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:414-425
    pub fn SetIsBlockInInline(&mut self) {
        self.is_block_in_inline_ = true;
    }

    pub fn SetIsLineForParallelFlow(&mut self) {
        self.is_line_for_parallel_flow_ = true;
    }

    pub fn SetIsBlockFragmentationContextRoot(&mut self) {
        self.is_fragmentation_context_root_ = true;
    }

    pub fn IsBlockFragmentationContextRoot(&self) -> bool {
        self.is_fragmentation_context_root_
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:431-457
    pub fn SetRequiresContentBeforeBreaking(&mut self, value: bool) {
        self.requires_content_before_breaking_ = value;
    }

    pub fn RequiresContentBeforeBreaking(&self) -> bool {
        self.requires_content_before_breaking_
    }

    pub fn ClampBreakAppeal(&mut self, appeal: BreakAppeal) {
        self.break_appeal_ = self.break_appeal_.min(appeal);
    }

    pub fn SetHasDescendantThatDependsOnPercentageBlockSize(&mut self, value: bool) {
        self.has_descendant_that_depends_on_percentage_block_size_ = value;
    }

    pub fn MarkHasDescendantThatDependsOnPercentageBlockSize(&mut self) {
        self.SetHasDescendantThatDependsOnPercentageBlockSize(true);
    }

    pub fn SetAnnotationOverflow(&mut self, overflow: LayoutUnit) {
        self.annotation_overflow_ = overflow;
    }

    pub fn AnnotationOverflow(&self) -> LayoutUnit {
        self.annotation_overflow_
    }

    pub fn SetBlockEndAnnotationSpace(&mut self, space: LayoutUnit) {
        self.block_end_annotation_space_ = space;
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:464-483
    pub fn MinimalSpaceShortage(&self) -> Option<LayoutUnit> {
        if self.minimal_space_shortage_ == kIndefiniteSize {
            None
        } else {
            Some(self.minimal_space_shortage_)
        }
    }

    pub fn PropagateTallestUnbreakableBlockSize(&mut self, size: LayoutUnit) {
        debug_assert!(self.GetConstraintSpace().IsInitialColumnBalancingPass());
        self.tallest_unbreakable_block_size_ = self.tallest_unbreakable_block_size_.max(size);
    }

    pub fn SetHasRunningAnchorTransformAnimation(&mut self) {
        self.has_running_anchor_transform_animation_ = true;
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:295-309
    pub fn HasOutOfFlowPositionedCandidates(&self) -> bool {
        !self.oof_positioned_candidates_.is_empty()
    }

    // cpp: layoutng_out_of_flow/out_of_flow_fragment_builder.cc:90-139
    pub fn SwapOutOfFlowPositionedCandidates(
        &mut self,
        candidates: &mut HeapVector<LogicalOofPositionedNode>,
    ) {
        debug_assert!(candidates.is_empty());
        if self.oof_candidates_may_have_anchors_ {
            let mut index_cache =
                (self.oof_positioned_candidates_.len() > 32).then(IndexCache::default);
            self.oof_positioned_candidates_.sort_by(|a, b| {
                let a_inline = a.InlineContainer();
                let b_inline = b.InlineContainer();
                if a_inline != b_inline {
                    let a_depth = if a_inline.is_null() {
                        0
                    } else {
                        unsafe { &*a_inline }.Depth()
                    };
                    let b_depth = if b_inline.is_null() {
                        0
                    } else {
                        unsafe { &*b_inline }.Depth()
                    };
                    if a_depth != b_depth {
                        return b_depth.cmp(&a_depth);
                    }
                }
                let a_box = a.Node().GetLayoutBox();
                let b_box = b.Node().GetLayoutBox();
                if a_box == b_box {
                    return std::cmp::Ordering::Equal;
                }
                let cache = index_cache.as_mut().map_or(std::ptr::null_mut(), |v| v);
                if unsafe { &*a_box }.IsBeforeInPreOrder(unsafe { &*b_box }, cache) {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            });
            self.oof_candidates_may_have_anchors_ = false;
            if let Some(cache) = index_cache.as_mut() {
                for map in cache.values() {
                    if !map.Get().is_null() {
                        unsafe { &mut *map.Get() }.clear();
                    }
                }
                cache.clear();
            }
        }
        std::mem::swap(&mut self.oof_positioned_candidates_, candidates);
    }

    // cpp: layoutng_out_of_flow/out_of_flow_fragment_builder.cc:197-200
    pub fn ClearOutOfFlowPositionedCandidates(&mut self) {
        self.oof_candidates_may_have_anchors_ = false;
        self.oof_positioned_candidates_.clear();
    }

    // cpp: layoutng_out_of_flow/out_of_flow_fragment_builder.cc:141-146
    pub fn SwapMulticolsWithPendingOOFs(
        &mut self,
        multicols: &mut HeapHashMap<
            Member<LayoutBox>,
            Member<MulticolWithPendingOofs<LogicalOffset>>,
        >,
    ) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        debug_assert!(multicols.is_empty());
        std::mem::swap(&mut self.multicols_with_pending_oofs_, multicols);
    }

    // cpp: layoutng_out_of_flow/out_of_flow_fragment_builder.cc:148-165
    pub fn SwapOutOfFlowFragmentainerDescendants(
        &mut self,
        descendants: &mut HeapVector<LogicalOofNodeForFragmentation>,
    ) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        debug_assert!(descendants.is_empty());
        if self.oof_fragmentainer_descendants_may_have_anchors_ {
            self.oof_positioned_fragmentainer_descendants_
                .sort_by(|a, b| {
                    let a_box = a.Node().GetLayoutBox();
                    let b_box = b.Node().GetLayoutBox();
                    if a_box == b_box {
                        return std::cmp::Ordering::Equal;
                    }
                    if unsafe { &*a_box }.IsBeforeInPreOrderDefault(unsafe { &*b_box }) {
                        std::cmp::Ordering::Less
                    } else {
                        std::cmp::Ordering::Greater
                    }
                });
            self.oof_fragmentainer_descendants_may_have_anchors_ = false;
        }
        std::mem::swap(
            &mut self.oof_positioned_fragmentainer_descendants_,
            descendants,
        );
    }

    // cpp: layoutng_out_of_flow/out_of_flow_fragment_builder.cc:167-195
    pub fn TransferOutOfFlowCandidates(
        &mut self,
        destination_builder: &mut FragmentBuilder,
        additional_offset: LogicalOffset,
        multicol: Option<&MulticolWithPendingOofs<LogicalOffset>>,
    ) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        for candidate in &mut self.oof_positioned_candidates_ {
            let node = candidate.Node();
            candidate.IncreaseStaticPositionOffset(additional_offset);
            if let Some(multicol) = multicol {
                if !multicol.fixedpos_containing_block.Fragment().is_null()
                    && node.Style().GetPosition() == EPosition::kFixed
                {
                    debug_assert!(candidate.InlineContainer().is_null());
                    destination_builder.AddOutOfFlowFragmentainerDescendant(
                        &LogicalOofNodeForFragmentation::new(
                            node,
                            candidate.StaticPosition(),
                            candidate.RequiresContentBeforeBreaking(),
                            multicol.fixedpos_inline_container.clone(),
                            multicol.fixedpos_containing_block.clone(),
                            multicol.fixedpos_containing_block.clone(),
                            multicol.fixedpos_inline_container.clone(),
                        ),
                    );
                    continue;
                }
            }
            destination_builder
                .oof_positioned_candidates_
                .push(candidate.clone());
        }
        destination_builder.oof_candidates_may_have_anchors_ |=
            self.oof_candidates_may_have_anchors_;
        self.ClearOutOfFlowPositionedCandidates();
    }

    pub fn HasOutOfFlowPositionedDescendants(&self) -> bool {
        !self.oof_positioned_descendants_.is_empty()
    }

    pub fn HasOutOfFlowFragmentainerDescendants(&self) -> bool {
        !self.oof_positioned_fragmentainer_descendants_.is_empty()
    }

    pub fn HasMulticolsWithPendingOOFs(&self) -> bool {
        !self.multicols_with_pending_oofs_.is_empty()
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:335-352
    pub fn HasOutOfFlowFragmentChild(&self) -> bool {
        self.has_out_of_flow_fragment_child_
    }

    pub fn SetHasOutOfFlowFragmentChild(&mut self, value: bool) {
        self.has_out_of_flow_fragment_child_ = value;
    }

    pub fn HasOutOfFlowInFragmentainerSubtree(&self) -> bool {
        self.has_out_of_flow_in_fragmentainer_subtree_
    }

    pub fn SetHasOutOfFlowInFragmentainerSubtree(&mut self, value: bool) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        self.has_out_of_flow_in_fragmentainer_subtree_ = value;
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:241-243
    // cpp: layoutng_fragment_tree/fragment_builder.cc:630-644
    pub fn AddOutOfFlowChildCandidate(
        &mut self,
        child: &BlockNode,
        static_position: &LogicalStaticPosition,
        allow_top_layer_nodes: bool,
    ) {
        debug_assert!(!child.IsNull());
        if child.IsInTopOrViewTransitionLayer() && !allow_top_layer_nodes {
            return;
        }
        self.oof_candidates_may_have_anchors_ |= child.MayContainAnchor();
        let requires_content_before_breaking = self.RequiresContentBeforeBreaking();
        self.oof_positioned_candidates_
            .push(LogicalOofPositionedNode::new(
                child.clone(),
                std::ptr::null(),
                *static_position,
                requires_content_before_breaking,
            ));
    }

    pub fn AddOutOfFlowChildCandidateDefault(
        &mut self,
        child: &BlockNode,
        static_position: &LogicalStaticPosition,
    ) {
        self.AddOutOfFlowChildCandidate(child, static_position, false);
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:250-254
    // The definition belongs to //src/layoutng_inline/line_box_fragment_builder.cc.
    pub fn AddOutOfFlowInlineChildCandidate(
        &mut self,
        child: BlockNode,
        child_offset: &LogicalOffset,
        inline_container_writing_direction: WritingDirectionMode,
        line_box_block_size: LayoutUnit,
    ) {
        unsafe {
            FragmentBuilderAddOutOfFlowInlineChildCandidate(
                self,
                child,
                child_offset,
                inline_container_writing_direction,
                line_box_block_size,
            )
        }
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:244-245
    // cpp: layoutng_fragment_tree/fragment_builder.cc:646-653
    pub fn AddOutOfFlowChildCandidateWithBreakToken(
        &mut self,
        child: &BlockNode,
        break_token: &crate::block_break_token::BlockBreakToken,
    ) {
        self.oof_candidates_may_have_anchors_ |= child.MayContainAnchor();
        let requires_content_before_breaking = self.RequiresContentBeforeBreaking();
        self.oof_positioned_candidates_
            .push(LogicalOofPositionedNode::new(
                child.clone(),
                break_token,
                LogicalStaticPosition::default(),
                requires_content_before_breaking,
            ));
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:256-259
    // cpp: layoutng_fragment_tree/fragment_builder.cc:655-669
    pub fn AddOutOfFlowFragmentainerDescendant(
        &mut self,
        descendant: &LogicalOofNodeForFragmentation,
    ) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        self.oof_fragmentainer_descendants_may_have_anchors_ |=
            descendant.Node().MayContainAnchor();
        self.oof_positioned_fragmentainer_descendants_
            .push(descendant.clone());
    }

    pub fn AddOutOfFlowFragmentainerDescendantFromOof(
        &mut self,
        descendant: &LogicalOofPositionedNode,
    ) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        debug_assert!(!descendant.IsForFragmentation());
        let fragmented = LogicalOofNodeForFragmentation::from_oof(descendant);
        self.AddOutOfFlowFragmentainerDescendant(&fragmented);
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:261
    // cpp: layoutng_fragment_tree/fragment_builder.cc:671-674
    pub fn AddOutOfFlowDescendant(&mut self, descendant: &LogicalOofPositionedNode) {
        self.oof_positioned_descendants_.push(descendant.clone());
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:272-275
    // cpp: layoutng_fragment_tree/fragment_builder.cc:676-685
    pub fn AddMulticolWithPendingOOFs(
        &mut self,
        multicol: &BlockNode,
        multicol_info: *mut MulticolWithPendingOofs<LogicalOffset>,
    ) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        let box_object = multicol.GetLayoutBox();
        debug_assert!(unsafe { &*box_object }.IsMulticolContainer());
        let key = Member::from_ptr(box_object);
        if self.multicols_with_pending_oofs_.Contains(&key) {
            return;
        }
        self.multicols_with_pending_oofs_
            .insert(key, Member::from_ptr(multicol_info));
    }

    pub fn AddMulticolWithPendingOOFsDefault(&mut self, multicol: &BlockNode) {
        self.AddMulticolWithPendingOOFs(
            multicol,
            MakeGarbageCollected(MulticolWithPendingOofs::default()),
        );
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:318
    // cpp: layoutng_fragment_tree/fragment_builder.cc:687-708
    pub fn MoveOutOfFlowDescendantCandidatesToDescendants(&mut self) {
        debug_assert!(self.oof_positioned_descendants_.is_empty());
        let layout_inline = DynamicTo::<LayoutInline>(self.layout_object_);
        if !layout_inline.is_null()
            && unsafe { &*layout_inline }.CanContainAbsolutePositionObjects()
        {
            for candidate in &mut self.oof_positioned_candidates_ {
                if candidate.InlineContainer().is_null()
                    && IsInlineContainerForNode(&candidate.Node(), layout_inline)
                {
                    candidate.SetInlineContainer(layout_inline);
                }
            }
        }
        std::mem::swap(
            &mut self.oof_positioned_candidates_,
            &mut self.oof_positioned_descendants_,
        );
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:502-503
    // cpp: layoutng_fragment_tree/fragment_builder.cc:175-182
    fn EnsureStickyDescendants(
        &mut self,
    ) -> &mut GCedHeapVector<SplitAxisItem<LayoutBoxModelObject>> {
        if self.sticky_descendants_.is_null() {
            self.sticky_descendants_ = MakeGarbageCollected(GCedHeapVector::default());
        }
        unsafe { &mut *self.sticky_descendants_ }
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:201
    // cpp: layoutng_fragment_tree/fragment_builder.cc:184-219
    pub fn PropagateStickyDescendants(&mut self, child: &PhysicalFragment) {
        let has_sticky_position = child.HasStickyConstrainedPosition();
        let sticky_descendants = child.StickyDescendants();
        if !has_sticky_position && sticky_descendants.is_empty() {
            return;
        }
        let scrollable_axes = self.GetOverflowScrollAxes();
        if has_sticky_position {
            let axes = LayoutBoxModelObject::StickyConstrainedAxes(child.Style());
            let (consumed, pending) = PartitionAxes(scrollable_axes, axes);
            self.EnsureStickyDescendants().push(SplitAxisItem::new(
                To::<LayoutBoxModelObject>(child.GetMutableLayoutObject()),
                consumed,
                pending,
            ));
        }
        for item in sticky_descendants {
            let pending_object = item.GetIfPending();
            if !pending_object.is_null() {
                let (consumed, pending) = PartitionAxes(scrollable_axes, item.PendingAxes());
                self.EnsureStickyDescendants().push(SplitAxisItem::new(
                    pending_object,
                    consumed,
                    pending,
                ));
            }
        }
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:504
    // cpp: layoutng_fragment_tree/fragment_builder.cc:221-226
    fn EnsureSnapAreas(&mut self) -> &mut GCedHeapVector<SnapArea> {
        if self.snap_areas_.is_null() {
            self.snap_areas_ = MakeGarbageCollected(GCedHeapVector::default());
        }
        unsafe { &mut *self.snap_areas_ }
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:515
    // cpp: layoutng_fragment_tree/fragment_builder.cc:406-420
    fn GetOverflowScrollAxes(&self) -> PhysicalAxes {
        if self.node_.IsNull() || self.node_.IsInline() || self.IsFragmentainerBoxType() {
            return kPhysicalAxesNone;
        }
        let box_object = DynamicTo::<LayoutBox>(self.GetLayoutObject());
        if !box_object.is_null() && unsafe { &*box_object }.IsScrollContainer() {
            if !unsafe { &*box_object }.GetScrollableArea().is_null() {
                return unsafe { &*box_object }.ScrollableAxesForLayout();
            }
        }
        kPhysicalAxesNone
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:517
    // cpp: layoutng_fragment_tree/fragment_builder.cc:422-435
    fn GetScrollSnapAxes(&self) -> PhysicalAxes {
        let overflow_scroll_axes = self.GetOverflowScrollAxes();
        if RuntimeEnabledFeatures::SingleAxisScrollContainersForScrollSnapEnabled() {
            return overflow_scroll_axes;
        }
        if overflow_scroll_axes == kPhysicalAxesNone {
            kPhysicalAxesNone
        } else {
            kPhysicalAxesBoth
        }
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:519
    // cpp: layoutng_fragment_tree/fragment_builder.cc:437-462
    fn ResolveSnapArea(&self, snap_area: &SnapArea) -> SnapArea {
        let scroll_snap_axes = self.GetScrollSnapAxes();
        if scroll_snap_axes == kPhysicalAxesNone {
            return snap_area.clone();
        }
        let (writing_direction_mode, pending_axes) = if !snap_area.Resolved() {
            let direction = self.GetWritingDirection();
            let element = unsafe { &*snap_area.GetElement() };
            (
                Some(direction),
                GetScrollSnapAlignAxes(&element.ComputedStyleRef().GetScrollSnapAlign(), direction),
            )
        } else {
            (
                snap_area.ContainerWritingDirectionMode(),
                snap_area.PendingAxes(),
            )
        };
        let (consumed, pending) = PartitionAxes(scroll_snap_axes, pending_axes);
        SnapArea::new(
            snap_area.GetElement(),
            consumed,
            pending,
            writing_direction_mode,
        )
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:202
    // cpp: layoutng_fragment_tree/fragment_builder.cc:228-270
    pub fn PropagateSnapAreas(&mut self, child: &PhysicalFragment) {
        let mut resolved_child_snap_areas = HeapVector::<SnapArea>::default();
        if child.IsSnapArea() {
            let box_fragment = unsafe { &*To::<PhysicalBoxFragment>(child as *const _) };
            if box_fragment.GetBreakToken().is_null() {
                let node = unsafe { &*child.GetLayoutObject() }.GetNode();
                let element = To::<Element>(node);
                resolved_child_snap_areas
                    .push(self.ResolveSnapArea(&SnapArea::from_element(element)));
            }
        }
        for item in child.SnapAreas() {
            if item.IsPending() {
                resolved_child_snap_areas.push(self.ResolveSnapArea(item));
            }
        }
        if resolved_child_snap_areas.is_empty() {
            return;
        }

        let new_element = resolved_child_snap_areas[0].GetElement();
        let new_box = unsafe { &*new_element }.GetLayoutBox();
        let snap_areas = self.EnsureSnapAreas();
        let insertion_pos = if new_box.is_null() {
            snap_areas.len()
        } else {
            let mut position = 0;
            for i in (1..=snap_areas.len()).rev() {
                let existing = snap_areas[i - 1].GetElement();
                let existing_box = unsafe { &*existing }.GetLayoutBox();
                if !existing_box.is_null()
                    && unsafe { &*existing_box }.IsBeforeInPreOrderDefault(unsafe { &*new_box })
                {
                    position = i;
                    break;
                }
            }
            position
        };
        snap_areas.InsertVector(
            insertion_pos
                .try_into()
                .expect("snap area count exceeds 32 bits"),
            &resolved_child_snap_areas,
        );
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:539
    // cpp: layoutng_fragment_tree/fragment_builder.cc:383-390
    fn UpdateScrollInitialTarget(&mut self, new_target: *const LayoutObject) {
        if new_target != self.scroll_start_target_
            && (self.scroll_start_target_.is_null()
                || unsafe { &*new_target }
                    .IsBeforeInPreOrderDefault(unsafe { &*self.scroll_start_target_ }))
        {
            self.scroll_start_target_ = new_target;
        }
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:513
    // cpp: layoutng_fragment_tree/fragment_builder.cc:392-404
    fn PropagateScrollInitialTarget(&mut self, child: &PhysicalFragment) {
        if child.Style().ScrollInitialTarget() != EScrollInitialTarget::kNone {
            let child_object = child.GetMutableLayoutObject();
            if !child_object.is_null() {
                self.UpdateScrollInitialTarget(child_object);
            }
        }
        let target = child.PropagatedScrollInitialTarget();
        if !target.Get().is_null() {
            self.UpdateScrollInitialTarget(target.Get());
        }
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:527
    // cpp: layoutng_fragment_tree/fragment_builder.cc:608-628
    pub(crate) fn AddChildInternal(&mut self, child: &PhysicalFragment, offset: LogicalOffset) {
        if child.IsListMarker() {
            self.children_
                .insert(0, LogicalFragmentLink::new(child, offset));
            return;
        }
        if child.IsTextControlPlaceholder() {
            let size = self.children_.len();
            if size > 0 {
                self.children_
                    .insert(size - 1, LogicalFragmentLink::new(child, offset));
                return;
            }
        }
        self.children_.push(LogicalFragmentLink::new(child, offset));
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:358-370
    // cpp: layoutng_fragment_tree/fragment_builder.cc:721-898
    pub fn PropagateOOFPositionedInfo(
        &mut self,
        fragment: &PhysicalFragment,
        offset: LogicalOffset,
        relative_offset: LogicalOffset,
        offset_adjustment: LogicalOffset,
        inline_container: *const OofInlineContainer<LogicalOffset>,
        containing_block_adjustment: LayoutUnit,
        containing_block: *const OofContainingBlock<LogicalOffset>,
        fixedpos_containing_block: *const OofContainingBlock<LogicalOffset>,
        fixedpos_inline_container: *const OofInlineContainer<LogicalOffset>,
        additional_fixedpos_offset: LogicalOffset,
    ) {
        debug_assert!(fragment.NeedsOOFPositionedInfoPropagation());
        let adjusted_offset = offset + offset_adjustment + relative_offset;
        let converter = WritingModeConverter::new(self.GetWritingDirection(), fragment.Size());
        let descendants = unsafe { &*fragment.OutOfFlowPositionedDescendants() };
        for descendant in descendants {
            let node = descendant.Node();
            let mut logical_descendant = PhysicalOofPositionedNodeToLogical(descendant, &converter);
            let mut static_position = logical_descendant.StaticPosition();
            let new_inline_container = logical_descendant.InlineContainerInfoMut();
            if new_inline_container.Container().is_null()
                && !inline_container.is_null()
                && IsInlineContainerForNode(&node, unsafe { &*inline_container }.Container())
            {
                *new_inline_container = unsafe { &*inline_container }.clone();
            } else if !RuntimeEnabledFeatures::FragmentedOofInCbEnabled()
                && !new_inline_container.Container().is_null()
            {
                new_inline_container.IncreaseRelativeOffset(relative_offset);
            }

            if !RuntimeEnabledFeatures::FragmentedOofInCbEnabled()
                && (!fixedpos_containing_block.is_null()
                    || additional_fixedpos_offset != LogicalOffset::default())
                && node.Style().GetPosition() == EPosition::kFixed
            {
                static_position.offset += additional_fixedpos_offset;
                static_position.offset +=
                    relative_offset - unsafe { &*fixedpos_containing_block }.RelativeOffset();
                if !fixedpos_inline_container.is_null() {
                    static_position.offset -=
                        unsafe { &*fixedpos_inline_container }.RelativeOffset();
                }
                if !fixedpos_containing_block.is_null()
                    && (!unsafe { &*fixedpos_containing_block }.Fragment().is_null()
                        || self.node_.IsPaginatedRoot())
                {
                    let new_fixedpos_inline_container = if fixedpos_inline_container.is_null() {
                        OofInlineContainer::default()
                    } else {
                        unsafe { &*fixedpos_inline_container }.clone()
                    };
                    self.AddOutOfFlowFragmentainerDescendant(&LogicalOofNodeForFragmentation::new(
                        node,
                        static_position,
                        descendant.RequiresContentBeforeBreaking(),
                        new_fixedpos_inline_container.clone(),
                        unsafe { &*fixedpos_containing_block }.clone(),
                        unsafe { &*fixedpos_containing_block }.clone(),
                        new_fixedpos_inline_container,
                    ));
                    continue;
                }
            }
            static_position.offset += adjusted_offset;
            debug_assert!(!self
                .oof_positioned_candidates_
                .iter()
                .any(|candidate| candidate.Node() == node));
            self.oof_candidates_may_have_anchors_ |= node.MayContainAnchor();
            logical_descendant.SetStaticPositionOffset(static_position.offset);
            self.oof_positioned_candidates_.push(logical_descendant);
        }

        let oof_data = fragment.GetFragmentedOofData();
        if oof_data.is_null() {
            return;
        }
        let oof_data = unsafe { &*oof_data };
        debug_assert!(
            !oof_data.multicols_with_pending_oofs.is_empty()
                || !oof_data.oof_positioned_fragmentainer_descendants.is_empty()
        );
        let box_fragment = DynamicTo::<PhysicalBoxFragment>(fragment as *const PhysicalFragment);
        let is_column_spanner =
            !box_fragment.is_null() && unsafe { &*box_fragment }.IsColumnSpanAll();
        for (multicol, value) in oof_data.multicols_with_pending_oofs.iter() {
            let multicol_info = unsafe { &*value.Get() };
            let mut multicol_offset = converter.ToLogicalOffset(
                multicol_info.multicol_offset,
                foundation::PhysicalSize::default(),
            );
            let fixedpos_inline_relative_offset = converter.ToLogicalOffset(
                multicol_info.fixedpos_inline_container.RelativeOffset(),
                foundation::PhysicalSize::default(),
            );
            let mut new_fixedpos_inline_container = OofInlineContainer::new(
                multicol_info.fixedpos_inline_container.Container(),
                fixedpos_inline_relative_offset,
            );
            let mut fixedpos_containing_block_fragment =
                multicol_info.fixedpos_containing_block.Fragment();
            self.AdjustFixedposContainerInfo(
                box_fragment,
                relative_offset,
                &mut new_fixedpos_inline_container,
                &mut fixedpos_containing_block_fragment,
                std::ptr::null(),
            );
            let mut fixedpos_containing_block_offset = LogicalOffset::default();
            let mut fixedpos_containing_block_rel_offset = LogicalOffset::default();
            let mut is_inside_column_spanner = multicol_info
                .fixedpos_containing_block
                .IsInsideColumnSpanner();
            if !fixedpos_containing_block_fragment.is_null() {
                fixedpos_containing_block_offset = converter.ToLogicalOffset(
                    multicol_info.fixedpos_containing_block.Offset(),
                    unsafe { &*fixedpos_containing_block_fragment }.Size(),
                );
                fixedpos_containing_block_rel_offset = RelativeInsetToLogical(
                    multicol_info.fixedpos_containing_block.RelativeOffset(),
                    self.GetWritingDirection(),
                );
                fixedpos_containing_block_rel_offset += relative_offset;
                if !fragment.IsFragmentainerBox() {
                    fixedpos_containing_block_offset += offset;
                }
                fixedpos_containing_block_offset.block_offset += containing_block_adjustment;
                if is_column_spanner {
                    is_inside_column_spanner = true;
                }
            } else {
                multicol_offset += adjusted_offset;
            }
            let fixedpos_clipped_container_block_offset = None;
            let multicol_node = BlockNode::new(multicol.Get());
            self.AddMulticolWithPendingOOFs(
                &multicol_node,
                MakeGarbageCollected(MulticolWithPendingOofs::new(
                    multicol_offset,
                    OofContainingBlock::new(
                        fixedpos_containing_block_offset,
                        fixedpos_containing_block_rel_offset,
                        fixedpos_containing_block_fragment,
                        fixedpos_clipped_container_block_offset,
                        is_inside_column_spanner,
                    ),
                    new_fixedpos_inline_container,
                )),
            );
        }
        self.PropagateOOFFragmentainerDescendants(
            fragment,
            offset,
            relative_offset,
            containing_block_adjustment,
            containing_block,
            fixedpos_containing_block,
            std::ptr::null_mut(),
        );
    }

    pub fn PropagateOOFPositionedInfoDefault(
        &mut self,
        fragment: &PhysicalFragment,
        offset: LogicalOffset,
        relative_offset: LogicalOffset,
    ) {
        self.PropagateOOFPositionedInfo(
            fragment,
            offset,
            relative_offset,
            LogicalOffset::default(),
            std::ptr::null(),
            LayoutUnit::default(),
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
            LogicalOffset::default(),
        );
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:531-537
    // cpp: layoutng_fragment_tree/fragment_builder.cc:1083-1118
    fn AdjustFixedposContainerInfo(
        &self,
        box_fragment: *const PhysicalBoxFragment,
        relative_offset: LogicalOffset,
        fixedpos_inline_container: &mut OofInlineContainer<LogicalOffset>,
        fixedpos_containing_block_fragment: &mut *const PhysicalFragment,
        current_inline_container: *const OofInlineContainer<LogicalOffset>,
    ) {
        if box_fragment.is_null() {
            return;
        }
        let box_fragment_ref = unsafe { &*box_fragment };
        let layout_object = box_fragment_ref.GetLayoutObject();
        if !(*fixedpos_containing_block_fragment).is_null() || layout_object.is_null() {
            return;
        }
        if !current_inline_container.is_null()
            && !unsafe { &*current_inline_container }.Container().is_null()
            && unsafe { &*(*current_inline_container).Container() }.CanContainFixedPositionObjects()
        {
            *fixedpos_inline_container = unsafe { &*current_inline_container }.clone();
            *fixedpos_containing_block_fragment = box_fragment as *const PhysicalFragment;
        } else if unsafe { &*layout_object }.CanContainFixedPositionObjects() {
            if fixedpos_inline_container.Container().is_null()
                && unsafe { &*layout_object }.IsLayoutInline()
            {
                *fixedpos_inline_container =
                    OofInlineContainer::new(To::<LayoutInline>(layout_object), relative_offset);
            } else if !unsafe { &*layout_object }.IsLayoutInline() {
                *fixedpos_containing_block_fragment = box_fragment as *const PhysicalFragment;
            }
        } else if !fixedpos_inline_container.Container().is_null()
            && layout_object
                == unsafe { &*fixedpos_inline_container.Container() }.ContainingBlock()
                    as *const LayoutObject
        {
            *fixedpos_containing_block_fragment = box_fragment as *const PhysicalFragment;
        }
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:375-382
    // cpp: layoutng_fragment_tree/fragment_builder.cc:900-1081
    pub fn PropagateOOFFragmentainerDescendants(
        &mut self,
        fragment: &PhysicalFragment,
        offset: LogicalOffset,
        relative_offset: LogicalOffset,
        containing_block_adjustment: LayoutUnit,
        containing_block: *const OofContainingBlock<LogicalOffset>,
        fixedpos_containing_block: *const OofContainingBlock<LogicalOffset>,
        out_list: *mut HeapVector<LogicalOofNodeForFragmentation>,
    ) {
        debug_assert!(!RuntimeEnabledFeatures::FragmentedOofInCbEnabled());
        let oof_data = fragment.GetFragmentedOofData();
        if oof_data.is_null()
            || unsafe { &*oof_data }
                .oof_positioned_fragmentainer_descendants
                .is_empty()
        {
            return;
        }
        let converter = WritingModeConverter::new(self.GetWritingDirection(), fragment.Size());
        let box_fragment = DynamicTo::<PhysicalBoxFragment>(fragment as *const PhysicalFragment);
        let is_column_spanner =
            !box_fragment.is_null() && unsafe { &*box_fragment }.IsColumnSpanAll();
        for descendant in &unsafe { &*oof_data }.oof_positioned_fragmentainer_descendants {
            let mut containing_block_fragment = descendant.containing_block.Fragment();
            let mut container_inside_column_spanner =
                descendant.containing_block.IsInsideColumnSpanner();
            let mut fixedpos_container_inside_column_spanner =
                descendant.fixedpos_containing_block.IsInsideColumnSpanner();
            if containing_block_fragment.is_null() {
                debug_assert!(!box_fragment.is_null());
                containing_block_fragment = box_fragment as *const PhysicalFragment;
            } else if !box_fragment.is_null()
                && unsafe { &*box_fragment }.IsFragmentationContextRoot()
            {
                if container_inside_column_spanner {
                    container_inside_column_spanner = false;
                    fixedpos_container_inside_column_spanner = false;
                } else {
                    debug_assert!(!fixedpos_container_inside_column_spanner);
                    continue;
                }
            }
            if is_column_spanner {
                container_inside_column_spanner = true;
            }
            let mut containing_block_offset = converter.ToLogicalOffset(
                descendant.containing_block.Offset(),
                unsafe { &*containing_block_fragment }.Size(),
            );
            let mut containing_block_rel_offset = RelativeInsetToLogical(
                descendant.containing_block.RelativeOffset(),
                self.GetWritingDirection(),
            );
            containing_block_rel_offset += relative_offset;
            if !fragment.IsFragmentainerBox() {
                containing_block_offset += offset;
            }
            containing_block_offset.block_offset += containing_block_adjustment;

            let updated_clipped_container_block_offset =
                |descendant_containing_block: &OofContainingBlock<PhysicalOffset>| {
                    let mut clipped = descendant_containing_block.ClippedContainerBlockOffset();
                    if clipped.is_none() && fragment.HasNonVisibleBlockOverflow() {
                        clipped = Some(LayoutUnit::default());
                    }
                    if let Some(value) = &mut clipped {
                        if !fragment.IsFragmentainerBox() {
                            *value += offset.block_offset;
                        }
                        *value += containing_block_adjustment;
                    }
                    if clipped.is_none() && !containing_block.is_null() {
                        let inherited = unsafe { &*containing_block }.ClippedContainerBlockOffset();
                        if inherited.is_some() {
                            clipped = inherited;
                        }
                    }
                    clipped
                };
            let clipped_container_block_offset =
                updated_clipped_container_block_offset(&descendant.containing_block);

            let inline_relative_offset = converter.ToLogicalOffset(
                descendant.InlineContainerInfo().RelativeOffset(),
                foundation::PhysicalSize::default(),
            );
            let new_inline_container =
                OofInlineContainer::new(descendant.InlineContainer(), inline_relative_offset);
            let containing_block_converter = WritingModeConverter::new(
                self.GetWritingDirection(),
                unsafe { &*containing_block_fragment }.Size(),
            );
            let mut static_position = descendant
                .StaticPosition()
                .ConvertToLogical(&containing_block_converter);
            if !new_inline_container.Container().is_null()
                && !box_fragment.is_null()
                && containing_block_fragment == box_fragment as *const PhysicalFragment
            {
                static_position.offset -= inline_relative_offset;
            }

            let fixedpos_inline_relative_offset = converter.ToLogicalOffset(
                descendant.fixedpos_inline_container.RelativeOffset(),
                foundation::PhysicalSize::default(),
            );
            let mut new_fixedpos_inline_container = OofInlineContainer::new(
                descendant.fixedpos_inline_container.Container(),
                fixedpos_inline_relative_offset,
            );
            let mut fixedpos_containing_block_fragment =
                descendant.fixedpos_containing_block.Fragment();
            self.AdjustFixedposContainerInfo(
                box_fragment,
                relative_offset,
                &mut new_fixedpos_inline_container,
                &mut fixedpos_containing_block_fragment,
                &new_inline_container,
            );

            let mut fixedpos_containing_block_offset = LogicalOffset::default();
            let mut fixedpos_containing_block_rel_offset = LogicalOffset::default();
            let mut fixedpos_clipped_container_block_offset = None;
            if !fixedpos_containing_block_fragment.is_null() {
                fixedpos_containing_block_offset = converter.ToLogicalOffset(
                    descendant.fixedpos_containing_block.Offset(),
                    unsafe { &*fixedpos_containing_block_fragment }.Size(),
                );
                fixedpos_containing_block_rel_offset = RelativeInsetToLogical(
                    descendant.fixedpos_containing_block.RelativeOffset(),
                    self.GetWritingDirection(),
                );
                fixedpos_containing_block_rel_offset += relative_offset;
                if !fragment.IsFragmentainerBox() {
                    fixedpos_containing_block_offset += offset;
                }
                fixedpos_containing_block_offset.block_offset += containing_block_adjustment;
                fixedpos_clipped_container_block_offset =
                    updated_clipped_container_block_offset(&descendant.fixedpos_containing_block);
                if is_column_spanner {
                    fixedpos_container_inside_column_spanner = true;
                }
            }
            if fixedpos_containing_block_fragment.is_null() && !fixedpos_containing_block.is_null()
            {
                let inherited = unsafe { &*fixedpos_containing_block };
                fixedpos_containing_block_fragment = inherited.Fragment();
                fixedpos_containing_block_offset = inherited.Offset();
                fixedpos_containing_block_rel_offset = inherited.RelativeOffset();
            }
            let oof_node = LogicalOofNodeForFragmentation::new(
                descendant.Node(),
                static_position,
                descendant.RequiresContentBeforeBreaking(),
                new_inline_container,
                OofContainingBlock::new(
                    containing_block_offset,
                    containing_block_rel_offset,
                    containing_block_fragment,
                    clipped_container_block_offset,
                    container_inside_column_spanner,
                ),
                OofContainingBlock::new(
                    fixedpos_containing_block_offset,
                    fixedpos_containing_block_rel_offset,
                    fixedpos_containing_block_fragment,
                    fixedpos_clipped_container_block_offset,
                    fixedpos_container_inside_column_spanner,
                ),
                new_fixedpos_inline_container,
            );
            if out_list.is_null() {
                self.AddOutOfFlowFragmentainerDescendant(&oof_node);
            } else {
                unsafe { &mut *out_list }.push(oof_node);
            }
        }
    }

    pub fn PropagateOOFFragmentainerDescendantsDefault(
        &mut self,
        fragment: &PhysicalFragment,
        offset: LogicalOffset,
        relative_offset: LogicalOffset,
        containing_block_adjustment: LayoutUnit,
        containing_block: *const OofContainingBlock<LogicalOffset>,
        fixedpos_containing_block: *const OofContainingBlock<LogicalOffset>,
    ) {
        self.PropagateOOFFragmentainerDescendants(
            fragment,
            offset,
            relative_offset,
            containing_block_adjustment,
            containing_block,
            fixedpos_containing_block,
            std::ptr::null_mut(),
        );
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:332-333
    // cpp: layoutng_fragment_tree/fragment_builder.cc:710-719
    pub fn BlockOffsetAdjustmentForFragmentainer(
        &self,
        fragmentainer_consumed_block_size: LayoutUnit,
    ) -> LayoutUnit {
        if RuntimeEnabledFeatures::FragmentedOofInCbEnabled() {
            return LayoutUnit::default();
        }
        if self.IsFragmentainerBoxType() && !self.PreviousBreakToken().is_null() {
            let token = To::<crate::block_break_token::BlockBreakToken>(self.PreviousBreakToken());
            return unsafe { &*token }.ConsumedBlockSize();
        }
        fragmentainer_consumed_block_size
    }

    pub fn BlockOffsetAdjustmentForFragmentainerDefault(&self) -> LayoutUnit {
        self.BlockOffsetAdjustmentForFragmentainer(LayoutUnit::default())
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:462
    // cpp: layoutng_fragment_tree/fragment_builder.cc:1120-1129
    pub fn PropagateSpaceShortage(&mut self, space_shortage: Option<LayoutUnit>) {
        debug_assert!(!self.GetConstraintSpace().IsInitialColumnBalancingPass());
        UpdateMinimalSpaceShortage(space_shortage, &mut self.minimal_space_shortage_);
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:487
    // cpp: layoutng_fragment_tree/fragment_builder.cc:1131-1139
    pub fn Finalize(&mut self) {
        #[cfg(debug_assertions)]
        {
            debug_assert!(!self.is_finalized_);
            self.is_finalized_ = true;
        }
        self.has_final_size_ = true;
        self.PropagateSizeDependentData();
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:489
    // cpp: layoutng_fragment_tree/fragment_builder.cc:1141-1144
    pub fn Abort(&mut self, status: EStatus) -> *const LayoutResult {
        MakeGarbageCollected(LayoutResult::from_failed_fragment_builder(status, self))
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:492
    // cpp: layoutng_fragment_tree/fragment_builder.cc:1148-1159
    #[cfg(debug_assertions)]
    pub fn ToString(&self) -> String {
        let mut builder = StringBuilder::default();
        builder.Append(&format!(
            "FragmentBuilder {:.2}x{:.2}, Children {}\n",
            self.InlineSize().ToFloat(),
            self.BlockSize().ToFloat(),
            self.children_.len()
        ));
        for child in &self.children_ {
            builder.Append(&unsafe { &*child.get() }.DumpFragmentTree(
                PhysicalFragment::DumpAll & !PhysicalFragment::DumpHeaderText,
                std::ptr::null(),
                None,
                2,
            ));
        }
        builder.ReleaseString()
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:542
    // cpp: layoutng_fragment_tree/fragment_builder.cc:1161-1168
    pub(crate) fn PropagateSizeDependentData(&mut self) {
        debug_assert!(self.has_final_size_);
        let pending = std::mem::take(&mut self.children_with_size_dependent_propagation_);
        for link in &pending {
            self.PropagateChildAnchors(unsafe { &*link.get() }, link.offset);
        }
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:546-547
    // cpp: layoutng_fragment_tree/fragment_builder.cc:1170-1200
    fn SetNamedTrigger(&mut self, name: &TriggerScopedName, owner: *const Element) {
        let named_triggers = self.EnsureNamedTriggers();
        let key = Member::from_ptr(name as *const TriggerScopedName as *mut TriggerScopedName);
        let Some(existing) = named_triggers.get(&key) else {
            named_triggers.Set(key, Member::from_ptr(owner as *mut Element));
            return;
        };
        if existing.Get() == owner as *mut Element {
            return;
        }
        debug_assert!(!owner.is_null());
        let owner_object = unsafe { &*owner }.GetLayoutObject();
        debug_assert!(!owner_object.is_null());
        let existing_owner = existing.Get();
        debug_assert!(!existing_owner.is_null());
        let existing_object = unsafe { &*existing_owner }.GetLayoutObject();
        debug_assert!(!existing_object.is_null());
        if unsafe { &*existing_object }.IsBeforeInPreOrderDefault(unsafe { &*owner_object }) {
            named_triggers.Set(key, Member::from_ptr(owner as *mut Element));
            let updated = named_triggers
                .get(&key)
                .expect("named trigger remains present");
            debug_assert_eq!(unsafe { &*updated.Get() }.GetLayoutObject(), owner_object);
        }
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:544
    // cpp: layoutng_fragment_tree/fragment_builder.cc:1202-1227
    fn PropagateNamedTriggers(&mut self, child: &PhysicalFragment) {
        let child_element = DynamicTo::<Element>(child.GetNode());
        let descendant_names = child.NamedTriggers();
        if child_element.is_null() && descendant_names.is_null() {
            return;
        }
        if !child_element.is_null() {
            for name in unsafe { &*child_element }.InputTimelineTriggerNames() {
                let name = name.Get();
                if !name.is_null() {
                    let scoped = ToTriggerScopedName(unsafe { &*name }, unsafe { &*child_element });
                    self.SetNamedTrigger(unsafe { &*scoped }, child_element);
                }
            }
        }
        if !descendant_names.is_null() {
            for (key, value) in unsafe { &*descendant_names }.iter() {
                self.SetNamedTrigger(unsafe { &*key.Get() }, value.Get());
            }
        }
    }

    // cpp: layoutng_fragment_tree/fragment_builder.h:545
    // cpp: layoutng_fragment_tree/fragment_builder.cc:1229-1235
    fn EnsureNamedTriggers(&mut self) -> &mut TriggerScopedNameMap {
        if self.named_triggers_.is_null() {
            self.named_triggers_ = MakeGarbageCollected(TriggerScopedNameMap::default());
        }
        unsafe { &mut *self.named_triggers_ }
    }
}

// cpp: layoutng_fragment_tree/fragment_builder.h:41-49
impl Drop for FragmentBuilder {
    fn drop(&mut self) {
        self.children_.clear();
        self.oof_positioned_candidates_.clear();
        self.oof_positioned_fragmentainer_descendants_.clear();
        self.oof_positioned_descendants_.clear();
        self.multicols_with_pending_oofs_.clear();
        self.child_break_tokens_.clear();
    }
}

#![allow(non_snake_case)]

use std::ptr;

use foundation::{EFloat, LayoutUnit, MakeGarbageCollected, TextDirection};
use layoutng_assembly::block_break_token::BlockBreakToken;
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_assembly::internal::break_appeal::kBreakAppealPerfect;
use layoutng_assembly::internal::constraint_space::{ConstraintSpace, FragmentationType};
use layoutng_assembly::internal::constraint_space_builder::ConstraintSpaceBuilder;
use layoutng_assembly::internal::exclusions::exclusion_area::{ExclusionArea, ExclusionShapeData};
use layoutng_assembly::internal::exclusions::exclusion_space::ExclusionSpace;
use layoutng_assembly::internal::exclusions::layout_opportunity::LayoutOpportunity;
use layoutng_assembly::internal::fragmentation_utils::{
    AdjustMarginsForFragmentation, CalculateSpaceShortage, CalculateUnbreakableBlockSize,
    IsBreakInside, MovePastBreakpointFull, SetupSpaceBuilderForFragmentationFromSpace,
    ShouldAvoidBreakInside,
};
use layoutng_assembly::internal::length_utils::{
    ComputeBorders, ComputeMarginsFor, ComputeMarginsForInlineSize, ComputePadding,
};
use layoutng_assembly::internal::positioned_float::PositionedFloat;
use layoutng_assembly::internal::space_utils::{
    AdjustToClearance, SetOrthogonalFallbackInlineSizeIfNeeded,
};
use layoutng_assembly::internal::unpositioned_float::UnpositionedFloat;
use layoutng_assembly::layout_result::{EStatus, LayoutResult};
use layoutng_assembly::logical_fragment::LogicalFragment;
use layoutng_assembly::physical_box_fragment::PhysicalBoxFragment;
use layoutng_geometry::geometry::bfc_offset::BfcOffset;
use layoutng_geometry::geometry::bfc_rect::BfcRect;
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_style::style::computed_style_constants::ShapeBox;

// cpp: layoutng_float/float_layout_algorithm.cc:27-38
fn AdjustToTopEdgeAlignmentRule(exclusion_space: &ExclusionSpace, offset: &BfcOffset) -> BfcOffset {
    let mut adjusted_offset = *offset;
    adjusted_offset.block_offset = adjusted_offset
        .block_offset
        .max(exclusion_space.LastFloatBlockStart());
    adjusted_offset
}

// cpp: layoutng_float/float_layout_algorithm.cc:40-61
fn FindLayoutOpportunityForFloat(
    unpositioned_float: &UnpositionedFloat,
    exclusion_space: &ExclusionSpace,
    fragment_margins: &BoxStrut,
    inline_size: LayoutUnit,
) -> LayoutOpportunity {
    let mut adjusted_origin_point =
        AdjustToTopEdgeAlignmentRule(exclusion_space, &unpositioned_float.origin_bfc_offset);
    let direction = unpositioned_float.parent_space.Direction();
    let clear_type = unpositioned_float.ClearType(direction);
    let float_type = unpositioned_float
        .node
        .Style()
        .FloatingWithDirection(direction);
    let clearance_offset = exclusion_space
        .ClearanceOffset(clear_type)
        .max(exclusion_space.InitialLetterClearanceOffsetForFloat(float_type));
    AdjustToClearance(clearance_offset, &mut adjusted_origin_point);
    exclusion_space.FindLayoutOpportunity(
        &adjusted_origin_point,
        unpositioned_float.available_size.inline_size,
        direction,
        inline_size + fragment_margins.InlineSum(),
    )
}

// cpp: layoutng_float/float_layout_algorithm.cc:63-105
fn CreateConstraintSpaceForFloat(
    unpositioned_float: &UnpositionedFloat,
    origin_block_offset: Option<LayoutUnit>,
    margins: Option<BoxStrut>,
) -> ConstraintSpace {
    let style = unpositioned_float.node.Style();
    let parent_space = unpositioned_float.parent_space;
    let mut builder = ConstraintSpaceBuilder::new(parent_space, style.GetWritingDirection(), true);
    SetOrthogonalFallbackInlineSizeIfNeeded(
        unpositioned_float.parent_style,
        unpositioned_float.node.clone().into(),
        &mut builder,
    );
    builder.SetIsPaintedAtomically(true);
    builder.SetIsHiddenForPaint(unpositioned_float.is_hidden_for_paint);
    if let Some(origin_block_offset) = origin_block_offset {
        debug_assert!(margins.is_some());
        debug_assert!(parent_space.HasBlockFragmentation());
        debug_assert_eq!(style.GetWritingMode(), parent_space.GetWritingMode());
        SetupSpaceBuilderForFragmentationFromSpace(
            parent_space,
            &unpositioned_float.node.clone().into(),
            unpositioned_float.fragmentainer_block_offset + origin_block_offset,
            unpositioned_float.fragmentainer_block_size,
            false,
            &mut builder,
        );
        let margin_edge_offset = unpositioned_float.fragmentainer_block_offset
            + origin_block_offset
            - margins.unwrap().block_start;
        if margin_edge_offset <= LayoutUnit::default() {
            builder.SetIsAtFragmentainerStart();
        }
    } else {
        builder.SetFragmentationType(FragmentationType::kFragmentNone);
    }
    builder.SetAvailableSize(unpositioned_float.available_size);
    builder.SetPercentageResolutionSize(unpositioned_float.percentage_size);
    builder.ToConstraintSpace()
}

// cpp: layoutng_float/float_layout_algorithm.cc:107-145
fn CreateExclusionShapeData(
    margins: &BoxStrut,
    unpositioned_float: &UnpositionedFloat,
) -> *const ExclusionShapeData {
    let layout_box = unpositioned_float.node.GetLayoutBox();
    debug_assert!(!unsafe { &*layout_box }.GetShapeOutsideInfo().is_null());
    let parent_space = unpositioned_float.parent_space;
    let direction = parent_space.Direction();
    let new_margins = BoxStrut::new(
        margins.LineLeft(direction),
        margins.LineRight(direction),
        margins.block_start,
        margins.block_end,
    );
    let mut shape_insets = BoxStrut::default();
    let style = unpositioned_float.node.Style();
    match unsafe { &*style.ShapeOutside() }.CssBox() {
        ShapeBox::kMarginBox => shape_insets -= new_margins,
        ShapeBox::kBorderBox => {}
        ShapeBox::kPaddingBox | ShapeBox::kContentBox => {
            let space = CreateConstraintSpaceForFloat(unpositioned_float, None, None);
            let mut strut = ComputeBorders(&space, &unpositioned_float.node);
            if unsafe { &*style.ShapeOutside() }.CssBox() == ShapeBox::kContentBox {
                strut += ComputePadding(&space, style);
            }
            shape_insets = strut
                .ConvertToPhysical(style.GetWritingDirection())
                .ConvertToLogical(foundation::WritingDirectionMode::new(
                    parent_space.GetWritingMode(),
                    TextDirection::kLtr,
                ));
        }
    }
    MakeGarbageCollected(ExclusionShapeData::new(
        layout_box,
        &new_margins,
        &shape_insets,
    ))
}

// cpp: layoutng_float/float_layout_algorithm.cc:147-169
fn CreateExclusionArea(
    fragment: &LogicalFragment,
    float_margin_bfc_offset: &BfcOffset,
    margins: &BoxStrut,
    unpositioned_float: &UnpositionedFloat,
    float_type: EFloat,
) -> *const ExclusionArea {
    let start_offset = *float_margin_bfc_offset;
    let end_offset = BfcOffset::new(
        start_offset.line_offset
            + (fragment.InlineSize() + margins.InlineSum()).ClampNegativeToZero(),
        start_offset.block_offset
            + (fragment.BlockSize() + margins.BlockSum()).ClampNegativeToZero(),
    );
    let shape_data = if !unsafe { &*unpositioned_float.node.GetLayoutBox() }
        .GetShapeOutsideInfo()
        .is_null()
    {
        CreateExclusionShapeData(margins, unpositioned_float)
    } else {
        ptr::null()
    };
    ExclusionArea::Create(
        &BfcRect::new(start_offset, end_offset),
        float_type,
        shape_data,
    )
}

// cpp: layoutng_float/float_layout_algorithm.cc:171-189
fn LayoutFloatWithoutFragmentation(unpositioned_float: &mut UnpositionedFloat) {
    if !unpositioned_float.layout_result.is_null() {
        return;
    }
    let space = CreateConstraintSpaceForFloat(unpositioned_float, None, None);
    unpositioned_float.layout_result =
        unpositioned_float
            .node
            .Layout(&space, unpositioned_float.token, ptr::null(), ptr::null());
    unpositioned_float.margins = ComputeMarginsFor(
        &space,
        unpositioned_float.node.Style(),
        unpositioned_float.parent_space,
    );
}

// cpp: layoutng_float/float_layout_algorithm.h:16-17
// cpp: layoutng_float/float_layout_algorithm.cc:193-210
pub fn ComputeMarginBoxInlineSizeForUnpositionedFloat(
    unpositioned_float: *mut UnpositionedFloat,
) -> LayoutUnit {
    debug_assert!(!unpositioned_float.is_null());
    let unpositioned_float = unsafe { &mut *unpositioned_float };
    LayoutFloatWithoutFragmentation(unpositioned_float);
    debug_assert!(!unpositioned_float.layout_result.is_null());
    let fragment = unsafe { &*unpositioned_float.layout_result }.GetPhysicalFragment();
    debug_assert!(fragment.GetBreakToken().is_null());
    let parent_space = unpositioned_float.parent_space;
    (LogicalFragment::new(parent_space.GetWritingDirection(), fragment).InlineSize()
        + unpositioned_float.margins.InlineSum())
    .ClampNegativeToZero()
}

// cpp: layoutng_float/float_layout_algorithm.h:19-20
// cpp: layoutng_float/float_layout_algorithm.cc:212-475
pub fn PositionFloat(
    unpositioned_float: *mut UnpositionedFloat,
    exclusion_space: *mut ExclusionSpace,
) -> PositionedFloat {
    debug_assert!(!unpositioned_float.is_null());
    let unpositioned_float = unsafe { &mut *unpositioned_float };
    let exclusion_space = unsafe { &mut *exclusion_space };
    let parent_space = unpositioned_float.parent_space;
    let node: BlockNode = unpositioned_float.node.clone();
    let is_same_writing_mode = node.Style().GetWritingMode() == parent_space.GetWritingMode();
    let is_fragmentable = is_same_writing_mode && parent_space.HasBlockFragmentation();

    let mut layout_result: *const LayoutResult = ptr::null();
    let mut fragment_margins = BoxStrut::default();
    let mut opportunity = LayoutOpportunity::default();
    let fragmentainer_block_size = unpositioned_float.fragmentainer_block_size;
    let mut need_break_before = false;

    // cpp: layoutng_float/float_layout_algorithm.cc:230-243
    if !is_fragmentable {
        LayoutFloatWithoutFragmentation(unpositioned_float);
        layout_result = unpositioned_float.layout_result;
        fragment_margins = unpositioned_float.margins;
        let float_fragment = LogicalFragment::new(
            parent_space.GetWritingDirection(),
            unsafe { &*layout_result }.GetPhysicalFragment(),
        );
        opportunity = FindLayoutOpportunityForFloat(
            unpositioned_float,
            exclusion_space,
            &fragment_margins,
            float_fragment.InlineSize(),
        );
    } else {
        // cpp: layoutng_float/float_layout_algorithm.cc:244-276
        fragment_margins = ComputeMarginsForInlineSize(
            node.Style(),
            unpositioned_float.percentage_size.inline_size,
            parent_space.GetWritingDirection(),
        );
        AdjustMarginsForFragmentation(unpositioned_float.token, &mut fragment_margins);
        let mut fragmentainer_delta: LayoutUnit;
        let mut optimistically_placed = false;
        if !unpositioned_float.layout_result.is_null() {
            let float_fragment = LogicalFragment::new(
                parent_space.GetWritingDirection(),
                unsafe { &*unpositioned_float.layout_result }.GetPhysicalFragment(),
            );
            opportunity = FindLayoutOpportunityForFloat(
                unpositioned_float,
                exclusion_space,
                &fragment_margins,
                float_fragment.InlineSize(),
            );
            fragmentainer_delta =
                opportunity.rect.start_offset.block_offset + fragment_margins.block_start;
        } else {
            fragmentainer_delta =
                unpositioned_float.origin_bfc_offset.block_offset + fragment_margins.block_start;
            optimistically_placed = true;
        }

        // cpp: layoutng_float/float_layout_algorithm.cc:278-317
        let mut is_at_fragmentainer_start;
        loop {
            let space = CreateConstraintSpaceForFloat(
                unpositioned_float,
                Some(fragmentainer_delta - parent_space.ExpectedBfcBlockOffset()),
                Some(fragment_margins),
            );
            is_at_fragmentainer_start = space.IsAtFragmentainerStart();
            layout_result = node.Layout(&space, unpositioned_float.token, ptr::null(), ptr::null());
            debug_assert_eq!(unsafe { &*layout_result }.Status(), EStatus::kSuccess);
            if !optimistically_placed {
                break;
            }
            let float_fragment = LogicalFragment::new(
                parent_space.GetWritingDirection(),
                unsafe { &*layout_result }.GetPhysicalFragment(),
            );
            opportunity = FindLayoutOpportunityForFloat(
                unpositioned_float,
                exclusion_space,
                &fragment_margins,
                float_fragment.InlineSize(),
            );
            let new_fragmentainer_delta =
                opportunity.rect.start_offset.block_offset + fragment_margins.block_start;
            debug_assert!(fragmentainer_delta <= new_fragmentainer_delta);
            if fragmentainer_delta < new_fragmentainer_delta {
                fragmentainer_delta = new_fragmentainer_delta;
                optimistically_placed = false;
                continue;
            }
            break;
        }

        // cpp: layoutng_float/float_layout_algorithm.cc:319-372
        if !is_at_fragmentainer_start {
            let fragmentainer_block_offset = unpositioned_float.FragmentainerOffsetAtBfc()
                + opportunity.rect.start_offset.block_offset
                + fragment_margins.block_start;
            let break_token = unsafe { &*layout_result }
                .GetPhysicalFragment()
                .GetBreakToken()
                .cast::<BlockBreakToken>();
            let is_at_block_end = break_token.is_null() || unsafe { &*break_token }.IsAtBlockEnd();
            if !is_at_block_end {
                fragment_margins.block_end = LayoutUnit::default();
            }
            if !MovePastBreakpointFull(
                parent_space,
                node.clone().into(),
                unsafe { &*layout_result },
                fragmentainer_block_offset,
                fragmentainer_block_size,
                kBreakAppealPerfect,
                ptr::null_mut(),
                false,
                ptr::null_mut(),
            ) {
                need_break_before = true;
            } else if is_at_block_end && parent_space.HasKnownFragmentainerBlockSize() {
                let float_fragment = LogicalFragment::new(
                    parent_space.GetWritingDirection(),
                    unsafe { &*layout_result }.GetPhysicalFragment(),
                );
                let outer_block_end = fragmentainer_block_offset
                    + float_fragment.BlockSize()
                    + fragment_margins.block_end;
                if outer_block_end > fragmentainer_block_size
                    && !IsBreakInside(unpositioned_float.token)
                {
                    need_break_before = true;
                }
            }
        }
    }

    // cpp: layoutng_float/float_layout_algorithm.cc:375-387
    let physical_fragment = unsafe {
        &*foundation::To::<PhysicalBoxFragment>(
            unsafe { &*layout_result }.GetPhysicalFragment() as *const _
        )
    };
    let float_fragment =
        LogicalFragment::new(parent_space.GetWritingDirection(), physical_fragment);
    let mut float_margin_bfc_offset = opportunity.rect.start_offset;
    if unpositioned_float.IsLineRight(parent_space.Direction()) {
        let float_margin_box_inline_size =
            float_fragment.InlineSize() + fragment_margins.InlineSum();
        float_margin_bfc_offset.line_offset +=
            opportunity.rect.InlineSize() - float_margin_box_inline_size;
    }

    // cpp: layoutng_float/float_layout_algorithm.cc:389-426
    if parent_space.HasBlockFragmentation()
        && !need_break_before
        && !IsBreakInside(unpositioned_float.token)
        && exclusion_space
            .NeedsBreakBeforeFloat(unpositioned_float.ClearType(parent_space.Direction()))
    {
        need_break_before = true;
    }
    let float_type = node.Style().FloatingWithDirection(parent_space.Direction());
    if need_break_before {
        let past_everything = BfcOffset::new(
            LayoutUnit::default(),
            unpositioned_float.FragmentainerSpaceLeft() + parent_space.ExpectedBfcBlockOffset(),
        );
        let exclusion = ExclusionArea::CreateWithoutShape(
            &BfcRect::new(past_everything, past_everything),
            float_type,
        );
        exclusion_space.Add(exclusion);
        exclusion_space.SetHasBreakBeforeFloat(float_type);
    } else {
        let exclusion = CreateExclusionArea(
            &float_fragment,
            &float_margin_bfc_offset,
            &fragment_margins,
            unpositioned_float,
            float_type,
        );
        exclusion_space.Add(exclusion);
        let break_token = physical_fragment.GetBreakToken();
        if !break_token.is_null() && !unsafe { &*break_token }.IsAtBlockEnd() {
            exclusion_space.SetHasBreakInsideFloat(float_type);
        }
    }

    // cpp: layoutng_float/float_layout_algorithm.cc:428-438
    let float_bfc_offset = BfcOffset::new(
        float_margin_bfc_offset.line_offset + fragment_margins.LineLeft(parent_space.Direction()),
        float_margin_bfc_offset.block_offset + fragment_margins.block_start,
    );
    let mut break_before_token: *const BlockBreakToken = ptr::null();
    if need_break_before {
        break_before_token = BlockBreakToken::CreateBreakBeforeAtDefaultOffset(node.into(), false);
    }

    // cpp: layoutng_float/float_layout_algorithm.cc:440-470
    let mut minimum_space_shortage = LayoutUnit::default();
    let mut tallest_unbreakable_block_size = LayoutUnit::default();
    if parent_space.HasBlockFragmentation()
        && parent_space.BlockFragmentationType() == FragmentationType::kFragmentColumn
    {
        let fragmentainer_block_offset =
            unpositioned_float.FragmentainerOffsetAtBfc() + float_bfc_offset.block_offset;
        if parent_space.HasKnownFragmentainerBlockSize() {
            if !break_before_token.is_null() || !physical_fragment.GetBreakToken().is_null() {
                minimum_space_shortage = CalculateSpaceShortage(
                    parent_space,
                    layout_result,
                    fragmentainer_block_offset,
                    fragmentainer_block_size,
                    None,
                );
            }
        } else if ShouldAvoidBreakInside(parent_space, unsafe { &*layout_result }) {
            debug_assert!(parent_space.IsInitialColumnBalancingPass());
            let margins = physical_fragment.Margins();
            let logical_margins = margins.ConvertToLogical(parent_space.GetWritingDirection());
            tallest_unbreakable_block_size = CalculateUnbreakableBlockSize(
                parent_space,
                unsafe { &*layout_result },
                fragmentainer_block_offset - logical_margins.block_start,
            );
        }
    }

    // cpp: layoutng_float/float_layout_algorithm.cc:472-475
    PositionedFloat::new(
        layout_result,
        break_before_token,
        &float_bfc_offset,
        tallest_unbreakable_block_size,
        minimum_space_shortage,
    )
}

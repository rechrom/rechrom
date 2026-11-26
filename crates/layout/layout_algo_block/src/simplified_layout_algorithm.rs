#![allow(non_snake_case)]

// Source: third_party/blink/renderer/core/layout/simplified_layout_algorithm.cc
// and simplified_layout_algorithm.h (Chromium
// 6c1d401fcca5e1b0030563a90c2f2bba168e0c15).
//
// "Simplified" layout re-uses the previous fragment's children and geometry,
// re-running layout only for children that themselves need simplified layout,
// and re-placing the OOF-positioned descendants (OutOfFlowLayoutPart). It is
// what BlockNode::Layout runs for LayoutCacheStatus::kNeedsSimplifiedLayout,
// for example when an absolutely positioned descendant changes size and its
// containing block is marked NeedsSimplifiedLayout by
// LayoutObject::MarkContainerChainForLayout.

use foundation::{kIndefiniteSize, LayoutUnit, MarginStrut, PhysicalSize, To, WritingDirectionMode};
use layout_base::block_break_token::BlockBreakToken;
use layout_base::box_fragment_builder::BoxFragmentBuilder;
use layout_base::break_token::BreakToken;
use layout_base::internal::block_node::BlockNode;
use layout_base::internal::layout_algorithm::{LayoutAlgorithm, LayoutAlgorithmParams};
use layout_base::internal::layout_box::LayoutBox;
use layout_base::internal::length_utils::ComputeBlockSizeForFragment;
use layout_base::layout_result::LayoutResult;
use layout_base::logical_fragment::LogicalFragment;
use layout_base::physical_box_fragment::PhysicalBoxFragment;
use layout_base::physical_fragment::PhysicalFragment;
use layout_base::physical_fragment_link::PhysicalFragmentLink;
use layout_geometry::geometry::logical_offset::LogicalOffset;
use layout_geometry::geometry::writing_mode_converter::WritingModeConverter;

// cpp: simplified_layout_algorithm.h:37-72
pub struct SimplifiedLayoutAlgorithm<'a> {
    base_: LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>,
    previous_result_: &'a LayoutResult,
    writing_direction_: WritingDirectionMode,
    previous_physical_container_size_: PhysicalSize,
}

impl<'a> SimplifiedLayoutAlgorithm<'a> {
    // cpp: simplified_layout_algorithm.cc:23-212
    pub fn new(
        params: &LayoutAlgorithmParams,
        result: &'a LayoutResult,
        keep_old_size: bool,
    ) -> Self {
        let mut base_ =
            LayoutAlgorithm::<BlockNode, BoxFragmentBuilder, BlockBreakToken>::from_params(params);
        let writing_direction_ = base_.Style().GetWritingDirection();
        debug_assert!(!base_.Node().IsReplaced());

        let is_block_flow = base_.Node().IsBlockFlow();
        let physical_fragment =
            unsafe { &*To::<PhysicalBoxFragment>(result.GetPhysicalFragment()) };

        let builder = &mut base_.container_builder_;
        builder.SetIsNewFormattingContext(physical_fragment.IsFormattingContextRoot());
        builder.SetIsFirstForNode(physical_fragment.IsFirstForNode());
        if physical_fragment.IsFragmentationContextRoot() {
            builder.SetIsBlockFragmentationContextRoot();
        }

        if keep_old_size {
            // When we're cloning a fragment to insert additional fragmentainers
            // to hold OOFs, re-use the old break token. This may not be the
            // last fragment.
            builder.PresetNextBreakToken(physical_fragment.GetBreakToken().cast::<BreakToken>());
        }

        if is_block_flow && !physical_fragment.IsFieldsetContainer() {
            let is_inline_formatting_context_root = base_
                .Node()
                .IsInlineFormattingContextRoot(std::ptr::null_mut());
            let is_table_cell = base_.GetConstraintSpace().IsTableCell();
            debug_assert!(
                base_.GetConstraintSpace().GetBfcOffset()
                    == result.GetConstraintSpaceForCaching().GetBfcOffset()
            );
            let builder = &mut base_.container_builder_;
            builder.SetIsInlineFormattingContext(is_inline_formatting_context_root);
            builder.SetStyleVariant(physical_fragment.GetStyleVariant());

            if result.SubtreeModifiedMarginStrut() {
                builder.SetSubtreeModifiedMarginStrut();
            }
            builder.SetEndMarginStrut(&result.EndMarginStrut());

            // Ensure that the parent layout hasn't asked us to move our BFC
            // position (checked above).
            builder.SetBfcLineOffset(result.BfcLineOffset());
            if let Some(bfc_block_offset) = result.BfcBlockOffset() {
                builder.SetBfcBlockOffset(bfc_block_offset);
            }

            // LayoutResult stores the optional lines-until-clamp as 0 when
            // unset (layout_result.h LinesUntilClamp).
            let lines_until_clamp = result.LinesUntilClamp();
            if lines_until_clamp != 0 {
                builder.SetLinesUntilClamp(Some(lines_until_clamp));
            }

            builder.SetExclusionSpace(result.GetExclusionSpace());

            if result.IsSelfCollapsing() {
                builder.SetIsSelfCollapsing();
            }
            if result.IsPushedByFloats() {
                builder.SetIsPushedByFloats();
            }
            builder.SetAdjoiningObjectTypes(result.GetAdjoiningObjectTypes());

            if is_table_cell {
                builder.SetHasCollapsedBorders(physical_fragment.HasCollapsedBorders());
                builder.SetTableCellColumnIndex(physical_fragment.TableCellColumnIndex());
            } else {
                debug_assert!(!physical_fragment.HasCollapsedBorders());
            }
        } else {
            // Only block-flow layout sets the following fields.
            debug_assert!(physical_fragment.IsFormattingContextRoot());
            debug_assert!(!result.SubtreeModifiedMarginStrut());
            debug_assert!(result.BfcLineOffset() == LayoutUnit::default());
            debug_assert!(result.BfcBlockOffset().unwrap_or_default() == LayoutUnit::default());
            debug_assert!(result.GetExclusionSpace().IsEmpty());
            debug_assert!(!result.IsSelfCollapsing());
            debug_assert!(!result.IsPushedByFloats());

            let builder = &mut base_.container_builder_;
            if physical_fragment.IsFieldsetContainer() {
                builder.SetIsFieldsetContainer();
            }
            if physical_fragment.IsMathMLFraction() {
                builder.SetIsMathMLFraction();
            }
            // SetCustomLayoutData(result.CustomLayoutData()): RunSimplifiedLayout
            // declines results that carry custom layout data (see below).
            debug_assert!(result.CustomLayoutData().is_null());
        }

        let builder = &mut base_.container_builder_;
        if physical_fragment.IsTable() {
            builder.SetTableColumnCount(result.TableColumnCount());
            builder.SetTableGridRect(physical_fragment.TableGridRect());
            builder.SetHasCollapsedBorders(physical_fragment.HasCollapsedBorders());

            let table_column_geometries = physical_fragment.TableColumnGeometries();
            if !table_column_geometries.is_null() {
                builder.SetTableColumnGeometries(unsafe { &*table_column_geometries }.to_vec());
            }
            let table_collapsed_borders = physical_fragment.TableCollapsedBorders();
            if !table_collapsed_borders.is_null() {
                builder.SetTableCollapsedBorders(unsafe { &*table_collapsed_borders });
            }
            let table_collapsed_borders_geometry =
                physical_fragment.TableCollapsedBordersGeometry();
            if !table_collapsed_borders_geometry.is_null() {
                builder.SetTableCollapsedBordersGeometry(Box::new(
                    unsafe { &*table_collapsed_borders_geometry }.clone(),
                ));
            }
        } else if physical_fragment.IsTableSection() {
            if let Some(section_start_row_index) = physical_fragment.TableSectionStartRowIndex() {
                let section_row_offsets = physical_fragment
                    .TableSectionRowOffsets()
                    .map_or_else(Vec::new, |offsets| offsets.to_vec());
                builder.SetTableSectionCollapsedBordersGeometry(
                    section_start_row_index,
                    section_row_offsets,
                );
            }
        }

        if physical_fragment.IsGrid() || physical_fragment.IsGridLanes() {
            builder.SetGridLayoutData(result.GetGridLayoutData());
        } else if physical_fragment.IsFrameSet() {
            let frame_set_layout_data = physical_fragment.GetFrameSetLayoutData();
            if !frame_set_layout_data.is_null() {
                builder.TransferFrameSetLayoutData(Box::new(
                    unsafe { &*frame_set_layout_data }.clone(),
                ));
            }
        }

        if physical_fragment.IsHiddenForPaint() {
            builder.SetIsHiddenForPaint(true);
        }

        let gap_geometry = physical_fragment.GetGapGeometry();
        if !gap_geometry.is_null() {
            builder.SetGapGeometry(gap_geometry);
        }

        if let Some(first_baseline) = physical_fragment.FirstBaseline() {
            builder.SetFirstBaseline(first_baseline);
        }
        if let Some(last_baseline) = physical_fragment.LastBaseline() {
            builder.SetLastBaseline(last_baseline);
        }
        if physical_fragment.UseLastBaselineForInlineBaseline() {
            builder.SetUseLastBaselineForInlineBaseline();
        }
        if physical_fragment.IsTablePart() {
            builder.SetIsTablePart();
        }

        let old_block_size = LogicalFragment::new(writing_direction_, physical_fragment).BlockSize();
        if keep_old_size {
            base_.container_builder_.SetFragmentBlockSize(old_block_size);
        } else {
            base_
                .container_builder_
                .SetIntrinsicBlockSize(result.IntrinsicBlockSize());

            // Only block-flow is allowed to change its block-size during
            // "simplified" layout, all other layout types must remain the same
            // size.
            if is_block_flow {
                let initial_inline_size = base_.container_builder_.InitialBorderBoxSize().inline_size;
                let new_block_size = ComputeBlockSizeForFragment(
                    base_.GetConstraintSpace(),
                    base_.Node(),
                    base_.BorderPadding(),
                    result.IntrinsicBlockSize(),
                    initial_inline_size,
                    kIndefiniteSize,
                );
                base_.container_builder_.SetFragmentBlockSize(new_block_size);
            } else {
                base_.container_builder_.SetFragmentBlockSize(old_block_size);
            }
        }

        Self {
            base_,
            previous_result_: result,
            writing_direction_,
            // We need the previous physical container size to calculate the
            // position of any child fragments.
            previous_physical_container_size_: physical_fragment.Size(),
        }
    }

    // cpp: simplified_layout_algorithm.cc:220-305
    pub fn Layout(&mut self) -> *const LayoutResult {
        // Since simplified layout's |Layout()| function deals with laying out
        // children, we can early out if we are display-locked.
        if self.base_.Node().ChildLayoutBlockedByDisplayLock() {
            return self.base_.container_builder_.ToBoxFragment();
        }

        let previous_result = self.previous_result_;
        let previous_fragment =
            unsafe { &*To::<PhysicalBoxFragment>(previous_result.GetPhysicalFragment()) };

        for child_link in previous_fragment.Children() {
            let child_fragment: &PhysicalFragment = child_link;

            // We'll add OOF-positioned candidates below.
            if child_fragment.IsOutOfFlowPositioned() {
                continue;
            }

            // We don't need to relayout list-markers, or line-box fragments.
            if child_fragment.IsListMarker() || child_fragment.IsLineBox() {
                self.AddChildFragment(child_link, child_fragment, None, false);
                continue;
            }

            // Add the (potentially updated) layout result.
            let result =
                BlockNode::new(To::<LayoutBox>(child_fragment.GetMutableLayoutObject()))
                    .SimplifiedLayout(child_fragment);

            // The child may have failed "simplified" layout! (Due to adding /
            // removing scrollbars). In this case we also return a nullptr,
            // indicating a full layout is required.
            if result.is_null() {
                return std::ptr::null();
            }
            let result = unsafe { &*result };

            let end_margin_strut = result.EndMarginStrut();
            // No margins should pierce outside formatting-context roots.
            debug_assert!(
                !result.GetPhysicalFragment().IsFormattingContextRoot()
                    || end_margin_strut.IsEmpty()
            );

            self.AddChildFragment(
                child_link,
                result.GetPhysicalFragment(),
                Some(&end_margin_strut),
                result.IsSelfCollapsing(),
            );
        }

        // Iterate through all our OOF-positioned children and add them as
        // candidates.
        let mut child = self.base_.Node().FirstChild();
        while !child.IsNull() {
            if child.IsOutOfFlowPositioned() {
                // TODO(ikilpatrick): Accessing the static-position from the
                // layer isn't ideal. We should save this on the physical
                // fragment which initially calculated it.
                let layer = unsafe { &*child.GetLayoutBox() }.Layer();
                let position = unsafe { &*layer }.GetStaticPosition();
                self.base_.container_builder_.AddOutOfFlowChildCandidate(
                    &BlockNode::from(child.clone()),
                    &position,
                    false,
                );
            }
            child = child.NextSibling();
        }

        if previous_fragment.HasItems() {
            // Simplified layout of fragments with items isn't supported. Give
            // up.
            return std::ptr::null();
        }

        // Some layout types (grid) manually calculate their inflow-bounds
        // rather than use the value determined inside the builder. Just
        // explicitly set this from the previous fragment for all types.
        if let Some(inflow_bounds) = previous_fragment.InflowBounds() {
            let inflow_bounds = WritingModeConverter::new(
                self.writing_direction_,
                self.previous_physical_container_size_,
            )
            .ToLogicalRect(inflow_bounds);
            self.base_.container_builder_.SetInflowBounds(inflow_bounds);
        }
        let builder = &mut self.base_.container_builder_;
        builder.SetHasAdjoiningObjectDescendants(previous_fragment.HasAdjoiningObjectDescendants());
        builder.SetMayHaveDescendantAboveBlockStart(
            previous_fragment.MayHaveDescendantAboveBlockStart(),
        );
        builder.SetHasDescendantThatDependsOnPercentageBlockSize(
            previous_result.HasDescendantThatDependsOnPercentageBlockSize(),
        );
        builder.SetInitialBreakBefore(previous_result.InitialBreakBefore());
        builder.SetPreviousBreakAfter(previous_result.FinalBreakAfter());

        builder.HandleOofsAndSpecialDescendants();

        builder.ToBoxFragment()
    }

    // cpp: simplified_layout_algorithm.cc:319-335
    fn AddChildFragment(
        &mut self,
        old_fragment: &PhysicalFragmentLink,
        new_fragment: &PhysicalFragment,
        margin_strut: Option<&MarginStrut>,
        is_self_collapsing: bool,
    ) {
        debug_assert!(old_fragment.Size() == new_fragment.Size());

        // Determine the previous position in the logical coordinate system.
        let child_offset = WritingModeConverter::new(
            self.writing_direction_,
            self.previous_physical_container_size_,
        )
        .ToLogicalOffset(old_fragment.Offset(), new_fragment.Size());
        // Any relative offset will have already been applied, avoid re-adding
        // one.
        let relative_offset = Some(LogicalOffset::default());

        // Add the new fragment to the builder.
        self.base_.container_builder_.AddChild(
            new_fragment,
            child_offset,
            margin_strut,
            is_self_collapsing,
            relative_offset,
            std::ptr::null(),
        );
    }
}

// cpp: block_node.cc:1491-1501 (BlockNode::RunSimplifiedLayout), installed as
// the registry's simplified-layout entry.
pub fn RunSimplifiedLayout(
    params: &LayoutAlgorithmParams,
    previous_result: &LayoutResult,
) -> *const LayoutResult {
    let previous_box_fragment =
        unsafe { &*To::<PhysicalBoxFragment>(previous_result.GetPhysicalFragment()) };
    // Chromium runs LayoutWithItemsBuilder for a fragment with items, which
    // re-runs the children's simplified layout and then always gives up
    // (SimplifiedLayoutAlgorithm::Layout returns nullptr when the previous
    // fragment has items). The caller then performs the full layout, which
    // lays out those same children. Decline directly; the outcome is the same
    // full layout.
    if previous_box_fragment.HasItems() {
        return std::ptr::null();
    }
    // CSS Layout API results carry a script value the builder retains by
    // reference count; this port's builder takes an owned reference that the
    // LayoutResult does not hand out. Such boxes take the full layout.
    if !previous_result.CustomLayoutData().is_null() {
        return std::ptr::null();
    }
    SimplifiedLayoutAlgorithm::new(params, previous_result, false).Layout()
}

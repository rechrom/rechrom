// Copyright 2017 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in ../../../third_party/LICENSE.chromium.
//
// Source: Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15,
// third_party/blink/renderer/core/layout/column_layout_algorithm.{cc,h}.
// This translation admits a single, non-nested line of columns. Unsupported
// source branches are rejected explicitly, never replaced by ordinary blocks.
use foundation::{
    kIndefiniteSize, EColumnFill, EColumnWrap, HeapVector, LayoutUnit, Length, Member,
    MinimumValueForLength, To, Traceable, UnsupportedLayout, Visitor,
};
use layoutng_assembly::{
    block_break_token::BlockBreakToken,
    box_fragment_builder::BoxFragmentBuilder,
    internal::{
        algorithm_entry::NativeAlgorithm,
        block_node::BlockNode,
        break_appeal::{kBreakAppealPerfect, BreakAppeal},
        constraint_space::{AutoSizeBehavior, ConstraintSpace, FragmentationType},
        constraint_space_builder::ConstraintSpaceBuilder,
        fragmentation_utils::{
            BlockSizeForFragmentation, CreateConstraintSpaceForFragmentainer,
            UpdateMinimalSpaceShortage,
        },
        layout_algorithm::{LayoutAlgorithm, LayoutAlgorithmParams},
        layout_input_node::MinMaxSizesFloatInput,
        layout_object::LayoutObject,
        length_utils::{
            CalculateInitialFragmentGeometry, ClampIntrinsicBlockSize, ComputeBlockSizeForFragment,
            ResolveColumnGapForMulticol, ResolveInitialMaxBlockLength,
            ResolveInitialMinBlockLength, ResolveMainBlockLength, ResolveUsedColumnCount,
            ShrinkLogicalSize,
        },
        min_max_sizes::{MinMaxSizes, MinMaxSizesResult},
        space_utils::SetTextBoxTrimOnChildSpaceBuilder,
    },
    layout_result::{EStatus, LayoutResult},
    logical_box_fragment::LogicalBoxFragment,
    logical_fragment::LogicalFragment,
    physical_box_fragment::PhysicalBoxFragment,
    physical_fragment::BoxType,
};
use layoutng_block::{
    block_layout_algorithm::BlockLayoutAlgorithm, block_layout_utils::AlignBlockContent,
};
use layoutng_geometry::geometry::{
    layout_unit_diffuser::LayoutUnitDiffuser, logical_offset::LogicalOffset,
    logical_size::LogicalSize,
};
use layoutng_style::style::computed_style::ComputedStyle;
use std::ops::{Deref, DerefMut};

fn unsupported(reason: &'static str) -> ! {
    std::panic::panic_any(UnsupportedLayout::new(reason))
}

// column_layout_algorithm.cc:229-244. Multiply before conversion to LayoutUnit,
// preserving the source's percentage-gap and subpixel rounding behavior.
fn CombinedColumnGapSize(style: &ComputedStyle, available: LayoutUnit, count: i32) -> LayoutUnit {
    if count < 1 {
        return LayoutUnit::default();
    }
    if let Some(gap) = style.ColumnGap() {
        return MinimumValueForLength(
            &gap.Multiplied(available.ToFloat(), count as f64),
            available,
        );
    }
    LayoutUnit::from_signed(style.GetFontDescription().ComputedPixelSize() * count)
}

#[repr(C)]
pub struct ColumnLayoutAlgorithm {
    base: LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>,
    used_column_count_: i32,
    combined_column_inline_size_: LayoutUnit,
    remaining_content_block_size_: LayoutUnit,
    inline_stride_: LayoutUnit,
    tallest_unbreakable_block_size_: LayoutUnit,
}
impl Deref for ColumnLayoutAlgorithm {
    type Target = LayoutAlgorithm<BlockNode, BoxFragmentBuilder, BlockBreakToken>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for ColumnLayoutAlgorithm {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl NativeAlgorithm for ColumnLayoutAlgorithm {
    fn new(params: &LayoutAlgorithmParams) -> Self {
        Self::new(params)
    }
    fn layout(&mut self) -> *const LayoutResult {
        self.Layout()
    }
    fn compute_min_max_sizes(&mut self, _: &MinMaxSizesFloatInput) -> MinMaxSizesResult {
        self.ComputeMinMaxSizes()
    }
}
impl ColumnLayoutAlgorithm {
    pub fn new(params: &LayoutAlgorithmParams) -> Self {
        let mut base = LayoutAlgorithm::from_params(params);
        base.container_builder_.SetInitialTextBoxTrim();
        Self {
            base,
            used_column_count_: 1,
            combined_column_inline_size_: LayoutUnit::default(),
            remaining_content_block_size_: kIndefiniteSize,
            inline_stride_: LayoutUnit::default(),
            tallest_unbreakable_block_size_: LayoutUnit::default(),
        }
    }

    // The remaining upstream branches need a part walker, spanner layout,
    // outer fragmentation, gap accumulator, and fragmented OOF collaborators.
    // Keep admission in the algorithm owner and use canonical ComputedStyle.
    fn CheckSupported(&self) {
        if self.GetConstraintSpace().HasBlockFragmentation() || !self.GetBreakToken().is_null() {
            unsupported("multicol: nested fragmentation is not translated");
        }
        if self.Style().ColumnWrap() == EColumnWrap::kWrap || !self.Style().HasAutoColumnHeight() {
            unsupported("multicol: column wrapping and column-height are not translated");
        }
        if self.Style().CanGeneratePseudoElement(
            layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdColumn,
        ) || self.Style().HasScrollMarkerGroupBefore()
            || self.Style().HasScrollMarkerGroupAfter()
        {
            unsupported("multicol: column pseudo-elements and scroll markers are not translated");
        }
        if self.Style().HasGapRule() {
            unsupported("multicol: gap decorations are not translated");
        }
        if self.GetConstraintSpace().IsTableCell() {
            unsupported("multicol: table-cell finalization is not translated");
        }
        if !self
            .Node()
            .ListMarkerBlockNodeIfListItem()
            .GetLayoutBox()
            .is_null()
        {
            unsupported("multicol: container list-marker placement is not translated");
        }
        fn CheckChildren(object: &LayoutObject) {
            let mut child = object.SlowFirstChild();
            while !child.is_null() {
                let node = unsafe { &*child };
                if node.IsColumnSpanAll() {
                    unsupported("multicol: column spanners are not translated");
                }
                CheckChildren(node);
                child = node.NextSibling();
            }
        }
        CheckChildren(unsafe { &*self.Node().GetLayoutBox() });
    }

    // column_layout_algorithm.cc:265-429, non-nested, no-spanner branch.
    pub fn Layout(&mut self) -> *const LayoutResult {
        self.CheckSupported();
        let border_box = *self.container_builder_.InitialBorderBoxSize();
        let available = self.ChildAvailableSize().inline_size;
        self.used_column_count_ = ResolveUsedColumnCount(self.Style(), available);
        let gaps = CombinedColumnGapSize(self.Style(), available, self.used_column_count_ - 1);
        self.combined_column_inline_size_ = (available - gaps).ClampNegativeToZero();
        self.inline_stride_ = self.combined_column_inline_size_
            + CombinedColumnGapSize(self.Style(), available, self.used_column_count_);
        self.remaining_content_block_size_ =
            ShrinkLogicalSize(border_box, self.BorderScrollbarPadding()).block_size;
        if self.remaining_content_block_size_ != kIndefiniteSize {
            self.remaining_content_block_size_ =
                self.remaining_content_block_size_.ClampNegativeToZero();
        }
        self.container_builder_.SetIsBlockFragmentationContextRoot();
        let line_offset = self.BorderScrollbarPadding().block_start;
        let column_height = self.LayoutLine(line_offset);
        self.container_builder_.SetHasSeenAllChildren();
        let unconstrained = line_offset + column_height + self.BorderScrollbarPadding().block_end;
        let intrinsic = ClampIntrinsicBlockSize(
            self.GetConstraintSpace(),
            self.Node(),
            self.GetBreakToken(),
            self.BorderScrollbarPadding(),
            unconstrained,
            None,
        );
        let block_size = ComputeBlockSizeForFragment(
            self.GetConstraintSpace(),
            self.Node(),
            self.BorderPadding(),
            intrinsic,
            border_box.inline_size,
            kIndefiniteSize,
        );
        self.container_builder_
            .SetFragmentsTotalBlockSize(block_size);
        self.container_builder_.SetIntrinsicBlockSize(intrinsic);
        #[cfg(debug_assertions)]
        self.container_builder_.CheckNoBlockFragmentation();
        let style = self.Style() as *const ComputedStyle;
        AlignBlockContent(
            unsafe { &*style },
            std::ptr::null(),
            unconstrained,
            &mut self.container_builder_,
        );
        self.container_builder_.HandleOofsAndSpecialDescendants();
        self.container_builder_.ToBoxFragment()
    }

    fn ColumnInlineSize(&self) -> LayoutUnit {
        self.combined_column_inline_size_ / self.used_column_count_
    }
    fn ColumnPercentageResolutionSize(&self) -> LogicalSize {
        LogicalSize {
            inline_size: self.ColumnInlineSize(),
            block_size: self.ChildAvailableSize().block_size,
        }
    }

    // column_layout_algorithm.cc:798-1331, one line with inline overflow.
    fn LayoutLine(&mut self, line_offset: LayoutUnit) -> LayoutUnit {
        let balance = self.Style().GetColumnFill() == EColumnFill::kBalance;
        let mut size = LogicalSize {
            inline_size: self.ColumnInlineSize(),
            block_size: self.remaining_content_block_size_,
        };
        if balance || size.block_size == kIndefiniteSize {
            size.block_size = self.ResolveColumnAutoBlockSize(size, balance);
        }
        let columns = loop {
            let mut columns = HeapVector::<ResultWithOffset>::default();
            let mut token: *const BlockBreakToken = std::ptr::null();
            let mut violating_break = false;
            let mut forced_break_count = 0;
            let mut shortage = kIndefiniteSize;
            let mut inline_offset = self.BorderScrollbarPadding().inline_start;
            let mut diffuser =
                LayoutUnitDiffuser::new(self.inline_stride_, self.used_column_count_ as u32);
            loop {
                let space = CreateConstraintSpaceForFragmentainer(
                    self.GetConstraintSpace(),
                    FragmentationType::kFragmentColumn,
                    size,
                    self.ColumnPercentageResolutionSize(),
                    balance,
                    BreakAppeal::kBreakAppealLastResort,
                    &self.container_builder_,
                );
                let geometry = CalculateInitialFragmentGeometry(
                    &space,
                    self.Node(),
                    self.GetBreakToken(),
                    false,
                );
                let mut params = LayoutAlgorithmParams::new(self.Node().clone(), &geometry, &space);
                params.break_token = token;
                let mut algorithm = BlockLayoutAlgorithm::new(&params);
                algorithm.SetBoxType(BoxType::kColumnBox);
                let result = algorithm.Layout();
                let result_ref = unsafe { &*result };
                self.CheckColumnResult(result_ref);
                let column = unsafe {
                    &*To::<PhysicalBoxFragment>(result_ref.GetPhysicalFragment() as *const _)
                };
                columns.push(ResultWithOffset {
                    result: Member::from_ptr(result as *mut _),
                    offset: LogicalOffset {
                        inline_offset,
                        block_offset: line_offset,
                    },
                });
                UpdateMinimalSpaceShortage(result_ref.MinimalSpaceShortage(), &mut shortage);
                violating_break |= result_ref.GetBreakAppeal() != kBreakAppealPerfect;
                inline_offset += diffuser.Next();
                if columns.len() % self.used_column_count_ as usize == 0 {
                    diffuser = LayoutUnitDiffuser::new(
                        self.inline_stride_,
                        self.used_column_count_ as u32,
                    );
                }
                if result_ref.HasForcedBreak() {
                    forced_break_count += 1;
                }
                token = column.GetBreakToken();
                if token.is_null() {
                    break;
                }
            }
            if !balance
                || (!violating_break && columns.len() <= self.used_column_count_ as usize)
                || self.used_column_count_ <= forced_break_count + 1
            {
                break columns;
            }
            let mut new_size = size.block_size;
            if shortage > LayoutUnit::default() {
                new_size += shortage;
            }
            new_size = self.ConstrainColumnBlockSize(new_size);
            if new_size <= size.block_size {
                break columns;
            }
            size.block_size = new_size;
        };
        for entry in columns.iter() {
            let result = entry.result.Get();
            let offset = entry.offset;
            let column = unsafe {
                &*To::<PhysicalBoxFragment>((&*result).GetPhysicalFragment() as *const _)
            };
            self.container_builder_
                .AddChild(column, offset, None, false, None, std::ptr::null());
            self.PropagateBaselineFromChild(column, offset.block_offset);
        }
        size.block_size
    }

    fn CheckColumnResult(&self, result: &LayoutResult) {
        assert_eq!(
            result.Status(),
            EStatus::kSuccess,
            "column block algorithm must not abort"
        );
        if !result.GetColumnSpannerPath().is_null() {
            unsupported("multicol: column spanners are not translated");
        }
        // FragmentBuilder::AddChild propagates fragmentainer descendants from
        // each column into the multicol builder. HandleOofsAndSpecialDescendants
        // then runs the translated OutOfFlowLayoutPart after the final multicol
        // size is known, matching the source's deferred fragmented-OOF path.
    }

    // column_layout_algorithm.cc:1495-1517.
    fn PropagateBaselineFromChild(&mut self, child: &PhysicalBoxFragment, offset: LayoutUnit) {
        let fragment =
            LogicalBoxFragment::new(self.GetConstraintSpace().GetWritingDirection(), child);
        if let Some(baseline) = fragment.FirstBaseline() {
            let value = (offset + baseline).min(
                self.container_builder_
                    .FirstBaseline()
                    .unwrap_or(LayoutUnit::Max()),
            );
            self.container_builder_.SetFirstBaseline(value);
        }
        if let Some(baseline) = fragment.LastBaseline() {
            let value = (offset + baseline).max(
                self.container_builder_
                    .LastBaseline()
                    .unwrap_or(LayoutUnit::Min()),
            );
            self.container_builder_.SetLastBaseline(value);
        }
        self.container_builder_
            .SetUseLastBaselineForInlineBaseline();
    }

    // column_layout_algorithm.cc:1531-1730. First measure forced-break content
    // runs without soft breaks, distribute implicit breaks, then let LayoutLine
    // stretch by the block algorithm's actual minimum shortage.
    fn ResolveColumnAutoBlockSize(&mut self, size: LogicalSize, balance: bool) -> LayoutUnit {
        let space = self.CreateConstraintSpaceForBalancing(size);
        let geometry =
            CalculateInitialFragmentGeometry(&space, self.Node(), self.GetBreakToken(), false);
        let mut token = std::ptr::null();
        let mut runs = Vec::new();
        let mut tallest_content = LayoutUnit::default();
        self.tallest_unbreakable_block_size_ = LayoutUnit::default();
        loop {
            let mut params = LayoutAlgorithmParams::new(self.Node().clone(), &geometry, &space);
            params.break_token = token;
            let mut algorithm = BlockLayoutAlgorithm::new(&params);
            algorithm.SetBoxType(BoxType::kColumnBox);
            let result = unsafe { &*algorithm.Layout() };
            self.CheckColumnResult(result);
            let fragment =
                unsafe { &*To::<PhysicalBoxFragment>(result.GetPhysicalFragment() as *const _) };
            let logical =
                LogicalFragment::new(self.GetConstraintSpace().GetWritingDirection(), fragment);
            let height =
                BlockSizeForFragmentation(result, self.GetConstraintSpace().GetWritingDirection())
                    .max(logical.BlockSize());
            runs.push(ContentRun {
                size: height,
                breaks: 0,
            });
            tallest_content = tallest_content.max(height);
            self.tallest_unbreakable_block_size_ = self
                .tallest_unbreakable_block_size_
                .max(result.TallestUnbreakableBlockSize());
            token = fragment.GetBreakToken();
            if token.is_null() {
                break;
            }
        }
        if self.tallest_unbreakable_block_size_ >= tallest_content {
            return self.ConstrainColumnBlockSize(self.tallest_unbreakable_block_size_);
        }
        if balance {
            for _ in runs.len()..self.used_column_count_ as usize {
                // max_element retains the first of equal maxima.
                let mut tallest = 0;
                for i in 1..runs.len() {
                    if runs[i].ColumnBlockSize() > runs[tallest].ColumnBlockSize() {
                        tallest = i;
                    }
                }
                runs[tallest].breaks += 1;
            }
        }
        self.ConstrainColumnBlockSize(runs.iter().map(ContentRun::ColumnBlockSize).max().unwrap())
    }

    // column_layout_algorithm.cc:1735-1820, unconstrained outer context.
    fn ConstrainColumnBlockSize(&self, size: LayoutUnit) -> LayoutUnit {
        let extra = self.BorderScrollbarPadding().BlockSum();
        let size = size.max(self.tallest_unbreakable_block_size_) + extra;
        let space = self.GetConstraintSpace();
        let style = self.Style();
        let mut max = ResolveInitialMaxBlockLength(
            space,
            style,
            self.BorderPadding(),
            style.LogicalMaxHeight(),
            kIndefiniteSize,
        );
        let auto = if space.IsBlockAutoBehaviorStretch() {
            Length::Stretch()
        } else {
            Length::FitContent()
        };
        let extent = ResolveMainBlockLength(
            space,
            style,
            self.BorderPadding(),
            style.LogicalHeight(),
            Some(&auto),
            &|_| kIndefiniteSize,
            kIndefiniteSize,
        );
        if extent != kIndefiniteSize {
            max = max.min(extent);
        }
        max = max.max(ResolveInitialMinBlockLength(
            space,
            style,
            self.BorderPadding(),
            style.LogicalMinHeight(),
            kIndefiniteSize,
        ));
        let mut size = (size.min(max) - extra).ClampNegativeToZero();
        if self.remaining_content_block_size_ > LayoutUnit::default() {
            size = size.min(self.remaining_content_block_size_);
        }
        size
    }

    // column_layout_algorithm.cc:1843-1862.
    fn CreateConstraintSpaceForBalancing(&self, size: LogicalSize) -> ConstraintSpace {
        let mut builder = ConstraintSpaceBuilder::new(
            self.GetConstraintSpace(),
            self.Style().GetWritingDirection(),
            true,
        );
        builder.SetFragmentationType(FragmentationType::kFragmentColumn);
        builder.SetShouldPropagateChildBreakValues(true);
        builder.SetAvailableSize(LogicalSize {
            inline_size: size.inline_size,
            block_size: kIndefiniteSize,
        });
        builder.SetInlineAutoBehavior(AutoSizeBehavior::kStretchImplicit);
        builder.SetPercentageResolutionSize(self.ColumnPercentageResolutionSize());
        builder.SetIsAnonymous(true);
        builder.SetIsInColumnBfc();
        builder.SetIsInsideBalancedColumns();
        if self.container_builder_.ShouldTextBoxTrim() {
            SetTextBoxTrimOnChildSpaceBuilder(&self.container_builder_, &mut builder);
        }
        builder.ToConstraintSpace()
    }

    // column_layout_algorithm.cc:432-497, with spanners excluded at admission.
    pub fn ComputeMinMaxSizes(&mut self) -> MinMaxSizesResult {
        self.CheckSupported();
        let override_size = self.Node().OverrideIntrinsicContentInlineSize();
        if override_size != kIndefiniteSize {
            let size = self.BorderScrollbarPadding().InlineSum() + override_size;
            return MinMaxSizesResult {
                sizes: MinMaxSizes {
                    min_size: size,
                    max_size: size,
                },
                depends_on_block_constraints: false,
                ..Default::default()
            };
        }
        let mut builder = ConstraintSpaceBuilder::new(
            self.GetConstraintSpace(),
            self.Style().GetWritingDirection(),
            true,
        );
        builder.SetIsAnonymous(true);
        builder.SetIsInColumnBfc();
        let space = builder.ToConstraintSpace();
        let geometry =
            CalculateInitialFragmentGeometry(&space, self.Node(), std::ptr::null(), true);
        let params = LayoutAlgorithmParams::new(self.Node().clone(), &geometry, &space);
        let mut algorithm = BlockLayoutAlgorithm::new(&params);
        let mut result = algorithm.ComputeMinMaxSizes(&MinMaxSizesFloatInput::default());
        if !self.Style().HasAutoColumnWidth() {
            let width = LayoutUnit::from_f32(self.Style().ColumnWidth());
            result.sizes.min_size = result.sizes.min_size.min(width);
            result.sizes.max_size = result.sizes.max_size.max(width).max(result.sizes.min_size);
        }
        let count = i32::from(self.Style().ColumnCount());
        let extra = ResolveColumnGapForMulticol(self.Style(), LayoutUnit::default()) * (count - 1);
        if self.Style().HasAutoColumnWidth() {
            result.sizes.min_size = result.sizes.min_size * count + extra;
        }
        result.sizes.max_size = result.sizes.max_size * count + extra;
        result.sizes += self.BorderScrollbarPadding().InlineSum();
        result
    }
}
struct ContentRun {
    size: LayoutUnit,
    breaks: i32,
}
impl ContentRun {
    fn ColumnBlockSize(&self) -> LayoutUnit {
        LayoutUnit::FromFloatCeil(self.size.ToFloat() / (self.breaks + 1) as f32)
    }
}

// Same traced, temporary ownership as Chromium's HeapVector<ResultWithOffset>.
struct ResultWithOffset {
    result: Member<LayoutResult>,
    offset: LogicalOffset,
}
impl Traceable for ResultWithOffset {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        visitor.Trace(&self.result);
    }
}

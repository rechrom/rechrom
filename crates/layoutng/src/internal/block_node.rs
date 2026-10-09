#![allow(non_snake_case)]

#[cfg(debug_assertions)]
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(debug_assertions)]
use std::sync::{Mutex, OnceLock};

use super::algorithm_forward::CSSLayoutDefinition;
use foundation::{
    kIndefiniteSize, DynamicTo, EAspectRatioType, EBoxSizing, EDisplay, IsParallelWritingMode,
    LayoutUnit, MinimumValueForLength, PhysicalRect, PhysicalSize, StrCat, String as BlinkString,
    TextDirection, To, UnsupportedLayout, WritingDirectionMode, WritingMode,
};
use layoutng_fragment_tree::block_break_token::BlockBreakToken;
use layoutng_fragment_tree::layout_result::EStatus;
use layoutng_fragment_tree::layout_result::LayoutResult;
use layoutng_fragment_tree::logical_fragment::LogicalFragment;
use layoutng_fragment_tree::physical_box_fragment::PhysicalBoxFragment;
use layoutng_fragment_tree::physical_fragment::PhysicalFragment;
use layoutng_geometry::geometry::box_strut::BoxStrut;
use layoutng_geometry::geometry::fragment_geometry::FragmentGeometry;
use layoutng_geometry::geometry::logical_size::{LogicalSize, ToLogicalSize};
use layoutng_geometry::geometry::overflow_clip_axes::{
    kNoOverflowClip, kOverflowClipX, kOverflowClipY,
};
use layoutng_geometry::geometry::physical_rect::PhysicalRectExt;
use layoutng_mathml::mathml_node_metadata::MathScriptType;
use layoutng_style::style::computed_style::ComputedStyle;

use super::column_spanner_path::ColumnSpannerPath;
use super::constraint_space::{BaselineAlgorithmType, ConstraintSpace, LayoutResultCacheSlot};
use super::constraint_space_builder::ConstraintSpaceBuilder;
use super::disable_layout_side_effects_scope::DisableLayoutSideEffectsScope;
use super::early_break::EarlyBreak;
use super::form_control_sizing_service::ComputedStyleControlSizingExt;
use super::form_control_types::FormControlType;
use super::form_node_metadata::{HTMLInputElement, HTMLSelectElement};
use super::fragmentation_utils::{FragmentIndex, InvolvedInBlockFragmentation, IsBreakInside};
use super::inline_node::InlineNode;
use super::layout_algorithm::LayoutAlgorithmParams;
use super::layout_algorithm_set::LayoutAlgorithmEntry;
use super::layout_block::LayoutBlock;
use super::layout_block_flow::LayoutBlockFlow;
use super::layout_box::LayoutBox;
use super::layout_box_utils::UpdateChildLayoutBoxLocations;
use super::layout_input::NodeKind;
use super::layout_input_node::{LayoutInputNode, LayoutInputNodeType, MinMaxSizesFloatInput};
use super::layout_invalidation_reason;
use super::layout_node_metadata::{Element, Node};
use super::layout_object::{LayoutObject, MarkingBehavior};
use super::layout_pass_scope::LayoutPassScope;
use super::layout_utils::LayoutCacheStatus;
use super::length_utils::{
    AddScrollbarFreeze, CalculateInitialFragmentGeometry, ComputeMinMaxInlineSizesFromAspectRatio,
    ComputePhysicalMarginsForSpace, ComputeScrollbars, InlineSizeFromAspectRatio, SizeType,
};
use super::min_max_sizes::{MinMaxSizes, MinMaxSizesResult};
use super::scroll_layout_scope::{DelayScrollOffsetClampScope, FreezeScrollbarsRootScope};
use super::space_utils::SetOrthogonalFallbackInlineSizeIfNeeded;

// cpp: layoutng/internal/block_node.h:39-44
// C++ BlockNode adds no data to LayoutInputNode; preserve its base at offset zero.
#[repr(C)]
pub struct BlockNode {
    pub base: LayoutInputNode,
}

// cpp: layoutng/internal/block_node.h:215-216
impl PartialEq for BlockNode {
    fn eq(&self, other: &Self) -> bool {
        self.GetLayoutBox() == other.GetLayoutBox()
    }
}

impl Eq for BlockNode {}

// cpp: layoutng/internal/block_node.h:256-261
impl foundation::DowncastFrom<LayoutInputNode> for BlockNode {
    fn AllowFrom(node: &LayoutInputNode) -> bool {
        node.IsBlock()
    }
}

const _: () = assert!(std::mem::offset_of!(BlockNode, base) == 0);

// Rust cannot add inherent methods to BlockNode from another package. The
// original out-of-package member definitions connect through these typed
// providers when the corresponding packages are assembled.
unsafe extern "Rust" {
    fn BlockNodeGetFieldsetContentFromForms(node: &BlockNode) -> BlockNode;
    fn BlockNodeIsCustomLayoutLoadedFromCustom(node: &BlockNode) -> bool;
    fn BlockNodeGetCustomLayoutDefinitionFromCustom(node: &BlockNode) -> *mut CSSLayoutDefinition;
    fn BlockNodeScriptTypeFromMathML(node: &BlockNode) -> MathScriptType;
    fn BlockNodeHasIndexFromMathML(node: &BlockNode) -> bool;
    fn BlockNodeFinishPageContainerLayoutFromPaged(node: &BlockNode, result: *const LayoutResult);
}

#[cfg(debug_assertions)]
struct ScrollbarChangeDebugGuard(usize);

#[cfg(debug_assertions)]
impl ScrollbarChangeDebugGuard {
    fn changed() -> &'static Mutex<HashSet<usize>> {
        static CHANGED: OnceLock<Mutex<HashSet<usize>>> = OnceLock::new();
        CHANGED.get_or_init(|| Mutex::new(HashSet::new()))
    }

    fn new(box_: *mut LayoutBox) -> Self {
        let identity = box_ as usize;
        assert!(Self::changed().lock().unwrap().insert(identity));
        Self(identity)
    }
}

#[cfg(debug_assertions)]
impl Drop for ScrollbarChangeDebugGuard {
    fn drop(&mut self) {
        Self::changed().lock().unwrap().remove(&self.0);
    }
}

impl Clone for BlockNode {
    fn clone(&self) -> Self {
        Self::new(self.GetLayoutBox())
    }
}

// cpp: layoutng/internal/block_node.h:257-259
// C++ To<BlockNode>(LayoutInputNode) copies the tagged base after a checked
// downcast. Keep the base value and reject an inline node.
impl From<LayoutInputNode> for BlockNode {
    fn from(node: LayoutInputNode) -> Self {
        assert!(node.IsBlock());
        Self { base: node }
    }
}

// cpp: layoutng/internal/block_node.h:39-45
impl From<BlockNode> for LayoutInputNode {
    fn from(node: BlockNode) -> Self {
        node.base
    }
}

impl Deref for BlockNode {
    type Target = LayoutInputNode;
    fn deref(&self) -> &LayoutInputNode {
        &self.base
    }
}

impl DerefMut for BlockNode {
    fn deref_mut(&mut self) -> &mut LayoutInputNode {
        &mut self.base
    }
}

impl BlockNode {
    // cpp: layoutng/internal/block_node.h:43-46
    pub fn new(box_: *mut LayoutBox) -> Self {
        Self {
            base: LayoutInputNode::Create(box_, LayoutInputNodeType::kBlock),
        }
    }

    pub fn null() -> Self {
        Self::new(std::ptr::null_mut())
    }

    // cpp: layoutng/internal/block_node.h:96
    pub fn NextSibling(&self) -> LayoutInputNode {
        self.NextBlockSibling().base
    }

    // cpp: layoutng/internal/block_node.h:215-218
    pub fn EqualsBlockNode(&self, other: &BlockNode) -> bool {
        self.GetLayoutBox() == other.GetLayoutBox()
    }

    pub fn EqualsLayoutInputNode(&self, other: &LayoutInputNode) -> bool {
        other.Type() == LayoutInputNodeType::kBlock && self.GetLayoutBox() == other.GetLayoutBox()
    }

    // cpp: layoutng/internal/block_node.h:255-258
    pub fn AllowFrom(node: &LayoutInputNode) -> bool {
        node.IsBlock()
    }

    // cpp: layoutng/internal/block_node.h:125
    // Body: //src/layoutng_forms/layout_fieldset.cc.
    pub fn GetFieldsetContent(&self) -> BlockNode {
        unsafe { BlockNodeGetFieldsetContentFromForms(self) }
    }

    // cpp: layoutng/internal/block_node.h:177-178
    // Bodies: //src/layoutng_custom/layout_custom.cc.
    pub fn IsCustomLayoutLoaded(&self) -> bool {
        unsafe { BlockNodeIsCustomLayoutLoadedFromCustom(self) }
    }

    pub fn GetCustomLayoutDefinition(&self) -> *mut CSSLayoutDefinition {
        unsafe { BlockNodeGetCustomLayoutDefinitionFromCustom(self) }
    }

    // cpp: layoutng/internal/block_node.h:189,192
    // Bodies: //src/layoutng_mathml/mathml_node_metadata.cc.
    pub fn ScriptType(&self) -> MathScriptType {
        unsafe { BlockNodeScriptTypeFromMathML(self) }
    }

    pub fn HasIndex(&self) -> bool {
        unsafe { BlockNodeHasIndexFromMathML(self) }
    }

    // cpp: layoutng/internal/block_node.h:213
    // Body: //src/layoutng_paged/paginated_root_layout_algorithm_core.cc.
    pub fn FinishPageContainerLayout(&self, result: *const LayoutResult) {
        unsafe { BlockNodeFinishPageContainerLayoutFromPaged(self, result) }
    }

    // cpp: layoutng/internal/block_node.cc:250-482
    pub fn Layout(
        &self,
        constraint_space: &ConstraintSpace,
        break_token: *const BlockBreakToken,
        early_break: *const EarlyBreak,
        column_spanner_path: *const ColumnSpannerPath,
    ) -> *const LayoutResult {
        let box_ = self.GetLayoutBox();
        let mut root_trace = if browser_tracing::enabled()
            && unsafe { &*(box_ as *mut LayoutObject) }.Parent().is_null()
        {
            Some(browser_tracing::span("layout", "BlockNode.RootLayout"))
        } else {
            None
        };
        let previous_result = unsafe { &mut *box_ }.GetCachedLayoutResult(break_token);
        if !previous_result.is_null() {
            constraint_space.GetExclusionSpace().PreInitialize(
                unsafe { &*previous_result }
                    .GetConstraintSpaceForCaching()
                    .GetExclusionSpace(),
            );
        }

        let mut cache_status = LayoutCacheStatus::kNeedsLayout;
        let mut fragment_geometry: Option<FragmentGeometry> = None;
        let needed_layout = unsafe { &*box_ }.NeedsLayout();
        let mut layout_result = unsafe { &mut *box_ }.CachedLayoutResult(
            constraint_space,
            break_token,
            early_break,
            column_spanner_path,
            &mut fragment_geometry,
            &mut cache_status,
        );
        if let Some(trace) = root_trace.as_mut() {
            trace.set("needed_layout", needed_layout as u8 as f64);
            trace.set("cache_status", cache_status as i32 as f64);
            trace.set(
                "returned_previous_result",
                (layout_result == previous_result) as u8 as f64,
            );
        }

        if matches!(
            cache_status,
            LayoutCacheStatus::kHit | LayoutCacheStatus::kNeedsSimplifiedLayout
        ) && needed_layout
            && constraint_space.CacheSlot() == LayoutResultCacheSlot::kLayout
            && unsafe { &*box_ }.HasBrokenSpine()
            && !self.ChildLayoutBlockedByDisplayLock()
        {
            layout_result = LayoutResult::CloneWithPostLayoutFragments(unsafe { &*layout_result });
            let new_fragment = unsafe {
                &*To::<PhysicalBoxFragment>(
                    unsafe { &*layout_result }.GetPhysicalFragment() as *const _
                )
            };
            let clear_trailing_results =
                !new_fragment.GetBreakToken().is_null() && new_fragment.HasItems();
            self.StoreResultInLayoutBox(layout_result, break_token, clear_trailing_results);
            unsafe { &mut *box_ }.ClearHasBrokenSpine();
        }

        if cache_status == LayoutCacheStatus::kHit {
            debug_assert!(!layout_result.is_null());
            self.UpdateMarginPaddingInfoIfNeeded(
                constraint_space,
                unsafe { &*layout_result }.GetPhysicalFragment(),
            );
            self.UpdateShapeOutsideInfoIfNeeded(unsafe { &*layout_result }, constraint_space);
            if !unsafe { &*box_ }.NeedsLayout() {
                return layout_result;
            }
        }

        if fragment_geometry.is_none() {
            fragment_geometry = Some(CalculateInitialFragmentGeometry(
                constraint_space,
                self,
                break_token,
                false,
            ));
        }
        self.PrepareForLayout();
        let mut params = LayoutAlgorithmParams::new(
            self.clone(),
            fragment_geometry.as_ref().expect("fragment geometry"),
            constraint_space,
        );
        params.break_token = break_token;
        params.early_break = early_break;
        params.column_spanner_path = column_spanner_path;
        let block_flow = DynamicTo::<LayoutBlockFlow>(self.GetLayoutBox());

        if cache_status == LayoutCacheStatus::kNeedsSimplifiedLayout
            && (block_flow.is_null() || !unsafe { &*block_flow }.IsFragmentationContextRoot())
        {
            debug_assert!(!layout_result.is_null());
            #[cfg(debug_assertions)]
            let previous_result = layout_result;
            layout_result = self.RunSimplifiedLayout(&params, unsafe { &*layout_result });
            #[cfg(debug_assertions)]
            if !layout_result.is_null() {
                unsafe { &*layout_result }
                    .CheckSameForSimplifiedLayout(unsafe { &*previous_result }, true);
            }
        } else if cache_status == LayoutCacheStatus::kCanReuseLines {
            params.previous_result = layout_result;
            layout_result = std::ptr::null();
        } else {
            layout_result = std::ptr::null();
        }

        let mut scrollbars_before = ComputeScrollbars(constraint_space, self);
        let inline_size_before = fragment_geometry
            .as_ref()
            .expect("fragment geometry")
            .border_box_size
            .inline_size;
        let intrinsic_logical_widths_dirty_before = unsafe { &*box_ }.IntrinsicLogicalWidthsDirty();
        if layout_result.is_null() {
            layout_result = LayoutWithAlgorithm(&params);
        }
        let _delay_clamp_scope = DelayScrollOffsetClampScope::new();
        let optional_old_box_size = if unsafe { &*layout_result }.Status() == EStatus::kSuccess
            && unsafe { &*layout_result }
                .GetPhysicalFragment()
                .GetBreakToken()
                .is_null()
        {
            Some(unsafe { &*box_ }.StitchedSize())
        } else {
            None
        };
        self.FinishLayout(
            block_flow,
            constraint_space,
            break_token,
            layout_result,
            optional_old_box_size,
        );

        if !intrinsic_logical_widths_dirty_before && unsafe { &*box_ }.IntrinsicLogicalWidthsDirty()
        {
            *fragment_geometry.as_mut().expect("fragment geometry") =
                CalculateInitialFragmentGeometry(constraint_space, self, break_token, false);
        }

        let mut scrollbars_after = ComputeScrollbars(constraint_space, self);
        if (scrollbars_before != scrollbars_after
            || inline_size_before
                != fragment_geometry
                    .as_ref()
                    .expect("fragment geometry")
                    .border_box_size
                    .inline_size)
            && !DisableLayoutSideEffectsScope::IsDisabled()
            && !IsBreakInside(break_token)
        {
            let mut freeze_horizontal = false;
            let mut freeze_vertical = false;
            if constraint_space.CacheSlot() == LayoutResultCacheSlot::kMeasure {
                freeze_horizontal = true;
                freeze_vertical = true;
            }
            loop {
                AddScrollbarFreeze(
                    &scrollbars_before,
                    &scrollbars_after,
                    constraint_space.GetWritingDirection(),
                    &mut freeze_horizontal,
                    &mut freeze_vertical,
                );
                scrollbars_before = scrollbars_after;
                let _freezer = FreezeScrollbarsRootScope::from_box(
                    unsafe { &*box_ },
                    freeze_horizontal,
                    freeze_vertical,
                );
                let old_box_size = unsafe { &*box_ }.StitchedSize();
                params.previous_result = std::ptr::null();
                unsafe { &mut *box_ }.SetShouldSkipLayoutCache(true);
                #[cfg(debug_assertions)]
                let _debug_guard = ScrollbarChangeDebugGuard::new(self.GetLayoutBox());
                unsafe { &mut *box_ }.SetNeedsLayoutWithMarking(
                    unsafe { &layout_invalidation_reason::kScrollbarChanged },
                    MarkingBehavior::kMarkOnlyThis,
                );
                *fragment_geometry.as_mut().expect("fragment geometry") =
                    CalculateInitialFragmentGeometry(constraint_space, self, break_token, false);
                layout_result = LayoutWithAlgorithm(&params);
                self.FinishLayout(
                    block_flow,
                    constraint_space,
                    break_token,
                    layout_result,
                    Some(old_box_size),
                );
                #[cfg(debug_assertions)]
                drop(_debug_guard);
                scrollbars_after = ComputeScrollbars(constraint_space, self);
                debug_assert!(
                    !freeze_horizontal || !freeze_vertical || scrollbars_after == scrollbars_before
                );
                if scrollbars_after == scrollbars_before {
                    break;
                }
            }
        }
        self.UpdateShapeOutsideInfoIfNeeded(unsafe { &*layout_result }, constraint_space);
        layout_result
    }

    // cpp: layoutng/internal/block_node.cc:662-841
    pub fn ComputeMinMaxSizes(
        &self,
        container_writing_mode: WritingMode,
        type_: SizeType,
        constraint_space: &ConstraintSpace,
        float_input: MinMaxSizesFloatInput,
    ) -> MinMaxSizesResult {
        let box_ = self.GetLayoutBox();
        if self.IsListItem() {
            UpdateListMarkerText(unsafe { &mut *self.GetLayoutBox().cast::<LayoutObject>() });
        }

        let mut cached_fragment_geometry: Option<FragmentGeometry> = None;
        let is_orthogonal_flow_root =
            !IsParallelWritingMode(container_writing_mode, self.Style().GetWritingMode());
        let is_in_perform_layout = LayoutPassScope::IsActive();
        if !is_in_perform_layout
            && (is_orthogonal_flow_root
                || self.IsGrid()
                || self.IsGridLanes()
                || (self.IsFlexibleBox() && self.Style().ResolvedIsColumnFlexDirection()))
        {
            let geometry =
                IntrinsicFragmentGeometry(&mut cached_fragment_geometry, constraint_space, self);
            let border_padding = geometry.border + geometry.padding;
            let inline_sum = border_padding.InlineSum();
            return MinMaxSizesResult::new(
                MinMaxSizes {
                    min_size: inline_sum,
                    max_size: inline_sum,
                },
                false,
            );
        }

        if is_orthogonal_flow_root {
            let _disable_side_effects = if !unsafe { &*box_ }.NeedsLayout() {
                Some(DisableLayoutSideEffectsScope::new())
            } else {
                None
            };
            let layout_result = self.Layout(
                constraint_space,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
            );
            debug_assert!(unsafe { &*layout_result }.Status() == EStatus::kSuccess);
            let inline_size = LogicalFragment::new(
                WritingDirectionMode::new(container_writing_mode, TextDirection::kLtr),
                unsafe { &*layout_result }.GetPhysicalFragment(),
            )
            .InlineSize();
            let style = self.Style();
            let depends_on_block_constraints = style.LogicalWidth().HasAuto()
                || style.LogicalWidth().HasPercentOrStretch()
                || style.LogicalMinWidth().HasPercentOrStretch()
                || style.LogicalMaxWidth().HasPercentOrStretch();
            return MinMaxSizesResult::new(
                MinMaxSizes {
                    min_size: inline_size,
                    max_size: inline_size,
                },
                depends_on_block_constraints,
            );
        }

        let depends_on_block_constraints = || {
            let style = self.Style();
            style.LogicalHeight().HasPercentOrStretch()
                || style.LogicalMinHeight().HasPercentOrStretch()
                || style.LogicalMaxHeight().HasPercentOrStretch()
                || (style.LogicalHeight().HasAuto()
                    && constraint_space.IsBlockAutoBehaviorStretch())
        };

        if self.IsReplaced() {
            let inline_size =
                IntrinsicFragmentGeometry(&mut cached_fragment_geometry, constraint_space, self)
                    .border_box_size
                    .inline_size;
            let mut sizes = MinMaxSizes::default();
            sizes.Assign(inline_size);
            return MinMaxSizesResult::new(sizes, depends_on_block_constraints());
        }

        let has_aspect_ratio = !self.Style().AspectRatio().IsAuto();
        if has_aspect_ratio && type_ == SizeType::kContent {
            let geometry =
                IntrinsicFragmentGeometry(&mut cached_fragment_geometry, constraint_space, self);
            let border_padding = geometry.border + geometry.padding;
            if geometry.border_box_size.block_size != kIndefiniteSize {
                let inline_size_from_ar = InlineSizeFromAspectRatio(
                    &border_padding,
                    &self.Style().LogicalAspectRatio(),
                    self.Style().BoxSizingForAspectRatio(),
                    geometry.border_box_size.block_size,
                );
                return MinMaxSizesResult::with_applied_aspect_ratio(
                    MinMaxSizes {
                        min_size: inline_size_from_ar,
                        max_size: inline_size_from_ar,
                    },
                    depends_on_block_constraints(),
                    true,
                );
            }
        }

        let mut result: Option<MinMaxSizesResult> = None;
        if CanUseCachedIntrinsicInlineSizes(constraint_space, &float_input, self) {
            if !unsafe { &*box_ }.IntrinsicLogicalWidthsDependsOnBlockConstraints() {
                result = Some(unsafe { &*box_ }.CachedIndefiniteIntrinsicLogicalWidths());
            } else {
                let initial_block_size = IntrinsicFragmentGeometry(
                    &mut cached_fragment_geometry,
                    constraint_space,
                    self,
                )
                .border_box_size
                .block_size;
                let will_use_parent_percent_size = initial_block_size == kIndefiniteSize
                    && self.UseParentPercentageResolutionBlockSizeForChildren();
                if !will_use_parent_percent_size {
                    result = unsafe { &*box_ }.CachedIntrinsicLogicalWidths(initial_block_size);
                }
            }
        } else {
            unsafe { &mut *box_ }.SetIntrinsicLogicalWidthsDirty(MarkingBehavior::kMarkOnlyThis);
        }

        if result.is_none() {
            let geometry =
                IntrinsicFragmentGeometry(&mut cached_fragment_geometry, constraint_space, self);
            result = Some(ComputeMinMaxSizesWithAlgorithm(
                &LayoutAlgorithmParams::new(self.clone(), geometry, constraint_space),
                &float_input,
            ));
            let border_padding = geometry.border + geometry.padding;
            if let Some(min_size) = ContentMinimumInlineSize(self, &border_padding) {
                result.as_mut().expect("measured sizes").sizes.min_size = min_size;
            }
            unsafe { &mut *box_ }.SetIntrinsicLogicalWidths(
                geometry.border_box_size.block_size,
                result.as_ref().expect("measured sizes"),
            );
            if self.IsTableCell() {
                unsafe { &mut *box_ }
                    .SetIntrinsicLogicalWidthsBorderSizes(&constraint_space.TableCellBorders());
            }
        }

        if has_aspect_ratio {
            let geometry =
                IntrinsicFragmentGeometry(&mut cached_fragment_geometry, constraint_space, self);
            if geometry.border_box_size.block_size == kIndefiniteSize {
                let border_padding = geometry.border + geometry.padding;
                let min_max = ComputeMinMaxInlineSizesFromAspectRatio(
                    constraint_space,
                    self,
                    &border_padding,
                );
                let sizes = &mut result.as_mut().expect("measured sizes").sizes;
                sizes.min_size = min_max.ClampSizeToMinAndMax(sizes.min_size);
                sizes.max_size = min_max.ClampSizeToMinAndMax(sizes.max_size);
            }
        }

        let result = result.as_mut().expect("measured sizes");
        result.depends_on_block_constraints = (depends_on_block_constraints()
            || self.UseParentPercentageResolutionBlockSizeForChildren())
            && (result.depends_on_block_constraints || has_aspect_ratio);
        *result
    }

    // cpp: layoutng/internal/block_node.cc:53-64
    pub fn LayoutRepeatableRoot(
        &self,
        constraint_space: &ConstraintSpace,
        break_token: *const BlockBreakToken,
    ) -> *const LayoutResult {
        let algorithms = LayoutPassScope::Algorithms();
        let callback = if algorithms.is_null() {
            None
        } else {
            unsafe { &*algorithms }
                .fragmentainer_support
                .layout_repeatable_root
        };
        let callback = callback.unwrap_or_else(|| {
            std::panic::panic_any(UnsupportedLayout::new(
                "fragment repetition module is not installed",
            ))
        });
        callback(self, constraint_space, break_token)
    }

    // cpp: layoutng/internal/block_node.cc:66-76
    pub fn FinishRepeatableRoot(&self) {
        let algorithms = LayoutPassScope::Algorithms();
        let callback = if algorithms.is_null() {
            None
        } else {
            unsafe { &*algorithms }
                .fragmentainer_support
                .finish_repeatable_root
        };
        let callback = callback.unwrap_or_else(|| {
            std::panic::panic_any(UnsupportedLayout::new(
                "fragment repetition module is not installed",
            ))
        });
        callback(self);
    }

    // cpp: layoutng/internal/block_node.cc:843-868
    pub fn NextBlockSibling(&self) -> BlockNode {
        let mut next_sibling = unsafe { &*self.GetLayoutBox() }.NextSibling();
        while !next_sibling.is_null() && unsafe { &*next_sibling }.IsInline() {
            #[cfg(debug_assertions)]
            {
                if !unsafe { &*next_sibling }.IsText() {
                    unsafe { &*next_sibling }.ShowLayoutTreeForThis();
                }
                debug_assert!(unsafe { &*next_sibling }.IsText());
            }
            unsafe { &mut *next_sibling }.ClearNeedsLayout();
            next_sibling = unsafe { &*next_sibling }.NextSibling();
        }
        if next_sibling.is_null() {
            return BlockNode::null();
        }
        BlockNode::new(To::<LayoutBox>(next_sibling))
    }

    // cpp: layoutng/internal/block_node.cc:870-915
    pub fn FirstChild(&self) -> LayoutInputNode {
        if self.ChildLayoutBlockedByDisplayLock() {
            return LayoutInputNode::null();
        }
        let box_ = unsafe { &*self.GetLayoutBox() };
        let block = DynamicTo::<LayoutBlock>(self.GetLayoutBox());
        if block.is_null() {
            return BlockNode::new(To::<LayoutBox>(box_.SlowFirstChild())).base;
        }
        let block_ref = unsafe { &*block };
        let mut child = block_ref.FirstChild();
        if child.is_null() {
            return LayoutInputNode::null();
        }
        if !block_ref.ChildrenInline() {
            return BlockNode::new(To::<LayoutBox>(child)).base;
        }
        let inline_node = InlineNode::new(To::<LayoutBlockFlow>(block));
        if !inline_node.IsBlockLevel() {
            return inline_node.base;
        }
        while !child.is_null() && unsafe { &*child }.IsInline() {
            debug_assert!(unsafe { &*child }.IsText());
            unsafe { &mut *child }.ClearNeedsLayout();
            child = unsafe { &*child }.NextSibling();
        }
        if child.is_null() {
            return LayoutInputNode::null();
        }
        debug_assert!(unsafe { &*child }.IsFloatingOrOutOfFlowPositioned());
        BlockNode::new(To::<LayoutBox>(child)).base
    }

    // cpp: layoutng/internal/block_node.cc:917-927
    pub fn GetRenderedLegend(&self) -> BlockNode {
        if !self.IsFieldsetContainer() {
            return BlockNode::null();
        }
        let mut child = unsafe { &*self.GetLayoutBox() }.SlowFirstChild();
        while !child.is_null() {
            if unsafe { &*child }.IsRenderedLegendCandidate() {
                return BlockNode::new(To::<LayoutBox>(child));
            }
            child = unsafe { &*child }.NextSibling();
        }
        BlockNode::null()
    }

    // cpp: layoutng/internal/block_node.cc:929-936
    pub fn EmptyLineBlockSize(&self, incoming_break_token: *const BlockBreakToken) -> LayoutUnit {
        if IsBreakInside(incoming_break_token) {
            return LayoutUnit::default();
        }
        unsafe { &*self.GetLayoutBox() }
            .FirstLineStyleRef()
            .ComputedLineHeightAsFixed()
    }

    // cpp: layoutng/internal/block_node.cc:938-940
    pub fn ToString(&self) -> BlinkString {
        StrCat(&[
            BlinkString::from("BlockNode: "),
            unsafe { &*self.GetLayoutBox() }.ToString(),
        ])
    }

    // cpp: layoutng/internal/block_node.cc:484-540
    pub fn SimplifiedLayout(&self, previous_fragment: &PhysicalFragment) -> *const LayoutResult {
        let box_ = unsafe { &*self.GetLayoutBox() };
        let previous_result = box_.GetSingleCachedLayoutResult();
        debug_assert!(!previous_result.is_null());
        let previous_result_ref = unsafe { &*previous_result };
        if !std::ptr::eq(previous_result_ref.GetPhysicalFragment(), previous_fragment) {
            return std::ptr::null();
        }
        if !box_.NeedsLayout() {
            return previous_result;
        }
        debug_assert!(box_.NeedsSimplifiedLayoutOnly() || box_.ChildLayoutBlockedByDisplayLock());
        let space = previous_result_ref.GetConstraintSpaceForCaching();
        let result = self.Layout(space, std::ptr::null(), std::ptr::null(), std::ptr::null());
        if unsafe { &*result }.Status() != EStatus::kSuccess {
            return std::ptr::null();
        }
        let old_fragment = unsafe {
            &*To::<PhysicalBoxFragment>(previous_result_ref.GetPhysicalFragment() as *const _)
        };
        let new_fragment = unsafe {
            &*To::<PhysicalBoxFragment>(unsafe { &*result }.GetPhysicalFragment() as *const _)
        };
        if old_fragment.Size() != new_fragment.Size() {
            return std::ptr::null();
        }
        if old_fragment.FirstBaseline() != new_fragment.FirstBaseline() {
            return std::ptr::null();
        }
        if old_fragment.LastBaseline() != new_fragment.LastBaseline() {
            return std::ptr::null();
        }
        #[cfg(debug_assertions)]
        unsafe { &*result }.CheckSameForSimplifiedLayout(previous_result_ref, true);
        result
    }

    // cpp: layoutng/internal/block_node.cc:542-551
    pub fn PrepareForLayout(&self) {
        if self.IsListItem() {
            let item = self.GetLayoutBox().cast::<LayoutObject>();
            UpdateListMarkerText(unsafe { &mut *item });
        }
    }

    // cpp: layoutng/internal/block_node.cc:553-636
    pub fn FinishLayout(
        &self,
        block_flow: *mut LayoutBlockFlow,
        constraint_space: &ConstraintSpace,
        break_token: *const BlockBreakToken,
        layout_result: *const LayoutResult,
        _old_box_size: Option<PhysicalSize>,
    ) {
        let box_ = self.GetLayoutBox();
        if DisableLayoutSideEffectsScope::IsDisabled() {
            unsafe { &mut *box_ }.AddMeasureLayoutResult(layout_result);
            return;
        }
        let result = unsafe { &*layout_result };
        if result.Status() != EStatus::kSuccess {
            unsafe { &mut *box_ }.SetShouldSkipLayoutCache(true);
            return;
        }
        let physical_fragment =
            unsafe { &*To::<PhysicalBoxFragment>(result.GetPhysicalFragment() as *const _) };
        if unsafe { &*box_ }.IsSVGRoot() {
            let mut content_rect: PhysicalRect = physical_fragment.LocalRect();
            content_rect.Contract(&(physical_fragment.Borders() + physical_fragment.Padding()));
            if !unsafe { &*box_ }.NeedsLayout() {
                unsafe { &mut *box_ }.SetNeedsLayoutWithMarking(
                    unsafe { &layout_invalidation_reason::kSizeChanged },
                    MarkingBehavior::kMarkOnlyThis,
                );
            }
            let algorithms = LayoutPassScope::Algorithms();
            let callback = if algorithms.is_null() {
                None
            } else {
                unsafe { &*algorithms }.svg_support.layout_root
            };
            let callback = callback.unwrap_or_else(|| {
                std::panic::panic_any(UnsupportedLayout::new("SVG layout module is not installed"))
            });
            callback(
                unsafe { &mut *self.GetLayoutBox().cast::<LayoutObject>() },
                &content_rect,
            );
        }
        let clear_trailing_results =
            !break_token.is_null() || unsafe { &*box_ }.PhysicalFragmentCount() > 1;
        self.StoreResultInLayoutBox(layout_result, break_token, clear_trailing_results);
        if !block_flow.is_null() {
            let mut has_inline_children =
                !physical_fragment.Items().is_null() || HasInlineChildren(block_flow);
            if has_inline_children && unsafe { &*box_ }.ChildLayoutBlockedByDisplayLock() {
                has_inline_children = false;
                unsafe { &mut *box_ }
                    .SetChildNeedsLayoutWithBehavior(MarkingBehavior::kMarkOnlyThis);
            }
            if !has_inline_children {
                unsafe { &mut *block_flow }.ClearInlineNodeData();
            }
        } else {
            debug_assert!(!physical_fragment.HasItems());
        }
        self.CopyFragmentDataToLayoutBox(constraint_space, result, break_token);
    }

    // cpp: layoutng/internal/block_node.cc:942-966
    pub fn CopyFragmentDataToLayoutBox(
        &self,
        constraint_space: &ConstraintSpace,
        layout_result: &LayoutResult,
        previous_break_token: *const BlockBreakToken,
    ) {
        let physical_fragment =
            unsafe { &*To::<PhysicalBoxFragment>(layout_result.GetPhysicalFragment() as *const _) };
        let is_last_fragment = physical_fragment.GetBreakToken().is_null();
        self.UpdateMarginPaddingInfoIfNeeded(constraint_space, physical_fragment);
        if !InvolvedInBlockFragmentation(constraint_space, previous_break_token) {
            UpdateChildLayoutBoxLocations(physical_fragment);
        }
        if is_last_fragment {
            unsafe { &mut *self.GetLayoutBox() }.UpdateAfterLayout();
        }
    }

    // cpp: layoutng/internal/block_node.cc:1168-1200
    pub fn UpdateMarginPaddingInfoIfNeeded(
        &self,
        space: &ConstraintSpace,
        fragment: &PhysicalFragment,
    ) {
        if space.IsTableCell() {
            return;
        }
        let style = self.Style();
        if style.MayHaveMargin() {
            let box_fragment = unsafe { &*To::<PhysicalBoxFragment>(fragment as *const _) };
            box_fragment
                .GetMutableForContainerLayout()
                .SetMargins(ComputePhysicalMarginsForSpace(space, style));
        }
        if style.MayHaveMargin() || style.MayHavePadding() {
            let box_ = unsafe { &mut *self.GetLayoutBox() };
            let containing_block = box_.ContainingBlock();
            if !containing_block.is_null()
                && unsafe { &*containing_block }.IsLayoutGridOrGridLanes()
            {
                box_.SetOverrideContainingBlockContentLogicalWidth(
                    space.MarginPaddingPercentageResolutionSize().inline_size,
                );
            }
        }
    }

    // cpp: layoutng/internal/block_node.cc:1084-1126
    pub fn HandleScrollMarkerGroup(&self) {
        let group_node = self.GetScrollMarkerGroup();
        if group_node.GetLayoutBox().is_null() {
            return;
        }
        let result = unsafe { &*group_node.GetLayoutBox() }.GetCachedLayoutResult(std::ptr::null());
        if result.is_null() {
            return;
        }
        let result_ref = unsafe { &*result };
        let fragment =
            unsafe { &*To::<PhysicalBoxFragment>(result_ref.GetPhysicalFragment() as *const _) };
        debug_assert!(fragment.IsOnlyForNode());
        let space = result_ref.GetConstraintSpaceForCaching();
        let new_result =
            group_node.Layout(space, std::ptr::null(), std::ptr::null(), std::ptr::null());
        let new_fragment = unsafe {
            &*To::<PhysicalBoxFragment>(unsafe { &*new_result }.GetPhysicalFragment() as *const _)
        };
        fragment
            .GetMutableForCloning()
            .ReplaceChildren(new_fragment);
        group_node.StoreResultInLayoutBox(result, std::ptr::null(), false);
    }

    // cpp: layoutng/internal/block_node.cc:1128-1157
    pub fn LayoutAtomicInline(
        &self,
        parent_constraint_space: &ConstraintSpace,
        parent_style: &ComputedStyle,
        use_first_line_style: bool,
        baseline_algorithm_type: BaselineAlgorithmType,
    ) -> *const LayoutResult {
        let mut builder = ConstraintSpaceBuilder::new(
            parent_constraint_space,
            self.Style().GetWritingDirection(),
            true,
        );
        SetOrthogonalFallbackInlineSizeIfNeeded(parent_style, self.base.clone(), &mut builder);
        builder.SetIsPaintedAtomically(true);
        builder.SetUseFirstLineStyle(use_first_line_style);
        builder.SetIsHiddenForPaint(parent_constraint_space.IsHiddenForPaint());
        builder.SetBaselineAlgorithmType(baseline_algorithm_type);
        builder.SetAvailableSize(parent_constraint_space.AvailableSize());
        builder.SetPercentageResolutionSize(if self.IsReplaced() {
            parent_constraint_space.ReplacedChildPercentageResolutionSize()
        } else {
            parent_constraint_space.PercentageResolutionSize()
        });
        let constraint_space = builder.ToConstraintSpace();
        let result = self.Layout(
            &constraint_space,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
        );
        if !DisableLayoutSideEffectsScope::IsDisabled() {
            unsafe { &mut *self.GetLayoutBox() }.ClearNeedsLayout();
        }
        result
    }

    // cpp: layoutng/internal/block_node.cc:638-660
    pub fn StoreResultInLayoutBox(
        &self,
        result: *const LayoutResult,
        break_token: *const BlockBreakToken,
        clear_trailing_results: bool,
    ) {
        let fragment = unsafe {
            &*To::<PhysicalBoxFragment>(unsafe { &*result }.GetPhysicalFragment() as *const _)
        };
        let mut fragment_idx = 0;
        let box_ = unsafe { &mut *self.GetLayoutBox() };
        if fragment.IsOnlyForNode() {
            box_.SetCachedLayoutResult(result, 0);
        } else {
            fragment_idx = FragmentIndex(break_token);
            box_.SetLayoutResult(result, fragment_idx);
        }
        if clear_trailing_results {
            box_.ShrinkLayoutResults(fragment_idx + 1);
        }
    }

    // cpp: layoutng/internal/block_node.cc:1159-1166
    pub fn RunSimplifiedLayout(
        &self,
        params: &LayoutAlgorithmParams,
        previous_result: &LayoutResult,
    ) -> *const LayoutResult {
        let algorithms = LayoutPassScope::Algorithms();
        if algorithms.is_null() {
            return std::ptr::null();
        }
        let callback = unsafe { &*algorithms }.simplified_support.layout;
        if let Some(callback) = callback {
            callback(params, previous_result)
        } else {
            std::ptr::null()
        }
    }

    // cpp: layoutng/internal/block_node.cc:1202-1219
    pub fn UpdateShapeOutsideInfoIfNeeded(
        &self,
        layout_result: &LayoutResult,
        constraint_space: &ConstraintSpace,
    ) {
        let box_ = unsafe { &*self.GetLayoutBox() };
        if !box_.IsFloating() || box_.GetShapeOutsideInfo().is_null() {
            return;
        }
        if layout_result.Status() != EStatus::kSuccess {
            return;
        }
        let algorithms = LayoutPassScope::Algorithms();
        let callback = if algorithms.is_null() {
            None
        } else {
            unsafe { &*algorithms }.float_support.update_shape_outside
        };
        let callback = callback.unwrap_or_else(|| {
            std::panic::panic_any(UnsupportedLayout::new(
                "float layout module is not installed",
            ))
        });
        callback(self, layout_result, constraint_space);
    }

    // cpp: layoutng/internal/block_node.cc:968-1018
    pub fn UseParentPercentageResolutionBlockSizeForChildren(&self) -> bool {
        let block = DynamicTo::<LayoutBlock>(self.GetLayoutBox());
        if block.is_null() {
            return false;
        }
        let block = unsafe { &*block };
        let style = self.Style();
        let in_quirks_mode = unsafe { &*self.GetLayoutBox() }.InQuirksModeForLayout();
        if block.IsAnonymous() {
            let parent = block.Parent();
            if !in_quirks_mode && !parent.is_null() && unsafe { &*parent }.IsFieldset() {
                return false;
            }
            let display = style.Display();
            return display == EDisplay::kBlock
                || display == EDisplay::kInlineBlock
                || display == EDisplay::kFlowRoot;
        }
        if !in_quirks_mode || !style.LogicalHeight().IsAuto() {
            return false;
        }
        if self.IsQuirkyAndFillsViewport() {
            return false;
        }
        let node = self.GetDOMNode();
        if unsafe { &*node }.IsInUserAgentShadowRoot() {
            let host = unsafe { &*node }.OwnerShadowHost();
            let input = DynamicTo::<HTMLInputElement>(host);
            if !input.is_null()
                && unsafe { &*input }.FormControlType() == FormControlType::kInputRange
            {
                return true;
            }
        }
        !block.IsLayoutReplaced()
            && !block.IsTableCell()
            && !block.IsOutOfFlowPositioned()
            && !block.IsLayoutGridOrGridLanes()
            && !block.IsFlexibleBox()
            && !block.IsLayoutCustom()
    }

    // cpp: layoutng/internal/block_node.cc:1020-1035
    pub fn IsInlineFormattingContextRoot(&self, first_child_out: *mut InlineNode) -> bool {
        let block = DynamicTo::<LayoutBlockFlow>(self.GetLayoutBox());
        if !block.is_null() && unsafe { &*block }.ChildrenInline() {
            let first_child = self.FirstChild();
            if first_child.IsInline() {
                if !first_child_out.is_null() {
                    unsafe { *first_child_out = InlineNode { base: first_child } };
                }
                return true;
            }
        }
        false
    }

    // cpp: layoutng/internal/block_node.cc:1037-1039
    pub fn IsInlineLevel(&self) -> bool {
        unsafe { &*self.GetLayoutBox() }.IsInline()
    }

    // cpp: layoutng/internal/block_node.cc:1041-1043
    pub fn IsInTopOrViewTransitionLayer(&self) -> bool {
        unsafe { &*self.GetLayoutBox() }.IsInTopOrViewTransitionLayer()
    }

    // cpp: layoutng/internal/block_node.cc:1045-1074
    pub fn GetReplacedAspectRatio(&self) -> LogicalSize {
        debug_assert!(self.IsReplaced());
        let ar_type = self.Style().AspectRatio().GetType();
        if ar_type == EAspectRatioType::kRatio {
            return self.Style().LogicalAspectRatio();
        }
        if !unsafe { &*self.GetLayoutBox() }.ShouldApplyAnySizeContainment() {
            let algorithms = LayoutPassScope::Algorithms();
            let callback = if algorithms.is_null() {
                None
            } else {
                unsafe { &*algorithms }.replaced_sizing.natural_sizing_info
            };
            let callback = callback.unwrap_or_else(|| {
                std::panic::panic_any(UnsupportedLayout::new(
                    "replaced layout module is not installed",
                ))
            });
            let sizing_info = callback(&self.base);
            if !sizing_info.aspect_ratio.IsEmpty() {
                return ToLogicalSize(sizing_info.aspect_ratio, self.Style().GetWritingMode());
            }
        }
        if ar_type == EAspectRatioType::kAutoAndRatio {
            return self.Style().LogicalAspectRatio();
        }
        LogicalSize::default()
    }

    // cpp: layoutng/internal/block_node.cc:1076-1082
    pub fn HasNonVisibleBlockOverflow(&self) -> bool {
        let clip_axes = self.GetOverflowClipAxes();
        if self.Style().IsHorizontalWritingMode() {
            return clip_axes & kOverflowClipY != kNoOverflowClip;
        }
        clip_axes & kOverflowClipX != kNoOverflowClip
    }
}

// cpp: layoutng/internal/block_node.h:264-273
pub struct DevtoolsReadonlyLayoutScope;

// cpp: layoutng/internal/block_node.cc:1220-1233
static G_DEVTOOLS_LAYOUT: AtomicBool = AtomicBool::new(false);

impl DevtoolsReadonlyLayoutScope {
    pub fn InDevtoolsLayout() -> bool {
        G_DEVTOOLS_LAYOUT.load(Ordering::Relaxed)
    }

    pub fn new() -> Self {
        debug_assert!(!Self::InDevtoolsLayout());
        G_DEVTOOLS_LAYOUT.store(true, Ordering::Relaxed);
        Self
    }
}

impl Drop for DevtoolsReadonlyLayoutScope {
    fn drop(&mut self) {
        debug_assert!(Self::InDevtoolsLayout());
        G_DEVTOOLS_LAYOUT.store(false, Ordering::Relaxed);
    }
}

// cpp: layoutng/internal/block_node.cc:674-684
fn IntrinsicFragmentGeometry<'a>(
    cached: &'a mut Option<FragmentGeometry>,
    constraint_space: &ConstraintSpace,
    node: &BlockNode,
) -> &'a FragmentGeometry {
    cached.get_or_insert_with(|| {
        CalculateInitialFragmentGeometry(constraint_space, node, std::ptr::null(), true)
    })
}

// cpp: layoutng/internal/block_node.cc:78-80
fn HasInlineChildren(block_flow: *const LayoutBlockFlow) -> bool {
    let block_flow = unsafe { &*block_flow };
    !block_flow.FirstChild().is_null() && block_flow.ChildrenInline()
}

// cpp: layoutng/internal/block_node.cc:82-87
fn UpdateListMarkerText(item: &mut LayoutObject) {
    let algorithms = LayoutPassScope::Algorithms();
    let callback = if algorithms.is_null() {
        None
    } else {
        unsafe { &*algorithms }.list_support.update_marker_text
    };
    let callback = callback.unwrap_or_else(|| {
        std::panic::panic_any(UnsupportedLayout::new(
            "list layout module is not installed",
        ))
    });
    callback(item);
}

// cpp: layoutng/internal/block_node.cc:91-112
fn SelectAlgorithm(params: &LayoutAlgorithmParams) -> LayoutAlgorithmEntry {
    let algorithms = LayoutPassScope::Algorithms();
    assert!(
        !algorithms.is_null(),
        "LayoutNG algorithm set was not installed"
    );
    let algorithms = unsafe { &*algorithms };
    let box_ = unsafe { &*params.node.GetLayoutBox() };
    if box_.IsFlexibleBox() {
        return algorithms.flex;
    }
    if box_.IsTable() {
        return algorithms.table;
    }
    if box_.IsTableRow() {
        return algorithms.table_row;
    }
    if box_.IsTableSection() {
        return algorithms.table_section;
    }
    if box_.IsLayoutCustom() {
        return algorithms.custom;
    }
    if box_.IsMathML() {
        return algorithms.mathml;
    }
    if box_.IsLayoutGrid() {
        return algorithms.grid;
    }
    if box_.IsLayoutGridLanes() {
        return algorithms.grid_lanes;
    }
    if box_.IsLayoutReplaced() {
        return algorithms.replaced;
    }
    if box_.IsFieldset() {
        return algorithms.fieldset;
    }
    if box_.IsFrameSet() {
        return algorithms.frameset;
    }
    if box_.IsMulticolContainer() {
        return algorithms.multicol;
    }
    if box_.Parent().is_null() && params.node.IsPaginatedRoot() {
        return algorithms.paged;
    }
    algorithms.block
}

// cpp: layoutng/internal/block_node.cc:114-120
fn LayoutWithAlgorithm(params: &LayoutAlgorithmParams) -> *const LayoutResult {
    let run = SelectAlgorithm(params)
        .layout
        .expect("LayoutNG algorithm for this node was not assembled");
    run(params)
}

// cpp: layoutng/internal/block_node.cc:122-129
fn ComputeMinMaxSizesWithAlgorithm(
    params: &LayoutAlgorithmParams,
    float_input: &MinMaxSizesFloatInput,
) -> MinMaxSizesResult {
    let measure = SelectAlgorithm(params)
        .measure
        .expect("LayoutNG intrinsic-size algorithm was not assembled");
    measure(params, float_input)
}

// cpp: layoutng/internal/block_node.cc:131-190
fn CanUseCachedIntrinsicInlineSizes(
    constraint_space: &ConstraintSpace,
    float_input: &MinMaxSizesFloatInput,
    node: &BlockNode,
) -> bool {
    let box_ = unsafe { &*node.GetLayoutBox() };
    if box_.IntrinsicLogicalWidthsDirty() {
        return false;
    }
    if float_input.float_left_inline_size != LayoutUnit::default()
        || float_input.float_right_inline_size != LayoutUnit::default()
    {
        return false;
    }
    let style = node.Style();
    if style.MayHavePadding()
        && (style.PaddingTop().HasPercent()
            || style.PaddingRight().HasPercent()
            || style.PaddingBottom().HasPercent()
            || style.PaddingLeft().HasPercent())
    {
        return false;
    }
    if node.IsTableCell()
        && *box_.IntrinsicLogicalWidthsBorderSizes() != constraint_space.TableCellBorders()
    {
        return false;
    }
    if node.IsGrid() || node.IsGridLanes() {
        if style.LogicalMinWidth().HasPercentOrStretch()
            || style.LogicalMaxWidth().HasPercentOrStretch()
        {
            return false;
        }
        if !style.AspectRatio().IsAuto()
            && (style.LogicalMinHeight().HasPercentOrStretch()
                || style.LogicalMaxHeight().HasPercentOrStretch())
        {
            return false;
        }
    }
    if node.IsFlexibleBox()
        && style.ResolvedIsColumnFlexDirection()
        && !style.ResolvedIsFlexNowrap()
        && (style.LogicalMinHeight().HasPercentOrStretch()
            || style.LogicalMaxHeight().HasPercentOrStretch())
    {
        return false;
    }
    true
}

// cpp: layoutng/internal/block_node.cc:192-245
fn ContentMinimumInlineSize(
    block_node: &BlockNode,
    border_padding: &BoxStrut,
) -> Option<LayoutUnit> {
    if block_node.IsTable() {
        return None;
    }
    let node = block_node.GetDOMNode();
    let element = if node.is_null() {
        std::ptr::null_mut()
    } else {
        DynamicTo::<Element>(node)
    };
    let element_data = if element.is_null() {
        None
    } else {
        unsafe { &*element }.InputElementData().as_ref()
    };
    if !node.is_null()
        && unsafe { &*node }.InputKind() == NodeKind::kMarquee
        && element_data.map_or(true, |data| data.marquee_horizontal)
    {
        return Some(border_padding.InlineSum());
    }
    let style = block_node.Style();
    let main_inline_size = style.LogicalWidth();
    if !main_inline_size.HasPercent() {
        return None;
    }
    let mut inline_size = MinimumValueForLength(main_inline_size, LayoutUnit::default());
    if style.BoxSizing() == EBoxSizing::kBorderBox {
        inline_size = std::cmp::max(border_padding.InlineSum(), inline_size);
    } else {
        inline_size += border_padding.InlineSum();
    }
    let apply_form_sizing = style.ApplyControlFixedSize(node as *const Node);
    if block_node.IsTextControl() && apply_form_sizing {
        return Some(inline_size);
    }
    if !DynamicTo::<HTMLSelectElement>(node).is_null() && apply_form_sizing {
        return Some(inline_size);
    }
    let input_element = DynamicTo::<HTMLInputElement>(node);
    if !input_element.is_null() {
        let control_type = unsafe { &*input_element }.FormControlType();
        if control_type == FormControlType::kInputFile && apply_form_sizing {
            return Some(inline_size);
        }
        if control_type == FormControlType::kInputRange {
            return Some(inline_size);
        }
    }
    None
}

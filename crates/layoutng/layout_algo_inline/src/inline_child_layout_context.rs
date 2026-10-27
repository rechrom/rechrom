// C++: layoutng_inline/inline_child_layout_context.h/.cc.
#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{HeapVector, LayoutUnit, MakeGarbageCollectedDefault, Member};
use layout_fragment::box_fragment_builder::BoxFragmentBuilder;
use layout_fragment::break_token::BreakToken;
use layout_fragment::fragment_items_builder::FragmentItemsBuilder;
use layout_fragment::inline_break_token::InlineBreakToken;
use layout_fragment::logical_line_item::LogicalLineItems;
use layoutng::internal::inline_item::InlineItems;
use layoutng::internal::inline_node::InlineNode;

use crate::inline_box_state::InlineLayoutStateStack;
use crate::line_info::LineInfo;
use crate::score_line_break_context::{ScoreLineBreakContext, ScoreLineBreakContextOf};
use crate::text_fit_utils::ParagraphScale;

// cpp: layoutng_inline/inline_child_layout_context.cc:40-45
fn IsBlockFragmented(fragment_builder: &BoxFragmentBuilder) -> bool {
    let space = fragment_builder.GetConstraintSpace();
    space.HasBlockFragmentation() && space.HasKnownFragmentainerBlockSize()
}

// cpp: layoutng_inline/inline_child_layout_context.h:29-135
pub struct InlineChildLayoutContext {
    container_builder_: *mut BoxFragmentBuilder,
    items_builder_: FragmentItemsBuilder,
    line_info_: *mut LineInfo,
    score_line_break_context_: Option<std::ptr::NonNull<dyn ScoreLineBreakContext>>,
    temp_logical_line_items_: *mut LogicalLineItems,
    box_states_: Option<InlineLayoutStateStack>,
    items_: *const InlineItems,
    item_index_: u32,
    parallel_flow_break_tokens_: HeapVector<Member<BreakToken>>,
    balanced_available_width_: Option<LayoutUnit>,
    minimum_scale_: ParagraphScale,
    is_measuring_scale_: bool,
}

// layoutng's declaration-only context is passed as this inline-owned context
// by the algorithm callbacks. The cast preserves the source pointer identity.
// cpp: layoutng_inline/inline_child_layout_context.h:64-66
#[unsafe(no_mangle)]
pub extern "Rust" fn InlineChildLayoutContextParallelFlowBreakTokens<'a>(
    context: &'a layoutng::internal::algorithm_forward::InlineChildLayoutContext,
) -> &'a HeapVector<Member<BreakToken>> {
    unsafe { &*(context as *const _ as *const InlineChildLayoutContext) }.ParallelFlowBreakTokens()
}

#[allow(non_snake_case)]
impl InlineChildLayoutContext {
    // cpp: layoutng_inline/inline_child_layout_context.cc:49-71
    fn new(node: &InlineNode, container_builder: &mut BoxFragmentBuilder) -> Self {
        let items_builder = FragmentItemsBuilder::new_for_inline(
            node,
            container_builder.GetWritingDirection(),
            IsBlockFragmented(container_builder),
        );
        Self {
            container_builder_: container_builder,
            items_builder_: items_builder,
            line_info_: std::ptr::null_mut(),
            score_line_break_context_: None,
            temp_logical_line_items_: std::ptr::null_mut(),
            box_states_: None,
            items_: std::ptr::null(),
            item_index_: 0,
            parallel_flow_break_tokens_: HeapVector::default(),
            balanced_available_width_: None,
            minimum_scale_: ParagraphScale::default(),
            is_measuring_scale_: false,
        }
    }

    // cpp: layoutng_inline/inline_child_layout_context.h:39-40
    pub fn ItemsBuilder(&mut self) -> &mut FragmentItemsBuilder {
        &mut self.items_builder_
    }

    // cpp: layoutng_inline/inline_child_layout_context.h:42-44
    pub fn GetScoreLineBreakContext(&mut self) -> Option<&mut dyn ScoreLineBreakContext> {
        self.score_line_break_context_
            .as_mut()
            .map(|context| unsafe { context.as_mut() })
    }

    // cpp: layoutng_inline/inline_child_layout_context.h:146-156
    pub fn GetLineInfo(
        &mut self,
        break_token: *const InlineBreakToken,
        is_cached_out: &mut bool,
    ) -> &mut LineInfo {
        debug_assert!(!*is_cached_out);
        if !self.line_info_.is_null() {
            return unsafe { &mut *self.line_info_ };
        }
        self.GetScoreLineBreakContext()
            .expect("inline child context storage was not bound")
            .GetLineInfoList()
            .Get(break_token, is_cached_out)
    }

    // cpp: layoutng_inline/inline_child_layout_context.h:158-166
    pub fn AcquireTempLogicalLineItems(&mut self) -> &mut LogicalLineItems {
        if !self.temp_logical_line_items_.is_null() {
            let line_items = self.temp_logical_line_items_;
            self.temp_logical_line_items_ = std::ptr::null_mut();
            let line_items = unsafe { &mut *line_items };
            debug_assert_eq!(line_items.size(), 0);
            return line_items;
        }
        unsafe { &mut *MakeGarbageCollectedDefault::<LogicalLineItems>() }
    }

    // cpp: layoutng_inline/inline_child_layout_context.h:168-173
    pub fn ReleaseTempLogicalLineItems(&mut self, line_items: &mut LogicalLineItems) {
        line_items.clear();
        self.temp_logical_line_items_ = line_items;
    }

    // cpp: layoutng_inline/inline_child_layout_context.h:52-54
    pub fn HasBoxStates(&self) -> bool {
        self.box_states_.is_some()
    }
    pub fn BoxStates(&mut self) -> *mut InlineLayoutStateStack {
        self.box_states_
            .as_mut()
            .map_or(std::ptr::null_mut(), |states| states)
    }
    pub fn ResetBoxStates(&mut self) -> &mut InlineLayoutStateStack {
        self.box_states_.insert(InlineLayoutStateStack::default())
    }

    // cpp: layoutng_inline/inline_child_layout_context.cc:78-84
    pub fn BoxStatesIfValidForItemIndex(
        &mut self,
        items: &InlineItems,
        item_index: u32,
    ) -> *mut InlineLayoutStateStack {
        if self.items_ == items as *const _ && self.item_index_ == item_index {
            return self.BoxStates();
        }
        std::ptr::null_mut()
    }

    // cpp: layoutng_inline/inline_child_layout_context.h:62-65
    pub fn SetItemIndex(&mut self, items: &InlineItems, item_index: u32) {
        self.items_ = items;
        self.item_index_ = item_index;
    }
    pub fn ParallelFlowBreakTokens(&self) -> &HeapVector<Member<BreakToken>> {
        &self.parallel_flow_break_tokens_
    }

    // cpp: layoutng_inline/inline_child_layout_context.cc:86-94
    pub fn ClearParallelFlowBreakTokens(&mut self) {
        self.parallel_flow_break_tokens_.Shrink(0);
    }
    pub fn PropagateParallelFlowBreakToken(&mut self, token: *const BreakToken) {
        self.parallel_flow_break_tokens_
            .push_back(Member::from_ptr(token as *mut BreakToken));
    }

    // cpp: layoutng_inline/inline_child_layout_context.h:70-75
    pub fn BalancedAvailableWidth(&self) -> &Option<LayoutUnit> {
        &self.balanced_available_width_
    }
    pub fn SetBalancedAvailableWidth(&mut self, value: Option<LayoutUnit>) {
        self.balanced_available_width_ = value;
    }

    // cpp: layoutng_inline/inline_child_layout_context.cc:96-109
    pub fn EnableMeasuringModeIfNecessary(&mut self, paragraph_scale: Option<&ParagraphScale>) {
        if let Some(paragraph_scale) = paragraph_scale {
            debug_assert!(!self.is_measuring_scale_);
            debug_assert!(paragraph_scale.scale > 0.0);
            self.minimum_scale_ = *paragraph_scale;
        } else {
            self.is_measuring_scale_ = true;
        }
    }
    pub fn IsMeasuringScale(&self) -> bool {
        self.is_measuring_scale_
    }
    pub fn MeasuredScale(&self) -> ParagraphScale {
        self.minimum_scale_
    }
}

// cpp: layoutng_inline/inline_child_layout_context.cc:73-76
impl Drop for InlineChildLayoutContext {
    fn drop(&mut self) {
        unsafe { &mut *self.container_builder_ }.SetItemsBuilder(std::ptr::null_mut());
        self.parallel_flow_break_tokens_.clear();
    }
}

// The wrappers' first field is the base so block_inline_layout's source-like
// pointer conversion has the same address. Deref gives both derived classes
// the base public interface without copying the context.
// cpp: layoutng_inline/inline_child_layout_context.h:137-150
#[repr(C)]
pub struct SimpleInlineChildLayoutContext {
    base: InlineChildLayoutContext,
    line_info_storage_: LineInfo,
}

impl SimpleInlineChildLayoutContext {
    #[inline(always)]
    pub fn new(node: &InlineNode, builder: &mut BoxFragmentBuilder) -> Self {
        Self {
            base: InlineChildLayoutContext::new(node, builder),
            line_info_storage_: LineInfo::default(),
        }
    }

    // Rust return values may move before reaching the caller's stack slot.
    // Bind source-style self pointers only after the context is in that slot.
    #[inline(always)]
    pub fn BindStorage(&mut self) {
        self.base.line_info_ = &mut self.line_info_storage_;
        self.base.score_line_break_context_ = None;
        unsafe { &mut *self.base.container_builder_ }
            .SetItemsBuilder(&mut self.base.items_builder_);
    }
}

impl Deref for SimpleInlineChildLayoutContext {
    type Target = InlineChildLayoutContext;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for SimpleInlineChildLayoutContext {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

// cpp: layoutng_inline/inline_child_layout_context.h:152-178
#[repr(C)]
pub struct OptimalInlineChildLayoutContext<const MAX_LINES: usize> {
    base: InlineChildLayoutContext,
    score_line_break_context_instance_: ScoreLineBreakContextOf<MAX_LINES>,
}

impl<const MAX_LINES: usize> OptimalInlineChildLayoutContext<MAX_LINES> {
    #[inline(always)]
    pub fn new(node: &InlineNode, builder: &mut BoxFragmentBuilder) -> Self {
        Self {
            base: InlineChildLayoutContext::new(node, builder),
            score_line_break_context_instance_: ScoreLineBreakContextOf::<MAX_LINES>::new(),
        }
    }

    #[inline(always)]
    pub fn BindStorage(&mut self) {
        self.base.line_info_ = std::ptr::null_mut();
        self.base.score_line_break_context_ = Some(std::ptr::NonNull::from(
            &mut self.score_line_break_context_instance_ as &mut dyn ScoreLineBreakContext,
        ));
        unsafe { &mut *self.base.container_builder_ }
            .SetItemsBuilder(&mut self.base.items_builder_);
    }
}

impl<const MAX_LINES: usize> Deref for OptimalInlineChildLayoutContext<MAX_LINES> {
    type Target = InlineChildLayoutContext;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl<const MAX_LINES: usize> DerefMut for OptimalInlineChildLayoutContext<MAX_LINES> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

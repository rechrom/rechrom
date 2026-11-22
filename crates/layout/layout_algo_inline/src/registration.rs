// C++: layoutng_inline/assembly.h/.cc.
// The inline registration is owned by this package. Concrete object constructors
// and InlineNode::Layout remain in their source-owned modules and must be
// linked in the shared LayoutNG/inline assembly before enabling this module.
#![allow(non_snake_case)]

use foundation::{MakeGarbageCollected, PhysicalRect, To, WritingMode};
use crate::break_token::BreakToken;
use crate::inline_cursor::InlineCursor;
use crate::layout_result::LayoutResult;
use layout_core::internal::algorithm_forward::InlineChildLayoutContext;
use layout_core::internal::column_spanner_path::ColumnSpannerPath;
use layout_core::internal::constraint_space::ConstraintSpace;
use layout_core::internal::inline_node::InlineNode;
use layout_core::internal::layout_block_flow::LayoutBlockFlow;
use layout_core::internal::layout_inline::LayoutInline;
use layout_core::internal::layout_input_node::MinMaxSizesFloatInput;
use layout_core::internal::layout_node_metadata::{Element, Node, Text};
use layout_core::internal::layout_object::LayoutObject;
use layout_core::internal::layout_text::LayoutText;
use layout_core::internal::layout_text_combine::LayoutTextCombine;
use layout_core::internal::min_max_sizes::MinMaxSizesResult;
use layout_core::internal::selection_state::SelectionState;
use layout_core::internal::text_combine_style_service::{
    AdjustStyleForCombinedText, AdjustStyleForTextCombine,
};
use layout_core::layout_registry::LayoutRegistry;
use layout_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};

use crate::inline_child_layout_context::InlineChildLayoutContextParallelFlowBreakTokens;
use crate::layout_br::LayoutBR;
use crate::layout_ruby_as_block::LayoutRubyAsBlock;
use crate::layout_text::layout_text_is_selected;
use crate::layout_word_break::LayoutWordBreak;
use crate::text_combine_tree::InsertInlineTextChild;

// cpp: layoutng_inline/assembly.cc:21-28
fn LayoutInlineAlgorithm(
    node: &InlineNode,
    space: &ConstraintSpace,
    break_token: *const BreakToken,
    column_spanner_path: *const ColumnSpannerPath,
    context: *mut InlineChildLayoutContext,
) -> *const LayoutResult {
    let _trace = browser_tracing::span("layout", "layout_algo_inline.Layout");
    node.Layout(space, break_token, column_spanner_path, context)
}

// cpp: layoutng_inline/assembly.cc:30-36
fn MeasureInline(
    node: &InlineNode,
    writing_mode: WritingMode,
    space: &ConstraintSpace,
    float_input: &MinMaxSizesFloatInput,
) -> MinMaxSizesResult {
    let _trace = browser_tracing::span("layout", "layout_algo_inline.Measure");
    node.ComputeMinMaxSizes(writing_mode, space, float_input)
}

// cpp: layoutng_inline/assembly.cc:38-42
fn CreateBlockRubyObject(node: &mut Node, _style: &ComputedStyle) -> *mut LayoutObject {
    MakeGarbageCollected(LayoutRubyAsBlock::new(To::<Element>(&mut *node))) as *mut LayoutObject
}

// cpp: layoutng_inline/assembly.cc:44-48
fn CreateInlineObject(node: &mut Node, _style: &ComputedStyle) -> *mut LayoutObject {
    MakeGarbageCollected(LayoutInline::new(To::<Element>(&mut *node))) as *mut LayoutObject
}

// cpp: layoutng_inline/assembly.cc:50-54
fn CreateLineBreakObject(node: &mut Node, _style: &ComputedStyle) -> *mut LayoutObject {
    let element = To::<Element>(&mut *node);
    MakeGarbageCollected(LayoutBR::new(unsafe { &mut *element })) as *mut LayoutObject
}

// cpp: layoutng_inline/assembly.cc:56-60
fn CreateTextObject(node: &mut Node, _style: &ComputedStyle) -> *mut LayoutObject {
    let text = To::<Text>(&mut *node);
    MakeGarbageCollected(LayoutText::new(
        text.cast::<Node>(),
        unsafe { &*text }.data().clone(),
    )) as *mut LayoutObject
}

// cpp: layoutng_inline/assembly.cc:62-66
fn CreateWordBreakObject(node: &mut Node, _style: &ComputedStyle) -> *mut LayoutObject {
    let element = To::<Element>(&mut *node);
    MakeGarbageCollected(LayoutWordBreak::new(unsafe { &mut *element })) as *mut LayoutObject
}

// cpp: layoutng_inline/assembly.cc:68-79
fn UpdateInlineStyle(
    block: &mut LayoutBlockFlow,
    old_style: *const ComputedStyle,
    new_style: &ComputedStyle,
) {
    block.SetNeedsCollectInlines();
    if !old_style.is_null()
        && unsafe { &*old_style }.InitialLetter().IsNormal() != new_style.InitialLetter().IsNormal()
    {
        let parent = block.Parent();
        if !parent.is_null() {
            unsafe { &mut *parent }.SetNeedsCollectInlines();
        }
    }
}

// cpp: layoutng_inline/assembly.cc:81-91
fn UpdateAnonymousTextCombineStyle(child: &mut LayoutObject, builder: &mut ComputedStyleBuilder) {
    if !LayoutTextCombine::IsSupportedMode(builder.GetWritingMode()) {
        let first_child = child.SlowFirstChild();
        debug_assert!(
            unsafe { &*first_child }.IsBR()
                || unsafe { &*To::<LayoutText>(first_child) }.IsWordBreak()
                || unsafe { &*first_child }.GetNode().is_null()
        );
        return;
    }
    unsafe { AdjustStyleForTextCombine(builder) };
}

// cpp: core/css/resolver/style_adjuster.cc:414-431
fn AdjustCombinedTextStyle(builder: &mut ComputedStyleBuilder) {
    unsafe { AdjustStyleForCombinedText(builder) };
}

fn TextFragmentSelectionFacts(
    text: &LayoutText,
) -> layout_core::internal::layout_object_factory_set::TextFragmentSelectionFacts {
    let fragment = unsafe { &*To::<crate::layout_text_fragment::LayoutTextFragment>(text) };
    let node = unsafe { fragment.AssociatedTextNode().as_ref() };
    let state = node
        .and_then(|n| unsafe { n.GetLayoutObject().as_ref() })
        .map_or(SelectionState::kNone, |o| o.GetSelectionState());
    layout_core::internal::layout_object_factory_set::TextFragmentSelectionFacts {
        node_length: node.map(|n| n.length()),
        state,
        start: fragment.Start(),
        length: fragment.FragmentLength(),
    }
}
fn IsTextFragmentSelected(text: &LayoutText) -> bool {
    debug_assert!(text.IsTextFragment());
    layout_text_is_selected(text)
}

fn AdjustTextCombineBoundingBox(object: &LayoutObject, rect: &PhysicalRect) -> PhysicalRect {
    debug_assert!(object.IsLayoutTextCombine());
    unsafe { &*To::<LayoutTextCombine>(object) }.AdjustRectForBoundingBox(rect)
}

fn RecalcTextCombineContentsInkOverflow(object: &LayoutObject, cursor: *const ()) -> PhysicalRect {
    debug_assert!(object.IsLayoutTextCombine());
    debug_assert!(!cursor.is_null());
    unsafe { &*To::<LayoutTextCombine>(object) }
        .RecalcContentsInkOverflow(unsafe { &*cursor.cast::<InlineCursor>() })
}

// cpp: layoutng_inline/assembly.h:4-4
// cpp: layoutng_inline/assembly.cc:95-112
pub fn Register(registry: &mut LayoutRegistry) {
    registry.algorithms.inline_support.performance_owner = Some("layout_algo_inline");
    registry.objects.block_ruby = Some(CreateBlockRubyObject);
    registry.objects.inline_box = Some(CreateInlineObject);
    registry.objects.inline_line_break = Some(CreateLineBreakObject);
    registry.objects.inline_text = Some(CreateTextObject);
    registry.objects.insert_inline_text_child = Some(InsertInlineTextChild);
    registry.objects.inline_word_break = Some(CreateWordBreakObject);
    registry.objects.update_inline_style = Some(UpdateInlineStyle);
    registry.objects.update_anonymous_text_combine_style = Some(UpdateAnonymousTextCombineStyle);
    registry.objects.adjust_combined_text_style = Some(AdjustCombinedTextStyle);
    registry.objects.is_text_fragment_selected = Some(IsTextFragmentSelected);
    registry.objects.text_fragment_selection_facts = Some(TextFragmentSelectionFacts);
    registry.objects.adjust_text_combine_bounding_box = Some(AdjustTextCombineBoundingBox);
    registry.objects.recalc_text_combine_contents_ink_overflow =
        Some(RecalcTextCombineContentsInkOverflow);
    registry.algorithms.inline_support.is_block_level = Some(|node| {
        node.PrepareLayoutIfNeeded();
        let block = unsafe { &*node.GetLayoutBlockFlow() };
        let data = unsafe { &*block.GetInlineNodeData() };
        data.IsBlockLevel()
    });
    registry.algorithms.inline_support.layout = Some(LayoutInlineAlgorithm);
    registry.algorithms.inline_support.measure = Some(MeasureInline);
    registry.algorithms.inline_support.parallel_flow_break_tokens =
        Some(InlineChildLayoutContextParallelFlowBreakTokens);
}

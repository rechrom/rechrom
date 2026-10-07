// C++: layoutng_inline/assembly.h/.cc.
// The inline installer is owned by this package. Concrete object constructors
// and InlineNode::Layout remain in their source-owned modules and must be
// linked in the shared LayoutNG/inline assembly before enabling this module.
#![allow(non_snake_case)]

use foundation::{MakeGarbageCollected, To, WritingMode};
use layoutng::internal::algorithm_forward::InlineChildLayoutContext;
use layoutng::internal::column_spanner_path::ColumnSpannerPath;
use layoutng::internal::constraint_space::ConstraintSpace;
use layoutng::internal::inline_node::InlineNode;
use layoutng::internal::layout_block_flow::LayoutBlockFlow;
use layoutng::internal::layout_inline::LayoutInline;
use layoutng::internal::layout_input_node::MinMaxSizesFloatInput;
use layoutng::internal::layout_node_metadata::{Element, Node, Text};
use layoutng::internal::layout_object::LayoutObject;
use layoutng::internal::layout_text::LayoutText;
use layoutng::internal::layout_text_combine::LayoutTextCombine;
use layoutng::internal::min_max_sizes::MinMaxSizesResult;
use layoutng::internal::text_combine_style_service::AdjustStyleForTextCombine;
use layoutng::layout_assembly::LayoutAssembly;
use layoutng_fragment_tree::break_token::BreakToken;
use layoutng_fragment_tree::layout_result::LayoutResult;
use layoutng_style::style::computed_style::{ComputedStyle, ComputedStyleBuilder};

use crate::layout_br::LayoutBR;
use crate::layout_ruby_as_block::LayoutRubyAsBlock;
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
    node.Layout(space, break_token, column_spanner_path, context)
}

// cpp: layoutng_inline/assembly.cc:30-36
fn MeasureInline(
    node: &InlineNode,
    writing_mode: WritingMode,
    space: &ConstraintSpace,
    float_input: &MinMaxSizesFloatInput,
) -> MinMaxSizesResult {
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

// cpp: layoutng_inline/assembly.h:4-4
// cpp: layoutng_inline/assembly.cc:95-112
pub fn InstallInlineAlgorithm(assembly: &mut LayoutAssembly) {
    assembly.objects.block_ruby = Some(CreateBlockRubyObject);
    assembly.objects.inline_box = Some(CreateInlineObject);
    assembly.objects.inline_line_break = Some(CreateLineBreakObject);
    assembly.objects.inline_text = Some(CreateTextObject);
    assembly.objects.insert_inline_text_child = Some(InsertInlineTextChild);
    assembly.objects.inline_word_break = Some(CreateWordBreakObject);
    assembly.objects.update_inline_style = Some(UpdateInlineStyle);
    assembly.objects.update_anonymous_text_combine_style = Some(UpdateAnonymousTextCombineStyle);
    assembly.algorithms.inline_support.is_block_level = Some(|node| {
        node.PrepareLayoutIfNeeded();
        let block = unsafe { &*node.GetLayoutBlockFlow() };
        let data = unsafe { &*block.GetInlineNodeData() };
        data.IsBlockLevel()
    });
    assembly.algorithms.inline_support.layout = Some(LayoutInlineAlgorithm);
    assembly.algorithms.inline_support.measure = Some(MeasureInline);
}

// C++: layoutng_inline/layout_inline_core.cc. The LayoutInline object and its
// inline declarations remain owned by //src/layoutng/internal/layout_inline.h.
#![allow(non_snake_case)]

use foundation::{DynamicTo, EDisplay, IsA, RuntimeEnabledFeatures};
use layoutng::internal::layout_block_flow::LayoutBlockFlow;
use layoutng::internal::layout_box::LayoutBox;
use layoutng::internal::layout_box_model_object::LayoutBoxModelObject;
use layoutng::internal::layout_inline::LayoutInline;
use layoutng::internal::layout_invalidation_reason;
use layoutng::internal::layout_node_metadata::Element;
use layoutng::internal::layout_object::{LayoutObject, StyleChangeContext};
use layoutng_fragment_tree::fragment_items::FragmentItems;
use layoutng_style::style::anonymous_style::CreateAnonymousStyleBuilderWithDisplay;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::style_difference::StyleDifference;

// cpp: layoutng_inline/layout_inline_core.cc:18-25
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutInlineWillBeDestroyed(this: &mut LayoutInline) {
    this.CheckIsNotDestroyed();
    if this.FirstInlineFragmentItemIndex() != 0 {
        FragmentItems::LayoutObjectWillBeDestroyed(this);
        this.ClearFirstInlineFragmentItemIndex();
    }
    LayoutBoxModelObject::WillBeDestroyed(this);
}

// cpp: layoutng_inline/layout_inline_core.cc:27-45
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutInlineStyleDidChange(
    this: &mut LayoutInline,
    difference: StyleDifference,
    old_style: *const ComputedStyle,
    new_style: &ComputedStyle,
    context: &StyleChangeContext,
) {
    this.CheckIsNotDestroyed();
    LayoutBoxModelObject::StyleDidChange(this, difference, old_style, new_style, context);
    if !this.ShouldCreateBoxFragment() {
        this.UpdateShouldCreateBoxFragment();
    }
    if difference.needs_reshape() {
        this.SetNeedsCollectInlines();
    }
    if RuntimeEnabledFeatures::AnnotationSpaceOnStartEnabled() && this.IsInlineRubyText() {
        let view = this.View();
        if !view.is_null() {
            unsafe { &mut *view }.SetContainsAnnotations();
        }
    }
    this.PropagateStyleToAnonymousChildren();
}

// cpp: layoutng_inline/layout_inline_core.cc:47-58
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutInlineComputeInitialShouldCreateBoxFragmentWithStyle(
    this: &LayoutInline,
    style: &ComputedStyle,
) -> bool {
    this.CheckIsNotDestroyed();
    if this.IsSVGInline() {
        return true;
    }
    if style.HasBoxDecorationBackground()
        || style.MayHavePadding()
        || style.MayHaveMargin()
        || !style.AnchorName().Get().is_null()
    {
        return true;
    }
    let element = DynamicTo::<Element>(this.GetNode());
    !element.is_null() && unsafe { &*element }.InputMayBeImplicitAnchor()
}

// cpp: layoutng_inline/layout_inline_core.cc:60-71
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutInlineComputeInitialShouldCreateBoxFragment(
    this: &LayoutInline,
) -> bool {
    this.CheckIsNotDestroyed();
    let style = this.StyleRef();
    if this.CanContainAbsolutePositionObjects()
        || LayoutInlineComputeInitialShouldCreateBoxFragmentWithStyle(this, style)
        || this.ShouldApplyPaintContainment()
        || this.ShouldApplyLayoutContainment()
    {
        return true;
    }
    let first_line_style = this.FirstLineStyleRef();
    !std::ptr::eq(style, first_line_style)
        && LayoutInlineComputeInitialShouldCreateBoxFragmentWithStyle(this, first_line_style)
}

// cpp: layoutng_inline/layout_inline_core.cc:73-84
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutInlineUpdateShouldCreateBoxFragment(this: &mut LayoutInline) {
    this.CheckIsNotDestroyed();
    if this.IsInLayoutNGInlineFormattingContext() {
        if this.ShouldCreateBoxFragment() {
            return;
        }
    } else {
        this.SetIsInLayoutNGInlineFormattingContext(true);
        this.SetShouldCreateBoxFragment(false);
    }
    if LayoutInlineComputeInitialShouldCreateBoxFragment(this) {
        this.SetShouldCreateBoxFragment(true);
    }
}

// cpp: layoutng_inline/layout_inline_core.cc:86-108
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutInlineAddChild(
    this: &mut LayoutInline,
    new_child: *mut LayoutObject,
    mut before_child: *mut LayoutObject,
) {
    this.CheckIsNotDestroyed();
    while !before_child.is_null() && unsafe { &*before_child }.IsTablePart() {
        before_child = unsafe { &*before_child }.Parent();
    }

    let new_child_ref = unsafe { &*new_child };
    if !new_child_ref.IsInline()
        && !new_child_ref.IsFloatingOrOutOfFlowPositioned()
        && !new_child_ref.IsTablePart()
    {
        LayoutInlineAddChildAsBlockInInline(this, new_child, before_child);
        return;
    }

    if !before_child.is_null()
        && unsafe { &*before_child }.Parent() != this as *mut _ as *mut LayoutObject
    {
        let parent = unsafe { &*before_child }.Parent();
        debug_assert!(unsafe { &*parent }.IsBlockInInline());
        debug_assert!(IsA::<LayoutBlockFlow>(parent));
        debug_assert_eq!(
            unsafe { &*parent }.Parent(),
            this as *mut _ as *mut LayoutObject
        );
        before_child = this.SplitAnonymousBoxesAroundChild(before_child);
    }

    // Explicit base call: the virtual LayoutObject::AddChild would dispatch
    // back to this provider instead of executing the source base insertion.
    this.AddChildBase(new_child, before_child);
    unsafe { &mut *new_child }.SetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(
        &raw const layout_invalidation_reason::kChildChanged,
    );
}

// cpp: layoutng_inline/layout_inline_core.cc:110-133
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutInlineAddChildAsBlockInInline(
    this: &mut LayoutInline,
    new_child: *mut LayoutObject,
    before_child: *mut LayoutObject,
) {
    this.CheckIsNotDestroyed();
    debug_assert!(!unsafe { &*new_child }.IsInline());
    let mut anonymous_box: *mut LayoutBlockFlow = std::ptr::null_mut();
    if before_child.is_null() {
        anonymous_box = DynamicTo::<LayoutBlockFlow>(this.LastChild());
    } else if unsafe { &*before_child }.IsInline()
        || unsafe { &*before_child }.IsFloatingOrOutOfFlowPositioned()
    {
        anonymous_box = DynamicTo::<LayoutBlockFlow>(unsafe { &*before_child }.PreviousSibling());
    } else {
        anonymous_box = DynamicTo::<LayoutBlockFlow>(unsafe { &*before_child }.Parent());
        debug_assert!(!anonymous_box.is_null());
        debug_assert!(unsafe { &*anonymous_box }.IsBlockInInline());
        unsafe { &mut *anonymous_box }.AddChild(new_child, before_child);
        return;
    }
    if anonymous_box.is_null() || !unsafe { &*anonymous_box }.IsBlockInInline() {
        anonymous_box = LayoutInlineCreateAnonymousContainerForBlockChildren(this);
        this.AddChildBase(anonymous_box.cast(), before_child);
    }
    unsafe { &mut *anonymous_box }.AddChild(new_child, std::ptr::null_mut());
}

// cpp: layoutng_inline/layout_inline_core.cc:135-144
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutInlineCreateAnonymousContainerForBlockChildren(
    this: &LayoutInline,
) -> *mut LayoutBlockFlow {
    this.CheckIsNotDestroyed();
    let mut builder = CreateAnonymousStyleBuilderWithDisplay(
        this.StyleRef(),
        EDisplay::kBlock,
        this.StyleRef().AppliedTextDecorationData(),
    );
    let containing_block = this.ContainingBlock();
    assert!(!containing_block.is_null());
    builder.SetDirection(unsafe { &*containing_block }.StyleRef().Direction());
    let style = builder.TakeStyle();
    LayoutBlockFlow::CreateAnonymous(this, unsafe { &*style })
}

// cpp: layoutng_inline/layout_inline_core.cc:146-152
#[unsafe(no_mangle)]
pub extern "Rust" fn LayoutInlineCreateAnonymousBoxToSplit(
    this: &LayoutInline,
    box_to_split: *const LayoutBox,
) -> *mut LayoutBox {
    this.CheckIsNotDestroyed();
    debug_assert!(unsafe { &*box_to_split }.IsBlockInInline());
    debug_assert!(IsA::<LayoutBlockFlow>(box_to_split as *const LayoutObject));
    LayoutInlineCreateAnonymousContainerForBlockChildren(this).cast()
}

#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{
    DynamicTo, EColumnSpan, EDisplay, HeapVector, MakeGarbageCollected, Member,
    PaintInvalidationReason, PhysicalOffset, RuntimeEnabledFeatures, String as BlinkString, To,
    UnsupportedLayout,
};
use layoutng_fragment_tree::fragment_items::FragmentItems;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::{ContentDistributionType, ContentPosition};
use layoutng_style::style::outline_type::OutlineType;
use layoutng_style::style::style_difference::StyleDifference;

use super::editing::forward::PositionWithAffinity;
use super::form_node_metadata::HTMLImageElement;
use super::inline_node_data::InlineNodeData;
use super::layout_block::LayoutBlock;
use super::layout_box::LayoutBox;
use super::layout_box_model_object::LayoutBoxModelObject;
use super::layout_invalidation_reason;
use super::layout_node_metadata::{ContainerNode, Node};
use super::layout_object::{LayoutObject, StyleChangeContext};
use super::layout_pass_scope::LayoutObjectFactoryScope;
use super::outline_info::LayoutOutlineInfo;
use super::outline_rect_collector::OutlineRectCollector;

// cpp: layoutng/internal/layout_block_flow.cc:80-82
fn IsInnerEditorChild(block: &LayoutBlockFlow) -> bool {
    let parent = block.Parent();
    !parent.is_null() && unsafe { &*parent }.IsTextControlInnerEditor()
}

// cpp: layoutng/internal/layout_block_flow.cc:84-87
fn IsMergeableAnonymousBlock(block: &LayoutBlockFlow) -> bool {
    block.IsAnonymousBlockFlow()
        && !block.BeingDestroyed()
        && !block.IsViewTransitionRoot()
        && !IsInnerEditorChild(block)
}

// cpp: layoutng/internal/layout_block_flow.cc:89-96
fn PreviousSiblingIgnoringOutsideListMarker(object: *const LayoutObject) -> *mut LayoutObject {
    let mut object = unsafe { &*object }.PreviousSibling();
    while !object.is_null() && unsafe { &*object }.IsLayoutOutsideListMarker() {
        object = unsafe { &*object }.PreviousSibling();
    }
    object
}

// cpp: layoutng/internal/layout_block_flow.cc:98-106
fn ReparentSubsequentFloatingOrOutOfFlow(
    from: *mut LayoutBlockFlow,
    to: *mut LayoutBlockFlow,
    mut next: *mut LayoutObject,
) {
    while !next.is_null() && unsafe { &*next }.IsFloatingOrOutOfFlowPositioned() {
        let sibling = unsafe { &*next }.NextSibling();
        unsafe { &mut *from }.MoveChildTo(to.cast(), next, std::ptr::null_mut(), true);
        next = sibling;
    }
}

// cpp: layoutng/internal/layout_block_flow.cc:108-116
fn ReparentPrecedingFloatingOrOutOfFlow(
    from: *mut LayoutBlockFlow,
    to: *mut LayoutBlockFlow,
    mut prev: *mut LayoutObject,
) {
    while !prev.is_null() && unsafe { &*prev }.IsFloatingOrOutOfFlowPositioned() {
        let sibling = unsafe { &*prev }.NextSibling();
        let before_child = unsafe { &*to }.FirstChild();
        unsafe { &mut *from }.MoveChildTo(to.cast(), prev, before_child, true);
        prev = sibling;
    }
}

// cpp: layoutng/internal/layout_block_flow.cc:432-480
fn GetInlineRun(
    start: *mut LayoutObject,
    boundary: *mut LayoutObject,
    inline_run_start: &mut *mut LayoutObject,
    inline_run_end: &mut *mut LayoutObject,
) {
    let mut curr = start;
    if !curr.is_null() && unsafe { &*curr }.IsLayoutOutsideListMarker() {
        curr = unsafe { &*curr }.NextSibling();
    }
    loop {
        while !curr.is_null()
            && !(unsafe { &*curr }.IsInline()
                || unsafe { &*curr }.IsFloatingOrOutOfFlowPositioned())
        {
            curr = unsafe { &*curr }.NextSibling();
        }
        *inline_run_start = curr;
        *inline_run_end = curr;
        if curr.is_null() {
            return;
        }
        let mut saw_inline = unsafe { &*curr }.IsInline();
        curr = unsafe { &*curr }.NextSibling();
        while !curr.is_null()
            && (unsafe { &*curr }.IsInline() || unsafe { &*curr }.IsFloatingOrOutOfFlowPositioned())
            && curr != boundary
        {
            *inline_run_end = curr;
            if unsafe { &*curr }.IsInline() {
                saw_inline = true;
            }
            curr = unsafe { &*curr }.NextSibling();
        }
        if saw_inline {
            break;
        }
    }
}

// cpp: layoutng/internal/layout_block_flow.cc:120-124
#[repr(C)]
struct SameSizeAsLayoutBlockFlow {
    block_: LayoutBlock,
    inline_node_data_: Member<InlineNodeData>,
}
const _: [(); std::mem::size_of::<LayoutBlockFlow>()] =
    [(); std::mem::size_of::<SameSizeAsLayoutBlockFlow>()];

unsafe extern "Rust" {
    fn DispatchLayoutBlockFlowAllowsInlineChildren(flow: *const LayoutBlockFlow) -> bool;
    fn DispatchLayoutBlockFlowAllowsColumns(flow: *const LayoutBlockFlow) -> bool;
    fn DispatchLayoutBlockFlowCreatesNewFormattingContext(flow: *const LayoutBlockFlow) -> bool;
    fn DispatchLayoutBlockFlowIsFragmentationContextRoot(flow: *const LayoutBlockFlow) -> bool;
    fn DispatchLayoutBlockFlowWillCollectInlines(flow: *mut LayoutBlockFlow);

    fn LayoutBlockFlowUpdateForMulticolFromMulticol(flow: *mut LayoutBlockFlow);
    fn LayoutBlockFlowShouldTruncateOverflowingTextFromInline(flow: *const LayoutBlockFlow)
        -> bool;

    fn LayoutBlockFlowSetShouldDoFullPaintInvalidationForFirstLine(flow: *mut LayoutBlockFlow);
    fn LayoutBlockFlowPositionForPoint(
        flow: *const LayoutBlockFlow,
        point: &PhysicalOffset,
    ) -> PositionWithAffinity;
    fn LayoutBlockFlowShouldMoveCaretToHorizontalBoundaryWhenPastTopOrBottom(
        flow: *const LayoutBlockFlow,
    ) -> bool;
    fn LayoutBlockFlowInvalidateDisplayItemClients(
        flow: *const LayoutBlockFlow,
        reason: PaintInvalidationReason,
    );
    fn LayoutBlockFlowNodeForHitTest(flow: *const LayoutBlockFlow) -> *mut Node;
    fn LayoutBlockFlowAddOutlineRects(
        flow: *const LayoutBlockFlow,
        collector: &mut dyn OutlineRectCollector,
        info: *mut LayoutOutlineInfo,
        offset: &PhysicalOffset,
        outline_type: OutlineType,
    );
}

// cpp: layoutng/internal/layout_block_flow.h:61-65
// cpp: layoutng/internal/layout_block_flow.cc:137-137
// cpp: layoutng/internal/layout_block_flow.h:164-164
#[repr(C)]
pub struct LayoutBlockFlow {
    block_: LayoutBlock,
    pub(crate) inline_node_data_: Member<InlineNodeData>,
}

// cpp: layoutng/internal/layout_block_flow.h:172-179
impl foundation::DowncastFrom<LayoutObject> for LayoutBlockFlow {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsLayoutBlockFlow()
    }
}

// Both immediate C++ bases occupy offset zero and use the same runtime tag.
// cpp: layoutng/internal/layout_block_flow.h:172-179
impl foundation::DowncastFrom<LayoutBlock> for LayoutBlockFlow {
    fn AllowFrom(block: &LayoutBlock) -> bool {
        block.IsLayoutBlockFlow()
    }
}

impl foundation::DowncastFrom<LayoutBox> for LayoutBlockFlow {
    fn AllowFrom(box_: &LayoutBox) -> bool {
        box_.IsLayoutBlockFlow()
    }
}

impl foundation::DowncastFrom<LayoutBoxModelObject> for LayoutBlockFlow {
    fn AllowFrom(model: &LayoutBoxModelObject) -> bool {
        model.IsLayoutBlockFlow()
    }
}

const _: () = assert!(std::mem::offset_of!(LayoutBlockFlow, block_) == 0);

impl LayoutBlockFlow {
    fn box_base_mut(&mut self) -> &mut LayoutBox {
        unsafe { &mut *(self as *mut LayoutBlockFlow).cast::<LayoutBox>() }
    }

    // cpp: layoutng/internal/layout_block_flow.h:130-130
    // cpp: layoutng/internal/layout_block_flow.cc:61-64
    pub fn ResetInlineNodeData(&mut self) {
        self.CheckIsNotDestroyed();
        self.inline_node_data_ = Member::from_ptr(MakeGarbageCollected(InlineNodeData::default()));
    }

    // cpp: layoutng/internal/layout_block_flow.h:132-132
    // cpp: layoutng/internal/layout_block_flow.cc:66-76
    pub fn ClearInlineNodeData(&mut self) {
        self.CheckIsNotDestroyed();
        let data = self.inline_node_data_.Get();
        if !data.is_null() {
            unsafe { &mut *data }.items.clear();
            unsafe { &mut *data }.text_content = BlinkString::default();
            self.inline_node_data_ = Member::default();
        }
    }

    // During this C++ base constructor the virtual AllowsInlineChildren call
    // resolves to LayoutBlockFlow, not to any later-derived class.
    // cpp: layoutng/internal/layout_block_flow.cc:126-130
    pub fn new(node: *mut ContainerNode) -> Self {
        let mut flow = Self {
            block_: LayoutBlock::new(node),
            inline_node_data_: Member::default(),
        };
        flow.SetRuntimeClass(super::layout_object::LayoutObjectClass::BlockFlow);
        if flow.AllowsInlineChildrenBase() {
            flow.SetChildrenInline(true);
        }
        flow
    }

    // cpp: layoutng/internal/layout_block_flow.h:103-103
    // cpp: layoutng/internal/layout_block_flow.cc:132-135
    pub fn AllowsInlineChildrenBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        !self.IsMulticolContainer() && !self.IsScrollMarkerGroup()
    }

    pub fn AllowsInlineChildren(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutBlockFlowAllowsInlineChildren(self) }
    }

    // cpp: layoutng/internal/layout_block_flow.h:67-68
    // cpp: layoutng/internal/layout_block_flow.cc:139-145
    pub fn CreateAnonymous(
        input_owner: &LayoutObject,
        style: &ComputedStyle,
    ) -> *mut LayoutBlockFlow {
        let flow = MakeGarbageCollected(Self::new(std::ptr::null_mut()));
        unsafe { &mut *flow }.SetInputOwnerForAnonymous(input_owner);
        unsafe { &mut *flow }.SetStyle(style as *const ComputedStyle);
        flow
    }

    // cpp: layoutng/internal/layout_block_flow.h:92-92
    // cpp: layoutng/internal/layout_block_flow.cc:147-151
    pub fn IsInitialLetterBox(&self) -> bool {
        self.CheckIsNotDestroyed();
        let node = self.GetNode();
        !node.is_null()
            && unsafe { &*node }.IsFirstLetterPseudoElement()
            && !self.StyleRef().InitialLetter().IsNormal()
    }

    // cpp: layoutng/internal/layout_block_flow.h:75-75
    // cpp: layoutng/internal/layout_block_flow.cc:153-164
    pub fn CanContainFirstFormattedLine(&self) -> bool {
        self.CheckIsNotDestroyed();
        !self.IsAnonymousBlockFlow()
            || (if RuntimeEnabledFeatures::TextBoxTrimForNestedListEnabled() {
                PreviousSiblingIgnoringOutsideListMarker(self as *const Self as *const LayoutObject)
                    .is_null()
            } else {
                self.PreviousSibling().is_null()
            })
            || self.IsFlexItem()
            || self.IsGridItem()
    }

    // cpp: layoutng/internal/layout_block_flow.h:153-154
    // cpp: layoutng/internal/layout_block_flow.cc:166-212
    fn AddChildBeforeDescendant(
        &mut self,
        new_child: *mut LayoutObject,
        before_descendant: *mut LayoutObject,
    ) {
        self.CheckIsNotDestroyed();
        let self_object = self as *mut LayoutBlockFlow as *mut LayoutObject;
        debug_assert_ne!(unsafe { &*before_descendant }.Parent(), self_object);
        let mut container = unsafe { &*before_descendant }.Parent();
        while unsafe { &*container }.Parent() != self_object {
            container = unsafe { &*container }.Parent();
        }
        debug_assert!(!container.is_null());
        assert!(unsafe { &*container }.IsAnonymous());
        if unsafe { &*container }.IsAnonymousBlockFlow() {
            if (unsafe { &*new_child }.IsInline()
                && !unsafe { &*new_child }.IsLayoutOutsideListMarker())
                || unsafe { &*new_child }.IsFloatingOrOutOfFlowPositioned()
                || !unsafe { &*before_descendant }.PreviousSibling().is_null()
            {
                unsafe { &mut *container }.AddChild(new_child, before_descendant);
            } else {
                self.AddChild(new_child, unsafe { &*before_descendant }.Parent());
            }
            return;
        }
        debug_assert!(unsafe { &*container }.IsTable());
        if unsafe { &*new_child }.IsTablePart() {
            unsafe { &mut *container }.AddChild(new_child, before_descendant);
            return;
        }
        let before_child = self.SplitAnonymousBoxesAroundChild(before_descendant);
        assert_eq!(unsafe { &*before_child }.Parent(), self_object);
        self.AddChild(new_child, before_child);
    }

    pub fn AddChild(&mut self, new_child: *mut LayoutObject, before_child: *mut LayoutObject) {
        self.AddChildBase(new_child, before_child);
    }

    // cpp: layoutng/internal/layout_block_flow.h:77-79
    pub fn AddChildDefault(&mut self, new_child: *mut LayoutObject) {
        self.AddChild(new_child, std::ptr::null_mut());
    }

    // cpp: layoutng/internal/layout_block_flow.cc:214-291
    pub fn AddChildBase(
        &mut self,
        new_child: *mut LayoutObject,
        mut before_child: *mut LayoutObject,
    ) {
        self.CheckIsNotDestroyed();
        let self_object = self as *mut LayoutBlockFlow as *mut LayoutObject;
        if !before_child.is_null() && unsafe { &*before_child }.Parent() != self_object {
            self.AddChildBeforeDescendant(new_child, before_child);
            return;
        }
        let mut made_boxes_non_inline = false;
        let child_is_block_level = !unsafe { &*new_child }.IsInline()
            && !unsafe { &*new_child }.IsFloatingOrOutOfFlowPositioned();
        if self.ChildrenInline() {
            if child_is_block_level {
                self.MakeChildrenNonInline(before_child);
                made_boxes_non_inline = true;
                if !before_child.is_null() && unsafe { &*before_child }.Parent() != self_object {
                    before_child = unsafe { &*before_child }.Parent();
                    debug_assert!(unsafe { &*before_child }.IsAnonymousBlockFlow());
                    debug_assert_eq!(unsafe { &*before_child }.Parent(), self_object);
                }
            }
        } else if !child_is_block_level {
            let after_child = if before_child.is_null() {
                self.LastChild()
            } else {
                unsafe { &*before_child }.PreviousSibling()
            };
            if !after_child.is_null() && unsafe { &*after_child }.IsAnonymousBlockFlow() {
                unsafe { &mut *after_child }.AddChild(new_child, std::ptr::null_mut());
                return;
            }
            if unsafe { &*new_child }.IsInline()
                && !unsafe { &*new_child }.IsLayoutOutsideListMarker()
            {
                let new_block_flow = To::<LayoutBlockFlow>(self.CreateAnonymousBlock());
                self.box_base_mut()
                    .AddChildBase(new_block_flow.cast(), before_child);
                ReparentPrecedingFloatingOrOutOfFlow(
                    self,
                    new_block_flow,
                    unsafe { &*new_block_flow }.PreviousSibling(),
                );
                unsafe { &mut *new_block_flow }.AddChild(new_child, std::ptr::null_mut());
                ReparentSubsequentFloatingOrOutOfFlow(
                    self,
                    new_block_flow,
                    unsafe { &*new_block_flow }.NextSibling(),
                );
                return;
            }
        }
        self.box_base_mut().AddChildBase(new_child, before_child);
        let parent_block_flow = DynamicTo::<LayoutBlockFlow>(self.Parent());
        if !parent_block_flow.is_null() && made_boxes_non_inline && IsMergeableAnonymousBlock(self)
        {
            let next_sibling = self.NextSibling();
            unsafe { &mut *parent_block_flow }
                .ChildrenMut()
                .RemoveChildNodeDefault(
                    parent_block_flow.cast(),
                    self as *mut LayoutBlockFlow as *mut LayoutObject,
                );
            self.MoveAllChildrenToBefore(parent_block_flow.cast(), next_sibling, true);
            self.Destroy();
        }
    }

    pub fn RemoveChild(&mut self, old_child: *mut LayoutObject) {
        self.RemoveChildBase(old_child);
    }

    // cpp: layoutng/internal/layout_block_flow.h:79-79
    // cpp: layoutng/internal/layout_block_flow.cc:293-348
    pub fn RemoveChildBase(&mut self, old_child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        let prev = unsafe { &*old_child }.PreviousSibling();
        let next = unsafe { &*old_child }.NextSibling();
        if !prev.is_null() && !next.is_null() && !unsafe { &*old_child }.IsInline() {
            let prev_block_flow = DynamicTo::<LayoutBlockFlow>(prev);
            if !prev_block_flow.is_null() && IsMergeableAnonymousBlock(unsafe { &*prev_block_flow })
            {
                ReparentSubsequentFloatingOrOutOfFlow(self, prev_block_flow, next);
            }
            let next_block_flow = DynamicTo::<LayoutBlockFlow>(unsafe { &*prev }.NextSibling());
            if !next_block_flow.is_null() && IsMergeableAnonymousBlock(unsafe { &*next_block_flow })
            {
                ReparentPrecedingFloatingOrOutOfFlow(self, next_block_flow, prev);
            }
        }
        self.block_.RemoveChildBase(old_child);
        if self.IsAnonymous() && IsInnerEditorChild(self) && !self.BeingDestroyed() {
            let factories = LayoutObjectFactoryScope::Objects();
            let remove = if factories.is_null() {
                None
            } else {
                unsafe { &*factories }.remove_inner_editor_child
            };
            let Some(remove) = remove else {
                std::panic::panic_any(UnsupportedLayout::new(
                    "forms layout module is not installed",
                ));
            };
            remove(self, unsafe { &mut *old_child });
            return;
        }
        if self.FirstChild() == self.LastChild() {
            let child_block_flow = DynamicTo::<LayoutBlockFlow>(self.FirstChild());
            if !child_block_flow.is_null()
                && IsMergeableAnonymousBlock(unsafe { &*child_block_flow })
            {
                self.CollapseAnonymousBlockChild(child_block_flow);
            }
        }
        if !self.FirstChild().is_null()
            && !self.BeingDestroyed()
            && !unsafe { &*old_child }.IsFloatingOrOutOfFlowPositioned()
            && !unsafe { &*old_child }.IsAnonymousBlockFlow()
        {
            self.MakeChildrenInlineIfPossible();
        }
        if self.FirstChild().is_null() && IsMergeableAnonymousBlock(self) {
            self.Destroy();
        }
    }

    // cpp: layoutng/internal/layout_block_flow.h:81-81
    // cpp: layoutng/internal/layout_block_flow.cc:350-358
    pub fn CanMergeWithBase(&self, other: &LayoutBoxModelObject) -> bool {
        let other_block_flow = DynamicTo::<LayoutBlockFlow>(other as *const LayoutBoxModelObject);
        if other_block_flow.is_null() {
            return false;
        }
        IsMergeableAnonymousBlock(self) && IsMergeableAnonymousBlock(unsafe { &*other_block_flow })
    }

    pub fn CanMergeWith(&self, other: &LayoutBoxModelObject) -> bool {
        self.block_.CanMergeWith(other)
    }

    // cpp: layoutng/internal/layout_block_flow.cc:360-373
    fn AllowsCollapseAnonymousBlockChild(&self, child: &LayoutBlockFlow) -> bool {
        if child.BeingDestroyed() {
            return false;
        }
        if child.IsViewTransitionRoot() {
            return false;
        }
        !child.ChildrenInline() || self.AllowsInlineChildren()
    }

    // cpp: layoutng/internal/layout_block_flow.h:83-83
    // cpp: layoutng/internal/layout_block_flow.cc:375-387
    pub fn CollapseAnonymousBlockChild(&mut self, child: *mut LayoutBlockFlow) {
        self.CheckIsNotDestroyed();
        if !self.AllowsCollapseAnonymousBlockChild(unsafe { &*child }) {
            return;
        }
        self.SetNeedsLayoutAndIntrinsicWidthsRecalcAndFullPaintInvalidation(unsafe {
            std::ptr::addr_of!(layout_invalidation_reason::kChildAnonymousBlockChanged)
        });
        let next_sibling = unsafe { &*child }.NextSibling();
        let has_layer = unsafe { &*child }.HasLayer();
        unsafe { &mut *child }.MoveAllChildrenToBefore(
            self as *mut _ as *mut LayoutBoxModelObject,
            next_sibling,
            has_layer,
        );
        self.SetChildrenInline(unsafe { &*child }.ChildrenInline());
        let self_object = self as *mut LayoutBlockFlow as *mut LayoutObject;
        self.ChildrenMut()
            .RemoveChildNode(self_object, child.cast(), has_layer);
        unsafe { &mut *child }.Destroy();
    }

    // cpp: layoutng/internal/layout_block_flow.h:156-156
    // cpp: layoutng/internal/layout_block_flow.cc:389-430
    pub fn MakeChildrenInlineIfPossible(&mut self) {
        self.CheckIsNotDestroyed();
        if !self.AllowsInlineChildren() {
            return;
        }
        if self.IsAnonymousBlockFlow() {
            return;
        }
        let mut blocks_to_remove = HeapVector::<Member<LayoutBlockFlow>>::default();
        let mut child = self.FirstChild();
        while !child.is_null() {
            let current = unsafe { &*child };
            if current.IsFloating() || current.IsOutOfFlowPositioned() {
                child = current.NextSibling();
                continue;
            }
            let child_block_flow = DynamicTo::<LayoutBlockFlow>(child);
            if !current.IsAnonymousBlockFlow() || child_block_flow.is_null() {
                return;
            }
            if unsafe { &*child_block_flow }.BeingDestroyed() {
                return;
            }
            if !current.ChildrenInline() {
                return;
            }
            blocks_to_remove.push_back(Member::from_ptr(child_block_flow));
            child = current.NextSibling();
        }
        for child in blocks_to_remove.iter() {
            self.CollapseAnonymousBlockChild(child.Get());
        }
        self.SetChildrenInline(true);
    }

    // cpp: layoutng/internal/layout_block_flow.cc:482-523
    pub fn MakeChildrenNonInline(&mut self, insertion_point: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        let self_object = self as *mut LayoutBlockFlow as *mut LayoutObject;
        debug_assert!(
            insertion_point.is_null() || unsafe { &*insertion_point }.Parent() == self_object
        );
        self.SetChildrenInline(false);
        self.ClearInlineNodeData();
        let mut child = self.FirstChild();
        if child.is_null() {
            return;
        }
        while !child.is_null() {
            let mut inline_run_start = std::ptr::null_mut();
            let mut inline_run_end = std::ptr::null_mut();
            GetInlineRun(
                child,
                insertion_point,
                &mut inline_run_start,
                &mut inline_run_end,
            );
            if inline_run_start.is_null() {
                break;
            }
            child = unsafe { &*inline_run_end }.NextSibling();
            let block = self.CreateAnonymousBlock();
            self.ChildrenMut()
                .InsertChildNodeDefault(self_object, block.cast(), inline_run_start);
            self.MoveChildrenTo(
                block.cast(),
                inline_run_start,
                child,
                std::ptr::null_mut(),
                false,
            );
        }
        #[cfg(debug_assertions)]
        {
            let mut current = self.FirstChild();
            while !current.is_null() {
                debug_assert!(
                    !unsafe { &*current }.IsInline()
                        || unsafe { &*current }.IsLayoutOutsideListMarker()
                );
                current = unsafe { &*current }.NextSibling();
            }
        }
        self.SetShouldDoFullPaintInvalidation();
    }

    // cpp: layoutng/internal/layout_block_flow.h:158-158
    pub fn MakeChildrenNonInlineDefault(&mut self) {
        self.MakeChildrenNonInline(std::ptr::null_mut());
    }

    // cpp: layoutng/internal/layout_block_flow.h:150-150
    // cpp: layoutng/internal/layout_block_flow.cc:525-534
    pub fn DirtyLinesFromChangedChild(&mut self, child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        if unsafe { &*child }.IsInLayoutNGInlineFormattingContext() {
            FragmentItems::DirtyLinesFromChangedChild(unsafe { &*child }, self);
        }
    }

    // cpp: layoutng/internal/layout_block_flow.h:95-95
    // cpp: layoutng/internal/layout_block_flow.cc:536-563
    pub fn AllowsColumnsBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        if self.IsRuby() {
            return false;
        }
        if self.StyleRef().IsDisplayLayoutCustom() {
            return false;
        }
        if self.IsMathML() {
            return false;
        }
        if !DynamicTo::<HTMLImageElement>(self.GetNode()).is_null() {
            return false;
        }
        true
    }

    pub fn AllowsColumns(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutBlockFlowAllowsColumns(self) }
    }

    // cpp: layoutng/internal/layout_block_flow.h:105-105
    // cpp: layoutng/internal/layout_block_flow.cc:565-603
    pub fn CreatesNewFormattingContextBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        let style = self.StyleRef();
        if self.IsInline()
            || self.IsFloatingOrOutOfFlowPositioned()
            || self.IsScrollContainer()
            || self.IsFlexItem()
            || self.IsCustomItem()
            || self.IsDocumentElement()
            || self.IsGridItem()
            || self.IsGridLanesItem()
            || self.IsWritingModeRoot()
            || self.IsMathItem()
            || style.Display() == EDisplay::kFlowRoot
            || style.Display() == EDisplay::kFlowRootListItem
            || self.ShouldApplyPaintContainment()
            || self.ShouldApplyLayoutContainment()
            || style.IsContainerForSizeContainerQueries()
            || style.HasLineClamp()
            || style.SpecifiesColumns()
            || style.GetColumnSpan() == EColumnSpan::kAll
        {
            return true;
        }
        if self.CanvasDrawElementEnabledForLayout() && unsafe { &*self.Parent() }.IsCanvas() {
            return true;
        }
        if style.AlignContent().GetPosition() != ContentPosition::kNormal
            || style.AlignContent().Distribution() != ContentDistributionType::kDefault
        {
            return true;
        }
        if self.IsRenderedLegend() {
            return true;
        }
        if self.IsSemiReplaced() {
            return true;
        }
        false
    }

    pub fn CreatesNewFormattingContext(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutBlockFlowCreatesNewFormattingContext(self) }
    }

    // cpp: layoutng/internal/layout_block_flow.h:135-139
    // cpp: layoutng/internal/layout_block_flow.cc:605-624
    pub fn StyleDidChange(
        &mut self,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        style_change_context: &StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
        self.block_
            .StyleDidChange(diff, old_style, new_style, style_change_context);
        let factories = LayoutObjectFactoryScope::Objects();
        let update_multicol = if factories.is_null() {
            None
        } else {
            unsafe { &*factories }.update_multicol
        };
        if let Some(update_multicol) = update_multicol {
            update_multicol(self, &diff, old_style, new_style);
        } else if (diff.NeedsFullLayout() || old_style.is_null())
            && self.StyleRef().SpecifiesColumns()
        {
            std::panic::panic_any(UnsupportedLayout::new(
                "multicol layout module is not installed",
            ));
        }
        if diff.needs_reshape() && !factories.is_null() {
            if let Some(update_inline_style) = unsafe { &*factories }.update_inline_style {
                update_inline_style(self, old_style, new_style);
            }
        }
    }

    // cpp: layoutng/internal/layout_block_flow.h:70-73
    pub fn IsLayoutBlockFlow(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    // cpp: layoutng/internal/layout_block_flow.h:87-90
    pub fn IsFragmentationContextRootBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsMulticolContainer()
    }

    pub fn IsFragmentationContextRoot(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutBlockFlowIsFragmentationContextRoot(self) }
    }

    // cpp: layoutng/internal/layout_block_flow.h:107-110
    pub fn GetNameBase(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        "LayoutBlockFlow"
    }

    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        self.block_.GetName()
    }

    // cpp: layoutng/internal/layout_block_flow.h:120-128
    pub fn GetInlineNodeData(&self) -> *mut InlineNodeData {
        self.CheckIsNotDestroyed();
        self.inline_node_data_.Get()
    }

    pub fn TakeInlineNodeData(&mut self) -> *mut InlineNodeData {
        self.CheckIsNotDestroyed();
        let data = self.inline_node_data_.Get();
        self.inline_node_data_ = Member::default();
        data
    }

    // cpp: layoutng/internal/layout_block_flow.h:133-133
    pub fn WillCollectInlinesBase(&mut self) {
        self.CheckIsNotDestroyed();
    }

    pub fn WillCollectInlines(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { DispatchLayoutBlockFlowWillCollectInlines(self) };
    }

    // cpp: layoutng/internal/layout_block_flow.h:97-99
    pub fn UpdateForMulticol(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutBlockFlowUpdateForMulticolFromMulticol(self) };
    }

    // cpp: layoutng/internal/layout_block_flow.h:112-112
    pub fn SetShouldDoFullPaintInvalidationForFirstLine(&mut self) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutBlockFlowSetShouldDoFullPaintInvalidationForFirstLine(self) };
    }

    // cpp: layoutng/internal/layout_block_flow.h:114-114
    pub fn PositionForPoint(&self, point: &PhysicalOffset) -> PositionWithAffinity {
        self.CheckIsNotDestroyed();
        unsafe { LayoutBlockFlowPositionForPoint(self, point) }
    }

    // cpp: layoutng/internal/layout_block_flow.h:116-116
    pub fn ShouldMoveCaretToHorizontalBoundaryWhenPastTopOrBottom(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutBlockFlowShouldMoveCaretToHorizontalBoundaryWhenPastTopOrBottom(self) }
    }

    // cpp: layoutng/internal/layout_block_flow.h:141-141
    pub fn InvalidateDisplayItemClients(&self, reason: PaintInvalidationReason) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutBlockFlowInvalidateDisplayItemClients(self, reason) };
    }

    // cpp: layoutng/internal/layout_block_flow.h:143-143
    pub fn NodeForHitTest(&self) -> *mut Node {
        self.CheckIsNotDestroyed();
        unsafe { LayoutBlockFlowNodeForHitTest(self) }
    }

    // cpp: layoutng/internal/layout_block_flow.h:145-148
    pub fn AddOutlineRects(
        &self,
        collector: &mut dyn OutlineRectCollector,
        info: *mut LayoutOutlineInfo,
        additional_offset: &PhysicalOffset,
        outline_type: OutlineType,
    ) {
        self.CheckIsNotDestroyed();
        unsafe {
            LayoutBlockFlowAddOutlineRects(self, collector, info, additional_offset, outline_type)
        };
    }

    // cpp: layoutng/internal/layout_block_flow.h:160-161
    pub fn ShouldTruncateOverflowingText(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutBlockFlowShouldTruncateOverflowingTextFromInline(self) }
    }
}

// cpp: layoutng/internal/layout_block_flow.h:172-177
pub fn LayoutBlockFlowAllowFrom(object: &LayoutObject) -> bool {
    object.IsLayoutBlockFlow()
}

impl Deref for LayoutBlockFlow {
    type Target = LayoutBlock;

    fn deref(&self) -> &Self::Target {
        &self.block_
    }
}

impl DerefMut for LayoutBlockFlow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.block_
    }
}

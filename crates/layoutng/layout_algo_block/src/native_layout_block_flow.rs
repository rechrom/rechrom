#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{
    DynamicTo, EColumnSpan, EDisplay, HeapVector, MakeGarbageCollected, Member,
    PaintInvalidationReason, PhysicalOffset, RuntimeEnabledFeatures, String as BlinkString, To,
    UnsupportedLayout,
};
use layout_fragment::fragment_items::FragmentItems;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::computed_style_constants::{
    ContentDistributionType, ContentPosition, PseudoId,
};
use layoutng_style::style::outline_type::OutlineType;
use layoutng_style::style::style_difference::StyleDifference;

use super::super::editing::forward::PositionWithAffinity;
use super::super::form_node_metadata::HTMLImageElement;
use super::super::inline_node_data::InlineNodeData;
use super::super::layout_block::LayoutBlock;
use super::super::layout_box::LayoutBox;
use super::super::layout_box_model_object::LayoutBoxModelObject;
use super::super::layout_invalidation_reason;
use super::super::layout_node_metadata::{ContainerNode, Element, ElementType, Node};
use super::super::layout_object::{LayoutObject, StyleChangeContext};
use super::super::layout_pass_scope::LayoutObjectFactoryScope;
use super::super::outline_info::LayoutOutlineInfo;
use super::super::outline_rect_collector::OutlineRectCollector;

#[path = "../../layout_invalidation/src/native_layout_block_flow_style_change.rs"]
mod layout_invalidation_native_layout_block_flow_style_change;
use layout_invalidation_native_layout_block_flow_style_change::LayoutBlockFlowStyleDidChange;

#[path = "../../layout_invalidation/src/native_layout_block_flow_child_normalization.rs"]
mod layout_invalidation_native_layout_block_flow_child_normalization;
use layout_invalidation_native_layout_block_flow_child_normalization::{
    LayoutBlockFlowCollapseAnonymousBlockChild, LayoutBlockFlowDirtyLinesFromChangedChild,
    LayoutBlockFlowMakeChildrenInlineIfPossible, LayoutBlockFlowMakeChildrenNonInline,
};

#[path = "../../layout_invalidation/src/native_layout_block_flow_child_mutation.rs"]
mod layout_invalidation_native_layout_block_flow_child_mutation;
use layout_invalidation_native_layout_block_flow_child_mutation::{
    LayoutBlockFlowAddChildBase, LayoutBlockFlowCanMergeWithBase, LayoutBlockFlowRemoveChildBase,
};

#[path = "../../layout_invalidation/src/native_layout_block_flow_multicol_update.rs"]
mod layout_invalidation_native_layout_block_flow_multicol_update;
use layout_invalidation_native_layout_block_flow_multicol_update::LayoutBlockFlowUpdateForMulticol;

// cpp: layoutng/internal/layout_block_flow.cc:89-96
fn PreviousSiblingIgnoringOutsideListMarker(object: *const LayoutObject) -> *mut LayoutObject {
    let mut object = unsafe { &*object }.PreviousSibling();
    while !object.is_null() && unsafe { &*object }.IsLayoutOutsideListMarker() {
        object = unsafe { &*object }.PreviousSibling();
    }
    object
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
        flow.SetRuntimeClass(super::super::layout_object::LayoutObjectClass::BlockFlow);
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

    pub fn AddChild(&mut self, new_child: *mut LayoutObject, before_child: *mut LayoutObject) {
        self.AddChildBase(new_child, before_child);
    }

    // cpp: layoutng/internal/layout_block_flow.h:77-79
    pub fn AddChildDefault(&mut self, new_child: *mut LayoutObject) {
        self.AddChild(new_child, std::ptr::null_mut());
    }

    // cpp: layoutng/internal/layout_block_flow.cc:214-291
    pub fn AddChildBase(&mut self, new_child: *mut LayoutObject, before_child: *mut LayoutObject) {
        LayoutBlockFlowAddChildBase(self, new_child, before_child)
    }

    pub fn RemoveChild(&mut self, old_child: *mut LayoutObject) {
        self.RemoveChildBase(old_child);
    }

    // cpp: layoutng/internal/layout_block_flow.h:79-79
    // cpp: layoutng/internal/layout_block_flow.cc:293-348
    pub fn RemoveChildBase(&mut self, old_child: *mut LayoutObject) {
        LayoutBlockFlowRemoveChildBase(self, old_child)
    }

    // cpp: layoutng/internal/layout_block_flow.h:81-81
    // cpp: layoutng/internal/layout_block_flow.cc:350-358
    pub fn CanMergeWithBase(&self, other: &LayoutBoxModelObject) -> bool {
        LayoutBlockFlowCanMergeWithBase(self, other)
    }

    pub fn CanMergeWith(&self, other: &LayoutBoxModelObject) -> bool {
        self.block_.CanMergeWith(other)
    }

    // cpp: layoutng/internal/layout_block_flow.h:83-83
    // cpp: layoutng/internal/layout_block_flow.cc:375-387
    pub fn CollapseAnonymousBlockChild(&mut self, child: *mut LayoutBlockFlow) {
        LayoutBlockFlowCollapseAnonymousBlockChild(self, child)
    }

    // cpp: layoutng/internal/layout_block_flow.h:156-156
    // cpp: layoutng/internal/layout_block_flow.cc:389-430
    pub fn MakeChildrenInlineIfPossible(&mut self) {
        LayoutBlockFlowMakeChildrenInlineIfPossible(self)
    }

    // cpp: layoutng/internal/layout_block_flow.cc:482-523
    pub fn MakeChildrenNonInline(&mut self, insertion_point: *mut LayoutObject) {
        LayoutBlockFlowMakeChildrenNonInline(self, insertion_point)
    }

    // cpp: layoutng/internal/layout_block_flow.h:158-158
    pub fn MakeChildrenNonInlineDefault(&mut self) {
        self.MakeChildrenNonInline(std::ptr::null_mut());
    }

    // cpp: layoutng/internal/layout_block_flow.h:150-150
    // cpp: layoutng/internal/layout_block_flow.cc:525-534
    pub fn DirtyLinesFromChangedChild(&mut self, child: *mut LayoutObject) {
        LayoutBlockFlowDirtyLinesFromChangedChild(self, child)
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
        LayoutBlockFlowStyleDidChange(self, diff, old_style, new_style, style_change_context)
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
    // cpp: layoutng/internal/layout_block_flow.cc:650-707
    pub fn UpdateForMulticol(&mut self) {
        LayoutBlockFlowUpdateForMulticol(self)
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

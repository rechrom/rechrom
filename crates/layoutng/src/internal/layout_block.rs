#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{
    DynamicTo, EDisplay, EPosition, LayoutUnit, MakeGarbageCollected, MinimumValueForLength,
    PhysicalOffset, PhysicalRect, RuntimeEnabledFeatures, To, UnsupportedLayout,
};
use layoutng_style::style::anonymous_style::CreateAnonymousStyleBuilderWithDisplay;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::style_difference::StyleDifference;

use super::caret_rect::CaretShape;
use super::disable_layout_side_effects_scope::DisableLayoutSideEffectsScope;
use super::editing::forward::PositionWithAffinity;
use super::hit_test_phase::HitTestPhase;
use super::layout_block_flow::LayoutBlockFlow;
use super::layout_box::LayoutBox;
use super::layout_box_core_services::LayoutBoxRecalcScrollableOverflowNG;
use super::layout_node_metadata::{ContainerNode, Element};
use super::layout_object::{
    HitTestLocation, HitTestResult, LayoutObject, MarkingBehavior, PaintInfo,
    PaintInvalidatorContext, RecalcScrollableOverflowResult, StyleChangeContext,
};
use super::layout_object_child_list::LayoutObjectChildList;
use super::layout_object_factory_set::AnonymousLayoutBlockFactory;
use super::layout_pass_scope::{LayoutObjectFactoryScope, LayoutPassScope};
use super::loader::resource::image_resource_observer::{CanDeferInvalidation, WrappedImagePtr};
use super::outline_info::LayoutOutlineInfo;
use super::outline_rect_collector::OutlineRectCollector;
use layoutng_style::style::outline_type::OutlineType;

unsafe extern "Rust" {
    fn LayoutBlockPositionForPoint(
        block: *const LayoutBlock,
        point: &PhysicalOffset,
    ) -> PositionWithAffinity;
    fn LayoutBlockPaint(block: *const LayoutBlock, info: &PaintInfo);
    fn LayoutBlockNodeAtPoint(
        block: *mut LayoutBlock,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        accumulated_offset: &PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool;
    fn LayoutBlockAddOutlineRects(
        block: *const LayoutBlock,
        collector: &mut dyn OutlineRectCollector,
        info: *mut LayoutOutlineInfo,
        additional_offset: &PhysicalOffset,
        outline_type: OutlineType,
    );
    fn LayoutBlockInvalidatePaint(block: *const LayoutBlock, context: &PaintInvalidatorContext);
    fn LayoutBlockImageChanged(
        block: *mut LayoutBlock,
        image: WrappedImagePtr,
        defer: CanDeferInvalidation,
    );
    fn LayoutBlockLocalCaretRect(
        block: *const LayoutBlock,
        caret_offset: i32,
        caret_shape: CaretShape,
    ) -> PhysicalRect;
    fn LayoutBlockIsInlineBoxWrapperActuallyChild(block: *const LayoutBlock) -> bool;
    fn LayoutBlockPositionForPointIfOutsideAtomicInlineLevel(
        block: *const LayoutBlock,
        point: &PhysicalOffset,
    ) -> PositionWithAffinity;
}

// cpp: layoutng/internal/layout_block.cc:52-56
#[repr(C)]
struct SameSizeAsLayoutBlock {
    box_: LayoutBox,
    children_: LayoutObjectChildList,
}
const _: [(); std::mem::size_of::<LayoutBlock>()] =
    [(); std::mem::size_of::<SameSizeAsLayoutBlock>()];

// cpp: layoutng/internal/layout_block.h:94-99
// cpp: layoutng/internal/layout_block.h:244-244
#[repr(C)]
pub struct LayoutBlock {
    box_: LayoutBox,
    pub(crate) children_: LayoutObjectChildList,
}

// cpp: layoutng/internal/layout_block.h:251-258
impl foundation::DowncastFrom<LayoutObject> for LayoutBlock {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsLayoutBlock()
    }
}

// LayoutBox is the first field of LayoutBlock; apply the same source tag test
// when Rust callers hold the immediate base rather than LayoutObject.
// cpp: layoutng/internal/layout_block.h:251-258
impl foundation::DowncastFrom<LayoutBox> for LayoutBlock {
    fn AllowFrom(box_: &LayoutBox) -> bool {
        box_.IsLayoutBlock()
    }
}

const _: () = assert!(std::mem::offset_of!(LayoutBlock, box_) == 0);

impl LayoutBlock {
    // cpp: layoutng/internal/layout_block.h:96-96
    // cpp: layoutng/internal/layout_block.cc:58-61
    pub fn new(node: *mut ContainerNode) -> Self {
        let block = Self {
            box_: LayoutBox::new(node),
            children_: LayoutObjectChildList::default(),
        };
        block.SetRuntimeClass(super::layout_object::LayoutObjectClass::Block);
        block
    }

    // cpp: layoutng/internal/layout_block.h:185-185
    // cpp: layoutng/internal/layout_block.cc:63-67
    pub fn WillBeDestroyed(&mut self) {
        self.CheckIsNotDestroyed();
        self.box_.WillBeDestroyed();
    }

    // cpp: layoutng/internal/layout_block.h:202-202
    // cpp: layoutng/internal/layout_block.cc:69-74
    pub fn RespectsCSSOverflow(&self) -> bool {
        self.CheckIsNotDestroyed();
        let element = DynamicTo::<Element>(self.GetNode());
        element.is_null() || !unsafe { &*element }.InputIsViewportDefining()
    }

    // cpp: layoutng/internal/layout_block.h:133-134
    // cpp: layoutng/internal/layout_block.cc:76-100
    fn AddChildBeforeDescendant(
        &mut self,
        new_child: *mut LayoutObject,
        before_descendant: *mut LayoutObject,
    ) {
        self.CheckIsNotDestroyed();
        debug_assert!(!self.IsLayoutBlockFlow());
        let self_object = self as *mut LayoutBlock as *mut LayoutObject;
        debug_assert_ne!(unsafe { &*before_descendant }.Parent(), self_object);

        if !unsafe { &*new_child }.IsInline() && !unsafe { &*new_child }.IsTablePart() {
            let before_child = self.SplitAnonymousBoxesAroundChild(before_descendant);
            debug_assert_eq!(unsafe { &*before_child }.Parent(), self_object);
            self.AddChild(new_child, before_child);
            return;
        }

        let mut before_descendant_container = unsafe { &*before_descendant }.Parent();
        while unsafe { &*before_descendant_container }.Parent() != self_object {
            before_descendant_container = unsafe { &*before_descendant_container }.Parent();
        }
        assert!(unsafe { &*before_descendant_container }.IsAnonymous());
        assert!(unsafe { &*before_descendant_container }.IsLayoutBlockFlow());
        unsafe { &mut *before_descendant_container }.AddChild(new_child, before_descendant);
    }

    // A call through the class's virtual entry keeps derived insertion rules.
    // cpp: layoutng/internal/layout_block.h:137-138
    pub fn AddChild(&mut self, new_child: *mut LayoutObject, before_child: *mut LayoutObject) {
        let object = self as *mut LayoutBlock as *mut LayoutObject;
        unsafe { &mut *object }.AddChild(new_child, before_child);
    }

    // cpp: layoutng/internal/layout_block.h:137-138
    pub fn AddChildDefault(&mut self, new_child: *mut LayoutObject) {
        self.AddChild(new_child, std::ptr::null_mut());
    }

    // cpp: layoutng/internal/layout_block.cc:102-135
    pub fn AddChildBase(&mut self, new_child: *mut LayoutObject, before_child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        let self_object = self as *mut LayoutBlock as *mut LayoutObject;
        if !before_child.is_null() && unsafe { &*before_child }.Parent() != self_object {
            self.AddChildBeforeDescendant(new_child, before_child);
            return;
        }
        debug_assert!(!self.ChildrenInline());
        if unsafe { &*new_child }.IsInline() || unsafe { &*new_child }.IsTablePart() {
            let after_child = if before_child.is_null() {
                self.LastChild()
            } else {
                unsafe { &*before_child }.PreviousSibling()
            };
            if !after_child.is_null() && unsafe { &*after_child }.IsAnonymousBlockFlow() {
                unsafe { &mut *after_child }.AddChild(new_child, std::ptr::null_mut());
                return;
            }
            let new_box = self.CreateAnonymousBlock();
            self.box_.AddChildBase(new_box.cast(), before_child);
            // C++ calls LayoutObject::AddChild virtually here. The anonymous
            // box is a LayoutBlockFlow and may own inline children.
            unsafe { &mut *new_box.cast::<LayoutObject>() }
                .AddChild(new_child, std::ptr::null_mut());
            return;
        }
        self.box_.AddChildBase(new_child, before_child);
    }

    // cpp: layoutng/internal/layout_block.h:140-140
    // cpp: layoutng/internal/layout_block.cc:137-183
    pub fn RemovePositionedObjects(&mut self, stay_within: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        let process_positioned_object_removal = |positioned_object: *mut LayoutObject| -> bool {
            if !stay_within.is_null()
                && (!unsafe { &*positioned_object }.IsDescendantOf(stay_within)
                    || stay_within == positioned_object)
            {
                return false;
            }
            unsafe { &mut *positioned_object }
                .SetChildNeedsLayoutWithBehavior(MarkingBehavior::kMarkOnlyThis);
            unsafe { &mut *positioned_object }.MarkParentForSpannerOrOutOfFlowPositionedChange();
            true
        };

        let mut has_positioned_children_in_fragment_tree = false;
        for fragment in self.PhysicalFragments() {
            if !fragment.HasOutOfFlowFragmentChild() {
                continue;
            }
            for fragment_child in fragment.Children() {
                if !fragment_child.IsOutOfFlowPositioned() {
                    continue;
                }
                let child = fragment_child.GetMutableLayoutObject();
                if !child.is_null() && process_positioned_object_removal(child) {
                    has_positioned_children_in_fragment_tree = true;
                }
            }
        }
        if has_positioned_children_in_fragment_tree {
            let containing_block = self.ContainingBlockForAbsolutePosition();
            if !containing_block.is_null() {
                unsafe { &mut *containing_block }
                    .SetChildNeedsLayoutWithBehavior(MarkingBehavior::kMarkContainerChain);
            }
        }
    }

    // cpp: layoutng/internal/layout_block.h:145-145
    // cpp: layoutng/internal/layout_block.cc:185-197
    pub fn TextIndentOffset(&self) -> LayoutUnit {
        self.CheckIsNotDestroyed();
        let style = self.StyleRef();
        let length = style.TextIndent();
        if length.IsZero() || style.IsTextIndentHanging() {
            return LayoutUnit::default();
        }
        let mut cw = LayoutUnit::default();
        if length.HasPercent() {
            cw = self.ContentLogicalWidth();
        }
        MinimumValueForLength(length, cw)
    }

    // cpp: layoutng/internal/layout_block.h:190-190
    // cpp: layoutng/internal/layout_block.cc:199-206
    pub fn HasLineIfEmpty(&self) -> bool {
        self.CheckIsNotDestroyed();
        // TextControlInnerEditorElement::CustomStyleForLayoutObject reserves
        // an editable line even before text exists. The host projection does
        // not run that DOM custom-style hook, so preserve its layout policy here.
        if self.IsTextControlInnerEditor() {
            return true;
        }
        let element = DynamicTo::<Element>(self.GetNode());
        if !element.is_null() && unsafe { &*element }.InputIsRootEditable() {
            return true;
        }
        self.FirstLineStyleRef().HasLineIfEmpty()
    }

    // cpp: layoutng/internal/layout_block.h:176-178
    // cpp: layoutng/internal/layout_block.cc:208-238
    pub fn FirstLineStyleParentBlock(&self) -> *const LayoutBlock {
        self.CheckIsNotDestroyed();
        let first_line_block = self as *const LayoutBlock;
        if self.IsInline() {
            return std::ptr::null();
        }
        if self.IsFloatingOrOutOfFlowPositioned() {
            return std::ptr::null();
        }
        let parent_block = self.Parent();
        if parent_block.is_null() || !unsafe { &*parent_block }.BehavesLikeBlockContainer() {
            return std::ptr::null();
        }
        let parent_layout_block = To::<LayoutBlock>(parent_block);
        let mut first_child = unsafe { &*parent_layout_block }.FirstChild();
        while unsafe { &*first_child }.IsFloatingOrOutOfFlowPositioned()
            || (RuntimeEnabledFeatures::FirstLineOnListItemEnabled()
                && unsafe { &*first_child }.IsListMarker())
        {
            first_child = unsafe { &*first_child }.NextSibling();
        }
        if first_child != first_line_block as *mut LayoutObject {
            return std::ptr::null();
        }
        parent_layout_block
    }

    // cpp: layoutng/internal/layout_block.h:180-182
    // cpp: layoutng/internal/layout_block.cc:240-252
    pub fn NearestInnerBlockWithFirstLine(&mut self) -> *mut LayoutBlockFlow {
        self.CheckIsNotDestroyed();
        if self.ChildrenInline() {
            return To::<LayoutBlockFlow>(self as *mut LayoutBlock);
        }
        let mut child = self.FirstChild();
        while !child.is_null()
            && !unsafe { &*child }.IsFloatingOrOutOfFlowPositioned()
            && unsafe { &*child }.IsLayoutBlockFlow()
        {
            if unsafe { &*child }.ChildrenInline() {
                return To::<LayoutBlockFlow>(child);
            }
            child = unsafe { &*To::<LayoutBlock>(child) }.FirstChild();
        }
        std::ptr::null_mut()
    }

    // cpp: layoutng/internal/layout_block.h:152-155
    pub fn CreateAnonymousBlock(&self) -> *mut LayoutBlock {
        self.CheckIsNotDestroyed();
        Self::CreateAnonymousWithParentAndDisplay(
            self as *const LayoutBlock as *const LayoutObject,
            EDisplay::kBlock,
        )
    }

    // cpp: layoutng/internal/layout_block.h:157-158
    // cpp: layoutng/internal/layout_block.cc:254-258
    pub fn CreateAnonymousBoxWithSameTypeAs(&self, parent: *const LayoutObject) -> *mut LayoutBox {
        self.CheckIsNotDestroyed();
        Self::CreateAnonymousWithParentAndDisplay(parent, self.StyleRef().Display()).cast()
    }

    // cpp: layoutng/internal/layout_block.h:125-125
    // cpp: layoutng/internal/layout_block.cc:260-263
    pub fn GetNameBase(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        std::process::abort()
    }

    // cpp: layoutng/internal/layout_block.h:149-151
    // cpp: layoutng/internal/layout_block.cc:265-332
    pub fn CreateAnonymousWithParentAndDisplay(
        parent: *const LayoutObject,
        display: EDisplay,
    ) -> *mut LayoutBlock {
        let new_display = match display {
            EDisplay::kFlex | EDisplay::kInlineFlex => EDisplay::kFlex,
            EDisplay::kGrid | EDisplay::kInlineGrid => EDisplay::kGrid,
            EDisplay::kGridLanes | EDisplay::kInlineGridLanes => EDisplay::kGridLanes,
            EDisplay::kFlowRoot => EDisplay::kFlowRoot,
            EDisplay::kBlockMath => EDisplay::kBlockMath,
            _ => EDisplay::kBlock,
        };
        let parent_ref = unsafe { &*parent };
        let mut new_style_builder = CreateAnonymousStyleBuilderWithDisplay(
            parent_ref.StyleRef(),
            new_display,
            parent_ref.StyleRef().AppliedTextDecorationData(),
        );
        parent_ref.UpdateAnonymousChildStyle(std::ptr::null_mut(), &mut new_style_builder);
        let new_style = new_style_builder.TakeStyle();

        let factories = LayoutPassScope::Objects();
        let require_factory = |factory: Option<AnonymousLayoutBlockFactory>, capability: &str| {
            let Some(factory) = factory else {
                panic!("{capability} anonymous layout factory was not assembled");
            };
            factory()
        };
        let layout_block = if new_display == EDisplay::kFlex {
            require_factory(
                if factories.is_null() {
                    None
                } else {
                    unsafe { &*factories }.anonymous_flex
                },
                "flex",
            )
        } else if new_display == EDisplay::kGrid {
            require_factory(
                if factories.is_null() {
                    None
                } else {
                    unsafe { &*factories }.anonymous_grid
                },
                "grid",
            )
        } else if new_display == EDisplay::kGridLanes {
            require_factory(
                if factories.is_null() {
                    None
                } else {
                    unsafe { &*factories }.anonymous_grid_lanes
                },
                "grid-lanes",
            )
        } else if new_display == EDisplay::kBlockMath {
            require_factory(
                if factories.is_null() {
                    None
                } else {
                    unsafe { &*factories }.anonymous_mathml
                },
                "mathml",
            )
        } else {
            debug_assert!(new_display == EDisplay::kBlock || new_display == EDisplay::kFlowRoot);
            MakeGarbageCollected(LayoutBlockFlow::new(std::ptr::null_mut())).cast::<LayoutBlock>()
        };
        unsafe { &mut *layout_block }.SetInputOwnerForAnonymous(parent_ref);
        unsafe { &mut *layout_block }.SetStyle(new_style);
        layout_block
    }

    // cpp: layoutng/internal/layout_block.h:160-160
    // cpp: layoutng/internal/layout_block.cc:334-338
    pub fn RecalcScrollableOverflow(&mut self) -> RecalcScrollableOverflowResult {
        self.CheckIsNotDestroyed();
        debug_assert!(!DisableLayoutSideEffectsScope::IsDisabled());
        LayoutBoxRecalcScrollableOverflowNG(&mut self.box_)
    }

    // cpp: layoutng/internal/layout_block.h:197-201
    // cpp: layoutng/internal/layout_block.cc:340-379
    pub fn StyleDidChange(
        &mut self,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        style_change_context: &StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
        let update_svg_text = diff.transform_changed() && self.HasSVGTextDescendants();
        let factories = LayoutObjectFactoryScope::Objects();
        if update_svg_text
            && (factories.is_null()
                || unsafe { &*factories }.svg_block_text_scale.is_none()
                || unsafe { &*factories }.update_svg_block_style.is_none())
        {
            std::panic::panic_any(UnsupportedLayout::new("SVG layout module is not installed"));
        }
        let old_squared_scale = if update_svg_text {
            unsafe { &*factories }.svg_block_text_scale.unwrap()(self)
        } else {
            1.0
        };
        self.box_
            .StyleDidChange(diff, old_style, new_style, style_change_context);
        if !old_style.is_null() && !self.Parent().is_null() {
            if unsafe { &*old_style }.GetPosition() != new_style.GetPosition()
                && new_style.GetPosition() != EPosition::kStatic
            {
                let cb = self.ContainingBlock();
                if !cb.is_null() {
                    unsafe { &mut *cb }
                        .RemovePositionedObjects(self as *mut _ as *mut LayoutObject);
                }
            }
        }
        self.PropagateStyleToAnonymousChildren();
        if update_svg_text {
            unsafe { &*factories }.update_svg_block_style.unwrap()(
                self,
                old_squared_scale,
                old_style,
                new_style,
            );
        }
    }

    // cpp: layoutng/internal/layout_block.h:162-162
    // cpp: layoutng/internal/layout_block.cc:381-386
    pub fn RecalcVisualOverflow(&mut self) {
        self.CheckIsNotDestroyed();
        let object = unsafe { &mut *(self as *mut LayoutBlock).cast::<LayoutObject>() };
        object.RecalcVisualOverflow();
    }

    // cpp: layoutng/internal/layout_block.h:101-105
    pub fn FirstChild(&self) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        debug_assert_eq!(
            self.Children() as *const _,
            self.VirtualChildren() as *const _
        );
        self.Children().FirstChild()
    }

    // cpp: layoutng/internal/layout_block.h:106-110
    pub fn LastChild(&self) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        debug_assert_eq!(
            self.Children() as *const _,
            self.VirtualChildren() as *const _
        );
        self.Children().LastChild()
    }

    // cpp: layoutng/internal/layout_block.h:116-123
    pub fn Children(&self) -> &LayoutObjectChildList {
        self.CheckIsNotDestroyed();
        &self.children_
    }

    pub fn ChildrenMut(&mut self) -> &mut LayoutObjectChildList {
        self.CheckIsNotDestroyed();
        &mut self.children_
    }

    // cpp: layoutng/internal/layout_block.h:125-125
    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        self.box_.GetName()
    }

    // cpp: layoutng/internal/layout_block.h:149-151
    pub fn CreateAnonymousWithParent(parent: *const LayoutObject) -> *mut LayoutBlock {
        Self::CreateAnonymousWithParentAndDisplay(parent, EDisplay::kBlock)
    }

    // cpp: layoutng/internal/layout_block.h:147-147
    pub fn PositionForPoint(&self, point: &PhysicalOffset) -> PositionWithAffinity {
        self.CheckIsNotDestroyed();
        unsafe { LayoutBlockPositionForPoint(self, point) }
    }

    // cpp: layoutng/internal/layout_block.h:188-188
    pub fn Paint(&self, info: &PaintInfo) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutBlockPaint(self, info) };
    }

    // cpp: layoutng/internal/layout_block.h:192-195
    pub fn NodeAtPoint(
        &mut self,
        result: &mut HitTestResult,
        location: &HitTestLocation,
        accumulated_offset: &PhysicalOffset,
        phase: HitTestPhase,
    ) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutBlockNodeAtPoint(self, result, location, accumulated_offset, phase) }
    }

    // cpp: layoutng/internal/layout_block.h:205-208
    pub fn AddOutlineRects(
        &self,
        collector: &mut dyn OutlineRectCollector,
        info: *mut LayoutOutlineInfo,
        additional_offset: &PhysicalOffset,
        outline_type: OutlineType,
    ) {
        self.CheckIsNotDestroyed();
        unsafe {
            LayoutBlockAddOutlineRects(self, collector, info, additional_offset, outline_type)
        };
    }

    // cpp: layoutng/internal/layout_block.h:210-213
    pub fn IsInSelfHitTestingPhase(&self, phase: HitTestPhase) -> bool {
        self.CheckIsNotDestroyed();
        phase == HitTestPhase::kSelfBlockBackground
    }

    // cpp: layoutng/internal/layout_block.h:231-231
    pub fn InvalidatePaint(&self, context: &PaintInvalidatorContext) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutBlockInvalidatePaint(self, context) };
    }

    // cpp: layoutng/internal/layout_block.h:233-233
    pub fn ImageChanged(&mut self, image: WrappedImagePtr, defer: CanDeferInvalidation) {
        self.CheckIsNotDestroyed();
        unsafe { LayoutBlockImageChanged(self, image, defer) };
    }

    // cpp: layoutng/internal/layout_block.h:236-237
    pub fn LocalCaretRect(&self, caret_offset: i32, caret_shape: CaretShape) -> PhysicalRect {
        self.CheckIsNotDestroyed();
        unsafe { LayoutBlockLocalCaretRect(self, caret_offset, caret_shape) }
    }

    // cpp: layoutng/internal/layout_block.h:238-238
    pub fn IsInlineBoxWrapperActuallyChild(&self) -> bool {
        self.CheckIsNotDestroyed();
        unsafe { LayoutBlockIsInlineBoxWrapperActuallyChild(self) }
    }

    // cpp: layoutng/internal/layout_block.h:241-242
    pub fn PositionForPointIfOutsideAtomicInlineLevel(
        &self,
        point: &PhysicalOffset,
    ) -> PositionWithAffinity {
        self.CheckIsNotDestroyed();
        unsafe { LayoutBlockPositionForPointIfOutsideAtomicInlineLevel(self, point) }
    }

    // This virtual override always returns the block's child-list member.
    // cpp: layoutng/internal/layout_block.h:215-223
    pub fn VirtualChildrenBase(&self) -> *mut LayoutObjectChildList {
        self.CheckIsNotDestroyed();
        std::ptr::addr_of!(self.children_).cast_mut()
    }

    pub fn VirtualChildren(&self) -> *mut LayoutObjectChildList {
        self.VirtualChildrenBase()
    }

    // cpp: layoutng/internal/layout_block.h:225-228
    pub fn IsLayoutBlockBase(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    pub fn IsLayoutBlock(&self) -> bool {
        self.IsLayoutBlockBase()
    }
}

// cpp: layoutng/internal/layout_block.h:251-256
pub fn LayoutBlockAllowFrom(object: &LayoutObject) -> bool {
    object.IsLayoutBlock()
}

impl Deref for LayoutBlock {
    type Target = LayoutBox;

    fn deref(&self) -> &Self::Target {
        &self.box_
    }
}

impl DerefMut for LayoutBlock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.box_
    }
}

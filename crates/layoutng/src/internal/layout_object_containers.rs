#![allow(non_snake_case)]

use foundation::{DynamicTo, EPosition, To};

use super::layout_block::LayoutBlock;
use super::layout_block_flow::LayoutBlockFlow;
use super::layout_box::LayoutBox;
use super::layout_object::{AncestorSkipInfo, LayoutObject};

// cpp: layoutng/internal/layout_object_containers.cc:33-50
fn FindColumnSpannerContainer(
    spanner: &LayoutObject,
    skip_info: *mut AncestorSkipInfo,
) -> *mut LayoutObject {
    debug_assert!(spanner.IsColumnSpanAll());
    let mut walker = spanner.Parent();
    while !walker.is_null() {
        let object = unsafe { &*walker };
        if object.IsMulticolContainer() {
            return walker;
        }
        if !skip_info.is_null() {
            unsafe { &mut *skip_info }.Update(object);
        }
        walker = object.ContainingBlockWithSkipInfo(skip_info) as *mut LayoutObject;
    }
    std::ptr::null_mut()
}

// cpp: layoutng/internal/layout_object_containers.cc:52-69
fn FindAncestorByPredicate(
    descendant: &LayoutObject,
    skip_info: *mut AncestorSkipInfo,
    predicate: impl Fn(*mut LayoutObject) -> bool,
) -> *mut LayoutObject {
    let mut object = descendant.Parent();
    while !object.is_null() {
        if predicate(object) {
            return object;
        }
        let object_ref = unsafe { &*object };
        if !skip_info.is_null() {
            unsafe { &mut *skip_info }.Update(object_ref);
        }
        if object_ref.IsColumnSpanAll() {
            object = FindColumnSpannerContainer(object_ref, skip_info);
            continue;
        }
        object = object_ref.Parent();
    }
    std::ptr::null_mut()
}

impl LayoutObject {
    // The source's optional skip-info argument is split into a zero-argument
    // entry and an explicit WithSkipInfo entry throughout this module.
    // cpp: layoutng/internal/layout_object.h:1822
    pub fn ContainerForAbsolutePosition(&self) -> *mut LayoutObject {
        self.ContainerForAbsolutePositionWithSkipInfo(std::ptr::null_mut())
    }

    // cpp: layoutng/internal/layout_object_containers.cc:73-79
    pub fn ContainerForAbsolutePositionWithSkipInfo(
        &self,
        skip_info: *mut AncestorSkipInfo,
    ) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        FindAncestorByPredicate(self, skip_info, |candidate| {
            unsafe { &*candidate }.CanContainAbsolutePositionObjects()
        })
    }

    // cpp: layoutng/internal/layout_object.h:1824
    pub fn ContainerForFixedPosition(&self) -> *mut LayoutObject {
        self.ContainerForFixedPositionWithSkipInfo(std::ptr::null_mut())
    }

    // cpp: layoutng/internal/layout_object_containers.cc:81-88
    pub fn ContainerForFixedPositionWithSkipInfo(
        &self,
        skip_info: *mut AncestorSkipInfo,
    ) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        debug_assert!(!self.IsText());
        FindAncestorByPredicate(self, skip_info, |candidate| {
            unsafe { &*candidate }.CanContainFixedPositionObjects()
        })
    }

    // cpp: layoutng/internal/layout_object.h:1826
    pub fn ContainerForColumnSpanner(&self) -> *mut LayoutObject {
        self.ContainerForColumnSpannerWithSkipInfo(std::ptr::null_mut())
    }

    // cpp: layoutng/internal/layout_object_containers.cc:90-94
    pub fn ContainerForColumnSpannerWithSkipInfo(
        &self,
        skip_info: *mut AncestorSkipInfo,
    ) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        FindColumnSpannerContainer(self, skip_info)
    }

    // cpp: layoutng/internal/layout_object.h:3479-3480
    pub fn ContainingBlockForAbsolutePosition(&self) -> *mut LayoutBlock {
        self.ContainingBlockForAbsolutePositionWithSkipInfo(std::ptr::null_mut())
    }

    // cpp: layoutng/internal/layout_object_containers.cc:96-101
    pub fn ContainingBlockForAbsolutePositionWithSkipInfo(
        &self,
        skip_info: *mut AncestorSkipInfo,
    ) -> *mut LayoutBlock {
        self.CheckIsNotDestroyed();
        let container = self.ContainerForAbsolutePositionWithSkipInfo(skip_info);
        if container.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &mut *container }.InclusiveContainingBlockWithSkipInfo(skip_info)
        }
    }

    // cpp: layoutng/internal/layout_object.h:3483-3484
    pub fn ContainingBlockForFixedPosition(&self) -> *mut LayoutBlock {
        self.ContainingBlockForFixedPositionWithSkipInfo(std::ptr::null_mut())
    }

    // cpp: layoutng/internal/layout_object_containers.cc:103-108
    pub fn ContainingBlockForFixedPositionWithSkipInfo(
        &self,
        skip_info: *mut AncestorSkipInfo,
    ) -> *mut LayoutBlock {
        self.CheckIsNotDestroyed();
        let container = self.ContainerForFixedPositionWithSkipInfo(skip_info);
        if container.is_null() {
            std::ptr::null_mut()
        } else {
            unsafe { &mut *container }.InclusiveContainingBlockWithSkipInfo(skip_info)
        }
    }

    // cpp: layoutng/internal/layout_object.h:2251
    pub fn InclusiveContainingBlock(&mut self) -> *mut LayoutBlock {
        self.InclusiveContainingBlockWithSkipInfo(std::ptr::null_mut())
    }

    // cpp: layoutng/internal/layout_object_containers.cc:110-115
    pub fn InclusiveContainingBlockWithSkipInfo(
        &mut self,
        skip_info: *mut AncestorSkipInfo,
    ) -> *mut LayoutBlock {
        self.CheckIsNotDestroyed();
        let layout_block = DynamicTo::<LayoutBlock>(self as *mut LayoutObject);
        if !layout_block.is_null() {
            layout_block
        } else {
            self.ContainingBlockWithSkipInfo(skip_info)
        }
    }

    // cpp: layoutng/internal/layout_object_containers.cc:117-127
    pub fn EnclosingBox(&self) -> *mut LayoutBox {
        self.CheckIsNotDestroyed();
        let mut current = self as *const LayoutObject as *mut LayoutObject;
        while !current.is_null() {
            let object = unsafe { &*current };
            if object.IsBox() {
                return To::<LayoutBox>(current);
            }
            current = object.Parent();
        }
        std::ptr::null_mut()
    }

    // cpp: layoutng/internal/layout_object_containers.cc:129-145
    pub fn FragmentItemsContainer(&self) -> *mut LayoutBlockFlow {
        self.CheckIsNotDestroyed();
        debug_assert!(!self.IsOutOfFlowPositioned());
        let block_flow = DynamicTo::<LayoutBlockFlow>(self.ContainingNGBox());
        if block_flow.is_null() {
            return std::ptr::null_mut();
        }
        #[cfg(debug_assertions)]
        {
            let mut walker = self.Parent();
            while walker != block_flow as *mut LayoutObject {
                debug_assert!(!walker.is_null());
                debug_assert!(!unsafe { &*walker }.IsLayoutBlock());
                walker = unsafe { &*walker }.Parent();
            }
        }
        block_flow
    }

    // cpp: layoutng/internal/layout_object_containers.cc:147-160
    pub fn ContainingNGBox(&self) -> *mut LayoutBox {
        self.CheckIsNotDestroyed();
        let parent = self.Parent();
        if !parent.is_null() {
            if unsafe { &*parent }.IsMedia() {
                return To::<LayoutBox>(parent);
            }
            if unsafe { &*parent }.IsCanvas() && self.CanvasDrawElementEnabledForLayout() {
                return To::<LayoutBox>(parent);
            }
        }
        // LayoutBlock derives from LayoutBox with its base at offset zero.
        self.ContainingBlock() as *mut LayoutBox
    }

    // cpp: layoutng/internal/layout_object_containers.cc:162-181
    pub fn ContainingFragmentationContextRoot(&self) -> *mut LayoutBlock {
        self.CheckIsNotDestroyed();
        if !self.MightBeInsideFragmentationContext() {
            return std::ptr::null_mut();
        }
        let mut found_column_spanner = self.IsColumnSpanAll();
        let mut ancestor = self.ContainingBlock();
        while !ancestor.is_null() {
            let object = unsafe { &*(ancestor as *const LayoutObject) };
            let box_ = unsafe { &*(ancestor as *const LayoutBox) };
            if box_.IsFragmentationContextRoot() {
                if found_column_spanner {
                    return object.ContainingFragmentationContextRoot();
                }
                return ancestor;
            }
            if object.IsColumnSpanAll() {
                found_column_spanner = true;
            }
            ancestor = object.ContainingBlock();
        }
        std::ptr::null_mut()
    }

    // cpp: layoutng/internal/layout_object_containers.cc:183-188
    pub fn IsFirstInlineFragmentSafe(&self) -> bool {
        self.CheckIsNotDestroyed();
        debug_assert!(self.IsInline());
        let block_flow = self.FragmentItemsContainer();
        !block_flow.is_null() && !unsafe { &*(block_flow as *const LayoutObject) }.NeedsLayout()
    }

    // cpp: layoutng/internal/layout_object_containers.cc:190-200
    pub fn ContainingBlockForTextOverflow(&self) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        let mut block = self.ContainingBlock() as *mut LayoutObject;
        if !block.is_null() && unsafe { &*block }.IsAnonymous() {
            block = unsafe { &*block }.Parent();
        }
        if !block.is_null() && !unsafe { &*block }.BehavesLikeBlockContainer() {
            return std::ptr::null_mut();
        }
        block
    }

    // cpp: layoutng/internal/layout_object.h:1820
    pub fn Container(&self) -> *mut LayoutObject {
        self.ContainerWithSkipInfo(std::ptr::null_mut())
    }

    // cpp: layoutng/internal/layout_object_containers.cc:202-226
    pub fn ContainerWithSkipInfo(&self, skip_info: *mut AncestorSkipInfo) -> *mut LayoutObject {
        self.CheckIsNotDestroyed();
        if !skip_info.is_null() {
            #[cfg(debug_assertions)]
            unsafe { &mut *skip_info }.AssertClean();
        }
        if self.IsTextOrSVGChild() {
            return self.Parent();
        }
        let position = self.StyleRef().GetPosition();
        if position == EPosition::kFixed {
            return self.ContainerForFixedPositionWithSkipInfo(skip_info);
        }
        if position == EPosition::kAbsolute {
            return self.ContainerForAbsolutePositionWithSkipInfo(skip_info);
        }
        if self.IsColumnSpanAll() {
            return self.ContainerForColumnSpannerWithSkipInfo(skip_info);
        }
        self.Parent()
    }

    // cpp: layoutng/internal/layout_object.h:2240
    pub fn ContainingBlock(&self) -> *mut LayoutBlock {
        self.ContainingBlockWithSkipInfo(std::ptr::null_mut())
    }

    // cpp: layoutng/internal/layout_object_containers.cc:228-252
    pub fn ContainingBlockWithSkipInfo(
        &self,
        skip_info: *mut AncestorSkipInfo,
    ) -> *mut LayoutBlock {
        self.CheckIsNotDestroyed();
        if !self.IsTextOrSVGChild() {
            if self.StyleRef().GetPosition() == EPosition::kFixed {
                return self.ContainingBlockForFixedPositionWithSkipInfo(skip_info);
            }
            if self.StyleRef().GetPosition() == EPosition::kAbsolute {
                return self.ContainingBlockForAbsolutePositionWithSkipInfo(skip_info);
            }
        }
        if self.IsColumnSpanAll() {
            return DynamicTo::<LayoutBlock>(self.ContainerForColumnSpannerWithSkipInfo(skip_info));
        }
        let mut object = self.Parent();
        while !object.is_null() && !unsafe { &*object }.IsLayoutBlock() {
            if !skip_info.is_null() {
                unsafe { &mut *skip_info }.Update(unsafe { &*object });
            }
            object = unsafe { &*object }.Parent();
        }
        DynamicTo::<LayoutBlock>(object)
    }

    // cpp: layoutng/internal/layout_object_containers.cc:254-257
    pub fn MightBeInsideFragmentationContext(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.IsInsideMulticol() || (self.IsPrintingForLayout() && !self.IsLayoutView())
    }
}

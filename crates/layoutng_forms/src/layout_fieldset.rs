#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{EDisplay, LayoutUnit, To, Traceable, Visitor};
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_assembly::internal::layout_block::LayoutBlock;
use layoutng_assembly::internal::layout_block_flow::LayoutBlockFlow;
use layoutng_assembly::internal::layout_box::LayoutBox;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng_style::style::computed_style::ComputedStyleBuilder;

// cpp: layoutng_forms/layout_fieldset.cc:13-19
#[unsafe(no_mangle)]
pub extern "Rust" fn BlockNodeGetFieldsetContentFromForms(node: &BlockNode) -> BlockNode {
    if !node.IsFieldsetContainer() {
        return BlockNode::null();
    }
    let fieldset = To::<LayoutFieldset>(node.GetLayoutBox());
    BlockNode::new(
        unsafe { &*fieldset }
            .FindAnonymousFieldsetContentBox()
            .cast(),
    )
}

// cpp: layoutng_forms/layout_fieldset.h:13-13
#[repr(C)]
pub struct LayoutFieldset {
    flow: LayoutBlockFlow,
}

impl LayoutFieldset {
    // cpp: layoutng_forms/layout_fieldset.h:15-15
    // cpp: layoutng_forms/layout_fieldset.cc:21-23
    pub fn new(element: *mut Element) -> Self {
        let mut flow = LayoutBlockFlow::new(element.cast());
        flow.SetRuntimeClass(LayoutObjectClass::Fieldset);
        flow.SetChildrenInline(false);
        Self { flow }
    }

    // cpp: layoutng_forms/layout_fieldset.h:17-20
    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        "LayoutFieldset"
    }

    // cpp: layoutng_forms/layout_fieldset.h:25-28
    pub fn CreatesNewFormattingContext(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    // cpp: layoutng_forms/layout_fieldset.h:30-35
    pub fn ContentLayoutBox(&self) -> *mut LayoutBox {
        self.CheckIsNotDestroyed();
        self.FindAnonymousFieldsetContentBox().cast()
    }

    // cpp: layoutng_forms/layout_fieldset.cc:25-39
    pub fn FindAnonymousFieldsetContentBox(&self) -> *mut LayoutBlock {
        let first_child = self.FirstChild();
        if first_child.is_null() {
            return std::ptr::null_mut();
        }
        if unsafe { &*first_child }.IsAnonymous() {
            return To::<LayoutBlock>(first_child);
        }
        let last_child = unsafe { &*first_child }.NextSibling();
        debug_assert!(last_child.is_null() || unsafe { &*last_child }.NextSibling().is_null());
        if !last_child.is_null() && unsafe { &*last_child }.IsAnonymous() {
            return To::<LayoutBlock>(last_child);
        }
        std::ptr::null_mut()
    }

    // cpp: layoutng_forms/layout_fieldset.h:37-41
    // cpp: layoutng_forms/layout_fieldset.cc:228-238
    pub fn FindInFlowLegendIn(fieldset: &LayoutBlock) -> *mut LayoutBox {
        debug_assert!(fieldset.IsFieldset());
        let mut legend = fieldset.FirstChild();
        while !legend.is_null() {
            if unsafe { &*legend }.IsRenderedLegendCandidate() {
                return To::<LayoutBox>(legend);
            }
            legend = unsafe { &*legend }.NextSibling();
        }
        std::ptr::null_mut()
    }

    pub fn FindInFlowLegend(&self) -> *mut LayoutBox {
        self.CheckIsNotDestroyed();
        Self::FindInFlowLegendIn(unsafe { &*(self as *const Self).cast::<LayoutBlock>() })
    }

    // cpp: layoutng_forms/layout_fieldset.h:44-47
    pub fn IsFieldset(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    // cpp: layoutng_forms/layout_fieldset.h:22-23
    // cpp: layoutng_forms/layout_fieldset.cc:41-66
    pub fn AddChild(&mut self, new_child: *mut LayoutObject, mut before_child: *mut LayoutObject) {
        if unsafe { &*new_child }.IsRenderedLegendCandidate() && self.FindInFlowLegend().is_null() {
            let first_child = self.FirstChild();
            self.flow.AddChild(new_child, first_child);
            return;
        }

        if !before_child.is_null() && unsafe { &*before_child }.IsRenderedLegend() {
            let before_node = unsafe { &*unsafe { &*before_child }.GetNode() }.nextSibling();
            before_child = if !before_node.is_null() {
                unsafe { &*before_node }.GetLayoutObject()
            } else {
                std::ptr::null_mut()
            };
        }

        let fieldset_content = self.FindAnonymousFieldsetContentBox();
        debug_assert!(!fieldset_content.is_null());
        unsafe { &mut *fieldset_content }.AddChild(new_child, before_child);
    }

    // cpp: layoutng_forms/layout_fieldset.h:48-48
    // cpp: layoutng_forms/layout_fieldset.cc:68-114
    pub fn InsertedIntoTree(&mut self) {
        self.flow.InsertedIntoTreeBase();
        if !self.FindAnonymousFieldsetContentBox().is_null() {
            return;
        }

        let display = match self.StyleRef().Display() {
            EDisplay::kFlex | EDisplay::kInlineFlex => EDisplay::kFlex,
            EDisplay::kGrid | EDisplay::kInlineGrid => EDisplay::kGrid,
            EDisplay::kGridLanes | EDisplay::kInlineGridLanes => EDisplay::kGridLanes,
            _ => EDisplay::kFlowRoot,
        };
        let fieldset_content = LayoutBlock::CreateAnonymousWithParentAndDisplay(
            (self as *const LayoutFieldset).cast::<LayoutObject>(),
            display,
        );
        unsafe { &mut *(self as *mut LayoutFieldset).cast::<LayoutObject>() }
            .AddChildBase(fieldset_content.cast(), std::ptr::null_mut());

        let content = unsafe { &mut *fieldset_content };
        let content_style = content.StyleRef();
        let is_fixed_container = content.ComputeIsFixedContainer(content_style);
        let is_absolute_container =
            content.ComputeIsAbsoluteContainer(content_style, is_fixed_container);
        content.SetCanContainFixedPositionObjects(is_fixed_container);
        content.SetCanContainAbsolutePositionObjects(is_absolute_container);
    }

    // cpp: layoutng_forms/layout_fieldset.h:49-51
    // cpp: layoutng_forms/layout_fieldset.cc:116-212
    pub fn UpdateAnonymousChildStyle(
        &self,
        _child: *const LayoutObject,
        child_style_builder: &mut ComputedStyleBuilder,
    ) {
        let style = self.StyleRef();
        child_style_builder.SetAlignContent(style.AlignContent());
        child_style_builder.SetAlignItems(style.AlignItems());

        child_style_builder.SetBorderBottomLeftRadius(style.BorderBottomLeftRadius());
        child_style_builder.SetBorderBottomRightRadius(style.BorderBottomRightRadius());
        child_style_builder.SetBorderTopLeftRadius(style.BorderTopLeftRadius());
        child_style_builder.SetBorderTopRightRadius(style.BorderTopRightRadius());

        child_style_builder.SetPaddingTop(style.PaddingTop());
        child_style_builder.SetPaddingRight(style.PaddingRight());
        child_style_builder.SetPaddingBottom(style.PaddingBottom());
        child_style_builder.SetPaddingLeft(style.PaddingLeft());
        child_style_builder.SetBoxDecorationBreak(style.BoxDecorationBreak());

        if style.SpecifiesColumns() && self.AllowsColumns() {
            if !style.HasAutoColumnCount() {
                child_style_builder.SetColumnCount(style.ColumnCount());
            }
            if !style.HasAutoColumnWidth() {
                child_style_builder.SetColumnWidth(style.ColumnWidth());
            }
            if !style.HasAutoColumnHeight() {
                child_style_builder.SetColumnHeight(style.ColumnHeight());
            }
            child_style_builder.SetColumnWrap(style.ColumnWrap());
        }
        child_style_builder.SetColumnGap(style.ColumnGap());
        child_style_builder.SetColumnFill(style.GetColumnFill());
        child_style_builder.SetColumnRuleColor(style.ColumnRuleColor());
        child_style_builder.SetRowRuleColor(style.RowRuleColor());
        child_style_builder.SetColumnRuleStyle(style.ColumnRuleStyle());
        child_style_builder.SetRowRuleStyle(style.RowRuleStyle());
        child_style_builder.SetColumnRuleWidth(style.ColumnRuleWidth());
        child_style_builder.SetRowRuleWidth(style.RowRuleWidth());
        child_style_builder.SetColumnRuleBreak(style.ColumnRuleBreak());
        child_style_builder.SetRowRuleBreak(style.RowRuleBreak());
        child_style_builder.SetRuleOverlap(style.RuleOverlap());
        child_style_builder.SetColumnRuleVisibilityItems(style.ColumnRuleVisibilityItems());
        child_style_builder.SetRowRuleVisibilityItems(style.RowRuleVisibilityItems());
        child_style_builder.SetColumnRuleInsetCapStart(style.ColumnRuleInsetCapStart());
        child_style_builder.SetColumnRuleInsetCapEnd(style.ColumnRuleInsetCapEnd());
        child_style_builder.SetColumnRuleInsetJunctionStart(style.ColumnRuleInsetJunctionStart());
        child_style_builder.SetColumnRuleInsetJunctionEnd(style.ColumnRuleInsetJunctionEnd());
        child_style_builder.SetRowRuleInsetCapStart(style.RowRuleInsetCapStart());
        child_style_builder.SetRowRuleInsetCapEnd(style.RowRuleInsetCapEnd());
        child_style_builder.SetRowRuleInsetJunctionStart(style.RowRuleInsetJunctionStart());
        child_style_builder.SetRowRuleInsetJunctionEnd(style.RowRuleInsetJunctionEnd());

        child_style_builder.SetFlexDirection(style.FlexDirection());
        child_style_builder.SetFlexWrap(style.FlexWrap());

        child_style_builder.SetGridAutoColumns(style.GridAutoColumns());
        child_style_builder.SetGridAutoFlow(style.GetGridAutoFlow());
        child_style_builder.SetGridAutoRows(style.GridAutoRows());
        child_style_builder.SetGridColumnEnd(style.GridColumnEnd());
        child_style_builder.SetGridColumnStart(style.GridColumnStart());
        child_style_builder.SetGridRowEnd(style.GridRowEnd());
        child_style_builder.SetGridRowStart(style.GridRowStart());
        child_style_builder.SetGridTemplateColumns(style.SpecifiedGridTemplateColumns());
        child_style_builder.SetGridTemplateRows(style.SpecifiedGridTemplateRows());
        child_style_builder.SetGridTemplateAreas(style.GridTemplateAreas());
        child_style_builder.SetRowGap(style.RowGap());
        child_style_builder.SetJustifyContent(style.JustifyContent());
        child_style_builder.SetJustifyItems(style.JustifyItems());
        child_style_builder.SetOverflowX(style.OverflowX());
        child_style_builder.SetOverflowY(style.OverflowY());
        child_style_builder.SetScrollbarGutter(style.ScrollbarGutter());
        child_style_builder.SetUnicodeBidi(style.GetUnicodeBidi());
    }

    // cpp: layoutng_forms/layout_fieldset.h:55-59
    pub fn ComputeCanCompositeBackgroundAttachmentFixed(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng_forms/layout_fieldset.h:61-64
    pub fn RespectsCSSOverflow(&self) -> bool {
        self.CheckIsNotDestroyed();
        false
    }

    // cpp: layoutng_forms/layout_fieldset.h:65-67
    // cpp: layoutng_forms/layout_fieldset.cc:214-226
    pub fn ScrollWidth(&self) -> LayoutUnit {
        let content = self.FindAnonymousFieldsetContentBox();
        if !content.is_null() {
            return unsafe { &*content }.ScrollWidth();
        }
        self.flow.ScrollWidthBase()
    }

    pub fn ScrollHeight(&self) -> LayoutUnit {
        let content = self.FindAnonymousFieldsetContentBox();
        if !content.is_null() {
            return unsafe { &*content }.ScrollHeight();
        }
        self.flow.ScrollHeightBase()
    }
}

impl Deref for LayoutFieldset {
    type Target = LayoutBlockFlow;
    fn deref(&self) -> &Self::Target {
        &self.flow
    }
}

impl DerefMut for LayoutFieldset {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.flow
    }
}

impl foundation::DowncastFrom<LayoutObject> for LayoutFieldset {
    // cpp: layoutng_forms/layout_fieldset.h:70-75
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsFieldset()
    }
}

impl foundation::DowncastFrom<LayoutBox> for LayoutFieldset {
    fn AllowFrom(box_: &LayoutBox) -> bool {
        box_.IsFieldset()
    }
}

const _: () = assert!(std::mem::offset_of!(LayoutFieldset, flow) == 0);

impl Traceable for LayoutFieldset {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.flow.Trace(visitor);
    }
}

pub fn FieldsetAddChild(
    flow: &mut LayoutBlockFlow,
    child: *mut LayoutObject,
    before_child: *mut LayoutObject,
) {
    unsafe { &mut *(flow as *mut LayoutBlockFlow).cast::<LayoutFieldset>() }
        .AddChild(child, before_child);
}

pub fn FieldsetInsertedIntoTree(flow: &mut LayoutBlockFlow) {
    unsafe { &mut *(flow as *mut LayoutBlockFlow).cast::<LayoutFieldset>() }.InsertedIntoTree();
}

pub fn FieldsetUpdateAnonymousChildStyle(
    flow: &LayoutBlockFlow,
    child: *const LayoutObject,
    builder: &mut ComputedStyleBuilder,
) {
    unsafe { &*(flow as *const LayoutBlockFlow).cast::<LayoutFieldset>() }
        .UpdateAnonymousChildStyle(child, builder);
}

pub fn FieldsetContentLayoutBox(flow: &mut LayoutBlockFlow) -> *mut LayoutBox {
    unsafe { &mut *(flow as *mut LayoutBlockFlow).cast::<LayoutFieldset>() }.ContentLayoutBox()
}

pub fn FieldsetScrollWidth(flow: &LayoutBlockFlow) -> LayoutUnit {
    unsafe { &*(flow as *const LayoutBlockFlow).cast::<LayoutFieldset>() }.ScrollWidth()
}

pub fn FieldsetScrollHeight(flow: &LayoutBlockFlow) -> LayoutUnit {
    unsafe { &*(flow as *const LayoutBlockFlow).cast::<LayoutFieldset>() }.ScrollHeight()
}

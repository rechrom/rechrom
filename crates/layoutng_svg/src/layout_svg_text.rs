#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{DowncastFrom, DynamicTo, Traceable, Visitor};
use layoutng_assembly::internal::block_node::BlockNode;
use layoutng_assembly::internal::constraint_space_builder::ConstraintSpaceBuilder;
use layoutng_assembly::internal::layout_block::LayoutBlock;
use layoutng_assembly::internal::layout_block_flow::LayoutBlockFlow;
use layoutng_assembly::internal::layout_box::LayoutBox;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{LayoutObject, LayoutObjectClass};
use layoutng_assembly::internal::layout_pass_scope::LayoutObjectFactoryScope;
use layoutng_assembly::internal::svg_layout_info::{SVGLayoutInfo, SVGLayoutResult};
use layoutng_geometry::geometry::logical_size::LogicalSize;
use layoutng_style::style::computed_style::ComputedStyle;

// cpp: layoutng_svg/layout_svg_text.h:10-33
#[repr(C)]
pub struct LayoutSVGText {
    flow: LayoutBlockFlow,
    needs_text_metrics_update: bool,
}

impl LayoutSVGText {
    // cpp: layoutng_svg/svg_layout_objects.cc:105-105
    pub fn new(element: *mut Element) -> Self {
        let flow = LayoutBlockFlow::new(element.cast());
        flow.SetRuntimeClass(LayoutObjectClass::SvgText);
        Self {
            flow,
            needs_text_metrics_update: true,
        }
    }

    // cpp: layoutng_svg/layout_svg_text.h:14-25
    pub fn SetNeedsTextMetricsUpdate(&mut self) {
        self.needs_text_metrics_update = true;
        self.SetNeedsCollectInlines();
    }
    // cpp: layoutng_svg/svg_layout_objects.cc:107-109
    pub fn NeedsTextMetricsUpdate(&self) -> bool {
        self.needs_text_metrics_update
    }
    pub fn GetName(&self) -> &'static str {
        "LayoutSVGText"
    }
    pub fn CreatesNewFormattingContext(&self) -> bool {
        true
    }
    pub fn IsSVG(&self) -> bool {
        true
    }
    pub fn IsSVGText(&self) -> bool {
        true
    }

    // cpp: layoutng_svg/svg_layout_objects.cc:111-114
    pub fn IsChildAllowed(&self, child: *mut LayoutObject, _style: &ComputedStyle) -> bool {
        !child.is_null() && (unsafe { &*child }.IsText() || unsafe { &*child }.IsInline())
    }

    // cpp: layoutng_svg/svg_layout_objects.cc:116-125
    pub fn AddChild(&mut self, child: *mut LayoutObject, before: *mut LayoutObject) {
        assert!(!child.is_null());
        assert!(self.IsChildAllowed(child, unsafe { &*child }.StyleRef()));
        self.flow.AddChild(child, before);
        self.SetNeedsCollectInlines();
    }
    pub fn RemoveChild(&mut self, child: *mut LayoutObject) {
        self.flow.RemoveChild(child);
        self.SetNeedsCollectInlines();
    }

    // cpp: layoutng_svg/svg_layout_objects.cc:127-134
    pub fn InsertedIntoTree(&mut self) {
        let box_ = unsafe { &mut *(self as *mut Self).cast::<LayoutBox>() };
        box_.InsertedIntoTree();
        let mut ancestor = self.Parent();
        while !ancestor.is_null() {
            let block = DynamicTo::<LayoutBlock>(ancestor);
            if !block.is_null() {
                unsafe { &mut *block }.AddSvgTextDescendant(box_);
            }
            ancestor = unsafe { &*ancestor }.Parent();
        }
    }

    // cpp: layoutng_svg/svg_layout_objects.cc:136-143
    pub fn WillBeRemovedFromTree(&mut self) {
        let box_ = unsafe { &mut *(self as *mut Self).cast::<LayoutBox>() };
        let mut ancestor = self.Parent();
        while !ancestor.is_null() {
            let block = DynamicTo::<LayoutBlock>(ancestor);
            if !block.is_null() {
                unsafe { &mut *block }.RemoveSvgTextDescendant(box_);
            }
            ancestor = unsafe { &*ancestor }.Parent();
        }
        box_.WillBeRemovedFromTree();
    }

    // cpp: layoutng_svg/svg_layout_objects_layout.cc:31-48
    pub fn UpdateSVGLayout(&mut self, info: &SVGLayoutInfo) -> SVGLayoutResult {
        if self.needs_text_metrics_update || info.scale_factor_changed || info.viewport_changed {
            let mut child = self.FirstChild();
            let this = (self as *mut Self).cast::<LayoutObject>();
            while !child.is_null() {
                if unsafe { &*child }.IsSVGInlineText() {
                    let objects = LayoutObjectFactoryScope::Objects();
                    let update = unsafe { objects.as_ref() }
                        .and_then(|objects| objects.svg_inline_text_update_scaled_font)
                        .expect("SVG inline text font update is not installed");
                    update(unsafe { &mut *child });
                }
                child = unsafe { &*child }.NextInPreOrder(this);
            }
            self.SetNeedsCollectInlines();
            self.needs_text_metrics_update = false;
        }
        let style = self.StyleRef();
        let mut builder = ConstraintSpaceBuilder::new_without_parent_space(
            style.GetWritingMode(),
            style.GetWritingDirection(),
            true,
            true,
            false,
        );
        builder.SetAvailableSize(LogicalSize::default());
        let space = builder.ToConstraintSpace();
        BlockNode::new((self as *mut Self).cast()).Layout(
            &space,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null(),
        );
        SVGLayoutResult::new(true, true)
    }
}

impl Deref for LayoutSVGText {
    type Target = LayoutBlockFlow;
    fn deref(&self) -> &Self::Target {
        &self.flow
    }
}
impl DerefMut for LayoutSVGText {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.flow
    }
}
const _: () = assert!(std::mem::offset_of!(LayoutSVGText, flow) == 0);
impl Traceable for LayoutSVGText {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.flow.Trace(visitor);
    }
}
impl DowncastFrom<LayoutObject> for LayoutSVGText {
    // cpp: layoutng_svg/layout_svg_text.h:35-38
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsSVGText()
    }
}

pub fn SvgTextAddChild(
    flow: &mut LayoutBlockFlow,
    child: *mut LayoutObject,
    before: *mut LayoutObject,
) {
    unsafe { &mut *(flow as *mut LayoutBlockFlow).cast::<LayoutSVGText>() }.AddChild(child, before)
}
pub fn SvgTextRemoveChild(flow: &mut LayoutBlockFlow, child: *mut LayoutObject) {
    unsafe { &mut *(flow as *mut LayoutBlockFlow).cast::<LayoutSVGText>() }.RemoveChild(child)
}
pub fn SvgTextInsertedIntoTree(flow: &mut LayoutBlockFlow) {
    unsafe { &mut *(flow as *mut LayoutBlockFlow).cast::<LayoutSVGText>() }.InsertedIntoTree()
}
pub fn SvgTextWillBeRemovedFromTree(flow: &mut LayoutBlockFlow) {
    unsafe { &mut *(flow as *mut LayoutBlockFlow).cast::<LayoutSVGText>() }.WillBeRemovedFromTree()
}
pub fn SvgTextSetNeedsMetricsUpdate(object: &mut LayoutObject) {
    unsafe { &mut *(object as *mut LayoutObject).cast::<LayoutSVGText>() }
        .SetNeedsTextMetricsUpdate()
}

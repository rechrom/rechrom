#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{DynamicTo, IsA, To, Traceable, Visitor};
use layoutng_assembly::internal::form_node_metadata::HTMLTextAreaElement;
use layoutng_assembly::internal::layout_block_flow::LayoutBlockFlow;
use layoutng_assembly::internal::layout_box_model_object::LayoutBoxModelObject;
use layoutng_assembly::internal::layout_invalidation_reason;
use layoutng_assembly::internal::layout_node_metadata::Element;
use layoutng_assembly::internal::layout_object::{
    LayoutObject, LayoutObjectClass, StyleChangeContext,
};
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::style_difference::StyleDifference;

// cpp: layoutng_forms/layout_text_control_inner_editor.h:12-16,46-48
#[repr(C)]
pub struct LayoutTextControlInnerEditor {
    flow: LayoutBlockFlow,
    is_multiline_: bool,
}

impl LayoutTextControlInnerEditor {
    // cpp: layoutng_forms/layout_text_control_inner_editor.cc:12-14
    pub fn new(element: *mut Element) -> Self {
        let flow = LayoutBlockFlow::new(element.cast());
        let is_multiline_ = IsA::<HTMLTextAreaElement>(unsafe { &*element }.OwnerShadowHost());
        flow.SetRuntimeClass(LayoutObjectClass::TextControlInnerEditor);
        Self {
            flow,
            is_multiline_,
        }
    }

    // cpp: layoutng_forms/layout_text_control_inner_editor.h:18-21
    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        "LayoutTextControlInnerEditor"
    }

    // cpp: layoutng_forms/layout_text_control_inner_editor.h:23-26
    pub fn IsTextControlInnerEditor(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    // cpp: layoutng_forms/layout_text_control_inner_editor.h:28-37
    pub fn IsMultiline(&self) -> bool {
        self.CheckIsNotDestroyed();
        self.is_multiline_
    }

    pub fn AllowsInlineChildren(&self) -> bool {
        self.flow.AllowsInlineChildrenBase() && !self.IsMultiline()
    }

    // cpp: layoutng_forms/layout_text_control_inner_editor.h:39-40
    // cpp: layoutng_forms/layout_text_control_inner_editor.cc:16-66
    pub fn AddChild(&mut self, new_child: *mut LayoutObject, before_child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        if !self.is_multiline_ {
            self.flow.AddChild(new_child, before_child);
            return;
        }

        if before_child.is_null() {
            let last_anonymous = DynamicTo::<LayoutBlockFlow>(self.LastChild());
            if !last_anonymous.is_null() {
                let last_child = unsafe { &*last_anonymous }.LastChild();
                if !unsafe { &*last_child }.IsBR() {
                    unsafe { &mut *last_anonymous }.AddChild(new_child, std::ptr::null_mut());
                    return;
                }
            }
            let anonymous = LayoutBlockFlow::CreateAnonymous(&self.flow, self.StyleRef());
            self.flow.AddChild(anonymous.cast(), std::ptr::null_mut());
            unsafe { &mut *anonymous }.AddChild(new_child, std::ptr::null_mut());
            return;
        }

        debug_assert!(!self.FirstChild().is_null());
        let before_parent = To::<LayoutBlockFlow>(unsafe { &*before_child }.Parent());
        if !unsafe { &*before_parent }.IsAnonymous() {
            debug_assert_eq!(before_parent, &mut self.flow as *mut LayoutBlockFlow);
            let previous = unsafe { &*before_child }.PreviousSibling();
            if !previous.is_null() {
                debug_assert!(unsafe { &*previous }.IsAnonymousBlockFlow());
                let previous_last = unsafe { &*previous }.SlowLastChild();
                if previous_last.is_null() || !unsafe { &*previous_last }.IsBR() {
                    unsafe { &mut *previous }.AddChild(new_child, std::ptr::null_mut());
                    return;
                }
            }
            let anonymous = LayoutBlockFlow::CreateAnonymous(&self.flow, self.StyleRef());
            self.flow.AddChild(anonymous.cast(), before_child);
            unsafe { &mut *anonymous }.AddChild(new_child, std::ptr::null_mut());
            return;
        }

        if !unsafe { &*new_child }.IsBR() {
            unsafe { &mut *before_parent }.AddChild(new_child, before_child);
            return;
        }
        let anonymous = LayoutBlockFlow::CreateAnonymous(&self.flow, self.StyleRef());
        self.flow.AddChild(anonymous.cast(), before_parent.cast());
        let first_child = unsafe { &*before_parent }.FirstChild();
        unsafe { &mut *before_parent }.MoveChildrenTo(
            anonymous.cast::<LayoutBoxModelObject>(),
            first_child,
            before_child,
            std::ptr::null_mut(),
            true,
        );
        unsafe { &mut *anonymous }.AddChild(new_child, std::ptr::null_mut());
    }

    // cpp: layoutng_forms/layout_text_control_inner_editor.h:41-44
    // cpp: layoutng_forms/layout_text_control_inner_editor.cc:68-81
    pub fn StyleDidChange(
        &mut self,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        style_change_context: &StyleChangeContext,
    ) {
        self.flow
            .StyleDidChange(diff, old_style, new_style, style_change_context);
        if self.is_multiline_
            && !old_style.is_null()
            && unsafe { &*old_style }.UsedUserModify() != self.StyleRef().UsedUserModify()
            && self.FirstChild().is_null()
        {
            self.SetNeedsLayoutAndIntrinsicWidthsRecalc(
                &raw const layout_invalidation_reason::kStyleChange,
            );
        }
    }
}

// The C++ class embeds LayoutBlockFlow as its first base. Rust keeps both the
// layout address and the checked downcast invariant explicit.
impl Deref for LayoutTextControlInnerEditor {
    type Target = LayoutBlockFlow;
    fn deref(&self) -> &Self::Target {
        &self.flow
    }
}

impl DerefMut for LayoutTextControlInnerEditor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.flow
    }
}

impl foundation::DowncastFrom<LayoutObject> for LayoutTextControlInnerEditor {
    // cpp: layoutng_forms/layout_text_control_inner_editor.h:50-55
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsTextControlInnerEditor()
    }
}

const _: () = assert!(std::mem::offset_of!(LayoutTextControlInnerEditor, flow) == 0);

impl Traceable for LayoutTextControlInnerEditor {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.flow.Trace(visitor);
    }
}

pub fn InnerEditorAllowsInlineChildren(flow: &LayoutBlockFlow) -> bool {
    unsafe { &*(flow as *const LayoutBlockFlow).cast::<LayoutTextControlInnerEditor>() }
        .AllowsInlineChildren()
}

pub fn InnerEditorAddChild(
    flow: &mut LayoutBlockFlow,
    child: *mut LayoutObject,
    before_child: *mut LayoutObject,
) {
    unsafe { &mut *(flow as *mut LayoutBlockFlow).cast::<LayoutTextControlInnerEditor>() }
        .AddChild(child, before_child);
}

pub fn InnerEditorStyleDidChange(
    flow: &mut LayoutBlockFlow,
    diff: StyleDifference,
    old_style: *const ComputedStyle,
    new_style: &ComputedStyle,
    context: &StyleChangeContext,
) {
    unsafe { &mut *(flow as *mut LayoutBlockFlow).cast::<LayoutTextControlInnerEditor>() }
        .StyleDidChange(diff, old_style, new_style, context);
}

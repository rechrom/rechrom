// C++: layoutng_inline/layout_ruby_as_block.h/.cc.
#![allow(non_snake_case)]

use std::ops::{Deref, DerefMut};

use foundation::{EDisplay, MakeGarbageCollected, Traceable, Visitor};
use layoutng::internal::layout_block_flow::LayoutBlockFlow;
use layoutng::internal::layout_inline::LayoutInline;
use layoutng::internal::layout_node_metadata::Element;
use layoutng::internal::layout_object::{LayoutObject, StyleChangeContext};
use layoutng_style::style::anonymous_style::CreateAnonymousStyleBuilderWithDisplay;
use layoutng_style::style::computed_style::ComputedStyle;
use layoutng_style::style::style_difference::StyleDifference;

// cpp: layoutng_inline/layout_ruby_as_block.h:13-22
#[repr(C)]
pub struct LayoutRubyAsBlock {
    block_flow_: LayoutBlockFlow,
}

const _: () = assert!(std::mem::offset_of!(LayoutRubyAsBlock, block_flow_) == 0);

// cpp: layoutng_inline/layout_ruby_as_block.h:41-46
impl foundation::DowncastFrom<LayoutObject> for LayoutRubyAsBlock {
    fn AllowFrom(object: &LayoutObject) -> bool {
        object.IsRuby() && !object.IsLayoutInline()
    }
}

impl LayoutRubyAsBlock {
    // cpp: layoutng_inline/layout_ruby_as_block.cc:12-15
    pub fn new(element: *mut Element) -> Self {
        Self {
            block_flow_: LayoutBlockFlow::new(element.cast()),
        }
    }

    // cpp: layoutng_inline/layout_ruby_as_block.h:24-31
    pub fn GetName(&self) -> &'static str {
        self.CheckIsNotDestroyed();
        "LayoutRubyAsBlock"
    }
    pub fn IsRuby(&self) -> bool {
        self.CheckIsNotDestroyed();
        true
    }

    // cpp: layoutng_inline/layout_ruby_as_block.h:33-34
    // cpp: layoutng_inline/layout_ruby_as_block.cc:17-36
    pub fn AddChild(&mut self, child: *mut LayoutObject, before_child: *mut LayoutObject) {
        self.CheckIsNotDestroyed();
        let mut inline_ruby = self.FirstChild();
        if inline_ruby.is_null() {
            inline_ruby =
                MakeGarbageCollected(LayoutInline::new(std::ptr::null_mut())) as *mut LayoutObject;
            unsafe { &mut *inline_ruby }.SetInputOwnerForAnonymous(self);
            let mut style_builder = CreateAnonymousStyleBuilderWithDisplay(
                self.StyleRef(),
                EDisplay::kRuby,
                self.StyleRef().AppliedTextDecorationData(),
            );
            unsafe { &mut *inline_ruby }.SetStyle(style_builder.TakeStyle());
            self.block_flow_.AddChildDefault(inline_ruby);
        } else if before_child == inline_ruby {
            let first_child = unsafe { &*inline_ruby }.SlowFirstChild();
            unsafe { &mut *inline_ruby }.AddChild(child, first_child);
            return;
        }
        unsafe { &mut *inline_ruby }.AddChild(child, before_child);
    }

    pub fn AddChildDefault(&mut self, child: *mut LayoutObject) {
        self.AddChild(child, std::ptr::null_mut());
    }

    // cpp: layoutng_inline/layout_ruby_as_block.h:35-38
    // cpp: layoutng_inline/layout_ruby_as_block.cc:38-59
    pub fn StyleDidChange(
        &mut self,
        diff: StyleDifference,
        old_style: *const ComputedStyle,
        new_style: &ComputedStyle,
        style_change_context: &StyleChangeContext,
    ) {
        self.CheckIsNotDestroyed();
        self.block_flow_
            .StyleDidChange(diff, old_style, new_style, style_change_context);
        self.PropagateStyleToAnonymousChildren();
        let inline_ruby = self.FirstChild();
        if !inline_ruby.is_null() {
            let mut style_builder = CreateAnonymousStyleBuilderWithDisplay(
                new_style,
                unsafe { &*inline_ruby }.StyleRef().Display(),
                new_style.AppliedTextDecorationData(),
            );
            self.UpdateAnonymousChildStyle(inline_ruby, &mut style_builder);
            unsafe { &mut *inline_ruby }.SetStyle(style_builder.TakeStyle());
        }
    }
}

impl Deref for LayoutRubyAsBlock {
    type Target = LayoutBlockFlow;
    fn deref(&self) -> &Self::Target {
        &self.block_flow_
    }
}
impl DerefMut for LayoutRubyAsBlock {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.block_flow_
    }
}

// LayoutRubyAsBlock introduces no traced fields beyond LayoutBlockFlow.
impl Traceable for LayoutRubyAsBlock {
    fn Trace(&self, visitor: &mut Visitor<'_>) {
        self.block_flow_.Trace(visitor);
    }
}

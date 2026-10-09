// Copyright (C) 2007, 2008, 2012 Apple Inc. All rights reserved.
// cpp: third_party/blink/renderer/core/css/css_keyframe_rule.h
// cpp: third_party/blink/renderer/core/css/css_keyframe_rule.cc
// Oilpan Trace, wrapper macros and default destructor have no Rust body.

#![allow(non_snake_case)]

use crate::css_keyframes_rule::{
    CSSKeyframesBackend, CSSKeyframesCSSRuleType, CSSKeyframesMutationTarget, CSSKeyframesRule,
};
use crate::style_rule::{StyleRuleBase, StyleRuleDependencies};
use crate::style_rule_keyframe::StyleRuleKeyframe;
use foundation::String;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

/// The actual KeyframeStyleRuleCSSStyleDeclaration operation used by Reattach.
/// Its implementation must retain its CSSKeyframeRule owner, as CSSOM does.
pub trait KeyframeStyleDeclaration<D: CSSKeyframesBackend> {
    fn Reattach(&self, properties: Rc<D::PropertySet>);
}

// cpp: css_keyframe_rule.h:40-64
pub struct CSSKeyframeRule<D: CSSKeyframesBackend> {
    keyframe_: RefCell<Rc<RefCell<StyleRuleKeyframe<D>>>>,
    parent_rule_: RefCell<Option<Rc<CSSKeyframesRule<D>>>>,
    properties_cssom_wrapper_: RefCell<Option<Weak<D::CSSStyleDeclaration>>>,
}

impl<D: CSSKeyframesBackend> CSSKeyframeRule<D> {
    // cpp: css_keyframe_rule.cc:39-43
    pub fn new(
        keyframe: Rc<RefCell<StyleRuleKeyframe<D>>>,
        parent: Option<Rc<CSSKeyframesRule<D>>>,
    ) -> Rc<Self> {
        Rc::new(Self {
            keyframe_: RefCell::new(keyframe),
            parent_rule_: RefCell::new(parent),
            properties_cssom_wrapper_: RefCell::new(None),
        })
    }

    pub fn GetType(&self) -> CSSKeyframesCSSRuleType {
        CSSKeyframesCSSRuleType::kKeyframeRule
    }
    pub fn cssText(&self) -> String {
        self.keyframe_.borrow().borrow().CssText()
    }
    pub fn keyText(&self) -> String {
        self.keyframe_.borrow().borrow().KeyText()
    }
    pub fn parentRule(&self) -> Option<Rc<CSSKeyframesRule<D>>> {
        self.parent_rule_.borrow().clone()
    }
    pub fn SetParentRule(&self, parent: Option<Rc<CSSKeyframesRule<D>>>) {
        *self.parent_rule_.borrow_mut() = parent;
    }
    pub fn parentStyleSheet(&self) -> Option<Rc<D::CSSStyleSheet>> {
        self.parentRule()
            .and_then(|parent| parent.parentStyleSheet())
    }

    // cpp: css_keyframe_rule.cc:47-64. Throwing records an exception but does
    // not skip the parent's diff/version notification, including invalid keys.
    pub fn setKeyText(
        &self,
        execution_context: &D::ExecutionContext,
        key_text: &String,
        exception_state: &mut D::ExceptionState,
    ) {
        let _mutation_scope = D::BeginRuleMutation(CSSKeyframesMutationTarget::Keyframe(self));
        if !self
            .keyframe_
            .borrow()
            .borrow_mut()
            .SetKeyText(execution_context, key_text)
        {
            let mut message: Vec<u16> = "The key '".encode_utf16().collect();
            message.extend_from_slice(key_text.Span16().unwrap_or_default());
            message.extend("' is invalid and cannot be parsed".encode_utf16());
            D::ThrowSyntaxError(exception_state, &String::from_utf16(&message));
        }
        if let Some(parent) = self.parentRule() {
            if let Some(sheet) = parent.parentStyleSheet() {
                D::NotifyDiffUnrepresentable(&sheet);
            }
            parent.StyleChanged();
        }
    }

    // cpp: css_keyframe_rule.cc:66-73
    pub fn style(self: &Rc<Self>) -> Rc<D::CSSStyleDeclaration> {
        if let Some(wrapper) = self
            .properties_cssom_wrapper_
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
        {
            return wrapper;
        }
        let properties = self.keyframe_.borrow().borrow().MutableProperties();
        let wrapper = D::NewKeyframeStyleDeclaration(properties, Rc::clone(self));
        *self.properties_cssom_wrapper_.borrow_mut() = Some(Rc::downgrade(&wrapper));
        wrapper
    }

    // Typed downcast of the existing StyleRuleBase, retaining its allocation.
    pub fn Reattach<S>(&self, rule: &StyleRuleBase<S>)
    where
        S: StyleRuleDependencies<StyleRuleKeyframe = RefCell<StyleRuleKeyframe<D>>>,
    {
        let StyleRuleBase::Keyframe(keyframe) = rule else {
            panic!("CSSKeyframeRule::Reattach requires a keyframe rule");
        };
        self.ReattachKeyframe(Rc::clone(keyframe));
    }

    // cpp: css_keyframe_rule.cc:75-80
    pub fn ReattachKeyframe(&self, keyframe: Rc<RefCell<StyleRuleKeyframe<D>>>) {
        *self.keyframe_.borrow_mut() = keyframe;
        let wrapper = self
            .properties_cssom_wrapper_
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade);
        if let Some(wrapper) = wrapper {
            wrapper.Reattach(self.keyframe_.borrow().borrow().MutableProperties());
        }
    }
}

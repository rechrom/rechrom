// Copyright (C) 2007, 2008, 2012 Apple Inc. All rights reserved.
// cpp: third_party/blink/renderer/core/css/css_keyframes_rule.h
// cpp: third_party/blink/renderer/core/css/css_keyframes_rule.cc
// Oilpan Trace, wrapper/casting macros and default destructors are omitted.

#![allow(non_snake_case)]

use crate::css_keyframe_rule::{CSSKeyframeRule, KeyframeStyleDeclaration};
use crate::css_markup::SerializeIdentifierTo;
use crate::style_rule::{RuleType, StyleRuleBase, StyleRuleDependencies};
use crate::style_rule_keyframe::{KeyframeOffset, StyleRuleKeyframe, StyleRuleKeyframeBackend};
use foundation::{AtomicString, String};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};

// Required CSSRule::Type dependency (css_rule.h:61-62). These CSSOM
// web-exposed tags differ from StyleRuleBase's kKeyframes/kKeyframe tags.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CSSKeyframesCSSRuleType {
    kKeyframesRule = 7,
    kKeyframeRule = 8,
}

pub enum CSSKeyframesMutationTarget<'a, D: CSSKeyframesBackend> {
    Keyframe(&'a CSSKeyframeRule<D>),
    Keyframes(&'a CSSKeyframesRule<D>),
}

/// Untranslated CSSOM, parser and execution-context dependencies. All calls
/// require a real implementation. RuleMutationScope must implement the source
/// scope's exit behavior in Drop. Declarations and live rule lists retain their
/// supplied owner strongly; caches below are weak to avoid Rc ownership cycles.
pub trait CSSKeyframesBackend: StyleRuleKeyframeBackend {
    type CSSStyleSheet;
    type CSSParserContext;
    type SecureContextMode;
    type ExceptionState;
    type RuleMutationScope;
    type CSSStyleDeclaration: KeyframeStyleDeclaration<Self>;
    type CSSRuleList: CSSKeyframesRuleList<Self>;
    type Document;

    fn BeginRuleMutation(target: CSSKeyframesMutationTarget<'_, Self>) -> Self::RuleMutationScope;
    fn NotifyDiffUnrepresentable(sheet: &Self::CSSStyleSheet);
    fn ThrowSyntaxError(state: &mut Self::ExceptionState, message: &String);
    fn GetSecureContextMode(context: &Self::ExecutionContext) -> Self::SecureContextMode;
    fn ParserContext(
        rule: &CSSKeyframesRule<Self>,
        mode: Self::SecureContextMode,
    ) -> Rc<Self::CSSParserContext>;
    fn NewCSSParserContext(
        context: Rc<Self::CSSParserContext>,
        sheet: Option<Rc<Self::CSSStyleSheet>>,
    ) -> Rc<Self::CSSParserContext>;
    fn ParseKeyframeRule(
        context: &Self::CSSParserContext,
        text: &String,
    ) -> Option<Rc<RefCell<StyleRuleKeyframe<Self>>>>;
    fn ParseKeyframeKeyListWithContext(
        context: &Self::CSSParserContext,
        text: &String,
    ) -> Option<Vec<KeyframeOffset>>;
    fn NewKeyframeStyleDeclaration(
        properties: Rc<Self::PropertySet>,
        owner: Rc<CSSKeyframeRule<Self>>,
    ) -> Rc<Self::CSSStyleDeclaration>;
    fn NewLiveCSSRuleList(owner: Rc<CSSKeyframesRule<Self>>) -> Rc<Self::CSSRuleList>;
    fn SingleOwnerDocument(sheet: Option<&Self::CSSStyleSheet>) -> Option<Rc<Self::Document>>;
    fn CountCSSKeyframesRuleAnonymousIndexedGetter(document: &Self::Document);
}

/// Required live-list operations: implementations forward every access to the
/// retained owner, so appending, deleting and reattaching are observed live.
pub trait CSSKeyframesRuleList<D: CSSKeyframesBackend> {
    fn length(&self) -> usize;
    fn Item(&self, index: usize) -> Option<Rc<CSSKeyframeRule<D>>>;
}

// cpp: css_keyframes_rule.h:42-81
pub struct StyleRuleKeyframes<D: CSSKeyframesBackend> {
    keyframes_: Vec<Rc<RefCell<StyleRuleKeyframe<D>>>>,
    name_: AtomicString,
    version_: u32,
    is_prefixed_: bool,
}

impl<D: CSSKeyframesBackend> Default for StyleRuleKeyframes<D> {
    // cpp: css_keyframes_rule.cc:46-47
    fn default() -> Self {
        Self {
            keyframes_: Vec::new(),
            name_: AtomicString::default(),
            version_: 0,
            is_prefixed_: false,
        }
    }
}
impl<D: CSSKeyframesBackend> Clone for StyleRuleKeyframes<D> {
    // cpp: css_keyframes_rule.cc:49. The copy constructor is shallow.
    fn clone(&self) -> Self {
        Self::new(
            self.keyframes_.clone(),
            self.name_.clone(),
            self.version_,
            self.is_prefixed_,
        )
    }
}
impl<D: CSSKeyframesBackend> StyleRuleKeyframes<D> {
    // cpp: css_keyframes_rule.cc:51-60
    pub fn new(
        keyframes: Vec<Rc<RefCell<StyleRuleKeyframe<D>>>>,
        name: AtomicString,
        version: u32,
        is_vendor_prefixed: bool,
    ) -> Self {
        Self {
            keyframes_: keyframes,
            name_: name,
            version_: version & 0x7fff_ffff,
            is_prefixed_: is_vendor_prefixed,
        }
    }
    pub fn GetType(&self) -> RuleType {
        RuleType::kKeyframes
    }
    pub fn Keyframes(&self) -> &[Rc<RefCell<StyleRuleKeyframe<D>>>] {
        &self.keyframes_
    }
    // cpp: css_keyframes_rule.cc:64-69
    pub fn ParserAppendKeyframe(&mut self, keyframe: Option<Rc<RefCell<StyleRuleKeyframe<D>>>>) {
        if let Some(keyframe) = keyframe {
            self.keyframes_.push(keyframe);
        }
    }
    // cpp: css_keyframes_rule.cc:71-74
    pub fn WrapperAppendKeyframe(&mut self, keyframe: Rc<RefCell<StyleRuleKeyframe<D>>>) {
        self.keyframes_.push(keyframe);
        self.StyleChanged();
    }
    // cpp: css_keyframes_rule.cc:76-79
    pub fn WrapperRemoveKeyframe(&mut self, index: usize) {
        self.keyframes_.remove(index);
        self.StyleChanged();
    }
    pub fn GetName(&self) -> AtomicString {
        self.name_.clone()
    }
    pub fn SetName(&mut self, name: &String) {
        self.name_ = if name.IsNull() {
            AtomicString::default()
        } else {
            AtomicString::from_utf16(name.Span16().unwrap_or_default())
        };
    }
    pub fn IsVendorPrefixed(&self) -> bool {
        self.is_prefixed_
    }
    pub fn SetVendorPrefixed(&mut self, is_prefixed: bool) {
        self.is_prefixed_ = is_prefixed;
    }
    pub fn StyleChanged(&mut self) {
        self.version_ = self.version_.wrapping_add(1) & 0x7fff_ffff;
    }
    pub fn Version(&self) -> u32 {
        self.version_
    }
    pub fn Copy(&self) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(self.clone()))
    }
    // cpp: css_keyframes_rule.cc:81-94. Search starts at the last rule.
    pub fn FindKeyframeIndex(&self, context: &D::CSSParserContext, key: &String) -> i32 {
        let Some(keys) = D::ParseKeyframeKeyListWithContext(context, key) else {
            return -1;
        };
        for i in (0..self.keyframes_.len()).rev() {
            if self.keyframes_[i].borrow().Keys() == keys {
                return i as i32;
            }
        }
        -1
    }
}

// cpp: css_keyframes_rule.h:91-130. Weak caches retain every observable Rc
// identity: when any caller holds a wrapper it upgrades to that exact object.
// Once no caller holds it, recreation is unobservable. Children retain their
// parent strongly so parentRule remains valid while a child is held externally.
pub struct CSSKeyframesRule<D: CSSKeyframesBackend> {
    keyframes_rule_: RefCell<Rc<RefCell<StyleRuleKeyframes<D>>>>,
    parent_style_sheet_: RefCell<Option<Rc<D::CSSStyleSheet>>>,
    child_rule_cssom_wrappers_: RefCell<Vec<Option<Weak<CSSKeyframeRule<D>>>>>,
    rule_list_cssom_wrapper_: RefCell<Option<Weak<D::CSSRuleList>>>,
    is_prefixed_: Cell<bool>,
}
impl<D: CSSKeyframesBackend> CSSKeyframesRule<D> {
    // cpp: css_keyframes_rule.cc:101-106
    pub fn new(
        keyframes_rule: Rc<RefCell<StyleRuleKeyframes<D>>>,
        parent: Option<Rc<D::CSSStyleSheet>>,
    ) -> Rc<Self> {
        let length = keyframes_rule.borrow().Keyframes().len();
        let prefixed = keyframes_rule.borrow().IsVendorPrefixed();
        Rc::new(Self {
            keyframes_rule_: RefCell::new(keyframes_rule),
            parent_style_sheet_: RefCell::new(parent),
            child_rule_cssom_wrappers_: RefCell::new(vec![None; length]),
            rule_list_cssom_wrapper_: RefCell::new(None),
            is_prefixed_: Cell::new(prefixed),
        })
    }
    pub fn GetType(&self) -> CSSKeyframesCSSRuleType {
        CSSKeyframesCSSRuleType::kKeyframesRule
    }
    pub fn Keyframes(&self) -> Rc<RefCell<StyleRuleKeyframes<D>>> {
        self.keyframes_rule_.borrow().clone()
    }
    pub fn parentStyleSheet(&self) -> Option<Rc<D::CSSStyleSheet>> {
        self.parent_style_sheet_.borrow().clone()
    }
    pub fn SetParentStyleSheet(&self, parent: Option<Rc<D::CSSStyleSheet>>) {
        *self.parent_style_sheet_.borrow_mut() = parent;
    }
    pub fn name(&self) -> String {
        let name = self.keyframes_rule_.borrow().borrow().GetName();
        if name.IsNull() {
            String::default()
        } else {
            String::from_utf16(name.utf16_units().unwrap_or_default())
        }
    }
    // cpp: css_keyframes_rule.cc:110-117. SetName does not increment Version.
    pub fn setName(&self, name: &String) {
        let _mutation_scope = D::BeginRuleMutation(CSSKeyframesMutationTarget::Keyframes(self));
        if let Some(sheet) = self.parentStyleSheet() {
            D::NotifyDiffUnrepresentable(&sheet);
        }
        self.keyframes_rule_.borrow().borrow_mut().SetName(name);
    }
    // cpp: css_keyframes_rule.cc:119-141
    pub fn appendRule(&self, execution_context: &D::ExecutionContext, rule_text: &String) {
        let context = D::ParserContext(self, D::GetSecureContextMode(execution_context));
        let context = D::NewCSSParserContext(context, self.parentStyleSheet());
        let Some(keyframe) = D::ParseKeyframeRule(&context, rule_text) else {
            return;
        };
        let _mutation_scope = D::BeginRuleMutation(CSSKeyframesMutationTarget::Keyframes(self));
        if let Some(sheet) = self.parentStyleSheet() {
            D::NotifyDiffUnrepresentable(&sheet);
        }
        self.keyframes_rule_
            .borrow()
            .borrow_mut()
            .WrapperAppendKeyframe(keyframe);
        self.child_rule_cssom_wrappers_
            .borrow_mut()
            .resize_with(self.length(), || None);
    }
    // cpp: css_keyframes_rule.cc:143-167
    pub fn deleteRule(&self, execution_context: &D::ExecutionContext, key: &String) {
        let context = D::ParserContext(self, D::GetSecureContextMode(execution_context));
        let index = self
            .keyframes_rule_
            .borrow()
            .borrow()
            .FindKeyframeIndex(&context, key);
        if index < 0 {
            return;
        }
        let index = index as usize;
        let _mutation_scope = D::BeginRuleMutation(CSSKeyframesMutationTarget::Keyframes(self));
        if let Some(sheet) = self.parentStyleSheet() {
            D::NotifyDiffUnrepresentable(&sheet);
        }
        self.keyframes_rule_
            .borrow()
            .borrow_mut()
            .WrapperRemoveKeyframe(index);
        if let Some(child) = self.child_rule_cssom_wrappers_.borrow()[index]
            .as_ref()
            .and_then(Weak::upgrade)
        {
            child.SetParentRule(None);
        }
        self.child_rule_cssom_wrappers_.borrow_mut().remove(index);
    }
    // cpp: css_keyframes_rule.cc:169-177
    pub fn findRule(
        self: &Rc<Self>,
        execution_context: &D::ExecutionContext,
        key: &String,
    ) -> Option<Rc<CSSKeyframeRule<D>>> {
        let context = D::ParserContext(self, D::GetSecureContextMode(execution_context));
        let index = self
            .keyframes_rule_
            .borrow()
            .borrow()
            .FindKeyframeIndex(&context, key);
        if index >= 0 {
            self.Item(index as usize, true)
        } else {
            None
        }
    }
    // cpp: css_keyframes_rule.cc:179-197
    pub fn cssText(&self) -> String {
        let mut result: Vec<u16> = if self.IsVendorPrefixed() {
            "@-webkit-keyframes "
        } else {
            "@keyframes "
        }
        .encode_utf16()
        .collect();
        SerializeIdentifierTo(&self.name(), &mut result, false);
        result.extend(" { \n".encode_utf16());
        for keyframe in self.keyframes_rule_.borrow().borrow().Keyframes() {
            result.extend("  ".encode_utf16());
            result.extend_from_slice(keyframe.borrow().CssText().Span16().unwrap_or_default());
            result.push(b'\n' as u16);
        }
        result.push(b'}' as u16);
        String::from_utf16(&result)
    }
    pub fn length(&self) -> usize {
        self.keyframes_rule_.borrow().borrow().Keyframes().len()
    }
    // cpp: css_keyframes_rule.cc:203-219. trigger_use_counters is unused upstream.
    pub fn Item(
        self: &Rc<Self>,
        index: usize,
        _trigger_use_counters: bool,
    ) -> Option<Rc<CSSKeyframeRule<D>>> {
        if index >= self.length() {
            return None;
        }
        if let Some(child) = self.child_rule_cssom_wrappers_.borrow()[index]
            .as_ref()
            .and_then(Weak::upgrade)
        {
            return Some(child);
        }
        let keyframe = self.keyframes_rule_.borrow().borrow().Keyframes()[index].clone();
        let child = CSSKeyframeRule::new(keyframe, Some(Rc::clone(self)));
        self.child_rule_cssom_wrappers_.borrow_mut()[index] = Some(Rc::downgrade(&child));
        Some(child)
    }
    // cpp: css_keyframes_rule.cc:221-230. Count even for an out-of-range index.
    pub fn AnonymousIndexedGetter(self: &Rc<Self>, index: usize) -> Option<Rc<CSSKeyframeRule<D>>> {
        let sheet = self.parentStyleSheet();
        if let Some(document) = D::SingleOwnerDocument(sheet.as_deref()) {
            D::CountCSSKeyframesRuleAnonymousIndexedGetter(&document);
        }
        self.Item(index, true)
    }
    // cpp: css_keyframes_rule.cc:232-239
    pub fn cssRules(self: &Rc<Self>) -> Rc<D::CSSRuleList> {
        if let Some(list) = self
            .rule_list_cssom_wrapper_
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
        {
            return list;
        }
        let list = D::NewLiveCSSRuleList(Rc::clone(self));
        *self.rule_list_cssom_wrapper_.borrow_mut() = Some(Rc::downgrade(&list));
        list
    }
    pub fn IsVendorPrefixed(&self) -> bool {
        self.is_prefixed_.get()
    }
    pub fn SetVendorPrefixed(&self, is_prefixed: bool) {
        self.is_prefixed_.set(is_prefixed);
    }
    pub fn StyleChanged(&self) {
        self.keyframes_rule_.borrow().borrow_mut().StyleChanged();
    }
    pub fn Reattach<S>(&self, rule: &StyleRuleBase<S>)
    where
        S: StyleRuleDependencies<StyleRuleKeyframes = RefCell<StyleRuleKeyframes<D>>>,
    {
        let StyleRuleBase::Keyframes(keyframes) = rule else {
            panic!("CSSKeyframesRule::Reattach requires a keyframes rule");
        };
        self.ReattachKeyframes(Rc::clone(keyframes));
    }
    // cpp: css_keyframes_rule.cc:241-252. CHECK_EQ is a production invariant.
    pub fn ReattachKeyframes(&self, keyframes: Rc<RefCell<StyleRuleKeyframes<D>>>) {
        *self.keyframes_rule_.borrow_mut() = keyframes;
        assert_eq!(
            self.child_rule_cssom_wrappers_.borrow().len(),
            self.length()
        );
        for (index, child) in self.child_rule_cssom_wrappers_.borrow().iter().enumerate() {
            if let Some(child) = child.as_ref().and_then(Weak::upgrade) {
                child.ReattachKeyframe(
                    self.keyframes_rule_.borrow().borrow().Keyframes()[index].clone(),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css_property_names::CSSPropertyID;
    use crate::style_rule::StyleRulePropertySet;
    use crate::style_rule_keyframe::TimelineNamedRange;

    struct Property(bool);
    impl StyleRulePropertySet for Property {
        type CSSValue = ();
        fn IsMutable(&self) -> bool {
            self.0
        }
        fn MutableCopy(&self) -> Rc<Self> {
            Rc::new(Self(true))
        }
        fn HasFailedOrCanceledSubresources(&self) -> bool {
            false
        }
        fn GetPropertyCSSValue(&self, _: CSSPropertyID) -> Option<Rc<()>> {
            None
        }
    }
    struct Sheet {
        events: RefCell<Vec<&'static str>>,
        use_count: Rc<Cell<usize>>,
    }
    struct Context {
        mode: bool,
        with_sheet: bool,
    }
    struct MutationScope(Option<Rc<Sheet>>);
    impl Drop for MutationScope {
        fn drop(&mut self) {
            if let Some(sheet) = &self.0 {
                sheet.events.borrow_mut().push("end");
            }
        }
    }
    struct Declaration {
        properties: RefCell<Rc<Property>>,
        owner: Rc<CSSKeyframeRule<Backend>>,
        reattachments: Cell<usize>,
    }
    impl KeyframeStyleDeclaration<Backend> for Declaration {
        fn Reattach(&self, properties: Rc<Property>) {
            *self.properties.borrow_mut() = properties;
            self.reattachments.set(self.reattachments.get() + 1);
        }
    }
    struct RuleList(Rc<CSSKeyframesRule<Backend>>);
    impl CSSKeyframesRuleList<Backend> for RuleList {
        fn length(&self) -> usize {
            self.0.length()
        }
        fn Item(&self, index: usize) -> Option<Rc<CSSKeyframeRule<Backend>>> {
            self.0.Item(index, true)
        }
    }
    struct Backend;
    fn keys(text: &String) -> Option<Vec<KeyframeOffset>> {
        let percent = match text.Utf8().as_str() {
            "from" => 0.0,
            "50%" => 0.5,
            "to" => 1.0,
            _ => return None,
        };
        Some(vec![KeyframeOffset::new(
            TimelineNamedRange::kNone,
            percent,
        )])
    }
    fn frame(key: &str) -> Rc<RefCell<StyleRuleKeyframe<Backend>>> {
        Rc::new(RefCell::new(StyleRuleKeyframe::new(
            keys(&String::from(key)).unwrap(),
            Rc::new(Property(false)),
        )))
    }
    impl StyleRuleKeyframeBackend for Backend {
        type ExecutionContext = bool;
        type PropertySet = Property;
        fn ParseKeyframeKeyList(_: &bool, text: &String) -> Option<Vec<KeyframeOffset>> {
            keys(text)
        }
        fn TimelineRangeNameToString(_: TimelineNamedRange) -> String {
            unreachable!("test uses unnamed offsets")
        }
        fn FormatNumber(value: f64) -> String {
            String::from(format!("{value}").as_str())
        }
        fn PropertiesAsText(_: &Property) -> String {
            String::from("opacity: 1;")
        }
    }
    impl CSSKeyframesBackend for Backend {
        type CSSStyleSheet = Sheet;
        type CSSParserContext = Context;
        type SecureContextMode = bool;
        type ExceptionState = Vec<String>;
        type RuleMutationScope = MutationScope;
        type CSSStyleDeclaration = Declaration;
        type CSSRuleList = RuleList;
        type Document = Cell<usize>;
        fn BeginRuleMutation(target: CSSKeyframesMutationTarget<'_, Self>) -> MutationScope {
            let sheet = match target {
                CSSKeyframesMutationTarget::Keyframe(rule) => rule.parentStyleSheet(),
                CSSKeyframesMutationTarget::Keyframes(rule) => rule.parentStyleSheet(),
            };
            if let Some(sheet) = &sheet {
                sheet.events.borrow_mut().push("begin");
            }
            MutationScope(sheet)
        }
        fn NotifyDiffUnrepresentable(sheet: &Sheet) {
            sheet.events.borrow_mut().push("dirty");
        }
        fn ThrowSyntaxError(state: &mut Vec<String>, message: &String) {
            state.push(message.clone());
        }
        fn GetSecureContextMode(context: &bool) -> bool {
            *context
        }
        fn ParserContext(_: &CSSKeyframesRule<Self>, mode: bool) -> Rc<Context> {
            Rc::new(Context {
                mode,
                with_sheet: false,
            })
        }
        fn NewCSSParserContext(context: Rc<Context>, sheet: Option<Rc<Sheet>>) -> Rc<Context> {
            Rc::new(Context {
                mode: context.mode,
                with_sheet: sheet.is_some(),
            })
        }
        fn ParseKeyframeRule(
            context: &Context,
            text: &String,
        ) -> Option<Rc<RefCell<StyleRuleKeyframe<Self>>>> {
            assert!(context.mode && context.with_sheet);
            (text.Utf8() == "from {}").then(|| frame("from"))
        }
        fn ParseKeyframeKeyListWithContext(
            context: &Context,
            text: &String,
        ) -> Option<Vec<KeyframeOffset>> {
            assert!(context.mode && !context.with_sheet);
            keys(text)
        }
        fn NewKeyframeStyleDeclaration(
            properties: Rc<Property>,
            owner: Rc<CSSKeyframeRule<Self>>,
        ) -> Rc<Declaration> {
            Rc::new(Declaration {
                properties: RefCell::new(properties),
                owner,
                reattachments: Cell::new(0),
            })
        }
        fn NewLiveCSSRuleList(owner: Rc<CSSKeyframesRule<Self>>) -> Rc<RuleList> {
            Rc::new(RuleList(owner))
        }
        fn SingleOwnerDocument(sheet: Option<&Sheet>) -> Option<Rc<Cell<usize>>> {
            sheet.map(|sheet| sheet.use_count.clone())
        }
        fn CountCSSKeyframesRuleAnonymousIndexedGetter(document: &Cell<usize>) {
            document.set(document.get() + 1);
        }
    }

    #[test]
    fn cssom_identity_mutation_notifications_live_list_and_reattach() {
        let first = frame("from");
        let middle = frame("to");
        let last = frame("to");
        let data = Rc::new(RefCell::new(StyleRuleKeyframes::new(
            vec![first.clone(), middle, last],
            AtomicString::from_str("anim"),
            7,
            false,
        )));
        let copy = data.borrow().Copy();
        assert!(Rc::ptr_eq(&copy.borrow().Keyframes()[0], &first));
        data.borrow_mut().ParserAppendKeyframe(None);
        assert_eq!(data.borrow().Version(), 7);
        let sheet = Rc::new(Sheet {
            events: RefCell::new(Vec::new()),
            use_count: Rc::new(Cell::new(0)),
        });
        let rule = CSSKeyframesRule::new(data.clone(), Some(sheet.clone()));
        let first_wrapper = rule.Item(0, true).unwrap();
        assert!(Rc::ptr_eq(&first_wrapper, &rule.Item(0, false).unwrap()));
        let list = rule.cssRules();
        assert!(Rc::ptr_eq(&list, &rule.cssRules()));
        assert!(Rc::ptr_eq(&list.Item(0).unwrap(), &first_wrapper));
        let declaration = first_wrapper.style();
        assert!(Rc::ptr_eq(&declaration, &first_wrapper.style()));
        assert!(declaration.properties.borrow().IsMutable());

        rule.setName(&String::from("9 anim"));
        assert_eq!(data.borrow().Version(), 7);
        assert!(rule
            .cssText()
            .Utf8()
            .starts_with("@keyframes \\39 \\ anim { \n"));
        assert_eq!(*sheet.events.borrow(), ["begin", "dirty", "end"]);
        sheet.events.borrow_mut().clear();
        rule.appendRule(&true, &String::from("invalid"));
        assert!(sheet.events.borrow().is_empty());
        assert_eq!(list.length(), 3);
        let last_wrapper = rule.findRule(&true, &String::from("to")).unwrap();
        assert!(Rc::ptr_eq(&last_wrapper, &rule.Item(2, true).unwrap()));
        rule.deleteRule(&true, &String::from("to"));
        assert!(last_wrapper.parentRule().is_none());
        assert_eq!(list.length(), 2);
        assert_eq!(data.borrow().Version(), 8);
        rule.appendRule(&true, &String::from("from {}"));
        assert_eq!(list.length(), 3);
        assert_eq!(data.borrow().Version(), 9);

        let mut exceptions = Vec::new();
        sheet.events.borrow_mut().clear();
        first_wrapper.setKeyText(&true, &String::from("bad"), &mut exceptions);
        assert_eq!(
            exceptions[0].Utf8(),
            "The key 'bad' is invalid and cannot be parsed"
        );
        assert_eq!(first_wrapper.keyText().Utf8(), "0%");
        assert_eq!(data.borrow().Version(), 10);
        assert_eq!(*sheet.events.borrow(), ["begin", "dirty", "end"]);
        first_wrapper.setKeyText(&true, &String::from("50%"), &mut exceptions);
        assert_eq!(first_wrapper.keyText().Utf8(), "50%");
        assert_eq!(data.borrow().Version(), 11);

        let replacement_frame = frame("to");
        let replacement = Rc::new(RefCell::new(StyleRuleKeyframes::new(
            vec![replacement_frame.clone(), frame("from"), frame("from")],
            AtomicString::from_str("new"),
            0x7fff_ffff,
            true,
        )));
        rule.ReattachKeyframes(replacement.clone());
        assert_eq!(first_wrapper.keyText().Utf8(), "100%");
        assert!(Rc::ptr_eq(&first_wrapper, &rule.Item(0, true).unwrap()));
        assert_eq!(declaration.reattachments.get(), 1);
        assert!(Rc::ptr_eq(
            &declaration.properties.borrow(),
            &replacement_frame.borrow().Properties()
        ));
        assert_eq!(list.length(), 3);
        assert!(!rule.IsVendorPrefixed());
        rule.SetVendorPrefixed(true);
        assert!(rule.cssText().Utf8().starts_with("@-webkit-keyframes new"));
        rule.StyleChanged();
        assert_eq!(replacement.borrow().Version(), 0);
        assert!(rule.AnonymousIndexedGetter(99).is_none());
        assert_eq!(sheet.use_count.get(), 1);

        let weak_parent = Rc::downgrade(&rule);
        drop(list);
        drop(rule);
        assert!(weak_parent.upgrade().is_some());
        assert!(first_wrapper.parentRule().is_some());
        drop(first_wrapper);
        assert!(declaration.owner.parentRule().is_some());
        drop(declaration);
        assert!(weak_parent.upgrade().is_none());
    }
}

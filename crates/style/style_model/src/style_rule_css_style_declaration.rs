// Source ledger: effective = nonblank source lines after stripping comments,
// minus the boilerplate exclusions below (function braces are counted).
// style_rule_css_style_declaration.h: physical 58; effective 7; mapped 7.
// style_rule_css_style_declaration.cc: physical 87; effective 38; mapped 38.
// Omitted .h:1-37,39-40,42-45,54-58: license, includes, class/namespace
// wrappers, default destructor and Oilpan Trace declaration.
// Omitted .cc:1-34,48-49,82-87: license, includes, namespace wrappers,
// default destructor and Oilpan Trace. Rust Rc retains referenced objects.
// Production logic pending in this source pair: 0. The actual CSSOM, Document,
// StyleSheetContents and PropertySetCSSStyleDeclaration implementations are
// external required dependencies; this ledger does not claim their bodies.

use std::cell::RefCell;
use std::rc::Rc;

// cpp: abstract_property_set_css_style_declaration.h:99-107.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MutationType {
    kNoChanges,
    kIndependentPropertyChanged,
    kPropertyChanged,
}

/// The real base declaration owns its execution context and property-set slot.
/// No default implementation can fabricate a base declaration or its storage.
pub trait PropertySetCSSStyleDeclaration: Sized {
    type ExecutionContext;
    type MutableCSSPropertyValueSet;

    fn new(
        context: Option<Rc<Self::ExecutionContext>>,
        property_set: Rc<RefCell<Self::MutableCSSPropertyValueSet>>,
    ) -> Self;
    fn PropertySetStorage(&mut self) -> &mut Rc<RefCell<Self::MutableCSSPropertyValueSet>>;
}

/// Calls to the original external classes, with their identities retained.
/// `IsStyleRule` is the typed boundary for CSSRule::GetType() == kStyleRule;
/// `DidMutateRules` calls CSSStyleSheet::DidMutate(Mutation::kRules).
pub trait StyleRuleCSSStyleDeclarationBackend: Sized {
    type Base: PropertySetCSSStyleDeclaration;
    type CSSRule;
    type CSSStyleSheet;
    type Document;
    type StyleSheetContents;
    type StyleRule;

    fn ParentStyleSheet(rule: &Self::CSSRule) -> Option<Rc<Self::CSSStyleSheet>>;
    fn SingleOwnerDocument(sheet: Option<&Self::CSSStyleSheet>) -> Option<Rc<Self::Document>>;
    fn GetExecutionContext(
        document: &Self::Document,
    ) -> Option<Rc<<Self::Base as PropertySetCSSStyleDeclaration>::ExecutionContext>>;
    fn WillMutateRules(sheet: &Self::CSSStyleSheet);
    fn Contents(sheet: &Self::CSSStyleSheet) -> Rc<Self::StyleSheetContents>;
    fn IsStyleRule(rule: &Self::CSSRule) -> bool;
    fn GetStyleRule(rule: &Self::CSSRule) -> Rc<Self::StyleRule>;
    fn NotifyRuleChanged(contents: &Self::StyleSheetContents, rule: Rc<Self::StyleRule>);
    fn NotifyDiffUnrepresentable(contents: &Self::StyleSheetContents);
    fn DidMutateRules(sheet: &Self::CSSStyleSheet);
}

// cpp: style_rule_css_style_declaration.h:36,53.
pub struct StyleRuleCSSStyleDeclaration<B: StyleRuleCSSStyleDeclarationBackend> {
    base: B::Base,
    parent_rule: Option<Rc<B::CSSRule>>,
}

impl<B: StyleRuleCSSStyleDeclarationBackend> StyleRuleCSSStyleDeclaration<B> {
    // cpp: style_rule_css_style_declaration.h:38; .cc:35-46.
    // The constructor dereferences parent_rule in Chromium, so it is nonnull.
    pub fn new(
        property_set: Rc<
            RefCell<<B::Base as PropertySetCSSStyleDeclaration>::MutableCSSPropertyValueSet>,
        >,
        parent_rule: Rc<B::CSSRule>,
    ) -> Self {
        let context =
            if B::SingleOwnerDocument(B::ParentStyleSheet(&parent_rule).as_deref()).is_some() {
                // Preserve the source's second lookup rather than caching ownership.
                let document = B::SingleOwnerDocument(B::ParentStyleSheet(&parent_rule).as_deref())
                    .expect("single owner document disappeared between constructor lookups");
                B::GetExecutionContext(&document)
            } else {
                None
            };
        Self {
            base: B::Base::new(context, property_set),
            parent_rule: Some(parent_rule),
        }
    }

    /// Exposes the inherited declaration to its actual CSSOM implementation.
    pub fn Base(&self) -> &B::Base {
        &self.base
    }

    // cpp: style_rule_css_style_declaration.h:48.
    pub fn parentRule(&self) -> Option<&Rc<B::CSSRule>> {
        self.parent_rule.as_ref()
    }

    // cpp: style_rule_css_style_declaration.h:50; .cc:50-54.
    pub fn WillMutate(&self) {
        if let Some(rule) = &self.parent_rule {
            if B::ParentStyleSheet(rule).is_some() {
                B::WillMutateRules(
                    &B::ParentStyleSheet(rule)
                        .expect("parent stylesheet disappeared between mutation lookups"),
                );
            }
        }
    }

    // cpp: style_rule_css_style_declaration.h:51; .cc:56-71.
    pub fn DidMutate(&self, _mutation_type: MutationType) {
        // Even kNoChanges signals the stylesheet: WillMutate/DidMutate pair.
        if let Some(rule) = &self.parent_rule {
            if B::ParentStyleSheet(rule).is_some() {
                let contents = B::Contents(
                    &B::ParentStyleSheet(rule)
                        .expect("parent stylesheet disappeared between mutation lookups"),
                );
                if B::IsStyleRule(rule) {
                    B::NotifyRuleChanged(&contents, B::GetStyleRule(rule));
                } else {
                    B::NotifyDiffUnrepresentable(&contents);
                }
                // NotifyRuleChanged may affect CSSOM ownership. Chromium reads
                // the rule's parent again before the final DidMutate callback.
                B::DidMutateRules(
                    &B::ParentStyleSheet(rule)
                        .expect("parent stylesheet disappeared during mutation notification"),
                );
            }
        }
    }

    // cpp: style_rule_css_style_declaration.h:46; .cc:73-75.
    pub fn ParentStyleSheet(&self) -> Option<Rc<B::CSSStyleSheet>> {
        self.parent_rule.as_deref().and_then(B::ParentStyleSheet)
    }

    // cpp: style_rule_css_style_declaration.h:41; .cc:77-80.
    pub fn Reattach(
        &mut self,
        property_set: Rc<
            RefCell<<B::Base as PropertySetCSSStyleDeclaration>::MutableCSSPropertyValueSet>,
        >,
    ) {
        *self.base.PropertySetStorage() = property_set;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct Base {
        context: Option<Rc<u32>>,
        properties: Rc<RefCell<Vec<u32>>>,
    }
    impl PropertySetCSSStyleDeclaration for Base {
        type ExecutionContext = u32;
        type MutableCSSPropertyValueSet = Vec<u32>;
        fn new(context: Option<Rc<u32>>, properties: Rc<RefCell<Vec<u32>>>) -> Self {
            Self {
                context,
                properties,
            }
        }
        fn PropertySetStorage(&mut self) -> &mut Rc<RefCell<Vec<u32>>> {
            &mut self.properties
        }
    }
    struct Rule {
        parent: RefCell<Option<Rc<Sheet>>>,
        style_rule: Option<Rc<u32>>,
        lookups: Cell<usize>,
    }
    struct Sheet {
        context: Option<Rc<u32>>,
        contents: Rc<Contents>,
    }
    struct Contents {
        log: Rc<RefCell<Vec<&'static str>>>,
        changed: RefCell<Option<Rc<u32>>>,
    }
    struct Backend;
    impl StyleRuleCSSStyleDeclarationBackend for Backend {
        type Base = Base;
        type CSSRule = Rule;
        type CSSStyleSheet = Sheet;
        type Document = u32;
        type StyleSheetContents = Contents;
        type StyleRule = u32;
        fn ParentStyleSheet(rule: &Rule) -> Option<Rc<Sheet>> {
            rule.lookups.set(rule.lookups.get() + 1);
            rule.parent.borrow().clone()
        }
        fn SingleOwnerDocument(sheet: Option<&Sheet>) -> Option<Rc<u32>> {
            sheet.and_then(|sheet| sheet.context.clone())
        }
        fn GetExecutionContext(document: &u32) -> Option<Rc<u32>> {
            Some(Rc::new(*document))
        }
        fn WillMutateRules(sheet: &Sheet) {
            sheet.contents.log.borrow_mut().push("will");
        }
        fn Contents(sheet: &Sheet) -> Rc<Contents> {
            sheet.contents.clone()
        }
        fn IsStyleRule(rule: &Rule) -> bool {
            rule.style_rule.is_some()
        }
        fn GetStyleRule(rule: &Rule) -> Rc<u32> {
            rule.style_rule.as_ref().unwrap().clone()
        }
        fn NotifyRuleChanged(contents: &Contents, rule: Rc<u32>) {
            contents.log.borrow_mut().push("changed");
            *contents.changed.borrow_mut() = Some(rule);
        }
        fn NotifyDiffUnrepresentable(contents: &Contents) {
            contents.log.borrow_mut().push("unrepresentable");
        }
        fn DidMutateRules(sheet: &Sheet) {
            sheet.contents.log.borrow_mut().push("did");
        }
    }

    #[test]
    fn declaration_keeps_identity_and_pairs_all_mutation_outcomes() {
        let log = Rc::new(RefCell::new(vec![]));
        let contents = Rc::new(Contents {
            log: log.clone(),
            changed: RefCell::new(None),
        });
        let sheet = Rc::new(Sheet {
            context: Some(Rc::new(7)),
            contents: contents.clone(),
        });
        let rule = Rc::new(Rule {
            parent: RefCell::new(Some(sheet.clone())),
            style_rule: Some(Rc::new(9)),
            lookups: Cell::new(0),
        });
        let properties = Rc::new(RefCell::new(vec![1]));
        let mut declaration =
            StyleRuleCSSStyleDeclaration::<Backend>::new(properties.clone(), rule.clone());
        assert_eq!(declaration.Base().context.as_deref(), Some(&7));
        assert_eq!(rule.lookups.get(), 2);
        assert!(Rc::ptr_eq(declaration.parentRule().unwrap(), &rule));
        assert!(Rc::ptr_eq(&declaration.ParentStyleSheet().unwrap(), &sheet));
        assert!(Rc::ptr_eq(&declaration.Base().properties, &properties));
        let replacement = Rc::new(RefCell::new(vec![2]));
        declaration.Reattach(replacement.clone());
        assert!(Rc::ptr_eq(&declaration.Base().properties, &replacement));
        for outcome in [
            MutationType::kNoChanges,
            MutationType::kIndependentPropertyChanged,
            MutationType::kPropertyChanged,
        ] {
            let before = rule.lookups.get();
            declaration.WillMutate();
            declaration.DidMutate(outcome);
            assert_eq!(rule.lookups.get() - before, 5);
        }
        assert_eq!(
            &*log.borrow(),
            &["will", "changed", "did", "will", "changed", "did", "will", "changed", "did"]
        );
        assert!(Rc::ptr_eq(
            contents.changed.borrow().as_ref().unwrap(),
            rule.style_rule.as_ref().unwrap()
        ));
        log.borrow_mut().clear();
        rule.parent.borrow_mut().take();
        declaration.WillMutate();
        declaration.DidMutate(MutationType::kPropertyChanged);
        assert!(declaration.ParentStyleSheet().is_none());
        assert!(log.borrow().is_empty());

        let other_rule = Rc::new(Rule {
            parent: RefCell::new(Some(Rc::new(Sheet {
                context: None,
                contents,
            }))),
            style_rule: None,
            lookups: Cell::new(0),
        });
        let other = StyleRuleCSSStyleDeclaration::<Backend>::new(properties, other_rule.clone());
        assert!(other.Base().context.is_none());
        assert_eq!(other_rule.lookups.get(), 1);
        other.WillMutate();
        other.DidMutate(MutationType::kNoChanges);
        assert_eq!(&*log.borrow(), &["will", "unrepresentable", "did"]);
        log.borrow_mut().clear();
        declaration.parent_rule = None;
        declaration.WillMutate();
        declaration.DidMutate(MutationType::kNoChanges);
        assert!(declaration.parentRule().is_none());
        assert!(declaration.ParentStyleSheet().is_none());
        assert!(log.borrow().is_empty());
    }
}

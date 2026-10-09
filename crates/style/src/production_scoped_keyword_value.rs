// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! css_scoped_keyword_value.h:26-58 / .cc:12-33, document-root domain.
use super::{ProductionCSSValueDispatch, Value};
use crate::css_value::{CSSValuePayload, CSSValueSubclass, CSSValueTreeScope};
use foundation::{CSSValueID, String, TreeScope};
use std::rc::Rc;

#[derive(Clone)]
pub struct CSSScopedKeywordValue {
    value_id: CSSValueID,
    tree_scope: *const TreeScope,
    populated: bool,
}
impl CSSScopedKeywordValue {
    pub fn GetValueID(&self) -> CSSValueID {
        self.value_id
    }
    pub fn GetTreeScope(&self) -> *const TreeScope {
        self.tree_scope
    }
    pub fn GetPopulatedTreeScope(&self) -> *const TreeScope {
        assert!(self.populated);
        self.tree_scope
    }
    // The production owner supports the existing null document-root domain.
    // It has no non-null shadow TreeScope binding; no owner is manufactured.
    pub(crate) fn PopulateForDocumentRoot(&self) -> Self {
        assert!(!self.populated);
        Self {
            value_id: self.value_id,
            tree_scope: std::ptr::null(),
            populated: true,
        }
    }
}
impl CSSValueSubclass for CSSScopedKeywordValue {
    fn CustomCSSText(&self) -> String {
        String::from(crate::css_value_keywords::GetCSSValueName(self.value_id))
    }
    fn Equals(&self, other: &Self) -> bool {
        self.populated == other.populated
            && self.tree_scope == other.tree_scope
            && self.value_id == other.value_id
    }
}
impl CSSValueTreeScope<ProductionCSSValueDispatch> for CSSScopedKeywordValue {
    fn PopulateWithTreeScope<'a>(&'a self, scope: Option<&'a ()>) -> &'a Value {
        assert!(
            scope.is_none(),
            "non-null TreeScope requires a real application owner binding"
        );
        let value = Value::new(CSSValuePayload::kScopedKeywordClass(
            self.PopulateForDocumentRoot(),
        ));
        Box::leak(Box::new(value))
    }
}
pub fn scoped_keyword(id: CSSValueID) -> Rc<Value> {
    let mut value = Value::new(CSSValuePayload::kScopedKeywordClass(
        CSSScopedKeywordValue {
            value_id: id,
            tree_scope: std::ptr::null(),
            populated: false,
        },
    ));
    value.StateMut().SetNeedsTreeScopePopulation(true);
    Rc::new(value)
}

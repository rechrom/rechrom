// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! replace_node_with_span_command.cc:47-100. The elementary undo command swaps
//! native node identity while moving the original children and attributes.
use super::{
    paragraph_editing_context::{EditResult, EditingAborted},
    simple_edit_commands::{SimpleEditCommand, SimpleEditingDOM},
};
use foundation::QualifiedName;
use std::{cell::RefCell, rc::Rc};
pub trait SpanReplacementDOM: SimpleEditingDOM {
    fn IsConnected(&self, node: &Self::Node) -> bool;
    fn Attributes(&self, node: &Self::Node) -> Vec<(QualifiedName, Self::AttributeValue)>;
}
pub struct ReplaceNodeWithSpanCommand<D: SpanReplacementDOM> {
    dom: Rc<D>,
    element: D::Node,
    span: RefCell<Option<D::Node>>,
}
impl<D: SpanReplacementDOM> ReplaceNodeWithSpanCommand<D> {
    pub fn new(dom: Rc<D>, element: D::Node) -> Self {
        Self {
            dom,
            element,
            span: RefCell::new(None),
        }
    }
    pub fn SpanElement(&self) -> Option<D::Node> {
        self.span.borrow().clone()
    }
    fn Swap(&self, new: &D::Node, old: &D::Node) -> EditResult {
        let parent = self.dom.Parent(old).ok_or(EditingAborted)?;
        if !self.dom.InsertBefore(&parent, new, Some(old)) {
            return Err(EditingAborted);
        }
        for child in self.dom.Children(old) {
            if !self.dom.AppendChild(new, &child) {
                return Err(EditingAborted);
            }
        }
        for (name, _) in self.dom.Attributes(new) {
            self.dom.SetAttribute(new, &name, None);
        }
        // Source snapshots attributes before synchronous attribute callbacks.
        let attributes = self.dom.Attributes(old);
        for (name, value) in attributes {
            self.dom.SetAttribute(new, &name, Some(&value));
        }
        self.dom.Remove(old).then_some(()).ok_or(EditingAborted)
    }
}
impl<D: SpanReplacementDOM> SimpleEditCommand for ReplaceNodeWithSpanCommand<D> {
    fn DoApply(&self) -> EditResult {
        if !self.dom.IsConnected(&self.element) {
            return Ok(());
        }
        if self.span.borrow().is_none() {
            *self.span.borrow_mut() = Some(self.dom.CreateSpan());
        }
        self.Swap(&self.span.borrow().clone().unwrap(), &self.element)
    }
    fn DoUnapply(&self) {
        if let Some(span) = self
            .span
            .borrow()
            .clone()
            .filter(|n| self.dom.IsConnected(n))
        {
            let _ = self.Swap(&self.element, &span);
        }
    }
}

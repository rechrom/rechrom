// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: append_node_command.cc:34-56; insert_node_before_command.cc:37-84;
// remove_node_command.cc:36-101; split_text_node_command.cc:36-111;
// split_element_command.cc:35-108; delete_from_text_node_command.cc:35-76;
// set_node_attribute_command.cc:35-56. These are source elementary commands,
// not a general text editor or a copied mutation/DOM model.
use super::edit_command::PasswordEchoBehavior;
use super::paragraph_editing_context::{EditResult, EditingAborted};
use foundation::{AtomicString, QualifiedName};
use std::{cell::RefCell, rc::Rc};

fn IdAttributeName() -> QualifiedName {
    QualifiedName::new(
        AtomicString::default(),
        AtomicString::from_str("id"),
        AtomicString::default(),
    )
}

/// Settings.json5:80-95. Enabling echo is separate from the native reveal
/// capability; the embedding must provide that capability before enabling it.
#[derive(Clone, Copy, Debug, Default)]
pub struct PasswordEchoSettings {
    pub physical: bool,
    pub touch: bool,
}
impl PasswordEchoSettings {
    pub fn ShouldEchoPassword(&self, behavior: PasswordEchoBehavior) -> bool {
        match behavior {
            PasswordEchoBehavior::kEchoIfPasswordEchoPhysicalEnabled => self.physical,
            PasswordEchoBehavior::kEchoIfPasswordEchoTouchEnabled => self.touch,
            PasswordEchoBehavior::kDoNotEcho => false,
        }
    }
}

pub trait SimpleEditingDOM {
    type Node: Clone + PartialEq;
    type AttributeValue: Clone;
    fn Parent(&self, node: &Self::Node) -> Option<Self::Node>;
    fn NextSibling(&self, node: &Self::Node) -> Option<Self::Node>;
    fn FirstChild(&self, node: &Self::Node) -> Option<Self::Node>;
    fn LastChild(&self, node: &Self::Node) -> Option<Self::Node>;
    fn Children(&self, node: &Self::Node) -> Vec<Self::Node>;
    fn IsEditable(&self, node: &Self::Node) -> bool;
    fn InActiveDocument(&self, node: &Self::Node) -> bool;
    fn HasFrame(&self, node: &Self::Node) -> bool;
    fn HasDocumentElement(&self, node: &Self::Node) -> bool;
    fn IsPlaceholderBreak(&self, node: &Self::Node) -> bool;
    fn TextAreaEmptyPlaceholderBreakEnabled(&self) -> bool;
    fn NonEmptyBlockquotesOnOutdentingEnabled(&self) -> bool;
    fn UpdateStyleAndLayoutTree(&self);
    fn UpdateStyleAndLayout(&self);
    fn AppendChild(&self, parent: &Self::Node, node: &Self::Node) -> bool;
    fn InsertBefore(
        &self,
        parent: &Self::Node,
        node: &Self::Node,
        reference: Option<&Self::Node>,
    ) -> bool;
    fn Remove(&self, node: &Self::Node) -> bool;
    fn CreateText(&self, text: &[u16]) -> Self::Node;
    fn CreateSpan(&self) -> Self::Node;
    fn CloneWithoutChildren(&self, node: &Self::Node) -> Self::Node;
    fn Text(&self, node: &Self::Node) -> Rc<[u16]>;
    fn ReplaceText(&self, node: &Self::Node, offset: usize, count: usize, text: &[u16]) -> bool;
    fn MoveMarkers(&self, from: &Self::Node, length: usize, to: &Self::Node);
    fn Attribute(&self, node: &Self::Node, name: &QualifiedName) -> Option<Self::AttributeValue>;
    fn SetAttribute(
        &self,
        node: &Self::Node,
        name: &QualifiedName,
        value: Option<&Self::AttributeValue>,
    );
    fn ShouldEchoPassword(&self, behavior: PasswordEchoBehavior) -> bool;
    fn MomentarilyRevealLastTypedCharacterIfSecure(&self, node: &Self::Node, offset: usize);
}
pub trait SimpleEditCommand {
    fn DoApply(&self) -> EditResult;
    fn DoUnapply(&self);
    fn DoReapply(&self) {
        let _ = self.DoApply();
    }
}

// Source merge_identical_elements_command.cc:37-103.
pub struct MergeIdenticalElementsCommand<D: SimpleEditingDOM> {
    dom: Rc<D>,
    first: D::Node,
    second: D::Node,
    at_child: RefCell<Option<D::Node>>,
}
impl<D: SimpleEditingDOM> MergeIdenticalElementsCommand<D> {
    pub fn new(dom: Rc<D>, first: D::Node, second: D::Node) -> Self {
        debug_assert!(dom.NextSibling(&first).as_ref() == Some(&second));
        Self {
            dom,
            first,
            second,
            at_child: RefCell::new(None),
        }
    }
}
impl<D: SimpleEditingDOM> SimpleEditCommand for MergeIdenticalElementsCommand<D> {
    fn DoApply(&self) -> EditResult {
        if self.dom.NextSibling(&self.first).as_ref() != Some(&self.second)
            || !self.dom.IsEditable(&self.first)
            || !self.dom.IsEditable(&self.second)
        {
            return Ok(());
        }
        let at_child = self.dom.FirstChild(&self.second);
        *self.at_child.borrow_mut() = at_child.clone();
        for child in self.dom.Children(&self.first) {
            self.dom
                .InsertBefore(&self.second, &child, at_child.as_ref());
        }
        self.dom.Remove(&self.first);
        Ok(())
    }
    fn DoUnapply(&self) {
        let at_child = self.at_child.borrow_mut().take();
        let Some(parent) = self
            .dom
            .Parent(&self.second)
            .filter(|parent| self.dom.IsEditable(parent))
        else {
            return;
        };
        if !self
            .dom
            .InsertBefore(&parent, &self.first, Some(&self.second))
        {
            return;
        }
        let mut children = Vec::new();
        let mut node = self.dom.FirstChild(&self.second);
        while let Some(child) = node.filter(|child| Some(child) != at_child.as_ref()) {
            node = self.dom.NextSibling(&child);
            children.push(child);
        }
        for child in children {
            self.dom.AppendChild(&self.first, &child);
        }
    }
}

// Source wrap_contents_in_dummy_span_command.cc:36-88.
pub struct WrapContentsInDummySpanCommand<D: SimpleEditingDOM> {
    dom: Rc<D>,
    element: D::Node,
    span: RefCell<Option<D::Node>>,
}
impl<D: SimpleEditingDOM> WrapContentsInDummySpanCommand<D> {
    pub fn new(dom: Rc<D>, element: D::Node) -> Self {
        Self {
            dom,
            element,
            span: RefCell::new(None),
        }
    }
    fn ExecuteApply(&self, span: &D::Node) {
        for child in self.dom.Children(&self.element) {
            self.dom.AppendChild(span, &child);
        }
        self.dom.AppendChild(&self.element, span);
    }
}
impl<D: SimpleEditingDOM> SimpleEditCommand for WrapContentsInDummySpanCommand<D> {
    fn DoApply(&self) -> EditResult {
        let span = self.dom.CreateSpan();
        *self.span.borrow_mut() = Some(span.clone());
        self.ExecuteApply(&span);
        Ok(())
    }
    fn DoUnapply(&self) {
        let Some(span) = self
            .span
            .borrow()
            .clone()
            .filter(|_| self.dom.IsEditable(&self.element))
        else {
            return;
        };
        for child in self.dom.Children(&span) {
            self.dom.AppendChild(&self.element, &child);
        }
        self.dom.Remove(&span);
    }
    fn DoReapply(&self) {
        let Some(span) = self
            .span
            .borrow()
            .clone()
            .filter(|_| self.dom.IsEditable(&self.element))
        else {
            return;
        };
        self.ExecuteApply(&span);
    }
}
pub struct AppendNodeCommand<D: SimpleEditingDOM> {
    dom: Rc<D>,
    parent: D::Node,
    node: D::Node,
}
impl<D: SimpleEditingDOM> AppendNodeCommand<D> {
    pub fn new(dom: Rc<D>, parent: D::Node, node: D::Node) -> Self {
        Self { dom, parent, node }
    }
}
impl<D: SimpleEditingDOM> SimpleEditCommand for AppendNodeCommand<D> {
    fn DoApply(&self) -> EditResult {
        if !self.dom.IsEditable(&self.parent) && self.dom.InActiveDocument(&self.parent) {
            return Ok(());
        }
        self.dom.AppendChild(&self.parent, &self.node);
        Ok(())
    }
    fn DoUnapply(&self) {
        if self.dom.IsEditable(&self.parent) {
            self.dom.Remove(&self.node);
        }
    }
}
pub struct InsertNodeBeforeCommand<D: SimpleEditingDOM> {
    dom: Rc<D>,
    node: D::Node,
    reference: D::Node,
    assume_editable: bool,
}
impl<D: SimpleEditingDOM> InsertNodeBeforeCommand<D> {
    pub fn new(dom: Rc<D>, node: D::Node, reference: D::Node, assume_editable: bool) -> Self {
        Self {
            dom,
            node,
            reference,
            assume_editable,
        }
    }
}
impl<D: SimpleEditingDOM> SimpleEditCommand for InsertNodeBeforeCommand<D> {
    fn DoApply(&self) -> EditResult {
        let parent = self.dom.Parent(&self.reference);
        self.dom.UpdateStyleAndLayoutTree();
        let Some(parent) =
            parent.filter(|parent| self.assume_editable || self.dom.IsEditable(parent))
        else {
            return Ok(());
        };
        if !self
            .dom
            .InsertBefore(&parent, &self.node, Some(&self.reference))
        {
            return Err(EditingAborted);
        }
        Ok(())
    }
    fn DoUnapply(&self) {
        self.dom.UpdateStyleAndLayoutTree();
        if self
            .dom
            .Parent(&self.reference)
            .is_some_and(|parent| self.dom.IsEditable(&parent))
        {
            self.dom.Remove(&self.node);
        }
    }
}
pub struct RemoveNodeCommand<D: SimpleEditingDOM> {
    dom: Rc<D>,
    node: D::Node,
    assume_editable: bool,
    parent: RefCell<Option<D::Node>>,
    reference: RefCell<Option<D::Node>>,
}
impl<D: SimpleEditingDOM> RemoveNodeCommand<D> {
    pub fn new(dom: Rc<D>, node: D::Node, assume_editable: bool) -> Self {
        Self {
            dom,
            node,
            assume_editable,
            parent: RefCell::new(None),
            reference: RefCell::new(None),
        }
    }
}
impl<D: SimpleEditingDOM> SimpleEditCommand for RemoveNodeCommand<D> {
    fn DoApply(&self) -> EditResult {
        let Some(parent) = self.dom.Parent(&self.node) else {
            return Ok(());
        };
        if !self.assume_editable {
            self.dom.UpdateStyleAndLayoutTree();
            if !self.dom.IsEditable(&parent) && self.dom.InActiveDocument(&parent) {
                return Ok(());
            }
        }
        *self.parent.borrow_mut() = Some(parent);
        let next = self.dom.NextSibling(&self.node);
        *self.reference.borrow_mut() = if self.dom.TextAreaEmptyPlaceholderBreakEnabled()
            && next
                .as_ref()
                .is_some_and(|node| self.dom.IsPlaceholderBreak(node))
        {
            None
        } else {
            next
        };
        self.dom.Remove(&self.node);
        if !self.dom.HasFrame(&self.node) || !self.dom.HasDocumentElement(&self.node) {
            return Err(EditingAborted);
        }
        Ok(())
    }
    fn DoUnapply(&self) {
        let parent = self.parent.borrow_mut().take();
        let reference = self.reference.borrow_mut().take();
        let Some(parent) = parent.filter(|parent| self.dom.IsEditable(parent)) else {
            return;
        };
        if reference.is_none() && self.dom.TextAreaEmptyPlaceholderBreakEnabled() {
            let last = self.dom.LastChild(&parent);
            if let Some(last) = last.filter(|node| self.dom.IsPlaceholderBreak(node)) {
                self.dom.Remove(&last);
            }
        }
        self.dom
            .InsertBefore(&parent, &self.node, reference.as_ref());
    }
}
pub struct SplitTextNodeCommand<D: SimpleEditingDOM> {
    dom: Rc<D>,
    right: D::Node,
    offset: usize,
    left: RefCell<Option<D::Node>>,
}
impl<D: SimpleEditingDOM> SplitTextNodeCommand<D> {
    pub fn new(dom: Rc<D>, right: D::Node, offset: usize) -> Self {
        debug_assert!(
            offset > 0 && offset < dom.Text(&right).len(),
            "source split leaves nonempty original right text"
        );
        Self {
            dom,
            right,
            offset,
            left: RefCell::new(None),
        }
    }
    fn InsertText1AndTrimText2(&self, left: &D::Node) {
        let Some(parent) = self.dom.Parent(&self.right) else {
            return;
        };
        if !self.dom.InsertBefore(&parent, left, Some(&self.right)) {
            return;
        }
        self.dom.ReplaceText(&self.right, 0, self.offset, &[]);
        self.dom.UpdateStyleAndLayout();
    }
}
impl<D: SimpleEditingDOM> SimpleEditCommand for SplitTextNodeCommand<D> {
    fn DoApply(&self) -> EditResult {
        if !self
            .dom
            .Parent(&self.right)
            .is_some_and(|parent| self.dom.IsEditable(&parent))
        {
            return Ok(());
        }
        let text = self.dom.Text(&self.right);
        let prefix = &text[..self.offset.min(text.len())];
        if prefix.is_empty() {
            return Ok(());
        }
        let left = self.dom.CreateText(prefix);
        *self.left.borrow_mut() = Some(left.clone());
        self.dom.MoveMarkers(&self.right, self.offset, &left);
        self.InsertText1AndTrimText2(&left);
        Ok(())
    }
    fn DoUnapply(&self) {
        let left = self.left.borrow().clone();
        let Some(left) = left.filter(|node| self.dom.IsEditable(node)) else {
            return;
        };
        let prefix = self.dom.Text(&left);
        self.dom.ReplaceText(&self.right, 0, 0, &prefix);
        self.dom.UpdateStyleAndLayout();
        self.dom.MoveMarkers(&left, prefix.len(), &self.right);
        self.dom.Remove(&left);
    }
    fn DoReapply(&self) {
        let left = self.left.borrow().clone();
        let Some(left) = left else {
            return;
        };
        if !self
            .dom
            .Parent(&self.right)
            .is_some_and(|parent| self.dom.IsEditable(&parent))
        {
            return;
        }
        self.dom.MoveMarkers(&self.right, self.offset, &left);
        self.InsertText1AndTrimText2(&left);
    }
}
pub struct SplitElementCommand<D: SimpleEditingDOM> {
    dom: Rc<D>,
    right: D::Node,
    at_child: D::Node,
    left: RefCell<Option<D::Node>>,
}
impl<D: SimpleEditingDOM> SplitElementCommand<D> {
    pub fn new(dom: Rc<D>, right: D::Node, at_child: D::Node) -> Self {
        Self {
            dom,
            right,
            at_child,
            left: RefCell::new(None),
        }
    }
    fn ExecuteApply(&self, left: &D::Node) {
        if self.dom.Parent(&self.at_child).as_ref() != Some(&self.right) {
            return;
        }
        let mut children = Vec::new();
        let mut current = self.dom.FirstChild(&self.right);
        while let Some(node) = current {
            if node == self.at_child {
                break;
            }
            current = self.dom.NextSibling(&node);
            children.push(node);
        }
        let Some(parent) = self
            .dom
            .Parent(&self.right)
            .filter(|node| self.dom.IsEditable(node))
        else {
            return;
        };
        if self.dom.NonEmptyBlockquotesOnOutdentingEnabled() && children.is_empty() {
            return;
        }
        if !self.dom.InsertBefore(&parent, left, Some(&self.right)) {
            return;
        }
        self.dom.SetAttribute(&self.right, &IdAttributeName(), None);
        for child in children {
            self.dom.AppendChild(left, &child);
        }
    }
}
impl<D: SimpleEditingDOM> SimpleEditCommand for SplitElementCommand<D> {
    fn DoApply(&self) -> EditResult {
        let left = self.dom.CloneWithoutChildren(&self.right);
        *self.left.borrow_mut() = Some(left.clone());
        self.ExecuteApply(&left);
        Ok(())
    }
    fn DoUnapply(&self) {
        let left = self.left.borrow().clone();
        let Some(left) =
            left.filter(|node| self.dom.IsEditable(node) && self.dom.IsEditable(&self.right))
        else {
            return;
        };
        let children = self.dom.Children(&left);
        let reference = self.dom.FirstChild(&self.right);
        for child in children {
            self.dom
                .InsertBefore(&self.right, &child, reference.as_ref());
        }
        if let Some(id) = self.dom.Attribute(&left, &IdAttributeName()) {
            self.dom
                .SetAttribute(&self.right, &IdAttributeName(), Some(&id));
        }
        self.dom.Remove(&left);
    }
    fn DoReapply(&self) {
        let left = self.left.borrow().clone();
        if let Some(left) = left {
            self.ExecuteApply(&left);
        }
    }
}
pub struct DeleteFromTextNodeCommand<D: SimpleEditingDOM> {
    dom: Rc<D>,
    node: D::Node,
    offset: usize,
    count: usize,
    text: RefCell<Rc<[u16]>>,
}
impl<D: SimpleEditingDOM> DeleteFromTextNodeCommand<D> {
    pub fn new(dom: Rc<D>, node: D::Node, offset: usize, count: usize) -> Self {
        Self {
            dom,
            node,
            offset,
            count,
            text: RefCell::new(Rc::from([])),
        }
    }
}
impl<D: SimpleEditingDOM> SimpleEditCommand for DeleteFromTextNodeCommand<D> {
    fn DoApply(&self) -> EditResult {
        self.dom.UpdateStyleAndLayoutTree();
        if !self.dom.IsEditable(&self.node) {
            return Ok(());
        }
        let text = self.dom.Text(&self.node);
        if self.offset > text.len() {
            return Ok(());
        }
        *self.text.borrow_mut() =
            Rc::from(&text[self.offset..self.offset.saturating_add(self.count).min(text.len())]);
        self.dom
            .ReplaceText(&self.node, self.offset, self.count, &[]);
        Ok(())
    }
    fn DoUnapply(&self) {
        if self.dom.IsEditable(&self.node) {
            let text = self.text.borrow().clone();
            self.dom.ReplaceText(&self.node, self.offset, 0, &text);
        }
    }
}
pub struct SetNodeAttributeCommand<D: SimpleEditingDOM> {
    dom: Rc<D>,
    node: D::Node,
    name: QualifiedName,
    value: Option<D::AttributeValue>,
    old: RefCell<Option<D::AttributeValue>>,
}
impl<D: SimpleEditingDOM> SetNodeAttributeCommand<D> {
    pub fn new(
        dom: Rc<D>,
        node: D::Node,
        name: QualifiedName,
        value: Option<D::AttributeValue>,
    ) -> Self {
        Self {
            dom,
            node,
            name,
            value,
            old: RefCell::new(None),
        }
    }
}
impl<D: SimpleEditingDOM> SimpleEditCommand for SetNodeAttributeCommand<D> {
    fn DoApply(&self) -> EditResult {
        *self.old.borrow_mut() = self.dom.Attribute(&self.node, &self.name);
        self.dom
            .SetAttribute(&self.node, &self.name, self.value.as_ref());
        Ok(())
    }
    fn DoUnapply(&self) {
        let old = self.old.borrow_mut().take();
        self.dom.SetAttribute(&self.node, &self.name, old.as_ref());
    }
}

// Source insert_into_text_node_command.cc:40-94.
pub struct InsertIntoTextNodeCommand<D: SimpleEditingDOM> {
    dom: Rc<D>,
    node: D::Node,
    offset: usize,
    text: Rc<[u16]>,
    echo: PasswordEchoBehavior,
}
impl<D: SimpleEditingDOM> InsertIntoTextNodeCommand<D> {
    pub fn new(
        dom: Rc<D>,
        node: D::Node,
        offset: usize,
        text: Rc<[u16]>,
        echo: PasswordEchoBehavior,
    ) -> Self {
        debug_assert!(!text.is_empty());
        Self {
            dom,
            node,
            offset,
            text,
            echo,
        }
    }
}
impl<D: SimpleEditingDOM> SimpleEditCommand for InsertIntoTextNodeCommand<D> {
    fn DoApply(&self) -> EditResult {
        let echo = self.dom.ShouldEchoPassword(self.echo);
        if echo {
            self.dom.UpdateStyleAndLayout();
        }
        if !self.dom.IsEditable(&self.node) {
            return Ok(());
        }
        if echo {
            self.dom.MomentarilyRevealLastTypedCharacterIfSecure(
                &self.node,
                self.offset + self.text.len() - 1,
            );
        }
        self.dom.ReplaceText(&self.node, self.offset, 0, &self.text);
        Ok(())
    }
    fn DoUnapply(&self) {
        if self.dom.IsEditable(&self.node) {
            self.dom
                .ReplaceText(&self.node, self.offset, self.text.len(), &[]);
        }
    }
}
// Source set_character_data_command.cc:26-79. Replacement is one CharacterData
// mutation, preserving its observer record and live-range update boundary.
pub struct SetCharacterDataCommand<D: SimpleEditingDOM> {
    dom: Rc<D>,
    node: D::Node,
    offset: usize,
    count: usize,
    text: Rc<[u16]>,
    old: RefCell<Rc<[u16]>>,
    echo: PasswordEchoBehavior,
}
impl<D: SimpleEditingDOM> SetCharacterDataCommand<D> {
    pub fn new(
        dom: Rc<D>,
        node: D::Node,
        offset: usize,
        count: usize,
        text: Rc<[u16]>,
        echo: PasswordEchoBehavior,
    ) -> Self {
        Self {
            dom,
            node,
            offset,
            count,
            text,
            old: RefCell::new(Rc::from([])),
            echo,
        }
    }
}
impl<D: SimpleEditingDOM> SimpleEditCommand for SetCharacterDataCommand<D> {
    fn DoApply(&self) -> EditResult {
        self.dom.UpdateStyleAndLayoutTree();
        if !self.dom.IsEditable(&self.node) {
            return Ok(());
        }
        let old = self.dom.Text(&self.node);
        if self.offset > old.len() {
            return Ok(());
        }
        *self.old.borrow_mut() =
            Rc::from(&old[self.offset..self.offset.saturating_add(self.count).min(old.len())]);
        if self.dom.ShouldEchoPassword(self.echo) {
            self.dom.MomentarilyRevealLastTypedCharacterIfSecure(
                &self.node,
                self.offset.wrapping_add(self.text.len()).wrapping_sub(1),
            );
        }
        self.dom
            .ReplaceText(&self.node, self.offset, self.count, &self.text);
        Ok(())
    }
    fn DoUnapply(&self) {
        self.dom.UpdateStyleAndLayoutTree();
        if !self.dom.IsEditable(&self.node) {
            return;
        }
        let old = self.old.borrow().clone();
        self.dom
            .ReplaceText(&self.node, self.offset, self.text.len(), &old);
    }
}

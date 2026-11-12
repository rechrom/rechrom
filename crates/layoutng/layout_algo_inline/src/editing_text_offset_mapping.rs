// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: core/editing/text_offset_mapping.cc:25-92,106-114,154-271,482-485.
//! Native half of TextOffsetMapping's inline-contents selection. DOM/flat-tree
//! forward traversal remains in Editing. Only the selected block's opaque
//! shared identity crosses the Layout package boundary.
use super::LayoutNode;
use crate::internal::{
    layout_block::LayoutBlock, layout_block_flow::LayoutBlockFlow, layout_node_metadata::Node,
    layout_object::LayoutObject,
};
use foundation::{DynamicTo, Persistent, RuntimeEnabledFeatures};

/// The actual retained LayoutBlockFlow identity, including anonymous blocks.
/// Equality is TextOffsetMapping::InlineContents::operator==; no DOM node ID,
/// LayoutObject pointer, geometry or tree projection is exposed to consumers.
#[derive(Clone)]
pub struct EditingInlineContentsBlockIdentity {
    owner: Persistent<LayoutBlockFlow>,
    before: Option<Persistent<LayoutObject>>,
    first: Option<Persistent<LayoutObject>>,
    last: Option<Persistent<LayoutObject>>,
    after: Option<Persistent<LayoutObject>>,
}
impl PartialEq for EditingInlineContentsBlockIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.owner.Get() == other.owner.Get()
    }
}
impl Eq for EditingInlineContentsBlockIdentity {}
impl EditingInlineContentsBlockIdentity {
    // Source: text_offset_mapping.cc:502-587. Source pointer state is kept on
    // this existing inline-contents owner; only native Position values leave.
    pub fn GetRange(&self) -> Option<(crate::EditingPosition, crate::EditingPosition)> {
        use crate::{EditingPosition as P, EditingPositionAnchorType as A};
        let Some(first) = self.first.as_ref() else {
            let node = unsafe { (&*self.owner.Get()).NonPseudoNode().as_ref() }?;
            return Some((
                P::new(node.InputId(), 0),
                P::WithAnchor(node.InputId(), 0, A::AfterChildren),
            ));
        };
        let first = unsafe { (&*first.Get()).NonPseudoNode().as_ref() }?;
        let last = unsafe { (&*self.last.as_ref()?.Get()).NonPseudoNode().as_ref() }?;
        let begin = if first.IsTextNode() {
            P::new(first.InputId(), 0)
        } else {
            P::WithAnchor(first.InputId(), 0, A::BeforeAnchor)
        };
        let end = if last.IsTextNode() {
            let text = unsafe {
                &*(last as *const Node as *const crate::internal::layout_node_metadata::Text)
            };
            P::new(last.InputId(), text.length() as usize)
        } else {
            P::WithAnchor(last.InputId(), 0, A::AfterAnchor)
        };
        Some((begin, end))
    }
    pub fn LastPositionBeforeBlockFlow(&self) -> crate::EditingPosition {
        use crate::{EditingPosition as P, EditingPositionAnchorType as A};
        if let Some(before) = &self.before {
            let mut block = unsafe { &*before.Get() }.SlowLastChild();
            while let Some(object) = unsafe { block.as_ref() } {
                if let Some(node) = unsafe { object.NonPseudoNode().as_ref() } {
                    return P::WithAnchor(node.InputId(), 0, A::AfterChildren);
                }
                let mut last = std::ptr::null();
                let mut runner = object.SlowFirstChild();
                while let Some(current) = unsafe { runner.as_ref() } {
                    if !current.NonPseudoNode().is_null() {
                        last = current.NonPseudoNode();
                    }
                    runner = NextForInlineContents(current, object);
                }
                if let Some(node) = unsafe { last.as_ref() } {
                    return P::WithAnchor(node.InputId(), 0, A::AfterAnchor);
                }
                block = object.PreviousSibling();
            }
        }
        let node = unsafe { (&*self.owner.Get()).NonPseudoNode().as_ref() }.or_else(|| {
            self.first
                .as_ref()
                .and_then(|first| unsafe { (&*first.Get()).NonPseudoNode().as_ref() })
        });
        node.map_or_else(P::default, |node| {
            P::WithAnchor(node.InputId(), 0, A::BeforeAnchor)
        })
    }
    pub fn FirstPositionAfterBlockFlow(&self) -> crate::EditingPosition {
        use crate::{EditingPosition as P, EditingPositionAnchorType as A};
        if let Some(after) = &self.after {
            let mut block = unsafe { &*after.Get() }.SlowFirstChild();
            while let Some(object) = unsafe { block.as_ref() } {
                if let Some(node) = unsafe { object.NonPseudoNode().as_ref() } {
                    return P::WithAnchor(node.InputId(), 0, A::BeforeAnchor);
                }
                let mut runner = object.SlowFirstChild();
                while let Some(current) = unsafe { runner.as_ref() } {
                    if let Some(node) = unsafe { current.NonPseudoNode().as_ref() } {
                        return P::WithAnchor(node.InputId(), 0, A::BeforeAnchor);
                    }
                    runner = NextForInlineContents(current, object);
                }
                block = object.NextSibling();
            }
        }
        let node = unsafe { (&*self.owner.Get()).NonPseudoNode().as_ref() }.or_else(|| {
            self.last
                .as_ref()
                .and_then(|last| unsafe { (&*last.Get()).NonPseudoNode().as_ref() })
        });
        node.map_or_else(P::default, |node| {
            P::WithAnchor(node.InputId(), 0, A::AfterAnchor)
        })
    }
}
impl std::fmt::Debug for EditingInlineContentsBlockIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EditingInlineContentsBlockIdentity")
    }
}

fn HasNonPseudoNode(parent: &LayoutObject) -> bool {
    if !parent.NonPseudoNode().is_null() {
        return true;
    }
    let mut runner = parent as *const LayoutObject;
    while let Some(object) = unsafe { runner.as_ref() } {
        if !object.NonPseudoNode().is_null() {
            return true;
        }
        runner = object.NextInPreOrder(parent);
    }
    false
}
fn CanBeInlineContentsContainer(object: &LayoutObject) -> bool {
    let Some(block) = (unsafe { DynamicTo::<LayoutBlockFlow>(object).as_ref() }) else {
        return false;
    };
    if !block.ChildrenInline() || block.IsInline() || block.IsRuby() {
        return false;
    }
    !block.NonPseudoNode().is_null() || HasNonPseudoNode(block)
}
fn RootInlineContentsContainerOf(block: &LayoutBlockFlow) -> &LayoutBlockFlow {
    debug_assert!(block.ChildrenInline());
    let mut root = block;
    let mut runner = block.ContainingBlock();
    while let Some(containing) = unsafe { runner.as_ref() } {
        let Some(flow) = (unsafe { DynamicTo::<LayoutBlockFlow>(containing).as_ref() }) else {
            break;
        };
        if !containing.ChildrenInline() {
            break;
        }
        root = flow;
        runner = containing.ContainingBlock();
    }
    debug_assert!(!root.IsInline());
    root
}
fn ComputeInlineContentsAsBlockFlow(object: &LayoutObject) -> Option<&LayoutBlockFlow> {
    let mut block = DynamicTo::<LayoutBlock>(object);
    if block.is_null() {
        block = object.ContainingBlock();
    }
    debug_assert!(!block.is_null());
    let block = unsafe { block.as_ref() }?;
    let flow = unsafe { DynamicTo::<LayoutBlockFlow>(block).as_ref() }?;
    if !flow.ChildrenInline() {
        return None;
    }
    let flow = if flow.IsInline() {
        RootInlineContentsContainerOf(flow)
    } else {
        flow
    };
    CanBeInlineContentsContainer(flow).then_some(flow)
}
fn ShouldSkipChildren(node: &Node) -> bool {
    let facts = node.InputEditingFacts();
    facts.is_text_control || facts.has_user_agent_shadow_root
}
fn NextForInlineContents(object: &LayoutObject, container: &LayoutObject) -> *mut LayoutObject {
    if object.IsBlockInInline()
        || unsafe { object.NonPseudoNode().as_ref() }.is_some_and(ShouldSkipChildren)
    {
        object.NextInPreOrderAfterChildren(container)
    } else {
        object.NextInPreOrder(container)
    }
}
fn NodeIsDescendantOf(node: &Node, ancestor: *const Node) -> bool {
    let Some(ancestor) = (unsafe { ancestor.as_ref() }) else {
        return false;
    };
    if node.InputEditingFacts().tree_scope_id != ancestor.InputEditingFacts().tree_scope_id {
        return false;
    }
    let mut parent = node.parentNode();
    while let Some(current) = unsafe { parent.as_ref() } {
        let current_node: &Node = current;
        if std::ptr::eq(current_node, ancestor) {
            return true;
        }
        parent = current_node.parentNode();
    }
    false
}
/// Retains the source first/last and block-in-inline scan to preserve its null
/// result for anonymous wrappers. Only equality's block identity is published.
fn CreateInlineContentsFromBlockFlow(
    flow: &LayoutBlockFlow,
    target: &LayoutObject,
) -> Option<EditingInlineContentsBlockIdentity> {
    debug_assert!(flow.ChildrenInline());
    debug_assert!(!target.NonPseudoNode().is_null());
    let mut object = flow.FirstChild();
    let mut before: *const LayoutObject = std::ptr::null();
    let mut first: *const LayoutObject = std::ptr::null();
    let mut last: *const LayoutObject = std::ptr::null();
    while let Some(current) = unsafe { object.as_ref() } {
        if !current.NonPseudoNode().is_null() {
            last = object;
            if first.is_null() {
                first = object;
            }
        }
        if std::ptr::eq(current, target) {
            last = first;
            break;
        }
        if current.IsFloatingOrOutOfFlowPositioned()
            && unsafe { current.GetNode().as_ref() }
                .is_some_and(|node| NodeIsDescendantOf(node, target.GetNode()))
        {
            last = first;
            break;
        }
        if current.IsBlockInInline() {
            if target.IsDescendantOf(current) {
                last = first;
                break;
            }
            before = object;
            first = std::ptr::null();
            last = std::ptr::null();
        }
        object = NextForInlineContents(current, flow);
    }
    if first.is_null() {
        if flow.NonPseudoNode().is_null()
            && RuntimeEnabledFeatures::CreateInlineContentsAnonymousBlockEnabled()
        {
            return None;
        }
        return Some(EditingInlineContentsBlockIdentity {
            owner: Persistent::from_ptr(flow as *const _ as *mut _),
            before: None,
            first: None,
            last: None,
            after: None,
        });
    }
    let mut after: *const LayoutObject = std::ptr::null();
    while let Some(current) = unsafe { object.as_ref() } {
        if current.IsBlockInInline() {
            after = object;
            break;
        }
        if !current.NonPseudoNode().is_null() {
            last = object;
        }
        object = NextForInlineContents(current, flow);
    }
    debug_assert!(!last.is_null());
    debug_assert!(before.is_null() || unsafe { &*before }.IsBlockInInline());
    debug_assert!(after.is_null() || unsafe { &*after }.IsBlockInInline());
    let flow_object: &LayoutObject = flow;
    debug_assert!(unsafe { &*first }.IsDescendantOf(flow_object));
    debug_assert!(unsafe { &*last }.IsDescendantOf(flow_object));
    Some(EditingInlineContentsBlockIdentity {
        owner: Persistent::from_ptr(flow as *const _ as *mut _),
        before: (!before.is_null()).then(|| Persistent::from_ptr(before as *mut _)),
        first: Some(Persistent::from_ptr(first as *mut _)),
        last: Some(Persistent::from_ptr(last as *mut _)),
        after: (!after.is_null()).then(|| Persistent::from_ptr(after as *mut _)),
    })
}
impl LayoutNode<'_> {
    /// Native ComputeInlineContentsFromNode. Source UA-shadow adjustment uses
    /// the canonical DOM facts supplied with this node, including non-control
    /// UA roots; the text-control-specific native control_host_ is not reused.
    pub fn EditingInlineContentsBlockIdentity(&self) -> Option<EditingInlineContentsBlockIdentity> {
        let _scope = crate::layout_engine::LayoutPassScope::new(
            &self.output.engine.algorithms_,
            &self.output.engine.objects_,
        );
        let node = unsafe { self.object.NonPseudoNode().as_ref() }?;
        let object = unsafe { node.GetLayoutObject().as_ref() }?;
        let flow = ComputeInlineContentsAsBlockFlow(object)?;
        if flow.IsAnonymous() {
            if let Some(host_id) = node.InputEditingFacts().user_agent_shadow_host_id {
                let host = self.output.engine.tree_.as_ref()?.OutputNode(host_id)?;
                return CreateInlineContentsFromBlockFlow(flow, host);
            }
        }
        CreateInlineContentsFromBlockFlow(flow, object)
    }
}

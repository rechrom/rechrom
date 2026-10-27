//! Cached facts collected from the resident native LayoutObject tree.
//!
//! This source is included by `layout_engine` at the reciprocal assembly
//! boundary. Keeping the full-tree walk here gives both interval traces and
//! sampling profilers the same invalidation package owner.

use foundation::TextEmphasisMark;

use crate::internal::{
    layout_input::NodeKind, layout_object::LayoutObject,
    layout_object_factory_set::LayoutObjectFactorySet, layout_pass_scope::LayoutObjectFactoryScope,
};

#[derive(Clone, Copy)]
pub(super) struct LayoutTreeFacts {
    pub(super) object_count: usize,
    pub(super) contains_annotations: bool,
    pub(super) has_floating_object: bool,
    pub(super) has_form_control: bool,
}

#[inline(never)]
pub(super) fn collect_layout_tree_facts(
    root: &LayoutObject,
    objects: &LayoutObjectFactorySet,
) -> LayoutTreeFacts {
    let mut trace =
        layout_invalidation::trace_span(layout_invalidation::TracePhase::CollectTreeFacts);
    // C++ virtual dispatch remains available while walking the tree before
    // LayoutPassScope. Rust's explicit SVG vtable needs the same factory
    // callbacks during this one-time scan.
    let _object_factory_scope = LayoutObjectFactoryScope::new(objects);
    let mut facts = LayoutTreeFacts {
        object_count: 0,
        contains_annotations: false,
        has_floating_object: false,
        has_form_control: false,
    };
    let mut object = root as *const LayoutObject as *mut LayoutObject;
    while !object.is_null() {
        let current = unsafe { &*object };
        facts.object_count += 1;
        facts.has_floating_object |= current.IsFloating();
        let node = current.GetNode();
        facts.has_form_control |=
            !node.is_null() && unsafe { &*node }.InputKind() == NodeKind::kFormControl;
        facts.contains_annotations |= current.IsInlineRubyText()
            || current.StyleRef().GetTextEmphasisMark() != TextEmphasisMark::kNone;
        object = current.NextInPreOrder(root);
    }
    trace.set("objects", facts.object_count as f64);
    facts
}

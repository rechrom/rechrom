#![allow(non_snake_case)]

//! Resident DOM/CSSOM ownership and typed document mutations.
//!
//! `DocumentEngine` owns the mutable document arena. It does not resolve
//! style, run layout, fetch resources, execute scripts or schedule frames.

use cssom::CSSStyleSheet;
use dom::{
    dom_mutation::{DOMMutation, DOMMutationError, DOMMutationType},
    error::{DOMException, DOMExceptionKind},
    persistent_document::DOMNodeType,
    Document, DOM,
};
use html::html_parser::ParseHTMLFragment;
use layoutng_assembly::internal::layout_input::Offset;
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleMutation {
    Recalculate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutMutation {
    InvalidateGeometry,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConnectedSubtreeRequest {
    pub root_node_id: u64,
    pub prepare_scripts: bool,
    pub image_request: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceRequest {
    ResetImage { node_id: u64 },
    DiscoverConnectedSubtree(ConnectedSubtreeRequest),
}

/// Cross-engine work produced by one document mutation.
///
/// The detailed selector invalidation set and accumulated style impact remain
/// resident in the Document, matching Blink's Node/Document dirty bits. These
/// fields only tell the owner which lifecycle engines need an opportunity to
/// consume that state and which resource discovery work must be routed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DocumentEffects {
    pub style: Option<StyleMutation>,
    pub layout: Option<LayoutMutation>,
    pub resource: Vec<ResourceRequest>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollEffect {
    pub target_node_id: u64,
    pub event_target_node_id: u64,
    pub before: Offset,
    pub offset: Offset,
}

/// Owns one resident DOM/CSSOM arena. Clone only the explicit `Handle` when an
/// existing parser or Web API requires a shared arena reference.
pub struct DocumentEngine {
    dom: Rc<RefCell<DOM>>,
}

impl Default for DocumentEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DocumentEngine {
    pub fn new() -> Self {
        Self {
            dom: Rc::new(RefCell::new(DOM::new())),
        }
    }

    pub fn Handle(&self) -> Rc<RefCell<DOM>> {
        self.dom.clone()
    }

    pub fn borrow(&self) -> Ref<'_, DOM> {
        self.dom.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<'_, DOM> {
        self.dom.borrow_mut()
    }

    /// The document-side half of the initial rendering barrier. Navigation
    /// and frame policy stay with Page/OpenEngine; DOM tree inspection stays
    /// with the document owner.
    pub fn HasRenderableRoot(&self) -> bool {
        fn Contains(document: &Document, index: usize) -> bool {
            document.Node(index).IsHTMLElement("body")
                || document.Node(index).IsHTMLElement("frameset")
                || document
                    .Node(index)
                    .Children()
                    .iter()
                    .any(|&child| Contains(document, child))
        }
        let owner = self.dom.borrow();
        let document = owner.GetDocument();
        Contains(document, document.Root())
    }

    pub fn AppendStyleSheet(&self, sheet: CSSStyleSheet) -> DocumentEffects {
        Self::AppendStyleSheetTo(self.dom.borrow_mut().GetDocumentMut(), sheet)
    }

    pub fn ApplyAnimationMutation(&self, mutation: animation::DocumentMutation) -> DocumentEffects {
        match mutation {
            animation::DocumentMutation::ApplyStyleBatch(samples) => {
                let mut dom = self.dom.borrow_mut();
                for sample in samples {
                    if dom.GetDocument().FindNodeById(sample.node_id).is_none() {
                        dom::error::invalid_argument("Animation target missing");
                    }
                    dom.SetAnimationStyle(sample.node_id, sample.effect_id, sample.declarations);
                }
                DocumentEffects {
                    style: Some(StyleMutation::Recalculate),
                    layout: None,
                    resource: Vec::new(),
                }
            }
        }
    }

    /// Update the DOM-visible scroll state and return the semantic event
    /// target. Lifecycle admission and compositor synchronization remain with
    /// the Page which connects Document, Paint and Compositor.
    pub fn ApplyScrollOffset(&self, target_node_id: u64, offset: Offset) -> Option<ScrollEffect> {
        if !offset.x.is_finite() || !offset.y.is_finite() {
            dom::error::invalid_argument("invalid scroll mutation");
        }
        let mut owner = self.dom.borrow_mut();
        let tree = owner.GetDocument();
        let Some(target) = tree.FindNodeById(target_node_id) else {
            dom::error::invalid_argument("invalid scroll mutation");
        };
        let before = tree.ScrollOffsetFor(target);
        if before == offset {
            return None;
        }
        let event_target_node_id =
            if tree.Node(target).IsHTMLElement("html") || tree.Node(target).IsHTMLElement("body") {
                tree.Node(tree.Root()).Id()
            } else {
                target_node_id
            };
        owner.GetDocumentMut().SetScrollOffset(target, offset);
        Some(ScrollEffect {
            target_node_id,
            event_target_node_id,
            before,
            offset,
        })
    }

    /// The streaming parser temporarily owns the same arena while the DOM
    /// facade holds an empty swap slot. Route that borrowed-arena case through
    /// the same document boundary without re-borrowing the facade.
    pub fn AppendStyleSheetTo(document: &mut Document, sheet: CSSStyleSheet) -> DocumentEffects {
        document.AppendStyleSheet(sheet);
        DocumentEffects {
            style: Some(StyleMutation::Recalculate),
            // Active-sheet diffing decides whether computed geometry changed.
            layout: None,
            resource: Vec::new(),
        }
    }

    pub fn ApplyDOMMutation(
        &self,
        mutation: &DOMMutation,
    ) -> Result<DocumentEffects, DOMMutationError> {
        if mutation.mutation_type == DOMMutationType::kParseDocument {
            return Err(DOMException {
                kind: DOMExceptionKind::InvalidArgument,
                message: "parse document requires the document parser",
            });
        }
        let attribute_mutation = matches!(
            mutation.mutation_type,
            DOMMutationType::kSetAttribute | DOMMutationType::kRemoveAttribute
        );
        let changed_image_request = self.ChangedImageRequest(mutation);
        self.dom
            .borrow_mut()
            .ApplyMutationWithFragmentParser(mutation, ParseInnerHTMLMutation)?;
        let connected_subtree = match mutation.mutation_type {
            DOMMutationType::kAppendChild | DOMMutationType::kInsertBefore => {
                Some(ConnectedSubtreeRequest {
                    root_node_id: mutation.child_node_id,
                    prepare_scripts: true,
                    image_request: true,
                })
            }
            DOMMutationType::kSetAttribute
            | DOMMutationType::kSetTextContent
            | DOMMutationType::kSetInnerHTML => Some(ConnectedSubtreeRequest {
                root_node_id: mutation.target_node_id,
                prepare_scripts: mutation.mutation_type != DOMMutationType::kSetInnerHTML,
                image_request: mutation.mutation_type != DOMMutationType::kSetAttribute
                    || IsImageRequestAttribute(mutation),
            }),
            _ => None,
        };
        let mut resource = Vec::with_capacity(
            usize::from(changed_image_request.is_some()) + usize::from(connected_subtree.is_some()),
        );
        if let Some(node_id) = changed_image_request {
            resource.push(ResourceRequest::ResetImage { node_id });
        }
        if let Some(request) = connected_subtree {
            resource.push(ResourceRequest::DiscoverConnectedSubtree(request));
        }
        Ok(DocumentEffects {
            // The Document has already recorded the precise dirty nodes. This
            // is a lifecycle admission signal, not a duplicate invalidation.
            style: Some(StyleMutation::Recalculate),
            // Attribute mutations retain geometry until computed-style diffing
            // proves layout changed. Tree/text mutations cannot retain it.
            layout: (!attribute_mutation).then_some(LayoutMutation::InvalidateGeometry),
            resource,
        })
    }

    fn ChangedImageRequest(&self, mutation: &DOMMutation) -> Option<u64> {
        if !matches!(
            mutation.mutation_type,
            DOMMutationType::kSetAttribute | DOMMutationType::kRemoveAttribute
        ) || !mutation.namespace_uri.is_empty()
            || !mutation.name.eq_ignore_ascii_case("src")
        {
            return None;
        }
        let owner = self.dom.borrow();
        let tree = owner.GetDocument();
        tree.FindNodeById(mutation.target_node_id)
            .filter(|&index| tree.Node(index).IsHTMLElement("img"))
            .filter(|&index| {
                mutation.mutation_type == DOMMutationType::kRemoveAttribute
                    || tree
                        .Node(index)
                        .FindAttribute("src")
                        .is_some_and(|attribute| attribute.value != mutation.value)
            })
            .map(|_| mutation.target_node_id)
    }
}

fn IsImageRequestAttribute(mutation: &DOMMutation) -> bool {
    matches!(
        mutation.mutation_type,
        DOMMutationType::kSetAttribute | DOMMutationType::kRemoveAttribute
    ) && mutation.namespace_uri.is_empty()
        && matches!(
            mutation.name.to_ascii_lowercase().as_str(),
            "src" | "srcset" | "sizes" | "referrerpolicy"
        )
}

fn ParseInnerHTMLMutation(
    document: &mut Document,
    mutation: &DOMMutation,
) -> Result<(), DOMMutationError> {
    let Some(mut target) = document.FindNodeById(mutation.target_node_id) else {
        return Err(DOMException {
            kind: DOMExceptionKind::InvalidArgument,
            message: "invalid innerHTML target",
        });
    };
    if !matches!(
        document.Node(target).Type(),
        DOMNodeType::kElement | DOMNodeType::kDocumentFragment
    ) {
        return Err(DOMException {
            kind: DOMExceptionKind::InvalidArgument,
            message: "invalid innerHTML target",
        });
    }
    let context = document.Node(target).clone();
    let fragment = ParseHTMLFragment(document, &context, &mutation.value);
    if let Some(contents) = document.TemplateContents(target) {
        target = contents;
    }
    while let Some(&child) = document.Node(target).Children().last() {
        document.Remove(child);
    }
    document.TakeAllChildren(fragment, target);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dom::persistent_document::{DOMAttribute, DOMNamespace};

    fn connected_image(engine: &DocumentEngine) -> u64 {
        let mut owner = engine.borrow_mut();
        let tree = owner.GetDocumentMut();
        let image = tree.CreateElementDefault(DOMNamespace::kHTML, "img".into());
        tree.SetAttribute(
            image,
            DOMAttribute {
                local_name: "src".into(),
                value: "before.png".into(),
                ..Default::default()
            },
        );
        tree.AppendChild(tree.Root(), image);
        tree.Node(image).Id()
    }

    #[test]
    fn attribute_mutation_schedules_style_and_resource_without_eager_layout() {
        let engine = DocumentEngine::new();
        let image_id = connected_image(&engine);
        let effects = engine
            .ApplyDOMMutation(&DOMMutation {
                mutation_type: DOMMutationType::kSetAttribute,
                target_node_id: image_id,
                name: "src".into(),
                value: "after.png".into(),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(effects.style, Some(StyleMutation::Recalculate));
        assert_eq!(effects.layout, None);
        assert_eq!(
            effects.resource,
            vec![
                ResourceRequest::ResetImage { node_id: image_id },
                ResourceRequest::DiscoverConnectedSubtree(ConnectedSubtreeRequest {
                    root_node_id: image_id,
                    prepare_scripts: true,
                    image_request: true,
                }),
            ]
        );
    }

    #[test]
    fn tree_mutation_invalidates_geometry_and_routes_subtree_discovery() {
        let engine = DocumentEngine::new();
        let (root_id, child_id) = {
            let mut owner = engine.borrow_mut();
            let tree = owner.GetDocumentMut();
            let child = tree.CreateElementDefault(DOMNamespace::kHTML, "div".into());
            (tree.Node(tree.Root()).Id(), tree.Node(child).Id())
        };
        let effects = engine
            .ApplyDOMMutation(&DOMMutation {
                mutation_type: DOMMutationType::kAppendChild,
                target_node_id: root_id,
                child_node_id: child_id,
                ..Default::default()
            })
            .unwrap();

        assert_eq!(effects.style, Some(StyleMutation::Recalculate));
        assert_eq!(effects.layout, Some(LayoutMutation::InvalidateGeometry));
        assert_eq!(
            effects.resource,
            vec![ResourceRequest::DiscoverConnectedSubtree(
                ConnectedSubtreeRequest {
                    root_node_id: child_id,
                    prepare_scripts: true,
                    image_request: true,
                }
            )]
        );
    }

    #[test]
    fn scroll_mutation_updates_document_state_and_suppresses_noop() {
        let engine = DocumentEngine::new();
        let node_id = {
            let mut owner = engine.borrow_mut();
            let tree = owner.GetDocumentMut();
            let node = tree.CreateElementDefault(DOMNamespace::kHTML, "div".into());
            tree.AppendChild(tree.Root(), node);
            tree.Node(node).Id()
        };
        let offset = Offset { x: 3.0, y: 14.0 };
        let effect = engine.ApplyScrollOffset(node_id, offset).unwrap();

        assert_eq!(effect.target_node_id, node_id);
        assert_eq!(effect.event_target_node_id, node_id);
        assert_eq!(effect.before, Offset::default());
        assert_eq!(effect.offset, offset);
        let owner = engine.borrow();
        let tree = owner.GetDocument();
        assert_eq!(
            tree.ScrollOffsetFor(tree.FindNodeById(node_id).unwrap()),
            offset
        );
        drop(owner);
        assert!(engine.ApplyScrollOffset(node_id, offset).is_none());
    }
}

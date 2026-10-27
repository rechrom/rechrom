// Copyright 2026 The Chromium Authors. BSD-style license; see Chromium LICENSE.
// cpp: third_party/blink/renderer/core/css/style_engine.cc:4073-4100

#![allow(non_snake_case)]

use std::rc::Rc;

/// Operations required by the FlatTree style-recalc walk. The lifecycle
/// owner supplies the concrete DOM operations while this package owns the
/// traversal order and propagation rules.
pub trait FlatTreeStyleWalkBackend {
    type Element;
    type ContainerNode;
    type StyleRecalcChange;
    type StyleRecalcContext;

    fn ElementRecalcStyle(
        element: &Self::Element,
        change: Self::StyleRecalcChange,
        context: &Self::StyleRecalcContext,
    );
    fn ElementStyleRecalcParent(element: &Self::Element) -> Option<Rc<Self::ContainerNode>>;
    fn ContainerStyleRecalcParent(node: &Self::ContainerNode) -> Option<Rc<Self::ContainerNode>>;
    fn ContainerAsElement(node: &Self::ContainerNode) -> Option<Rc<Self::Element>>;
    fn ElementRecalcTraversalRootAncestor(element: &Self::Element);
    fn ContainerClearChildNeedsStyleRecalc(node: &Self::ContainerNode);
    fn ElementIsHTMLBody(element: &Self::Element) -> bool;
}

/// Recalculate one FlatTree root, then walk its style-recalc ancestors in the
/// same order as Blink. Returns whether writing mode and direction must be
/// propagated back to the HTML root after the caller clears its retained root.
pub fn RecalcStyleRoot<B: FlatTreeStyleWalkBackend>(
    root: &B::Element,
    root_has_flat_tree_parent: bool,
    change: B::StyleRecalcChange,
    context: &B::StyleRecalcContext,
    in_interleaved_style_recalc: bool,
) -> bool {
    let mut trace = browser_tracing::span("style_walk", "FlatTree.RecalcStyleRoot");
    trace.set("interleaved", in_interleaved_style_recalc as u8 as f64);
    B::ElementRecalcStyle(root, change, context);
    let mut ancestor = B::ElementStyleRecalcParent(root);
    let mut ancestors = 0usize;
    while let Some(current) = ancestor {
        ancestors += 1;
        if !in_interleaved_style_recalc {
            if let Some(element) = B::ContainerAsElement(&current) {
                B::ElementRecalcTraversalRootAncestor(&element);
            }
        }
        B::ContainerClearChildNeedsStyleRecalc(&current);
        ancestor = B::ContainerStyleRecalcParent(&current);
    }
    trace.set("ancestors", ancestors as f64);
    !root_has_flat_tree_parent || B::ElementIsHTMLBody(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct Element {
        name: &'static str,
        parent: Option<Rc<Container>>,
        is_body: bool,
    }

    struct Container {
        name: &'static str,
        element: Option<Rc<Element>>,
        parent: Option<Rc<Container>>,
    }

    thread_local! {
        static EVENTS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    }

    struct Backend;
    impl FlatTreeStyleWalkBackend for Backend {
        type Element = Element;
        type ContainerNode = Container;
        type StyleRecalcChange = ();
        type StyleRecalcContext = ();

        fn ElementRecalcStyle(element: &Element, _: (), _: &()) {
            EVENTS.with(|events| events.borrow_mut().push(format!("recalc:{}", element.name)));
        }

        fn ElementStyleRecalcParent(element: &Element) -> Option<Rc<Container>> {
            element.parent.clone()
        }

        fn ContainerStyleRecalcParent(node: &Container) -> Option<Rc<Container>> {
            node.parent.clone()
        }

        fn ContainerAsElement(node: &Container) -> Option<Rc<Element>> {
            node.element.clone()
        }

        fn ElementRecalcTraversalRootAncestor(element: &Element) {
            EVENTS.with(|events| {
                events
                    .borrow_mut()
                    .push(format!("ancestor:{}", element.name))
            });
        }

        fn ContainerClearChildNeedsStyleRecalc(node: &Container) {
            EVENTS.with(|events| events.borrow_mut().push(format!("clear:{}", node.name)));
        }

        fn ElementIsHTMLBody(element: &Element) -> bool {
            element.is_body
        }
    }

    #[test]
    fn walks_ancestors_in_order_and_suppresses_callbacks_during_interleaving() {
        let html = Rc::new(Element {
            name: "html",
            parent: None,
            is_body: false,
        });
        let outer = Rc::new(Container {
            name: "outer",
            element: Some(html),
            parent: None,
        });
        let section = Rc::new(Element {
            name: "section",
            parent: Some(outer.clone()),
            is_body: false,
        });
        let inner = Rc::new(Container {
            name: "inner",
            element: Some(section),
            parent: Some(outer),
        });
        let root = Element {
            name: "root",
            parent: Some(inner),
            is_body: false,
        };

        assert!(!RecalcStyleRoot::<Backend>(&root, true, (), &(), false));
        EVENTS.with(|events| {
            assert_eq!(
                events.take(),
                [
                    "recalc:root",
                    "ancestor:section",
                    "clear:inner",
                    "ancestor:html",
                    "clear:outer",
                ]
            )
        });

        assert!(RecalcStyleRoot::<Backend>(&root, false, (), &(), true));
        EVENTS.with(|events| {
            assert_eq!(events.take(), ["recalc:root", "clear:inner", "clear:outer"])
        });
    }
}

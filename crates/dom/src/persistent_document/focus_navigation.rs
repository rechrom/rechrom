//! Chromium Document's live sequential-focus-navigation Range owner.
//! Source: core/dom/document.cc:6232-6247,6257-6351.
//! The Range is registered with the document's normal mutation fixups.
use crate::persistent_document::{DOMNodeType, PersistentDocument};
use crate::range::DOMBoundaryPoint;

impl PersistentDocument {
    pub fn SetSequentialFocusNavigationStartingPoint(&mut self, node: Option<usize>) {
        let Some(node) = node.filter(|&node| self.NodeDocumentHandle(node) == self.RootHandle())
        else {
            self.sequential_focus_navigation_starting_point = None;
            return;
        };
        let id = self.Node(node).Id();
        let end = if matches!(
            self.Node(node).Type(),
            DOMNodeType::kText | DOMNodeType::kComment | DOMNodeType::kProcessingInstruction
        ) {
            self.Node(node).Data().encode_utf16().count()
        } else {
            self.Node(node).Children().len()
        };
        let start = DOMBoundaryPoint {
            node_id: id,
            offset: 0,
        };
        let end = DOMBoundaryPoint {
            node_id: id,
            offset: end,
        };
        if let Some(range) = &self.sequential_focus_navigation_starting_point {
            self.SetRange(range, start, end)
                .expect("node contents define a valid same-document range");
        } else {
            self.sequential_focus_navigation_starting_point = Some(
                self.CreateRange(start, end)
                    .expect("node contents define a valid range"),
            );
        }
    }
    pub fn SequentialFocusNavigationRangeStart(&self) -> Option<usize> {
        let range = self.Range(self.sequential_focus_navigation_starting_point.as_ref()?)?;
        self.FindNodeById(range.start.node_id)
    }
    /// The focused element takes precedence. Scroll-marker activation's source
    /// adjustment is applied by FocusController using the marker's Layout owner.
    pub fn SequentialFocusNavigationStartingPoint(
        &self,
        forward: bool,
        focused: Option<usize>,
    ) -> Option<usize> {
        if focused.is_some() {
            return focused;
        }
        let range = self.Range(self.sequential_focus_navigation_starting_point.as_ref()?)?;
        let container = self.FindNodeById(range.start.node_id)?;
        if !self.IsConnectedToDocument(container) {
            return None;
        }
        if !range.Collapsed() {
            if self.Node(container).Type() == DOMNodeType::kElement {
                return Some(container);
            }
            let mut neighbor = if forward {
                self.FocusPreviousNode(container, false)
            } else {
                self.FocusNextNode(container, false)
            };
            while let Some(node) = neighbor {
                if self.Node(node).Type() == DOMNodeType::kElement {
                    return Some(node);
                }
                neighbor = if forward {
                    self.FocusPreviousNode(node, false)
                } else {
                    self.FocusNextNode(node, false)
                };
            }
            return self.ParentOrShadowHostElement(container);
        }
        if self.Node(container).Type() == DOMNodeType::kElement
            && self.Node(container).Children().is_empty()
            && range.start.offset == 0
        {
            return Some(container);
        }
        let first = self
            .Node(container)
            .Children()
            .get(range.start.offset)
            .copied()
            .or_else(|| self.FocusNextSkippingChildren(container, false))?;
        if let Some(host) = self.ShadowHost(first) {
            return Some(host);
        }
        if forward {
            let mut previous = self.FocusPreviousNode(first, true);
            while let Some(node) = previous {
                if self.Node(node).Type() == DOMNodeType::kElement {
                    return Some(node);
                }
                previous = self.FocusPreviousNode(node, true);
            }
        }
        let mut next = Some(first);
        while let Some(node) = next {
            if self.Node(node).Type() == DOMNodeType::kElement {
                return Some(node);
            }
            next = self.FocusNextNode(node, true);
        }
        None
    }
    fn FocusTraversalParent(&self, node: usize, flat: bool) -> Option<usize> {
        if !flat {
            return self.Node(node).Parent();
        }
        self.FlatTreeParent(node)
    }
    pub(super) fn FocusNextSkippingChildren(&self, mut node: usize, flat: bool) -> Option<usize> {
        loop {
            let parent = self.FocusTraversalParent(node, flat)?;
            let next = if flat {
                self.FlatTreeNextSibling(node)
            } else {
                let children = self.Node(parent).Children();
                let position = children.iter().position(|&child| child == node)?;
                children.get(position + 1).copied()
            };
            if next.is_some() {
                return next;
            }
            node = parent;
        }
    }
    pub(super) fn FocusNextNode(&self, node: usize, flat: bool) -> Option<usize> {
        let child = if flat {
            self.FlatTreeChildren(node).first().copied()
        } else {
            self.Node(node).Children().first().copied()
        };
        child.or_else(|| self.FocusNextSkippingChildren(node, flat))
    }
    pub(super) fn FocusPreviousNode(&self, node: usize, flat: bool) -> Option<usize> {
        let parent = self.FocusTraversalParent(node, flat)?;
        let previous = if flat {
            self.FlatTreePreviousSibling(node)
        } else {
            let children = self.Node(parent).Children();
            let position = children.iter().position(|&child| child == node)?;
            position
                .checked_sub(1)
                .and_then(|position| children.get(position))
                .copied()
        };
        let Some(mut previous) = previous else {
            return Some(parent);
        };
        loop {
            let last = if flat {
                self.FlatTreeChildren(previous).last().copied()
            } else {
                self.Node(previous).Children().last().copied()
            };
            let Some(last) = last else {
                return Some(previous);
            };
            previous = last;
        }
    }
}

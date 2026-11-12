//! DOM-tree NodeTraversal/ElementTraversal source primitives. Queries borrow
//! the canonical sibling/parent links; no traversal list is materialized.
use super::{DOMNodeType, PersistentDocument};
impl PersistentDocument {
    pub fn NextNodeWithin(
        &self,
        mut node: usize,
        stay_within: Option<usize>,
        skip_children: bool,
    ) -> Option<usize> {
        if !skip_children {
            if let Some(&child) = self.Node(node).Children().first() {
                return Some(child);
            }
        }
        loop {
            if Some(node) == stay_within {
                return None;
            }
            if let Some(sibling) = self.Node(node).NextSibling() {
                return Some(sibling);
            }
            node = self.Node(node).Parent()?;
        }
    }
    pub fn ContainsNode(&self, ancestor: usize, node: usize) -> bool {
        let mut node = Some(node);
        while let Some(current) = node {
            if current == ancestor {
                return true;
            }
            node = self.Node(current).Parent();
        }
        false
    }
    pub fn FirstElementChild(&self, node: usize) -> Option<usize> {
        self.Node(node)
            .Children()
            .iter()
            .copied()
            .find(|&n| self.Node(n).Type() == DOMNodeType::kElement)
    }
    pub fn NextElementSibling(&self, node: usize) -> Option<usize> {
        let mut node = self.Node(node).NextSibling();
        while let Some(current) = node {
            if self.Node(current).Type() == DOMNodeType::kElement {
                return Some(current);
            }
            node = self.Node(current).NextSibling();
        }
        None
    }
}

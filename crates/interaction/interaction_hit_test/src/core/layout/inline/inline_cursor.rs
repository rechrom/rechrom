/*
 * Copyright (C) 2017 The Chromium Authors
 * Use of this source code is governed by a BSD-style license that can be
 * found in the Chromium LICENSE file.
 */
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
use crate::core::paint::box_fragment_painter::{FragmentItemRef, FragmentItems, PhysicalFragment};

// Source: core/layout/inline/inline_cursor.h:676-706;
// core/layout/inline/inline_cursor.cc:1326-1374. The source backward cursor's
// index vector is moved, never reconstructed by the interaction adapter.
// Cursor clones share immutable source indices and retain their own position.
pub struct InlineCursor<'a, L: ?Sized> {
    pub items: &'a dyn FragmentItems<L>,
    pub position: Option<usize>,
    pub parent: Option<usize>,
    siblings: std::rc::Rc<Vec<usize>>,
    sibling_index: Option<usize>,
}
impl<L: ?Sized> Clone for InlineCursor<'_, L> {
    fn clone(&self) -> Self {
        Self {
            items: self.items,
            position: self.position,
            parent: self.parent,
            siblings: self.siblings.clone(),
            sibling_index: self.sibling_index,
        }
    }
}
impl<'a, L: ?Sized> InlineCursor<'a, L> {
    /// A forward source cursor positioned on one inline box. Its painter
    /// visits descendants, so constructing it must not walk root siblings.
    pub fn ForInlineBox(items: &'a dyn FragmentItems<L>, position: usize) -> Self {
        Self {
            items,
            position: Some(position),
            parent: None,
            siblings: std::rc::Rc::new(Vec::new()),
            sibling_index: None,
        }
    }
    pub fn Root(items: &'a dyn FragmentItems<L>) -> Self {
        Self::ForParent(items, None)
    }
    fn ForParent(items: &'a dyn FragmentItems<L>, parent: Option<usize>) -> Self {
        let siblings: std::rc::Rc<Vec<usize>> = std::rc::Rc::new(items.SiblingItemIndices(parent));
        let sibling_index = siblings.len().checked_sub(1);
        Self {
            items,
            position: sibling_index.map(|i| siblings[i]),
            parent,
            siblings,
            sibling_index,
        }
    }
    pub fn Current(&self) -> Option<FragmentItemRef<'a, L>> {
        self.position.map(|i| self.items.At(i))
    }
    pub fn CursorForDescendants(&self) -> Self {
        if let Some(parent) = self.position {
            Self::ForParent(self.items, Some(parent))
        } else {
            Self {
                items: self.items,
                position: None,
                parent: self.parent,
                siblings: std::rc::Rc::new(Vec::new()),
                sibling_index: None,
            }
        }
    }
    pub fn MoveToPreviousSibling(&mut self) {
        self.sibling_index = self.sibling_index.and_then(|i| i.checked_sub(1));
        self.position = self.sibling_index.map(|i| self.siblings[i]);
    }
    pub fn ContainerFragment(&self) -> &'a dyn PhysicalFragment<L> {
        self.items.ContainerFragment()
    }
}

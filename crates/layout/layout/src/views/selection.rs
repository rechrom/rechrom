//! Wave E: selection, caret and editing queries.
//!
//! Commands of this family cross boundary 3 as
//! `LayoutMutation::Selection(SelectionMutation)`; the calls below return
//! data and stay synchronous queries over the same resident engine.

use std::ops::{Deref, DerefMut};

use layout_base::layout_engine::LayoutEngine as CoreLayoutEngine;
use layout_base::layout_selection::SelectionStyleAndPaintInvalidation;

pub use layout_base::layout_selection::{LayoutSelectionNodes, SelectionMutation};

use crate::LayoutEngine;

/// Borrowed selection query view. `LayoutEngine::Selection` lends it shared
/// (read-only queries), `LayoutEngine::SelectionMut` exclusively (queries that
/// also update resident selection state and return its effects).
pub struct SelectionQueries<E> {
    engine: E,
}

impl LayoutEngine {
    #[inline]
    pub fn Selection(&self) -> SelectionQueries<&'_ CoreLayoutEngine> {
        SelectionQueries {
            engine: &self.engine,
        }
    }

    #[inline]
    pub fn SelectionMut(&mut self) -> SelectionQueries<&'_ mut CoreLayoutEngine> {
        SelectionQueries {
            engine: &mut self.engine,
        }
    }
}

impl<E: Deref<Target = CoreLayoutEngine>> SelectionQueries<E> {
    /// Call-scoped identity of the node's ContainingBlockForTextOverflow.
    #[inline]
    pub fn SelectionTextOverflowOwnerIdentity(&self, node_id: u64) -> Option<usize> {
        self.engine.SelectionTextOverflowOwnerIdentity(node_id)
    }
}

impl<E: DerefMut<Target = CoreLayoutEngine>> SelectionQueries<E> {
    #[inline]
    pub fn InvalidateStyleAndPaintForSelection(&mut self) -> SelectionStyleAndPaintInvalidation {
        self.engine.InvalidateStyleAndPaintForSelection()
    }

    #[inline]
    pub fn SetContainsSelectionFocusByIdentity(
        &mut self,
        owner_identity: usize,
        contains_focus: bool,
    ) -> Option<u64> {
        self.engine
            .SetContainsSelectionFocusByIdentity(owner_identity, contains_focus)
    }
}

#![allow(non_snake_case)]

use std::marker::PhantomData;

// The C++ template requires GridTree::SubtreeSize(wtf_size_t). GridLayoutTree
// implements this interface in grid_layout_data.rs.
pub trait GridSubtreeTree {
    fn SubtreeSize(&self, index: u32) -> u32;
}

// cpp: layoutng/internal/grid_subtree.h:50-96
pub struct GridSubtree<GridTree: GridSubtreeTree> {
    pub(crate) subtree_root_: u32,
    parent_end_index_: u32,
    tree_: PhantomData<GridTree>,
}

impl<GridTree: GridSubtreeTree> Default for GridSubtree<GridTree> {
    // cpp: layoutng/internal/grid_subtree.h:56-56
    fn default() -> Self {
        Self {
            subtree_root_: u32::MAX,
            parent_end_index_: u32::MAX,
            tree_: PhantomData,
        }
    }
}

impl<GridTree: GridSubtreeTree> GridSubtree<GridTree> {
    // cpp: layoutng/internal/grid_subtree.h:52-52
    pub fn IsPresent(&self) -> bool {
        self.subtree_root_ != u32::MAX
    }

    // cpp: layoutng/internal/grid_subtree.h:58-61
    pub fn SetSubtreeRoot(&mut self, grid_tree: &GridTree, subtree_root: u32) {
        self.subtree_root_ = subtree_root;
        self.parent_end_index_ = self.NextSiblingIndex(grid_tree);
    }

    // cpp: layoutng/internal/grid_subtree.h:63-67
    pub fn FirstChild(&self, grid_tree: &GridTree) -> Self {
        Self::new_bounded(
            self.NextSiblingIndex(grid_tree),
            self.subtree_root_.wrapping_add(1),
        )
    }

    // cpp: layoutng/internal/grid_subtree.h:69-72
    pub fn NextSibling(&self, grid_tree: &GridTree) -> Self {
        Self::new_bounded(self.parent_end_index_, self.NextSiblingIndex(grid_tree))
    }

    // cpp: layoutng/internal/grid_subtree.h:79-89
    fn new_bounded(parent_end_index: u32, subtree_root: u32) -> Self {
        debug_assert!(subtree_root <= parent_end_index);
        if subtree_root < parent_end_index {
            Self {
                subtree_root_: subtree_root,
                parent_end_index_: parent_end_index,
                tree_: PhantomData,
            }
        } else {
            Self::default()
        }
    }

    // cpp: layoutng/internal/grid_subtree.h:91-96
    fn NextSiblingIndex(&self, grid_tree: &GridTree) -> u32 {
        let subtree_size = grid_tree.SubtreeSize(self.subtree_root_);
        debug_assert!(subtree_size > 0);
        self.subtree_root_.wrapping_add(subtree_size)
    }

    // Access to the source's protected subtree_root_ for algorithm packages.
    pub fn SubtreeRoot(&self) -> u32 {
        self.subtree_root_
    }
}

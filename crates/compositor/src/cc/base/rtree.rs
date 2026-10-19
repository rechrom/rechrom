// Translation status: complete
// Copyright 2015 The Chromium Authors. BSD-style license in LICENSE.
// Source: cc/base/rtree.h:1-47. Includes/guards and the template namespace map
// to this module; node indices replace the source's stable HeapArray pointers.
use crate::adapter::geometry::Rect;
use std::collections::BTreeMap;

// Source: cc/base/rtree.h:48-162.
const K_MIN_CHILDREN: usize = 6;
const K_MAX_CHILDREN: usize = 11;
struct Branch<T: Default> {
    subtree: Option<usize>,
    payload: T,
    bounds: Rect,
}
impl<T: Default> Default for Branch<T> {
    fn default() -> Self {
        Self {
            subtree: None,
            payload: T::default(),
            bounds: Rect::default(),
        }
    }
}
impl<T: Default + Clone> Clone for Branch<T> {
    fn clone(&self) -> Self {
        Self {
            subtree: self.subtree,
            payload: self.payload.clone(),
            bounds: self.bounds,
        }
    }
}
struct Node<T: Default> {
    num_children: u16,
    level: u16,
    children: [Branch<T>; K_MAX_CHILDREN],
}
impl<T: Default> Node<T> {
    fn new(level: u16) -> Self {
        Self {
            num_children: 0,
            level,
            children: std::array::from_fn(|_| Branch::default()),
        }
    }
}
/// Native bulk-loaded RTree. Nodes are allocated at their final count before
/// construction, and internal links are stable indices in that same owner.
/// Payloads are read only on actual leaf branches. SearchRefs lends those resident
/// values and cannot outlive the tree.
pub struct RTree<T: Default> {
    num_data_elements_: usize,
    nodes_: Vec<Node<T>>,
    root_: Branch<T>,
    has_valid_bounds_: bool,
}
// Source: cc/base/rtree.h:164-168.
impl<T: Default> Default for RTree<T> {
    fn default() -> Self {
        Self {
            num_data_elements_: 0,
            nodes_: Vec::new(),
            root_: Branch::default(),
            has_valid_bounds_: true,
        }
    }
}
impl<T: Default> RTree<T> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn has_valid_bounds(&self) -> bool {
        self.has_valid_bounds_
    }
    // Source: cc/base/rtree.h:232-241. Reallocation is forbidden after Build's
    // exact allocation; this index is the source stable Node* identity.
    fn AllocateNodeAtLevel(&mut self, level: u16, nodes_size: &mut usize) -> usize {
        let index = *nodes_size;
        *nodes_size += 1;
        self.nodes_[index] = Node::new(level);
        index
    }
    // Source: cc/base/rtree.h:326-340.
    pub fn Search<'a>(&'a self, query: &Rect, mut result_handler: impl FnMut(&'a T, Rect)) {
        if self.num_data_elements_ == 0 {
            return;
        }
        let root = self.root_.subtree.expect("source CHECK(root_.subtree)");
        if !self.has_valid_bounds_ {
            self.SearchRecursiveFallback(root, query, &mut result_handler);
        } else if query.Intersects(self.root_.bounds) {
            self.SearchRecursive(root, query, &mut result_handler);
        }
    }
    // Source: cc/base/rtree.h:358-365. Returned references point directly into
    // native leaf storage; no copied payload vector is fabricated by traversal.
    pub fn SearchRefs<'a>(&'a self, query: &Rect, results: &mut Vec<&'a T>) {
        results.clear();
        self.Search(query, |payload, _| results.push(payload));
    }
    // Source: cc/base/rtree.h:367-384.
    fn SearchRecursive<'a>(
        &'a self,
        index: usize,
        query: &Rect,
        callback: &mut impl FnMut(&'a T, Rect),
    ) {
        let node = &self.nodes_[index];
        for i in 0..usize::from(node.num_children) {
            let child = &node.children[i];
            if query.Intersects(child.bounds) {
                if node.level == 0 {
                    callback(&child.payload, child.bounds);
                } else {
                    self.SearchRecursive(
                        child.subtree.expect("source CHECK(child.subtree)"),
                        query,
                        callback,
                    );
                }
            }
        }
    }
    // Source: cc/base/rtree.h:386-405. Preserve the fixed source call at line 402:
    // after this fallback level, the native implementation calls SearchRecursive.
    fn SearchRecursiveFallback<'a>(
        &'a self,
        index: usize,
        query: &Rect,
        callback: &mut impl FnMut(&'a T, Rect),
    ) {
        let node = &self.nodes_[index];
        for i in 0..usize::from(node.num_children) {
            let child = &node.children[i];
            if node.level == 0 {
                if query.Intersects(child.bounds) {
                    callback(&child.payload, child.bounds);
                }
            } else {
                self.SearchRecursive(
                    child.subtree.expect("source CHECK(child.subtree)"),
                    query,
                    callback,
                );
            }
        }
    }
    // Source: cc/base/rtree.h:407-413.
    pub fn bounds(&self) -> Option<Rect> {
        if self.has_valid_bounds_ {
            Some(self.root_.bounds)
        } else {
            None
        }
    }
}
impl<T: Default + Clone> RTree<T> {
    // Source: cc/base/rtree.h:178-230.
    pub fn Build(
        &mut self,
        item_count: usize,
        bounds_getter: impl Fn(usize) -> Rect,
        payload_getter: impl Fn(usize) -> T,
    ) {
        debug_assert_eq!(0, self.num_data_elements_);
        let mut branches = Vec::with_capacity(item_count);
        for i in 0..item_count {
            let bounds = bounds_getter(i);
            if bounds.IsEmpty() {
                continue;
            }
            branches.push(Branch {
                subtree: None,
                payload: payload_getter(i),
                bounds,
            });
        }
        self.num_data_elements_ = branches.len();
        let mut nodes_size = 0;
        if self.num_data_elements_ == 1 {
            self.nodes_ = std::iter::repeat_with(|| Node::new(0)).take(1).collect();
            let index = self.AllocateNodeAtLevel(0, &mut nodes_size);
            self.root_.subtree = Some(index);
            self.root_.bounds = branches[0].bounds;
            self.nodes_[index].num_children = 1;
            self.nodes_[index].children[0] = branches.pop().unwrap();
        } else if self.num_data_elements_ > 1 {
            let mut node_count = 0;
            let mut n = self.num_data_elements_;
            while n > 1 {
                n = (n + K_MAX_CHILDREN - 1) / K_MAX_CHILDREN;
                node_count += n;
            }
            self.nodes_ = std::iter::repeat_with(|| Node::new(0))
                .take(node_count)
                .collect();
            self.root_ = self.BuildRecursive(&mut branches, 0, &mut nodes_size);
        }
        assert_eq!(self.nodes_.len(), nodes_size);
    }
    // Source: cc/base/rtree.h:243-324. The native input order is intentionally
    // retained; no spatial sort is introduced. Clone occurs exactly at native
    // Branch copy assignments; the single final branch is moved out.
    fn BuildRecursive(
        &mut self,
        branches: &mut Vec<Branch<T>>,
        level: u16,
        nodes_size: &mut usize,
    ) -> Branch<T> {
        if branches.len() == 1 {
            return branches.pop().unwrap();
        }
        let mut remainder = branches.len() % K_MAX_CHILDREN;
        if remainder > 0 {
            if remainder >= K_MIN_CHILDREN {
                remainder = 0;
            } else {
                remainder = K_MIN_CHILDREN - remainder;
            }
        }
        let mut current_branch = 0;
        let mut new_branch_index = 0;
        while current_branch < branches.len() {
            let mut increment_by = K_MAX_CHILDREN;
            if remainder != 0 {
                if remainder <= K_MAX_CHILDREN - K_MIN_CHILDREN {
                    increment_by -= remainder;
                    remainder = 0;
                } else {
                    increment_by = K_MIN_CHILDREN;
                    remainder -= K_MAX_CHILDREN - K_MIN_CHILDREN;
                }
            }
            let index = self.AllocateNodeAtLevel(level, nodes_size);
            let node = &mut self.nodes_[index];
            node.num_children = 1;
            node.children[0] = branches[current_branch].clone();
            let mut branch = Branch {
                subtree: Some(index),
                payload: T::default(),
                bounds: branches[current_branch].bounds,
            };
            current_branch += 1;
            let (mut x, mut y, mut right, mut bottom) = (
                branch.bounds.x,
                branch.bounds.y,
                branch.bounds.right(),
                branch.bounds.bottom(),
            );
            let mut k = 1;
            while k < increment_by && current_branch < branches.len() {
                let bounds = branches[current_branch].bounds;
                x = x.min(bounds.x);
                y = y.min(bounds.y);
                right = right.max(bounds.right());
                bottom = bottom.max(bounds.bottom());
                node.children[k] = branches[current_branch].clone();
                node.num_children += 1;
                current_branch += 1;
                k += 1;
            }
            branch.bounds = Rect::new(x, y, right.saturating_sub(x), bottom.saturating_sub(y));
            let overflow = branch.bounds.right() != right || branch.bounds.bottom() != bottom;
            self.has_valid_bounds_ &= !overflow;
            debug_assert!(new_branch_index < current_branch);
            branches[new_branch_index] = branch;
            new_branch_index += 1;
        }
        branches.truncate(new_branch_index);
        self.BuildRecursive(branches, level + 1, nodes_size)
    }
    // Source: cc/base/rtree.h:342-356. Output containers are cleared before
    // native traversal, matching source even for an empty tree/query.
    pub fn SearchTo(&self, query: &Rect, results: &mut Vec<T>, mut rects: Option<&mut Vec<Rect>>) {
        results.clear();
        if let Some(rects) = &mut rects {
            rects.clear();
        }
        self.Search(query, |payload, rect| {
            results.push(payload.clone());
            if let Some(rects) = &mut rects {
                rects.push(rect);
            }
        });
    }
}
impl RTree<usize> {
    // Source: cc/base/rtree.h:170-176.
    pub fn BuildRects(&mut self, items: &[Rect]) {
        self.Build(items.len(), |i| items[i], |i| i);
    }
}
impl<T: Default + Clone + Ord> RTree<T> {
    // Source: cc/base/rtree.h:415-438. The recursive map insertion uses source
    // payload ordering, with later duplicate payloads replacing prior values.
    pub fn GetAllBoundsForTracing(&self) -> BTreeMap<T, Rect> {
        let mut result = BTreeMap::new();
        if self.num_data_elements_ > 0 {
            self.GetAllBoundsRecursive(
                self.root_.subtree.expect("source CHECK(root_.subtree)"),
                &mut result,
            );
        }
        result
    }
    fn GetAllBoundsRecursive(&self, index: usize, result: &mut BTreeMap<T, Rect>) {
        let node = &self.nodes_[index];
        for i in 0..usize::from(node.num_children) {
            let child = &node.children[i];
            if node.level == 0 {
                result.insert(child.payload.clone(), child.bounds);
            } else {
                self.GetAllBoundsRecursive(
                    child.subtree.expect("source CHECK(child.subtree)"),
                    result,
                );
            }
        }
    }
}

// Source: cc/base/rtree.h:440-442. Namespace/guard map to the Rust module.

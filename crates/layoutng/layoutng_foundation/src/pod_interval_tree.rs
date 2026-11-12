//! Chromium WTF pod_interval.h:42-171, pod_interval_tree.h:42-260 and
//! pod_red_black_tree.h:95-570. Augmented red-black interval owner.
//! Arena slots replace native arena pointers; ordering compares only Low,
//! equality also compares High and data identity. No sorted-vector index.
#[derive(Clone)]
pub struct PodInterval<T, D> {
    low: T,
    high: T,
    data: D,
    min_low: T,
    max_high: T,
}
impl<T: Copy + PartialOrd, D> PodInterval<T, D> {
    pub fn new(low: T, high: T, data: D) -> Self {
        Self {
            low,
            high,
            data,
            min_low: low,
            max_high: high,
        }
    }
    pub fn Low(&self) -> T {
        self.low
    }
    pub fn High(&self) -> T {
        self.high
    }
    pub fn Data(&self) -> &D {
        &self.data
    }
    pub fn Overlaps(&self, low: T, high: T) -> bool {
        !(self.high < low || high < self.low)
    }
}
impl<T: PartialEq, D: PartialEq> PartialEq for PodInterval<T, D> {
    fn eq(&self, other: &Self) -> bool {
        self.low == other.low && self.high == other.high && self.data == other.data
    }
}
struct Node<T, D> {
    interval: PodInterval<T, D>,
    links: [Option<usize>; 2],
    parent: Option<usize>,
    red: bool,
}
pub struct PodIntervalTree<T, D> {
    nodes: Vec<Option<Node<T, D>>>,
    free: Vec<usize>,
    root: Option<usize>,
}
impl<T, D> Default for PodIntervalTree<T, D> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            free: Vec::new(),
            root: None,
        }
    }
}
impl<T: Copy + PartialOrd, D: Clone + PartialEq> PodIntervalTree<T, D> {
    pub fn CreateInterval(low: T, high: T, data: D) -> PodInterval<T, D> {
        PodInterval::new(low, high, data)
    }
    fn node(&self, i: usize) -> &Node<T, D> {
        self.nodes[i].as_ref().expect("live interval arena slot")
    }
    fn node_mut(&mut self, i: usize) -> &mut Node<T, D> {
        self.nodes[i].as_mut().expect("live interval arena slot")
    }
    fn link(&self, i: usize, side: usize) -> Option<usize> {
        self.node(i).links[side]
    }
    fn parent(&self, i: usize) -> Option<usize> {
        self.node(i).parent
    }
    fn red(&self, i: Option<usize>) -> bool {
        i.is_some_and(|i| self.node(i).red)
    }
    fn set_red(&mut self, i: Option<usize>, red: bool) {
        if let Some(i) = i {
            self.node_mut(i).red = red;
        }
    }
    fn search(&self, node: Option<usize>, value: &PodInterval<T, D>) -> Option<usize> {
        let i = node?;
        let n = self.node(i);
        if value.low < n.interval.low {
            return self.search(n.links[0], value);
        }
        if n.interval.low < value.low {
            return self.search(n.links[1], value);
        }
        if value == &n.interval {
            return Some(i);
        }
        self.search(n.links[0], value)
            .or_else(|| self.search(n.links[1], value))
    }
    pub fn Contains(&self, value: &PodInterval<T, D>) -> bool {
        self.search(self.root, value).is_some()
    }
    fn update(&mut self, i: usize) -> bool {
        let n = self.node(i);
        let mut low = n.interval.low;
        let mut high = n.interval.high;
        if let Some(left) = n.links[0] {
            low = self.node(left).interval.min_low;
            let left_high = self.node(left).interval.max_high;
            if high < left_high {
                high = left_high;
            }
        }
        if let Some(right) = n.links[1] {
            let right_high = self.node(right).interval.max_high;
            if high < right_high {
                high = right_high;
            }
        }
        let changed = !(low == n.interval.min_low && high == n.interval.max_high);
        let n = self.node_mut(i);
        n.interval.min_low = low;
        n.interval.max_high = high;
        changed
    }
    fn propagate(&mut self, mut i: Option<usize>) {
        while let Some(current) = i {
            if !self.update(current) {
                break;
            }
            i = self.parent(current);
        }
    }
    // side=0 is LeftRotate; side=1 is RightRotate. Source's mirror branches
    // have the same parent rewiring and lowest-to-highest augmentation order.
    fn rotate(&mut self, x: usize, side: usize) -> usize {
        let other = 1 - side;
        let y = self.link(x, other).expect("rotation child");
        let middle = self.link(y, side);
        self.node_mut(x).links[other] = middle;
        if let Some(middle) = middle {
            self.node_mut(middle).parent = Some(x);
        }
        let parent = self.parent(x);
        self.node_mut(y).parent = parent;
        if let Some(p) = parent {
            let index = usize::from(self.link(p, 1) == Some(x));
            self.node_mut(p).links[index] = Some(y);
        } else {
            self.root = Some(y);
        }
        self.node_mut(y).links[side] = Some(x);
        self.node_mut(x).parent = Some(y);
        self.update(x);
        self.update(y);
        y
    }
    pub fn Add(&mut self, interval: PodInterval<T, D>) {
        let node = Node {
            interval,
            links: [None, None],
            parent: None,
            red: true,
        };
        let mut x = if let Some(slot) = self.free.pop() {
            self.nodes[slot] = Some(node);
            slot
        } else {
            self.nodes.push(Some(node));
            self.nodes.len() - 1
        };
        let mut cursor = self.root;
        let mut parent = None;
        while let Some(i) = cursor {
            parent = cursor;
            cursor = self.link(
                i,
                usize::from(!(self.node(x).interval.low < self.node(i).interval.low)),
            );
        }
        self.node_mut(x).parent = parent;
        if let Some(p) = parent {
            let side = usize::from(!(self.node(x).interval.low < self.node(p).interval.low));
            self.node_mut(p).links[side] = Some(x);
        } else {
            self.root = Some(x);
        }
        self.update(x);
        let mut update_start = parent;
        while Some(x) != self.root && self.red(self.parent(x)) {
            let p = self.parent(x).expect("red parent");
            let g = self.parent(p).expect("red parent cannot be root");
            let side = usize::from(self.link(g, 1) == Some(p));
            let uncle = self.link(g, 1 - side);
            if self.red(uncle) {
                self.set_red(Some(p), false);
                self.set_red(uncle, false);
                self.set_red(Some(g), true);
                self.update(p);
                x = g;
                self.update(x);
                update_start = self.parent(x);
            } else {
                if Some(x) == self.link(p, 1 - side) {
                    x = p;
                    self.rotate(x, side);
                }
                let p = self.parent(x).unwrap();
                let g = self.parent(p).unwrap();
                self.set_red(Some(p), false);
                self.set_red(Some(g), true);
                let root = self.rotate(g, 1 - side);
                update_start = self.parent(root);
            }
        }
        self.propagate(update_start);
        self.set_red(self.root, false);
    }
    pub fn Remove(&mut self, value: &PodInterval<T, D>) -> bool {
        let Some(z) = self.search(self.root, value) else {
            return false;
        };
        let y = if self.link(z, 0).is_none() || self.link(z, 1).is_none() {
            z
        } else {
            let mut cursor = self.link(z, 1).unwrap();
            while let Some(left) = self.link(cursor, 0) {
                cursor = left;
            }
            cursor
        };
        let x = self.link(y, 0).or_else(|| self.link(y, 1));
        let x_parent = self.parent(y);
        if let Some(x) = x {
            self.node_mut(x).parent = x_parent;
        }
        if let Some(p) = x_parent {
            let side = usize::from(self.link(p, 1) == Some(y));
            self.node_mut(p).links[side] = x;
        } else {
            self.root = x;
        }
        if y != z {
            self.node_mut(z).interval = self.node(y).interval.clone();
            self.update(z);
            self.propagate(self.parent(z));
        }
        if x_parent.is_some_and(|p| p != y && p != z) {
            self.propagate(x_parent);
        }
        if !self.node(y).red {
            self.delete_fixup(x, x_parent);
        }
        self.nodes[y] = None;
        self.free.push(y);
        true
    }
    fn delete_fixup(&mut self, mut x: Option<usize>, mut parent: Option<usize>) {
        while x != self.root && !self.red(x) {
            let p = parent.expect("nonroot double-black parent");
            let side = usize::from(self.link(p, 1) == x);
            let mut w = self.link(p, 1 - side).expect("red-black sibling");
            if self.node(w).red {
                self.set_red(Some(w), false);
                self.set_red(Some(p), true);
                self.rotate(p, side);
                w = self.link(p, 1 - side).unwrap();
            }
            if !self.red(self.link(w, side)) && !self.red(self.link(w, 1 - side)) {
                self.set_red(Some(w), true);
                x = Some(p);
                parent = self.parent(p);
            } else {
                if !self.red(self.link(w, 1 - side)) {
                    self.set_red(self.link(w, side), false);
                    self.set_red(Some(w), true);
                    self.rotate(w, 1 - side);
                    w = self.link(p, 1 - side).unwrap();
                }
                self.set_red(Some(w), self.node(p).red);
                self.set_red(Some(p), false);
                self.set_red(self.link(w, 1 - side), false);
                self.rotate(p, side);
                x = self.root;
                parent = x.and_then(|i| self.parent(i));
            }
        }
        self.set_red(x, false);
    }
    pub fn AllOverlaps(&self, low: T, high: T) -> Vec<PodInterval<T, D>> {
        let mut result = Vec::new();
        self.overlaps(self.root, low, high, &mut result);
        result
    }
    fn overlaps(&self, node: Option<usize>, low: T, high: T, result: &mut Vec<PodInterval<T, D>>) {
        let Some(i) = node else {
            return;
        };
        let n = self.node(i);
        if high < n.interval.min_low || n.interval.max_high < low {
            return;
        }
        self.overlaps(n.links[0], low, high, result);
        if n.interval.Overlaps(low, high) {
            result.push(n.interval.clone());
        }
        self.overlaps(n.links[1], low, high, result);
    }
    pub fn NextIntervalPoint(&self, start: T) -> Option<T> {
        self.next_point(start, self.root)
    }
    fn next_point(&self, start: T, node: Option<usize>) -> Option<T> {
        let n = self.node(node?);
        if n.interval.max_high < start {
            return None;
        }
        if start < n.interval.min_low {
            return Some(n.interval.min_low);
        }
        let left = self.next_point(start, n.links[0]);
        let minimum = |a: T, b: T| if b < a { b } else { a };
        if start < n.interval.low {
            return Some(left.map_or(n.interval.low, |v| minimum(n.interval.low, v)));
        }
        let current = if start < n.interval.high {
            Some(left.map_or(n.interval.high, |v| minimum(n.interval.high, v)))
        } else {
            left
        };
        let right = self.next_point(start, n.links[1]);
        match (current, right) {
            (Some(a), Some(b)) => Some(minimum(a, b)),
            (a, b) => a.or(b),
        }
    }
    pub fn CheckInvariants(&self) -> bool {
        if self.red(self.root) {
            return false;
        }
        self.check(self.root, None).is_some()
    }
    fn check(&self, node: Option<usize>, parent: Option<usize>) -> Option<(usize, Option<(T, T)>)> {
        let Some(i) = node else {
            return Some((1, None));
        };
        let n = self.node(i);
        if n.parent != parent || (n.red && (self.red(n.links[0]) || self.red(n.links[1]))) {
            return None;
        }
        let (lb, left) = self.check(n.links[0], Some(i))?;
        let (rb, right) = self.check(n.links[1], Some(i))?;
        if lb != rb {
            return None;
        }
        let min = left.map_or(n.interval.low, |v| v.0);
        let mut max = n.interval.high;
        for range in [left, right].into_iter().flatten() {
            if max < range.1 {
                max = range.1;
            }
        }
        if min != n.interval.min_low || max != n.interval.max_high {
            return None;
        }
        Some((lb + usize::from(!n.red), Some((min, max))))
    }
}

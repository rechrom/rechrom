// Source: base/containers/flat_tree.h: actual default Pair/GetFirst/std::less/vector specialization.
// Rust Vec maps the native contiguous vector owner; allocation ABI/growth/capacity is adapted, not asserted equal.
pub struct PairFlatTree<K: Ord + Copy, V: Copy> {
    body_: Vec<(K, V)>,
}
impl<K: Ord + Copy, V: Copy> PairFlatTree<K, V> {
    // Native default constructor defaults comparator and container, with no allocation.
    pub fn new() -> Self {
        Self { body_: Vec::new() }
    }
    // Source: flat_tree.h: const lower_bound/equal_range/find inline methods.
    fn lower_bound(&self, key: &K) -> usize {
        self.body_.partition_point(|value| value.0 < *key)
    }
    fn equal_range(&self, key: &K) -> (usize, usize) {
        let lower = self.lower_bound(key);
        if lower == self.body_.len() || *key < self.body_[lower].0 {
            (lower, lower)
        } else {
            (lower, lower + 1)
        }
    }
    pub fn find(&self, key: &K) -> Option<&(K, V)> {
        let range = self.equal_range(key);
        if range.0 == range.1 {
            None
        } else {
            Some(&self.body_[range.0])
        }
    }
    // Source: flat_tree.h: insert(value_type&&) -> emplace_key_args -> unsafe_emplace.
    pub fn insert(&mut self, value: (K, V)) -> (usize, bool) {
        let lower = self.lower_bound(&value.0);
        if lower == self.body_.len() || value.0 < self.body_[lower].0 {
            self.body_.insert(lower, value);
            return (lower, true);
        }
        (lower, false)
    }
}

// Source: base/containers/flat_map.h: internal::GetFirst and actual inherited default flat_tree domain.
// Explicitly restricted to the native std::less key domain expressible by Ord; no native C++ ABI claim.
pub struct GetFirst;
impl GetFirst {
    pub fn get<K, V>(p: &(K, V)) -> &K {
        &p.0
    }
}
pub struct flat_map<K: Ord + Copy, V: Copy> {
    tree: super::flat_tree::PairFlatTree<K, V>,
}
impl<K: Ord + Copy, V: Copy> flat_map<K, V> {
    pub fn new() -> Self {
        Self {
            tree: super::flat_tree::PairFlatTree::new(),
        }
    }
    pub fn find(&self, key: &K) -> Option<&(K, V)> {
        self.tree.find(key)
    }
    pub fn insert(&mut self, value: (K, V)) -> (usize, bool) {
        self.tree.insert(value)
    }
}

// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
//! Persistent hash-trie storage for inherited style data.
//!
//! cpp: third_party/blink/renderer/core/style/style_variables.h HashTrieNode.
//! Copies share the root. A write clones only the nodes between the root and
//! the changed leaf; unrelated subtrees and their key/value owners stay shared.

use std::{
    fmt,
    hash::{Hash, Hasher},
    rc::Rc,
};

const FANOUT_BITS: u32 = 5;
const FANOUT_MASK: u64 = (1 << FANOUT_BITS) - 1;

#[derive(Clone)]
enum Node<V> {
    Leaf {
        hash: u64,
        entries: Box<[(std::string::String, V)]>,
    },
    Branch {
        bitmap: u32,
        children: Box<[Rc<Node<V>>]>,
    },
}

fn KeyHash(key: &str) -> u64 {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hash);
    hash.finish()
}

fn Slot(hash: u64, shift: u32) -> u32 {
    ((hash >> shift) & FANOUT_MASK) as u32
}

fn ChildIndex(bitmap: u32, bit: u32) -> usize {
    (bitmap & bit.wrapping_sub(1)).count_ones() as usize
}

fn Leaf<V>(hash: u64, key: std::string::String, value: V) -> Rc<Node<V>> {
    Rc::new(Node::Leaf {
        hash,
        entries: vec![(key, value)].into_boxed_slice(),
    })
}

fn MergeLeaves<V>(left: Rc<Node<V>>, right: Rc<Node<V>>, shift: u32) -> Rc<Node<V>> {
    let left_hash = match left.as_ref() {
        Node::Leaf { hash, .. } => *hash,
        Node::Branch { .. } => unreachable!("only leaves are merged"),
    };
    let right_hash = match right.as_ref() {
        Node::Leaf { hash, .. } => *hash,
        Node::Branch { .. } => unreachable!("only leaves are merged"),
    };
    let left_slot = Slot(left_hash, shift);
    let right_slot = Slot(right_hash, shift);
    if left_slot == right_slot {
        return Rc::new(Node::Branch {
            bitmap: 1 << left_slot,
            children: vec![MergeLeaves(left, right, shift + FANOUT_BITS)].into_boxed_slice(),
        });
    }
    let bitmap = (1 << left_slot) | (1 << right_slot);
    let children = if left_slot < right_slot {
        vec![left, right]
    } else {
        vec![right, left]
    };
    Rc::new(Node::Branch {
        bitmap,
        children: children.into_boxed_slice(),
    })
}

fn Insert<V: Clone>(
    node: &Rc<Node<V>>,
    hash: u64,
    key: std::string::String,
    value: V,
    shift: u32,
) -> Rc<Node<V>> {
    match node.as_ref() {
        Node::Leaf {
            hash: leaf_hash,
            entries,
        } => {
            if *leaf_hash != hash {
                return MergeLeaves(node.clone(), Leaf(hash, key, value), shift);
            }
            let mut next = entries.to_vec();
            if let Some((_, old)) = next.iter_mut().find(|(name, _)| name == &key) {
                *old = value;
            } else {
                next.push((key, value));
            }
            Rc::new(Node::Leaf {
                hash,
                entries: next.into_boxed_slice(),
            })
        }
        Node::Branch { bitmap, children } => {
            let slot = Slot(hash, shift);
            let bit = 1 << slot;
            let index = ChildIndex(*bitmap, bit);
            let mut next = children.to_vec();
            let next_bitmap;
            if bitmap & bit == 0 {
                next.insert(index, Leaf(hash, key, value));
                next_bitmap = bitmap | bit;
            } else {
                next[index] = Insert(&next[index], hash, key, value, shift + FANOUT_BITS);
                next_bitmap = *bitmap;
            }
            Rc::new(Node::Branch {
                bitmap: next_bitmap,
                children: next.into_boxed_slice(),
            })
        }
    }
}

fn Remove<V: Clone>(node: &Rc<Node<V>>, hash: u64, key: &str, shift: u32) -> Option<Rc<Node<V>>> {
    match node.as_ref() {
        Node::Leaf {
            hash: leaf_hash,
            entries,
        } => {
            if *leaf_hash != hash {
                return Some(node.clone());
            }
            let next = entries
                .iter()
                .filter(|(name, _)| name != key)
                .cloned()
                .collect::<Vec<_>>();
            match next.len() {
                0 => None,
                length if length == entries.len() => Some(node.clone()),
                _ => Some(Rc::new(Node::Leaf {
                    hash,
                    entries: next.into_boxed_slice(),
                })),
            }
        }
        Node::Branch { bitmap, children } => {
            let slot = Slot(hash, shift);
            let bit = 1 << slot;
            if bitmap & bit == 0 {
                return Some(node.clone());
            }
            let index = ChildIndex(*bitmap, bit);
            let mut next = children.to_vec();
            match Remove(&next[index], hash, key, shift + FANOUT_BITS) {
                Some(child) => next[index] = child,
                None => {
                    next.remove(index);
                    if next.is_empty() {
                        return None;
                    }
                    if next.len() == 1 && matches!(next[0].as_ref(), Node::Leaf { .. }) {
                        return Some(next.remove(0));
                    }
                    return Some(Rc::new(Node::Branch {
                        bitmap: bitmap & !bit,
                        children: next.into_boxed_slice(),
                    }));
                }
            }
            Some(Rc::new(Node::Branch {
                bitmap: *bitmap,
                children: next.into_boxed_slice(),
            }))
        }
    }
}

#[derive(Clone)]
pub struct PersistentHashTrieMap<V> {
    root: Option<Rc<Node<V>>>,
    length: usize,
}

impl<V> Default for PersistentHashTrieMap<V> {
    fn default() -> Self {
        Self {
            root: None,
            length: 0,
        }
    }
}

impl<V> PersistentHashTrieMap<V> {
    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        match (&self.root, &other.root) {
            (None, None) => true,
            (Some(left), Some(right)) => Rc::ptr_eq(left, right),
            _ => false,
        }
    }

    /// Compares two immutable maps while preserving the source HashTrieNode
    /// equality fast paths. Equal roots and equal shared subtrees never visit
    /// their leaves; independently built roots are compared in trie order
    /// without hashing every key again.
    ///
    /// cpp: third_party/blink/renderer/core/style/style_variables.h:178-216.
    pub fn equivalent_by(
        &self,
        other: &Self,
        mut values_equal: impl FnMut(&V, &V) -> bool,
    ) -> bool {
        fn NodesEquivalentBy<V>(
            left: &Rc<Node<V>>,
            right: &Rc<Node<V>>,
            values_equal: &mut impl FnMut(&V, &V) -> bool,
        ) -> bool {
            if Rc::ptr_eq(left, right) {
                return true;
            }
            match (left.as_ref(), right.as_ref()) {
                (
                    Node::Leaf {
                        hash: left_hash,
                        entries: left_entries,
                    },
                    Node::Leaf {
                        hash: right_hash,
                        entries: right_entries,
                    },
                ) => {
                    left_hash == right_hash
                        && left_entries.len() == right_entries.len()
                        && left_entries.iter().all(|(left_name, left_value)| {
                            right_entries
                                .iter()
                                .find(|(right_name, _)| right_name == left_name)
                                .is_some_and(|(_, right_value)| {
                                    values_equal(left_value, right_value)
                                })
                        })
                }
                (
                    Node::Branch {
                        bitmap: left_bitmap,
                        children: left_children,
                    },
                    Node::Branch {
                        bitmap: right_bitmap,
                        children: right_children,
                    },
                ) => {
                    left_bitmap == right_bitmap
                        && left_children.len() == right_children.len()
                        && left_children.iter().zip(right_children.iter()).all(
                            |(left_child, right_child)| {
                                NodesEquivalentBy(left_child, right_child, values_equal)
                            },
                        )
                }
                _ => false,
            }
        }

        if self.ptr_eq(other) {
            return true;
        }
        if self.length != other.length {
            return false;
        }
        match (&self.root, &other.root) {
            (None, None) => true,
            (Some(left), Some(right)) => NodesEquivalentBy(left, right, &mut values_equal),
            _ => false,
        }
    }

    pub fn root_id(&self) -> usize {
        self.root
            .as_ref()
            .map_or(0, |root| Rc::as_ptr(root) as *const () as usize)
    }

    pub fn get(&self, key: &str) -> Option<&V> {
        self.get_key_value(key).map(|(_, value)| value)
    }

    pub fn get_key_value(&self, key: &str) -> Option<(&std::string::String, &V)> {
        let hash = KeyHash(key);
        let mut node = self.root.as_deref()?;
        let mut shift = 0;
        loop {
            match node {
                Node::Leaf {
                    hash: leaf_hash,
                    entries,
                } => {
                    return (*leaf_hash == hash)
                        .then(|| entries.iter().find(|(name, _)| name == key))
                        .flatten()
                        .map(|(name, value)| (name, value));
                }
                Node::Branch { bitmap, children } => {
                    let bit = 1 << Slot(hash, shift);
                    if bitmap & bit == 0 {
                        return None;
                    }
                    node = children[ChildIndex(*bitmap, bit)].as_ref();
                    shift += FANOUT_BITS;
                }
            }
        }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    pub fn iter(&self) -> Iter<'_, V> {
        Iter::new(self.root.as_deref())
    }

    pub fn keys(&self) -> Keys<'_, V> {
        Keys(self.iter())
    }

    pub fn values(&self) -> Values<'_, V> {
        Values(self.iter())
    }

    pub fn storage_nodes(&self) -> usize {
        fn Count<V>(node: &Node<V>) -> usize {
            match node {
                Node::Leaf { .. } => 1,
                Node::Branch { children, .. } => {
                    1 + children.iter().map(|child| Count(child)).sum::<usize>()
                }
            }
        }
        self.root.as_deref().map_or(0, Count)
    }
}

impl<V: Clone> PersistentHashTrieMap<V> {
    pub fn insert(&mut self, key: std::string::String, value: V) -> Option<V> {
        let old = self.get(&key).cloned();
        let hash = KeyHash(&key);
        self.root = Some(match &self.root {
            Some(root) => Insert(root, hash, key, value, 0),
            None => Leaf(hash, key, value),
        });
        if old.is_none() {
            self.length += 1;
        }
        old
    }

    pub fn remove(&mut self, key: &str) -> Option<V> {
        let old = self.get(key).cloned()?;
        self.root = Remove(self.root.as_ref().unwrap(), KeyHash(key), key, 0);
        self.length -= 1;
        Some(old)
    }
}

impl<V: fmt::Debug> fmt::Debug for PersistentHashTrieMap<V> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_map().entries(self.iter()).finish()
    }
}

pub struct Iter<'a, V> {
    stack: Vec<(&'a Node<V>, usize)>,
}

impl<'a, V> Iter<'a, V> {
    fn new(root: Option<&'a Node<V>>) -> Self {
        Self {
            stack: root.into_iter().map(|node| (node, 0)).collect(),
        }
    }
}

impl<'a, V> Iterator for Iter<'a, V> {
    type Item = (&'a std::string::String, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (node, index) = self.stack.last_mut()?;
            match node {
                Node::Leaf { entries, .. } => {
                    if *index == entries.len() {
                        self.stack.pop();
                        continue;
                    }
                    let entry = &entries[*index];
                    *index += 1;
                    return Some((&entry.0, &entry.1));
                }
                Node::Branch { children, .. } => {
                    if *index == children.len() {
                        self.stack.pop();
                        continue;
                    }
                    let child = children[*index].as_ref();
                    *index += 1;
                    self.stack.push((child, 0));
                }
            }
        }
    }
}

pub struct Keys<'a, V>(Iter<'a, V>);
impl<'a, V> Iterator for Keys<'a, V> {
    type Item = &'a std::string::String;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(key, _)| key)
    }
}

pub struct Values<'a, V>(Iter<'a, V>);
impl<'a, V> Iterator for Values<'a, V> {
    type Item = &'a V;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, value)| value)
    }
}

impl<V: Clone> FromIterator<(std::string::String, V)> for PersistentHashTrieMap<V> {
    fn from_iter<T: IntoIterator<Item = (std::string::String, V)>>(iter: T) -> Self {
        let mut result = Self::default();
        for (key, value) in iter {
            result.insert(key, value);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn updates_share_untouched_paths_and_preserve_snapshots() {
        let mut parent = PersistentHashTrieMap::default();
        for index in 0..4096 {
            parent.insert(format!("--v{index}"), index);
        }
        let snapshot = parent.clone();
        assert!(parent.ptr_eq(&snapshot));
        let untouched = snapshot.get_key_value("--v2048").unwrap().0.as_ptr();
        parent.insert("--v7".into(), 9000);
        assert_eq!(snapshot.get("--v7"), Some(&7));
        assert_eq!(parent.get("--v7"), Some(&9000));
        assert_eq!(
            parent.get_key_value("--v2048").unwrap().0.as_ptr(),
            untouched
        );
        assert!(!parent.ptr_eq(&snapshot));
        assert_eq!(parent.len(), snapshot.len());
    }

    #[test]
    fn removal_path_copies_without_mutating_shared_root() {
        let mut parent = PersistentHashTrieMap::default();
        for index in 0..256 {
            parent.insert(format!("--v{index}"), index);
        }
        let mut child = parent.clone();
        assert_eq!(child.remove("--v31"), Some(31));
        assert_eq!(parent.get("--v31"), Some(&31));
        assert_eq!(child.get("--v31"), None);
        assert_eq!(child.len(), parent.len() - 1);
    }

    #[test]
    fn equality_short_circuits_shared_subtrees_and_accepts_rebuilt_roots() {
        let mut original = PersistentHashTrieMap::default();
        for index in 0..4096 {
            original.insert(format!("--v{index}"), index);
        }

        let shared = original.clone();
        let mut comparisons = 0;
        assert!(original.equivalent_by(&shared, |left, right| {
            comparisons += 1;
            left == right
        }));
        assert_eq!(comparisons, 0, "identical roots require no leaf work");

        let mut one_change = original.clone();
        one_change.insert("--v7".into(), 9000);
        comparisons = 0;
        assert!(!original.equivalent_by(&one_change, |left, right| {
            comparisons += 1;
            left == right
        }));
        assert_eq!(
            comparisons, 1,
            "unchanged shared branches must remain pointer fast paths"
        );

        let rebuilt = (0..4096)
            .rev()
            .map(|index| (format!("--v{index}"), index))
            .collect::<PersistentHashTrieMap<_>>();
        comparisons = 0;
        assert!(original.equivalent_by(&rebuilt, |left, right| {
            comparisons += 1;
            left == right
        }));
        assert_eq!(comparisons, original.len());
    }

    #[test]
    fn randomized_operations_match_hash_map_and_preserve_snapshots() {
        let mut trie = PersistentHashTrieMap::default();
        let mut model = HashMap::new();
        let mut snapshots = Vec::new();
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        for step in 0..20_000 {
            // Deterministic xorshift64 keeps this test independent of a rand
            // dependency while exercising updates and removals across many
            // unrelated HAMT paths.
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let key = format!("--v{}", state as usize % 1024);
            if state & 3 == 0 {
                assert_eq!(trie.remove(&key), model.remove(&key));
            } else {
                let value = (state >> 32) as i32;
                assert_eq!(trie.insert(key.clone(), value), model.insert(key, value));
            }
            assert_eq!(trie.len(), model.len());
            assert_eq!(trie.is_empty(), model.is_empty());
            if step % 257 == 0 {
                snapshots.push((trie.clone(), model.clone()));
            }
        }
        let materialized = trie
            .iter()
            .map(|(key, value)| (key.clone(), *value))
            .collect::<HashMap<_, _>>();
        assert_eq!(materialized, model);
        for (snapshot, expected) in snapshots {
            assert_eq!(snapshot.len(), expected.len());
            for (key, value) in expected {
                assert_eq!(snapshot.get(&key), Some(&value));
            }
        }
    }
}

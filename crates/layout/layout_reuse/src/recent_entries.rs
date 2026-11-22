/// Small insertion-ordered cache used by LayoutNG reuse paths.
///
/// A hit is promoted to the newest slot. Inserting at capacity evicts the
/// oldest slot, matching Chromium's small-vector layout caches.
pub struct RecentEntries<T, const CAPACITY: usize> {
    entries: Vec<T>,
}

impl<T, const CAPACITY: usize> Default for RecentEntries<T, CAPACITY> {
    fn default() -> Self {
        assert!(CAPACITY > 0, "reuse cache capacity must be non-zero");
        Self {
            entries: Vec::new(),
        }
    }
}

impl<T, const CAPACITY: usize> RecentEntries<T, CAPACITY> {
    pub fn touch_where(&mut self, mut predicate: impl FnMut(&T) -> bool) -> Option<&T> {
        let index = self.entries.iter().rposition(&mut predicate)?;
        if index != self.entries.len() - 1 {
            let entry = self.entries.remove(index);
            self.entries.push(entry);
        }
        self.entries.last()
    }

    pub fn push(&mut self, entry: T) {
        if self.entries.len() == CAPACITY {
            self.entries.remove(0);
        }
        self.entries.push(entry);
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.entries.iter()
    }

    pub fn storage(&self) -> &Vec<T> {
        &self.entries
    }

    pub fn last(&self) -> Option<&T> {
        self.entries.last()
    }
}

#[cfg(test)]
mod tests {
    use super::RecentEntries;

    #[test]
    fn promotes_hits_and_evicts_the_oldest_entry() {
        let mut entries = RecentEntries::<u8, 3>::default();
        entries.push(1);
        entries.push(2);
        entries.push(3);

        assert_eq!(entries.touch_where(|entry| *entry == 1), Some(&1));
        assert_eq!(entries.iter().copied().collect::<Vec<_>>(), vec![2, 3, 1]);

        entries.push(4);
        assert_eq!(entries.iter().copied().collect::<Vec<_>>(), vec![3, 1, 4]);
        assert_eq!(entries.touch_where(|entry| *entry == 2), None);
    }
}

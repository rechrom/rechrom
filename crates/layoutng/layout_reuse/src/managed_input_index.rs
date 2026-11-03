use super::*;

impl ManagedInputIndex {
    #[inline(always)]
    pub(super) fn Remove(&mut self, key: &InputKey, record: &ManagedObject) {
        remove_managed_input(self, key, record)
    }

    #[inline(always)]
    pub(super) fn Insert(&mut self, key: InputKey, record: &ManagedObject) {
        insert_managed_input(self, key, record)
    }
}

#[inline(never)]
fn remove_managed_input(index: &mut ManagedInputIndex, key: &InputKey, record: &ManagedObject) {
    if let Some(keys) = index.by_id.get_mut(&record.input.id) {
        keys.retain(|candidate| candidate != key);
        if keys.is_empty() {
            index.by_id.remove(&record.input.id);
        }
    }
    if let Some(keys) = index.children.get_mut(&record.parent) {
        keys.retain(|candidate| candidate != key);
        if keys.is_empty() {
            index.children.remove(&record.parent);
        }
    }
}

#[inline(never)]
fn insert_managed_input(index: &mut ManagedInputIndex, key: InputKey, record: &ManagedObject) {
    index
        .by_id
        .entry(record.input.id)
        .or_default()
        .push(key.clone());
    index.children.entry(record.parent).or_default().push(key);
}

#[inline(never)]
pub(super) fn BuildManagedInputIndex(storage: &LayoutObjectTreeStorage) -> &ManagedInputIndex {
    storage.input_index.get_or_init(|| {
        let mut index = ManagedInputIndex::default();
        for (key, record) in &storage.managed {
            index
                .by_id
                .entry(record.input.id)
                .or_default()
                .push(key.clone());
            index
                .children
                .entry(record.parent)
                .or_default()
                .push(key.clone());
        }
        index
    })
}

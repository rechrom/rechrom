//! Native LayoutObject-tree update transaction.
//!
//! This source is included as a child of `layout_object_builder` at the
//! reciprocal assembly boundary. The standalone invalidation package remains
//! free of native LayoutObject types, while subtree preparation, retirement,
//! and failure cleanup retain an invalidation-qualified sampling path.

use super::*;

#[inline(never)]
pub(super) fn reattach(tree: &mut LayoutObjectTree, node_ids: &[u64]) {
    let storage = unsafe { &mut *tree.storage_ };
    assert!(storage.updating, "reattachment belongs to the tree update");
    storage.reattach.extend(node_ids.iter().copied());
}

#[inline(never)]
pub(super) fn place(
    storage: &mut LayoutObjectTreeStorage,
    parent: *mut LayoutObject,
    object: *mut LayoutObject,
    node: *mut Node,
) {
    if parent.is_null() {
        assert!(
            storage.root.Get().is_null() || storage.root.Get() == object,
            "A LayoutObjectTree has exactly one root"
        );
        storage.root.Assign(object);
        return;
    }
    let parent_node = unsafe { &*parent }.GetNode().cast::<ContainerNode>();
    let previous = storage
        .last_child
        .get(&parent)
        .copied()
        .unwrap_or(std::ptr::null_mut());
    let linked = unsafe { &*node };
    if linked.parentNode() != parent_node
        || linked.previousSibling() != previous
        || unsafe { &*object }.Parent().is_null()
    {
        storage.stats.moved += 1;
        let _factory_scope = LayoutObjectFactoryScope::new(&storage.factories);
        let mut before = if previous.is_null() {
            unsafe { &*parent_node }.firstChild()
        } else {
            unsafe { &*previous }.nextSibling()
        };
        if before == node {
            before = unsafe { &*node }.nextSibling();
        }
        let mut before_object = std::ptr::null_mut();
        let mut candidate = before;
        while !candidate.is_null() {
            let next = unsafe { &*candidate }.GetLayoutObject();
            if !next.is_null() && !unsafe { &*next }.WasDestroyedForTreeUpdate() {
                before_object = next;
                break;
            }
            candidate = unsafe { &*candidate }.nextSibling();
        }
        if !unsafe { &*object }.Parent().is_null() {
            unsafe { &mut *object }.Remove();
        }
        unsafe { &mut *parent_node }.InsertForTreeUpdate(unsafe { &mut *node }, before);
        unsafe { &mut *parent }.AddChild(object, before_object);
    }
    NativeNodeMetadataRelations::AttachIncrementalRelations(unsafe { &mut *node })
        .unwrap_or_else(|error| panic!("{error:?}"));
    storage.last_child.insert(parent, node);
}

#[inline(never)]
pub(super) fn update_subtree(
    tree: &mut LayoutObjectTree,
    id: u64,
    parent_id: u64,
    build: impl FnOnce(&mut LayoutObjectTree, &mut LayoutObject),
) -> bool {
    let storage = unsafe { &mut *tree.storage_ };
    assert!(!storage.updating);
    let keys = storage.InputKeys(id).to_vec();
    if keys.len() != 1 {
        return false;
    }
    let record = &storage.managed[&keys[0]];
    let parent = record.parent;
    if parent.is_null()
        || record.input.style_generated
        || unsafe { &*record.object }.WasDestroyedForTreeUpdate()
        || unsafe { &*record.object }.Parent() != parent
        || unsafe { &*parent }.IsAnonymous()
        || unsafe { &*(*parent).GetNode() }.InputId() != parent_id
    {
        return false;
    }
    let previous = unsafe { &*record.node }.previousSibling();
    let mut collect_trace =
        layout_invalidation::trace_span(layout_invalidation::TracePhase::CollectSubtree);
    let mut affected = keys;
    let mut position = 0;
    while position < affected.len() {
        let object = storage.managed[&affected[position]].object;
        affected.extend_from_slice(storage.ChildKeys(object));
        position += 1;
    }
    collect_trace.set("objects", affected.len() as f64);
    drop(collect_trace);
    let stable_nodes = affected
        .iter()
        .map(|key| storage.managed[key].node)
        .collect::<Vec<_>>();
    let parent_node = unsafe { &*parent }.GetNode();
    let mut index = storage.input_index.take().unwrap();
    assert!(storage.pending.is_empty());
    storage.updating = true;
    storage.occurrences.clear();
    storage.last_child.clear();
    storage.last_child.insert(parent, previous);
    unsafe { &mut *(*parent).GetNode() }.UnsealForTreeUpdate();
    storage.attached_keys.clear();
    storage.stats = LayoutTreeUpdateStats::default();
    let prepare_trace =
        layout_invalidation::trace_span(layout_invalidation::TracePhase::PrepareSubtree);
    for key in affected {
        let record = storage.managed.remove(&key).unwrap();
        index.Remove(&key, &record);
        let node = unsafe { &mut *record.node };
        node.UnsealForTreeUpdate();
        NativeNodeMetadataRelations::ResetForTreeUpdate(node);
        storage.pending.insert(key, record);
    }
    drop(prepare_trace);
    storage.partial_index = Some(index);
    complete_update(tree, |tree| build(tree, unsafe { &mut *parent }));
    let stats = tree.UpdateStats();
    if stats.created == 0 && stats.updated == 0 && stats.removed == 0 && stats.moved == 0 {
        // Reused descendants reattach their local control relations during the
        // partial transaction. When every input is unchanged, those relations
        // are complete and the document relations can be resealed locally.
        if !parent_node.is_null() {
            unsafe { &mut *parent_node }.seal_input_tree();
        }
        for node in stable_nodes {
            unsafe { &mut *node }.seal_input_tree();
        }
    }
    true
}

#[inline(never)]
pub(super) fn complete_update<T>(
    tree: &mut LayoutObjectTree,
    build: impl FnOnce(&mut LayoutObjectTree) -> T,
) -> T {
    let storage = unsafe { &mut *tree.storage_ };
    let fonts: *mut dyn NativeFontResolver = &mut *storage.fonts;
    let factories = storage.factories;
    let _heap = LayoutHeapScope::new();
    // One RebuildLayoutTree transaction owns the construction environment.
    let _font_scope = unsafe { NativeFontResolverScope::new_from_raw(fonts) };
    let _factory_scope = LayoutObjectFactoryScope::new(&factories);
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| build(tree))) {
        Ok(value) => {
            finish_update(tree);
            value
        }
        Err(error) => {
            let _heap = LayoutHeapScope::new();
            let storage = unsafe { &mut *tree.storage_ };
            let _factories = LayoutObjectFactoryScope::new(&storage.factories);
            for object in storage.objects.values() {
                let object = unsafe { &mut *object.Get() };
                if !object.WasDestroyedForTreeUpdate() {
                    object.Destroy();
                }
            }
            for node in storage.metadata.values() {
                unsafe { &mut *node.Get() }.DetachForTreeUpdate();
            }
            storage.root = Persistent::from_ptr(std::ptr::null_mut());
            storage.objects.clear();
            storage.metadata.clear();
            storage.retired.clear();
            storage.managed.clear();
            storage.pending.clear();
            storage.partial_index = None;
            storage.attached_keys.clear();
            storage.input_index.take();
            storage.reattach.clear();
            storage.last_child.clear();
            storage.updating = false;
            std::panic::resume_unwind(error);
        }
    }
}

#[inline(never)]
pub(super) fn begin_update(tree: &mut LayoutObjectTree) {
    let storage = unsafe { &mut *tree.storage_ };
    assert!(!storage.updating, "layout tree update already active");
    storage.updating = true;
    debug_assert!(storage.retired.is_empty());
    storage.partial_index = None;
    storage.attached_keys.clear();
    storage.input_index.take();
    // Reuse both maps' buckets across complete projections.
    debug_assert!(storage.pending.is_empty());
    std::mem::swap(&mut storage.pending, &mut storage.managed);
    storage.root = Persistent::from_ptr(std::ptr::null_mut());
    storage.occurrences.clear();
    storage.last_child.clear();
    storage.stats = LayoutTreeUpdateStats::default();
    for node in storage.metadata.values() {
        let node = unsafe { &mut *node.Get() };
        node.UnsealForTreeUpdate();
        NativeNodeMetadataRelations::ResetForTreeUpdate(node);
    }
}

#[inline(never)]
pub(super) fn finish_update(tree: &mut LayoutObjectTree) {
    let _heap_scope = LayoutHeapScope::new();
    let storage = unsafe { &mut *tree.storage_ };
    assert!(storage.updating, "layout tree update is not active");
    let _factory_scope = LayoutObjectFactoryScope::new(&storage.factories);
    let mut retire_trace =
        layout_invalidation::trace_span(layout_invalidation::TracePhase::RetireObjects);
    retire_trace.set("objects", storage.pending.len() as f64);
    for old in storage.pending.values() {
        if !unsafe { &*old.object }.WasDestroyedForTreeUpdate() {
            unsafe { &mut *old.object }.Destroy();
        }
        unsafe { &mut *old.node }.DetachForTreeUpdate();
        storage.stats.removed += 1;
        storage.retired.push((old.object, old.node));
    }
    storage.pending.clear();
    // Destroy every removed object before releasing any roots, preserving
    // transaction lifetime and parent dependencies.
    for (object, node) in storage.retired.drain(..) {
        storage.objects.remove(&object);
        storage.metadata.remove(&node);
    }
    drop(retire_trace);
    if let Some(mut index) = storage.partial_index.take() {
        for key in storage.attached_keys.drain(..) {
            index.Insert(key.clone(), &storage.managed[&key]);
        }
        assert!(storage.input_index.set(index).is_ok());
    }
    storage.reattach.clear();
    storage.updating = false;
}

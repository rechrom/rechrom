//! Host ownership half of traced wrappers. It consumes typed host facts;
//! no DOM, host borrows, JS execution or source callbacks occur during GC.
use super::*;
use std::collections::{HashMap, HashSet};
pub(super) const WRAPPER_SLOT: u64 = u64::MAX;
pub(super) struct HostCell {
    weak: WeakObject,
    generation: u64,
}
#[derive(Default)]
pub(super) struct HostGraph {
    pub(super) source: Option<Rc<dyn HostObjectTraceSource>>,
    declared: HashSet<HostObjectId>,
    edges: HashMap<HostObjectId, HashMap<u64, HostObjectId>>,
    lists: HashMap<HostObjectId, Rc<[HostObjectId]>>,
    ephemerons: HashMap<HostObjectId, HashMap<u64, (HostObjectId, HostObjectId)>>,
    cells: HashMap<HostObjectId, HostCell>,
    roots: HashMap<HostObjectId, HashSet<u64>>,
    rooted_cells: HashMap<HostObjectId, JsObject>,
    released: Rc<RefCell<Vec<(HostObjectId, u64)>>>,
    next_generation: u64,
    pending: Vec<HostObjectTraceChange>,
}
impl HostGraph {
    pub(super) fn new(source: Option<Rc<dyn HostObjectTraceSource>>) -> Self {
        Self {
            source,
            ..Default::default()
        }
    }
    pub(super) fn Manages(&self, id: HostObjectId) -> bool {
        self.declared.contains(&id)
    }
    pub(super) fn TakeReleased(&mut self) -> Vec<HostObjectId> {
        let released = std::mem::take(&mut *self.released.borrow_mut());
        let mut ids = Vec::new();
        for (id, generation) in released {
            if self
                .cells
                .get(&id)
                .is_some_and(|cell| cell.generation == generation)
            {
                self.cells.remove(&id);
                ids.push(id);
            }
        }
        ids
    }
    fn EnsureCell(
        &mut self,
        ctx: &mut Context,
        id: HostObjectId,
        work: &mut Vec<(HostObjectId, JsObject)>,
        keep: &mut Vec<JsObject>,
    ) -> JsResult<JsObject> {
        if let Some(object) = self.cells.get(&id).and_then(|cell| cell.weak.upgrade()) {
            return Ok(object);
        }
        self.next_generation += 1;
        let generation = self.next_generation;
        let released = self.released.clone();
        let object = ctx.new_traced_object(Some(Box::new(move || {
            released.borrow_mut().push((id, generation))
        })))?;
        self.cells.insert(
            id,
            HostCell {
                weak: WeakObject::new(&object),
                generation,
            },
        );
        keep.push(object.clone());
        work.push((id, object.clone()));
        Ok(object)
    }
    fn Populate(
        &mut self,
        ctx: &mut Context,
        work: &mut Vec<(HostObjectId, JsObject)>,
        keep: &mut Vec<JsObject>,
    ) -> JsResult<()> {
        // Iterative host graph walk also handles deep trees and cycles.
        while let Some((id, object)) = work.pop() {
            let edges = self
                .edges
                .get(&id)
                .map(|edges| {
                    edges
                        .iter()
                        .map(|(&slot, &target)| (slot, target))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            for (slot, target) in edges {
                let target = self.EnsureCell(ctx, target, work, keep)?;
                ctx.set_traced_reference(&object, slot, Some(&target))?;
            }
            if let Some(targets) = self.lists.get(&id).cloned() {
                let values = targets
                    .iter()
                    .map(|&id| self.EnsureCell(ctx, id, work, keep).map(Value::Object))
                    .collect::<JsResult<Vec<_>>>()?;
                ctx.set_traced_references(&object, &values)?;
            }
            let ephemerons: Vec<_> = self
                .ephemerons
                .get(&id)
                .map(|entries| {
                    entries
                        .iter()
                        .map(|(&slot, &entry)| (slot, entry))
                        .collect()
                })
                .unwrap_or_default();
            for (slot, (key, target)) in ephemerons {
                let key = self.EnsureCell(ctx, key, work, keep)?;
                let target = self.EnsureCell(ctx, target, work, keep)?;
                let map = ctx.new_ephemeron(&key, &target)?;
                ctx.set_traced_ephemeron(&object, slot, Some(&map))?;
            }
        }
        Ok(())
    }
    pub(super) fn Cell(
        &mut self,
        ctx: &mut Context,
        id: HostObjectId,
        keep: &mut Vec<JsObject>,
    ) -> JsResult<JsObject> {
        let mut work = Vec::new();
        let object = self.EnsureCell(ctx, id, &mut work, keep)?;
        self.Populate(ctx, &mut work, keep)?;
        Ok(object)
    }
    pub(super) fn Apply(
        &mut self,
        ctx: &mut Context,
        changes: Vec<HostObjectTraceChange>,
    ) -> JsResult<Vec<JsObject>> {
        self.pending.extend(changes);
        if self.pending.is_empty() {
            return Ok(Vec::new());
        }
        let mut trace = browser_tracing::span("javascript", "HostObjectTrace");
        trace.set("changes", self.pending.len() as f64);
        let generation_before = self.next_generation;
        let mut changed = HashMap::<HostObjectId, HashSet<u64>>::new();
        let mut root_changes = HashSet::new();
        let mut list_changes = HashSet::new();
        let mut function_changes =
            HashMap::<(HostObjectId, u64), Option<JavaScriptFunction>>::new();
        let mut value_changes = HashMap::<(HostObjectId, u64), Option<JavaScriptValue>>::new();
        let mut ephemeron_changes = HashMap::new();
        for change in self.pending.iter().cloned() {
            match change {
                HostObjectTraceChange::Object { id } => {
                    self.declared.insert(id);
                }
                HostObjectTraceChange::ReferenceList { owner, targets } => {
                    self.declared.insert(owner);
                    self.declared.extend(targets.iter().copied());
                    self.lists.insert(owner, targets);
                    list_changes.insert(owner);
                }
                HostObjectTraceChange::Retire { id } => {
                    self.declared.remove(&id);
                    self.edges.remove(&id);
                    self.lists.remove(&id);
                    self.ephemerons.remove(&id);
                    self.roots.remove(&id);
                    self.rooted_cells.remove(&id);
                    self.cells.remove(&id);
                }
                HostObjectTraceChange::Reference {
                    owner,
                    slot,
                    target,
                } => {
                    // A native MemberScriptValue can change representation
                    // (host wrapper, VM object, function or primitive). Preserve
                    // source write order when several updates share a slot.
                    function_changes.remove(&(owner, slot));
                    value_changes.remove(&(owner, slot));
                    assert_ne!(
                        slot, WRAPPER_SLOT,
                        "wrapper reference slot is backend-owned"
                    );
                    self.declared.insert(owner);
                    if let Some(target) = target {
                        self.declared.insert(target);
                        self.edges.entry(owner).or_default().insert(slot, target);
                    } else if let Some(edges) = self.edges.get_mut(&owner) {
                        edges.remove(&slot);
                    }
                    changed.entry(owner).or_default().insert(slot);
                }
                HostObjectTraceChange::ClearReferenceRange {
                    owner,
                    first_slot,
                    end_slot,
                } => {
                    assert!(first_slot <= end_slot && end_slot != u64::MAX);
                    if let Some(edges) = self.edges.get_mut(&owner) {
                        let slots: Vec<_> = edges
                            .keys()
                            .copied()
                            .filter(|slot| *slot >= first_slot && *slot < end_slot)
                            .collect();
                        for slot in slots {
                            edges.remove(&slot);
                            changed.entry(owner).or_default().insert(slot);
                        }
                    }
                    function_changes.retain(|&(id, slot), _| {
                        id != owner || slot < first_slot || slot >= end_slot
                    });
                    value_changes.retain(|&(id, slot), _| {
                        id != owner || slot < first_slot || slot >= end_slot
                    });
                }
                HostObjectTraceChange::FunctionReference {
                    owner,
                    slot,
                    function,
                } => {
                    if let Some(edges) = self.edges.get_mut(&owner) {
                        edges.remove(&slot);
                    }
                    if let Some(slots) = changed.get_mut(&owner) {
                        slots.remove(&slot);
                    }
                    value_changes.remove(&(owner, slot));
                    assert_ne!(
                        slot, WRAPPER_SLOT,
                        "wrapper reference slot is backend-owned"
                    );
                    self.declared.insert(owner);
                    function_changes.insert((owner, slot), function);
                }
                HostObjectTraceChange::ValueReference { owner, slot, value } => {
                    if let Some(edges) = self.edges.get_mut(&owner) {
                        edges.remove(&slot);
                    }
                    if let Some(slots) = changed.get_mut(&owner) {
                        slots.remove(&slot);
                    }
                    function_changes.remove(&(owner, slot));
                    assert_ne!(
                        slot, WRAPPER_SLOT,
                        "wrapper reference slot is backend-owned"
                    );
                    self.declared.insert(owner);
                    value_changes.insert((owner, slot), value);
                }
                HostObjectTraceChange::Ephemeron { owner, slot, entry } => {
                    self.declared.insert(owner);
                    if let Some((key, target)) = entry {
                        self.declared.insert(key);
                        self.declared.insert(target);
                        self.ephemerons
                            .entry(owner)
                            .or_default()
                            .insert(slot, (key, target));
                    } else if let Some(entries) = self.ephemerons.get_mut(&owner) {
                        entries.remove(&slot);
                    }
                    ephemeron_changes.insert((owner, slot), entry);
                }
                HostObjectTraceChange::Root { id, reason, rooted } => {
                    self.declared.insert(id);
                    if rooted {
                        self.roots.entry(id).or_default().insert(reason);
                    } else if let Some(roots) = self.roots.get_mut(&id) {
                        roots.remove(&reason);
                    }
                    root_changes.insert(id);
                }
            }
        }
        let mut keep = Vec::new();
        let mut work = Vec::new();
        // Root the new graph before dropping any old roots/edges.
        for id in root_changes
            .iter()
            .copied()
            .filter(|id| self.roots.get(id).is_some_and(|roots| !roots.is_empty()))
            .collect::<Vec<_>>()
        {
            let object = self.EnsureCell(ctx, id, &mut work, &mut keep)?;
            self.rooted_cells.insert(id, object);
        }
        let mut prepared = Vec::new();
        for (id, slots) in changed {
            let Some(object) = self.cells.get(&id).and_then(|cell| cell.weak.upgrade()) else {
                continue;
            };
            keep.push(object.clone());
            for slot in slots {
                let target = self
                    .edges
                    .get(&id)
                    .and_then(|edges| edges.get(&slot))
                    .copied();
                let target = target
                    .map(|id| self.EnsureCell(ctx, id, &mut work, &mut keep))
                    .transpose()?;
                prepared.push((object.clone(), slot, target));
            }
        }
        let mut prepared_lists = Vec::new();
        for id in list_changes {
            let Some(object) = self.cells.get(&id).and_then(|cell| cell.weak.upgrade()) else {
                continue;
            };
            let Some(targets) = self.lists.get(&id).cloned() else {
                continue;
            };
            let values = targets
                .iter()
                .map(|&id| {
                    self.EnsureCell(ctx, id, &mut work, &mut keep)
                        .map(Value::Object)
                })
                .collect::<JsResult<Vec<_>>>()?;
            prepared_lists.push((object, values));
        }
        self.Populate(ctx, &mut work, &mut keep)?;
        for ((id, slot), function) in function_changes {
            let object = if function.is_some() {
                self.EnsureCell(ctx, id, &mut work, &mut keep)?
            } else if let Some(object) = self.cells.get(&id).and_then(|cell| cell.weak.upgrade()) {
                object
            } else {
                continue;
            };
            let target = function
                .as_ref()
                .map(|function| {
                    function
                        .Implementation::<JsObject>()
                        .cloned()
                        .ok_or_else(|| ctx.type_error("Function belongs to another engine"))
                })
                .transpose()?;
            if let Some(target) = target.as_ref() {
                ctx.validate_value(&Value::Object(target.clone()))?;
            }
            prepared.push((object, slot, target));
        }
        self.Populate(ctx, &mut work, &mut keep)?;
        let mut prepared_values = Vec::new();
        for ((id, slot), value) in value_changes {
            let object = if value.is_some() {
                self.EnsureCell(ctx, id, &mut work, &mut keep)?
            } else if let Some(object) = self.cells.get(&id).and_then(|cell| cell.weak.upgrade()) {
                object
            } else {
                continue;
            };
            let target = value
                .as_ref()
                .map(|value| {
                    value
                        .Implementation::<Value>()
                        .cloned()
                        .ok_or_else(|| ctx.type_error("Value belongs to another engine"))
                })
                .transpose()?;
            if let Some(target) = &target {
                ctx.validate_value(target)?;
            }
            prepared_values.push((object, slot, target));
        }
        self.Populate(ctx, &mut work, &mut keep)?;
        let mut prepared_ephemerons = Vec::new();
        for ((id, slot), entry) in ephemeron_changes {
            let object = if entry.is_some() {
                self.EnsureCell(ctx, id, &mut work, &mut keep)?
            } else if let Some(object) = self.cells.get(&id).and_then(|cell| cell.weak.upgrade()) {
                object
            } else {
                continue;
            };
            if self
                .cells
                .get(&id)
                .is_some_and(|cell| cell.generation > generation_before)
            {
                // Populate initializes all fields of a newly allocated cell.
                // Do not allocate the same private WeakMap a second time.
                continue;
            }
            let map = if let Some((key, target)) = entry {
                let key = self.EnsureCell(ctx, key, &mut work, &mut keep)?;
                let target = self.EnsureCell(ctx, target, &mut work, &mut keep)?;
                Some(ctx.new_ephemeron(&key, &target)?)
            } else {
                None
            };
            prepared_ephemerons.push((object, slot, map));
        }
        self.Populate(ctx, &mut work, &mut keep)?;
        // All allocations and new references are ready while old physical
        // edges/roots still protect their objects. Commit performs no JS
        // allocation; removals cannot expose a half-published host graph.
        for (object, slot, target) in prepared {
            if let Some(previous) = ctx.set_traced_reference(&object, slot, target.as_ref())? {
                keep.push(previous);
            }
        }
        for (object, slot, target) in prepared_values {
            if let Some(Value::Object(previous)) =
                ctx.set_traced_value_reference(&object, slot, target.as_ref())?
            {
                keep.push(previous);
            }
        }
        for (object, slot, map) in prepared_ephemerons {
            if let Some(previous) = ctx.set_traced_ephemeron(&object, slot, map.as_ref())? {
                keep.push(previous);
            }
        }
        for (object, values) in prepared_lists {
            for value in ctx.set_traced_references(&object, &values)? {
                if let Value::Object(object) = value {
                    keep.push(object);
                }
            }
        }
        for id in root_changes {
            if self.roots.get(&id).is_none_or(|roots| roots.is_empty()) {
                if let Some(previous) = self.rooted_cells.remove(&id) {
                    keep.push(previous);
                }
            }
        }
        self.pending.clear();
        trace.set(
            "created_cells",
            (self.next_generation - generation_before) as f64,
        );
        trace.set("live_cells", self.cells.len() as f64);
        Ok(keep)
    }
    pub(super) fn Clear(&mut self) {
        self.rooted_cells.clear();
        self.cells.clear();
        self.edges.clear();
        self.lists.clear();
        self.ephemerons.clear();
        self.roots.clear();
        self.declared.clear();
        self.pending.clear();
        self.source = None;
    }
}

pub(super) fn SynchronizeHostGraph(
    ctx: &mut Context,
    state: &Rc<RefCell<State>>,
) -> JsResult<Vec<JsObject>> {
    let mut roots = Vec::new();
    for namespace in host_namespaces::States(state) {
        roots.extend(SynchronizeNamespaceGraph(ctx, &namespace)?);
    }
    Ok(roots)
}
fn SynchronizeNamespaceGraph(
    ctx: &mut Context,
    state: &Rc<RefCell<State>>,
) -> JsResult<Vec<JsObject>> {
    let _boundary = bridge_profile::enter(bridge_profile::Phase::Graph);
    let source = state.borrow().host_graph.source.clone();
    let Some(source) = source else {
        return Ok(Vec::new());
    };
    let released = {
        let mut state = state.borrow_mut();
        let released = state.host_graph.TakeReleased();
        for id in &released {
            if let Some(wrapper) = state.wrappers.remove(id) {
                state
                    .namespaces
                    .RemoveWrapperIdentity(wrapper.object.identity());
                state.wrapper_ids.remove(&wrapper.object.identity());
            }
        }
        released
    };
    if !released.is_empty() {
        source.Released(&released);
    }
    let changes = {
        let _collect = bridge_profile::enter(bridge_profile::Phase::GraphSource);
        source.TakeChanges()
    };
    let _apply = bridge_profile::enter(bridge_profile::Phase::GraphApply);
    state.borrow_mut().host_graph.Apply(ctx, changes)
}

#[cfg(test)]
mod ephemeron_tests {
    use super::*;
    #[test]
    fn rooted_registry_does_not_root_a_key_through_its_value_cycle() {
        let mut context = Context::new();
        let mut graph = HostGraph::new(None);
        let stack_roots = graph
            .Apply(
                &mut context,
                vec![
                    HostObjectTraceChange::Object { id: 1 },
                    HostObjectTraceChange::Object { id: 2 },
                    HostObjectTraceChange::Object { id: 3 },
                    HostObjectTraceChange::Root {
                        id: 1,
                        reason: 0,
                        rooted: true,
                    },
                    HostObjectTraceChange::Reference {
                        owner: 3,
                        slot: 0,
                        target: Some(2),
                    },
                    HostObjectTraceChange::Ephemeron {
                        owner: 1,
                        slot: 0,
                        entry: Some((2, 3)),
                    },
                ],
            )
            .unwrap();
        let key = graph.cells[&2].weak.upgrade().unwrap();
        drop(stack_roots);
        context.collect_garbage();
        assert!(
            graph.cells[&3].weak.upgrade().is_some(),
            "live key retains its value"
        );
        drop(key);
        context.collect_garbage();
        assert!(
            graph.cells[&2].weak.upgrade().is_none(),
            "value-to-key cycle cannot rescue a weak key"
        );
        assert!(
            graph.cells[&3].weak.upgrade().is_none(),
            "registry retains no value after key collection"
        );
        assert!(
            graph.cells[&1].weak.upgrade().is_some(),
            "the registry remains rooted"
        );
    }
}

// Ordinary host wrappers are the cell itself. Exotic wrappers additionally
// retain their public Proxy identity while the host target is reachable.
pub(super) fn AttachWrapper(
    ctx: &mut Context,
    object: &JsObject,
    target: &JsObject,
) -> JsResult<()> {
    if object.identity() != target.identity() {
        ctx.set_traced_reference(target, WRAPPER_SLOT, Some(object))?;
    }
    Ok(())
}

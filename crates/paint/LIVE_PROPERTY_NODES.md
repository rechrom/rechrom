# S2 design: live paint property nodes

Status: written 2026-11-23 at main `7f978395`; S2-1 and S2-3 (markers and
node values sequence-confined), S2-4 (in-place PrePaint slot updates, D2)
and S2-5 (in-place direct and retained-scroll updates) are implemented,
S2-6 is not (see "As implemented" below).
Stage S2 of the live-property-tree plan (S0 map, S1 split
LayerizeEngine from CompositorEngine, **S2 live nodes**, S3 delete the
snapshot machinery, S4 contract/guards, S5 verification).

Hard prerequisite: **S1b-7** (LayerizeEngine / PAC runs on the
`page-document-*` thread and no `Arc<PaintArtifact>` is sent to
`page-compositor-*`). S2 before S1b-7 would turn today's benign cross-thread
lifecycle-atomic race into undefined behaviour; see section 3.1.

Chromium revision `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`. Rechrom line
numbers refer to `7f978395`. Chromium paths are abbreviated:
`pgp/` = `third_party/blink/renderer/platform/graphics/paint/`,
`pgc/` = `third_party/blink/renderer/platform/graphics/compositing/`,
`core/paint/` = `third_party/blink/renderer/core/paint/`.

## As implemented (2026-11-24; S2-1 and S2-3 first part on main `a6d70d05`, node values on top of `a8122ce9`)

Decisions D1-D6 were fixed before implementation (D1 per-call `with_access`
in layerize handles, explicit proof elsewhere; D2 builder-context OOF parent;
D3 keep `DirectlyUpdateScrollOffsetTransform` in Layerize; D4 keep the nested
SVG `PaintRecording`, no S2-7; D5 unaliased; D6 fold S2-2 into S2-3).

Landed:

- **S2-1** (`feat(paint): add the sequence-confined property access
  primitive`): `paint_model::property_cell` with `PaintSequenceId`,
  `PropertyAccess` / `PropertyWriter`, `with_access`, `with_writer`,
  `PropertyCell`, `MutableCell` and unit tests (cross-thread `get`/`set`
  panic, reentrant writer panic, reentrant `with_mut` panic, foreign-thread
  `Drop`, distinct sequences). Deviation from 3.3/3.5: the GeometryMapper
  cache generations are a separate sequence-local `Cell` (free functions
  `TransformCacheGeneration`, `ClipCacheGeneration`, `ClearTransformCache`,
  `ClearClipCache`) rather than a field of `PropertyAccess`, so cache code can
  read them inside a writer scope without a `RefCell` borrow. The only
  `unsafe` is the four `UnsafeCell` dereferences and the `Send`/`Sync` impls
  of the two cell types, justified in the module comment (invariants 1-3).
- **S2-2/S2-3, first part** (`refactor(paint): confine property node markers
  to the document sequence`): `PaintPropertyNodeIdentity` keeps `changed`,
  `changed_sequence_number` and the cc node mapping in `MutableCell`s owned by
  the creating sequence (no atomics); `node_changed`, `clear_changed`,
  `changed_sequence_number`, `cc_node_id`, `set_cc_node_id`,
  `CompositorSimpleValuesUpdated`, `ClearChangedToRoot[In]`,
  `Changed`/`ChangedToRoot[In]` and every `property_node_geometry::*Changed*`
  query take `&PropertyAccess`. Root sentinels are per sequence (a
  thread-local in `PropertyTreeState::default`; `Default` is kept because the
  thread-local needs no proof). Both cache generations are per sequence.
  Layerize handles and the Paint engine's composited-animation collector take
  the proof per call (D1). Private `PaintPropertyNodeLifecycle::updated`
  takes it per call too; it is deleted by S2-4.

- **S2-3, node values** (four commits, one per node type in link order:
  scroll, clip, effect, transform; each compiles on its own). Every node is
  now `{ lifecycle, id, [geometry_mapper_cache,] state: PropertyCell<
  *PaintPropertyNodeState> }`; the `State` struct holds Chromium's
  `Node::State` fields plus the `parent` link (and Rechrom's Layout
  association fields) under the same names. `Node::new(lifecycle, id,
  state)` replaces struct literals; `state(&a)` reads, `state_mut()` edits a
  snapshot that is not shared yet (no proof needed: `&mut` is exclusive).
  The Chromium-named predicates (`Matrix`, `ScrollNode`, `HasRealEffects`,
  `PaintClipRect`, ...) are implemented on the `State` and exposed on the
  node through `state_getters!` wrappers that take `&PropertyAccess`; every
  node has `Parent(&a)`. `PaintPropertyNodeGeometry::Parent` takes the
  proof. `PropertyCell::with_owner` was added (no new `unsafe`);
  `PropertyCell`'s `Debug` prints the value only on its owner sequence.
  Proof plumbing: explicit in `PaintPropertyTreeBuilder` (a field set at its
  one construction site), `CullRect::ApplyPaintProperties` and its helpers,
  `property_node_geometry`, `pre_paint_state_compare` (one `with_access` per
  comparison); per call (`with_access`) in the native GeometryMapper handles
  and context, layerize handles (`runtime::paint::read`), the snapshot store
  and rebinders, the PrePaint reparent / direct-animation / resident hit-test
  helpers, Paint recording, interaction hit test and the Page.

Deviations and open points of S2-3:

- Snapshot semantics are unchanged on purpose: node `Clone` copies the state
  (one `with_access`), `PartialEq` still compares lifecycle, id and state
  values, and identity is still the lifecycle `Arc`, not `Arc::ptr_eq` on the
  node. Identity by pointer only becomes true with in-place `Update` (S2-4);
  doing it now would change behaviour of the snapshot store.
- The typed caches (`OpaquePaintCacheSlot` / per-node `Mutex` replacement)
  and `CacheTreeScope` removal are not done: snapshot nodes are still
  resolved through different `FrozenPropertyTrees` until S2-6.
- D1 in GeometryMapper is only partly explicit: the source GeometryMapper is
  generic over the handle traits it shares with layerize
  (`TransformCacheNode`, `GeometryMapperNode`, `GeometryMapperContext`), so
  its native handles take the proof per call. Making it explicit means
  threading `&PropertyAccess` through those trait methods (or carrying it in
  the context) and should be decided on PrePaint / Layerize p50 evidence.
- S2-4 hazard: the per-call `with_access` sites inside PrePaint
  (`pre_paint_tree_walk.rs` reparent, rebase, direct-animation collection,
  `resident_hit_test.rs`, `cull_rect_updater.rs`) would panic once the walk
  runs under `with_writer`; S2-4 must route them through the walk's proof.
- Tests: `paint_model`, `paint` and `layerize` test targets compile;
  `prepaint` tests remain broken as on main (their builder fixtures call
  `PaintPropertyTreeBuilder::new` without the new proof argument too).

- **S2-4** (`refactor(paint): update property slot nodes in place during
  PrePaint`, 72bddaf6). `PaintPropertyNodeStore` is ObjectPaintProperties:
  `SlotId(key)` gives a slot's stable node id (the builder's
  `RegisterProperty(fragment, role) -> id` uses it), and
  `Update{Transform,Clip,Effect,Scroll}(writer, key, node)` creates the
  slot (kNodeAddedOrRemoved) or calls the node's in-place
  `Update(&self, writer, State, anim)` (SetParent + ComputeChange +
  AddChanged; transform/clip changes bump the sequence's cache
  generations). A node whose state links a newer allocation of the same
  entity stores that state without promoting the change category. The
  pass's maximum change and intersection-observation effect are recorded
  per update; slots no object registered are removed at the end of every
  walk (`RetainLiveSlots`). Viewport clip and visual-viewport nodes use the
  same slots. Builder `*_id` fields are the slot nodes' ids
  (`SyncPropertyIds`); `NodeCache::own_change` records the build's change
  so `SameGeometry` still sees value changes on identical live nodes.
  **D2**: `LogicalOutOfFlowEffectContext` gives an absolute/fixed object
  whose logical parent occurrence is not a physical ancestor that
  occurrence's recorded effect state (`logical_effect_contexts`, updated per
  visited node and rebuilt from the final tree in every post-update pass)
  before admission, so its effect is updated once with the final parent.
  The post-update pass keeps the logical tree moves and only fixes up, in
  place, an occurrence the physical walk reached after the object
  (`PrePaint.RestoreLogicalTree logical_effect_fixups`).
  Deleted: `PropertyNodeUpdater`, `update_snapshot!`,
  `PersistPropertyStates`, the `raw_*` builder caches and
  `RestoreBuilderStates`' property restore, `RebaseOwnEffect`, the cloning
  `OwnNodeRebase` (now a proof that own nodes keep their links),
  `prior_property_keys`, `GatherRetainedRawPropertyIds`, the key half of
  `SameBuilderStructure`/`SameBuilderState` (same slot = same node entity),
  `PaintPropertyNodeLifecycle::parent_id`, `unresolved_parent_ids`,
  `snapshot_stats` and the `PrePaint.ParentIds`/`PrePaint.Snapshots` traces.
- **S2-5** (`refactor(paint): apply direct animation and scroll updates in
  place`, 6a892206). Direct composited animation updates admit all proposals
  from computed changes (`DirectTransformAndOriginChange`,
  `ComputeOpacityChange`) and then update the slot nodes in place; the
  retained scroll path updates ScrollTranslation/StickyTranslation in place
  (`retained_scroll_update` returns the admitted values,
  `ApplyRetainedScrollUpdate` writes them, same change categories as before)
  and no longer rewrites chunk states, display-item scroll translations or
  viewport metadata. Deleted: `PublishDirect*Snapshot`, `staged_snapshots`,
  `RetainedScrollSnapshotRebinder`, `cleared_change_markers`,
  `PaintPropertyTrees::clear_state_change_markers`,
  `CommitDirectScrollPropertySnapshots`, `RebindResidentPropertySnapshots`.

Deviations of S2-4/S2-5:

- Proof scopes are narrow, not a walk-wide writer: the PrePaint walk runs
  under no proof, reads take `with_access` per expression and every slot
  update takes `with_writer` for that one update. A walk-wide writer would
  make every per-call `with_access` reachable from the walk (native
  GeometryMapper handles, `CullRect`, node `Clone`/`PartialEq`/`ToJSON`)
  panic. The builder therefore no longer holds the explicit proof S2-3 gave
  it (D1 partly reverted in PrePaint). The invariant to keep: no slot
  update may run inside a `with_access` closure (it panics, it cannot race).
- `OpaquePaintCacheSlot`, `CacheTreeScope`, `FrozenPropertyTrees`, the
  resolvers and `ArtifactPropertyStateRebinder` stay until S2-6: with live
  nodes they resolve every link to itself but are still read.
- The retained scroll path updates nodes before its coverage proof; a
  rejection leaves the new values in place and the ordinary lifecycle that
  follows computes the same values (Chromium's PrePaint also updates the
  scroll translation before Paint decides).
- Node `Clone`/`PartialEq` are still value-based with `with_access`; they
  must not run inside a writer scope (none does). Make them identity in S2-6.
- `prepaint` unit tests still do not compile (pre-existing); the named
  S2-4 tests were not run.
- Fixed after runtime verification (`fix(prepaint): refresh retained flat
  transforms under live property nodes`): PrePaint's flat transform lists
  (`InheritedContext`/`ChildContext::transforms`) are a Rechrom mirror of
  the property tree. Retained nodes kept the flat values of the pass that
  built them; with snapshots their retained contexts also kept that pass's
  nodes, with live nodes they do not. Retained nodes now refresh the
  inherited prefix of their flat lists. Any other retained flat geometry
  that is combined with live node values is the same hazard class.

Not landed (stopped here because D: had 1.4 GB free): S2-6. No iteration
build and no runtime verification (section 6) have been done for S2-4/S2-5.

Runtime facts this part changes: a read of a node marker, cc id or node
value from a thread other than the one that created the node now panics
(previously a relaxed atomic race or an unchecked shared read); this includes
node `Clone` and `PartialEq` (and so `PartialEq` of any type holding a node
`Arc`, which delegates to the node). `Debug` does not panic.
`PropertyTreeState::default()` on a second thread yields that thread's own
roots. After S1b-7 all such readers run on the Page document thread; raster
threads replay only `PaintRecording::items` and the impl-side commit
vocabulary (`layerize::commit`) names no paint property node.

## 0. Target in one paragraph

A Blink paint property node is one long-lived object per
ObjectPaintProperties slot. `Update(parent, State)` mutates it in place and
returns `PaintPropertyChangeType`; the parent link is an owning reference;
`changed_`, `changed_sequence_number_`, `cc_node_id_` and the GeometryMapper
cache are `mutable` members written through `const` references by PAC and
GeometryMapper; any transform/clip change bumps a single generation
(`GeometryMapperTransformCache::ClearCache`) that invalidates every cache. A
`PaintArtifact` holds `Member` references to these same nodes, so every
retained artifact (PAC's previous artifact, the raster invalidator's old
artifact, cached subsequences) observes current values and current parents
with no rebinding. S2 gives Rechrom exactly this: `Arc<Node>` identity,
in-place `Update`, `Cell` change flags, one cache generation per document
sequence, and an access discipline that makes all of it sound in Rust because,
after S1b-7, every reader and writer runs on the document sequence.

## 1. Current state map (7f978395)

### 1.1 Node types, identity and "revisions"

All in `crates/paint/paint_model/src/paint_property_tree.rs`.

| Item | Lines | What it is today |
|---|---|---|
| `PaintPropertyChangeType` | 25-51 | Same five values as `pgp/paint_property_node.h:30-56`. Keep. |
| `PaintPropertyNodeIdentity` | 56-121 | Separate `Arc` allocation per node *entity*: `cc_sequence_and_node_id: AtomicU64`, `changed: AtomicU8`, `changed_sequence_number: AtomicI32`. Atomics exist because PAC on the compositor thread writes them while PrePaint on the document thread reads them. |
| `PaintPropertyNodeLifecycle` | 124-213 | Per-*snapshot* value: `identity: Arc<Identity>`, `revision: u64`, per-snapshot `changed`, `parent_id`. `PartialEq` = same identity **and** same revision (155-158). `updated()` (178-187) makes the next revision. `CompositorSimpleValuesUpdated` (202-212) is a CAS on the shared atomic. |
| `TransformPaintPropertyNode` | 251-278, impl 312-657 | Plain `#[derive(Clone, PartialEq)]` struct; every field public; `parent: Option<Arc<Self>>`, `scroll`, `scroll_parent_scroll_translation` are `Arc` links; `geometry_mapper_cache: OpaquePaintCacheSlot`. `DirectlyUpdateTransformAndOrigin` (499-517) mutates a `&mut` clone and resets only that clone's cache. `ClearChangedToRoot[In]` (520-541). |
| `ClipPaintPropertyNode` | 680-700, impl 702-877 | Same pattern; `local_transform_space`, `pixel_moving_filter` links; `ClearChangedToRoot[In]` 793-812. |
| `EffectPaintPropertyNode` | 914-964, impl 1002-1419 | `DirectlyUpdateOpacity` 1302-1315; `ClearChangedToRoot[In]` 1317-1340. |
| `ScrollPaintPropertyNode` | 1421-1442, impl 1512-1740 | `ClearChangedToRoot[In]` 1599-1620. |
| `PropertyTreeState` | 1742-1996 | Three `Arc`s; `PartialEq` by entity (`same_node`) 1749-1757. `Default` = process-wide `static ROOT: OnceLock` (1765-1866) shared by **all Pages and threads**. `ClearChangedToRoot[In]` 1892-1910. |
| `OpaquePaintCacheSlot` | `opaque_paint_cache_slot.rs:1-45` | `OnceLock<Arc<dyn Any + Send + Sync>>`; holds a `Mutex<cache>`; `Clone` deliberately yields an empty slot, so every new snapshot starts a cold cache. |

Consequences: one Blink node is represented by a chain of immutable `Arc`
snapshots sharing an `Arc<Identity>`. A child snapshot's `parent` `Arc` may
point at an **older** snapshot of its parent; readers therefore "resolve"
every link through a publication directory (1.4).

### 1.2 Writers (all on the document thread today, except 1.6)

| Writer | Location | Mechanism |
|---|---|---|
| PrePaint builder, raw nodes | `crates/paint/prepaint/src/pre_paint_tree_walk/paint_property_tree_builder.rs:460` (transforms), `:1597` (scroll), `:1903/1933/1967` (scroll/sticky/replaced translations), and the clip/effect builders in the same file | Every built object allocates **fresh** `Arc` nodes with raw per-pass ids, registered by `RegisterProperty` (`pre_paint_tree_walk.rs:7751-7770`) into `property_keys` / `prior_property_keys` (`:1059`, `:1091`). |
| Logical out-of-flow effect reparent | `pre_paint_tree_walk.rs:4025` `RebindLogicalEffectState`, `:5002` `RebaseOwnEffect`, `:4195` `RestoreLogicalOutOfFlowParents`, `:4518` `RestoreBuilderStates` | Clones the raw effect, relinks parent/transform space/output clip after the physical walk. Raw states are cached per node (`cache.raw_*`) and restored each pass. |
| Canonicalisation into slots | `pre_paint_tree_walk.rs:8026-8033` (post-walk) -> `PersistPropertyStates` `:4445-4475` -> `PropertyNodeUpdater` (`paint_property_tree.rs:4056`, `update_snapshot!` 4446-4490, `transform/clip/effect/scroll` 4563-4725, `finish` 4726-4734, `finish_slots` 4736-4766) | Maps raw ids to `PaintPropertyKey` slots (`KeyedPropertyNodes` 2252-2332, store `PaintPropertyNodeStore` 2703-3611), computes `*_state_change` (4201-4440, a 1:1 translation of `State::ComputeChange`), and either reuses the previous snapshot or allocates a new revision. `finish` bumps both global cache generations when anything changed. |
| Visual viewport nodes | `paint_property_tree.rs:2899` `ensure_single_layout_viewport_nodes`, called at `pre_paint_tree_walk.rs:7880` | Same snapshot replacement. |
| Composited animation direct update | `pre_paint_tree_walk.rs:8208` `DirectlyUpdateCompositedAnimationProperties` -> `PublishDirectTransformSnapshot` / `PublishDirectEffectSnapshot` (`paint_property_tree.rs:2829-2861`, called at `pre_paint_tree_walk.rs:8398`, `:8417`) | Clones the slot node, `DirectlyUpdate*` on the clone, replaces the slot. Older artifacts keep the old value. |
| Retained scroll (property-only scroll) | `crates/paint/paint_commit/src/scroll_paint_update.rs:915` `TryUpdateScrollPropertiesWithPrePaint` -> `retained_scroll_update` (`paint_property_tree.rs:3259-3369`) -> `staged_snapshots` (3400-3450) via `RetainedScrollSnapshotRebinder` (3861-4054) -> `CommitDirectScrollPropertySnapshots` (`pre_paint_tree_walk.rs:6031-6039`, call at `scroll_paint_update.rs:1378`) -> `RebindResidentPropertySnapshots` (`pre_paint_tree_walk.rs:5975-6029`) | Builds a whole new store, rewrites every resident `PropertyTreeState` and the artifact's chunk states and viewport metadata (`scroll_paint_update.rs:1345-1376`). |
| Change-marker clearing in Paint | `paint_property_tree.rs:3375` `cleared_change_markers`, `:2073` `PaintPropertyTrees::clear_state_change_markers` | Produces new snapshots with cleared per-snapshot markers. |
| Tests | 81 struct-literal constructions (`grep -E "(Transform|Clip|Effect|Scroll)PaintPropertyNode \{"`): 33 in `paint_property_tree.rs`, 18 in the builder, rest in `paint_record`, `paint_reuse`, `paint_walk`, `paint_chunk`, `cull_rect_updater.rs`, `interaction_hit_test` | All non-builder sites are `#[cfg(test)]`; production painters never create nodes. |

### 1.3 Publication / freezing

| Step | Location |
|---|---|
| `PaintPropertyNodeStore::published_trees` / `invalidate_published` / `freeze` | `paint_property_tree.rs:2772-2789` (`OnceLock<Arc<FrozenPropertyTrees>>`, 2716) |
| `FrozenPropertyTrees` (id -> current snapshot), `verify_state` | 2334-2701 |
| `PrePaintTreeWalk::PublishedTrees` | `pre_paint_tree_walk.rs:5967-5969` |
| Recording gets the trees | `crates/paint/src/paint_engine.rs:694` |
| Cached-subsequence rebinding `ArtifactPropertyStateRebinder` (3624-3858) | `paint_engine.rs:880-912` (chunk states, hit-test scroll translations, display-item scroll translations) |
| Every chunk gets `PublishedTrees(Some(trees))` | `paint_engine.rs:913-922`; field `crates/paint/paint_chunk/src/model.rs:32` |
| Artifact field `property_trees` | `crates/paint/paint_record/src/artifact.rs:306`; set again in `CommitPaintResult` `paint_engine.rs:1447-1463` (with `verify_state` coverage trace) |

### 1.4 Readers

Thread column is today / after S1b-7.

| Reader | Location | Resolves links through | Thread |
|---|---|---|---|
| PrePaint builder, cull rect updater, invalidators, state compare | `prepaint/src/**`, `paint_reuse/src/pre_paint_state_compare.rs` | raw `Arc` links (`LinkedProperties`) + `property_keys` id indirection (`SameBuilderStructure`, `pre_paint_tree_walk.rs:6090-6110`) | doc / doc |
| Paint record / chunker / painters | `paint_record/src/artifact.rs:199`, `:797`; `blocking_wheel_event_regions.rs`; `box_fragment_painter.rs` | `PropertyNodeResolver` over `chunk.property_trees` | doc / doc |
| Retained scroll path | `scroll_paint_update.rs:167-400`, `1060-1140` | `LinkedProperties` + `PaintPropertyTrees::transform_for` | doc / doc |
| Non-source mapper (`paint_model/src/geometry_mapper.rs` 102-300), `property_node_geometry.rs` | `*In(resolver, ...)` variants | any resolver | doc + compositor / doc |
| Source GeometryMapper | `paint_model/src/geometry_mapper_source/*`: `native.rs:12` `NativeTrees = Option<Arc<FrozenPropertyTrees>>`, handles carry it (`native.rs:45-60`, 82); `traits.rs:15` `TransformCacheNode: Send + Sync`, `:29`, `:73` `Trees()` | `FrozenPropertyTrees` | doc + compositor / doc |
| Resident hit test and layer clipper | `prepaint/src/resident_hit_test.rs:355`, `resident_layer_clipper.rs:72`; `crates/interaction/interaction_hit_test/src/resident.rs:1036-1060` | `published_trees()` | doc / doc |
| PAC, PropertyTreeManager, PendingLayer, PaintChunksToCcLayer | `crates/layerize/src/third_party/blink/renderer/platform/graphics/compositing/*` via handles `crates/layerize/src/runtime/paint.rs:147-283` (`HandleTrees`, `resolve_link!`, `link_*`, `from_arc_in`), `pending_nodes.rs`, `scroll_nodes.rs`, `artifact_nodes.rs`, `geometry_nodes.rs`, `geometry_mapper.rs:26-93` | `HandleTrees` = artifact's `FrozenPropertyTrees` | **compositor** / doc |
| Raster invalidator | `.../graphics/paint/raster_invalidator.rs:88` (`old_paint_artifact_`), `:197` `ChunkPropertiesChanged`; `runtime/paint/invalidation.rs:200-300` | handles; old chunk geometry is a captured `PaintChunkInfo` value | compositor / doc |
| LayerizeEngine previous artifact | `crates/layerize/src/layerize_engine.rs:108`, `:512`, `:565`; `runtime/paint_services.rs:849-960` (direct scroll update 867-905, property-change classification 907-960) | handles | compositor / doc |
| Video / drag-image conversion | `layerize_engine.rs:218-229` `ConvertPaintArtifactSubsetToRecord` | artifact trees | doc / doc |
| Artifact validation (incl. nested SVG artifacts) | `runtime/paint/validation.rs:147-205`, `:272-286` | raw links | compositor / doc |
| Raster | `crates/raster/src/execution/translated.rs:7098-7160`, `:9166-9260` | **none**: replays `items` only | raster / raster |
| impl-side cc (`crates/compositor/src/cc/**`) | none (S0 verified) | - | - |

### 1.5 Holders that can reach other threads

Nodes are *held* (not read) off the document thread through:

1. `ImplCommand::Commit { artifact }` (`apps/rechrom/src/page_runtime.rs:3390`, consumed `:1884`) - removed by S1b-7.
2. Main cc `Layer` objects: `LayerObject: Send + Sync` (`crates/layerize/src/cc/layers/layer.rs:184`); `ContentLayerClientImpl` (`.../compositing/content_layer_client_impl.rs:42-49`, `Arc<RwLock<..>>` at 81/480) owns the raster invalidator with `old_paint_artifact_` (`SetOldPaintArtifact` 525). The impl thread runs `FinishCommit` over the main layer tree while main is parked (S1b proxy design), so these types must stay `Send + Sync`. Impl code never reads Blink nodes through them.
3. SVG images: `DocumentPaintArtifactRecord { artifact: Arc<PaintRecording> }` (`paint_record/src/artifact.rs:219-226`, produced `crates/document_image/src/svg_image.rs:623`) is an `image_resource::DocumentPaintRecord: Any + Send + Sync` (`crates/image_resource/src/lib.rs:338`). Raster threads hold it and replay `record.artifact.items` only (`translated.rs:9166-9183`); `layerize/src/commit/paint/image_values.rs:163` and `PaintRecord::from_artifact` (`layerize/src/commit/paint.rs:63-71`) also take items/resources only. The last `Arc` may be dropped on a raster thread.
4. The process-wide root sentinels (`paint_property_tree.rs:1769`) are shared by every Page thread.

### 1.6 Mutable state written across threads today

| State | Written by | Read by |
|---|---|---|
| `Identity.changed` | `add_changed` in PrePaint (`updated`, 178-187); `clear_changed` and `CompositorSimpleValuesUpdated` by PAC on the compositor thread (`runtime/paint.rs:383-410`, `paint_artifact_compositor.rs:2017-2032`, `paint_services.rs:810-813`, `:897`) | PrePaint `update_snapshot!` (4467-4471), PAC `NodeChanged` (`pending_layer.rs:290`, `:688`, `property_tree_manager.rs:639`, `:1867`) |
| `Identity.changed_sequence_number` | PAC `ClearChangedToRoot` | PAC |
| `Identity.cc_sequence_and_node_id` | PAC `set_cc_node_id` (`runtime/paint.rs:276`, `pending_nodes.rs:60/243/367`, `scroll_nodes.rs:47`) | PAC |
| `S_GLOBAL_GENERATION` x2 | `geometry_mapper_transform_cache.rs:23`, `geometry_mapper_clip_cache.rs:19`; bumped by `PropertyNodeUpdater::finish` (doc) | caches on both threads |
| per-node cache `Mutex` | `WithTransformCache` (`geometry_mapper_transform_cache.rs:404-418`), clip `:186` | both threads |
| `CacheTreeScope` (`Weak<FrozenPropertyTrees>`) | transform 102-117, clip 66 | both threads |

The `changed` race is logical, not memory-unsafe (atomics): a PAC
`ClearChangedToRoot` on the compositor thread can clear a marker PrePaint just
raised for the next frame. `CacheTreeScope` exists only because one node
snapshot is shared by two published trees that two threads map concurrently.

### 1.7 Snapshot / resolver / rebinder construction sites

`FrozenPropertyTrees`: `paint_property_tree.rs:2772-2789` only (frozen once
per store change). `PublishedTrees(Some(..))`: `paint_engine.rs:918-919`.
`ArtifactPropertyStateRebinder::new`: `paint_engine.rs:886`.
`RetainedScrollSnapshotRebinder`: `paint_property_tree.rs:2073-2084`,
`3400-3450`. `PropertyNodeUpdater`: `paint_property_tree.rs:3565`, used at
`pre_paint_tree_walk.rs:8026`. `NativeTrees` handles: `native.rs:51`, `:82`.
`HandleTrees` handles: `runtime/paint.rs:181`, `:350`, `:421`, `:458-502`,
`pending_values.rs:159`, `geometry_mapper.rs:45-93`,
`geometry_nodes.rs:26`, `:155`, `layerize_engine.rs:225-227`.
`LinkedProperties`: `scroll_paint_update.rs:1065-1269`,
`paint_model/src/geometry_mapper.rs:176-283`.

## 2. Chromium target mapping

| Chromium | Rust target (S2) |
|---|---|
| `PaintPropertyNode : GarbageCollected`, `Member<const Node>` / `scoped_refptr` | `Arc<TransformPaintPropertyNode>` etc. Pointer = identity (`Arc::ptr_eq`). No arena: an old artifact must keep a removed node alive exactly like a traced `Member`. `PaintPropertyNodeIdentity` and `PaintPropertyNodeLifecycle.revision/parent_id` disappear. |
| `Node::State state_` (`transform_paint_property_node.h:110-134`) | `state: PropertyCell<TransformState>` (section 3). Values move from public struct fields into a `State` struct with the same field names; read accessors mirror Chromium (`Matrix()`, `Origin()`, `ScrollNode()`, `Parent()` ...). |
| `Member<const PaintPropertyNode> parent_` (`paint_property_node.h:232`) | `parent: Option<Arc<Self>>` inside the cell (it changes in `SetParent`). Always the live node. |
| `Update(parent, State&&, AnimationState)` (`transform_paint_property_node.h:173-185`, clip `:168-178`, effect `:216-227`, scroll `:109-120`) | `fn Update(&self, w: &mut PropertyWriter, parent: &Arc<Self>, state: State, anim) -> PaintPropertyChangeType`: `SetParent` + existing `*_state_change` (4201-4440) + `AddChanged`. |
| `SetParent` (`paint_property_node.h:134-145`) | same; pointer compare, `AddChanged(kChangedOnlyValues)`. |
| `AddChanged` (`paint_property_node.h:185-188`); transform override clears both caches (`transform_paint_property_node.h:50-55`); clip clears clip cache (`clip_paint_property_node.h:53-57`) | `changed: Cell<PaintPropertyChangeType>` max; transform/clip variants bump the sequence's cache generations (3.5). |
| `mutable changed_`, `changed_sequence_number_`, `cc_node_id_`, `cc_sequence_number_` (`paint_property_node.h:214-230`) | plain `Cell`s on the node, written through `&Node` + `&PropertyAccess` (no atomics). |
| `CompositorSimpleValuesUpdated` (`paint_property_node.h:82-85`) | `Cell` compare-and-set. |
| `ClearChanged` / `ClearChangedToRoot` (`transform_paint_property_node.cc:151-164`, `property_tree_state.h:142-146`) | follow live `Parent()` / `ScrollNode()`; all `*In(resolver)` variants deleted. |
| `ObjectPaintProperties::Update<NodeType>` (`object_paint_properties.h:528-551`): create returns `kNodeAddedOrRemoved`, else `node->Update` | `PaintPropertyNodeStore` slot (`PaintPropertyKey` = owner + role) becomes the ObjectPaintProperties: `UpdateTransform(key, parent, state)` creates or updates in place. Clear = remove slot (old artifacts keep the `Arc`). |
| Aliases (`ADD_ALIAS_NODE`, `UpdateAlias` 553-575; `Unalias()`) | Not used by Rechrom production today (`paint_property_tree.rs:214-215`). S2 keeps nodes unaliased; `RebaseOwnEffect`-style wrappers become real slot updates (4.3). Adding aliases is out of scope (open decision D5). |
| `DirectlyUpdateTransformAndOrigin` / `DirectlyUpdateOpacity` (`object_paint_properties.h:503-522`) | in place on the slot node through `&mut PropertyWriter`; `PublishDirect*Snapshot` deleted. |
| `PropertyTreeStateOrAlias { Member transform_, clip_, effect_ }` (`property_tree_state.h:64-66`), `PaintChunk::properties` (`paint_chunk.h:163`) | unchanged shape: `PropertyTreeState { Arc, Arc, Arc }`; chunk keeps it; `property_trees` / `PublishedTrees` fields deleted. |
| PAC previous artifact, `PendingLayer` chunk subsets, `RasterInvalidator::old_paint_artifact_` | unchanged `Arc<PaintArtifact>` retention; reads now see live nodes, exactly like Chromium. Old *values* that Chromium needs are already captured (`PaintChunkInfo` transform/clip, `raster_invalidator.rs:197-260`). |
| `PaintArtifactCompositor::Update` tail `ClearChangedToRoot(g_s_property_tree_sequence_number)` (`paint_artifact_compositor.cc:1378-1388`) | `paint_artifact_compositor.rs:2017-2032` unchanged except link resolution. |
| `GeometryMapperTransformCache::ClearCache` / `s_global_generation` (`geometry_mapper_transform_cache.cc:15-24`), clip equivalent | one generation pair per document sequence (3.5). `CacheTreeScope` deleted. |
| `TransformPaintPropertyNode::GetTransformCache` lazily allocated `mutable Member<GeometryMapperTransformCache>` (`transform_paint_property_node.h:472-485`) | `transform_cache: CacheCell<Option<Box<GeometryMapperTransformCache>>>` typed field; `OpaquePaintCacheSlot` and the per-node `Mutex` deleted (the slot existed to break a crate cycle that no longer exists: the caches live in `paint_model`). |
| `TransformPaintPropertyNode::Root()` (`transform_paint_property_node.cc:117-122`, `DEFINE_STATIC_LOCAL`: one per process, painted on one main thread) | one root set per document sequence (thread), created lazily; `PropertyTreeState::Root()` replaces `Default` (3.4). |
| `paint_property_tree_builder.cc:3798-3827` `DirectlyUpdateScrollOffsetTransform` from inside the builder | stays in `LayerizeEngine::UpdatePaintArtifact` (`paint_services.rs:867-905`) for S2; now operates on the live node. Moving it into PrePaint is open decision D3. |
| `LocalFrameView` -> `PaintArtifactCompositor::SetNeedsUpdate` when property trees changed | typed `property_tree_changed: PaintPropertyChangeType` from PrePaint (the store's `changed`) carried with the artifact into `UpdatePaintArtifact`, replacing the heuristic at `paint_services.rs:941-957`. |

## 3. Soundness design

### 3.1 Thread facts after S1b-7

- Writers: only the PrePaint walk and its direct-update entries, on the
  document thread.
- Readers: Paint, PrePaint, Interaction hit test, LayerizeEngine (PAC,
  PropertyTreeManager, PaintChunksToCcLayer, RasterInvalidator, conversions),
  SVG `document_image` lifecycle, artifact validation - all on the same
  document thread (the SVG decoder is `Rc<RefCell<..>>` owned by the Page,
  `crates/browser/src/page.rs:10574`, `apps/rechrom/src/engine.rs:478`; the
  Page itself is constructed inside the document thread, `page_runtime.rs:710-728`).
- Mutable-member writers (`ClearChangedToRoot`, `set_cc_node_id`,
  `CompositorSimpleValuesUpdated`, caches): same thread.
- Off-thread *holders* that never read (1.5 items 2 and 3): main cc layers
  during `FinishCommit`, raster threads holding SVG records. They need the
  types to stay `Send + Sync` and may run the final `Drop`.
- No parallel workers in `paint`, `layerize`, `interaction`,
  `document_image` or `compositor_layerize` (no rayon / `thread::scope` /
  spawn in those crates).

So nodes need: `Send + Sync` types (for the holders), but all *access*
confined to the one document sequence that owns them.

### 3.2 Options considered

| Option | Verdict |
|---|---|
| A. Atomics for every field / `RwLock<State>` per node | Rejected. Matrices and filter lists are not atomic; a lock per read on GeometryMapper ancestor walks (the hottest PrePaint and Layerize loop) costs an atomic RMW per access and cannot hand out `&Matrix`. |
| B. `Rc` + `RefCell` (`!Send`) | Rejected. Breaks `LayerObject: Send + Sync`, `DocumentPaintRecord: Send + Sync`, `PaintArtifact` in every `Send` bound; the main-layer `FinishCommit` path would need `unsafe impl Send` wrappers anyway. |
| C. Arena of nodes + ids | Rejected. Old artifacts must keep removed nodes alive (Member semantics) -> generational ids plus artifact-scoped keep-alive; reintroduces the id indirection S3 deletes. |
| D. `Arc` + sequence-confined cells (chosen) | `Send + Sync` holders, compile-time no-aliasing for writes, runtime thread check folded into a token compare. |

### 3.3 Chosen design: `Arc` nodes with sequence-confined state

New module `crates/paint/paint_model/src/property_cell.rs` (no new crate):

```rust
/// One document sequence (thread). Lazily assigned per thread; never reused.
#[derive(Copy, Clone, Eq, PartialEq)] pub struct PaintSequenceId(NonZeroU32);

/// Shared read proof: exists only on the owning thread (!Send, !Sync).
pub struct PropertyAccess { id: PaintSequenceId, generations: Cell<[u32; 2]>, roots: Roots, _not_send: PhantomData<*const ()> }
/// Exclusive write proof: &mut PropertyAccess.
pub type PropertyWriter = PropertyAccess;

thread_local! { static ACCESS: RefCell<Option<PropertyAccess>> }
pub fn with_access<R>(f: impl FnOnce(&PropertyAccess) -> R) -> R;      // borrow()
pub fn with_writer<R>(f: impl FnOnce(&mut PropertyAccess) -> R) -> R;  // borrow_mut()

pub struct PropertyCell<T> { owner: PaintSequenceId, value: UnsafeCell<T> }
impl<T> PropertyCell<T> {
    #[inline] pub fn get<'a>(&'a self, a: &'a PropertyAccess) -> &'a T { assert!(self.owner == a.id); unsafe { &*self.value.get() } }
    #[inline] pub fn set(&self, w: &mut PropertyAccess, v: T) -> T { assert!(self.owner == w.id); unsafe { mem::replace(&mut *self.value.get(), v) } }
}
unsafe impl<T: Send> Send for PropertyCell<T> {}
unsafe impl<T: Send> Sync for PropertyCell<T> {}
```

Soundness argument:

1. `PropertyAccess` is `!Send + !Sync` and lives in a thread-local, so a
   `&PropertyAccess` with id `X` exists only on thread `X`.
2. Every read or write of node state requires a proof whose id equals the
   node's owner, so all accesses to one node happen on one thread.
3. Writes need `&mut PropertyAccess`; reads return `&'a T` tied to a shared
   `&'a PropertyAccess`. The borrow checker therefore forbids any live `&T`
   across an `Update`, which is the aliasing rule `RefCell` would check at
   runtime, at zero per-access cost. Reentrancy (a write requested while a
   read phase holds the thread-local borrow) panics in `with_writer`, once
   per phase, not per access.
4. `Drop` takes `&mut self`, needs no proof, and is sound on any thread: the
   last `Arc` owner has exclusive access. Dropping a node decrements its
   parent's `Arc` atomically.
5. Off-thread holders can name nodes but cannot obtain a proof for them; an
   accidental read panics instead of racing (the release-mode `assert!` is
   one register compare against a value already in cache; it stays).

Node layout (transform shown; clip/effect/scroll identical in shape):

```rust
pub struct TransformPaintPropertyNode {
    state: PropertyCell<TransformState>,            // parent + Chromium State
    changed: MutableCell<PaintPropertyChangeType>,   // Cell-like, proof-checked
    changed_sequence_number: MutableCell<i32>,
    cc_node: MutableCell<(i32, i32)>,               // (sequence, id)
    transform_cache: MutableCell<Option<Box<GeometryMapperTransformCache>>>,
    debug_id: u64,                                   // dump label only
}
```

`MutableCell<T>` is `PropertyCell` with `Cell` semantics under a *shared*
proof (`get(&a) -> T: Copy`, `set(&a, T)`, and `with_mut(&a, |&mut T|)` for
the cache with a reentrancy flag; the flag is a non-atomic `Cell<bool>` and
stays in release builds because `with_mut` hands out `&mut T`). That is
Chromium's `mutable` under `const`: PAC and GeometryMapper write through
`&PropertyAccess`. Cache recursion (`UpdateIfNeeded` asking the parent's
cache) touches a different node's cell, so the flag never trips in correct
code.

Hot-path cost versus today:

- Field read: today `Arc` deref; S2 `Arc` deref + one `u32` compare.
- Change flags / cc ids: today `Relaxed` atomics (plain loads/stores on
  x86-64/arm64, but `fetch_max` / CAS are locked RMWs); S2 plain `Cell`.
- Cache: today `OnceLock` + `Mutex::lock` per cache access plus
  `CacheTreeScope` weak compare; S2 `Option<Box>` behind a `Cell` flag.
- Snapshot machinery removed: no per-pass `Arc::new` per changed or
  reference-changed node, no rebinding walks, no freeze (`NodesById::of` over
  every slot), no `verify_state`.
- Thread-local access: once per phase (`with_access` around the PrePaint
  walk, Paint, `LayerizeEngine::UpdatePaintArtifact`, a hit test), not per
  read. On MSVC `thread_local!` with `const` init is native TLS.

Expected net: PrePaint and Layerize p50 decrease (the snapshot and rebinding
work is measurable in today's traces: `PrePaint.PersistPropertyStates`,
`PaintCommit.RebindPropertyStates`, `ScrollPaint.RebindPropertyTrees`).

### 3.4 Roots

`PropertyTreeState::Root()` / `TransformPaintPropertyNode::Root()` return the
current sequence's roots stored in `PropertyAccess` (created on first access
on that thread). This is Chromium's per-process root generalised to one per
document thread, needed because Rechrom runs several Pages per process: the
shared `static ROOT` today receives `ClearChanged` and `set_cc_node_id`
writes from every Page's PAC. `Default for PropertyTreeState` is removed (it
cannot reach the thread-local without a proof); callers use
`PropertyTreeState::Root(&a)`.

### 3.5 GeometryMapper cache invalidation

`PropertyAccess.generations: [transform, clip]` replaces both
`S_GLOBAL_GENERATION` statics. `AddChanged` on a transform bumps both, on a
clip bumps the clip generation (Chromium's exact overrides).
`UpdateIfNeeded` compares the node cache's generation with the sequence's.
`CacheTreeScope`, `NativeTrees`, `Trees()`, `PropertyNodeUpdater::finish`'s
global clear, and the "new snapshot starts a cold cache" behaviour
(`OpaquePaintCacheSlot::clone`) are deleted. Per-sequence generations also
stop one Page's PrePaint from invalidating every other Page's caches.

### 3.6 What remains `Send + Sync`

`PaintArtifact`, `PaintChunk`, `PropertyTreeState`, node types (through
`unsafe impl` on the cells only), `DocumentPaintArtifactRecord`, cc main
layers. Optional follow-up (open decision D4): make
`DocumentPaintArtifactRecord` carry `items + resources + content_revisions`
only (Chromium's SVGImage hands out a `cc::PaintRecord`), which removes
holder 1.5(3) entirely and lets the nested-artifact property walk in
`validation.rs:272-286` go.

## 4. Implementation plan

Each step is one commit, compiles (`cargo check -p rechrom_app`, one run at
the end of the step, `-j 1` on this machine), and keeps behaviour. Steps 2-6
are the risky core; they can be squashed into fewer commits if an
intermediate state does not type-check cheaply, but each listed boundary is a
valid bisection point.

**S2-0. Gate.** S1b-7 merged and runtime-verified (matrix, DPR 2 trackpad
replay + round trip, commit wait). Proof: no `ImplCommand::Commit { artifact`
in `apps/rechrom/src/page_runtime.rs`; `layerize_boundary` test passes
without the transitional allowance from `af7e98cd`; `grep -r PaintArtifact
crates/compositor/src` finds only comments.

**S2-1. Access primitive.** Add `paint_model::property_cell`
(`PaintSequenceId`, `PropertyAccess`, `with_access`, `with_writer`,
`PropertyCell`, `MutableCell`), unused. Check: unit tests in
`paint_model` for cross-thread `get` panic, reentrant `with_writer` panic,
foreign-thread `Drop`.

**S2-2. Phase scopes.** Enter `with_access` / `with_writer` at the phase
roots: `PaintEngine` PrePaint walk (writer) and Paint (reader),
`PrePaintTreeWalk::DirectlyUpdateCompositedAnimationProperties` and the
retained scroll path (writer), `LayerizeEngine::UpdatePaintArtifact` and
`ConvertPaintArtifactSubsetToRecord`, interaction hit-test entry, SVG
document lifecycle entry, test helpers. Pass `&PropertyAccess` down by
replacing the existing context parameters: `PropertyNodeResolver` /
`LinkedProperties` arguments, `NativeTrees`, `HandleTrees`,
`PublishedTrees`. Nodes are not yet live: the proof is threaded but unused.
Check: `cargo check`. This is the large mechanical commit; it touches the
same call sites S3 would otherwise touch.

**S2-3. Node layout.** Move node values into `State` structs inside
`PropertyCell`; accessors take `&PropertyAccess`; `Cell` change flags and cc
ids replace `PaintPropertyNodeIdentity` atomics; identity = `Arc::ptr_eq`;
`PartialEq` derives on nodes become identity (today they compare values +
revision); per-sequence roots (3.4); typed caches + per-sequence generations
(3.5). Test constructors become `Node::Create(&mut w, parent, state)`
(Chromium `Create`), replacing the 81 struct literals. Delete
`PaintPropertyNodeIdentity`, `PaintPropertyNodeLifecycle.revision`,
`.parent_id`, per-snapshot `changed`, `OpaquePaintCacheSlot`,
`CacheTreeScope`, both `S_GLOBAL_GENERATION`s. At this commit the store still
replaces slot `Arc`s (snapshot behaviour preserved by creating a new node for
each changed slot), so semantics are unchanged. Check: `cargo check`;
`paint_model` + `prepaint` unit tests.

**S2-4. In-place Update in PrePaint.** `PaintPropertyNodeStore` becomes
ObjectPaintProperties: `Update{Transform,Clip,Effect,Scroll}(key, parent,
state)` create-or-`Update` in place. The builder calls it directly where it
allocates today (`paint_property_tree_builder.rs:460`, `:1597`, `:1903`,
`:1933`, `:1967`, clip/effect builders), with the slot key it already
registers (`RegisterProperty`). The logical out-of-flow effect parent must be
known before the effect `Update` (Chromium walks containing-block order);
either compute it in the builder context (preferred) or defer only OOF effect
updates to the existing reparent step and call `Update` there once with the
final parent (D2). Delete: raw-node `cache.raw_*` restore
(`RestoreBuilderStates` property part), `PersistPropertyStates`,
`PropertyNodeUpdater`, `update_snapshot!`, `same_snapshot_references`,
`snapshot_stats`, `property_keys` / `prior_property_keys` id indirection,
`RebaseOwnEffect`, `GatherRetainedRawPropertyIds`, the id half of
`SameBuilderStructure`. Clean retained subtrees keep their node `Arc`s (no
rebind). The pass's maximum change type is the store's `changed`. Check:
`cargo check`; prepaint tests
`independent_transform_properties_keep_source_order_and_slots`,
`will_change_effects_retain_identity_nodes_and_additional_transform_reason`,
`sticky_translation_rounds_total_and_retains_local_remainder`; then the
DevTools matrix at the end of S2-6 (one build).

**S2-5. Direct updates in place.** `DirectlyUpdateCompositedAnimationProperties`
calls `DirectlyUpdateTransformAndOrigin` / `DirectlyUpdateOpacity` on the
slot node (`&mut PropertyWriter`); `PublishDirect*Snapshot` deleted; the
`DirectCompositedAnimationPropertyUpdate` carries `Arc`s to the same live
nodes. Retained scroll: `TryUpdateScrollPropertiesWithPrePaint` updates the
ScrollTranslation / StickyTranslation slot nodes in place (Chromium
`UpdateScrollTranslation` + `UpdateStickyTranslation`), stops rewriting chunk
states and viewport metadata (same `Arc`s), and captures the *recorded*
matrices it compares against (`rebase_rect` callers, `scroll_paint_update.rs:1060-1140`)
as values at capture time instead of reading old snapshots. Delete
`retained_scroll_update`, `staged_snapshots`, `RetainedScrollSnapshotRebinder`,
`cleared_change_markers`, `CommitDirectScrollPropertySnapshots`,
`RebindResidentPropertySnapshots`, `PaintPropertyTrees::clear_state_change_markers`.
Check: `cargo check`; `retained_scroll_update_tests` rewritten against
in-place updates.

**S2-6. Paint / Layerize read live nodes.** Delete
`ArtifactPropertyStateRebinder` and its call (`paint_engine.rs:880-912`),
`PublishedTrees` / `chunk.property_trees` / `artifact.property_trees` and
their assignments (`paint_engine.rs:694`, `913-922`, `1447-1463`),
`FrozenPropertyTrees` usage in `resident_hit_test.rs:355`,
`resident_layer_clipper.rs:72`, `interaction_hit_test/src/resident.rs:1036-1060`.
Layerize handles drop the `trees` field and `resolve_link!`; `link_*` become
plain `Parent()` / state reads through the PAC context's `&PropertyAccess`.
Replace the property-change heuristic (`paint_services.rs:941-957`) with the
typed PrePaint change value. Check: `cargo check`; then one
`--profile iteration` build of `rechrom_app` and the full runtime matrix
(section 6).

**S2-7. (optional, D4) SVG record values.** `DocumentPaintArtifactRecord`
carries ops/resources only; remove nested artifact property walks.

### 4.1 What S3 then deletes (inventory)

S2 already deletes most of the snapshot machinery as each step makes it dead.
S3 removes what survives only as dead code or compatibility shims:

- `FrozenPropertyTrees`, `NodesById`, `FrozenTreeCoverage`, `verify_state`,
  `PaintPropertyNodeStore::{published_trees, invalidate_published, freeze,
  *_by_id, unresolved_parent_ids}`, `published` field.
- `PropertyNodeResolver` trait and all impls (2384-2541), `LinkedProperties`,
  every `*In(resolver, ..)` variant (`ClearChangedToRootIn`,
  `SourceToDestinationProjectionIn`, `SourceToDestinationRectIn`,
  `LocalToAncestorClipRectIn`, `TransformToRoot/Ancestor<R>`).
- `NativeTrees`, `from_arc_in`, `from_state_in`, `Trees()` in
  `geometry_mapper_source/{native,traits}.rs`; `HandleTrees`, `resolve_link!`,
  `link_same/transform/clip/effect/scroll`, `from_source_in` in
  `layerize/src/runtime/paint.rs` and `paint/*`.
- `PaintPropertyTrees` dedup directory (1998-2157) incl. `*_for`,
  `*_snapshots`, `from_states`, `insert_*`.
- `PersistedPropertyNode`, `KeyedPropertyNodes.ids`, `key_of_id`, `by_id`.
- `is_scroll_only_update_from` (3459), replaced by the typed change value.
- `paint_model/src/geometry_mapper.rs` non-source mapper if nothing reads it
  after the resolver parameter is gone (it duplicates the source GeometryMapper;
  verify callers then delete or reduce to the matrix helpers).
- Trace instants `PrePaint.ParentIds`, `PrePaint.Snapshots`,
  `PaintCommit.RebindPropertyStates`, `frozen_*` counters,
  `ScrollPaint.RebindPropertyTrees`.
- Comments that justify snapshots (`paint_property_tree.rs:1-6`, `52-55`,
  `2207-2212`, `2822-2828`, `3254-3258`, `3613-3623`;
  `geometry_mapper_transform_cache.rs:94-101`; `opaque_paint_cache_slot.rs`).
- `crates/paint/chromium_translation.json` entries pointing at deleted items.

## 5. Risks

1. **Stale reads by previous-artifact comparisons.** Any code that keeps an
   old artifact or old node to read *old values* silently starts reading new
   values. Known sites: retained scroll coverage (`scroll_paint_update.rs`
   recorded vs current matrices, `RetainedScrollPaintState::capture`), the
   scrollbar input comparison (`paint_services.rs:918-926`, display-item
   payloads, not nodes: safe), PAC `same_nodes` (identity: safe), raster
   invalidator (captured `PaintChunkInfo`: safe). Mitigation: S2-5 captures
   values; during S2-3 grep every `old`/`previous`/`recorded` binding that
   reaches a node value accessor and classify it as identity, change flag or
   value.
2. **Raster invalidation correctness.** Chromium computes
   `ChunkPropertiesChanged` from captured old geometry plus live change flags
   on the new state. Today Rechrom sometimes reads the old snapshot's flags.
   With one flag per node the PAC sequence protocol
   (`ClearChangedToRoot` at the end of `Update`) is the only clearer; a
   PrePaint pass that runs without a following PAC update (PrePaint-only walk
   for hit testing, aborted main frame) leaves flags raised exactly like
   Chromium. Verify under-invalidation with the logic matrix (incremental vs
   fresh) and the scroller round trip.
3. **Scroll offset fast path.** `DirectlyUpdateScrollOffsetTransform` +
   `CompositorSimpleValuesUpdated` must run before PAC classification on the
   live node and must not run for a node whose change was already folded into
   a full update. Today it is keyed on the snapshot's `node_changed()` of the
   *new* artifact's viewport list; after S2 the same list holds live nodes.
   Risk: a scroll that also changed another property in the same frame;
   covered by the DPR 2 trackpad replay with rollback (`REPLAY_ROUNDTRIP=1`)
   and the sticky page.
4. **Identity of an unchanged artifact.** Property-only updates no longer
   produce a new artifact `Arc` (today the scroll path mutates the artifact
   and rebinds). Page / LayerizeEngine admission keyed on `Arc::ptr_eq`
   (e.g. `page_runtime.rs:1222-1228`, removed or moved by S1b-7) must accept
   "same artifact, properties changed" through the typed change value, or
   composited scroll on the main path stalls.
5. **Logical OOF effect parents** (S2-4). Updating an effect with its
   physical parent first and then its logical parent reports a spurious
   change every pass (the reason `PersistPropertyStates` runs post-walk,
   `pre_paint_tree_walk.rs:4441-4444`). The effect `Update` must happen once,
   with the final parent.
6. **Borrow-checker friction in the builder.** Reads of parent state cannot
   be held across `Update`; builder code must copy the values it needs first.
   Expected local churn, not a design blocker.
7. **Handle plumbing in translated PAC code.** Adapter traits
   (`adapter/paint_artifact_compositor.rs:90`, `:110`, `adapter/pending_layer.rs`)
   call node methods without a context. If threading `&PropertyAccess` through
   `B::` traits is too invasive, handle methods may call `with_access` per
   call (one TLS access + `RefCell` borrow), measured by Layerize p50 (D1).
8. **Performance.** Expected improvement (3.3), but the new per-access
   compare and loss of the "reuse unchanged snapshot" fast path must be
   measured: PrePaint p50 and Layerize p50 on the matrix pages and
   `resize_fps`; regression budget 2 % or the hot readers switch to hoisted
   proofs.
9. **Multiple Pages per process.** Per-sequence roots and generations change
   behaviour for chrome + content + popups in one process; verify a
   two-Page session (browser chrome + content) in the matrix.

## 6. Runtime verification (after S2-6, one iteration build)

All through one headless `target/iteration/rechrom` process via DevTools.

1. DevTools correctness matrix: the 10 matrix pages + sticky + scroller,
   `tools/pixel_diff.py` against Chromium and against the pre-S2 baseline
   screenshots: `changed_pixels: 0` where the baseline had 0; logic
   incremental vs fresh unchanged from main.
2. Interaction check (`interact`): hit testing, hover, click, focus.
3. Trackpad replay at `--force-device-scale-factor 2` of
   `apps/rechrom/fixtures/trackpad_scroll.jsonl` (scratchpad
   `replay_trackpad.ps1`): no panics (in particular no `PropertyCell` owner
   assertion), deterministic final offsets, `REPLAY_ROUNDTRIP=1` scroller
   round trip 0 px.
4. `resize_fps`: p50 not worse than main (101.6 ms at `7f978395`).
5. Traces: PrePaint p50 and Layerize p50 (`prepaint`,
   `compositor.layerize` categories) vs main; `Layerize.DirectScrollOffsetUpdate`
   still reports updates during scroll; no `PAC under-invalidation` dumps.
6. Composited animation pages (transform and opacity): direct updates
   applied (`Compositor.Layerize.DirectAnimationPropertyUpdate applied=1`).
7. `paint_model` published-tree test replaced by the `property_cell` tests;
   `source_layout`, `dependency_boundary`, `layerize_boundary` guards pass.

## 7. Open decisions

- **D1. Proof plumbing in Layerize.** Thread `&PropertyAccess` explicitly
  through PAC adapter traits (zero cost, larger diff) or let layerize handles
  use `with_access` per call (small diff, TLS cost). Recommendation: explicit
  in GeometryMapper and PrePaint; per-call in layerize handles first, then
  explicit where Layerize p50 shows it.
- **D2. Logical OOF effect parent.** Compute it in the builder context
  (Chromium-shaped, larger PrePaint change) or keep a single deferred
  `Update` in the reparent step (smaller, keeps a Rechrom-only phase).
- **D3. `DirectlyUpdateScrollOffsetTransform` location.** Keep in
  `LayerizeEngine::UpdatePaintArtifact` (no Paint -> Layerize dependency) or
  move into the builder as Chromium does via a narrow injected trait on the
  Paint input (`paint_property_tree_builder.cc:3798-3827`, `1161-1194`).
- **D4. SVG `DocumentPaintArtifactRecord`.** Keep the whole nested
  `PaintRecording` (nodes stay reachable from raster threads, sound but
  unusual) or publish an ops-only record like Chromium's `cc::PaintRecord`
  (S2-7).
- **D5. Aliases.** Keep Rechrom unaliased (status quo) or translate
  `*PaintPropertyNodeAlias` while the node layer is open anyway.
- **D6. Commit granularity.** S2-2 and S2-3 are each large mechanical
  commits; acceptable as single commits, or fold S2-2 into S2-3 to avoid
  threading an unused proof for one commit.

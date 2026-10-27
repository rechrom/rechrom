# Layout package map

Layout is being separated by lifecycle responsibility instead of mirroring the
old `layoutng_*` source-package names at the repository boundary.

- `layout_invalidation` owns stable invalidation vocabulary and tree-agnostic
  propagation policy. The core assembly implements a narrow adapter for its
  private `LayoutObject` identity; container-chain, intrinsic-width, anchor,
  inline-collection, multicol, subtree-listener, spanner/out-of-flow parent,
  layout-caused paint, and paint-only ancestor walks execute from the
  invalidation package. Its assembly-only native tree-update module also owns
  subtree collection/preparation, full update begin/finish, object retirement,
  and failed-transaction cleanup. The cached resident-tree facts scan is
  likewise an out-of-line `layout_invalidation_native_tree_facts` traversal,
  exposed as `layout_invalidation.CollectTreeFacts` rather than core engine
  self time. Native `LayoutObject` invalidation adapters, dirty-bit mutation,
  subtree-listener entry points, container-chain propagation,
  inline-formatting-context lifecycle, and geometry/paint invalidation methods are physically in
  `native_layout_tree_invalidation.rs` and
  `native_geometry_invalidation.rs`; core keeps only compatibility facades.
  The hot `SetNeedsLayout*`, `ClearNeedsLayout*`, and intrinsic-width dirtying
  composition lives in `native_layout_object_dirty_state.rs`; these small
  per-object operations stay inlineable and delegate only the actual ancestor
  propagation to the out-of-line invalidation traversal frames.
  Generic LayoutObject topology—including generated/enclosing-node resolution,
  root-view discovery, descendant checks, common-ancestor search, root
  attachment checks, and cached preorder ordering—is physically owned by
  `native_layout_object_tree.rs`.
  Tree-sized queries retain invalidation sampling frames, while single-step
  preorder iteration stays inlineable.
  The native child-list record and its insertion, removal, destruction, inline
  item invalidation, and listener-propagation policy live in
  `native_layout_object_child_list.rs`. Tree-sized mutation paths retain
  invalidation sampling frames, while first/last-child access stays inlineable.
  LayoutObject-level add/remove dispatch, factory-aware base insertion, text
  normalization, and adjacent box-model merging live in
  `native_layout_object_child_mutation.rs`. Derived and base add/remove stages
  retain separate invalidation frames for structural churn attribution.
  Box-model child moves, anonymous-branch splitting, and recursive anonymous
  box merging are similarly owned by
  `native_layout_box_model_tree_mutation.rs`; fixed-cost padding and width
  queries remain in the core compatibility module instead of being mislabeled
  as invalidation work.
  Overflow-recalc container propagation and anonymous-child style propagation
  are split out of the generic hot-object module into
  `native_layout_object_hot_invalidation.rs`. Their ancestor/subtree scans keep
  invalidation sampling frames, while first-line style and view access remain
  ordinary inlineable core paths.
  Editing and selection-triggered invalidation are similarly owned by
  `native_input_invalidation.rs`; selection reports root, visited, and text
  leaf counts once per operation rather than emitting per-object events.
  Positioned-object removal and OOF fragment scanning live in
  `native_positioned_object_invalidation.rs`; affected objects and their
  absolute-position containing block are dirtied beneath one invalidation
  sampling frame instead of `LayoutBlock` self time.
  LayoutBlock descendant insertion routing and anonymous-container ancestry
  discovery live in `native_layout_block_child_mutation.rs`, keeping structural
  mutation traversal in the same invalidation performance domain.
  The targeted `NodeInput` transaction's validation, inherited-text expansion,
  and mutation batch live in `native_node_input_update.rs`, preserving the
  existing operation-level trace while adding an invalidation sampling frame.
  CSS zoom keeps its unit-zoom borrowed fast path in the compatibility facade;
  non-unit style cloning plus grid, shadow, and background-layer scaling live
  in `native_css_zoom.rs` under one invalidation sampling frame.
  Constraint-space equality—including provider identity, font catalog, image
  geometry, and shape-image revisions—is owned by
  `native_constraint_equivalence.rs`; the complete admission check remains an
  out-of-line invalidation sampling entry while the engine records only one
  operation-level summary.
  Engine mutation admission and dispatch—including tree/subtree transactions,
  node and paint input updates, scroll offsets, editing invalidation, and the
  retained-fragment invalidation handoff—live in
  `native_engine_mutation.rs`. `LayoutEngine` keeps source-compatible public
  forwarding methods, while the expensive decision tree retains an
  invalidation-owned sampling frame.
  Native tree preparation is owned by `native_prepare_tree.rs`; its out-of-line
  sampling frame is split into relation finalization, font rebinding,
  shape/image refresh, and inline-fragment preparation trace phases so a slow
  `PrepareTree` operation can be assigned to the specific invalidation class.
  Both the targeted and full resident-tree font-rebinding walks are physically
  in that owner as out-of-line symbols, so their self time does not fall back
  into the generic native boundary. The complete fixed-field native style
  conversion body is physically owned by `native_input.rs`, and its per-node
  transaction plus root constraint-space projection live in
  `native_input_projection.rs`. The former core boundary is now a re-export-only
  facade. StyleEngine identity reuse remains an inline fast path; actual style
  reconstruction and root constraint construction have out-of-line
  invalidation frames containing the existing algorithm/reuse/fragment
  subframes, so profiles show both the outer input cost and the expensive
  specialized step.
  Variable-size input equivalence is owned by the same package:
  `native_computed_style_equivalence.rs` compares generated native-style
  invalidation bits, shared extended style, and layout fields, while
  `native_element_data_equivalence.rs` compares frame, SVG, custom-layout, and
  form-control metadata. Their inline trait/inherent methods forward to
  package-qualified, out-of-line invalidation sampling functions (Rust method
  symbols otherwise omit the impl's source module). The shared
  `layout_input.rs` file remains the
  schema and implementation loader, so regressions in retained-input admission
  resolve to `layout_invalidation` instead of generic core self time.
  Managed-input descriptor admission and geometry-metadata normalization live
  in `native_managed_input_admission.rs`; metadata cloning, non-style equality,
  and native-style root retention keep distinct invalidation sampling frames,
  while the fixed-size form-control paint-state check remains inlineable.
  The complete per-node retained-object admission transaction is physically in
  `native_managed_object_update.rs`: unchanged reuse, compatibility fallback,
  style preparation, metadata refresh/allocation, object creation, placement,
  and retained-record publication execute beneath one package-qualified
  invalidation frame while preserving the nested reuse, algorithm, and
  fragment-owner frames.
  Initial retained node-input construction and subsequent descriptor replacement
  are implemented by `native_node_input_state.rs`; cloning author-sized element
  metadata and debug text therefore appears under invalidation-qualified frames
  rather than `StandaloneNodeData` or `Node` core methods.
  Native `LayoutObject` factory selection, element dispatch, and block/list allocation live in
  `native_layout_object_factory.rs`, giving text, SVG, form, table, flex/grid
  and custom object creation invalidation-owned sampling entries inside the
  tree-update transaction. Tree preparation invokes initial-letter cap-height
  fitting through `layout_algo_inline/native_initial_letter_style.rs`; the
  traversal remains invalidation work while the typography policy and its
  decreasing-size font-resolution loop remain part of the inline algorithm.
  Incremental attachment placement and sibling scanning now live with the rest
  of the tree transaction in `native_tree_update.rs`, so object reordering and
  metadata relation attachment are charged to invalidation rather than the
  builder facade. Batched reattachment-id admission is routed through the same
  owner, preventing subtree-sized set extension from appearing as a core
  `LayoutObjectTree` method.
  Transform-change admission that scans retained fragments for propagated
  anchors lives in `native_transform_invalidation.rs`; style invalidation can
  therefore be distinguished from fragment geometry projection in sampling
  profiles.
  LayoutBox retirement disassociates inline items, the measure cache, and all
  physical fragments from `native_layout_box_fragment_retirement.rs`, keeping
  destruction cleanup under an invalidation-owned sampling frame.
  LayoutView anonymous-page teardown is owned by
  `native_layout_view_destruction.rs`; its object-count-sized destruction walk
  retains an invalidation sampling frame rather than appearing as root-view
  self time.
  LayoutView resize-result tracking, annotation-driven relayout, base style
  refresh/diff dispatch, counter-style invalidation, and descendant paint
  invalidation are colocated in `native_layout_view_invalidation.rs`. The
  propagation/provider entries remain out of line, so root-level style and
  explicit invalidation regressions resolve to `layout_invalidation` rather
  than the core view type.
  Root child admission and insertion are owned by
  `native_layout_view_child_mutation.rs`; the potentially structural base
  insertion retains an invalidation frame while fixed-cost admission checks
  remain inlineable.
  Generic visual-overflow, paint, and display-item client invalidation dispatch
  lives in `native_layout_object_paint_invalidation.rs`. Propagating provider
  entries and anonymous-wrapper cleanup retain invalidation sampling frames;
  the empty base line-dirty hook remains inlineable.
  Hit-test paint flags, selection invalidation, prepaint subtree propagation,
  touch/wheel interaction state, soft-navigation and container-timing marks,
  scroll-anchor cleanup, SVG transform descendants, reinsert handling, and
  selection-child style invalidation plus self-painting-layer overflow marking
  are grouped in
  `native_layout_object_prepaint_invalidation.rs`. Propagation/provider stages
  retain separate invalidation frames. Delayed paint state, container-timing
  inheritance, effective touch-action policy, and flag clearing are colocated
  there; direct bit work and simple predicates/setters remain inlineable.
  Layout, prepaint, and paint display-lock gates plus context lookup are unified
  in `native_layout_object_display_lock.rs`. These shared fixed-cost admissions
  remain inlineable across layout, reuse, prepaint, and paint callers.
  PaintLayer tree insertion, removal, moves, and insertion-point search live in
  `native_layout_object_layer_mutation.rs`, with distinct invalidation frames
  so structural layer churn is separated from layer lookup cost.
  LayoutBox style-change admission and dispatch are owned by
  `native_layout_box_style_change.rs`: position-chain invalidation, grid
  placement, overflow clips, inline recollection, multicol spanners, and
  transform-triggered relayout remain distinguishable from the lifecycle
  facade.
  PaintLayer creation, refresh, and destruction triggered by those style
  changes live in `native_layout_box_model_layer_invalidation.rs`, with
  separate invalidation sampling frames; fixed-cost layer-type classification
  remains on the core typed record.
  The enclosing `LayoutBoxModelObject` lifecycle transaction—destruction,
  base style dispatch, transform/layer refresh, and anchor-position scroll-data
  admission—lives in `native_layout_box_model_lifecycle.rs`. Destruction and
  completed style updates retain invalidation-owned sampling frames.
  Style-derived LayoutObject/LayoutBoxModelObject/LayoutBox flags—including
  positioned-container admission, inline/floating classification, transform
  state, overflow clipping, and scrollable-area refresh—are physically owned
  by `native_layout_node_style.rs`. The two per-object `UpdateFromStyle`
  stages retain distinct invalidation sampling frames.
  Derived block style-change stages are split into
  `native_layout_block_style_change.rs` and
  `native_layout_block_flow_style_change.rs`; SVG text scaling, positioned
  object removal, anonymous-child propagation, multicol refresh, column
  pseudo changes, and inline reshape callbacks remain invalidation-owned.
  The multicol refresh implementation itself lives in
  `native_layout_block_flow_multicol_update.rs`, including form-control
  admission, descendant flag propagation, and inline/block child conversion,
  so its subtree walk is visible as invalidation work rather than core time.
  The base `LayoutObject` style transaction and complete transform-difference
  classifier are owned by `native_layout_object_style_update.rs`; mask-image
  observer updates, derived style-difference adjustment, pseudo-element and
  first-line application, anonymous-child style dispatch/override policy, and
  will/did-change dispatch remain inside the same invalidation sampling domain.
  Element refresh, first-line style lookup,
  cached/uncached pseudo-style services, and explicit mutation outside style
  recalc are owned by `native_layout_object_style_services.rs`. Refresh,
  uncached resolution, and explicit mutation retain separate invalidation
  sampling frames while cached lookups remain inlineable. The public style and
  pseudo-style transaction entry points also live in the invalidation owner;
  pseudo inheritance and the nested style transaction remain separately
  attributable in profiles.
  Base and box-level tree lifecycle policy is physically split into
  `native_layout_object_tree_lifecycle.rs` and
  `native_layout_box_tree_lifecycle.rs`: inline dirtying, implicit-anchor
  propagation, leftover-child destruction, shape/fragment retirement, and
  custom-child attachment all remain invalidation-owned. Object-level virtual
  dispatch and base work retain separate frames so profiles distinguish derived
  lifecycle overhead from invalidation propagation.
  Tree-depth recomputation, descendant depth propagation, scroll-anchor style
  propagation and priority notification, plus paint-property ancestor marking,
  live in
  `native_layout_object_state_propagation.rs`. Mask fill-layer comparison and
  the complete image-observer subscription/update surface live in
  `native_layout_object_image_observers.rs`; mask, fill, cursor, shape, and
  first-line update paths keep their work under invalidation sampling frames.
  The pre-paint subtree-walk reason bitset is physically owned by
  `native_pre_paint_subtree_walk_reasons.rs`; its constant-time bit operations
  stay inlineable beneath the invalidation entry frames.
  Default trigger scope remains an inline no-op; non-default scope validation,
  atom/list construction, and name iteration are owned by
  `native_trigger_scope_projection.rs` and sampled as invalidation work.
  Shared anchor/trigger `NamingScope` identity and its parent-element plus
  scoped-name lookup live in `native_naming_scope.rs`; the complete lookup uses
  one invalidation frame without per-ancestor trace events.
  The fragment-retained trigger scoped-name alias and named-trigger map live in
  `layout_fragment/src/native_trigger_scoped_name.rs`; its wrapper stays
  inlineable so ancestor lookup is still charged to the invalidation-owned
  `NamingScope` frame.
  Empty timeline-trigger metadata remains an inline no-op; nonempty name
  validation and scoped-name allocation live in
  `native_timeline_trigger_metadata.rs` under an invalidation sampling frame.
  Element metadata construction and incremental input refresh—including
  form/SVG/replaced/canvas/scroll validation and derived-state updates—live in
  `native_element_metadata_input.rs` as separate invalidation sampling entries.
  Metadata-tree append admission, ancestor-cycle detection, detach, insertion,
  and link mutation are owned by `native_metadata_append.rs`, keeping structural
  mutation samples out of the metadata value facade.
  Whole-tree node-metadata finalization—including preorder collection, viewport
  propagation, implicit-anchor resolution, frameset/list-marker validation, and
  input sealing—is owned by `native_node_metadata_finalize.rs`; its multi-pass
  transaction has one invalidation-qualified sampling frame.
  Per-node control-host ancestry and generated form-control relationship
  admission are owned by `native_node_metadata_incremental_relations.rs`, so
  incremental relation walks no longer accrue to the metadata facade.
  The general parent-element ancestry lookup is physically isolated in
  `native_node_metadata_tree.rs` with an invalidation sampling frame, leaving
  scalar node metadata accessors in the core record.
  Anonymous block-flow normalization is owned by
  `native_layout_block_flow_child_normalization.rs`: inline-run discovery,
  anonymous-wrapper collapse/creation, child-mode conversion, and changed-line
  dirtying retain invalidation-owned sampling frames instead of appearing as
  `LayoutBlockFlow` self time.
  Its sibling `native_layout_block_flow_child_mutation.rs` owns insertion and
  removal routing, anonymous-block merge admission, and floating/out-of-flow
  reparenting. This keeps structural mutation walks attributable to the same
  invalidation package while leaving source-compatible methods in core.
- `layout_reuse` owns caches and the size-based and
  block-formatting-context cache decision trees. Native
  geometry is supplied through core fact adapters rather than pulling
  `LayoutObject`, `ConstraintSpace`, or fragment implementation types into the
  standalone package. The complete native `CachedLayoutResult` decision tree
  is physically owned by `layout_reuse/src/native_cache_lookup.rs` and compiled
  only at the reciprocal assembly boundary; `LayoutBox` retains a thin entry
  method for source compatibility. Reusable font-environment snapshot matching,
  cache transfer, and per-pass resolver binding are physically owned by
  `native_layout_environment_cache.rs`; one low-cardinality
  `layout_reuse.FontEnvironment` phase records reuse versus rebuild, while the
  matching and binding entries stay visible to sampling profilers. Deferred,
  shared-engine, and newly-created resolver selection remains in
  `native_font_resolver.rs`. The request schema, resolver interface, direct
  `FontEngine` adapter, and selector-backed family projection are shared layout
  boundary services; cache reuse does not own general font resolution.
  Per-pass resolver TLS installation and style request projection live with
  native input preparation in `layout_invalidation`.
  Resident `LayoutEngine` output admission is owned by
  `native_engine_output_reuse.rs`; fragment validity, pending scroll/paint/
  inline/subtree work, output kind, and native dirty state form one explicit
  out-of-line reuse predicate rather than core lifecycle self time.
  Per-box measure-result admission, measure-cache invalidation, cached fragment
  lookup, and measure lookup are owned by
  `native_layout_box_result_cache.rs`; the transition into publication is an
  explicit call to the commit owner instead of one mixed `LayoutBox` routine.
  Native size-based admission, geometry validation, and block-formatting-
  context reuse adaptation are physically owned by
  `native_layout_utils.rs`; their two public decisions retain reuse sampling
  frames. Shared content-alignment and static-position policy stays in
  `layout_alignment_utils.rs` and is inlined into its block, table, inline, or
  out-of-flow caller. `MinMaxSizesCache` owns cached entries, while the
  `MinMaxSizes` value and arithmetic live in `layoutng_geometry`.
  The managed-input lookup index keeps a branch-only inline hit path, while its
  first full map scan and incremental key removal/insertion are owned by
  `native_managed_input_index.rs`; cache-hit overhead stays minimal, while
  cache-build and mutation samples resolve to `layout_reuse`.
  Existing-object class/style compatibility, including author-sized timeline
  trigger comparison and native reinsertion admission, lives in
  `native_managed_object_compatibility.rs`; inline method shims lead to distinct
  reuse-qualified object-kind, complete-update, and targeted-style frames.
  Native style reconstruction keeps its object-identity fast return in the
  facade. Font-family validation and request construction remain part of the
  input projection transaction in
  `layout_invalidation/native_style_font_resolution.rs`; the resolver delegates
  selection and caching to FontEngine.
  Paint-only native-style replacement preserves the previously resolved font
  through `native_paint_only_font_binding.rs`, so the occasional style clone is
  charged to reuse rather than generic tree-builder self time.
  LayoutObject regular/first-line fallback validity admission is likewise owned
  by `native_layout_object_font_validity.rs`, giving stale-font cache misses a
  reuse-qualified sampling frame without per-object trace events.
- `layout_algo_*` owns concrete formatting and replaced-content algorithms.
  Shared page/column breaking policy has its own
  `layout_algo_fragmentation` owner rather than accumulating under generic
  core. Fragment-builder setup, finalization, child-break selection,
  breakpoint advancement, soft-break attempts, fragmentainer constraint-space
  construction, and stitched-fragment offset projection retain coarse
  out-of-line sampling frames. The source-compatible core module is a
  re-export-only facade, and the owner adds no per-child trace events.
  `BreakAppeal`, `EarlyBreak`, and their target state are colocated with this
  policy, so fragmentation control data no longer appears core-owned. The
  allocating `ConstraintSpace::CloneWithoutFragmentation` projection also
  forwards to a package-qualified fragmentation sampling function.
  Cross-format intrinsic and used-size policy similarly belongs to
  `layout_algo_sizing`. Min/max block and inline resolution, fragment inline/
  block sizing, replaced-size resolution, initial fragment geometry, and
  content contribution calculation retain sizing-owned sampling frames. This
  separates regressions in common CSS sizing from the concrete block, flex,
  grid, table, form, float, or out-of-flow caller without emitting a trace for
  every box. Orthogonal fallback constraint sizing has its own coarse sizing
  sampling frame, and the min/max constraint-space builder is colocated with
  that policy. Baseline-mode/group helpers are forced inline into flex/grid
  callers. `length_utils`, `space_utils`, `baseline_utils`, and the min/max
  builder's legacy core module remain only compatibility facades.
  `ConstraintSpace` itself remains a shared core schema: its fixed-cost clone
  body is forced inline, while exclusion-state cloning forwards to a
  package-qualified float-owner frame and the block-in-inline text-trim clone
  keeps an inline fast path plus an out-of-line, package-qualified
  `layout_algo_inline` slow-path frame.
  Inline collapsible-whitespace predecessor lookup, including floating/out-of-flow
  sibling skipping and transformed-text tail inspection, is physically owned by
  `layout_algo_inline/src/native_collapsible_whitespace.rs`; its linear sibling
  walk retains an inline-algorithm sampling frame instead of generic tree self time.
  Inline items, shaping segments, per-item results, item spans/text indices,
  `InlineNode` state and behavior, and text-combine layout are physically owned by
  `layout_algo_inline/src/native_inline_*.rs` and
  `native_layout_text_combine.rs`. The old core paths are re-export-only facades
  (or a loader for the `InlineNode` impl), while expensive shaping, segmentation,
  and layout calls continue through the inline package providers; profiles can
  therefore distinguish inline work from shared LayoutNG infrastructure.
  `HyphenResult` is colocated in `native_hyphen_result.rs`, and its HarfBuzz
  shaping provider retains an out-of-line inline-package sampling frame without
  adding a high-frequency trace event.
  Used-font metrics, text-fit scale state, and text-item classification are
  similarly colocated in `native_used_font.rs`, `native_text_fit_scale.rs`, and
  `native_text_item_type.rs`; fragments retain their values through the boundary
  facade while metric computation remains attributable to the inline package.
  The `LayoutInline` and `LayoutText` typed interfaces now live in
  `native_layout_inline.rs` and `native_layout_text.rs`. Their lifecycle and
  geometry calls still delegate to the specialized providers—including the
  separately sampled fragment client-geometry owner—while core retains only
  stable re-export paths.
  Variable-length transform mappings and the small `LayoutText` core impl are
  colocated in `native_variable_length_transform_result.rs` and
  `native_layout_text_core.rs`, completing the typed text boundary without
  moving shared editing or selection state into the algorithm package. The
  LayoutView index that registers those per-text results is owned by
  `native_layout_view_variable_length_transform_index.rs`; mutation entries
  retain inline-algorithm sampling frames while lookup remains inlineable.
  Generic `LayoutObject` editing-position queries—including point-to-position,
  affinity construction, before/after traversal, and DOM-node/object
  association—are owned by `native_layout_object_editing_queries.rs`. Provider
  entries retain inline-algorithm frames, while selection-leaf predicates and
  the default association wrapper remain inlineable.
  Per-object construction determines page versus column fragmentation through
  `layout_algo_block/src/native_fragmentation_context.rs`; its containing-chain
  scan has a block-algorithm sampling frame instead of builder self time.
  Block sibling inline-run skipping, first layout-child selection, and fieldset
  legend discovery are owned by
  `layout_algo_block/src/native_block_node_traversal.rs`; each traversal retains
  its own block-algorithm sampling frame.
  The `BlockNode` algorithm interface and callback dispatch are physically
  colocated in `layout_algo_block/src/native_block_node.rs`; the generic core
  path is now a re-export-only facade, while shared `LayoutInputNode` and
  `ConstraintSpace` abstractions remain outside any concrete algorithm package.
  LayoutObject containing-block, absolute/fixed container, column-spanner, and
  fragmentation-root queries are physically owned by
  `native_layout_object_containers.rs`. DOM scroll-parent and offset-parent
  resolution share that owner. The tree-sized scans retain block sampling
  frames while high-frequency default wrappers remain inlineable.
  Container-chain containment, enclosing-box lookup, and rendered-fieldset-
  legend scans no longer accumulate in generic `LayoutObject` core services.
  Root-view constraint-space construction, intrinsic minimum-width calculation,
  and the first `BlockNode::Layout` dispatch live in
  `native_layout_view_root_layout.rs`, so root block work enters profiles
  beneath block-algorithm sampling frames rather than `LayoutView` core time.
  Root-box lookup, paginated-layout admission, anonymous page-object creation,
  and named-page lookup live in `native_layout_view_block_queries.rs`.
  Allocation and page-index provider entries retain block-algorithm sampling
  frames while the fixed-cost root and pagination predicates remain inlineable.
  `LayoutBlock` and `LayoutBlockFlow` typed interfaces are likewise colocated in
  `native_layout_block.rs` and `native_layout_block_flow.rs`. Their style,
  child-mutation, positioned-object, and multicol invalidation scans remain
  delegated to `layout_invalidation`, preserving lifecycle attribution beneath
  the block-owned object boundary.
  LayoutBlock first-line sibling skipping and nested inner-block descent are
  owned by `layout_algo_block/src/native_layout_block_traversal.rs`; both scans
  retain block-algorithm sampling frames rather than LayoutBlock self time.
  Column-spanner traversal state and block-level list-marker placement state are
  physically owned by `native_column_spanner_path.rs` and
  `native_unpositioned_list_marker.rs`. Marker layout, fragment insertion, and
  float-intrusion computation retain separate block sampling frames; marker
  content creation and symbol measurement remain attributable to
  `layout_algo_list`. Generic summary/list-marker role predicates are
  physically owned by
  `layout_algo_list/src/native_layout_object_list_marker_queries.rs`; their
  fixed-cost provider dispatch remains inlineable beneath list work.
  Line-clamp state, ancestor-chain resolution, and final clamped block-size
  calculation are physically owned by
  `layout_algo_block/src/native_line_clamp_data.rs`; their small value queries
  remain inlineable through a re-export-only compatibility facade.
  Custom layout's serialized payload and constraint-space custom state are
  physically owned by `layout_algo_custom/src/native_custom_layout_payload.rs`
  and `native_constraint_space_custom_data.rs`. Byte copying retains a custom
  algorithm sampling frame, while pointer-identity reuse admission and state
  access remain inlineable.
  Grid track-list expansion, named-line normalization, auto-track projection,
  line-position conversion, and template-area construction are physically owned
  by `layout_algo_grid/src/native_grid_style_projection.rs`; input-sized work has
  Grid sampling frames while small value conversions remain inlineable.
  Authored track-list copying, owned-vector admission, and clearing are likewise
  implemented by `native_grid_track_list_input.rs`; public value-type methods
  inline into package-qualified Grid sampling functions, and the shared
  `layout_input.rs` retains only the input schema and owner-module loader.
  Grid layout data, baseline state, subgrid projection, and retained subtree
  comparison are physically owned by
  `layout_algo_grid/src/native_grid_layout_data.rs`; subtree-sized equality work
  retains a Grid sampling frame. Grid item/candidate state and track/range/set
  collections are physically owned by `native_grid_item_data.rs` and
  `native_grid_track_collection_types.rs`; their non-inline computation remains
  in the existing Grid provider modules while small accessors stay inlineable.
  Grid-lanes grouping and the retained subtree cursor are colocated in
  `native_grid_lanes_item_group.rs` and `native_grid_subtree.rs`.
  Table constraint-space rows, sections, cells, and reuse admission are owned by
  `layout_algo_table/src/native_table_constraint_space_data.rs`; row-cell and
  section-row scans retain distinct table sampling frames. Collapsed-border
  state and column locations are colocated in `native_table_borders_types.rs`
  and `native_table_column_location.rs`, with heavy border computation still
  routed through the table provider symbols. Cell/column/row/section state,
  grouped-child iteration, and the typed `TableNode` facade live in
  `native_table_layout_algorithm_types.rs` and `native_table_node.rs`.
  `shape-outside` contour, radius, inset, and reference-box validation is owned
  by `layout_algo_float/src/native_shape_style_projection.rs`; author-sized shape
  projection has a float-algorithm sampling frame rather than core input self time.
  ShapeOutsideInfo weak-map admission, cached geometry, writing-mode projection,
  and float-factory dispatch live in `native_shape_outside_info.rs`; map misses
  and shape cache misses retain separate float sampling frames.
  The complete exclusion-space implementation—including shelf maintenance,
  opportunity enumeration, derived-geometry cache rebuild, exclusion insertion,
  incremental update, and space merge—is physically owned by
  `layout_algo_float/src/native_exclusion_space.rs`. The legacy core path is a
  re-export-only facade, and all input-sized operations retain float-qualified
  sampling frames while the cache-hit path stays lightweight.
  `ExclusionArea`, its optional shape payload, allocation, initial-letter
  construction, offset-copy path, and identity equality are physically owned by
  `native_exclusion_area.rs`; allocation/copy entries remain out of line while
  the old core source is a re-export-only facade.
  The shape trait, line-segment value, raster `ShapeInterval` state, and
  per-line `ShapeExclusions` collections are likewise owned by
  `native_shape.rs`, `native_shape_interval.rs`, and
  `native_shape_exclusions.rs`; fixed-size shape math remains inlineable,
  while cloning an input-sized exclusion list retains a float sampling frame.
  Shape-aware `LayoutOpportunity` projection is colocated in
  `native_layout_opportunity.rs`; below-shape admission, left/right exclusion
  scans, and excluded-segment geometry retain distinct float sampling frames.
  Positioned and unpositioned float state are colocated in
  `native_positioned_float.rs` and `native_unpositioned_float.rs`, leaving their
  tiny accessors inlineable while exclusion search and placement stay visible in
  float-qualified frames.
  Flex diagnostic line/item geometry is physically owned by
  `layout_algo_flex/src/native_devtools_flex_info.rs`; exporting diagnostic data
  no longer makes flex-specific state appear as generic core ownership.
  SVG path commands, shape transforms, and textPath point validation are owned
  by `layout_algo_svg/src/native_svg_metadata_validation.rs`; ordinary element
  construction bypasses the owner while SVG geometry gets an SVG sampling frame.
  LayoutObject SVG provider dispatch and the default SVG geometry surface live
  in `native_layout_object_svg_dispatch.rs`; recursive update work enters through
  one SVG-qualified frame while constant/default virtual bodies stay inlineable.
  SVG character positioning, inline-node ranges, layout flags, length-adjust
  policy, and text-attribute build requests are physically colocated in
  `native_svg_character_data.rs`, `native_svg_inline_node_data.rs`,
  `native_svg_layout_info.rs`, `native_svg_length_adjust_type.rs`, and
  `native_svg_text_attributes_request.rs`. Their legacy core modules are
  re-export-only facades, so SVG state no longer appears to be generic core work.
  Resolved text-layout attribute iteration is colocated in
  `native_resolved_text_layout_attributes_iterator.rs`; ordinary same/current
  index access remains inlineable, while advancing through the resolved list
  retains an SVG sampling frame even when invoked by inline line breaking.
  The LayoutView-owned SVG text descendant map is mutated from
  `native_svg_text_descendant_index.rs`; add/remove operations retain SVG
  sampling frames rather than accumulating in root-view self time. The map
  accessor and non-scaling-stroke summary bit are colocated there as inlineable
  SVG-owned state.
  The typed `LayoutReplaced`/`LayoutInputReplaced` interface is likewise owned by
  `layout_algo_replaced/src/native_layout_replaced.rs`; heavy natural sizing and
  content-rect work remains routed through the replaced provider symbols and its
  existing package-qualified sampling entry.
  Multicol break-token state is physically owned by
  `layout_algo_multicol/src/native_multicol_break_token_data.rs`, keeping retained
  algorithm state adjacent to the code that creates and consumes it.
  Form-control classification, fixed-size policy, and HTML form-node metadata
  are physically owned by `layout_algo_forms/src/native_form_control_types.rs`,
  `native_form_control_sizing_service.rs`, and `native_form_node_metadata.rs`.
  Construction still enters through the traced `layout_algo_forms.CreateMetadata`
  callback, so metadata cost and later intrinsic sizing remain attributable to
  the forms package while legacy core paths stay re-export-only.
  Generated list-marker child lookup is owned by
  `layout_algo_list/src/native_list_marker_lookup.rs`, keeping its sibling scan
  visible as list-algorithm work. Root-level counter and list-item population
  counts live in `native_layout_view_list_index.rs`; their O(1) mutations stay
  inlineable while no longer appearing as generic LayoutView policy.
  Block/multicol column-spanner admission, including ancestor scanning and
  formatting-context barriers, is physically owned by
  `layout_algo_block/src/native_column_spanner_admission.rs`; its complete
  admission and ancestry walk remain distinct block-algorithm sampling frames.
  Out-of-flow anchor-map traversal, explicit target selection, and implicit
  anchor admission are physically owned by
  `layout_algo_out_of_flow/src/native_anchor_lookup.rs`; their out-of-line
  sampling frames keep anchor-positioning search cost out of generic
  `LayoutBox` core services.
  AnchorMap ordering, enumeration, acceptable-reference search, duplicate
  merge, and child-fragment projection are physically owned by
  `layout_algo_out_of_flow/src/native_anchor_map_operations.rs`; every
  anchor-count or fragment-count scan retains an out-of-flow sampling frame.
  `AnchorMap`, its keys, entries, and `PhysicalAnchorReference` are colocated in
  `native_anchor_map.rs`; transform-animation admission lives beside it in
  `native_anchor_map_services.rs`, leaving both former core paths as re-export
  facades.
  Anchor-position scroll state is likewise physically owned by
  `layout_algo_out_of_flow/src/native_anchor_position_scroll_data.rs`:
  scroll-container ancestry, fallback-range validation, chained-anchor
  invalidation, and dependent recursion retain separate out-of-flow sampling
  frames while the legacy core module is a re-export-only facade.
  Its native scroll-container id/offset/origin snapshot and transform-overscroll
  admission live in `native_anchor_scroll_services.rs`; these fixed-cost facts
  stay inlineable beneath the surrounding out-of-flow frames.
  Its `NonOverflowingScrollRange` fallback value is physically colocated in
  `native_non_overflowing_scroll_range.rs`, keeping anchor fallback state out of
  generic core modules without forcing a sampling frame around its tiny query.
  Physical/logical OOF candidates, inline and containing-block descriptors,
  fragmented descendants, pending multicol state, and coordinate conversion
  are physically owned by
  `layout_algo_out_of_flow/src/native_oof_positioned_node.rs`; the tiny value
  accessors remain inlineable instead of adding artificial profiling frames.
  Position-try fallback state is physically owned by
  `native_successful_position_fallback.rs`, while remembered/speculative
  anchor-scroll maps live in `native_out_of_flow_data.rs`; its fixed-cost state
  access remains inlineable, and fallback/pending-map equivalence checks retain
  separate out-of-flow sampling frames.
  Inline containing-block geometry, its traced map, and provider entry points
  are colocated in `native_inline_containing_block_utils.rs`, so future provider
  sampling and allocation cost resolve directly to the out-of-flow package.
- `layout_fragment` owns the native-result lifetime handle and fragment-local
  revision policy, plus engine-neutral retained-paint boundary values such as
  fragment identity/geometry, sticky and scroll geometry, inline client
  geometry, collapsed-table section geometry, scrollbar-axis client state,
  fieldset/column/table/scrollbar paint payloads, root-canvas background
  selection, resolved form-control paint
  state, immutable glyph runs, MathML/text-combine paint payloads, retained
  font/image resources, SVG text/viewport geometry, shared fragment-local
  geometry/revision storage, and PrePaint invalidation facts. The heavy paint
  property payload remains assembly-supplied but is retained through a generic
  fragment-owned shared handle. Shared edges, border styles, resolved appearance/control/font enums,
  font variations, host viewport geometry, linear paint colors, the three-value
  Layout/Paint writing mode, and the 4×4 boundary transform live in
  `layoutng_foundation`, so these values do not depend on the concrete Layout
  engine. Table and canvas payloads are parameterized by the assembly-owned
  immutable style snapshot, keeping their topology and geometry in the
  fragment owner without importing LayoutNG's style implementation. The
  canonical graph sources remain in this directory while their
  concrete LayoutNG type edges are progressively moved behind adapters.
  Frame-set row/column sizes, border admissions, and paint flags are physically
  owned by `native_frame_set_layout_data.rs`; cloning its variable-length
  vectors retains a fragment sampling frame rather than appearing as core
  value-copy self time.
  The retained `MathMLPaintInfo` payload is physically owned by
  `layout_fragment/src/native_mathml_paint_info.rs`; the old core path only
  re-exports it, separating retained paint memory from MathML algorithm time.
  Inline client-geometry enumeration, including culled-inline discovery,
  ruby-text exclusion, border-box projection, and per-container bounds, is
  physically owned by `native_inline_client_geometry.rs`; `LayoutInline`
  retains only a compatibility entry point, so this fragment walk is visible
  as fragment work in sampling profiles.
  Root viewport geometry is likewise physically owned by
  `native_layout_view_geometry.rs`: layout/viewport-unit sizes, scrollbar
  deduction and modes, printing/pagination viewport state, overflow clipping,
  fixed-position offsets, and retained background rectangles no longer accrue
  as generic `LayoutView` self time. Layout-size and overflow-clip projection
  retain distinct out-of-line fragment sampling frames; scalar state accessors
  remain inlineable.
  Root hit testing, hit-test cache clearing, visual-rect/quad projection, and
  local/ancestor coordinate mapping are physically owned by
  `native_layout_view_fragment_queries.rs`. Traversal-capable provider entries
  retain fragment sampling frames while the leaf result update remains
  inlineable, keeping query attribution without adding a frame per result.
  Hit-test counters and view-transition-root lookup are colocated with those
  queries. Fixed root paint/compositing defaults live in
  `native_layout_view_paint_policy.rs`; because they are O(1), they remain
  inlineable instead of manufacturing profiler noise.
  The same boundary now covers every `LayoutObject` through
  `native_layout_object_fragment_queries.rs`: all-phase and point hit testing,
  occlusion testing, visual-rect projection, property-container lookup, caret
  geometry dispatch, hit-test request admission, and local/ancestor mapping are
  no longer core-owned.
  Traversal-capable provider entries retain fragment frames; node/result leaf
  adapters, hit-test admission, and the explicitly optimized visual-rect fast
  path remain inlineable.
  Selection state policy and retained selection geometry are grouped in
  `native_layout_object_selection_queries.rs`. Absolute rectangle aggregation
  retains a fragment sampling frame while fixed-cost selection predicates stay
  inlineable, separating selection geometry cost from inline editing traversal.
  Container offsets and the complete ancestor/local/absolute rect, quad, point,
  and transform conversion family are physically owned by
  `native_layout_object_coordinate_conversion.rs`. Scroll-origin projection,
  the five provider-backed conversion operations, and direct ancestor-offset
  lookup retain fragment frames; container admission, overscroll defaults,
  type conversion, and null-ancestor convenience wrappers remain inlineable.
  Quad enumeration and the absolute bounding-box variants live in
  `native_layout_object_bounding_geometry.rs`; provider-backed aggregation,
  including layer rectangles, descendant bounds, and scroll-into-view geometry,
  has distinct fragment sampling frames.
  Scroll-container, overscroll-backdrop, and scroll-marker classification plus
  containing scroll box/layer lookup are owned by
  `native_layout_object_scroll_container_queries.rs`. Fixed policy remains
  inlineable, while the three axis/view-aware ancestry providers retain
  separate fragment sampling frames.
  Object paint dispatch, draggable-region/compositing queries, outline
  collection, canvas drawing-owner lookup/subtree policy, and debug rectangles
  are colocated in `native_layout_object_paint_dispatch.rs`. Per-object paint
  dispatch and recursive canvas predicates remain inlineable, while the three
  potentially multi-fragment outline collection entries and canvas drawing
  ancestry lookup retain fragment sampling frames.
  Background-attachment/compositing flags, obscured-background state, and image
  orientation policy live in `native_layout_object_paint_state.rs`. These are
  fixed-cost per-object values and remain inlineable instead of producing a
  misleading fragment frame for every paint candidate.
  Retained-fragment style-variant selection, visited color resolution, grouping,
  and clip predicates live in `native_layout_object_effective_style.rs`. The
  standard-style path remains inlineable; only the non-standard variant path,
  which may resolve a containing block, retains a fragment sampling frame.
  Generic stitched row-gap dispatch lives in
  `native_layout_object_gap_queries.rs` and remains inlineable. The concrete
  flex and grid break-token/gap-data paths retain separate algorithm-package
  frames, so stitched-gap cost is attributed to its formatting algorithm.
  Fragment-less-box classification plus generic inline-fragment presence and
  first-item-index bookkeeping live in
  `native_layout_object_inline_fragment_state.rs`. Its per-object base/dispatch
  surface remains inlineable; scans stay attributable to fragment items and
  cursors rather than a synthetic bookkeeping frame.
  Paint-effect predicates plus enclosing and painting-layer lookup are owned by
  `native_layout_object_layer_queries.rs`; the ancestor-capable lookups retain
  fragment frames while fixed-cost predicates and default wrappers stay
  inlineable.
  Multi-fragment bounding-box and ancestor-quad projection are similarly
  owned by `native_layout_box_fragment_geometry.rs`, with separate out-of-line
  entries for aggregate geometry and coordinate mapping. Inline-fragment
  resolved-direction lookup is owned by the same geometry module, with style
  direction as its explicit no-fragment fallback.
  Direct layout-result/physical-fragment access, cached-geometry validity,
  inline-fragment indices, and fragment-consistency checks are physically
  owned by `native_layout_box_fragment_access.rs`. These fixed-cost hot
  accessors remain inlineable; only their aggregate geometry consumers create
  fragment sampling frames.
  The same owner now contains previous break-token lookup, physical-edge
  selection, stitched block-size scanning, contracted border/padding geometry,
  multi-result size aggregation, and location-container ancestor lookup. These operations retain distinct
  out-of-line frames so fragmented-size regressions do not appear as generic
  `LayoutBox` self time.
  Layout-result scrollable-overflow aggregation—including writing-mode
  stitching, repeated-fragment termination, and padding containment—is owned
  by `native_layout_result_overflow.rs`, keeping this potentially linear scan
  out of `LayoutBox` self time.
  The physical-fragment list, index search, item-presence scan, and iterator
  advancement are physically owned by `native_physical_fragment_list.rs`;
  `LayoutBox` re-exports the compatibility types without owning their scans.
  Multi-fragment ink-overflow stitching is owned by
  `native_fragment_visual_overflow.rs`, including writing-mode offsets,
  repeated-fragment termination, and the single-fragment fast path. The full
  `LayoutBox` visual-overflow mutation, clipping, and query surface is
  colocated in `native_layout_box_visual_overflow.rs`. Scrollable, visual, and
  previous-overflow storage is colocated in `native_overflow_model.rs`; core
  only re-exports these modules.
  The compact `InkOverflow` storage, mutation policy, and text-overflow provider
  traits are physically owned by `native_ink_overflow.rs`; the core path only
  re-exports them, while box/inline export remains in the separately sampled
  `native_ink_overflow_export.rs` stage.
  Base `LayoutObject` scrollable- and visual-overflow child recursion is owned
  by `native_layout_object_overflow_recalc.rs`; its public base dispatch surface
  is colocated there while derived dispatch remains intact and parent traversal
  self time is charged to the fragment package. Its
  normal-flow child refresh entry also retains a fragment sampling frame, so
  conditional overflow work cannot collapse back into LayoutObject core time.
  Scroll-container ancestor discovery and local-to-ancestor point projection
  are owned by `native_layout_object_geometry_queries.rs`, separating shared
  geometry traversal from sticky and out-of-flow algorithm self time.
  PaintLayer identity, outline metrics/geometry/collection, and post-layout
  text-overflow scroll snapshots are physically owned by
  `native_paint_layer.rs`, `native_outline_info.rs`,
  `native_outline_geometry.rs`, `native_outline_rect_collector.rs`, and
  `native_text_overflow_post_layout_snapshot.rs`. Snapshot refresh keeps a
  fragment sampling frame because it may schedule layout invalidation.
  Its assembly-only native object-index module owns both the document-order
  LayoutObject scan and the uncullable-paint ancestor walk used by fragment
  publication and paint-only refreshes. Its native sticky-offset module owns
  the complete logical-preorder, per-axis constraint refresh, keeping
  `UpdateStickyOffsets` samples out of the core boundary facade. Scroll-only
  source collection, fragment recursion, scrollbar-thumb refresh, and revision
  propagation live in `native_scroll_refresh.rs` under the
  `layout_fragment.RefreshScrollPaintProperties` phase. The targeted
  native-object-to-fragment directory is physically owned by
  `native_scroll_paint_property_index.rs`; rebuild and refresh remain
  out-of-line, and targeted refresh emits
  `layout_fragment.RefreshScrollPaintIndex` with dirty-id and fragment counts.
  Native `ComputedStyle` and clip-path projection are also physically located
  in `layout_fragment/src/native_paint_style.rs` and
  `native_clip_path_export.rs`; the old core module is a re-export-only facade,
  and the high-level style projection entries stay out of line for sampling.
  Retained paint-input admission and mutation live in
  `native_paint_input_update.rs`; the `PaintInput` and `ValidatePaintInput`
  phases now contain fragment-owned sampling frames rather than builder logic.
  The retained clip, shader, filter, transform, mask, and `PaintStyleData`
  schema itself is physically owned by `native_paint_input.rs`; the old core
  module is a re-export-only compatibility facade.
  Scroll-offset input fan-out and native node metadata synchronization are
  physically owned by `native_scroll_input_update.rs`; the out-of-line entry
  makes scroll-only retained-input cost resolve to `layout_fragment`.
  Native paint-grouping/filter projection and transform-operation projection
  are physically owned by `native_style_effect_projection.rs`; author-sized
  effect lists retain separate fragment sampling frames while the input facade
  only sequences the projections. Clip command serialization and percentage
  polygon construction live in `native_clip_path_projection.rs` beneath one
  fragment sampling frame; the former core module is a thin re-export facade.
  Gradient-stop admission and ordered mask-layer construction live in
  `native_mask_image_projection.rs` beneath one mask-chain sampling frame;
  fixed-cost enum and length conversions remain in the boundary facade.
  The same module owns targeted `PaintStyle` mutation and resource-version
  admission. The following retained-fragment source scan, inline ink-overflow
  invalidation, recursive paint payload refresh, and global canvas/scroll
  refresh live in `native_retained_paint_refresh.rs` behind an out-of-line
  fragment sampling frame, completing the fragment-owned paint-only update
  boundary. Full native `LayoutResult` publication is likewise physically
  owned by `native_layout_result_publish.rs`; object indexing, sticky update,
  cull construction, fragment population, logical-parent resolution, and
  global paint export retain distinct fragment trace phases beneath that
  package-owned sampling frame. Its physical-fragment and inline-item recursive
  population chain is physically separated into
  `native_fragment_paint_population.rs`, with both recursive entries retained
  out of line so self time and recursion resolve directly to `layout_fragment`.
  Root-canvas background transfer admission and its document-element metadata
  recursion live in `native_background_transfer.rs` under a fragment sampling
  frame instead of `LayoutBoxModelObject` core self time.
  Retained gap geometry—including grid/flex/multicol intersection generation,
  fragmentation-range adjustment, equality, crossing-size scans, and ink
  extents—is physically owned by `native_gap_geometry.rs`. The old nested core
  path is a re-export-only facade, and input-sized public operations retain
  fragment sampling frames while scalar geometry access stays inlineable.
  Its complete retained data model is colocated in `native_cross_gap.rs`,
  `native_main_gap.rs`, `native_gap_intersection.rs`, `native_gap_utils.rs`, and
  `native_gap_decoration_utils.rs`; the corresponding core modules are all
  re-export-only compatibility paths.
  Per-object paint-property projection—including pre-paint invalidation facts,
  native client state, cached style projection, SVG resource resolution,
  sticky metadata, and form-control payloads—lives in
  `native_fragment_property_export.rs`; ordinary publication and retained
  refresh have separate out-of-line fragment entry symbols. Shared numeric,
  writing-mode, source/style-map, node-identity and text-decoration projection
  support is physically grouped in `native_fragment_export_support.rs` rather
  than leaking back into the core boundary. Paint-time glyph
  run construction, ShapeResult traversal, text-combine emphasis/geometry,
  text baseline queries, and list-marker payload projection are physically in
  `native_text_paint_export.rs`; the shape traversal and public paint-time
  glyph builder retain separate fragment sampling frames. SVG text transforms,
  stitched decorations, box paint geometry, scroll-container geometry, and
  content sizing are grouped in `native_fragment_geometry_export.rs`, with
  separate SVG-text and box-geometry sampling entries. Specialized payloads
  are split again by cost domain: `native_scrollbar_paint_export.rs`,
  `native_specialized_paint_export.rs` for table/MathML/frameset/fieldset/
  column work, and `native_collapsed_table_paint_export.rs` for collapsed-edge
  and fragmented-section reconstruction. Their expensive entries retain
  package-qualified sampling symbols without introducing per-node trace events.
  Publication culling, fragment instance identity allocation, and native SVG
  child expansion share `native_fragment_export_state.rs`; cull construction
  and SVG expansion remain distinct sampling entries while the shared state is
  hidden behind the reciprocal assembly boundary. Native box/inline ink and
  glyph-bound overflow projection live in `native_ink_overflow_export.rs`;
  border, mask-box, and reflection nine-piece resource resolution lives in
  `native_nine_piece_image_export.rs`. Each cost domain has its own fragment
  sampling entry. Whole-tree post-publication work is also separated:
  `native_global_paint_export.rs` owns canvas and main-thread-scroll facts,
  `native_fragment_relationship_export.rs` owns logical-parent resolution and
  sliced-inline stitching, and `native_sticky_paint_export.rs` owns sticky
  constraint projection. The successful layout transaction's preorder source
  collection and sticky update are grouped with their trace phase in
  `native_sticky_layout_commit.rs`, preventing that whole-tree scan from being
  charged to `LayoutEngine::RunLayout`. Scrollbar thumb geometry has its own
  `native_scrollbar_thumb_geometry.rs` sampling frame. Paint-backed inline
  materialization policy is owned by `native_inline_fragment_policy.rs`; both
  typed-style and native-style admission retain fragment sampling frames even
  when invoked from object construction or invalidation preparation.
  Retained fragment style mutation, native mutable-fragment traversal, and
  flattened PrePaint input collection are owned by
  `native_prepaint_input_update.rs`, with separate sampling entries beneath
  the existing operation-level fragment trace phases.
  Engine-level paint publication, targeted paint-data refresh, retained
  paint-input refresh, and scroll CullRect republishing are implemented by the
  out-of-line extension entries in `native_engine_paint_update.rs`;
  `LayoutEngine` retains only stable public forwarding methods, so profiler
  stacks resolve the work to the fragment package without changing callers.
  The no-layout retained-scroll path is isolated in
  `native_engine_scroll_refresh.rs`: targeted index refresh, full fragment-walk
  fallback, index reconstruction, and heap handoff share one fragment sampling
  frame, while actual relayout fallback remains core orchestration.
  The move-only `LayoutPaintOutput` facade likewise delegates CullRect scroll
  threshold search and mutable paint-input recursion to
  `native_layout_paint_output.rs`; both outer entries and recursive walks stay
  out of line for fragment-owned sampling attribution.
  Native cursor child expansion across physical fragments, inline items and
  SVG LayoutObject children is owned by
  `native_fragment_cursor_children.rs`; the public cursor keeps only a thin
  compatibility method, paint-input filtering is performed by the same owner,
  and all expansion stages keep fragment sampling frames.
  The reusable physical/inline descendant listener framework is physically in
  `native_tree_traversal_utils.rs`, including culled-inline discovery and its
  recursive entry; the legacy core module is a re-export-only facade.
  Page border-box discovery, page-area extraction, and stitched page content
  geometry are owned by `native_pagination_fragment_utils.rs`; the former
  pagination core module is also a re-export-only facade.
  Native child-box location projection is grouped in
  `native_layout_box_location_update.rs`: inline/physical descendants,
  fragmentainer offsets, deferred OOF locations, pagination stitching and the
  traversal listener are physically fragment-owned; the core utility is a
  re-export-only facade.
  Post-layout scrollable-overflow recalculation is isolated in
  `native_scrollable_overflow_recalc.rs`, including fragmentainer recursion,
  inline-item descent, per-fragment calculator calls, mutable overflow updates,
  and child result aggregation.
  The calculator's fragment recursion, item walk, transform projection, and
  fragmented out-of-flow ancestor clipping are physically owned by
  `native_scrollable_overflow_calculation.rs`, with separate sampling frames for
  recursion, item aggregation, and propagation. The calculator record, result
  accumulation, padding/origin policy and child/item entry points are colocated
  in `native_scrollable_overflow_calculator.rs`; core only re-exports it.
  Sticky per-axis constraint state and offset calculation live in
  `native_sticky_position_scrolling_constraints.rs`; ancestor coordinate
  mapping, complete constraint construction, and sticky-container lookup live
  in `native_layout_sticky_constraints.rs`. The tree-depth-dependent operations
  retain fragment-qualified sampling frames while small value queries remain
  inlineable.
  `PaintLayerScrollableArea`, scrollbar existence and geometry, scroll-origin
  updates, offset clamping, and disposal are physically owned by
  `native_layout_scrollable_area.rs`. LayoutBoxModelObject admission,
  allocation, replacement, and disposal of that retained client live in
  `native_layout_box_model_scrollable_area.rs`, with lifecycle sampling frames.
  Post-layout scrollbar client state remains fragment-owned. Per-box scrollbar
  struts, stable-gutter admission, overlay policy, and hypothetical theme
  thickness affect the available layout space and therefore live in
  `layout_algo_sizing/native_layout_box_scrollbars.rs`.
  Native scrollbar and scroll-corner display-client identity is colocated in
  `native_scrollbar_identity.rs`; the old core module is re-export-only.
  Custom scrollbar part lifetime, style mutation, button/track rectangles,
  margins, thickness, and length resolution live in
  `native_custom_scrollbar.rs` and
  `native_layout_custom_scrollbar_part.rs`, with the scalar part identity in
  `native_scrollbar_part.rs`. Part allocation/style mutation keeps a fragment
  sampling frame; fixed-size identity and geometry remain inlineable.
  Scroll snap-area element identity, consumed/pending axes, and container
  writing direction live in `native_snap_area.rs`, next to the fragment builder
  that propagates and resolves them; all scalar access stays inlineable.
  Freeze/root/delayed-clamp lifetimes and
  the delayed clamp queue live in `native_scroll_layout_scope.rs`; queue
  draining retains a fragment sampling frame while thread-local counter checks
  remain inlineable.
  Table column geometry and collapsed-border geometry retained by physical
  fragments are owned by `native_table_fragment_data.rs`, rather than being
  grouped with the table algorithm merely because their names are table-specific.
  CSS motion-path contour measurement, point sampling, rounded-polygon
  construction, and normalized command traversal are owned by
  `native_motion_path_geometry.rs`. Basic-shape resolution, circle/ellipse/
  inset construction, zoom handling, and final distance sampling are colocated
  in `native_motion_path_projection.rs`; the core path is now re-export-only,
  so author-sized geometry appears as fragment time.
  Transform reference-box selection, child-fragment transform construction,
  motion-path box projection, and transform-state accumulation live in
  `native_transform_utils.rs`. The two fragment-dependent projection entries
  retain fragment sampling frames; the core transform utility is a facade.
  Retained `LayoutBoxModelObject` transform rebuild, container transform and
  perspective composition, anonymous-ancestor lookup, and perspective-origin
  resolution are owned by `native_layout_transform.rs`, with out-of-line
  frames around the tree/geometry-sized paths.
  CSSOM client-rect queries—including recursive fragment-path discovery,
  sticky/SVG/CSS transform composition, scroll translation, inline rect
  collection and final mapping—are grouped in `native_client_rects.rs` with
  separate out-of-line sampling frames beneath the existing fragment trace.
  CSSOM scroll ranges and scalar layout metrics are isolated in
  `native_layout_metrics.rs`, including fragmented border-box union, native
  scroll-area fallback, padding/border/content projection, table/view sizing,
  and CSS zoom conversion.
  On-demand text cursor operations live in
  `native_text_fragment_queries.rs`: glyph payload construction, primary-font
  baseline, caret inline position, text-range rectangles, and point-to-offset
  hit testing retain separate fragment sampling frames.
  Text-control caret rectangles and hit testing are built on top of those
  queries by `native_caret_geometry.rs`; visible-text traversal and empty-editor
  fallback recursion are fragment-owned. `CaretShape` and the cursor-based
  `CaretRectServices` contract live in `native_caret_rect.rs`; the two legacy
  caret modules retain only compatibility APIs and result types.
  Targeted update cache maintenance is isolated
  in `native_paint_cache_refresh.rs`; style-source, sticky-object, and
  uncullable-object index rebuilds report the single low-cardinality
  `layout_fragment.RefreshBoundaryCache` phase and package-owned symbols.
- `layout_commit` owns the typed Layout-to-Paint publication contract and the
  ordered transactions for environment publication and completed Paint
  client commit. Its assembly-only `native_client_commit.rs` owns the hot
  native-client and retained-fragment traversals; the core entry is a narrow
  adapter. Per-box layout-result append/replace/truncate, fragment-item
  finalization, validation, and post-layout spine rebuilding are physically
  owned by `native_layout_box_result_commit.rs`, with separate sampling frames
  for each publication stage. `native_layout_box_post_layout_commit.rs` owns
  the post-install overflow, transform, scroll-area and dirty-state handoff,
  keeping that lifecycle step visible as commit time rather than LayoutBox
  algorithm self time.
  Root-view post-layout dispatch, pending-selection publication, and scroll
  dimension snapshotting are physically owned by
  `native_layout_view_commit.rs`; each provider boundary retains its own
  out-of-line commit frame rather than accumulating as `LayoutView` core time.
  Stable input-id lookup of the published non-generated LayoutObject is owned
  by `native_output_node_lookup.rs`; immutable and mutable facade accessors
  inline into one commit-qualified candidate-selection frame.
  Completed-result publication remains commit-owned. Per-pass font,
  hyphenation, phrase-break, text-transform, and shape-image resolver
  installation/teardown live in the shared boundary's
  `native_layout_environment.rs`, because environment creation precedes the
  algorithm pass. Each concrete resolver retains its functional owner.

`layoutng_assembly` remains the temporary composition boundary for translated
LayoutNG, fragment, and inline sources that still have reciprocal C++ type
dependencies. A source directory or `#[path]` module records functional source
ownership; only a Cargo manifest compiled as a dependency establishes an
independent crate boundary. Documentation and ownership tests must preserve
that distinction instead of describing composition modules as independently
compiled packages. Cargo package names, library names, dependency keys, source
paths, and trace prefixes deliberately use the same names where a real package
boundary exists; compatibility aliases are not allowed because they make
profiles and dependency reports ambiguous. The composition-only dependency key
`layout_fragment_boundary` is a temporary exception in the two `_assembly`
crates: it distinguishes the real standalone crate from their
`extern crate self as layout_fragment` source-translation alias, and is not
re-exported as a compatibility package.

The public Layout/Paint writing mode is explicitly named
`LayoutPaintWritingMode` at the foundation boundary. It intentionally differs
from Chromium's five-value native `WritingMode`; the native-to-artifact adapter
is the only place where sideways modes collapse to their physical vertical
direction.

`layout_algo_inline_assembly` is a compile-only composition profile used to
verify the same reciprocal inline source set. It is not a performance owner;
runtime work from those sources must retain the `layout_algo_inline.*` prefix
and `layout_algo_inline_package::*` sampling path.
Native hyphenation, text-transform, and phrase-break provider adapters are
physically owned by `layout_algo_inline/src/native_text_input_environment.rs`;
their validation calls retain sampling frames without per-word trace events.
Native shape-image lookup and bitmap validation are likewise owned by
`layout_algo_float/src/native_shape_image_environment.rs`; its resolver trait,
TLS scope, and current-resolver contract are colocated in
`native_shape_image_services.rs` rather than generic core.

## Performance attribution contract

- Every concrete formatting entry is traced as
  `layout_algo_<type>.Layout` or `layout_algo_<type>.Measure`. Supporting hot
  paths such as float positioning, out-of-flow layout, inline layout, list
  marker measurement, form intrinsic sizing, table constraint resolution, and
  SVG text preparation retain that package prefix. Native algorithm
  registration validates both trace names against its declared
  `layout_algo_*` owner, and retains that owner on the installed algorithm
  entry so the final browser assembly can verify every supported slot. Support
  callback groups (inline, float, table, forms, list, replaced sizing, SVG, and
  out-of-flow) retain the same owner metadata for the same reason. Optional root-dispatch traces travel through the same
  registered entry instead of being named by core, so a generic or
  cross-package name fails during assembly. `LayoutEngine` also validates the
  complete algorithm registry at construction: an executable slot must install
  its package owner, layout callback, and measurement callback atomically, and
  every support-callback group must install its owner together with its first
  callback. A deliberately unsupported slot must remain fully empty. This
  prevents partially assembled algorithms or support paths from producing
  unowned samples. Cross-algorithm replaced-content sizing enters through the
  out-of-line `layout_algo_replaced.NaturalSizingInfo` callback, so its CPU
  time is not charged to the outer block, flex, or grid algorithm.
  The generic native layout/measure trampolines are also kept out of line and
  monomorphized with the concrete algorithm type; demangled sampling symbols
  therefore include `layout_algo_block`, `layout_algo_flex`,
  `layout_algo_grid`, `layout_algo_table`, or the corresponding package even
  though the shared trampoline implementation lives at the assembly boundary.
  Assembly contracts allow core `#[inline(never)]` frames only for the generic
  algorithm trampolines and an explicit, reviewed list of shared boundaries
  whose operation spans multiple functional owners. Input/tree-sized work with
  a narrower owner must move to that lifecycle or concrete algorithm owner.
  Shared fixed-cost helpers remain inlineable so their CPU cost folds into the
  package-qualified caller instead of creating an ownerless core bucket.
  The custom package exposes no `Layout` span until an executable algorithm is
  installed, but its input-sized payload-copy support already has a custom
  sampling frame. MathML remains type-only and emits no runtime span.
- Lifecycle work is traced with its owning package prefix:
  `layout_invalidation.*`, `layout_reuse.*`, `layout_fragment.*`, and
  `layout_commit.*`. Invalidation adapters select a package-owned `TracePhase`
  rather than spelling trace names in the assembly layer.
- `layout_commit.*` is limited to transaction ordering and completed-client
  publication. Fragment export, object indexing, paint-input certification,
  sticky/cull refresh, logical-parent resolution, and scroll-property index
  construction use package-owned `layout_fragment.*` subphases. This keeps the
  commit facade's inclusive time from hiding which fragment stage consumed it.
  Completed-client publication is split into package-owned
  `CollectNativeClients` and `RefreshFragmentClients` phases, with one summary
  field set per traversal rather than per-node events. Their out-of-line
  `layout_commit_package::commit_paint_client_state` sampling symbol keeps the
  native and fragment walks under the same performance owner.
  The concrete `LayoutEngine` backend for auxiliary-client validation, native
  client commit, PrePaint revision commit, and resident-result publication is
  physically in `native_engine_paint_commit.rs`; each ordered transaction
  stage keeps a commit-owned sampling frame while the engine retains only the
  typed construction site.
  Successful native layout publication is handled by
  `native_engine_layout_publish.rs`: its begin frame transfers the previous
  resident owner, scroll index, PrePaint revision and targeted-subtree hint;
  after native layout, PrePaint certification and scroll-index construction
  retain fragment-owned subphases while installing the new resident result
  remains one out-of-line commit transaction. The retained-scroll fallback
  which intentionally runs layout without publication is implemented in the
  same owner and reports `layout_commit.RunLayoutWithoutPublication` instead
  of generic engine self time.
  Post-layout container-query size publication is likewise owned by
  `native_container_query_sizes.rs`; its complete native preorder scan and
  content-box/zoom projection stay behind one out-of-line commit sampling
  entry, while the public engine query remains source compatible.
  Page scroll-offset snapshot publication is isolated in
  `native_scroll_offset_snapshot.rs`; scanning managed native records,
  excluding generated objects and reading retained element offsets no longer
  appears as `LayoutObjectTree` builder self time.
- Assembly tests recursively reject lifecycle and algorithm trace-name
  literals declared outside their owner directory. The same contract verifies
  that every performance owner's directory, Cargo package/library name, and
  dependency key agree; only the documented composition-only fragment alias is
  exempt. Bottom-up, call-tree, and overview views derive grouping, explicit
  package labels, filters, and color from the same package prefix as the flame
  chart. Each `layout_algo_*` owner has a stable package color instead of
  collapsing all algorithms into the generic Layout color. Exported
  performance summaries also include exclusive owner totals and package-owned
  percentile rows, so API consumers and the Statistics pane use the same
  attribution as the interactive views.
- Reuse is aggregated once per native layout pass. The dispatch span reports
  lookup count, hit/miss outcomes, and total cache-lookup time; per-node cache
  events are forbidden because they distort the hot path and flood traces.
  Package-owned `PassSummary` and `DecisionSummary` events stay below the
  trace collector's eight-field limit while distinguishing incompatible
  constraint spaces, intrinsic-size changes, equal/stable-size hits, and
  native geometry checks. `BfcDecisionSummary` separately groups BFC reuse
  rejection pressure by float context, margin mutation, descendant geometry,
  self-collapsing clearance, forced offsets, and exclusion clearance. The BFC
  decision function is out of line in `layout_reuse`, so sampling and trace
  diagnostics point at the same owner. The enclosing native lookup is also an
  out-of-line `layout_reuse_package::cached_layout_result` symbol, preventing
  the core `LayoutBox` facade from absorbing its inclusive CPU time.
  Only lookups taking at least 50 microseconds emit a
  `layout_reuse.SlowLookup` interval for outlier localization.
  The native eight-entry measurement cache is physically implemented in
  `layout_reuse/src/native_measure_cache.rs`; its `Find` decision remains
  out-of-line for sampling while continuing to contribute to the enclosing
  aggregated reuse session instead of emitting a span for every node.
- A facade span may orchestrate several packages, but it must not be the only
  visible span around package-owned work. New unlabelled loops crossing a
  package boundary should be treated as an observability regression.
- DevTools derives the performance owner from the trace-name prefix. Flame
  chart color, selected-event package, and self-time totals therefore use the
  same package identity instead of the generic `layout` trace category.
- `LayoutEngine.*` is reserved for top-level transactions that may cross
  package boundaries. Leaf work must use one of the package prefixes above.
- Hot tree policy is also kept out-of-line in its owner package, so sampling
  profilers resolve propagation to `layout_invalidation::propagation::*` even
  when the concrete tree adapter lives in `layoutng_assembly`.
- LayoutObject update entry points delegate to the out-of-line
  `layout_invalidation_package::{update_subtree,begin_update,finish_update}`
  transaction. Consequently `CollectSubtree`, `PrepareSubtree`, and
  `RetireObjects` spans contain package-qualified sampling frames instead of
  core `layout_object_builder` loops.
- The opaque native result owner and scroll-property revision policy resolve
  to `layout_fragment::LayoutResultHandle` and
  `layout_fragment::revision::*`. The package also owns the
  `layout_fragment.RevisionUpdate` and `layout_fragment.ExportTree` trace
  constructors; commit and assembly adapters only add native traversal work
  and statistics. Revision counters are package-owned and split into a main
  interval plus `RevisionReasonSummary`, so all counters survive the trace
  collector's field limit. Native graph casts remain confined to the assembly
  cursor adapter.
- Fragment object indexing resolves to the out-of-line
  `layout_fragment_native::ensure_layout_boundary_object_index` and
  `mark_uncullable_paint_ancestors` symbols. Core boundary functions only
  preserve translated entry points, so `IndexObjects` inclusive time and
  sampling stacks agree on the fragment owner.

`layout_fragment` is now a standalone workspace package for the opaque native
result lifetime handle. Its translated graph implementation and the inline
implementation still have reciprocal concrete-type dependencies with LayoutNG
and are composed by `layoutng_assembly`. Those sources remain nested under
`layout_fragment_package` and `layout_algo_inline_package`, so demangled
sampling symbols retain their owner while the remaining concrete declarations
are split behind assembly adapters.

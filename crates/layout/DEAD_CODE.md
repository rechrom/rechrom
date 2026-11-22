# Layout dead code and untranslated stubs

Analysis only; no code was changed. Snapshot: `main` at `5a6ee664`, 2026-11-22.
Warning data comes from `D:\upstream-dl\build_fix.log`, a full
`cargo build --profile iteration -p rechrom_app --message-format short`.
Scope: every package under `crates/layout/`, including `paint_input_types` and
`layout_style_paint`.

## How this was measured

- **Stubs.** `layout_assembly/src/untranslated_symbols.rs` defines 252
  `#[no_mangle] extern "C" fn X() -> !` aborting stubs. For each symbol, every
  `fn X(` declaration in the tracked sources was found, plus every call site
  and the function that contains it (the "wrapper"). Then every by-name
  reference to that wrapper in the workspace was counted.
- **Reachability.** The stub file says the stubs exist only because MSVC
  `link.exe` resolves symbols before it strips dead code. ld64 and lld link the
  same binary without the stubs. So **none of the 252 symbols is reachable from
  the shipped app today**. The classes below therefore answer a different
  question: is the caller code an orphan that can go (a), or a translated
  Chromium mechanism that is missing its wiring (b)?
  - (a) **dead**: either the wrapper has no callers, or Rechrom has a live
    equivalent somewhere else (Paint, Interaction, the Style backend, another
    translated function). Delete the wrapper, the `extern` declaration and the
    stub.
  - (b) **Chromium gap**: the behaviour is missing from the live pipeline and
    the declaring code is its natural home. Keep it, and track it as a gap.
  - (c) **unclear**: needs a decision about the integration path first.
- **Name-reference counts are approximate.** The "Name refs to wrapper" column
  counts `Name(` / `.Name(` anywhere in the workspace, so common names
  (`Paint`, `new`, `ToString`, `HitTest`) are inflated by unrelated types. A
  `0` is strong evidence of no caller. A non-zero count was spot-checked for the
  families below; it is not a proof.
- **Warnings.** Every dead-code or unused warning was taken from the log. Each
  was attributed to the directory that owns the source file after `..` paths
  were resolved, then de-duplicated by file, line, column and message. Most
  Layout sources compile inside `layout_assembly` through `#[path]` /
  `include!`, so "compiled via" is shown separately.
- **Compiled files.** A `mod` / `#[path]` / `include!` resolver (inline `mod {}`
  blocks included) was run from every library, binary and test root in the
  workspace. Two simplifications: `cfg` gates were not evaluated, and inline
  `#[path]` nesting is approximate.

## Summary

| Item | Count |
|---|---|
| Aborting stubs in `untranslated_symbols.rs` | 252 |
| Class (a): dead, delete wrapper + `extern` decl + stub | 186 |
| Class (b): Chromium gap, keep and track | 48 |
| Class (c): unclear | 18 |
| Stubs whose wrappers have **zero** by-name references (the safest deletions) | 134 (99 a, 25 b, 10 c) |
| Stubs whose Rust body is **already translated** but not `#[no_mangle]`, so the `extern` wrapper links to the abort stub | 11 |
| Dead-code/unused warnings in Layout crates (de-duplicated) | 370 |
| Of those, `extern` declarations that nobody calls (they need no stub and are pure leftovers) | 103 warnings / 100 names (93 of them are not in the stub list) |
| `.rs` files under `crates/layout` that are never compiled | 0 (apart from 3 shell `lib.rs` files, see below) |
| Stray non-source files | 2 (`layout_core/src/fragment/layout_result_publish.rs.tmp`, `layout_core/Cargo.lock`) |

## Surprising findings

1. **11 translated bodies are cut off by their own stubs.** In
   `layout_algo_inline/src/inline_node.rs` (and three sibling files), the
   following are ordinary mangled `pub fn`s:
   - `InlineNodeSetTextWithOffset` (:986)
   - `InlineNodeFindSvgTextChunks` (:554)
   - `InlineNodeComputeOffsetMapping` (:632)
   - `InlineNodeEnsureData` (:2088)
   - `InlineNodeMaybeDirtyData` (:1234)
   - `InlineNodeCheckConsistency` (:2597)
   - `InlineNodeNeedsShapingForTesting` (:2592)
   - `InlineNodeInvalidatePrepareLayoutForTest` (:1148)
   - `InlineItemSegmentsToRanges` (`inline_item_segment.rs:161`)
   - `InlineItemResultToString` (`inline_item_result.rs:89`)
   - `InlineItemToString` (`layout_core/src/fragment/logical_line_item.rs:581`)

   *R5 part 2 (2026-11-22):* the base wrappers and `extern` declarations of
   `InlineNodeSetTextWithOffset`, `InlineNodeFindSvgTextChunks`,
   `InlineNodeComputeOffsetMapping`, `InlineNodeCheckConsistency` and
   `InlineItemResultToString` are gone (the last is reachable through
   `InlineItemResultAlgorithms::ToString`), and so are their 5 stubs; the
   counts below predate that.

   Their `*_interface.rs` wrappers call an `extern` with the same name. Because
   the body is mangled, that resolves to the abort stub. Code in the same module
   calls the body directly, which is why layout works. Through the interface,
   `InlineNode::SetTextWithOffset` (Chromium's incremental text-edit fast path,
   `layout_text.cc` → `inline_node.cc`) would abort, even though it is fully
   translated.
2. **`layout_algo_inline_assembly` is a second, unused assembly crate.** (Deleted in R5.) It
   `include!`s `layout_core`, `layout_fragment` and `layout_algo_inline` a
   second time, and nothing depends on it. Only `cargo build --workspace`
   builds it; the app build does not.
3. **Four packages are empty shells.** Nothing depends on
   `layout_algo_fragmentation`, `layout_algo_sizing`, `layout_algo_inline` or
   `layout_core`. Their sources compile only inside `layout_assembly` through
   `#[path]`.
   - The first two `lib.rs` files contain only a doc comment.
   - `layout_algo_inline/src/lib.rs` gates all 48 of its modules on a
     `translation_in_progress` feature of its own package. Nothing enables that
     feature; the live code is the same file `include!`d by `layout_assembly`,
     which has the feature on.
   - `layout_core` is excluded from the workspace, yet carries its own stray
     `Cargo.lock`.
4. **Name collisions hide the real call graph.** Many stubbed wrappers share a
   name with a live, translated function on a different owner:
   - `FragmentRepeater::LayoutRepeatableRoot` vs
     `BlockNode::LayoutRepeatableRoot` (`layout_core/src/block/block_node.rs:632`).
   - `LayoutBox::IsEligibleForSizeContainment` vs
     `LayoutObject::IsEligibleForSizeContainment`.
   - `ComputedStyle::GetBaseImportantSet` vs `StyleBaseData::GetBaseImportantSet`.
   - `ComputedStyle::VisitedDependentColor` vs `VisitedDependentColorFast`.

   When a `LayoutBox`-level shadow method is deleted, any `LayoutBox` caller
   falls back to the `LayoutObject` method through `Deref`. That changes
   behaviour from an abort to the live dispatch, so the compiler alone will not
   flag it.
5. **Highlight pseudo styles are never matched-properties-cached.**
   `MatchedPropertiesCache::Find` (`style/style_resolve/src/matched_properties_cache.rs:294`)
   is generic and never instantiated. Production calls only
   `FindForNonHighlightWithStylesAndData`. That makes the two
   `*HighlightOriginatingElementDataEqual` stubs a performance gap, not a
   correctness one.
6. **A possible dead generic `StyleEngine` (outside Layout).** The generic
   `StyleEngine<B: StyleEngineBackend>` in `crates/style/style/src/style_engine.rs`
   (6.5k lines) appears to have no `StyleEngineBackend` implementor anywhere in
   the workspace. Its counter (`NewCountersAttachmentContext`) and
   `AnalyzeInclusiveAncestor` paths are therefore uninstantiated. This belongs in
   a Style audit.
7. **Several web-visible features have no live implementation at all.**
   - `content: counter()`: `CounterContentData::new` is a stub and is never
     called.
   - `offsetParent` / `offsetLeft` / `offsetTop`: no binding exists outside
     Layout.
   - Relative, contrast and alpha colors that depend on `currentcolor`: their
     types are never constructed; only `color-mix()` is live.
   - CSS Layout API, CSS Paint API, MathML layout and paged media.

## Stub families

| Class | Family | Stubs | Chromium source (for gaps) / reason (for dead) |
|---|---|---|---|
| a | paint / hit-test / geometry shim on LayoutObject, LayoutBox, LayoutBlock, LayoutText, LayoutInline, LayoutView | 72 | Paint, PrePaint and hit testing live in `crates/paint` and `crates/interaction` (for example `interaction_hit_test/src/core/layout/*`). These Layout-side entry points (`Paint`, `NodeAtPoint`, `HitTest*`, `InvalidatePaint*`, `InvalidateDisplayItemClients`, `*Layers`, `MapLocalToAncestor`, `PositionForPoint`, ...) are not on the production path. |
| a | other LayoutObject / ComputedStyle shim | 72 | No live caller. A live equivalent exists under another name or owner (Style backend traits, `VisitedDependentColorFast`, the `LayoutObject` dispatch methods, `scroll_snap`, DOM display locks, ...). |
| a | custom properties on ComputedStyle | 14 | The live custom-property path is `style/style_resolve/src/custom_properties.rs`. |
| a | debug / test helper (`ToString`, `fmt`, `Trace`, `DebugRect`, `DecoratedName`, `*ForTest*`, `CheckConsistency`) | 18 | Debug-only. 5 of them also have an unexported translated body. |
| a | translated body not exported | 5 | `EnsureData`, `MaybeDirtyData`, `ComputeOffsetMapping`, `FindSvgTextChunks`, `SegmentsToRanges`. Delete the interface wrapper, `extern` and stub (callers already use the body), or add `#[no_mangle]`. |
| a | unused constructor | 3 | Production uses `LayoutObjectTree::new_with_factories`; `SetLayoutNeededForbiddenScope` is never constructed. |
| a | duplicate | 2 | `FragmentRepeater`. `BlockNode` has the translated repeatable-root code. |
| b | CSS counters / content alt text | 19 | `core/css/counters_attachment_context.cc`, `core/layout/layout_counter.cc`, `core/style/content_data.cc`, `core/css/list_style_type_data.cc` (`symbols()`), `core/layout/list/list_item_ordinal.cc` (`ItemInsertedOrRemoved`), `core/layout/layout_view.cc` (`InvalidateLayoutForCounterStyleChanges`) |
| b | currentcolor-dependent relative / contrast / alpha colors | 9 | `core/css/style_color.cc`: `UnresolvedRelativeColor`, `UnresolvedContrastColor`, `UnresolvedAlphaColor` |
| b | CSS Layout API | 4 | `core/layout/custom/layout_custom.cc`, `custom_layout_child.cc`, `core/layout/block_node.cc` |
| b | MathML layout | 3 | `core/layout/block_node.cc` (`HasIndex`, `ScriptType`), `core/layout/mathml/layout_mathml_block.h` (`IsMathMLRoot`). `layout_algo_mathml` holds only metadata. |
| b | paged media / printing | 3 | `core/layout/block_node.cc` (`FinishPageContainerLayout`), `core/layout/layout_view.cc` (`NamedPageAtIndex`, `CreateAnonymousPageLayoutObject`) |
| b | accessibility | 3 | `core/layout/layout_text.cc` (`FirstAbstractInlineTextBox`, `LocalBoundingBoxRectForAccessibility`), `core/layout/layout_inline.cc`. Also reached from `spatial_navigation_geometry.rs`. |
| b | offsetParent / offsetLeft / offsetTop | 2 | `core/layout/layout_object.cc` (`OffsetParent`), `core/layout/layout_box_model_object.cc` (`AdjustedPositionRelativeTo`) |
| b | incremental text edit | 2 | `core/layout/layout_text.cc` `SetTextWithOffset` → `core/layout/inline/inline_node.cc` `InlineNode::SetTextWithOffset`. The body is already translated at `inline_node.rs:986`; only the export and wiring are missing. |
| b | matched-properties cache for highlight pseudos | 2 | Generated `computed_style_base.h` `(Non)HighlightOriginatingElementDataEqual`, used by `core/css/resolver/matched_properties_cache.cc`. Performance only. |
| b | CSS Paint API | 1 | `core/style/computed_style.cc` `ComputedStyleBuilder::AddPaintImage` |
| c | OffsetMapping static queries | 13 | `core/layout/inline/offset_mapping.cc`. Editing goes through the `interaction_default` context traits; decide whether those should call these Chromium queries before deleting them. |
| c | SVG `url()` resource clients | 4 | `core/style/style_svg_resource.cc`. Live Style code (`document_style_engine.rs`, `element_style_resources.rs`) references the type, but it is never constructed. Check how `fill` / `stroke` / `clip-path` / `filter` `url()` references actually reach SVG resources. |
| c | anchor position-try scroll ranges | 1 | `core/layout/layout_box.cc` `NonOverflowingScrollRanges`, called from `anchor_position_scroll_data.rs` |

## Stub table

Paths are relative to `crates/layout/`. "Declared in" lists the `extern`
declarations. "Wrapper(s)" are the functions that contain a call to the stub
symbol. "Name refs" is the approximate workspace-wide count of calls to each
wrapper name; see the caveat above.

| # | Symbol | Declared in | Wrapper(s) calling it | Name refs to wrapper | Class | Family |
|---|---|---|---|---|---|---|
| 1 | `AltCounterContentDataUpdateText` | layout_style/src/style/content_data.rs:805 | AltCounterContentData::UpdateText (layout_style/src/style/content_data.rs:589) | UpdateText 0 | b | CSS counters / content alt text |
| 2 | `BasicShapeCircleFloatValueForRadiusInBox` | layout_style/src/style/basic_shapes.rs:493 | BasicShapeCircle::FloatValueForRadiusInBox (layout_style/src/style/basic_shapes.rs:167) | FloatValueForRadiusInBox 0 | a | other LayoutObject/ComputedStyle shim |
| 3 | `BasicShapeEllipseResolveRadii` | layout_style/src/style/basic_shapes.rs:498 | BasicShapeEllipse::ResolveRadii (layout_style/src/style/basic_shapes.rs:238) | ResolveRadii 0 | a | other LayoutObject/ComputedStyle shim |
| 4 | `BlockNodeFinishPageContainerLayoutFromPaged` | layout_core/src/block/block_node.rs:109 | BlockNode::FinishPageContainerLayout (layout_core/src/block/block_node.rs:232) | FinishPageContainerLayout 0 | b | paged media / printing |
| 5 | `BlockNodeGetCustomLayoutDefinitionFromCustom` | layout_core/src/block/block_node.rs:106 | BlockNode::GetCustomLayoutDefinition (layout_core/src/block/block_node.rs:216) | GetCustomLayoutDefinition 0 | b | CSS Layout API |
| 6 | `BlockNodeHasIndexFromMathML` | layout_core/src/block/block_node.rs:108 | BlockNode::HasIndex (layout_core/src/block/block_node.rs:226) | HasIndex 0 | b | MathML layout |
| 7 | `BlockNodeIsCustomLayoutLoadedFromCustom` | layout_core/src/block/block_node.rs:105 | BlockNode::IsCustomLayoutLoaded (layout_core/src/block/block_node.rs:212) | IsCustomLayoutLoaded 1 | b | CSS Layout API |
| 8 | `BlockNodeScriptTypeFromMathML` | layout_core/src/block/block_node.rs:107 | BlockNode::ScriptType (layout_core/src/block/block_node.rs:222) | ScriptType 0 | b | MathML layout |
| 9 | `ComputedStyleBaseFindChangedGroups` | layout_style/src/style/computed_style_base.rs:28276 | ComputedStyleBase::FindChangedGroups (layout_style/src/style/computed_style_base.rs:25796) | FindChangedGroups 0 | a | other LayoutObject/ComputedStyle shim |
| 10 | `ComputedStyleBaseHighlightOriginatingElementDataEqual` | layout_style/src/style/computed_style_base.rs:134 | ComputedStyleBase::HighlightOriginatingElementDataEqual (layout_style/src/style/computed_style_base.rs:402) | HighlightOriginatingElementDataEqual 1 | b | MPC for highlight pseudos |
| 11 | `ComputedStyleBaseNonHighlightOriginatingElementDataEqual` | layout_style/src/style/computed_style_base.rs:139 | ComputedStyleBase::NonHighlightOriginatingElementDataEqual (layout_style/src/style/computed_style_base.rs:407) | NonHighlightOriginatingElementDataEqual 1 | b | MPC for highlight pseudos |
| 12 | `ComputedStyleBaseTransitionAllDiff` | layout_style/src/style/computed_style_base.rs:28269 | ComputedStyleBase::TransitionAllDiff (layout_style/src/style/computed_style_base.rs:25786) | TransitionAllDiff 0 | a | other LayoutObject/ComputedStyle shim |
| 13 | `ComputedStyleBaseTransitionAllWithDiscreteDiff` | layout_style/src/style/computed_style_base.rs:28271 | ComputedStyleBase::TransitionAllWithDiscreteDiff (layout_style/src/style/computed_style_base.rs:25791) | TransitionAllWithDiscreteDiff 0 | a | other LayoutObject/ComputedStyle shim |
| 14 | `ComputedStyleBuilderAddPaintImage` | layout_style/src/style/computed_style.rs:5214 | ComputedStyleBuilder::AddPaintImage (layout_style/src/style/computed_style.rs:4234) | AddPaintImage 1 | b | CSS Paint API |
| 15 | `ComputedStyleBuilderBaseResetTapHighlightColor` | layout_style/src/style/computed_style_base.rs:17639 | ComputedStyleBuilderBase::ResetTapHighlightColor (layout_style/src/style/computed_style_base.rs:17646) | ResetTapHighlightColor 0 | a | other LayoutObject/ComputedStyle shim |
| 16 | `ComputedStyleBuilderBaseResetTextEmphasisPosition` | layout_style/src/style/computed_style_base.rs:18333 | ComputedStyleBuilderBase::ResetTextEmphasisPosition (layout_style/src/style/computed_style_base.rs:18339) | ResetTextEmphasisPosition 0 | a | other LayoutObject/ComputedStyle shim |
| 17 | `ComputedStyleBuilderGetVariableData` | layout_style/src/style/computed_style.rs:5267 | ComputedStyleBuilder::GetVariableData (layout_style/src/style/computed_style.rs:5043) | GetVariableData 6 | a | custom properties on ComputedStyle |
| 18 | `ComputedStyleBuilderMutableInheritedVariables` | layout_style/src/style/computed_style.rs:5273 | ComputedStyleBuilder::MutableInheritedVariables (layout_style/src/style/computed_style.rs:5049) | MutableInheritedVariables 2 | a | custom properties on ComputedStyle |
| 19 | `ComputedStyleBuilderMutableNonInheritedVariables` | layout_style/src/style/computed_style.rs:5276 | ComputedStyleBuilder::MutableNonInheritedVariables (layout_style/src/style/computed_style.rs:5054) | MutableNonInheritedVariables 2 | a | custom properties on ComputedStyle |
| 20 | `ComputedStyleBuilderSetInheritedVariablesFrom` | layout_style/src/style/computed_style.rs:5280 | ComputedStyleBuilder::SetInheritedVariablesFrom (layout_style/src/style/computed_style.rs:5059) | SetInheritedVariablesFrom 0 | a | custom properties on ComputedStyle |
| 21 | `ComputedStyleBuilderSetNonInheritedVariablesFrom` | layout_style/src/style/computed_style.rs:5284 | ComputedStyleBuilder::SetNonInheritedVariablesFrom (layout_style/src/style/computed_style.rs:5064) | SetNonInheritedVariablesFrom 0 | a | custom properties on ComputedStyle |
| 22 | `ComputedStyleCanMatchSizeContainerQueries` | layout_style/src/style/computed_style.rs:5259 | ComputedStyle::CanMatchSizeContainerQueries (layout_style/src/style/computed_style.rs:2600) | CanMatchSizeContainerQueries 3 | a | other LayoutObject/ComputedStyle shim |
| 23 | `ComputedStyleCanRenderBorderImage` | layout_style/src/style/computed_style.rs:5179 | ComputedStyle::CanRenderBorderImage (layout_style/src/style/computed_style.rs:276) | CanRenderBorderImage 0 | a | other LayoutObject/ComputedStyle shim |
| 24 | `ComputedStyleCopyChildDependentFlagsFrom` | layout_style/src/style/computed_style.rs:5227 | ComputedStyle::CopyChildDependentFlagsFrom (layout_style/src/style/computed_style.rs:910) | CopyChildDependentFlagsFrom 0 | a | other LayoutObject/ComputedStyle shim |
| 25 | `ComputedStyleDependsOnFunc` | layout_style/src/style/computed_style.rs:5141 | ComputedStyle::DependsOnFunc (layout_style/src/style/computed_style.rs:322) | DependsOnFunc 0 | a | other LayoutObject/ComputedStyle shim |
| 26 | `ComputedStyleDiffAffectsContainerQueries` | layout_style/src/style/computed_style.rs:5236 | ComputedStyle::DiffAffectsContainerQueries (layout_style/src/style/computed_style.rs:667) | DiffAffectsContainerQueries 0 | a | other LayoutObject/ComputedStyle shim |
| 27 | `ComputedStyleGetBaseImportantSet` | layout_style/src/style/computed_style.rs:5265 | ComputedStyle::GetBaseImportantSet (layout_style/src/style/computed_style.rs:1029) | GetBaseImportantSet 4 | a | other LayoutObject/ComputedStyle shim |
| 28 | `ComputedStyleGetBorderEdgeInfo` | layout_style/src/style/computed_style.rs:5241 | ComputedStyle::GetBorderEdgeInfo (layout_style/src/style/computed_style.rs:3002) | GetBorderEdgeInfo 1 | a | other LayoutObject/ComputedStyle shim |
| 29 | `ComputedStyleGetVariableNames` | layout_style/src/style/computed_style.rs:5231 | ComputedStyle::GetVariableNames (layout_style/src/style/computed_style.rs:929) | GetVariableNames 0 | a | custom properties on ComputedStyle |
| 30 | `ComputedStyleGetVariableNamesCount` | layout_style/src/style/computed_style.rs:5229 | ComputedStyle::GetVariableNamesCount (layout_style/src/style/computed_style.rs:924) | GetVariableNamesCount 0 | a | custom properties on ComputedStyle |
| 31 | `ComputedStyleHasAnimationTrigger` | layout_style/src/style/computed_style.rs:5211 | ComputedStyle::HasAnimationTrigger (layout_style/src/style/computed_style.rs:3859) | HasAnimationTrigger 0 | a | other LayoutObject/ComputedStyle shim |
| 32 | `ComputedStyleHighlightPseudoElementStylesDependOnRelativeUnits` | layout_style/src/style/computed_style.rs:5163 | ComputedStyle::HighlightPseudoElementStylesDependOnRelativeUnits (layout_style/src/style/computed_style.rs:286) | HighlightPseudoElementStylesDependOnRelativeUnits 0 | a | other LayoutObject/ComputedStyle shim |
| 33 | `ComputedStyleHighlightPseudoElementStylesHaveVariableReferences` | layout_style/src/style/computed_style.rs:5167 | ComputedStyle::HighlightPseudoElementStylesHaveVariableReferences (layout_style/src/style/computed_style.rs:311) | HighlightPseudoElementStylesHaveVariableReferences 0 | a | custom properties on ComputedStyle |
| 34 | `ComputedStyleIsInterleavingRoot` | layout_style/src/style/computed_style.rs:5261<br>crates/style/style/src/style_engine.rs:1196 | ComputedStyle::IsInterleavingRoot (layout_style/src/style/computed_style.rs:2605)<br>StyleEngine::AnalyzeInclusiveAncestor (crates/style/style/src/style_engine.rs:2435) | AnalyzeInclusiveAncestor 2 IsInterleavingRoot 0 | a | other LayoutObject/ComputedStyle shim |
| 35 | `ComputedStyleIsRenderedInTopLayer` | layout_style/src/style/computed_style.rs:5181 | ComputedStyle::IsRenderedInTopLayer (layout_style/src/style/computed_style.rs:281) | IsRenderedInTopLayer 1 | a | other LayoutObject/ComputedStyle shim |
| 36 | `ComputedStyleListStyleStringValue` | layout_style/src/style/computed_style.rs:5171 | ComputedStyle::ListStyleStringValue (layout_style/src/style/computed_style.rs:266) | ListStyleStringValue 0 | a | other LayoutObject/ComputedStyle shim |
| 37 | `ComputedStyleMarkerShouldBeInside` | layout_style/src/style/computed_style.rs:5173 | ComputedStyle::MarkerShouldBeInside (layout_style/src/style/computed_style.rs:271) | MarkerShouldBeInside 1 | a | other LayoutObject/ComputedStyle shim |
| 38 | `ComputedStyleShouldApplyAnyContainment` | layout_style/src/style/computed_style.rs:5253 | ComputedStyle::ShouldApplyAnyContainmentValue (layout_style/src/style/computed_style.rs:2588) | ShouldApplyAnyContainmentValue 2 | a | other LayoutObject/ComputedStyle shim |
| 39 | `ComputedStyleVisitedDependentColor` | layout_style/src/style/computed_style.rs:5183 | ComputedStyle::VisitedDependentColor (layout_style/src/style/computed_style.rs:3052) | VisitedDependentColor 10 | a | other LayoutObject/ComputedStyle shim |
| 40 | `ComputedStyleVisitedDependentColorWithUnvisited` | layout_style/src/style/computed_style.rs:5189 | ComputedStyle::VisitedDependentColorWithUnvisited (layout_style/src/style/computed_style.rs:3064) | VisitedDependentColorWithUnvisited 0 | a | other LayoutObject/ComputedStyle shim |
| 41 | `ContentDataConcatenateAltText` | layout_style/src/style/content_data.rs:792 | pub trait ContentData::ConcatenateAltText (layout_style/src/style/content_data.rs:91) | ConcatenateAltText 0 | b | CSS counters / content alt text |
| 42 | `ContentDataHasAltCounterContent` | layout_style/src/style/content_data.rs:791 | pub trait ContentData::HasAltCounterContent (layout_style/src/style/content_data.rs:87) | HasAltCounterContent 0 | b | CSS counters / content alt text |
| 43 | `CounterContentDataConstruct` | layout_style/src/style/content_data.rs:793 | CounterContentData::new (layout_style/src/style/content_data.rs:442) | new 30661 | b | CSS counters / content alt text |
| 44 | `CounterContentDataResolveCounterStyle` | layout_style/src/style/content_data.rs:800 | CounterContentData::ResolveCounterStyle (layout_style/src/style/content_data.rs:473) | ResolveCounterStyle 0 | b | CSS counters / content alt text |
| 45 | `CountersAttachmentContextCalculateInitialValueForReversed` | layout_core/src/internal/css/counters_attachment_context.rs:33 | CountersAttachmentContext::CalculateInitialValueForReversed (layout_core/src/internal/css/counters_attachment_context.rs:181) | CalculateInitialValueForReversed 0 | b | CSS counters / content alt text |
| 46 | `CountersAttachmentContextDeepClone` | layout_core/src/internal/css/counters_attachment_context.rs:13 | CountersAttachmentContext::DeepClone (layout_core/src/internal/css/counters_attachment_context.rs:144) | DeepClone 0 | b | CSS counters / content alt text |
| 47 | `CountersAttachmentContextElementGeneratesListItemCounter` | layout_core/src/internal/css/counters_attachment_context.rs:32 | CountersAttachmentContext::ElementGeneratesListItemCounter (layout_core/src/internal/css/counters_attachment_context.rs:173) | ElementGeneratesListItemCounter 0 | b | CSS counters / content alt text |
| 48 | `CountersAttachmentContextEnterObject` | layout_core/src/internal/css/counters_attachment_context.rs:16 | CountersAttachmentContext::EnterObject (layout_core/src/internal/css/counters_attachment_context.rs:148) | EnterObject 0 | b | CSS counters / content alt text |
| 49 | `CountersAttachmentContextGetCounterValues` | layout_core/src/internal/css/counters_attachment_context.rs:26 | CountersAttachmentContext::GetCounterValues (layout_core/src/internal/css/counters_attachment_context.rs:161) | GetCounterValues 0 | b | CSS counters / content alt text |
| 50 | `CountersAttachmentContextLeaveObject` | layout_core/src/internal/css/counters_attachment_context.rs:21 | CountersAttachmentContext::LeaveObject (layout_core/src/internal/css/counters_attachment_context.rs:152) | LeaveObject 0 | b | CSS counters / content alt text |
| 51 | `CountersAttachmentContextNew` | layout_core/src/internal/css/counters_attachment_context.rs:12 | CountersAttachmentContext::new (layout_core/src/internal/css/counters_attachment_context.rs:134) | new 30661 | b | CSS counters / content alt text |
| 52 | `CSSVariableDataHash` | layout_style/src/style/forward.rs:123 | CSSVariableData::Hash (layout_style/src/style/forward.rs:112) | Hash 62 | a | custom properties on ComputedStyle |
| 53 | `DispatchLayoutObjectDescendantIsolationRequirementsChanged` | layout_core/src/internal/layout_object.rs:810 | LayoutObject::DescendantIsolationRequirementsChanged (layout_core/src/internal/layout_object.rs:1459) | DescendantIsolationRequirementsChanged 0 | a | paint / hit-test / geometry shim |
| 54 | `DispatchLayoutObjectHasNonIsolatedBlendingDescendants` | layout_core/src/internal/layout_object.rs:809 | LayoutObject::HasNonIsolatedBlendingDescendants (layout_core/src/internal/layout_object.rs:1450) | HasNonIsolatedBlendingDescendants 1 | a | paint / hit-test / geometry shim |
| 55 | `DispatchLayoutObjectInLayoutNGInlineFormattingContextWillChange` | layout_core/src/internal/layout_object.rs:701<br>layout_core/src/invalidation/layout_tree_invalidation.rs:21 | LayoutObject::InLayoutNGInlineFormattingContextWillChange (layout_core/src/invalidation/layout_tree_invalidation.rs:51) | InLayoutNGInlineFormattingContextWillChange 3 | a | debug/test helper |
| 56 | `DispatchLayoutObjectIsFrame` | layout_core/src/internal/layout_object.rs:822 | LayoutObject::IsFrame (layout_core/src/internal/layout_object.rs:3648) | IsFrame 0 | a | other LayoutObject/ComputedStyle shim |
| 57 | `DispatchLayoutObjectIsLayoutCustomScrollbarPart` | layout_core/src/internal/layout_object.rs:830 | LayoutObject::IsLayoutCustomScrollbarPart (layout_core/src/internal/layout_object.rs:3736) | IsLayoutCustomScrollbarPart 0 | a | other LayoutObject/ComputedStyle shim |
| 58 | `DispatchLayoutObjectIsLayoutImageReplacement` | layout_core/src/internal/layout_object.rs:829 | LayoutObject::IsLayoutImageReplacement (layout_core/src/internal/layout_object.rs:3725) | IsLayoutImageReplacement 0 | a | other LayoutObject/ComputedStyle shim |
| 59 | `DispatchLayoutObjectIsListMarkerImage` | layout_core/src/internal/layout_object.rs:823 | LayoutObject::IsListMarkerImage (layout_core/src/internal/layout_object.rs:3659) | IsListMarkerImage 0 | a | other LayoutObject/ComputedStyle shim |
| 60 | `DispatchLayoutObjectIsMathMLRoot` | layout_core/src/internal/layout_object.rs:824 | LayoutObject::IsMathMLRoot (layout_core/src/internal/layout_object.rs:3670) | IsMathMLRoot 1 | b | MathML layout |
| 61 | `DispatchLayoutObjectIsProgress` | layout_core/src/internal/layout_object.rs:825 | LayoutObject::IsProgress (layout_core/src/internal/layout_object.rs:3681) | IsProgress 0 | a | other LayoutObject/ComputedStyle shim |
| 62 | `DispatchLayoutObjectIsSVGFilterPrimitive` | layout_core/src/internal/layout_object.rs:840 | LayoutObject::IsSVGFilterPrimitive (layout_core/src/internal/layout_object.rs:3846) | IsSVGFilterPrimitive 0 | a | other LayoutObject/ComputedStyle shim |
| 63 | `DispatchLayoutObjectIsViewTransitionContent` | layout_core/src/internal/layout_object.rs:833 | LayoutObject::IsViewTransitionContent (layout_core/src/internal/layout_object.rs:3769) | IsViewTransitionContent 0 | a | other LayoutObject/ComputedStyle shim |
| 64 | `DispatchLayoutObjectSetNeedsBoundariesUpdate` | layout_core/src/internal/layout_object.rs:807 | LayoutObject::SetNeedsBoundariesUpdate (layout_core/src/internal/layout_object.rs:1429) | SetNeedsBoundariesUpdate 0 | a | paint / hit-test / geometry shim |
| 65 | `DispatchLayoutObjectSetNeedsTransformUpdate` | layout_core/src/internal/layout_object.rs:806 | LayoutObject::SetNeedsTransformUpdate (layout_core/src/internal/layout_object.rs:1420) | SetNeedsTransformUpdate 0 | a | paint / hit-test / geometry shim |
| 66 | `DocumentMarkerGetType` | layout_core/src/internal/document_marker.rs:316 | DocumentMarker::GetType (layout_core/src/internal/document_marker.rs:243) | GetType 1270 | a | other LayoutObject/ComputedStyle shim |
| 67 | `FragmentRepeaterFinishRepeatableRoot` | layout_core/src/internal/fragment_repeater.rs:78 | FragmentRepeater::FinishRepeatableRoot (layout_core/src/internal/fragment_repeater.rs:29) | FinishRepeatableRoot 2 | a | duplicate |
| 68 | `FragmentRepeaterLayoutRepeatableRoot` | layout_core/src/internal/fragment_repeater.rs:73 | FragmentRepeater::LayoutRepeatableRoot (layout_core/src/internal/fragment_repeater.rs:24) | LayoutRepeatableRoot 3 | a | duplicate |
| 69 | `GridTrackListClear` | layout_style/src/style/grid_track_list.rs:354 | GridTrackList::Clear (layout_style/src/style/grid_track_list.rs:334) | Clear 199 | a | other LayoutObject/ComputedStyle shim |
| 70 | `GridTrackListToString` | layout_style/src/style/grid_track_list.rs:127 | GridTrackList::ToString (layout_style/src/style/grid_track_list.rs:120) | ToString 370 | a | debug/test helper |
| 71 | `GridTrackRepeaterToString` | layout_style/src/style/grid_track_list.rs:69 | GridTrackRepeater::ToString (layout_style/src/style/grid_track_list.rs:62) | ToString 370 | a | debug/test helper |
| 72 | `InlineItemResultToString` | layout_core/src/inline/inline_item_result_interface.rs:32 | InlineItemResult::ToString (layout_core/src/inline/inline_item_result_interface.rs:228) | ToString 370 | a | debug/test helper (body not exported) (unexported body: layout_algo_inline/src/inline_item_result.rs:89) |
| 73 | `InlineItemSegmentsToRanges` | layout_fragment/src/inline/inline_item_segment_interface.rs:51 | InlineItemSegments::ToRanges (layout_fragment/src/inline/inline_item_segment_interface.rs:231) | ToRanges 0 | a | translated body not exported (unexported body: layout_algo_inline/src/inline_item_segment.rs:161) |
| 74 | `InlineItemToString` | layout_fragment/src/inline/inline_item_interface.rs:48 | InlineItem::ToString (layout_fragment/src/inline/inline_item_interface.rs:489)<br>LogicalLineItem::fmt (layout_core/src/fragment/logical_line_item.rs:612) | ToString 370 fmt 67 | a | debug/test helper (body not exported) (unexported body: layout_core/src/fragment/logical_line_item.rs:581) |
| 75 | `InlineItemTypeToStringFromInline` | layout_fragment/src/inline/inline_item_interface.rs:31 | InlineItem::InlineItemTypeToString (layout_fragment/src/inline/inline_item_interface.rs:206) | InlineItemTypeToString 2 | a | debug/test helper |
| 76 | `InlineNodeCheckConsistency` | layout_core/src/inline/inline_node_interface.rs:64 | InlineNode::CheckConsistency (layout_core/src/inline/inline_node_interface.rs:286) | CheckConsistency 10 | a | debug/test helper (body not exported) (unexported body: layout_algo_inline/src/inline_node.rs:2597) |
| 77 | `InlineNodeComputeOffsetMapping` | layout_core/src/inline/inline_node_interface.rs:125 | ReusingTextShaper::InlineNodeComputeOffsetMappingIfNeeded (layout_algo_inline/src/inline_node.rs:626)<br>InlineNode::ComputeOffsetMapping (layout_core/src/inline/inline_node_interface.rs:397) | ComputeOffsetMapping 0 InlineNodeComputeOffsetMappingIfNeeded 1 | a | translated body not exported (unexported body: layout_algo_inline/src/inline_node.rs:632) |
| 78 | `InlineNodeEnsureData` | layout_core/src/inline/inline_node_interface.rs:123 | InlineNode::EnsureData (layout_core/src/inline/inline_node_interface.rs:391) | EnsureData 0 | a | translated body not exported (unexported body: layout_algo_inline/src/inline_node.rs:2088) |
| 79 | `InlineNodeFindSvgTextChunks` | layout_core/src/inline/inline_node_interface.rs:81 | ReusingTextShaper::InlineNodeCollectInlines (layout_algo_inline/src/inline_node.rs:521)<br>InlineNode::FindSvgTextChunks (layout_core/src/inline/inline_node_interface.rs:330) | FindSvgTextChunks 0 InlineNodeCollectInlines 1 | a | translated body not exported (unexported body: layout_algo_inline/src/inline_node.rs:554) |
| 80 | `InlineNodeInvalidatePrepareLayoutForTest` | layout_core/src/inline/inline_node_interface.rs:42 | InlineNode::InvalidatePrepareLayoutForTest (layout_core/src/inline/inline_node_interface.rs:224) | InvalidatePrepareLayoutForTest 0 | a | debug/test helper (body not exported) (unexported body: layout_algo_inline/src/inline_node.rs:1148) |
| 81 | `InlineNodeMaybeDirtyData` | layout_core/src/inline/inline_node_interface.rs:122 | InlineNode::MaybeDirtyData (layout_core/src/inline/inline_node_interface.rs:388) | MaybeDirtyData 0 | a | translated body not exported (unexported body: layout_algo_inline/src/inline_node.rs:1234) |
| 82 | `InlineNodeNeedsShapingForTesting` | layout_core/src/inline/inline_node_interface.rs:72 | InlineNode::NeedsShapingForTesting (layout_core/src/inline/inline_node_interface.rs:304) | NeedsShapingForTesting 0 | a | debug/test helper (body not exported) (unexported body: layout_algo_inline/src/inline_node.rs:2592) |
| 83 | `InlineNodeSetTextWithOffset` | layout_core/src/inline/inline_node_interface.rs:48 | InlineNode::SetTextWithOffset (layout_core/src/inline/inline_node_interface.rs:248) | SetTextWithOffset 0 | b | incremental text edit (body not exported) (unexported body: layout_algo_inline/src/inline_node.rs:986) |
| 84 | `LayoutBlockFlowInvalidateDisplayItemClients` | layout_core/src/block/layout_block_flow.rs:88 | LayoutBlockFlow::InvalidateDisplayItemClients (layout_core/src/block/layout_block_flow.rs:445) | InvalidateDisplayItemClients 0 | a | paint / hit-test / geometry shim |
| 85 | `LayoutBlockFlowNodeForHitTest` | layout_core/src/block/layout_block_flow.rs:92 | LayoutBlockFlow::NodeForHitTest (layout_core/src/block/layout_block_flow.rs:451) | NodeForHitTest 22 | a | paint / hit-test / geometry shim |
| 86 | `LayoutBlockFlowPositionForPoint` | layout_core/src/block/layout_block_flow.rs:81 | LayoutBlockFlow::PositionForPoint (layout_core/src/block/layout_block_flow.rs:433) | PositionForPoint 16 | a | paint / hit-test / geometry shim |
| 87 | `LayoutBlockFlowSetShouldDoFullPaintInvalidationForFirstLine` | layout_core/src/block/layout_block_flow.rs:80 | LayoutBlockFlow::SetShouldDoFullPaintInvalidationForFirstLine (layout_core/src/block/layout_block_flow.rs:427) | SetShouldDoFullPaintInvalidationForFirstLine 0 | a | paint / hit-test / geometry shim |
| 88 | `LayoutBlockFlowShouldMoveCaretToHorizontalBoundaryWhenPastTopOrBottom` | layout_core/src/block/layout_block_flow.rs:85 | LayoutBlockFlow::ShouldMoveCaretToHorizontalBoundaryWhenPastTopOrBottom (layout_core/src/block/layout_block_flow.rs:439) | ShouldMoveCaretToHorizontalBoundaryWhenPastTopOrBottom 5 | a | paint / hit-test / geometry shim |
| 89 | `LayoutBlockImageChanged` | layout_core/src/block/layout_block.rs:69 | LayoutBlock::ImageChanged (layout_core/src/block/layout_block.rs:484) | ImageChanged 0 | a | paint / hit-test / geometry shim |
| 90 | `LayoutBlockInvalidatePaint` | layout_core/src/block/layout_block.rs:68 | LayoutBlock::InvalidatePaint (layout_core/src/block/layout_block.rs:478) | InvalidatePaint 30 | a | paint / hit-test / geometry shim |
| 91 | `LayoutBlockNodeAtPoint` | layout_core/src/block/layout_block.rs:61 | LayoutBlock::NodeAtPoint (layout_core/src/block/layout_block.rs:445) | NodeAtPoint 20 | a | paint / hit-test / geometry shim |
| 92 | `LayoutBlockPaint` | layout_core/src/block/layout_block.rs:60 | LayoutBlock::Paint (layout_core/src/block/layout_block.rs:433) | Paint 62 | a | paint / hit-test / geometry shim |
| 93 | `LayoutBlockPositionForPoint` | layout_core/src/block/layout_block.rs:56 | LayoutBlock::PositionForPoint (layout_core/src/block/layout_block.rs:427) | PositionForPoint 16 | a | paint / hit-test / geometry shim |
| 94 | `LayoutBlockPositionForPointIfOutsideAtomicInlineLevel` | layout_core/src/block/layout_block.rs:74 | LayoutBlock::PositionForPointIfOutsideAtomicInlineLevel (layout_core/src/block/layout_block.rs:509) | PositionForPointIfOutsideAtomicInlineLevel 0 | a | paint / hit-test / geometry shim |
| 95 | `LayoutBoxClearPaintFlagsProvider` | layout_core/src/internal/layout_box.rs:269 | LayoutBox::ClearPaintFlags (layout_core/src/internal/layout_box.rs:786) | ClearPaintFlags 3 | a | paint / hit-test / geometry shim |
| 96 | `LayoutBoxComputeBackgroundPaintLocationProvider` | layout_core/src/internal/layout_box.rs:291 | LayoutBox::ComputeBackgroundPaintLocation (layout_core/src/internal/layout_box.rs:841) | ComputeBackgroundPaintLocation 0 | a | paint / hit-test / geometry shim |
| 97 | `LayoutBoxComputeCanCompositeBackgroundAttachmentFixedProvider` | layout_core/src/internal/layout_box.rs:326 | LayoutBox::ComputeCanCompositeBackgroundAttachmentFixed (layout_core/src/internal/layout_box.rs:511) | ComputeCanCompositeBackgroundAttachmentFixed 0 | a | paint / hit-test / geometry shim |
| 98 | `LayoutBoxGetCustomLayoutChildProvider` | layout_core/src/internal/layout_box.rs:309 | LayoutBox::GetCustomLayoutChild (layout_core/src/internal/layout_box.rs:530) | GetCustomLayoutChild 0 | b | CSS Layout API |
| 99 | `LayoutBoxHasHitTestableOverflowProvider` | layout_core/src/internal/layout_box.rs:255 | LayoutBox::HasHitTestableOverflow (layout_core/src/internal/layout_box.rs:644) | HasHitTestableOverflow 0 | a | paint / hit-test / geometry shim |
| 100 | `LayoutBoxHitTestAllPhasesProvider` | layout_core/src/internal/layout_box.rs:243 | LayoutBox::HitTestAllPhases (layout_core/src/internal/layout_box.rs:631) | HitTestAllPhases 5 | a | paint / hit-test / geometry shim |
| 101 | `LayoutBoxHitTestOverflowControlProvider` | layout_core/src/internal/layout_box.rs:261 | LayoutBox::HitTestOverflowControl (layout_core/src/internal/layout_box.rs:774) | HitTestOverflowControl 8 | a | paint / hit-test / geometry shim |
| 102 | `LayoutBoxImageChangedProvider` | layout_core/src/internal/layout_box.rs:315 | LayoutBox::ImageChanged (layout_core/src/internal/layout_box.rs:494) | ImageChanged 0 | a | paint / hit-test / geometry shim |
| 103 | `LayoutBoxInvalidateItemsProvider` | layout_core/src/internal/layout_box.rs:256 | LayoutBox::InvalidateItems (layout_core/src/internal/layout_box.rs:649) | InvalidateItems 0 | a | other LayoutObject/ComputedStyle shim |
| 104 | `LayoutBoxInvalidatePaintForTickmarksProvider` | layout_core/src/internal/layout_box.rs:271 | LayoutBox::InvalidatePaintForTickmarks (layout_core/src/internal/layout_box.rs:795) | InvalidatePaintForTickmarks 0 | a | paint / hit-test / geometry shim |
| 105 | `LayoutBoxInvalidatePaintProvider` | layout_core/src/internal/layout_box.rs:308 | LayoutBox::InvalidatePaint (layout_core/src/internal/layout_box.rs:547) | InvalidatePaint 30 | a | paint / hit-test / geometry shim |
| 106 | `LayoutBoxIsEligibleForPaintOrLayoutContainmentProvider` | layout_core/src/internal/layout_box.rs:324 | LayoutBox::IsEligibleForPaintOrLayoutContainment (layout_core/src/internal/layout_box.rs:503) | IsEligibleForPaintOrLayoutContainment 5 | a | paint / hit-test / geometry shim |
| 107 | `LayoutBoxIsEligibleForSizeContainmentProvider` | layout_core/src/internal/layout_box.rs:325 | LayoutBox::IsEligibleForSizeContainment (layout_core/src/internal/layout_box.rs:507) | IsEligibleForSizeContainment 6 | a | other LayoutObject/ComputedStyle shim |
| 108 | `LayoutBoxLayoutSubtreeRootProvider` | layout_core/src/internal/layout_box.rs:242 | LayoutBox::LayoutSubtreeRoot (layout_core/src/internal/layout_box.rs:622) | LayoutSubtreeRoot 0 | a | other LayoutObject/ComputedStyle shim |
| 109 | `LayoutBoxMayIntersectProvider` | layout_core/src/internal/layout_box.rs:249 | LayoutBox::MayIntersect (layout_core/src/internal/layout_box.rs:640) | MayIntersect 15 | a | paint / hit-test / geometry shim |
| 110 | `LayoutBoxModelObjectAdjustedPositionRelativeTo` | layout_core/src/internal/layout_box_model_object.rs:56 | LayoutBoxModelObject::AdjustedPositionRelativeTo (layout_core/src/internal/layout_box_model_object.rs:326) | AdjustedPositionRelativeTo 1 | b | offsetParent / offsetLeft/Top |
| 111 | `LayoutBoxModelObjectComputeStickyPositionConstraints` | layout_core/src/internal/layout_box_model_object.rs:39 | LayoutBoxModelObject::ComputeStickyPositionConstraints (layout_core/src/internal/layout_box_model_object.rs:223) | ComputeStickyPositionConstraints 0 | a | paint / hit-test / geometry shim |
| 112 | `LayoutBoxModelObjectUpdateCanCompositeBackgroundAttachmentFixed` | layout_core/src/internal/layout_box_model_object.rs:52 | LayoutBoxModelObject::UpdateCanCompositeBackgroundAttachmentFixed (layout_core/src/internal/layout_box_model_object.rs:317) | UpdateCanCompositeBackgroundAttachmentFixed 0 | a | paint / hit-test / geometry shim |
| 113 | `LayoutBoxMutableClearPreviousGapGeometriesProvider` | layout_core/src/internal/layout_box.rs:224 | LayoutBoxMutableForPainting::ClearPreviousGapGeometries (layout_core/src/internal/layout_box.rs:398) | ClearPreviousGapGeometries 0 | a | paint / hit-test / geometry shim |
| 114 | `LayoutBoxMutableSavePreviousGapGeometriesProvider` | layout_core/src/internal/layout_box.rs:223 | LayoutBoxMutableForPainting::SavePreviousGapGeometries (layout_core/src/internal/layout_box.rs:394) | SavePreviousGapGeometries 0 | a | paint / hit-test / geometry shim |
| 115 | `LayoutBoxMutableSetPreviousGeometryForLayoutShiftTrackingProvider` | layout_core/src/internal/layout_box.rs:225 | LayoutBoxMutableForPainting::SetPreviousGeometryForLayoutShiftTracking (layout_core/src/internal/layout_box.rs:408) | SetPreviousGeometryForLayoutShiftTracking 0 | a | other LayoutObject/ComputedStyle shim |
| 116 | `LayoutBoxMutableUpdateBackgroundPaintLocationProvider` | layout_core/src/internal/layout_box.rs:231 | LayoutBoxMutableForPainting::UpdateBackgroundPaintLocation (layout_core/src/internal/layout_box.rs:419) | UpdateBackgroundPaintLocation 0 | a | paint / hit-test / geometry shim |
| 117 | `LayoutBoxNonOverflowingScrollRangesProvider` | layout_core/src/internal/layout_box.rs:272 | LayoutBox::NonOverflowingScrollRanges (layout_core/src/internal/layout_box.rs:799) | NonOverflowingScrollRanges 2 | c | anchor position-try scroll ranges |
| 118 | `LayoutBoxNotifyContainingDisplayLocksForAnchorPositioningProvider` | layout_core/src/internal/layout_box.rs:278 | LayoutBox::NotifyContainingDisplayLocksForAnchorPositioning (layout_core/src/internal/layout_box.rs:812) | NotifyContainingDisplayLocksForAnchorPositioning 0 | a | other LayoutObject/ComputedStyle shim |
| 119 | `LayoutBoxOverrideTickmarksProvider` | layout_core/src/internal/layout_box.rs:270 | LayoutBox::OverrideTickmarks (layout_core/src/internal/layout_box.rs:791) | OverrideTickmarks 0 | a | paint / hit-test / geometry shim |
| 120 | `LayoutBoxPaintProvider` | layout_core/src/internal/layout_box.rs:307 | LayoutBox::Paint (layout_core/src/internal/layout_box.rs:535) | Paint 62 | a | paint / hit-test / geometry shim |
| 121 | `LayoutBoxScrollerFromScrollMarkerGroupProvider` | layout_core/src/internal/layout_box.rs:241 | LayoutBox::ScrollerFromScrollMarkerGroup (layout_core/src/internal/layout_box.rs:618) | ScrollerFromScrollMarkerGroup 0 | a | other LayoutObject/ComputedStyle shim |
| 122 | `LayoutBoxUpdateScrollSnapMappingAfterStyleChangeProvider` | layout_core/src/internal/layout_box.rs:303 | LayoutBox::UpdateScrollSnapMappingAfterStyleChange (layout_core/src/internal/layout_box.rs:600) | UpdateScrollSnapMappingAfterStyleChange 0 | a | other LayoutObject/ComputedStyle shim |
| 123 | `LayoutCounterDestructFromInline` | layout_algo_inline/src/layout_counter.rs:135 | LayoutCounter::drop (layout_algo_inline/src/layout_counter.rs:50) | drop 1167 | b | CSS counters / content alt text |
| 124 | `LayoutCounterGenerateCounterTextFromInline` | layout_algo_inline/src/layout_counter.rs:137 | LayoutCounter::GenerateCounterText (layout_algo_inline/src/layout_counter.rs:75) | GenerateCounterText 0 | b | CSS counters / content alt text |
| 125 | `LayoutCounterTraceFromInline` | layout_algo_inline/src/layout_counter.rs:136 | LayoutCounter::Trace (layout_algo_inline/src/layout_counter.rs:56) | Trace 801 | b | CSS counters / content alt text |
| 126 | `LayoutCounterUpdateCounterFromInline` | layout_algo_inline/src/layout_counter.rs:142 | LayoutCounter::UpdateCounter (layout_algo_inline/src/layout_counter.rs:79) | UpdateCounter 0 | b | CSS counters / content alt text |
| 127 | `LayoutInlineDebugRect` | layout_core/src/inline/layout_inline.rs:89 | LayoutInline::DebugRect (layout_core/src/inline/layout_inline.rs:410) | DebugRect 0 | a | debug/test helper |
| 128 | `LayoutInlineHitTestCulledInline` | layout_core/src/inline/layout_inline.rs:82 | LayoutInline::HitTestCulledInline (layout_core/src/inline/layout_inline.rs:378) | HitTestCulledInline 4 | a | paint / hit-test / geometry shim |
| 129 | `LayoutInlineInvalidateDisplayItemClients` | layout_core/src/inline/layout_inline.rs:102 | LayoutInline::InvalidateDisplayItemClients (layout_core/src/inline/layout_inline.rs:430) | InvalidateDisplayItemClients 0 | a | paint / hit-test / geometry shim |
| 130 | `LayoutInlineLocalBoundingBoxRectForAccessibility` | layout_core/src/inline/layout_inline.rs:68 | LayoutInline::LocalBoundingBoxRectForAccessibility (layout_core/src/inline/layout_inline.rs:250) | LocalBoundingBoxRectForAccessibility 3 | b | accessibility |
| 131 | `LayoutInputNodeGetCustomLayoutChildFromCustom` | layout_core/src/internal/layout_input_node.rs:340 | LayoutInputNode::GetCustomLayoutChild (layout_core/src/internal/layout_input_node.rs:146) | GetCustomLayoutChild 0 | b | CSS Layout API |
| 132 | `LayoutObjectAbsoluteBoundingBoxRectForUnboundedElementProvider` | layout_core/src/internal/layout_object.rs:641 | LayoutObject::AbsoluteBoundingBoxRectForUnboundedElement (layout_core/src/fragment/layout_object_bounding_geometry.rs:111) | AbsoluteBoundingBoxRectForUnboundedElement 0 | a | other LayoutObject/ComputedStyle shim |
| 133 | `LayoutObjectAddDraggableRegionsProvider` | layout_core/src/internal/layout_object.rs:706 | LayoutObject::AddDraggableRegions (layout_core/src/fragment/layout_object_paint_dispatch.rs:54) | AddDraggableRegions 0 | a | other LayoutObject/ComputedStyle shim |
| 134 | `LayoutObjectAddLayersProvider` | layout_core/src/internal/layout_object.rs:868 | LayoutObject::AddLayers (layout_core/src/invalidation/layout_object_layer_mutation.rs:10) | AddLayers 2 | a | paint / hit-test / geometry shim |
| 135 | `LayoutObjectCollectOutlineRectsAndAdvanceProvider` | layout_core/src/internal/layout_object.rs:680 | LayoutObject::CollectOutlineRectsAndAdvance (layout_core/src/fragment/layout_object_paint_dispatch.rs:26) | CollectOutlineRectsAndAdvance 0 | a | paint / hit-test / geometry shim |
| 136 | `LayoutObjectCreateBlockFlowOrListItemProvider` | layout_core/src/internal/layout_object.rs:894<br>layout_core/src/invalidation/layout_object_factory.rs:17 | LayoutObject::CreateBlockFlowOrListItem (layout_core/src/invalidation/layout_object_factory.rs:36) | CreateBlockFlowOrListItem 0 | a | other LayoutObject/ComputedStyle shim |
| 137 | `LayoutObjectCreateObjectProvider` | layout_core/src/internal/layout_object.rs:890<br>layout_core/src/invalidation/layout_object_factory.rs:13 | LayoutObject::CreateObject (layout_core/src/invalidation/layout_object_factory.rs:27) | CreateObject 0 | a | other LayoutObject/ComputedStyle shim |
| 138 | `LayoutObjectDebugRectProvider` | layout_core/src/internal/layout_object.rs:759 | LayoutObject::DebugRect (layout_core/src/fragment/layout_object_paint_dispatch.rs:148) | DebugRect 0 | a | debug/test helper |
| 139 | `LayoutObjectDecoratedNameProvider` | layout_core/src/internal/layout_object.rs:867 | LayoutObject::DecoratedName (layout_core/src/internal/layout_object.rs:3909) | DecoratedName 0 | a | debug/test helper |
| 140 | `LayoutObjectEnclosingNodeProvider` | layout_core/src/internal/layout_object.rs:899<br>layout_core/src/invalidation/layout_object_tree.rs:11 | LayoutObject::EnclosingNode (layout_core/src/invalidation/layout_object_tree.rs:30) | EnclosingNode 2 | a | other LayoutObject/ComputedStyle shim |
| 141 | `LayoutObjectFindNextLayerProvider` | layout_core/src/internal/layout_object.rs:875 | LayoutObject::FindNextLayer (layout_core/src/invalidation/layout_object_layer_mutation.rs:36) | FindNextLayer 1 | a | paint / hit-test / geometry shim |
| 142 | `LayoutObjectGeneratingNodeProvider` | layout_core/src/internal/layout_object.rs:898<br>layout_core/src/invalidation/layout_object_tree.rs:10 | LayoutObject::GeneratingNode (layout_core/src/invalidation/layout_object_tree.rs:24) | GeneratingNode 1 | a | other LayoutObject/ComputedStyle shim |
| 143 | `LayoutObjectGetCachedPseudoElementStyleProvider` | layout_core/src/internal/layout_object.rs:644 | LayoutObject::GetCachedPseudoElementStyle (layout_core/src/invalidation/layout_object_style_services.rs:24) | GetCachedPseudoElementStyle 2 | a | other LayoutObject/ComputedStyle shim |
| 144 | `LayoutObjectGetDisplayLockContextProvider` | layout_core/src/internal/layout_object.rs:685 | LayoutObject::GetDisplayLockContext (layout_core/src/invalidation/layout_object_display_lock.rs:44) | GetDisplayLockContext 0 | a | other LayoutObject/ComputedStyle shim |
| 145 | `LayoutObjectGetImageOrientationProvider` | layout_core/src/internal/layout_object.rs:964 | LayoutObject::GetImageOrientation (layout_core/src/fragment/layout_object_paint_state.rs:70) | GetImageOrientation 0 | a | other LayoutObject/ComputedStyle shim |
| 146 | `LayoutObjectGetPropertyContainerProvider` | layout_core/src/internal/layout_object.rs:732 | LayoutObject::GetPropertyContainer (layout_core/src/fragment/layout_object_fragment_queries.rs:142) | GetPropertyContainer 0 | a | paint / hit-test / geometry shim |
| 147 | `LayoutObjectGetUncachedPseudoElementStyleProvider` | layout_core/src/internal/layout_object.rs:648 | LayoutObject::GetUncachedPseudoElementStyle (layout_core/src/invalidation/layout_object_style_services.rs:30) | GetUncachedPseudoElementStyle 0 | a | other LayoutObject/ComputedStyle shim |
| 148 | `LayoutObjectHitTestAllPhasesProvider` | layout_core/src/internal/layout_object.rs:714 | LayoutObject::HitTestAllPhases (layout_core/src/fragment/layout_object_fragment_queries.rs:22) | HitTestAllPhases 5 | a | paint / hit-test / geometry shim |
| 149 | `LayoutObjectHitTestForOcclusionProvider` | layout_core/src/internal/layout_object.rs:676 | LayoutObject::HitTestForOcclusion (layout_core/src/fragment/layout_object_fragment_queries.rs:12) | HitTestForOcclusion 0 | a | paint / hit-test / geometry shim |
| 150 | `LayoutObjectInvalidateDisplayItemClientsProvider` | layout_core/src/internal/layout_object.rs:755 | LayoutObject::InvalidateDisplayItemClients (layout_core/src/invalidation/layout_object_paint_invalidation.rs:81) | InvalidateDisplayItemClients 0 | a | paint / hit-test / geometry shim |
| 151 | `LayoutObjectInvalidatePaintProvider` | layout_core/src/internal/layout_object.rs:751 | LayoutObject::InvalidatePaint (layout_core/src/invalidation/layout_object_paint_invalidation.rs:75) | InvalidatePaint 30 | a | paint / hit-test / geometry shim |
| 152 | `LayoutObjectInvalidateSelectionOnStyleChangeProvider` | layout_core/src/internal/layout_object.rs:922 | LayoutObject::InvalidateSelectionOnStyleChange (layout_core/src/invalidation/layout_object_prepaint_invalidation.rs:228) | InvalidateSelectionOnStyleChange 0 | a | paint / hit-test / geometry shim |
| 153 | `LayoutObjectInvalidateSubtreePositionTryProvider` | layout_core/src/internal/layout_object.rs:908 | LayoutObject::InvalidateSubtreePositionTry (layout_core/src/invalidation/layout_object_prepaint_invalidation.rs:176) | InvalidateSubtreePositionTry 0 | a | other LayoutObject/ComputedStyle shim |
| 154 | `LayoutObjectIsBackdropForOverscrollAreaParentProvider` | layout_core/src/internal/layout_object.rs:881 | LayoutObject::IsBackdropForOverscrollAreaParent (layout_core/src/fragment/layout_object_scroll_container_queries.rs:19) | IsBackdropForOverscrollAreaParent 0 | a | other LayoutObject/ComputedStyle shim |
| 155 | `LayoutObjectIsInListMarkerProvider` | layout_core/src/internal/layout_object.rs:888 | LayoutObject::IsInListMarker (layout_algo_list/src/layout_object_list_marker_queries.rs:18) | IsInListMarker 0 | a | other LayoutObject/ComputedStyle shim |
| 156 | `LayoutObjectIsInputButtonProvider` | layout_core/src/internal/layout_object.rs:882 | LayoutObject::IsInputButton (layout_core/src/internal/layout_object.rs:3922) | IsInputButton 0 | a | other LayoutObject/ComputedStyle shim |
| 157 | `LayoutObjectIsListMarkerForSummaryProvider` | layout_core/src/internal/layout_object.rs:887 | LayoutObject::IsListMarkerForSummary (layout_algo_list/src/layout_object_list_marker_queries.rs:12) | IsListMarkerForSummary 0 | a | other LayoutObject/ComputedStyle shim |
| 158 | `LayoutObjectIsMenuListProvider` | layout_core/src/internal/layout_object.rs:883 | LayoutObject::IsMenuList (layout_core/src/internal/layout_object.rs:3928) | IsMenuList 4 | a | other LayoutObject/ComputedStyle shim |
| 159 | `LayoutObjectIsScrollMarkerGroupBeforeProvider` | layout_core/src/internal/layout_object.rs:886 | LayoutObject::IsScrollMarkerGroupBefore (layout_core/src/fragment/layout_object_scroll_container_queries.rs:29) | IsScrollMarkerGroupBefore 0 | a | other LayoutObject/ComputedStyle shim |
| 160 | `LayoutObjectIsScrollMarkerProvider` | layout_core/src/internal/layout_object.rs:885 | LayoutObject::IsScrollMarker (layout_core/src/fragment/layout_object_scroll_container_queries.rs:24) | IsScrollMarker 0 | a | other LayoutObject/ComputedStyle shim |
| 161 | `LayoutObjectMapLocalToAncestorProvider` | layout_core/src/internal/layout_object.rs:738 | LayoutObject::MapLocalToAncestor (layout_core/src/fragment/layout_object_fragment_queries.rs:177) | MapLocalToAncestor 0 | a | paint / hit-test / geometry shim |
| 162 | `LayoutObjectMarkContainerTimingChangedProvider` | layout_core/src/internal/layout_object.rs:934 | LayoutObject::MarkContainerTimingChanged (layout_core/src/invalidation/layout_object_prepaint_invalidation.rs:264) | MarkContainerTimingChanged 0 | a | other LayoutObject/ComputedStyle shim |
| 163 | `LayoutObjectMarkSoftNavigationContextChangedProvider` | layout_core/src/internal/layout_object.rs:933 | LayoutObject::MarkSoftNavigationContextChanged (layout_core/src/invalidation/layout_object_prepaint_invalidation.rs:258) | MarkSoftNavigationContextChanged 0 | a | other LayoutObject/ComputedStyle shim |
| 164 | `LayoutObjectMoveLayersProvider` | layout_core/src/internal/layout_object.rs:870 | LayoutObject::MoveLayers (layout_core/src/invalidation/layout_object_layer_mutation.rs:24) | MoveLayers 0 | a | paint / hit-test / geometry shim |
| 165 | `LayoutObjectNodeAtPointProvider` | layout_core/src/internal/layout_object.rs:725 | LayoutObject::NodeAtPoint (layout_core/src/fragment/layout_object_fragment_queries.rs:87) | NodeAtPoint 20 | a | paint / hit-test / geometry shim |
| 166 | `LayoutObjectNotifyPriorityScrollAnchorStatusChangedProvider` | layout_core/src/internal/layout_object.rs:889 | LayoutObject::NotifyPriorityScrollAnchorStatusChanged (layout_core/src/invalidation/layout_object_state_propagation.rs:7) | NotifyPriorityScrollAnchorStatusChanged 0 | a | other LayoutObject/ComputedStyle shim |
| 167 | `LayoutObjectOffsetParentProvider` | layout_core/src/block/layout_object_containers.rs:17<br>layout_core/src/internal/layout_object.rs:904 | LayoutObject::OffsetParent (layout_core/src/block/layout_object_containers.rs:81) | OffsetParent 1 | b | offsetParent / offsetLeft/Top |
| 168 | `LayoutObjectPaintProvider` | layout_core/src/internal/layout_object.rs:705 | LayoutObject::Paint (layout_core/src/fragment/layout_object_paint_dispatch.rs:50) | Paint 62 | a | paint / hit-test / geometry shim |
| 169 | `LayoutObjectRemoveLayersProvider` | layout_core/src/internal/layout_object.rs:869 | LayoutObject::RemoveLayers (layout_core/src/invalidation/layout_object_layer_mutation.rs:17) | RemoveLayers 0 | a | paint / hit-test / geometry shim |
| 170 | `LayoutObjectScrollParentProvider` | layout_core/src/block/layout_object_containers.rs:13<br>layout_core/src/internal/layout_object.rs:900 | LayoutObject::ScrollParent (layout_core/src/block/layout_object_containers.rs:71) | ScrollParent 1 | a | other LayoutObject/ComputedStyle shim |
| 171 | `LayoutObjectSetLayoutNeededForbiddenScopeConstructProvider` | layout_core/src/internal/layout_object.rs:670 | SetLayoutNeededForbiddenScope::new (layout_core/src/internal/layout_object.rs:1044) | new 30661 | a | unused constructor |
| 172 | `LayoutObjectSetLayoutNeededForbiddenScopeDestructProvider` | layout_core/src/internal/layout_object.rs:673 | SetLayoutNeededForbiddenScope::drop (layout_core/src/internal/layout_object.rs:1051) | drop 1167 | a | unused constructor |
| 173 | `LayoutObjectSetShouldInheritContainerTimingRootProvider` | layout_core/src/internal/layout_object.rs:861 | mod package_contract_tests::HasEffectiveAllowedTouchAction (layout_assembly/src/lib.rs:8407)<br>LayoutObject::SetShouldInheritContainerTimingRoot (layout_core/src/invalidation/layout_object_prepaint_invalidation.rs:144) | HasEffectiveAllowedTouchAction 0 SetShouldInheritContainerTimingRoot 1 | a | other LayoutObject/ComputedStyle shim |
| 174 | `LayoutObjectSetSVGDescendantMayHaveTransformRelatedOperationsProvider` | layout_core/src/internal/layout_object.rs:944 | LayoutObject::SetSVGDescendantMayHaveTransformRelatedOperations (layout_core/src/invalidation/layout_object_prepaint_invalidation.rs:276) | SetSVGDescendantMayHaveTransformRelatedOperations 0 | a | paint / hit-test / geometry shim |
| 175 | `LayoutObjectTreeNew` | layout_core/src/internal/layout_object_builder.rs:67 | LayoutObjectTree::new (layout_core/src/internal/layout_object_builder.rs:251) | new 30661 | a | unused constructor |
| 176 | `LayoutObjectUpdateAfterReinsertProvider` | layout_core/src/internal/layout_object.rs:947 | LayoutObject::UpdateAfterReinsert (layout_core/src/invalidation/layout_object_prepaint_invalidation.rs:282) | UpdateAfterReinsert 0 | a | other LayoutObject/ComputedStyle shim |
| 177 | `LayoutObjectUpdateHitTestResultProvider` | layout_core/src/internal/layout_object.rs:720 | LayoutObject::UpdateHitTestResult (layout_core/src/fragment/layout_object_fragment_queries.rs:72) | UpdateHitTestResult 4 | a | paint / hit-test / geometry shim |
| 178 | `LayoutObjectVisibleToHitTestRequestProvider` | layout_core/src/internal/layout_object.rs:912 | LayoutObject::VisibleToHitTestRequest (layout_core/src/fragment/layout_object_fragment_queries.rs:167) | VisibleToHitTestRequest 7 | a | paint / hit-test / geometry shim |
| 179 | `LayoutTextDebugRect` | layout_core/src/inline/layout_text_interface.rs:117 | LayoutText::DebugRect (layout_core/src/inline/layout_text_interface.rs:712) | DebugRect 0 | a | debug/test helper |
| 180 | `LayoutTextEnsureNodeId` | layout_core/src/inline/layout_text_interface.rs:126 | LayoutText::EnsureNodeId (layout_core/src/inline/layout_text_interface.rs:731) | EnsureNodeId 0 | a | paint / hit-test / geometry shim |
| 181 | `LayoutTextFirstAbstractInlineTextBox` | layout_core/src/inline/layout_text_interface.rs:116 | LayoutText::FirstAbstractInlineTextBox (layout_core/src/inline/layout_text_interface.rs:709) | FirstAbstractInlineTextBox 0 | b | accessibility |
| 182 | `LayoutTextFragmentBlockForAccompanyingFirstLetter` | layout_core/src/inline/layout_text_fragment.rs:26 | LayoutTextFragment::BlockForAccompanyingFirstLetter (layout_core/src/inline/layout_text_fragment.rs:221) | BlockForAccompanyingFirstLetter 0 | a | other LayoutObject/ComputedStyle shim |
| 183 | `LayoutTextFragmentOwnerNodeId` | layout_core/src/inline/layout_text_fragment.rs:34 | LayoutTextFragment::OwnerNodeId (layout_core/src/inline/layout_text_fragment.rs:245) | OwnerNodeId 5 | a | paint / hit-test / geometry shim |
| 184 | `LayoutTextFragmentUpdateHitTestResult` | layout_core/src/inline/layout_text_fragment.rs:29 | LayoutTextFragment::UpdateHitTestResult (layout_core/src/inline/layout_text_fragment.rs:242) | UpdateHitTestResult 4 | a | paint / hit-test / geometry shim |
| 185 | `LayoutTextGetOrResetContentCaptureManager` | layout_core/src/inline/layout_text_interface.rs:181 | LayoutText::GetOrResetContentCaptureManager (layout_core/src/inline/layout_text_interface.rs:859) | GetOrResetContentCaptureManager 0 | a | paint / hit-test / geometry shim |
| 186 | `LayoutTextGetTextBoxInfo` | layout_core/src/inline/layout_text_interface.rs:92 | LayoutText::GetTextBoxInfo (layout_core/src/inline/layout_text_interface.rs:625) | GetTextBoxInfo 0 | a | paint / hit-test / geometry shim |
| 187 | `LayoutTextInsertedIntoTree` | layout_core/src/inline/layout_text_fragment.rs:38 | LayoutTextFragment::InsertedIntoTree (layout_core/src/inline/layout_text_fragment.rs:217) | InsertedIntoTree 7 | a | other LayoutObject/ComputedStyle shim |
| 188 | `LayoutTextInvalidateDisplayItemClients` | layout_core/src/inline/layout_text_interface.rs:153 | LayoutText::InvalidateDisplayItemClients (layout_core/src/inline/layout_text_interface.rs:799) | InvalidateDisplayItemClients 0 | a | paint / hit-test / geometry shim |
| 189 | `LayoutTextInvalidatePaint` | layout_core/src/inline/layout_text_interface.rs:152 | LayoutText::InvalidatePaint (layout_core/src/inline/layout_text_interface.rs:796) | InvalidatePaint 30 | a | paint / hit-test / geometry shim |
| 190 | `LayoutTextLocalBoundingBoxRectForAccessibility` | layout_core/src/inline/layout_text_interface.rs:64 | LayoutText::LocalBoundingBoxRectForAccessibility (layout_core/src/inline/layout_text_interface.rs:552) | LocalBoundingBoxRectForAccessibility 3 | b | accessibility |
| 191 | `LayoutTextLocalQuadsInFlippedBlocksDirection` | layout_core/src/inline/layout_text_interface.rs:68 | LayoutText::LocalQuadsInFlippedBlocksDirection (layout_core/src/inline/layout_text_interface.rs:561) | LocalQuadsInFlippedBlocksDirection 1 | a | paint / hit-test / geometry shim |
| 192 | `LayoutTextLogicalStartingPointAndHeight` | layout_core/src/inline/layout_text_interface.rs:136 | LayoutText::LogicalStartingPointAndHeight (layout_core/src/inline/layout_text_interface.rs:752) | LogicalStartingPointAndHeight 0 | a | paint / hit-test / geometry shim |
| 193 | `LayoutTextSetTextWithOffset` | layout_core/src/inline/layout_text_interface.rs:102 | LayoutText::SetTextWithOffset (layout_core/src/inline/layout_text_interface.rs:661) | SetTextWithOffset 0 | b | incremental text edit |
| 194 | `LayoutTextVisualOverflowRect` | layout_core/src/inline/layout_text_interface.rs:80 | LayoutText::VisualOverflowRect (layout_core/src/inline/layout_text_interface.rs:581) | VisualOverflowRect 20 | a | paint / hit-test / geometry shim |
| 195 | `LayoutViewCacheScrollDimensionsProvider` | layout_core/src/internal/layout_view.rs:132 | LayoutView::CacheScrollDimensions (layout_core/src/commit/layout_view_commit.rs:25) | CacheScrollDimensions 0 | a | other LayoutObject/ComputedStyle shim |
| 196 | `LayoutViewClearHitTestCacheProvider` | layout_core/src/internal/layout_view.rs:109 | LayoutView::ClearHitTestCache (layout_core/src/fragment/layout_view_fragment_queries.rs:36) | ClearHitTestCache 1 | a | paint / hit-test / geometry shim |
| 197 | `LayoutViewCommitPendingSelectionProvider` | layout_core/src/internal/layout_view.rs:110 | LayoutView::CommitPendingSelection (layout_core/src/commit/layout_view_commit.rs:19) | CommitPendingSelection 0 | a | paint / hit-test / geometry shim |
| 198 | `LayoutViewCreateAnonymousPageLayoutObjectProvider` | layout_core/src/internal/layout_view.rs:95 | LayoutView::CreateAnonymousPageLayoutObject (layout_core/src/block/layout_view_block_queries.rs:39) | CreateAnonymousPageLayoutObject 0 | b | paged media / printing |
| 199 | `LayoutViewGetViewTransitionRootProvider` | layout_core/src/internal/layout_view.rs:131 | LayoutView::GetViewTransitionRoot (layout_core/src/fragment/layout_view_fragment_queries.rs:94) | GetViewTransitionRoot 0 | a | other LayoutObject/ComputedStyle shim |
| 200 | `LayoutViewHitTestNoLifecycleUpdateProvider` | layout_core/src/internal/layout_view.rs:104 | LayoutView::HitTestNoLifecycleUpdate (layout_core/src/fragment/layout_view_fragment_queries.rs:31) | HitTestNoLifecycleUpdate 8 | a | paint / hit-test / geometry shim |
| 201 | `LayoutViewHitTestProvider` | layout_core/src/internal/layout_view.rs:99 | LayoutView::HitTest (layout_core/src/fragment/layout_view_fragment_queries.rs:22) | HitTest 30 | a | paint / hit-test / geometry shim |
| 202 | `LayoutViewInvalidateLayoutForCounterStyleChangesProvider` | layout_core/src/internal/layout_view.rs:117 | LayoutView::InvalidateLayoutForCounterStyleChanges (layout_core/src/invalidation/layout_view_invalidation.rs:63) | InvalidateLayoutForCounterStyleChanges 0 | b | CSS counters / content alt text |
| 203 | `LayoutViewInvalidatePaintForViewAndDescendantsProvider` | layout_core/src/internal/layout_view.rs:118 | LayoutView::InvalidatePaintForViewAndDescendants (layout_core/src/invalidation/layout_view_invalidation.rs:69) | InvalidatePaintForViewAndDescendants 2 | a | paint / hit-test / geometry shim |
| 204 | `LayoutViewMapAncestorToLocalProvider` | layout_core/src/internal/layout_view.rs:119 | LayoutView::MapAncestorToLocal (layout_core/src/fragment/layout_view_fragment_queries.rs:80) | MapAncestorToLocal 4 | a | paint / hit-test / geometry shim |
| 205 | `LayoutViewMapLocalToAncestorProvider` | layout_core/src/internal/layout_view.rs:125 | LayoutView::MapLocalToAncestor (layout_core/src/fragment/layout_view_fragment_queries.rs:90) | MapLocalToAncestor 0 | a | paint / hit-test / geometry shim |
| 206 | `LayoutViewNamedPageAtIndexProvider` | layout_core/src/internal/layout_view.rs:116 | LayoutView::NamedPageAtIndex (layout_core/src/block/layout_view_block_queries.rs:45) | NamedPageAtIndex 0 | b | paged media / printing |
| 207 | `LayoutViewUpdateHitTestResultProvider` | layout_core/src/internal/layout_view.rs:111 | LayoutView::UpdateHitTestResult (layout_core/src/fragment/layout_view_fragment_queries.rs:70) | UpdateHitTestResult 4 | a | paint / hit-test / geometry shim |
| 208 | `ListItemOrdinalItemInsertedOrRemoved` | layout_algo_list/src/list_item_ordinal.rs:48 | ListItemOrdinal::ItemInsertedOrRemoved (layout_algo_list/src/list_item_ordinal.rs:43) | ItemInsertedOrRemoved 2 | b | CSS counters / content alt text |
| 209 | `ListStyleTypeDataCreateSymbolsFunction` | layout_style/src/style/list_style_type_data.rs:144 | ListStyleTypeData::CreateSymbolsFunction (layout_style/src/style/list_style_type_data.rs:87) | CreateSymbolsFunction 0 | b | CSS counters / content alt text |
| 210 | `MutableForPaintingSetShouldDoFullPaintInvalidationProvider` | layout_core/src/internal/layout_object.rs:935 | MutableForPainting::SetShouldDoFullPaintInvalidation (layout_core/src/internal/layout_object.rs:1093) | SetShouldDoFullPaintInvalidation 21 | a | paint / hit-test / geometry shim |
| 211 | `MutableForPaintingSetShouldDoFullPaintInvalidationWithoutLayoutChangeProvider` | layout_core/src/internal/layout_object.rs:939 | MutableForPainting::SetShouldDoFullPaintInvalidationWithoutLayoutChange (layout_core/src/internal/layout_object.rs:1103) | SetShouldDoFullPaintInvalidationWithoutLayoutChange 20 | a | paint / hit-test / geometry shim |
| 212 | `NGInlineFormattingContextOf` | layout_core/src/inline/offset_mapping.rs:290 | LayoutObjectConverter::NGInlineFormattingContextOfPosition (layout_core/src/inline/offset_mapping.rs:709) | NGInlineFormattingContextOfPosition 0 | c | OffsetMapping static queries |
| 213 | `OffsetMappingAcceptsPosition` | layout_core/src/inline/offset_mapping.rs:252 | OffsetMapping::AcceptsPosition (layout_core/src/inline/offset_mapping.rs:325) | AcceptsPosition 0 | c | OffsetMapping static queries |
| 214 | `OffsetMappingEndOfLastNonCollapsedContent` | layout_core/src/inline/offset_mapping.rs:274 | OffsetMapping::EndOfLastNonCollapsedContent (layout_core/src/inline/offset_mapping.rs:532) | EndOfLastNonCollapsedContent 0 | c | OffsetMapping static queries |
| 215 | `OffsetMappingForceGetFor` | layout_core/src/inline/offset_mapping.rs:254 | OffsetMapping::ForceGetFor (layout_core/src/inline/offset_mapping.rs:331) | ForceGetFor 1 | c | OffsetMapping static queries |
| 216 | `OffsetMappingGetCharacterBefore` | layout_core/src/inline/offset_mapping.rs:286 | OffsetMapping::GetCharacterBefore (layout_core/src/inline/offset_mapping.rs:541) | GetCharacterBefore 0 | c | OffsetMapping static queries |
| 217 | `OffsetMappingGetForPosition` | layout_core/src/inline/offset_mapping.rs:253 | OffsetMapping::GetForPosition (layout_core/src/inline/offset_mapping.rs:328) | GetForPosition 1 | c | OffsetMapping static queries |
| 218 | `OffsetMappingGetInlineFormattingContextOfPosition` | layout_core/src/inline/offset_mapping.rs:255 | OffsetMapping::GetInlineFormattingContextOfPosition (layout_core/src/inline/offset_mapping.rs:360) | GetInlineFormattingContextOfPosition 0 | c | OffsetMapping static queries |
| 219 | `OffsetMappingGetMappingUnitForPosition` | layout_core/src/inline/offset_mapping.rs:258 | OffsetMapping::GetMappingUnitForPosition (layout_core/src/inline/offset_mapping.rs:365) | GetMappingUnitForPosition 0 | c | OffsetMapping static queries |
| 220 | `OffsetMappingGetMappingUnitsForDOMRange` | layout_core/src/inline/offset_mapping.rs:262 | OffsetMapping::GetMappingUnitsForDOMRange (layout_core/src/inline/offset_mapping.rs:368) | GetMappingUnitsForDOMRange 0 | c | OffsetMapping static queries |
| 221 | `OffsetMappingGetTextContentOffset` | layout_core/src/inline/offset_mapping.rs:266 | OffsetMapping::GetTextContentOffset (layout_core/src/inline/offset_mapping.rs:526) | GetTextContentOffset 0 | c | OffsetMapping static queries |
| 222 | `OffsetMappingIsAfterNonCollapsedContent` | layout_core/src/inline/offset_mapping.rs:282 | OffsetMapping::IsAfterNonCollapsedContent (layout_core/src/inline/offset_mapping.rs:538) | IsAfterNonCollapsedContent 0 | c | OffsetMapping static queries |
| 223 | `OffsetMappingIsBeforeNonCollapsedContent` | layout_core/src/inline/offset_mapping.rs:278 | OffsetMapping::IsBeforeNonCollapsedContent (layout_core/src/inline/offset_mapping.rs:535) | IsBeforeNonCollapsedContent 1 | c | OffsetMapping static queries |
| 224 | `OffsetMappingStartOfNextNonCollapsedContent` | layout_core/src/inline/offset_mapping.rs:270 | OffsetMapping::StartOfNextNonCollapsedContent (layout_core/src/inline/offset_mapping.rs:529) | StartOfNextNonCollapsedContent 0 | c | OffsetMapping static queries |
| 225 | `PhysicalOffsetToString` | layout_fragment/src/physical_fragment.rs:55 | PhysicalOffset::ToString (layout_fragment/src/physical_fragment.rs:50) | ToString 370 | a | debug/test helper |
| 226 | `StyleColorColorFromKeyword` | layout_style/src/css/style_color.rs:763 | StyleColor::ColorFromKeyword (layout_style/src/css/style_color.rs:613) | ColorFromKeyword 2 | a | other LayoutObject/ComputedStyle shim |
| 227 | `StyleColorFormat` | layout_style/src/css/style_color.rs:717 | StyleColor::fmt (layout_style/src/css/style_color.rs:152) | fmt 67 | a | debug/test helper |
| 228 | `StyleColorIsColorKeyword` | layout_style/src/css/style_color.rs:769 | StyleColor::IsColorKeyword (layout_style/src/css/style_color.rs:616) | IsColorKeyword 1 | a | other LayoutObject/ComputedStyle shim |
| 229 | `StyleColorToCSSValue` | layout_style/src/css/style_color.rs:762<br>crates/style/style_resolve/src/style_builder_converter.rs:3394 | StyleColor::ToCSSValue (layout_style/src/css/style_color.rs:597)<br>StyleBuilderConverter::ComputeColorValue (crates/style/style_resolve/src/style_builder_converter.rs:4047) | ComputeColorValue 2 ToCSSValue 4 | a | other LayoutObject/ComputedStyle shim |
| 230 | `StyleHighlightDataStylesDependOnFunc` | layout_style/src/style/style_highlight_data.rs:213 | StyleHighlightData::StylesDependOnFunc (layout_style/src/style/style_highlight_data.rs:159) | StylesDependOnFunc 0 | a | other LayoutObject/ComputedStyle shim |
| 231 | `StyleInheritedVariablesSetData` | layout_style/src/style/style_inherited_variables.rs:61 | StyleInheritedVariables::SetData (layout_style/src/style/style_inherited_variables.rs:20) | SetData 19 | a | custom properties on ComputedStyle |
| 232 | `StyleInitialDataEquals` | layout_style/src/style/style_initial_data.rs:48 | StyleInitialData::Equals (layout_style/src/style/style_initial_data.rs:21) | Equals 144 | a | custom properties on ComputedStyle |
| 233 | `StyleNonInheritedVariablesSetData` | layout_style/src/style/style_non_inherited_variables.rs:58 | StyleNonInheritedVariables::SetData (layout_style/src/style/style_non_inherited_variables.rs:20) | SetData 19 | a | custom properties on ComputedStyle |
| 234 | `StyleSVGResourceAddClient` | layout_style/src/style/style_svg_resource.rs:67 | StyleSVGResource::AddClient (layout_style/src/style/style_svg_resource.rs:26) | AddClient 4 | c | SVG url() resource clients |
| 235 | `StyleSVGResourceConstruct` | layout_style/src/style/style_svg_resource.rs:62 | StyleSVGResource::new (layout_style/src/style/style_svg_resource.rs:15) | new 30661 | c | SVG url() resource clients |
| 236 | `StyleSVGResourceDestroy` | layout_style/src/style/style_svg_resource.rs:66 | StyleSVGResource::drop (layout_style/src/style/style_svg_resource.rs:57) | drop 1167 | c | SVG url() resource clients |
| 237 | `StyleSVGResourceRemoveClient` | layout_style/src/style/style_svg_resource.rs:68 | StyleSVGResource::RemoveClient (layout_style/src/style/style_svg_resource.rs:32) | RemoveClient 4 | c | SVG url() resource clients |
| 238 | `StyleVariablesFormat` | layout_style/src/style/style_variables.rs:511 | StyleVariables::fmt (layout_style/src/style/style_variables.rs:491) | fmt 67 | a | debug/test helper |
| 239 | `StyleVariablesSetData` | layout_style/src/style/style_variables.rs:501 | StyleVariables::SetData (layout_style/src/style/style_variables.rs:429) | SetData 19 | a | custom properties on ComputedStyle |
| 240 | `StyleVariablesSetValue` | layout_style/src/style/style_variables.rs:506 | StyleVariables::SetValue (layout_style/src/style/style_variables.rs:434) | SetValue 26 | a | custom properties on ComputedStyle |
| 241 | `TextFitToString` | layout_style/src/style/text_fit.rs:84 | TextFit::ToString (layout_style/src/style/text_fit.rs:79) | ToString 370 | a | debug/test helper |
| 242 | `UnresolvedAlphaColorConstruct` | layout_style/src/css/style_color.rs:755 | UnresolvedAlphaColor::new (layout_style/src/css/style_color.rs:406) | new 30661 | b | currentcolor-dependent relative/contrast/alpha colors |
| 243 | `UnresolvedAlphaColorResolve` | layout_style/src/css/style_color.rs:761 | UnresolvedAlphaColor::Resolve (layout_style/src/css/style_color.rs:413) | Resolve 203 | b | currentcolor-dependent relative/contrast/alpha colors |
| 244 | `UnresolvedAlphaColorToCSSValue` | layout_style/src/css/style_color.rs:760 | UnresolvedAlphaColor::ToCSSValue (layout_style/src/css/style_color.rs:410) | ToCSSValue 4 | b | currentcolor-dependent relative/contrast/alpha colors |
| 245 | `UnresolvedColorFunctionFormat` | layout_style/src/css/style_color.rs:721 | UnresolvedColorFunction::fmt (layout_style/src/css/style_color.rs:158) | fmt 67 | a | debug/test helper |
| 246 | `UnresolvedColorMixToCSSValue` | layout_style/src/css/style_color.rs:733 | UnresolvedColorMix::ToCSSValue (layout_style/src/css/style_color.rs:220) | ToCSSValue 4 | a | other LayoutObject/ComputedStyle shim |
| 247 | `UnresolvedContrastColorConstruct` | layout_style/src/css/style_color.rs:749 | UnresolvedContrastColor::new (layout_style/src/css/style_color.rs:366) | new 30661 | b | currentcolor-dependent relative/contrast/alpha colors |
| 248 | `UnresolvedContrastColorResolve` | layout_style/src/css/style_color.rs:751 | UnresolvedContrastColor::Resolve (layout_style/src/css/style_color.rs:373) | Resolve 203 | b | currentcolor-dependent relative/contrast/alpha colors |
| 249 | `UnresolvedContrastColorToCSSValue` | layout_style/src/css/style_color.rs:750 | UnresolvedContrastColor::ToCSSValue (layout_style/src/css/style_color.rs:370) | ToCSSValue 4 | b | currentcolor-dependent relative/contrast/alpha colors |
| 250 | `UnresolvedRelativeColorConstruct` | layout_style/src/css/style_color.rs:735 | UnresolvedRelativeColor::new (layout_style/src/css/style_color.rs:312) | new 30661 | b | currentcolor-dependent relative/contrast/alpha colors |
| 251 | `UnresolvedRelativeColorResolve` | layout_style/src/css/style_color.rs:745 | UnresolvedRelativeColor::Resolve (layout_style/src/css/style_color.rs:330) | Resolve 203 | b | currentcolor-dependent relative/contrast/alpha colors |
| 252 | `UnresolvedRelativeColorToCSSValue` | layout_style/src/css/style_color.rs:744 | UnresolvedRelativeColor::ToCSSValue (layout_style/src/css/style_color.rs:327) | ToCSSValue 4 | b | currentcolor-dependent relative/contrast/alpha colors |

## Build-log warnings per crate

Counts are de-duplicated warnings, grouped by the crate directory that owns the
file. "unused method" and "unused fn" count **warnings**: one warning may name
several items ("multiple methods are never used"). "unused fn" includes
`extern` declarations that nothing calls. "unused mut" means `variable does not
need to be mutable`. `paint_input_types` has no warnings of these kinds.

| Crate | unused fn | unused method | never-constructed struct/enum | never-constructed variant | unused field | unused const/static | unused type/trait | unused macro | unused import | unused variable | value never read | unused mut | unreachable code | other dead | Compiled via |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| layout | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 5 | 0 | 0 | 0 | 0 | 0 | layout |
| layout_algo_block | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 7 | 0 | 1 | 0 | 0 | 0 | layout_algo_block |
| layout_algo_flex | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | layout_algo_flex |
| layout_algo_float | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | layout_algo_float, layout_assembly |
| layout_algo_fragmentation | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | 0 | layout_assembly |
| layout_algo_grid | 2 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 6 | 0 | 0 | 0 | 0 | 0 | layout_algo_grid, layout_assembly |
| layout_algo_inline | 19 | 5 | 0 | 0 | 2 | 0 | 0 | 0 | 8 | 7 | 2 | 0 | 0 | 0 | layout_assembly |
| layout_algo_list | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | layout_algo_list |
| layout_algo_multicol | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | layout_algo_multicol |
| layout_algo_out_of_flow | 0 | 2 | 0 | 0 | 1 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | layout_algo_out_of_flow |
| layout_algo_replaced | 2 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | layout_assembly |
| layout_algo_sizing | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 0 | layout_assembly |
| layout_algo_svg | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | layout_algo_svg |
| layout_algo_table | 2 | 1 | 0 | 0 | 1 | 0 | 0 | 0 | 1 | 0 | 1 | 0 | 0 | 0 | layout_algo_table, layout_assembly |
| layout_core | 58 | 2 | 0 | 0 | 9 | 0 | 1 | 0 | 23 | 1 | 0 | 1 | 3 | 0 | layout_assembly |
| layout_fragment | 2 | 6 | 0 | 2 | 2 | 0 | 0 | 0 | 16 | 1 | 0 | 0 | 1 | 1 | layout_assembly, layout_fragment |
| layout_invalidation | 3 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 1 | 0 | 0 | layout_assembly |
| layout_reuse | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | layout_assembly |
| layout_style | 19 | 78 | 0 | 0 | 2 | 0 | 0 | 0 | 38 | 0 | 0 | 0 | 0 | 0 | layout_style |
| layout_style_paint | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 1 | 0 | layout_style_paint |
| **Total** | **107** | **102** | **0** | **2** | **18** | **0** | **1** | **0** | **112** | **10** | **6** | **4** | **7** | **1** | |

### What the "unused fn" warnings are

103 of the 107 are `extern` declarations of untranslated functions that nothing
calls. Because nothing references them, they need no stub.

| File | Uncalled `extern` declarations |
|---|---|
| `layout_core/src/internal/layout_object.rs` | 48 |
| `layout_core/src/inline/layout_inline.rs` | 17 |
| `layout_core/src/internal/css/counters_attachment_context.rs` | 10 |
| `layout_style/src/style/computed_style.rs` | 9 |
| `layout_style/src/css/style_color.rs` | 5 |
| `layout_style/src/style/basic_shapes.rs` | 3 |
| `layout_core/src/invalidation/layout_object_style_update.rs` | 3 |
| `layout_core/src/table/table_layout_algorithm_types_interface.rs` | 2 |
| `layout_core/src/replaced/layout_replaced_interface.rs` | 2 |
| `layout_algo_inline/src/layout_counter.rs` | 2 |
| `layout_style/src/style/content_data.rs` | 1 |
| `layout_fragment/src/grid/grid_track_collection_types.rs` | 1 |

Seven of these names are also in the stub list:

- `LayoutObjectCreateObjectProvider`
- `LayoutObjectCreateBlockFlowOrListItemProvider`
- `LayoutObjectGeneratingNodeProvider`
- `LayoutObjectEnclosingNodeProvider`
- `LayoutObjectScrollParentProvider`
- `LayoutObjectOffsetParentProvider`
- `DispatchLayoutObjectInLayoutNGInlineFormattingContextWillChange`

Each is declared twice. The copy in `layout_object.rs` is unused, and a second
copy in another file is the referenced one.

The remaining 4 "unused fn" warnings are real bodies:

- `ComputedStyleData::from_initial_style` / `new_with_builder_pass_key_from_initial`
- `GridSetBuilder::new_from_tracks*`
- `FragmentDataIterator::from_list`
- `PhysicalFragmentRareData::move_from`

### What the "unused method" warnings are

- 75 of `layout_style`'s 78 are generated `ComputedStyleBase` accessors in
  `layout_style/src/style/computed_style_base.rs` (29,264 lines): 32 unused
  `Copy` methods, `Mutable*Internal`, `Set*InternalOwned` and so on. These
  mirror Chromium's generated class. They are leftovers, not a subsystem.
- The rest are scattered leftovers:
  - `Trace` methods in `out_of_flow_layout_part.rs`, `line_truncator.rs` and
    `layout_box_location_update.rs`.
  - `InlineCursor::SlowMoveTo*`
  - `PhysicalBoxFragment::AddOutlineRects`
  - `LineBreaker::AddHyphenAtCheckedIndex`
  - `ScoreLineBreaker::Node`
  - `ShapeOutsideInfo::ReferenceBoxPhysicalSize`
  - `SvgTextLayoutAlgorithm::IsVerticalUpward`
  - Table `Advance*ToNonEmptySection`
  - `MapToVisualRectInAncestorSpaceInternalFastPath`
  - `LayoutObject::AdjustStyleDifference`, `ApplyPseudoElementStyleChanges`,
    `ApplyFirstLineChanges`, `FirstLineStyleWithoutFallback` and
    `InvalidateSelectedChildrenOnStyleChange`

### Fields, variants and unreachable code

- Unused fields:
  - `LayoutEnvironment`: 7 fields (`text_transform`, `shape_images`,
    `text_and_images`, `phrase_break*`, `hyphenation*`), in
    `layout_core/src/internal/boundary/layout_environment.rs:12,60`.
  - `LayoutObject::registered_as_first_line_image_observer_`
  - `LayoutPassScope::object_factory_scope_`
  - `FragmentBuilder::is_may_have_descendant_above_block_start_explicitly_set_`
    and `is_finalized_`
  - `LineBreaker::is_initial_letter_box_`
  - `TextFitUtils::device_pixel_ratio_`
  - `ScrollableOverflowCalculator::has_non_visible_overflow_`
  - `LayoutEnvironmentCache::transient`
  - `BorderEdge::is_present_`
  - `StyleCachedData::variable_names_`
  - plus the fields of the dead `CountersAttachmentContext`,
    `FragmentRepeater` and `SetLayoutNeededForbiddenScope` types.
- Never-constructed variants:
  - `PaintLayer` `kDoesNotNeedDescendantDependentUpdate`
  - `text_decoration_info.rs` `Android` / `Apple` (platform branches)
- Unreachable patterns or expressions: `paint_style_export.rs:539`,
  `fragment_geometry_export.rs:422`, `baseline_utils.rs:96`,
  `fragmentation_utils.rs:73`, `layout_box.rs:1852`, `layout_object.rs:2594`,
  `relative_utils.rs:74`.
- Unused imports (112) and variables (10) are spread across files. The largest
  clusters are:
  - `layout_core/src/internal/boundary/layout_boundary.rs`: 10 warnings
  - `layout_algo_block/src/excluded_block_boundaries.rs`: 7
  - `layout/src/engine.rs`: 5
  - `layout_algo_grid/src/grid_layout_algorithm.rs`: 4

## Dead subsystems (whole translated units with no live caller)

| Unit | Size | Evidence | Status |
|---|---|---|---|
| `CountersAttachmentContext` (`layout_core/src/internal/css/counters_attachment_context.rs`) | 282 lines | 7 stubs, 10 uncalled `extern`s, unused fields and methods; the only outside use is `CounterEntry` in `layoutng_trace_bindings.rs` | Dead, but it is the Chromium home of CSS counters (gap b) |
| CSS counter content (`CounterContentData`, `AltCounterContentData`, `LayoutCounter` update/text paths in `layout_algo_inline/src/layout_counter.rs`, 145 lines) | partial | The `CounterContentData::new` stub is never called; `LayoutCounter::ListStyle` is the only live entry | Gap (b) |
| `FragmentRepeater` (`layout_core/src/internal/fragment_repeater.rs`) | 93 lines | `FragmentRepeater::` is never referenced; `BlockNode` holds the translated code | Delete |
| `StyleSVGResource` (`layout_style/src/style/style_svg_resource.rs`) | 69 lines | Constructor, `Drop`, `AddClient` and `RemoveClient` are all stubs; it is never constructed, but the type is referenced by live Style code | Unclear (c) |
| `UnresolvedRelativeColor`, `UnresolvedContrastColor`, `UnresolvedAlphaColor` (`layout_style/src/css/style_color.rs:299-430`) | ~130 lines | Constructor, `Resolve` and `ToCSSValue` are all stubs; never constructed | Gap (b) |
| LayoutObject-side paint / hit-test entry points spread over `layout_core/src/fragment/layout_object_fragment_queries.rs` (194), `layout_view_fragment_queries.rs` (96), `layout_object_paint_dispatch.rs` (150), `layout_core/src/invalidation/layout_object_layer_mutation.rs` (47), `layout_object_image_observers.rs` (132) | mixed files | About 30 stub-calling wrappers; these files also hold live helpers | Delete function by function |
| `layout_algo_inline_assembly` package | 27-line `lib.rs` + manifest | No dependents; it re-`include!`s `layout_core`, `layout_fragment` and `layout_algo_inline` | Deleted in R5 (the `Cargo.lock` entry drops on the next build) |
| Generic `MatchedPropertiesCache::Find` (Style, outside scope) | ~95 lines | Never instantiated; production uses `FindForNonHighlightWithStylesAndData` | Leftover for Style to decide |

## Uncompiled and stray files

- **No uncompiled `.rs` files.** Every `.rs` file under `crates/layout` (1,034)
  is reached from a library or test root. Test-only files: `layout/tests/*`,
  `layout_algo_svg/tests/*`, `layout_geometry/tests/*`, and the `#[cfg(test)]`
  modules in `layout_algo_table/src/lib.rs`.
- **Shell packages that compile nothing used by the app:**
  - `layout_algo_fragmentation/src/lib.rs` and `layout_algo_sizing/src/lib.rs`:
    doc comment only.
  - `layout_algo_inline/src/lib.rs` (before R5, when it became a real crate):
    all modules were gated on a package feature
    that nothing enables.
  - `layout_algo_inline_assembly/src/lib.rs`: no dependents (deleted in R5).

  These source directories are still the owning location of files that
  `layout_assembly` compiles through `#[path]`, so only the package shells (and
  the manifests, if the workspace member is dropped) are candidates, not the
  sources.
- **Feature-gated modules:**
  - `layout_algo_block`: `block_inline_layout` (`inline_extension`, enabled by
    the default `layout/inline_layout`) and `excluded_block_boundaries`
    (`block_only_host`, enabled by `layout` and `layout_algo_table`). Both
    compile.
  - `layout_core` / `layout_algo_inline`: `expensive_dchecks` blocks.
- **Stray files:**
  - `layout_core/src/fragment/layout_result_publish.rs.tmp`: 0 bytes, tracked by
    git, dated 2026-11-21. The real `layout_result_publish.rs` sits next to it.
  - `layout_core/Cargo.lock`: 14.8 KB, tracked by git, from building the
    excluded `layout_core` package on its own.

## Recommended deletion list (safest first)

1. **Stray files.** Delete `layout_core/src/fragment/layout_result_publish.rs.tmp`
   and `layout_core/Cargo.lock`. No code impact.
2. **Uncalled `extern` declarations (100 names / 103 declarations; see the
   table above).** Delete them. The compiler already proves that nothing calls
   them. Drop the 7 duplicates that also have stubs together with step 4.
3. **Unused imports, variables, `mut`s and dead fields from the warnings
   table.** These are mechanical and compiler-verified. Leave generated
   `ComputedStyleBase` accessors alone unless generator parity is not wanted.
4. **The 99 class-(a) stubs whose wrappers have zero by-name references.** This includes 5 of the unexported-body cases.
   Delete wrapper + `extern` declaration + stub name. Removing a wrapper that is
   still referenced turns into a compile error, so the compiler verifies this
   step.
5. **The 11 unexported-body cases.**
   - For the 5 class-(a) cases not already covered by step 4, delete the `*_interface.rs` wrapper and its
     `extern` (callers already use the body), or call the body directly.
   - For `InlineNodeSetTextWithOffset` / `LayoutTextSetTextWithOffset`, route
     the wrapper to the translated body instead of deleting it (gap b).
6. **The remaining 82 class-(a) stubs (wrapper has same-name references).**
   Delete them, then rely on `cargo check -p rechrom_app`. Watch the
   `LayoutBox`-level shadow methods: after deletion, `Deref` may silently route
   a `LayoutBox` call to the `LayoutObject` method. Review each such call site.
7. **Dead units: `FragmentRepeater` and `layout_algo_inline_assembly`.** Delete
   them. Then decide on the empty shell packages (`layout_algo_fragmentation`,
   `layout_algo_sizing`, the `layout_algo_inline` gated `lib.rs`). That is a
   packaging decision that AGENTS.md ownership rules affect.
8. **Do not delete without a decision:**
   - The 48 class-(b) gap stubs, which record untranslated Chromium behaviour:
     counters, relative/contrast/alpha colors, CSS Layout API, MathML, paged
     media, offsetParent, accessibility, Paint API, highlight MPC.
   - The 18 class-(c) stubs: OffsetMapping, StyleSVGResource, position-try
     scroll ranges.

   Either translate them, or move them to an explicit gap list before removing
   the code.

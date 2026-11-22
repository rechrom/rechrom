# Layout package map (R0)

Analysis only; no code was changed. Snapshot: `main` at `3d5c7d8d`
(Layout sources identical to `5a6ee664`), 2026-11-22. This sizes the
"real Layout packages" plan (R1–R7): every package under `crates/layout`
becomes a crate that compiles only its own files, dependencies point one way,
and packages above the base register into a `LayoutEngine`-owned registry with
one `Register(&mut LayoutRegistry)` function.

Target layering used for "upward" below (bottom to top):

| level | packages |
|---|---|
| L0 roots | `layout_geometry`, `layout_style`, `paint_input_types`, `layout_style_paint` (already clean, not analysed further) |
| L1 | `layout_fragment` (fragment data; LayoutObject as an opaque handle) |
| L2 base | `layout_core` + `layout_assembly` real code → `layout_base` |
| L3 | `layout_invalidation`, `layout_reuse`, `layout_commit`, every `layout_algo_*` (peers; any L3→L3 reference is reported as *lateral*) |
| L4 | `layout` (front crate) |

## Key findings

1. **Every `#[path]` crossing lives inside one compile unit.** `layout_assembly`
   compiles 645 files: 3 `include!` (core, fragment and inline roots) plus
   343 `#[path]` modules, 311 of them crossing a package directory. No
   standalone crate uses `#[path]` or `include!`. `layout_algo_inline_assembly`
   repeats the same unit (3 `include!` + 339 `#[path]`) for compile checks only.
2. **Most upward edges come from where files sit on disk, not from real
   dependencies.** 675 of the 1,002 upward item references (67%) disappear when
   files move to the package their translated source belongs to. The
   translation's own `// cpp:` citations name the snapshot package. Files citing
   `layoutng/...` are base code even when they sit under
   `layout_algo_*` or `layout_invalidation`: 45 invalidation files, 22 inline
   files, 12 each in block and float, 11 in out-of-flow. 128 more references go
   away when fragment files that are really LayoutObject/PaintLayer code move up
   into base. 199 references remain. They need registry slots, extension traits
   or splitting files at method level (§4.3).
3. **The fragment↔base cycle is the largest: 160 items + 286 methods, from
   126 of the 148 assembly-compiled fragment files.** Only 39 fragment files
   are fragment *data*. Another 111 fragment files (30.7k lines) are
   LayoutObject/LayoutBox/LayoutView methods (33 inherent `impl` blocks, 375
   fns), PaintLayer/scrolling, paint export and builders, and they belong in
   base. The data closure also pulls down 12 core files (`ConstraintSpace`,
   `StyleVariant`, `TextOffsetRange`, …) and 29 algorithm data files
   (`ExclusionSpace`, `OofPositionedNode`, `GridLayoutData`,
   `TableConstraintSpaceData`, `UsedFont`, `InlineItem`, …), because
   `LayoutResult` holds `ConstraintSpace` by value (`space_:
   UnsafeCell<ConstraintSpace>`).
4. **The layering contradicts itself for `layout_invalidation`, `layout_reuse`
   and `layout_commit`** (§5.1). Today their standalone crates sit *below*
   core: core depends on them for vocabulary, propagation and trace. Their
   assembly-compiled files are mostly inherent `impl LayoutObject` /
   `impl LayoutBox` blocks: 32 blocks / 194 fns in invalidation, plus
   per-box result-cache and result-commit methods. Base calls these on hot
   per-object paths. Rust coherence forbids inherent impls in another crate, so
   these files cannot sit above base. Only the engine-level phases can (tree
   update, node input update, input invalidation, publish, cache lookup).
5. **Some packages are really libraries.** `layout_algo_sizing`
   (`length_utils`, `space_utils`, `baseline_utils`) and
   `layout_algo_fragmentation` (`fragmentation_utils`, `break_appeal`,
   `early_break`) are used by every algorithm (sizing: 12 L3 packages, 164 items;
   fragmentation: 11 L3 packages, 116 items) and by base and fragment. Nothing
   dispatches into them, so they have nothing to `Register`. They belong in
   base, with the fragment-data pieces in L1.
6. **A registry already exists in four forms.** In total that is about 234
   fn-pointer slots, plus about 70 cross-package link-time symbols (§2.3,
   §6.1). The new slots the moves require are about 46 fn pointers in about 19
   groups (§6.2). That assumes per-object lifecycle code moves into base.
   Keeping it above base would need about 200 more per-object slots, which is
   not viable.

## 0. Method and confidence

Scripts are in
`C:\Users\ADMINI~1\AppData\Local\Temp\2\claude\4a080e1c2fe6a277\04b28603-c070-487f-aad9-43b9a5a4be75\scratchpad\r0\`.
That folder is session scratch and may be cleared:

- `scan.pl` walks every compile unit. Units are the `[lib] path` of each
  workspace member under `crates/layout`. `layout_core` is not a workspace
  member and the standalone `layout_algo_inline` compiles nothing, because every
  module is gated on `translation_in_progress`. The walk follows `mod x;`,
  `#[path]`, inline `mod {}` and `include!` (the included file's directory is
  its module directory), and `extern crate self as …` aliases. It indexes
  module-level items, `use` trees, `impl` blocks and their methods.
- **Resolution** follows Rust 2018 path rules from each file's module: `crate::`
  / `self::` / `super::` / the alias crates `layout_core::` /
  `layout_fragment::` / `layout_algo_inline::` / `layout_assembly::`, through
  explicit and glob re-exports. A standalone crate's `layout_assembly::` path
  is resolved inside the assembly unit. Bare CamelCase identifiers and
  `name(` / `name!` calls are resolved through module scope (globs included).
  When scope lookup fails, a unique global item name is used (`name-fallback`).
  Outside `untranslated_symbols.rs`, which is excluded, that fallback was
  needed 14 times. Only 8 path references stayed ambiguous between packages;
  they are listed in `ambiguous.tsv`.
- **Methods are a heuristic.** A `.Name(` call counts against package P when
  only P defines a method `Name` in an `impl` of a layout-defined type or trait.
  Calls are dropped when the calling package also defines `Name`, or when
  `Name` is one lowercase word (`len`, `get`, …) or a common std name. Method
  counts are therefore a lower bound with some noise (for example `IsFixed
  [Column]`). Tables keep them in a separate column.
- **Not seen:** code generated by macros, `cfg(feature)` differences (all
  modules are treated as compiled), and anything in `#[cfg(test)]` modules
  (excluded).
- **Counts:** "items" means distinct (defining file, item) pairs that a package
  references. "refs" means the number of referencing files.
- **Chromium ownership:** the translated files' `// cpp:` comments cite the
  pinned *snapshot* packages. `layoutng/internal/...` is the base package
  (Chromium `core/layout/`). `layoutng_fragment_tree/...` is the fragment tree.
  `layoutng_<algo>/...` is that algorithm (Chromium `core/layout/<algo>/`).
  These citations are the primary evidence. Chromium paths given beside them
  come from the Blink tree layout and were not re-read at the pinned revision
  in this session.

## 1. Summary

| package | target level | .rs files | lines | files compiled by its own crate | files compiled by another unit | files declared (#[path]/mod) by another package's file | upward/lateral items (paths) | upward/lateral methods (heuristic) |
|---|---|---:|---:|---:|---|---:|---:|---:|
| layout_fragment | L1 fragment | 151 | 55505 | 3 | layout_assembly: 148 | 114 | 246 | 384 |
| layout_core | L2 base | 233 | 35848 | 0 | layout_assembly: 233 | 3 | 288 | 83 |
| layout_assembly | L2 base | 5 | 3492 | 5 | - | 0 | 10 | 18 |
| layout_invalidation | L3 above base | 67 | 12883 | 6 | layout_assembly: 61 | 57 | 26 | 30 |
| layout_reuse | L3 above base | 20 | 2538 | 9 | layout_assembly: 11 | 11 | 13 | 6 |
| layout_commit | L3 above base | 11 | 1012 | 2 | layout_assembly: 9 | 9 | 3 | 6 |
| layout_algo_block | L3 above base | 23 | 9542 | 8 | layout_assembly: 15 | 12 | 87 | 33 |
| layout_algo_inline | L3 above base | 95 | 37484 | 0 | layout_assembly: 95 | 41 | 35 | 59 |
| layout_algo_flex | L3 above base | 14 | 5867 | 13 | layout_assembly: 1 | 1 | 56 | 9 |
| layout_algo_grid | L3 above base | 29 | 13279 | 20 | layout_assembly: 9 | 9 | 44 | 5 |
| layout_algo_table | L3 above base | 35 | 9873 | 30 | layout_assembly: 5 | 5 | 37 | 10 |
| layout_algo_float | L3 above base | 20 | 4477 | 8 | layout_assembly: 12 | 12 | 21 | 2 |
| layout_algo_out_of_flow | L3 above base | 22 | 8116 | 11 | layout_assembly: 11 | 10 | 33 | 8 |
| layout_algo_replaced | L3 above base | 7 | 1906 | 6 | layout_assembly: 1 | 1 | 4 | 6 |
| layout_algo_svg | L3 above base | 41 | 6657 | 30 | layout_assembly: 9 | 9 | 13 | 22 |
| layout_algo_forms | L3 above base | 19 | 3130 | 14 | layout_assembly: 5 | 5 | 26 | 17 |
| layout_algo_list | L3 above base | 11 | 1198 | 8 | layout_assembly: 3 | 3 | 2 | 5 |
| layout_algo_multicol | L3 above base | 4 | 653 | 3 | layout_assembly: 1 | 1 | 18 | 4 |
| layout_algo_sizing | L3 above base | 6 | 3153 | 1 | layout_assembly: 5 | 5 | 4 | 4 |
| layout_algo_fragmentation | L3 above base | 5 | 1860 | 1 | layout_assembly: 4 | 4 | 4 | 4 |
| layout_algo_custom | L3 above base | 4 | 94 | 2 | layout_assembly: 2 | 2 | 0 | 0 |
| layout_algo_mathml | L3 above base | 2 | 17 | 2 | - | 0 | 0 | 0 |
| layout | L4 front | 20 | 1682 | 19 | - | 0 | 0 | 0 |

Columns: "files declared by another package's file" is the metric quoted in the
plan. It measures 114 for fragment and 41 for inline, where the plan says 115
and 42; every other package matches. "Upward/lateral" counts are distinct items
whose defining package is above this one, or a different L3 package.

Lines include tests. After `62ac4b3c`, `layout_assembly/src/lib.rs` is 303
lines, down from 12,025.

## 2. Composition today

### 2.1 `include!` and `#[path]`

- `layout_assembly/src/lib.rs`: `include!("../../layout_core/src/lib.rs")` at
  the crate root, `layout_fragment/src/lib.rs` as `layout_fragment_package`,
  and `layout_algo_inline/src/lib.rs` as `layout_algo_inline_package`. Both are
  glob re-exported at the root. The root also declares `extern crate self as
  layout_algo_inline / layout_fragment / layout_core / layout_assembly`.
- `layout_algo_inline_assembly/src/lib.rs` contains the same three
  `include!`s plus `#[path]` to two assembly files. Nothing depends on it.
- `layout_algo_block/src/lib.rs` contains `extern crate layout_assembly as
  layout_algo_inline / layout_fragment / layout_core`, so block's sources
  name assembly items through the old aliases.
- 783 `layout_assembly::` paths in 145 files, spread over 13 package directories, must be
  rewritten once the assembly is gone. Inside the unit, 3,164 cross-package
  `use`/path references in 521 files go through `crate::` / `super::` /
  aliases.

`#[path]` declarations in the assembly unit, by declaring package → target
package (343 `#[path]` + 3 `include!`):

| declaring → target | n | declaring → target | n |
|---|---:|---|---:|
| core → fragment | 105 | core → reuse | 11 |
| core → invalidation | 50 | core → out_of_flow | 10 |
| core → inline | 38 | core → commit | 9 |
| core → core | 15 | core → svg | 9 |
| core → block | 12 | block → invalidation | 7 |
| core → float | 11 | core → grid | 6 |
| fragment → fragment | 6 | core → forms / sizing / table | 5 each |
| assembly → fragment | 4 (+1 include) | core → fragmentation | 4 |
| invalidation → invalidation | 4 | core → list | 3 |
| block → block, inline → inline, inline → fragment | 3 each | assembly → grid, core → custom, fragment → inline, invalidation → core | 2 each |
| core → flex / multicol / replaced, invalidation → float / grid / fragment, out_of_flow → out_of_flow | 1 each | assembly → core, assembly → inline | 1 include each |

Appendix A lists every declaration.

### 2.2 Who compiles whose files

The only compile unit that crosses packages is `layout_assembly`. Files each
package loses to it: fragment 148/151, core 233/233, inline 95/95,
invalidation 61/67, block 15/23, float 12/20, reuse 11/20, out_of_flow 11/22,
commit 9/11, grid 9/29, svg 9/41, sizing 5/6, forms 5/19, table 5/35,
fragmentation 4/5, list 3/11, custom 2/4, flex/multicol/replaced 1 each.

### 2.3 Link-time edges (`extern "Rust"/"C"` declarations vs `#[no_mangle]` definitions)

859 symbols are declared in `extern` blocks and 740 are defined with
`#[no_mangle]`; 252 of them are aborting stubs in `untranslated_symbols.rs`
(see `DEAD_CODE.md`). These are hidden cross-package calls that the path
analysis does not see. Edges where a symbol is declared in one package and
defined in another:

| declared in → defined in | symbols | after the target layering |
|---|---:|---|
| core → assembly (`Dispatch*` virtual dispatch) | 106 | internal to base |
| core → fragment (`LayoutBox*Provider`) | 13 | downward; becomes direct calls once the providers move to base with their LayoutBox methods |
| invalidation → assembly (`Dispatch*`) | 12 | internal to base (the callers move to base, §4) |
| core → invalidation (`LayoutObject*Provider`) | 10 | internal to base once the providers move down |
| inline → fragment (`LayoutInline*`, `LayoutText*`, `InkOverflow*ForTextCombine`) | 9 | internal to base (LayoutInline/LayoutText methods) |
| fragment → inline (`FragmentItemsBuilder*FromInline/ForInline`, `InlineItemsData*`, `FragmentBuilderAddOutOfFlowInlineChildCandidate`) | 6 | upward: slots, or move the callers up (§6.2) |
| block → assembly (`DispatchLayoutBlockFlow*`) | 5 | internal to base |
| core → table (`LayoutInputNode*FromTable`) | 4 | upward: slot group |
| core → inline (`AdjustStyleFor*Combine*`, `InlineChildLayoutContextParallelFlowBreakTokens`, `LayoutBoxPositionForPointInFragmentsProvider`) | 4 | 2 internal to base, 2 slots |
| fragment → assembly (`DispatchBreakTokenAlgorithmDataGetTotalRowGapCount`, `DispatchLayoutBoxModelObjectStickyContainer`) | 2 | callers move up, or 2 slots |
| inline → table (`LayoutInnerTextTableRowNeedsNewline`, `LayoutTextIteratorTableFacts`) | 2 | lateral: slots registered by table |
| block → forms (`BlockNodeGetFieldsetContentFromForms`), block → inline (`LayoutBlockFlowShouldTruncateOverflowingTextFromInline`), inline → list (`LayoutEditingListItemValue`), core → block (`InlineChildLayoutContextParallelFlowBreakTokens`) | 1 each | slots |

Risk: `InlineChildLayoutContextParallelFlowBreakTokens` is defined
`#[no_mangle]` in both `layout_algo_block/src/excluded_block_boundaries.rs`
(`cfg(not(feature = "inline_extension"))`) and
`layout_algo_inline/src/inline_child_layout_context.rs`. A feature switch picks
the definition, so the duplicate has to become one slot.

### 2.4 Registration mechanisms that exist today

| mechanism | where | slots |
|---|---|---:|
| `LayoutAlgorithmSet` (14 `LayoutAlgorithmEntry` × 4 fields + 12 `*Support` structs: inline 6, table 5, fragmentainer 5, out_of_flow 2, float 5, replaced_sizing 2, simplified 2, list 6, forms 3, mathml 5, svg 40, grid_lanes 3) | `layout_core/src/internal/layout_algorithm_set.rs` | 140 |
| `LayoutObjectFactorySet` | `layout_core/src/internal/layout_object_factory_set.rs` | 74 |
| `LayoutBoundarySupport` (`create_environment`, `prepare_constraints`, `prepare_tree`, `publish_layout_result`) | `layout_core/src/layout_boundary_support.rs` | 4 |
| `OnceLock` statics with `Register*` functions (flex/grid row-gap, grid/replaced/image/video `StyleDidChange`, replaced visual overflow, SVG root children, `GridObjectVirtuals` ×4) | `layout_assembly/src/block_virtual_dispatch.rs` | 13 |
| `RegisterSvgInlineTextFontVirtuals` (2 `OnceLock`s) | `layout_algo_inline/src/layout_text_interface.rs` | 2 |
| `FullLayoutRegistry()` (`extern "Rust"`; `layout/src/lib.rs` `CreateLayoutRegistry` fills the table through 14 `Register` calls, named `Install*` before R1) | `layout_core/src/layout_registry.rs` | 1 |

That is about 234 fn pointers, plus the ~70 cross-package link-time symbols
above.

## 3. Reference matrix


**Distinct referenced items** (row = referencing package, column = defining package; `*` = upward or lateral relative to the target layering)

| from \ to | frag | core | asm | inval | reuse | commit | block | inline | flex | grid | table | float | out_of_flow | replaced | svg | forms | list | multicol | sizing | fragmentation | custom | mathml | front |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| frag |  | 160* |  | 15* | 1* | 1* | 5* | 17* | 1* | 1* | 3* | 1* | 15* | 4* | 1* | 2* |  |  | 2* | 16* | 1* |  |  |
| core | 260 |  |  | 70* | 20* | 24* | 26* | 45* | 1* | 11* | 6* | 16* | 17* | 4* | 8* | 19* | 1* |  | 1* | 16* | 3* |  |  |
| asm | 5 | 27 |  | 1* |  |  | 2* | 2* |  | 1* |  |  |  | 4* |  |  |  |  |  |  |  |  |  |
| inval | 17 | 161 |  |  | 3* |  | 3* | 3* |  | 4* |  | 2* |  | 2* | 1* | 8* |  |  |  |  |  |  |  |
| reuse | 6 | 19 |  | 1* |  | 1* | 3* |  |  |  |  |  |  |  |  |  |  |  | 3* | 5* |  |  |  |
| commit | 9 | 19 |  |  |  |  | 1* | 1* |  |  |  |  |  | 1* |  |  |  |  |  |  |  |  |  |
| block | 22 | 68 |  | 14* | 2* |  |  | 10* |  |  |  | 2* |  | 1* |  | 4* |  |  | 29* | 24* |  | 1* |  |
| inline | 55 | 80 |  | 2* |  |  | 5* |  |  |  |  | 5* |  | 1* | 8* | 1* |  |  | 12* | 1* |  |  |  |
| flex | 15 | 20 | 2 |  |  |  | 2* |  |  |  | 1* |  |  |  |  |  |  |  | 33* | 20* |  |  |  |
| grid | 22 | 34 | 4 | 5* |  |  | 2* |  |  |  |  | 1* |  |  |  |  |  |  | 22* | 14* |  |  |  |
| table | 13 | 34 |  | 2* |  |  | 4* |  |  |  |  |  |  |  |  |  |  |  | 14* | 17* |  |  |  |
| float | 5 | 15 |  | 4* |  |  | 1* |  |  |  |  |  |  | 1* |  |  |  |  | 7* | 8* |  |  |  |
| out_of_flow | 21 | 24 |  |  |  |  | 2* | 1* |  | 1* |  |  |  |  |  |  |  |  | 17* | 12* |  |  |  |
| replaced | 7 | 30 | 4 | 2* |  |  | 1* |  |  |  |  |  |  |  |  |  |  |  | 1* |  |  |  |  |
| svg | 7 | 30 | 1 | 3* |  |  | 3* | 6* |  |  |  |  |  | 1* |  |  |  |  |  |  |  |  |  |
| forms | 9 | 24 |  |  |  |  | 3* |  |  |  |  |  |  |  |  |  |  |  | 14* | 9* |  |  |  |
| list |  | 11 |  |  |  |  | 1* | 1* |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| multicol | 11 | 12 |  |  |  |  | 3* |  |  |  |  |  |  |  |  |  |  |  | 10* | 5* |  |  |  |
| sizing | 2 | 15 |  |  |  |  | 1* |  |  |  | 1* |  |  |  |  | 1* |  |  |  | 1* |  |  |  |
| fragmentation | 10 | 9 |  |  |  |  | 2* |  |  |  |  |  |  |  |  |  |  |  | 2* |  |  |  |  |
| custom |  | 2 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| mathml |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| front | 63 | 157 |  |  |  | 3 | 8 | 10 | 1 | 2 | 1 | 1 | 1 | 3 | 1 | 1 | 1 | 1 |  |  |  |  |  |

**Distinct referenced methods** (row = referencing package, column = defining package; `*` = upward or lateral relative to the target layering)

| from \ to | frag | core | asm | inval | reuse | commit | block | inline | flex | grid | table | float | out_of_flow | replaced | svg | forms | list | multicol | sizing | fragmentation | custom | mathml | front |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| frag |  | 286* |  | 21* |  |  | 11* | 31* |  |  | 7* | 1* | 16* | 8* |  |  |  |  | 1* | 2* |  |  |  |
| core | 140 |  |  | 32* |  |  | 6* | 22* |  |  | 4* | 1* | 3* | 3* | 2* | 7* |  |  | 2* | 1* |  |  |  |
| asm | 16 | 41 |  | 10* |  |  | 3* | 3* |  |  |  |  |  | 2* |  |  |  |  |  |  |  |  |  |
| inval | 37 | 208 |  |  |  |  | 12* | 11* |  |  | 1* | 1* |  | 1* |  | 4* |  |  |  |  |  |  |  |
| reuse | 37 | 45 |  | 1* |  |  |  | 2* |  |  |  | 3* |  |  |  |  |  |  |  |  |  |  |  |
| commit | 20 | 26 |  | 4* |  |  | 1* |  |  |  |  |  |  | 1* |  |  |  |  |  |  |  |  |  |
| block | 133 | 175 |  | 7* |  | 1* |  | 14* |  |  |  | 5* |  |  |  | 2* |  |  | 3* | 1* |  |  |  |
| inline | 190 | 111 |  | 23* | 1* |  | 13* |  |  |  | 2* | 11* |  |  | 7* |  |  |  | 2* |  |  |  |  |
| flex | 91 | 28 |  | 2* |  |  | 1* | 2* |  | 1* | 1* |  |  |  |  |  |  |  | 2* |  |  |  |  |
| grid | 61 | 37 |  |  |  |  | 1* | 3* |  |  | 1* |  |  |  |  |  |  |  |  |  |  |  |  |
| table | 64 | 91 |  | 8* |  |  |  |  |  |  |  |  |  |  |  |  |  |  | 1* | 1* |  |  |  |
| float | 11 | 23 |  |  |  |  |  | 1* |  |  |  |  |  | 1* |  |  |  |  |  |  |  |  |  |
| out_of_flow | 96 | 46 |  | 3* |  |  | 2* | 3* |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| replaced | 28 | 36 |  | 3* |  |  |  | 3* |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| svg | 30 | 36 |  | 7* |  |  |  | 14* |  |  |  |  |  | 1* |  |  |  |  |  |  |  |  |  |
| forms | 39 | 45 |  | 7* |  |  | 4* | 3* |  |  | 2* |  |  |  |  |  |  |  | 1* |  |  |  |  |
| list | 1 | 28 |  | 3* |  |  |  | 2* |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| multicol | 22 | 18 |  | 4* |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| sizing | 11 | 58 |  |  |  |  | 2* | 1* |  |  | 1* |  |  |  |  |  |  |  |  |  |  |  |  |
| fragmentation | 66 | 40 |  |  |  |  | 3* |  |  |  |  | 1* |  |  |  |  |  |  |  |  |  |  |  |
| custom |  | 1 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| mathml |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| front |  | 3 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |

Diagonal cells are blank. The front-crate row is downward and is shown only
for completeness. `asm` row: references to stubs from
`untranslated_symbols.rs` are excluded.

## 4. Upward edges

### 4.1 By layer pair

| pair | edges | items | methods |
|---|---:|---:|---:|
| L1 fragment → L2 base | 1 | 160 | 286 |
| L1 fragment → L3 | 16 | 86 | 98 |
| L2 base → L3 | 22 | 298 | 101 |
| L3 → L3 (lateral) | 99 | 426 | 230 |
| **total** | **138** | **970** | **715** |

Edges in both directions (cycles today): block↔inline (10+15m / 5+13m),
block↔sizing, block↔fragmentation, block↔invalidation (14+7m / 3+12m),
block↔forms, block↔float, block↔reuse, block↔replaced, block↔commit,
inline↔invalidation (2+23m / 3+11m), inline↔svg (8+7m / 6+14m),
inline↔float, inline↔sizing, inline↔forms, inline↔replaced, inline↔reuse,
invalidation↔forms / grid / float / replaced / svg / table / reuse,
reuse↔block / inline / invalidation, sizing↔forms / table / fragmentation,
float↔fragmentation, replaced↔block, grid↔invalidation. Almost all of them
come from base-owned files that sit physically in an L3 directory.

### 4.2 Resolution per edge

Resolution codes:

- **(a) move the item down.** The defining file moves to base or fragment
  because it is data or a pure helper used below. The evidence is a
  `layoutng/...` `cpp:` citation, an inherent impl on a base type, or use by
  fragment data.
- **(b) registry slot.** Base must call behaviour implemented above.
- **(c) move the referencing code up.** The referencing file belongs with
  the higher package.

Per-edge decomposition, computed by applying the file moves of Appendix B and
re-checking each reference. a = resolved because the target moved down,
c = resolved because the referencing file moved up, r = still upward after the
moves (§4.3). The a/c/r columns count distinct items per class. An item
referenced from several files can fall in more than one class, so a+c+r can
exceed "items". Edges that only have method references are listed after the
table.

| edge (from → to) | items | methods | a | c | r |
|---|---:|---:|---:|---:|---:|
| fragment -> core | 160 | 286 | 14 | 128 | 39 |
| core -> invalidation | 70 | 32 | 32 | 0 | 38 |
| core -> inline | 45 | 22 | 22 | 0 | 23 |
| flex -> sizing | 33 | 2 | 33 | 0 | 0 |
| block -> sizing | 29 | 3 | 29 | 0 | 0 |
| core -> block | 26 | 6 | 12 | 0 | 14 |
| block -> fragmentation | 24 | 1 | 24 | 0 | 0 |
| core -> commit | 24 | 0 | 6 | 0 | 18 |
| grid -> sizing | 22 | 0 | 22 | 0 | 0 |
| fragment -> inline | 17 | 31 | 15 | 0 | 6 |
| core -> reuse | 20 | 0 | 15 | 0 | 5 |
| flex -> fragmentation | 20 | 0 | 20 | 0 | 0 |
| core -> forms | 19 | 7 | 19 | 0 | 0 |
| fragment -> fragmentation | 16 | 2 | 15 | 0 | 2 |
| core -> out_of_flow | 17 | 3 | 17 | 0 | 0 |
| out_of_flow -> sizing | 17 | 0 | 17 | 0 | 0 |
| table -> fragmentation | 17 | 1 | 17 | 0 | 0 |
| core -> fragmentation | 16 | 1 | 16 | 0 | 0 |
| core -> float | 16 | 1 | 16 | 0 | 0 |
| fragment -> invalidation | 15 | 21 | 8 | 0 | 7 |
| fragment -> out_of_flow | 15 | 16 | 15 | 0 | 0 |
| grid -> fragmentation | 14 | 0 | 14 | 0 | 0 |
| forms -> sizing | 14 | 1 | 14 | 0 | 0 |
| block -> invalidation | 14 | 7 | 13 | 0 | 1 |
| table -> sizing | 14 | 1 | 14 | 0 | 0 |
| inline -> sizing | 12 | 2 | 12 | 0 | 0 |
| out_of_flow -> fragmentation | 12 | 0 | 12 | 0 | 0 |
| core -> grid | 11 | 0 | 11 | 0 | 0 |
| multicol -> sizing | 10 | 0 | 10 | 0 | 0 |
| block -> inline | 10 | 14 | 4 | 0 | 6 |
| forms -> fragmentation | 9 | 0 | 9 | 0 | 0 |
| inline -> svg | 8 | 7 | 8 | 0 | 0 |
| float -> fragmentation | 8 | 0 | 8 | 0 | 0 |
| core -> svg | 8 | 2 | 7 | 0 | 1 |
| invalidation -> forms | 8 | 4 | 8 | 0 | 0 |
| fragment -> block | 5 | 11 | 5 | 0 | 2 |
| float -> sizing | 7 | 0 | 7 | 0 | 0 |
| core -> table | 6 | 4 | 6 | 0 | 0 |
| svg -> inline | 6 | 14 | 6 | 0 | 0 |
| multicol -> fragmentation | 5 | 0 | 5 | 0 | 0 |
| grid -> invalidation | 5 | 0 | 5 | 0 | 0 |
| reuse -> fragmentation | 5 | 0 | 5 | 0 | 0 |
| inline -> float | 5 | 11 | 5 | 0 | 0 |
| inline -> block | 5 | 13 | 5 | 0 | 0 |
| table -> block | 4 | 0 | 3 | 0 | 2 |
| fragment -> replaced | 4 | 8 | 0 | 0 | 4 |
| core -> replaced | 4 | 3 | 0 | 0 | 4 |
| invalidation -> grid | 4 | 0 | 4 | 0 | 0 |
| block -> forms | 4 | 2 | 4 | 0 | 0 |
| float -> invalidation | 4 | 0 | 4 | 0 | 0 |
| assembly -> replaced | 4 | 2 | 0 | 0 | 4 |
| reuse -> sizing | 3 | 0 | 3 | 0 | 0 |
| fragment -> table | 3 | 7 | 3 | 0 | 0 |
| invalidation -> inline | 3 | 11 | 3 | 0 | 0 |
| forms -> block | 3 | 4 | 3 | 0 | 0 |
| core -> custom | 3 | 0 | 2 | 0 | 1 |
| svg -> block | 3 | 0 | 3 | 0 | 0 |
| out_of_flow -> block | 2 | 2 | 2 | 0 | 1 |
| fragmentation -> block | 2 | 3 | 2 | 0 | 1 |
| svg -> invalidation | 3 | 7 | 3 | 0 | 0 |
| invalidation -> block | 3 | 12 | 2 | 0 | 1 |
| invalidation -> reuse | 3 | 0 | 0 | 0 | 3 |
| multicol -> block | 3 | 0 | 1 | 0 | 2 |
| reuse -> block | 3 | 0 | 3 | 0 | 0 |
| block -> float | 2 | 5 | 2 | 0 | 0 |
| grid -> block | 2 | 1 | 2 | 0 | 0 |
| fragment -> forms | 2 | 0 | 2 | 0 | 0 |
| assembly -> inline | 2 | 3 | 2 | 0 | 0 |
| out_of_flow -> inline | 1 | 3 | 1 | 0 | 1 |
| replaced -> invalidation | 2 | 3 | 2 | 0 | 0 |
| invalidation -> float | 2 | 1 | 2 | 0 | 0 |
| fragment -> sizing | 2 | 1 | 2 | 0 | 0 |
| fragmentation -> sizing | 2 | 0 | 2 | 0 | 0 |
| assembly -> block | 2 | 3 | 2 | 0 | 0 |
| table -> invalidation | 2 | 8 | 2 | 0 | 0 |
| flex -> block | 2 | 1 | 2 | 0 | 0 |
| inline -> invalidation | 2 | 23 | 2 | 0 | 0 |
| invalidation -> replaced | 2 | 1 | 0 | 0 | 2 |
| block -> reuse | 2 | 0 | 1 | 0 | 1 |
| fragment -> reuse | 1 | 0 | 0 | 0 | 1 |
| fragment -> float | 1 | 1 | 1 | 0 | 0 |
| fragment -> svg | 1 | 0 | 1 | 0 | 0 |
| sizing -> fragmentation | 1 | 0 | 1 | 0 | 0 |
| block -> replaced | 1 | 0 | 0 | 0 | 1 |
| grid -> float | 1 | 0 | 1 | 0 | 0 |
| list -> block | 1 | 0 | 1 | 0 | 0 |
| assembly -> invalidation | 1 | 10 | 1 | 0 | 0 |
| sizing -> forms | 1 | 0 | 1 | 0 | 0 |
| flex -> table | 1 | 1 | 1 | 0 | 0 |
| commit -> inline | 1 | 0 | 1 | 0 | 0 |
| replaced -> sizing | 1 | 0 | 1 | 0 | 0 |
| float -> replaced | 1 | 1 | 0 | 0 | 1 |
| float -> block | 1 | 0 | 1 | 0 | 0 |
| assembly -> grid | 1 | 0 | 1 | 0 | 0 |
| out_of_flow -> grid | 1 | 0 | 1 | 0 | 0 |
| fragment -> flex | 1 | 0 | 1 | 0 | 0 |
| core -> flex | 1 | 0 | 1 | 0 | 0 |
| reuse -> commit | 1 | 0 | 0 | 0 | 1 |
| sizing -> block | 1 | 2 | 1 | 0 | 0 |
| inline -> replaced | 1 | 0 | 0 | 0 | 1 |
| reuse -> invalidation | 1 | 1 | 1 | 0 | 0 |
| replaced -> block | 1 | 0 | 1 | 0 | 0 |
| commit -> block | 1 | 1 | 1 | 0 | 0 |
| fragment -> custom | 1 | 0 | 1 | 0 | 0 |
| list -> inline | 1 | 2 | 1 | 0 | 0 |
| invalidation -> svg | 1 | 0 | 0 | 0 | 1 |
| core -> sizing | 1 | 2 | 1 | 0 | 0 |
| inline -> forms | 1 | 0 | 1 | 0 | 0 |
| sizing -> table | 1 | 1 | 1 | 0 | 0 |
| core -> list | 1 | 0 | 1 | 0 | 0 |
| fragment -> grid | 1 | 0 | 1 | 0 | 0 |
| fragment -> commit | 1 | 0 | 0 | 0 | 1 |
| inline -> fragmentation | 1 | 0 | 1 | 0 | 0 |
| commit -> replaced | 1 | 1 | 0 | 0 | 1 |
| block -> mathml | 1 | 0 | 0 | 0 | 1 |
| svg -> replaced | 1 | 1 | 0 | 0 | 1 |

Edges with only method references (heuristic): forms→invalidation 7,
multicol→invalidation 4, commit→invalidation 4, grid→inline 3,
list→invalidation 3, replaced→inline 3, reuse→float 3, forms→inline 3,
out_of_flow→invalidation 3, inline→table 2, flex→inline 2,
flex→invalidation 2, reuse→inline 2, forms→table 2, and 1 each for
fragmentation→float, grid→table, invalidation→table, inline→reuse,
flex→grid, sizing→inline, block→commit, float→inline. Nearly all of these
call LayoutObject methods that live in invalidation files (`SetNeedsLayout`,
`NextInPreOrder`, …) or LayoutText/LayoutInline/UsedFont methods. They become
downward once those files move to base or fragment. Treat `IsFixed [Column]`
and `HasBorder [TableBorders]` as name collisions.

#### The large edges in detail

**fragment → core (160 items, 286 methods; 126 referencing files).**

- Top items: `LayoutObject` ×69 files, `LayoutBox` ×53, `LayoutBoxModelObject`
  ×38, `LayoutView` ×16, `Node`/`Element` ×15, `MapCoordinatesFlags` ×12,
  `LayoutPassScope` ×12, `PaintFragmentCursor` ×9, `ScrollbarOrientation` ×8,
  `StyleVariant` ×7, `PaintStyle` ×6, the `layout_boundary` module ×18.
- Top methods: `CheckIsNotDestroyed` ×36, `GetScrollableArea` ×17,
  `PhysicalFragments` ×11, `SlowFirstChild` ×11,
  `IsInLayoutNGInlineFormattingContext` ×10, `PhysicalBorderBoxRect` ×9.
- Resolution:
  - **(c)** 111 files move up to base: 32 LayoutObject/LayoutBox/LayoutView
    method files (4.8k lines), 24 PaintLayer/scroll files (9.7k), 21 paint
    export/publish files (7.1k), 8 builders (5.5k), 8 geometry-query files
    (0.9k) and 18 others (2.7k). These hold 33 inherent impl blocks / 375 fns
    on core types. Inherent impls cannot stay in another crate.
  - **(a)** Data types move down: `StyleVariant`, `TextOffsetRange`,
    `SplitAxisItem`, `HitTestPhase`, `SelectionState`, `MapCoordinatesFlags`,
    `TextDecorationOffset`, `ConstraintSpace`, `DisableLayoutSideEffectsScope`,
    `LayoutTheme`, `AnchorScopedName`, `LineLayoutOpportunity`.
  - **(opaque handle)** Kept data files still name `LayoutObject`, `LayoutBox`,
    `LayoutBoxModelObject`, `LayoutView`, `LayoutBlockFlow`, `LayoutInline`,
    `LayoutText`, `LayoutTextCombine`, `BlockNode`, `InlineNode`,
    `LayoutInputNode`, `Node`/`Element`, `PaintLayer` and
    `PaintLayerScrollableArea`.
  - The kept files call about 113 distinct base behaviours (98 core,
    8 invalidation, 6 block, 1 replaced). The default resolution is (c) at
    method level: the fragment method that dereferences the LayoutObject
    becomes an extension-trait method in base. Call syntax `fragment.Foo()`
    survives with a `use`.

**core → invalidation (70 items, 32 methods).** Splits into three groups:

- **(a)** LayoutObject-bound files move to base: `layout_object_child_list`,
  `layout_object_state_propagation`, `layout_box_style_change`,
  `layout_box_tree_lifecycle`, `layout_object_factory` and the 27 files with
  inherent impls. Source: `layoutng/internal/layout_object.h`,
  `layout_tree_invalidation.cc`, `layout_object_hot.cc`; Chromium
  `core/layout/layout_object.cc`, `layout_box.cc`,
  `layout_object_child_list.cc`.
- **(b)** Engine-level phases: `tree_update` (7 fns), `input_invalidation`
  (5), `node_input_update` (2), `tree_facts` (1), `engine_mutation`
  (`EngineMutation` trait), `prepare_tree` (already a slot).
- **(a)** Pure helpers: `managed_input_admission` (8 predicates),
  `constraint_equivalence` (5), `css_zoom`, `style_font_resolution`,
  `timeline_trigger_metadata`, `trigger_scope_projection`,
  `node_input_state`.
- The standalone crate's `propagation`, `layout_invalidation_state`,
  `layout_invalidation_reason`, `content_change_type` and `trace` sit below
  core today. See §5.1.

**core → inline (45 items, 23 methods).**

- **(a)** to base: `LayoutText`, `LayoutInline`, `LayoutTextFragment`,
  `LayoutTextCombine`, `InlineNode` (the `*_interface.rs` and `*_core.rs`
  files), `InlineNodeData`, `InlineItemSpan`, `OffsetMapping`,
  `VariableLengthTransformResult`, `text_input_environment`. All cite
  `layoutng/internal/…`; Chromium `core/layout/layout_text.h`,
  `layout_inline.h`, `layout_text_fragment.h`, `layout_text_combine.h`.
- **(a)** to fragment: `UsedFont`, `InlineItem`, `InlineItemSegment(s)`,
  `InlineItemTextIndex`, `TextFitScale`, `TextItemType`.
- **(c)** The editing-facts types (`Editing*Facts`, `EditingTextRun*`,
  `VisualCaretMove*`, `EditingFind*`, `InnerTextLayoutFacts`, …) are named by
  `layout_output.rs`, `layout_engine.rs`, `layout_selection.rs`,
  `engine/selection/range_geometry.rs`, `engine/selection/text_control_hard_wrap.rs` and `caret/mod.rs`.
  Those wrapper methods move into inline as extension traits on
  `LayoutNode`/`LayoutOutput`, re-exported by the front crate. Inline already
  implements them as 19 files of inherent `impl LayoutNode/LayoutOutput/
  LayoutObject/InlineCursor/PhysicalBoxFragment` blocks, 118 fns, which
  must become extension traits anyway.

**assembly → inline (50 items) and assembly → block (18) were stub
references.** They come entirely from `untranslated_symbols.rs` and are
excluded from the matrix. Distribute the stubs to the declaring crates.

**core → block (26 items, 6 methods).**

- **(a)** to base: `LayoutBlock`, `LayoutBlockFlow`, `BlockNode`
  (`block_node.cc`, 1,358 lines), `column_spanner_admission`, the
  `layout_box_autoscroll` cluster (14 items; Chromium `LayoutBox::Autoscroll` /
  `ScrollRectToVisible` in `layout_box.cc`), `layout_block_caret` and
  `fragmentation_context`.
- **(a)** to fragment: `ColumnSpannerPath`, `LineClampData`. Base→block then
  becomes empty.

**core → commit (24 items).**

- **(a)** `layout_box_result_commit` (6 fns, source
  `layoutng/internal/layout_box_fragment_data.cc`; Chromium
  `LayoutBox::SetLayoutResult` / `AddLayoutResult` in `layout_box.cc`) and
  `output_node_lookup` move down.
- **(b)** Engine publish phases stay above: `begin_layout_output`,
  `publish_layout_output`, `run_layout_without_publication`,
  `run_provisional_layout_without_publication`, `commit_paint_client_state`,
  `collect_container_query_sizes`, the scroll-offset snapshot.
- The standalone crate's lib.rs (`LayoutUpdateKind`,
  `LayoutPublicationBackend`, `PublishLayoutResult`, …) is used by base
  (§5.1).

**core → reuse (20 items).**

- **(a)** `layout_box_result_cache` (6 per-box fns, source
  `layout_box_fragment_data.cc`), `measure_cache` (data on LayoutBox),
  `layout_utils` and `layout_environment_cache` / `font_resolver` move down.
  All cite `layoutng`.
- **(b)** `cached_layout_result` (Chromium `LayoutBox::CachedLayoutResult`,
  one call per box layout) and `has_current_output`.
- The standalone `block_formatting_context`, `cache_decision`,
  `recent_entries`, `layout_cache_status` and `trace` are used by base (§5.1).

**core → forms (19).** **(a)** `form_node_metadata_interface` (the DOM
projection element classes `HTMLInputElement`, `TextControlElement`, …;
source `layoutng/internal/form_node_metadata.h`) and `form_control_types`
move to base. 18 of the references are `Traceable` impls in
`layoutng_trace_bindings.rs`.

**core → out_of_flow (17), float (16), fragmentation (16), grid (11),
svg (8), table (6), replaced (4), custom (3), sizing/list/flex (1).**
All **(a)**: data and LayoutBox helpers citing `layoutng/internal`.

- Out-of-flow: `oof_positioned_node`, `out_of_flow_data`, `anchor_map`,
  `anchor_position_scroll_data`, `anchor_lookup`,
  `non_overflowing_scroll_range`, `successful_position_fallback`.
- Float: the exclusion files and `shape_outside_info`.
- Grid: `grid_layout_data`, `grid_item_data`, `grid_track_collection_types`,
  `grid_lanes_item_group`.
- Table: `table_borders_types`, `table_constraint_space_data`,
  `table_node_interface`, `table_layout_algorithm_types_interface`.
- Replaced: `layout_replaced_interface` (LayoutReplaced, LayoutVideo,
  LayoutMedia; Chromium `core/layout/layout_replaced.h`).
- Custom: `custom_layout_api`, `constraint_space_custom_data`,
  `custom_layout_payload`.
- `layoutng_trace_bindings.rs` holds 24 `Traceable` impl blocks (89 type
  references) for types from 12 packages. Each impl is **(c)**: it moves next
  to its type.

**fragment → inline (17), fragment → out_of_flow (15), fragment →
fragmentation (16), fragment → invalidation (15), fragment → block,
replaced, table (≤5 each).**

- **(a)**: `UsedFont`, `InlineItem*`, `TextFitScale`, `TextItemType`,
  `OofPositionedNode`, `AnchorMap`, `BreakAppeal`, `EarlyBreak`,
  `TableBorders`, `TableConstraintSpaceData` and `NonOverflowingScrollRange`
  move into fragment.
- The others are LayoutObject-class references: LayoutInline, LayoutText,
  LayoutBlockFlow and LayoutReplaced move to base and become opaque handles.
- `fragmentation_utils` (`IsBreakInside`, `CanPaintMultipleFragmentsForFragment`)
  stays in base. The 2 residual calls come from fragment data and move up
  with their caller methods.

**Lateral algorithm edges into sizing (164 items from 12 L3 packages) and
fragmentation (116 items from 11 L3 packages).** **(a)**: both packages move into
base (Chromium `core/layout/length_utils.h`, `space_utils.h`,
`baseline_utils.h`, `fragmentation_utils.h` sit at `core/layout/` root and
are used by `block_node.cc`). `break_appeal` and `early_break` go to fragment
because `LayoutResult` holds them.

**Lateral edges into block from flex, grid, table, forms, out_of_flow, svg,
multicol, inline, reuse and invalidation.** They name `BlockNode`,
`LayoutBlock` and `LayoutBlockFlow`, which become base. What remains:

- multicol → block: `BlockLayoutAlgorithm`, `AlignBlockContent`. This is an
  acyclic Cargo dependency today; keep it.
- table → block: `BlockChildIterator`. **(a)** to base (Chromium
  `core/layout/block_child_iterator.h`).
- block → inline: 6 items from `block_inline_layout.rs`. **(c)** move
  `RegisterInlineSupport` into inline, which then fills the existing
  `inline_support` slot.

### 4.3 What remains after the moves (needs slots or method-level work)

After the Appendix B moves, 20 package pairs still point upward (199 item
references). Each one is resolved as follows:

| residual edge | items (files) | content | resolution |
|---|---:|---|---|
| fragment → base | 134 (35 kept data files) | opaque-handle types: `LayoutObject`, `LayoutBox`, `LayoutBoxModelObject`, `LayoutView`, `LayoutBlockFlow`, `LayoutInline`, `LayoutText`, `LayoutTextCombine`, `BlockNode`, `InlineNode`, `LayoutInputNode`, `Node`/`Element`/`ContainerNode`, `PaintLayer`, `PaintLayerScrollableArea` (≈17); builders and their consumers (`BoxFragmentBuilder`, `FragmentBuilder`, `FragmentItemsBuilder`, `LogicalLineItem`, `LogicalFragment`, `ConstraintSpaceBuilder`); style enums and helpers from `layout_input.rs` (`Display`, `Position`, `Overflow`, `FloatSide`, `ListStyleType`, `BoxDecorationBreak`, `OverscrollBehavior`, `NodeKind`, `PaintStyle`, …, 12) and `style_projection.rs` (`FragmentStyle*`, 8); ≈60 `Live*`/`Export*` paint-export fns called from `fragment_tree.rs`; `anchor_map_operations` (5), `fragmentation_utils` (2), `transform_utils`, `pagination_fragment_utils`, `scrollable_overflow_calculation` | opaque handle (R4 step 1); style enums and `FragmentStyle*` helpers **(a)** down (they only read `ComputedStyle`); builder constructors **(c)**; `fragment_tree.rs` split **(c)**: its live paint queries move to base with the export cluster; the remaining helper calls **(c)** at method level |
| base → invalidation | 59 (15) | engine phases (`tree_update` 7, `input_invalidation` 6, `node_input_update` 2, `tree_facts` 2, `EngineMutation`); helpers (`managed_input_admission` 8, `constraint_equivalence` 5, `css_zoom`, `style_font_resolution`, `timeline_trigger_metadata`, `trigger_scope_projection`, `node_input_state` 2, `positioned_object_invalidation`, `pre_paint_subtree_walk_reasons` 2); standalone vocabulary (`propagation` 17, `trace` 2) | phases **(b)** (§6.2); helpers **(a)**; vocabulary stays a root below base (§5.1) |
| base → inline | 26 (14) | editing facts and queries (`Editing*`, `VisualCaretMove*`, `InnerTextLayoutFacts`, `PositionForPoint`, `InlineCaretPosition`, 21); `LayoutTextFragment`, `LayoutInlineLocalCaretRect`, `OffsetMapping`, `needs_collapsible_whitespace`, `InlineItemResultRubyColumn` | editing **(c)**: extension traits in inline, and the base wrappers move up; the rest **(a)** to base |
| base → commit | 18 (10) | publish phases (4), `commit_paint_client_state`, `collect_container_query_sizes`, scroll-offset snapshot (2); `output_node`/`output_node_mut`; `PaintCommit`; standalone lib (`LayoutUpdateKind`, `LayoutPublicationBackend`, `PublishLayoutResult`, `PaintCommitOutput`, `PaintClientCommit`, `CommitPaintResult`, `LayoutCommitTransaction`) | phases **(b)**; lookups and `PaintCommit` **(a)**; lib stays a root below base (§5.1) |
| base → block | 16 (7) | `layout_box_autoscroll` (14: `LayoutBoxAutoscroll*`, `ScrollRectToVisible*`, `NativeScroll*`, `ScrollAlignment*`, `ScrollIntoViewParams`), `fragmentation_context`, `LayoutBlockLocalCaretRect` | **(a)** to base (Chromium `layout_box.cc` / `layout_block.cc`) |
| base → reuse | 11 (9) | `cached_layout_result`, `has_current_output`; `PreservePaintOnlyFontBinding`; standalone vocabulary (`block_formatting_context` 2, `cache_decision` 2, `RecentEntries`, `LayoutTreeUpdateStats`, `trace` 2) | **(b)** 2; **(a)** 1; vocabulary root (§5.1) |
| block → inline | 6 (1) | `block_inline_layout.rs` uses `Simple/OptimalInlineChildLayoutContext`, `kMaxLinesFor*`, `MeasurePerBlockScale`, `ParagraphScale` | **(c)** move `block_inline_layout.rs` into inline, which fills the existing `inline_support` slot |
| base → replaced | 5 (14) | `LayoutReplaced`, `LayoutVideo`, `LayoutMedia`, `LayoutInputReplaced`, `VideoDisplayMode` | **(a)** `layout_replaced_interface.rs` to base; this also clears fragment→replaced (2) and float/inline/svg/block→replaced (1 each) |
| multicol → block | 2 (1) | `BlockLayoutAlgorithm`, `AlignBlockContent` | keep the acyclic Cargo dependency (Chromium `ColumnLayoutAlgorithm` uses `BlockLayoutAlgorithm` directly) |
| invalidation → reuse, reuse → commit | 2 + 1 | `LayoutTreeUpdateStats`, `retain_tree_span`; `LayoutUpdateKind` | standalone vocabulary below base (§5.1) |
| table → block | 1 (2) | `BlockChildIterator` | **(a)** to base |
| base → mathml / svg / custom | 1 each | `MathScriptType`, `ValidateSvgElementInput`, `CustomLayout` trait | **(a)** to base |

### 4.4 Cross-package `impl` blocks (coherence blockers)

Inherent impls on a type that another package defines, and trait impls where
neither trait nor type is local, must move with the type or become extension
traits. Tests excluded.

| impl site → type's package | blocks | fns | notes |
|---|---:|---:|---|
| fragment → core | 39 | ~375 | inherent: `LayoutObject` ×17, `LayoutBox` ×5, `LayoutView` ×5, `LayoutBoxModelObject` ×4, `PaintStyle` (141 fns in `paint_input.rs`), `LayoutNode`; plus 5 std-trait impls on `PaintStyle` and others, and the local trait `EnginePaintUpdate` for `LayoutEngine` |
| invalidation → core | 35 | 194 | `LayoutObject` ×25, `LayoutBoxModelObject` ×4, `LayoutView` ×2, `ComputedStyle`, `Element`, `Node`, `LayoutObjectTreeStorage`, `LayoutEngine` (local trait) |
| inline → core | 37 | 107 | `LayoutNode` ×29 (editing), `LayoutObject` ×2, `LayoutOutput`, `LayoutView`, `LayoutEngine`, `ConstraintSpace`, `Text`, `LayoutInputNode` (`From`) |
| inline → fragment | 3 | 11 | `InlineCursor`, `PhysicalBoxFragment` (editing queries) |
| block → core | 4 | 36 | `LayoutObject` (29 containers), `LayoutView` |
| core → block | 6 | 18 | `BlockNode` (16 in `block_node_data.rs`), `LayoutBlock`, `LayoutBlockFlow` |
| core → fragment | 3 | 30 | local traits `LayoutAlgorithmBuilder` / `LayoutAlgorithmCoreBuilder` for `BoxFragmentBuilder` |
| core → forms / inline / out_of_flow / replaced | 19 | 19 | `Traceable` impls in `layoutng_trace_bindings.rs` |
| svg → core / block | 3 | 11 | `LayoutObject` (svg dispatch), `LayoutView`, `LayoutBlock` |
| list, sizing, commit, reuse, forms, custom, fragmentation, grid → core | 1–3 each | 1–9 | `LayoutView`, `LayoutBox`, `LayoutEngine`, `ConstraintSpace`, `ConstraintSpaceBuilder`, `RareData`, `GridTrackListInput` |
| grid → fragment | 3 | 5 | `gap_segment_state_aggregator.rs` implements traits on the fragment-declared `GapSegmentStateAggregator` (it is in the assembly only for coherence) |
| out_of_flow → fragment | 2 | 2 | local trait `InlineFragmentItemView` for `FragmentItem` (legal) |

## 5. Contradictions with the target layering

### 5.1 Lifecycle packages are below base today, and much of their code is LayoutObject members

- `layout_core/Cargo.toml` depends on the standalone crates
  `layout_invalidation`, `layout_reuse` and `layout_commit`. Base therefore
  uses their vocabulary from *below*:
  - invalidation: `propagation` (8 `*InvalidationTree` traits + 9
    `propagate_*` fns), `LayoutInvalidationState`, `MarkingBehavior`, `trace`;
  - reuse: `cache_decision`, `block_formatting_context`, `RecentEntries`,
    `LayoutCacheStatus`, `LayoutTreeUpdateStats`, `trace`;
  - commit: `LayoutUpdateKind`, `LayoutPublicationBackend`,
    `PublishLayoutResult`, `trace`.
- The target puts these packages above base. Yet 45 invalidation files,
  8 reuse files and 3 commit files cite `layoutng/internal`. 32 invalidation
  impl blocks are inherent `impl LayoutObject/LayoutBox/LayoutBoxModelObject/
  LayoutView`, and base calls these methods (`SetNeedsLayout*`,
  `SetShouldDoFullPaintInvalidation*`, child-list mutation, style
  will/did-change, result cache and commit) on per-object hot paths.
- **Recommended (option A):** keep each standalone crate as an L0.5 root
  below base, unchanged in role. It owns the vocabulary, the tree-agnostic
  propagation over traits, and the trace category. This satisfies the existing
  rule that "trace names are declared only by their owner package".
  - Move the LayoutObject-bound files into base. They keep calling
    `layout_invalidation::trace::span(...)`, so profiles still attribute them
    to invalidation.
  - Put only the engine-level phases in an above-base half
    (`layout_invalidation`, `layout_reuse` and `layout_commit` proper, each
    with a `Register`). That half needs about 30 fn pointers (§6.2).
  - Naming is an open choice: either `layout_invalidation` (root) plus
    `layout_invalidation_engine` (above), or keep the names and move the
    engine phases into the front crate.
- **Option B (strict target):** everything above base. This needs a slot or
  extension trait for every LayoutObject/LayoutBox member listed in §4.4 that
  base calls, which is about 200 per-object fns. Extension traits are
  impossible when base itself calls them, and AGENTS.md §1.6 requires
  preserving element-by-element hot paths. Not recommended.

### 5.2 `layout_algo_sizing` and `layout_algo_fragmentation` are libraries, not algorithms

They have no layout entry point to register. Every algorithm and base calls
them directly, and their sources cite `layoutng/internal/length_utils.h`,
`fragmentation_utils.h`, `space_utils.h` and `baseline_utils.h`. Fold them into
base (keeping their trace categories in base), and move `break_appeal` and
`early_break` into fragment. If separate attribution matters, they could stay
as L1.5 crates between fragment and base: they need `ConstraintSpace`,
`BoxFragmentBuilder` and `BlockNode`, so they would sit *above* base only if
base did not call them, and base does (16 items).

### 5.3 Builders are construction code, but the snapshot files them with the fragment tree

`box_fragment_builder`, `fragment_builder`, `fragment_items_builder`,
`logical_line_item`, `logical_line_container`, `logical_box_fragment` and
`logical_fragment` cite `layoutng_fragment_tree/…`. They also hold and call
`BlockNode`/`LayoutInputNode` (about 20 methods: `IsAnonymous`,
`IsFixedContainer`, `IsReplaced`, …), and `LayoutResult::new(builder)`,
`PhysicalBoxFragment::Create(builder)` and `FragmentItems::new(builder)`
consume them.

- **Recommendation:** builders go to base. The `Create(builder)`
  constructors move with them as base functions that return fragment data.
  This departs from the snapshot's package placement. It matches Chromium,
  where builders and LayoutObject live in the same `core/layout` target.
- Alternative: keep builders in fragment and make `BlockNode` /
  `LayoutInputNode` opaque handles with about 25 query slots (**guess**).
  This costs more and is slower.

### 5.4 Fragment data references LayoutObject behaviour, not just identity

A pure opaque handle is not enough. The 39 kept fragment data files call
about 113 distinct base behaviours, for example:

- `PhysicalFragment::IsTextControlContainer` → `Element`;
- `FragmentItems` finalization → `SetMayBeNonContiguousIfc`;
- `InlineCursor` → `IsInLayoutNGInlineFormattingContext`;
- `physical_box_fragment.rs` → `LayoutBox::PhysicalFragments` and
  `FragmentDataFromPhysicalFragment`.

Moving each such method into a base extension trait is mechanical, but it
touches the most-used fragment APIs. `fragment_tree.rs` also mixes
`LayoutResultHandle` data with 58 references to live paint-export queries
(`Live*`, `Export*`). It has to be split.

### 5.5 Paint/scroll code inside `layout_fragment`

24 PaintLayer/scroll files (9.7k lines) and 21 paint-export/publish files
(7.1k) live in `layout_fragment`:

- PaintLayer/scroll: `paint_layer*`, `paint_layer_scrollable_area*`,
  `scroll_anchor`, `snap_coordinator`, `sticky_*`, `custom_scrollbar`; Chromium
  `core/paint/paint_layer*.cc`, `core/layout/scroll_anchor.cc`,
  `core/page/scrolling/`.
- Export/publish: Rechrom glue with no `cpp:` citation, mostly called from
  `internal/boundary/layout_boundary.rs` (91 references).

They are not fragment data. The map sends both clusters to base. The export
cluster and `layout_boundary.rs` could instead form part of `layout_commit`
above base, reached through the existing `LayoutBoundarySupport` slots. That
is a decision for R4 or R6.

## 6. Registry slots

### 6.1 Carried over (re-homed into the `LayoutEngine` registry)

`LayoutAlgorithmSet` (140), `LayoutObjectFactorySet` (74),
`LayoutBoundarySupport` (4), the `block_virtual_dispatch` `OnceLock`s (13),
the inline SVG text-font `OnceLock`s (2) and `FullLayoutRegistry` (1, removed)
come to about 234 fields. The 14 `Install*Algorithm` / `Install*Module`
functions become `Register`. Global `OnceLock`s become per-engine fields. The
registry is still reached during layout through `LayoutPassScope`
(thread-local), so no hot-path signature changes.

R1 status: the table is `LayoutRegistry` (`layout_core/src/layout_registry.rs`,
same fields as the former `LayoutAssembly`). Each algorithm crate exposes
`registration::Register(&mut LayoutRegistry)`; block keeps the feature-gated
`block_inline_layout::RegisterInlineSupport`, and the base boundary is
`internal::boundary::registration::Register`. `layout::CreateLayoutRegistry`
calls them in the former install order and `LayoutEngine::new(&LayoutRegistry)`
still copies the table, so construction is unchanged. The `OnceLock` slots are
not moved yet.

### 6.2 New slots required by the upward edges

Signatures are taken from the current functions. `LayoutObjectTree`,
`ConstraintSpace` and similar types are the base and fragment types after the
moves. "→ down" marks a recommended alternative that needs no slot.

| slot (group) | signature | implementing package | replaces |
|---|---|---|---|
| `tree_update.begin` | `fn(&mut LayoutObjectTree)` | invalidation | `tree_update::begin_update` |
| `tree_update.update_subtree` | `fn(&mut LayoutObjectTree, u64, u64, &mut dyn FnMut(&mut LayoutObjectTree, &mut LayoutObject)) -> bool` | invalidation | `update_subtree` (today generic `impl FnOnce`; must become `dyn`) |
| `tree_update.reattach` | `fn(&mut LayoutObjectTree, &[u64])` | invalidation | `reattach` |
| `tree_update.complete` | `fn(&mut LayoutObjectTree, &mut dyn FnMut(&mut LayoutObjectTree))` | invalidation | `complete_update<T>` (generic return must be carried by the closure) |
| `tree_update.finish` | `fn(&mut LayoutObjectTree)` | invalidation | `finish_update` |
| `tree_update.place` | `fn(&mut LayoutObjectTreeStorage, *mut LayoutObject, *mut LayoutObject, *mut Node)` | invalidation | `place` |
| `tree_update.detach` | `fn(&mut LayoutObjectTree, u64, bool) -> bool` | invalidation | `detach_element_layout_tree` |
| `node_input.update` | `fn(&mut LayoutObjectTree, u64, &ComputedStyle, Option<ElementData>) -> Option<NodeInputUpdate>` | invalidation | `update_node_input` |
| `node_input.video_state` | `fn(&mut LayoutObjectTree, u64, VideoLayoutState) -> Option<*mut LayoutObject>` | invalidation | `update_video_layout_state` |
| `input_invalidation` (5) | `editing: fn(&mut LayoutObjectTree, u64) -> bool`; `selection_paint` / `caret_text_paint: fn(&mut LayoutObjectTree, u64) -> Vec<*mut LayoutObject>`; `paint_resource_clients: fn(&mut LayoutObjectTree, &PaintResourceFacts, &PaintResourceFacts) -> usize`; `document_image_clients: fn(&mut LayoutObjectTree, &[PaintImage]) -> usize` | invalidation | `input_invalidation::*` |
| `tree_facts` | `fn(&LayoutObject, &LayoutObjectFactorySet) -> LayoutTreeFacts` | invalidation | `collect_layout_tree_facts` (`LayoutTreeFacts` → down) |
| `apply_mutation` | `fn(&mut LayoutEngine, LayoutMutation<'_>) -> bool` (+ `geometry_input_matches: fn(&LayoutEngine, u64, &ComputedStyle, &StyleResourceBindings, &Option<ElementData>) -> bool`) | invalidation | `EngineMutation` trait impl. Alternative: the front crate calls it directly, with no slot |
| `prepare_tree` | (exists in `LayoutBoundarySupport`) | invalidation | — |
| `cached_layout_result` | `fn(&mut LayoutBox, &ConstraintSpace, *const BlockBreakToken, *const EarlyBreak, *const ColumnSpannerPath, *mut Option<FragmentGeometry>, *mut LayoutCacheStatus) -> *const LayoutResult` | reuse | `cache_lookup::cached_layout_result` (one indirect call per box; or → down) |
| `has_current_output` | `fn(&LayoutEngine, LayoutUpdateKind) -> bool` | reuse | `engine_output_reuse` |
| `font_resolver.construction` / `.engine` | `fn(&ConstraintSpace) -> Box<dyn FontResolver>` | reuse | `font_resolver.rs` (→ down recommended) |
| `publish.begin` | `fn(&mut LayoutEngine, LayoutUpdateKind) -> LayoutOutputPreparation` | commit | `begin_layout_output` |
| `publish.publish` | `fn(&mut LayoutEngine, *mut LayoutObject, LayoutUpdateKind, LayoutResultHandle)` | commit | `publish_layout_output` |
| `publish.run_without_publication` / `.run_provisional` | `fn(&mut LayoutEngine)` | commit | `run_*layout_without_publication` |
| `publish_layout_result` | (exists in `LayoutBoundarySupport`) | commit | — |
| `commit_paint_client_state` | `fn(&LayoutObjectTree, &HashSet<u64>, &mut LayoutResultHandle)` | commit | `client_commit` |
| `container_query_sizes` | `fn(&LayoutEngine) -> Vec<ContainerQuerySizeSnapshot>` | commit | `container_query_sizes` |
| `scroll_offsets.commit` / `.output` | `fn(&mut LayoutObjectTree)` / `fn(&LayoutObjectTree) -> Vec<(u64, Offset)>` | commit | `scroll_offset_snapshot` |
| `inline_support.parallel_flow_break_tokens` | current `InlineChildLayoutContextParallelFlowBreakTokens` signature | inline | link-time symbol (declared in core, defined in inline *and* block) |
| `inline_support.position_for_point_in_fragments` | current `LayoutBoxPositionForPointInFragmentsProvider` signature | inline | link-time symbol |
| `inline_support.should_truncate_overflowing_text` | `fn(*const LayoutBlockFlow) -> bool` | inline | `LayoutBlockFlowShouldTruncateOverflowingTextFromInline` |
| `inline_items_data` (3) | `check_consistency`, `open_tag_items`, `offset_map: fn(&InlineItemsData) -> &Option<TextOffsetMap>` | inline | fragment → inline link-time symbols (or move `InlineItemsData` callers up) |
| `fragment_items_builder` (2) | `new_for_inline`, `add_previous_items_from_inline` | inline | link-time symbols (vanish if `FragmentItemsBuilder` moves to inline) |
| `out_of_flow_inline_child_candidate` | current `FragmentBuilderAddOutOfFlowInlineChildCandidate` signature | inline | link-time symbol |
| `table_node` (4) | `is_empty_section: fn(&LayoutInputNode) -> bool`; `cell_colspan` / `cell_rowspan` / `column_span: fn(&LayoutInputNode) -> u32` | table | `LayoutInputNode*FromTable` |
| `forms_support.fieldset_content` | `fn(&BlockNode) -> BlockNode` | forms | `BlockNodeGetFieldsetContentFromForms` |
| `editing.table_row_needs_newline` / `.table_facts` | `fn(*const LayoutObject) -> bool` / `fn(*const LayoutObject) -> (bool, bool, bool)` | table | inline → table link-time symbols |
| `editing.list_item_value` | `fn(*const LayoutObject) -> Option<i32>` | list | inline → list link-time symbol |
| `row_gap.sticky_container` / `.total_row_gap_count` | existing `Dispatch*` signatures | base (internal) | fragment → assembly symbols (disappear once the callers move to base) |

Totals: about 46 new fn pointers in about 19 groups. That is invalidation 17,
reuse 4, commit 8, inline 9, table 6, forms 1 and list 1. The two
`row_gap` entries are internal to base and not counted. Removing the items
marked "→ down" brings it to about 43. Fragment's method-level moves (§5.4) add
5–15 slots on top (**guess**; this depends on how many data algorithms must
call LayoutObject behaviour in place rather than from a base extension
method).

## 7. Per-stage estimate (R1–R7)

The stage boundaries are a proposal (**guess**); the counts are measured.

| stage | content | files moved | lines moved | new slots | main risks |
|---|---|---:|---:|---:|---|
| R1 registry | `LayoutEngine` owns a registry built from `LayoutAlgorithmSet`, `LayoutObjectFactorySet`, `LayoutBoundarySupport`, the 12 `OnceLock` statics (15 fn pointers) and `FullLayoutRegistry`; `Install*` → `Register` (rename done, see §6.1); front crate calls them | 0 (≈20 edited) | — | 0 (≈234 re-homed) | process-global `OnceLock` → per-engine (tests that build several engines); `LayoutPassScope` access stays thread-local |
| R2 leaf algorithms | move base-owned files out of float, out_of_flow, grid, svg, table, forms, list, custom, flex, multicol, replaced into `layout_core/`; their fragment-data files into `layout_fragment/`; move the `Traceable` impls to their types; fold sizing and fragmentation into base | 64 (+11 sizing/fragmentation) | ≈13.9k (+5.0k) | ≈8 (table 4 + 2 editing, forms 1, list 1) | `grid_track_collection.rs` `#[no_mangle]` bodies and `gap_segment_state_aggregator.rs` coherence (impl on a fragment-declared type); a clean `#[path]` removal still compiles in the assembly until R6 |
| R3 lifecycle | invalidation/reuse/commit: 61 LayoutObject-bound files → base, 1 → fragment (≈10.8k lines); engine phases stay above with slots; decide §5.1 (root crate + engine half) | 62 | ≈10.8k | ≈29 | trace-ownership contract; per-object hot paths must stay direct calls; `tree_update` generic closures become `dyn` |
| R4 fragment | fragment data crate (39 own + 3 standalone + 12 core + 29 algo data files (moved in R2) ≈ 82 files); 111 fragment files → base; builders → base; LayoutObject opaque handle; ≈113 base behaviours called from data → base extension traits; split `fragment_tree.rs` | ≈123 (111 up, 12 down) | ≈33.1k | 5–15 (**guess**) | the largest edge (160 items + 286 methods); `LayoutResult` embeds `ConstraintSpace` (closure into exclusions, line clamp, table/grid/custom rare data); `PaintStyle` (141 fns impl'd in fragment on a core type); fragment API churn for Paint/Interaction through the front crate |
| R5 inline | 16 base-cited inline files → base (LayoutText/LayoutInline/… ≈3.6k lines), 6 data files → fragment (≈1.1k); 19 editing-query files → extension traits in inline (118 fns); `block_inline_layout.rs` → inline; link-time symbols → slots; drop the `translation_in_progress` gating | 23 + 19 converted | ≈4.7k moved | ≈9 | inline is 37.5k lines and has never compiled standalone; the duplicate `InlineChildLayoutContextParallelFlowBreakTokens` definition; editing API surface used by DOM/Interaction via the front crate |
| R6 base crate | `layout_core` → `layout_base`, absorbing the assembly's real code (`block_virtual_dispatch.rs` 2.8k, `out_of_flow_element_data`, `layout_invalidation_reason_symbols`); distribute `untranslated_symbols.rs` stubs to the declaring crates (or delete them per `DEAD_CODE.md`); delete the `include!`s and `layout_algo_inline_assembly` | 5 + 2 crates removed | ≈3.5k | 0 | 783 `layout_assembly::` paths in 145 files and about 2,300 `crate::`/`super::` paths in core/fragment/inline need a scripted rewrite; MSVC needs every extern symbol defined in the linked crate set |
| R7 front crate | `layout` builds the engine and calls every `Register`; remove the `extern crate layout_assembly as …` aliases in block; one-way Cargo graph check; README | ≈20 edited | — | 0 | the ordering of `Register` calls (the order `CreateLayoutRegistry` in `layout/src/lib.rs` documents) |

### Status: R2+R3 file relocation (done)

All Appendix B files that `layout_assembly` compiles through `#[path]` or a
`mod` in an `include!`d root now sit in their owner package. The clear §4.3
base moves came along too. This was a physical move only: every
`#[path]` / `include!` string was rewritten, the compile unit declares the
same modules under the same names, and no logic changed. Paths elsewhere in
this map (§1–§6, Appendices A and B) still describe the snapshot before the
move. Base files go to `layout_core/src/<area>/`, which becomes `layout_base`
in R6. Fragment data files go to `layout_fragment/src/<area>/`.

| from | to | files |
|---|---|---:|
| `layout_fragment` | `layout_core/src/fragment/` | 110 |
| `layout_invalidation` | `layout_core/src/invalidation/` (45 Appendix B + 9 §4.3 helpers) | 54 |
| `layout_invalidation` | `layout_fragment/src/invalidation/` | 1 |
| `layout_reuse` | `layout_core/src/reuse/` | 7 |
| `layout_commit` | `layout_core/src/commit/` (3 + `output_node_lookup`, `engine_paint_commit`) | 5 |
| `layout_core/src/internal/` | `layout_fragment/src/core/` | 12 |
| `layout_algo_block` | `layout_core/src/block/` (10 + autoscroll, `fragmentation_context`, `layout_block_caret`) / `layout_fragment/src/block/` | 13 / 2 |
| `layout_algo_inline` | `layout_core/src/inline/` (16 + `layout_text_fragment`, `offset_mapping`) / `layout_fragment/src/inline/` | 18 / 6 |
| `layout_algo_float` | `layout_core/src/float/` / `layout_fragment/src/float/` | 7 / 5 |
| `layout_algo_out_of_flow` | `layout_core/src/out_of_flow/` / `layout_fragment/src/out_of_flow/` | 8 / 3 |
| `layout_algo_grid` | `layout_core/src/grid/` / `layout_fragment/src/grid/` | 3 / 3 |
| `layout_algo_table` | `layout_core/src/table/` / `layout_fragment/src/table/` | 2 / 3 |
| `layout_algo_svg` | `layout_core/src/svg/` (8 + `svg_metadata_validation`) | 9 |
| `layout_algo_sizing` | `layout_core/src/sizing/` | 5 |
| `layout_algo_fragmentation` | `layout_core/src/fragmentation/` / `layout_fragment/src/fragmentation/` | 2 / 2 |
| `layout_algo_forms` | `layout_core/src/forms/` / `layout_fragment/src/forms/` | 1 / 2 |
| `layout_algo_list` | `layout_core/src/list/` | 2 |
| `layout_algo_custom` | `layout_fragment/src/custom/` | 2 |
| `layout_algo_flex` | `layout_fragment/src/flex/` | 1 |
| `layout_algo_multicol` | `layout_core/src/multicol/` | 1 |
| `layout_algo_replaced` | `layout_core/src/replaced/` (`layout_replaced_interface`) | 1 |

The `layout_algo_sizing` and `layout_algo_fragmentation` packages were left
holding only an empty `lib.rs` and nothing named them, so both were removed
from the workspace.

Not moved:

- **The standalone crate compiles the file itself, so moving it needs a
  crate-level change.**
  - `layout_invalidation`: `content_change_type.rs`,
    `layout_invalidation_reason.rs`, `layout_invalidation_state.rs`.
  - `layout_reuse`: `layout_cache_status.rs`, `min_max_sizes_cache.rs`.
  - `layout_algo_block`: `block_child_iterator.rs`.
  - `layout_algo_custom`: `custom_layout_api.rs`.
  - `layout_algo_mathml`: `mathml_node_metadata.rs` (`MathScriptType`).
- **`layout_reuse/src/layout_environment_cache.rs`.** It declares the trace
  literal `"layout_reuse.FontEnvironment"`. The
  `lifecycle_trace_names_are_declared_only_by_their_owner_package` test only
  allows that literal under `layout_reuse/`, so the file has to switch to a
  `layout_reuse::trace` helper before it can move.
- **`layout_fragment/src/lib.rs`.** It stays as the `include!`d fragment
  module root, and R4 step 8 merges it with `package.rs`.
- **`layout_algo_list/src/layout_object_list_marker_queries.rs`.** It no
  longer exists.

### Suggested order inside R4 (fragment)

1. Add the opaque `LayoutObject` handle (and `BlockNode`/`LayoutInputNode`
   handles) to fragment, plus the typed accessor extension trait in base. Make
   no behaviour change.
2. Move the data closure down. In order: `StyleVariant`, `TextOffsetRange`,
   `SplitAxisItem`, `HitTestPhase`, `SelectionState`, `MapCoordinatesFlags`,
   `TextDecorationOffset`; then `ConstraintSpace` with its rare data
   (`ExclusionSpace` and the exclusion files, `LineClampData`,
   `TableConstraintSpaceData`, `CustomData`, `GridLayoutSubtree`,
   `BreakAppeal`); then the `LayoutResult` payload (`EarlyBreak`,
   `ColumnSpannerPath`, `GridLayoutData`, `DevtoolsFlexInfo`,
   `NonOverflowingScrollRange`, `SerializedScriptValue`); then fragment
   payload types (`OofPositionedNode`, `AnchorMap`, `TableBorders`,
   `UsedFont`, `InlineItem*`).
3. Move the 32 LayoutObject/LayoutBox/LayoutView method files (inherent impls)
   up. This step is mechanical.
4. Move the PaintLayer/scroll cluster (24 files) up.
5. Move builders (8 files) up, along with the `Create(builder)` constructors.
6. Move the paint-export/publish cluster (21) and the remaining glue (26) up.
   Split `fragment_tree.rs`.
7. Turn the ≈113 data→base behaviour calls into base extension-trait methods,
   adding a slot only where the call must stay inside a fragment algorithm.
8. Fragment compiles alone (`package.rs` + `lib.rs` merged); delete its
   `include!`.

### Suggested order inside R5 (inline)

1. Move the base classes (`layout_text_interface`, `layout_text_core`,
   `layout_inline`, `layout_text_combine_interface`, `layout_text_fragment`,
   `inline_node_interface`, `inline_node_core`, `inline_node_data`,
   `layout_object_editing_queries`'s `AssociatedLayoutObjectOf`,
   `text_input_environment`, `variable_length_transform_result`,
   `offset_mapping`) to base. Move the data files to fragment if R4 did not
   already.
2. Turn the inline→fragment and core→inline link-time symbols into
   `inline_support` slots. Pick one `ParallelFlowBreakTokens` definition.
3. Convert the editing inherent impls on `LayoutNode`, `LayoutOutput`,
   `LayoutObject`, `InlineCursor` and `PhysicalBoxFragment` into extension
   traits. Move the base wrapper methods that name the editing facts up into
   inline, and re-export them through the front crate.
4. Move `block_inline_layout.rs` (`RegisterInlineSupport`) into inline.
5. Remove the `translation_in_progress` gating so inline compiles standalone
   on base + fragment.

R4 before R5 is assumed. LayoutText and LayoutInline are referenced by
fragment data (`LayoutInline` ×17 files), so the opaque handle from R4 step 1
must exist first.

## Appendix A: `#[path]` / `include!` inventory (assembly unit)

The duplicate `layout_algo_inline_assembly` unit declares the same set (339
`#[path]`) and is omitted. **x** marks a target outside the declaring file's
package. The paths are relative to `crates/layout/`.

| declaring file | n | targets (`[mod=]pkg:file`; mod omitted when it is the file stem or `<pkg>_<stem>`; a_ = layout_algo_) |
|---|---:|---|
| `algo_block/src/block_node.rs` | 1 | a_block:block_node_traversal.rs |
| `algo_block/src/layout_block.rs` | 5 | invalidation:positioned_object_invalidation.rs, invalidation:layout_block_style_change.rs, a_block:layout_block_caret.rs, invalidation:layout_block_child_mutation.rs, a_block:layout_block_traversal.rs |
| `algo_block/src/layout_block_flow.rs` | 4 | invalidation:layout_block_flow_style_change.rs, invalidation:layout_block_flow_child_normalization.rs, invalidation:layout_block_flow_child_mutation.rs, invalidation:layout_block_flow_multicol_update.rs |
| `algo_inline/src/layout_inline.rs` | 4 | visual_rect=fragment:layout_inline_visual_rect.rs, a_inline:layout_inline_caret.rs, fragment:inline_client_geometry.rs, fragment:inline_quad_geometry.rs |
| `algo_inline/src/layout_text.rs` | 2 | editing_queries=a_inline:layout_text_editing_queries.rs, quad_geometry=a_inline:layout_text_quad_geometry.rs |
| `algo_out_of_flow/src/anchor_map.rs` | 1 | a_out_of_flow:anchor_map_operations.rs |
| `assembly/src/lib.rs` | 9 | include! (root)=core:lib.rs, include! layout_fragment_package=fragment:lib.rs, include! layout_algo_inline_package=a_inline:lib.rs, fragment:decorating_box.rs, fragment:inline_paint_context.rs, fragment:line_relative_rect.rs, fragment:text_decoration_info.rs, grid_gap_segment_state_aggregator=a_grid:gap_segment_state_aggregator.rs, grid_track_collection_implementation=a_grid:grid_track_collection.rs |
| `core/src/internal/anchor_map.rs` | 1 | a_out_of_flow:anchor_map.rs |
| `core/src/internal/anchor_map_services.rs` | 1 | a_out_of_flow:anchor_map_services.rs |
| `core/src/internal/anchor_position_scroll_data.rs` | 1 | a_out_of_flow:anchor_position_scroll_data.rs |
| `core/src/internal/anchor_scroll_services.rs` | 1 | a_out_of_flow:anchor_scroll_services_interface.rs |
| `core/src/internal/baseline_utils.rs` | 1 | a_sizing:baseline_utils.rs |
| `core/src/internal/block_node.rs` | 1 | a_block:block_node.rs |
| `core/src/internal/boundary/clip_path_projection.rs` | 1 | fragment:clip_path_operation_projection.rs |
| `core/src/internal/boundary/input.rs` | 1 | invalidation:input.rs |
| `core/src/internal/boundary/layout_boundary.rs` | 27 | reuse:layout_environment_cache.rs, a_float:shape_image_environment.rs, a_inline:text_input_environment.rs, layout_boundary_layout_environment=core:internal/boundary/layout_environment.rs, reuse:font_resolver.rs, invalidation:layout_object_factory.rs, fragment:fragment_export_support.rs, fragment:fragment_property_export.rs, fragment:fragment_paint_data_init.rs, fragment:text_paint_export.rs, fragment:fragment_geometry_export.rs, fragment:scrollbar_paint_export.rs, fragment:specialized_paint_export.rs, fragment:collapsed_table_paint_export.rs, fragment:ink_overflow_export.rs, fragment:nine_piece_image_export.rs, fragment:global_paint_export.rs, fragment:inline_fragment_paint_policy.rs, fragment:scrollbar_thumb_geometry.rs, fragment:sticky_paint_export.rs, fragment:scroll_paint_property_refresh.rs, fragment:scroll_paint_property_index.rs, invalidation:prepare_tree.rs, a_inline:initial_letter_style.rs, fragment:layout_result_publish.rs, fragment:retained_paint_refresh.rs, fragment:paint_cache_refresh.rs |
| `core/src/internal/boundary/mask_image_projection.rs` | 1 | fragment:mask_image_style_projection.rs |
| `core/src/internal/boundary/node_metadata_input.rs` | 2 | invalidation:node_metadata_finalize.rs, invalidation:node_metadata_incremental_relations.rs |
| `core/src/internal/boundary/paint_style_export.rs` | 1 | fragment:paint_style_export.rs |
| `core/src/internal/break_appeal.rs` | 1 | a_fragmentation:break_appeal.rs |
| `core/src/internal/caret_rect.rs` | 1 | fragment:caret_rect.rs |
| `core/src/internal/column_spanner_path.rs` | 1 | a_block:column_spanner_path.rs |
| `core/src/internal/constraint_space.rs` | 2 | a_fragmentation:constraint_space_fragmentation.rs, a_inline:constraint_space_block_in_inline.rs |
| `core/src/internal/constraint_space_builder_style.rs` | 1 | a_sizing:min_max_constraint_space_builder.rs |
| `core/src/internal/constraint_space_custom_data.rs` | 1 | a_custom:constraint_space_custom_data.rs |
| `core/src/internal/css/out_of_flow_data.rs` | 1 | a_out_of_flow:out_of_flow_data.rs |
| `core/src/internal/css/successful_position_fallback.rs` | 1 | a_out_of_flow:successful_position_fallback.rs |
| `core/src/internal/css_zoom.rs` | 1 | invalidation:css_zoom.rs |
| `core/src/internal/custom_layout_payload.rs` | 1 | a_custom:custom_layout_payload.rs |
| `core/src/internal/custom_scrollbar.rs` | 1 | fragment:custom_scrollbar.rs |
| `core/src/internal/devtools_flex_info.rs` | 1 | a_flex:devtools_flex_info.rs |
| `core/src/internal/early_break.rs` | 1 | a_fragmentation:early_break.rs |
| `core/src/internal/exclusions/exclusion_area.rs` | 1 | a_float:exclusion_area.rs |
| `core/src/internal/exclusions/exclusion_space.rs` | 1 | a_float:exclusion_space.rs |
| `core/src/internal/exclusions/layout_opportunity.rs` | 1 | a_float:layout_opportunity.rs |
| `core/src/internal/exclusions/shape_exclusions.rs` | 1 | a_float:shape_exclusions.rs |
| `core/src/internal/form_control_sizing_service.rs` | 1 | a_forms:form_control_sizing_service.rs |
| `core/src/internal/form_control_types.rs` | 1 | a_forms:form_control_types.rs |
| `core/src/internal/form_node_metadata.rs` | 1 | a_forms:form_node_metadata_interface.rs |
| `core/src/internal/fragmentation_utils.rs` | 1 | a_fragmentation:fragmentation_utils.rs |
| `core/src/internal/frame_set_layout_data.rs` | 1 | fragment:frame_set_layout_data.rs |
| `core/src/internal/gap/cross_gap.rs` | 1 | fragment:cross_gap.rs |
| `core/src/internal/gap/gap_decoration_utils.rs` | 1 | fragment:gap_decoration_utils.rs |
| `core/src/internal/gap/gap_geometry.rs` | 1 | fragment:gap_geometry.rs |
| `core/src/internal/gap/gap_intersection.rs` | 1 | fragment:gap_intersection.rs |
| `core/src/internal/gap/gap_utils.rs` | 1 | fragment:gap_utils.rs |
| `core/src/internal/gap/main_gap.rs` | 1 | fragment:main_gap.rs |
| `core/src/internal/grid_item.rs` | 1 | a_grid:grid_item_data.rs |
| `core/src/internal/grid_lanes_item_group.rs` | 1 | a_grid:grid_lanes_item_group.rs |
| `core/src/internal/grid_layout_data.rs` | 1 | a_grid:grid_layout_data.rs |
| `core/src/internal/grid_subtree.rs` | 1 | a_grid:grid_subtree.rs |
| `core/src/internal/grid_track_collection.rs` | 1 | a_grid:grid_track_collection_types.rs |
| `core/src/internal/hyphen_result.rs` | 1 | a_inline:hyphen_result_interface.rs |
| `core/src/internal/ink_overflow.rs` | 1 | fragment:ink_overflow.rs |
| `core/src/internal/inline_containing_block_utils.rs` | 1 | a_out_of_flow:inline_containing_block_utils_interface.rs |
| `core/src/internal/inline_item.rs` | 1 | a_inline:inline_item_interface.rs |
| `core/src/internal/inline_item_result.rs` | 1 | a_inline:inline_item_result_interface.rs |
| `core/src/internal/inline_item_segment.rs` | 1 | a_inline:inline_item_segment_interface.rs |
| `core/src/internal/inline_item_span.rs` | 1 | a_inline:inline_item_span.rs |
| `core/src/internal/inline_item_text_index.rs` | 1 | a_inline:inline_item_text_index.rs |
| `core/src/internal/inline_node.rs` | 1 | a_inline:inline_node_interface.rs |
| `core/src/internal/inline_node_core.rs` | 1 | a_inline:inline_node_core.rs |
| `core/src/internal/inline_node_data.rs` | 1 | a_inline:inline_node_data.rs |
| `core/src/internal/layout_block.rs` | 1 | a_block:layout_block.rs |
| `core/src/internal/layout_block_flow.rs` | 1 | a_block:layout_block_flow.rs |
| `core/src/internal/layout_box.rs` | 14 | layout_fragment_visual_rect=fragment:layout_box_visual_rect.rs, layout_fragment_box_caret=fragment:layout_box_caret.rs, fragment:layout_box_fragment_geometry.rs, fragment:layout_box_fragment_access.rs, fragment:layout_box_background.rs, fragment:layout_box_resource_priority.rs, fragment:layout_result_overflow.rs, fragment:physical_fragment_list.rs, invalidation:transform_invalidation.rs, invalidation:layout_box_fragment_retirement.rs, reuse:layout_box_result_cache.rs, commit:layout_box_result_commit.rs, a_block:column_spanner_admission.rs, a_block:layout_box_autoscroll.rs |
| `core/src/internal/layout_box_core_services.rs` | 2 | fragment:scrollable_overflow_recalc.rs, a_out_of_flow:anchor_lookup.rs |
| `core/src/internal/layout_box_layout.rs` | 1 | commit:layout_box_post_layout_commit.rs |
| `core/src/internal/layout_box_lifecycle.rs` | 2 | invalidation:layout_box_style_change.rs, invalidation:layout_box_tree_lifecycle.rs |
| `core/src/internal/layout_box_model_lifecycle.rs` | 1 | invalidation:layout_box_model_lifecycle.rs |
| `core/src/internal/layout_box_model_object.rs` | 5 | fragment:background_transfer.rs, fragment:layout_box_model_scrollable_area.rs, fragment:layout_box_model_outline.rs, fragment:empty_element_caret.rs, invalidation:layout_box_model_layer_invalidation.rs |
| `core/src/internal/layout_box_model_tree.rs` | 1 | invalidation:layout_box_model_tree_mutation.rs |
| `core/src/internal/layout_box_scrollbar_gutter.rs` | 1 | a_sizing:layout_box_scrollbars.rs |
| `core/src/internal/layout_box_utils.rs` | 1 | fragment:layout_box_location_update.rs |
| `core/src/internal/layout_box_visual_overflow.rs` | 1 | fragment:layout_box_visual_overflow.rs |
| `core/src/internal/layout_custom_scrollbar_part.rs` | 1 | fragment:layout_custom_scrollbar_part.rs |
| `core/src/internal/layout_geometry_invalidation.rs` | 1 | layout_invalidation_geometry=invalidation:geometry_invalidation.rs |
| `core/src/internal/layout_inline.rs` | 1 | a_inline:layout_inline.rs |
| `core/src/internal/layout_input.rs` | 3 | a_grid:grid_track_list_input.rs, invalidation:element_data_equivalence.rs, invalidation:computed_style_equivalence.rs |
| `core/src/internal/layout_node_metadata.rs` | 7 | layout_algo_svg_metadata_validation=a_svg:svg_metadata_validation.rs, invalidation:timeline_trigger_metadata.rs, layout_algo_list_marker_lookup=a_list:list_marker_lookup.rs, invalidation:element_metadata_input.rs, invalidation:metadata_append.rs, invalidation:node_input_state.rs, invalidation:node_metadata_tree.rs |
| `core/src/internal/layout_node_style.rs` | 1 | invalidation:layout_node_style.rs |
| `core/src/internal/layout_object.rs` | 21 | invalidation:layout_object_state_propagation.rs, invalidation:layout_object_image_observers.rs, fragment:layout_object_fragment_queries.rs, fragment:visual_rect_mapping.rs, fragment:layout_object_selection_queries.rs, fragment:layout_object_paint_dispatch.rs, fragment:layout_object_paint_state.rs, fragment:layout_object_effective_style.rs, fragment:layout_object_gap_queries.rs, fragment:layout_object_inline_fragment_state.rs, invalidation:layout_object_paint_invalidation.rs, fragment:layout_object_layer_queries.rs, fragment:layout_object_coordinate_conversion.rs, fragment:layout_object_bounding_geometry.rs, fragment:layout_object_scroll_container_queries.rs, a_inline:layout_object_editing_queries.rs, a_list:layout_object_list_marker_queries.rs, invalidation:layout_object_layer_mutation.rs, invalidation:layout_object_style_services.rs, invalidation:layout_object_prepaint_invalidation.rs, invalidation:layout_object_display_lock.rs |
| `core/src/internal/layout_object_builder.rs` | 14 | a_block:fragmentation_context.rs, a_inline:collapsible_whitespace.rs, commit:output_node_lookup.rs, commit:scroll_offset_snapshot.rs, layout_fragment_paint_input=fragment:paint_input_update.rs, layout_fragment_scroll_input=fragment:scroll_input_update.rs, layout_invalidation_input_updates=invalidation:input_invalidation.rs, invalidation:managed_input_admission.rs, invalidation:managed_object_update.rs, layout_invalidation_node_input=invalidation:node_input_update.rs, layout_invalidation_package=invalidation:tree_update.rs, reuse:managed_input_index.rs, reuse:managed_object_compatibility.rs, reuse:paint_only_font_binding.rs |
| `core/src/internal/layout_object_child_list.rs` | 1 | invalidation:layout_object_child_list.rs |
| `core/src/internal/layout_object_containers.rs` | 1 | a_block:layout_object_containers.rs |
| `core/src/internal/layout_object_core_services.rs` | 7 | invalidation:layout_object_style_update.rs, invalidation:layout_object_tree_lifecycle.rs, invalidation:layout_object_child_mutation.rs, fragment:layout_object_overflow_recalc.rs, fragment:layout_object_geometry_queries.rs, reuse:layout_object_font_validity.rs, a_svg:layout_object_svg_dispatch.rs |
| `core/src/internal/layout_object_hot.rs` | 1 | invalidation:layout_object_hot_invalidation.rs |
| `core/src/internal/layout_object_inlines.rs` | 1 | invalidation:layout_object_dirty_state.rs |
| `core/src/internal/layout_object_tree.rs` | 1 | invalidation:layout_object_tree.rs |
| `core/src/internal/layout_replaced.rs` | 1 | a_replaced:layout_replaced_interface.rs |
| `core/src/internal/layout_scrollable_area.rs` | 1 | fragment:paint_layer_scrollable_area.rs |
| `core/src/internal/layout_sticky_constraints.rs` | 1 | fragment:layout_sticky_constraints.rs |
| `core/src/internal/layout_text.rs` | 1 | a_inline:layout_text_interface.rs |
| `core/src/internal/layout_text_combine.rs` | 1 | a_inline:layout_text_combine_interface.rs |
| `core/src/internal/layout_text_core.rs` | 1 | a_inline:layout_text_core.rs |
| `core/src/internal/layout_transform.rs` | 1 | fragment:paint_layer_transform.rs |
| `core/src/internal/layout_tree_invalidation.rs` | 1 | layout_invalidation_tree=invalidation:layout_tree_invalidation.rs |
| `core/src/internal/layout_utils.rs` | 1 | reuse:layout_utils.rs |
| `core/src/internal/layout_view.rs` | 14 | layout_fragment_visual_rect=fragment:layout_view_visual_rect.rs, fragment:frame_view_pending_property_updates.rs, invalidation:layout_view_destruction.rs, invalidation:layout_view_invalidation.rs, invalidation:layout_view_child_mutation.rs, a_block:layout_view_root_layout.rs, a_block:layout_view_block_queries.rs, a_svg:svg_text_descendant_index.rs, a_inline:layout_view_variable_length_transform_index.rs, a_list:layout_view_list_index.rs, fragment:layout_view_geometry.rs, fragment:layout_view_fragment_queries.rs, fragment:layout_view_paint_policy.rs, commit:layout_view_commit.rs |
| `core/src/internal/length_utils.rs` | 1 | a_sizing:length_utils.rs |
| `core/src/internal/line_clamp_data.rs` | 1 | a_block:line_clamp_data.rs |
| `core/src/internal/mathml_paint_info.rs` | 1 | fragment:mathml_paint_info.rs |
| `core/src/internal/measure_cache.rs` | 1 | reuse:measure_cache.rs |
| `core/src/internal/motion_path_geometry.rs` | 1 | fragment:motion_path_shape_projection.rs |
| `core/src/internal/multicol_break_token_data.rs` | 1 | a_multicol:multicol_break_token_data.rs |
| `core/src/internal/naming_scope.rs` | 1 | invalidation:naming_scope.rs |
| `core/src/internal/non_overflowing_scroll_range.rs` | 1 | a_out_of_flow:non_overflowing_scroll_range.rs |
| `core/src/internal/oof_positioned_node.rs` | 1 | a_out_of_flow:oof_positioned_node.rs |
| `core/src/internal/outline_geometry.rs` | 1 | fragment:outline_geometry.rs |
| `core/src/internal/outline_info.rs` | 1 | fragment:outline_info.rs |
| `core/src/internal/outline_rect_collector.rs` | 1 | fragment:outline_rect_collector.rs |
| `core/src/internal/overflow_model.rs` | 1 | fragment:overflow_model.rs |
| `core/src/internal/pagination_utils.rs` | 1 | fragment:pagination_fragment_utils.rs |
| `core/src/internal/paint_input.rs` | 1 | fragment:paint_input.rs |
| `core/src/internal/paint_layer.rs` | 1 | fragment:paint_layer.rs |
| `core/src/internal/positioned_float.rs` | 1 | a_float:positioned_float.rs |
| `core/src/internal/pre_paint_subtree_walk_reasons.rs` | 1 | invalidation:pre_paint_subtree_walk_reasons.rs |
| `core/src/internal/resolved_text_layout_attributes_iterator.rs` | 1 | a_svg:resolved_text_layout_attributes_iterator.rs |
| `core/src/internal/scroll_anchor.rs` | 1 | fragment:scroll_anchor.rs |
| `core/src/internal/scroll_layout_scope.rs` | 1 | fragment:paint_layer_scrollable_area_scope.rs |
| `core/src/internal/scrollable_overflow_calculator.rs` | 1 | fragment:scrollable_overflow_calculator.rs |
| `core/src/internal/scrollbar.rs` | 1 | fragment:scrollbar_identity.rs |
| `core/src/internal/scrollbar_part.rs` | 1 | fragment:scrollbar_part.rs |
| `core/src/internal/shapes/shape.rs` | 1 | a_float:shape.rs |
| `core/src/internal/shapes/shape_image_services.rs` | 1 | a_float:shape_image_services_interface.rs |
| `core/src/internal/shapes/shape_interval.rs` | 1 | a_float:shape_interval.rs |
| `core/src/internal/shapes/shape_outside_info.rs` | 1 | a_float:shape_outside_info.rs |
| `core/src/internal/snap_area.rs` | 1 | fragment:snap_area.rs |
| `core/src/internal/space_utils.rs` | 1 | a_sizing:space_utils.rs |
| `core/src/internal/sticky_position_scrolling_constraints.rs` | 1 | fragment:sticky_position_scrolling_constraints.rs |
| `core/src/internal/svg_character_data.rs` | 1 | a_svg:svg_character_data.rs |
| `core/src/internal/svg_inline_node_data.rs` | 1 | a_svg:svg_inline_node_data.rs |
| `core/src/internal/svg_layout_info.rs` | 1 | a_svg:svg_layout_info.rs |
| `core/src/internal/svg_length_adjust_type.rs` | 1 | a_svg:svg_length_adjust_type.rs |
| `core/src/internal/svg_text_attributes_request.rs` | 1 | a_svg:svg_text_attributes_request.rs |
| `core/src/internal/table_borders.rs` | 1 | a_table:table_borders_types.rs |
| `core/src/internal/table_column_location.rs` | 1 | a_table:table_column_location.rs |
| `core/src/internal/table_constraint_space_data.rs` | 1 | a_table:table_constraint_space_data.rs |
| `core/src/internal/table_fragment_data.rs` | 1 | fragment:table_fragment_data.rs |
| `core/src/internal/table_layout_algorithm_types.rs` | 1 | a_table:table_layout_algorithm_types_interface.rs |
| `core/src/internal/table_node.rs` | 1 | a_table:table_node_interface.rs |
| `core/src/internal/text_fit_scale.rs` | 1 | a_inline:text_fit_scale.rs |
| `core/src/internal/text_item_type.rs` | 1 | a_inline:text_item_type.rs |
| `core/src/internal/text_overflow_post_layout_snapshot.rs` | 1 | fragment:text_overflow_post_layout_snapshot.rs |
| `core/src/internal/transform_utils.rs` | 1 | fragment:transform_utils.rs |
| `core/src/internal/tree_traversal_utils.rs` | 1 | fragment:tree_traversal_utils.rs |
| `core/src/internal/trigger_scoped_name.rs` | 1 | fragment:trigger_scoped_name.rs |
| `core/src/internal/unpositioned_float.rs` | 1 | a_float:unpositioned_float.rs |
| `core/src/internal/unpositioned_list_marker.rs` | 1 | a_block:unpositioned_list_marker_interface.rs |
| `core/src/internal/used_font.rs` | 1 | a_inline:used_font_interface.rs |
| `core/src/internal/variable_length_transform_result.rs` | 1 | a_inline:variable_length_transform_result.rs |
| `core/src/layout_engine.rs` | 18 | invalidation:tree_facts.rs, core:layout_tree_update.rs, core:engine/mod.rs, core:layout_output.rs, fragment:prepaint_input_update.rs, fragment:engine_paint_update.rs, fragment:layout_paint_output.rs, fragment:retained_scroll_refresh.rs, fragment:sticky_layout_commit.rs, invalidation:constraint_equivalence.rs, invalidation:engine_mutation.rs, commit:engine_paint_commit.rs, commit:container_query_sizes.rs, commit:engine_layout_publish.rs, reuse:engine_output_reuse.rs, a_inline:engine_password_echo.rs, a_forms:slider_track_geometry.rs, a_forms:spin_button_geometry.rs |
| `core/src/layout_output.rs` | 21 | core:engine/hit_test/hit_test.rs, a_inline:spatial_navigation_geometry.rs, fragment:fragment_client_rect_queries.rs, fragment:fragment_cursor_children.rs, fragment:layout_metrics.rs, fragment:text_fragment_queries.rs, a_inline:editing_text_runs.rs, a_inline:layout_output_editing_queries.rs, a_inline:visual_caret_movement.rs, a_inline:editing_line_movement.rs, a_inline:editing_text_offset_mapping.rs, a_inline:editing_text_iterator_facts.rs, a_inline:editing_inner_text.rs, a_inline:editing_visible_bounds.rs, fragment:grid_focusgroup_cursor.rs, a_inline:editing_bidi_range.rs, core:engine/form_control/input_control_geometry.rs, a_inline:editing_find_buffer.rs, core:engine/observers/resize_observer_geometry.rs, core:engine/selection/range_geometry.rs, a_inline:editing_apply_style_facts.rs |
| `core/src/lib.rs` | 6 | layout_reuse_package=reuse:cache_lookup.rs, layout_commit_package=commit:client_commit.rs, layout_fragment_object_index=fragment:layout_boundary_object_index.rs, layout_fragment_sticky_offsets=fragment:paint_layer_scrollable_area_sticky_offsets.rs, a_inline:editing_caret.rs, a_inline:editing_caret_rect.rs |
| `core/src/engine/hit_test/hit_test.rs` | 1 | svg_hit_test=core:engine/hit_test/svg_hit_test.rs |
| `core/src/engine/scroll/scroll_source.rs` | 1 | overscroll_source=core:engine/scroll/overscroll_source.rs |
| `fragment/src/fragment_tree.rs` | 1 | fragment:paint_resource_usage.rs |
| `fragment/src/inline_cursor.rs` | 1 | layout_algo_inline_editing_queries=a_inline:inline_cursor_editing_queries.rs |
| `fragment/src/layout_box_visual_overflow.rs` | 1 | fragment:fragment_visual_overflow.rs |
| `fragment/src/motion_path_shape_projection.rs` | 1 | fragment:motion_path_geometry_helpers.rs |
| `fragment/src/paint_layer_scrollable_area.rs` | 1 | native_snap=fragment:snap_coordinator.rs |
| `fragment/src/physical_box_fragment.rs` | 1 | layout_algo_inline_editing_queries=a_inline:physical_box_editing_queries.rs |
| `fragment/src/scrollable_overflow_calculator.rs` | 1 | fragment:scrollable_overflow_calculation.rs |
| `fragment/src/scrollbar_identity.rs` | 1 | input_geometry=fragment:scrollbar_input_geometry.rs |
| `invalidation/src/input.rs` | 9 | core:internal/boundary/clip_path_projection.rs, layout_algo_grid_style_projection=a_grid:grid_style_projection.rs, core:internal/boundary/mask_image_projection.rs, a_float:shape_style_projection.rs, fragment:style_effect_projection.rs, invalidation:style_font_resolution.rs, invalidation:trigger_scope_projection.rs, invalidation:font_resolver_scope.rs, invalidation:input_projection.rs |

## Appendix B: proposed file destinations

These rules produced the §4.2 decomposition:

- fragment data closure → `layout_fragment`;
- every other fragment file → base;
- core, assembly, sizing and fragmentation → base;
- any other file citing `layoutng/internal` → base;
- invalidation/reuse/commit files with inherent impls on base types → base.

§4.3 lists the extra base moves recommended on top (autoscroll,
`layout_replaced_interface`, `block_child_iterator`, `layout_text_fragment`,
`offset_mapping`, `custom_layout_api`, `svg_metadata_validation`,
MathML `MathScriptType`, …). Paths are relative to each package's `src/`.

- **layout_algo_block → layout_base** (10): `block_node.rs`, `block_node_traversal.rs`, `column_spanner_admission.rs`, `layout_block.rs`, `layout_block_flow.rs`, `layout_block_traversal.rs`, `layout_object_containers.rs`, `layout_view_block_queries.rs`, `layout_view_root_layout.rs`, `unpositioned_list_marker_interface.rs`
- **layout_algo_block → layout_fragment** (2): `column_spanner_path.rs`, `line_clamp_data.rs`
- **layout_algo_custom → layout_fragment** (2): `constraint_space_custom_data.rs`, `custom_layout_payload.rs`
- **layout_algo_flex → layout_fragment** (1): `devtools_flex_info.rs`
- **layout_algo_float → layout_base** (7): `positioned_float.rs`, `shape_image_environment.rs`, `shape_image_services_interface.rs`, `shape_interval.rs`, `shape_outside_info.rs`, `shape_style_projection.rs`, `unpositioned_float.rs`
- **layout_algo_float → layout_fragment** (5): `exclusion_area.rs`, `exclusion_space.rs`, `layout_opportunity.rs`, `shape.rs`, `shape_exclusions.rs`
- **layout_algo_forms → layout_base** (1): `form_control_sizing_service.rs`
- **layout_algo_forms → layout_fragment** (2): `form_control_types.rs`, `form_node_metadata_interface.rs`
- **layout_algo_fragmentation → layout_base** (3): `constraint_space_fragmentation.rs`, `fragmentation_utils.rs`, `lib.rs`
- **layout_algo_fragmentation → layout_fragment** (2): `break_appeal.rs`, `early_break.rs`
- **layout_algo_grid → layout_base** (3): `grid_item_data.rs`, `grid_lanes_item_group.rs`, `grid_style_projection.rs`
- **layout_algo_grid → layout_fragment** (3): `grid_layout_data.rs`, `grid_subtree.rs`, `grid_track_collection_types.rs`
- **layout_algo_inline → layout_base** (16): `constraint_space_block_in_inline.rs`, `hyphen_result_interface.rs`, `initial_letter_style.rs`, `inline_item_result_interface.rs`, `inline_item_span.rs`, `inline_node_core.rs`, `inline_node_data.rs`, `inline_node_interface.rs`, `layout_inline.rs`, `layout_object_editing_queries.rs`, `layout_text_combine_interface.rs`, `layout_text_core.rs`, `layout_text_interface.rs`, `layout_view_variable_length_transform_index.rs`, `text_input_environment.rs`, `variable_length_transform_result.rs`
- **layout_algo_inline → layout_fragment** (6): `inline_item_interface.rs`, `inline_item_segment_interface.rs`, `inline_item_text_index.rs`, `text_fit_scale.rs`, `text_item_type.rs`, `used_font_interface.rs`
- **layout_algo_list → layout_base** (3): `layout_object_list_marker_queries.rs`, `layout_view_list_index.rs`, `list_marker_lookup.rs`
- **layout_algo_multicol → layout_base** (1): `multicol_break_token_data.rs`
- **layout_algo_out_of_flow → layout_base** (8): `anchor_lookup.rs`, `anchor_map_operations.rs`, `anchor_map_services.rs`, `anchor_position_scroll_data.rs`, `anchor_scroll_services_interface.rs`, `inline_containing_block_utils_interface.rs`, `out_of_flow_data.rs`, `successful_position_fallback.rs`
- **layout_algo_out_of_flow → layout_fragment** (3): `anchor_map.rs`, `non_overflowing_scroll_range.rs`, `oof_positioned_node.rs`
- **layout_algo_sizing → layout_base** (6): `baseline_utils.rs`, `layout_box_scrollbars.rs`, `length_utils.rs`, `lib.rs`, `min_max_constraint_space_builder.rs`, `space_utils.rs`
- **layout_algo_svg → layout_base** (8): `layout_object_svg_dispatch.rs`, `resolved_text_layout_attributes_iterator.rs`, `svg_character_data.rs`, `svg_inline_node_data.rs`, `svg_layout_info.rs`, `svg_length_adjust_type.rs`, `svg_text_attributes_request.rs`, `svg_text_descendant_index.rs`
- **layout_algo_table → layout_base** (2): `table_layout_algorithm_types_interface.rs`, `table_node_interface.rs`
- **layout_algo_table → layout_fragment** (3): `table_borders_types.rs`, `table_column_location.rs`, `table_constraint_space_data.rs`
- **layout_commit → layout_base** (3): `layout_box_post_layout_commit.rs`, `layout_box_result_commit.rs`, `layout_view_commit.rs`
- **layout_core → layout_fragment** (12): `internal/anchor_scope.rs`, `internal/constraint_space.rs`, `internal/disable_layout_side_effects_scope.rs`, `internal/exclusions/line_layout_opportunity.rs`, `internal/hit_test_phase.rs`, `internal/layout_theme.rs`, `internal/map_coordinates_flags.rs`, `internal/selection_state.rs`, `internal/split_axis_item.rs`, `internal/style_variant.rs`, `internal/text_decoration_offset.rs`, `internal/text_offset_range.rs`
- **layout_fragment → layout_base** (111): `background_transfer.rs`, `box_fragment_builder.rs`, `caret_rect.rs`, `clip_path_operation_projection.rs`, `collapsed_table_paint_export.rs`, `custom_scrollbar.rs`, `empty_element_caret.rs`, `engine_paint_update.rs`, `fragment_builder.rs`, `fragment_client_rect_queries.rs`, `fragment_cursor_children.rs`, `fragment_export_support.rs`, `fragment_geometry_export.rs`, `fragment_items_builder.rs`, `fragment_paint_data_init.rs`, `fragment_property_export.rs`, `fragment_visual_overflow.rs`, `frame_view_pending_property_updates.rs`, `global_paint_export.rs`, `grid_focusgroup_cursor.rs`, `hit_test_geometry.rs`, `ink_overflow_export.rs`, `inline_client_geometry.rs`, `inline_fragment_paint_policy.rs`, `inline_items_data.rs`, `inline_quad_geometry.rs`, `layout_boundary_object_index.rs`, `layout_box_background.rs`, `layout_box_caret.rs`, `layout_box_fragment_access.rs`, `layout_box_fragment_geometry.rs`, `layout_box_location_update.rs`, `layout_box_model_outline.rs`, `layout_box_model_scrollable_area.rs`, `layout_box_resource_priority.rs`, `layout_box_visual_overflow.rs`, `layout_box_visual_rect.rs`, `layout_custom_scrollbar_part.rs`, `layout_inline_visual_rect.rs`, `layout_metrics.rs`, `layout_object_bounding_geometry.rs`, `layout_object_coordinate_conversion.rs`, `layout_object_effective_style.rs`, `layout_object_fragment_queries.rs`, `layout_object_gap_queries.rs`, `layout_object_geometry_queries.rs`, `layout_object_inline_fragment_state.rs`, `layout_object_layer_queries.rs`, `layout_object_overflow_recalc.rs`, `layout_object_paint_dispatch.rs`, `layout_object_paint_state.rs`, `layout_object_scroll_container_queries.rs`, `layout_object_selection_queries.rs`, `layout_paint_output.rs`, `layout_result_overflow.rs`, `layout_result_publish.rs`, `layout_sticky_constraints.rs`, `layout_view_fragment_queries.rs`, `layout_view_geometry.rs`, `layout_view_paint_policy.rs`, `layout_view_visual_rect.rs`, `lib.rs`, `logical_box_fragment.rs`, `logical_fragment.rs`, `logical_fragment_link.rs`, `logical_line_container.rs`, `logical_line_item.rs`, `mask_image_style_projection.rs`, `motion_path_geometry_helpers.rs`, `motion_path_shape_projection.rs`, `nine_piece_image_export.rs`, `outline_geometry.rs`, `overflow_model.rs`, `pagination_fragment_utils.rs`, `paint_cache_refresh.rs`, `paint_input.rs`, `paint_input_update.rs`, `paint_layer.rs`, `paint_layer_scrollable_area.rs`, `paint_layer_scrollable_area_scope.rs`, `paint_layer_scrollable_area_sticky_offsets.rs`, `paint_layer_transform.rs`, `paint_resource_usage.rs`, `paint_style_export.rs`, `physical_fragment_list.rs`, `prepaint_input_update.rs`, `retained_paint_refresh.rs`, `retained_scroll_refresh.rs`, `scroll_anchor.rs`, `scroll_input_update.rs`, `scroll_paint_property_index.rs`, `scroll_paint_property_refresh.rs`, `scrollable_overflow_calculation.rs`, `scrollable_overflow_recalc.rs`, `scrollbar_identity.rs`, `scrollbar_input_geometry.rs`, `scrollbar_paint_export.rs`, `scrollbar_part.rs`, `scrollbar_thumb_geometry.rs`, `snap_coordinator.rs`, `specialized_paint_export.rs`, `sticky_layout_commit.rs`, `sticky_paint_export.rs`, `style_effect_projection.rs`, `text_fragment_queries.rs`, `text_overflow_post_layout_snapshot.rs`, `text_paint_export.rs`, `trace_bindings.rs`, `transform_utils.rs`, `tree_traversal_utils.rs`, `visual_rect_mapping.rs`
- **layout_invalidation → layout_base** (48): `computed_style_equivalence.rs`, `content_change_type.rs`, `element_metadata_input.rs`, `font_resolver_scope.rs`, `geometry_invalidation.rs`, `input.rs`, `input_projection.rs`, `layout_block_child_mutation.rs`, `layout_block_flow_child_mutation.rs`, `layout_block_flow_child_normalization.rs`, `layout_block_flow_multicol_update.rs`, `layout_block_flow_style_change.rs`, `layout_block_style_change.rs`, `layout_box_fragment_retirement.rs`, `layout_box_model_layer_invalidation.rs`, `layout_box_model_lifecycle.rs`, `layout_box_model_tree_mutation.rs`, `layout_box_style_change.rs`, `layout_box_tree_lifecycle.rs`, `layout_invalidation_reason.rs`, `layout_invalidation_state.rs`, `layout_node_style.rs`, `layout_object_child_list.rs`, `layout_object_child_mutation.rs`, `layout_object_dirty_state.rs`, `layout_object_display_lock.rs`, `layout_object_factory.rs`, `layout_object_hot_invalidation.rs`, `layout_object_image_observers.rs`, `layout_object_layer_mutation.rs`, `layout_object_paint_invalidation.rs`, `layout_object_prepaint_invalidation.rs`, `layout_object_state_propagation.rs`, `layout_object_style_services.rs`, `layout_object_style_update.rs`, `layout_object_tree.rs`, `layout_object_tree_lifecycle.rs`, `layout_tree_invalidation.rs`, `layout_view_child_mutation.rs`, `layout_view_destruction.rs`, `layout_view_invalidation.rs`, `managed_object_update.rs`, `metadata_append.rs`, `node_metadata_finalize.rs`, `node_metadata_incremental_relations.rs`, `node_metadata_tree.rs`, `prepare_tree.rs`, `transform_invalidation.rs`
- **layout_invalidation → layout_fragment** (1): `naming_scope.rs`
- **layout_reuse → layout_base** (10): `font_resolver.rs`, `layout_box_result_cache.rs`, `layout_cache_status.rs`, `layout_environment_cache.rs`, `layout_object_font_validity.rs`, `layout_utils.rs`, `managed_input_index.rs`, `managed_object_compatibility.rs`, `measure_cache.rs`, `min_max_sizes_cache.rs`

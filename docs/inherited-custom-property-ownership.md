# Share inherited custom-property backing in the production resolver

The production StyleResolver path passed inherited `&CustomProperties` through
its existing cascade boundary, but ProductionStyleCascadeBackend copied both
HashMaps and every String key, then retained a new map for each descendant.
The legacy DocumentStyleEngine::Cascade had a parent-sharing fast path that this
production path did not reach. A nonempty inherited variable map also prevents
the MatchedPropertiesCache hit, so that cache could not remove these copies.

The repair keeps the existing typed side table and collaborator interfaces.
CustomProperties owns immutable `Rc<HashMap>` backing for untyped data and
registered typed values. Cascade construction and ordinary substitution clone
only handles. Changed values detach with `Rc::make_mut`; equal values, empty
merges, absent removals, unchanged registration retention and already-initial
non-inherited values preserve their backing. Local overrides and old snapshots
remain independent. Diagnostics and shorthand scratch remain cascade-local.

Chromium source pin: `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.

- [StyleVariables sharing on copy/assignment](https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/style/style_variables.h#L371-L395)
- [Equality before copy-on-write](https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/style/style_variables.h#L97-L165)
- [Inherited variable owner](https://github.com/chromium/chromium/blob/6c1d401fcca5e1b0030563a90c2f2bba168e0c15/third_party/blink/renderer/core/style/style_inherited_variables.h#L20-L63)

This restores immutable sharing/COW semantics in the established production
representation; it does not claim Chromium's per-trie-node copy complexity.
Rechrom's existing stricter feature/animation-taint/attribute-taint equality is
preserved rather than weakened for storage reuse. Registered initial values are
still computed in the correct font environment before comparison.

The regression exercises the actual DocumentStyleEngine production resolver,
checks no compatibility fallback, and fails before on distinct inherited key
storage. It covers local overrides, descendant inheritance, root mutation,
retained old snapshots, registered inherit:false reset, explicit inherit, and
both shared map backings. Existing cyclic, registered font-relative and animation
controls validate unchanged cascade behavior. The semantic owner census dedupes
actual Rc map/token backing; bucket payload estimates are not allocator sizes.

No global deduplication cache, cross-document owner, cache threshold, stylesheet
policy, ancestor/scope invalidation, or page-specific branch is introduced.

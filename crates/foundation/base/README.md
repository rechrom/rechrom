# Foundation primitives

Imported unchanged from the supplied `layoutng/rust/foundation` source crate,
including its independent tests and captured reference data. This internal
crate owns the translated geometry, writing-mode, and text primitives.

The parent `layoutng_foundation` crate reexports these primitives and adds the
interfaces consumed by the rest of Rechrom. Dependencies flow from the parent
to this crate only; this crate has no dependencies on the parent or other
Rechrom engines. The original package/lib name `foundation` is preserved for
its independent tests; the parent imports it as `foundation_base`.

## Import provenance

All 75 original files were verified byte-for-byte against the supplied
`layoutng/rust/foundation` directory. `SOURCE-MANIFEST.sha256` records their
relative paths and SHA-256 hashes; this README and the manifest are local
documentation, not part of that supplied source.

The supplied directory contains no LICENSE, copyright attribution file,
repository metadata or pinned upstream revision. Individual source comments
identify translated C++ paths, but they do not establish the provenance or
licensing of the Rust translation. Its precise revision, translation authorship
and applicable license therefore remain unverified. The Chromium license
preserved at `third_party/LICENSE.chromium` must not be assumed to establish
those missing facts or to relicense this import. Obtain the original source
package's attribution and license information before public distribution.

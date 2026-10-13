# Exact paint-plane projection for non-composited perspective clips

Pinned Chromium: `6c1d401fcca5e1b0030563a90c2f2bba168e0c15`.

## Defect and production route

The Bilibili homepage exposed a fatal `unsupported-property-clip` while lowering
an ordinary overflow clip under `perspective:1000px`. An isolated offline
control succeeds with `perspective:none` and fails with perspective enabled.
The failure does not need site JavaScript, images, networking, or a 3D child.

`LayerTileEngine` lowers PaintArtifact records before grouping pending layers.
`raster_properties` treated every non-2D 4x4 matrix as a retained anchor, and
`project_record_rect` then required the clip's entire ancestry to be 2D affine.
This rejected CSS perspective even when every descendant remains in z=0.
Simply allowing a bounding box there is insufficient: canonical recording also
needs consistent clip transforms, plain-clip pixel phase, and ancestor identity.

## Chromium behavior and bounded translation

- `PropertyTreeState::CanUpcastWith` (103–197) uses actual transform-compositing
  boundaries and compatible resolved backface visibility, not a generic matrix
  type test
- `PaintPropertyTreeBuilder::UpdatePerspective` (3355–3383) does not give a
  perspective-only node a direct compositing reason
- `CompositingReasonFinder` (439–448, 542–567) distinguishes perspective with 3D
  descendants from a perspective-only transform
- `GeometryMapper::SourceToDestinationProjectionInternal` (70–194) projects
  between paint planes; each clip uses its own local transform (472–497)
- `PaintChunksToCcLayer::ApplyTransform` (293–315) emits the exact projection,
  while `StartClip` (572–608) records actual AA rect/rrect/path clip operations

The internal `record_plane_transform` admits a conservative, composition-closed
subset: finite invertible new 4x4 matrices with m2=m6=m14=0, m3=m7=0 and m10=m15=1.
The z=0 plane stays z=0, its depth normal is preserved, and homogeneous w is exactly 1. Its six xy coefficients
are therefore already the exact affine projection, with no divide, approximation
or f32 arithmetic reordering. CSS perspective about any xy origin is included.
Existing 2D affine behavior, including singular affine maps, is unchanged.

This is a deliberately bounded receiver capability, not Chromium's full singular
projection behavior. Newly admitted non-affine singular matrices, nonunit w,
rotateY/tilted or depth-translated planes, camera crossings and nonfinite inputs
remain unsupported. A nonplanar transform remains anchored and the current
raster receiver still reports its explicit unsupported condition. No invisible
replacement or empty-success frame is generated.

The exact plane value is used for upcast eligibility, property-clip mapping,
axis bounds and translation classification. Identity-plane perspective wrappers
can separate two equivalent clip spaces in the property tree. They are treated
as the same recording plane only along the actual omitted ancestor chain and
only when every intervening projected transform is identity. Unrelated branches
and nonidentity intermediates cannot use this proof.

Direct compositing reasons, composited scrolls, 3D contexts, resolved backface
changes, effect output clips and the existing unsupported clip-shape/filter
boundaries remain barriers. Native property nodes and all original 4x4 matrices
stay unchanged. The raw flat-record affine validator remains strict.

## Ownership and tests

Only LayerTile's pure geometry/record planning changes. Its helper is crate-local;
no cross-crate API is added. Raster still executes true ClipOps and Renderer still
owns retained 4x4 composition. Scheduling bounding rectangles never replace the
actual rect, rounded or path clip.

`browser/tests/perspective_clip_raster.rs` has fail-before/pass-after production
fixtures for nested perspective plus overflow/rounded clips, fractional
perspective origins, translation and rotation. Perspective and the planar
control must render nonempty content and match every pixel. LayerTile unit tests
cover exact coefficients, native identity/matrix preservation, direct/scroll/
context/backface barriers, unrelated transforms and explicit unsupported cases.
Existing Wikipedia inline/fetched SVG edge tests cover the earlier clip route,
DPR1/DPR2 and positioned colored backgrounds without tolerance.

Official source URLs and SHA-256 hashes, intermediate failures, build/test logs,
Release captures and current live-site limits are retained in the accompanying
Bilibili evidence report.

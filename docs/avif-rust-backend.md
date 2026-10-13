# AVIF: bounded pure-Rust SDR production decoding

## Ownership and source mapping

The existing injected `ImageDecoder` boundary now dispatches AVIF by the pinned
CrabbyAvif compatible-file-type signature, inspecting at most 144 bytes as Blink
`AVIFImageDecoder::MatchesAVIFSignature` does. MIME and URL suffix are not codec
selectors. `image_decoder/avif-backend` is enabled by default; `--no-default-features`
retains an explicit codec-disabled build. No DOM, Style, Layout, Paint, resource,
or browser assembly algorithm changes are part of this integration.

Encoded bytes enter `image_decoder`; only immutable owned straight RGBA8,
storage width/height, and existing `ImageOrientation` leave it. Raster continues
to own premultiplication. No codec internals or profile objects cross that API.

CrabbyAvif `8f2b74b4d8be956395f527020c2190762e086f98`, pinned by Chromium
`6c1d401fcca5e1b0030563a90c2f2bba168e0c15`, owns container parsing,
derived-image/grid assembly, and alpha. Its narrow codec adapter uses pure-Rust
rav1d 1.1.0 with assembly/native features disabled and two official robustness
backports. The port is not an exact translation of Chromium's newer native
dav1d revision; see `vendor/{crabby-avif,rav1d}/RECHROM.md` and source manifests.

The adapter follows Blink's complete-data settings: two threads, ignore
EXIF/XMP, exclude only ClapValid and PixiRequired strictness, no incremental or
progressive feed, explicit still-primary selection. Container dimensions, depth,
chroma layout and alpha presence must agree with the decoded picture. The exact
12-entry irot/imir table is retained, without rotating storage pixels or swapping
storage dimensions. Host budgets are 64 MiB encoded, 16 Mi pixels and maximum
dimension 32768; these are admission policy, not format or total-memory limits.

## Admitted matrix

- 8-bit still primary images, including ordinary supported derived/grid images
- YUV400/420/422/444; full or limited range; matrix 709/601/470bg/unspecified
- Identity matrix: full-range 444 (exact channel permutation), or 400
- Primaries: sRGB/BT709 or unspecified, as normalized by pinned Crabby
- Transfer: sRGB, unspecified, BT709, BT601, BT2020_10bit, BT2020_12bit
- Straight source alpha or opaque; existing orientation metadata

Those transfer enum aliases are intentionally rendered as sRGB by pinned
Chromium `ui/gfx/color_space.cc` 995–1011 after `VideoColorSpace` mapping. This
is Chromium's rendering policy, not a claim their standard curves are identical.
Unspecified primaries/transfer map to BT709/sRGB; unspecified reconstruction
matrix uses BT601. The original parser's normalized CICP values are retained,
including its unknown-value normalization; no parallel BMFF parser is added.

`io::ErrorKind::Unsupported` remains explicit for sequences/animation timelines,
progressive/layered stills, non-8-bit output, ICC, other primaries/transfers/matrices,
clean aperture, non-square/zero aspect ratio, HDR/CLLI/gainmap metadata,
premultiplied source alpha and identity with subsampled or limited-range color.
These conservative crop/aspect gates are not represented as Blink's behavior:
Blink ignores pasp and ignores invalid/nonzero-origin clap; admitted origin-zero
clap would require its validated YUV view *before* chroma reconstruction.

## Exact RGB reconstruction

`avif_rgb8.rs` is a private safe-Rust translation of libyuv
`a0fa6329391922faca64955c1a7bf635ab87d833`, Chromium's pinned revision:

- `row_common.cc`: I601/JPEG/H709/F709 integer constants, `YuvPixel`, `YPixel`
- `scale_common.cc`: `ScaleRowUp2_Linear_C`, `ScaleRowUp2_Bilinear_C`
- `scale_any.cc`: SUH2LANY/SU2BLANY endpoint rules
- `convert_argb.cc`: `I420ToARGBMatrixBilinear`, 422 linear and 444/400 dispatch
- Pinned Crabby `reformat/libyuv.rs`: matrix/range and alpha dispatch

420 rounds the 2D 9:3:3:1 filter once, not two successive rounded 1D passes.
Both horizontal endpoints are explicitly replicated, including odd width;
vertical top/bottom and odd-height behavior follow the libyuv loop. No generic
resampler, floating-point matrix or compensating tolerance substitutes for it.
Like pinned libyuv, reconstruction does not branch on chroma sample position;
this is source parity, not a broader standards claim about arbitrary siting.

Identity 444/full uses the same direct V/Y/U-to-R/G/B mapping as pinned Crabby's
fallback, because libyuv deliberately declines this case. The official lossless
403×302 AVIF/PNG pair validates every output channel exactly, even though its ICC
metadata excludes that particular encoded file from production admission.

## Independent oracles and tests

`tests/fixtures/avif-source/reference/` contains the deterministic input generator
and reference runner. The runner links an evidence-only build of the exact
pinned official libyuv and independently checks scalar-versus-native-SIMD equality
before recording expected pixels. It is never linked into production or tests.

The checked-in corpus has 832 cases / 147,968 pixels across all admitted YUV
layouts, both ranges, all four nonidentity matrix values, opaque/alpha, 13 odd/even
and one-pixel extent pairs, extrema and pseudorandom inputs. Each stored pixel is
checked with no tolerance by the Rust test. `alpha-libyuv.bin` contains official
alpha fixture planes and the independent result; its PNG is a browser paint
control. The retained real Bilibili banner has a separate evidence-only native
oracle and an opt-in all-pixel comparison. YUV planes are produced by the Rust
AV1 backend, so these tests prove conversion parity, not independent AV1 decoder
conformance. The lossless official PNG is a separate AV1 reconstruction oracle.

`browser/tests/avif_raster.rs` checks actual resource fetching, invocation of the
production decoder, resource settlement, image arrival in paint resources, and
nonblank painting against the independent PNG at DPR 1/1.25/2 over two backgrounds.
Existing JPEG/PNG, malformed/budget, orientation, grid, alpha/ownership, raw-depth,
thread/seek and SVG-mask/edge controls remain separate.

Representative commands:

```sh
cargo test --release --locked -p image_decoder --lib
cargo test --release --locked -p image_decoder --no-default-features --lib
cargo test --release --locked -p browser --test avif_raster --test svg_edge_raster --test local_svg_masks
cargo build --release --locked -p rechrom_app
```

The exact upstream rav1d crash reproduction and source/native-code audit are
retained in preceding integration evidence. The finite tests do not establish
exhaustive AV1/AVIF safety, fuzz/sanitizer coverage, whole-format completeness,
whole-workspace green, or cross-browser whole-page pixel parity.

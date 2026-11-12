# Offscreen video GPU golden

`video_gpu_golden.mjs` captures Chromium's ordinary GPU video foreign layer at
960x540, DPR 1 and forced sRGB. It also decodes the same first frame through
AVFoundation and generates small, temporary stage candidates. No browser window
is opened and no captured pixels are retained unless requested.

Run the baseline:

```sh
node tools/video_gpu_golden.mjs
```

Compare a Rechrom offscreen screenshot and make pixel identity mandatory:

```sh
node tools/video_gpu_golden.mjs \
  --candidate work/rechrom-video.png \
  --require-exact
```

The report records exactness, changed pixels, RGBA MAE and channel maxima. It
also rejects a software/SwiftShader Chrome capture. `--keep-artifacts` is only
for diagnosis; the default run writes one JSON report and deletes all PNG/RGBA
scratch data.

The controlled candidates answer separate questions:

- `centered_sampling_identity`: normalized linear sampling of the Y and UV
  planes must reproduce CoreImage's unmanaged output exactly.
- `bt709_apple_to_srgb`: isolates macOS gamma 1.961 to sRGB from YUV sampling.
- `chroma_nearest_to_centered`: measures the cost of co-sited nearest UV
  sampling while keeping matrix, transfer and quantization fixed.
- `fp16_stage_rounding`: deliberately rounds every sampled and arithmetic
  stage to half precision; it detects an accidental `half` shader path.
- `unorm_floor_instead_of_round`: distinguishes floor/truncation from the
  required round-to-nearest UNORM result.

The source constants are deliberately copied from Chromium's
`third_party/skia/src/core/SkYUVMath.cpp` high-precision Rec.709 limited matrix.
The gamma is Chromium's `SkNamedTransferFnExt::kRec709Apple` from
`skia/ext/skcolorspace_trfn.h`. This tool is an oracle probe, not a second
production renderer.

The report also captures the paused frame through both `new VideoFrame(video)`
and `MediaStreamTrackProcessor`. It copies each frame in its native format and
compares tightly packed Y/UV bytes with the same AVFoundation decode. The
separate `copyTo({format:'RGBA'})` comparison deliberately identifies the
conversion boundary: Chrome's RGBA copy applies the Rec.709 limited YUV matrix,
but it is not the color-managed display result. A native-plane mismatch points
to decode; exact native planes plus an exact unmanaged RGBA copy confines any
remaining screenshot difference to the GPU/display color-management path.

## Resize and video-damage acceptance

Use the same paused fixture first, then repeat it while advancing frames. Drive
resize through the normal headless viewport entry point and record these event
families in one monotonic trace:

1. Raw and accepted size: winit `Resized` or CDP
   `Emulation.setDeviceMetricsOverride`, accepted viewport revision, and
   BeginFrame sequence.
2. Document work: style/layout/prepaint/paint revisions and whether the video
   paint artifact or only its transform/clip changed.
3. Surface identity: LocalSurfaceId allocation, `Surface::CommitFrame`,
   `Surface::ActivateFrame`, and `Display::SetSurfaceId`.
4. Damage: video child/foreign-layer old and new bounds, aggregated root damage,
   renderer `damage_pixels`, and `EarlyOut_NoDamage`.
5. Video resources: decoded frame revision, NV12 upload count, texture identity,
   allocation/reuse, CPU raster, readback, and synchronous wait count.
6. Presentation: `Display::Resize`, `Display::DrawAndSwap`, SwapBuffers,
   `OffscreenFramePublished` dimensions, request-to-matching-frame latency, and
   stale-size frames after the final resize.

Required invariants after warm-up:

- Duplicate same-size notifications allocate no LocalSurfaceId, perform no
  layout/paint/upload, and produce either no frame or `EarlyOut_NoDamage`.
- A burst applies the latest size on BeginFrame; work is bounded by accepted
  sizes rather than raw notifications, and no old-size surface backlog remains
  after settling.
- A paused frame keeps the same decoded revision and texture identity. Resize
  causes zero NV12 re-decodes/uploads, CPU video rasters, readbacks, or waits.
- Root full damage is allowed once for each real output-buffer size change.
  It must not be interpreted as permission to re-raster or re-upload the video
  resource. The child video contribution is limited to the union of its old and
  new visible bounds.
- During playback, uploads track distinct decoded frame revisions, not resize
  samples. A resize must not duplicate one frame's upload.
- The settled offscreen frame has the requested size and remains pixel-equal to
  the normal GPU golden at the corresponding final viewport.

Chromium reference trace names include `Display::Resize`,
`Display::SetSurfaceId`, `Surface::CommitFrame`, `Surface::ActivateFrame`,
`EarlyOut_NoDamage`, `Display::DrawAndSwap`, and `SkiaRenderer::SwapBuffers`.
Rechrom should correlate these with `Compositor.Frame.Submit`,
`OffscreenFramePublished`, `PresentReturn`, and renderer `damage_pixels` rather
than infer performance from screenshot completion alone.

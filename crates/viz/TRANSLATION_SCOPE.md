# Viz translation scope

The public input remains immutable `CompositorFrame` submission plus surface,
BeginFrame/deadline and swap facts. `VizOutput` is the only Viz-to-Renderer
interface. It owns the source-shaped `AggregatedFrame`, stable display-resource
leases, root presentation data and the opaque draw receipt.

Included is the responsibility closure for the CPU browser path:

- Viz surface identity and ranges;
- Surface, SurfaceAllocationGroup, SurfaceManager, references and activation
  deadlines;
- Aggregated frame/pass values and frame resolution;
- display damage tracking and surface aggregation;
- frame deadline selection and display scheduling.

Excluded are DirectRenderer/SoftwareRenderer (owned by the Renderer package),
GPU/Skia output surfaces, overlays, copy output, video
capture, transitions, IPC, tracing backends, platform presentation and OS task
runners. These are capabilities or downstream execution, not Viz's retained
surface-management and aggregation state.

The existing `VizEngine` API is an adapter over the translated owners. It does
not duplicate their aggregation, damage or scheduling decisions. Renderer sees
only `VizOutput`; Page passes that value through and returns it whole to
`VizEngine::DidDraw` after successful rendering.

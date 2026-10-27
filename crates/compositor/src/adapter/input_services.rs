//! Typed service dependencies of the translated compositor/input interfaces.
//! Rechrom-owned adapters, excluded from Chromium translation coverage.

/// Opaque 128-bit Viz identity. Empty tags explicitly mean "no tag"; the host
/// supplies nonempty tokens allocated by its Viz owner rather than cc creating
/// a second tag allocator.
pub use compositor_input::offset_tag::OffsetTag;

/// Source-owned classes/services not yet translated in this slice. Associated
/// types preserve object identity, borrowing and ownership without fabricated
/// empty implementations or numeric substitutes. Integration supplies native
/// implementations; no behavior is stubbed by this type-level boundary.
pub trait InputHostTypes: Sized {
    type InputHandler;
    type CompositorCommitData;
    type MutatorHost: ?Sized;
    type ScrollTree: ?Sized;
    type ScrollbarOrientation;
    type ScrollNode: ?Sized;
    type LayerTreeSettings: ?Sized;
    type LayerTreeHostImpl: ?Sized;
    type LatencyInfo;
    type LatencyInfoSwapPromiseMonitor: ?Sized;
    type EventMetricsScopedMonitor: ?Sized;
    type EventMetricsDoneCallback;
    type FrameSequenceTrackerType;
    type SmoothEffectDrivingThread;
    type BeginMainFrameReason;
    fn OtherBeginMainFrameReason() -> Self::BeginMainFrameReason;
}

/// Floating-point base::Seconds conversion, retaining signed microseconds and
/// saturated float-to-int conversion at the typed time boundary.
pub fn Seconds(seconds: f64) -> crate::adapter::time::TimeDelta {
    crate::adapter::time::TimeDelta::Microseconds((seconds * 1_000_000.0) as i64)
}

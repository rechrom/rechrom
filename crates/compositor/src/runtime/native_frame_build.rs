//! Native frame-build diagnostics, geometry values and instrumentation. This
//! facade never owns a Host, frame, tree or the algorithms selecting render passes.
use super::engine_factory::NativeFactoryCapabilities;
use crate::{
    adapter::{frame_data::FrameDataTraceServices, geometry::Rect, layer_tree_host_impl::*},
    cc::{scheduler::draw_result::DrawResult, trees::frame_data::FrameData},
    NativeCompositorTypes,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
// Source dependency: base/debug/crash_logging.h:66-97; .cc:48-88. This is an
// actual native crash annotation owner. A scope sets the key, and Drop clears
// the same key; it never captures compositor state or pretends to be a lock.
#[derive(Default)]
struct NativeCrashKeys {
    values: Mutex<BTreeMap<String, Vec<u8>>>,
}
struct NativeCrashKeyScope {
    owner: Arc<NativeCrashKeys>,
    name: String,
}
impl HostCrashKey for NativeCrashKeyScope {}
impl Drop for NativeCrashKeyScope {
    fn drop(&mut self) {
        self.owner
            .values
            .lock()
            .expect("crash annotations")
            .remove(&self.name);
    }
}
pub(crate) struct NativeFrameBuildServices {
    native: Arc<NativeFactoryCapabilities>,
    crash: Arc<NativeCrashKeys>,
}
impl NativeFrameBuildServices {
    pub(crate) fn new(native: Arc<NativeFactoryCapabilities>) -> Self {
        Self {
            native,
            crash: Arc::new(NativeCrashKeys::default()),
        }
    }
}
impl FrameDataTraceServices for NativeFrameBuildServices {
    type TracedValueJSON = HostFrameTraceJson<super::host::TraceValue>;
    fn GetClientNameForMetrics(&self) -> Option<&str> {
        self.native.diagnostics.client_name_for_metrics.as_deref()
    }
    fn TraceCategoryEnabled(&self, category: &str) -> bool {
        self.native.frame_trace_categories.contains(category)
    }
    fn NewTracedValueJSON(&self) -> Self::TracedValueJSON {
        HostFrameTraceJson(super::native_host_picture::boxed_trace_value())
    }
}
impl HostImplBServices<NativeCompositorTypes> for NativeFrameBuildServices {
    fn TraceEmptyDamageRect(&self) {
        self.native.events.Emit("cc", "EmptyDamageRect".into())
    }
    fn TraceCalculateRenderPassesBegin(&self, size: u64, high: bool) {
        self.native.events.Emit(
            "cc",
            format!("CalculateRenderPasses begin size={size} requires_high_res={high}"),
        )
    }
    fn TraceCalculateRenderPassesEnd(&self, result: DrawResult, missing: Option<i32>) {
        self.native.events.Emit(
            "cc",
            format!("CalculateRenderPasses end result={result:?} missing_tiles={missing:?}"),
        )
    }
    fn ShouldRecordSubsampledMetric(&self, probability: f64) -> bool {
        assert!((0.0..=1.0).contains(&probability));
        let mut bytes = [0u8; 8];
        getrandom::fill(&mut bytes).expect("native sampling entropy");
        let uniform = (u64::from_ne_bytes(bytes) >> 11) as f64 / ((1u64 << 53) as f64);
        uniform < probability
    }
    fn CrashKey(&mut self, name: &str, value: HostCrashKeyValue) -> Box<dyn HostCrashKey> {
        let (value, size) = match value {
            HostCrashKeyValue::String64(v) => (v, 64),
            HostCrashKeyValue::String32(v) => (v, 32),
            HostCrashKeyValue::String256(v) => (v, 256),
            HostCrashKeyValue::Boolean(v) => (v.to_string(), 32),
            HostCrashKeyValue::Number(v) => (v.to_string(), 32),
        };
        let mut bytes = value.into_bytes();
        bytes.truncate(size);
        self.crash
            .values
            .lock()
            .expect("crash annotations")
            .insert(name.into(), bytes);
        Box::new(NativeCrashKeyScope {
            owner: self.crash.clone(),
            name: name.into(),
        })
    }
    fn DumpWillBeCheck(&self, condition: bool, message: &str) {
        if !condition {
            let keys = self.crash.values.lock().expect("crash annotations");
            self.native.events.Emit(
                "compositor.check",
                format!("{message}; crash_keys={:?}", *keys),
            )
        }
    }
    fn AddVisibleContentArea(&mut self, stats: HostImplRenderingStatsId, area: i64) {
        self.native
            .rendering_stats
            .AddVisibleContentArea(stats, area)
    }
    fn AddApproximatedVisibleContentArea(&mut self, stats: HostImplRenderingStatsId, area: i64) {
        self.native
            .rendering_stats
            .AddApproximatedVisibleContentArea(stats, area)
    }
    fn TickWorkletAnimations(&mut self, host: &mut super::input::MutatorHost) {
        assert_eq!(host.0.identity(), self.native.animations.identity());
        self.native.animations.TickWorkletAnimations()
    }
    fn TracePrepareToDraw(&self, frame: i32) {
        self.native
            .events
            .Emit("cc", format!("PrepareToDraw source_frame_number={frame}"))
    }
    fn VerboseLogFrame(&self, frame: &FrameData<NativeCompositorTypes>) {
        if self
            .native
            .diagnostics
            .maximum_vlog_level
            .is_some_and(|level| level >= 3)
        {
            self.native
                .events
                .Emit("compositor.vlog", frame.ToString(self))
        }
    }
    fn BeginMainFrameReasonScroll(&self) -> super::input::BeginMainFrameReason {
        super::input::BeginMainFrameReason::kScroll
    }
    // Source dependency: ui/gfx/geometry/rect.cc:119-123,132-135;
    // ui/gfx/geometry/insets_outsets_base.cc:15-18.
    fn RectOutset(&self, rect: &mut Rect, amount: i32) {
        let inset = amount.saturating_neg();
        rect.Inset(inset, inset, inset, inset)
    }
    fn RectInsetsString(&self, outer: Rect, inner: Rect) -> String {
        format!(
            "x:{},{} y:{},{}",
            inner.x.saturating_sub(outer.x),
            outer.right().saturating_sub(inner.right()),
            inner.y.saturating_sub(outer.y),
            outer.bottom().saturating_sub(inner.bottom())
        )
    }
}

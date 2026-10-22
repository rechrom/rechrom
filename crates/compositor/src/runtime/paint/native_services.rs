//! Production CPU Paint value, recording and diagnostic capabilities.
//! These owners never retain a Page, Layer, Tile, PAC or compositor host.
use super::*;
use crate::{
    adapter::{
        display_item_list::{DisplayCanvas, DisplayPaintOpBuffer},
        geometry::{Rect, RectF, Vector2dF},
        input_types::ElementId,
        layer_tree_impl::Color4f,
        prioritized_tile::ScrollOffsetMap,
    },
    runtime::{
        paint_services::{CpuPaintFeatures, CpuPaintPlatform, CpuPaintServices},
        skia::SkRegion,
    },
};
use paint::cc::paint::blend_mode::BlendMode;
use paint::cc::paint::{
    paint_op_buffer::PaintOpBufferMetadata,
    paint_op_buffer_iterator::{PaintOpBufferView, PaintOpView},
    solid_color_analyzer::SolidColorAnalyzer,
};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

/// Diagnostics are independent of normal Paint. Under-invalidation checking
/// is selected through CpuPaintFeatures and requires a pixel-diagnostic backend;
/// this constructor explicitly rejects it instead of returning empty pixels.
#[derive(Clone, Copy, Debug)]
pub struct CpuNativePaintDiagnostics {
    pub trace: bool,
    pub verbose_level: i32,
    pub histogram: bool,
}
#[derive(Debug)]
pub enum CpuNativePaintConfigurationError {
    UnderInvalidationRasterBackendRequired,
    Entropy(String),
}
impl std::fmt::Display for CpuNativePaintConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self{Self::UnderInvalidationRasterBackendRequired=>write!(f,"under-invalidation checking needs a real diagnostic raster/overlay backend; disable that diagnostic explicitly for the CPU Paint constructor"),Self::Entropy(e)=>write!(f,"Paint metric sampler entropy: {e}")}
    }
}
impl std::error::Error for CpuNativePaintConfigurationError {}
struct Diagnostics {
    config: CpuNativePaintDiagnostics,
    crash_keys: Mutex<BTreeMap<(String, String), String>>,
    random: AtomicU64,
    diagnostic_dump_count: AtomicU64,
}
impl Diagnostics {
    fn event(&self, kind: &str, name: &str, value: serde_json::Value) {
        if self.config.trace {
            eprintln!(
                "{}",
                serde_json::json!({"component":"paint","kind":kind,"name":name,"value":value})
            );
        }
    }
    fn scope(self: &Arc<Self>, name: &str, value: serde_json::Value) -> PaintTraceScope {
        self.event("begin", name, value);
        Box::new(TraceScope {
            diagnostics: self.clone(),
            name: name.to_owned(),
            crash_key: None,
        })
    }
}
struct TraceScope {
    diagnostics: Arc<Diagnostics>,
    name: String,
    crash_key: Option<(String, String)>,
}
impl CpuPaintTraceScope for TraceScope {}
impl Drop for TraceScope {
    fn drop(&mut self) {
        if let Some(key) = self.crash_key.take() {
            self.diagnostics
                .crash_keys
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(&key);
        }
        self.diagnostics
            .event("end", &self.name, serde_json::Value::Null);
    }
}
struct ObjectTrace {
    diagnostics: Arc<Diagnostics>,
    id: i32,
}
impl CpuPaintObjectTracker for ObjectTrace {}
impl Drop for ObjectTrace {
    fn drop(&mut self) {
        self.diagnostics
            .event("object-deleted", "PictureLayer", serde_json::json!(self.id));
    }
}
struct NativeAnalysisTrace(Arc<Diagnostics>);
impl CpuPaintAnalysisTrace for NativeAnalysisTrace {
    fn Scope(&self, name: &'static str) -> PaintTraceScope {
        self.0.scope(name, serde_json::Value::Null)
    }
}

pub struct CpuNativePaintValueServices {
    diagnostics: Arc<Diagnostics>,
}
impl CpuPaintValueServices<SkRegion> for CpuNativePaintValueServices {
    fn Metadata(&self, b: &CpuPaintOpBuffer<SkRegion>) -> CpuBufferMetadata {
        super::native_metadata::project(b)
    }
    fn AnalyzeAddedOp(
        &self,
        op: CpuPaintOpRef<'_, SkRegion>,
        m: &mut PaintOpBufferMetadata<crate::adapter::layer_tree_host_impl::ContentColorUsage>,
    ) {
        super::native_metadata::analyze(op, m)
    }
    fn NeedsAdditionalInvalidationForLCDText(
        &self,
        new: &CpuPaintOpBuffer<SkRegion>,
        old: &CpuPaintOpBuffer<SkRegion>,
    ) -> bool {
        new.canonical_metadata()
            .NeedsAdditionalInvalidationForLCDText(old.canonical_metadata())
    }
    fn GenerateDiscardableImageMap(
        &self,
        b: &CpuPaintOpBuffer<SkRegion>,
        bounds: Rect,
        offsets: &ScrollOffsetMap,
        decoding: Option<
            &mut BTreeMap<crate::adapter::image_animation::PaintImageId, ImageDecodingMode>,
        >,
        worklets: Option<&mut CpuPaintWorkletInputs>,
    ) -> CpuDiscardableImageMap {
        super::native_discardable::NativeImageMap::Generate(
            b,
            bounds,
            offsets,
            decoding,
            worklets,
            Arc::new(NativeAnalysisTrace(self.diagnostics.clone())),
        )
    }
    fn DetermineIfSolidColor(
        &self,
        b: &CpuPaintOpBuffer<SkRegion>,
        r: Rect,
        max: i32,
        offsets: Option<&[usize]>,
    ) -> Option<Color4f> {
        SolidColorAnalyzer::DetermineIfSolidColor(
            b,
            foundation::gfx::Rect::new(
                foundation::gfx::Point::new(r.x, r.y),
                foundation::gfx::Size::new(r.width, r.height),
            ),
            max,
            offsets,
            &CpuSolidColorBackend::new(Arc::new(NativeAnalysisTrace(self.diagnostics.clone()))),
        )
        .map(|c| Color4f {
            red: c[0],
            green: c[1],
            blue: c[2],
            alpha: c[3],
        })
    }
    fn TraceSnapshot(
        &self,
        b: &CpuPaintOpBuffer<SkRegion>,
        bounds: Option<Rect>,
        all: &BTreeMap<usize, Rect>,
        scrolls: &BTreeMap<
            ElementId,
            crate::cc::paint::display_item_list::RasterInducingScrollInfo,
        >,
    ) {
        if self.diagnostics.config.trace {
            eprintln!("{}", self.ToTraceString(b, bounds, all, scrolls));
        }
    }
    fn ToTraceString(
        &self,
        b: &CpuPaintOpBuffer<SkRegion>,
        bounds: Option<Rect>,
        all: &BTreeMap<usize, Rect>,
        scrolls: &BTreeMap<
            ElementId,
            crate::cc::paint::display_item_list::RasterInducingScrollInfo,
        >,
    ) -> String {
        let ops = (0..b.Used())
            .map(|i| log_op(b.GetOp(i), i))
            .collect::<Vec<_>>();
        serde_json::json!({"ops":ops,"bounds":bounds.map(rect_json),"opBounds":all.iter().map(|(i,r)|serde_json::json!({"offset":i,"rect":rect_json(*r)})).collect::<Vec<_>>(),"rasterInducingScrolls":scrolls.keys().map(|e|format!("{e:?}")).collect::<Vec<_>>()}).to_string()
    }
    fn TraceRasterBegin(&self, n: usize) -> bool {
        self.diagnostics.event(
            "begin",
            "DisplayItemList::Raster",
            serde_json::json!({"total_ops":n}),
        );
        self.diagnostics.config.trace
    }
    fn TraceRasterEnd(&self, n: usize) {
        self.diagnostics.event(
            "end",
            "DisplayItemList::Raster",
            serde_json::json!({"rastered_ops":n}),
        );
    }
}
fn rect_json(r: Rect) -> serde_json::Value {
    serde_json::json!([r.x, r.y, r.width, r.height])
}
fn log_op(op: CpuPaintOpRef<'_, SkRegion>, offset: usize) -> serde_json::Value {
    let mut value = serde_json::json!({"offset":offset,"type":format!("{:?}",op.GetType())});
    match op {
        CpuPaintOpRef::Native { op, .. } => {
            value["rect"] =
                serde_json::json!([op.rect.x, op.rect.y, op.rect.width, op.rect.height]);
            value["color"] =
                serde_json::json!([op.color.red, op.color.green, op.color.blue, op.color.alpha]);
            value["image_resource"] = serde_json::json!(op.resource_id);
            value["matrix"] = serde_json::json!(op.transform.values);
            value["path"] =
                serde_json::json!(op.path.iter().map(|p| format!("{p:?}")).collect::<Vec<_>>());
            value["glyph_count"] = serde_json::json!(op.glyphs.len());
        }
        CpuPaintOpRef::Converted(CpuPaintOp::DrawColor { color, blend }) => {
            value["color"] = serde_json::json!([color.red, color.green, color.blue, color.alpha]);
            value["blend"] = serde_json::json!(format!("{blend:?}"));
        }
        CpuPaintOpRef::Converted(CpuPaintOp::DrawRecord(r)) => {
            value["ops"] = serde_json::json!(r.buffer().map_or(0, |b| b.Used()));
        }
        _ => {}
    }
    value
}
/// Native recording output: replay appends real operations/immutable payload
/// leases into the new record owner. It does not render pixels or copy a scene.
struct Recorder {
    buffer: CpuPaintOpBuffer<SkRegion>,
    bounds: Rect,
    clips: super::native_discardable::MapCanvas,
    log: Vec<serde_json::Value>,
}
impl Recorder {
    fn new(values: Arc<dyn CpuPaintValueServices<SkRegion>>) -> Self {
        Self {
            buffer: CpuPaintOpBuffer::new(values),
            bounds: Rect::default(),
            clips: super::native_discardable::make_canvas(Rect::new(0, 0, i32::MAX, i32::MAX)),
            log: Vec::new(),
        }
    }
    fn emit(&mut self, op: CpuPaintOp<SkRegion>) {
        self.log.push(log_op(op.AsRef(), self.buffer.Used()));
        self.buffer.push(op);
    }
    fn record(
        &mut self,
        b: &CpuPaintOpBuffer<SkRegion>,
        p: &mut RasterPlaybackParams<'_, SkRegion>,
        offsets: Option<&[usize]>,
    ) {
        use paint::cc::paint::discardable_image_map::{
            DiscardableImageBackend, ImageGatheringCanvas,
        };
        for index in 0..offsets.map_or(b.Used(), |o| o.len()) {
            let i = offsets.map_or(index, |o| o[index]);
            let op = b.GetOp(i);
            match op {
                CpuPaintOpRef::Converted(CpuPaintOp::DrawRecord(r)) => {
                    if let Some(nested) = r.buffer() {
                        self.record(nested, p, None);
                    }
                }
                CpuPaintOpRef::Converted(CpuPaintOp::DrawScrollingContents { element, list }) => {
                    let offset = p
                        .raster_inducing_scroll_offsets
                        .expect("source scrolling playback requires its actual offsets")
                        .get(element)
                        .expect("source CHECK(scroll offset)");
                    let (x, y) = (-offset.x(), -offset.y());
                    self.emit(CpuPaintOp::Save);
                    self.emit(CpuPaintOp::Translate(x, y));
                    self.clips.save();
                    self.clips.translate(x, y);
                    self.record(list.paint_op_buffer(), p, None);
                    self.clips.restore();
                    self.emit(CpuPaintOp::Restore);
                }
                _ => {
                    if !op.IsDrawOp() {
                        let original = self.clips.getLocalToDevice();
                        super::native_discardable::raster_state(op, &mut self.clips, &original);
                    }
                    self.emit(b.RetainOpAt(i));
                }
            }
        }
    }
}
impl DisplayCanvas for Recorder {
    fn LocalClipBounds(&self) -> Option<Rect> {
        self.clips.LocalClipBounds()
    }
}
impl CpuPaintCanvas<SkRegion> for Recorder {
    fn Playback(
        &mut self,
        b: &CpuPaintOpBuffer<SkRegion>,
        p: &mut RasterPlaybackParams<'_, SkRegion>,
        _: bool,
        offsets: Option<&[usize]>,
    ) {
        self.record(b, p, offsets)
    }
}
impl CpuLoggingCanvasValue<SkRegion> for Recorder {
    fn IntoLog(self: Box<Self>) -> serde_json::Value {
        serde_json::Value::Array(self.log)
    }
}
impl CpuPictureRecorderValue<SkRegion> for Recorder {
    fn BeginRecording(&mut self, w: i32, h: i32) {
        assert!(self.buffer.Used() == 0, "source recorder begins once");
        self.bounds = Rect::new(0, 0, w, h);
        self.clips = super::native_discardable::make_canvas(self.bounds);
    }
    fn Clear(&mut self, c: Color4f) {
        self.emit(CpuPaintOp::DrawColor {
            color: c,
            blend: BlendMode::kSrc,
        });
    }
    fn DrawColor(&mut self, c: Color4f) {
        self.emit(CpuPaintOp::DrawColor {
            color: c,
            blend: BlendMode::kSrcOver,
        });
    }
    fn FinishRecordingAsPicture(mut self: Box<Self>) -> Arc<Picture<SkRegion>> {
        Arc::new(Picture {
            record: self.buffer.ReleaseAsRecord(),
            bounds: self.bounds,
        })
    }
}
pub struct CpuNativePaintPlatform {
    values: Arc<CpuNativePaintValueServices>,
    diagnostics: Arc<Diagnostics>,
}
impl CpuPaintPlatform<SkRegion> for CpuNativePaintPlatform {
    fn VlogIsOn(&self, level: i32) -> bool {
        level <= self.diagnostics.config.verbose_level
    }
    fn CrashKey(&self, name: &str, section: &str, value: String) -> PaintTraceScope {
        self.diagnostics
            .crash_keys
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert((name.to_owned(), section.to_owned()), value);
        Box::new(TraceScope {
            diagnostics: self.diagnostics.clone(),
            name: name.to_owned(),
            crash_key: Some((name.to_owned(), section.to_owned())),
        })
    }
    fn DumpWithoutCrashing(&self) {
        // Chromium sends these crash keys to its out-of-process dump
        // reporter. Serializing the full property trees to stderr on the Page
        // path can itself block rendering for seconds. Preserve an observable,
        // rate-limited diagnostic while keeping the already-retained keys in
        // their owner for a debugger/dump backend.
        let count = self
            .diagnostics
            .diagnostic_dump_count
            .fetch_add(1, Ordering::Relaxed)
            + 1;
        if count == 1 || count.is_power_of_two() {
            let keys = self
                .diagnostics
                .crash_keys
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            let chunk = keys
                .get(&("ChunkId".to_owned(), "id".to_owned()))
                .cloned()
                .unwrap_or_default();
            eprintln!(
                "{}",
                serde_json::json!({
                    "component": "paint",
                    "kind": "diagnostic-dump",
                    "count": count,
                    "crash_key_count": keys.len(),
                    "chunk": chunk,
                })
            );
        }
    }
    fn BeginPacUpdateTrace(&self) -> PaintTraceScope {
        self.diagnostics
            .scope("PaintArtifactCompositor::Update", serde_json::Value::Null)
    }
    fn ShouldRecordSubsampledMetric(&self, p: f64) -> bool {
        if !self.diagnostics.config.histogram {
            return false;
        }
        let mut x = self
            .diagnostics
            .random
            .fetch_add(0x9e3779b97f4a7c15, Ordering::Relaxed);
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
        x ^= x >> 31;
        (x >> 11) as f64 * (1. / ((1u64 << 53) as f64)) < p
    }
    fn HistogramCounts100(&self, name: &str, v: i32) {
        if self.diagnostics.config.histogram {
            eprintln!("Paint histogram {name}={v}");
        }
    }
    fn ColorNameForLayoutTree(&self, c: Color4f) -> String {
        let byte = |v: f32| (v.clamp(0., 1.) * 255.).round() as u8;
        if c.alpha == 1. {
            format!(
                "#{:02X}{:02X}{:02X}",
                byte(c.red),
                byte(c.green),
                byte(c.blue)
            )
        } else {
            format!(
                "#{:02X}{:02X}{:02X}{:02X}",
                byte(c.red),
                byte(c.green),
                byte(c.blue),
                byte(c.alpha)
            )
        }
    }
    fn CreateLoggingCanvas(&self) -> CpuLoggingCanvas<SkRegion> {
        CpuLoggingCanvas(Box::new(Recorder::new(self.values.clone())))
    }
    fn Diagnostic(&self, message: &str) {
        eprintln!("Paint: {message}");
    }
    fn TraceLayer(&self, event: &str, element: Option<ElementId>) {
        self.diagnostics
            .event("layer", event, serde_json::json!(format!("{element:?}")));
    }
    fn Vector2dFToString(&self, v: Vector2dF) -> String {
        format!("{},{}", v.x(), v.y())
    }
    fn TraceScope(&self, category: &str, name: &str, count: Option<usize>) {
        self.diagnostics
            .event(category, name, serde_json::json!(count));
    }
    fn CreateObjectTracker(&self, id: i32) -> ObjectTracker {
        self.diagnostics
            .event("object-created", "PictureLayer", serde_json::json!(id));
        Box::new(ObjectTrace {
            diagnostics: self.diagnostics.clone(),
            id,
        })
    }
    fn BeginPushPropertiesTrace(&self) -> PaintTraceScope {
        self.diagnostics
            .scope("PictureLayer::PushPropertiesTo", serde_json::Value::Null)
    }
    fn BeginUpdateTrace(&self, n: i32) -> PaintTraceScope {
        self.diagnostics
            .scope("PictureLayer::Update", serde_json::json!(n))
    }
    fn BeginUpdateLayerTask(&self, layer: i32, host: i32) -> PaintTraceScope {
        self.diagnostics.scope(
            "UpdateLayer",
            serde_json::json!({"layer":layer,"host":host}),
        )
    }
    fn CreatePictureRecorder(&self) -> CpuPictureRecorder<SkRegion> {
        CpuPictureRecorder(Box::new(Recorder::new(self.values.clone())))
    }
}
pub struct CpuNativeRasterTrackingServices {
    diagnostics: Arc<Diagnostics>,
}
impl CpuRasterTrackingServices<SkRegion> for CpuNativeRasterTrackingServices {
    fn PaintUnderInvalidationCheckingEnabled(&self) -> bool {
        false
    } // Constructor validates the explicit feature selection.
    fn IsTracingRasterInvalidations(&self) -> bool {
        self.diagnostics.config.trace
    }
    fn Rasterize(&self, _: &CpuPaintRecord<SkRegion>, _: Rect) -> Option<CpuTrackingBitmap> {
        unreachable!(
            "source CheckUnderInvalidations is guarded by explicitly disabled diagnostic feature"
        )
    }
    fn BuildUnderInvalidationRecord(
        &self,
        _: CpuPaintRecord<SkRegion>,
        _: CpuTrackingBitmap,
        _: Rect,
    ) -> CpuPaintRecord<SkRegion> {
        unreachable!("under-invalidation diagnostic backend was not selected")
    }
    fn LogUnderInvalidation(&self, n: &str, x: i32, y: i32, old: u32, new: u32) {
        eprintln!("under-invalidation {n} at ({x},{y}): {old:08x}->{new:08x}");
    }
    fn LogMoreUnderInvalidations(&self) {
        eprintln!("additional Paint under-invalidations suppressed");
    }
}
impl CpuPaintServices<SkRegion> {
    /// The single default CPU assembly constructor. All normal Paint services
    /// are real value/record owners; unsupported pixel diagnostics fail here.
    pub fn new_native(
        features: CpuPaintFeatures,
        diagnostics: CpuNativePaintDiagnostics,
    ) -> Result<Self, CpuNativePaintConfigurationError> {
        if features.paint_under_invalidation_checking {
            return Err(CpuNativePaintConfigurationError::UnderInvalidationRasterBackendRequired);
        }
        let mut seed = [0u8; 8];
        getrandom::fill(&mut seed)
            .map_err(|e| CpuNativePaintConfigurationError::Entropy(e.to_string()))?;
        let diagnostics = Arc::new(Diagnostics {
            config: diagnostics,
            crash_keys: Mutex::new(BTreeMap::new()),
            random: AtomicU64::new(u64::from_ne_bytes(seed)),
            diagnostic_dump_count: AtomicU64::new(0),
        });
        let values = Arc::new(CpuNativePaintValueServices {
            diagnostics: diagnostics.clone(),
        });
        let tracking = Arc::new(CpuNativeRasterTrackingServices {
            diagnostics: diagnostics.clone(),
        });
        let platform = Arc::new(CpuNativePaintPlatform {
            values: values.clone(),
            diagnostics,
        });
        Ok(Self::new(features, values, tracking, platform))
    }
}

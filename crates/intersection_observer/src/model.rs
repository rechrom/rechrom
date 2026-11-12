use foundation::{gfx, Length};
use std::{rc::Rc, time::Duration};
pub const MINIMUM_THRESHOLD: f32 = f32::MIN_POSITIVE;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliveryBehavior {
    PostTask,
    PostLifecycle,
    PostLayout,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MarginTarget {
    Root,
    Target,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OcclusionState {
    Unknown,
    GuaranteedNotOccluded,
    PossiblyOccluded,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequiredIntersectionMechanism {
    PreciseClipPath,
    VisibilityHitTest,
    DistortingVisualEffects,
    CrossFrameGeometry,
    FilterVisualRect,
    MaskVisualClip,
    ReplacedContentTransform,
    FrameOcclusionState,
}
#[derive(Clone)]
pub struct IntersectionObserverParams {
    pub margin: Vec<Length>,
    pub margin_target: MarginTarget,
    pub scroll_margin: Vec<Length>,
    pub thresholds: Vec<f32>,
    pub fraction_of_root: bool,
    pub behavior: DeliveryBehavior,
    pub delay: Duration,
    pub track_visibility: bool,
    pub always_report_root_bounds: bool,
    pub use_overflow_clip_edge: bool,
    pub expose_occluder_id: bool,
    pub internal: bool,
}
impl Default for IntersectionObserverParams {
    fn default() -> Self {
        Self {
            margin: vec![],
            margin_target: MarginTarget::Root,
            scroll_margin: vec![],
            thresholds: vec![],
            fraction_of_root: false,
            behavior: DeliveryBehavior::PostLifecycle,
            delay: Duration::ZERO,
            track_visibility: false,
            always_report_root_bounds: false,
            use_overflow_clip_edge: false,
            expose_occluder_id: false,
            internal: true,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ObserverOptionsError {
    MarginArity,
    ThresholdRange,
    VisibilityDelay,
}
impl IntersectionObserverParams {
    // intersection_observer.cc:160-198,251-277. The CSS parser owns textual
    // margin tokens; this native interface receives source fixed/percent Lengths.
    pub fn Normalize(mut self) -> Result<Self, ObserverOptionsError> {
        self.margin = NormalizeMargins(&self.margin)?;
        self.scroll_margin = NormalizeMargins(&self.scroll_margin)?;
        if self.scroll_margin.iter().all(Length::IsZero) {
            self.scroll_margin.clear();
        }
        // Internal source clients supply their typed thresholds directly.
        // Parsing/validation belongs to the script-options Create overload.
        if !self.internal {
            if self.thresholds.is_empty() {
                self.thresholds.push(0.0);
            }
            if self
                .thresholds
                .iter()
                .any(|v| !v.is_finite() || *v < 0.0 || *v > 1.0)
            {
                return Err(ObserverOptionsError::ThresholdRange);
            }
            self.thresholds.sort_by(f32::total_cmp);
            if self.track_visibility && self.delay < Duration::from_millis(100) {
                return Err(ObserverOptionsError::VisibilityDelay);
            }
        }
        Ok(self)
    }
    pub fn RootMargin(&self) -> &[Length] {
        if self.margin_target == MarginTarget::Root {
            &self.margin
        } else {
            &[]
        }
    }
    pub fn TargetMargin(&self) -> &[Length] {
        if self.margin_target == MarginTarget::Target {
            &self.margin
        } else {
            &[]
        }
    }
}
fn NormalizeMargins(values: &[Length]) -> Result<Vec<Length>, ObserverOptionsError> {
    Ok(match values.len() {
        0 => vec![Length::Fixed(0); 4],
        1 => vec![values[0].clone(); 4],
        2 => vec![
            values[0].clone(),
            values[1].clone(),
            values[0].clone(),
            values[1].clone(),
        ],
        3 => vec![
            values[0].clone(),
            values[1].clone(),
            values[2].clone(),
            values[1].clone(),
        ],
        4 => values.to_vec(),
        _ => return Err(ObserverOptionsError::MarginArity),
    })
}
/// IntersectionObservation's retained source rect cache. It contains only
/// derived geometry; no LayoutObject or copied tree crosses the interface.
#[derive(Clone, Default)]
pub struct CachedRects {
    pub local_target_rect: gfx::RectF,
    pub unscrolled_unclipped_intersection_rect: gfx::RectF,
    pub min_scroll_delta_to_update: [f32; 2],
    pub does_intersect: bool,
    pub pre_margin_target_rect_is_empty: bool,
    pub valid: bool,
}
#[derive(Clone)]
pub struct RootGeometry {
    pub zoom: f32,
    pub pre_margin_local_root_rect: gfx::RectF,
    pub local_root_rect: gfx::RectF,
    pub root_to_view_transform: gfx::Transform,
}
#[derive(Clone, Debug, Default)]
pub struct IntersectionGeometry {
    pub target_rect: gfx::RectF,
    pub root_rect: gfx::RectF,
    pub intersection_rect: gfx::RectF,
    pub unclipped_intersection_rect: gfx::RectF,
    pub intersection_ratio: f64,
    pub threshold_index: usize,
    pub did_compute_geometry: bool,
    pub is_visible: bool,
    pub occluder_node_id: Option<u64>,
    pub should_report_root_bounds: bool,
}
impl IntersectionGeometry {
    pub fn IsIntersecting(&self) -> bool {
        self.threshold_index > 0
    }
}
#[derive(Clone)]
pub struct IntersectionObserverEntry {
    pub geometry: IntersectionGeometry,
    pub timestamp: f64,
    pub target: Rc<dyn crate::IntersectionObserverTarget>,
}
pub mod compute_flags {
    pub const REPORT_IMPLICIT_ROOT_BOUNDS: u32 = 1 << 0;
    pub const EXPLICIT_ROOT_NEEDS_UPDATE: u32 = 1 << 1;
    pub const IMPLICIT_ROOT_NEEDS_UPDATE: u32 = 1 << 2;
    pub const ANCESTOR_FRAME_DETACHED: u32 = 1 << 3;
    pub const IGNORE_DELAY: u32 = 1 << 4;
    pub const CAN_SKIP_STICKY_FRAME_TRACKING: u32 = 1 << 5;
    pub const POST_LAYOUT_ONLY: u32 = 1 << 6;
    pub const SCROLL_AND_VISIBILITY_ONLY: u32 = 1 << 7;
    pub const CONSUME_SCROLL_DELTA: u32 = 1 << 8;
    pub const UPDATE_TRACKING: u32 = 1 << 9;
}
pub mod geometry_flags {
    pub const REPORT_ROOT_BOUNDS: u32 = 1 << 0;
    pub const COMPUTE_VISIBILITY: u32 = 1 << 1;
    pub const FRACTION_OF_ROOT: u32 = 1 << 2;
    pub const FRAME_VIEWPORT: u32 = 1 << 3;
    pub const CONVERT_TO_CSS_PIXELS: u32 = 1 << 4;
    pub const OVERFLOW_CLIP_EDGE: u32 = 1 << 5;
    pub const RESPECT_FILTERS: u32 = 1 << 6;
    pub const SCROLL_AND_VISIBILITY_ONLY: u32 = 1 << 7;
    pub const USE_CACHED_RECTS: u32 = 1 << 8;
    pub const ROOT_IMPLICIT: u32 = 1 << 9;
    pub const DID_COMPUTE: u32 = 1 << 10;
    pub const IS_VISIBLE: u32 = 1 << 11;
    pub const EXPOSE_OCCLUDER_ID: u32 = 1 << 12;
}
impl RootGeometry {
    pub fn UpdateMargin(&mut self, margin: &[Length]) {
        self.local_root_rect = self.pre_margin_local_root_rect;
        ApplyMargin(
            &mut self.local_root_rect,
            margin,
            self.zoom,
            self.pre_margin_local_root_rect.size(),
        );
    }
}
/// intersection_geometry.cc:38-76, including source percentage truncation.
pub fn ApplyMargin(rect: &mut gfx::RectF, margin: &[Length], zoom: f32, reference: gfx::SizeF) {
    if margin.is_empty() {
        return;
    }
    assert_eq!(margin.len(), 4);
    let resolve = |value: &Length, reference: f32| {
        if value.IsPercent() {
            foundation::LayoutUnit::from_signed((reference * value.PercentValue() / 100.0) as i32)
        } else {
            assert!(value.IsFixed());
            foundation::LayoutUnit::from_f32(value.Pixels() * zoom)
        }
    };
    rect.OutsetEdges(gfx::OutsetsF::new(
        resolve(&margin[0], reference.height()).ToFloat(),
        resolve(&margin[1], reference.width()).ToFloat(),
        resolve(&margin[2], reference.height()).ToFloat(),
        resolve(&margin[3], reference.width()).ToFloat(),
    ));
}

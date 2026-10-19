//! Paint owns the compositor filter value. Authored style operations are consumed
//! here, before publishing Effect property nodes; compositor never rebuilds a
//! filter list while traversing the PaintArtifact.
use crate::cc::paint::{
    filter_operation::{FilterOperation, FilterType},
    filter_operations::FilterOperations,
};
use foundation::gfx::Point;
use layoutng_assembly::internal::{
    layout_input::Offset,
    layout_input_types::Color,
    paint_input::{PaintFilterOperation, PaintFilterType},
};
use skia::include::core::SkColor::SkColor4f;
use std::{fmt, sync::Arc};

/// Explicitly retained unsupported input, never an empty reference filter.
#[derive(Clone, Debug, PartialEq)]
pub struct FilterInputBlocker {
    pub operation_index: usize,
    pub operation_count: usize,
    pub reason: FilterInputBlockReason,
    pub source: Arc<PaintFilterOperation>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterInputBlockReason {
    NonFiniteValue,
    ReferenceOwnerNotLowered,
    ReflectionOwnerNotLowered,
}
impl fmt::Display for FilterInputBlocker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "filter operation {}: {:?}",
            self.operation_index, self.reason
        )
    }
}
impl std::error::Error for FilterInputBlocker {}

/// Source: third_party/blink/renderer/platform/graphics/paint/
/// effect_paint_property_node.h:66-95. One canonical operations owner per effect,
/// rather than a second authored vector beside a compositor conversion cache.
#[derive(Clone, Debug, PartialEq)]
pub enum PropertyFilterOperations {
    Ready(FilterOperations),
    Blocked(FilterInputBlocker),
}
impl Default for PropertyFilterOperations {
    fn default() -> Self {
        Self::Ready(FilterOperations::default())
    }
}
impl PropertyFilterOperations {
    /// Source: third_party/blink/renderer/core/paint/filter_effect_builder.cc:373-510;
    /// platform/graphics/compositor_filter_operations.cc:22-109. PrePaint has
    /// already resolved zoom, percentages, currentColor and image resources.
    pub fn from_authored(input: &[PaintFilterOperation]) -> Self {
        let mut operations = Vec::with_capacity(input.len());
        for (operation_index, op) in input.iter().enumerate() {
            let blocked = |reason| {
                Self::Blocked(FilterInputBlocker {
                    operation_index,
                    operation_count: input.len(),
                    reason,
                    source: Arc::new(op.clone()),
                })
            };
            if ![
                op.amount,
                op.offset.x,
                op.offset.y,
                op.blur_radius,
                f64::from(op.color.red),
                f64::from(op.color.green),
                f64::from(op.color.blue),
                f64::from(op.color.alpha),
            ]
            .iter()
            .all(|v| v.is_finite())
            {
                return blocked(FilterInputBlockReason::NonFiniteValue);
            }
            use PaintFilterType::*;
            let value = match op.r#type {
                kGrayscale => FilterOperation::CreateGrayscaleFilter(op.amount as f32),
                kSepia => FilterOperation::CreateSepiaFilter(op.amount as f32),
                kSaturate => FilterOperation::CreateSaturateFilter(op.amount as f32),
                kHueRotate => FilterOperation::CreateHueRotateFilter(op.amount as f32),
                kInvert => FilterOperation::CreateInvertFilter(op.amount as f32),
                kBrightness => FilterOperation::CreateBrightnessFilter(op.amount as f32),
                kContrast => FilterOperation::CreateContrastFilter(op.amount as f32),
                kOpacity => FilterOperation::CreateOpacityFilter(op.amount as f32),
                kBlur => FilterOperation::CreateBlurFilter(op.amount as f32),
                kDropShadow => FilterOperation::CreateDropShadowFilter(
                    Point::new(op.offset.x.floor() as i32, op.offset.y.floor() as i32),
                    op.blur_radius as f32,
                    // AppendDropShadowFilter passes Color::Rgb through SkColor.
                    SkColor4f::new(
                        byte_color(op.color.red),
                        byte_color(op.color.green),
                        byte_color(op.color.blue),
                        byte_color(op.color.alpha),
                    ),
                ),
                kReference => return blocked(FilterInputBlockReason::ReferenceOwnerNotLowered),
                kBoxReflect => return blocked(FilterInputBlockReason::ReflectionOwnerNotLowered),
            };
            operations.push(value);
        }
        Self::Ready(FilterOperations::new(operations))
    }
    pub fn canonical(&self) -> Result<&FilterOperations, &FilterInputBlocker> {
        match self {
            Self::Ready(value) => Ok(value),
            Self::Blocked(error) => Err(error),
        }
    }
    pub fn blocker(&self) -> Option<&FilterInputBlocker> {
        self.canonical().err()
    }
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Ready(value) if value.IsEmpty())
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Ready(value) => value.size(),
            Self::Blocked(error) => error.operation_count,
        }
    }
    pub fn is_finite(&self) -> bool {
        self.canonical().is_ok()
    }
    /// Resource preparation may inspect an explicitly blocked reflection's
    /// actual source payload; this does not certify it as a native filter.
    pub fn reflection_mask_layers(
        &self,
    ) -> &[layoutng_assembly::internal::paint_input::PaintReflectionMaskLayer] {
        match self {
            Self::Blocked(error) if error.source.r#type == PaintFilterType::kBoxReflect => {
                &error.source.reflection_mask_layers
            }
            _ => &[],
        }
    }

    /// Explicit legacy raster command boundary. The returned values are the
    /// newly emitted replay command payload, never cached on a property node.
    /// A blocked native owner propagates its typed error before any operation
    /// is emitted; it cannot become a partial or empty chain.
    pub fn replay_operations(&self) -> Result<Vec<PaintFilterOperation>, FilterReplayError> {
        self.canonical()
            .map_err(|e| FilterReplayError::Input(e.clone()))?
            .operations()
            .iter()
            .map(replay_operation)
            .collect()
    }
}
fn byte_color(value: f32) -> f32 {
    ((value.clamp(0., 1.) * 255.).round() as u8) as f32 / 255.
}
#[derive(Clone, Debug, PartialEq)]
pub enum FilterReplayError {
    Input(FilterInputBlocker),
    UnsupportedCanonicalType(FilterType),
}
impl fmt::Display for FilterReplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(e) => e.fmt(f),
            Self::UnsupportedCanonicalType(t) => {
                write!(f, "legacy raster cannot replay canonical {t:?}")
            }
        }
    }
}
impl std::error::Error for FilterReplayError {}
impl From<FilterReplayError> for std::io::Error {
    fn from(e: FilterReplayError) -> Self {
        Self::new(std::io::ErrorKind::Unsupported, e)
    }
}
fn replay_operation(op: &FilterOperation) -> Result<PaintFilterOperation, FilterReplayError> {
    use FilterType::*;
    let mut value = PaintFilterOperation::default();
    value.r#type = match op.r#type() {
        GRAYSCALE => PaintFilterType::kGrayscale,
        SEPIA => PaintFilterType::kSepia,
        SATURATE => PaintFilterType::kSaturate,
        HUE_ROTATE => PaintFilterType::kHueRotate,
        INVERT => PaintFilterType::kInvert,
        BRIGHTNESS => PaintFilterType::kBrightness,
        CONTRAST => PaintFilterType::kContrast,
        OPACITY => PaintFilterType::kOpacity,
        BLUR => PaintFilterType::kBlur,
        DROP_SHADOW => PaintFilterType::kDropShadow,
        kind => return Err(FilterReplayError::UnsupportedCanonicalType(kind)),
    };
    value.amount = f64::from(op.amount());
    if op.r#type() == DROP_SHADOW {
        let offset = op.offset();
        value.offset = Offset {
            x: f64::from(offset.x()),
            y: f64::from(offset.y()),
        };
        value.blur_radius = f64::from(op.amount());
        let color = op.drop_shadow_color();
        value.color = Color {
            red: color.fR,
            green: color.fG,
            blue: color.fB,
            alpha: color.fA,
        };
    }
    Ok(value)
}

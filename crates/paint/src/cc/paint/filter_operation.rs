// Translation status: complete
// Copyright 2013 The Chromium Authors. BSD-style license in LICENSE.
//! Canonical cc FilterOperation values and bounds policy. Actual immutable
//! PaintFilter image graphs retain their owning Skia value allocation.
#![allow(non_snake_case, non_camel_case_types, dead_code)]
use foundation::gfx::{OutsetsF, Point, Rect, RectF, ToEnclosingRect};
use std::sync::Arc;
pub type FilterColor = super::color::Color4f;
// Source: cc/paint/filter_operation.h:39-57.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilterType {
    GRAYSCALE,
    SEPIA,
    SATURATE,
    HUE_ROTATE,
    INVERT,
    BRIGHTNESS,
    CONTRAST,
    OPACITY,
    BLUR,
    DROP_SHADOW,
    COLOR_MATRIX,
    ZOOM,
    REFERENCE,
    SATURATING_BRIGHTNESS,
    ALPHA_THRESHOLD,
    OFFSET,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlurTileMode {
    kClamp,
    kRepeat,
    kMirror,
    kDecal,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapDirection {
    Forward,
    Reverse,
}
/// Minimal device-transform capability required by filter bounds mapping.
///
/// Chromium passes an SkMatrix because FilterOperation needs only
/// SkMatrix::mapVector. Keeping that semantic operation at this boundary lets
/// each raster implementation use its own actual Skia matrix without exposing
/// either concrete matrix type through Paint, Compositor or Renderer APIs.
pub trait FilterMapTransform {
    fn MapVector(&self, x: f32, y: f32) -> (f32, f32);
}
/// Source PaintFilter::MapRect is an immutable image-filter value operation.
/// This boundary cannot be implemented by a default/empty/cull approximation.
pub trait PaintFilter: Send + Sync + std::fmt::Debug {
    fn MapRect(
        &self,
        rect: Rect,
        ctm: Option<&dyn FilterMapTransform>,
        direction: MapDirection,
    ) -> Rect;
    fn TypeToString(&self) -> &'static str;
    /// Borrows the actual immutable value graph for CPU execution. The replay
    /// owner carries source inputs; consumers never reverse-engineer MapRect.
    fn ReplayGraph(&self) -> PaintFilterReplay<'_>;
}
pub enum PaintFilterReplay<'a> {
    Operations(&'a super::filter_operations::FilterOperations),
    Reference {
        operation: &'a layoutng_assembly::internal::paint_input::PaintFilterOperation,
        reference_box: crate::PaintRect,
        reference_scale: f64,
    },
    Reflection(&'a layoutng_assembly::internal::paint_input::PaintFilterOperation),
}
// Source: cc/paint/filter_operation.h:272-283; cc/paint/filter_operation.cc:48-144.
#[derive(Clone, Debug)]
pub struct FilterOperation {
    type_: FilterType,
    amount_: f32,
    offset_: Point,
    drop_shadow_color_: FilterColor,
    image_filter_: Option<Arc<dyn PaintFilter>>,
    matrix_: [f32; 20],
    zoom_inset_: i32,
    shape_: Vec<Rect>,
    blur_tile_mode_: Option<BlurTileMode>,
}
impl Default for FilterOperation {
    fn default() -> Self {
        Self::new(FilterType::GRAYSCALE, 0.)
    }
}
// Source: cc/paint/filter_operation.cc:25-46.
impl PartialEq for FilterOperation {
    fn eq(&self, b: &Self) -> bool {
        use FilterType::*;
        if self.type_ != b.type_ {
            return false;
        }
        match self.type_ {
            COLOR_MATRIX => self.matrix_ == b.matrix_,
            BLUR => self.amount_ == b.amount_ && self.blur_tile_mode_ == b.blur_tile_mode_,
            DROP_SHADOW => {
                self.amount_ == b.amount_
                    && self.offset_ == b.offset_
                    && self.drop_shadow_color_ == b.drop_shadow_color_
            }
            REFERENCE => match (&self.image_filter_, &b.image_filter_) {
                (None, None) => true,
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                _ => false,
            },
            ALPHA_THRESHOLD => self.shape_ == b.shape_,
            OFFSET => self.offset_ == b.offset_,
            _ => self.amount_ == b.amount_,
        }
    }
}
impl FilterOperation {
    fn new(type_: FilterType, amount_: f32) -> Self {
        Self {
            type_,
            amount_,
            offset_: Point::default(),
            drop_shadow_color_: FilterColor::default(),
            image_filter_: None,
            matrix_: [0.; 20],
            zoom_inset_: 0,
            shape_: Vec::new(),
            blur_tile_mode_: None,
        }
    }
    // Source: cc/paint/filter_operation.h:61-113,186-238.
    pub fn r#type(&self) -> FilterType {
        self.type_
    }
    pub fn amount(&self) -> f32 {
        debug_assert!(!matches!(
            self.type_,
            FilterType::ALPHA_THRESHOLD | FilterType::COLOR_MATRIX | FilterType::REFERENCE
        ));
        self.amount_
    }
    pub fn offset(&self) -> Point {
        debug_assert!(matches!(
            self.type_,
            FilterType::DROP_SHADOW | FilterType::OFFSET
        ));
        self.offset_
    }
    pub fn drop_shadow_color(&self) -> FilterColor {
        debug_assert_eq!(self.type_, FilterType::DROP_SHADOW);
        self.drop_shadow_color_
    }
    pub fn image_filter(&self) -> &Option<Arc<dyn PaintFilter>> {
        debug_assert_eq!(self.type_, FilterType::REFERENCE);
        &self.image_filter_
    }
    pub fn matrix(&self) -> &[f32; 20] {
        debug_assert_eq!(self.type_, FilterType::COLOR_MATRIX);
        &self.matrix_
    }
    pub fn zoom_inset(&self) -> i32 {
        debug_assert_eq!(self.type_, FilterType::ZOOM);
        self.zoom_inset_
    }
    pub fn shape(&self) -> &[Rect] {
        debug_assert_eq!(self.type_, FilterType::ALPHA_THRESHOLD);
        &self.shape_
    }
    pub fn blur_tile_mode(&self) -> BlurTileMode {
        debug_assert_eq!(self.type_, FilterType::BLUR);
        self.blur_tile_mode_
            .expect("BLUR tile mode must be initialized")
    }
    pub fn CreateEmptyFilter() -> Self {
        Self::default()
    }
    pub fn set_type(&mut self, t: FilterType) {
        self.type_ = t;
    }
    pub fn set_amount(&mut self, v: f32) {
        debug_assert!(!matches!(
            self.type_,
            FilterType::ALPHA_THRESHOLD | FilterType::COLOR_MATRIX | FilterType::REFERENCE
        ));
        self.amount_ = v;
    }
    pub fn set_offset(&mut self, v: Point) {
        debug_assert!(matches!(
            self.type_,
            FilterType::DROP_SHADOW | FilterType::OFFSET
        ));
        self.offset_ = v;
    }
    pub fn set_drop_shadow_color(&mut self, v: FilterColor) {
        debug_assert_eq!(self.type_, FilterType::DROP_SHADOW);
        self.drop_shadow_color_ = v;
    }
    pub fn set_image_filter(&mut self, v: Option<Arc<dyn PaintFilter>>) {
        debug_assert_eq!(self.type_, FilterType::REFERENCE);
        self.image_filter_ = v;
    }
    pub fn set_matrix(&mut self, v: &[f32; 20]) {
        debug_assert_eq!(self.type_, FilterType::COLOR_MATRIX);
        self.matrix_ = *v;
    }
    pub fn set_zoom_inset(&mut self, v: i32) {
        debug_assert_eq!(self.type_, FilterType::ZOOM);
        self.zoom_inset_ = v;
    }
    pub fn set_shape(&mut self, v: &[Rect]) {
        debug_assert_eq!(self.type_, FilterType::ALPHA_THRESHOLD);
        self.shape_ = v.to_vec();
    }
    pub fn set_blur_tile_mode(&mut self, v: BlurTileMode) {
        debug_assert_eq!(self.type_, FilterType::BLUR);
        self.blur_tile_mode_ = Some(v);
    }
}
// Source: cc/paint/filter_operation.h:116-185; cc/paint/filter_operation.cc:64-144.
impl FilterOperation {
    pub fn CreateGrayscaleFilter(v: f32) -> Self {
        Self::new(FilterType::GRAYSCALE, v)
    }
    pub fn CreateSepiaFilter(v: f32) -> Self {
        Self::new(FilterType::SEPIA, v)
    }
    pub fn CreateSaturateFilter(v: f32) -> Self {
        Self::new(FilterType::SATURATE, v)
    }
    pub fn CreateHueRotateFilter(v: f32) -> Self {
        Self::new(FilterType::HUE_ROTATE, v)
    }
    pub fn CreateInvertFilter(v: f32) -> Self {
        Self::new(FilterType::INVERT, v)
    }
    pub fn CreateBrightnessFilter(v: f32) -> Self {
        Self::new(FilterType::BRIGHTNESS, v)
    }
    pub fn CreateContrastFilter(v: f32) -> Self {
        Self::new(FilterType::CONTRAST, v)
    }
    pub fn CreateOpacityFilter(v: f32) -> Self {
        Self::new(FilterType::OPACITY, v)
    }
    pub fn CreateBlurFilter(v: f32) -> Self {
        Self::CreateBlurFilterWithTileMode(v, BlurTileMode::kDecal)
    }
    pub fn CreateBlurFilterWithTileMode(v: f32, mode: BlurTileMode) -> Self {
        let mut r = Self::new(FilterType::BLUR, v);
        r.blur_tile_mode_ = Some(mode);
        r
    }
    pub fn CreateDropShadowFilter(offset: Point, sigma: f32, color: FilterColor) -> Self {
        let mut r = Self::new(FilterType::DROP_SHADOW, sigma);
        r.offset_ = offset;
        r.drop_shadow_color_ = color;
        r
    }
    pub fn CreateColorMatrixFilter(matrix: [f32; 20]) -> Self {
        let mut r = Self::new(FilterType::COLOR_MATRIX, 0.);
        r.matrix_ = matrix;
        r
    }
    pub fn CreateZoomFilter(v: f32, inset: i32) -> Self {
        let mut r = Self::new(FilterType::ZOOM, v);
        r.zoom_inset_ = inset;
        r
    }
    pub fn CreateReferenceFilter(filter: Option<Arc<dyn PaintFilter>>) -> Self {
        let mut r = Self::new(FilterType::REFERENCE, 0.);
        r.image_filter_ = filter;
        r
    }
    pub fn CreateSaturatingBrightnessFilter(v: f32) -> Self {
        Self::new(FilterType::SATURATING_BRIGHTNESS, v)
    }
    pub fn CreateAlphaThresholdFilter(shape: &[Rect]) -> Self {
        let mut r = Self::new(FilterType::ALPHA_THRESHOLD, 0.);
        r.shape_ = shape.to_vec();
        r
    }
    pub fn CreateOffsetFilter(offset: Point) -> Self {
        let mut r = Self::new(FilterType::OFFSET, 0.);
        r.offset_ = offset;
        r
    }
}
// Source: cc/paint/filter_operation.cc:146-218.
fn no_op(t: FilterType) -> FilterOperation {
    use FilterType::*;
    match t {
        GRAYSCALE | SEPIA | HUE_ROTATE | INVERT | SATURATING_BRIGHTNESS => {
            FilterOperation::new(t, 0.)
        }
        SATURATE | BRIGHTNESS | CONTRAST | OPACITY => FilterOperation::new(t, 1.),
        BLUR => FilterOperation::CreateBlurFilter(0.),
        DROP_SHADOW => {
            FilterOperation::CreateDropShadowFilter(Point::default(), 0., FilterColor::default())
        }
        COLOR_MATRIX => {
            let mut m = [0.; 20];
            for i in [0, 6, 12, 18] {
                m[i] = 1.;
            }
            FilterOperation::CreateColorMatrixFilter(m)
        }
        ZOOM => FilterOperation::CreateZoomFilter(1., 0),
        REFERENCE => FilterOperation::CreateReferenceFilter(None),
        ALPHA_THRESHOLD => FilterOperation::CreateAlphaThresholdFilter(&[]),
        OFFSET => FilterOperation::CreateOffsetFilter(Point::default()),
    }
}
fn clamp_amount(v: f32, t: FilterType) -> f32 {
    use FilterType::*;
    match t {
        GRAYSCALE | SEPIA | INVERT | OPACITY => {
            if v < 0. {
                0.
            } else if 1. < v {
                1.
            } else {
                v
            }
        }
        SATURATE | BRIGHTNESS | CONTRAST | BLUR | DROP_SHADOW => {
            if v < 0. {
                0.
            } else {
                v
            }
        }
        ZOOM => {
            if v < 1. {
                1.
            } else {
                v
            }
        }
        HUE_ROTATE | SATURATING_BRIGHTNESS => v,
        _ => panic!("unreachable filter amount type"),
    }
}
// Source: ui/gfx/animation/tween.cc:272-279,314-317,213-246.
fn float_between(p: f64, a: f32, b: f32) -> f32 {
    (a as f64 + (b - a) as f64 * p) as f32
}
fn int_between(p: f64, a: i32, b: i32) -> i32 {
    (0.5 + (a as f64 + (b as f64 - a as f64) * p)).floor() as i32
}
fn color_between(p: f64, a: FilterColor, b: FilterColor) -> FilterColor {
    let mut alpha = float_between(p, a.alpha, b.alpha);
    if alpha <= 0. {
        return FilterColor::default();
    }
    if 1. < alpha {
        alpha = 1.;
    }
    let component = |x: f32, y: f32| float_between(p, x * a.alpha, y * b.alpha) / alpha;
    FilterColor::new(
        component(a.red, b.red),
        component(a.green, b.green),
        component(a.blue, b.blue),
        alpha,
    )
}
// Source: cc/paint/filter_operation.cc:220-275.
impl FilterOperation {
    pub fn Blend(from: Option<&Self>, to: Option<&Self>, progress: f64) -> Self {
        let mut result = Self::CreateEmptyFilter();
        if from.is_none() && to.is_none() {
            return result;
        }
        let from_noop;
        let to_noop;
        let a = if let Some(a) = from {
            a
        } else {
            from_noop = no_op(to.unwrap().type_);
            &from_noop
        };
        let b = if let Some(b) = to {
            b
        } else {
            to_noop = no_op(from.unwrap().type_);
            &to_noop
        };
        if a.type_ != b.type_ {
            return result;
        }
        debug_assert_ne!(b.type_, FilterType::COLOR_MATRIX);
        result.set_type(b.type_);
        if b.type_ == FilterType::REFERENCE {
            result.set_image_filter(if progress > 0.5 {
                b.image_filter_.clone()
            } else {
                a.image_filter_.clone()
            });
            return result;
        }
        if b.type_ == FilterType::ALPHA_THRESHOLD {
            result.set_shape(if progress > 0.5 { &b.shape_ } else { &a.shape_ });
            return result;
        }
        result.set_amount(clamp_amount(
            float_between(progress, a.amount(), b.amount()),
            b.type_,
        ));
        if b.type_ == FilterType::BLUR {
            result.set_blur_tile_mode(b.blur_tile_mode());
        } else if b.type_ == FilterType::DROP_SHADOW {
            result.set_offset(Point::new(
                int_between(progress, a.offset_.x(), b.offset_.x()),
                int_between(progress, a.offset_.y(), b.offset_.y()),
            ));
            result.set_drop_shadow_color(color_between(
                progress,
                a.drop_shadow_color_,
                b.drop_shadow_color_,
            ));
        } else if b.type_ == FilterType::ZOOM {
            result.set_zoom_inset(int_between(progress, a.zoom_inset_, b.zoom_inset_).max(0));
        }
        result
    }
}
// Source: cc/paint/filter_operation.cc:323-332.
fn map_std_deviation(sigma: f32, ctm: Option<&dyn FilterMapTransform>) -> (f32, f32) {
    let (x, y) = ctm.map_or((sigma, sigma), |m| m.MapVector(sigma, sigma));
    (x * 3., y * 3.)
}
// Source: cc/paint/filter_operation.cc:334-415.
impl FilterOperation {
    fn MapRectInternal(
        &self,
        rect: Rect,
        ctm: Option<&dyn FilterMapTransform>,
        direction: MapDirection,
    ) -> Rect {
        match self.type_ {
            FilterType::BLUR => {
                let spread = map_std_deviation(self.amount_, ctm);
                let mut result = RectF::from(rect);
                let sx = spread.0.abs();
                let sy = spread.1.abs();
                // Preserve source OutsetsF::VH(spread_x,spread_y), including
                // its vertical/horizontal order under anisotropic transforms.
                result.OutsetEdges(OutsetsF::new(sx, sy, sx, sy));
                ToEnclosingRect(result)
            }
            FilterType::DROP_SHADOW => {
                let spread = map_std_deviation(self.amount_, ctm);
                let mut result = RectF::from(rect);
                let sx = spread.0.abs();
                let sy = spread.1.abs();
                result.OutsetEdges(OutsetsF::new(sy, sx, sy, sx));
                let mut offset = ctm.map_or(
                    (self.offset_.x() as f32, self.offset_.y() as f32),
                    |m| m.MapVector(self.offset_.x() as f32, self.offset_.y() as f32),
                );
                if direction == MapDirection::Reverse {
                    offset.0 = -offset.0;
                    offset.1 = -offset.1;
                }
                result.Offset(offset.0, offset.1);
                result.Union(RectF::from(rect));
                ToEnclosingRect(result)
            }
            FilterType::REFERENCE => {
                if let Some(filter) = &self.image_filter_ {
                    filter.MapRect(rect, ctm, direction)
                } else {
                    rect
                }
            }
            FilterType::OFFSET => {
                let mut offset = ctm.map_or(
                    (self.offset_.x() as f32, self.offset_.y() as f32),
                    |m| m.MapVector(self.offset_.x() as f32, self.offset_.y() as f32),
                );
                if direction == MapDirection::Reverse {
                    offset.0 = -offset.0;
                    offset.1 = -offset.1;
                }
                let mut r = RectF::from(rect);
                r.Offset(offset.0, offset.1);
                ToEnclosingRect(r)
            }
            _ => rect,
        }
    }
    pub fn MapRect(&self, rect: Rect, ctm: Option<&dyn FilterMapTransform>) -> Rect {
        self.MapRectInternal(rect, ctm, MapDirection::Forward)
    }
    pub fn MapRectReverse(
        &self,
        rect: Rect,
        ctm: Option<&dyn FilterMapTransform>,
    ) -> Rect {
        self.MapRectInternal(rect, ctm, MapDirection::Reverse)
    }
}

// Source: cc/paint/filter_operation.cc:277-321.
impl FilterOperation {
    /// Owned diagnostic JSON is the TracedValue output, never filter state.
    pub fn AsValueInto(&self) -> serde_json::Value {
        use serde_json::json;
        use FilterType::*;
        let mut value = serde_json::Map::new();
        value.insert("type".into(), json!(self.type_ as i32));
        match self.type_ {
            GRAYSCALE
            | SEPIA
            | SATURATE
            | HUE_ROTATE
            | INVERT
            | BRIGHTNESS
            | CONTRAST
            | OPACITY
            | BLUR
            | SATURATING_BRIGHTNESS => {
                value.insert("amount".into(), json!(self.amount_ as f64));
            }
            DROP_SHADOW => {
                value.insert("std_deviation".into(), json!(self.amount_ as f64));
                value.insert("offset".into(), json!([self.offset_.x(), self.offset_.y()]));
                // Source: include/core/SkColor.h SkColor4f::toSkColor.
                let byte = |v: f32| ((v.clamp(0., 1.) * 255.).round()) as u32;
                let c = self.drop_shadow_color_;
                let color =
                    (byte(c.alpha) << 24)
                        | (byte(c.red) << 16)
                        | (byte(c.green) << 8)
                        | byte(c.blue);
                value.insert("color".into(), json!(color as i32));
            }
            COLOR_MATRIX => {
                value.insert("matrix".into(), json!(self.matrix_.map(|v| v as f64)));
            }
            ZOOM => {
                value.insert("amount".into(), json!(self.amount_ as f64));
                value.insert("inset".into(), json!(self.zoom_inset_));
            }
            REFERENCE => {
                value.insert("is_null".into(), json!(self.image_filter_.is_none()));
                if let Some(f) = &self.image_filter_ {
                    value.insert("filter_type".into(), json!(f.TypeToString()));
                }
            }
            ALPHA_THRESHOLD => {
                let mut shape = Vec::new();
                for r in &self.shape_ {
                    shape.extend([r.x(), r.y(), r.width(), r.height()]);
                }
                value.insert("shape".into(), json!(shape));
            }
            OFFSET => {
                value.insert("offset".into(), json!([self.offset_.x(), self.offset_.y()]));
            }
        }
        serde_json::Value::Object(value)
    }
}

// Source: cc/paint/filter_operation.h:56-57.
impl FilterType {
    pub const FILTER_TYPE_LAST: Self = Self::OFFSET;
}

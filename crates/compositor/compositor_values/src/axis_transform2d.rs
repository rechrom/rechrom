//! gfx::AxisTransform2d value: the raster transform cc tilings select and the
//! raster service replays with (cc/raster/raster_buffer.h Playback).
use compositor_tree::geometry::{AxisTransform2d, RectF, Vector2dF};

/// Source: ui/gfx/geometry/axis_transform2d.h:38-55,97-113,132-156.
/// Pure native gfx value subset consumed by Tile/coverage. Float clamping is
/// performed by the existing Foundation geometry owner, including NaN rules.
#[derive(Clone, Debug, PartialEq)]
pub struct RasterTransform {
    scale: Vector2dF,
    translation: Vector2dF,
}
impl Default for RasterTransform {
    fn default() -> Self {
        Self {
            scale: Vector2dF::new(1., 1.),
            translation: Default::default(),
        }
    }
}
impl AxisTransform2d for RasterTransform {
    fn FromScaleAndTranslation(scale: Vector2dF, translation: Vector2dF) -> Self {
        Self { scale, translation }
    }
    fn scale(&self) -> &Vector2dF {
        &self.scale
    }
    fn translation(&self) -> &Vector2dF {
        &self.translation
    }
    fn PreScale(&mut self, value: Vector2dF) {
        self.scale = Vector2dF::new(self.scale.x() * value.x(), self.scale.y() * value.y());
    }
    fn MapRect(&self, rect: RectF) -> RectF {
        debug_assert!(self.scale.x() >= 0. && self.scale.y() >= 0.);
        let clamp = |v: f32| foundation::gfx::Transform::clamp_float_geometry(f64::from(v));
        RectF::new(
            clamp(rect.x() * self.scale.x() + self.translation.x()),
            clamp(rect.y() * self.scale.y() + self.translation.y()),
            clamp(rect.width() * self.scale.x()),
            clamp(rect.height() * self.scale.y()),
        )
    }
    fn InverseMapRect(&self, rect: RectF) -> RectF {
        debug_assert!(self.scale.x() > 0. && self.scale.y() > 0.);
        let clamp = |v: f32| foundation::gfx::Transform::clamp_float_geometry(f64::from(v));
        RectF::new(
            clamp((rect.x() - self.translation.x()) * (1. / self.scale.x())),
            clamp((rect.y() - self.translation.y()) * (1. / self.scale.y())),
            clamp(rect.width() * (1. / self.scale.x())),
            clamp(rect.height() * (1. / self.scale.y())),
        )
    }
}

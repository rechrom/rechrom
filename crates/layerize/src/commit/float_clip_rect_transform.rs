//! Thin native gfx value operations for the source FloatClipRect owner.
//! Moved from compositor/src/runtime/paint/geometry_values.rs with the
//! Transform value it implements (coherence: the trait is Paint's).
use super::{
    geometry::{RectF, Vector2dF},
    transform::{RoundedRectTransformBackend, Transform},
};
use paint_model::geometry_mapper_source::traits::FloatClipRectTransform;
impl<R: RoundedRectTransformBackend> FloatClipRectTransform for Transform<R> {
    fn IsIdentityOr2dTranslation(&self) -> bool {
        // Same predicate as the compositor's ConversionTransform impl.
        self.as_source().IsIdentityOrTranslation() && self.as_source().rc(2, 3) == 0.
    }
    fn To2dTranslation(&self) -> Vector2dF {
        let value = self.as_source().To2dTranslation();
        Vector2dF::new(value.x(), value.y())
    }
    fn MapRect(&self, rect: RectF) -> RectF {
        let value = self.as_source().MapRect(foundation::gfx::RectF::new(
            foundation::gfx::PointF::new(rect.x(), rect.y()),
            foundation::gfx::SizeF::new(rect.width(), rect.height()),
        ));
        RectF::new(value.x(), value.y(), value.width(), value.height())
    }
}

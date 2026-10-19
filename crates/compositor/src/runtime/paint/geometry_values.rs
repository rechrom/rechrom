//! Thin native gfx value operations for the source FloatClipRect owner.
use crate::{
    adapter::{
        geometry::{RectF, Vector2dF},
        paint_geometry::FloatClipRectTransform,
    },
    runtime::geometry::{RoundedRectTransformBackend, Transform},
};
impl<R: RoundedRectTransformBackend> FloatClipRectTransform for Transform<R> {
    fn IsIdentityOr2dTranslation(&self) -> bool {
        crate::adapter::paint_chunks_to_cc_layer::ConversionTransform::IsIdentityOr2dTranslation(
            self,
        )
    }
    fn To2dTranslation(&self) -> Vector2dF {
        let value = self.native().To2dTranslation();
        Vector2dF::new(value.x(), value.y())
    }
    fn MapRect(&self, rect: RectF) -> RectF {
        let value = self.native().MapRect(foundation::gfx::RectF::new(
            foundation::gfx::PointF::new(rect.x(), rect.y()),
            foundation::gfx::SizeF::new(rect.width(), rect.height()),
        ));
        RectF::new(value.x(), value.y(), value.width(), value.height())
    }
}

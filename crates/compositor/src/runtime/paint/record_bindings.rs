//! Impl-side trait bindings of the cc/paint values in `layerize::commit::paint`
//! (HostImpl/TileManager image facets, the PictureLayerImpl worklet facet and
//! the PictureLayer NodeInfo facet). The values are shared by both halves; the
//! traits are compositor-local, so their impls live here.
use super::{CpuDrawImage, CpuPaintImage, CpuWorkletInput, TargetColorParams, WorkletPropertyKey, WorkletPropertyValue};
use crate::adapter::{
    geometry::{Rect, Vector2dF},
    layer_tree_host_impl::ContentColorUsage,
};

impl crate::adapter::layer_tree_host_impl::HostPaintImage for CpuPaintImage {
    fn stable_id(&self) -> crate::adapter::image_animation::PaintImageId {
        self.0.stable_id()
    }
    fn ShouldAnimate(&self) -> bool {
        self.0.ShouldAnimate()
    }
}
impl crate::adapter::layer_tree_host_impl::HostDecodePaintImage for CpuPaintImage {
    fn GetContentColorUsage(&self) -> ContentColorUsage {
        self.0.ContentColorUsage()
    }
}
impl crate::adapter::layer_tree_host_impl::HostDrawImage for CpuDrawImage {
    type PaintImage = CpuPaintImage;
    fn paint_image(&self) -> &CpuPaintImage {
        &self.image
    }
}
impl crate::adapter::tile_manager::TileManagerPaintImage for CpuPaintImage {
    type Id = crate::adapter::image_animation::PaintImageId;
    type WorkletInput = CpuWorkletInput;
    fn stable_id(&self) -> Self::Id {
        self.0.stable_id()
    }
    fn ShouldAnimate(&self) -> bool {
        self.0.ShouldAnimate()
    }
    fn IsLazyGenerated(&self) -> bool {
        self.0.IsLazyGenerated()
    }
    fn IsComplete(&self) -> bool {
        self.0.IsComplete()
    }
    fn IsMultipart(&self) -> bool {
        self.0.IsMultipart()
    }
    fn IsPaintWorklet(&self) -> bool {
        self.0.WorkletInput().is_some()
    }
    fn GetPaintWorkletInput(&self) -> &Self::WorkletInput {
        self.0
            .WorkletInput()
            .expect("source DCHECK(IsPaintWorklet())")
    }
}
impl
    crate::adapter::tile_manager::TileManagerTargetColorParams<
        crate::runtime::resources::RasterColorSpace,
    > for TargetColorParams
{
    fn color_space(&self) -> &crate::runtime::resources::RasterColorSpace {
        &self.color_space
    }
    fn GetHdrHeadroom(&self) -> f32 {
        Self::GetHdrHeadroom(self)
    }
}
impl<
        B: crate::adapter::tile_manager::TileManagerTypes<
            PaintImage = CpuPaintImage,
            TargetColorParams = TargetColorParams,
        >,
    > crate::adapter::tile_manager::TileManagerDrawImage<B> for CpuDrawImage
{
    fn paint_image(&self) -> &CpuPaintImage {
        &self.image
    }
    // Source: cc/tiles/checker_image_tracker.cc:80-92. Overflow vetoes
    // checkering as the largest possible decode; negative dimensions cannot
    // convert to size_t and take the same checked-numeric failure branch.
    fn EstimatedDecodedBytes(&self) -> usize {
        usize::try_from(self.source_rect.width)
            .ok()
            .and_then(|w| {
                usize::try_from(self.source_rect.height)
                    .ok()
                    .and_then(|h| 4usize.checked_mul(w)?.checked_mul(h))
            })
            .unwrap_or(usize::MAX)
    }
    fn WithRasterParameters(
        &self,
        scale: f32,
        frame_index: usize,
        target: &TargetColorParams,
    ) -> Self {
        // Source: cc/paint/draw_image.cc:74-87. The external image owner has
        // already decomposed its original matrix; this constructor scales that value.
        let mut target = self.LimitTargetDynamicRange(target.clone());
        // Source: cc/paint/draw_image.cc:105-113.
        if self.image.0.GetReinterpretAsSRGB() {
            target.color_space = Default::default();
        }
        Self {
            image: self.image.clone(),
            source_rect: self.source_rect,
            filter_quality: self.filter_quality,
            use_dark_mode: self.use_dark_mode,
            dynamic_range_limit: self.dynamic_range_limit,
            scale: Vector2dF::new(self.scale.x() * scale, self.scale.y() * scale),
            matrix_is_decomposable: self.matrix_is_decomposable,
            frame_index,
            target: Some(target),
        }
    }
}
impl
    crate::adapter::picture_layer_impl::PicturePaintWorkletInput<
        WorkletPropertyKey,
        WorkletPropertyValue,
    > for CpuWorkletInput
{
    fn ValueChangeShouldCauseRepaint(
        &self,
        previous: &WorkletPropertyValue,
        next: &WorkletPropertyValue,
    ) -> bool {
        self.0.ValueChangeShouldCauseRepaint(previous, next)
    }
}

impl crate::adapter::picture_layer::PictureLayerNodeInfo for crate::runtime::display_item_list::NodeInfo {
    fn visual_rect(&self) -> Rect {
        self.visual_rect
    }

    fn set_visual_rect(&mut self, rect: Rect) {
        self.visual_rect = rect;
    }
}
